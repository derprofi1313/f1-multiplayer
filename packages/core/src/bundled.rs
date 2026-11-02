//! Addons that ship with F1 Multiplayer: a release carries them as `.swoftyaddon` packages in `addons/`
//! beside `bin/`, a checkout has them in `native/bundled/` after `native/build.sh`. At start the launcher
//! installs each one that isn't installed yet, switched off in every profile, and updates an installed one
//! that is older than its package. An addon the player removed stays removed: `bundled.json` in the data
//! folder records what was seeded.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use semver::Version;

use crate::manifest::{AddonManifest, MANIFEST_FILE};
use crate::store::Store;
use crate::{Error, Result, io_err};

/// Names another folder to take the packages from.
pub const BUNDLED_ENV: &str = "F1MP_BUNDLED_DIR";
pub const RELEASE_DIR: &str = "addons";
pub const PACKAGE_EXT: &str = "swoftyaddon";
/// In the data folder: addon id -> the bundled version last seeded.
pub const SEEN_FILE: &str = "bundled.json";

/// The folder holding the bundled packages: the env override, `addons/` beside `bin/` in a release, or
/// `native/bundled/` in this checkout.
pub fn bundled_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os(BUNDLED_ENV) {
        return Some(PathBuf::from(dir));
    }
    let bin = std::env::current_exe().ok().and_then(|e| Some(e.parent()?.to_path_buf()));
    [
        bin.as_ref().and_then(|b| Some(b.parent()?.join(RELEASE_DIR))),
        Some(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../native/bundled")),
    ]
    .into_iter()
    .flatten()
    .find(|d| !packages(d).is_empty())
}

/// The `.swoftyaddon` files directly in `dir`, sorted.
pub fn packages(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == PACKAGE_EXT))
        .collect();
    out.sort();
    out
}

/// The manifest at the root of a package, read without unpacking it.
pub fn package_manifest(pkg: &Path) -> Result<AddonManifest> {
    let file = std::fs::File::open(pkg).map_err(io_err(pkg))?;
    let mut zip = zip::ZipArchive::new(file)?;
    let mut entry = zip
        .by_name(MANIFEST_FILE)
        .map_err(|_| Error::Other(format!("{}: no {MANIFEST_FILE} at the root", pkg.display())))?;
    let mut text = String::new();
    entry.read_to_string(&mut text).map_err(io_err(pkg))?;
    Ok(AddonManifest::parse(&text)?)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Never seeded and not installed: install it.
    Install,
    /// Installed at an older version than the package: replace it.
    Upgrade,
    /// Installed at the package's version or newer.
    Keep,
    /// Seeded once and since removed by the player: leave it removed.
    Removed,
}

pub fn decide(seeded: Option<&Version>, installed: Option<&Version>, bundled: &Version) -> Action {
    match (seeded, installed) {
        (_, Some(v)) if v < bundled => Action::Upgrade,
        (_, Some(_)) => Action::Keep,
        (None, None) => Action::Install,
        (Some(_), None) => Action::Removed,
    }
}

/// Installs or updates the packages in `dir`; returns one note per change.
pub fn seed(store: &Store, dir: &Path) -> Result<Vec<String>> {
    let seen_path = store.root.join(SEEN_FILE);
    let mut seen: BTreeMap<String, String> = if seen_path.is_file() {
        crate::read_json(&seen_path).unwrap_or_default()
    } else {
        BTreeMap::new()
    };
    let mut notes = Vec::new();
    let mut changed = false;
    for pkg in packages(dir) {
        let m = match package_manifest(&pkg) {
            Ok(m) => m,
            Err(e) => {
                notes.push(format!("bundled addon {} skipped: {e}", pkg.display()));
                continue;
            }
        };
        if store.is_builtin(&m.id) {
            continue;
        }
        let seeded = seen.get(&m.id).and_then(|v| Version::parse(v).ok());
        let installed = installed_version(store, &m.id);
        let action = decide(seeded.as_ref(), installed.as_ref(), &m.version);
        match action {
            Action::Install | Action::Upgrade => match store.install_addon(&pkg) {
                Ok(_) => notes.push(match &installed {
                    Some(old) => format!("updated the bundled addon {} from {old} to {}", m.id, m.version),
                    None => format!("installed the bundled addon {} {} (off until you enable it)", m.id, m.version),
                }),
                Err(e) => {
                    notes.push(format!("could not install the bundled addon {} {}: {e}", m.id, m.version));
                    continue;
                }
            },
            Action::Keep | Action::Removed => {}
        }
        let record = m.version.to_string();
        if action != Action::Removed && seen.get(&m.id) != Some(&record) {
            seen.insert(m.id.clone(), record);
            changed = true;
        }
    }
    if changed {
        crate::write_json(&seen_path, &seen)?;
    }
    Ok(notes)
}

fn installed_version(store: &Store, id: &str) -> Option<Version> {
    let dir = store.addons_dir().join(id);
    dir.join(MANIFEST_FILE)
        .is_file()
        .then(|| AddonManifest::load(&dir).ok().map(|m| m.version))
        .flatten()
}

impl Store {
    /// Seeds the addons that ship with the app (see the module docs). No bundled folder: nothing to do.
    pub fn seed_bundled(&self) -> Result<Vec<String>> {
        match bundled_dir() {
            Some(dir) => seed(self, &dir),
            None => Ok(Vec::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "t.bundled";

    fn v(s: &str) -> Version {
        Version::parse(s).unwrap()
    }

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("f1mp-bundled-{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&d).ok();
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn package(dir: &Path, version: &str) -> PathBuf {
        std::fs::create_dir_all(dir).unwrap();
        for old in packages(dir) {
            std::fs::remove_file(old).unwrap();
        }
        let path = dir.join(format!("{ID}.{PACKAGE_EXT}"));
        let manifest = format!(
            r#"{{"schema_version":1,"id":"{ID}","name":"Bundled","version":"{version}","author":"t","compat":{{"game":"f1_25"}},"options":[{{"id":"on","type":"toggle","default":true,"binds":"native"}}]}}"#
        );
        let mut z = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
        let opts = zip::write::SimpleFileOptions::default();
        z.start_file(MANIFEST_FILE, opts).unwrap();
        std::io::Write::write_all(&mut z, manifest.as_bytes()).unwrap();
        z.start_file("native/plugin.dll", opts).unwrap();
        std::io::Write::write_all(&mut z, version.as_bytes()).unwrap();
        z.finish().unwrap();
        path
    }

    #[test]
    fn decide_installs_once_upgrades_older_and_respects_a_removal() {
        let b = v("0.3.0+build.1");
        assert_eq!(decide(None, None, &b), Action::Install);
        assert_eq!(decide(None, Some(&v("0.2.0")), &b), Action::Upgrade);
        assert_eq!(decide(Some(&v("0.3.0+build.1")), Some(&v("0.3.0+build.1")), &b), Action::Keep);
        assert_eq!(decide(Some(&v("0.3.0+build.1")), None, &b), Action::Removed);
        // A newer build of the same version replaces the older build.
        assert_eq!(decide(None, Some(&b), &v("0.3.0+build.2")), Action::Upgrade);
        assert_eq!(decide(None, Some(&v("0.4.0")), &b), Action::Keep);
    }

    #[test]
    fn seeding_installs_switched_off_then_upgrades_and_keeps_a_removal() {
        let root = tmp("seed");
        let store = Store::new(root.join("data"));
        let dir = root.join("bundled");

        package(&dir, "0.3.0+build.1");
        let notes = seed(&store, &dir).unwrap();
        assert_eq!(notes.len(), 1, "{notes:?}");
        assert!(notes[0].contains("installed the bundled addon t.bundled"), "{notes:?}");
        assert_eq!(installed_version(&store, ID), Some(v("0.3.0+build.1")));
        // Switched off: no profile enables it.
        let profile = store.profile(crate::store::DEFAULT_PROFILE).unwrap();
        assert!(!profile.addons.iter().any(|a| a.id == ID && a.enabled));

        // The same package again changes nothing.
        assert!(seed(&store, &dir).unwrap().is_empty());

        // A newer build replaces it.
        package(&dir, "0.3.0+build.2");
        let notes = seed(&store, &dir).unwrap();
        assert!(notes[0].contains("from 0.3.0+build.1 to 0.3.0+build.2"), "{notes:?}");

        // Removed by the player: it stays removed.
        std::fs::remove_dir_all(store.addons_dir().join(ID)).unwrap();
        assert!(seed(&store, &dir).unwrap().is_empty());
        assert_eq!(installed_version(&store, ID), None);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_broken_package_is_skipped_with_a_note() {
        let root = tmp("broken");
        let store = Store::new(root.join("data"));
        let dir = root.join("bundled");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("bad.{PACKAGE_EXT}")), b"not a zip").unwrap();
        let notes = seed(&store, &dir).unwrap();
        assert_eq!(notes.len(), 1);
        assert!(notes[0].contains("skipped"), "{notes:?}");
        std::fs::remove_dir_all(&root).ok();
    }

    /// The in-game ADDON SETTINGS page stores each text as a UI string of at most 255 bytes; a longer one
    /// stops the profile install before the game starts (it did for 0.2.0's description).
    #[test]
    fn the_bundled_addons_texts_fit_the_games_ui_strings() {
        const LIMIT: usize = 255;
        let m = AddonManifest::parse(include_str!("../../../addons/realistic-barriers/swofty.json")).unwrap();
        let panel = format!("by {}, v{}\n{}", m.author, m.version, m.description.as_deref().unwrap_or_default());
        assert!(panel.len() <= LIMIT, "description panel is {} bytes: {panel}", panel.len());
        for o in &m.options {
            // A database-bound option gets " Applies after a restart." added.
            let body = format!("{} Applies after a restart.", o.description.as_deref().unwrap_or_default());
            let label = o.label.as_deref().unwrap_or_default();
            assert!(body.len() <= LIMIT, "{}: description is {} bytes", o.id, body.len());
            assert!(label.len() <= LIMIT, "{}: label is {} bytes", o.id, label.len());
        }
    }
}
