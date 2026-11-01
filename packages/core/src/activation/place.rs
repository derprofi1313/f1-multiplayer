use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use md5::{Digest as _, Md5};
use swofty_formats::erp::{Entry, ErpFile, Resource, STORED, ZSTD, compress};
use swofty_formats::gvdb::Gvdb;
use swofty_formats::pkg::{Node, Pkg};

use super::ailine::{self, Spacing};
use super::grid;
use crate::map::TrackPackage;
use crate::{Error, Result, io_err};

pub const ASSET_ROOT: &str = "2025_asset_groups";
pub const SLOT_DIR: &str = "ui_package/texture_arrays/invitational_events/event_images";
pub const SLOTS: &[&str] = &[
    "invitational_image_4",
    "invitational_image_10",
    "invitational_image_13",
    "invitational_image_14",
    "invitational_image_15",
    "invitational_image_16",
    "invitational_image_17",
    "invitational_image_18",
];
const EAID: &str = "eaid://";
const PLACEMENT_VERSION: u32 = 3;
const GRID_WORLD: &str = "f1_26_myteam";

pub const WORLD_FALLBACKS: &[(&str, &[&str])] = &[
    ("f1", &["f1_26", "f1_myteam"]),
    ("f1_myteam", &["f1"]),
    ("classic", &["f1", "f1_26"]),
    ("f1_story", &["f1"]),
    ("f1_previous_story", &["f1_story", "f1"]),
    ("f1_previous_myteam", &["f1_myteam", "f1"]),
    ("f2", &["f1"]),
    ("f2_previous", &["f2", "f1"]),
    ("f1_26", &["f1"]),
    ("f1_26_myteam", &["f1_26", "f1_myteam", "f1"]),
    ("f2_26", &["f2", "f1_26", "f1"]),
];

pub fn slot_file(slot: &str) -> String {
    format!("{ASSET_ROOT}/{SLOT_DIR}/{slot}.erp")
}

pub fn slot_asset_group(slot: &str) -> String {
    format!("asset_groups/{SLOT_DIR}/{slot}.assetgroup")
}

pub fn slot_asset_root() -> String {
    format!("asset_groups/{SLOT_DIR}/")
}

pub fn track_file(code: &str) -> String {
    format!("{ASSET_ROOT}/environment_package/tracks/{code}/{code}.erp")
}

pub fn track_group(code: &str, rel: &str) -> String {
    format!("asset_groups/environment_package/tracks/{code}/{rel}.assetgroup")
}

pub fn track_groups<'a>(code: &str, names: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let prefix = format!("{EAID}environment_package/tracks/{code}/");
    let mut worlds = BTreeSet::new();
    let mut garages = BTreeSet::new();
    let mut ai = BTreeSet::new();
    for name in names {
        let Some(rest) = name.strip_prefix(&prefix) else {
            continue;
        };
        let rest = rest.split('?').next().unwrap_or(rest);
        if let Some(v) = rest
            .strip_prefix("wep/")
            .and_then(|w| w.strip_prefix(code))
            .and_then(|w| w.strip_prefix('_'))
            .and_then(|w| w.strip_suffix(".wep"))
        {
            worlds.insert(format!("wep/{code}_{v}"));
        } else if let Some(n) = rest
            .strip_prefix("probes/probes_garage_")
            .and_then(|p| p.strip_suffix(".shprobes"))
        {
            garages.insert(format!("probes/probes_garage_{n}"));
        } else if let Some([package, team, _]) = rest
            .strip_prefix("ai/")
            .map(|a| a.splitn(3, '/').collect::<Vec<_>>())
            .as_deref()
        {
            ai.insert(format!("ai/{package}/{team}/{team}"));
        }
    }
    let template = [
        format!("bake/{code}_ao.bake"),
        format!("bake/{code}_light_list.bake"),
        format!("bake/{code}_radiance.bake"),
        "probes/config.ddgi".into(),
        "probes/key_2019".into(),
    ];
    std::iter::once(code.to_owned())
        .chain(worlds)
        .chain(template)
        .chain(garages)
        .chain(ai)
        .map(|rel| track_group(code, &rel))
        .collect()
}

pub fn mip_dir(code: &str) -> String {
    format!("environment_package/tracks/{code}/mipmaps")
}

fn world_eaid(code: &str, variant: &str) -> String {
    format!("{EAID}environment_package/tracks/{code}/wep/{code}_{variant}.wep")
}

fn world_variant<'a>(code: &str, eaid: &'a str) -> Option<&'a str> {
    let prefix = format!("{EAID}environment_package/tracks/{code}/wep/{code}_");
    eaid.strip_prefix(&prefix)?.strip_suffix(".wep")
}

const FILENAME: &str = "[FILENAME]";
pub const TRACK_LAYERS: &str = "track_data.track_layers";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LayerWorld {
    pub group: String,
    pub eri: String,
}

impl LayerWorld {
    pub fn new(group: &str, eri: &str) -> Option<Self> {
        let group_ok = group.starts_with("asset_groups/") && group.ends_with(".assetgroup");
        let eri_ok = eri.starts_with(EAID) && eri.ends_with(".wep") && !eri.contains('?');
        let stock = std::iter::once("common")
            .chain(WORLD_FALLBACKS.iter().map(|(v, _)| *v))
            .any(|v| eri == world_eaid(FILENAME, v));
        (group_ok && eri_ok && !stock && group.contains(FILENAME) && eri.contains(FILENAME)).then(|| Self {
            group: group.to_owned(),
            eri: eri.to_owned(),
        })
    }

    pub fn group(&self, code: &str) -> String {
        self.group.replace(FILENAME, code)
    }

    pub fn eri(&self, code: &str) -> String {
        self.eri.replace(FILENAME, code)
    }

    fn label(&self, code: &str) -> String {
        let eri = self.eri(code);
        eri[EAID.len()..eri.len() - ".wep".len()].to_owned()
    }

    fn name(&self, code: &str) -> String {
        let label = self.label(code);
        label.rsplit('/').next().unwrap_or(&label).to_owned()
    }
}

pub fn layer_worlds(db: &Gvdb) -> Vec<LayerWorld> {
    let n = db.instance_count(TRACK_LAYERS, "TrackLayer").unwrap_or(0);
    let worlds: BTreeSet<LayerWorld> = (0..n)
        .filter_map(|i| {
            let text = |f: &str| db.get_text(&format!("{TRACK_LAYERS}.TrackLayer_{i}.{f}"));
            LayerWorld::new(&text("worldAssetGroup")?, &text("worldEri")?)
        })
        .collect();
    worlds.into_iter().collect()
}

pub fn missing_worlds(worlds: &[LayerWorld], code: &str, provided: impl Fn(&str) -> bool) -> Vec<LayerWorld> {
    worlds
        .iter()
        .filter(|w| !provided(&w.group(code)))
        .cloned()
        .collect()
}

pub struct PlaceInput<'a> {
    pub pkg: &'a TrackPackage,
    pub dir: &'a Path,
    pub game: &'a Path,
    pub slot: Option<(&'a str, &'a Path)>,
    pub cars: u32,
    pub worlds: &'a [LayerWorld],
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Placement {
    pub dir: PathBuf,
    pub archive: String,
    pub groups: Vec<String>,
    pub replaced: BTreeMap<String, PathBuf>,
    pub added: BTreeMap<String, PathBuf>,
    pub aliases: BTreeMap<String, String>,
    pub mips: usize,
    pub grid: Option<grid::Extension>,
    pub repaired: Vec<String>,
    pub spaced: Vec<String>,
    pub notes: Vec<String>,
}

fn alias_world(
    file: &ErpFile,
    world: &Resource,
    graph: &Resource,
    eaid: String,
    name: String,
) -> Result<[Resource; 2]> {
    let mut graph = file.load(graph)?;
    graph.name = format!("{eaid}?event_graph");
    let mut world = file.load(world)?;
    let data = world.fragments[0].data()?;
    let mut pkg = Pkg::parse(&data)?;
    let root = pkg
        .root
        .as_obj_mut()
        .ok_or_else(|| Error::Map(format!("{}: world root is not an object", world.name)))?;
    for (key, value) in [("name", name), ("event_graph", graph.name.clone())] {
        match root.get_mut(key).and_then(Node::as_val_mut) {
            Some(v) if v.as_str().is_some() => v.set_str(value),
            _ => return Err(Error::Map(format!("{}: world has no {key} string", world.name))),
        }
    }
    set_data(&mut world, 0, &pkg.to_bytes()?)?;
    world.name = eaid;
    Ok([world, graph])
}

fn set_data(res: &mut Resource, index: usize, data: &[u8]) -> Result<()> {
    let f = res
        .fragments
        .get_mut(index)
        .ok_or_else(|| Error::Map(format!("{}: no fragment {index}", res.name)))?;
    let zstd =
        f.compression == ZSTD || (f.compression != STORED && f.raw.starts_with(&[0x28, 0xb5, 0x2f, 0xfd]));
    f.raw = if zstd { compress(data)? } else { data.to_vec() };
    f.size = data.len() as u64;
    f.packed = f.raw.len() as u64;
    res.hash = res.content_hash()?;
    Ok(())
}

pub fn shipped_name(path: &str) -> String {
    let hex = crate::baseline::hex(&Md5::digest(path.as_bytes()));
    let base = path.rsplit('/').next().unwrap_or(path);
    format!("{}_{base}", &hex[..12])
}

fn repoint_mips(
    file: &ErpFile,
    res: &Resource,
    code: &str,
    shipped: &BTreeSet<String>,
) -> Result<Option<(Resource, String)>> {
    if res.kind() != "GfxSurfaceRes" || res.fragments.len() < 3 {
        return Ok(None);
    }
    let mut loaded = file.load(res)?;
    let data = loaded.fragments[2].data()?;
    let Some(&len) = data.first() else {
        return Ok(None);
    };
    let len = len as usize;
    if len == 0 || data.len() < 1 + len {
        return Ok(None);
    }
    let path = String::from_utf8_lossy(&data[1..1 + len])
        .trim_end_matches('\0')
        .to_owned();
    let base = path.rsplit('/').next().unwrap_or(&path).to_owned();
    let name = if shipped.contains(&base) {
        base
    } else {
        let hashed = shipped_name(&path);
        if !shipped.contains(&hashed) {
            return Ok(None);
        }
        hashed
    };
    let target = format!("{}/{name}", mip_dir(code));
    let nul = data[1..1 + len].ends_with(&[0]);
    let mut text = target.into_bytes();
    if nul {
        text.push(0);
    }
    if text.len() > u8::MAX as usize {
        return Err(Error::Map(format!("{}: mip path is too long", res.name)));
    }
    let mut out = Vec::with_capacity(data.len() + text.len());
    out.push(text.len() as u8);
    out.extend_from_slice(&text);
    out.extend_from_slice(&data[1 + len..]);
    set_data(&mut loaded, 2, &out)?;
    Ok(Some((loaded, name)))
}

fn stamp(input: &PlaceInput) -> Result<String> {
    let erp = input.dir.join(&input.pkg.erp);
    let meta = std::fs::metadata(&erp).map_err(io_err(&erp))?;
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos());
    let slot = match input.slot {
        Some((name, original)) => format!("{name}|{}", crate::baseline::sha256_file(original)?),
        None => "natural".into(),
    };
    let manifest = serde_json::to_vec(input.pkg).expect("serialisable");
    let mut h = <sha2::Sha256 as sha2::Digest>::new();
    sha2::Digest::update(&mut h, &manifest);
    sha2::Digest::update(
        &mut h,
        format!(
            "{}|{}|{mtime}|{slot}|{PLACEMENT_VERSION}|{}",
            erp.display(),
            meta.len(),
            input.cars
        ),
    );
    for w in input.worlds {
        sha2::Digest::update(&mut h, format!("|{}|{}", w.group, w.eri));
    }
    Ok(crate::baseline::hex(&sha2::Digest::finalize(h))[..16].to_owned())
}

pub fn place(input: &PlaceInput, cache: &Path) -> Result<Placement> {
    let code = &input.pkg.track_code;
    let file = ErpFile::open(&input.dir.join(&input.pkg.erp))?;
    let common = world_eaid(code, "common");
    if !file.erp.resources.iter().any(|r| r.name == common) {
        return Err(Error::Map(format!(
            "{} has no {common}; the track folder must match the track code",
            input.pkg.erp
        )));
    }
    let who = input.pkg.display_name();
    let fix = format!(
        "Turn {who} off in this profile, and set any race weekend that uses it back to another track."
    );
    let (fixed, broken) = crate::erpbuild::repair_stats_in(&file)?;
    if !broken.is_empty() {
        return Err(Error::Other(crate::erpbuild::unrepairable_message(
            who, &broken, &fix,
        )));
    }
    let stamp = stamp(input)?;
    let out = cache.join(match fixed.is_empty() {
        true => stamp,
        false => format!("{stamp}-stats"),
    });
    let mut placement = Placement {
        dir: out.clone(),
        repaired: fixed.iter().map(|(_, r)| r.name.clone()).collect(),
        ..Placement::default()
    };
    let shipped: BTreeMap<String, PathBuf> = input
        .pkg
        .copy_directories
        .iter()
        .flat_map(|c| walk(&input.dir.join(c.source.trim_end_matches('/'))).unwrap_or_default())
        .filter_map(|p| Some((p.file_name()?.to_str()?.to_owned(), p)))
        .collect();
    let names: BTreeSet<String> = shipped.keys().cloned().collect();
    let present = variants(&file, code);
    let archive = match input.slot {
        Some((slot, _)) => slot_file(slot),
        None => track_file(code),
    };
    let archive_path = out.join(&archive);
    let done = out.join(".done");
    let mut owned = Vec::new();
    for (v, fallbacks) in WORLD_FALLBACKS {
        if present.contains(v) {
            continue;
        }
        let Some(from) = fallbacks.iter().find(|f| present.contains(**f)) else {
            placement
                .notes
                .push(format!("no world can stand in for {code}_{v}"));
            continue;
        };
        let find = |name: &str| file.erp.resources.iter().find(|r| r.name == name);
        let src = world_eaid(code, from);
        let (Some(world), Some(graph)) = (find(&src), find(&format!("{src}?event_graph"))) else {
            continue;
        };
        owned.extend(alias_world(
            &file,
            world,
            graph,
            world_eaid(code, v),
            format!("{code}_{v}"),
        )?);
        placement
            .aliases
            .insert(format!("{code}_{v}"), format!("{code}_{from}"));
    }
    let mut layer_groups = Vec::new();
    if input.slot.is_none() {
        for w in input.worlds {
            let eri = w.eri(code);
            if !file.erp.resources.iter().any(|r| r.name == eri) {
                let name = w.name(code);
                let own = name
                    .strip_prefix(&format!("{code}_"))
                    .and_then(|v| present.get(v).copied());
                let Some(from) = own.or_else(|| grid_variants().find(|v| present.contains(v))) else {
                    placement
                        .notes
                        .push(format!("no world can stand in for {}", w.label(code)));
                    continue;
                };
                let find = |name: &str| file.erp.resources.iter().find(|r| r.name == name);
                let src = world_eaid(code, from);
                let (Some(world), Some(graph)) = (find(&src), find(&format!("{src}?event_graph"))) else {
                    continue;
                };
                owned.extend(alias_world(&file, world, graph, eri, name)?);
                placement.aliases.insert(w.label(code), format!("{code}_{from}"));
            }
            layer_groups.push(w.group(code));
        }
    }
    let mut repointed = BTreeMap::new();
    let mut used = BTreeSet::new();
    if !names.is_empty() {
        for (i, r) in file.erp.resources.iter().enumerate() {
            if let Some((res, name)) = repoint_mips(&file, r, code, &names)? {
                repointed.insert(i, res);
                used.insert(name);
            }
        }
    }
    placement.mips = repointed.len();
    for (i, res) in fixed {
        repointed.entry(i).or_insert(res);
    }
    for (i, r) in file.erp.resources.iter().enumerate() {
        if r.kind() != "AISplineData" {
            continue;
        }
        let (mut res, edited) = match repointed.remove(&i) {
            Some(res) => (res, true),
            None => (file.load(r)?, false),
        };
        match ailine::give_routes_space(&res.fragments[0].data()?) {
            Spacing::Fine => {
                if edited {
                    repointed.insert(i, res);
                }
            }
            Spacing::Fixed { doc, routes } => {
                set_data(&mut res, 0, &doc.to_bytes())?;
                if res.has_stats() {
                    let stats = res.fragments[1].data()?;
                    let version = u32::from_le_bytes(stats[4..8].try_into().expect("4 bytes"));
                    set_data(&mut res, 1, &doc.string_stats(version))?;
                }
                placement.spaced.extend(routes);
                repointed.insert(i, res);
            }
            Spacing::Unfixable(routes) => {
                return Err(Error::Other(ailine::unfixable_message(who, &routes, &fix)));
            }
        }
    }
    match extend_grid(&file, code, input.cars) {
        Ok(Some((i, res, ext))) => {
            repointed.insert(i, res);
            placement.grid = Some(ext);
        }
        Ok(None) => {}
        Err(e) => placement.notes.push(format!(
            "its world could not be given room for {0} cars ({e}); a Grand Prix there with {0} cars crashes",
            input.cars
        )),
    }
    if input.slot.is_none() {
        placement.groups = track_groups(
            code,
            file.erp.resources.iter().chain(&owned).map(|r| r.name.as_str()),
        );
        for group in layer_groups {
            if !placement.groups.contains(&group) {
                placement.groups.push(group);
            }
        }
    }
    if !done.is_file() {
        let own: BTreeSet<&str> = file.erp.resources.iter().map(|r| r.name.as_str()).collect();
        let mut entries = Vec::new();
        if let Some((_, original)) = input.slot {
            let slot_file_in = ErpFile::open(original)?;
            for r in slot_file_in
                .erp
                .resources
                .iter()
                .filter(|r| !own.contains(r.name.as_str()))
            {
                entries.push(Entry::Owned(slot_file_in.load(r)?));
            }
        }
        let mut repointed = repointed;
        for (i, r) in file.erp.resources.iter().enumerate() {
            entries.push(match repointed.remove(&i) {
                Some(res) => Entry::Owned(res),
                None => Entry::Copy(r),
            });
        }
        entries.extend(owned.into_iter().map(Entry::Owned));
        let dir = archive_path.parent().expect("has parent");
        std::fs::create_dir_all(dir).map_err(io_err(dir))?;
        file.write(entries, &archive_path)?;
        std::fs::write(&done, b"").map_err(io_err(&done))?;
    }
    if input.slot.is_none() && !input.game.join(&archive).is_file() {
        placement.added.insert(archive.clone(), archive_path);
    } else {
        placement.replaced.insert(archive.clone(), archive_path);
    }
    placement.archive = archive;
    for name in &used {
        placement
            .added
            .insert(format!("{}/{name}", mip_dir(code)), shipped[name].clone());
    }
    let unused = names.len() - used.len();
    if unused > 0 {
        placement.notes.push(format!(
            "{unused} shipped mip file(s) belong to no texture and are left out"
        ));
    }
    Ok(placement)
}

fn world_layers(file: &ErpFile, eaid: &str) -> Result<Vec<String>> {
    let Some(world) = file.erp.resources.iter().find(|r| r.name == eaid) else {
        return Ok(Vec::new());
    };
    let pkg = Pkg::parse(&file.load(world)?.fragments[0].data()?)?;
    let layers = pkg
        .root
        .as_obj()
        .and_then(|o| o.get("layers"))
        .and_then(Node::as_arr)
        .map(|a| {
            a.items
                .iter()
                .filter_map(|n| Some(n.as_val()?.as_str()?.to_owned()))
                .collect()
        })
        .unwrap_or_default();
    Ok(layers)
}

fn variants<'a>(file: &'a ErpFile, code: &str) -> BTreeSet<&'a str> {
    file.erp
        .resources
        .iter()
        .filter_map(|r| world_variant(code, &r.name))
        .collect()
}

fn grid_variants() -> impl Iterator<Item = &'static str> {
    let fallbacks = WORLD_FALLBACKS
        .iter()
        .find(|(v, _)| *v == GRID_WORLD)
        .map_or(&[][..], |(_, f)| *f);
    std::iter::once(GRID_WORLD).chain(fallbacks.iter().copied())
}

pub fn extend_grid(
    file: &ErpFile,
    code: &str,
    cars: u32,
) -> Result<Option<(usize, Resource, grid::Extension)>> {
    let present = variants(file, code);
    let Some(variant) = grid_variants().find(|v| present.contains(v)) else {
        return Ok(None);
    };
    let mut names = world_layers(file, &world_eaid(code, "common"))?;
    for name in world_layers(file, &world_eaid(code, variant))? {
        if !names.contains(&name) {
            names.push(name);
        }
    }
    let mut found = Vec::new();
    let mut roots = Vec::new();
    for (i, r) in file.erp.resources.iter().enumerate() {
        if names.contains(&r.name) {
            let res = file.load(r)?;
            roots.push(Pkg::parse(&res.fragments[0].data()?)?);
            found.push((i, res));
        }
    }
    let mut layers: Vec<Node> = roots.iter().map(|p| p.root.clone()).collect();
    let Some(ext) = grid::extend(code, &mut layers, cars)? else {
        return Ok(None);
    };
    let mut pkg = roots.swap_remove(ext.layer);
    pkg.root = layers.swap_remove(ext.layer);
    let (i, mut res) = found.swap_remove(ext.layer);
    set_data(&mut res, 0, &pkg.to_bytes()?)?;
    Ok(Some((i, res, ext)))
}

fn walk(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Ok(out);
    };
    for e in rd {
        let p = e.map_err(io_err(dir))?.path();
        if p.is_dir() {
            out.extend(walk(&p)?);
        } else {
            out.push(p);
        }
    }
    out.sort();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_paths_match_the_game_layout() {
        assert_eq!(
            slot_file("invitational_image_4"),
            "2025_asset_groups/ui_package/texture_arrays/invitational_events/event_images/invitational_image_4.erp"
        );
        assert_eq!(
            slot_asset_group("invitational_image_4"),
            "asset_groups/ui_package/texture_arrays/invitational_events/event_images/invitational_image_4.assetgroup"
        );
        assert_eq!(
            world_variant(
                "okayama",
                "eaid://environment_package/tracks/okayama/wep/okayama_f1_26.wep"
            ),
            Some("f1_26")
        );
        assert_eq!(
            shipped_name("a/b/c.tif.mipmaps").len(),
            12 + 1 + "c.tif.mipmaps".len()
        );
    }

    #[test]
    fn a_package_with_stale_tsmd_stats_is_repaired_or_refused() {
        use swofty_formats::bxml::{Document, Element, Kind};
        use swofty_formats::erp::{Erp, Fragment, HEADER_LEN, MAGIC, STORED};
        let dir = std::env::temp_dir().join(format!("swofty-place-tsmd-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();
        let pkg = TrackPackage::parse(crate::map::tests::OKAYAMA_LIKE.as_bytes()).unwrap();
        let line = |gate: &str| Document {
            root: Element {
                kind: Kind::Element,
                pad: 0,
                name: b"track".to_vec(),
                attrs: vec![(b"name".to_vec(), gate.as_bytes().to_vec())],
                children: Vec::new(),
                end_pad: 0,
            },
            tail: Vec::new(),
        };
        let fragment = |raw: Vec<u8>| Fragment {
            name: *b"temp",
            offset: 0,
            size: raw.len() as u64,
            flags: 0x10,
            compression: STORED,
            packed: raw.len() as u64,
            raw,
        };
        let resource = |name: &str, kind: &str, fragments: Vec<Fragment>| Resource {
            name: name.into(),
            kind: swofty_formats::erp::kind(kind).unwrap(),
            unk: vec![0; 4],
            fragments,
            hash: [0; 16],
        };
        let mut header = [0u8; HEADER_LEN];
        header[..4].copy_from_slice(MAGIC);
        header[4..8].copy_from_slice(&3u32.to_le_bytes());
        let spline = "eaid://environment_package/tracks/testring/ai/testring_ai_track.aispline";
        let archive = |data: Vec<u8>| {
            Erp {
                header,
                version: 3,
                resources: vec![
                    resource(&world_eaid("testring", "common"), "WEP", vec![fragment(vec![0])]),
                    resource(
                        spline,
                        "AISplineData",
                        vec![fragment(data), fragment(line("default").string_stats(1))],
                    ),
                ],
            }
            .to_bytes()
        };
        let input = PlaceInput {
            pkg: &pkg,
            dir: &dir,
            game: &dir,
            slot: None,
            cars: 22,
            worlds: &[],
        };
        let edited = line("a longer racing line");
        std::fs::write(dir.join(&pkg.erp), archive(edited.to_bytes())).unwrap();
        let placement = place(&input, &dir.join("cache")).unwrap();
        assert_eq!(placement.repaired, [spline]);
        assert!(placement.dir.to_string_lossy().ends_with("-stats"));
        let placed = ErpFile::open(&placement.dir.join(&placement.archive)).unwrap();
        let res = placed.load(placed.erp.find(spline).unwrap()).unwrap();
        assert_eq!(res.fragments[1].data().unwrap(), edited.string_stats(1));
        assert!(crate::erpbuild::stale_stats_in(&placed).unwrap().is_empty());

        let mut cut = line("default").to_bytes();
        cut.truncate(cut.len() - 3);
        std::fs::write(dir.join(&pkg.erp), archive(cut)).unwrap();
        let Err(e) = place(&input, &dir.join("cache")) else {
            panic!("the package should be refused");
        };
        assert_eq!(
            e.to_string(),
            "Test Ring would crash F1 25: its testring_ai_track.aispline doesn't match the size header the game \
             reads with it, and the launcher can't read the file to fix the header. Turn Test Ring off in this \
             profile, and set any race weekend that uses it back to another track."
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn an_ai_pit_lane_without_space_changes_is_fixed_in_the_placed_archive() {
        use swofty_formats::bxml::Document;
        use swofty_formats::erp::{Erp, Fragment, HEADER_LEN, MAGIC, STORED};
        let dir = std::env::temp_dir().join(format!("swofty-place-space-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();
        let pkg = TrackPackage::parse(crate::map::tests::OKAYAMA_LIKE.as_bytes()).unwrap();
        let fragment = |raw: Vec<u8>| Fragment {
            name: *b"temp",
            offset: 0,
            size: raw.len() as u64,
            flags: 0x10,
            compression: STORED,
            packed: raw.len() as u64,
            raw,
        };
        let resource = |name: &str, kind: &str, fragments: Vec<Fragment>| Resource {
            name: name.into(),
            kind: swofty_formats::erp::kind(kind).unwrap(),
            unk: vec![0; 4],
            fragments,
            hash: [0; 16],
        };
        let mut header = [0u8; HEADER_LEN];
        header[..4].copy_from_slice(MAGIC);
        header[4..8].copy_from_slice(&3u32.to_le_bytes());
        let spline = "eaid://environment_package/tracks/testring/ai/testring_ai_track.aispline";
        let line = ailine::tests::aispline(|_| (-4.1, 3.0), 7);
        let bytes = Erp {
            header,
            version: 3,
            resources: vec![
                resource(&world_eaid("testring", "common"), "WEP", vec![fragment(vec![0])]),
                resource(
                    spline,
                    "AISplineData",
                    vec![fragment(line.to_bytes()), fragment(line.string_stats(1))],
                ),
            ],
        }
        .to_bytes();
        std::fs::write(dir.join(&pkg.erp), bytes).unwrap();
        let input = PlaceInput {
            pkg: &pkg,
            dir: &dir,
            game: &dir,
            slot: None,
            cars: 22,
            worlds: &[],
        };
        let placement = place(&input, &dir.join("cache")).unwrap();
        assert_eq!(placement.spaced, ["pit"]);
        assert!(placement.repaired.is_empty());
        let placed = ErpFile::open(&placement.dir.join(&placement.archive)).unwrap();
        let res = placed.load(placed.erp.find(spline).unwrap()).unwrap();
        let data = res.fragments[0].data().unwrap();
        assert_ne!(data, line.to_bytes());
        assert_eq!(ailine::give_routes_space(&data), Spacing::Fine);
        assert_eq!(
            res.fragments[1].data().unwrap(),
            Document::parse(&data).unwrap().string_stats(1)
        );
        assert!(crate::erpbuild::stale_stats_in(&placed).unwrap().is_empty());
        assert_eq!(
            ailine::fixed_note(pkg.display_name(), &placement.spaced),
            "fixed the AI pit lane of Test Ring so F1 25 doesn't crash when an AI car drives it"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn natural_groups_come_from_the_archive() {
        let t = "eaid://environment_package/tracks/okayama";
        let names = [
            format!("{t}/wep/okayama_f1.wep"),
            format!("{t}/wep/okayama_common.wep?event_graph"),
            format!("{t}/wep/okayama_common.wep"),
            format!("{t}/probes/probes_garage_02.shprobes"),
            format!("{t}/probes/key0_2019.diffuse.dds"),
            format!("{t}/ai/f1_2025_vehicle_package/mclaren/mclaren.acpd"),
            format!("{t}/ai/f1_2025_vehicle_package/mclaren/mclaren.brpd"),
            format!("{t}/ai/okayama_ai_track.aispline"),
            "eaid://environment_package/tracks/sepang/wep/sepang_f1.wep".into(),
        ];
        let g = "asset_groups/environment_package/tracks/okayama";
        assert_eq!(
            track_groups("okayama", names.iter().map(String::as_str)),
            [
                format!("{g}/okayama.assetgroup"),
                format!("{g}/wep/okayama_common.assetgroup"),
                format!("{g}/wep/okayama_f1.assetgroup"),
                format!("{g}/bake/okayama_ao.bake.assetgroup"),
                format!("{g}/bake/okayama_light_list.bake.assetgroup"),
                format!("{g}/bake/okayama_radiance.bake.assetgroup"),
                format!("{g}/probes/config.ddgi.assetgroup"),
                format!("{g}/probes/key_2019.assetgroup"),
                format!("{g}/probes/probes_garage_02.assetgroup"),
                format!("{g}/ai/f1_2025_vehicle_package/mclaren/mclaren.assetgroup"),
            ]
        );
        assert_eq!(
            track_file("okayama"),
            "2025_asset_groups/environment_package/tracks/okayama/okayama.erp"
        );
    }

    const MOD_GROUP: &str =
        "asset_groups/mod_environment_package/tracks/[FILENAME]/wep/[FILENAME]_x.assetgroup";
    const MOD_ERI: &str = "eaid://mod_environment_package/tracks/[FILENAME]/wep/[FILENAME]_x.wep";

    fn world_archive(dir: &Path, pkg: &TrackPackage, extra: &[String]) {
        use swofty_formats::erp::{Erp, Fragment, HEADER_LEN, MAGIC};
        use swofty_formats::pkg::{Arr, Obj, Val};
        let fragment = |raw: Vec<u8>| Fragment {
            name: *b"temp",
            offset: 0,
            size: raw.len() as u64,
            flags: 0x10,
            compression: STORED,
            packed: raw.len() as u64,
            raw,
        };
        let resource = |name: &str, kind: &str, data: Vec<u8>| Resource {
            name: name.into(),
            kind: swofty_formats::erp::kind(kind).unwrap(),
            unk: vec![0; 4],
            fragments: vec![fragment(data)],
            hash: [0; 16],
        };
        let world = |eaid: &str| {
            let name = eaid.rsplit('/').next().unwrap().trim_end_matches(".wep");
            let root = Obj::new(vec![
                (Val::str("name"), Node::Val(Val::str(name))),
                (
                    Val::str("event_graph"),
                    Node::Val(Val::str(format!("{eaid}?event_graph"))),
                ),
                (Val::str("layers"), Node::Arr(Arr::new(Vec::new()))),
            ]);
            let pkg = Pkg {
                name: *b"WRLD",
                root: Node::Obj(root),
                vca: Vec::new(),
                layout: None,
            };
            [
                resource(eaid, "World", pkg.to_bytes().unwrap()),
                resource(&format!("{eaid}?event_graph"), "EventGraph", vec![1]),
            ]
        };
        let mut header = [0u8; HEADER_LEN];
        header[..4].copy_from_slice(MAGIC);
        header[4..8].copy_from_slice(&3u32.to_le_bytes());
        let eaids = ["common", "f1", GRID_WORLD]
            .iter()
            .map(|v| world_eaid("testring", v))
            .chain(extra.iter().cloned());
        let bytes = Erp {
            header,
            version: 3,
            resources: eaids.flat_map(|e| world(&e)).collect(),
        }
        .to_bytes();
        std::fs::write(dir.join(&pkg.erp), bytes).unwrap();
    }

    fn place_with_worlds(name: &str, extra: &[String]) -> (Placement, ErpFile) {
        let dir = std::env::temp_dir().join(format!("swofty-place-{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();
        let pkg = TrackPackage::parse(crate::map::tests::OKAYAMA_LIKE.as_bytes()).unwrap();
        world_archive(&dir, &pkg, extra);
        let worlds = [LayerWorld::new(MOD_GROUP, MOD_ERI).unwrap()];
        let input = PlaceInput {
            pkg: &pkg,
            dir: &dir,
            game: &dir,
            slot: None,
            cars: 0,
            worlds: &worlds,
        };
        let placement = place(&input, &dir.join("cache")).unwrap();
        let placed = ErpFile::open(&placement.dir.join(&placement.archive)).unwrap();
        (placement, placed)
    }

    #[test]
    fn an_addon_layer_world_gets_a_stand_in_on_an_added_track() {
        let (placement, placed) = place_with_worlds("layer-world", &[]);
        let eri = "eaid://mod_environment_package/tracks/testring/wep/testring_x.wep";
        assert_eq!(
            placement
                .aliases
                .get("mod_environment_package/tracks/testring/wep/testring_x")
                .map(String::as_str),
            Some("testring_f1_26_myteam")
        );
        let group = "asset_groups/mod_environment_package/tracks/testring/wep/testring_x.assetgroup";
        assert_eq!(placement.groups.iter().filter(|g| *g == group).count(), 1);
        let world = placed.load(placed.erp.find(eri).unwrap()).unwrap();
        let pkg = Pkg::parse(&world.fragments[0].data().unwrap()).unwrap();
        let root = pkg.root.as_obj().unwrap();
        let text = |k: &str| {
            root.get(k)
                .and_then(Node::as_val)
                .and_then(|v| v.as_str())
                .map(str::to_owned)
        };
        assert_eq!(text("name").as_deref(), Some("testring_x"));
        assert_eq!(text("event_graph"), Some(format!("{eri}?event_graph")));
        assert!(placed.erp.find(&format!("{eri}?event_graph")).is_ok());
        std::fs::remove_dir_all(placement.dir.parent().unwrap().parent().unwrap()).ok();
    }

    #[test]
    fn a_track_that_ships_the_layer_world_keeps_its_own() {
        let eri = "eaid://mod_environment_package/tracks/testring/wep/testring_x.wep";
        let (placement, placed) = place_with_worlds("own-layer-world", &[eri.to_owned()]);
        assert!(placement.aliases.keys().all(|k| !k.ends_with("testring_x")));
        assert_eq!(placed.erp.resources.iter().filter(|r| r.name == eri).count(), 1);
        assert!(placement.groups.contains(
            &"asset_groups/mod_environment_package/tracks/testring/wep/testring_x.assetgroup".into()
        ));
        std::fs::remove_dir_all(placement.dir.parent().unwrap().parent().unwrap()).ok();
    }

    #[test]
    fn layer_worlds_skip_the_stock_worlds_and_groups_an_addon_registers() {
        use swofty_formats::gvdb::Record;
        let stock_group =
            "asset_groups/environment_package/tracks/[FILENAME]/wep/[FILENAME]_f1_26.assetgroup";
        let rows = [
            ("f1_26", stock_group.to_owned(), world_eaid(FILENAME, "f1_26")),
            ("mod_a", MOD_GROUP.to_owned(), MOD_ERI.to_owned()),
            ("mod_b", MOD_GROUP.to_owned(), MOD_ERI.to_owned()),
        ];
        let mut records = vec![Record::new(
            &format!("{TRACK_LAYERS}._TrackLayer_instancecount_"),
            0,
            rows.len().to_string(),
        )];
        for (i, (label, group, eri)) in rows.iter().enumerate() {
            let row = format!("{TRACK_LAYERS}.TrackLayer_{i}");
            records.push(Record::new(&format!("{row}._metadata_"), 4, *label));
            records.push(Record::new(&format!("{row}.worldAssetGroup"), 4, group.as_str()));
            records.push(Record::new(&format!("{row}.worldEri"), 4, eri.as_str()));
        }
        let worlds = layer_worlds(&Gvdb { records });
        assert_eq!(worlds, [LayerWorld::new(MOD_GROUP, MOD_ERI).unwrap()]);
        let registered = "asset_groups/mod_environment_package/tracks/monaco/wep/monaco_x.assetgroup";
        assert!(missing_worlds(&worlds, "monaco", |g| g == registered).is_empty());
        assert_eq!(missing_worlds(&worlds, "testring", |g| g == registered), worlds);
        assert_eq!(LayerWorld::new("asset_groups/fixed.assetgroup", MOD_ERI), None);
    }
}
