use std::collections::BTreeMap;
use std::path::PathBuf;

use swofty_core::asset::Source;
use swofty_core::erpbuild::ErpBuild;
use swofty_core::ops::{ErpOp, MaterialValue};
use swofty_formats::material;

const CAR: &str = "2025_asset_groups/f1_2025_vehicle_package/teams/redbull/wep/redbull.erp";
const PAINT: &str = "eaid://f1_2025_vehicle_package/teams/redbull/materials/redbull_paint.material";

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
fn sets_a_car_paint_constant_and_keeps_the_rest_of_the_material() {
    let Some(game) = game() else {
        return;
    };
    let original = |rel: &str| game.join(rel);
    let resolve = |s: &Source| Ok(s.resolve(&game)?);
    let mut b = ErpBuild::new(&original);
    let op = ErpOp::SetMaterial {
        erp: CAR.into(),
        resource: PAINT.into(),
        params: BTreeMap::from([
            (
                "me_lacquerColour".to_owned(),
                MaterialValue::Text("0.1 1 0.1".into()),
            ),
            ("me_flakesStrength".to_owned(), MaterialValue::Scalar(0.5)),
        ]),
    };
    b.apply_for(&op, "a", &resolve).unwrap();
    let (_, res, _, origin) = b.resources().find(|(_, r, _, _)| r.name == PAINT).unwrap();
    assert_eq!(origin, "a");
    let body = res.fragments[0].data().unwrap();
    assert_eq!(
        material::get(&body, "me_lacquerColour").unwrap(),
        [0.1f32 as f64, 1.0, 0.1f32 as f64]
    );
    assert_eq!(material::get(&body, "me_flakesStrength").unwrap(), [0.5]);
    assert_eq!(material::get(&body, "me_lacquerIor").unwrap(), [1.3f32 as f64]);
    let bad = ErpOp::SetMaterial {
        erp: CAR.into(),
        resource: PAINT.into(),
        params: BTreeMap::from([("me_albedoMapHeapIndex".to_owned(), MaterialValue::Scalar(1.0))]),
    };
    let err = b.apply_for(&bad, "a", &resolve).unwrap_err().to_string();
    assert!(
        err.contains("me_albedoMapHeapIndex is a bindless heap index"),
        "{err}"
    );
}
