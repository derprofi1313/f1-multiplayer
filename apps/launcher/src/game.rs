use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use swofty_core::app_control::{self, Block};
use swofty_core::busy::Busy;
use swofty_core::copies::{self, Found, GameCopy};
use swofty_core::crash::{LogScan, StartFailure, Symbols, fatal_name};
use swofty_core::ea_app;
use swofty_core::install::{Game, Source};
use swofty_core::launch;
use swofty_core::multiplayer::bridge::armed_env;
use swofty_core::process::{GameProcess, game_processes};
use swofty_core::quarantine;
use swofty_core::recovery::{self, Blocked, Notice, Scope, SessionLock};
use swofty_core::session::{LaunchMode, SessionRecord};
use swofty_core::stock;
use swofty_core::store::Store;
use swofty_core::vault;
use swofty_core::{GAME_EXE, steam, steam_client};

const KB: &str = include_str!("../../../re/db/functions.tsv");

#[derive(Debug, Clone, PartialEq)]
pub struct Install {
    pub dir: PathBuf,
    pub build_id: Option<u64>,
    pub exe_version: Option<String>,
    pub mode: String,
    pub via_steam: bool,
    pub manual: bool,
    pub source: Source,
    pub moved: Option<ea_app::Moved>,
}

impl Install {
    pub fn version(&self) -> Option<String> {
        self.exe_version
            .clone()
            .or_else(|| self.build_id.map(|b| b.to_string()))
    }
}

/// Whose a running F1 25 is: the active copy's, another listed copy's, or
/// none the launcher can tell.
#[derive(Debug, Clone, PartialEq)]
pub enum Owner {
    Active,
    Other(Install),
    Unknown,
}

pub fn owner(g: &GameProcess, active: Option<&Install>, copies: &[Install]) -> Owner {
    if active.is_some_and(|a| g.is_in(&a.dir)) {
        return Owner::Active;
    }
    copies
        .iter()
        .find(|c| g.is_in(&c.dir))
        .cloned()
        .map_or(Owner::Unknown, Owner::Other)
}

/// Every copy found, and the one Play uses.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Copies {
    pub all: Vec<Install>,
    pub active: Option<Install>,
    pub missing: Vec<PathBuf>,
}

pub struct Probe {
    pub install: Option<Copies>,
    pub processes: Vec<GameProcess>,
    pub session: Option<SessionRecord>,
    pub requests: Option<SystemTime>,
    pub merged: bool,
    pub recovered: Option<Notice>,
    pub launch_stage: Option<String>,
    pub mods_blocked: Option<quarantine::Report>,
    pub not_stock: Option<stock::Report>,
    pub blocked: Option<Blocked>,
    pub blocked_at: Option<f64>,
    pub launch_failed: Option<launch::Failure>,
    pub busy: Option<Busy>,
    pub(crate) banner_profile: String,
    pub(crate) banner: Option<Option<crate::career_saves::Banner>>,
    pub(crate) saves_marker: bool,
    pub(crate) saves_print: Option<u64>,
}

static VAULT_WATCH: std::sync::LazyLock<std::sync::Mutex<vault::watch::Watch>> =
    std::sync::LazyLock::new(Default::default);

/// The career vault's snapshot of an F1 25 the launcher didn't start, and
/// again once it closes.
fn watch_vault(store: &Store, processes: &[GameProcess]) {
    let pid = processes.first().map(|p| p.pid);
    let session = || SessionLock::held_elsewhere(&store.root) || Busy::live(&store.root).is_some();
    let lines = match VAULT_WATCH.lock() {
        Ok(mut w) => w.tick(&store.root, pid, session),
        Err(_) => Vec::new(),
    };
    lines.iter().for_each(|l| crate::app_log::note(l));
}

/// At launcher start, once an unfinished session is rolled back: move old
/// save backups into the career vault and snapshot the save, unless F1 25
/// runs (the probe's watch takes that one) or a session holds the lock.
pub fn vault_at_start(store: &Store) {
    if !game_processes().is_empty() {
        if swofty_core::launcher_saves::marker(&store.root).is_none() {
            crate::app_log::note(
                "saves: F1 25 is running; the import of the Steam save into the launcher saves waits for the next launcher start",
            );
        }
        return;
    }
    let Ok(Some(_lock)) = SessionLock::try_acquire(&store.root) else {
        return;
    };
    for l in vault::watch::at_launcher_start(&store.root) {
        crate::app_log::note(&l);
    }
    import_first_start(store);
}

static IMPORT_MISS_LOGGED: AtomicBool = AtomicBool::new(false);

fn import_first_start(store: &Store) {
    let profiles = store.profile_names().unwrap_or_default();
    let running = vault::restore::Running::now();
    let log_miss = !IMPORT_MISS_LOGGED.swap(true, Ordering::Relaxed);
    log_import(store, &profiles, running, log_miss);
}

pub(crate) fn import_now(store: &Store) {
    let profiles = store.profile_names().unwrap_or_default();
    log_import(store, &profiles, vault::restore::Running::now(), true);
}

fn log_import(store: &Store, profiles: &[String], running: vault::restore::Running, log_miss: bool) {
    for l in swofty_core::launcher_saves::first_start_import(
        &store.root,
        profiles,
        running,
        vault::now(),
        log_miss,
    ) {
        crate::app_log::note(&l);
    }
}

static IMPORT_RETRY: AtomicI64 = AtomicI64::new(0);
const IMPORT_RETRY_SECS: i64 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RetryGate {
    game_running: bool,
    pending: bool,
    launch_in_progress: bool,
    steam_usable: bool,
    ea_copy: bool,
    now: i64,
    last: i64,
}

fn should_retry_import(g: RetryGate) -> bool {
    !g.game_running
        && g.pending
        && !g.launch_in_progress
        && !g.ea_copy
        && g.steam_usable
        && g.now - g.last >= IMPORT_RETRY_SECS
}

fn launch_in_progress(store: &Store) -> bool {
    Busy::live(&store.root).is_some()
        || SessionLock::held_elsewhere(&store.root)
        || std::fs::read_to_string(store.root.join(steam_client::STAGE_FILE))
            .is_ok_and(|s| !s.trim().is_empty())
}

/// While the first-start import has found no Steam save, try again once Steam is signed in.
fn retry_import(store: &Store, processes: &[GameProcess]) {
    if !processes.is_empty()
        || !swofty_core::launcher_saves::import_pending(&store.root)
        || launch_in_progress(store)
    {
        return;
    }
    let now = vault::now();
    let last = IMPORT_RETRY.load(Ordering::Relaxed);
    if now - last < IMPORT_RETRY_SECS {
        return;
    }
    let gate = RetryGate {
        game_running: false,
        pending: true,
        launch_in_progress: false,
        steam_usable: swofty_core::steam_client::status().state.usable(),
        ea_copy: copies::find_active(&store.config()).is_some_and(|c| c.source == Source::EaApp),
        now,
        last,
    };
    if !should_retry_import(gate) {
        return;
    }
    IMPORT_RETRY.store(now, Ordering::Relaxed);
    let Ok(Some(_lock)) = SessionLock::try_acquire(&store.root) else {
        return;
    };
    import_first_start(store);
}

pub fn probe(
    store: &Store,
    rediscover: bool,
    seen: Option<SystemTime>,
    profile: &str,
    banner_due: bool,
    saves_open: bool,
) -> Probe {
    let processes = game_processes();
    watch_vault(store, &processes);
    retry_import(store, &processes);
    let (requests, merged) = merge_requests(store, processes.is_empty(), seen);
    let saves_print =
        saves_open.then(|| crate::career_saves::saves_print(&store.root, profile, !processes.is_empty()));
    Probe {
        install: rediscover.then(|| discover_all(store)),
        processes,
        session: store.latest_session().map(|d| SessionRecord::read(&d)),
        requests,
        merged,
        recovered: Notice::read(&store.root),
        mods_blocked: quarantine::Report::read(&store.root),
        not_stock: stock::Report::read(&store.root),
        blocked: Blocked::read(&store.root),
        blocked_at: blocked_at(&store.root),
        launch_failed: launch::Failure::read(&store.root),
        busy: build_state(store),
        banner_profile: profile.to_owned(),
        banner: banner_due.then(|| crate::career_saves::SavesPage::banner_for(&store.root, profile)),
        saves_marker: swofty_core::launcher_saves::marker(&store.root).is_some(),
        saves_print,
        launch_stage: std::fs::read_to_string(store.root.join(steam_client::STAGE_FILE))
            .ok()
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty()),
    }
}

/// The build a Play is running. One whose process ended is rolled back
/// here, as at launcher start.
fn build_state(store: &Store) -> Option<Busy> {
    Busy::read(&store.root)?;
    let live = Busy::live(&store.root);
    if live.is_none() {
        match recover_stale(store) {
            Ok(lines) => lines.iter().for_each(|l| eprintln!("swofty-launcher: {l}")),
            Err(e) => eprintln!("swofty-launcher: could not clean up an unfinished build: {e}"),
        }
        if let Ok(Some(_lock)) = SessionLock::try_acquire(&store.root) {
            Busy::clear(&store.root);
        }
    }
    live
}

pub fn blocked_at(data: &Path) -> Option<f64> {
    std::fs::metadata(Blocked::path(data))
        .and_then(|m| m.modified())
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs_f64())
}

/// At launcher start: put back what an unfinished `f1mp run` session left
/// in the game folder, unless a session is still live. A `f1mp apply`
/// install stays.
pub fn recover_stale(store: &Store) -> Result<Vec<String>, String> {
    if !game_processes().is_empty() {
        return Ok(Vec::new());
    }
    let Some(install) = discover(store) else {
        return Ok(Vec::new());
    };
    let Some(lock) = SessionLock::try_acquire(&store.root).map_err(|e| e.to_string())? else {
        return Ok(Vec::new());
    };
    let g = Game::locate(&store.root, install.dir).map_err(|e| e.to_string())?;
    let r = recovery::recover(&lock, &store.root, &g, Scope::Sessions).map_err(|e| e.to_string())?;
    Ok(r.lines())
}

/// The Restore game folder button: what `f1mp restore` does.
pub fn restore(store: &Store, dir: PathBuf) -> Result<String, String> {
    let g = Game::locate(&store.root, dir).map_err(|e| e.to_string())?;
    let done = recovery::restore(&store.root, &g).map_err(|e| e.to_string())?;
    if !done.left.is_empty() {
        let left: Vec<String> = done.left.iter().map(ToString::to_string).collect();
        return Err(format!("Not everything could be restored: {}", left.join("; ")));
    }
    let mut text = "The game folder matches the original game files.".to_owned();
    if !done.set_aside.is_empty() {
        text.push_str(&format!(
            " {} moved to {}.",
            match done.set_aside.len() {
                1 => "1 file was".to_owned(),
                n => format!("{n} files were"),
            },
            recovery::set_aside_dir(&store.root).display()
        ));
    }
    Ok(text)
}

fn merge_requests(store: &Store, idle: bool, seen: Option<SystemTime>) -> (Option<SystemTime>, bool) {
    let profile = store.active_profile();
    let Some(written) = std::fs::metadata(store.requests_path(&profile))
        .and_then(|m| m.modified())
        .ok()
    else {
        return (None, false);
    };
    let settled = idle && written.elapsed().is_ok_and(|age| age > Duration::from_secs(10));
    if seen == Some(written) && !settled {
        return (seen, false);
    }
    let merged = store.merge_requests(&profile, settled).ok().flatten();
    (Some(written), merged.is_some_and(|m| !m.changed.is_empty()))
}

pub fn discover(store: &Store) -> Option<Install> {
    discover_all(store).active
}

pub fn discover_all(store: &Store) -> Copies {
    let config = store.config();
    let found = copies::discover(&config);
    let active = copies::active(&found, &config);
    let recorded = if active.is_some_and(|c| c.source == Source::EaApp) {
        ea_app::recorded_here()
    } else {
        Vec::new()
    };
    let active = active.map(|c| c.dir.clone());
    let all: Vec<Install> = found.iter().map(|c| install(c, &recorded)).collect();
    Copies {
        active: active.and_then(|dir| all.iter().find(|i| i.dir == dir).cloned()),
        missing: copies::missing(&found, &config),
        all,
    }
}

fn install(copy: &GameCopy, recorded: &[PathBuf]) -> Install {
    let steam_root = copy.steam.as_ref().map(|g| g.steam_root.clone());
    let via_steam = steam_root.as_deref().is_some_and(|root| {
        steam::launch_options(root)
            .iter()
            .any(|(_, o)| steam::wrapper_installed(o))
    });
    let mode = if cfg!(windows) {
        "Windows".to_owned()
    } else {
        match steam_root.as_deref().and_then(steam::compat_tool) {
            Some(tool) => format!("Proton · {tool}"),
            None => "Proton".to_owned(),
        }
    };
    Install {
        dir: copy.dir.clone(),
        build_id: copy.build_id(),
        exe_version: copy.version(),
        mode,
        via_steam,
        manual: copy.found == Found::Added,
        source: copy.source,
        moved: (copy.source == Source::EaApp)
            .then(|| ea_app::moved(&copy.dir, recorded, ea_app::is_install))
            .flatten(),
    }
}

pub const DIRECT_START_FILE: &str = "swofty/direct_start.json";

pub struct Launch {
    child: Child,
    direct_start: Option<PathBuf>,
    pub since: Instant,
    pub at: f64,
    pub log: PathBuf,
    pub saw_game: bool,
    pub stage: Option<String>,
}

impl Launch {
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        store: &Store,
        profile: &str,
        install: &Install,
        mode: LaunchMode,
        netplay: Option<&str>,
        row_ids: Option<&Path>,
        direct_start: Option<&str>,
        steam_saves_once: bool,
    ) -> Result<Self, String> {
        let cli = cli_path(store).ok_or_else(|| app_control::missing(&expected_cli(store)))?;
        let at = now_secs();
        let log = store.root.join("state/launcher-run.log");
        let out = std::fs::create_dir_all(store.root.join("state"))
            .and_then(|_| File::create(&log))
            .map_err(|e| format!("{}: {e}", log.display()))?;
        let err = out.try_clone().map_err(|e| e.to_string())?;
        std::fs::remove_file(store.root.join(steam_client::STAGE_FILE)).ok();
        let mut c = Command::new(&cli);
        c.arg("--data-dir")
            .arg(&store.root)
            .arg("--profile")
            .arg(profile)
            .arg("run");
        // Vanilla starts F1 25 as it shipped, with no build and no Swofty file; every other
        // profile builds and installs its overlay first.
        if profile == swofty_core::profile::VANILLA {
            c.arg("--vanilla");
        } else {
            c.arg("--apply");
        }
        if mode.is_vr() {
            c.arg("--vr");
        }
        if steam_saves_once {
            c.arg("--steam-saves-once");
        }
        c.arg("--game").arg(&install.dir);
        if let Some(armed) = netplay {
            c.envs(armed_env(armed));
        }
        if let Some(file) = row_ids {
            c.env(swofty_core::row_ids::ENV, file);
        }
        let request_file = install.dir.join(DIRECT_START_FILE);
        std::fs::remove_file(&request_file).ok();
        let direct_start = match direct_start {
            Some(request) => {
                std::fs::create_dir_all(install.dir.join("swofty"))
                    .and_then(|()| std::fs::write(&request_file, request))
                    .map_err(|e| format!("{}: {e}", request_file.display()))?;
                Some(request_file)
            }
            None => None,
        };
        #[cfg(windows)]
        std::os::windows::process::CommandExt::creation_flags(&mut c, 0x0800_0000);
        let child = c
            .stdin(Stdio::null())
            .stdout(out)
            .stderr(err)
            .spawn()
            .map_err(|e| not_started(store, &log, &cli, &e))?;
        Ok(Self {
            child,
            direct_start,
            since: Instant::now(),
            at,
            log,
            saw_game: false,
            stage: None,
        })
    }

    pub fn finished(&mut self) -> Option<bool> {
        self.child.try_wait().ok().flatten().map(|s| s.success())
    }

    pub fn failure(&self) -> String {
        failure_line(&std::fs::read_to_string(&self.log).unwrap_or_default())
            .unwrap_or_else(|| "the launch exited before the game started".into())
    }
}

impl Drop for Launch {
    fn drop(&mut self) {
        if let Some(file) = &self.direct_start {
            std::fs::remove_file(file).ok();
        }
    }
}

/// A CLI that would not start: the error goes in the launch log. One Windows
/// refused is also recorded for Send Logs, and says what blocked it.
fn not_started(store: &Store, log: &Path, cli: &Path, e: &std::io::Error) -> String {
    let block = Block::of(cli, e);
    let mut lines = format!("swofty-launcher: could not start {}: {e}\n", cli.display());
    if let Some(b) = &block {
        lines.push_str(&format!("swofty-launcher: {}\n", b.headline()));
        launch::Failure::new(launch::Cause::Blocked(b.clone()))
            .write(&store.root)
            .ok();
    }
    std::fs::write(log, lines).ok();
    block.map_or_else(|| e.to_string(), |b| b.text())
}

/// The wrapper's last error, from its `swofty: ` line, or else the last line it
/// wrote. An error over several lines gives its first, which says what failed.
fn failure_line(log: &str) -> Option<String> {
    let lines: Vec<&str> = log.lines().filter(|l| !l.trim().is_empty()).collect();
    lines
        .iter()
        .rev()
        .find_map(|l| l.strip_prefix("swofty: "))
        .or_else(|| lines.last().copied())
        .map(|l| l.trim_end_matches(':').to_owned())
}

/// A failure as the play bar leads with it: a wrapper's error with its
/// innermost cause first and what the wrapper was doing after it, and any
/// other as it is.
pub fn cause_first(failure: &str) -> String {
    let Some(reason) = failure
        .strip_prefix("the wrapper exited (")
        .and_then(|rest| rest.split_once(") "))
        .and_then(|(_, rest)| rest.split_once(": "))
        .map(|(_, reason)| reason)
    else {
        return failure.to_owned();
    };
    match reason.rsplit_once(": ") {
        Some((doing, cause)) if !cause.trim().is_empty() => format!("{cause} ({doing})"),
        _ => reason.to_owned(),
    }
}

const CLI: &str = if cfg!(windows) { "f1mp.exe" } else { "f1mp" };

fn beside_launcher() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|e| Some(e.parent()?.join(CLI)))
}

pub(crate) fn cli_path(store: &Store) -> Option<PathBuf> {
    let path = std::env::var_os("PATH").unwrap_or_default();
    beside_launcher()
        .into_iter()
        .chain([store.bin_dir().join(CLI)])
        .chain(std::env::split_paths(&path).map(|d| d.join(CLI)))
        .find(|p| p.is_file())
}

fn expected_cli(store: &Store) -> PathBuf {
    beside_launcher().unwrap_or_else(|| store.bin_dir().join(CLI))
}

pub fn crash_summary(session: &SessionRecord, game: Option<&Path>) -> Result<String, StartFailure> {
    let mut symbols = Symbols::parse_kb(KB);
    if let Some(dir) = game {
        symbols.load_pdata(&dir.join(GAME_EXE)).ok();
    }
    let mut scan = LogScan::default();
    if let Ok(f) = File::open(session.wine_log()) {
        for line in BufReader::new(f).split(b'\n').map_while(Result::ok) {
            scan.feed(&String::from_utf8_lossy(&line));
        }
    }
    if let Some(failure) = scan.start_failure {
        return Err(failure);
    }
    let code = session.exit_code.unwrap_or_default();
    let fast_fail = swofty_core::fast_fail::session_lines(&session.dir);
    if !fast_fail.is_empty() {
        return Ok(fast_fail.join("\n"));
    }
    Ok(match scan.last_fatal {
        Some((c, addr)) => format!(
            "exit code {code}, last {} at {}",
            fatal_name(c).unwrap_or("exception"),
            scan.describe(addr, &symbols)
        ),
        None => format!("exit code {code}"),
    })
}

pub fn now_secs() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

pub fn ago(secs: f64) -> String {
    let s = secs.max(0.0) as u64;
    match s {
        0..60 => "just now".into(),
        60..3600 => format!("{} min ago", s / 60),
        3600..86_400 => format!("{} h ago", s / 3600),
        _ => format!("{} d ago", s / 86_400),
    }
}

pub fn duration(secs: u64) -> String {
    match secs {
        0..60 => format!("{secs} s"),
        60..3600 => format!("{} min", secs / 60),
        _ => format!("{} h {} min", secs / 3600, secs % 3600 / 60),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use swofty_core::app_control::Block;
    use swofty_core::launch;
    use swofty_core::store::Store;

    use super::{IMPORT_RETRY_SECS, RetryGate, cause_first, failure_line, not_started, should_retry_import};

    fn idle_steam() -> RetryGate {
        RetryGate {
            game_running: false,
            pending: true,
            launch_in_progress: false,
            steam_usable: true,
            ea_copy: false,
            now: 1000,
            last: 0,
        }
    }

    #[test]
    fn a_steam_copy_with_nothing_running_retries_the_import() {
        assert!(should_retry_import(idle_steam()));
    }

    #[test]
    fn an_ea_copy_never_retries_the_import() {
        assert!(!should_retry_import(RetryGate {
            ea_copy: true,
            ..idle_steam()
        }));
    }

    #[test]
    fn a_launch_in_progress_never_retries_the_import() {
        assert!(!should_retry_import(RetryGate {
            launch_in_progress: true,
            ..idle_steam()
        }));
        assert!(!should_retry_import(RetryGate {
            game_running: true,
            ..idle_steam()
        }));
    }

    #[test]
    fn the_import_retry_needs_a_usable_steam_and_a_pending_import() {
        assert!(!should_retry_import(RetryGate {
            steam_usable: false,
            ..idle_steam()
        }));
        assert!(!should_retry_import(RetryGate {
            pending: false,
            ..idle_steam()
        }));
    }

    #[test]
    fn the_import_retries_are_spaced() {
        let g = idle_steam();
        assert!(!should_retry_import(RetryGate {
            last: g.now - IMPORT_RETRY_SECS + 1,
            ..g
        }));
        assert!(should_retry_import(RetryGate {
            last: g.now - IMPORT_RETRY_SECS,
            ..g
        }));
    }

    #[test]
    fn a_cli_that_does_not_start_says_so_in_the_launch_log() {
        let root = std::env::temp_dir().join(format!("swofty-launcher-not-started-{}", std::process::id()));
        std::fs::create_dir_all(root.join("state")).unwrap();
        let store = Store::new(&root);
        let log = root.join("state/launcher-run.log");
        let cli = Path::new(r"C:\F1 Multiplayer\bin\swofty.exe");
        let missing = std::io::Error::from_raw_os_error(2);
        assert_eq!(not_started(&store, &log, cli, &missing), missing.to_string());
        let written = std::fs::read_to_string(&log).unwrap();
        assert_eq!(
            written,
            format!("swofty-launcher: could not start {}: {missing}\n", cli.display())
        );
        assert_eq!(launch::Failure::read(&root), None);
        let blocked = std::io::Error::from_raw_os_error(4551);
        let text = not_started(&store, &log, cli, &blocked);
        let written = std::fs::read_to_string(&log).unwrap();
        match Block::of(cli, &blocked) {
            Some(b) => {
                assert_eq!(text, b.text());
                assert!(
                    written.ends_with(&format!("swofty-launcher: {}\n", b.headline())),
                    "{written}"
                );
                assert_eq!(
                    launch::Failure::read(&root).map(|f| f.cause),
                    Some(launch::Cause::Blocked(b))
                );
            }
            None => assert_eq!(text, blocked.to_string()),
        }
        assert!(written.contains("could not start"), "{written}");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_failed_launch_shows_the_first_line_of_the_wrappers_error() {
        let log = "[info] installing profile default\nswofty: 2 UI edit(s) break rules the game's UI loader enforces:\n  a.erp: b.ui: page.kid: on pressed -> missing page.gone\n  a.erp: b.ui: page.kid: id\n";
        assert_eq!(
            failure_line(log).as_deref(),
            Some("2 UI edit(s) break rules the game's UI loader enforces")
        );
        assert_eq!(failure_line("[info] starting\nexit\n").as_deref(), Some("exit"));
        assert_eq!(failure_line("\n"), None);
    }

    #[test]
    fn a_wrapper_failure_leads_with_its_innermost_cause() {
        let blocked = "the download was blocked on this PC or network, typically by antivirus HTTPS or web \
                       scanning, a VPN or proxy, a network filter or a Wi-Fi login page; turning that off or trying \
                       another network fixes it";
        let log = format!(
            "[info] building profile default\n\
             [wrapper] downloading offline play 0.3.11 from https://x\n\
             swofty: the wrapper exited (exit code: 1) before the game started: downloading the offline play \
             extension: offline play 0.3.11: {blocked}\n\
             (could not download https://x/v0.3.11/f1op_hook.dll: io: received corrupt message of type \
             InvalidContentType)\n"
        );
        let failure = failure_line(&log).unwrap();
        assert!(
            failure.starts_with("the wrapper exited (exit code: 1)"),
            "{failure}"
        );
        assert_eq!(
            cause_first(&failure),
            format!("{blocked} (downloading the offline play extension: offline play 0.3.11)")
        );
        assert_eq!(
            cause_first(
                "the wrapper exited (exit status: 3) before the game started: version.dll already exists in the \
                 game folder; run f1mp restore"
            ),
            "version.dll already exists in the game folder; run f1mp restore"
        );
        assert_eq!(
            cause_first(
                "the wrapper exited (signal: 9 (SIGKILL)) while the game was running: installing profile default: \
                 /games/F1 25/swofty/x.erp: Permission denied (os error 13)"
            ),
            "Permission denied (os error 13) (installing profile default: /games/F1 25/swofty/x.erp)"
        );
        for kept in [
            r"the wrapper exited (exit code: 1) before the game started without output; see C:\F1 Multiplayer\state\wrapper-run.log",
            "the wrapper did not start within 10 min; check the Steam launch option (f1mp doctor)",
            "building profile default: reading x: missing",
            "an antivirus removed mod files, so F1 25 was not started",
        ] {
            assert_eq!(cause_first(kept), kept);
        }
    }

    #[test]
    fn an_addon_that_needs_a_newer_game_shows_the_update() {
        let outdated = "Classic Seasons needs F1 25 1.0.141.7865, the latest update. Your F1 25 is 1.0.136.6820. \
                        Update F1 25 in Steam or the EA app, then press Play again.";
        let log = format!(
            "[info] launching F1 25 directly; waiting for the wrapper, output in C:\\F1 Multiplayer\\state/wrapper-run.log\n\
             [wrapper] [info] installing profile default\n\
             swofty: the wrapper exited (exit code: 1) before the game started: {outdated}\n\
             (applying swofty.classic-seasons: eaid://ui_package/databases/overrides.uidb: \
             overrides.Contents.teams.Contents.mp_team_select_logos.Contents: no entry [Name=vanamersfoort_26])\n"
        );
        let failure = failure_line(&log).unwrap();
        assert_eq!(
            failure,
            format!("the wrapper exited (exit code: 1) before the game started: {outdated}")
        );
        assert_eq!(cause_first(&failure), outdated);
    }
}
