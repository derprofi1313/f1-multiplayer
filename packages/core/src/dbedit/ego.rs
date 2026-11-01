//! Imports exports of PavelGames' EGO GameDatabases Editor
//! (`"format": "ego-gamedatabases-editor-records"`).

use std::collections::BTreeMap;

use serde::Deserialize;

pub(crate) const FORMAT: &str = "ego-gamedatabases-editor-records";

#[derive(Deserialize)]
struct File {
    records: Records,
}

#[derive(Deserialize)]
struct Records {
    #[serde(default)]
    modified_records: BTreeMap<String, String>,
    #[serde(default)]
    added_records: Vec<serde_json::Value>,
}

/// The (key, value text) changes of an EGO export. Added records carry an
/// obfuscated key that stays undecoded; a file with any is refused.
pub(crate) fn modified(doc: serde_json::Value, file_name: &str) -> Result<Vec<(u64, String)>, String> {
    let file: File = serde_json::from_value(doc).map_err(|e| format!("{file_name}: {e}"))?;
    let added = file.records.added_records.len();
    if added > 0 {
        return Err(format!(
            "{file_name} adds {added} new record(s) and changes {} value(s); only value changes import, \
             so this file cannot be imported",
            file.records.modified_records.len()
        ));
    }
    file.records
        .modified_records
        .into_iter()
        .map(|(k, hex)| {
            let key = k
                .parse::<u64>()
                .map_err(|_| format!("{file_name}: bad record key {k}"))?;
            let value = unhex(&hex)
                .and_then(|b| String::from_utf8(b).ok())
                .ok_or_else(|| format!("{file_name}: record {k} has a bad value"))?;
            Ok((key, value))
        })
        .collect()
}

fn unhex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) || !s.is_ascii() {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_modified_records_and_refuses_added_ones() {
        let doc = serde_json::json!({"format": FORMAT, "format_version": 1, "metadata": {},
            "records": {"modified_records": {"10159482347583347520": "4A756C79"}, "added_records": []}});
        assert_eq!(
            modified(doc, "a.json").unwrap(),
            [(10159482347583347520, "July".to_owned())]
        );
        let doc = serde_json::json!({"format": FORMAT, "records": {"modified_records": {"1": "31"},
            "added_records": [{"key_encoded": "x", "tag": 2, "default_value": "y"}]}});
        let err = modified(doc, "b.json").unwrap_err();
        assert!(err.contains("adds 1 new record(s)"), "{err}");
    }
}
