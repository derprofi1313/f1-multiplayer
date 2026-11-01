use std::path::PathBuf;

use swofty_core::asset::Source;
use swofty_core::erpbuild::ErpBuild;
use swofty_core::ops::ErpOp;
use swofty_formats::erp::{Erp, ErpFile};
use swofty_formats::texarray::Array;

const UI: &str = "2025_asset_groups/ui_package/common_front_end.erp";
const BOOT: &str = "2025_asset_groups/vfx_package/common.erp";

fn game() -> Option<PathBuf> {
    match std::env::var_os("F1_25_GAME") {
        Some(d) => Some(PathBuf::from(d)),
        None => {
            eprintln!("F1_25_GAME is not set; skipping");
            None
        }
    }
}

#[test]
fn merges_two_addons_logo_slices_from_the_game_array() {
    let Some(game) = game() else {
        return;
    };
    let stock = ErpFile::open(&game.join(UI)).unwrap();
    let (res, array) = stock
        .erp
        .resources
        .iter()
        .filter(|r| r.name.contains("team_logos") && r.name.ends_with(".texarray.image"))
        .find_map(|r| {
            let loaded = stock.load(r).unwrap();
            Array::read(&loaded).filter(|a| a.count >= 3).map(|a| (loaded, a))
        })
        .expect("a team logo texture array in common_front_end.erp");
    let dir = std::env::temp_dir().join(format!("swofty-real-merge-slices-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).unwrap();
    for (name, slice) in [("a.erp", 0), ("b.erp", 2)] {
        let mut data = array.data.clone();
        let n = array.slice_len();
        data[slice * n..(slice + 1) * n].iter_mut().for_each(|b| *b = !*b);
        let mut changed = res.clone();
        changed.set_fragment(1, &data).unwrap();
        let mut erp = Erp {
            header: stock.erp.header,
            version: stock.erp.version,
            resources: vec![changed],
        };
        std::fs::write(dir.join(name), erp.to_bytes()).unwrap();
    }
    let original = |rel: &str| game.join(rel);
    let resolve = |s: &Source| Ok(s.resolve(&dir)?);
    for target in [BOOT, UI] {
        let mut b = ErpBuild::new(&original);
        for (origin, source) in [("a", "a.erp"), ("b", "b.erp")] {
            let op = ErpOp::MergeSlices {
                erp: target.into(),
                resource: res.name.clone(),
                source: Source::addon(source),
            };
            b.apply_for(&op, origin, &resolve).unwrap();
        }
        let merges = b.take_merges();
        let changed: Vec<&[usize]> = merges.iter().map(|m| m.changed.as_slice()).collect();
        assert_eq!(changed, [&[0][..], &[2][..]], "{target}");
        assert!(
            merges
                .iter()
                .all(|m| !m.whole && m.notes.is_empty() && m.slices == array.count)
        );
        let (_, merged, _, origin) = b.resources().find(|(_, r, _, _)| r.name == res.name).unwrap();
        assert_eq!(origin, "b");
        let out = Array::read(merged).unwrap();
        assert_eq!(
            merged.fragments[0].data().unwrap(),
            res.fragments[0].data().unwrap()
        );
        for i in 0..array.count {
            assert_eq!(
                out.slice(i) == array.slice(i),
                i != 0 && i != 2,
                "{target} slice {i}"
            );
        }
        if target == UI {
            assert!(b.resized().is_empty());
        }
    }
    std::fs::remove_dir_all(&dir).ok();
}
