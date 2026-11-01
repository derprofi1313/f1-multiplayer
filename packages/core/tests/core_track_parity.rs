use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

use serde_json::json;
use swofty_core::activation::db::{Registry, TrackHost, activate};
use swofty_core::map::TrackPackage;
use swofty_formats::gvdb::{Gvdb, Record};

const CASES: &[(&str, &str, Option<&str>, &[&str])] = &[
    ("okayama", "melbourne", Some("melbourne"), &[]),
    ("bahrain24", "bahrain", None, &[]),
    ("delhi", "zandvoort", None, &[]),
    (
        "pictures",
        "bahrain",
        None,
        &["fe_world.tracks_spline", "tracks.flags_circle"],
    ),
];

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../native/test/tracks/fixture")
}

fn unescape(s: &str) -> String {
    let mut out = String::new();
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match it.next() {
            Some('t') => out.push('\t'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some(o) => out.push(o),
            None => out.push('\\'),
        }
    }
    out
}

fn fixture() -> Gvdb {
    let text = std::fs::read_to_string(fixture_dir().join("tracks.tsv"))
        .expect("native/test/tracks/fixture/tracks.tsv");
    let records = text
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            let mut parts = l.splitn(3, '\t');
            let key = u64::from_str_radix(parts.next().unwrap(), 16).unwrap();
            let tag: u8 = parts.next().unwrap().parse().unwrap();
            Record {
                key,
                tag,
                value: unescape(parts.next().unwrap_or("")).into_bytes(),
            }
        })
        .collect();
    Gvdb { records }
}

#[test]
fn track_lowering_parity() {
    let base = fixture();
    let before: HashMap<u64, (u8, Vec<u8>)> = base
        .records
        .iter()
        .map(|r| (r.key, (r.tag, r.value.clone())))
        .collect();
    let reg = Registry::builtin();
    for (code, from, slot, lists) in CASES {
        let mut db = base.clone();
        let pkg: TrackPackage = serde_json::from_value(json!({
            "track_code": code,
            "erp": format!("{code}.erp"),
            "activation": { "copy_from_track": from, "copy_slot": slot }
        }))
        .unwrap();
        let lists: BTreeSet<String> = lists.iter().map(|l| l.to_string()).collect();
        activate(&mut db, &pkg, &reg, Some(&TrackHost::Natural), &lists).unwrap();
        let mut records: Vec<&Record> = db
            .records
            .iter()
            .filter(|r| before.get(&r.key) != Some(&(r.tag, r.value.clone())))
            .collect();
        records.sort_by_key(|r| r.key);
        let out: Vec<_> = records
            .iter()
            .map(|r| json!({ "key": format!("{:016x}", r.key), "tag": r.tag, "text": r.text() }))
            .collect();
        // Keys go in sorted order: a workspace build turns on serde_json's preserve_order (through gpui) and a
        // swofty-core build alone does not, and the text must be the same under both.
        let doc = json!({ "from": from, "records": out, "slot": slot, "track": code });
        let text = serde_json::to_string_pretty(&doc).unwrap() + "\n";
        let path = fixture_dir().join("expected").join(format!("{code}.json"));
        if std::env::var_os("SWOFTY_WRITE_PARITY").is_some() || !path.exists() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &text).unwrap();
            continue;
        }
        let have = std::fs::read_to_string(&path).unwrap();
        assert!(
            have == text,
            "{code}: the launcher's records differ from {}; rerun with SWOFTY_WRITE_PARITY=1 and run native/test/run_tracks_test.sh",
            path.display()
        );
    }
}
