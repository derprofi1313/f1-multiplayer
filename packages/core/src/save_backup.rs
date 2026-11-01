//! The profile save backups in `$DATA/save-backups/` from before the career
//! vault. The vault imports them (`vault::migrate`) and removes the folder at
//! its first prune; `take` stays as the copy-and-check the vault reuses.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::baseline::sha256_file;
use crate::{Error, Result, io_err};

pub const DIR: &str = "save-backups";
pub const KEEP: usize = 10;
pub const PREFIX: &str = "savegame@profile#";
pub const MANIFEST: &str = "manifest.json";
pub const NOTICE: &str = "state/save-notice.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedFile {
    pub name: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub created: String,
    pub profile: String,
    pub reason: String,
    #[serde(default)]
    pub absent: Vec<String>,
    pub save_dir: PathBuf,
    pub files: Vec<SavedFile>,
}

impl Manifest {
    pub fn total(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backup {
    pub name: String,
    pub dir: PathBuf,
    pub manifest: Manifest,
}

/// `YYYYMMDD-HHMMSS` in UTC, the name prefix of a backup.
pub fn stamp(secs: i64) -> String {
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}{m:02}{d:02}-{:02}{:02}{:02}",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

pub fn now_stamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    stamp(secs)
}

/// The seconds a [`stamp`] names.
pub fn parse_stamp(s: &str) -> Option<i64> {
    let (date, time) = s.get(..15)?.split_once('-')?;
    if date.len() != 8 || time.len() != 6 || !date.chars().chain(time.chars()).all(|c| c.is_ascii_digit()) {
        return None;
    }
    let n = |r: std::ops::Range<usize>, of: &str| of[r].parse::<i64>().ok();
    let (y, m, d) = (n(0..4, date)?, n(4..6, date)?, n(6..8, date)?);
    let (hh, mm, ss) = (n(0..2, time)?, n(2..4, time)?, n(4..6, time)?);
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) || hh > 23 || mm > 59 || ss > 60 {
        return None;
    }
    let y = y - i64::from(m <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Some(days * 86_400 + hh * 3600 + mm * 60 + ss)
}

pub fn backups_dir(root: &Path) -> PathBuf {
    root.join(DIR)
}

/// Installed addons with `saves.affects` that the profile leaves out: every one
/// in the vanilla profile, otherwise those neither enabled nor always on.
pub fn absent<'a>(
    installed: impl IntoIterator<Item = (&'a str, bool, bool)>,
    enabled: &[&str],
    vanilla: bool,
) -> Vec<String> {
    let mut out: Vec<String> = installed
        .into_iter()
        .filter(|&(id, affects, locked)| affects && (vanilla || !(locked || enabled.contains(&id))))
        .map(|(id, _, _)| id.to_owned())
        .collect();
    out.sort();
    out.dedup();
    out
}

/// `absent` for a profile and the addons installed in the store.
pub fn absent_in(store: &crate::store::Store, profile: &crate::profile::Profile) -> Result<Vec<String>> {
    let registry = store.registry()?;
    let enabled: Vec<&str> = profile.enabled().collect();
    Ok(absent(
        registry
            .iter()
            .map(|e| (e.id(), e.addon.manifest().saves.affects, e.locked)),
        &enabled,
        profile.is_vanilla(),
    ))
}

/// The profile save's files in `save_dir`, by name.
pub fn profile_files(save_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(save_dir)
        .map_err(io_err(save_dir))?
        .flatten()
        .filter(|e| e.file_name().to_str().is_some_and(|n| n.starts_with(PREFIX)))
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .map(|e| e.path())
        .collect();
    out.sort();
    Ok(out)
}

fn record(path: &Path) -> Result<SavedFile> {
    let size = std::fs::metadata(path).map_err(io_err(path))?.len();
    Ok(SavedFile {
        name: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        size,
        sha256: sha256_file(path)?,
    })
}

fn safe(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Copies the profile save into `$DATA/save-backups/<stamp>-<profile>/` with a
/// manifest, then keeps the newest `KEEP` backups. `None` when there is no
/// profile save to copy.
pub fn take(
    root: &Path,
    save_dir: &Path,
    profile: &str,
    absent: &[String],
    stamp: &str,
    reason: &str,
) -> Result<Option<Backup>> {
    let files = profile_files(save_dir)?;
    if files.is_empty() {
        return Ok(None);
    }
    let base = backups_dir(root);
    let mut name = format!("{}-{}", safe(stamp), safe(profile));
    let mut n = 2;
    while base.join(&name).exists() {
        name = format!("{}-{}-{n}", safe(stamp), safe(profile));
        n += 1;
    }
    let dir = base.join(&name);
    let part = base.join(format!(".{name}.part"));
    std::fs::remove_dir_all(&part).ok();
    std::fs::create_dir_all(&part).map_err(io_err(&part))?;
    let mut saved = Vec::new();
    for f in &files {
        let to = part.join(f.file_name().unwrap_or_default());
        std::fs::copy(f, &to).map_err(io_err(f))?;
        let copy = record(&to)?;
        if copy != record(f)? {
            std::fs::remove_dir_all(&part).ok();
            return Err(Error::Other(format!(
                "{} changed while it was backed up; try again with F1 25 closed",
                f.display()
            )));
        }
        saved.push(copy);
    }
    let manifest = Manifest {
        created: stamp.to_owned(),
        profile: profile.to_owned(),
        reason: reason.to_owned(),
        absent: absent.to_vec(),
        save_dir: save_dir.to_owned(),
        files: saved,
    };
    let json = serde_json::to_vec_pretty(&manifest).map_err(|e| Error::Other(e.to_string()))?;
    std::fs::write(part.join(MANIFEST), json).map_err(io_err(&part))?;
    std::fs::rename(&part, &dir).map_err(io_err(&dir))?;
    prune(root, KEEP)?;
    Ok(Some(Backup { name, dir, manifest }))
}

/// Every backup with a readable manifest, newest first.
pub fn list(root: &Path) -> Result<Vec<Backup>> {
    let base = backups_dir(root);
    let Ok(entries) = std::fs::read_dir(&base) else {
        return Ok(Vec::new());
    };
    let mut out: Vec<Backup> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_str()?.to_owned();
            if name.starts_with('.') {
                return None;
            }
            let data = std::fs::read(e.path().join(MANIFEST)).ok()?;
            let manifest = serde_json::from_slice(&data).ok()?;
            Some(Backup {
                name,
                dir: e.path(),
                manifest,
            })
        })
        .collect();
    out.sort_by(|a, b| (&b.manifest.created, &b.name).cmp(&(&a.manifest.created, &a.name)));
    Ok(out)
}

/// Removes all but the newest `keep` backups; returns the removed names.
pub fn prune(root: &Path, keep: usize) -> Result<Vec<String>> {
    let mut removed = Vec::new();
    for b in list(root)?.into_iter().skip(keep) {
        std::fs::remove_dir_all(&b.dir).map_err(io_err(&b.dir))?;
        removed.push(b.name);
    }
    Ok(removed)
}

pub fn find(root: &Path, name: &str) -> Result<Backup> {
    list(root)?.into_iter().find(|b| b.name == name).ok_or_else(|| {
        Error::Other(format!(
            "no save backup named {name}; `f1mp saves list` lists them"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("swofty-save-backup-{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&d).ok();
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn saves(root: &Path, data: &[u8]) -> PathBuf {
        let dir = root.join("remote");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("savegame@profile#NXDSMWW.BWW"), data).unwrap();
        std::fs::write(dir.join("savegame@profile#QKRHMYXE"), [7u8; 416]).unwrap();
        std::fs::write(dir.join("savegame@highlights001#ABC"), b"replay").unwrap();
        dir
    }

    #[test]
    fn stamps_in_utc() {
        assert_eq!(stamp(0), "19700101-000000");
        assert_eq!(stamp(1_793_062_123), "20261027-004843");
        for secs in [0, 951_782_400, 1_709_164_799, 1_793_062_123, 4_102_444_800] {
            assert_eq!(parse_stamp(&stamp(secs)), Some(secs), "{secs}");
        }
        assert_eq!(parse_stamp("20261027-004843-default"), Some(1_793_062_123));
        assert_eq!(parse_stamp("2026102-004843"), None);
        assert_eq!(parse_stamp("20261327-004843"), None);
    }

    #[test]
    fn absent_lists_left_out_addons_that_affect_saves() {
        let installed = [
            ("swofty.progressive", true, false),
            ("swofty.drivers", true, false),
            ("swofty.look", false, false),
            ("swofty.core", true, true),
        ];
        assert_eq!(
            absent(installed, &["swofty.drivers"], false),
            ["swofty.progressive"]
        );
        assert!(absent(installed, &["swofty.drivers", "swofty.progressive"], false).is_empty());
        assert_eq!(
            absent(installed, &[], true),
            ["swofty.core", "swofty.drivers", "swofty.progressive"]
        );
    }

    #[test]
    fn takes_only_profile_files_with_a_manifest() {
        let root = tmp("take");
        let dir = saves(&root, &[1u8; 900]);
        let b = take(
            &root,
            &dir,
            "default",
            &["swofty.progressive".into()],
            "20261027-010203",
            "session",
        )
        .unwrap()
        .unwrap();
        assert_eq!(b.name, "20261027-010203-default");
        let names: Vec<_> = b.manifest.files.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(
            names,
            ["savegame@profile#NXDSMWW.BWW", "savegame@profile#QKRHMYXE"]
        );
        assert_eq!(b.manifest.total(), 900 + 416);
        assert_eq!(
            std::fs::read(b.dir.join("savegame@profile#NXDSMWW.BWW")).unwrap(),
            [1u8; 900]
        );
        assert!(!b.dir.join("savegame@highlights001#ABC").exists());
        assert_eq!(list(&root).unwrap(), std::slice::from_ref(&b));
        let again = take(&root, &dir, "default", &[], "20261027-010203", "session")
            .unwrap()
            .unwrap();
        assert_eq!(again.name, "20261027-010203-default-2");
        let empty = root.join("empty");
        std::fs::create_dir_all(&empty).unwrap();
        assert!(
            take(&root, &empty, "default", &[], "20261027-010204", "session")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn keeps_the_newest_ten() {
        let root = tmp("prune");
        let dir = saves(&root, b"data");
        for i in 0..12 {
            take(
                &root,
                &dir,
                "p/1",
                &[],
                &format!("20261027-0100{i:02}"),
                "session",
            )
            .unwrap();
        }
        let left = list(&root).unwrap();
        assert_eq!(left.len(), KEEP);
        assert_eq!(left[0].name, "20261027-010011-p_1");
        assert_eq!(left[KEEP - 1].name, "20261027-010002-p_1");
    }
}
