use std::path::PathBuf;

use serde_json::{Value, json};
use swofty_core::manifest::AddonManifest;
use swofty_core::tracks::is_track_addon;

const NAMED: &str = r#"{ "schema_version": 1, "ops": [
  { "op": "track.add", "source": "track" },
  { "op": "lang.set", "language": "*", "key": "lng_okayama_title", "value": "Okayama" },
  { "op": "db.set", "select": "track_data.game_setup.TrackDataSet[_metadata_=f1].TrackData[_metadata_=okayama]",
    "field": "TrackNameData_0.title", "value": "lng_okayama_title" }
] }"#;

struct Addon {
    dir: PathBuf,
}

impl Addon {
    fn new(name: &str, manifest: Value, ops: &str) -> (Self, AddonManifest) {
        let dir = std::env::temp_dir().join(format!("swofty-track-addon-{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        std::fs::write(dir.join("assets/track.json"), ops).unwrap();
        let m: AddonManifest = serde_json::from_value(manifest).unwrap();
        (Self { dir }, m)
    }
}

impl Drop for Addon {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.dir).ok();
    }
}

fn manifest(extra: Value) -> Value {
    let mut m = json!({
        "schema_version": 1,
        "id": "someone.okayama-named",
        "name": "Okayama with its own name",
        "version": "1.0.0",
        "author": "Someone",
        "compat": { "game": "f1_25" },
        "operations": { "assets": ["assets/track.json"] }
    });
    for (k, v) in extra.as_object().unwrap() {
        if k == "operations" {
            for (ok, ov) in v.as_object().unwrap() {
                m["operations"][ok] = ov.clone();
            }
        } else {
            m[k] = v.clone();
        }
    }
    m
}

#[test]
fn a_track_with_its_own_name_is_a_track_addon() {
    let (a, m) = Addon::new("named", manifest(json!({})), NAMED);
    assert!(is_track_addon(&a.dir, &m));
}

#[test]
fn any_other_edit_makes_it_an_ordinary_addon() {
    let livery = NAMED.replace(
        "] }",
        r#", { "op": "db.set", "select": "entrants.teams.team[_metadata_=toro_rosso_26]", "field": "display_name_code", "value": "AT" } ] }"#,
    );
    let (a, m) = Addon::new("livery", manifest(json!({})), &livery);
    assert!(
        !is_track_addon(&a.dir, &m),
        "a db.set on something other than a track name"
    );

    let names_only = r#"{ "schema_version": 1, "ops": [
      { "op": "lang.set", "language": "*", "key": "lng_x", "value": "X" } ] }"#;
    let (a, m) = Addon::new("names", manifest(json!({})), names_only);
    assert!(!is_track_addon(&a.dir, &m), "no track at all");
}

#[test]
fn code_settings_files_and_save_data_keep_it_on_the_addons_page() {
    for (name, extra) in [
        (
            "native",
            json!({ "operations": { "native": [{ "dll": "native/x.dll" }] } }),
        ),
        (
            "files",
            json!({ "operations": { "files": [{ "source": "a", "target": "b" }] } }),
        ),
        (
            "options",
            json!({ "options": [{ "id": "o", "type": "toggle", "label": "O", "default": false, "binds": "native" }] }),
        ),
        ("saves", json!({ "saves": { "affects": true } })),
    ] {
        let Ok(m) = serde_json::from_value::<AddonManifest>(manifest(extra.clone())) else {
            panic!("{name}: the test manifest does not parse: {extra}");
        };
        let (a, _) = Addon::new(name, manifest(json!({})), NAMED);
        assert!(!is_track_addon(&a.dir, &m), "{name}");
    }
}

#[test]
fn an_op_file_that_does_not_read_is_an_ordinary_addon() {
    let (a, m) = Addon::new("broken", manifest(json!({})), "{ not json");
    assert!(!is_track_addon(&a.dir, &m));
}
