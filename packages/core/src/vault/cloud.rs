//! Steam Cloud status, read-only from `remotecache.vdf`. The launcher never
//! writes that file.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::vdf::{self, Vdf};

/// Steam's record of one file as it last synced it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub size: u64,
    pub sha: String,
    pub syncstate: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Synced,
    Waiting,
    Unknown,
    Local,
}

impl Status {
    pub fn text(self) -> &'static str {
        match self {
            Self::Synced => "synced",
            Self::Waiting => "waiting to upload",
            Self::Unknown => "unknown",
            Self::Local => "local only (no Steam Cloud)",
        }
    }
}

/// Every file block of a `remotecache.vdf`, by file name.
pub fn parse(src: &str) -> BTreeMap<String, Entry> {
    let Some(root) = vdf::parse(src) else {
        return BTreeMap::new();
    };
    let app = root
        .get(&crate::STEAM_APP_ID.to_string())
        .or_else(|| root.entries().map(|(_, v)| v).next());
    app.into_iter()
        .flat_map(Vdf::entries)
        .filter_map(|(name, v)| {
            let sha = v.get("sha")?.str()?.to_ascii_lowercase();
            Some((
                name.clone(),
                Entry {
                    size: v
                        .get("size")
                        .and_then(Vdf::str)
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0),
                    sha,
                    syncstate: v.get("syncstate").and_then(Vdf::str).unwrap_or("").to_owned(),
                },
            ))
        })
        .collect()
}

pub fn read(path: &Path) -> BTreeMap<String, Entry> {
    std::fs::read_to_string(path)
        .map(|s| parse(&s))
        .unwrap_or_default()
}

/// One file: synced when Steam's last sync had this SHA-1 and marked it
/// synced, waiting when the local file differs from what Steam has.
pub fn status(entry: Option<&Entry>, sha1: &str) -> Status {
    match entry {
        None => Status::Unknown,
        Some(e) if e.sha.eq_ignore_ascii_case(sha1) && e.syncstate == "1" => Status::Synced,
        Some(_) => Status::Waiting,
    }
}

/// The whole profile save: synced only when every file is, waiting when any
/// file is.
pub fn combine(all: impl IntoIterator<Item = Status>) -> Status {
    let all: Vec<Status> = all.into_iter().collect();
    if all.is_empty() {
        return Status::Unknown;
    }
    if all.contains(&Status::Local) {
        return Status::Local;
    }
    if all.contains(&Status::Waiting) {
        return Status::Waiting;
    }
    if all.iter().all(|s| *s == Status::Synced) {
        return Status::Synced;
    }
    Status::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#""3059520"
{
	"ChangeNumber"		"510"
	"OSType"		"0"
	"savegame@profile#NXDSMWW.BWW"
	{
		"root"		"0"
		"size"		"900"
		"localtime"		"1793365844"
		"time"		"1793365843"
		"remotetime"		"1793365843"
		"sha"		"AAAA0000000000000000000000000000000000AA"
		"syncstate"		"1"
		"persiststate"		"0"
		"platformstosync2"		"-1"
	}
	"savegame@profile#QKRHMYXE"
	{
		"root"		"0"
		"size"		"416"
		"sha"		"bbbb0000000000000000000000000000000000bb"
		"syncstate"		"2"
	}
}
"#;

    #[test]
    fn reads_file_blocks_and_their_status() {
        let m = parse(FIXTURE);
        assert_eq!(m.len(), 2);
        let data = &m["savegame@profile#NXDSMWW.BWW"];
        assert_eq!(data.size, 900);
        assert_eq!(data.sha, "aaaa0000000000000000000000000000000000aa");
        assert_eq!(data.syncstate, "1");
        assert_eq!(
            status(Some(data), "AAAA0000000000000000000000000000000000AA"),
            Status::Synced
        );
        assert_eq!(status(Some(data), "ffff"), Status::Waiting);
        let index = &m["savegame@profile#QKRHMYXE"];
        assert_eq!(
            status(Some(index), "bbbb0000000000000000000000000000000000bb"),
            Status::Waiting
        );
        assert_eq!(status(m.get("savegame@profile#D1990.UEL"), "x"), Status::Unknown);
        assert_eq!(combine([Status::Synced, Status::Synced]), Status::Synced);
        assert_eq!(combine([Status::Synced, Status::Waiting]), Status::Waiting);
        assert_eq!(combine([Status::Synced, Status::Unknown]), Status::Unknown);
        assert_eq!(combine([]), Status::Unknown);
        assert!(parse("not a vdf {").is_empty());
    }
}
