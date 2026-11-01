use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use serde_json::Value;
use swofty_core::asset::{AssetOp, FileKind, OpFile, Source};
use swofty_core::erpbuild::{ErpBuild, unsupported};
use swofty_core::manifest::{AddonManifest, Binds, OptionType};
use swofty_core::ops::{DbOp, DbOps, ErpOp, apply_db_ops};
use swofty_core::pkgedit::value_text;
use swofty_core::profile::Profile;
use swofty_core::runtime::db_edits;
use swofty_core::settings::render_ops;
use swofty_formats::gvdb::{Gvdb, Record, instance_count_path};
use swofty_formats::hash::{fnv1a64, fnv1a64_from};
use swofty_formats::pkg::Node;

const GVDB: &str = "game_package/gamedatabases/gamedatabases.2025.bin";
const LIST: &str = "entrants.entrant_lists.entrant_list_34";
const SETUP: &str = "entrants.entrant_setup_modes.entrant_setup_mode";
const WORLD: u64 = 0x4466_68bc_25ba_571c;
const REFERENCE: u64 = 0x7afa_51a9_cd0e_4b9a;
const SIZES: [u64; 5] = [22, 24, 26, 28, 30];
const STOCK_TEAMS: usize = 358;
const STOCK_VEHICLES: usize = 413;

fn addon_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../addons/custom-grid-size")
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())))
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn manifest() -> AddonManifest {
    AddonManifest::load(&addon_dir()).unwrap()
}

fn files(kind: &str) -> Vec<String> {
    let m = manifest();
    match kind {
        "database" => m.operations.database,
        _ => m.operations.erp,
    }
}

fn render(rel: &str, cars: u64) -> Value {
    let m = manifest();
    let mut p = Profile::new("test");
    p.set_option(&m, "cars", cars.into()).unwrap();
    let mut doc = read_json(&addon_dir().join(rel));
    render_ops(&mut doc, &m, &p).unwrap();
    doc
}

fn db_ops(cars: u64) -> DbOps {
    serde_json::from_value(render(&files("database")[0], cars)).unwrap()
}

fn erp_ops(rel: &str, cars: u64) -> Vec<AssetOp> {
    OpFile::from_value(render(rel, cars), FileKind::Erp).unwrap().ops
}

fn skipped(ops: &[AssetOp]) -> Option<&'static str> {
    ops.iter().find_map(|op| match op {
        AssetOp::Erp(op) => unsupported(op),
        _ => None,
    })
}

fn game() -> Option<PathBuf> {
    match std::env::var_os("F1_25_DIR") {
        Some(d) => Some(PathBuf::from(d)),
        None => {
            eprintln!("F1_25_DIR is not set; skipping");
            None
        }
    }
}

struct Db<'a> {
    records: &'a [Record],
    index: HashMap<u64, usize>,
}

impl<'a> Db<'a> {
    fn new(db: &'a Gvdb) -> Self {
        let index = db.records.iter().enumerate().map(|(i, r)| (r.key, i)).collect();
        Self {
            records: &db.records,
            index,
        }
    }

    fn get(&self, path: &str) -> Option<&Record> {
        self.key(fnv1a64(path))
    }

    fn key(&self, key: u64) -> Option<&Record> {
        self.index.get(&key).map(|&i| &self.records[i])
    }

    fn text(&self, path: &str) -> Option<String> {
        self.get(path).map(|r| r.text().into_owned())
    }

    fn hashed(&self, base: u64, suffix: &str) -> Option<String> {
        self.key(fnv1a64_from(base, suffix))
            .map(|r| r.text().into_owned())
    }

    fn count(&self, table: &str, row: &str) -> Option<usize> {
        self.text(&instance_count_path(table, row))?.parse().ok()
    }

    fn row_of(&self, base: u64, label: &str) -> Option<usize> {
        (0..4096)
            .take_while(|i| self.hashed(base, &format!("{i}._metadata_")).is_some())
            .find(|i| self.hashed(base, &format!("{i}._metadata_")).as_deref() == Some(label))
    }

    fn setup_row(&self, label: &str) -> String {
        let n = self
            .count("entrants.entrant_setup_modes", "entrant_setup_mode")
            .unwrap();
        (0..n)
            .map(|i| format!("{SETUP}_{i}"))
            .find(|r| self.text(&format!("{r}._metadata_")).as_deref() == Some(label))
            .unwrap()
    }

    fn labels(&self, table: &str, row: &str) -> Vec<String> {
        (0..self.count(table, row).unwrap())
            .map(|i| {
                self.text(&format!("{table}.{row}_{i}._metadata_"))
                    .unwrap_or_default()
            })
            .collect()
    }
}

fn assert_stock(db: &Gvdb) {
    let stock = db.instance_count(LIST, "entrant") == Some(22)
        && db.instance_count("entrants.teams", "team") == Some(STOCK_TEAMS)
        && db.instance_count("entrants.vehicles", "vehicle") == Some(STOCK_VEHICLES);
    assert!(stock, "F1_25_DIR must hold the stock gamedatabases.2025.bin");
}

#[test]
fn manifest_declares_the_cars_slider_and_its_files() {
    let m = manifest();
    assert_eq!(m.name, "Custom Grid Size");
    assert_eq!(
        m.description.as_deref(),
        Some("Choose how many cars race in F1 2026 Grand Prix sessions.")
    );
    let [o] = m.options.as_slice() else {
        panic!("one option")
    };
    assert_eq!(
        (o.id.as_str(), o.kind, o.binds),
        ("cars", OptionType::Slider, Binds::Db)
    );
    assert_eq!((o.min, o.max, o.step), (Some(22.0), Some(30.0), Some(2.0)));
    assert_eq!(o.default_value(), Value::from(26));
    let states: Vec<Value> = o.states().into_iter().map(|s| s.value).collect();
    assert_eq!(states, SIZES.map(Value::from));
    assert_eq!(files("database"), ["db/grid.json"]);
    let tracks = files("erp");
    assert_eq!(tracks.len(), 28);
    for rel in std::iter::once("db/grid.json".to_owned()).chain(tracks.iter().cloned()) {
        let doc = read_json(&addon_dir().join(&rel));
        for op in doc["ops"].as_array().unwrap() {
            let when = op["when"]
                .as_object()
                .unwrap_or_else(|| panic!("{rel}: every op names its grid size"));
            let cars = when["cars"].as_u64().unwrap();
            assert!(when.len() == 1 && SIZES.contains(&cars), "{rel}: when {when:?}");
        }
    }
    for rel in &tracks {
        for cars in SIZES {
            let ops = erp_ops(rel, cars);
            assert_eq!(skipped(&ops), None, "{rel}");
            assert_eq!(ops.is_empty(), cars <= 24, "{rel} at {cars}");
        }
    }
}

struct Built {
    before: Vec<u8>,
    db: Gvdb,
    ops: DbOps,
}

fn build(game: &Path, cars: u64) -> Built {
    let before = std::fs::read(game.join(GVDB)).unwrap();
    let mut db = Gvdb::parse(&before).unwrap();
    assert_stock(&db);
    let ops = db_ops(cars);
    apply_db_ops(&mut db, &ops, &BTreeMap::<String, Vec<String>>::new()).unwrap();
    Built { before, db, ops }
}

fn sources(ops: &DbOps, table: &str) -> Vec<(String, BTreeMap<String, String>)> {
    ops.ops
        .iter()
        .filter_map(|op| match op {
            DbOp::AddRow {
                table: t, from, set, ..
            } if t == table => Some((from.clone(), set.clone())),
            _ => None,
        })
        .collect()
}

fn source_label(from: &str) -> &str {
    from.rsplit_once('=').unwrap().1.trim_end_matches(']')
}

fn check_database(b: &Built, cars: u64) {
    let db = &Db::new(&b.db);
    let extra = ((cars - 22) / 2) as usize;
    let fields = &b.ops.fields;
    assert_eq!(db.count(LIST, "entrant"), Some(cars as usize));
    assert_eq!(db.count("entrants.teams", "team"), Some(STOCK_TEAMS + extra));
    assert_eq!(
        db.count("entrants.vehicles", "vehicle"),
        Some(STOCK_VEHICLES + extra)
    );
    for (label, want) in [
        ("f1_26", cars),
        ("f1_generic_2026", cars),
        ("f1_custom_team_2026", cars + 2),
    ] {
        let row = db.setup_row(label);
        assert_eq!(
            db.text(&format!("{row}.max_entrants")),
            Some(want.to_string()),
            "{label}"
        );
    }
    let f1 = db.row_of(WORLD, "f1_26").unwrap();
    let myteam = db.row_of(WORLD, "myteam_26").unwrap();
    for f in ["worldEri", "worldAssetGroup"] {
        let world = db.hashed(WORLD, &format!("{f1}.{f}")).unwrap();
        let wanted = db.hashed(WORLD, &format!("{myteam}.{f}")).unwrap();
        assert_eq!(world == wanted, cars > 22, "{f} at {cars}: {world}");
    }
    let teams = sources(&b.ops, "entrants.teams");
    let vehicles = sources(&b.ops, "entrants.vehicles");
    assert_eq!((teams.len(), vehicles.len()), (extra, extra));
    let distinct = |v: &[(String, BTreeMap<String, String>)]| {
        v.iter()
            .map(|(f, _)| source_label(f))
            .collect::<BTreeSet<_>>()
            .len()
    };
    assert_eq!(distinct(&teams), extra, "team sources must be distinct");
    assert_eq!(distinct(&vehicles), extra, "vehicle sources must be distinct");
    let team_labels = db.labels("entrants.teams", "team");
    let vehicle_labels = db.labels("entrants.vehicles", "vehicle");
    for (table, row, stock, list, added) in [
        ("entrants.teams", "team", STOCK_TEAMS, &team_labels, &teams),
        (
            "entrants.vehicles",
            "vehicle",
            STOCK_VEHICLES,
            &vehicle_labels,
            &vehicles,
        ),
    ] {
        let schema = &fields[&format!("{table}.{row}")];
        for (k, (from, set)) in added.iter().enumerate() {
            let src_row = list.iter().position(|l| l == source_label(from)).unwrap();
            assert!(src_row < stock, "{from} must be a stock row");
            let src = format!("{table}.{row}_{src_row}");
            let dst = format!("{table}.{row}_{}", stock + k);
            assert_eq!(
                db.text(&format!("{dst}._metadata_")).as_ref(),
                set.get("_metadata_")
            );
            for f in schema {
                let s = db
                    .get(&format!("{src}.{f}"))
                    .unwrap_or_else(|| panic!("{src}.{f} missing"));
                let d = db
                    .get(&format!("{dst}.{f}"))
                    .unwrap_or_else(|| panic!("{dst}.{f} missing at {cars} cars"));
                assert_eq!(d.tag, s.tag, "{dst}.{f}");
                match set.get(f) {
                    Some(v) => assert_eq!(&*d.text(), v, "{dst}.{f}"),
                    None => assert_eq!(d.value, s.value, "{dst}.{f}"),
                }
            }
        }
        let stats: BTreeSet<String> = (0..stock + added.len())
            .map(|i| db.text(&format!("{table}.{row}_{i}.stat_id")).unwrap())
            .collect();
        assert_eq!(
            stats.len(),
            stock + added.len(),
            "{table} stat ids must be unique"
        );
    }
    assert_eq!(
        fields[&format!("{}.{}", "entrants.vehicles", "vehicle")].len(),
        246
    );
    assert_eq!(
        fields["entrants.vehicles.vehicle"]
            .iter()
            .filter(|f| f.starts_with("ai_vehicle_config."))
            .count(),
        218
    );
    assert_eq!(fields["entrants.teams.team"].len(), 30);
    let entrant = &fields[&format!("{LIST}.entrant")];
    assert_eq!(entrant.len(), 10);
    let mut numbers = BTreeSet::new();
    for i in 0..cars as usize {
        let row = format!("{LIST}.entrant_{i}");
        for f in entrant {
            assert!(
                db.get(&format!("{row}.{f}")).is_some(),
                "{row}.{f} missing at {cars} cars"
            );
        }
        let team = db.text(&format!("{row}.team")).unwrap();
        let vehicle = db.text(&format!("{row}.vehicle")).unwrap();
        assert!(
            team_labels.contains(&team),
            "{row}: team {team} is outside the counted rows"
        );
        assert!(
            vehicle_labels.contains(&vehicle),
            "{row}: vehicle {vehicle} is outside the counted rows"
        );
        let own = db.text(&format!("{row}.car_number")).unwrap();
        let number = if own == "255" {
            let reference = db.text(&format!("{row}.entrant_reference")).unwrap();
            let r = db
                .row_of(REFERENCE, &reference)
                .unwrap_or_else(|| panic!("{row}: no reference {reference}"));
            db.hashed(REFERENCE, &format!("{r}.car_number")).unwrap()
        } else {
            let reference = db.text(&format!("{row}.entrant_reference")).unwrap();
            assert!(
                db.row_of(REFERENCE, &reference).is_some(),
                "{row}: no reference {reference}"
            );
            own
        };
        assert!(
            numbers.insert(number.clone()),
            "car number {number} is used twice at {cars} cars"
        );
    }
    assert!(db.get(&format!("{LIST}.entrant_{cars}._metadata_")).is_none());
    let bytes = b.db.to_bytes();
    assert!(
        Gvdb::parse(&bytes).unwrap().to_bytes() == bytes,
        "round trip at {cars} cars"
    );
    if cars == 22 {
        assert!(bytes == b.before, "22 cars must leave the database as shipped");
    }
}

#[test]
fn database_builds_for_every_grid_size() {
    let Some(game) = game() else { return };
    if !game.join(GVDB).is_file() {
        eprintln!("no gamedatabases.2025.bin under F1_25_DIR; skipping");
        return;
    }
    for cars in SIZES {
        let b = build(&game, cars);
        check_database(&b, cars);
    }
}

#[test]
fn runtime_database_edits_carry_every_grid_size() {
    let Some(game) = game() else { return };
    if !game.join(GVDB).is_file() {
        eprintln!("no gamedatabases.2025.bin under F1_25_DIR; skipping");
        return;
    }
    for cars in SIZES {
        let b = build(&game, cars);
        let stock = Gvdb::parse(&b.before).unwrap();
        let edits = db_edits(&stock, &b.db).unwrap_or_else(|e| panic!("{cars} cars: {e}"));
        let mut records: HashMap<u64, (u8, Vec<u8>)> = stock
            .records
            .iter()
            .map(|r| (r.key, (r.tag, r.value.clone())))
            .collect();
        for e in &edits {
            let key = u64::from_str_radix(&e.hash, 16).unwrap();
            records.insert(key, (e.tag, e.value.clone().into_bytes()));
        }
        let built: HashMap<u64, (u8, Vec<u8>)> =
            b.db.records
                .iter()
                .map(|r| (r.key, (r.tag, r.value.clone())))
                .collect();
        assert_eq!(records, built, "{cars} cars");
    }
}

fn instances(node: &Node) -> Vec<&Node> {
    node.as_obj()
        .unwrap()
        .get("instances")
        .unwrap()
        .as_arr()
        .unwrap()
        .items
        .iter()
        .collect()
}

fn ids(layer: &Node, prefix: &str) -> BTreeSet<u32> {
    instances(layer)
        .iter()
        .filter_map(|i| value_text(i, "name").ok())
        .filter_map(|n| n.split_once('$').map(|(_, o)| o.to_owned()))
        .filter_map(|o| o.strip_prefix(prefix)?.parse().ok())
        .collect()
}

fn capacity(rel: &str) -> u32 {
    let doc = read_json(&addon_dir().join(rel));
    doc["ops"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|op| op["edits"].as_array().into_iter().flatten())
        .filter_map(|e| e["as"].as_str()?.split_once("$GridSlot")?.1.parse::<u32>().ok())
        .max()
        .unwrap_or(24)
}

#[test]
fn tracks_build_for_every_grid_size() {
    let Some(game) = game() else { return };
    let tracks: Vec<String> = files("erp")
        .into_iter()
        .filter(|rel| {
            let doc = erp_ops(rel, 30);
            let present = doc.iter().all(|op| match op {
                AssetOp::Erp(ErpOp::PatchPkg { erp, .. }) => game.join(erp).is_file(),
                _ => true,
            });
            if !present {
                eprintln!("{rel}: archive not under F1_25_DIR; skipping");
            }
            present
        })
        .collect();
    if tracks.is_empty() {
        return;
    }
    let original = |rel: &str| game.join(rel);
    for cars in SIZES {
        let mut b = ErpBuild::new(&original);
        for rel in &tracks {
            for op in erp_ops(rel, cars) {
                let AssetOp::Erp(op) = op else { continue };
                b.apply(&op, &|s: &Source| Ok(s.resolve(&game)?))
                    .unwrap_or_else(|e| panic!("{rel} at {cars} cars: {e}"));
            }
        }
        if cars <= 24 {
            assert_eq!(b.changed().count(), 0, "{cars} cars must not touch any track");
            continue;
        }
        let mut warned = BTreeSet::new();
        for rel in &tracks {
            let cap = capacity(rel);
            let fits = (cars as u32).min(cap);
            let track = rel.trim_start_matches("tracks/").trim_end_matches(".json");
            let ops = erp_ops(rel, cars);
            for op in &ops {
                match op {
                    AssetOp::Warn { message } => {
                        assert!(cap < cars as u32, "{track}: warns at {cars} cars but holds {cap}");
                        assert!(message.contains(&format!("room for {cap} cars")), "{message}");
                        warned.insert(track.to_owned());
                    }
                    AssetOp::Erp(ErpOp::PatchPkg {
                        erp,
                        resource,
                        fragment,
                        ..
                    }) => {
                        let pkg = b.resource(erp, resource, *fragment).unwrap().unwrap();
                        let again = pkg.to_bytes().unwrap();
                        assert!(
                            swofty_formats::pkg::Pkg::parse(&again)
                                .unwrap()
                                .to_bytes()
                                .unwrap()
                                == again
                        );
                        let all = instances(&pkg.root);
                        let uids: BTreeSet<String> =
                            all.iter().map(|i| value_text(i, "uniqueid").unwrap()).collect();
                        assert_eq!(uids.len(), all.len(), "{track}: duplicate uniqueid in {resource}");
                        if !resource.ends_with("vehicles_current_myteam.layer") {
                            continue;
                        }
                        let garages = fits / 2;
                        assert_eq!(
                            ids(&pkg.root, "F1Garage"),
                            (12..=garages).collect(),
                            "{track} garages at {cars}"
                        );
                        assert_eq!(
                            ids(&pkg.root, "PitBox"),
                            (12..=garages).collect(),
                            "{track} pit boxes at {cars}"
                        );
                        assert_eq!(
                            ids(&pkg.root, "PitGarage"),
                            (23..=fits).collect(),
                            "{track} garage spots at {cars}"
                        );
                        let slots = ids(&pkg.root, "GridSlot");
                        assert!(
                            (25..=fits).all(|s| slots.contains(&s)) && !slots.contains(&(fits + 1)),
                            "{track} grid at {cars}"
                        );
                        for g in 13..=garages {
                            let garage = all
                                .iter()
                                .find(|i| value_text(i, "name").unwrap().ends_with(&format!("$F1Garage{g}")))
                                .unwrap();
                            for (side, spot) in [("left", 2 * g - 1), ("right", 2 * g)] {
                                let link =
                                    value_text(garage, &format!("overrides.F1Garage.{side} car placement"))
                                        .unwrap();
                                let target = all
                                    .iter()
                                    .find(|i| {
                                        value_text(i, "name")
                                            .unwrap()
                                            .ends_with(&format!("$PitGarage{spot}"))
                                    })
                                    .unwrap();
                                assert_eq!(
                                    link,
                                    value_text(target, "uniqueid").unwrap(),
                                    "{track} garage {g} {side}"
                                );
                            }
                            assert_eq!(
                                value_text(garage, "overrides.F1Garage.id").unwrap(),
                                hex_le(g),
                                "{track} garage {g} id"
                            );
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
        let expect: BTreeSet<String> = tracks
            .iter()
            .filter(|rel| capacity(rel) < cars as u32)
            .map(|rel| {
                rel.trim_start_matches("tracks/")
                    .trim_end_matches(".json")
                    .to_owned()
            })
            .collect();
        assert_eq!(warned, expect, "warnings at {cars} cars");
        eprintln!(
            "{cars} cars: {} archives patched, {} tracks clamped",
            b.changed().count(),
            warned.len()
        );
    }
}

fn hex_le(v: u32) -> String {
    v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
