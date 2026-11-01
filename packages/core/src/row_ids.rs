//! The install's ids for the rows `add-row` gives an id of its own, in `$DATA/registry/row-ids.json`
//! next to the core's `ids.json` (docs/std/registry.md "Stat ids"). A key names an addon's row,
//! `<addon>/<table>.<row>/<label>`, and keeps its id at every build of every profile, whatever else
//! the build adds, since careers in the shared Steam save hold the ids.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use swofty_formats::gvdb::{Gvdb, instance_count_path};
use swofty_formats::hash::fnv1a64;

use crate::ops::{Claimed, DbOp, DbOps, IdFrom, RowIds, add_row_name, row_shape, stat_id_field};
use crate::stdlib::reports;
use crate::{Loaded, Result, io_err};

mod recover;

pub use recover::Recovery;

pub const FILE: &str = "row-ids.json";
/// The table name of the track rows `track.add` adds; the game reads their `statId` in a byte, 0xff = none.
const TRACK_TABLE: &str = "tracks.Track";
pub const TRACK_STAT_MAX: i64 = 254;
const LOCK: &str = "row-ids.lock";
const VERSION: u32 = 1;

/// The core's registries whose `ids.json` ids are ids of these tables' rows.
const REGISTRY_TABLES: &[(&str, &str)] = &[
    ("vehicle", "entrants.vehicles.vehicle"),
    (
        "vehicle.package",
        "entrants.vehicles.vehicle_packages.vehicle_package",
    ),
    ("content.package", "entrants.entrant_lists.entrant_list"),
    ("content.package.scenario", "game_setup.scenarios.scenario"),
    ("team.identity", "entrants.teams.team"),
    ("driver.identity", "entrants.drivers.driver"),
    ("driver.head", "entrants.animation.driver_heads.driver_head"),
];

/// The tables saves hold ids of, and the name `swofty/id_aliases.json` gives each.
const ALIAS_TABLES: &[(&str, &str)] = &[
    ("entrants.teams.team", "teams"),
    ("entrants.vehicles.vehicle", "vehicles"),
    ("entrants.drivers.driver", "drivers"),
];

pub const ALIASES_FILE: &str = "id_aliases.json";

/// The environment variable the launcher sets for `f1mp run` to the file of the host's ids for a room
/// launch ([`Overrides`]).
pub const ENV: &str = "SWOFTY_ROOM_ROW_IDS";

/// `swofty/id_aliases.json`: for each of `teams`, `vehicles` and `drivers`, the ids added rows had
/// before (1.6.10 and 1.6.11 hashed ids, ids before 1.6.10) mapped to the id they have now. The core
/// reads it at StateLoadSession to load a save made under the earlier ids.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdAliases {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub tables: BTreeMap<String, BTreeMap<i64, i64>>,
}

impl IdAliases {
    pub fn is_empty(&self) -> bool {
        self.tables.values().all(BTreeMap::is_empty)
    }

    pub fn write(&self, path: &Path) -> Result<()> {
        let file = IdAliases {
            version: VERSION,
            tables: self.tables.clone(),
        };
        crate::write_json(path, &file)
    }
}

/// The ids a room's host gave the addons' rows, by addon and then `<table>.<row>/<label>`: a joiner
/// builds one session with them, in place of the ids its own `row-ids.json` keeps, so both copies'
/// fingerprints match. They are never written to `row-ids.json` (see [`Store::lend`]) and they are part
/// of the build's cache key.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Overrides {
    pub rows: BTreeMap<String, BTreeMap<String, i64>>,
}

impl Overrides {
    pub fn is_empty(&self) -> bool {
        self.rows.values().all(BTreeMap::is_empty)
    }

    /// The overrides for the addons in `order`, as `<addon>/<table>.<row>/<label>` keys.
    fn keys(&self, order: &[String]) -> BTreeMap<String, i64> {
        order
            .iter()
            .filter_map(|addon| Some((addon, self.rows.get(addon)?)))
            .flat_map(|(addon, rows)| rows.iter().map(move |(row, id)| (format!("{addon}/{row}"), *id)))
            .collect()
    }

    /// What goes into the build's cache key for the addons in `order`; empty when none apply.
    pub fn stamp(&self, order: &[String]) -> Option<String> {
        let keys = self.keys(order);
        (!keys.is_empty()).then(|| {
            format!(
                "room row ids {};",
                serde_json::to_string(&keys).expect("serialisable")
            )
        })
    }

    pub fn read(path: &Path) -> Result<Self> {
        match crate::load_json::<Overrides>(path)? {
            Loaded::Found(o) => Ok(o),
            Loaded::Missing | Loaded::Damaged(_) => Ok(Self::default()),
        }
    }

    pub fn write(&self, path: &Path) -> Result<()> {
        crate::write_json(path, self)
    }
}

static LENT: std::sync::Mutex<Option<Overrides>> = std::sync::Mutex::new(None);

/// Makes this process's builds use `overrides` (a room launch's), or none. The `f1mp run` that starts
/// the game for a room calls it once; no other process does.
pub fn lend(overrides: Option<Overrides>) {
    *LENT.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = overrides.filter(|o| !o.is_empty());
}

pub fn lent() -> Option<Overrides> {
    LENT.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct File {
    #[serde(default)]
    version: u32,
    #[serde(default)]
    rows: BTreeMap<String, Entry>,
    /// The profiles whose last build before the store seeded it.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    seeded: BTreeSet<String>,
    /// The ids 1.6.10 and 1.6.11 gave from the rows' keys were put back once (see [`Store::recover`]).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    recovered: bool,
    /// Every id a key's row had before the one it keeps, so saves made under those ids still load.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    aliases: BTreeMap<String, BTreeSet<i64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Entry {
    id: i64,
    from: IdFrom,
}

/// One row the store keeps an id for: its table, label, id and how it got it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kept {
    pub table: String,
    pub label: String,
    pub id: i64,
    pub from: IdFrom,
}

/// The rows of `addon` the store in `data` keeps ids for, by table and label.
pub fn kept_rows(data: &Path, addon: &str) -> Result<Vec<Kept>> {
    let file: File = match crate::load_json(&path(data))? {
        Loaded::Found(f) => f,
        Loaded::Missing | Loaded::Damaged(_) => File::default(),
    };
    let prefix = format!("{addon}/");
    Ok(file
        .rows
        .into_iter()
        .filter_map(|(key, e)| {
            let (table, label) = key.strip_prefix(&prefix)?.split_once('/')?;
            Some(Kept {
                table: table.to_owned(),
                label: label.to_owned(),
                id: e.id,
                from: e.from,
            })
        })
        .collect())
}

pub fn path(data: &Path) -> PathBuf {
    reports::path(data, FILE)
}

/// Held while a build reads and writes `row-ids.json`; another build waits until it is dropped.
pub fn lock(data: &Path) -> Result<std::fs::File> {
    let dir = reports::dir(data);
    std::fs::create_dir_all(&dir).map_err(io_err(&dir))?;
    let path = dir.join(LOCK);
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .map_err(io_err(&path))?;
    file.lock().map_err(io_err(&path))?;
    Ok(file)
}

/// What a build left of its database: its overlay's database file `rel`, when the build wrote one, and
/// its `swofty/assets.json`, whose database edits the core applies in game when the build routes the
/// addons' rows there (the overlay's file then holds the tracks' rows at most, or is not written).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastBuild {
    pub db: Option<PathBuf>,
    pub edits: Option<PathBuf>,
}

/// The profile's last build in `overlays`, other than the build `current`; for a profile with none,
/// the newest of every profile's builds beside it.
pub fn last_build(overlays: &Path, current: &str, rel: &str) -> Option<LastBuild> {
    let of = |d: PathBuf| {
        let file = |p: PathBuf| {
            let at = std::fs::metadata(&p)
                .ok()
                .filter(|m| m.is_file())?
                .modified()
                .ok()?;
            Some((at, p))
        };
        let db = file(d.join(rel));
        let edits = file(d.join(crate::runtime::ASSETS_FILE));
        let at = db.iter().chain(&edits).map(|(at, _)| *at).max()?;
        Some((
            at,
            LastBuild {
                db: db.map(|(_, p)| p),
                edits: edits.map(|(_, p)| p),
            },
        ))
    };
    let newest = |dirs: Vec<PathBuf>| {
        dirs.into_iter()
            .filter_map(of)
            .max_by_key(|(at, _)| *at)
            .map(|(_, b)| b)
    };
    let subdirs = |dir: &Path| -> Vec<PathBuf> {
        std::fs::read_dir(dir)
            .map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect())
            .unwrap_or_default()
    };
    let own = subdirs(overlays)
        .into_iter()
        .filter(|d| d.file_name().is_some_and(|n| n != current))
        .collect();
    newest(own).or_else(|| {
        let profiles = subdirs(overlays.parent()?);
        newest(
            profiles
                .iter()
                .filter(|p| p.as_path() != overlays)
                .flat_map(|p| subdirs(p))
                .collect(),
        )
    })
}

#[derive(Deserialize)]
struct Assets {
    #[serde(default)]
    db_edits: Vec<Edit>,
}

#[derive(Deserialize)]
struct Edit {
    hash: String,
    value: String,
}

// The last build's database as the game loads it: the overlay's file, with the build's runtime edits over it.
struct LastDb {
    db: Option<(Gvdb, HashMap<u64, usize>)>,
    edits: HashMap<u64, String>,
}

impl LastDb {
    fn read(last: &LastBuild) -> Option<Self> {
        let db = last
            .db
            .as_ref()
            .and_then(|p| Gvdb::parse(&std::fs::read(p).ok()?).ok())
            .map(|db| {
                let index = db.records.iter().enumerate().map(|(i, r)| (r.key, i)).collect();
                (db, index)
            });
        let edits: HashMap<u64, String> = last
            .edits
            .as_ref()
            .and_then(|p| serde_json::from_slice::<Assets>(&std::fs::read(p).ok()?).ok())
            .map(|a| {
                a.db_edits
                    .into_iter()
                    .filter_map(|e| Some((u64::from_str_radix(&e.hash, 16).ok()?, e.value)))
                    .collect()
            })
            .unwrap_or_default();
        (db.is_some() || !edits.is_empty()).then_some(Self { db, edits })
    }

    fn text(&self, path: &str) -> Option<String> {
        let key = fnv1a64(path);
        if let Some(v) = self.edits.get(&key) {
            return Some(v.clone());
        }
        let (db, index) = self.db.as_ref()?;
        index.get(&key).map(|&i| db.records[i].text().into_owned())
    }
}

#[derive(Default)]
struct Seed {
    last: Option<LastBuild>,
    db: Option<Option<LastDb>>,
    stock_rows: HashMap<String, usize>,
    tables: HashMap<String, HashMap<String, Vec<i64>>>,
}

impl Seed {
    // The id of the `nth` row labelled `label` that the last build added to `table`'s rows named `row`.
    fn id(&mut self, table: &str, row: &str, label: &str, nth: usize) -> Option<i64> {
        let last = self.last.as_ref()?;
        let db = self.db.get_or_insert_with(|| LastDb::read(last)).as_ref()?;
        let name = format!("{table}.{row}");
        let skip = self.stock_rows.get(&name).copied().unwrap_or(0);
        let rows = self.tables.entry(name).or_insert_with(|| {
            let field = stat_id_field(table, row);
            let n = db
                .text(&instance_count_path(table, row))
                .and_then(|v| v.trim().parse::<usize>().ok())
                .unwrap_or(0);
            let mut out: HashMap<String, Vec<i64>> = HashMap::new();
            for i in skip..n {
                let label = db.text(&format!("{table}.{row}_{i}._metadata_"));
                let id = db
                    .text(&format!("{table}.{row}_{i}.{field}"))
                    .and_then(|v| v.trim().parse().ok());
                if let (Some(label), Some(id)) = (label, id) {
                    out.entry(label).or_default().push(id);
                }
            }
            out
        });
        rows.get(label)?.get(nth).copied()
    }
}

/// The kept ids, the ids the core's registry holds, and what this build claimed and changed.
#[derive(Default)]
pub struct Store {
    rows: BTreeMap<String, i64>,
    from: HashMap<String, IdFrom>,
    seeded: BTreeSet<String>,
    recovered: bool,
    aliases: BTreeMap<String, BTreeSet<i64>>,
    existed: bool,
    by_id: HashMap<String, BTreeMap<i64, String>>,
    pinned: HashMap<String, HashMap<i64, String>>,
    seed: Seed,
    claimed: HashSet<String>,
    building: HashSet<String>,
    added: usize,
    moved: usize,
    reused: usize,
    changed: bool,
    read_only: bool,
    lent: BTreeMap<String, i64>,
    lent_build: HashSet<String>,
}

fn addon_of(key: &str) -> &str {
    key.split('/').next().unwrap_or(key)
}

fn table_of(key: &str) -> &str {
    key.split('/').nth(1).unwrap_or_default()
}

fn row_name(key: &str) -> String {
    let (addon, rest) = key.split_once('/').unwrap_or(("", key));
    format!("{rest} of {addon}")
}

impl Store {
    /// The store in `data`, the ids the core's `ids.json` holds (in `data`, else in the game folder
    /// `game`), and, at `profile`'s first build with the store, its last build's database `last` to
    /// seed rows the store has no id for. The strings are warnings.
    pub fn load(
        data: &Path,
        game: Option<&Path>,
        profile: &str,
        last: Option<LastBuild>,
    ) -> (Self, Vec<String>) {
        let mut warnings = Vec::new();
        let mut store = Self::default();
        let file = path(data);
        match crate::load_json::<File>(&file) {
            Ok(Loaded::Found(f)) => {
                store.read_only = f.version > VERSION;
                store.seeded = f.seeded;
                store.recovered = f.recovered;
                store.aliases = f.aliases;
                store.existed = true;
                for (key, e) in f.rows {
                    store.from.insert(key.clone(), e.from);
                    store.insert(key, e.id);
                }
            }
            Ok(Loaded::Missing) => {}
            Ok(Loaded::Damaged(to)) => warnings.push(format!(
                "database: {FILE} was damaged and was moved to {}; added rows take the ids of the last build",
                to.display()
            )),
            Err(e) => {
                store.read_only = true;
                warnings.push(format!(
                    "database: {FILE} can't be read ({e}); it is left as it is"
                ));
            }
        }
        if let Some(ids) = reports::found(data, game, reports::IDS)
            .and_then(|p| std::fs::read(p).ok())
            .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        {
            store.pinned = registry_ids(&ids);
        }
        if store.seeded.insert(profile.to_owned()) {
            store.seed.last = last;
            store.changed = true;
        }
        (store, warnings)
    }

    fn insert(&mut self, key: String, id: i64) {
        let table = key.split('/').nth(1).unwrap_or_default().to_owned();
        if let Some(old) = self.rows.insert(key.clone(), id)
            && let Some(ids) = self.by_id.get_mut(&table)
            && ids.get(&old) == Some(&key)
        {
            ids.remove(&old);
        }
        if let Some(prev) = self.by_id.entry(table).or_default().insert(id, key.clone())
            && prev != key
        {
            self.rows.remove(&prev);
        }
    }

    fn put(&mut self, key: &str, id: i64, from: IdFrom) {
        match self.rows.get(key) {
            Some(&kept) if kept == id => return,
            Some(&kept) => {
                self.moved += 1;
                self.alias(key, kept);
            }
            None => self.added += 1,
        }
        self.insert(key.to_owned(), id);
        self.from.insert(key.to_owned(), from);
        self.changed = true;
    }

    fn alias(&mut self, key: &str, old: i64) {
        if self.aliases.entry(key.to_owned()).or_default().insert(old) {
            self.changed = true;
        }
    }

    /// Gives the rows of `overrides` for the addons in `order` the host's ids for this build only: the store is
    /// not written afterwards, and the aliases leave these rows out. An id another row of the build keeps
    /// (one in `order`, not itself overridden) or a registry record of an addon in `order` holds is
    /// refused, naming both rows. A registry record of an addon outside the build doesn't register this
    /// session, so its id is lent.
    pub fn lend(&mut self, overrides: &Overrides, order: &[String]) -> std::result::Result<usize, String> {
        let wanted = overrides.keys(order);
        self.lent_build = order.iter().cloned().collect();
        for key in wanted.keys() {
            if let Some(old) = self.rows.remove(key)
                && let Some(ids) = self.by_id.get_mut(table_of(key))
                && ids.get(&old).is_some_and(|k| k == key)
            {
                ids.remove(&old);
            }
        }
        let mut given: HashMap<(&str, i64), &str> = HashMap::new();
        for (key, &id) in &wanted {
            let table = table_of(key);
            if let Some(&other) = given.get(&(table, id)) {
                return Err(format!(
                    "the host's stat id {id} is given to both {} and {}",
                    row_name(other),
                    row_name(key)
                ));
            }
            given.insert((table, id), key);
            if let Some(other) = self.by_id.get(table).and_then(|ids| ids.get(&id))
                && order.iter().any(|a| a == addon_of(other))
            {
                return Err(format!(
                    "the host's stat id {id} for {} is the one your {} uses",
                    row_name(key),
                    row_name(other)
                ));
            }
            if let Some(record) = self.pinned.get(table).and_then(|ids| ids.get(&id))
                && self.pin_holds(record)
            {
                return Err(format!(
                    "the host's stat id {id} for {} is held by the game's registry record {record} here",
                    row_name(key)
                ));
            }
        }
        for (key, &id) in &wanted {
            self.insert(key.clone(), id);
            self.from.insert(key.clone(), IdFrom::Seeded);
        }
        self.lent = wanted;
        self.read_only = true;
        Ok(self.lent.len())
    }

    /// Whether a registry record keeps its id from this build. While a host's ids are lent, a record of an
    /// addon outside the lent build doesn't register this session, so its id is free.
    fn pin_holds(&self, record: &str) -> bool {
        self.lent_build.is_empty() || !record.contains('/') || self.lent_build.contains(addon_of(record))
    }

    /// The first lent row the build gave another id than the host's, as the message that refuses the build.
    pub fn lent_missed(&self) -> Option<String> {
        self.lent.iter().find_map(|(key, &id)| {
            let got = self.rows.get(key).copied();
            (got != Some(id)).then(|| {
                format!(
                    "the host's stat id {id} for {} can't be used here: the build gave it {}",
                    row_name(key),
                    got.map_or("none".to_owned(), |g| g.to_string())
                )
            })
        })
    }

    /// The ids rows had before the ones they keep, by the tables saves hold them of; see [`IdAliases`].
    pub fn id_aliases(&self) -> IdAliases {
        let mut out = IdAliases::default();
        let mut seen: HashMap<(&str, i64), Option<i64>> = HashMap::new();
        for (key, olds) in &self.aliases {
            let Some(&now) = self.rows.get(key).filter(|_| !self.lent.contains_key(key)) else {
                continue;
            };
            let table = key.split('/').nth(1).unwrap_or_default();
            for &old in olds {
                if old == now || self.by_id.get(table).is_some_and(|ids| ids.contains_key(&old)) {
                    continue;
                }
                match seen.entry((table, old)) {
                    std::collections::hash_map::Entry::Vacant(v) => {
                        v.insert(Some(now));
                    }
                    std::collections::hash_map::Entry::Occupied(mut o) => {
                        if o.get() != &Some(now) {
                            o.insert(None);
                        }
                    }
                }
            }
        }
        for ((table, old), now) in seen {
            let (Some(now), Some(name)) = (
                now,
                ALIAS_TABLES.iter().find(|(t, _)| *t == table).map(|(_, n)| *n),
            ) else {
                continue;
            };
            out.tables.entry(name.to_owned()).or_default().insert(old, now);
        }
        out
    }

    /// Reads, for each table the `add-row` ops in `files` add rows to, how many rows `db` holds from
    /// the stock database (`stock` holds its keys), so the seed matches only the rows the last build
    /// added. `tracks` holds the keys of the rows the tracks added.
    pub fn plan<'a>(
        &mut self,
        db: &Gvdb,
        stock: &HashSet<u64>,
        tracks: &HashSet<u64>,
        files: impl IntoIterator<Item = (&'a str, &'a DbOps)>,
    ) {
        let _ = tracks;
        let mut tables: BTreeSet<(String, String)> = BTreeSet::new();
        for (_, ops) in files {
            for op in &ops.ops {
                let DbOp::AddRow { table, from, .. } = op else {
                    continue;
                };
                if table.contains(['[', '{']) || row_shape(table).contains("_N") {
                    continue;
                }
                if let Some(row) = add_row_name(table, from) {
                    tables.insert((table.clone(), row.to_owned()));
                }
            }
        }
        let index: HashMap<u64, usize> = db.records.iter().enumerate().map(|(i, r)| (r.key, i)).collect();
        for (table, row) in tables {
            let count = index
                .get(&fnv1a64(instance_count_path(&table, &row)))
                .and_then(|&i| db.records[i].text().trim().parse::<usize>().ok())
                .unwrap_or(0);
            let stock_rows = (0..count)
                .take_while(|i| stock.contains(&fnv1a64(format!("{table}.{row}_{i}._metadata_"))))
                .count();
            self.seed.stock_rows.insert(format!("{table}.{row}"), stock_rows);
        }
    }

    /// Starts a build of `addons`; ids kept for other addons' rows are reused when a table is full.
    pub fn building(&mut self, addons: impl IntoIterator<Item = String>) {
        self.building = addons.into_iter().collect();
        self.claimed.clear();
        (self.added, self.moved, self.reused) = (0, 0, 0);
    }

    /// The id kept for `key`.
    pub fn get(&self, key: &str) -> Option<i64> {
        self.rows.get(key).copied()
    }

    /// The stat id of the track row `addon` adds for the track `code` through `track.add`, kept in the store under
    /// `<addon>/tracks.Track/<code>`: the id kept for it, else `seed` (the id the last build gave it), else the first
    /// free id from `top`, the table's highest plus one, to [`TRACK_STAT_MAX`] and then from 0. `held` says whether a
    /// track row of this build holds an id. An id kept for any other track stays that track's whether or not its addon
    /// is on, so no id is ever given to two tracks.
    pub fn track_stat_id(
        &mut self,
        addon: &str,
        code: &str,
        seed: Option<i64>,
        top: i64,
        held: &dyn Fn(i64) -> bool,
    ) -> std::result::Result<(i64, String), String> {
        let base = format!("{addon}/{TRACK_TABLE}/{code}");
        let mut nth = 0;
        let key = loop {
            let key = match nth {
                0 => base.clone(),
                n => format!("{base}#{}", n + 1),
            };
            if self.claimed.insert(key.clone()) {
                break key;
            }
            nth += 1;
        };
        let blocked = |s: &Self, id: i64| {
            !(0..=TRACK_STAT_MAX).contains(&id)
                || held(id)
                || s.by_id
                    .get(TRACK_TABLE)
                    .and_then(|ids| ids.get(&id))
                    .is_some_and(|k| *k != key)
        };
        let kept = self.get(&key);
        if let Some(id) = kept.filter(|id| !blocked(self, *id)) {
            return Ok((id, format!("stat id {id} (kept from {FILE})")));
        }
        if kept.is_none()
            && let Some(id) = seed.filter(|id| !blocked(self, *id))
        {
            self.put(&key, id, IdFrom::Seeded);
            return Ok((id, format!("stat id {id} (as the last build gave it)")));
        }
        let from = match kept {
            Some(_) => IdFrom::Moved,
            None => IdFrom::Next,
        };
        let id = (top.max(0)..=TRACK_STAT_MAX)
            .chain(0..top.clamp(0, TRACK_STAT_MAX + 1))
            .find(|id| !blocked(self, *id))
            .ok_or_else(|| {
                format!(
                    "{code}: tracks have no free statId up to {TRACK_STAT_MAX}, the highest the game reads"
                )
            })?;
        self.put(&key, id, from);
        let why = match kept {
            Some(old) => format!("stat id {id} (the {old} kept for it is held by another track)"),
            None => format!("stat id {id} (new, kept in {FILE})"),
        };
        Ok((id, why))
    }

    /// The build line saying what the last build changed, if anything.
    pub fn summary(&self) -> Option<String> {
        if self.added + self.moved + self.reused == 0 {
            return None;
        }
        let mut parts = vec![format!("{} new", self.added)];
        if self.moved > 0 {
            parts.push(format!("{} moved", self.moved));
        }
        if self.reused > 0 {
            parts.push(format!("{} reused", self.reused));
        }
        Some(format!(
            "database: {FILE} keeps {} added-row id(s) ({})",
            self.rows.len(),
            parts.join(", ")
        ))
    }

    /// Writes the store to `data` when this build changed it.
    pub fn save(&self, data: &Path) -> Result<()> {
        if !self.changed || self.read_only {
            return Ok(());
        }
        let rows = self
            .rows
            .iter()
            .map(|(key, &id)| {
                let from = self.from.get(key).copied().unwrap_or(IdFrom::Hashed);
                (key.clone(), Entry { id, from })
            })
            .collect();
        let file = File {
            version: VERSION,
            rows,
            seeded: self.seeded.clone(),
            recovered: self.recovered,
            aliases: self.aliases.clone(),
        };
        crate::write_json(&path(data), &file)
    }
}

// The ids `ids.json` holds and its careers use, by the table whose rows they are ids of.
fn registry_ids(doc: &Value) -> HashMap<String, HashMap<i64, String>> {
    let mut out: HashMap<String, HashMap<i64, String>> = HashMap::new();
    for (registry, table) in REGISTRY_TABLES {
        let ids = out.entry((*table).to_owned()).or_default();
        if let Some(records) = doc["ids"][*registry].as_object() {
            ids.extend(records.iter().filter_map(|(record, e)| {
                e.as_i64()
                    .or_else(|| e["id"].as_i64())
                    .map(|id| (id, record.clone()))
            }));
        }
        if let Some(careers) = doc["careers"].as_object() {
            for career in careers.values() {
                if let Some(records) = career[*registry].as_object() {
                    ids.extend(
                        records
                            .iter()
                            .filter_map(|(record, e)| e.as_i64().map(|id| (id, record.clone()))),
                    );
                }
            }
        }
    }
    out.retain(|_, ids| !ids.is_empty());
    out
}

/// The store as one addon's build sees it.
pub struct Scoped<'a> {
    pub addon: &'a str,
    pub store: &'a RefCell<Store>,
}

impl RowIds for Scoped<'_> {
    fn claim(&self, table: &str, row: &str, label: &str) -> Claimed {
        let mut s = self.store.borrow_mut();
        let base = format!("{}/{table}.{row}/{label}", self.addon);
        let mut nth = 0;
        let key = loop {
            let key = match nth {
                0 => base.clone(),
                n => format!("{base}#{}", n + 1),
            };
            if !s.claimed.contains(&key) {
                break key;
            }
            nth += 1;
        };
        s.claimed.insert(key.clone());
        let kept = s.get(&key);
        let seed = match kept {
            Some(_) => None,
            None => s.seed.id(table, row, label, nth),
        };
        Claimed { key, kept, seed }
    }

    fn taken(&self, table: &str, row: &str, key: &str, id: i64) -> bool {
        let s = self.store.borrow();
        let table = format!("{table}.{row}");
        s.by_id
            .get(&table)
            .and_then(|ids| ids.get(&id))
            .is_some_and(|k| k != key)
            || s.pinned
                .get(&table)
                .and_then(|ids| ids.get(&id))
                .is_some_and(|record| s.pin_holds(record))
    }

    fn keep(&self, _: &str, _: &str, key: &str, id: i64, from: IdFrom) {
        self.store.borrow_mut().put(key, id, from);
    }

    fn reclaim(
        &self,
        table: &str,
        row: &str,
        key: &str,
        free: &dyn Fn(i64) -> bool,
    ) -> Option<(i64, String)> {
        let table = format!("{table}.{row}");
        let spare: Vec<(i64, String)> = {
            let s = self.store.borrow();
            let pinned = s.pinned.get(&table);
            s.by_id
                .get(&table)?
                .iter()
                .filter(|(id, k)| {
                    !s.building.contains(addon_of(k)) && pinned.is_none_or(|p| !p.contains_key(id))
                })
                .map(|(id, k)| (*id, k.clone()))
                .collect()
        };
        let (id, from) = spare.into_iter().find(|(id, _)| free(*id))?;
        let mut s = self.store.borrow_mut();
        s.put(key, id, IdFrom::Reused);
        s.reused += 1;
        Some((id, from))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::{Features, Op, Origin};
    use crate::ops::{DbReport, stat_id_ceiling};
    use crate::planner::Planner;
    use std::collections::BTreeSet;
    use swofty_formats::gvdb::Record;

    const VEHICLES: (&str, &str) = ("entrants.vehicles", "vehicle");
    const PACKAGES: (&str, &str) = ("entrants.vehicles.vehicle_packages", "vehicle_package");

    fn db((table, row): (&str, &str), stock: i64) -> Gvdb {
        Gvdb {
            records: vec![
                Record::new(&format!("{table}._{row}_instancecount_"), 0, "1"),
                Record::new(&format!("{table}.{row}_0._metadata_"), 4, "base"),
                Record::new(&format!("{table}.{row}_0.stat_id"), 0, stock.to_string()),
            ],
        }
    }

    pub(super) struct Built {
        pub(super) ids: BTreeMap<String, i64>,
        pub(super) store: Store,
        pub(super) lines: Vec<String>,
    }

    fn try_build(
        table: (&str, &str),
        stock: i64,
        addons: &[(&str, &[&str])],
        store: Store,
    ) -> crate::Result<Built> {
        try_build_on(db(table, stock), table, addons, store)
    }

    pub(super) fn try_build_on(
        mut gvdb: Gvdb,
        (table, row): (&str, &str),
        addons: &[(&str, &[&str])],
        store: Store,
    ) -> crate::Result<Built> {
        let features = Features::default();
        let opt_in = BTreeSet::new();
        let mut p = Planner::new(&features, &opt_in);
        for (addon, labels) in addons {
            p.add_fields(
                addon,
                BTreeMap::from([(
                    format!("{table}.{row}"),
                    vec!["_metadata_".to_owned(), "stat_id".to_owned()],
                )]),
            );
            for label in *labels {
                let json = format!(
                    r#"{{"op":"db.add-row","table":"{table}","from":"{table}.{row}_0","set":{{"_metadata_":"{label}"}}}}"#
                );
                p.push(Op {
                    op: serde_json::from_str(&json).unwrap(),
                    origin: Origin {
                        addon: (*addon).to_owned(),
                        item: None,
                    },
                });
            }
        }
        p.keep_row_ids(store);
        let first = gvdb.instance_count(table, row).unwrap();
        let stock_keys = gvdb.records.iter().map(|r| r.key).collect();
        let mut report = DbReport::default();
        p.apply_db_report(&mut gvdb, &stock_keys, &HashSet::new(), &mut report)?;
        let n = gvdb.instance_count(table, row).unwrap();
        let field = stat_id_field(table, row);
        let ids = (first..n)
            .map(|i| {
                let label = gvdb.get_text(&format!("{table}.{row}_{i}._metadata_")).unwrap();
                let id = gvdb.get_text(&format!("{table}.{row}_{i}.{field}")).unwrap();
                (label, id.parse().unwrap())
            })
            .collect();
        Ok(Built {
            ids,
            store: p.take_row_ids().unwrap(),
            lines: report.lines,
        })
    }

    fn build(table: (&str, &str), stock: i64, addons: &[(&str, &[&str])], store: Store) -> Built {
        try_build(table, stock, addons, store).unwrap()
    }

    fn data_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("swofty-row-ids-{}-{name}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn put(path: &Path, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    fn key(addon: &str, (table, row): (&str, &str), label: &str) -> String {
        format!("{addon}/{table}.{row}/{label}")
    }

    pub(super) const TEAMS: (&str, &str) = ("entrants.teams", "team");

    const STOCK_TRACKS: i64 = 46;

    fn track_build(store: &mut Store, on: &[(&str, &str)], seeds: &[(&str, i64)]) -> BTreeMap<String, i64> {
        store.building(on.iter().map(|(a, _)| (*a).to_owned()));
        let mut rows: BTreeSet<i64> = (0..STOCK_TRACKS).collect();
        let mut out = BTreeMap::new();
        for (addon, code) in on {
            let seed = seeds.iter().find(|(c, _)| c == code).map(|(_, id)| *id);
            let top = rows.iter().max().map_or(0, |m| m + 1);
            let held = rows.clone();
            let (id, _) = store
                .track_stat_id(addon, code, seed, top, &|id| held.contains(&id))
                .unwrap();
            rows.insert(id);
            out.insert((*code).to_owned(), id);
        }
        out
    }

    fn sixteen() -> Vec<(String, String)> {
        let mut v: Vec<(String, String)> = (0..15).map(|i| (format!("mod.t{i}"), format!("t{i}"))).collect();
        v.insert(9, ("mod.sokol".to_owned(), "sokol".to_owned()));
        v
    }

    fn refs(v: &[(String, String)]) -> Vec<(&str, &str)> {
        v.iter().map(|(a, c)| (a.as_str(), c.as_str())).collect()
    }

    #[test]
    fn a_track_keeps_its_stat_id_when_other_tracks_are_turned_off() {
        let all = sixteen();
        let mut store = Store::default();
        let first = track_build(&mut store, &refs(&all), &[]);
        assert_eq!(first["sokol"], STOCK_TRACKS + 9);
        let few: Vec<_> = all
            .iter()
            .filter(|(_, c)| c == "sokol" || c == "t2" || c == "t13")
            .cloned()
            .collect();
        let second = track_build(&mut store, &refs(&few), &[]);
        assert_eq!(second["sokol"], first["sokol"]);
        assert_eq!(second["t13"], first["t13"]);
        let only: Vec<_> = all.iter().filter(|(_, c)| c == "sokol").cloned().collect();
        assert_eq!(
            track_build(&mut store, &refs(&only), &[])["sokol"],
            first["sokol"]
        );
    }

    #[test]
    fn a_turned_off_tracks_stat_id_is_never_given_to_another_track() {
        let all = sixteen();
        let mut store = Store::default();
        let first = track_build(&mut store, &refs(&all), &[]);
        let off: Vec<_> = all.iter().filter(|(_, c)| c == "t14").cloned().collect();
        let mut with_new = refs(&off);
        with_new.insert(0, ("mod.fresh", "fresh"));
        let second = track_build(&mut store, &with_new, &[]);
        assert!(!first.values().any(|id| *id == second["fresh"]));
        assert_ne!(second["fresh"], first["t14"]);
        assert_ne!(second["fresh"], first["t13"]);
        let again = track_build(&mut store, &refs(&all), &[]);
        assert_eq!(again["t13"], first["t13"]);
        assert_eq!(again["t14"], first["t14"]);
        let ids: BTreeSet<i64> = again.values().copied().collect();
        assert_eq!(ids.len(), again.len());
    }

    #[test]
    fn a_track_without_a_kept_id_takes_the_id_the_last_build_gave_it() {
        let mut store = Store::default();
        let on = [("mod.sokol", "sokol"), ("mod.other", "other")];
        let got = track_build(&mut store, &on, &[("sokol", 48), ("other", 48)]);
        assert_eq!(got["sokol"], 48);
        assert_ne!(got["other"], 48);
        assert!(got["other"] >= STOCK_TRACKS);
    }

    #[test]
    fn track_stat_ids_stay_within_what_the_game_reads() {
        let mut store = Store::default();
        let (id, _) = store
            .track_stat_id("mod.a", "a", Some(300), 250, &|_| false)
            .unwrap();
        assert_eq!(id, 250);
        let err = store.track_stat_id("mod.b", "b", None, 0, &|_| true).unwrap_err();
        assert!(err.contains("254"), "{err}");
    }

    #[test]
    fn a_kept_track_id_that_stock_now_holds_moves_and_is_aliased() {
        let mut store = Store::default();
        store.track_stat_id("mod.a", "a", None, 46, &|_| false).unwrap();
        store.building(["mod.a".to_owned()]);
        let (id, why) = store
            .track_stat_id("mod.a", "a", None, 47, &|id| id == 46)
            .unwrap();
        assert_eq!(id, 47);
        assert!(why.contains("46"), "{why}");
        assert_eq!(store.get("mod.a/tracks.Track/a"), Some(47));
    }

    #[test]
    fn new_ids_are_the_next_free_ids_in_load_order_as_builds_before_1_6_10_gave_them() {
        let first = build(
            VEHICLES,
            10,
            &[("a", &["a1", "a2"]), ("b", &["b1"])],
            Store::default(),
        );
        assert_eq!(
            first.ids,
            BTreeMap::from([("a1".into(), 11), ("a2".into(), 12), ("b1".into(), 13)])
        );
        assert!(
            first
                .lines
                .iter()
                .any(|l| l.ends_with(": stat_id 13 (the source's is 10)")),
            "{:?}",
            first.lines
        );
        assert_eq!(first.store.from[&key("b", VEHICLES, "b1")], IdFrom::Next);
        let again = build(
            VEHICLES,
            10,
            &[("a", &["a1", "a2"]), ("b", &["b1"])],
            Store::default(),
        );
        assert_eq!(
            again.ids, first.ids,
            "identical setups built fresh get identical ids"
        );
    }

    // minik8824 (1565755736410161313): a fresh data folder at 1.6.11 hashed every id, and his save from before
    // 1.6.10 held Classic Seasons 2019's teams at 550-559, the ids those builds gave them in load order.
    #[test]
    fn a_fresh_install_gives_a_save_from_before_1_6_10_its_sequential_ids_back() {
        let years: [(&str, &str); 5] = [
            ("swofty.classic-seasons-2017", "cs17"),
            ("swofty.classic-seasons-2018", "cs18"),
            ("swofty.classic-seasons-2020", "cs20"),
            ("swofty.classic-seasons-2021", "cs21"),
            ("swofty.classic-seasons-2019", "cs19"),
        ];
        let teams = [
            "mercedes",
            "ferrari",
            "redbull",
            "renault",
            "haas",
            "mclaren",
            "racingpoint",
            "alfaromeo",
            "tororosso",
            "williams",
        ];
        let labels: Vec<Vec<String>> = years
            .iter()
            .map(|(_, y)| teams.iter().map(|t| format!("{y}_{t}")).collect())
            .collect();
        let refs: Vec<Vec<&str>> = labels
            .iter()
            .map(|l| l.iter().map(String::as_str).collect())
            .collect();
        let set: Vec<(&str, &[&str])> = years
            .iter()
            .zip(&refs)
            .map(|((a, _), l)| (*a, l.as_slice()))
            .collect();
        let fresh = try_build_on(table_to(TEAMS, 509), TEAMS, &set, Store::default()).unwrap();
        let cs19: Vec<i64> = labels[4].iter().map(|l| fresh.ids[l]).collect();
        assert_eq!(cs19, (550..560).collect::<Vec<_>>());
        assert_eq!(fresh.ids["cs17_mercedes"], 510);
        let cs16: Vec<String> = teams.iter().map(|t| format!("cs16_{t}")).collect();
        let cs16: Vec<&str> = cs16.iter().map(String::as_str).collect();
        let mut more: Vec<(&str, &[&str])> = vec![("swofty.classic-seasons-2016", cs16.as_slice())];
        more.extend(set.iter().copied());
        let later = try_build_on(table_to(TEAMS, 509), TEAMS, &more, fresh.store).unwrap();
        let cs19_later: Vec<i64> = labels[4].iter().map(|l| later.ids[l]).collect();
        assert_eq!(
            cs19_later, cs19,
            "a year added in front later leaves the kept ids where they are"
        );
        assert_eq!(later.ids["cs16_mercedes"], 560);
    }

    #[test]
    fn an_order_swap_keeps_each_rows_id() {
        let first = build(
            VEHICLES,
            10,
            &[("a", &["a1", "a2"]), ("b", &["b1"])],
            Store::default(),
        );
        let swapped = build(VEHICLES, 10, &[("b", &["b1"]), ("a", &["a1", "a2"])], first.store);
        assert_eq!(swapped.ids, first.ids);
        assert!(
            swapped.lines.iter().any(|l| l.ends_with(&format!(
                ": stat_id {} (the source's is 10; kept from row-ids.json)",
                first.ids["b1"]
            ))),
            "{:?}",
            swapped.lines
        );
        assert_eq!(swapped.store.summary(), None);
    }

    #[test]
    fn an_update_that_adds_a_row_keeps_the_old_ids_and_gives_the_new_row_a_fresh_one() {
        let first = build(
            VEHICLES,
            10,
            &[("a", &["a1", "a2"]), ("b", &["b1"])],
            Store::default(),
        );
        let updated = build(
            VEHICLES,
            10,
            &[("a", &["a0", "a1", "a2"]), ("b", &["b1"])],
            first.store,
        );
        for label in ["a1", "a2", "b1"] {
            assert_eq!(updated.ids[label], first.ids[label]);
        }
        assert_eq!(updated.ids["a0"], 14);
        assert_eq!(
            updated.store.summary().as_deref(),
            Some("database: row-ids.json keeps 4 added-row id(s) (1 new)")
        );
    }

    #[test]
    fn a_removed_addon_keeps_its_ids_and_gets_them_back() {
        let first = build(VEHICLES, 10, &[("a", &["a1"]), ("b", &["b1"])], Store::default());
        let without = build(VEHICLES, 10, &[("b", &["b1"]), ("c", &["c1"])], first.store);
        assert_eq!(without.ids["b1"], first.ids["b1"]);
        assert_ne!(without.ids["c1"], first.ids["a1"]);
        let back = build(
            VEHICLES,
            10,
            &[("a", &["a1"]), ("b", &["b1"]), ("c", &["c1"])],
            without.store,
        );
        assert_eq!(back.ids["a1"], first.ids["a1"]);
        assert_eq!(back.ids["b1"], first.ids["b1"]);
        assert_eq!(back.ids["c1"], without.ids["c1"]);
    }

    #[test]
    fn two_profiles_share_a_rows_id_through_the_install_store() {
        let data = data_dir("profiles");
        let (store, warnings) = Store::load(&data, None, "one", None);
        assert!(warnings.is_empty(), "{warnings:?}");
        let one = build(VEHICLES, 10, &[("a", &["a1"]), ("b", &["b1"])], store);
        one.store.save(&data).unwrap();
        let saved: Value = serde_json::from_slice(&std::fs::read(path(&data)).unwrap()).unwrap();
        assert_eq!(
            saved["rows"]["b/entrants.vehicles.vehicle/b1"]["id"],
            one.ids["b1"]
        );
        assert_eq!(saved["rows"]["b/entrants.vehicles.vehicle/b1"]["from"], "next");
        let (store, _) = Store::load(&data, None, "two", None);
        let two = build(VEHICLES, 10, &[("b", &["b1"])], store);
        assert_eq!(two.ids["b1"], one.ids["b1"]);
        std::fs::remove_dir_all(&data).ok();
    }

    #[test]
    fn ids_the_core_registry_holds_are_passed_over() {
        let data = data_dir("registry");
        let slot = 11;
        put(
            &reports::path(&data, reports::IDS),
            &format!(
                r#"{{"version":1,"ids":{{"vehicle":{{"mod.x/car":{{"id":{slot},"since":"2026-10-29"}},"mod.x/old":12}}}},
                "careers":{{"dc-1":{{"vehicle":{{"mod.y/car":13}}}}}}}}"#
            ),
        );
        let (store, _) = Store::load(&data, None, "p", None);
        let first = build(VEHICLES, 10, &[("a", &["a1"])], store);
        let want = 14;
        assert_eq!(first.ids["a1"], want);
        let mut kept = Store::default();
        kept.put(&key("a", VEHICLES, "a1"), 13, IdFrom::Next);
        kept.pinned = Store::load(&data, None, "p", None).0.pinned;
        let moved = build(VEHICLES, 10, &[("a", &["a1"])], kept);
        assert_eq!(moved.ids["a1"], want);
        assert!(
            moved.lines.iter().any(|l| l.ends_with(&format!(
                "stat_id {want} (the source's is 10; 13, kept for it, is kept for another row or held by a record of the core's registry)"
            ))),
            "{:?}",
            moved.lines
        );
        assert_eq!(moved.store.get(&key("a", VEHICLES, "a1")), Some(want));
        assert_eq!(moved.store.from[&key("a", VEHICLES, "a1")], IdFrom::Moved);
        std::fs::remove_dir_all(&data).ok();
    }

    #[test]
    fn driver_ids_above_255_skip_254_255_and_low_byte_0xff() {
        const DRIVERS: (&str, &str) = ("entrants.drivers", "driver");
        let labels: Vec<String> = (0..120).map(|i| format!("d{i}")).collect();
        let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
        let built = build(DRIVERS, 254, &[("a", &refs)], Store::default());
        let ids: BTreeSet<i64> = built.ids.values().copied().collect();
        assert_eq!(ids.len(), 120);
        assert!(ids.iter().any(|id| *id > 255));
        assert!(
            ids.iter()
                .all(|id| (208..=16382).contains(id) && *id != 254 && id & 0xff != 0xff)
        );
        let again = build(DRIVERS, 254, &[("a", &refs)], built.store);
        assert_eq!(again.ids, built.ids);
    }

    #[test]
    fn a_full_table_reuses_an_id_kept_for_an_addon_the_build_leaves_out_and_logs_it() {
        let first = build(
            PACKAGES,
            250,
            &[("a", &["a1", "a2", "a3", "a4"])],
            Store::default(),
        );
        let mut ids: Vec<i64> = first.ids.values().copied().collect();
        ids.sort();
        assert_eq!(ids, [251, 252, 253, 254]);
        let e = try_build(
            PACKAGES,
            250,
            &[("a", &["a1", "a2", "a3", "a4"]), ("b", &["b1"])],
            Store::default(),
        )
        .err()
        .unwrap();
        assert!(e.to_string().contains("has no free stat_id up to 254"), "{e}");
        let holder = |id: i64| first.ids.iter().find(|(_, v)| **v == id).unwrap().0.clone();
        let b = build(PACKAGES, 250, &[("b", &["b1", "b2"])], first.store);
        assert_eq!((b.ids["b1"], b.ids["b2"]), (251, 252));
        let line = format!(
            "b: add-row entrants.vehicles.vehicle_packages.vehicle_package_1 (from entrants.vehicles.vehicle_packages.vehicle_package_0): stat_id 251 (the source's is 250; no other id up to 254 is free; reused from {}, which this build leaves out)",
            key("a", PACKAGES, &holder(251))
        );
        assert!(b.lines.contains(&line), "{:?}", b.lines);
        assert_eq!(b.store.get(&key("a", PACKAGES, &holder(251))), None);
        assert_eq!(b.store.get(&key("a", PACKAGES, &holder(253))), Some(253));
        assert_eq!(b.store.from[&key("b", PACKAGES, "b1")], IdFrom::Reused);
        assert_eq!(
            b.store.summary().as_deref(),
            Some("database: row-ids.json keeps 4 added-row id(s) (2 new, 2 reused)")
        );
    }

    #[test]
    fn rows_the_store_has_no_id_for_take_the_profiles_last_build_ids() {
        let data = data_dir("seed");
        let overlays = data.join("cache/overlay/p");
        let (table, row) = VEHICLES;
        let mut last = db(VEHICLES, 10);
        last.set(&format!("{table}._{row}_instancecount_"), "3", None)
            .unwrap();
        for (i, (label, id)) in [("a1", "40"), ("b1", "41")].iter().enumerate() {
            last.set(&format!("{table}.{row}_{}._metadata_", i + 1), *label, Some(4))
                .unwrap();
            last.set(&format!("{table}.{row}_{}.stat_id", i + 1), *id, Some(0))
                .unwrap();
        }
        let rel = "game_package/db.bin";
        put(&overlays.join("old").join(rel), "");
        std::fs::write(overlays.join("old").join(rel), last.to_bytes()).unwrap();
        assert_eq!(
            last_build(&overlays, "new", rel),
            Some(LastBuild {
                db: Some(overlays.join("old").join(rel)),
                edits: None
            })
        );
        assert_eq!(last_build(&overlays, "old", rel), None);
        let (store, _) = Store::load(&data, None, "p", last_build(&overlays, "new", rel));
        let first = build(
            VEHICLES,
            10,
            &[("b", &["b1"]), ("a", &["a1"]), ("c", &["c1"])],
            store,
        );
        assert_eq!(
            first.ids,
            BTreeMap::from([("a1".into(), 40), ("b1".into(), 41), ("c1".into(), 42)])
        );
        assert!(
            first
                .lines
                .iter()
                .any(|l| l.ends_with("stat_id 41 (the source's is 10; as the last build gave it)")),
            "{:?}",
            first.lines
        );
        assert_eq!(first.store.from[&key("a", VEHICLES, "a1")], IdFrom::Seeded);
        first.store.save(&data).unwrap();
        let seed = |profile: &str| Store::load(&data, None, profile, last_build(&overlays, "new", rel)).0;
        assert_eq!(seed("p").seed.last, None);
        assert!(seed("q").seed.last.is_some());
        let kept = kept_rows(&data, "a").unwrap();
        assert_eq!(
            kept,
            [Kept {
                table: "entrants.vehicles.vehicle".into(),
                label: "a1".into(),
                id: 40,
                from: IdFrom::Seeded
            }]
        );
        std::fs::remove_dir_all(&data).ok();
    }

    #[test]
    fn two_rows_of_one_addon_with_one_label_keep_their_own_ids() {
        let first = build(VEHICLES, 10, &[("a", &["x", "x"])], Store::default());
        let x = first.store.get(&key("a", VEHICLES, "x")).unwrap();
        let x2 = first
            .store
            .get(&format!("{}#2", key("a", VEHICLES, "x")))
            .unwrap();
        assert_ne!(x, x2);
        let again = build(VEHICLES, 10, &[("b", &["b1"]), ("a", &["x", "x"])], first.store);
        assert_eq!(again.store.get(&key("a", VEHICLES, "x")), Some(x));
        assert_eq!(
            again.store.get(&format!("{}#2", key("a", VEHICLES, "x"))),
            Some(x2)
        );
    }

    const MODES: (&str, &str) = ("game_setup.game_setup_modes", "game_setup_mode");
    const ENTRANT_MODES: (&str, &str) = ("entrants.entrant_setup_modes", "entrant_setup_mode");

    // A table of stock rows holding the ids 0 to `top`.
    pub(super) fn table_to((table, row): (&str, &str), top: i64) -> Gvdb {
        let field = stat_id_field(table, row);
        let mut records = vec![Record::new(
            &format!("{table}._{row}_instancecount_"),
            0,
            (top + 1).to_string(),
        )];
        for i in 0..=top {
            records.push(Record::new(
                &format!("{table}.{row}_{i}._metadata_"),
                4,
                format!("stock_{i}"),
            ));
            records.push(Record::new(
                &format!("{table}.{row}_{i}.{field}"),
                0,
                i.to_string(),
            ));
        }
        Gvdb { records }
    }

    #[test]
    fn mode_ids_stay_under_the_mask_and_take_the_next_free_id_in_load_order() {
        let labels: Vec<String> = (0..10).map(|i| format!("mode_{i}")).collect();
        let many: Vec<&str> = labels.iter().map(String::as_str).collect();
        let addons: &[(&str, &[&str])] = &[("bydanix", &["bdx_2012_mode"]), ("many", &many)];
        let built = try_build_on(table_to(MODES, 20), MODES, addons, Store::default()).unwrap();
        assert_eq!(built.ids["bdx_2012_mode"], 21);
        assert_eq!(built.ids["mode_9"], 31);
        assert!(built.ids.values().all(|id| *id <= 31), "{:?}", built.ids);
        assert_eq!(
            built.store.from[&key("bydanix", MODES, "bdx_2012_mode")],
            IdFrom::Next
        );
        let full: &[(&str, &[&str])] = &[
            ("bydanix", &["bdx_2012_mode"]),
            ("many", &many),
            ("more", &["one"]),
        ];
        let e = try_build_on(table_to(MODES, 20), MODES, full, Store::default())
            .err()
            .unwrap();
        assert!(e.to_string().contains("has no free stat_id up to 31"), "{e}");
        let mut kept = Store::default();
        kept.put(&key("bydanix", MODES, "bdx_2012_mode"), 44, IdFrom::Hashed);
        let moved = try_build_on(
            table_to(MODES, 20),
            MODES,
            &[("bydanix", &["bdx_2012_mode"])],
            kept,
        )
        .unwrap();
        assert_eq!(moved.ids["bdx_2012_mode"], 21);
        assert!(
            moved.lines.iter().any(|l| l.ends_with(
                "stat_id 21 (the source's is 0; 44, kept for it, is above 31, the highest the game reads)"
            )),
            "{:?}",
            moved.lines
        );
        let core_first: &[(&str, &[&str])] =
            &[("swofty.core", &["swofty_f1"]), ("bydanix", &["bdx_2012_mode"])];
        let entrants = try_build_on(
            table_to(ENTRANT_MODES, 12),
            ENTRANT_MODES,
            core_first,
            Store::default(),
        )
        .unwrap();
        assert_eq!(
            (entrants.ids["swofty_f1"], entrants.ids["bdx_2012_mode"]),
            (13, 14)
        );
        let seasons = ("game_setup.seasons", "season");
        assert_eq!(stat_id_field(seasons.0, seasons.1), "statId");
        assert_eq!(stat_id_ceiling(seasons.0, seasons.1).map(|c| c.max), Some(31));
    }

    #[test]
    fn a_new_profile_seeds_from_another_profiles_last_build() {
        let data = data_dir("other-profile");
        let overlay = data.join("cache/overlay");
        let mut old = table_to(MODES, 20);
        let (table, row) = MODES;
        old.set(&format!("{table}._{row}_instancecount_"), "22", None)
            .unwrap();
        old.set(&format!("{table}.{row}_21._metadata_"), "bdx_2012_mode", Some(4))
            .unwrap();
        old.set(&format!("{table}.{row}_21.stat_id"), "25", Some(0))
            .unwrap();
        let rel = "game_package/db.bin";
        put(&overlay.join("ig10/older").join(rel), "");
        std::fs::write(overlay.join("ig10/older").join(rel), old.to_bytes()).unwrap();
        std::fs::create_dir_all(overlay.join("fresh")).unwrap();
        let last = last_build(&overlay.join("fresh"), "new", rel);
        assert_eq!(
            last.as_ref().and_then(|l| l.db.clone()),
            Some(overlay.join("ig10/older").join(rel))
        );
        let (store, _) = Store::load(&data, None, "fresh", last);
        let built = try_build_on(
            table_to(MODES, 20),
            MODES,
            &[("bydanix", &["bdx_2012_mode"])],
            store,
        )
        .unwrap();
        assert_eq!(built.ids["bdx_2012_mode"], 25);
        assert!(
            built
                .lines
                .iter()
                .any(|l| l.ends_with("stat_id 25 (the source's is 0; as the last build gave it)")),
            "{:?}",
            built.lines
        );
        assert_eq!(
            built.store.from[&key("bydanix", MODES, "bdx_2012_mode")],
            IdFrom::Seeded
        );
        std::fs::remove_dir_all(&data).ok();
    }

    fn host_ids(rows: &[(&str, &str, i64)]) -> Overrides {
        let mut o = Overrides::default();
        for (addon, label, id) in rows {
            o.rows
                .entry((*addon).to_owned())
                .or_default()
                .insert(format!("{}.{}/{label}", VEHICLES.0, VEHICLES.1), *id);
        }
        o
    }

    fn order(addons: &[&str]) -> Vec<String> {
        addons.iter().map(|a| (*a).to_owned()).collect()
    }

    fn players_store(name: &str) -> (PathBuf, Built) {
        let data = data_dir(name);
        let first = build(
            VEHICLES,
            10,
            &[("a", &["a1", "a2"]), ("b", &["b1"])],
            Store::default(),
        );
        first.store.save(&data).unwrap();
        (data, first)
    }

    #[test]
    fn the_hosts_ids_are_used_for_one_build_and_never_written() {
        let (data, own) = players_store("lend-apply");
        assert_eq!(
            own.ids,
            BTreeMap::from([("a1".into(), 11), ("a2".into(), 12), ("b1".into(), 13)])
        );
        let before = std::fs::read(path(&data)).unwrap();
        let (mut store, _) = Store::load(&data, None, "default", None);
        let host = host_ids(&[("a", "a1", 21), ("a", "a2", 22), ("b", "b1", 23)]);
        assert_eq!(store.lend(&host, &order(&["a", "b"])), Ok(3));
        let lent = build(VEHICLES, 10, &[("a", &["a1", "a2"]), ("b", &["b1"])], store);
        assert_eq!(
            lent.ids,
            BTreeMap::from([("a1".into(), 21), ("a2".into(), 22), ("b1".into(), 23)])
        );
        assert_eq!(lent.store.lent_missed(), None);
        lent.store.save(&data).unwrap();
        assert_eq!(
            std::fs::read(path(&data)).unwrap(),
            before,
            "row-ids.json is as the player's own build left it"
        );
        let (own_again, _) = Store::load(&data, None, "default", None);
        let again = build(VEHICLES, 10, &[("a", &["a1", "a2"]), ("b", &["b1"])], own_again);
        assert_eq!(
            again.ids, own.ids,
            "the next normal build has the player's own ids"
        );
        std::fs::remove_dir_all(&data).ok();
    }

    #[test]
    fn two_addons_that_swap_ids_with_the_host_are_not_a_collision() {
        let (data, _) = players_store("lend-swap");
        let (mut store, _) = Store::load(&data, None, "default", None);
        let host = host_ids(&[("a", "a1", 13), ("b", "b1", 11)]);
        assert_eq!(store.lend(&host, &order(&["a", "b"])), Ok(2));
        let lent = build(VEHICLES, 10, &[("a", &["a1", "a2"]), ("b", &["b1"])], store);
        assert_eq!(lent.ids["a1"], 13);
        assert_eq!(lent.ids["b1"], 11);
        assert_eq!(lent.store.lent_missed(), None);
        std::fs::remove_dir_all(&data).ok();
    }

    #[test]
    fn an_id_another_row_of_the_build_uses_is_refused_and_one_of_an_addon_left_out_is_not() {
        let (data, _) = players_store("lend-collide");
        let (mut store, _) = Store::load(&data, None, "default", None);
        let host = host_ids(&[("a", "a1", 13)]);
        let err = store.lend(&host, &order(&["a", "b"])).unwrap_err();
        assert!(
            err.contains("stat id 13") && err.contains("a1") && err.contains("b1 of b"),
            "{err}"
        );
        let (mut store, _) = Store::load(&data, None, "default", None);
        assert_eq!(store.lend(&host, &order(&["a"])), Ok(1));
        let lent = build(VEHICLES, 10, &[("a", &["a1", "a2"])], store);
        assert_eq!(lent.ids["a1"], 13);
        assert_eq!(lent.store.lent_missed(), None);
        std::fs::remove_dir_all(&data).ok();
    }

    #[test]
    fn a_host_id_the_game_registry_holds_is_refused() {
        let data = data_dir("lend-pinned");
        let (mut store, _) = Store::load(&data, None, "default", None);
        store.pinned.insert(
            "entrants.vehicles.vehicle".into(),
            HashMap::from([(40, "a/a_record".to_owned())]),
        );
        let err = store
            .lend(&host_ids(&[("a", "a1", 40)]), &order(&["a"]))
            .unwrap_err();
        assert!(err.contains("registry record a/a_record"), "{err}");
        std::fs::remove_dir_all(&data).ok();
    }

    #[test]
    fn a_lent_id_an_outside_addon_pins_is_the_one_the_build_gives() {
        let data = data_dir("lend-pinned-build");
        let (mut store, _) = Store::load(&data, None, "default", None);
        store.pinned.insert(
            "entrants.vehicles.vehicle".into(),
            HashMap::from([(13, "z.indycar/z_record".to_owned())]),
        );
        assert_eq!(store.lend(&host_ids(&[("a", "a1", 13)]), &order(&["a"])), Ok(1));
        let lent = build(VEHICLES, 10, &[("a", &["a1", "a2"])], store);
        assert_eq!(lent.ids["a1"], 13);
        assert_eq!(lent.store.lent_missed(), None);
        std::fs::remove_dir_all(&data).ok();
    }

    #[test]
    fn a_host_id_held_by_a_registry_record_of_an_addon_outside_the_build_is_lent() {
        let data = data_dir("lend-pinned-off");
        let (mut store, _) = Store::load(&data, None, "default", None);
        store.pinned.insert(
            "entrants.vehicles.vehicle".into(),
            HashMap::from([(40, "z.indycar/z_record".to_owned())]),
        );
        assert_eq!(store.lend(&host_ids(&[("a", "a1", 40)]), &order(&["a"])), Ok(1));
        std::fs::remove_dir_all(&data).ok();
    }

    #[test]
    fn the_cache_key_names_only_the_overrides_of_addons_in_the_build() {
        let host = host_ids(&[("a", "a1", 21), ("z", "z1", 5)]);
        let with = host.stamp(&order(&["a", "b"])).unwrap();
        assert!(
            with.contains("a/entrants.vehicles.vehicle/a1") && !with.contains("z1"),
            "{with}"
        );
        assert_ne!(
            Some(with),
            host_ids(&[("a", "a1", 22)]).stamp(&order(&["a", "b"]))
        );
        assert_eq!(host.stamp(&order(&["b"])), None);
        assert_eq!(Overrides::default().stamp(&order(&["a"])), None);
    }

    #[test]
    fn a_lent_row_leaves_the_id_aliases_alone() {
        let (data, _) = players_store("lend-aliases");
        let (mut store, _) = Store::load(&data, None, "default", None);
        let key = key("a", VEHICLES, "a1");
        store.aliases.insert(key.clone(), BTreeSet::from([5]));
        assert!(
            store
                .id_aliases()
                .tables
                .get("vehicles")
                .is_some_and(|t| t.get(&5) == Some(&11))
        );
        store
            .lend(&host_ids(&[("a", "a1", 21)]), &order(&["a", "b"]))
            .unwrap();
        assert!(store.id_aliases().is_empty(), "{:?}", store.id_aliases());
        std::fs::remove_dir_all(&data).ok();
    }

    #[test]
    fn overrides_round_trip_through_their_file() {
        let data = data_dir("lend-file");
        let file = data.join("room.json");
        let host = host_ids(&[("a", "a1", 21)]);
        host.write(&file).unwrap();
        assert_eq!(Overrides::read(&file).unwrap(), host);
        assert_eq!(
            Overrides::read(&data.join("none.json")).unwrap(),
            Overrides::default()
        );
        std::fs::remove_dir_all(&data).ok();
    }
}
