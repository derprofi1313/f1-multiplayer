//! The league → game overlay (W1-C): canonical, byte-identical files on every PC, named and
//! versioned by round, with the submodules' files reaching the manifest and the hash.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::gamesync::{self, ADDON_ID, GENERATOR, LEAGUE_FILE, MANIFEST_FILE, Overlay, SyncContext};
use super::*;
use crate::manifest::{AddonManifest, Multiplayer};

macro_rules! fixture {
    ($rel:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/classic-seasons-mini/",
            $rel
        ))
    };
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
        let read = |text: &str| serde_json::from_str::<Value>(text).unwrap();
        Self {
            content: read(fixture!("swofty.classic-seasons-2016/registry/content.json")),
            cs_drivers: read(fixture!("swofty.classic-seasons/registry/drivers.json")),
            liveries: read(fixture!("swofty.classic-seasons-2016/assets/liveries.json")),
            ai_slots: read(fixture!("swofty.classic-seasons/data/ai_slots.json")),
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

/// Anna at Ferrari and Ben at McLaren from `first_year`, with the game identity on.
fn career(first_year: u16) -> League {
    let setup = Setup {
        first_year,
        ..Setup::default()
    };
    let mut l = League::start(setup, &anna(), Some(7), 42).unwrap();
    apply(&mut l, &ben(), Action::Enter { number: Some(8) }).unwrap();
    apply(
        &mut l,
        &anna(),
        Action::Seat {
            slot: 2,
            replaces: None,
        },
    )
    .unwrap();
    apply(
        &mut l,
        &ben(),
        Action::Seat {
            slot: 8,
            replaces: None,
        },
    )
    .unwrap();
    apply(&mut l, &anna(), Action::Begin).unwrap();
    l.options.game_identity = true;
    l
}

fn overlay(l: &League, ctx: &Ctx) -> Overlay {
    gamesync::overlay(l, "c-1", "Anna & Ben", &ctx.get()).expect("an overlay while racing")
}

fn files(l: &League, ctx: &Ctx) -> Vec<(String, Vec<u8>)> {
    gamesync::files(&overlay(l, ctx), &ctx.get())
}

fn file<'a>(files: &'a [(String, Vec<u8>)], path: &str) -> &'a [u8] {
    &files
        .iter()
        .find(|(p, _)| p == path)
        .unwrap_or_else(|| panic!("no {path}"))
        .1
}

fn manifest(files: &[(String, Vec<u8>)]) -> AddonManifest {
    AddonManifest::parse(std::str::from_utf8(file(files, MANIFEST_FILE)).unwrap()).unwrap()
}

fn json_of(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).unwrap()
}

#[test]
fn files_are_byte_identical_for_the_same_league() {
    let ctx = Ctx::new();
    let anna = career(2016);
    // Ben's PC has the document as the host server sent it, and reads its own copy of Classic
    // Seasons.
    let ben: League = serde_json::from_str(&serde_json::to_string(&anna).unwrap()).unwrap();
    let mine = files(&anna, &ctx);
    assert_eq!(mine, files(&ben, &Ctx::new()));
    assert_eq!(mine, files(&anna, &ctx), "the same twice");

    let paths: Vec<&str> = mine.iter().map(|(p, _)| p.as_str()).collect();
    let mut sorted = paths.clone();
    sorted.sort();
    assert_eq!(paths, sorted, "sorted by path");
    assert!(paths.contains(&MANIFEST_FILE) && paths.contains(&LEAGUE_FILE));
    for (path, bytes) in &mine {
        let text = std::str::from_utf8(bytes).unwrap();
        assert!(
            !text.contains('\r') && text.ends_with("}\n"),
            "{path}: LF endings"
        );
        assert_eq!(
            bytes,
            &gamesync::canonical_json(&json_of(bytes)),
            "{path} is canonical"
        );
    }
    // Nothing in them depends on the PC or the time.
    let text: String = mine
        .iter()
        .map(|(_, b)| String::from_utf8(b.clone()).unwrap())
        .collect();
    assert!(!text.contains("\\\\") && !text.contains(":\\"), "no local paths");
}

#[test]
fn the_addon_is_named_and_versioned_by_round() {
    let ctx = Ctx::new();
    let mut l = career(2016);
    let first = files(&l, &ctx);
    let m = manifest(&first);
    let league = json_of(file(&first, LEAGUE_FILE));
    let hash = league["hash"].as_str().unwrap().to_owned();
    assert_eq!(hash.len(), 8);
    assert!(
        hash.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    );
    assert_eq!(
        league,
        json!({ "career": "c-1", "year": 2016, "round": 1, "hash": hash, "generator": GENERATOR })
    );
    assert_eq!(m.id, ADDON_ID);
    assert_eq!(m.name, "League career sync");
    assert_eq!(m.version.to_string(), format!("0.2016.1+{hash}"));
    assert_eq!(
        m.description.as_deref(),
        Some("Generated by F1 Multiplayer for career Anna & Ben; don't edit")
    );
    let requires: Vec<&str> = m.requires.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(requires, ["swofty.classic-seasons-2016"]);
    assert_eq!(m.load_after, ["swofty.classic-seasons*"]);
    assert_eq!(m.multiplayer(), Multiplayer::Gameplay, "a lobby checks it");
    assert!(m.operations.database.is_empty() && m.operations.assets.is_empty());

    apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    // Ben agrees: in a shared career simulating waits for everyone (W2-1).
    apply(&mut l, &ben(), Action::Vote { yes: true, force: false }).unwrap();
    let second = files(&l, &ctx);
    let next = json_of(file(&second, LEAGUE_FILE));
    assert_eq!(next["round"], 2);
    assert_ne!(next["hash"], league["hash"]);
    assert_eq!(
        manifest(&second).version.to_string(),
        format!("0.2016.2+{}", next["hash"].as_str().unwrap())
    );
    // Another career at the same round is another overlay.
    let other = gamesync::files(
        &gamesync::overlay(&l, "c-2", "Anna & Ben", &ctx.get()).unwrap(),
        &ctx.get(),
    );
    assert_ne!(json_of(file(&other, LEAGUE_FILE))["hash"], next["hash"]);
}

#[test]
fn the_game_s_own_seasons_need_no_classic_seasons_addon() {
    assert_eq!(
        gamesync::season_addon(2016).as_deref(),
        Some("swofty.classic-seasons-2016")
    );
    assert_eq!(
        gamesync::season_addon(2024).as_deref(),
        Some("swofty.classic-seasons-2024")
    );
    assert_eq!(gamesync::season_addon(2025), None);
    assert_eq!(gamesync::season_addon(2026), None);
    let l = career(2025);
    let none = Value::Null;
    let ctx = SyncContext {
        content: &none,
        cs_drivers: &none,
        liveries: &none,
        ai_slots: &none,
        loader_applies_rows: true,
    };
    let files = gamesync::files(&gamesync::overlay(&l, "c-1", "x", &ctx).unwrap(), &ctx);
    let m = manifest(&files);
    assert!(m.requires.is_empty());
    assert!(m.version.to_string().starts_with("0.2025.1+"));
}

#[test]
fn no_overlay_outside_racing_or_with_every_game_option_off() {
    let ctx = Ctx::new();
    let mut l = career(2016);
    l.options.game_identity = false;
    assert_eq!(gamesync::overlay(&l, "c-1", "x", &ctx.get()), None);
    l.options.game_parts = true;
    assert_eq!(
        gamesync::overlay(&l, "c-1", "x", &ctx.get()),
        None,
        "parts only refine performance"
    );
    for on in [
        |o: &mut Options| o.game_identity = true,
        |o: &mut Options| o.game_performance = true,
        |o: &mut Options| o.lineups = LineupMode::Dynamic,
    ] {
        let mut l = l.clone();
        on(&mut l.options);
        assert!(gamesync::overlay(&l, "c-1", "x", &ctx.get()).is_some());
    }

    let mut setup = League::start(Setup::default(), &anna(), Some(7), 42).unwrap();
    setup.options.game_identity = true;
    assert_eq!(setup.phase, Phase::Setup);
    assert_eq!(gamesync::overlay(&setup, "c-1", "x", &ctx.get()), None);
    assert_eq!(gamesync::identity_of(&setup, "u-anna"), None);
}

#[test]
fn an_option_that_is_off_writes_nothing() {
    let ctx = Ctx::new();
    let mut l = career(2016);
    let o = overlay(&l, &ctx);
    assert!(
        o.cars.is_empty() && o.renames.is_empty(),
        "only the identity is on"
    );
    l.options.game_identity = false;
    l.options.game_performance = true;
    let o = overlay(&l, &ctx);
    assert!(
        o.identities.is_empty() && o.renames.is_empty(),
        "only the performance is on"
    );
    l.options.game_performance = false;
    l.options.lineups = LineupMode::Dynamic;
    let o = overlay(&l, &ctx);
    assert!(
        o.identities.is_empty() && o.cars.is_empty(),
        "only the line-up is on"
    );

    // An overlay with no parts is the manifest and league.json alone.
    let mut off = o.clone();
    off.identities.clear();
    off.cars.clear();
    off.renames.clear();
    let paths: Vec<String> = gamesync::files(&off, &ctx.get())
        .into_iter()
        .map(|(p, _)| p)
        .collect();
    assert_eq!(paths, [LEAGUE_FILE, MANIFEST_FILE]);
}

#[test]
fn the_submodules_files_reach_the_manifest_and_the_hash() {
    let ctx = Ctx::new();
    let o = overlay(&career(2016), &ctx);
    let part = |path: &str, v: Value| gamesync::json_file(path, &v);
    let identity = vec![
        part(gamesync::IDENTITY_REGISTRY, json!({ "driver.identity": {} })),
        part(gamesync::IDENTITY_DB, json!({ "schema_version": 1, "ops": [] })),
        part(gamesync::IDENTITY_LANG, json!({ "schema_version": 1, "ops": [] })),
    ];
    let performance = vec![part(
        gamesync::HANDLING_REGISTRY,
        json!({ "vehicle.handling": { "a": 812.0 } }),
    )];
    let files = gamesync::assemble(&o, [identity.clone(), performance].concat());
    let paths: Vec<&str> = files.iter().map(|(p, _)| p.as_str()).collect();
    assert_eq!(
        paths,
        [
            gamesync::IDENTITY_DB,
            gamesync::IDENTITY_LANG,
            LEAGUE_FILE,
            gamesync::HANDLING_REGISTRY,
            gamesync::IDENTITY_REGISTRY,
            MANIFEST_FILE
        ]
    );
    assert!(
        std::str::from_utf8(file(&files, gamesync::HANDLING_REGISTRY))
            .unwrap()
            .contains("812.0")
    );
    let m = manifest(&files);
    assert_eq!(m.operations.database, [gamesync::IDENTITY_DB]);
    assert_eq!(m.operations.assets, [gamesync::IDENTITY_LANG]);
    assert_eq!(
        m.kind,
        [crate::manifest::Kind::Database, crate::manifest::Kind::Files]
    );

    // Each body file is in the hash; the manifest and league.json are not.
    let hash = |files: &[gamesync::File]| json_of(file(files, LEAGUE_FILE))["hash"].clone();
    let alone = gamesync::assemble(&o, identity.clone());
    assert_ne!(hash(&alone), hash(&files));
    let mut changed = identity.clone();
    changed[0] = part(
        gamesync::IDENTITY_REGISTRY,
        json!({ "driver.identity": { "x": 1 } }),
    );
    assert_ne!(hash(&gamesync::assemble(&o, changed)), hash(&alone));
    let body: BTreeMap<String, Vec<u8>> = alone
        .iter()
        .filter(|(p, _)| p != MANIFEST_FILE && p != LEAGUE_FILE)
        .cloned()
        .collect();
    assert_eq!(hash(&alone), json!(gamesync::hash8(&o, &body)));

    // A path outside registry/, db/ and lang/, or written twice, is left out.
    let stray = vec![
        part("swofty.json", json!({})),
        part("league.json", json!({})),
        part("../escape.json", json!({})),
        part("registry/../x.json", json!({})),
        part("registry\\x.json", json!({})),
        part("registry/.hidden.json", json!({})),
        part("registry/notes.txt", json!({})),
        part("native/x.json", json!({})),
        part(gamesync::IDENTITY_REGISTRY, json!({ "second": true })),
    ];
    let kept = gamesync::assemble(&o, [identity, stray].concat());
    assert_eq!(hash(&kept), hash(&alone));
    assert_eq!(kept, alone);
}

#[test]
fn canonical_json_sorts_keys_and_writes_one_decimal() {
    let v = json!({ "b": [1, 2.75, -3, 812.0, { "z": null, "a": true }], "a": "é\"\n", "c": {}, "d": [] });
    let text = String::from_utf8(gamesync::canonical_json(&v)).unwrap();
    assert_eq!(
        text,
        "{\n  \"a\": \"é\\\"\\n\",\n  \"b\": [\n    1,\n    2.8,\n    -3,\n    812.0,\n    {\n      \"a\": true,\n      \"z\": null\n    }\n  ],\n  \"c\": {},\n  \"d\": []\n}\n"
    );
    // f32 values from the league keep one decimal, whatever their f64 tail.
    let mass = json!(812.4_f32);
    assert_eq!(gamesync::canonical_json(&mass), b"812.4\n");
}

#[test]
fn identity_of_needs_racing_and_the_option() {
    let mut l = career(2016);
    // The wave-2 identity is still empty; the gate is what this checks.
    assert_eq!(gamesync::identity_of(&l, "u-nobody"), None);
    l.options.game_identity = false;
    assert_eq!(gamesync::identity_of(&l, "u-anna"), None);
}
