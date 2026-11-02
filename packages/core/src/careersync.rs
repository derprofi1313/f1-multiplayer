//! The league career's game sync on this PC: the generated addon [`gamesync::ADDON_ID`] in the
//! data folder's `addons/`, made from the shared league document for the next round.
//!
//! [`gamesync`] turns the league into files; this module reads what it needs from the installed
//! Classic Seasons addons ([`context`]), writes the addon and turns it on in a profile
//! ([`prepare`]), and takes it away again ([`restore`]). It never builds: the next launch builds
//! the profile as it builds any addon, and a LAN lobby fingerprints it as any gameplay addon, so
//! two PCs that prepared the same league document match.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::league::gamesync::{self, ADDON_ID, Overlay, SyncContext};
use crate::profile::{Profile, ProfileAddon};
use crate::store::Store;

/// The id prefix of the Classic Seasons addons; the overlay loads after all of them.
pub const CLASSIC_SEASONS_PREFIX: &str = "swofty.classic-seasons";
/// The Classic Seasons core: the shared drivers and the AI pace table.
pub const CLASSIC_SEASONS_CORE: &str = "swofty.classic-seasons";
/// Where [`prepare`] writes the addon before it moves it in; [`Store`] moves folders of this name
/// in place of copying them, and leaves hidden folders out of its addon list.
const SCRATCH_PREFIX: &str = ".importing-f1mp-league";

/// What this PC has of the overlay, for one profile.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SyncState {
    /// The overlay installed in the data folder, as its `league.json` says.
    pub prepared: Option<Prepared>,
    /// It is on in the profile.
    pub enabled: bool,
    /// The game is running: a changed overlay waits until it closes.
    pub game_running: bool,
}

/// An installed overlay: the career, the season and round it was made for, and its `hash8`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prepared {
    pub career: String,
    pub year: u16,
    pub round: u8,
    pub hash: String,
}

impl Prepared {
    /// The overlay `league.json` describes; `None` when it isn't one.
    pub fn parse(league_json: &[u8]) -> Option<Self> {
        let v: Value = serde_json::from_slice(league_json).ok()?;
        Some(Self {
            career: v.get("career")?.as_str()?.to_owned(),
            year: u16::try_from(v.get("year")?.as_u64()?).ok()?,
            round: u8::try_from(v.get("round")?.as_u64()?).ok()?,
            hash: v.get("hash")?.as_str()?.to_owned(),
        })
    }
}

/// The installed Classic Seasons files the generator reads, owned; [`OwnedSyncContext::get`]
/// lends them as a [`SyncContext`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OwnedSyncContext {
    pub content: Value,
    pub cs_drivers: Value,
    pub liveries: Value,
    pub ai_slots: Value,
    pub loader_applies_rows: bool,
}

impl OwnedSyncContext {
    pub fn get(&self) -> SyncContext<'_> {
        SyncContext {
            content: &self.content,
            cs_drivers: &self.cs_drivers,
            liveries: &self.liveries,
            ai_slots: &self.ai_slots,
            loader_applies_rows: self.loader_applies_rows,
        }
    }
}

/// Reads what the generator needs for `year` from the installed Classic Seasons addons. A year
/// Classic Seasons has a box for needs that year's addon and the core; a year the game has itself
/// (see [`gamesync::season_addon`]) reads the core's files when it is installed and goes without
/// them when it isn't.
pub fn context(store: &Store, year: u16) -> Result<OwnedSyncContext, String> {
    let installed = store.addons().map_err(|e| e.to_string())?;
    let loader_applies_rows =
        crate::builtin::loader_dir().is_some_and(|d| crate::stdlib::manifest::core_applies_rows(&d));
    let dir = |id: &str| installed.contains_key(id).then(|| store.addons_dir().join(id));
    let core = dir(CLASSIC_SEASONS_CORE);
    let Some(season) = gamesync::season_addon(year) else {
        let read = |rel: &str| {
            core.as_deref()
                .map(|d| read_optional(d, rel))
                .transpose()
                .map(Option::flatten)
        };
        return Ok(OwnedSyncContext {
            content: Value::Null,
            liveries: Value::Null,
            cs_drivers: read(DRIVERS)?.unwrap_or(Value::Null),
            ai_slots: read(AI_SLOTS)?.unwrap_or(Value::Null),
            loader_applies_rows,
        });
    };
    let Some(season_dir) = dir(&season) else {
        return Err(format!(
            "install Classic Seasons {year} first: the league career races {year} with its cars and drivers \
             (addon {season})"
        ));
    };
    let Some(core) = core else {
        return Err(format!(
            "install Classic Seasons first: Classic Seasons {year} needs it (addon {CLASSIC_SEASONS_CORE})"
        ));
    };
    Ok(OwnedSyncContext {
        content: read_required(&season_dir, CONTENT, &format!("Classic Seasons {year}"))?,
        liveries: read_required(&season_dir, LIVERIES, &format!("Classic Seasons {year}"))?,
        cs_drivers: read_required(&core, DRIVERS, "Classic Seasons")?,
        ai_slots: read_required(&core, AI_SLOTS, "Classic Seasons")?,
        loader_applies_rows,
    })
}

const CONTENT: &str = "registry/content.json";
const LIVERIES: &str = "assets/liveries.json";
const DRIVERS: &str = "registry/drivers.json";
const AI_SLOTS: &str = "data/ai_slots.json";

fn read_optional(dir: &Path, rel: &str) -> Result<Option<Value>, String> {
    let path = dir.join(rel);
    match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|e| format!("{} isn't JSON ({e}); install the addon again", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("couldn't read {}: {e}", path.display())),
    }
}

fn read_required(dir: &Path, rel: &str, name: &str) -> Result<Value, String> {
    read_optional(dir, rel)?
        .ok_or_else(|| format!("{name} has no {rel}; update it in the Addon Browser, or install it again"))
}

/// Writes the overlay's addon into the data folder when it differs from the installed one, and
/// turns it on in `profile` after Classic Seasons. It doesn't build: the next launch does. While
/// the game runs, a changed overlay is left unwritten and the state says so
/// ([`SyncState::game_running`]); the launcher prepares it again after the game closes.
pub fn prepare(store: &Store, profile: &str, overlay: &Overlay) -> Result<SyncState, String> {
    prepare_as(store, profile, overlay, crate::process::game_running())
}

/// [`prepare`] with the game's state known (tests pass `false`, whatever runs on the PC).
pub fn prepare_as(
    store: &Store,
    profile: &str,
    overlay: &Overlay,
    game_running: bool,
) -> Result<SyncState, String> {
    let ctx = context(store, overlay.year)?;
    let files = gamesync::files(overlay, &ctx.get());
    let mut p = store.profile(profile).map_err(|e| e.to_string())?;
    if p.is_vanilla() {
        return Err(format!(
            "the {profile} profile plays without addons; pick another profile for the league career"
        ));
    }
    let wanted: BTreeMap<String, Vec<u8>> = files.iter().cloned().collect();
    if installed_files(&addon_dir(store)).as_ref() != Some(&wanted) {
        if game_running {
            return Ok(state_as(store, profile, game_running));
        }
        write_addon(store, &files)?;
    }
    if place(&mut p) {
        store
            .update_profile(profile, |p| {
                place(p);
                Ok(())
            })
            .map_err(|e| e.to_string())?;
    }
    Ok(state_as(store, profile, game_running))
}

/// Takes the overlay out of every profile and deletes it, so `profile`'s addon list and the LAN
/// fingerprint are what they were before [`prepare`]. The next launch builds without it: the
/// same build (same build hash) as before it was prepared, made again, since the build keeps
/// only a profile's latest build folder.
pub fn restore(store: &Store, profile: &str) -> Result<(), String> {
    clear_scratch(store);
    if addon_dir(store).join(gamesync::MANIFEST_FILE).is_file() {
        store.remove_addon(ADDON_ID).map_err(|e| e.to_string())?;
    } else if addon_dir(store).exists() {
        std::fs::remove_dir_all(addon_dir(store)).map_err(|e| e.to_string())?;
    }
    let p = store.profile(profile).map_err(|e| e.to_string())?;
    if !p.is_vanilla() && p.addons.iter().any(|a| a.id == ADDON_ID) {
        store
            .update_profile(profile, |p| {
                p.addons.retain(|a| a.id != ADDON_ID);
                p.options.remove(ADDON_ID);
                p.applied.remove(ADDON_ID);
                Ok(())
            })
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// What this PC has of the overlay for `profile`.
pub fn state(store: &Store, profile: &str) -> SyncState {
    state_as(store, profile, crate::process::game_running())
}

pub(crate) fn state_as(store: &Store, profile: &str, game_running: bool) -> SyncState {
    let prepared = std::fs::read(addon_dir(store).join(gamesync::LEAGUE_FILE))
        .ok()
        .and_then(|b| Prepared::parse(&b));
    let enabled = store
        .profile(profile)
        .is_ok_and(|p| p.addons.iter().any(|a| a.id == ADDON_ID && a.enabled));
    SyncState {
        prepared,
        enabled,
        game_running,
    }
}

fn addon_dir(store: &Store) -> PathBuf {
    store.addons_dir().join(ADDON_ID)
}

/// The files of the installed overlay, by path relative to it with `/`; `None` without one.
fn installed_files(dir: &Path) -> Option<BTreeMap<String, Vec<u8>>> {
    if !dir.is_dir() {
        return None;
    }
    let files = crate::builtin::loader_files(dir).ok()?;
    let mut out = BTreeMap::new();
    for (rel, path) in files {
        out.insert(rel, std::fs::read(path).ok()?);
    }
    Some(out)
}

/// Writes `files` into a scratch folder in `addons/`, then has the store move it in place of the
/// installed overlay, whole or not at all.
fn write_addon(store: &Store, files: &[(String, Vec<u8>)]) -> Result<(), String> {
    clear_scratch(store);
    let scratch = store
        .addons_dir()
        .join(format!("{SCRATCH_PREFIX}-{}", std::process::id()));
    let written = (|| -> std::io::Result<()> {
        for (rel, bytes) in files {
            let path = scratch.join(rel);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, bytes)?;
        }
        Ok(())
    })();
    let installed = written
        .map_err(|e| format!("couldn't write the league career overlay: {e}"))
        .and_then(|()| {
            store
                .install_addon(&scratch)
                .map(|_| ())
                .map_err(|e| format!("couldn't install the league career overlay: {e}"))
        });
    if scratch.exists() {
        std::fs::remove_dir_all(&scratch).ok();
    }
    installed
}

/// Scratch folders a stopped [`prepare`] left behind.
fn clear_scratch(store: &Store) {
    let Ok(rd) = std::fs::read_dir(store.addons_dir()) else {
        return;
    };
    for e in rd.flatten() {
        if e.file_name().to_string_lossy().starts_with(SCRATCH_PREFIX) {
            std::fs::remove_dir_all(e.path()).ok();
        }
    }
}

/// Puts the overlay on in `p`, right after the last Classic Seasons addon it lists (at the end
/// when it lists none); whether that changed anything.
fn place(p: &mut Profile) -> bool {
    let at = p.addons.iter().position(|a| a.id == ADDON_ID);
    let last_cs = p
        .addons
        .iter()
        .rposition(|a| a.id.starts_with(CLASSIC_SEASONS_PREFIX));
    if let Some(at) = at
        && p.addons[at].enabled
        && last_cs.is_none_or(|cs| cs < at)
    {
        return false;
    }
    let entry = match at {
        Some(at) => p.addons.remove(at),
        None => ProfileAddon {
            id: ADDON_ID.into(),
            enabled: true,
            unknown: BTreeMap::new(),
        },
    };
    let last_cs = p
        .addons
        .iter()
        .rposition(|a| a.id.starts_with(CLASSIC_SEASONS_PREFIX));
    let to = last_cs.map_or(p.addons.len(), |cs| cs + 1);
    p.addons.insert(
        to,
        ProfileAddon {
            enabled: true,
            ..entry
        },
    );
    true
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use super::*;
    use crate::baseline::Identity;
    use crate::league::{self, Action, Actor, League, Setup};

    const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/classic-seasons-mini");
    const SEASON: &str = "swofty.classic-seasons-2016";
    const PROFILE: &str = "career";

    fn root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("f1mp-careersync-{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&root).ok();
        root
    }

    /// A data folder with the Classic Seasons core and `seasons` installed from the fixture, on in
    /// [`PROFILE`] and followed there by another addon.
    fn store_with(name: &str, seasons: &[&str]) -> Store {
        let root = root(name);
        let store = Store::new(root.join("data"));
        for id in std::iter::once(CLASSIC_SEASONS_CORE).chain(seasons.iter().copied()) {
            store.install_addon(&Path::new(FIXTURE).join(id)).unwrap();
        }
        let other = root.join("t.after");
        std::fs::create_dir_all(&other).unwrap();
        std::fs::write(
            other.join("swofty.json"),
            r#"{ "schema_version": 1, "id": "t.after", "name": "After", "version": "1.0.0", "author": "t",
                 "compat": { "game": "f1_25" }, "kind": ["database"] }"#,
        )
        .unwrap();
        store.install_addon(&other).unwrap();
        store
            .update_profile(PROFILE, |p| {
                for id in std::iter::once(CLASSIC_SEASONS_CORE)
                    .chain(seasons.iter().copied())
                    .chain(["t.after"])
                {
                    p.set_enabled(id, true)?;
                }
                Ok(())
            })
            .unwrap();
        store
    }

    fn store(name: &str) -> Store {
        store_with(name, &[SEASON])
    }

    fn clean(store: Store) {
        std::fs::remove_dir_all(store.root.parent().unwrap()).ok();
    }

    fn anna() -> Actor {
        Actor {
            id: "u-anna".into(),
            name: "Anna".into(),
            owner: true,
        }
    }

    /// A solo 2016 career at Ferrari with the game identity on.
    fn career() -> League {
        let mut l = League::start(Setup::default(), &anna(), Some(7), 42).unwrap();
        league::apply(
            &mut l,
            &anna(),
            Action::Seat {
                slot: 2,
                replaces: None,
            },
        )
        .unwrap();
        league::apply(&mut l, &anna(), Action::Begin).unwrap();
        l.options.game_identity = true;
        l
    }

    fn next_round(l: &mut League) {
        league::apply(l, &anna(), Action::Simulate { round: None }).unwrap();
    }

    fn undo(l: &mut League) {
        league::apply(l, &anna(), Action::Undo).unwrap();
    }

    fn overlay_of(store: &Store, l: &League) -> Overlay {
        let ctx = context(store, l.year()).unwrap();
        gamesync::overlay(l, "c-1", "Anna's career", &ctx.get()).expect("an overlay while racing")
    }

    fn ids(store: &Store) -> Vec<String> {
        store
            .profile(PROFILE)
            .unwrap()
            .addons
            .iter()
            .map(|a| a.id.clone())
            .collect()
    }

    fn profile_path(store: &Store) -> PathBuf {
        store.profiles_dir().join(format!("{PROFILE}.json"))
    }

    fn print(store: &Store) -> String {
        let game = Identity {
            build_id: "25251167".into(),
            exe_sha256: "a".repeat(64),
        };
        crate::multiplayer::fingerprint::fingerprint(store, &store.profile(PROFILE).unwrap(), &game)
            .unwrap()
            .digest
    }

    fn age(path: &Path) {
        let old = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000_000);
        std::fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(old)
            .unwrap();
    }

    fn modified(path: &Path) -> SystemTime {
        std::fs::metadata(path).unwrap().modified().unwrap()
    }

    #[test]
    fn prepare_enables_after_classic_seasons_and_skips_unchanged_writes() {
        let store = store("prepare");
        let mut l = career();
        let state = prepare_as(&store, PROFILE, &overlay_of(&store, &l), false).unwrap();
        let prepared = state.prepared.clone().expect("prepared");
        assert_eq!(
            (prepared.career.as_str(), prepared.year, prepared.round),
            ("c-1", 2016, 1)
        );
        assert_eq!(prepared.hash.len(), 8);
        assert!(state.enabled && !state.game_running);
        assert_eq!(state, state_as(&store, PROFILE, false));
        let listed = ["swofty.core", CLASSIC_SEASONS_CORE, SEASON, ADDON_ID, "t.after"];
        assert_eq!(ids(&store), listed);
        let installed = store.addons().unwrap();
        assert_eq!(installed[ADDON_ID].version.build.as_str(), prepared.hash);
        let order = crate::resolver::resolve(&store.profile(PROFILE).unwrap(), &installed)
            .unwrap()
            .order;
        let at = |id: &str| order.iter().position(|o| o == id).unwrap();
        assert!(at(ADDON_ID) > at(SEASON) && at(ADDON_ID) > at(CLASSIC_SEASONS_CORE));

        // The same round again writes nothing: not the addon, not the profile.
        let manifest = addon_dir(&store).join(gamesync::MANIFEST_FILE);
        age(&manifest);
        age(&profile_path(&store));
        let (was_manifest, was_profile) = (modified(&manifest), modified(&profile_path(&store)));
        assert_eq!(
            prepare_as(&store, PROFILE, &overlay_of(&store, &l), false).unwrap(),
            state
        );
        assert_eq!(modified(&manifest), was_manifest);
        assert_eq!(modified(&profile_path(&store)), was_profile);
        let scratch = std::fs::read_dir(store.addons_dir())
            .unwrap()
            .flatten()
            .filter(|e| e.file_type().unwrap().is_dir() && e.file_name().to_string_lossy().starts_with('.'))
            .count();
        assert_eq!(scratch, 0, "no scratch or staging folder is left");

        // Turned off by hand, it is turned on again in its place.
        store
            .update_profile(PROFILE, |p| p.set_enabled(ADDON_ID, false))
            .unwrap();
        assert!(!state_as(&store, PROFILE, false).enabled);
        assert!(
            prepare_as(&store, PROFILE, &overlay_of(&store, &l), false)
                .unwrap()
                .enabled
        );
        assert_eq!(ids(&store), listed);

        // The next round rewrites it.
        next_round(&mut l);
        let next = prepare_as(&store, PROFILE, &overlay_of(&store, &l), false)
            .unwrap()
            .prepared
            .unwrap();
        assert_eq!(next.round, 2);
        assert_ne!(next.hash, prepared.hash);
        assert_ne!(modified(&manifest), was_manifest);
        assert_eq!(ids(&store), listed);
        clean(store);
    }

    #[test]
    fn restore_puts_the_profile_back() {
        let store = store("restore");
        let profile_before = std::fs::read(profile_path(&store)).unwrap();
        let print_before = print(&store);
        prepare_as(&store, PROFILE, &overlay_of(&store, &career()), false).unwrap();
        assert_ne!(print(&store), print_before);

        restore(&store, PROFILE).unwrap();
        assert_eq!(std::fs::read(profile_path(&store)).unwrap(), profile_before);
        assert_eq!(print(&store), print_before);
        assert!(!addon_dir(&store).exists());
        assert!(!store.addons().unwrap().contains_key(ADDON_ID));
        assert_eq!(state_as(&store, PROFILE, false), SyncState::default());
        // Restoring again, with nothing prepared, changes nothing.
        restore(&store, PROFILE).unwrap();
        assert_eq!(std::fs::read(profile_path(&store)).unwrap(), profile_before);

        // A profile that only lists it (its folder deleted by hand) loses the entry too.
        prepare_as(&store, PROFILE, &overlay_of(&store, &career()), false).unwrap();
        std::fs::remove_dir_all(addon_dir(&store)).unwrap();
        restore(&store, PROFILE).unwrap();
        assert_eq!(std::fs::read(profile_path(&store)).unwrap(), profile_before);
        clean(store);
    }

    #[test]
    fn two_stores_get_the_same_fingerprint_and_a_new_round_changes_it() {
        let (a, b) = (store("pc-a"), store("pc-b"));
        let mut anna = career();
        // Ben's launcher has the document as the host server sent it.
        let mut ben: League = serde_json::from_str(&serde_json::to_string(&anna).unwrap()).unwrap();
        prepare_as(&a, PROFILE, &overlay_of(&a, &anna), false).unwrap();
        prepare_as(&b, PROFILE, &overlay_of(&b, &ben), false).unwrap();
        let files = |s: &Store| installed_files(&addon_dir(s)).unwrap();
        assert_eq!(files(&a), files(&b));
        let first = print(&a);
        assert_eq!(first, print(&b));

        next_round(&mut anna);
        next_round(&mut ben);
        prepare_as(&a, PROFILE, &overlay_of(&a, &anna), false).unwrap();
        prepare_as(&b, PROFILE, &overlay_of(&b, &ben), false).unwrap();
        assert_eq!(files(&a), files(&b));
        assert_eq!(print(&a), print(&b));
        assert_ne!(print(&a), first);

        // A PC still on the old round shows as a difference.
        undo(&mut ben);
        prepare_as(&b, PROFILE, &overlay_of(&b, &ben), false).unwrap();
        assert_eq!(print(&b), first);
        assert_ne!(print(&a), print(&b));
        clean(a);
        clean(b);
    }

    #[test]
    fn prepare_refuses_while_the_game_runs() {
        let store = store("running");
        let mut l = career();
        // Nothing is written while the game runs, not even a first overlay.
        let state = prepare_as(&store, PROFILE, &overlay_of(&store, &l), true).unwrap();
        let none = SyncState {
            prepared: None,
            enabled: false,
            game_running: true,
        };
        assert_eq!(state, none);
        assert!(!addon_dir(&store).exists());

        let first = prepare_as(&store, PROFILE, &overlay_of(&store, &l), false).unwrap();
        next_round(&mut l);
        let files = installed_files(&addon_dir(&store));
        let state = prepare_as(&store, PROFILE, &overlay_of(&store, &l), true).unwrap();
        assert!(state.game_running);
        assert_eq!(
            state.prepared, first.prepared,
            "the old round stays until the game closes"
        );
        assert_eq!(installed_files(&addon_dir(&store)), files);

        // An unchanged overlay is fine while it runs.
        let mut back = l.clone();
        undo(&mut back);
        let same = prepare_as(&store, PROFILE, &overlay_of(&store, &back), true).unwrap();
        assert_eq!(same.prepared, first.prepared);
        assert!(same.enabled && same.game_running);

        // Once it has closed, the new round goes in.
        let after = prepare_as(&store, PROFILE, &overlay_of(&store, &l), false).unwrap();
        assert_eq!(after.prepared.unwrap().round, 2);
        clean(store);
    }

    #[test]
    fn context_reports_a_missing_classic_seasons_year() {
        let store = store("context");
        let ctx = context(&store, 2016).unwrap();
        assert!(ctx.content["content.package"]["f1_2016"]["teams"].is_array());
        assert!(ctx.liveries["ops"].is_array());
        assert!(ctx.cs_drivers["driver.identity"]["felipe_nasr"].is_object());
        assert_eq!(ctx.ai_slots["percent_per_kg"], serde_json::json!(0.0308));

        let missing = context(&store, 2017).unwrap_err();
        assert!(
            missing.contains("install Classic Seasons 2017 first"),
            "{missing}"
        );
        assert!(missing.contains("swofty.classic-seasons-2017"), "{missing}");

        // The game's own seasons need no season addon.
        let stock = context(&store, 2025).unwrap();
        assert_eq!((&stock.content, &stock.liveries), (&Value::Null, &Value::Null));
        assert!(stock.ai_slots.is_object());

        // Without the core, a Classic Seasons year can't be read.
        store.remove_addon(CLASSIC_SEASONS_CORE).unwrap();
        let no_core = context(&store, 2016).unwrap_err();
        assert!(no_core.contains("install Classic Seasons first"), "{no_core}");
        assert_eq!(context(&store, 2026).unwrap().ai_slots, Value::Null);

        // A damaged or missing file is named.
        let other = store_with("context-damaged", &[SEASON]);
        let content = other.addons_dir().join(SEASON).join(CONTENT);
        std::fs::write(&content, "{").unwrap();
        let damaged = context(&other, 2016).unwrap_err();
        assert!(damaged.contains("content.json isn't JSON"), "{damaged}");
        std::fs::remove_file(&content).unwrap();
        let gone = context(&other, 2016).unwrap_err();
        assert!(gone.contains("has no registry/content.json"), "{gone}");
        clean(store);
        clean(other);
    }

    #[test]
    fn the_vanilla_profile_takes_no_overlay() {
        let store = store("vanilla");
        let overlay = overlay_of(&store, &career());
        let err = prepare_as(&store, crate::profile::VANILLA, &overlay, false).unwrap_err();
        assert!(err.contains("plays without addons"), "{err}");
        assert!(!addon_dir(&store).exists());
        clean(store);
    }

    #[test]
    fn the_launch_build_takes_the_overlay_and_a_restored_profile_builds_as_before() {
        let _pin = crate::builtin::LoaderPin::none();
        let store = store("build");
        let dir = store.root.parent().unwrap().join("game");
        std::fs::create_dir_all(dir.join("game_package/gamedatabases")).unwrap();
        std::fs::write(dir.join(crate::GAME_EXE), "exe").unwrap();
        // The rows the identity overlay's database fallback (no loader here) renames: Anna's seat.
        let db = crate::league::gamesync::identity::box_rows(2016, &["sebastian_vettel", "kimi_raikkonen"])
            .to_bytes();
        std::fs::write(dir.join(crate::build::GVDB), db).unwrap();
        let game = crate::install::Game {
            identity: Identity::detect(&dir, Some(1)).unwrap(),
            dir,
            user_reg: None,
        };
        let stock = crate::build::build(&store, PROFILE, &game).unwrap();
        assert!(!stock.order.iter().any(|id| id == ADDON_ID));
        prepare_as(&store, PROFILE, &overlay_of(&store, &career()), false).unwrap();
        let with = crate::build::build(&store, PROFILE, &game).unwrap();
        let at = |id: &str| with.order.iter().position(|o| o == id).unwrap();
        assert!(at(ADDON_ID) > at(SEASON), "{:?}", with.order);
        assert_ne!(with.buildhash, stock.buildhash);

        restore(&store, PROFILE).unwrap();
        let back = crate::build::build(&store, PROFILE, &game).unwrap();
        // The same inputs as before it was prepared: the same build. (The build keeps only a
        // profile's latest build folder, so this one is made again rather than reused.)
        assert_eq!(back.buildhash, stock.buildhash);
        assert_eq!(back.order, stock.order);
        clean(store);
    }
}
