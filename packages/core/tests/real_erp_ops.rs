use std::path::PathBuf;

use swofty_core::asset::{AssetOp, FileKind, OpFile, Source};
use swofty_core::erpbuild::ErpBuild;
use swofty_formats::erp::{Erp, ErpFile, Resource};

const ARCHIVE: &str = "2025_asset_groups/f1_safetycar_package/safety_car_2/wep/safety_car_2.erp";

fn game() -> Option<PathBuf> {
    match std::env::var_os("F1_25_DIR") {
        Some(d) => Some(PathBuf::from(d)),
        None => {
            eprintln!("F1_25_DIR is not set; skipping");
            None
        }
    }
}

#[test]
fn replaces_adds_and_removes_textures_in_a_game_archive() {
    let Some(game) = game() else {
        return;
    };
    let stock = ErpFile::open(&game.join(ARCHIVE)).unwrap();
    let textures: Vec<_> = stock
        .erp
        .resources
        .iter()
        .filter(|r| r.kind() == "GfxSurfaceRes" && r.fragments.len() == 3)
        .take(2)
        .collect();
    let [kept, gone] = textures.as_slice() else {
        panic!("{ARCHIVE} has fewer than two textures with mip paths");
    };
    let dir = std::env::temp_dir().join(format!("swofty-real-erp-ops-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).unwrap();
    let loaded = stock.load(kept).unwrap();
    let mut sources = Vec::new();
    for (i, f) in loaded.fragments.iter().enumerate() {
        std::fs::write(dir.join(format!("{i}.bin")), f.data().unwrap()).unwrap();
        sources.push(format!("\"{i}.bin\""));
    }
    let added = format!("{}_copy.image", kept.name.trim_end_matches(".image"));
    let file = OpFile::from_value(
        serde_json::from_str(&format!(
            r#"{{"schema_version":1,"ops":[
            {{"op":"replace-resource","erp":"{ARCHIVE}","resource":"{}","fragment":1,"source":"1.bin"}},
            {{"op":"add-resource","erp":"{ARCHIVE}","resource":"{added}","type":"GfxSurfaceRes","sources":[{}]}},
            {{"op":"remove-resource","erp":"{ARCHIVE}","resource":"{}"}}]}}"#,
            kept.name,
            sources.join(","),
            gone.name
        ))
        .unwrap(),
        FileKind::Erp,
    )
    .unwrap();
    let original = |rel: &str| game.join(rel);
    let mut b = ErpBuild::new(&original);
    for op in &file.ops {
        let AssetOp::Erp(op) = op else { continue };
        b.apply(op, &|s: &Source| Ok(s.resolve(&dir)?)).unwrap();
    }
    let out = dir.join("out");
    assert_eq!(b.write(&out).unwrap(), [ARCHIVE]);
    let written = Erp::parse(&std::fs::read(out.join(ARCHIVE)).unwrap()).unwrap();
    assert_eq!(written.resources.len(), stock.erp.resources.len());
    assert!(written.find(&gone.name).is_err());
    let replaced = written.find(&kept.name).unwrap();
    assert_eq!(replaced.hash, kept.hash);
    let copy = written.find(&added).unwrap();
    assert_eq!(
        (copy.kind, &copy.unk, copy.hash),
        (kept.kind, &kept.unk, kept.hash)
    );
    for (f, s) in copy.fragments.iter().zip(&loaded.fragments) {
        assert_eq!(
            (f.name, f.flags, f.compression, f.size),
            (s.name, s.flags, s.compression, s.size)
        );
        assert_eq!(f.data().unwrap(), s.data().unwrap());
    }
    let raw = |r: &Resource| r.fragments.iter().map(|f| f.raw.clone()).collect::<Vec<_>>();
    let mut copied = 0;
    for r in stock
        .erp
        .resources
        .iter()
        .filter(|r| r.name != gone.name && r.name != kept.name)
    {
        let w = written.find(&r.name).unwrap();
        assert_eq!(w.hash, r.hash);
        assert_eq!(raw(w), raw(&stock.load(r).unwrap()));
        copied += 1;
    }
    assert_eq!(copied, stock.erp.resources.len() - 2);
    std::fs::remove_dir_all(&dir).ok();
}

const SCENES: &str = "2025_asset_groups/ui_package/flow_team_hq.erp";
const SCENE: &str = "eaid://ui_package/art/screens/career/rnd_overview.idf";

fn grown(scene: &[u8]) -> Vec<u8> {
    use swofty_formats::pkg::{Node, Pkg, Val};
    let mut pkg = Pkg::parse(scene).unwrap();
    let root = pkg.root.as_obj_mut().unwrap();
    let entry = root.get_mut("scenelist").unwrap().as_arr_mut().unwrap().items[0]
        .as_obj_mut()
        .unwrap();
    let children = entry
        .get_mut("scene")
        .unwrap()
        .as_obj_mut()
        .unwrap()
        .get_mut("children")
        .unwrap()
        .as_arr_mut()
        .unwrap();
    let mut copy = children.items.last().unwrap().clone();
    copy.as_obj_mut()
        .unwrap()
        .set("name", Node::Val(Val::str("TUT_MASK_3")));
    children.items.push(copy);
    pkg.to_bytes().unwrap()
}

#[test]
fn round_trips_and_resizes_a_scene_in_a_game_archive() {
    let Some(game) = game() else {
        return;
    };
    let stock = ErpFile::open(&game.join(SCENES)).unwrap();
    let entry = stock.erp.find(SCENE).unwrap();
    assert_eq!(entry.kind(), "UILayout");
    let scene = stock.load(entry).unwrap().fragments[0].data().unwrap();
    let dir = std::env::temp_dir().join(format!("swofty-real-scene-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("same.idf"), &scene).unwrap();
    let bigger = grown(&scene);
    assert!(bigger.len() > scene.len());
    std::fs::write(dir.join("five.idf"), &bigger).unwrap();
    let original = |rel: &str| game.join(rel);
    let build = |src: &str| {
        let file = OpFile::from_value(
            serde_json::from_str(&format!(
                r#"{{"schema_version":1,"ops":[{{"op":"replace-resource","erp":"{SCENES}","resource":"{SCENE}","source":"{src}"}}]}}"#
            ))
            .unwrap(),
            FileKind::Erp,
        )
        .unwrap();
        let mut b = ErpBuild::new(&original);
        for op in &file.ops {
            let AssetOp::Erp(op) = op else { continue };
            b.apply(op, &|s: &Source| Ok(s.resolve(&dir)?)).unwrap();
        }
        b
    };
    let raw = |r: &Resource| r.fragments.iter().map(|f| f.raw.clone()).collect::<Vec<_>>();
    let check = |written: &Erp, data: &[u8]| {
        assert_eq!(written.resources.len(), stock.erp.resources.len());
        for (w, s) in written.resources.iter().zip(&stock.erp.resources) {
            assert_eq!((&w.name, w.kind, &w.unk), (&s.name, s.kind, &s.unk));
            if w.name == SCENE {
                assert_eq!(w.fragments[0].data().unwrap(), data);
                assert_eq!(w.fragments[0].size, data.len() as u64);
                assert_eq!(w.hash, w.content_hash().unwrap());
            } else {
                assert_eq!(w.hash, s.hash);
                assert_eq!(raw(w), raw(&stock.load(s).unwrap()));
            }
        }
    };

    let b = build("same.idf");
    assert!(b.resized().is_empty());
    let out = dir.join("same");
    assert_eq!(b.write(&out).unwrap(), [SCENES]);
    let written = Erp::parse(&std::fs::read(out.join(SCENES)).unwrap()).unwrap();
    check(&written, &scene);
    assert_eq!(written.find(SCENE).unwrap().hash, entry.hash);

    let mut b = build("five.idf");
    let resized = b.resized();
    assert_eq!(resized.len(), 1);
    assert_eq!(resized[0].erp, SCENES);
    assert_eq!(resized[0].resources[0].0, SCENE);
    let out = dir.join("five");
    assert_eq!(b.take_resized(&out).unwrap(), resized);
    assert_eq!(b.resources().count(), 0);
    let bytes = std::fs::read(out.join(SCENES)).unwrap();
    let mut written = Erp::parse(&bytes).unwrap();
    check(&written, &bigger);
    assert_eq!(written.to_bytes(), bytes);
    std::fs::remove_dir_all(&dir).ok();
}
