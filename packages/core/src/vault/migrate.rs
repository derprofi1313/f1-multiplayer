//! Import of `$DATA/save-backups/` into the vault. Each backup becomes a
//! `migrated` snapshot of the Steam account its `save_dir` names. The old
//! folder stays until the vault's first prune that removes a snapshot.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{Location, Notice, NoticeKind, Vault};
use crate::Result;
use crate::baseline::sha256_file;
use crate::save_backup;

const MARKER: &str = "migrated.json";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Marker {
    /// Old backup name to `<source>/<snapshot id>`.
    #[serde(default)]
    pub backups: BTreeMap<String, String>,
    #[serde(default)]
    pub retired: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct OldNotice {
    backup: String,
    #[serde(default)]
    absent: Vec<String>,
    before: u64,
    after: u64,
}

pub fn marker_path(data: &Path) -> PathBuf {
    Vault::root(data).join(MARKER)
}

pub fn marker(data: &Path) -> Option<Marker> {
    serde_json::from_slice(&std::fs::read(marker_path(data)).ok()?).ok()
}

/// Import every backup not imported yet; returns what it did, one line each.
pub fn migrate(data: &Path) -> Result<Vec<String>> {
    let old = save_backup::backups_dir(data);
    if !old.is_dir() {
        return Ok(Vec::new());
    }
    let _lock = Vault::migrate_lock(data, true)?;
    let mut m = marker(data).unwrap_or_default();
    if !old.is_dir() || m.retired {
        return Ok(Vec::new());
    }
    let mut notes = Vec::new();
    for b in save_backup::list(data)? {
        if m.backups.contains_key(&b.name) {
            continue;
        }
        let Some(loc) = Location::steam(&b.manifest.save_dir) else {
            notes.push(format!(
                "saves: kept save-backups/{} as it is: its save folder {} is not a Steam account's",
                b.name,
                b.manifest.save_dir.display()
            ));
            continue;
        };
        match Vault::of(data, &loc).and_then(|v| v.import(&b, &loc)) {
            Ok(id) => {
                notes.push(format!(
                    "saves: moved backup {} into the career vault as {}/{id}",
                    b.name, loc.source
                ));
                m.backups.insert(b.name.clone(), format!("{}/{id}", loc.source));
            }
            Err(e) => notes.push(format!(
                "saves: could not move backup {} into the vault: {e}",
                b.name
            )),
        }
    }
    crate::write_json(&marker_path(data), &m)?;
    let old_notice = data.join(save_backup::NOTICE);
    if let Some(n) = std::fs::read(&old_notice)
        .ok()
        .and_then(|d| serde_json::from_slice::<OldNotice>(&d).ok())
        && let Some((source, id)) = m.backups.get(&n.backup).and_then(|s| s.split_once('/'))
    {
        let kept_at = Vault::open(data, source)
            .and_then(|v| v.find(id))
            .map_or(0, |s| s.at);
        Notice {
            kind: NoticeKind::Shrank,
            source: source.to_owned(),
            snapshot: id.to_owned(),
            kept_at,
            before: n.before,
            after: n.after,
            absent: n.absent,
            at: super::now(),
        }
        .write(data)?;
        std::fs::remove_file(&old_notice).ok();
    }
    Ok(notes)
}

/// After a prune that removed a snapshot: delete the `save-backups/` folders
/// the marker names, each only once its snapshot and every blob it needs are
/// in the vault with the right hash. Folders the marker doesn't name stay, and
/// so does `save-backups/` while anything is left in it.
pub fn retire(data: &Path) {
    let Ok(Some(_lock)) = Vault::migrate_lock(data, false) else {
        return;
    };
    let Some(mut m) = marker(data) else {
        return;
    };
    if m.retired {
        return;
    }
    let old = save_backup::backups_dir(data);
    for (name, target) in &m.backups {
        let dir = old.join(name);
        if dir.is_dir() && !in_vault(data, target) {
            return;
        }
    }
    for name in m.backups.keys() {
        let dir = old.join(name);
        if dir.exists() && std::fs::remove_dir_all(&dir).is_err() {
            return;
        }
    }
    std::fs::remove_dir(&old).ok();
    if !old.exists() {
        m.retired = true;
        crate::write_json(&marker_path(data), &m).ok();
        std::fs::remove_file(data.join(save_backup::NOTICE)).ok();
    }
}

fn in_vault(data: &Path, target: &str) -> bool {
    let Some((source, id)) = target.split_once('/') else {
        return false;
    };
    let Ok(v) = Vault::open(data, source) else {
        return false;
    };
    let Ok(s) = v.find(id) else {
        return false;
    };
    s.files.iter().all(|f| {
        let blob = v.blob(&f.sha256);
        blob.is_file() && sha256_file(&blob).is_ok_and(|h| h == f.sha256)
    })
}
