use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use swofty_addon::asset::{AssetOp, FileKind, LangOp, OpFile, Origin, TrackOp};
use swofty_addon::manifest::AddonManifest;
use swofty_addon::ops::DbOp;
use swofty_formats::gvdb::{Gvdb, Record, ValueType};
use swofty_formats::hash::fnv1a64;

use crate::launcher::{ProfileTrack, ProfileTracks};
use crate::map::{TrackPackage, Unusable};
use crate::stdlib::deps::Std;
use crate::store::Store;
use crate::{Result, io_err};

const TRACKS: &str = "tracks.Track";
const EVENTS: &str = "schedule.all_track_events.fixed_event";
const ACTIVITIES: &str = "schedule.activities.activity_race_weekend";
const CONTENT_SETS: &str = "content_sets.content_set";
const SEASON_SETS: [&str; 2] = ["2025", "2026"];
const RESOURCE_PREFIX: &str = "swofty_";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PlanTrack {
    pub code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub addon: Option<String>,
}

struct Rows<'a> {
    by_key: HashMap<u64, &'a Record>,
}

impl<'a> Rows<'a> {
    fn new(db: &'a Gvdb) -> Self {
        Self {
            by_key: db.records.iter().map(|r| (r.key, r)).collect(),
        }
    }

    fn text(&self, path: &str) -> Option<String> {
        self.by_key
            .get(&fnv1a64(path))
            .map(|r| r.text().trim().to_owned())
    }

    fn table(&self, path: &str, fields: &[&str]) -> Vec<Vec<Option<String>>> {
        let (container, row) = path.rsplit_once('.').unwrap_or(("", path));
        let count = self
            .text(&format!("{container}._{row}_instancecount_"))
            .and_then(|n| n.parse::<usize>().ok())
            .unwrap_or(0);
        (0..count)
            .map(|i| {
                fields
                    .iter()
                    .map(|f| self.text(&format!("{path}_{i}.{f}")))
                    .collect()
            })
            .collect()
    }
}

pub fn list(db: &Gvdb, added: &[PlanTrack]) -> Vec<ProfileTrack> {
    let rows = Rows::new(db);
    let activities: HashMap<String, String> = rows
        .table(ACTIVITIES, &["_metadata_", "track"])
        .into_iter()
        .filter_map(|r| Some((r[0].clone()?, r[1].clone()?)))
        .collect();
    let mut events: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for r in rows.table(EVENTS, &["_metadata_", "activity"]) {
        let (Some(name), Some(activity)) = (r[0].clone(), r[1].clone()) else {
            continue;
        };
        let track = activities.get(&activity).cloned().unwrap_or(activity);
        events.entry(track).or_default().push(name);
    }
    let mut playable: Vec<String> = Vec::new();
    for (i, set) in rows.table(CONTENT_SETS, &["_metadata_"]).into_iter().enumerate() {
        if !set[0].as_deref().is_some_and(|l| SEASON_SETS.contains(&l)) {
            continue;
        }
        let lists = rows.table(&format!("{CONTENT_SETS}_{i}.available_tracks"), &["tracks"]);
        for list in lists.into_iter().filter_map(|r| r[0].clone()) {
            playable.extend(
                list.split(',')
                    .map(|c| c.trim().to_owned())
                    .filter(|c| !c.is_empty()),
            );
        }
    }
    let mut out: Vec<ProfileTrack> = Vec::new();
    for r in rows.table(TRACKS, &["_metadata_", "statId", "track_resource"]) {
        let (Some(code), Some(stat_id)) = (r[0].clone(), r[1].as_deref().and_then(|s| s.parse::<u32>().ok()))
        else {
            continue;
        };
        if out.iter().any(|t| t.code == code) {
            continue;
        }
        let package = added.iter().find(|p| p.code == code);
        let modded = package.is_some() || r[2].as_deref() == Some(&format!("{RESOURCE_PREFIX}{code}"));
        out.push(ProfileTrack {
            events: events.remove(&code).unwrap_or_default(),
            stat_id,
            playable: playable.contains(&code),
            modded,
            name: package.and_then(|p| p.name.clone()),
            addon: package.and_then(|p| p.addon.clone()),
            code,
        });
    }
    out
}

#[derive(Deserialize)]
struct PlanListing {
    #[serde(default)]
    buildhash: Option<String>,
    #[serde(default)]
    tracks: Vec<PlanTrack>,
}

type Cached = (PathBuf, Option<SystemTime>, u64, ProfileTracks);

static CACHE: Mutex<Option<Cached>> = Mutex::new(None);

fn stamp(path: &Path) -> (Option<SystemTime>, u64) {
    std::fs::metadata(path).map_or((None, 0), |m| (m.modified().ok(), m.len()))
}

pub fn for_profile(data: &Path, game: &crate::install::Game, profile: &str) -> Result<ProfileTracks> {
    let plan = crate::cache::latest_plan(&data.join("cache/overlay").join(profile));
    let listing: Option<PlanListing> = plan.as_deref().and_then(|p| crate::read_json(p).ok());
    let built_db = plan
        .as_deref()
        .and_then(Path::parent)
        .map(|d| d.join(crate::build::GVDB))
        .filter(|p| p.is_file());
    let db_path = built_db
        .clone()
        .unwrap_or_else(|| crate::build::original(data, game, crate::build::GVDB));
    let (modified, len) = stamp(&db_path);
    let buildhash = listing.as_ref().and_then(|l| l.buildhash.clone());
    let added = listing.map(|l| l.tracks).unwrap_or_default();
    {
        let cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((p, m, l, tracks)) = cache.as_ref()
            && *p == db_path
            && *m == modified
            && *l == len
            && tracks.buildhash == buildhash
        {
            return Ok(tracks.clone());
        }
    }
    let bytes = std::fs::read(&db_path).map_err(io_err(&db_path))?;
    let db = Gvdb::parse(&bytes)?;
    let tracks = ProfileTracks {
        built: plan.is_some(),
        buildhash,
        tracks: list(&db, &added),
    };
    *CACHE.lock().unwrap_or_else(|e| e.into_inner()) = Some((db_path, modified, len, tracks.clone()));
    Ok(tracks)
}

/// A data addon that is only tracks: its ops add one or more tracks and give
/// them names (`lang.set`, `lang.default`, `db.set` on a `TrackNameData` field), and it has no
/// code, screens, files, settings, launcher pages or other edits. The launcher
/// lists these on the Tracks page instead of the Addons page. An op file that
/// doesn't read makes the addon an ordinary one.
pub fn is_track_addon(dir: &Path, m: &AddonManifest) -> bool {
    let o = &m.operations;
    let other = !o.database.is_empty()
        || !o.erp.is_empty()
        || !o.ui.is_empty()
        || !o.files.is_empty()
        || !o.native.is_empty()
        || !o.screens.is_empty();
    if other || o.assets.is_empty() || m.launcher.is_some() || !m.options.is_empty() || m.saves.affects {
        return false;
    }
    let mut tracks = 0;
    for rel in &o.assets {
        let Some(doc) = std::fs::read(dir.join(rel))
            .ok()
            .and_then(|d| serde_json::from_slice(&d).ok())
        else {
            return false;
        };
        let Ok(file) = OpFile::from_value(doc, FileKind::Assets) else {
            return false;
        };
        for op in file.ops {
            match op {
                AssetOp::Track(TrackOp::Add { .. }) => tracks += 1,
                AssetOp::Lang(LangOp::Set { .. } | LangOp::Default { .. }) | AssetOp::Warn { .. } => {}
                AssetOp::Db(DbOp::Set { field, .. }) if field.starts_with("TrackNameData") => {}
                _ => return false,
            }
        }
    }
    tracks > 0
}

// The package folders an addon's `track.add` ops in its op files name.
fn added_dirs(store: &Store, id: &str, m: &AddonManifest) -> Vec<PathBuf> {
    let dir = store.addons_dir().join(id);
    let mut out = Vec::new();
    for rel in &m.operations.assets {
        let Some(file) = std::fs::read(dir.join(rel))
            .ok()
            .and_then(|d| serde_json::from_slice(&d).ok())
            .and_then(|doc| OpFile::from_value(doc, FileKind::Assets).ok())
        else {
            continue;
        };
        for op in file.ops {
            if let AssetOp::Track(TrackOp::Add { source }) = op
                && let Ok(d) = source.resolve(&crate::build::source_dir(store, id, source.root))
            {
                out.push(d);
            }
        }
    }
    out
}

/// The tracks an addon's `track.add` operations add, as the core lists them
/// (`<addon>/<code>`), with the venue each one's name matches.
pub fn older_ids(store: &Store, id: &str, m: &AddonManifest) -> Vec<(String, Option<String>)> {
    added_dirs(store, id, m)
        .iter()
        .filter_map(|d| crate::map::read_package(d).ok())
        .map(|pkg| {
            let venue = pkg
                .track_name
                .as_deref()
                .and_then(|n| Venues::shipped().by_name(n));
            (format!("{id}/{}", pkg.track_code), venue)
        })
        .collect()
}

/// The tracks installed addons add with `track.add`, named from their
/// packages, by addon id; the first addon to add a track keeps it. Ops or
/// packages that don't read are skipped.
pub fn installed(store: &Store) -> Vec<PlanTrack> {
    let mut out: Vec<PlanTrack> = Vec::new();
    for (id, m) in store.addons().unwrap_or_default() {
        for dir in added_dirs(store, &id, &m) {
            let Ok(pkg) = crate::map::read_package(&dir) else {
                continue;
            };
            if out.iter().any(|t| t.code == pkg.track_code) {
                continue;
            }
            out.push(PlanTrack {
                name: pkg.track_name.filter(|n| !n.trim().is_empty()),
                addon: Some(id.clone()),
                code: pkg.track_code,
            });
        }
    }
    out
}

/// The track packages an addon's op files add that builds leave out.
pub fn unusable(store: &Store, id: &str, m: &AddonManifest) -> Vec<Unusable> {
    added_dirs(store, id, m)
        .iter()
        .filter_map(|d| crate::map::read_package(d).err())
        .collect()
}

/// The registry of track records (`registry/track.json`, `tracks::tracks`).
pub const TRACK_REGISTRY: &str = "track";
/// `$DATA/registry/older-tracks.json`: the tracks `track.add` adds for addons with no track record.
pub const OLDER_FILE: &str = "older-tracks.json";
pub const OLDER_VERSION: u32 = 1;
const VENUES_JSON: &str = include_str!("../../sdk/std/data/venues.json");

/// A `track` record an addon's data file adds under its own id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordTrack {
    pub id: String,
    pub code: String,
    pub name: Option<String>,
    pub base: Option<String>,
    pub slot: Option<String>,
    pub files: Option<String>,
}

/// The `track` records the addon's data files (`data_files`, relative to `dir`) add, in file
/// then key order. A record without a `code` is skipped.
pub fn record_tracks(dir: &Path, addon: &str, data_files: &[String]) -> Vec<RecordTrack> {
    let mut out = Vec::new();
    for rel in data_files {
        let Some(doc) = std::fs::read_to_string(dir.join(rel))
            .ok()
            .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        else {
            continue;
        };
        let mut records: Vec<(&String, &serde_json::Value)> = doc
            .get(TRACK_REGISTRY)
            .and_then(|t| t.as_object())
            .into_iter()
            .flatten()
            .collect();
        records.sort_by_key(|(k, _)| *k);
        for (key, value) in records {
            if key.contains('/') || key.starts_with('$') {
                continue;
            }
            let s = |field: &str| value.get(field).and_then(|v| v.as_str()).map(str::to_owned);
            let Some(code) = s("code") else {
                continue;
            };
            out.push(RecordTrack {
                id: format!("{addon}/{key}"),
                code,
                name: s("name"),
                base: s("base"),
                slot: s("slot"),
                files: s("files"),
            });
        }
    }
    out
}

fn stock_code(id: &str) -> Option<&str> {
    id.strip_prefix("game/").filter(|c| !c.is_empty())
}

/// The package a track record's `files` folder holds, for the build to place: its
/// `manifest.json` when it has one, else the record's code, name, base and slot with
/// `<code>.erp` and the folder's `mipmaps/`.
pub fn record_package(
    addon_dir: &Path,
    t: &RecordTrack,
) -> std::result::Result<(PathBuf, TrackPackage), Unusable> {
    let unusable = |why: String| Unusable {
        code: Some(t.code.clone()),
        name: Some(t.name.clone().unwrap_or_else(|| t.code.clone())),
        why,
    };
    let Some(rel) = t.files.as_deref() else {
        return Err(unusable(format!(
            "its track record {} names no files folder",
            t.id
        )));
    };
    let Some(inside) = crate::stdlib::paths::inside(rel) else {
        return Err(unusable(format!(
            "its track record {} names {rel}, which leaves the addon's folder",
            t.id
        )));
    };
    let dir = addon_dir.join(inside.trim_end_matches('/'));
    if dir.join(crate::map::MANIFEST_FILE).is_file() {
        let pkg = crate::map::read_package(&dir)?;
        if pkg.track_code != t.code {
            return Err(unusable(format!(
                "its track record {} has code {}, and {rel}{} is for {}",
                t.id,
                t.code,
                crate::map::MANIFEST_FILE,
                pkg.track_code
            )));
        }
        return Ok((dir, pkg));
    }
    let base = t.base.as_deref().and_then(stock_code).map(str::to_owned);
    let slot = t.slot.as_deref().and_then(stock_code).map(str::to_owned);
    if base.is_none() && slot.is_none() {
        return Err(unusable(format!(
            "its track record {} names no stock base track",
            t.id
        )));
    }
    let erp = format!("{}.erp", t.code);
    if !dir.join(&erp).is_file() {
        return Err(unusable(format!("its files folder {rel} has no {erp}")));
    }
    let mips = dir.join("mipmaps").is_dir();
    let pkg = TrackPackage {
        track_code: t.code.clone(),
        track_name: t.name.clone(),
        version: None,
        author_name: None,
        erp,
        activation: crate::map::Activation {
            copy_from_track: base,
            copy_slot: slot,
            ..Default::default()
        },
        ers: Default::default(),
        copy_directories: match mips {
            true => vec![crate::map::CopyDirectory {
                source: "mipmaps/".into(),
                destination: format!("asset_groups/environment_package/tracks/{}/mipmaps/", t.code),
            }],
            false => Vec::new(),
        },
        ui: None,
        extra: Default::default(),
    };
    pkg.validate().map_err(|e| unusable(e.to_string()))?;
    Ok((dir, pkg))
}

/// The addons in `order` with a `track` record, in a data file or their lock: `track.add` only
/// places their files, and the core adds the rows.
pub fn with_records(order: &[String], std16: &BTreeMap<String, Std>) -> BTreeSet<String> {
    order
        .iter()
        .filter(|id| std16.get(*id).is_some_and(|s| s.has_records(TRACK_REGISTRY)))
        .cloned()
        .collect()
}

pub type BuildTrack = (PathBuf, TrackPackage, Origin);

/// The packages of the track records with a `files` folder that the addons in `order` add and
/// no `track.add` of theirs places, in load order, and the ones a build leaves out.
pub fn record_packages(
    store: &Store,
    order: &[String],
    std16: &BTreeMap<String, Std>,
    added: &[BuildTrack],
) -> (Vec<BuildTrack>, Vec<(Origin, Unusable)>) {
    let (mut out, mut left_out): (Vec<BuildTrack>, Vec<(Origin, Unusable)>) = (Vec::new(), Vec::new());
    for id in order {
        let Some(s) = std16.get(id).filter(|s| !s.data_files.is_empty()) else {
            continue;
        };
        let dir = store.addons_dir().join(id);
        let origin = Origin {
            addon: id.clone(),
            item: None,
        };
        for t in record_tracks(&dir, id, &s.data_files) {
            let placed = |list: &[BuildTrack]| {
                list.iter()
                    .find(|(_, p, _)| p.track_code == t.code)
                    .map(|(_, _, o)| o.addon.clone())
            };
            match placed(added).or_else(|| placed(&out)) {
                Some(by) if by == *id => continue,
                Some(by) => {
                    left_out.push((
                        origin.clone(),
                        Unusable {
                            code: Some(t.code.clone()),
                            name: Some(t.name.clone().unwrap_or_else(|| t.code.clone())),
                            why: format!("{by} adds a track with the code {} too", t.code),
                        },
                    ));
                    continue;
                }
                None => {}
            }
            if t.files.is_none() {
                continue;
            }
            match record_package(&dir, &t) {
                Ok((d, pkg)) => out.push((d, pkg, origin.clone())),
                Err(why) => left_out.push((origin.clone(), why)),
            }
        }
    }
    (out, left_out)
}

/// Leaves the folders of the build's track packages out of their addons' files for the index:
/// the build places a track's archive, mipmaps and menu art itself.
pub fn keep_track_folders(store: &Store, tracks: &[BuildTrack], std16: &mut BTreeMap<String, Std>) {
    for (dir, _, origin) in tracks {
        let Some(s) = std16.get_mut(&origin.addon) else {
            continue;
        };
        let Ok(rel) = dir.strip_prefix(store.addons_dir().join(&origin.addon)) else {
            continue;
        };
        let prefix = format!(
            "{}/",
            rel.to_string_lossy().replace('\\', "/").trim_end_matches('/')
        );
        if prefix == "/" {
            continue;
        }
        s.files.files.retain(|f| !f.starts_with(&prefix));
    }
}

/// Writes `$DATA/registry/older-tracks.json` for the tracks `track.add` added for addons with no
/// track record; `shipped` and `built` are the game's database and the build's.
pub fn write_older(
    data: &Path,
    profile: &str,
    added: &[(&str, &TrackPackage)],
    shipped: &Path,
    built: &Path,
) -> Result<()> {
    let path = crate::stdlib::reports::path(data, OLDER_FILE);
    let file = match added.is_empty() {
        true => OlderTracks {
            version: OLDER_VERSION,
            profile: profile.to_owned(),
            tracks: Vec::new(),
        },
        false => {
            let read = |p: &Path| -> Result<Gvdb> { Ok(Gvdb::parse(&std::fs::read(p).map_err(io_err(p))?)?) };
            older_tracks(profile, added, &read(shipped)?, &read(built)?, Venues::shipped())
        }
    };
    if crate::read_json::<OlderTracks>(&path).is_ok_and(|f| f == file) {
        return Ok(());
    }
    crate::write_json(&path, &file)
}

/// The tracks the last build recorded in `older-tracks.json`, for `profile`: their stat ids seed the ids the store keeps.
pub fn read_older(data: &Path, profile: &str) -> Vec<OlderTrack> {
    let path = crate::stdlib::reports::path(data, OLDER_FILE);
    crate::read_json::<OlderTracks>(&path)
        .ok()
        .filter(|f| f.profile == profile)
        .map(|f| f.tracks)
        .unwrap_or_default()
}

/// One venue of `packages/sdk/std/data/venues.json`.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Venue {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub stock: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Venues {
    #[serde(default)]
    pub venues: BTreeMap<String, Venue>,
}

/// Lower case, every run of characters other than `a-z0-9` one space, trimmed.
pub fn normalise(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.to_lowercase().chars() {
        match c.is_ascii_lowercase() || c.is_ascii_digit() {
            true => out.push(c),
            false if !out.ends_with(' ') => out.push(' '),
            false => {}
        }
    }
    out.trim().to_owned()
}

impl Venues {
    pub fn parse(text: &str) -> Option<Self> {
        serde_json::from_str(text).ok()
    }

    /// The venues the launcher ships (`packages/sdk/std/data/venues.json`).
    pub fn shipped() -> &'static Self {
        static PARSED: std::sync::OnceLock<Venues> = std::sync::OnceLock::new();
        PARSED.get_or_init(|| Self::parse(VENUES_JSON).unwrap_or_default())
    }

    /// `venue/<key>` of the one venue whose name or an alias equals `name` once normalised;
    /// `None` when none does or more than one does.
    pub fn by_name(&self, name: &str) -> Option<String> {
        let want = normalise(name);
        if want.is_empty() {
            return None;
        }
        let mut found = self.venues.iter().filter(|(_, v)| {
            std::iter::once(&v.name)
                .chain(&v.aliases)
                .any(|n| normalise(n) == want)
        });
        let (key, _) = found.next()?;
        found.next().is_none().then(|| format!("venue/{key}"))
    }

    /// Whether a stock track with assets provides `id` (`venue/<key>`).
    pub fn stock_provides(&self, id: &str) -> bool {
        id.strip_prefix("venue/")
            .and_then(|k| self.venues.get(k))
            .is_some_and(|v| v.stock.as_deref().is_some_and(|s| !s.is_empty()))
    }
}

/// One entry of `older-tracks.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OlderTrack {
    pub addon: String,
    pub code: String,
    pub name: String,
    pub base: String,
    pub slot: Option<String>,
    pub stat_id: Option<u32>,
    pub venue: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OlderTracks {
    pub version: u32,
    pub profile: String,
    pub tracks: Vec<OlderTrack>,
}

fn label_rows(db: &Gvdb) -> Vec<(String, Option<u32>)> {
    Rows::new(db)
        .table(TRACKS, &["_metadata_", "statId"])
        .into_iter()
        .filter_map(|r| Some((r[0].clone()?, r[1].as_deref().and_then(|s| s.parse().ok()))))
        .collect()
}

/// `older-tracks.json` for the tracks `track.add` added in a build, by addon id in the build's
/// order: `shipped` is the game's own database, `built` the build's. `slot` is the stock row a
/// track fills (`game/<code>`: a row the game ships with the track's code), else null; `base` is
/// the track its rows were cloned from.
pub fn older_tracks(
    profile: &str,
    added: &[(&str, &TrackPackage)],
    shipped: &Gvdb,
    built: &Gvdb,
    venues: &Venues,
) -> OlderTracks {
    let stock: Vec<String> = label_rows(shipped).into_iter().map(|(c, _)| c).collect();
    let ids = label_rows(built);
    let tracks = added
        .iter()
        .map(|(addon, pkg)| {
            let code = pkg.track_code.clone();
            let name = pkg.display_name().trim().to_owned();
            OlderTrack {
                addon: (*addon).to_owned(),
                venue: pkg.track_name.as_deref().and_then(|n| venues.by_name(n)),
                base: format!("game/{}", pkg.activation.base_track().unwrap_or(&code)),
                slot: stock.contains(&code).then(|| format!("game/{code}")),
                stat_id: ids.iter().find(|(c, _)| *c == code).and_then(|(_, id)| *id),
                name: match name.is_empty() {
                    true => code.clone(),
                    false => name,
                },
                code,
            }
        })
        .collect();
    OlderTracks {
        version: OLDER_VERSION,
        profile: profile.to_owned(),
        tracks,
    }
}

pub fn left_out_line(addon: &str, id: &str, why: &Unusable) -> String {
    match addon == id {
        true => format!("{id}: {}", why.text()),
        false => format!("{addon} ({id}): {}", why.text()),
    }
}

/// A [`left_out_line`] for each track package `m`'s op files add that builds leave out.
pub fn left_out_lines(store: &Store, m: &AddonManifest) -> Vec<String> {
    unusable(store, &m.id, m)
        .iter()
        .map(|u| left_out_line(&m.name, &m.id, u))
        .collect()
}

/// Puts each race weekend at a track a build left out back at its stock
/// track, and says so; a weekend the game doesn't ship is only named.
pub(crate) fn restore_weekends(db: &mut Gvdb, shipped: &Gvdb, left_out: &[&str]) -> Vec<String> {
    let text = |db: &Gvdb, path: &str| db.get_text(path).map(|t| t.trim().to_owned());
    let count = |db: &Gvdb, table: &str| {
        let (container, row) = table.rsplit_once('.').expect("a table path");
        db.instance_count(container, row).unwrap_or(0)
    };
    let mut gone: Option<Vec<&str>> = None;
    let mut out = Vec::new();
    for i in 0..count(db, ACTIVITIES) {
        let path = format!("{ACTIVITIES}_{i}.track");
        let Some(code) = text(db, &path).filter(|c| left_out.contains(&c.as_str())) else {
            continue;
        };
        let gone = gone.get_or_insert_with(|| {
            let have: Vec<String> = (0..count(db, TRACKS))
                .filter_map(|t| text(db, &format!("{TRACKS}_{t}._metadata_")))
                .collect();
            left_out
                .iter()
                .copied()
                .filter(|c| !have.iter().any(|h| h == c))
                .collect()
        });
        if !gone.contains(&code.as_str()) {
            continue;
        }
        let weekend = text(db, &format!("{ACTIVITIES}_{i}._metadata_")).unwrap_or_else(|| i.to_string());
        let stock = shipped
            .get(&path)
            .filter(|r| !left_out.contains(&r.text().trim()))
            .cloned();
        match stock {
            Some(r) => {
                let to = r.text().trim().to_owned();
                if db.set(&path, r.value, Some(r.tag)).is_ok() {
                    out.push(format!(
                        "race weekend {weekend}: races at {to} again; track {code} is left out of this build"
                    ));
                }
            }
            None => out.push(format!(
                "race weekend {weekend}: races at {code}, which is left out of this build; set its track in Race weekends"
            )),
        }
    }
    out
}

const TRACK_DATA: &str = "track_data.game_setup.TrackDataSet";
const GAME_MODES: &str = "game_setup.game_setup_modes.game_setup_mode";

/// A track a build's database names that its tracks table doesn't hold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MissingTrack {
    /// The record that names it.
    pub key: u64,
    /// The row that names it, such as `race weekend bahrain`.
    pub at: String,
    pub track: String,
    pub fix: MissingFix,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MissingFix {
    /// The record has its shipped track again.
    Stock(String),
    /// The track is out of the record's list.
    Dropped,
    /// The record is as it was: the game ships no track there the build holds.
    Kept,
}

impl MissingTrack {
    /// The build's warning, with the addon that set the record (`by`) and the addon
    /// that adds the track and is off (`adds`) when they are known.
    pub(crate) fn line(&self, by: Option<&str>, adds: Option<&str>) -> String {
        let (at, track) = (&self.at, &self.track);
        let head = match (by, &self.fix) {
            (Some(by), MissingFix::Dropped) => format!("{by} adds {track} to {at}"),
            (Some(by), _) => format!("{by} sets {at} to {track}"),
            (None, MissingFix::Dropped) => format!("{at} lists {track}"),
            (None, _) => format!("{at} is set to {track}"),
        };
        let fix = match adds {
            Some(addon) => format!("which is off: turn {addon} on"),
            None => "which isn't in this build".to_owned(),
        };
        match &self.fix {
            MissingFix::Stock(stock) => format!("{head}, {fix}; until then this build puts back {stock}"),
            MissingFix::Dropped => format!("{head}, {fix}; until then this build leaves it out"),
            MissingFix::Kept if adds.is_some() => {
                format!("{head}, {fix} or change it in DB Editor; the game won't start like this")
            }
            MissingFix::Kept => {
                format!("{head}, {fix}: change it in DB Editor; the game won't start like this")
            }
        }
    }
}

/// Points each record that names a track the build doesn't hold at its shipped track
/// again, or takes the track out of a record's list; a record the game ships with no
/// track the build holds is only named. The build holds its tracks table's tracks and
/// `in_game`, the tracks whose rows the game adds. A track the game ships the record
/// with is left as it is, and race weekends at a `left_out` track are
/// [`restore_weekends`]'s.
pub(crate) fn keep_track_refs(
    db: &mut Gvdb,
    shipped: &Gvdb,
    in_game: &[&str],
    left_out: &[&str],
) -> Vec<MissingTrack> {
    let at: HashMap<u64, usize> = db.records.iter().enumerate().map(|(i, r)| (r.key, i)).collect();
    let stock: HashMap<u64, &Record> = shipped.records.iter().map(|r| (r.key, r)).collect();
    let text = |db: &Gvdb, path: &str| {
        at.get(&fnv1a64(path))
            .map(|&i| db.records[i].text().trim().to_owned())
    };
    let count = |db: &Gvdb, table: &str| {
        let (container, row) = table.rsplit_once('.').expect("a table path");
        text(db, &swofty_formats::gvdb::instance_count_path(container, row))
            .and_then(|n| n.parse::<usize>().ok())
            .unwrap_or(0)
    };
    let label = |db: &Gvdb, table: &str, i: usize| {
        text(db, &format!("{table}_{i}._metadata_")).unwrap_or_else(|| i.to_string())
    };
    let mut have: HashSet<String> = (0..count(db, TRACKS))
        .filter_map(|i| text(db, &format!("{TRACKS}_{i}._metadata_")))
        .collect();
    if have.is_empty() {
        return Vec::new();
    }
    have.extend(in_game.iter().map(|c| (*c).to_owned()));
    let mut refs: Vec<(String, String, bool)> = Vec::new();
    for i in 0..count(db, ACTIVITIES) {
        let path = format!("{ACTIVITIES}_{i}.track");
        if text(db, &path).is_some_and(|c| left_out.contains(&c.as_str())) {
            continue;
        }
        refs.push((path, format!("race weekend {}", label(db, ACTIVITIES, i)), false));
    }
    for s in 0..count(db, TRACK_DATA) {
        let rows = format!("{TRACK_DATA}_{s}.TrackData");
        let set = label(db, TRACK_DATA, s);
        for i in 0..count(db, &rows) {
            let at = format!("track settings {set}/{}", label(db, &rows, i));
            refs.push((format!("{rows}_{i}.track"), at, false));
        }
    }
    for s in 0..count(db, CONTENT_SETS) {
        let rows = format!("{CONTENT_SETS}_{s}.available_tracks");
        let at = format!("content set {}", label(db, CONTENT_SETS, s));
        for i in 0..count(db, &rows) {
            refs.push((format!("{rows}_{i}.tracks"), at.clone(), true));
        }
    }
    for i in 0..count(db, GAME_MODES) {
        let at = format!("game mode {}", label(db, GAME_MODES, i));
        refs.push((format!("{GAME_MODES}_{i}.restricted_tracks"), at, true));
    }
    let mut out = Vec::new();
    for (path, row, list) in refs {
        let key = fnv1a64(&path);
        let Some(&i) = at.get(&key) else {
            continue;
        };
        let now = db.records[i].text().trim().to_owned();
        let shipped_text = stock
            .get(&key)
            .map(|r| r.text().trim().to_owned())
            .unwrap_or_default();
        if !list {
            if now.is_empty() || have.contains(&now) || now == shipped_text {
                continue;
            }
            let fix = match stock.get(&key).filter(|r| have.contains(r.text().trim())) {
                Some(r) => {
                    db.records[i].value = r.value.clone();
                    db.records[i].tag = r.tag;
                    MissingFix::Stock(r.text().trim().to_owned())
                }
                None => MissingFix::Kept,
            };
            out.push(MissingTrack {
                key,
                at: row,
                track: now,
                fix,
            });
            continue;
        }
        let codes: Vec<&str> = now.split(',').map(str::trim).filter(|c| !c.is_empty()).collect();
        let (kept, gone): (Vec<&str>, Vec<&str>) = codes
            .into_iter()
            .partition(|c| have.contains(*c) || shipped_text.split(',').any(|s| s.trim() == *c));
        if gone.is_empty() {
            continue;
        }
        db.records[i].value = kept.join(",").into_bytes();
        out.extend(gone.into_iter().map(|track| MissingTrack {
            key,
            at: row.clone(),
            track: track.to_owned(),
            fix: MissingFix::Dropped,
        }));
    }
    out
}

/// The installed addons, by track code, whose `track.add` or `track` record adds each of
/// `codes`, also where a build would leave the track out.
pub(crate) fn adders(store: &Store, codes: &BTreeSet<String>) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for (id, m) in store.addons().unwrap_or_default() {
        let dir = store.addons_dir().join(&id);
        let added = added_dirs(store, &id, &m)
            .iter()
            .filter_map(|d| match crate::map::read_package(d) {
                Ok(pkg) => Some(pkg.track_code),
                Err(why) => why.code,
            })
            .collect::<Vec<_>>();
        let recorded = record_tracks(&dir, &id, &crate::stdlib::data::files(&dir));
        for code in added.into_iter().chain(recorded.into_iter().map(|t| t.code)) {
            if codes.contains(&code) {
                out.entry(code).or_insert_with(|| id.clone());
            }
        }
    }
    out
}

/// The shipped records `db` holds with other values: what the tracks changed.
pub(crate) fn changed_records(db: &Gvdb, shipped: &Gvdb) -> Vec<Record> {
    let stock: HashMap<u64, &[u8]> = shipped
        .records
        .iter()
        .map(|r| (r.key, r.value.as_slice()))
        .collect();
    db.records
        .iter()
        .filter(|r| stock.get(&r.key).is_some_and(|v| *v != r.value.as_slice()))
        .cloned()
        .collect()
}

/// Puts back the track changes in `changed` that database ops undid. A mod
/// made from a whole game database carries the shipped value of every record
/// it holds, so a record an op set back to its shipped value gets the tracks'
/// value again, and a list an op rewrote otherwise gets back the entries the
/// tracks added to it. Returns the keys put back.
pub(crate) fn keep_changes(db: &mut Gvdb, shipped: &Gvdb, changed: &[Record]) -> Vec<u64> {
    let at: HashMap<u64, usize> = db.records.iter().enumerate().map(|(i, r)| (r.key, i)).collect();
    let stock: HashMap<u64, &Record> = shipped.records.iter().map(|r| (r.key, r)).collect();
    let mut kept = Vec::new();
    for ours in changed {
        let (Some(&i), Some(stock)) = (at.get(&ours.key), stock.get(&ours.key)) else {
            continue;
        };
        let now = &db.records[i];
        let value = if now.value == ours.value {
            None
        } else if now.value == stock.value {
            Some(ours.value.clone())
        } else if now.value_type() == ValueType::List && ours.value_type() == ValueType::List {
            merged_list(&now.value, &ours.value, &stock.value)
        } else {
            None
        };
        if let Some(value) = value {
            db.records[i].value = value;
            kept.push(ours.key);
        }
    }
    kept
}

fn merged_list(now: &[u8], ours: &[u8], stock: &[u8]) -> Option<Vec<u8>> {
    let items = |v: &[u8]| -> Vec<String> {
        String::from_utf8_lossy(v)
            .split(',')
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect()
    };
    let (mut out, shipped) = (items(now), items(stock));
    let added: Vec<String> = items(ours)
        .into_iter()
        .filter(|c| !shipped.contains(c) && !out.contains(c))
        .collect();
    if added.is_empty() {
        return None;
    }
    out.extend(added);
    Some(out.join(",").into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_changes_survive_ops_that_carry_shipped_values() {
        let list = |path: &str, v: &str| Record::new(path, 13, v);
        let int = |path: &str, v: &str| Record::new(path, 0, v);
        let shipped = Gvdb {
            records: vec![
                list("sets.a.tracks", "melbourne,suzuka"),
                list("sets.b.tracks", "melbourne"),
                list("sets.c.tracks", "melbourne"),
                int("tracks._Track_instancecount_", "2"),
                int("tracks.Track_0.statId", "5"),
            ],
        };
        let mut db = Gvdb {
            records: vec![
                list("sets.a.tracks", "melbourne,suzuka,testring"),
                list("sets.b.tracks", "melbourne,testring"),
                list("sets.c.tracks", "melbourne,testring"),
                int("tracks._Track_instancecount_", "3"),
                int("tracks.Track_0.statId", "6"),
                int("tracks.Track_2.statId", "50"),
            ],
        };
        let changed = changed_records(&db, &shipped);
        assert_eq!(changed.len(), 5);
        db.set("sets.a.tracks", "melbourne,suzuka", None).unwrap();
        db.set("sets.b.tracks", "suzuka,imola", None).unwrap();
        db.set("tracks._Track_instancecount_", "2", None).unwrap();
        db.set("tracks.Track_0.statId", "9", None).unwrap();
        let kept = keep_changes(&mut db, &shipped, &changed);
        assert_eq!(kept.len(), 3);
        assert_eq!(
            db.get_text("sets.a.tracks").as_deref(),
            Some("melbourne,suzuka,testring")
        );
        assert_eq!(
            db.get_text("sets.b.tracks").as_deref(),
            Some("suzuka,imola,testring")
        );
        assert_eq!(
            db.get_text("sets.c.tracks").as_deref(),
            Some("melbourne,testring")
        );
        assert_eq!(db.get_text("tracks._Track_instancecount_").as_deref(), Some("3"));
        assert_eq!(db.get_text("tracks.Track_0.statId").as_deref(), Some("9"));
    }

    fn rec(path: &str, value: &str) -> Record {
        Record::new(path, 2, value.as_bytes().to_vec())
    }

    fn db() -> Gvdb {
        let mut records = vec![
            rec("tracks._Track_instancecount_", "3"),
            rec("schedule.all_track_events._fixed_event_instancecount_", "3"),
            rec("schedule.activities._activity_race_weekend_instancecount_", "1"),
            rec("schedule.activities.activity_race_weekend_0._metadata_", "suzuka"),
            rec("schedule.activities.activity_race_weekend_0.track", "suzuka"),
            rec("content_sets._content_set_instancecount_", "2"),
            rec("content_sets.content_set_0._metadata_", "2024"),
            rec("content_sets.content_set_0._available_tracks_instancecount_", "1"),
            rec(
                "content_sets.content_set_0.available_tracks_0.tracks",
                "bahrain24",
            ),
            rec("content_sets.content_set_1._metadata_", "2026"),
            rec("content_sets.content_set_1._available_tracks_instancecount_", "1"),
            rec(
                "content_sets.content_set_1.available_tracks_0.tracks",
                "suzuka,okayama",
            ),
        ];
        for (i, (code, id, res)) in [
            ("suzuka", "13", "2019"),
            ("bahrain24", "35", "swofty_bahrain24"),
            ("okayama", "43", "swofty_okayama"),
        ]
        .into_iter()
        .enumerate()
        {
            records.push(rec(&format!("tracks.Track_{i}._metadata_"), code));
            records.push(rec(&format!("tracks.Track_{i}.statId"), id));
            records.push(rec(&format!("tracks.Track_{i}.track_resource"), res));
        }
        for (i, (name, activity)) in [
            ("rw_suzuka", "suzuka"),
            ("rw_suzuka_26", "suzuka"),
            ("rw_gone", "gone"),
        ]
        .into_iter()
        .enumerate()
        {
            records.push(rec(
                &format!("schedule.all_track_events.fixed_event_{i}._metadata_"),
                name,
            ));
            records.push(rec(
                &format!("schedule.all_track_events.fixed_event_{i}.activity"),
                activity,
            ));
        }
        Gvdb { records }
    }

    #[test]
    fn events_resolve_through_activities_and_packages_name_modded_tracks() {
        let added = [PlanTrack {
            code: "okayama".into(),
            name: Some("Okayama".into()),
            addon: Some("teamsimplified.mods".into()),
        }];
        let tracks = list(&db(), &added);
        let by = |c: &str| tracks.iter().find(|t| t.code == c).unwrap();
        assert_eq!(tracks.len(), 3);
        assert_eq!(by("suzuka").stat_id, 13);
        assert_eq!(by("suzuka").events, ["rw_suzuka", "rw_suzuka_26"]);
        assert!(!by("suzuka").modded);
        assert!(by("suzuka").playable && by("okayama").playable && !by("bahrain24").playable);
        assert!(
            by("bahrain24").modded && by("bahrain24").name.is_none() && by("bahrain24").events.is_empty()
        );
        let o = by("okayama");
        assert_eq!(
            (o.stat_id, o.modded, o.name.as_deref()),
            (43, true, Some("Okayama"))
        );
        assert_eq!(o.addon.as_deref(), Some("teamsimplified.mods"));
    }

    #[test]
    fn lists_the_tracks_installed_addons_add() {
        let root = std::env::temp_dir().join(format!("swofty-installed-tracks-{}", std::process::id()));
        std::fs::remove_dir_all(&root).ok();
        let store = Store::new(&root);
        let addon = |id: &str, ops: &str, package: Option<String>| {
            let dir = store.addons_dir().join(id);
            std::fs::create_dir_all(dir.join("ops")).unwrap();
            std::fs::write(
                dir.join("swofty.json"),
                format!(
                    r#"{{"schema_version":1,"id":"{id}","name":"{id}","version":"1.0.0","author":"t",
                    "compat":{{"game":"f1_25"}},"operations":{{"assets":["ops/track.json"]}}}}"#
                ),
            )
            .unwrap();
            std::fs::write(dir.join("ops/track.json"), ops).unwrap();
            if let Some(package) = package {
                std::fs::create_dir_all(dir.join("track")).unwrap();
                std::fs::write(dir.join("track/manifest.json"), package).unwrap();
            }
        };
        let add = r#"{"schema_version":1,"ops":[{"op":"track.add","source":"track"}]}"#;
        let like = |code: &str, name: &str| {
            crate::map::tests::OKAYAMA_LIKE
                .replace(
                    r#""track_code": "testring""#,
                    &format!(r#""track_code": "{code}""#),
                )
                .replace(
                    r#""track_name": "Test Ring""#,
                    &format!(r#""track_name": "{name}""#),
                )
        };
        addon("mod.bahrain24", add, Some(like("bahrain24", "Sepang")));
        addon("mod.okayama", add, Some(like("okayama", "")));
        addon("mod.twice", add, Some(like("okayama", "Again")));
        addon("mod.unread", add, None);
        addon("mod.other", r#"{"schema_version":1,"ops":[]}"#, None);
        addon(
            "mod.portimao",
            add,
            Some(r#"{ "track_code": "portimao", "track_name": "Portimao", "erp": "portimao.erp" }"#.into()),
        );
        let found = installed(&store);
        assert_eq!(
            found,
            [
                PlanTrack {
                    code: "bahrain24".into(),
                    name: Some("Sepang".into()),
                    addon: Some("mod.bahrain24".into()),
                },
                PlanTrack {
                    code: "okayama".into(),
                    name: None,
                    addon: Some("mod.okayama".into()),
                },
            ]
        );
        let addons = store.addons().unwrap();
        let bad = |id: &str| unusable(&store, id, &addons[id]);
        assert!(bad("mod.okayama").is_empty() && bad("mod.other").is_empty());
        assert_eq!(bad("mod.unread")[0].why, "it has no manifest.json");
        let portimao = bad("mod.portimao");
        assert_eq!(portimao.len(), 1);
        assert_eq!(portimao[0].why, crate::map::NO_BASE_TRACK);
        assert_eq!(
            left_out_line("Portimao", "mod.portimao", &portimao[0]),
            "Portimao (mod.portimao): track Portimao is left out; its manifest.json doesn't say which track \
             it replaces or copies. Import a fixed version, or ask its author for one"
        );
        assert!(
            left_out_line("mod.portimao", "mod.portimao", &portimao[0]).starts_with("mod.portimao: track ")
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn venues_match_a_track_name_never_its_code() {
        let v = Venues::shipped();
        assert!(v.venues.len() >= 33, "{}", v.venues.len());
        assert_eq!(
            normalise("  Autódromo José-Carlos  Pace "),
            "aut dromo jos carlos pace"
        );
        assert_eq!(normalise("Spa-Francorchamps"), "spa francorchamps");
        assert_eq!(v.by_name("Sepang").as_deref(), Some("venue/sepang"));
        assert_eq!(
            v.by_name("SEPANG international circuit").as_deref(),
            Some("venue/sepang")
        );
        assert_eq!(v.by_name("Okayama").as_deref(), Some("venue/okayama"));
        assert_eq!(v.by_name("Delhi").as_deref(), Some("venue/buddh"));
        assert_eq!(v.by_name("Igora Drive").as_deref(), Some("venue/igora_drive"));
        assert_eq!(v.by_name("bahrain24"), None);
        assert_eq!(v.by_name(""), None);
        assert_eq!(v.by_name("Test Ring"), None);
        let twice = Venues::parse(
            r#"{ "version": 1, "venues": {
                "a": { "name": "Ring", "country": "x", "aliases": [], "stock": null },
                "b": { "name": "Other", "country": "x", "aliases": ["ring"], "stock": "b" } } }"#,
        )
        .unwrap();
        assert_eq!(twice.by_name("Ring"), None, "two venues match: none");
        assert_eq!(twice.by_name("other").as_deref(), Some("venue/b"));
        assert!(twice.stock_provides("venue/b") && !twice.stock_provides("venue/a"));
        assert!(!twice.stock_provides("venue/zzz") && !twice.stock_provides("b"));
        assert!(v.stock_provides("venue/albert_park") && !v.stock_provides("venue/sochi"));
    }

    #[test]
    fn older_tracks_name_their_base_the_row_they_fill_their_stat_id_and_venue() {
        let pkg = |code: &str, name: Option<&str>, from: &str, slot: Option<&str>| TrackPackage {
            track_code: code.into(),
            track_name: name.map(str::to_owned),
            version: None,
            author_name: None,
            erp: format!("{code}.erp"),
            activation: crate::map::Activation {
                copy_from_track: Some(from.into()),
                copy_slot: slot.map(str::to_owned),
                ..Default::default()
            },
            ers: Default::default(),
            copy_directories: Vec::new(),
            ui: None,
            extra: Default::default(),
        };
        let shipped = {
            let mut d = db();
            d.set("tracks._Track_instancecount_", "4", None).unwrap();
            d.records.push(rec("tracks.Track_3._metadata_", "sochi"));
            d.records.push(rec("tracks.Track_3.statId", "30"));
            d
        };
        let mut built = shipped.clone();
        built.set("tracks._Track_instancecount_", "5", None).unwrap();
        built.records.push(rec("tracks.Track_4._metadata_", "okayama2"));
        built.records.push(rec("tracks.Track_4.statId", "44"));
        let okayama = pkg("okayama2", Some("Okayama"), "melbourne", Some("melbourne"));
        let sochi = pkg("sochi", Some("Sochi Autodrom"), "baku", None);
        let unnamed = pkg("ring", None, "zandvoort", Some("zandvoort"));
        let out = older_tracks(
            "Classic Seasons",
            &[
                ("mod.okayama", &okayama),
                ("mod.sochi", &sochi),
                ("mod.ring", &unnamed),
            ],
            &shipped,
            &built,
            Venues::shipped(),
        );
        assert_eq!((out.version, out.profile.as_str()), (1, "Classic Seasons"));
        let entry = |addon: &str,
                     code: &str,
                     name: &str,
                     base: &str,
                     slot: Option<&str>,
                     id: Option<u32>,
                     venue: Option<&str>| OlderTrack {
            addon: addon.into(),
            code: code.into(),
            name: name.into(),
            base: base.into(),
            slot: slot.map(str::to_owned),
            stat_id: id,
            venue: venue.map(str::to_owned),
        };
        assert_eq!(
            out.tracks,
            [
                entry(
                    "mod.okayama",
                    "okayama2",
                    "Okayama",
                    "game/melbourne",
                    None,
                    Some(44),
                    Some("venue/okayama")
                ),
                entry(
                    "mod.sochi",
                    "sochi",
                    "Sochi Autodrom",
                    "game/baku",
                    Some("game/sochi"),
                    Some(30),
                    Some("venue/sochi")
                ),
                entry("mod.ring", "ring", "ring", "game/zandvoort", None, None, None),
            ]
        );
        let json = serde_json::to_value(&out).unwrap();
        assert_eq!(json["tracks"][2]["slot"], serde_json::Value::Null);
        assert_eq!(json["tracks"][2]["venue"], serde_json::Value::Null);
    }

    #[test]
    fn track_records_become_packages_from_their_files_folder() {
        let root = std::env::temp_dir().join(format!("swofty-record-tracks-{}", std::process::id()));
        std::fs::remove_dir_all(&root).ok();
        std::fs::create_dir_all(root.join("registry")).unwrap();
        std::fs::create_dir_all(root.join("track/mipmaps")).unwrap();
        std::fs::create_dir_all(root.join("copy")).unwrap();
        std::fs::write(
            root.join("registry/track.json"),
            r#"{ "track": {
                "ring": { "code": "ring", "name": "Ring", "base": "game/austria", "files": "track/" },
                "copy": { "code": "testring", "base": "game/melbourne", "files": "copy/" },
                "nofiles": { "code": "nofiles", "base": "game/melbourne" },
                "outside": { "code": "outside", "base": "game/melbourne", "files": "../x/" },
                "$select": [],
                "game/melbourne": { "name": "a layer" },
                "nocode": { "base": "game/melbourne" } },
              "track.data": { "mod.x/ring/f1_2025": { "laps": 3 } } }"#,
        )
        .unwrap();
        std::fs::write(root.join("track/ring.erp"), b"erp").unwrap();
        std::fs::write(root.join("copy/manifest.json"), crate::map::tests::OKAYAMA_LIKE).unwrap();
        let found = record_tracks(&root, "mod.x", &["registry/track.json".into()]);
        let ids: Vec<&str> = found.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(
            ids,
            ["mod.x/copy", "mod.x/nofiles", "mod.x/outside", "mod.x/ring"]
        );
        let by = |id: &str| found.iter().find(|t| t.id == id).unwrap();
        let (dir, ring) = record_package(&root, by("mod.x/ring")).unwrap();
        assert_eq!(dir, root.join("track"));
        assert_eq!(
            (
                ring.track_code.as_str(),
                ring.track_name.as_deref(),
                ring.erp.as_str()
            ),
            ("ring", Some("Ring"), "ring.erp")
        );
        assert_eq!(ring.activation.base_track(), Some("austria"));
        assert_eq!(ring.copy_directories.len(), 1);
        let (_, copy) = record_package(&root, by("mod.x/copy")).unwrap();
        assert_eq!(copy.track_code, "testring");
        assert!(
            record_package(&root, by("mod.x/nofiles"))
                .unwrap_err()
                .why
                .contains("names no files")
        );
        assert!(
            record_package(&root, by("mod.x/outside"))
                .unwrap_err()
                .why
                .contains("leaves")
        );
        std::fs::remove_file(root.join("track/ring.erp")).unwrap();
        assert!(
            record_package(&root, by("mod.x/ring"))
                .unwrap_err()
                .why
                .contains("has no ring.erp")
        );
        let wrong = RecordTrack {
            code: "other".into(),
            ..by("mod.x/copy").clone()
        };
        assert!(
            record_package(&root, &wrong)
                .unwrap_err()
                .why
                .contains("is for testring")
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn weekends_at_a_left_out_track_go_back_to_their_stock_track() {
        let weekend = |i: usize, label: &str, track: &str| {
            [
                rec(&format!("{ACTIVITIES}_{i}._metadata_"), label),
                rec(&format!("{ACTIVITIES}_{i}.track"), track),
            ]
        };
        let mut shipped = db();
        shipped
            .set(
                "schedule.activities._activity_race_weekend_instancecount_",
                "2",
                None,
            )
            .unwrap();
        shipped.records.extend(weekend(1, "rw_okayama", "okayama"));
        let mut built = shipped.clone();
        built
            .set(
                "schedule.activities._activity_race_weekend_instancecount_",
                "4",
                None,
            )
            .unwrap();
        built
            .set(&format!("{ACTIVITIES}_0.track"), "portimao", None)
            .unwrap();
        built.records.extend(weekend(2, "rw_added", "portimao"));
        built.records.extend(weekend(3, "rw_other", "monza"));
        let warnings = restore_weekends(&mut built, &shipped, &["portimao", "okayama"]);
        assert_eq!(
            warnings,
            [
                "race weekend suzuka: races at suzuka again; track portimao is left out of this build",
                "race weekend rw_added: races at portimao, which is left out of this build; set its track in Race weekends",
            ]
        );
        let track = |i: usize| built.get_text(&format!("{ACTIVITIES}_{i}.track")).unwrap();
        assert_eq!(
            [track(0), track(1), track(2), track(3)],
            ["suzuka", "okayama", "portimao", "monza"]
        );
        let mut untouched = shipped.clone();
        assert!(restore_weekends(&mut untouched, &shipped, &["monza"]).is_empty());
    }

    #[test]
    fn records_naming_a_track_not_in_the_build_get_their_shipped_track_back() {
        let mut shipped = db();
        shipped
            .set(
                "schedule.activities._activity_race_weekend_instancecount_",
                "2",
                None,
            )
            .unwrap();
        shipped.records.extend([
            rec(&format!("{ACTIVITIES}_1._metadata_"), "okayama"),
            rec(&format!("{ACTIVITIES}_1.track"), "okayama"),
            rec("track_data.game_setup._TrackDataSet_instancecount_", "1"),
            rec(&format!("{TRACK_DATA}_0._metadata_"), "f1"),
            rec(&format!("{TRACK_DATA}_0._TrackData_instancecount_"), "1"),
            rec(&format!("{TRACK_DATA}_0.TrackData_0._metadata_"), "suzuka"),
            rec(&format!("{TRACK_DATA}_0.TrackData_0.track"), "suzuka"),
            rec("game_setup.game_setup_modes._game_setup_mode_instancecount_", "1"),
            rec(&format!("{GAME_MODES}_0._metadata_"), "f1"),
            rec(
                &format!("{GAME_MODES}_0.restricted_tracks"),
                "okayama,shipped_only",
            ),
        ]);
        let mut untouched = shipped.clone();
        assert!(keep_track_refs(&mut untouched, &shipped, &[], &[]).is_empty());
        let mut built = shipped.clone();
        let mut set = |path: &str, value: &str| built.set(path, value, None).unwrap();
        set("schedule.activities._activity_race_weekend_instancecount_", "4");
        set(&format!("{ACTIVITIES}_0.track"), "istanbul_park");
        set(&format!("{TRACK_DATA}_0.TrackData_0.track"), "paul_ricard");
        set(
            "content_sets.content_set_1.available_tracks_0.tracks",
            "suzuka,istanbul_park,okayama",
        );
        set(
            &format!("{GAME_MODES}_0.restricted_tracks"),
            "okayama,shipped_only,hockenheim,ring",
        );
        built.records.extend([
            rec(&format!("{ACTIVITIES}_2._metadata_"), "rw_added"),
            rec(&format!("{ACTIVITIES}_2.track"), "hockenheim"),
            rec(&format!("{ACTIVITIES}_3._metadata_"), "rw_left"),
            rec(&format!("{ACTIVITIES}_3.track"), "portimao"),
        ]);
        let missing = keep_track_refs(&mut built, &shipped, &["ring"], &["portimao"]);
        let got: Vec<(&str, &str, &MissingFix)> = missing
            .iter()
            .map(|m| (m.at.as_str(), m.track.as_str(), &m.fix))
            .collect();
        assert_eq!(
            got,
            [
                (
                    "race weekend suzuka",
                    "istanbul_park",
                    &MissingFix::Stock("suzuka".into())
                ),
                ("race weekend rw_added", "hockenheim", &MissingFix::Kept),
                (
                    "track settings f1/suzuka",
                    "paul_ricard",
                    &MissingFix::Stock("suzuka".into())
                ),
                ("content set 2026", "istanbul_park", &MissingFix::Dropped),
                ("game mode f1", "hockenheim", &MissingFix::Dropped),
            ]
        );
        assert_eq!(missing[0].key, fnv1a64(format!("{ACTIVITIES}_0.track")));
        let text = |path: &str| built.get_text(path).unwrap();
        assert_eq!(text(&format!("{ACTIVITIES}_0.track")), "suzuka");
        assert_eq!(text(&format!("{ACTIVITIES}_2.track")), "hockenheim");
        assert_eq!(text(&format!("{ACTIVITIES}_3.track")), "portimao");
        assert_eq!(text(&format!("{TRACK_DATA}_0.TrackData_0.track")), "suzuka");
        assert_eq!(
            text("content_sets.content_set_1.available_tracks_0.tracks"),
            "suzuka,okayama"
        );
        assert_eq!(
            text(&format!("{GAME_MODES}_0.restricted_tracks")),
            "okayama,shipped_only,ring"
        );
        let again = keep_track_refs(&mut built, &shipped, &["ring"], &["portimao"]);
        assert_eq!(again, [missing[1].clone()]);
        assert!(keep_track_refs(&mut Gvdb::default(), &shipped, &[], &[]).is_empty());
    }

    #[test]
    fn missing_track_lines_name_the_addon_that_set_it_and_the_one_to_turn_on() {
        let missing = |at: &str, fix: MissingFix| MissingTrack {
            key: 0,
            at: at.into(),
            track: "istanbul_park".into(),
            fix,
        };
        let weekend = missing("race weekend bahrain", MissingFix::Stock("bahrain".into()));
        assert_eq!(
            weekend.line(Some("My Calendar"), Some("Istanbul Park")),
            "My Calendar sets race weekend bahrain to istanbul_park, which is off: turn Istanbul Park on; \
             until then this build puts back bahrain"
        );
        assert_eq!(
            weekend.line(None, None),
            "race weekend bahrain is set to istanbul_park, which isn't in this build; until then this build puts \
             back bahrain"
        );
        assert_eq!(
            missing("content set 2026", MissingFix::Dropped).line(Some("My Calendar"), Some("Istanbul Park")),
            "My Calendar adds istanbul_park to content set 2026, which is off: turn Istanbul Park on; until then \
             this build leaves it out"
        );
        let added = missing("race weekend rw_added", MissingFix::Kept);
        assert_eq!(
            added.line(Some("My Calendar"), Some("Istanbul Park")),
            "My Calendar sets race weekend rw_added to istanbul_park, which is off: turn Istanbul Park on or \
             change it in DB Editor; the game won't start like this"
        );
        assert_eq!(
            added.line(Some("My Calendar"), None),
            "My Calendar sets race weekend rw_added to istanbul_park, which isn't in this build: change it in DB \
             Editor; the game won't start like this"
        );
    }

    #[test]
    fn adders_name_the_addon_whose_track_record_adds_a_track() {
        let root = std::env::temp_dir().join(format!("swofty-track-adders-{}", std::process::id()));
        std::fs::remove_dir_all(&root).ok();
        let store = Store::new(&root);
        let dir = store.addons_dir().join("track.testring");
        std::fs::create_dir_all(dir.join("registry")).unwrap();
        std::fs::write(
            dir.join("swofty.json"),
            r#"{"schema_version":1,"id":"track.testring","name":"Testring","version":"1.0.0","author":"t",
            "compat":{"game":"f1_25"}}"#,
        )
        .unwrap();
        std::fs::write(
            dir.join("registry/track.json"),
            r#"{"track":{"testring":{"code":"testring","name":"Testring","files":"track/"}}}"#,
        )
        .unwrap();
        let codes: BTreeSet<String> = ["testring".to_owned(), "nowhere".to_owned()].into();
        let got = adders(&store, &codes);
        std::fs::remove_dir_all(&root).ok();
        assert_eq!(
            got,
            BTreeMap::from([("testring".to_owned(), "track.testring".to_owned())])
        );
    }
}
