use std::path::PathBuf;

use swofty_core::activation::db::{Registry, TrackHost, activate};
use swofty_core::baseline::Identity;
use swofty_core::install::Game;
use swofty_core::map::TrackPackage;
use swofty_core::tracks::{PlanTrack, for_profile, list};
use swofty_formats::gvdb::Gvdb;

const GVDB: &str = "game_package/gamedatabases/gamedatabases.2025.bin";

fn game_dir() -> Option<PathBuf> {
    std::env::var_os("F1_25_DIR")
        .map(PathBuf::from)
        .filter(|d| d.join(GVDB).is_file())
}

fn package(code: &str, from: Option<&str>, slot: Option<&str>, name: &str) -> TrackPackage {
    let json = serde_json::json!({
        "track_code": code,
        "track_name": name,
        "erp": format!("{code}.erp"),
        "activation": { "copy_from_track": from, "copy_slot": slot },
    });
    TrackPackage::parse(&serde_json::to_vec(&json).unwrap()).unwrap()
}

fn activated(dir: &std::path::Path) -> (Gvdb, Vec<PlanTrack>) {
    let mut db = Gvdb::parse(&std::fs::read(dir.join(GVDB)).unwrap()).unwrap();
    let reg = Registry::builtin();
    let lists = std::collections::BTreeSet::new();
    let pkgs = [
        package("bahrain24", Some("bahrain"), None, "Sepang"),
        package("testring", Some("melbourne"), Some("melbourne"), "Test Ring"),
    ];
    let mut plan = Vec::new();
    for pkg in &pkgs {
        activate(&mut db, pkg, &reg, Some(&TrackHost::Natural), &lists).unwrap();
        plan.push(PlanTrack {
            code: pkg.track_code.clone(),
            name: pkg.track_name.clone(),
            addon: Some("teamsimplified.mods".into()),
        });
    }
    (db, plan)
}

#[test]
fn stock_events_and_installed_tracks_resolve_to_track_ids() {
    let Some(dir) = game_dir() else {
        eprintln!("F1_25_DIR is not set; skipping");
        return;
    };
    let stock = Gvdb::parse(&std::fs::read(dir.join(GVDB)).unwrap()).unwrap();
    let before = list(&stock, &[]);
    assert_eq!(before.len(), 43);
    assert!(before.iter().all(|t| !t.modded));
    let by =
        |v: &[swofty_core::launcher::ProfileTrack], c: &str| v.iter().find(|t| t.code == c).cloned().unwrap();
    let suzuka = by(&before, "suzuka");
    assert_eq!(suzuka.stat_id, 13);
    assert_eq!(suzuka.events, ["rw_suzuka", "rw_suzuka_26"]);
    assert_eq!(by(&before, "monaco").events, ["rw_monaco", "rw_monaco_26"]);
    assert_eq!(by(&before, "monaco").stat_id, 5);
    assert_eq!(
        by(&before, "bahrain").events,
        ["rw_bahrain", "rw_bahrain_beta", "rw_bahrain_26"]
    );
    assert!(by(&before, "bahrain24").events.is_empty());
    let playable: Vec<&str> = before
        .iter()
        .filter(|t| t.playable)
        .map(|t| t.code.as_str())
        .collect();
    assert_eq!(playable.len(), 28, "{playable:?}");
    assert!(!by(&before, "portimao").playable && !by(&before, "bahrain24").playable);
    let events: usize = before.iter().map(|t| t.events.len()).sum();
    assert_eq!(events, 63, "every fixed event names a track");

    let (db, plan) = activated(&dir);
    let after = list(&db, &plan);
    assert_eq!(after.len(), 44);
    let sepang = by(&after, "bahrain24");
    assert_eq!((sepang.stat_id, sepang.modded, sepang.playable), (35, true, true));
    assert_eq!(sepang.name.as_deref(), Some("Sepang"));
    assert!(sepang.events.is_empty(), "activation adds no fixed event");
    let ring = by(&after, "testring");
    assert_eq!((ring.stat_id, ring.modded, ring.playable), (43, true, true));
    assert_eq!(ring.name.as_deref(), Some("Test Ring"));
    assert!(ring.events.is_empty());
    assert_eq!(by(&after, "melbourne").stat_id, 0);
    assert!(!by(&after, "melbourne").modded);
    assert_eq!(after.iter().filter(|t| t.modded).count(), 2);
    let unnamed = list(&db, &[]);
    assert!(
        by(&unnamed, "testring").modded,
        "the cloned resource row marks a track without a plan"
    );
}

#[test]
fn the_profiles_latest_build_supplies_the_tracks() {
    let Some(dir) = game_dir() else {
        eprintln!("F1_25_DIR is not set; skipping");
        return;
    };
    let data = std::env::temp_dir().join(format!("swofty-profile-tracks-{}", std::process::id()));
    std::fs::remove_dir_all(&data).ok();
    let game = Game {
        dir: dir.clone(),
        user_reg: None,
        identity: Identity {
            build_id: "test".into(),
            exe_sha256: "0".repeat(64),
        },
    };
    let none = for_profile(&data, &game, "default").unwrap();
    assert!(!none.built);
    assert_eq!(none.tracks.len(), 43);

    let (db, plan) = activated(&dir);
    let overlay = data.join("cache/overlay/default/abc123");
    std::fs::create_dir_all(overlay.join("game_package/gamedatabases")).unwrap();
    std::fs::write(overlay.join(GVDB), db.to_bytes()).unwrap();
    std::fs::write(
        overlay.join("plan.json"),
        serde_json::to_vec(
            &serde_json::json!({ "profile": "default", "buildhash": "abc123", "tracks": plan }),
        )
        .unwrap(),
    )
    .unwrap();
    let built = for_profile(&data, &game, "default").unwrap();
    assert!(built.built);
    assert_eq!(built.buildhash.as_deref(), Some("abc123"));
    let ring = built.tracks.iter().find(|t| t.code == "testring").unwrap();
    assert_eq!((ring.stat_id, ring.name.as_deref()), (43, Some("Test Ring")));
    let other = for_profile(&data, &game, "other").unwrap();
    assert!(!other.built && other.tracks.len() == 43);
    std::fs::remove_dir_all(&data).ok();
}

#[test]
fn copy_slot_alone_activates_like_the_full_pair() {
    let Some(dir) = game_dir() else {
        eprintln!("F1_25_DIR is not set; skipping");
        return;
    };
    let stock = std::fs::read(dir.join(GVDB)).unwrap();
    let reg = Registry::builtin();
    let lists = std::collections::BTreeSet::new();
    let build = |pkg: &TrackPackage| {
        let mut db = Gvdb::parse(&stock).unwrap();
        let report = activate(&mut db, pkg, &reg, Some(&TrackHost::Natural), &lists).unwrap();
        (db, report.lines)
    };
    let (pair, pair_lines) = build(&package(
        "testring",
        Some("melbourne"),
        Some("melbourne"),
        "Test Ring",
    ));
    let (slot, slot_lines) = build(&package("testring", None, Some("melbourne"), "Test Ring"));
    assert_eq!(slot_lines, pair_lines);
    assert!(
        slot_lines
            .iter()
            .any(|l| l.starts_with("track data: testring cloned from melbourne in ")),
        "{slot_lines:?}"
    );
    assert!(
        slot.to_bytes() == pair.to_bytes(),
        "copy_slot alone builds the same database"
    );
    let plan = [PlanTrack {
        code: "testring".into(),
        name: Some("Test Ring".into()),
        addon: Some("teamsimplified.mods".into()),
    }];
    let ring = list(&slot, &plan)
        .into_iter()
        .find(|t| t.code == "testring")
        .unwrap();
    assert!(ring.modded && ring.playable);
}
