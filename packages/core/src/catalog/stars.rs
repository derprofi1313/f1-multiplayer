//! Stars: the addons this launcher's user starred, kept in
//! `$DATA/state/stars.json` with a random client id the registry links to the
//! user. A star changes the local set and count at once and waits in a queue
//! until a sync sends it; the registry's answer then replaces the set and the
//! counts, with the changes made while it was on the way applied again.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{TIMEOUT, Trouble, agent_name, base_url};
use crate::{Result, io_err};

pub const FILE: &str = "state/stars.json";
/// The header Swofty's registry reads the star client from. A header name cannot contain spaces.
pub const CLIENT_HEADER: &str = "X-Swofty-Client";
pub const SYNC_LIMIT: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Change {
    pub addon: String,
    pub starred: bool,
    pub at: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Stars {
    #[serde(default)]
    pub client: String,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub starred: BTreeSet<String>,
    #[serde(default)]
    pub counts: BTreeMap<String, u64>,
    /// Whether `counts` came from a sync, so an addon missing from it has none.
    #[serde(default)]
    pub counted: bool,
    #[serde(default)]
    pub pending: Vec<Change>,
    #[serde(default)]
    pub synced_at: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Reply {
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub stars: Vec<String>,
    #[serde(default)]
    pub counts: BTreeMap<String, u64>,
    #[serde(default)]
    pub applied: usize,
    #[serde(default)]
    pub server_time: String,
}

fn valid_client(id: &str) -> bool {
    id.len() == 32
        && id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

pub fn path(root: &Path) -> PathBuf {
    root.join(FILE)
}

impl Stars {
    /// The saved stars, with a new client id when there is none yet.
    pub fn load(root: &Path) -> Self {
        let mut stars: Self = std::fs::read_to_string(path(root))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        if !valid_client(&stars.client) {
            stars.client = crate::usage::random_hex(2);
            if let Err(e) = stars.save(root) {
                eprintln!("swofty-core: could not save the stars: {e}");
            }
        }
        stars
    }

    pub fn save(&self, root: &Path) -> Result<()> {
        let path = path(root);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(io_err(dir))?;
        }
        let text = serde_json::to_string_pretty(self).unwrap_or_default();
        std::fs::write(&path, text).map_err(io_err(&path))
    }

    pub fn has(&self, id: &str) -> bool {
        self.starred.contains(id)
    }

    /// Its star count: the last sync's, or `listed` from the catalog before any.
    pub fn count(&self, id: &str, listed: u64) -> u64 {
        self.counts
            .get(id)
            .copied()
            .unwrap_or(if self.counted { 0 } else { listed })
    }

    fn set(&mut self, id: &str, starred: bool, listed: u64) {
        if self.has(id) == starred {
            return;
        }
        let n = self.count(id, listed);
        self.counts
            .insert(id.to_owned(), if starred { n + 1 } else { n.saturating_sub(1) });
        if starred {
            self.starred.insert(id.to_owned());
        } else {
            self.starred.remove(id);
        }
    }

    /// Stars or unstars `id` now and queues the change; whether it is starred after.
    pub fn toggle(&mut self, id: &str, listed: u64, at: String) -> bool {
        let starred = !self.has(id);
        self.set(id, starred, listed);
        self.pending.retain(|c| c.addon != id);
        self.pending.push(Change {
            addon: id.to_owned(),
            starred,
            at,
        });
        starred
    }

    /// The changes the next sync sends.
    pub fn outgoing(&self) -> Vec<Change> {
        self.pending.iter().take(SYNC_LIMIT).cloned().collect()
    }

    /// Takes the registry's answer to sending `sent`.
    pub fn synced(&mut self, sent: &[Change], reply: Reply, at: String) {
        self.pending.retain(|c| !sent.contains(c));
        self.starred = reply.stars.into_iter().collect();
        self.counts = reply.counts;
        self.counted = true;
        for c in self.pending.clone() {
            self.set(&c.addon, c.starred, 0);
        }
        self.user = reply.user;
        self.synced_at = Some(at);
        self.error = None;
    }

    pub fn failed(&mut self, error: String) {
        self.error = Some(error);
    }
}

fn error_text(url: &str, status: u16, body: &str) -> String {
    let said = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("error")?.as_str().map(str::to_owned));
    match said {
        Some(e) => format!("{url}: HTTP {status}: {e}"),
        None => format!("{url}: HTTP {status}"),
    }
}

/// Sends `changes` for `client` and returns the user's stars and every count.
pub fn sync(client: &str, changes: &[Change]) -> std::result::Result<Reply, Trouble> {
    let url = format!("{}/v1/stars/sync", base_url());
    let body = serde_json::json!({ "changes": changes }).to_string();
    let response = ureq::post(&url)
        .header("User-Agent", agent_name())
        .header(CLIENT_HEADER, client)
        .header("Content-Type", "application/json")
        .config()
        .timeout_global(Some(TIMEOUT))
        .http_status_as_error(false)
        .build()
        .send(body)
        .map_err(|e| Trouble::Offline(format!("{url}: {e}")))?;
    let status = response.status().as_u16();
    let text = response
        .into_body()
        .with_config()
        .limit(4 * 1024 * 1024)
        .read_to_string()
        .map_err(|e| Trouble::Failed(format!("{url}: {e}")))?;
    if status >= 400 {
        return Err(Trouble::Failed(error_text(&url, status, &text)));
    }
    serde_json::from_str(&text).map_err(|e| Trouble::Failed(format!("{url}: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reply(stars: &[&str], counts: &[(&str, u64)]) -> Reply {
        Reply {
            user: Some("u_1".into()),
            stars: stars.iter().map(|s| (*s).to_owned()).collect(),
            counts: counts.iter().map(|(k, v)| ((*k).to_owned(), *v)).collect(),
            applied: 0,
            server_time: String::new(),
        }
    }

    #[test]
    fn stars_at_once_and_queues_one_change_per_addon() {
        let mut s = Stars::default();
        assert_eq!(s.count("a", 4), 4);
        assert!(s.toggle("a", 4, "t1".into()));
        assert!(s.has("a"));
        assert_eq!(s.count("a", 4), 5);
        assert!(!s.toggle("a", 4, "t2".into()));
        assert_eq!(s.count("a", 4), 4);
        assert_eq!(
            s.pending,
            [Change {
                addon: "a".into(),
                starred: false,
                at: "t2".into()
            }]
        );
        s.toggle("b", 0, "t3".into());
        assert_eq!(s.outgoing().len(), 2);
    }

    #[test]
    fn a_sync_replaces_the_set_and_counts() {
        let mut s = Stars::default();
        s.toggle("a", 1, "t1".into());
        let sent = s.outgoing();
        s.synced(&sent, reply(&["a", "c"], &[("a", 9), ("c", 3)]), "now".into());
        assert!(s.pending.is_empty());
        assert_eq!(s.starred, BTreeSet::from(["a".to_owned(), "c".to_owned()]));
        assert_eq!(s.count("a", 1), 9);
        assert_eq!(s.count("z", 7), 0);
        assert_eq!(s.synced_at.as_deref(), Some("now"));
        assert_eq!(s.user.as_deref(), Some("u_1"));
    }

    #[test]
    fn changes_made_during_a_sync_survive_it() {
        let mut s = Stars::default();
        s.toggle("a", 0, "t1".into());
        let sent = s.outgoing();
        s.toggle("a", 0, "t2".into());
        s.toggle("b", 2, "t3".into());
        s.synced(&sent, reply(&["a"], &[("a", 1), ("b", 2)]), "now".into());
        assert!(!s.has("a"));
        assert!(s.has("b"));
        assert_eq!(s.count("a", 0), 0);
        assert_eq!(s.count("b", 0), 3);
        let left: Vec<&str> = s.pending.iter().map(|c| c.at.as_str()).collect();
        assert_eq!(left, ["t2", "t3"]);
    }

    #[test]
    fn a_failed_sync_keeps_the_queue() {
        let mut s = Stars::default();
        s.toggle("a", 0, "t1".into());
        s.failed("offline".into());
        assert_eq!(s.pending.len(), 1);
        assert_eq!(s.error.as_deref(), Some("offline"));
        let sent = s.outgoing();
        s.synced(&sent, reply(&["a"], &[("a", 1)]), "now".into());
        assert!(s.error.is_none());
        assert!(s.pending.is_empty());
    }

    #[test]
    fn the_client_header_name_is_a_valid_http_token() {
        // A name with a space makes every sync fail with "invalid HTTP header name".
        assert!(!CLIENT_HEADER.is_empty());
        assert!(
            CLIENT_HEADER
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-'),
            "{CLIENT_HEADER} is not a valid header name"
        );
    }

    #[test]
    fn keeps_a_client_id_across_loads() {
        let dir = std::env::temp_dir().join(format!("swofty-stars-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let first = Stars::load(&dir);
        assert!(valid_client(&first.client), "{}", first.client);
        let mut s = first.clone();
        s.toggle("a", 0, "t1".into());
        s.save(&dir).unwrap();
        let again = Stars::load(&dir);
        assert_eq!(again.client, first.client);
        assert_eq!(again.pending.len(), 1);
        std::fs::write(path(&dir), "{\"client\":\"NOPE\"}").unwrap();
        assert_ne!(Stars::load(&dir).client, "NOPE");
        std::fs::remove_dir_all(&dir).ok();
        assert_eq!(
            error_text("u", 429, r#"{"error":"slow down"}"#),
            "u: HTTP 429: slow down"
        );
    }
}
