use std::path::PathBuf;

use swofty_core::activation::db::{Registry, Schema, Table, TrackHost, activate, datasets};
use swofty_core::activation::dbx::{Db, field, row, sub_count, sub_table};
use swofty_core::map::TrackPackage;
use swofty_formats::gvdb::Gvdb;

const GVDB: &str = "game_package/gamedatabases/gamedatabases.2025.bin";

fn pristine_db() -> Option<(Vec<u8>, Gvdb)> {
    let dir = std::env::var_os("F1_25_DIR").map(PathBuf::from)?;
    let data = std::fs::read(dir.join(GVDB)).ok()?;
    let db = Gvdb::parse(&data).unwrap();
    Some((data, db))
}

fn package(code: &str) -> Option<TrackPackage> {
    let root = std::env::var_os("SWOFTY_MAPS_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share/swofty/maps")))?;
    let json = std::fs::read(root.join(code).join("manifest.json")).ok()?;
    Some(TrackPackage::parse(&json).unwrap())
}

fn keys(db: &Db, schema: &Schema, r: u64, out: &mut Vec<(String, u64)>, prefix: &str) {
    for f in &schema.fields {
        let k = field(r, f);
        if db.has(k) {
            out.push((format!("{prefix}{f}"), k));
        }
    }
    for (name, sub) in &schema.tables {
        let c = sub_count(r, name);
        if let Some(n) = db.count(c) {
            out.push((format!("{prefix}_{name}_instancecount_"), c));
            for i in 0..n {
                keys(
                    db,
                    sub,
                    row(sub_table(r, name), i),
                    out,
                    &format!("{prefix}{name}_{i}."),
                );
            }
        }
    }
}

fn row_by(db: &Db, t: Table, label_field: &str, label: &str) -> Option<u64> {
    let n = t.count.and_then(|c| db.count(c)).unwrap_or(0);
    (0..n)
        .map(|i| row(t.base, i))
        .find(|&r| db.text(field(r, label_field)).as_deref() == Some(label))
}

fn assert_same_shape(db: &Db, schema: &Schema, src: u64, dst: u64, what: &str) {
    let (mut a, mut b) = (Vec::new(), Vec::new());
    keys(db, schema, src, &mut a, "");
    keys(db, schema, dst, &mut b, "");
    let names = |v: &Vec<(String, u64)>| v.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>();
    assert_eq!(names(&a), names(&b), "{what}: cloned row has a different key set");
    for ((n, ka), (_, kb)) in a.iter().zip(&b) {
        assert_eq!(
            db.get(*ka).unwrap().tag,
            db.get(*kb).unwrap().tag,
            "{what}.{n}: type differs"
        );
    }
}

#[test]
fn sepang_and_okayama_activate_completely() {
    let Some((data, mut gvdb)) = pristine_db() else {
        eprintln!("F1_25_DIR is not set; skipping");
        return;
    };
    let (Some(sepang), Some(okayama)) = (package("bahrain24"), package("okayama")) else {
        eprintln!("the bahrain24/okayama packages are not imported; skipping");
        return;
    };
    let reg = Registry::builtin();
    let before = gvdb.records.len();
    let lists: std::collections::BTreeSet<String> = ["tracks.flags_circle", "fe_world.tracks_26"]
        .map(String::from)
        .into();
    let slot = "invitational_image_10";
    let group =
        format!("asset_groups/ui_package/texture_arrays/invitational_events/event_images/{slot}.assetgroup");
    let key = format!("{slot}.assetgroup");
    let hosted = TrackHost::Slot {
        asset_group: &group,
        asset_root: "asset_groups/ui_package/texture_arrays/invitational_events/event_images/",
        key: &key,
    };
    for (pkg, route) in [(&sepang, TrackHost::Natural), (&okayama, hosted)] {
        let report = activate(&mut gvdb, pkg, &reg, Some(&route), &lists).unwrap();
        for l in &report.lines {
            eprintln!("{}: {l}", pkg.track_code);
        }
        for w in &report.warnings {
            eprintln!("{}: WARNING {w}", pkg.track_code);
        }
    }
    assert!(gvdb.records.len() > before);
    let bytes = gvdb.to_bytes();
    assert_ne!(bytes, data);
    let mut back = Gvdb::parse(&bytes).unwrap();
    assert_eq!(back.to_bytes(), bytes, "edited database does not round-trip");
    let db = Db::new(&mut back);

    let tracks = Table::of(&reg.track).unwrap();
    let n_tracks = db.count(tracks.count.unwrap()).unwrap();
    assert_eq!(n_tracks, 44, "one new track row for okayama");
    let ok = row(tracks.base, 43);
    assert_eq!(db.text(field(ok, "instanceName")).as_deref(), Some("okayama"));
    assert_eq!(db.text(field(ok, "fileName")).as_deref(), Some("okayama"));
    assert_eq!(db.text(field(ok, "statId")).as_deref(), Some("43"));
    assert_eq!(db.text(field(ok, "numberOfCorners")).as_deref(), Some("13"));
    let mel = row_by(&db, tracks, "_metadata_", "melbourne").unwrap();
    assert_same_shape(&db, &reg.track.schema, mel, ok, "tracks.Track okayama");
    for key in [
        "Daylight_0.start",
        "Daylight_0.end",
        "CareerHubFirstVisit_0.timeofday",
        "CareerHubLastVisit_0.timeofday",
        "TrackHighlights_0.TrackHighlight_0.ui_title",
        "TrackVideo_0.value",
    ] {
        assert_eq!(
            db.has(field(ok, key)),
            db.has(field(mel, key)),
            "tracks.Track okayama.{key}: the startup track reader at 0x142e4c640 reads it"
        );
        assert!(db.has(field(ok, key)), "okayama.{key} is missing");
    }
    let res = Table::of(reg.track_resources.as_ref().unwrap()).unwrap();
    assert_eq!(
        db.count(res.count.unwrap()),
        Some(6),
        "two track resource rows added"
    );
    let r = row_by(&db, res, "_metadata_", "swofty_okayama").unwrap();
    assert_eq!(
        db.text(field(r, "worldAssetGroup")).as_deref(),
        Some(
            "asset_groups/ui_package/texture_arrays/invitational_events/event_images/invitational_image_10.assetgroup"
        )
    );
    assert_eq!(
        db.text(field(r, "iblKey")).as_deref(),
        Some("invitational_image_10.assetgroup")
    );
    assert_eq!(
        db.text(field(r, "worldEri")).as_deref(),
        Some("eaid://environment_package/tracks/[FILENAME]/wep/[FILENAME]_common.wep")
    );
    assert_eq!(
        db.text(field(ok, "track_resource")).as_deref(),
        Some("swofty_okayama")
    );
    assert_eq!(
        db.text(field(ok, "TrackTextures_0.TrackTexture_0.value"))
            .as_deref(),
        Some("overrides.tracks.flags_circle.okayama")
    );
    let b24 = row_by(&db, tracks, "_metadata_", "bahrain24").unwrap();
    assert_eq!(
        db.text(field(b24, "track_resource")).as_deref(),
        Some("swofty_bahrain24")
    );
    let r = row_by(&db, res, "_metadata_", "swofty_bahrain24").unwrap();
    let tracks_dir = "asset_groups/environment_package/tracks/bahrain24";
    for (name, value) in [
        (
            "worldAssetGroup",
            format!("{tracks_dir}/wep/bahrain24_common.assetgroup"),
        ),
        (
            "aoBakeAssetGroup",
            format!("{tracks_dir}/bake/bahrain24_ao.bake.assetgroup"),
        ),
        (
            "radianceBakeAssetGroup",
            format!("{tracks_dir}/bake/bahrain24_radiance.bake.assetgroup"),
        ),
        (
            "lightListAssetGroup",
            format!("{tracks_dir}/bake/bahrain24_light_list.bake.assetgroup"),
        ),
        (
            "ddgiAssetGroup",
            format!("{tracks_dir}/probes/config.ddgi.assetgroup"),
        ),
        ("trackMapAssetGroup", format!("{tracks_dir}/bahrain24.assetgroup")),
        ("iblAssetGroupsRoot", format!("{tracks_dir}/probes/")),
        ("iblKey", "key_2019.assetgroup".into()),
    ] {
        assert_eq!(db.text(field(r, name)).as_deref(), Some(value.as_str()), "{name}");
    }
    assert_eq!(db.text(field(b24, "length_metres")).as_deref(), Some("5543"));
    assert_eq!(db.text(field(b24, "allow_night_start")).as_deref(), Some("False"));

    for ds in datasets(&db, &reg).unwrap() {
        let n = db.count(ds.table.count.unwrap()).unwrap();
        let probed = (0..)
            .take_while(|&i| db.has(field(row(ds.table.base, i), "_metadata_")))
            .count();
        assert_eq!(n, probed, "{}: row count matches the rows", ds.label);
        for (code, base) in [("bahrain24", "bahrain"), ("okayama", "melbourne")] {
            let src = row_by(&db, ds.table, "track", base);
            let dst = row_by(&db, ds.table, "track", code);
            assert_eq!(src.is_some(), dst.is_some(), "{} {code}", ds.label);
            if let (Some(s), Some(d)) = (src, dst) {
                assert_same_shape(&db, &reg.track_data.schema, s, d, &format!("{} {code}", ds.label));
            }
        }
        if ds.label == "f1_2026" {
            let d = row_by(&db, ds.table, "track", "okayama").unwrap();
            assert_eq!(
                db.text(field(d, "TyreData_0.compound_softer")).as_deref(),
                Some("C5")
            );
            assert_eq!(
                db.text(field(d, "TrackTextures_0.tracks_bg")).as_deref(),
                Some("overrides.fe_world.tracks_26.okayama")
            );
            assert_eq!(
                db.text(field(d, "TrackTextures_0.broadcast_ui_track")).as_deref(),
                Some("overrides.broadcast_ui_generic.track.melbourne")
            );
            let s = row_by(&db, ds.table, "track", "bahrain24").unwrap();
            assert_eq!(
                db.text(field(s, "TyreData_0.compound_harder")).as_deref(),
                Some("C2")
            );
            assert_eq!(
                db.text(field(s, "Session Start Times_0.race_start_time"))
                    .as_deref(),
                Some("14,00")
            );
        }
        if ds.label == "f1" {
            let d = row_by(&db, ds.table, "track", "okayama").unwrap();
            assert_eq!(
                db.text(field(d, "TrackNameData_0.circuit")).as_deref(),
                Some("lng_track_jpn_gp_title")
            );
            assert_eq!(
                db.text(field(d, "TrackNameData_0.file_name")).as_deref(),
                Some("okayama")
            );
        }
    }

    let mut pristine = Gvdb::parse(&data).unwrap();
    let old = Db::new(&mut pristine);
    for spec in &reg.per_track {
        let t = Table::of(spec).unwrap();
        let label = spec.labels.first().map(String::as_str).unwrap_or("_metadata_");
        let before = old.count(t.count.unwrap()).unwrap();
        let after = db.count(t.count.unwrap()).unwrap();
        let src = row_by(&db, t, label, "melbourne");
        let dst = row_by(&db, t, label, "okayama");
        assert_eq!(src.is_some(), dst.is_some(), "{}: okayama row", spec.id);
        if let (Some(s), Some(d)) = (src, dst) {
            assert_same_shape(&db, &spec.schema, s, d, &spec.id);
        }
        let b24 =
            row_by(&old, t, label, "bahrain24").is_none() && row_by(&old, t, label, "bahrain").is_some();
        assert_eq!(
            after,
            before + usize::from(dst.is_some()) + usize::from(b24),
            "{}: row count",
            spec.id
        );
        assert!(
            db.has(field(row(t.base, after - 1), "_metadata_")),
            "{}: last row",
            spec.id
        );
        assert!(
            !db.has(field(row(t.base, after), "_metadata_")),
            "{}: no row past the count",
            spec.id
        );
    }
    let canned = reg.per_track.iter().find(|s| s.id == "canned_dnfs").unwrap();
    assert!(
        row_by(&db, Table::of(canned).unwrap(), "_metadata_", "okayama").is_some(),
        "the per-race DNF lookup needs an okayama row"
    );
}

#[test]
fn map_worlds_get_room_for_the_grid() {
    use swofty_core::activation::grid::Room;
    use swofty_core::activation::place::extend_grid;
    use swofty_formats::erp::ErpFile;
    use swofty_formats::pkg::Pkg;

    let root = std::env::var_os("SWOFTY_MAPS_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share/swofty/maps")));
    for code in ["bahrain24", "okayama"] {
        let Some(pkg) = package(code) else {
            eprintln!("{code} is not imported; skipping");
            continue;
        };
        let file = ErpFile::open(&root.as_ref().unwrap().join(code).join(&pkg.erp)).unwrap();
        assert_eq!(extend_grid(&file, code, 22).unwrap().map(|e| e.2), None, "{code}");
        let (i, res, ext) = extend_grid(&file, code, 28).unwrap().unwrap();
        eprintln!("{code}: {} {:?} -> {:?}", res.name, ext.before, ext.after);
        assert_eq!(res.name, file.erp.resources[i].name);
        assert_eq!(
            ext.after,
            Room {
                grid_slots: 28,
                garages: 14,
                pit_boxes: 14
            },
            "{code}"
        );
        let before = Pkg::parse(
            &file.load(&file.erp.resources[i]).unwrap().fragments[0]
                .data()
                .unwrap(),
        )
        .unwrap();
        let after = Pkg::parse(&res.fragments[0].data().unwrap()).unwrap();
        let count = |p: &Pkg| {
            p.root
                .as_obj()
                .unwrap()
                .get("instances")
                .unwrap()
                .as_arr()
                .unwrap()
                .items
                .len()
        };
        let added = (ext.after.grid_slots - ext.before.grid_slots) * 2
            + (ext.after.garages - ext.before.garages)
            + (ext.after.pit_boxes - ext.before.pit_boxes);
        assert_eq!(count(&after), count(&before) + added as usize, "{code}");
        assert_eq!(res.hash, res.content_hash().unwrap());
    }
}

#[test]
fn stock_ai_lines_need_no_passing_space_fix() {
    use swofty_core::activation::ailine::{Spacing, give_routes_space};
    use swofty_formats::erp::ErpFile;

    let Some(dir) = std::env::var_os("F1_25_DIR").map(PathBuf::from) else {
        eprintln!("F1_25_DIR is not set; skipping");
        return;
    };
    let tracks = dir.join("2025_asset_groups/environment_package/tracks");
    let Ok(entries) = std::fs::read_dir(&tracks) else {
        eprintln!("{} is missing; skipping", tracks.display());
        return;
    };
    let mut lines = 0;
    for entry in entries {
        let code = entry.unwrap().file_name().to_string_lossy().into_owned();
        let erp = tracks.join(&code).join(format!("wep/{code}_common.erp"));
        let Ok(file) = ErpFile::open(&erp) else {
            continue;
        };
        for r in file.erp.resources.iter().filter(|r| r.kind() == "AISplineData") {
            let data = file.load(r).unwrap().fragments[0].data().unwrap();
            assert_eq!(give_routes_space(&data), Spacing::Fine, "{}", r.name);
            lines += 1;
        }
    }
    eprintln!("{lines} stock AI racing line(s) checked");
}
