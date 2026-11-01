use std::path::PathBuf;

use swofty_formats::erp::ErpFile;
use swofty_formats::pkg::{Node, Pkg};

/// The core's MULTIPLAYER CAREER home tile fires fe_career's two-player career routes from fe_home; the
/// routes patch must copy them into fe_home's group. Needs F1_25_DIR.
#[test]
fn home_gets_the_two_player_career_routes() {
    let Some(game) = std::env::var_os("F1_25_DIR") else {
        eprintln!("F1_25_DIR is not set; skipping");
        return;
    };
    let erp = PathBuf::from(game).join("2025_asset_groups/ui_package/persistent.erp");
    let file = ErpFile::open(&erp).unwrap();
    let res = file
        .load(
            file.erp
                .find("eaid://ui_package/routes/routes_frontend_general.uir")
                .unwrap(),
        )
        .unwrap();
    let out =
        swofty_core::ui::patch::routes(&res.fragments[0].data().unwrap(), Vec::new(), Vec::new()).unwrap();
    let p = Pkg::parse(&out).unwrap();
    let Node::Arr(gs) = p.root.as_obj().unwrap().get("RouteGroupings").unwrap() else {
        panic!("RouteGroupings is not an array")
    };
    let id = |g: &Node| {
        g.as_obj()
            .and_then(|o| o.get("Id"))
            .and_then(|v| v.as_val())
            .and_then(|v| v.as_str())
            .map(str::to_owned)
    };
    let home = gs
        .items
        .iter()
        .find(|g| id(g).as_deref() == Some("fe_home"))
        .unwrap();
    let text = swofty_core::ui::node::canonical(home);
    for r in swofty_core::ui::patch::MP_CAREER_ROUTES {
        assert!(text.contains(&format!("Str(\"{r}\")")), "fe_home lacks {r}");
    }
    let wizard = text.split("Str(\"career_wizard_online\")").nth(1).unwrap();
    assert!(!wizard[..wizard.find("Str(\"career_dlc").unwrap_or(wizard.len())].contains("\"fe_career\""));
}
