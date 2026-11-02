//! "Start anyway": files of other mods in the game folder that the player chose to keep.
//!
//! The launcher stops a launch on files it can't account for: game files that aren't Steam's copy
//! (`stock::Report`), and files in the way of the profile's build that F1 Multiplayer didn't put there
//! (`recovery::Blocked`). Both are often another platform's mods, copied in by hand. When the player
//! presses Start anyway, those files are accepted ([`accept`]), for this game folder and these exact
//! files, and later launches no longer stop on them:
//! - an edited game file is taken as it is: the session's backed-up original is the edited file, so it
//!   is the file that comes back when the session ends;
//! - a file in the way of the build is moved aside for the session ([`set_aside_for_session`]) and put
//!   back when the session's game files are rolled back ([`put_back`]), so the other mod is there again
//!   after playing. Files that aren't in the build's way are never touched.
//!
//! Nothing is deleted. A file that changed since it was accepted stops the launch again.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::install::Conflict;
use crate::{Result, io_err, read_json, write_json};

pub const ACCEPTED_FILE: &str = "state/accepted-foreign.json";
pub const ASIDE_FILE: &str = "state/session-aside.json";

/// The files the player chose to keep, for one game folder.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Accepted {
    pub game: PathBuf,
    /// Game files that aren't Steam's copy, by path.
    #[serde(default)]
    pub edited: BTreeSet<String>,
    /// Files in the build's way, by path, with the SHA-256 they had when accepted (`None`: any).
    #[serde(default)]
    pub extra: BTreeMap<String, Option<String>>,
}

impl Accepted {
    pub fn path(data: &Path) -> PathBuf {
        data.join(ACCEPTED_FILE)
    }

    /// What was accepted for `game`; nothing for another folder.
    pub fn for_game(data: &Path, game: &Path) -> Self {
        read_json::<Self>(&Self::path(data))
            .ok()
            .filter(|a| same_dir(&a.game, game))
            .unwrap_or_else(|| Self {
                game: game.to_path_buf(),
                ..Self::default()
            })
    }

    /// Whether every game file in `foreign` was accepted (and there is at least one).
    pub fn covers_edited(&self, foreign: &[String]) -> bool {
        !foreign.is_empty() && foreign.iter().all(|f| self.edited.contains(&norm(f)))
    }

    /// Whether every file in the way that isn't provably ours was accepted, with the bytes it has now.
    pub fn covers_extra(&self, conflicts: &[Conflict]) -> bool {
        let theirs: Vec<&Conflict> = conflicts.iter().filter(|c| !c.owned).collect();
        !theirs.is_empty()
            && theirs.iter().all(|c| match self.extra.get(&norm(&c.path)) {
                Some(None) => true,
                Some(Some(sha)) => c.sha256.as_deref().is_some_and(|now| now.eq_ignore_ascii_case(sha)),
                // An accepted edited game file the build replaces, whose Steam original is on record.
                None => c.replaced && self.edited.contains(&norm(&c.path)),
            })
    }
}

/// Start anyway: keep these files for `game` from now on, and drop the records that stopped the
/// last launch so the launcher's warning goes away.
pub fn accept(data: &Path, game: &Path, edited: &[String], extra: &[(String, Option<String>)]) -> Result<Accepted> {
    let mut a = Accepted::for_game(data, game);
    a.game = game.to_path_buf();
    a.edited.extend(edited.iter().map(|f| norm(f)));
    for (path, sha) in extra {
        a.extra.insert(norm(path), sha.clone());
    }
    write_json(&Accepted::path(data), &a)?;
    crate::recovery::Blocked::clear(data)?;
    crate::stock::Report::clear(data)?;
    Ok(a)
}

/// The files moved aside for the running session, to put back when it ends.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Aside {
    pub game: PathBuf,
    pub files: Vec<AsideFile>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AsideFile {
    /// The path in the game folder.
    pub path: String,
    /// Where it waits meanwhile.
    pub aside: PathBuf,
}

impl Aside {
    pub fn path(data: &Path) -> PathBuf {
        data.join(ASIDE_FILE)
    }

    pub fn read(data: &Path) -> Option<Self> {
        read_json(&Self::path(data)).ok()
    }
}

/// Moves the accepted files in the build's way (`conflicts` that aren't ours) out of the game folder
/// for this session, recording them so [`put_back`] returns them. Returns a note per file.
pub fn set_aside_for_session(data: &Path, game: &Path, conflicts: &[Conflict]) -> Result<Vec<String>> {
    let stamp = format!("{}-session", crate::catalog::now_secs());
    let mut record = Aside::read(data).filter(|a| same_dir(&a.game, game)).unwrap_or(Aside {
        game: game.to_path_buf(),
        files: Vec::new(),
    });
    let mut notes = Vec::new();
    for c in conflicts.iter().filter(|c| !c.owned) {
        let from = game.join(&c.path);
        if !from.is_file() {
            continue;
        }
        let to = crate::recovery::set_aside_dir(data).join(&stamp).join(&c.path);
        if let Some(dir) = to.parent() {
            std::fs::create_dir_all(dir).map_err(io_err(dir))?;
        }
        crate::move_file(&from, &to)?;
        record.files.push(AsideFile {
            path: c.path.clone(),
            aside: to,
        });
        // Save after every move, so a crash never loses track of a file.
        write_json(&Aside::path(data), &record)?;
        notes.push(format!(
            "start anyway: {} (another mod's file) waits outside the game folder during this session and goes back afterwards",
            c.path
        ));
    }
    Ok(notes)
}

/// Puts the files [`set_aside_for_session`] moved back into `game`, over whatever the rolled-back
/// session left there (the game's original, for an edited game file). Call it after the session's
/// game files are rolled back. A file that can't go back yet stays recorded for the next call.
pub fn put_back(data: &Path, game: &Path) -> Result<Vec<String>> {
    let Some(mut record) = Aside::read(data) else {
        return Ok(Vec::new());
    };
    if !same_dir(&record.game, game) {
        return Ok(Vec::new());
    }
    let mut notes = Vec::new();
    let mut left = Vec::new();
    for f in std::mem::take(&mut record.files) {
        if !f.aside.is_file() {
            notes.push(format!("start anyway: {} was no longer waiting in {}; skipped", f.path, f.aside.display()));
            continue;
        }
        let to = game.join(&f.path);
        let back = (|| -> Result<()> {
            if let Some(dir) = to.parent() {
                std::fs::create_dir_all(dir).map_err(io_err(dir))?;
            }
            if to.is_file() {
                crate::with_retry(&to, || std::fs::remove_file(&to))?;
            }
            crate::move_file(&f.aside, &to)
        })();
        match back {
            Ok(()) => notes.push(format!("start anyway: put {} (another mod's file) back", f.path)),
            Err(e) => {
                notes.push(format!("start anyway: {} could not go back yet ({e}); tried again later", f.path));
                left.push(f);
            }
        }
    }
    record.files = left;
    if record.files.is_empty() {
        let p = Aside::path(data);
        match std::fs::remove_file(&p) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(io_err(&p)(e)),
            _ => {}
        }
    } else {
        write_json(&Aside::path(data), &record)?;
    }
    Ok(notes)
}

fn norm(path: &str) -> String {
    path.replace('\\', "/")
}

fn same_dir(a: &Path, b: &Path) -> bool {
    let canon = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let (a, b) = (canon(a), canon(b));
    if cfg!(windows) {
        a.to_string_lossy().eq_ignore_ascii_case(&b.to_string_lossy())
    } else {
        a == b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> (PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!("f1mp-foreign-{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&root).ok();
        let (data, game) = (root.join("data"), root.join("game"));
        std::fs::create_dir_all(&data).unwrap();
        std::fs::create_dir_all(&game).unwrap();
        (data, game)
    }

    fn conflict(path: &str, sha: &str, owned: bool, replaced: bool) -> Conflict {
        Conflict {
            path: path.into(),
            sha256: Some(sha.into()),
            owned,
            replaced,
        }
    }

    #[test]
    fn accepted_files_cover_only_themselves_with_the_same_bytes() {
        let (data, game) = tmp("cover");
        let a = accept(
            &data,
            &game,
            &["2025_asset_groups/x.erp".into()],
            &[("version.dll".into(), Some("AB".into()))],
        )
        .unwrap();
        assert!(a.covers_edited(&["2025_asset_groups/x.erp".into()]));
        assert!(!a.covers_edited(&["2025_asset_groups/x.erp".into(), "y.erp".into()]));
        assert!(!a.covers_edited(&[]));
        assert!(a.covers_extra(&[conflict("version.dll", "ab", false, false), conflict("ours.dll", "1", true, false)]));
        // Changed since it was accepted: stops again.
        assert!(!a.covers_extra(&[conflict("version.dll", "cd", false, false)]));
        assert!(!a.covers_extra(&[conflict("other.dll", "ab", false, false)]));
        // An accepted edited game file in the build's way is covered too.
        assert!(a.covers_extra(&[conflict("2025_asset_groups/x.erp", "ee", false, true)]));
        assert!(!a.covers_extra(&[conflict("2025_asset_groups/x.erp", "ee", false, false)]));
        // Only provably-ours conflicts: nothing to accept.
        assert!(!a.covers_extra(&[conflict("ours.dll", "1", true, false)]));
        // Another game folder knows nothing.
        let elsewhere = data.parent().unwrap().join("other-game");
        assert!(Accepted::for_game(&data, &elsewhere).extra.is_empty());
        // Merges, and keeps what was accepted before.
        let a = accept(&data, &game, &["z.erp".into()], &[]).unwrap();
        assert!(a.covers_edited(&["2025_asset_groups/x.erp".into(), "z.erp".into()]));
        assert!(a.extra.contains_key("version.dll"));
    }

    #[test]
    fn files_in_the_way_wait_outside_the_session_and_come_back() {
        let (data, game) = tmp("aside");
        std::fs::create_dir_all(game.join("mods")).unwrap();
        std::fs::write(game.join("version.dll"), "their loader").unwrap();
        std::fs::write(game.join("mods/track.erp"), "their track").unwrap();
        std::fs::write(game.join("ours.dll"), "ours").unwrap();
        let found = [
            conflict("version.dll", "a", false, false),
            conflict("mods/track.erp", "b", false, true),
            conflict("ours.dll", "c", true, false),
        ];
        let notes = set_aside_for_session(&data, &game, &found).unwrap();
        assert_eq!(notes.len(), 2, "{notes:?}");
        assert!(!game.join("version.dll").exists());
        assert!(!game.join("mods/track.erp").exists());
        assert!(game.join("ours.dll").exists(), "provably-ours files are not ours to move here");
        assert_eq!(Aside::read(&data).unwrap().files.len(), 2);

        // The session wrote its own files there; the rollback put the game's original back.
        std::fs::write(game.join("version.dll"), "f1mp loader").unwrap();
        std::fs::write(game.join("mods/track.erp"), "the game's original").unwrap();
        let notes = put_back(&data, &game).unwrap();
        assert_eq!(notes.len(), 2, "{notes:?}");
        assert_eq!(std::fs::read_to_string(game.join("version.dll")).unwrap(), "their loader");
        assert_eq!(std::fs::read_to_string(game.join("mods/track.erp")).unwrap(), "their track");
        assert!(Aside::read(&data).is_none());
        // Nothing left: a second call does nothing.
        assert!(put_back(&data, &game).unwrap().is_empty());
    }

    #[test]
    fn accepting_clears_the_records_that_stopped_the_launch() {
        let (data, game) = tmp("clear");
        crate::stock::Report::new(vec!["x.erp".into()], &game).write(&data).unwrap();
        crate::recovery::Blocked::record(&data, &game, &[conflict("version.dll", "a", false, false)]).unwrap();
        accept(&data, &game, &["x.erp".into()], &[("version.dll".into(), Some("a".into()))]).unwrap();
        assert!(crate::stock::Report::read(&data).is_none());
        assert!(crate::recovery::Blocked::read(&data).is_none());
    }
}
