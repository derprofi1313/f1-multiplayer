use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use swofty_core::asset::{FileKind, OpFile, Origin};
use swofty_core::ops::{DbOps, DbReport, apply_db_ops_report};
use swofty_core::planner::{Features, Planner};
use swofty_core::rowkeys::{Names, RowKeys};
use swofty_core::runtime::db_edits;
use swofty_formats::gvdb::{Gvdb, Record};

const ITEMS: [(&str, &str); 8] = [
    ("suits", "Suit"),
    ("gloves", "Gloves"),
    ("helmets", "Helmet"),
    ("tops", "Tops"),
    ("trousers", "Trousers"),
    ("casual_footwear", "CasualFootWear"),
    ("casual_headwear", "CasualHeadwear"),
    ("eyewear", "Eyewear"),
];
const COLOURS: [&str; 4] = [
    "colour_1_default",
    "colour_2_default",
    "colour_3_default",
    "colour_4_default",
];

fn table(category: &str) -> String {
    format!("customisation.custom_avatar.{category}")
}

fn stock() -> Gvdb {
    let mut records = Vec::new();
    for (category, word) in ITEMS {
        let t = table(category);
        records.push(Record::new(&format!("{t}._{word}_instancecount_"), 0, "1"));
        let row = format!("{t}.{word}_0");
        records.push(Record::new(
            &format!("{row}._metadata_"),
            4,
            "2024-mercedes_lewis_hamilton",
        ));
        records.push(Record::new(&format!("{row}.id"), 0, "7"));
        records.push(Record::new(&format!("{row}.nb_colours"), 0, "0"));
        for (i, c) in COLOURS.iter().enumerate() {
            let value = if i == 0 { "-1710619" } else { "" };
            records.push(Record::new(&format!("{row}.{c}"), 11, value));
        }
    }
    Gvdb { records }
}

fn copy_op(category: &str, word: &str) -> String {
    let t = table(category);
    format!(
        r#"{{"op":"add-row","table":"{t}","from":"{t}.{word}[_metadata_=2024-mercedes_lewis_hamilton]",
        "set":{{"_metadata_":"1990-swofty"}}}}"#
    )
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
fn an_item_copy_gets_the_four_colour_defaults_and_keeps_them_on_the_runtime_route() {
    let ops: Vec<String> = ITEMS.iter().map(|(c, w)| copy_op(c, w)).collect();
    let (built, report) = build(&format!(r#"{{"schema_version":1,"ops":[{}]}}"#, ops.join(",")));
    for (category, word) in ITEMS {
        let t = table(category);
        assert_eq!(built.instance_count(&t, word), Some(2), "{t}");
        for (i, c) in COLOURS.iter().enumerate() {
            let src = built.get(&format!("{t}.{word}_0.{c}")).unwrap();
            let copy = built
                .get(&format!("{t}.{word}_1.{c}"))
                .unwrap_or_else(|| panic!("{t}.{word}_1 lacks {c}: {:#?}", report.lines));
            assert_eq!((copy.tag, &copy.value), (11, &src.value), "{t}.{word}_1.{c}");
            assert_eq!(copy.value.is_empty(), i > 0);
        }
    }
    let edits = db_edits(&stock(), &built).unwrap();
    let empty = edits.iter().filter(|e| e.tag == 11 && e.value.is_empty()).count();
    assert_eq!(empty, ITEMS.len() * 3);
}

#[test]
fn a_listed_item_copy_without_the_colour_defaults_warns() {
    let file = format!(
        r#"{{"schema_version":1,"fields":{{"{t}.Gloves":["_metadata_","id","nb_colours"]}},
        "ops":[{op}]}}"#,
        t = table("gloves"),
        op = copy_op("gloves", "Gloves").replacen(
            r#""op":"add-row","#,
            r#""op":"add-row","copy":"listed","#,
            1
        )
    );
    let ops: DbOps = serde_json::from_str(&file).unwrap();
    let none = BTreeMap::new();
    let names = Names {
        fields: &none,
        keys: RowKeys::builtin(),
        stock: None,
        tracks: None,
    };
    let mut db = stock();
    let mut report = DbReport::default();
    apply_db_ops_report(&mut db, &ops, &names, &mut report).unwrap();
    assert!(
        report.warnings.iter().any(|w| w.contains("misses 4 key(s) of the source row")
            && COLOURS.iter().all(|c| w.contains(c))),
        "{report:#?}"
    );
}

#[test]
fn a_stock_gloves_copy_brings_its_colour_defaults() {
    let Some(dir) = std::env::var_os("F1_25_DIR").map(PathBuf::from) else {
        eprintln!("F1_25_DIR is not set; skipping");
        return;
    };
    let Ok(data) = std::fs::read(dir.join("game_package/gamedatabases/gamedatabases.2025.bin")) else {
        eprintln!("no gamedatabases.2025.bin under F1_25_DIR; skipping");
        return;
    };
    let stock = Gvdb::parse(&data).unwrap();
    let mut db = stock.clone();
    let t = table("gloves");
    let n = db.instance_count(&t, "Gloves").unwrap();
    let ops: DbOps = serde_json::from_str(&format!(
        r#"{{"schema_version":1,"ops":[{}]}}"#,
        copy_op("gloves", "Gloves")
    ))
    .unwrap();
    let none = BTreeMap::new();
    let names = Names {
        fields: &none,
        keys: RowKeys::builtin(),
        stock: None,
        tracks: None,
    };
    apply_db_ops_report(&mut db, &ops, &names, &mut DbReport::default()).unwrap();
    for c in COLOURS {
        let copy = db.get(&format!("{t}.Gloves_{n}.{c}")).expect(c);
        assert_eq!(copy.tag, 11, "{c}");
    }
    assert!(db_edits(&stock, &db).is_ok());
}
