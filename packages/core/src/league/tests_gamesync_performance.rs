//! Tests of the league's gamesync::performance (W2-10): the teams' league pace as car masses in
//! the game, through the handling registry and the build.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::gamesync::performance::{self, KEEP_BELOW_KG, MAX_MASS, PartScales, TOP_MASS};
use super::gamesync::{self, HANDLING_REGISTRY, Overlay, SyncContext};
use super::progress::Upgrade;
use super::*;
use crate::activation::handling::{self, Op, Step, Target};

const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/classic-seasons-mini");
const SEASON_2016: &str = "swofty.classic-seasons-2016";

/// What `aislots.head_start(year, key)` of Classic Seasons gives each league slot (1, 2, …),
/// rounded to 6 digits: `classic-seasons/tools/aislots.py` run once with `python -I` over its
/// `data/ai_slots.json` and `data/performance.json` (teams matched to slots by their drivers).
const PYTHON_HEAD_STARTS: &[(u16, &[f32])] = &[
    (
        2016,
        &[
            0.0, 0.114, 0.031, 0.307, 0.148, 0.37, 0.183, 0.265, 0.437, 0.355, 0.437,
        ],
    ),
    (
        2017,
        &[0.0, 0.031, 0.114, 0.307, 0.183, 0.148, 0.265, 0.355, 0.437, 0.37],
    ),
    (
        2018,
        &[0.0, 0.031, 0.114, 0.37, 0.265, 0.183, 0.437, 0.355, 0.307, 0.148],
    ),
    (
        2019,
        &[0.0, 0.031, 0.114, 0.148, 0.37, 0.183, 0.437, 0.355, 0.307, 0.265],
    ),
    (
        2020,
        &[0.0, 0.307, 0.031, 0.183, 0.114, 0.148, 0.37, 0.265, 0.355, 0.437],
    ),
    (
        2021,
        &[0.0, 0.148, 0.031, 0.183, 0.307, 0.265, 0.37, 0.114, 0.355, 0.437],
    ),
    (
        2022,
        &[0.114, 0.0, 0.031, 0.183, 0.37, 0.148, 0.437, 0.265, 0.307, 0.355],
    ),
    (
        2023,
        &[0.114, 0.031, 0.0, 0.148, 0.183, 0.265, 0.355, 0.37, 0.437, 0.307],
    ),
    (
        2024,
        &[0.148, 0.114, 0.0, 0.031, 0.183, 0.37, 0.355, 0.265, 0.437, 0.307],
    ),
];

/// The 2016 masses Classic Seasons builds (`pace.mass(2016, key)`), by slot.
const PYTHON_MASSES_2016: [f32; 11] = [
    798.0, 825.0, 817.0, 861.0, 849.0, 897.0, 852.0, 860.0, 902.0, 869.0, 907.0,
];

fn fixture(rel: &str) -> Value {
    serde_json::from_slice(&std::fs::read(Path::new(FIXTURE).join(rel)).unwrap()).unwrap()
}

/// The fixture's Classic Seasons 2016 files, as `careersync::context` reads them.
struct Ctx {
    content: Value,
    cs_drivers: Value,
    liveries: Value,
    ai_slots: Value,
}

impl Ctx {
    fn new() -> Self {
        Self {
            content: fixture("swofty.classic-seasons-2016/registry/content.json"),
            cs_drivers: fixture("swofty.classic-seasons/registry/drivers.json"),
            liveries: fixture("swofty.classic-seasons-2016/assets/liveries.json"),
            ai_slots: fixture("swofty.classic-seasons/data/ai_slots.json"),
        }
    }

    /// A year the game has itself, without Classic Seasons installed.
    fn none() -> Self {
        Self {
            content: Value::Null,
            cs_drivers: Value::Null,
            liveries: Value::Null,
            ai_slots: Value::Null,
        }
    }

    fn get(&self) -> SyncContext<'_> {
        SyncContext {
            content: &self.content,
            cs_drivers: &self.cs_drivers,
            liveries: &self.liveries,
            ai_slots: &self.ai_slots,
            loader_applies_rows: true,
        }
    }
}

fn anna() -> Actor {
    Actor {
        id: "u-anna".into(),
        name: "Anna".into(),
        owner: true,
    }
}

fn ben() -> Actor {
    Actor {
        id: "u-ben".into(),
        name: "Ben".into(),
        owner: false,
    }
}

/// Anna at Ferrari and Ben at Toro Rosso from `first_year`, with the league pace in the game.
fn career(first_year: u16) -> League {
    let setup = Setup {
        first_year,
        ..Setup::default()
    };
    let mut l = League::start(setup, &anna(), Some(7), 42).unwrap();
    apply(&mut l, &ben(), Action::Enter { number: Some(8) }).unwrap();
    for (who, slot) in [(anna(), 2), (ben(), 8)] {
        apply(&mut l, &who, Action::Seat { slot, replaces: None }).unwrap();
    }
    apply(&mut l, &anna(), Action::Begin).unwrap();
    l.options.game_performance = true;
    l
}

fn next_round(l: &mut League) {
    super::tests::agreed(l, &anna(), Action::Simulate { round: None });
}

fn overlay(l: &League, ctx: &Ctx) -> Overlay {
    gamesync::overlay(l, "c-1", "Anna & Ben", &ctx.get()).expect("an overlay while racing")
}

fn files(l: &League, ctx: &Ctx) -> Vec<gamesync::File> {
    gamesync::files(&overlay(l, ctx), &ctx.get())
}

fn handling_json(files: &[gamesync::File]) -> Value {
    let (_, bytes) = files
        .iter()
        .find(|(p, _)| p == HANDLING_REGISTRY)
        .expect("a handling file");
    serde_json::from_slice(bytes).unwrap()
}

fn masses(l: &League, ctx: &Ctx, round: u8) -> BTreeMap<u8, f32> {
    performance::masses_at(l, &ctx.get(), round).expect("masses")
}

/// Every team's pace in `round` set to its gap for the year: the carry takes the development off.
fn without_development(l: &mut League, round: u8) {
    let season = l.season_mut();
    let moves: Vec<(u8, f32)> = season
        .teams
        .iter()
        .map(|t| (t.slot, season.pace_at(t.slot, round) - t.pace))
        .collect();
    for (slot, delta) in moves {
        let t = season.teams.iter_mut().find(|t| t.slot == slot).unwrap();
        t.carry -= delta;
    }
}

/// `n` levels of `part` for team `slot`, on the car from round `ready`.
fn upgrades(l: &mut League, slot: u8, part: Part, n: u8, ready: u8) {
    for level in 1..=n {
        l.season_mut().upgrades.push(Upgrade {
            slot,
            part,
            level,
            by: "u-anna".into(),
            cost: 800,
            round: 1,
            started: false,
            ready,
            node: None,
            failed: false,
        });
    }
}

#[test]
fn head_start_matches_the_python_tool() {
    let ai_slots = fixture("swofty.classic-seasons/data/ai_slots.json");
    for (year, want) in PYTHON_HEAD_STARTS {
        for (i, want) in want.iter().enumerate() {
            let slot = i as u8 + 1;
            let got = performance::head_start(*year, slot, &ai_slots).unwrap();
            assert!(
                (got - want).abs() < 1e-5,
                "{year} slot {slot}: {got} vs Python {want}"
            );
        }
        assert_eq!(
            performance::head_start(*year, want.len() as u8 + 1, &ai_slots),
            None,
            "{year} has no more teams"
        );
    }
    // A Classic Seasons year can't be told without the AI slots.
    assert_eq!(performance::head_start(2016, 1, &Value::Null), None);
    // A year the game has itself drives its own AI data: the head start is the team's own gap.
    let red_bull = performance::head_start(2025, 3, &Value::Null).unwrap();
    let mercedes = performance::head_start(2025, 1, &Value::Null).unwrap();
    assert_eq!(red_bull, 0.0);
    assert!((mercedes - 0.148).abs() < 1e-5, "{mercedes}");
    assert_eq!(performance::percent_per_kg(&ai_slots), 0.0308);
    assert_eq!(performance::percent_per_kg(&Value::Null), 0.0308);
}

#[test]
fn mass_follows_pace_and_head_start() {
    let ctx = Ctx::new();
    // Without development, the league's 2016 cars weigh what Classic Seasons builds them at.
    let mut l = career(2016);
    without_development(&mut l, 1);
    let built: Vec<f32> = masses(&l, &ctx, 1).into_values().collect();
    assert_eq!(built, PYTHON_MASSES_2016);

    // The real development moves them: each car is heavier by its gap to the fastest car less its
    // slot's head start, at 0.0308 % per kg.
    let l = career(2016);
    let season = l.season();
    let o = overlay(&l, &ctx);
    assert_eq!((o.year, o.round), (2016, 1));
    assert_eq!(o.cars.len(), 11, "every 2016 team, Manor too");
    let pace: Vec<f32> = season.teams.iter().map(|t| season.pace_at(t.slot, 1)).collect();
    let best = pace.iter().copied().fold(f32::INFINITY, f32::min);
    let left: Vec<f32> = season
        .teams
        .iter()
        .zip(&pace)
        .map(|(t, p)| p - best - performance::head_start(2016, t.slot, &ctx.ai_slots).unwrap())
        .collect();
    let lift = left.iter().copied().fold(0.0_f32, f32::min);
    for (car, x) in o.cars.iter().zip(&left) {
        let want = (TOP_MASS + (x - lift) / 0.0308).clamp(TOP_MASS, MAX_MASS).round();
        assert_eq!(car.mass_kg, want, "{}", car.team_key);
    }
    let lightest = o.cars.iter().map(|c| c.mass_kg).fold(f32::INFINITY, f32::min);
    assert_eq!(
        lightest, TOP_MASS,
        "the car with the most pace left is the reference"
    );
    let car = |slot: u8| o.cars.iter().find(|c| c.slot == slot).unwrap();
    assert_eq!(
        (car(2).team_key.as_str(), car(2).erp.as_str(), car(2).stock),
        ("ferrari", "cars/teams/ferrari/wep/ferrari.erp", false)
    );
    assert_eq!(car(1).erp, "cars/teams/mercedes/wep/mercedes.erp");
    // Manor drives in Sauber's slot from the addon's own archive (assets/cars.json).
    assert_eq!(
        (car(11).team_key.as_str(), car(11).erp.as_str()),
        ("manor", "cars/extra/teams/sauber/wep/sauber.erp")
    );

    // A team that is faster than its slot allows makes the others heavier, not itself lighter.
    let mut fast = career(2016);
    without_development(&mut fast, 1);
    let manor = fast.season_mut().teams.iter_mut().find(|t| t.slot == 11).unwrap();
    manor.carry -= 3.804 + 0.5;
    let m = masses(&fast, &ctx, 1);
    assert_eq!(m[&11], TOP_MASS);
    let behind = |slot: u8| PYTHON_MASSES_2016[slot as usize - 1] - TOP_MASS;
    // Manor's slot is 0.437 % slower than Mercedes' and the car 0.5 % faster than Mercedes: the
    // rest gain the 0.937 % Manor is ahead of where its mass can go.
    let shift = (0.437 + 0.5) / 0.0308;
    assert!(
        (m[&1] - (TOP_MASS + shift)).abs() <= 1.0,
        "{} vs {}",
        m[&1],
        TOP_MASS + shift
    );
    assert!((m[&2] - (TOP_MASS + behind(2) + shift)).abs() <= 1.0);
}

#[test]
fn an_arrived_upgrade_lowers_the_mass() {
    let ctx = Ctx::new();
    let plain = career(2016);
    let mut developed = plain.clone();
    // Five aero levels for Ferrari (0.35 % of a lap, 11.4 kg), on the car from round 3.
    upgrades(&mut developed, 2, Part::Aero, 5, 3);
    assert_eq!(
        masses(&plain, &ctx, 2),
        masses(&developed, &ctx, 2),
        "not there yet"
    );
    let (before, after) = (masses(&plain, &ctx, 3), masses(&developed, &ctx, 3));
    assert!(
        after[&2] <= before[&2] - 9.0,
        "Ferrari {} with the parts vs {} without",
        after[&2],
        before[&2]
    );
    for slot in [1, 3, 4] {
        assert!((after[&slot] - before[&slot]).abs() <= 1.0, "slot {slot}");
    }

    // The overlay for round 3 carries it.
    let mut l = developed.clone();
    next_round(&mut l);
    next_round(&mut l);
    let o = overlay(&l, &ctx);
    assert_eq!(o.round, 3);
    let ferrari = o.cars.iter().find(|c| c.slot == 2).unwrap();
    assert_eq!(ferrari.mass_kg, after[&2]);
    assert_eq!(ferrari.parts, None, "the per-part feel is off");
    let layer = &handling_json(&files(&l, &ctx))["vehicle.handling"]["swofty.classic-seasons-2016/ferrari"];
    assert_eq!(
        layer["edits"],
        json!([{ "path": "chassis.mass", "op": "set", "value": after[&2] as i64 }])
    );
}

#[test]
fn mass_is_clamped() {
    let ctx = Ctx::new();
    let mut l = career(2016);
    l.season_mut()
        .teams
        .iter_mut()
        .find(|t| t.slot == 11)
        .unwrap()
        .pace = 20.0;
    let m = masses(&l, &ctx, 1);
    assert_eq!(m[&11], MAX_MASS);
    assert!(m.values().all(|m| (TOP_MASS..=MAX_MASS).contains(m)));
    assert!(m.values().all(|m| *m == m.round()), "whole kilograms");
}

#[test]
fn small_changes_keep_the_previous_masses() {
    let prev: BTreeMap<u8, f32> = [(1, 800.0), (2, 820.0)].into();
    assert!(performance::keeps(&prev, &[(1, 800.9), (2, 819.2)].into()));
    assert!(!performance::keeps(&prev, &[(1, 800.9), (2, 818.9)].into()));
    assert!(!performance::keeps(&prev, &[(1, 800.0)].into()), "a team less");
    assert_eq!(KEEP_BELOW_KG, 1.0);

    // Rounds 1 and 2 of 2016 are before the first development checkpoint: the same masses, the
    // same handling file.
    let ctx = Ctx::new();
    let mut l = career(2016);
    let first = files(&l, &ctx);
    next_round(&mut l);
    let second = files(&l, &ctx);
    assert_eq!(handling_json(&first), handling_json(&second));

    // Through the real 2016 to 2024 seasons, a round whose pace moved by less than a kilogram on
    // every car keeps the masses of the round before; any other takes its own.
    let mut kept = 0;
    for year in 2016..=2024 {
        let l = career(year);
        let rounds = l.season().rounds.len() as u8;
        let raw = |round: u8| performance::pace_masses(&l, &ctx.get(), round).unwrap();
        let mut before = masses(&l, &ctx, 1);
        for round in 2..=rounds {
            let now = masses(&l, &ctx, round);
            if now == before {
                if raw(round) != before {
                    kept += 1;
                }
            } else {
                assert_eq!(now, raw(round), "{year} round {round} takes the new masses");
            }
            before = now;
        }
    }
    assert!(kept > 0, "some rounds keep their masses although the pace moved");
}

#[test]
fn handling_layers_resolve_to_the_classic_seasons_archives() {
    let ctx = Ctx::new();
    let mut l = career(2016);
    l.options.game_parts = true;
    upgrades(&mut l, 2, Part::Aero, 2, 1);
    upgrades(&mut l, 2, Part::Powertrain, 1, 1);
    let o = overlay(&l, &ctx);
    let dir = std::env::temp_dir().join(format!("f1mp-performance-layers-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    for (rel, bytes) in gamesync::files(&o, &ctx.get()) {
        let path = dir.join(&rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }
    let season_dir = Path::new(FIXTURE).join(SEASON_2016);
    let addons = vec![
        (SEASON_2016.to_owned(), season_dir.clone()),
        (gamesync::ADDON_ID.to_owned(), dir.clone()),
    ];
    let (maps, notes) = handling::read_maps(&addons, &|_, _| None);
    assert!(notes.is_empty(), "{notes:?}");
    assert_eq!(
        maps.len(),
        11,
        "{:?}",
        maps.iter().map(|m| &m.id).collect::<Vec<_>>()
    );

    // Each layer changes the archive the season addon ships for that team.
    let shipped: Vec<String> = ctx.liveries["ops"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|op| op["op"] == "file.put")
        .map(|op| op["source"].as_str().unwrap().to_owned())
        .collect();
    let original = |rel: &str| PathBuf::from("game").join(rel);
    let addon_dir = |id: &str| {
        if id == SEASON_2016 {
            season_dir.clone()
        } else {
            dir.clone()
        }
    };
    let sources = handling::sources(&maps, &original, &addon_dir);
    for car in &o.cars {
        let id = format!("{SEASON_2016}/{}", car.team_key);
        let map = maps
            .iter()
            .find(|m| m.id == id)
            .unwrap_or_else(|| panic!("no {id}"));
        assert_eq!(
            map.target,
            Target::Addon {
                addon: SEASON_2016.into(),
                path: car.erp.clone()
            }
        );
        assert!(sources.contains(&season_dir.join(&car.erp)));
        assert!(
            shipped.contains(&car.erp) || car.erp.starts_with("cars/extra/"),
            "{}",
            car.erp
        );
        let leaf = |key: &str| {
            map.plan.steps.iter().find_map(|s| match s {
                Step::Leaf(l) if handling::norm(&l.key) == handling::norm(key) => Some(l.clone()),
                _ => None,
            })
        };
        let mass = leaf("Mass").expect("a mass");
        assert_eq!(mass.section, "Chassis");
        assert_eq!(mass.op, Op::Set(format!("{}", car.mass_kg as i64)));
        if car.slot == 2 {
            // Ferrari's parts: more downforce and power, and their lap time back on the mass.
            let p = car.parts.expect("Ferrari has parts");
            assert_eq!(leaf("FrontCoeffLiftMultiplier").unwrap().section, "F1Aero");
            assert_eq!(
                leaf("RearCoeffLiftMultiplier").unwrap().op,
                Op::Set(format!("{:.4}", p.lift))
            );
            assert_eq!(leaf("FuelPowerScaleStandard").unwrap().section, "FuelECU");
            assert_eq!(
                leaf("FuelPowerScaleLean").unwrap().op,
                Op::Set(format!("{:.4}", 0.9 * p.power))
            );
        } else {
            assert_eq!(car.parts, None);
            assert!(leaf("FrontCoeffLiftMultiplier").is_none());
        }
    }
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn parts_move_pace_from_the_mass_to_aero_and_power() {
    let ctx = Ctx::new();
    let mut l = career(2016);
    upgrades(&mut l, 2, Part::Aero, 3, 1);
    upgrades(&mut l, 2, Part::Powertrain, 2, 1);
    upgrades(&mut l, 2, Part::Chassis, 1, 1);
    let mass_only = overlay(&l, &ctx);
    l.options.game_parts = true;
    let parts = overlay(&l, &ctx);
    let ferrari = |o: &Overlay| o.cars.iter().find(|c| c.slot == 2).unwrap().clone();
    let (m, p) = (ferrari(&mass_only), ferrari(&parts));
    // 0.21 % aero and 0.12 % powertrain go to the scales; the chassis stays on the mass.
    let back = (0.21 + 0.12) / 0.0308;
    assert!(
        (p.mass_kg - m.mass_kg - back).abs() <= 1.0,
        "{} vs {}",
        p.mass_kg,
        m.mass_kg
    );
    let PartScales { lift, power } = p.parts.unwrap();
    assert!((lift - 0.95 * (1.0 + 0.06 * 0.21)).abs() < 1e-5, "{lift}");
    assert!((power - (1.0 + 0.035 * 0.12)).abs() < 1e-5, "{power}");
    // Teams without parts keep their mass and get no scales.
    for (a, b) in mass_only.cars.iter().zip(&parts.cars) {
        if a.slot != 2 {
            assert_eq!((a.mass_kg, b.parts), (b.mass_kg, None));
        }
    }
}

#[test]
fn rounds_of_2025_use_the_stock_archives() {
    let none = Ctx::none();
    let l = career(2025);
    let o = overlay(&l, &none);
    assert_eq!(o.cars.len(), 10);
    let mercedes = &o.cars[0];
    assert_eq!(
        (mercedes.slot, mercedes.team_key.as_str(), mercedes.stock),
        (1, "mercedes", true)
    );
    assert_eq!(
        mercedes.erp,
        "2025_asset_groups/f1_2025_vehicle_package/teams/mercedes/wep/mercedes.erp"
    );
    let layers = handling_json(&files(&l, &none));
    let layer = &layers["vehicle.handling"]["mercedes"];
    assert_eq!(layer["vtf"], json!({ "stock": mercedes.erp }));
    assert_eq!(layer["edits"][0]["op"], "set");
    // The stock cars make their own gaps: without development, each weighs the reference mass.
    let mut flat = l.clone();
    without_development(&mut flat, 1);
    assert!(masses(&flat, &none, 1).values().all(|m| *m == TOP_MASS));
    // Parts stay off for the stock cars: their lift multipliers differ by team.
    let mut parts = l.clone();
    parts.options.game_parts = true;
    upgrades(&mut parts, 2, Part::Aero, 2, 1);
    assert!(overlay(&parts, &none).cars.iter().all(|c| c.parts.is_none()));

    // The layers resolve to the stock archives.
    let dir = std::env::temp_dir().join(format!("f1mp-performance-stock-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(dir.join("registry")).unwrap();
    std::fs::write(dir.join(HANDLING_REGISTRY), serde_json::to_vec(&layers).unwrap()).unwrap();
    let (maps, notes) = handling::read_maps(&[(gamesync::ADDON_ID.to_owned(), dir.clone())], &|_, _| None);
    assert!(notes.is_empty(), "{notes:?}");
    assert_eq!(maps.len(), 10);
    assert!(maps.iter().all(|m| matches!(&m.target, Target::Stock(rel)
        if rel.starts_with("2025_asset_groups/f1_2025_vehicle_package/teams/"))));
    std::fs::remove_dir_all(&dir).ok();

    // 2026 races the game's 2026 cars, Cadillac too, and adds the league's mass to their own.
    let l = career(2026);
    let o = overlay(&l, &none);
    assert_eq!(o.cars.len(), 11);
    assert_eq!(
        o.cars[10].erp,
        "2025_asset_groups/f1_2026_vehicle_package/teams/gm_cadillac/wep/gm_cadillac.erp"
    );
    let layers = handling_json(&files(&l, &none));
    for car in &o.cars {
        let edit = &layers["vehicle.handling"][&car.team_key]["edits"][0];
        assert_eq!(edit["op"], "add");
        assert_eq!(edit["value"], json!((car.mass_kg - TOP_MASS) as i64));
    }
}

#[test]
fn equal_performance_writes_no_cars() {
    let ctx = Ctx::new();
    let mut l = career(2016);
    l.options.rules.equal_performance = Some(true);
    let o = overlay(&l, &ctx);
    assert!(o.cars.is_empty());
    assert!(
        gamesync::files(&o, &ctx.get())
            .iter()
            .all(|(p, _)| p != HANDLING_REGISTRY)
    );
    l.options.rules.equal_performance = Some(false);
    assert_eq!(overlay(&l, &ctx).cars.len(), 11);
    // A Classic Seasons year without its AI slots writes nothing rather than wrong masses.
    let mut no_slots = Ctx::new();
    no_slots.ai_slots = Value::Null;
    assert!(overlay(&l, &no_slots).cars.is_empty());
}

#[test]
fn performance_files_are_byte_identical() {
    let ctx = Ctx::new();
    let mut anna = career(2016);
    anna.options.game_parts = true;
    upgrades(&mut anna, 2, Part::Aero, 1, 1);
    let ben: League = serde_json::from_str(&serde_json::to_string(&anna).unwrap()).unwrap();
    let mine = files(&anna, &ctx);
    assert_eq!(mine, files(&ben, &Ctx::new()));
    let (_, bytes) = mine.iter().find(|(p, _)| p == HANDLING_REGISTRY).unwrap();
    assert_eq!(
        bytes,
        &gamesync::canonical_json(&serde_json::from_slice(bytes).unwrap()),
        "canonical"
    );
    let text = std::str::from_utf8(bytes).unwrap();
    assert!(!text.contains('\r') && !text.contains('\\'), "{text}");
    // The masses reach the addon's hash: another pace is another overlay.
    let mut faster = anna.clone();
    upgrades(&mut faster, 2, Part::Aero, 3, 1);
    let hash = |f: &[gamesync::File]| {
        let (_, b) = f.iter().find(|(p, _)| p == gamesync::LEAGUE_FILE).unwrap();
        serde_json::from_slice::<Value>(b).unwrap()["hash"].clone()
    };
    assert_ne!(hash(&files(&faster, &ctx)), hash(&mine));
}

/// The build of a profile with the overlay sets the league's masses in the team archives: a
/// cut-down Classic Seasons 2016 with a handling file in each team archive, and the game's own
/// archives for 2025.
mod build {
    use super::*;
    use crate::baseline::Identity;
    use crate::careersync;
    use crate::store::Store;
    use swofty_formats::bxml::{Document, Element, Kind};
    use swofty_formats::erp::{Erp, ErpFile, Fragment, HEADER_LEN, MAGIC, Resource, STORED};
    use swofty_formats::gvdb::Gvdb;

    const PROFILE: &str = "career";
    const CORE: &str = "swofty.classic-seasons";

    fn el(name: &str, attrs: &[(&str, &str)], children: Vec<Element>) -> Element {
        Element {
            kind: Kind::Element,
            pad: 0,
            name: name.as_bytes().to_vec(),
            attrs: attrs
                .iter()
                .map(|(k, v)| (k.as_bytes().to_vec(), v.as_bytes().to_vec()))
                .collect(),
            children,
            end_pad: 0,
        }
    }

    /// A team archive with one handling file whose car weighs `mass`.
    fn archive(uri: &str, mass: &str) -> Vec<u8> {
        let doc = Document {
            root: el(
                "VTF",
                &[],
                vec![
                    el("Chassis", &[], vec![el("Mass", &[("value", mass)], vec![])]),
                    el(
                        "F1Aero",
                        &[],
                        vec![
                            el("FrontCoeffLiftMultiplier", &[("value", "0.95")], vec![]),
                            el("RearCoeffLiftMultiplier", &[("value", "0.95")], vec![]),
                        ],
                    ),
                ],
            ),
            tail: Vec::new(),
        };
        let data = doc.to_bytes();
        let mut header = [0u8; HEADER_LEN];
        header[..4].copy_from_slice(MAGIC);
        header[4..8].copy_from_slice(&3u32.to_le_bytes());
        Erp {
            header,
            version: 3,
            resources: vec![Resource {
                name: uri.into(),
                kind: swofty_formats::erp::kind("VTF").unwrap(),
                unk: vec![0; 4],
                fragments: vec![Fragment {
                    name: *b"temp",
                    offset: 0,
                    size: data.len() as u64,
                    flags: 0x10,
                    compression: STORED,
                    packed: data.len() as u64,
                    raw: data,
                }],
                hash: [0; 16],
            }],
        }
        .to_bytes()
    }

    fn mass_in(path: &Path) -> String {
        let file = ErpFile::open(path).unwrap();
        let r = &file.erp.resources[0];
        let doc = Document::parse(&file.load(r).unwrap().fragments[0].data().unwrap()).unwrap();
        let chassis = doc.root.child(b"Chassis").unwrap();
        String::from_utf8_lossy(chassis.child(b"Mass").unwrap().attr(b"value").unwrap()).into_owned()
    }

    fn uri(folder: &str) -> String {
        format!("eaid://x_vehicle_package/teams/{folder}/handling/{folder}.vtf")
    }

    struct Fixture {
        root: PathBuf,
        store: Store,
        game: crate::install::Game,
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.root).ok();
        }
    }

    /// A data folder with the Classic Seasons core from the fixture and a Classic Seasons 2016 that
    /// ships a team archive per team (its liveries, and Manor's archive in assets/cars.json), on
    /// in [`PROFILE`]; and a game folder with the stock 2025 team archives.
    fn setup(name: &str) -> Fixture {
        let root = std::env::temp_dir().join(format!("f1mp-performance-build-{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&root).ok();
        let store = Store::new(root.join("data"));
        store.install_addon(&Path::new(FIXTURE).join(CORE)).unwrap();

        let src = root.join("src").join(SEASON_2016);
        std::fs::create_dir_all(src.join("registry")).unwrap();
        std::fs::create_dir_all(src.join("assets")).unwrap();
        std::fs::copy(
            Path::new(FIXTURE).join(SEASON_2016).join("registry/content.json"),
            src.join("registry/content.json"),
        )
        .unwrap();
        let mut manifest = fixture(&format!("{SEASON_2016}/swofty.json"));
        manifest["operations"] = json!({ "assets": ["assets/liveries.json", "assets/cars.json"] });
        std::fs::write(src.join("swofty.json"), serde_json::to_vec(&manifest).unwrap()).unwrap();
        let put =
            |source: &str, target: &str| json!({ "op": "file.put", "source": source, "target": target });
        let mut liveries = Vec::new();
        for f in [
            "mercedes",
            "redbull",
            "ferrari",
            "williams",
            "force_india",
            "lotus",
            "toro_rosso",
            "sauber",
            "mclaren",
            "haas",
        ] {
            let source = format!("cars/teams/{f}/wep/{f}.erp");
            liveries.push(put(
                &source,
                &format!("2025_asset_groups/cs_2016_vehicle_package/teams/{f}/wep/{f}.erp"),
            ));
            std::fs::create_dir_all(src.join(&source).parent().unwrap()).unwrap();
            std::fs::write(src.join(&source), archive(&uri(f), "830")).unwrap();
        }
        let manor = "cars/extra/teams/sauber/wep/sauber.erp";
        std::fs::create_dir_all(src.join(manor).parent().unwrap()).unwrap();
        std::fs::write(src.join(manor), archive(&uri("sauber"), "830")).unwrap();
        let ops = |ops: Vec<Value>| serde_json::to_vec(&json!({ "schema_version": 1, "ops": ops })).unwrap();
        std::fs::write(src.join("assets/liveries.json"), ops(liveries)).unwrap();
        std::fs::write(
            src.join("assets/cars.json"),
            ops(vec![put(
                manor,
                "2025_asset_groups/cx_2016_vehicle_package/teams/sauber/wep/sauber.erp",
            )]),
        )
        .unwrap();
        store.install_addon(&src).unwrap();
        store
            .update_profile(PROFILE, |p| {
                p.set_enabled(CORE, true)?;
                p.set_enabled(SEASON_2016, true)
            })
            .unwrap();

        let dir = root.join("game");
        std::fs::create_dir_all(dir.join("game_package/gamedatabases")).unwrap();
        std::fs::write(dir.join(crate::GAME_EXE), "exe").unwrap();
        std::fs::write(
            dir.join(crate::build::GVDB),
            Gvdb { records: Vec::new() }.to_bytes(),
        )
        .unwrap();
        for (_, f) in &performance::STOCK_FOLDERS[..10] {
            let rel = format!("2025_asset_groups/f1_2025_vehicle_package/teams/{f}/wep/{f}.erp");
            std::fs::create_dir_all(dir.join(&rel).parent().unwrap()).unwrap();
            std::fs::write(dir.join(&rel), archive(&uri(f), "805")).unwrap();
        }
        let game = crate::install::Game {
            identity: Identity::detect(&dir, Some(1)).unwrap(),
            dir,
            user_reg: None,
        };
        Fixture { root, store, game }
    }

    fn built_archive(built: &crate::build::Built, rel: &str) -> PathBuf {
        built
            .plan
            .data
            .iter()
            .chain(&built.plan.added)
            .find(|(k, _)| k.eq_ignore_ascii_case(rel))
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| panic!("the build has no {rel}"))
    }

    #[test]
    fn the_launch_build_sets_the_league_masses_in_the_team_archives() {
        let _pin = crate::builtin::LoaderPin::none();
        let s = setup("cs");
        let l = career(2016);
        let ctx = careersync::context(&s.store, 2016).unwrap();
        let o = gamesync::overlay(&l, "c-1", "Anna & Ben", &ctx.get()).unwrap();
        assert_eq!(o.cars.len(), 11);
        careersync::prepare_as(&s.store, PROFILE, &o, false).unwrap();
        let built = crate::build::build(&s.store, PROFILE, &s.game).unwrap();
        assert!(
            built.warnings.iter().all(|w| !w.starts_with("handling")),
            "{:?}",
            built.warnings
        );
        for car in &o.cars {
            let target = if car.slot == 11 {
                "2025_asset_groups/cx_2016_vehicle_package/teams/sauber/wep/sauber.erp".to_owned()
            } else {
                format!(
                    "2025_asset_groups/cs_2016_vehicle_package/teams/{0}/wep/{0}.erp",
                    car.team_key
                )
            };
            assert_eq!(
                mass_in(&built_archive(&built, &target)),
                format!("{}", car.mass_kg as i64),
                "{}",
                car.team_key
            );
        }
        // The addon's own archives stay as they were.
        let ferrari = s
            .store
            .addons_dir()
            .join(SEASON_2016)
            .join("cars/teams/ferrari/wep/ferrari.erp");
        assert_eq!(mass_in(&ferrari), "830");

        // Restored, the build ships the addon's archives again.
        careersync::restore(&s.store, PROFILE).unwrap();
        let back = crate::build::build(&s.store, PROFILE, &s.game).unwrap();
        let target = "2025_asset_groups/cs_2016_vehicle_package/teams/ferrari/wep/ferrari.erp";
        assert_eq!(mass_in(&built_archive(&back, target)), "830");
    }

    #[test]
    fn the_launch_build_sets_the_league_masses_in_the_stock_2025_cars() {
        let _pin = crate::builtin::LoaderPin::none();
        let s = setup("stock");
        let mut l = career(2025);
        upgrades(&mut l, 2, Part::Aero, 5, 1);
        let ctx = careersync::context(&s.store, 2025).unwrap();
        let o = gamesync::overlay(&l, "c-1", "Anna & Ben", &ctx.get()).unwrap();
        assert!(o.cars.iter().all(|c| c.stock));
        careersync::prepare_as(&s.store, PROFILE, &o, false).unwrap();
        let built = crate::build::build(&s.store, PROFILE, &s.game).unwrap();
        for car in &o.cars {
            assert_eq!(
                mass_in(&built_archive(&built, &car.erp)),
                format!("{}", car.mass_kg as i64),
                "{}",
                car.team_key
            );
        }
    }
}
