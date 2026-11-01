//! The one-time setup before the first modded Play. The player says whether
//! the game folder holds mods Swofty did not add, the launcher puts back what
//! it changed and starts its record of the original game files over, and the
//! player verifies the game files in their store. The originals Swofty records
//! after that are the store's own.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::install::{Game, Source};
use crate::journal::Journal;
use crate::recovery::{Restored, SESSION_HELD, SessionLock};
use crate::{Error, Result, read_json, with_retry, write_json};

/// Raised to have every player go through the setup again, which starts each
/// record of original game files over.
pub const VERSION: u32 = 1;

/// Where [`reset`] moves the record of original game files until [`finish`]
/// deletes it.
pub const RETIRED: &str = "baseline-retired";

pub const NEEDED: &str = "F1 Multiplayer needs a one-time setup before modded play: open F1 Multiplayer and press Play, \
                          or run `f1mp setup reset`, verify the game files, then run `f1mp setup done`";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Setup {
    pub version: u32,
    /// The player said the game folder held mods Swofty did not add.
    pub other_mods: bool,
    pub reset_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub done_at: Option<u64>,
    /// The player saw the notice that the first launch after the setup is slow.
    #[serde(default)]
    pub noticed: bool,
}

fn path(data: &Path) -> PathBuf {
    data.join("state/setup.json")
}

pub fn read(data: &Path) -> Option<Setup> {
    read_json(&path(data)).ok()
}

pub fn done(data: &Path) -> bool {
    read(data).is_some_and(|s| s.version >= VERSION && s.done_at.is_some())
}

/// What [`reset`] did: the restore, and why the game folder could not be put
/// back from the old record when it could not.
#[derive(Debug, Default)]
pub struct Reset {
    pub restored: Restored,
    pub unrestored: Option<String>,
}

/// Put the game folder back as `f1mp restore` does, then start the record of
/// original game files over: the baselines move to [`RETIRED`] and the built
/// overlays are dropped. The player verifies the game files next.
///
/// When the old record can't put the folder back, a copy the store can verify
/// is reset anyway: the unfinished journals move to [`RETIRED`] with the old
/// record, Swofty's own files are removed against an empty record, and
/// verifying the game files puts back the rest. A copy no store can verify
/// keeps its record and the error is returned.
pub fn reset(data: &Path, game: &Game, other_mods: bool) -> Result<Reset> {
    let Some(lock) = SessionLock::try_acquire(data)? else {
        return Err(Error::Other(SESSION_HELD.into()));
    };
    let first = match crate::recovery::restore_locked(&lock, data, game) {
        Err(e) if Source::detect(&game.dir) == Source::Unknown => return Err(e),
        r => r,
    };
    let retired = data.join(RETIRED);
    if retired.exists() {
        with_retry(&retired, || std::fs::remove_dir_all(&retired))?;
    }
    let baseline = data.join("baseline");
    if baseline.is_dir() {
        with_retry(&baseline, || std::fs::rename(&baseline, &retired))?;
    }
    let out = match first {
        Ok(restored) => Reset {
            restored,
            unrestored: None,
        },
        Err(e) => {
            let journals = Journal::dir(data);
            if journals.is_dir() {
                std::fs::create_dir_all(&retired).map_err(crate::io_err(&retired))?;
                let to = retired.join("journal");
                with_retry(&journals, || std::fs::rename(&journals, &to))?;
            }
            Reset {
                restored: crate::recovery::restore_locked(&lock, data, game)?,
                unrestored: Some(e.to_string()),
            }
        }
    };
    let overlays = data.join("cache/overlay");
    if overlays.is_dir() {
        with_retry(&overlays, || std::fs::remove_dir_all(&overlays))?;
    }
    write_json(
        &path(data),
        &Setup {
            version: VERSION,
            other_mods,
            reset_at: crate::update::now_secs(),
            done_at: None,
            noticed: false,
        },
    )?;
    drop(lock);
    Ok(out)
}

/// Whether the launch checks this copy's game files against its store's own
/// list: a Steam copy whose depot manifests can be read. Such a copy skips
/// the verify when the player says no other mods are in the folder.
pub fn store_checks(game_dir: &Path) -> bool {
    crate::steam::find_game()
        .filter(|i| crate::copies::same_dir(&i.dir, game_dir))
        .is_some_and(|i| crate::stock::Depots::load(&i).is_ok())
}

/// The first launch after the setup builds every addon again and backs up the
/// original game files, which takes minutes, so Play shows a notice first.
pub fn first_launch(data: &Path) -> bool {
    read(data).is_some_and(|s| s.done_at.is_some() && !s.noticed)
}

pub fn record_notice(data: &Path) -> Result<()> {
    if let Some(mut setup) = read(data) {
        setup.noticed = true;
        write_json(&path(data), &setup)?;
    }
    Ok(())
}

/// The player verified the game files: modded play may start, and the old
/// record of original game files is deleted.
pub fn finish(data: &Path) -> Result<Setup> {
    let mut setup = read(data)
        .filter(|s| s.version >= VERSION)
        .ok_or_else(|| Error::Other("the setup hasn't put the game folder back yet; reset first".into()))?;
    setup.done_at = Some(crate::update::now_secs());
    write_json(&path(data), &setup)?;
    let retired = data.join(RETIRED);
    if retired.is_dir() {
        with_retry(&retired, || std::fs::remove_dir_all(&retired))?;
    }
    Ok(setup)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::baseline::Identity;

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("swofty-setup-{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&d).ok();
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn game(root: &Path) -> Game {
        let dir = root.join("F1 25");
        std::fs::create_dir_all(&dir).unwrap();
        Game {
            dir,
            user_reg: None,
            identity: Identity {
                build_id: "1".into(),
                exe_sha256: "0".repeat(64),
            },
        }
    }

    #[test]
    fn reset_moves_the_baselines_aside_and_finish_deletes_them() {
        let root = temp("reset");
        let data = root.join("data");
        let g = game(&root);
        crate::baseline::Baseline::new(&g.identity).save(&data).unwrap();
        std::fs::create_dir_all(data.join("cache/overlay/default/abc")).unwrap();
        assert!(!done(&data));

        reset(&data, &g, true).unwrap();
        assert!(!data.join("baseline").exists());
        assert!(data.join(RETIRED).join("1-000000000000/baseline.json").is_file());
        assert!(!data.join("cache/overlay").exists());
        let s = read(&data).unwrap();
        assert_eq!((s.version, s.other_mods, s.done_at), (VERSION, true, None));
        assert!(!done(&data));

        assert!(!first_launch(&data));
        finish(&data).unwrap();
        assert!(done(&data));
        assert!(!data.join(RETIRED).exists());
        assert!(first_launch(&data));
        record_notice(&data).unwrap();
        assert!(!first_launch(&data));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_reset_again_replaces_the_retired_copy() {
        let root = temp("again");
        let data = root.join("data");
        let g = game(&root);
        std::fs::create_dir_all(data.join("baseline/old")).unwrap();
        reset(&data, &g, false).unwrap();
        std::fs::create_dir_all(data.join("baseline/new")).unwrap();
        reset(&data, &g, false).unwrap();
        assert!(data.join(RETIRED).join("new").is_dir());
        assert!(!data.join(RETIRED).join("old").exists());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn finish_needs_a_reset_of_this_setup_version() {
        let root = temp("finish");
        let data = root.join("data");
        assert!(finish(&data).is_err());
        write_json(
            &path(&data),
            &Setup {
                version: VERSION - 1,
                other_mods: false,
                reset_at: 1,
                done_at: Some(2),
                noticed: false,
            },
        )
        .unwrap();
        assert!(!done(&data));
        assert!(finish(&data).is_err());
        std::fs::remove_dir_all(&root).ok();
    }

    fn broken_record(data: &Path, g: &Game) {
        let mut b = crate::baseline::Baseline::new(&g.identity);
        b.files.insert(
            "game.erp".into(),
            crate::baseline::FileRecord {
                size: 4,
                sha256: "f".repeat(64),
                source: crate::baseline::Source::FirstSight,
            },
        );
        b.save(data).unwrap();
        std::fs::write(g.dir.join("game.erp"), "mod!").unwrap();
        std::fs::write(g.dir.join(crate::install::PROXY), "proxy").unwrap();
        std::fs::create_dir_all(Journal::dir(data)).unwrap();
        std::fs::write(Journal::dir(data).join("stuck.json"), "{}").unwrap();
    }

    #[test]
    fn a_store_copy_is_reset_when_the_old_record_cannot_put_it_back() {
        let root = temp("broken-ea");
        let data = root.join("data");
        let g = game(&root);
        std::fs::create_dir_all(g.dir.join("__Installer")).unwrap();
        std::fs::write(g.dir.join("__Installer/installerdata.xml"), "<DiPManifest/>").unwrap();
        broken_record(&data, &g);

        let r = reset(&data, &g, true).unwrap();
        assert!(!Journal::dir(&data).exists(), "unfinished journals are set aside");
        assert!(data.join(RETIRED).join("journal").is_dir());
        assert!(r.unrestored.is_some());
        assert!(!data.join("baseline").exists());
        assert!(data.join(RETIRED).is_dir());
        assert!(
            !g.dir.join(crate::install::PROXY).exists(),
            "F1 Multiplayer's own files are removed"
        );
        assert_eq!(std::fs::read_to_string(g.dir.join("game.erp")).unwrap(), "mod!");
        assert!(read(&data).is_some_and(|s| s.other_mods && s.done_at.is_none()));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_copy_no_store_can_verify_keeps_its_record_when_it_cannot_be_put_back() {
        let root = temp("broken-unknown");
        let data = root.join("data");
        let g = game(&root);
        broken_record(&data, &g);
        std::fs::remove_dir_all(Journal::dir(&data)).unwrap();
        assert!(reset(&data, &g, false).is_err());
        assert!(data.join("baseline").is_dir());
        assert!(read(&data).is_none());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn reset_waits_for_a_running_session() {
        let root = temp("held");
        let data = root.join("data");
        let g = game(&root);
        std::fs::create_dir_all(data.join("baseline/kept")).unwrap();
        let _held = SessionLock::try_acquire(&data).unwrap().unwrap();
        assert!(reset(&data, &g, false).is_err());
        assert!(data.join("baseline/kept").is_dir());
        std::fs::remove_dir_all(&root).ok();
    }
}
