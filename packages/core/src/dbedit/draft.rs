//! The DB Editor's draft: the changes being edited, bound to a new addon or to
//! an installed DB Editor addon, autosaved to `<data>/dbedit/draft.json`.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use super::package::{self, Package};
use super::{Catalog, Edit, Edits, RowCopy};
use crate::manifest::Category;
use crate::store::Store;
use crate::{Error, Result};

pub const FILE: &str = "draft.json";
pub const FORMAT: &str = "swofty-db-draft";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Draft {
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub format_version: u32,
    /// The DB Editor addon the draft edits; `None` for a new addon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub addon: Option<String>,
    /// A name for Save as Addon, such as the file an import came from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The last author typed, kept across drafts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// The bound addon's `dbedit.json` time when `base` was read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stamp: Option<SystemTime>,
    /// The bound addon's saved edits.
    #[serde(default)]
    pub base: Vec<Edit>,
    /// The bound addon's saved copied rows.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub base_rows: Vec<RowCopy>,
    #[serde(default)]
    pub edits: Edits,
}

/// What Save as Addon asks for.
#[derive(Debug, Clone, PartialEq)]
pub struct Details {
    pub name: String,
    pub author: String,
    pub category: Option<Category>,
    pub description: String,
}

pub fn path(store: &Store) -> PathBuf {
    store.root.join("dbedit").join(FILE)
}

/// The draft file's hash, which tells another window's write from this one's.
pub type Stamp = Option<[u8; 32]>;

pub fn stamp(store: &Store) -> Stamp {
    use sha2::Digest;
    Some(sha2::Sha256::digest(std::fs::read(path(store)).ok()?).into())
}

/// The draft file as one launcher window last read or wrote it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DraftFile {
    pub seen: Stamp,
    /// Why a damaged draft could not be set aside; nothing is written over it.
    pub stuck: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Opened {
    Read(Draft),
    /// The file did not read and was moved to `kept`.
    SetAside {
        error: String,
        kept: PathBuf,
    },
    /// The file did not read and could not be moved.
    Stuck {
        error: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Written {
    Saved,
    /// Another window wrote the file since this one read or wrote it; it was left as it is.
    Newer,
}

impl DraftFile {
    pub fn open(&mut self, store: &Store) -> Opened {
        let error = match Draft::load(store) {
            Ok(d) => {
                self.seen = stamp(store);
                self.stuck = None;
                return Opened::Read(d);
            }
            Err(e) => e.to_string(),
        };
        match Draft::set_aside(store) {
            Ok(kept) => {
                self.seen = None;
                self.stuck = None;
                Opened::SetAside { error, kept }
            }
            Err(why) => {
                let error = format!("{error}\nIt could not be set aside: {why}");
                self.stuck = Some(error.clone());
                Opened::Stuck { error }
            }
        }
    }

    pub fn is_newer(&self, store: &Store) -> bool {
        self.stuck.is_none() && stamp(store) != self.seen
    }

    /// Writes `draft` unless the file is newer than this window's copy or is
    /// a damaged draft that still cannot be set aside.
    pub fn write(&mut self, store: &Store, draft: &Draft) -> Result<Written> {
        if let Some(why) = &self.stuck {
            if path(store).exists() && Draft::load(store).is_err() {
                Draft::set_aside(store)
                    .map_err(|_| Error::Other(format!("the damaged draft file is in the way: {why}")))?;
            }
            self.stuck = None;
            self.seen = None;
        }
        if self.is_newer(store) {
            return Ok(Written::Newer);
        }
        draft.clone().save(store)?;
        self.seen = stamp(store);
        Ok(Written::Saved)
    }

    /// Takes the file as it is now as this window's copy, so the next write replaces it.
    pub fn adopt(&mut self, store: &Store) {
        self.seen = stamp(store);
    }
}

fn same(a: &Edit, b: &Edit) -> bool {
    a.tag == b.tag && a.value == b.value
}

impl Draft {
    /// The saved draft; a missing file is an empty new draft.
    pub fn load(store: &Store) -> Result<Draft> {
        let file = path(store);
        if !file.is_file() {
            return Ok(Draft::default());
        }
        let mut d: Draft = crate::read_json(&file)?;
        if d.format != FORMAT {
            return Err(Error::Other(format!(
                "{}: format {:?} is not {FORMAT}",
                file.display(),
                d.format
            )));
        }
        d.edits.schema_version = 1;
        Ok(d)
    }

    /// Moves a draft file that does not read to `draft.damaged-<unix seconds>.json`.
    pub fn set_aside(store: &Store) -> Result<PathBuf> {
        let from = path(store);
        let secs = crate::conflicts::now() as u64;
        let to = (0..)
            .map(|n| match n {
                0 => from.with_file_name(format!("draft.damaged-{secs}.json")),
                n => from.with_file_name(format!("draft.damaged-{secs}-{n}.json")),
            })
            .find(|p| !p.exists())
            .expect("a free name");
        std::fs::rename(&from, &to).map_err(crate::io_err(&from))?;
        Ok(to)
    }

    pub fn save(&mut self, store: &Store) -> Result<()> {
        self.format = FORMAT.into();
        self.format_version = 1;
        self.edits.schema_version = 1;
        crate::write_json_shared(&path(store), self)
    }

    /// The draft of the installed addon `id`, keeping the last author.
    pub fn open(&mut self, store: &Store, id: &str) -> Result<()> {
        let stamp = package::modified(store, id);
        let p = Package::open(store, id)?;
        self.clear();
        self.addon = Some(id.to_owned());
        self.stamp = stamp;
        self.base = p.edits.edits.clone();
        self.base_rows = p.edits.rows.clone();
        self.edits = p.edits;
        Ok(())
    }

    /// An empty new draft, keeping the last author.
    pub fn clear(&mut self) {
        *self = Draft {
            author: self.author.take(),
            ..Draft::default()
        };
        self.edits.schema_version = 1;
    }

    /// How many keys and copied rows differ from the bound addon's saved ones.
    pub fn unsaved(&self) -> usize {
        let rows = self
            .edits
            .rows
            .iter()
            .filter(|r| !self.base_rows.contains(r))
            .count()
            + self
                .base_rows
                .iter()
                .filter(|r| !self.edits.rows.contains(r))
                .count();
        rows + self.unsaved_keys()
    }

    fn unsaved_keys(&self) -> usize {
        let base: HashMap<u64, &Edit> = self.base.iter().map(|e| (e.key, e)).collect();
        let mine: HashSet<u64> = self.edits.edits.iter().map(|e| e.key).collect();
        let changed = self
            .edits
            .edits
            .iter()
            .filter(|e| base.get(&e.key).is_none_or(|b| !same(b, e)))
            .count();
        changed + base.keys().filter(|k| !mine.contains(k)).count()
    }

    pub fn is_dirty(&self) -> bool {
        self.unsaved() > 0
    }

    /// Puts the edits back to the bound addon's saved ones, or none for a new addon.
    pub fn discard(&mut self) {
        self.edits.edits = self.base.clone();
        self.edits.rows = self.base_rows.clone();
    }

    /// Takes `base` as the addon's saved edits and rows and keeps this draft's
    /// unsaved changes on top of them.
    pub fn rebase(&mut self, base: Edits, stamp: Option<SystemTime>) {
        let same = |a: &RowCopy, b: &RowCopy| a.is(&b.table, &b.field, &b.to);
        let added: Vec<RowCopy> = self
            .edits
            .rows
            .iter()
            .filter(|r| !self.base_rows.contains(r))
            .cloned()
            .collect();
        let removed: Vec<RowCopy> = self
            .base_rows
            .iter()
            .filter(|r| !self.edits.rows.iter().any(|m| same(m, r)))
            .cloned()
            .collect();
        let mut rows = base.rows.clone();
        rows.retain(|r| !removed.iter().any(|x| same(x, r)));
        for a in added {
            match rows.iter_mut().find(|r| same(r, &a)) {
                Some(slot) => *slot = a,
                None => rows.push(a),
            }
        }
        self.rebase_keys(base.edits, stamp);
        self.base_rows = base.rows;
        self.edits.rows = rows;
    }

    fn rebase_keys(&mut self, base: Vec<Edit>, stamp: Option<SystemTime>) {
        let old: HashMap<u64, &Edit> = self.base.iter().map(|e| (e.key, e)).collect();
        let mine: HashSet<u64> = self.edits.edits.iter().map(|e| e.key).collect();
        let reverted: HashSet<u64> = old.keys().filter(|k| !mine.contains(k)).copied().collect();
        let changed: Vec<Edit> = self
            .edits
            .edits
            .iter()
            .filter(|e| old.get(&e.key).is_none_or(|b| !same(b, e)))
            .cloned()
            .collect();
        let mut out = base.clone();
        out.retain(|e| !reverted.contains(&e.key));
        for c in changed {
            match out.iter_mut().find(|e| e.key == c.key) {
                Some(slot) => *slot = c,
                None => out.push(c),
            }
        }
        self.base = base;
        self.stamp = stamp;
        self.edits.edits = out;
    }

    /// Writes the draft as a new DB Editor addon and clears the draft.
    pub fn save_as_addon(&mut self, store: &Store, catalog: &Catalog, d: &Details) -> Result<Package> {
        let mut p = Package::create(store, &d.name)?;
        let made = p
            .set_details(&d.name, "1.0.0", &d.author, &d.description)
            .and_then(|()| {
                p.set_category(d.category);
                p.edits = self.edits.clone();
                p.save(store, catalog)
            });
        if let Err(e) = made {
            Package::delete(store, &p.id).ok();
            return Err(e);
        }
        if !d.author.trim().is_empty() {
            self.author = Some(d.author.trim().to_owned());
        }
        self.clear();
        Ok(p)
    }

    /// Writes the draft into its addon, on top of any change saved there since
    /// the draft read it, and clears the draft.
    pub fn save_addon(&mut self, store: &Store, catalog: &Catalog) -> Result<Package> {
        let id = self
            .addon
            .clone()
            .ok_or_else(|| Error::Other("this draft is not an addon yet; use Save as Addon".into()))?;
        let stamp = package::modified(store, &id);
        let mut p = Package::open(store, &id)?;
        if stamp != self.stamp {
            self.rebase(p.edits.clone(), stamp);
        }
        p.edits = self.edits.clone();
        p.save(store, catalog)?;
        self.clear();
        Ok(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dbedit::catalog::tests::sample;
    use swofty_formats::hash::fnv1a64;

    const NUM: &str = "entrants.drivers.driver_0.car_number";
    const NUM1: &str = "entrants.drivers.driver_1.car_number";
    const TEAM: &str = "entrants.teams.team_0.prestige";

    fn store(name: &str) -> Store {
        let root = std::env::temp_dir().join(format!("swofty-dbdraft-{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&root).ok();
        Store::new(root)
    }

    fn details(name: &str) -> Details {
        Details {
            name: name.into(),
            author: "Max".into(),
            category: Some(Category::Seasons),
            description: "Numbers".into(),
        }
    }

    #[test]
    fn saves_a_new_draft_as_an_addon_and_clears_it() {
        let s = store("new");
        let cat = Catalog::load(&sample()).unwrap();
        let mut d = Draft::load(&s).unwrap();
        assert_eq!(d, Draft::default());
        d.edits.set(&cat, fnv1a64(NUM), "33").unwrap();
        assert_eq!(d.unsaved(), 1);
        d.save(&s).unwrap();
        let again = Draft::load(&s).unwrap();
        assert_eq!((again.edits.len(), again.unsaved()), (1, 1));

        let p = d.save_as_addon(&s, &cat, &details("Fast grid")).unwrap();
        assert_eq!(p.id, "dbedit.fast-grid");
        assert_eq!(
            (d.addon.as_deref(), d.edits.len(), d.author.as_deref()),
            (None, 0, Some("Max"))
        );
        let m = &s.addons().unwrap()[&p.id];
        assert_eq!(m.category, Some(Category::Seasons));
        assert_eq!(
            m.origin.as_ref().map(|o| o.tool.as_str()),
            Some(package::ORIGIN_TOOL)
        );
        assert_eq!(
            (m.author.as_str(), m.description.as_deref()),
            ("Max", Some("Numbers"))
        );
        assert_eq!(Package::open(&s, &p.id).unwrap().edits.len(), 1);
        assert!(d.save_addon(&s, &cat).is_err());
        std::fs::remove_dir_all(&s.root).ok();
    }

    #[test]
    fn edits_an_addon_and_keeps_changes_saved_elsewhere() {
        let s = store("bound");
        let cat = Catalog::load(&sample()).unwrap();
        let mut d = Draft::default();
        d.edits.set(&cat, fnv1a64(NUM), "33").unwrap();
        d.edits.set(&cat, fnv1a64(TEAM), "20").unwrap();
        let id = d.save_as_addon(&s, &cat, &details("Grid")).unwrap().id;

        d.open(&s, &id).unwrap();
        assert_eq!(
            (d.addon.as_deref(), d.unsaved(), d.edits.len()),
            (Some(id.as_str()), 0, 2)
        );
        d.edits.set(&cat, fnv1a64(NUM), "44").unwrap();
        d.edits.revert(fnv1a64(TEAM));
        assert_eq!(d.unsaved(), 2);
        d.discard();
        assert_eq!(d.unsaved(), 0);
        d.edits.set(&cat, fnv1a64(NUM), "44").unwrap();
        d.edits.revert(fnv1a64(TEAM));

        std::thread::sleep(std::time::Duration::from_millis(20));
        let mut other = Package::open(&s, &id).unwrap();
        other.edits.set(&cat, fnv1a64(NUM1), "55").unwrap();
        other.edits.set(&cat, fnv1a64(NUM), "66").unwrap();
        other.save(&s, &cat).unwrap();

        d.save_addon(&s, &cat).unwrap();
        assert_eq!(d.addon, None);
        let saved = Package::open(&s, &id).unwrap().edits;
        assert_eq!(saved.value(&cat, fnv1a64(NUM)), Some("44"));
        assert_eq!(saved.value(&cat, fnv1a64(NUM1)), Some("55"));
        assert!(saved.get(fnv1a64(TEAM)).is_none());
        std::fs::remove_dir_all(&s.root).ok();
    }

    #[test]
    fn rebases_unsaved_changes_and_refuses_a_damaged_file() {
        let s = store("rebase");
        let cat = Catalog::load(&sample()).unwrap();
        let mut base = Edits::default();
        base.set(&cat, fnv1a64(NUM), "33").unwrap();
        let mut d = Draft {
            base: base.edits.clone(),
            edits: base.clone(),
            ..Draft::default()
        };
        d.rebase(Edits::default(), None);
        assert_eq!(d.edits.len(), 0);
        let mut newer = base.clone();
        newer.set(&cat, fnv1a64(NUM1), "7").unwrap();
        d.edits.set(&cat, fnv1a64(TEAM), "20").unwrap();
        d.rebase(newer.clone(), None);
        let keys: Vec<u64> = d.edits.edits.iter().map(|e| e.key).collect();
        assert_eq!(keys, [fnv1a64(NUM), fnv1a64(NUM1), fnv1a64(TEAM)]);
        assert_eq!(d.unsaved(), 1);

        std::fs::create_dir_all(path(&s).parent().unwrap()).unwrap();
        std::fs::write(path(&s), b"{\"format\": \"swofty-db-dr").unwrap();
        assert!(Draft::load(&s).is_err());
        let aside = Draft::set_aside(&s).unwrap();
        assert!(aside.is_file() && !path(&s).exists());
        assert_eq!(Draft::load(&s).unwrap(), Draft::default());
        std::fs::remove_dir_all(&s.root).ok();
    }

    #[test]
    fn counts_discards_and_rebases_copied_rows() {
        use crate::dbedit::rows::tests::{HOLDERS, sample};
        let s = store("rows");
        let cat = Catalog::load(&sample().to_bytes()).unwrap();
        let copy = |e: &mut Edits, from: Option<&str>, to: &str| {
            e.copy_row(&cat, HOLDERS, "driver", from, to).unwrap();
        };
        let mut d = Draft::default();
        copy(&mut d.edits, Some("lewis_hamilton"), "romain_grosjean");
        assert_eq!(d.unsaved(), 1);
        let id = d.save_as_addon(&s, &cat, &details("Rows")).unwrap().id;
        d.open(&s, &id).unwrap();
        assert_eq!((d.unsaved(), d.base_rows.len()), (0, 1));
        assert!(d.edits.remove_row(HOLDERS, "driver", "romain_grosjean"));
        copy(&mut d.edits, None, "nico_hulkenberg");
        assert_eq!(d.unsaved(), 2);
        d.discard();
        assert_eq!((d.unsaved(), d.edits.rows[0].to.as_str()), (0, "romain_grosjean"));

        copy(&mut d.edits, None, "nico_hulkenberg");
        let mut theirs = Package::open(&s, &id).unwrap().edits;
        assert!(theirs.remove_row(HOLDERS, "driver", "romain_grosjean"));
        copy(&mut theirs, Some("max_verstappen"), "oliver_bearman");
        d.rebase(theirs, None);
        let to: Vec<&str> = d.edits.rows.iter().map(|r| r.to.as_str()).collect();
        assert_eq!(to, ["oliver_bearman", "nico_hulkenberg"]);
        assert_eq!(d.unsaved(), 1);
        std::fs::remove_dir_all(&s.root).ok();
    }

    #[test]
    fn sets_damaged_drafts_aside_under_new_names_and_never_writes_over_one() {
        let s = store("damaged");
        let damaged = b"{\"format\": \"swofty-db-dr";
        std::fs::create_dir_all(path(&s).parent().unwrap()).unwrap();
        let mut kept = Vec::new();
        for _ in 0..2 {
            std::fs::write(path(&s), damaged).unwrap();
            let mut f = DraftFile::default();
            let Opened::SetAside { kept: k, .. } = f.open(&s) else {
                panic!("not set aside")
            };
            assert_eq!(std::fs::read(&k).unwrap(), damaged);
            kept.push(k);
        }
        assert_ne!(kept[0], kept[1]);
        assert!(kept.iter().all(|k| {
            k.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("draft.damaged-")
        }));

        std::fs::write(path(&s), damaged).unwrap();
        let mut f = DraftFile {
            stuck: Some("read-only folder".into()),
            ..DraftFile::default()
        };
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let dir = path(&s).parent().unwrap().to_owned();
            let mode = |m| std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(m)).unwrap();
            mode(0o555);
            let locked = std::fs::File::create(dir.join("probe")).is_err();
            let refused = f.write(&s, &Draft::default());
            mode(0o755);
            if locked {
                assert!(refused.is_err());
                assert_eq!(std::fs::read(path(&s)).unwrap(), damaged);
                assert!(f.stuck.is_some());
            }
        }
        assert_eq!(f.write(&s, &Draft::default()).unwrap(), Written::Saved);
        assert!(f.stuck.is_none());
        assert_eq!(Draft::load(&s).unwrap().format, FORMAT);
        std::fs::remove_dir_all(&s.root).ok();
    }

    #[test]
    fn leaves_a_draft_another_window_wrote_until_it_is_adopted() {
        let s = store("windows");
        let cat = Catalog::load(&sample()).unwrap();
        let (mut one, mut two) = (DraftFile::default(), DraftFile::default());
        let Opened::Read(mut a) = one.open(&s) else {
            panic!()
        };
        let Opened::Read(mut b) = two.open(&s) else {
            panic!()
        };
        a.edits.set(&cat, fnv1a64(NUM), "33").unwrap();
        assert_eq!(one.write(&s, &a).unwrap(), Written::Saved);
        assert!(!one.is_newer(&s) && two.is_newer(&s));

        b.edits.set(&cat, fnv1a64(TEAM), "20").unwrap();
        assert_eq!(two.write(&s, &b).unwrap(), Written::Newer);
        assert_eq!(Draft::load(&s).unwrap().edits.len(), 1);
        assert!(Draft::load(&s).unwrap().edits.get(fnv1a64(NUM)).is_some());

        let Opened::Read(theirs) = two.open(&s) else {
            panic!()
        };
        assert_eq!(theirs.edits.edits, a.edits.edits);
        two.adopt(&s);
        assert_eq!(two.write(&s, &b).unwrap(), Written::Saved);
        assert!(one.is_newer(&s));
        assert!(Draft::load(&s).unwrap().edits.get(fnv1a64(TEAM)).is_some());
        std::fs::remove_dir_all(&s.root).ok();
    }
}
