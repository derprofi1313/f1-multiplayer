//! Tests of the league's gamesync::identity (W2-9): the players' names, codes, numbers and
//! nationalities in the game, as registry layers on the Classic Seasons box and as the database
//! fallback, and the AI-driven seat of an absent player matched under that identity.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::{Value, json};

use super::gamesync::identity::{self, FLAGS, GAME_NATIONALITIES, ISO_CODES, game_nationality};
use super::gamesync::{self, IDENTITY_DB, IDENTITY_LANG, IDENTITY_REGISTRY, SyncContext};
use super::*;
use crate::manifest::AddonManifest;

const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/classic-seasons-mini");

/// The fixture's Classic Seasons 2016 files, as `careersync::context` reads them.
struct Ctx {
    content: Value,
    cs_drivers: Value,
    liveries: Value,
    ai_slots: Value,
}

impl Ctx {
    fn new() -> Self {
        let read = |rel: &str| -> Value {
            serde_json::from_slice(&std::fs::read(Path::new(FIXTURE).join(rel)).unwrap()).unwrap()
        };
        Self {
            content: read("swofty.classic-seasons-2016/registry/content.json"),
            cs_drivers: read("swofty.classic-seasons/registry/drivers.json"),
            liveries: read("swofty.classic-seasons-2016/assets/liveries.json"),
            ai_slots: read("swofty.classic-seasons/data/ai_slots.json"),
        }
    }

    fn get(&self, loader_applies_rows: bool) -> SyncContext<'_> {
        SyncContext {
            content: &self.content,
            cs_drivers: &self.cs_drivers,
            liveries: &self.liveries,
            ai_slots: &self.ai_slots,
            loader_applies_rows,
        }
    }
}

fn actor(id: &str, name: &str) -> Actor {
    Actor {
        id: id.into(),
        name: name.into(),
        owner: id == "u-anna",
    }
}

fn anna() -> Actor {
    actor("u-anna", "Anna Müller")
}

fn ben() -> Actor {
    actor("u-ben", "Ben")
}

/// A career from `year` with the game identity on: each player as (actor, number, slot, the
/// driver whose seat they take); the first is the owner.
fn career(year: u16, players: &[(Actor, u8, u8, &str)]) -> League {
    let setup = Setup {
        first_year: year,
        last_year: year.max(Setup::default().last_year),
        ..Setup::default()
    };
    let (owner, number, ..) = &players[0];
    let mut l = League::start(setup, owner, Some(*number), 42).unwrap();
    for (a, number, ..) in &players[1..] {
        apply(
            &mut l,
            a,
            Action::Enter {
                number: Some(*number),
            },
        )
        .unwrap();
    }
    for (a, _, slot, replaces) in players {
        apply(
            &mut l,
            a,
            Action::Seat {
                slot: *slot,
                replaces: Some((*replaces).to_owned()),
            },
        )
        .unwrap();
    }
    apply(&mut l, owner, Action::Begin).unwrap();
    l.options.game_identity = true;
    l
}

/// Anna (#42, German) in Kimi Räikkönen's Ferrari and Ben (#8, Romain Grosjean's) in Felipe
/// Nasr's Sauber, a driver Classic Seasons adds.
fn anna_and_ben() -> League {
    let mut l = career(
        2016,
        &[(anna(), 42, 2, "kimi_raikkonen"), (ben(), 8, 9, "felipe_nasr")],
    );
    l.players[0].nationality = Some("DEU".into());
    l
}

fn files(l: &League, ctx: &SyncContext) -> Vec<(String, Vec<u8>)> {
    gamesync::files(&gamesync::overlay(l, "c-1", "Anna & Ben", ctx).unwrap(), ctx)
}

fn json_at(files: &[(String, Vec<u8>)], path: &str) -> Value {
    let (_, bytes) = files
        .iter()
        .find(|(p, _)| p == path)
        .unwrap_or_else(|| panic!("no {path}"));
    serde_json::from_slice(bytes).unwrap()
}

fn paths(files: &[(String, Vec<u8>)]) -> Vec<&str> {
    files.iter().map(|(p, _)| p.as_str()).collect()
}

fn identity_of(l: &League, player: &str) -> GameIdentity {
    gamesync::identity_of(l, player).expect("an identity")
}

#[test]
fn a_stock_driver_and_a_classic_seasons_driver_get_identity_layers() {
    let ctx = Ctx::new();
    let l = anna_and_ben();
    let files = files(&l, &ctx.get(true));
    assert_eq!(
        paths(&files),
        ["league.json", IDENTITY_REGISTRY, "swofty.json"],
        "the core applies the rows: no fallback"
    );
    let registry = json_at(&files, IDENTITY_REGISTRY);
    let layers = registry["driver.identity"].as_object().unwrap();
    assert_eq!(
        layers.keys().collect::<Vec<_>>(),
        ["game/kimi_raikkonen", "swofty.classic-seasons/felipe_nasr"]
    );
    assert_eq!(
        layers["game/kimi_raikkonen"],
        json!({
            "names": {
                "first": { "literal": "Anna" },
                "last": { "literal": "MÜLLER" },
                "full": { "literal": "Anna MÜLLER" },
                "code": "MUL",
            },
            "number": 42,
            "nationality": "germany",
        })
    );
    // Ben's 8 is Grosjean's that year: he races with Nasr's 12. He picked no nationality, so
    // Nasr's flag stays.
    assert_eq!(
        layers["swofty.classic-seasons/felipe_nasr"],
        json!({
            "names": {
                "first": { "literal": "" },
                "last": { "literal": "BEN" },
                "full": { "literal": "BEN" },
                "code": "BEN",
            },
            "number": 12,
        })
    );
    let o = gamesync::overlay(&l, "c-1", "x", &ctx.get(true)).unwrap();
    let ben = o.identities.iter().find(|i| i.player == "u-ben").unwrap();
    assert!(ben.number_clash && !o.identities[0].number_clash);
    assert_eq!((ben.year, ben.slot, ben.team_key.as_str()), (2016, 9, "sauber"));
    assert_eq!(ben.replaces, "felipe_nasr");

    // The same names, codes and numbers without the Classic Seasons files (the briefing's).
    let bare = gamesync::overlay(&l, "c-1", "x", &SyncContext::empty()).unwrap();
    for (a, b) in o.identities.iter().zip(&bare.identities) {
        assert_eq!(
            (&a.player, &a.full, &a.code, a.number, &a.nationality, &a.replaces),
            (&b.player, &b.full, &b.code, b.number, &b.nationality, &b.replaces)
        );
    }
    assert_eq!(bare.identities[1].team_key, "sauber");
}

#[test]
fn the_team_content_restates_both_seats() {
    let ctx = Ctx::new();
    let package = |l: &League| -> Value {
        let registry = json_at(&files(l, &ctx.get(true)), IDENTITY_REGISTRY);
        let packages = registry["content.package"].as_object().unwrap().clone();
        assert_eq!(
            packages.keys().collect::<Vec<_>>(),
            ["swofty.classic-seasons-2016/f1_2016"]
        );
        packages["swofty.classic-seasons-2016/f1_2016"].clone()
    };
    assert_eq!(
        package(&anna_and_ben()),
        json!({ "teams": [
            { "key": "ferrari", "drivers": [
                { "driver": "game/sebastian_vettel", "number": 5, "look": null },
                { "driver": "game/kimi_raikkonen", "number": 42, "look": null },
            ] },
            { "key": "sauber", "drivers": [
                { "driver": "game/marcus_ericsson", "number": 9, "look": null },
                { "driver": "swofty.classic-seasons/felipe_nasr", "number": 12, "look": null },
            ] },
        ] })
    );
    // Both players in one team: one team, both seats theirs.
    let l = career(
        2016,
        &[
            (anna(), 42, 2, "kimi_raikkonen"),
            (ben(), 43, 2, "sebastian_vettel"),
        ],
    );
    assert_eq!(
        package(&l),
        json!({ "teams": [
            { "key": "ferrari", "drivers": [
                { "driver": "game/sebastian_vettel", "number": 43, "look": null },
                { "driver": "game/kimi_raikkonen", "number": 42, "look": null },
            ] },
        ] })
    );
}

#[test]
fn fallback_ops_when_the_loader_doesnt_apply_rows() {
    let ctx = Ctx::new();
    let fb = files(&anna_and_ben(), &ctx.get(false));
    assert_eq!(
        paths(&fb),
        [
            IDENTITY_DB,
            IDENTITY_LANG,
            "league.json",
            IDENTITY_REGISTRY,
            "swofty.json"
        ]
    );
    let manifest = AddonManifest::parse(std::str::from_utf8(&fb.last().unwrap().1).unwrap()).unwrap();
    assert_eq!(manifest.operations.database, [IDENTITY_DB]);
    assert_eq!(manifest.operations.assets, [IDENTITY_LANG]);

    let set = |select: &str, field: &str, value: &str| json!({ "op": "set", "select": select, "field": field, "value": value });
    let kimi = "entrants.drivers.driver[_metadata_=kimi_raikkonen]";
    let nasr = "entrants.drivers.driver[_metadata_=felipe_nasr]";
    assert_eq!(
        json_at(&fb, IDENTITY_DB),
        json!({ "schema_version": 1, "ops": [
            set(kimi, "first_name", "lng_f1mp_kimi_raikkonen_first"),
            set(kimi, "last_name", "lng_f1mp_kimi_raikkonen_last"),
            set(kimi, "full_name", "lng_f1mp_kimi_raikkonen_full"),
            set(kimi, "initials", "MUL"),
            set(kimi, "nationality", "germany"),
            set(kimi, "car_number", "42"),
            set("entrants.entrant_lists.entrants.entrant[_metadata_=cs2016-kimi_raikkonen]", "car_number", "42"),
            set(nasr, "first_name", "lng_f1mp_felipe_nasr_first"),
            set(nasr, "last_name", "lng_f1mp_felipe_nasr_last"),
            set(nasr, "full_name", "lng_f1mp_felipe_nasr_full"),
            set(nasr, "initials", "BEN"),
            set(nasr, "car_number", "12"),
            set("entrants.entrant_lists.entrants.entrant[_metadata_=cs2016-felipe_nasr]", "car_number", "12"),
        ] })
    );
    let lang =
        |key: &str, value: &str| json!({ "op": "lang.set", "language": "*", "key": key, "value": value });
    assert_eq!(
        json_at(&fb, IDENTITY_LANG),
        json!({ "schema_version": 1, "ops": [
            lang("lng_f1mp_kimi_raikkonen_first", "Anna"),
            lang("lng_f1mp_kimi_raikkonen_last", "MÜLLER"),
            lang("lng_f1mp_kimi_raikkonen_full", "Anna MÜLLER"),
            lang("lng_f1mp_felipe_nasr_first", ""),
            lang("lng_f1mp_felipe_nasr_last", "BEN"),
            lang("lng_f1mp_felipe_nasr_full", "BEN"),
        ] })
    );

    // The game's own seasons: the stock rows, no Classic Seasons entrant list and no content.
    let l = career(2025, &[(anna(), 42, 2, "lewis_hamilton")]);
    let none = Value::Null;
    let bare = SyncContext {
        content: &none,
        cs_drivers: &ctx.cs_drivers,
        liveries: &none,
        ai_slots: &none,
        loader_applies_rows: false,
    };
    let game_files = files(&l, &bare);
    let ops = json_at(&game_files, IDENTITY_DB)["ops"]
        .as_array()
        .unwrap()
        .clone();
    assert!(
        ops.iter()
            .all(|op| op["select"] == "entrants.drivers.driver[_metadata_=lewis_hamilton]"),
        "{ops:?}"
    );
    let registry = json_at(&game_files, IDENTITY_REGISTRY);
    assert!(registry["driver.identity"]["game/lewis_hamilton"].is_object());
    assert!(registry.get("content.package").is_none());
}

/// Each car's number on `l`'s grid as the game shows it: the box's drivers whose rows stay
/// theirs, and the players.
fn grid_numbers(l: &League) -> Vec<u8> {
    let o = gamesync::overlay(l, "c-1", "x", &SyncContext::empty()).unwrap();
    let replaced: BTreeSet<&str> = o.identities.iter().map(|i| i.replaces.as_str()).collect();
    catalog_year(l.year())
        .unwrap()
        .teams
        .iter()
        .flat_map(|t| &t.drivers)
        .filter(|d| !replaced.contains(d.key.as_str()))
        .map(|d| d.number)
        .chain(o.identities.iter().map(|i| i.number))
        .collect()
}

#[test]
fn numbers_never_clash_on_the_grid() {
    for year in FIRST_YEAR..=LAST_YEAR {
        let cat = catalog_year(year).unwrap();
        let (t1, t2, t3) = (&cat.teams[0], &cat.teams[1], &cat.teams[2]);
        let (a, b, c) = (&t1.drivers[0], &t2.drivers[0], &t3.drivers[1]);
        // Anna asks for a number of another team's driver, Ben for the number of Anna's seat,
        // Carl for one nobody has that year, Dora for the number of her own seat.
        let free = (1..=99)
            .find(|n| !cat.teams.iter().flat_map(|t| &t.drivers).any(|d| d.number == *n))
            .unwrap();
        let mut players = vec![
            (anna(), t3.drivers[0].number, t1.slot, a.key.as_str()),
            (ben(), a.number, t2.slot, b.key.as_str()),
            (actor("u-carl", "Carl"), free, t3.slot, c.key.as_str()),
        ];
        let d = &cat.teams[3].drivers[0];
        players.push((
            actor("u-dora", "Dora"),
            d.number,
            cat.teams[3].slot,
            d.key.as_str(),
        ));
        let l = career(year, &players);
        let numbers = grid_numbers(&l);
        let unique: BTreeSet<u8> = numbers.iter().copied().collect();
        assert_eq!(unique.len(), numbers.len(), "{year}: {numbers:?}");

        let shown = |id: &str| identity_of(&l, id);
        assert_eq!(
            (shown("u-anna").number, shown("u-anna").number_clash),
            (a.number, true),
            "{year}"
        );
        assert_eq!(
            (shown("u-ben").number, shown("u-ben").number_clash),
            (b.number, true),
            "{year}"
        );
        assert_eq!(
            (shown("u-carl").number, shown("u-carl").number_clash),
            (free, false),
            "{year}"
        );
        assert_eq!(
            (shown("u-dora").number, shown("u-dora").number_clash),
            (d.number, false),
            "{year}"
        );
    }
}

#[test]
fn codes_are_unique_on_the_grid() {
    let l = career(
        2016,
        &[
            (actor("u-anna", "Lewis Hamster"), 40, 2, "kimi_raikkonen"),
            (actor("u-ben", "Hamid Hamilton"), 41, 4, "jenson_button"),
            (actor("u-carl", "Jo"), 42, 9, "felipe_nasr"),
            (actor("u-dora", "Zoë Ångström"), 43, 10, "esteban_gutierrez"),
            (actor("u-emil", "Nyck de Vries"), 45, 11, "rio_haryanto"),
        ],
    );
    let o = gamesync::overlay(&l, "c-1", "x", &SyncContext::empty()).unwrap();
    let code = |id: &str| o.identities.iter().find(|i| i.player == id).unwrap().code.clone();
    // HAM is Lewis Hamilton's, who keeps his seat.
    assert_eq!(code("u-anna"), "HAS");
    assert_eq!(code("u-ben"), "HAI");
    assert_eq!(code("u-carl"), "JOX");
    assert_eq!(code("u-dora"), "ANG");
    assert_eq!(code("u-emil"), "DEV");
    let e = o.identities.iter().find(|i| i.player == "u-emil").unwrap();
    assert_eq!((e.first.as_str(), e.last.as_str()), ("Nyck", "DE VRIES"));

    // Every code is the registry's form, and none is another driver's on the grid.
    let replaced: BTreeSet<&str> = o.identities.iter().map(|i| i.replaces.as_str()).collect();
    let grid: Vec<String> = catalog_year(2016)
        .unwrap()
        .teams
        .iter()
        .flat_map(|t| &t.drivers)
        .filter(|d| !replaced.contains(d.key.as_str()))
        .map(|d| {
            let (_, last) = identity::split_name(&d.name);
            last.chars().filter(char::is_ascii_alphabetic).take(3).collect()
        })
        .collect();
    let players: Vec<String> = o.identities.iter().map(|i| i.code.clone()).collect();
    for c in &players {
        assert!(
            c.len() == 3 && c.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit()),
            "{c}"
        );
        assert!(!grid.contains(c), "{c} is another driver's");
    }
    let unique: BTreeSet<&String> = players.iter().collect();
    assert_eq!(unique.len(), players.len(), "{players:?}");
}

#[test]
fn every_nationality_maps_to_a_game_label() {
    for (iso, name) in driver::NATIONALITIES {
        let label = game_nationality(iso).unwrap_or_else(|| panic!("{iso} ({name}) has no game label"));
        assert!(GAME_NATIONALITIES.contains(&label), "{iso}: {label}");
    }
    let iso: Vec<&str> = ISO_CODES.split_whitespace().collect();
    assert!(iso.len() >= 249);
    assert!(
        iso.iter()
            .all(|c| c.len() == 3 && c.bytes().all(|b| b.is_ascii_uppercase()))
    );
    assert_eq!(
        iso.iter().collect::<BTreeSet<_>>().len(),
        iso.len(),
        "no code twice"
    );
    // Each flag is one of the game's labels and has its country's code; every label but the
    // game's "other" has a flag.
    for (code, label) in FLAGS {
        assert!(GAME_NATIONALITIES.contains(label), "{label}");
        assert!(
            iso.contains(code) || ["ENG", "SCO", "WAL", "NIR"].contains(code),
            "{code}"
        );
    }
    let flagged: BTreeSet<&str> = FLAGS.iter().map(|(_, l)| *l).collect();
    let labels: BTreeSet<&str> = GAME_NATIONALITIES.iter().copied().collect();
    assert_eq!(labels.len(), 90);
    assert_eq!(
        labels.difference(&flagged).collect::<Vec<_>>(),
        [&identity::DEFAULT_NATIONALITY]
    );
    assert_eq!(game_nationality("deu"), Some("germany"));
    assert_eq!(game_nationality("GBR"), Some("britain"));
    assert_eq!(
        game_nationality("RUS"),
        Some("default"),
        "no Russian flag in the game"
    );
    assert_eq!(game_nationality("XYZ"), None);
    // Classic Seasons' drivers use the same labels.
    let cs = Ctx::new().cs_drivers;
    for (key, d) in cs["driver.identity"].as_object().unwrap() {
        let n = d["nationality"].as_str().unwrap();
        assert!(GAME_NATIONALITIES.contains(&n), "{key}: {n}");
    }
}

#[test]
fn identity_files_load_through_the_registry_index() {
    use crate::careersync;
    use crate::stdlib::{data, deps, index::Index};
    use crate::store::Store;

    let _pin = crate::builtin::LoaderPin::none();
    let root = std::env::temp_dir().join(format!("f1mp-identity-index-{}", std::process::id()));
    std::fs::remove_dir_all(&root).ok();
    let store = Store::new(root.join("data"));
    let season = "swofty.classic-seasons-2016";
    for id in ["swofty.classic-seasons", season] {
        store.install_addon(&Path::new(FIXTURE).join(id)).unwrap();
    }
    store
        .update_profile("career", |p| {
            p.set_enabled("swofty.classic-seasons", true)?;
            p.set_enabled(season, true)
        })
        .unwrap();
    let l = anna_and_ben();
    let ctx = careersync::context(&store, 2016).unwrap();
    assert!(
        !ctx.loader_applies_rows,
        "no loader here: the fallback comes along"
    );
    let overlay = gamesync::overlay(&l, "c-1", "Anna & Ben", &ctx.get()).unwrap();
    careersync::prepare_as(&store, "career", &overlay, false).unwrap();

    let installed = store.addons().unwrap();
    let order: Vec<String> = store
        .profile("career")
        .unwrap()
        .addons
        .iter()
        .filter(|a| a.enabled)
        .map(|a| a.id.clone())
        .collect();
    assert_eq!(
        order.last().map(String::as_str),
        Some(gamesync::ADDON_ID),
        "{order:?}"
    );
    let std = deps::read_all(&store, &installed);
    let placed = Index::plan("career", &order, &installed, &std, &store.addons_dir()).expect("an index");
    let entry = placed
        .index
        .addons
        .iter()
        .find(|a| a.id == gamesync::ADDON_ID)
        .expect("the overlay is in the index");
    assert_eq!(
        entry.data,
        [format!("addons/{}/{IDENTITY_REGISTRY}", gamesync::ADDON_ID)]
    );
    assert!(!entry.code);
    let report = deps::for_order("career", &order, &installed, &std);
    assert!(!report.stops(), "{:?}", report.stop_text());

    // Every layer goes on a record the game or an addon before it in load order has.
    let (_, src) = placed
        .files
        .iter()
        .find(|(rel, _)| rel.ends_with(IDENTITY_REGISTRY))
        .unwrap();
    let layers: Value = serde_json::from_slice(&std::fs::read(src).unwrap()).unwrap();
    let mut records: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for id in &order[..order.len() - 1] {
        let dir = store.addons_dir().join(id);
        for rel in data::files(&dir) {
            for r in data::records(id, &std::fs::read_to_string(dir.join(rel)).unwrap()) {
                records.entry(r.registry).or_default().insert(r.id);
            }
        }
    }
    let mut targets = 0;
    for (registry, by_target) in layers.as_object().unwrap() {
        for target in by_target.as_object().unwrap().keys() {
            assert!(
                target.starts_with("game/") || records.get(registry).is_some_and(|r| r.contains(target)),
                "{registry} {target} resolves to no record"
            );
            targets += 1;
        }
    }
    assert_eq!(targets, 3, "two drivers and the season's package");
    std::fs::remove_dir_all(&root).ok();
}

/// The game's report of a race: the season's grid in grid order, `reporter`'s car first and
/// winning; every other car as the game's AI drives it, under the name and number the game
/// shows for it (`shown` for a player's).
fn report_of(l: &League, reporter: &str, shown: &BTreeMap<&str, (&str, u8)>) -> RaceReport {
    let season = l.season();
    let mut cars: Vec<ReportCar> = season
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let mine = e.player.as_deref() == Some(reporter);
            let (name, number) = match e.player.as_deref() {
                Some(p) if !mine => shown[p],
                _ => (e.name.as_str(), e.number),
            };
            ReportCar {
                name: if mine {
                    "Player".into()
                } else {
                    name.to_uppercase()
                },
                number,
                team: e.slot,
                ai: !mine,
                mine,
                position: 0,
                grid: i as u8 + 1,
                laps: 30,
                status: 3,
                best_lap_ms: 90_000 + i as u32 * 10,
                time_ms: 3_600_000 + i as u64 * 1000,
                penalty_s: 0,
            }
        })
        .collect();
    cars.sort_by_key(|c| (!c.mine, c.grid));
    for (i, c) in cars.iter_mut().enumerate() {
        c.position = i as u8 + 1;
    }
    RaceReport {
        session: 0xabc,
        track: Some(0),
        session_type: 15,
        cars,
        ..RaceReport::default()
    }
}

#[test]
fn a_renamed_ai_seat_still_counts_for_the_absent_player() {
    // Ben (#31, no 2016 driver's) is away; the game's AI drives Jenson Button's McLaren as
    // "Ben BECKER" #31.
    let ben = actor("u-ben", "Ben Becker");
    let mut l = career(
        2016,
        &[(anna(), 42, 2, "kimi_raikkonen"), (ben, 31, 4, "jenson_button")],
    );
    let shown = identity_of(&l, "u-ben");
    assert_eq!((shown.last.as_str(), shown.number), ("BECKER", 31));
    let race = report_of(&l, "u-anna", &BTreeMap::from([("u-ben", ("Ben BECKER", 31))]));
    let warnings = apply(
        &mut l,
        &anna(),
        Action::Report {
            round: None,
            kind: None,
            force: false,
            report: race,
        },
    )
    .unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w == "Ben Becker's seat was driven by the game's AI"),
        "{warnings:?}"
    );
    let result = &l.season().rounds[0].race_result;
    let c = result.iter().find(|c| c.entry == "p:u-ben").expect("Ben counted");
    assert!(c.ai_driven);
    assert!(
        !warnings.iter().any(|w| w.contains("matched no driver")),
        "{warnings:?}"
    );

    // With the game showing only his last name, and with a clashing number shown as the seat's.
    let ben = actor("u-ben", "Ben Becker");
    let mut l = career(
        2016,
        &[(anna(), 42, 2, "kimi_raikkonen"), (ben, 44, 4, "jenson_button")],
    );
    assert_eq!(identity_of(&l, "u-ben").number, 22, "Hamilton's 44: Button's 22");
    let race = report_of(&l, "u-anna", &BTreeMap::from([("u-ben", ("Becker", 22))]));
    apply(
        &mut l,
        &anna(),
        Action::Report {
            round: None,
            kind: None,
            force: false,
            report: race,
        },
    )
    .unwrap();
    let c = l.season().rounds[0]
        .race_result
        .iter()
        .find(|c| c.entry == "p:u-ben")
        .expect("Ben counted")
        .clone();
    assert!(c.ai_driven);
}

#[test]
fn the_briefing_shows_the_game_identity() {
    let l = anna_and_ben();
    let game = l
        .briefing("u-anna")
        .unwrap()
        .game
        .expect("the identity option is on");
    assert_eq!(
        format!("{} #{}", game.full, game.number),
        "Anna MÜLLER #42",
        "the briefing's \"The game shows you as Anna MÜLLER #42\""
    );
    assert_eq!(game.code, "MUL");
    assert_eq!(game.nationality.as_deref(), Some("germany"));
    assert_eq!(game.replaces, "kimi_raikkonen");
    assert_eq!(game.team_key, "ferrari");
    let ben = l.briefing("u-ben").unwrap().game.unwrap();
    assert_eq!(
        (ben.full.as_str(), ben.number, ben.number_clash),
        ("BEN", 12, true)
    );
}

#[test]
fn identity_is_off_unless_the_option_is_on() {
    let ctx = Ctx::new();
    let mut l = anna_and_ben();
    l.options.game_identity = false;
    assert_eq!(gamesync::identity_of(&l, "u-anna"), None);
    assert_eq!(l.briefing("u-anna").unwrap().game, None);
    assert_eq!(gamesync::overlay(&l, "c-1", "x", &ctx.get(false)), None);
    l.options.game_performance = true;
    let o = gamesync::overlay(&l, "c-1", "x", &ctx.get(false)).unwrap();
    assert!(o.identities.is_empty());
    let files = gamesync::files(&o, &ctx.get(false));
    assert!(
        !paths(&files)
            .iter()
            .any(|p| [IDENTITY_REGISTRY, IDENTITY_DB, IDENTITY_LANG].contains(p)),
        "{:?}",
        paths(&files)
    );

    // A player who left the career is the seat's driver again; the others keep theirs.
    let mut l = anna_and_ben();
    l.players[1].left = true;
    let o = gamesync::overlay(&l, "c-1", "x", &ctx.get(true)).unwrap();
    assert_eq!(
        o.identities.iter().map(|i| i.player.as_str()).collect::<Vec<_>>(),
        ["u-anna"]
    );
}

#[test]
fn the_fallback_ops_build_into_the_game_database() {
    use crate::baseline::Identity;
    use crate::careersync;
    use crate::store::Store;
    use swofty_formats::gvdb::Gvdb;

    let _pin = crate::builtin::LoaderPin::none();
    let root = std::env::temp_dir().join(format!("f1mp-identity-build-{}", std::process::id()));
    std::fs::remove_dir_all(&root).ok();
    let store = Store::new(root.join("data"));
    let season = "swofty.classic-seasons-2016";
    for id in ["swofty.classic-seasons", season] {
        store.install_addon(&Path::new(FIXTURE).join(id)).unwrap();
    }
    store
        .update_profile("career", |p| {
            p.set_enabled("swofty.classic-seasons", true)?;
            p.set_enabled(season, true)
        })
        .unwrap();
    // The fixture's Classic Seasons has no database files: these rows stand in for the game's
    // and the ones Classic Seasons adds.
    let dir = root.join("game");
    std::fs::create_dir_all(dir.join("game_package/gamedatabases")).unwrap();
    std::fs::write(dir.join(crate::GAME_EXE), "exe").unwrap();
    let rows = identity::box_rows(2016, &["kimi_raikkonen", "felipe_nasr", "lewis_hamilton"]);
    std::fs::write(dir.join(crate::build::GVDB), rows.to_bytes()).unwrap();
    let game = crate::install::Game {
        identity: Identity::detect(&dir, Some(1)).unwrap(),
        dir,
        user_reg: None,
    };
    let ctx = careersync::context(&store, 2016).unwrap();
    let overlay = gamesync::overlay(&anna_and_ben(), "c-1", "Anna & Ben", &ctx.get()).unwrap();
    careersync::prepare_as(&store, "career", &overlay, false).unwrap();
    let built = crate::build::build(&store, "career", &game).unwrap();
    assert!(
        built.order.iter().any(|id| id == gamesync::ADDON_ID),
        "{:?}",
        built.order
    );

    let db = Gvdb::parse(&std::fs::read(built.overlay.join(crate::build::GVDB)).unwrap()).unwrap();
    let row = |i: usize, f: &str| db.get_text(&format!("entrants.drivers.driver_{i}.{f}")).unwrap();
    let got = |i: usize| {
        [
            row(i, "first_name"),
            row(i, "initials"),
            row(i, "nationality"),
            row(i, "car_number"),
            db.get_text(&format!("entrants.entrant_lists.entrants.entrant_{i}.car_number"))
                .unwrap(),
        ]
    };
    assert_eq!(
        got(0),
        ["lng_f1mp_kimi_raikkonen_first", "MUL", "germany", "42", "42"]
    );
    assert_eq!(
        got(1),
        ["lng_f1mp_felipe_nasr_first", "BEN", "finland", "12", "12"]
    );
    assert_eq!(
        row(2, "first_name"),
        "lng_lewis_hamilton_first",
        "a driver nobody replaces"
    );
    std::fs::remove_dir_all(&root).ok();
}
