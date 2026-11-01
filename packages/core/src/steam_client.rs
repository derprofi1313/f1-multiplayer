//! Whether the Steam client is running and signed in, and starting it when
//! it is not.
//!
//! Windows publishes the client in `HKCU\Software\Valve\Steam\ActiveProcess`:
//! `pid` is the running steam.exe (0 once it exits) and `ActiveUser` the
//! signed-in account (0 at the sign-in screen). Some signed-in clients keep
//! `ActiveUser` at 0, so a sign-in that `config/loginusers.vdf` dates after
//! the client started counts once `ActiveUser` has had time to follow it.
//! On Linux the client writes its pid to `~/.steam/steam.pid` and
//! `registry.vdf`; current clients keep `ActiveProcess` out of
//! `registry.vdf`, so a sign-in shows as `config/loginusers.vdf` being
//! written after the client started. Flatpak Steam keeps the same files
//! under `~/.var/app/com.valvesoftware.Steam`, with pids from its own
//! namespace.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::compat::{parse_reg_query, reg_dword};
use crate::vdf::{self, Vdf};

pub const FLATPAK_APP: &str = "com.valvesoftware.Steam";
pub const ACTIVE_PROCESS_KEY: &str = r"HKCU\Software\Valve\Steam\ActiveProcess";
pub const STEAM_KEY: &str = r"HKCU\Software\Valve\Steam";
pub const READY_TIMEOUT: Duration = Duration::from_secs(90);
pub const POLL_INTERVAL: Duration = Duration::from_millis(500);
/// What the launch is waiting on, for the launcher's status line.
pub const STAGE_FILE: &str = "state/launch-stage.txt";

const LOGIN_SLACK: Duration = Duration::from_secs(5);
/// How long `ActiveUser` gets to follow a sign-in `loginusers.vdf` records.
const ACTIVE_USER_LAG: Duration = Duration::from_secs(20);
const EXIT_GRACE: Duration = Duration::from_secs(3);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Stopped,
    /// A Steam process runs, but the client has not registered yet.
    Starting,
    /// The client runs and reports nobody signed in.
    SigningIn,
    /// The client runs, and nothing says whether it has signed in.
    Unconfirmed,
    Ready,
}

impl State {
    pub fn usable(self) -> bool {
        matches!(self, Self::Ready | Self::Unconfirmed)
    }
}

/// What the sign-in state was read from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Unknown,
    /// `ActiveProcess\ActiveUser`: the signed-in account, 0 for nobody.
    ActiveUser(u32),
    /// `loginusers.vdf` was written after the client started.
    LoginFile,
    /// `loginusers.vdf` predates the client.
    NoLogin,
    /// `ActiveUser` stays 0, but `loginusers.vdf` records a sign-in since the
    /// client started.
    LoginOverActiveUser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Status {
    pub state: State,
    pub pid: Option<u32>,
    pub signal: Signal,
}

impl Status {
    pub const STOPPED: Self = Self {
        state: State::Stopped,
        pid: None,
        signal: Signal::Unknown,
    };

    fn new(state: State, pid: u32, signal: Signal) -> Self {
        Self {
            state,
            pid: Some(pid),
            signal,
        }
    }

    pub fn headline(&self) -> &'static str {
        match self.state {
            State::Stopped => "not running",
            State::Starting => "starting",
            State::SigningIn | State::Unconfirmed => "waiting for sign-in",
            State::Ready => "ready",
        }
    }

    /// The pid and the sign-in evidence, e.g. `pid 4242, ActiveUser 0`.
    pub fn detail(&self) -> String {
        let signal = match (self.state, self.signal) {
            (State::Stopped, _) => None,
            (State::Starting, _) => Some("the client hasn't registered yet".to_owned()),
            (_, Signal::ActiveUser(user)) => Some(format!("ActiveUser {user}")),
            (_, Signal::LoginFile) => Some("loginusers.vdf written since it started".to_owned()),
            (_, Signal::NoLogin) => Some("no sign-in in loginusers.vdf since it started".to_owned()),
            (_, Signal::LoginOverActiveUser) => {
                Some("ActiveUser 0, but loginusers.vdf records a sign-in since it started".to_owned())
            }
            (_, Signal::Unknown) => Some("no sign-in signal".to_owned()),
        };
        self.pid
            .map(|p| format!("pid {p}"))
            .into_iter()
            .chain(signal)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.detail() {
            d if d.is_empty() => f.write_str(self.headline()),
            d => write!(f, "{} ({d})", self.headline()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Proc {
    pub pid: u32,
    /// The pid inside the process's own namespace (Flatpak's sandbox).
    pub ns_pid: u32,
    pub name: String,
    /// The first command-line argument.
    pub exe: String,
    pub started: Option<SystemTime>,
}

pub trait Registry {
    /// `reg query <key>` output, or None when it cannot run.
    fn query(&self, key: &str) -> Option<String>;
}

pub struct RegExe;

impl Registry for RegExe {
    fn query(&self, key: &str) -> Option<String> {
        crate::compat::windows_tool("reg", &["query", key])
    }
}

/// Steam on Windows. `started` dates a steam.exe, and `roots` are Steam
/// folders to look for `loginusers.vdf` in besides the registered one.
pub fn windows_status(
    reg: &dyn Registry,
    steam_pids: &[u32],
    started: &dyn Fn(u32) -> Option<SystemTime>,
    roots: &[PathBuf],
) -> Status {
    let active = reg.query(ACTIVE_PROCESS_KEY).unwrap_or_default();
    match reg_dword(&active, "pid").filter(|p| *p != 0) {
        Some(pid) if steam_pids.contains(&pid) => match reg_dword(&active, "ActiveUser") {
            Some(0) if signed_in_since(windows_login(reg, roots), started(pid), SystemTime::now()) => {
                Status::new(State::Ready, pid, Signal::LoginOverActiveUser)
            }
            Some(0) => Status::new(State::SigningIn, pid, Signal::ActiveUser(0)),
            Some(user) => Status::new(State::Ready, pid, Signal::ActiveUser(user)),
            None => Status::new(State::Unconfirmed, pid, Signal::Unknown),
        },
        _ => match steam_pids.first() {
            Some(&pid) => Status::new(State::Starting, pid, Signal::Unknown),
            None => Status::STOPPED,
        },
    }
}

/// Whether `login` came after the client `started` and long enough ago for
/// `ActiveUser` to have followed it.
fn signed_in_since(login: Option<SystemTime>, started: Option<SystemTime>, now: SystemTime) -> bool {
    let (Some(login), Some(started)) = (login, started) else {
        return false;
    };
    login + LOGIN_SLACK >= started && now.duration_since(login).is_ok_and(|d| d >= ACTIVE_USER_LAG)
}

/// The newest sign-in `loginusers.vdf` records under the registered Steam
/// folder or `roots`.
fn windows_login(reg: &dyn Registry, roots: &[PathBuf]) -> Option<SystemTime> {
    let registered = steam_key_values(reg);
    registered
        .path("SteamPath")
        .into_iter()
        .chain(
            registered
                .path("SteamExe")
                .and_then(|e| Some(e.parent()?.to_path_buf())),
        )
        .chain(roots.iter().cloned())
        .filter_map(|r| login_stamp(&r))
        .max()
}

struct RegValues(Vec<(String, String)>);

impl RegValues {
    fn path(&self, name: &str) -> Option<PathBuf> {
        self.0
            .iter()
            .find(|(n, d)| n.eq_ignore_ascii_case(name) && !d.trim().is_empty())
            .map(|(_, d)| PathBuf::from(d.trim()))
    }
}

fn steam_key_values(reg: &dyn Registry) -> RegValues {
    RegValues(
        reg.query(STEAM_KEY)
            .map(|l| parse_reg_query(&l))
            .unwrap_or_default(),
    )
}

struct Install {
    dot: PathBuf,
    roots: Vec<PathBuf>,
    sandboxed: bool,
}

fn installs(home: &Path) -> [Install; 2] {
    let flatpak = home.join(".var/app").join(FLATPAK_APP);
    [
        Install {
            dot: home.join(".steam"),
            roots: vec![
                home.join(".steam/steam"),
                home.join(".steam/root"),
                home.join(".local/share/Steam"),
            ],
            sandboxed: false,
        },
        Install {
            dot: flatpak.join(".steam"),
            roots: vec![flatpak.join(".local/share/Steam"), flatpak.join("data/Steam")],
            sandboxed: true,
        },
    ]
}

fn read_vdf(path: &Path) -> Option<Vdf> {
    vdf::parse(&std::fs::read_to_string(path).ok()?)
}

fn read_pid(path: &Path) -> Option<u32> {
    std::fs::read_to_string(path).ok()?.trim().parse().ok()
}

/// The newest `Timestamp` in the `loginusers.vdf` under `root`.
fn login_stamp(root: &Path) -> Option<SystemTime> {
    read_vdf(&root.join("config/loginusers.vdf"))
        .and_then(|v| {
            v.get("users")?
                .entries()
                .filter_map(|(_, u)| u.get("Timestamp")?.str()?.trim().parse::<u64>().ok())
                .max()
        })
        .map(|s| UNIX_EPOCH + Duration::from_secs(s))
}

/// When an account last signed in to the client under `root`.
fn last_login(root: &Path) -> Option<SystemTime> {
    let written = std::fs::metadata(root.join("config/loginusers.vdf"))
        .and_then(|m| m.modified())
        .ok();
    written.max(login_stamp(root))
}

fn login_state(
    active_user: Option<u32>,
    login: Option<SystemTime>,
    started: Option<SystemTime>,
) -> (State, Signal) {
    match (active_user, login, started) {
        (Some(0), ..) => (State::SigningIn, Signal::ActiveUser(0)),
        (Some(user), ..) => (State::Ready, Signal::ActiveUser(user)),
        (None, _, None) => (State::Ready, Signal::Unknown),
        (None, Some(login), Some(started)) if login + LOGIN_SLACK >= started => {
            (State::Ready, Signal::LoginFile)
        }
        _ => (State::Unconfirmed, Signal::NoLogin),
    }
}

fn is_client_binary(exe: &str) -> bool {
    exe.replace('\\', "/").ends_with("ubuntu12_32/steam")
}

pub fn linux_status(home: &Path, procs: &[Proc]) -> Status {
    let installs = installs(home);
    for i in &installs {
        let reg = read_vdf(&i.dot.join("registry.vdf"));
        let value = |keys: &[&str]| reg.as_ref()?.path(keys)?.str()?.trim().parse::<u32>().ok();
        let active = |name| {
            value(&[
                "Registry",
                "HKCU",
                "Software",
                "Valve",
                "Steam",
                "ActiveProcess",
                name,
            ])
        };
        let (active_pid, active_user) = (active("pid"), active("ActiveUser"));
        let pids: Vec<u32> = [
            read_pid(&i.dot.join("steam.pid")),
            read_pid(&i.dot.join("steam/steam.pid")),
            value(&["Registry", "HKLM", "Software", "Valve", "Steam", "SteamPID"]),
            active_pid,
        ]
        .into_iter()
        .flatten()
        .filter(|p| *p != 0)
        .collect();
        let is = |p: &Proc, pid: u32| p.pid == pid || (i.sandboxed && p.ns_pid == pid);
        let Some(client) = procs
            .iter()
            .find(|p| p.name == "steam" && pids.iter().any(|&pid| is(p, pid)))
        else {
            continue;
        };
        let user = active_user.filter(|_| active_pid.is_some_and(|pid| is(client, pid)));
        let login = i.roots.iter().filter_map(|r| last_login(r)).max();
        let (state, signal) = login_state(user, login, client.started);
        return Status::new(state, client.pid, signal);
    }
    if let Some(client) = procs
        .iter()
        .find(|p| p.name == "steam" && is_client_binary(&p.exe))
    {
        let login = installs
            .iter()
            .flat_map(|i| &i.roots)
            .filter_map(|r| last_login(r))
            .max();
        let (state, signal) = login_state(None, login, client.started);
        return Status::new(state, client.pid, signal);
    }
    match procs.iter().find(|p| p.name == "steam") {
        Some(p) => Status::new(State::Starting, p.pid, Signal::Unknown),
        None => Status::STOPPED,
    }
}

/// This user's processes named `steam`.
#[cfg(target_os = "linux")]
pub fn processes() -> Vec<Proc> {
    use std::os::unix::fs::MetadataExt;
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    let me = std::fs::metadata("/proc/self").map(|m| m.uid()).ok();
    entries
        .flatten()
        .filter_map(|e| {
            let pid: u32 = e.file_name().to_str()?.parse().ok()?;
            let dir = e.path();
            if e.metadata().map(|m| m.uid()).ok() != me {
                return None;
            }
            let comm = std::fs::read_to_string(dir.join("comm")).ok()?;
            let name = comm.trim_end();
            if name != "steam" {
                return None;
            }
            let stat = std::fs::read_to_string(dir.join("stat")).ok()?;
            let state = stat.rsplit_once(')')?.1.split_whitespace().next()?;
            if matches!(state, "Z" | "X") {
                return None;
            }
            let cmdline = std::fs::read(dir.join("cmdline")).unwrap_or_default();
            let exe = cmdline.split(|b| *b == 0).next().unwrap_or_default();
            let ns_pid = std::fs::read_to_string(dir.join("status"))
                .ok()
                .and_then(|s| {
                    s.lines()
                        .find_map(|l| l.strip_prefix("NSpid:"))?
                        .split_whitespace()
                        .last()?
                        .parse()
                        .ok()
                })
                .unwrap_or(pid);
            Some(Proc {
                pid,
                ns_pid,
                name: name.to_owned(),
                exe: String::from_utf8_lossy(exe).into_owned(),
                started: crate::process::started(pid),
            })
        })
        .collect()
}

#[cfg(not(target_os = "linux"))]
pub fn processes() -> Vec<Proc> {
    Vec::new()
}

#[cfg(windows)]
pub fn status() -> Status {
    let pids: Vec<u32> = crate::process::named("steam.exe").iter().map(|p| p.pid).collect();
    windows_status(
        &RegExe,
        &pids,
        &crate::process::started,
        &crate::steam::steam_roots(),
    )
}

#[cfg(not(windows))]
pub fn status() -> Status {
    match std::env::var_os("HOME") {
        Some(home) => linux_status(Path::new(&home), &processes()),
        None => Status::STOPPED,
    }
}

/// True inside a session Steam itself runs (Steam Deck Game Mode, Big
/// Picture's gamescope session), where the client is always up.
pub fn steam_session() -> bool {
    std::env::var_os("SteamGamepadUI").is_some_and(|v| !v.is_empty() && v != "0")
        || std::env::var("XDG_CURRENT_DESKTOP").is_ok_and(|d| d.eq_ignore_ascii_case("gamescope"))
}

#[derive(Debug, Clone, PartialEq)]
pub enum Starter {
    Exe(PathBuf),
    Flatpak(PathBuf),
    /// Windows without a known steam.exe: the `steam://` handler.
    Url,
}

impl Starter {
    /// The client with `args` (`-applaunch`, `-silent`); None for [`Starter::Url`].
    pub fn command<S: AsRef<OsStr>>(&self, args: &[S]) -> Option<Command> {
        let mut c = match self {
            Self::Exe(exe) => Command::new(exe),
            Self::Flatpak(flatpak) => {
                let mut c = Command::new(flatpak);
                c.args(["run", FLATPAK_APP]);
                c
            }
            Self::Url => return None,
        };
        c.args(args);
        Some(c)
    }

    /// Start the client in the background, detached from this process. On
    /// Windows, a registered steam.exe that will not start falls back to the
    /// `steam://` handler.
    pub fn start(&self) -> std::io::Result<Child> {
        match self.spawn_silent() {
            Err(e) if cfg!(windows) && matches!(self, Self::Exe(_)) => {
                Self::Url.spawn_silent().map_err(|_| e)
            }
            spawned => spawned,
        }
    }

    fn spawn_silent(&self) -> std::io::Result<Child> {
        let mut c = self.command(&["-silent"]).unwrap_or_else(|| {
            let mut c = Command::new("cmd");
            c.args(["/C", "start", "", "steam://open/main"]);
            c
        });
        c.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut c, 0);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const DETACHED_PROCESS: u32 = 0x0000_0008;
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            c.creation_flags(match self {
                Self::Url => CREATE_NO_WINDOW,
                _ => DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP,
            });
        }
        c.spawn()
    }
}

impl std::fmt::Display for Starter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exe(exe) => write!(f, "{} -silent", exe.display()),
            Self::Flatpak(_) => write!(f, "flatpak run {FLATPAK_APP} -silent"),
            Self::Url => f.write_str("steam://open/main"),
        }
    }
}

/// The client to start on Linux: `steam` on `path`, or Flatpak Steam, the
/// Flatpak one first when the game's Steam install is Flatpak's.
pub fn linux_starter(path: Option<&OsStr>, home: &Path, install: &[&Path]) -> Option<Starter> {
    let find = |bin: &str| {
        path.and_then(|p| {
            std::env::split_paths(p)
                .map(|d| d.join(bin))
                .find(|f| f.is_file())
        })
    };
    let app = home.join(".var/app").join(FLATPAK_APP);
    let in_flatpak = install.iter().any(|p| p.starts_with(&app));
    let flatpak = find("flatpak")
        .filter(|_| in_flatpak || app.is_dir())
        .map(Starter::Flatpak);
    let native = find("steam").map(Starter::Exe);
    if in_flatpak {
        flatpak.or(native)
    } else {
        native.or(flatpak)
    }
}

/// steam.exe from `SteamExe` or `SteamPath`, else a Steam root's, else the
/// `steam://` handler.
pub fn windows_starter(reg: &dyn Registry, roots: &[PathBuf]) -> Starter {
    let values = steam_key_values(reg);
    values
        .path("SteamExe")
        .or_else(|| values.path("SteamPath").map(|p| p.join("steam.exe")))
        .or_else(|| roots.iter().map(|r| r.join("steam.exe")).find(|e| e.is_file()))
        .map_or(Starter::Url, Starter::Exe)
}

#[cfg(windows)]
pub fn starter(_install: &[&Path]) -> Option<Starter> {
    Some(windows_starter(&RegExe, &crate::steam::steam_roots()))
}

#[cfg(not(windows))]
pub fn starter(install: &[&Path]) -> Option<Starter> {
    let home = PathBuf::from(std::env::var_os("HOME")?);
    linux_starter(std::env::var_os("PATH").as_deref(), &home, install)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Starting,
    SigningIn,
}

impl Stage {
    pub fn text(self) -> &'static str {
        match self {
            Self::Starting => "Starting Steam…",
            Self::SigningIn => "Waiting for Steam to sign in…",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Report {
    /// Steam as the launch found it.
    Detected(Status),
    /// Steam changed while the launch waited, this long into the wait.
    Changed(Status, Duration),
    /// For the launcher's status line; each stage once.
    Stage(Stage),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ready {
    /// Steam was already up.
    Running,
    /// Steam was started, or finished starting, and signed in.
    Started,
    /// The client is up, but it never said it signed in.
    Unconfirmed,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StartError {
    #[error("Steam didn't start within {0} s; open Steam and press Play again")]
    Timeout(u64),
    #[error("Steam is open but not signed in; sign in to Steam and press Play again")]
    SignedOut,
    #[error("could not start Steam ({0}); open Steam and press Play again")]
    Spawn(String),
}

#[derive(Debug, Clone, Copy)]
pub struct Wait {
    pub timeout: Duration,
    pub poll: Duration,
}

impl Default for Wait {
    fn default() -> Self {
        Self {
            timeout: READY_TIMEOUT,
            poll: POLL_INTERVAL,
        }
    }
}

struct Reaped(Option<Child>);

impl Drop for Reaped {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            std::thread::spawn(move || child.wait().ok());
        }
    }
}

/// Bring Steam up: start it when no Steam process runs, then wait until it
/// is signed in, reporting what it sees as it changes.
pub fn ensure(
    mut detect: impl FnMut() -> Status,
    start: impl FnOnce() -> std::io::Result<Child>,
    wait: Wait,
    mut report: impl FnMut(Report),
) -> Result<Ready, StartError> {
    let mut status = detect();
    report(Report::Detected(status));
    if status.state.usable() {
        return Ok(Ready::Running);
    }
    let mut shown = None;
    let mut emit = |r: Report| {
        if let Report::Stage(stage) = r {
            if shown == Some(stage) {
                return;
            }
            shown = Some(stage);
        }
        report(r);
    };
    let mut child = Reaped(None);
    if status.state == State::Stopped {
        emit(Report::Stage(Stage::Starting));
        child.0 = Some(start().map_err(|e| StartError::Spawn(e.to_string()))?);
    }
    let since = Instant::now();
    let mut failed: Option<(Instant, ExitStatus)> = None;
    loop {
        let state = status.state;
        match state {
            State::Ready => return Ok(Ready::Started),
            State::Stopped | State::Starting => emit(Report::Stage(Stage::Starting)),
            State::SigningIn | State::Unconfirmed => emit(Report::Stage(Stage::SigningIn)),
        }
        if let Some(c) = child.0.as_mut()
            && let Ok(Some(exit)) = c.try_wait()
        {
            child.0 = None;
            if !exit.success() {
                failed = Some((Instant::now(), exit));
            }
        }
        match failed {
            Some((at, exit)) if state == State::Stopped && at.elapsed() >= EXIT_GRACE => {
                return Err(StartError::Spawn(format!("Steam exited with {exit}")));
            }
            Some(_) if state != State::Stopped => failed = None,
            _ => {}
        }
        if since.elapsed() >= wait.timeout {
            break;
        }
        std::thread::sleep(wait.poll);
        let now = detect();
        if now != status {
            emit(Report::Changed(now, since.elapsed()));
        }
        status = now;
    }
    match status.state {
        State::Unconfirmed => Ok(Ready::Unconfirmed),
        State::SigningIn => Err(StartError::SignedOut),
        _ => Err(StartError::Timeout(wait.timeout.as_secs())),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::collections::BTreeMap;

    use super::*;

    struct Home(PathBuf);

    impl Home {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("swofty-steamc-{}-{name}", std::process::id()));
            std::fs::remove_dir_all(&dir).ok();
            std::fs::create_dir_all(dir.join(".steam")).unwrap();
            Self(dir)
        }

        fn write(&self, rel: &str, text: &str) {
            let path = self.0.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
    }

    impl Drop for Home {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).ok();
        }
    }

    fn proc(pid: u32, name: &str, exe: &str, started: Option<SystemTime>) -> Proc {
        Proc {
            pid,
            ns_pid: pid,
            name: name.into(),
            exe: exe.into(),
            started,
        }
    }

    fn active(pid: u32, user: u32) -> String {
        format!(
            r#""Registry" {{ "HKLM" {{ "Software" {{ "Valve" {{ "Steam" {{ "SteamPID" "{pid}" }} }} }} }}
               "HKCU" {{ "Software" {{ "Valve" {{ "Steam" {{ "ActiveProcess" {{
                 "pid" "{pid}" "ActiveUser" "{user}" }} }} }} }} }} }}"#
        )
    }

    const CLIENT: &str = "/home/u/.local/share/Steam/ubuntu12_32/steam";

    #[test]
    fn linux_needs_a_live_client_behind_the_pid_file() {
        let home = Home::new("pidfile");
        let earlier = Some(SystemTime::now() - Duration::from_secs(60));
        assert_eq!(linux_status(&home.0, &[]), Status::STOPPED);
        home.write(".steam/steam.pid", "4242\n");
        assert_eq!(linux_status(&home.0, &[]), Status::STOPPED);
        let reused = [proc(4242, "bash", "/bin/bash", earlier)];
        assert_eq!(linux_status(&home.0, &reused).state, State::Stopped);
        let bootstrap = [proc(4300, "steam", "/bin/bash", earlier)];
        assert_eq!(
            linux_status(&home.0, &bootstrap),
            Status::new(State::Starting, 4300, Signal::Unknown)
        );
        let client = [proc(4242, "steam", CLIENT, earlier)];
        assert_eq!(
            linux_status(&home.0, &client),
            Status::new(State::Unconfirmed, 4242, Signal::NoLogin)
        );
        home.write(".steam/steam/config/loginusers.vdf", r#""users" { }"#);
        assert_eq!(linux_status(&home.0, &client).state, State::Ready);
    }

    #[test]
    fn linux_reads_the_signed_in_account_when_steam_publishes_it() {
        let home = Home::new("active");
        let later = Some(SystemTime::now() + Duration::from_secs(3600));
        let client = [proc(77, "steam", CLIENT, later)];
        home.write(".steam/registry.vdf", &active(77, 0));
        assert_eq!(linux_status(&home.0, &client).state, State::SigningIn);
        home.write(".steam/registry.vdf", &active(77, 123_456));
        assert_eq!(linux_status(&home.0, &client).state, State::Ready);
        home.write(".steam/registry.vdf", &active(78, 123_456));
        home.write(".steam/steam.pid", "77");
        assert_eq!(
            linux_status(&home.0, &client).state,
            State::Unconfirmed,
            "a stale ActiveProcess from another client says nothing"
        );
    }

    #[test]
    fn linux_dates_the_last_sign_in_against_the_client_start() {
        let home = Home::new("login");
        let now = SystemTime::now();
        home.write(".steam/steam.pid", "9");
        home.write(
            ".local/share/Steam/config/loginusers.vdf",
            r#""users" { "76561198000000001" { "MostRecent" "1" "Timestamp" "1000" } }"#,
        );
        let file = home.0.join(".local/share/Steam/config/loginusers.vdf");
        std::fs::File::options()
            .write(true)
            .open(&file)
            .unwrap()
            .set_modified(UNIX_EPOCH + Duration::from_secs(1000))
            .unwrap();
        let at = |secs: u64| {
            [proc(
                9,
                "steam",
                CLIENT,
                Some(UNIX_EPOCH + Duration::from_secs(secs)),
            )]
        };
        assert_eq!(linux_status(&home.0, &at(990)).state, State::Ready);
        assert_eq!(linux_status(&home.0, &at(1004)).state, State::Ready);
        assert_eq!(linux_status(&home.0, &at(2000)).state, State::Unconfirmed);
        let stamp = now.duration_since(UNIX_EPOCH).unwrap().as_secs().to_string();
        home.write(
            ".local/share/Steam/config/loginusers.vdf",
            &format!(r#""users" {{ "1" {{ "Timestamp" "{stamp}" }} }}"#),
        );
        assert_eq!(linux_status(&home.0, &at(2000)).state, State::Ready);
        assert_eq!(
            linux_status(&home.0, &[proc(9, "steam", CLIENT, None)]).state,
            State::Ready
        );
    }

    #[test]
    fn linux_finds_flatpak_steam_by_its_sandbox_pid() {
        let home = Home::new("flatpak");
        home.write(".var/app/com.valvesoftware.Steam/.steam/steam.pid", "2");
        home.write(
            ".var/app/com.valvesoftware.Steam/.steam/registry.vdf",
            &active(2, 5),
        );
        let sandboxed = Proc {
            ns_pid: 2,
            ..proc(
                51_000,
                "steam",
                "/home/u/.var/app/com.valvesoftware.Steam/.local/share/Steam/ubuntu12_32/steam",
                None,
            )
        };
        assert_eq!(
            linux_status(&home.0, std::slice::from_ref(&sandboxed)),
            Status::new(State::Ready, 51_000, Signal::ActiveUser(5))
        );
        home.write(".steam/steam.pid", "2");
        let native = proc(60_000, "steam", "/bin/sh", None);
        assert_eq!(
            linux_status(&home.0, &[native.clone(), sandboxed]).pid,
            Some(51_000),
            "a native pid file never matches a sandbox pid"
        );
        std::fs::remove_dir_all(home.0.join(".var")).unwrap();
        let unfiled = proc(61_000, "steam", CLIENT, None);
        assert_eq!(
            linux_status(&home.0, &[native, unfiled]),
            Status::new(State::Ready, 61_000, Signal::Unknown),
            "the client binary counts without a pid file"
        );
    }

    struct FakeRegistry(BTreeMap<&'static str, String>);

    impl Registry for FakeRegistry {
        fn query(&self, key: &str) -> Option<String> {
            Some(self.0.get(key).cloned().unwrap_or_else(|| {
                "ERROR: The system was unable to find the specified registry key or value.\r\n".into()
            }))
        }
    }

    fn active_process(pid: u32, user: u32) -> String {
        format!(
            "\r\nHKEY_CURRENT_USER\\Software\\Valve\\Steam\\ActiveProcess\r\n    pid    REG_DWORD    {pid:#x}\r\n    \
             SteamClientDll    REG_SZ    C:\\Program Files (x86)\\Steam\\steamclient.dll\r\n    \
             ActiveUser    REG_DWORD    {user:#x}\r\n    Universe    REG_SZ    Public\r\n\r\n"
        )
    }

    fn win(reg: &FakeRegistry, steam_pids: &[u32]) -> Status {
        windows_status(reg, steam_pids, &|_| None, &[])
    }

    #[test]
    fn windows_matches_active_process_to_a_running_steam_exe() {
        let none = FakeRegistry(BTreeMap::new());
        assert_eq!(win(&none, &[]), Status::STOPPED);
        assert_eq!(win(&none, &[880]).state, State::Starting);
        let reg = |pid, user| FakeRegistry([(ACTIVE_PROCESS_KEY, active_process(pid, user))].into());
        assert_eq!(win(&reg(0, 0), &[]), Status::STOPPED);
        assert_eq!(win(&reg(0, 0), &[880]).state, State::Starting);
        assert_eq!(
            win(&reg(0x1a2b, 0), &[0x1a2b]),
            Status::new(State::SigningIn, 0x1a2b, Signal::ActiveUser(0))
        );
        assert_eq!(win(&reg(0x1a2b, 98_765), &[0x1a2b]).state, State::Ready);
        assert_eq!(
            win(&reg(0x1a2b, 98_765), &[]).state,
            State::Stopped,
            "a crashed client leaves its pid behind"
        );
        let unnamed =
            FakeRegistry([(ACTIVE_PROCESS_KEY, "    pid    REG_DWORD    0x10\r\n".to_owned())].into());
        assert_eq!(win(&unnamed, &[16]).state, State::Unconfirmed);
    }

    #[test]
    fn windows_takes_a_sign_in_loginusers_records_over_active_user_0() {
        let root = Home::new("winlogin");
        let now = SystemTime::now();
        let secs = |t: SystemTime| t.duration_since(UNIX_EPOCH).unwrap().as_secs();
        let login = |t: SystemTime| {
            root.write(
                "config/loginusers.vdf",
                &format!(
                    r#""users" {{ "76561198000000001" {{ "MostRecent" "1" "Timestamp" "{}" }}
                                 "76561198000000002" {{ "MostRecent" "0" "Timestamp" "1000" }} }}"#,
                    secs(t)
                ),
            )
        };
        let registered = |name: &str, path: PathBuf| {
            FakeRegistry(
                [
                    (ACTIVE_PROCESS_KEY, active_process(0x1a2b, 0)),
                    (
                        STEAM_KEY,
                        format!("    {name}    REG_SZ    {}\r\n", path.display()),
                    ),
                ]
                .into(),
            )
        };
        let reg = registered("SteamPath", root.0.clone());
        let at = |ago: u64| move |_: u32| Some(now - Duration::from_secs(ago));
        let pids = [0x1a2b];

        assert_eq!(
            windows_status(&reg, &pids, &at(120), &[]),
            Status::new(State::SigningIn, 0x1a2b, Signal::ActiveUser(0)),
            "no loginusers.vdf"
        );
        login(now - Duration::from_secs(60));
        assert_eq!(
            windows_status(&reg, &pids, &at(120), &[]),
            Status::new(State::Ready, 0x1a2b, Signal::LoginOverActiveUser)
        );
        assert_eq!(
            windows_status(&reg, &pids, &at(30), &[]).state,
            State::SigningIn,
            "a sign-in before this steam.exe started"
        );
        assert_eq!(
            windows_status(&reg, &pids, &|_| None, &[]).state,
            State::SigningIn,
            "a steam.exe it cannot date"
        );
        let other = registered("SteamPath", root.0.join("elsewhere"));
        assert_eq!(
            windows_status(&other, &pids, &at(120), &[]).state,
            State::SigningIn
        );
        assert_eq!(
            windows_status(&other, &pids, &at(120), std::slice::from_ref(&root.0)).state,
            State::Ready
        );
        let exe = registered("SteamExe", root.0.join("steam.exe"));
        assert_eq!(windows_status(&exe, &pids, &at(120), &[]).state, State::Ready);

        login(now - Duration::from_secs(5));
        assert_eq!(
            windows_status(&reg, &pids, &at(120), &[]).state,
            State::SigningIn,
            "ActiveUser gets time to follow a fresh sign-in"
        );
        root.write("config/loginusers.vdf", r#""users" { "1" { "MostRecent" "1" } }"#);
        assert_eq!(
            windows_status(&reg, &pids, &at(120), &[]).state,
            State::SigningIn,
            "the file changing is no sign-in"
        );
        login(now - Duration::from_secs(60));
        assert_eq!(
            windows_status(&reg, &[], &at(120), &[]),
            Status::STOPPED,
            "a crashed client leaves its pid behind"
        );
        let signed_in = FakeRegistry(
            [
                (ACTIVE_PROCESS_KEY, active_process(0x1a2b, 98_765)),
                (
                    STEAM_KEY,
                    format!("    SteamPath    REG_SZ    {}\r\n", root.0.display()),
                ),
            ]
            .into(),
        );
        assert_eq!(
            windows_status(&signed_in, &pids, &at(120), &[]).signal,
            Signal::ActiveUser(98_765)
        );
    }

    #[test]
    fn windows_starts_the_registered_steam_exe() {
        let reg = |listing: &str| FakeRegistry([(STEAM_KEY, listing.to_owned())].into());
        let both = "HKEY_CURRENT_USER\\Software\\Valve\\Steam\r\n    \
                    SteamExe    REG_SZ    d:/steam/steam.exe\r\n    SteamPath    REG_SZ    d:/other\r\n";
        assert_eq!(
            windows_starter(&reg(both), &[]),
            Starter::Exe("d:/steam/steam.exe".into())
        );
        assert_eq!(
            windows_starter(&reg("    SteamPath    REG_SZ    e:/steam\r\n"), &[]),
            Starter::Exe(PathBuf::from("e:/steam").join("steam.exe"))
        );
        let root = Home::new("winroot");
        std::fs::write(root.0.join("steam.exe"), "MZ").unwrap();
        assert_eq!(
            windows_starter(&FakeRegistry(BTreeMap::new()), std::slice::from_ref(&root.0)),
            Starter::Exe(root.0.join("steam.exe"))
        );
        assert_eq!(windows_starter(&FakeRegistry(BTreeMap::new()), &[]), Starter::Url);
        assert!(Starter::Url.command(&["-applaunch"]).is_none());
    }

    #[test]
    fn linux_starts_the_steam_that_holds_the_game() {
        let home = Home::new("starter");
        let bin = home.0.join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        let path = std::env::join_paths([&bin]).unwrap();
        assert_eq!(linux_starter(Some(&path), &home.0, &[]), None);
        std::fs::write(bin.join("flatpak"), "").unwrap();
        assert_eq!(
            linux_starter(Some(&path), &home.0, &[]),
            None,
            "flatpak without Steam's app folder"
        );
        std::fs::write(bin.join("steam"), "").unwrap();
        assert_eq!(
            linux_starter(Some(&path), &home.0, &[]),
            Some(Starter::Exe(bin.join("steam")))
        );
        let app = home.0.join(".var/app/com.valvesoftware.Steam");
        std::fs::create_dir_all(&app).unwrap();
        let root = app.join(".local/share/Steam");
        assert_eq!(
            linux_starter(Some(&path), &home.0, &[&root]),
            Some(Starter::Flatpak(bin.join("flatpak")))
        );
        std::fs::remove_file(bin.join("steam")).unwrap();
        assert_eq!(
            linux_starter(Some(&path), &home.0, &[]),
            Some(Starter::Flatpak(bin.join("flatpak")))
        );
        let cmd = Starter::Flatpak(bin.join("flatpak"))
            .command(&["-applaunch", "3059520"])
            .unwrap();
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(args, ["run", FLATPAK_APP, "-applaunch", "3059520"]);
        assert_eq!(linux_starter(None, &home.0, &[]), None);
    }

    fn quick() -> Wait {
        Wait {
            timeout: Duration::from_millis(300),
            poll: Duration::from_millis(10),
        }
    }

    fn scripted(states: &[State]) -> impl FnMut() -> Status + use<> {
        let states = states.to_vec();
        let i = Cell::new(0);
        move || {
            let n = i.get();
            i.set(n + 1);
            Status {
                state: states[n.min(states.len() - 1)],
                pid: None,
                signal: Signal::Unknown,
            }
        }
    }

    fn never() -> std::io::Result<Child> {
        panic!("Steam was started while it was already up")
    }

    fn stages(reports: &[Report]) -> Vec<Stage> {
        reports
            .iter()
            .filter_map(|r| match r {
                Report::Stage(s) => Some(*s),
                _ => None,
            })
            .collect()
    }

    fn changes(reports: &[Report]) -> Vec<State> {
        reports
            .iter()
            .filter_map(|r| match r {
                Report::Changed(s, _) => Some(s.state),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn says_what_it_sees() {
        assert_eq!(Status::STOPPED.to_string(), "not running");
        assert_eq!(
            Status::new(State::Starting, 9, Signal::Unknown).to_string(),
            "starting (pid 9, the client hasn't registered yet)"
        );
        assert_eq!(
            Status::new(State::SigningIn, 4242, Signal::ActiveUser(0)).to_string(),
            "waiting for sign-in (pid 4242, ActiveUser 0)"
        );
        assert_eq!(
            Status::new(State::Unconfirmed, 4242, Signal::NoLogin).to_string(),
            "waiting for sign-in (pid 4242, no sign-in in loginusers.vdf since it started)"
        );
        assert_eq!(
            Status::new(State::Ready, 4242, Signal::LoginFile).to_string(),
            "ready (pid 4242, loginusers.vdf written since it started)"
        );
        assert_eq!(
            Status::new(State::Ready, 4242, Signal::LoginOverActiveUser).to_string(),
            "ready (pid 4242, ActiveUser 0, but loginusers.vdf records a sign-in since it started)"
        );
    }

    #[test]
    fn leaves_a_running_steam_alone() {
        let seen = RefCell::new(Vec::new());
        for up in [State::Ready, State::Unconfirmed] {
            let got = ensure(scripted(&[up]), never, quick(), |r| seen.borrow_mut().push(r));
            assert_eq!(got, Ok(Ready::Running));
        }
        let seen = seen.into_inner();
        assert_eq!(seen.len(), 2);
        assert!(seen.iter().all(|r| matches!(r, Report::Detected(_))), "{seen:?}");
    }

    #[test]
    fn starts_steam_and_reports_each_stage_once() {
        let mut seen = Vec::new();
        let starts = Cell::new(0);
        let got = ensure(
            scripted(&[
                State::Stopped,
                State::Stopped,
                State::Starting,
                State::Unconfirmed,
                State::SigningIn,
                State::Ready,
            ]),
            || {
                starts.set(starts.get() + 1);
                Command::new("true").spawn()
            },
            quick(),
            |r| seen.push(r),
        );
        assert_eq!(got, Ok(Ready::Started));
        assert_eq!(starts.get(), 1);
        assert!(matches!(seen[0], Report::Detected(s) if s == Status::STOPPED));
        assert_eq!(stages(&seen), [Stage::Starting, Stage::SigningIn]);
        assert_eq!(
            changes(&seen),
            [
                State::Starting,
                State::Unconfirmed,
                State::SigningIn,
                State::Ready
            ]
        );
    }

    #[test]
    fn waits_for_a_steam_already_starting_without_starting_another() {
        let mut seen = Vec::new();
        let got = ensure(
            scripted(&[State::Starting, State::SigningIn, State::Ready]),
            never,
            quick(),
            |r| seen.push(r),
        );
        assert_eq!(got, Ok(Ready::Started));
        assert_eq!(stages(&seen), [Stage::Starting, Stage::SigningIn]);
    }

    #[test]
    fn gives_up_after_the_timeout() {
        let spawn = || Command::new("sleep").arg("5").spawn();
        let got = ensure(scripted(&[State::Stopped]), spawn, quick(), |_| {});
        assert_eq!(got, Err(StartError::Timeout(0)));
        assert_eq!(
            StartError::Timeout(90).to_string(),
            "Steam didn't start within 90 s; open Steam and press Play again"
        );
        let got = ensure(
            scripted(&[State::Stopped, State::SigningIn]),
            spawn,
            quick(),
            |_| {},
        );
        assert_eq!(got, Err(StartError::SignedOut));
        let got = ensure(
            scripted(&[State::Stopped, State::Unconfirmed]),
            spawn,
            quick(),
            |_| {},
        );
        assert_eq!(got, Ok(Ready::Unconfirmed));
    }

    #[test]
    fn says_why_steam_could_not_start() {
        let got = ensure(
            scripted(&[State::Stopped]),
            || {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Steam was not found",
                ))
            },
            quick(),
            |_| {},
        );
        assert_eq!(
            got.unwrap_err().to_string(),
            "could not start Steam (Steam was not found); open Steam and press Play again"
        );
    }

    #[cfg(unix)]
    #[test]
    fn reports_a_steam_that_exits_with_an_error() {
        let wait = Wait {
            timeout: Duration::from_secs(10),
            poll: Duration::from_millis(50),
        };
        let started = Instant::now();
        let got = ensure(
            scripted(&[State::Stopped]),
            || Command::new("sh").args(["-c", "exit 3"]).spawn(),
            wait,
            |_| {},
        );
        assert_eq!(
            got,
            Err(StartError::Spawn("Steam exited with exit status: 3".into()))
        );
        assert!(started.elapsed() < Duration::from_secs(6));
    }

    #[cfg(target_os = "linux")]
    mod fake_steam {
        use std::os::unix::fs::PermissionsExt;

        use super::*;

        static ONE_AT_A_TIME: std::sync::Mutex<()> = std::sync::Mutex::new(());

        struct Fake {
            home: Home,
            bin: PathBuf,
            _turn: std::sync::MutexGuard<'static, ()>,
        }

        impl Fake {
            fn new(name: &str, body: &str) -> Self {
                let turn = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
                let home = Home::new(name);
                let bin = home.0.join("bin");
                std::fs::create_dir_all(&bin).unwrap();
                let script = bin.join("steam");
                let body = body.replace("$FAKE_HOME", &home.0.display().to_string());
                std::fs::write(&script, format!("#!/bin/sh\n{body}\nwhile :; do sleep 1; done\n")).unwrap();
                std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
                Self {
                    home,
                    bin,
                    _turn: turn,
                }
            }

            fn starter(&self) -> Starter {
                let path = std::env::join_paths([&self.bin]).unwrap();
                linux_starter(Some(&path), &self.home.0, &[]).unwrap()
            }

            fn status(&self) -> Status {
                let ours: Vec<Proc> = processes()
                    .into_iter()
                    .filter(|p| crate::process::descends_from(p.pid, std::process::id()))
                    .collect();
                linux_status(&self.home.0, &ours)
            }
        }

        impl Drop for Fake {
            fn drop(&mut self) {
                let since = Instant::now();
                while let Some(pid) = self.status().pid {
                    Command::new("kill")
                        .args(["-TERM", "--", &format!("-{pid}")])
                        .status()
                        .ok();
                    if since.elapsed() > Duration::from_secs(5) {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
        }

        #[test]
        fn starts_a_fake_steam_and_waits_for_it_to_sign_in() {
            let fake = Fake::new(
                "signs-in",
                "sleep 0.3\necho $$ > \"$FAKE_HOME/.steam/steam.pid\"\nsleep 0.3\n\
                 printf '\"Registry\" { \"HKCU\" { \"Software\" { \"Valve\" { \"Steam\" { \"ActiveProcess\" { \"pid\" \"%s\" \"ActiveUser\" \"42\" } } } } } }' $$ > \"$FAKE_HOME/.steam/registry.vdf\"",
            );
            assert_eq!(fake.status(), Status::STOPPED);
            let starter = fake.starter();
            assert_eq!(starter, Starter::Exe(fake.bin.join("steam")));
            let mut seen = Vec::new();
            let wait = Wait {
                timeout: Duration::from_secs(20),
                poll: Duration::from_millis(50),
            };
            let got = ensure(|| fake.status(), || starter.start(), wait, |r| seen.push(r));
            assert_eq!(got, Ok(Ready::Started));
            assert_eq!(stages(&seen), [Stage::Starting, Stage::SigningIn]);
            let Some(Report::Changed(last, after)) = seen.last() else {
                panic!("{seen:?}");
            };
            assert_eq!((last.state, last.signal), (State::Ready, Signal::ActiveUser(42)));
            assert!(*after >= Duration::from_millis(500), "{after:?}");
            let up = fake.status();
            assert_eq!(up.state, State::Ready);
            let again = ensure(
                || fake.status(),
                never,
                wait,
                |r| assert!(matches!(r, Report::Detected(_)), "{r:?}"),
            );
            assert_eq!(again, Ok(Ready::Running));
            assert!(up.pid.is_some());
        }

        #[test]
        fn times_out_on_a_fake_steam_that_never_comes_up() {
            let fake = Fake::new("never", "true");
            let starter = fake.starter();
            let wait = Wait {
                timeout: Duration::from_millis(1500),
                poll: Duration::from_millis(50),
            };
            let started = Instant::now();
            let got = ensure(|| fake.status(), || starter.start(), wait, |_| {});
            assert_eq!(got, Err(StartError::Timeout(1)));
            assert!(started.elapsed() >= Duration::from_millis(1500));
        }
    }
}
