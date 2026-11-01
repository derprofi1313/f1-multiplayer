use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use swofty_core::ui::catalog::{self, Scenes};
use swofty_core::ui::check::{self, ArchiveUi};

fn archives(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() && !p.ends_with("texture_arrays") && !p.ends_with("textures") {
            archives(&p, out);
        } else if p.extension().is_some_and(|x| x == "erp") {
            out.push(p);
        }
    }
}

#[test]
fn every_stock_ui_and_scene_passes_the_catalogue() {
    let Some(game) = std::env::var_os("F1_25_DIR").map(PathBuf::from) else {
        eprintln!("F1_25_DIR is not set; skipping");
        return;
    };
    let mut erps = Vec::new();
    archives(&game.join("2025_asset_groups/ui_package"), &mut erps);
    erps.sort();
    let (cat, engine) = (catalog::stock(), catalog::engine());
    let (mut seen, mut bad, mut checked) = (BTreeSet::new(), Vec::new(), 0);
    for erp in erps {
        let name = erp.strip_prefix(&game).unwrap().to_string_lossy().into_owned();
        let archive = ArchiveUi::read(&name, &erp).unwrap();
        let mut scenes = Scenes::default();
        for (kind, frags) in archive.resources.values() {
            if kind == "UILayout" {
                scenes.add(&frags[0]).unwrap();
            }
        }
        for (res, (kind, frags)) in &archive.resources {
            if !seen.insert((res.clone(), frags.clone())) {
                continue;
            }
            for (i, data) in frags.iter().enumerate() {
                checked += 1;
                let frag = format!("{kind}#{i}");
                let mut found = Vec::new();
                if kind == "UIDF" {
                    let input = check::UiInput {
                        resource: res,
                        frag: &frag,
                        data,
                        stock: Some(data),
                    };
                    let r = check::ui_edit(&input, &scenes, cat, engine).unwrap();
                    found.extend(r.hard.into_iter().chain(r.warnings));
                    found.extend(check::scene_bindings(&name, &input, &scenes, &scenes, cat).unwrap());
                } else {
                    let r = check::scene_edit(res, data, cat).unwrap();
                    found.extend(r.hard.into_iter().chain(r.warnings));
                }
                bad.extend(found.iter().map(|m| format!("{name}: {res}: {m}")));
            }
        }
    }
    assert!(checked > 5000, "only {checked} fragments checked");
    assert!(
        bad.is_empty(),
        "{} findings on stock files:\n{}",
        bad.len(),
        bad[..bad.len().min(40)].join("\n")
    );
}

#[test]
fn the_era_picker_cases_split_on_the_visual_nav_rule() {
    let Some(dir) = std::env::var_os("SWOFTY_UI_CASES").map(PathBuf::from) else {
        eprintln!("SWOFTY_UI_CASES is not set; skipping");
        return;
    };
    let Some(game) = std::env::var_os("F1_25_DIR").map(PathBuf::from) else {
        eprintln!("F1_25_DIR is not set; skipping");
        return;
    };
    let stock = game.join("2025_asset_groups/ui_package/flow_f1_world.erp");
    let run = |file: &str| {
        check::archive_files(&[("flow_f1_world.erp", stock.as_path(), dir.join(file).as_path())]).unwrap()
    };
    let broken = run("breaks_newnav_flow_f1_world.erp");
    let nav: Vec<_> = broken
        .hard
        .iter()
        .filter(|h| h.code == "behaviour.nbvn_visualnav")
        .collect();
    assert_eq!(nav.len(), 1, "{broken:#?}");
    assert!(nav[0].at.contains("dynamic_era_select"), "{nav:#?}");
    assert!(nav[0].source.contains("0x144289830"), "{nav:#?}");
    let fixed = run("fixed_visualnav_flow_f1_world.erp");
    assert!(fixed.hard.is_empty(), "{fixed:#?}");
}
