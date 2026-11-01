use std::collections::BTreeMap;
use std::path::PathBuf;

use swofty_core::dbedit::deps::{self, Copy};
use swofty_core::ops::{DbOps, DbReport, apply_db_ops_report};
use swofty_core::rowkeys::{Names, RowKeys};
use swofty_formats::gvdb::Gvdb;

fn stock() -> Option<Gvdb> {
    let Some(dir) = std::env::var_os("F1_25_DIR").map(PathBuf::from) else {
        eprintln!("F1_25_DIR is not set; skipping");
        return None;
    };
    let Ok(data) = std::fs::read(dir.join("game_package/gamedatabases/gamedatabases.2025.bin")) else {
        eprintln!("no gamedatabases.2025.bin under F1_25_DIR; skipping");
        return None;
    };
    Some(Gvdb::parse(&data).unwrap())
}

fn build(db: Gvdb, ops: &str) -> deps::Report {
    build_report(db, ops).1
}

fn build_report(mut db: Gvdb, ops: &str) -> (DbReport, deps::Report) {
    let ops: DbOps = serde_json::from_str(ops).unwrap();
    let none = BTreeMap::new();
    let names = Names {
        fields: &none,
        keys: RowKeys::builtin(),
        stock: None,
        tracks: None,
    };
    let mut report = DbReport::default();
    apply_db_ops_report(&mut db, &ops, &names, &mut report).unwrap();
    let copies: Vec<Copy> = report
        .added_rows
        .iter()
        .map(|r| Copy {
            addon: "seasons".into(),
            from: r.from.clone(),
            row: r.row.clone(),
            ignore: ops.side_data.ignore.clone(),
            quiet: false,
        })
        .collect();
    let side = deps::check(&db, &copies, &|id| id.to_owned());
    (report, side)
}

const PACKAGE: &str = r#"{ "op": "add-row", "table": "entrants.vehicles.vehicle_packages",
    "from": "entrants.vehicles.vehicle_packages.vehicle_package[_metadata_=f1_2025]",
    "set": { "_metadata_": "cs_2019", "resource_path": "cs_2019_vehicle_package/teams/[FILENAME]/wep/[FILENAME]" } }"#;

const PRESET: &str = r#"{ "op": "add-row", "table": "car_setup.presets",
    "from": "car_setup.presets.vehicle_package_setups[_metadata_=f1_2025_vehicle_package]",
    "set": { "_metadata_": "cs_2019_vehicle_package" } }"#;

#[test]
fn a_season_package_copy_needs_setup_presets_in_the_real_database() {
    let Some(db) = stock() else { return };
    let flagged = |w: &[String]| {
        w.iter().any(|l| {
            l.contains("car_setup.presets.vehicle_package_setups has a row labelled f1_2025_vehicle_package")
                && l.contains("none labelled cs_2019_vehicle_package")
                && l.contains("Car_Load_SetupPreset (0x140c04fd0)")
        })
    };
    let side = build(
        db.clone(),
        &format!(r#"{{"schema_version":1,"ops":[{PACKAGE}]}}"#),
    );
    assert!(flagged(&side.warnings), "{side:#?}");
    let side = build(
        db,
        &format!(r#"{{"schema_version":1,"ops":[{PACKAGE},{PRESET}]}}"#),
    );
    assert!(!flagged(&side.findings), "{side:#?}");
    assert!(
        !side.findings.iter().any(|f| f.contains("ai_package_folder")),
        "{side:#?}"
    );
}

#[test]
fn a_team_copy_names_its_history_row_in_the_real_database() {
    let Some(db) = stock() else { return };
    let side = build(
        db,
        r#"{"schema_version":1,"ops":[{ "op": "add-row", "table": "entrants.teams",
        "from": "entrants.teams.team[_metadata_=williams_26]", "set": { "_metadata_": "team12_26" } }]}"#,
    );
    assert!(
        side.findings.iter().any(|w| w.starts_with(
            "seasons: [front end] showroom.teams_history.team_history has a row labelled williams_26 and none labelled team12_26"
        )),
        "{side:#?}"
    );
    assert!(
        side.findings
            .iter()
            .any(|w| w.contains("entrants.entrant_lists.entrant_list_N.entrant_N.team")),
        "{side:#?}"
    );
    assert!(
        !side
            .warnings
            .iter()
            .any(|w| w.contains("showroom") || w.contains("bank")),
        "{side:#?}"
    );
}

#[test]
fn a_new_driver_gets_one_review_line_in_the_real_database() {
    let Some(db) = stock() else { return };
    let ops = r#"{"schema_version":1,"ops":[{ "op": "add-row", "table": "entrants.drivers",
        "from": "entrants.drivers.driver[_metadata_=lando_norris]", "set": { "_metadata_": "felipe_nasr" } }]}"#;
    let side = build(db.clone(), ops);
    assert_eq!(side.warnings.len(), 1, "{side:#?}");
    assert!(
        side.warnings[0].starts_with("seasons: ") && side.warnings[0].contains("to review"),
        "{side:#?}"
    );
    assert!(
        side.findings.iter().any(|f| f.starts_with(
            "seasons: [review] entrants.expression_label.expression_label has a row labelled lando_norris"
        )),
        "{side:#?}"
    );
    assert!(
        side.findings
            .iter()
            .any(|f| f.starts_with("seasons: [career] accolades.")),
        "{side:#?}"
    );
    let acked = ops.replacen(
        r#""ops":"#,
        r#""side_data":{"ignore":["engine_management.*","entrants.*","*"]},"ops":"#,
        1,
    );
    let side = build(db, &acked);
    assert!(side.warnings.is_empty(), "{side:#?}");
    assert!(
        side.findings.iter().all(|f| f.starts_with("seasons: [ignored] ")),
        "{side:#?}"
    );
}

#[test]
fn a_setup_preset_copy_brings_every_slider_key_in_the_real_database() {
    let Some(db) = stock() else { return };
    let (report, _) = build_report(
        db,
        &format!(r#"{{"schema_version":1,"strict":true,"ops":[{PRESET}]}}"#),
    );
    assert!(
        report.lines.iter().any(
            |l| l.starts_with("add-row car_setup.presets.vehicle_package_setups_43")
                && l.ends_with("copied 421 key(s)")
        ),
        "{:#?}",
        report.lines
    );
    assert!(report.warnings.is_empty(), "{:#?}", report.warnings);
}
