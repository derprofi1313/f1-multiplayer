use std::collections::HashMap;

use super::catalog::plain;
use super::driver_manager::{self, Driver};
use super::schema::schema;
use super::{Catalog, Edits, LinkSpec, NodeId, NodeKind, modules};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HitKind {
    /// A row of a friendly module (a driver, team, track).
    ModuleRow,
    /// A driver Driver Manager adds when the profile builds.
    Added,
    Table,
    Row,
    Key,
    /// A key whose value matched.
    Value,
    /// A key whose English text matched.
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Node(NodeId),
    ModuleRow {
        module: String,
        scope: Option<String>,
        label: String,
    },
    /// An addon's page, where what the hit names is edited.
    Link(LinkSpec),
}

#[derive(Debug, Clone)]
pub struct Hit {
    pub kind: HitKind,
    pub target: Target,
    pub title: String,
    pub detail: String,
    pub score: u32,
}

struct ModuleEntry {
    module: usize,
    scope: Option<String>,
    label: String,
    name: String,
    hay: String,
}

/// Lowercase haystacks, built once per catalog on the first search.
pub(crate) struct Index {
    modules: Vec<ModuleEntry>,
    tables: Vec<(String, NodeId)>,
    rows: Vec<(String, NodeId)>,
    names: Vec<(String, Vec<NodeId>)>,
    values: Vec<(String, Vec<u32>)>,
    meanings: Vec<(String, NodeId)>,
    texts: Vec<(String, String, NodeId)>,
}

const PER_GROUP: usize = 40;

fn quality(hay: &str, q: &str) -> u32 {
    let word_start = |i: usize| i == 0 || hay[..i].ends_with([' ', '_', '-', '.']);
    match hay.find(q) {
        Some(_) if hay == q => 3,
        Some(i) if word_start(i) || hay.match_indices(q).any(|(i, _)| word_start(i)) => 2,
        Some(_) => 1,
        None => 0,
    }
}

fn short(v: &str) -> String {
    match v.char_indices().nth(80) {
        Some((i, _)) => format!("{}…", &v[..i]),
        None => v.to_owned(),
    }
}

impl Index {
    fn build(cat: &Catalog) -> Self {
        let edits = Edits::default();
        let mut index = Index {
            modules: Vec::new(),
            tables: Vec::new(),
            rows: Vec::new(),
            names: Vec::new(),
            values: Vec::new(),
            meanings: Vec::new(),
            texts: Vec::new(),
        };
        for (i, m) in modules().iter().enumerate() {
            let Ok(view) = m.view(cat, &edits, None) else {
                continue;
            };
            for r in view.rows {
                index.modules.push(ModuleEntry {
                    module: i,
                    scope: view.scope.clone(),
                    hay: format!("{}\n{}", r.name.to_lowercase(), r.label.to_lowercase()),
                    label: r.label,
                    name: r.name,
                });
            }
        }
        let mut names: HashMap<String, Vec<NodeId>> = HashMap::new();
        for id in 0..cat.len() as NodeId {
            let n = cat.node(id);
            match &n.kind {
                NodeKind::Table { word, .. } => {
                    let meaning = cat.table_meaning(id).unwrap_or_default();
                    index
                        .tables
                        .push((format!("{}\n{}", word.to_lowercase(), meaning.to_lowercase()), id));
                }
                NodeKind::Row { .. } => {
                    let hay = [n.label.as_deref(), n.display.as_deref()]
                        .into_iter()
                        .flatten()
                        .map(str::to_lowercase)
                        .collect::<Vec<_>>()
                        .join("\n");
                    if !hay.is_empty() {
                        index.rows.push((hay, id));
                    }
                }
                NodeKind::Key if !n.name.starts_with("0x") => {
                    names.entry(n.name.to_lowercase()).or_default().push(id);
                }
                _ => {}
            }
        }
        index.names = names.into_iter().collect();
        let mut values: HashMap<&str, Vec<u32>> = HashMap::new();
        let mut meta_node: HashMap<u32, NodeId> = HashMap::new();
        for i in 0..cat.records() as u32 {
            let r = cat.record(i);
            if matches!(r.tag, 2 | 3 | 4 | 13) && !r.value.is_empty() {
                values.entry(r.value).or_default().push(i);
            }
            if let Some(m) = cat.record_meta(i) {
                meta_node.entry(m).or_insert(cat.record_node(i));
            }
        }
        for (k, text) in cat.lang_entries() {
            if let Some(recs) = values.get(k) {
                let text = plain(text);
                index
                    .texts
                    .push((text.to_lowercase(), text, cat.record_node(recs[0])));
            }
        }
        index.values = values
            .into_iter()
            .map(|(v, recs)| (v.to_lowercase(), recs))
            .collect();
        let metas = &schema().metas;
        index.meanings = meta_node
            .into_iter()
            .filter(|(m, _)| !metas[*m as usize].meaning.is_empty())
            .map(|(m, node)| (metas[m as usize].meaning.to_lowercase(), node))
            .collect();
        index
    }
}

impl Catalog {
    /// Case-insensitive search over module rows, tables, rows, key paths, values,
    /// meanings and English text; best first.
    pub fn search(&self, query: &str, limit: usize) -> Vec<Hit> {
        self.search_with(query, limit, &[])
    }

    /// Like `search`, also over the names, codes and labels of the drivers Driver Manager adds.
    pub fn search_with(&self, query: &str, limit: usize, drivers: &[Driver]) -> Vec<Hit> {
        let q = query.trim().to_lowercase();
        if q.is_empty() || limit == 0 {
            return Vec::new();
        }
        let ix = self.search.get_or_init(|| Index::build(self));
        let mut hits = Vec::new();
        let mut group = |kind: HitKind,
                         scored: &mut Vec<(u32, usize)>,
                         make: &dyn Fn(usize) -> Vec<(String, String, Target)>| {
            scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
            let bonus = if kind == HitKind::ModuleRow {
                100
            } else {
                6 - kind as u32
            };
            for &(qual, i) in scored.iter().take(PER_GROUP) {
                for (title, detail, target) in make(i).into_iter().take(PER_GROUP) {
                    hits.push(Hit {
                        kind,
                        target,
                        title,
                        detail,
                        score: qual * 10 + bonus,
                    });
                }
            }
        };
        let score_all = |hays: &mut dyn Iterator<Item = &str>| -> Vec<(u32, usize)> {
            hays.enumerate()
                .filter_map(|(i, h)| {
                    let best = h.split('\n').map(|p| quality(p, &q)).max().unwrap_or(0);
                    (best > 0).then_some((best, i))
                })
                .collect()
        };
        let mods = modules();
        let mut s = score_all(&mut ix.modules.iter().map(|e| e.hay.as_str()));
        group(HitKind::ModuleRow, &mut s, &|i| {
            let e = &ix.modules[i];
            let m = &mods[e.module];
            vec![(
                e.name.clone(),
                format!("{} · {}", m.title, e.label),
                Target::ModuleRow {
                    module: m.id.clone(),
                    scope: e.scope.clone(),
                    label: e.label.clone(),
                },
            )]
        });
        let mut s = score_all(&mut ix.tables.iter().map(|e| e.0.as_str()));
        group(HitKind::Table, &mut s, &|i| {
            let id = ix.tables[i].1;
            let detail = self.table_meaning(id).unwrap_or_default().to_owned();
            vec![(self.path(id), detail, Target::Node(id))]
        });
        let mut s = score_all(&mut ix.rows.iter().map(|e| e.0.as_str()));
        group(HitKind::Row, &mut s, &|i| {
            let id = ix.rows[i].1;
            let n = self.node(id);
            let title = n.display.clone().or(n.label.clone()).unwrap_or_default();
            vec![(title, self.path(id), Target::Node(id))]
        });
        let key_hit = |id: NodeId| {
            let value = self
                .node(id)
                .record
                .map(|r| short(self.record(r).value))
                .unwrap_or_default();
            (self.path(id), value, Target::Node(id))
        };
        let mut s = score_all(&mut ix.names.iter().map(|e| e.0.as_str()));
        group(HitKind::Key, &mut s, &|i| {
            ix.names[i]
                .1
                .iter()
                .take(PER_GROUP)
                .map(|&id| key_hit(id))
                .collect()
        });
        let mut s = score_all(&mut ix.meanings.iter().map(|e| e.0.as_str()));
        group(HitKind::Key, &mut s, &|i| vec![key_hit(ix.meanings[i].1)]);
        let mut s = score_all(&mut ix.values.iter().map(|e| e.0.as_str()));
        group(HitKind::Value, &mut s, &|i| {
            ix.values[i]
                .1
                .iter()
                .take(PER_GROUP)
                .map(|&r| {
                    let id = self.record_node(r);
                    (short(self.record(r).value), self.path(id), Target::Node(id))
                })
                .collect()
        });
        let mut s = score_all(&mut ix.texts.iter().map(|e| e.0.as_str()));
        group(HitKind::Text, &mut s, &|i| {
            let (_, text, id) = &ix.texts[i];
            vec![(short(text), self.path(*id), Target::Node(*id))]
        });
        for d in drivers {
            let best = [d.name.to_lowercase(), d.code.to_lowercase(), d.label.clone()]
                .iter()
                .map(|h| quality(h, &q))
                .max()
                .unwrap_or(0);
            if best > 0 {
                hits.push(Hit {
                    kind: HitKind::Added,
                    target: Target::Link(driver_manager::link()),
                    title: d.title(),
                    detail: format!("{} · {} · #{}", d.label, d.code, d.number),
                    score: best * 10 + 100,
                });
            }
        }
        hits.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then(a.kind.cmp(&b.kind))
                .then(a.title.len().cmp(&b.title.len()))
        });
        hits.truncate(limit);
        hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_rows_tables_keys_and_values() {
        let cat = Catalog::load(&crate::dbedit::catalog::tests::sample()).unwrap();
        let hits = cat.search("Lando", 20);
        assert!(
            hits.iter()
                .any(|h| h.kind == HitKind::Row && h.title == "lando_norris"),
            "{hits:?}"
        );
        let hits = cat.search("car_number", 20);
        assert!(
            hits.iter()
                .any(|h| h.kind == HitKind::Key && h.title.ends_with("car_number")),
            "{hits:?}"
        );
        let hits = cat.search("driver", 5);
        assert!(hits.iter().any(|h| h.kind == HitKind::Table), "{hits:?}");
        assert!(cat.search("  ", 5).is_empty());
    }

    #[test]
    fn finds_driver_manager_drivers_by_name_code_and_label() {
        use crate::dbedit::driver_manager::tests;
        let cat = Catalog::from_gvdb(tests::database(), String::new());
        let dir = tests::temp("search");
        tests::fixture(&dir);
        let drivers = driver_manager::drivers_in(&dir, &cat);
        for q in ["swofty", "Jacob Sw", "SWO", "dm_208"] {
            let hits = cat.search_with(q, 20, &drivers);
            let h = hits
                .iter()
                .find(|h| h.kind == HitKind::Added)
                .unwrap_or_else(|| panic!("{q}: {hits:?}"));
            assert_eq!(h.title, "Jacob Swofty (Driver Manager)");
            assert_eq!(h.detail, "dm_208 · SWO · #21");
            assert_eq!(h.target, Target::Link(driver_manager::link()));
        }
        let hits = cat.search_with("dm_", 20, &drivers);
        let found: Vec<&str> = hits
            .iter()
            .filter(|h| h.kind == HitKind::Added)
            .map(|h| h.title.as_str())
            .collect();
        assert_eq!(
            found,
            [
                "Jacob Swofty (Driver Manager)",
                "Ret Ired (Driver Manager, retired)"
            ]
        );
        for left_out in ["dm_210", "Bad Code", "NOH", "Used Look"] {
            let hits = cat.search_with(left_out, 20, &drivers);
            assert!(
                !hits.iter().any(|h| h.kind == HitKind::Added),
                "{left_out}: {hits:?}"
            );
        }
        assert!(!cat.search("swofty", 20).iter().any(|h| h.kind == HitKind::Added));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn searches_the_real_database_quickly() {
        let Some(cat) = crate::dbedit::catalog::tests::real() else {
            eprintln!("F1_25_DIR is not set; skipping");
            return;
        };
        let t = std::time::Instant::now();
        cat.search("x", 1);
        eprintln!("index built in {:?}", t.elapsed());
        for q in ["verstappen", "monaco", "pit lane", "car_number", "a"] {
            let t = std::time::Instant::now();
            let hits = cat.search(q, 30);
            eprintln!(
                "{q}: {} hits in {:?}; first {:?}",
                hits.len(),
                t.elapsed(),
                hits.first().map(|h| (&h.kind, &h.title))
            );
        }
        let hits = cat.search("verstappen", 30);
        assert_eq!(hits[0].kind, HitKind::ModuleRow, "{hits:?}");
    }
}
