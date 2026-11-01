//! The launcher's side of the vault: a check at launcher start, and a
//! snapshot when it sees an F1 25 it didn't start (a Steam or EA app Play)
//! and again once that game closes.

use std::path::Path;

use super::{Checked, Location, Meta, Reason, Taken, Vault, migrate, now};

/// What a check did, for the launcher log.
pub fn describe(c: &Checked) -> Vec<String> {
    let mut out = Vec::new();
    match &c.taken {
        Taken::New(s) => out.push(format!(
            "saves: snapshot {} ({}, {} bytes)",
            s.id,
            s.reason.text(),
            s.total()
        )),
        Taken::Unchanged(s) => out.push(format!(
            "saves: the profile save is unchanged since snapshot {}",
            s.id
        )),
        Taken::NoSave => out.push("saves: no profile save to snapshot yet".into()),
    }
    if let Some(ch) = &c.outside {
        out.push(format!(
            "saves: the profile save changed outside the launcher since snapshot {} ({} -> {} bytes); that \
             snapshot is pinned",
            ch.from.id,
            ch.from.total(),
            ch.to.total()
        ));
    }
    if let Some(n) = &c.notice {
        out.push(format!("saves: {}", n.text()));
    }
    out
}

/// Move `save-backups/` in, then snapshot the save of the copy Play uses.
/// The caller holds the session lock and has checked that F1 25 is closed.
pub fn at_launcher_start(data: &Path) -> Vec<String> {
    let mut notes = migrate::migrate(data).unwrap_or_else(|e| {
        vec![format!(
            "saves: could not move save-backups into the career vault: {e}"
        )]
    });
    let Some(loc) = Location::active(data) else {
        return notes;
    };
    match Vault::of(data, &loc).and_then(|v| v.check(&loc, Meta::new(Reason::LauncherStart), now())) {
        Ok(c) => notes.extend(describe(&c)),
        Err(e) => notes.push(format!("saves: could not snapshot the profile save: {e}")),
    }
    notes
}

/// Follows F1 25 processes the launcher sees between probes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Watch {
    foreign: Option<u32>,
    ours: Option<u32>,
}

impl Watch {
    /// `game` is the running F1 25's pid, `session` whether a launcher
    /// session (the wrapper) holds the session lock; it's asked once per
    /// new game process.
    pub fn tick(&mut self, data: &Path, game: Option<u32>, session: impl FnOnce() -> bool) -> Vec<String> {
        match (game, self.foreign) {
            (Some(pid), Some(seen)) if pid == seen => Vec::new(),
            (Some(pid), _) if self.ours == Some(pid) => Vec::new(),
            (Some(pid), _) if session() => {
                self.ours = Some(pid);
                Vec::new()
            }
            (Some(pid), _) => {
                self.foreign = Some(pid);
                self.check(data, Meta::new(Reason::ForeignLaunch).pin("foreign-launch"))
            }
            (None, Some(_)) => {
                self.foreign = None;
                self.check(data, Meta::new(Reason::ForeignExit))
            }
            (None, None) => {
                self.ours = None;
                Vec::new()
            }
        }
    }

    fn check(&self, data: &Path, meta: Meta) -> Vec<String> {
        let Some(loc) = Location::active(data) else {
            return Vec::new();
        };
        match Vault::of(data, &loc).and_then(|v| v.check(&loc, meta, now())) {
            Ok(c) => describe(&c),
            Err(e) => vec![format!("saves: could not snapshot the profile save: {e}")],
        }
    }
}
