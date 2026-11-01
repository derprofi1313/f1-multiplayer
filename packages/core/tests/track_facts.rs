use std::path::PathBuf;

use swofty_core::activation::facts::{Scan, TRACKS_DIR, WorldsCache, facts};

const GVDB: &str = "game_package/gamedatabases/gamedatabases.2025.bin";

#[test]
fn the_stock_tracks_worlds_and_ai_data_scan_from_the_game_folder() {
    let Some(game) = std::env::var_os("F1_25_DIR").map(PathBuf::from) else {
        eprintln!("F1_25_DIR is not set; skipping");
        return;
    };
    let file = |rel: &str| game.join(rel);
    let scan = Scan {
        profile: "real",
        db: game.join(GVDB),
        game: &game,
        file: &file,
        placed: Vec::new(),
        added: Vec::new(),
        edits: None,
    };
    let started = std::time::Instant::now();
    let mut cache = WorldsCache::default();
    let f = facts(&scan, None, &mut cache).unwrap().unwrap();
    eprintln!(
        "{} track(s), {} layer(s), grid size {} in {:?}",
        f.tracks.len(),
        f.layers.len(),
        f.grid_size,
        started.elapsed()
    );
    for (code, t) in &f.tracks {
        let worlds: Vec<String> = t
            .worlds
            .iter()
            .map(|(w, r)| format!("{w} {}/{}/{}", r.grid, r.garages, r.pit_boxes))
            .collect();
        eprintln!(
            "{code}: {} | {} AI folder(s) | {} difficulty package(s)",
            worlds.join(", "),
            t.ai.len(),
            t.ai_difficulties.len()
        );
    }
    assert!(f.grid_size >= 20, "{}", f.grid_size);
    let world = |l: &str| f.layers.iter().find(|x| x.label == l).map(|x| x.world.as_str());
    assert_eq!(world("f1"), Some("f1"));
    assert_eq!(world("f1_26"), Some("f1_26"));
    let m = &f.tracks["melbourne"];
    let f1_26 = &m.worlds["f1_26"];
    assert!(
        f1_26.grid >= 22 && f1_26.garages >= 10 && f1_26.pit_boxes >= 10,
        "{f1_26:?}"
    );
    assert!(
        m.ai.iter().any(|a| a == "f1_2025_vehicle_package/ferrari"),
        "{:?}",
        m.ai
    );
    assert!(
        m.ai_difficulties.iter().any(|p| p == "f1_2025_vehicle_package"),
        "{:?}",
        m.ai_difficulties
    );
    let stock = std::fs::read_dir(game.join(TRACKS_DIR))
        .unwrap()
        .flatten()
        .filter(|e| {
            let code = e.file_name().to_string_lossy().into_owned();
            e.path().join(format!("wep/{code}_common.erp")).is_file()
        })
        .count();
    assert!(f.tracks.len() >= stock.min(20), "{} of {stock}", f.tracks.len());
    let started = std::time::Instant::now();
    let again = facts(&scan, None, &mut cache).unwrap().unwrap();
    eprintln!("from the cache in {:?}", started.elapsed());
    assert_eq!(again, f);
}

#[test]
fn an_installed_tracks_placed_archive_scans_with_its_stand_ins() {
    use swofty_core::activation::facts::Added;
    let Some(game) = std::env::var_os("F1_25_DIR").map(PathBuf::from) else {
        eprintln!("F1_25_DIR is not set; skipping");
        return;
    };
    let Some(data) = std::env::var_os("F1MP_DATA_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share/swofty")))
    else {
        return;
    };
    let shipped = data.join("addons/mod.okayama/track/okayama.erp");
    let placed = std::fs::read_dir(data.join("cache/maps/okayama"))
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| {
            e.path()
                .join("2025_asset_groups/environment_package/tracks/okayama/okayama.erp")
        })
        .find(|p| p.is_file());
    let (true, Some(placed)) = (shipped.is_file(), placed) else {
        eprintln!("mod.okayama is not installed and built; skipping");
        return;
    };
    let file = |rel: &str| game.join(rel);
    let scan = Scan {
        profile: "real",
        db: game.join(GVDB),
        game: &game,
        file: &file,
        placed: Vec::new(),
        added: vec![Added {
            code: "okayama".into(),
            shipped,
            placed,
            base: Some("melbourne".into()),
            ui_lists: Vec::new(),
        }],
        edits: None,
    };
    let f = facts(&scan, None, &mut WorldsCache::default()).unwrap().unwrap();
    let t = &f.tracks["okayama"];
    for (w, r) in &t.worlds {
        eprintln!(
            "okayama {w}: {}/{}/{} stand-in {:?}",
            r.grid, r.garages, r.pit_boxes, r.stand_in
        );
    }
    eprintln!("okayama: {} AI folder(s): {:?}", t.ai.len(), t.ai);
    assert!(!t.worlds.is_empty() && t.time_trial);
    assert!(
        t.worlds.values().all(|r| r.grid >= 20 && r.garages >= 10),
        "{:?}",
        t.worlds
    );
    assert!(
        t.ai.iter().any(|a| a.starts_with("f1_2025_vehicle_package/")),
        "{:?}",
        t.ai
    );
    assert!(!t.ai_difficulties.is_empty());
}
