use std::collections::BTreeSet;

use swofty_core::asset::{FileKind, OpFile, Origin};
use swofty_core::ops::DbReport;
use swofty_core::planner::{Features, Planner};
use swofty_formats::gvdb::{Gvdb, Record};

const MODES: &str = "entrants.entrant_setup_modes";
const MODE: &str = "entrants.entrant_setup_modes.entrant_setup_mode";

fn stock() -> Gvdb {
    let mut records = vec![Record::new(
        &format!("{MODES}._entrant_setup_mode_instancecount_"),
        0,
        "2",
    )];
    for (i, (label, id)) in [("f1_26", "11"), ("f1_generic_2026", "12")].iter().enumerate() {
        let row = format!("{MODE}_{i}");
        records.push(Record::new(&format!("{row}._metadata_"), 4, *label));
        records.push(Record::new(&format!("{row}.stat_id"), 0, *id));
        records.push(Record::new(&format!("{row}.max_entrants"), 0, "22"));
        for who in ["player", "second_player", "ai"] {
            for what in ["driver", "team", "vehicle"] {
                for flag in ["preloaded", "userDefined", "networkReplicated"] {
                    records.push(Record::new(&format!("{row}.{who}.{what}.{flag}"), 5, "True"));
                }
            }
        }
    }
    Gvdb { records }
}

fn build(file: &str) -> (Gvdb, DbReport) {
    let features = Features::default();
    let opt_in = BTreeSet::new();
    let mut planner = Planner::new(&features, &opt_in);
    let file = OpFile::from_value(serde_json::from_str(file).unwrap(), FileKind::Database).unwrap();
    let origin = Origin {
        addon: "f1-1990".into(),
        item: None,
    };
    planner.push_file(&origin, FileKind::Database, file);
    let mut db = stock();
    let keys = db.records.iter().map(|r| r.key).collect();
    let mut report = DbReport::default();
    planner
        .apply_db_report(&mut db, &keys, &Default::default(), &mut report)
        .unwrap();
    (db, report)
}

#[test]
fn a_clone_gets_the_run_time_keys_of_its_source_and_its_own_stat_id() {
    let (db, report) = build(&format!(
        r#"{{"schema_version":1,"ops":[{{"op":"add-row","table":"{MODES}",
        "from":"{MODE}[_metadata_=f1_26]","set":{{"_metadata_":"f1_1990"}}}}]}}"#
    ));
    let new = format!("{MODE}_2");
    assert_eq!(db.instance_count(MODES, "entrant_setup_mode"), Some(3));
    assert_eq!(
        db.get_text(&format!("{new}._metadata_")).as_deref(),
        Some("f1_1990")
    );
    assert_eq!(
        db.get_text(&format!("{new}.ai.vehicle.networkReplicated"))
            .as_deref(),
        Some("True")
    );
    assert_eq!(db.get(&format!("{new}.player.driver.preloaded")).unwrap().tag, 5);
    assert_eq!(db.get_text(&format!("{new}.stat_id")).as_deref(), Some("13"));
    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    assert!(
        report.lines.contains(&format!(
            "f1-1990: add-row {new} (from {MODE}[_metadata_=f1_26]): stat_id 13 (the source's is 11)"
        )),
        "{:?}",
        report.lines
    );
    assert!(
        report.lines.iter().any(|l| l.ends_with("copied 30 key(s)")),
        "{:?}",
        report.lines
    );
}

#[test]
fn a_short_fields_list_is_reported_and_a_listed_copy_warns_about_the_gap() {
    let fields = r#""fields":{"entrants.entrant_setup_modes.entrant_setup_mode":["_metadata_","stat_id","max_entrants"]}"#;
    let (db, report) = build(&format!(
        r#"{{"schema_version":1,{fields},"ops":[{{"op":"add-row","table":"{MODES}",
        "from":"{MODE}_0","set":{{"_metadata_":"f1_1990"}}}}]}}"#
    ));
    assert!(
        db.get(&format!("{MODE}_2.second_player.team.userDefined"))
            .is_some()
    );
    assert!(
        report
            .lines
            .iter()
            .any(|l| l.contains("copied 27 key(s) the fields list")),
        "{:?}",
        report.lines
    );

    let (db, report) = build(&format!(
        r#"{{"schema_version":1,{fields},"ops":[{{"op":"add-row","table":"{MODES}",
        "from":"{MODE}_0","copy":"listed","set":{{"_metadata_":"f1_1990"}}}}]}}"#
    ));
    assert!(
        db.get(&format!("{MODE}_2.second_player.team.userDefined"))
            .is_none()
    );
    assert_eq!(report.warnings.len(), 1, "{:?}", report.warnings);
    assert!(
        report.warnings[0].starts_with("f1-1990: add-row")
            && report.warnings[0]
                .contains("misses 27 key(s) of the source row, not copied: ai.driver.networkReplicated"),
        "{}",
        report.warnings[0]
    );
}
