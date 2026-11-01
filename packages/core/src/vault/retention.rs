//! Which snapshots the vault keeps: everything from the last 48 hours (at
//! most 40), then the newest of each day until two weeks old, then the newest
//! of each week for eight more weeks. Pinned snapshots and the newest one are
//! always kept. Over the size cap, the oldest unpinned snapshots go first.

use std::collections::{BTreeMap, BTreeSet};

pub const HOUR: i64 = 3600;
pub const DAY: i64 = 24 * HOUR;
pub const WEEK: i64 = 7 * DAY;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Policy {
    pub recent: i64,
    pub recent_max: usize,
    pub daily_until: i64,
    pub weekly_until: i64,
    pub cap_bytes: u64,
}

impl Policy {
    pub fn with_cap_mb(mb: u64) -> Self {
        Self {
            cap_bytes: mb.saturating_mul(1024 * 1024),
            ..Self::default()
        }
    }
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            recent: 48 * HOUR,
            recent_max: 40,
            daily_until: 14 * DAY,
            weekly_until: 14 * DAY + 8 * WEEK,
            cap_bytes: super::DEFAULT_CAP_MB * 1024 * 1024,
        }
    }
}

/// One snapshot as retention sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub id: String,
    pub at: i64,
    pub pinned: bool,
    pub blobs: Vec<(String, u64)>,
}

/// The bytes the distinct blobs of `items` take.
pub fn bytes<'a>(items: impl IntoIterator<Item = &'a Item>) -> u64 {
    let mut seen = BTreeMap::new();
    for i in items {
        for (sha, size) in &i.blobs {
            seen.insert(sha.as_str(), *size);
        }
    }
    seen.values().sum()
}

/// The ids to remove from `items`, oldest first.
pub fn plan(items: &[Item], now: i64, policy: &Policy) -> Vec<String> {
    let mut order: Vec<&Item> = items.iter().collect();
    order.sort_by(|a, b| (b.at, &b.id).cmp(&(a.at, &a.id)));
    let mut keep: BTreeSet<&str> = BTreeSet::new();
    let mut recent = 0;
    let mut days = BTreeSet::new();
    let mut weeks = BTreeSet::new();
    for (n, i) in order.iter().enumerate() {
        let age = (now - i.at).max(0);
        let kept = i.pinned || {
            let by_age = match age {
                a if a <= policy.recent => {
                    recent += 1;
                    recent <= policy.recent_max
                }
                a if a <= policy.daily_until => days.insert(i.at.div_euclid(DAY)),
                a if a <= policy.weekly_until => weeks.insert(i.at.div_euclid(WEEK)),
                _ => false,
            };
            by_age || n == 0
        };
        if kept {
            keep.insert(&i.id);
        }
    }
    loop {
        let kept: Vec<&Item> = order
            .iter()
            .copied()
            .filter(|i| keep.contains(i.id.as_str()))
            .collect();
        if bytes(kept.iter().copied()) <= policy.cap_bytes {
            break;
        }
        let Some(oldest) = kept.iter().rev().find(|i| !i.pinned && i.id != order[0].id) else {
            break;
        };
        keep.remove(oldest.id.as_str());
    }
    order
        .iter()
        .rev()
        .filter(|i| !keep.contains(i.id.as_str()))
        .map(|i| i.id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_793_404_800;

    fn item(id: &str, age: i64, pinned: bool, sha: &str, size: u64) -> Item {
        Item {
            id: id.into(),
            at: NOW - age,
            pinned,
            blobs: vec![(sha.into(), size)],
        }
    }

    #[test]
    fn keeps_recent_then_daily_then_weekly() {
        let mut items = Vec::new();
        for h in 0..45 {
            items.push(item(&format!("r{h:02}"), h * HOUR, false, &format!("r{h}"), 1));
        }
        for d in 3..=14 {
            items.push(item(
                &format!("d{d:02}a"),
                d * DAY + HOUR,
                false,
                &format!("da{d}"),
                1,
            ));
            items.push(item(
                &format!("d{d:02}b"),
                d * DAY + 2 * HOUR,
                false,
                &format!("db{d}"),
                1,
            ));
        }
        for w in 3..=12 {
            items.push(item(&format!("w{w:02}"), w * WEEK, false, &format!("w{w}"), 1));
        }
        items.push(item("pinned-old", 400 * DAY, true, "p", 1));
        let gone = plan(&items, NOW, &Policy::default());
        for h in 0..40 {
            assert!(!gone.contains(&format!("r{h:02}")), "r{h:02}");
        }
        for h in 40..45 {
            assert!(gone.contains(&format!("r{h:02}")), "r{h:02}");
        }
        for d in 3..=14 {
            assert!(!gone.contains(&format!("d{d:02}a")), "d{d:02}a");
            assert!(gone.contains(&format!("d{d:02}b")), "d{d:02}b");
        }
        for w in 3..=10 {
            assert!(!gone.contains(&format!("w{w:02}")), "w{w:02}");
        }
        assert!(gone.contains(&"w11".to_owned()));
        assert!(gone.contains(&"w12".to_owned()));
        assert!(!gone.contains(&"pinned-old".to_owned()));
        let at = |id: &String| items.iter().find(|i| &i.id == id).unwrap().at;
        assert!(gone.windows(2).all(|w| at(&w[0]) <= at(&w[1])), "oldest first");
    }

    #[test]
    fn the_cap_drops_the_oldest_unpinned_first() {
        let items = vec![
            item("newest", 0, false, "a", 400),
            item("pinned", HOUR, true, "b", 400),
            item("middle", 2 * HOUR, false, "c", 400),
            item("oldest", 3 * HOUR, false, "d", 400),
        ];
        let policy = Policy {
            cap_bytes: 1000,
            ..Policy::default()
        };
        assert_eq!(plan(&items, NOW, &policy), ["oldest", "middle"]);
        let tiny = Policy {
            cap_bytes: 1,
            ..Policy::default()
        };
        assert_eq!(
            plan(&items, NOW, &tiny),
            ["oldest", "middle"],
            "the newest and pins stay"
        );
        let shared = vec![
            item("newest", 0, false, "same", 900),
            item("older", HOUR, false, "same", 900),
        ];
        assert!(
            plan(&shared, NOW, &policy).is_empty(),
            "a shared blob counts once"
        );
        assert_eq!(Policy::with_cap_mb(500).cap_bytes, 500 * 1024 * 1024);
    }
}
