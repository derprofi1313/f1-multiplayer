//! The embedded gamedb catalogue (`tools/gamedb/catalog`).

use std::collections::HashMap;
use std::io::Read;
use std::sync::OnceLock;

use super::FieldMeta;

const KEYS_GZ: &[u8] = include_bytes!("../../../../tools/gamedb/catalog/keys.tsv.gz");
const TABLES: &str = include_str!("../../../../tools/gamedb/catalog/tables.tsv");

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Part {
    Name(String),
    Table(String),
    HashTable(u64),
    HashObject(u64),
}

/// A trie node over the key patterns; `leaf` is the schema key ending here.
#[derive(Debug, Default)]
pub(crate) struct SNode {
    pub part: Option<Part>,
    pub children: Vec<usize>,
    pub leaf: Option<u32>,
    /// Tables: index into `Schema::families`.
    pub family: Option<u32>,
}

#[derive(Debug, Default)]
pub(crate) struct Family {
    pub pattern: String,
    pub meaning: String,
    /// Relative field paths in catalogue order, `_metadata_` first.
    pub fields: Vec<String>,
}

#[derive(Debug, Default)]
pub(crate) struct Schema {
    pub metas: Vec<FieldMeta>,
    pub nodes: Vec<SNode>,
    pub families: Vec<Family>,
}

pub(crate) fn schema() -> &'static Schema {
    static SCHEMA: OnceLock<Schema> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        let mut text = String::new();
        flate2::read::GzDecoder::new(KEYS_GZ)
            .read_to_string(&mut text)
            .expect("the embedded catalogue is gzip text");
        Schema::parse(&text, TABLES, module_patterns())
    })
}

pub(crate) fn part(s: &str) -> Part {
    if let Some(inner) = s.strip_prefix('{')
        && let Some((hex, rest)) = inner.split_once('}')
        && let Ok(h) = u64::from_str_radix(hex, 16)
    {
        match rest {
            "N" => return Part::HashTable(h),
            "" => return Part::HashObject(h),
            _ => {}
        }
    }
    match s.strip_suffix("_N") {
        Some(word) if !word.is_empty() => Part::Table(word.to_owned()),
        _ => Part::Name(s.to_owned()),
    }
}

impl Schema {
    pub(crate) fn parse(keys: &str, tables: &str, extra: Vec<(String, FieldMeta)>) -> Self {
        let mut s = Schema {
            nodes: vec![SNode::default()],
            ..Self::default()
        };
        let mut family_of: HashMap<String, u32> = HashMap::new();
        let mut child_of: HashMap<(usize, String), usize> = HashMap::new();
        let listed = keys.lines().skip(1).filter_map(|line| {
            let c: Vec<&str> = line.split('\t').collect();
            (c.len() >= 13 && c[6] == "yes").then(|| {
                let meta = FieldMeta {
                    meaning: c[11].to_owned(),
                    unit: c[9].to_owned(),
                    range: c[8].to_owned(),
                    example: c[10].to_owned(),
                    evidence: c[12].to_owned(),
                };
                (c[0].to_owned(), meta)
            })
        });
        for (pattern, meta) in listed.chain(extra) {
            let parts: Vec<&str> = pattern.split('.').collect();
            let mut at = 0;
            let mut last_table = None;
            for (i, p) in parts.iter().enumerate() {
                at = match child_of.get(&(at, p.to_string())) {
                    Some(&c) => c,
                    None => {
                        let id = s.nodes.len();
                        s.nodes.push(SNode {
                            part: Some(part(p)),
                            ..SNode::default()
                        });
                        s.nodes[at].children.push(id);
                        child_of.insert((at, p.to_string()), id);
                        id
                    }
                };
                if matches!(s.nodes[at].part, Some(Part::Table(_) | Part::HashTable(_))) {
                    let fam = parts[..=i].join(".");
                    let next = s.families.len() as u32;
                    let f = *family_of.entry(fam.clone()).or_insert(next);
                    if f == next {
                        s.families.push(Family {
                            pattern: fam,
                            ..Family::default()
                        });
                    }
                    s.nodes[at].family = Some(f);
                    last_table = Some((f, i));
                }
            }
            if s.nodes[at].leaf.is_some() {
                continue;
            }
            s.nodes[at].leaf = Some(s.metas.len() as u32);
            s.metas.push(meta);
            if let Some((f, i)) = last_table {
                let rel = parts[i + 1..].join(".");
                let fields = &mut s.families[f as usize].fields;
                if rel == "_metadata_" {
                    fields.insert(0, rel);
                } else {
                    fields.push(rel);
                }
            }
        }
        for line in tables.lines().skip(1) {
            let c: Vec<&str> = line.split('\t').collect();
            if c.len() > 12
                && let Some(&f) = family_of.get(c[0])
            {
                s.families[f as usize].meaning = c[12].to_owned();
            }
        }
        s
    }
}

/// The catalogue pattern of a selector path: `a.b[_metadata_=x].c_3` gives `a.b_N.c_N`.
pub(crate) fn pattern(sel: &str) -> Option<String> {
    let mut out = Vec::new();
    let mut rest = Some(sel);
    while let Some(p) = rest {
        let (seg, tail) = crate::ops::first_segment(p).ok()?;
        let indexed = seg
            .name
            .rsplit_once('_')
            .filter(|(w, n)| !w.is_empty() && !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()));
        out.push(match (seg.select, indexed) {
            (Some(_), _) if seg.name.starts_with('{') => format!("{}N", seg.name),
            (Some(_), _) => format!("{}_N", seg.name),
            (None, Some((word, _))) => format!("{word}_N"),
            (None, None) => seg.name.to_owned(),
        });
        rest = tail;
    }
    Some(out.join("."))
}

/// Patterns for the relative paths the modules read, so the tree names them too.
fn module_patterns() -> Vec<(String, FieldMeta)> {
    let mut out = Vec::new();
    let mut add = |rows: &str, rel: &str, label: &str| {
        if rel.contains('{') {
            return;
        }
        if let (Some(rows), Some(rel)) = (pattern(rows), pattern(rel)) {
            let meta = FieldMeta {
                meaning: label.to_owned(),
                evidence: "db editor module".into(),
                ..FieldMeta::default()
            };
            out.push((format!("{rows}_N.{rel}"), meta));
        }
    };
    let named = |n: &Option<super::NameSpec>| -> Vec<String> {
        let Some(n) = n else { return Vec::new() };
        let mut v: Vec<String> = n.field.iter().cloned().collect();
        if let Some(t) = &n.template {
            v.extend(
                t.split('{')
                    .skip(1)
                    .filter_map(|x| x.split_once('}'))
                    .map(|(f, _)| f.to_owned()),
            );
        }
        v
    };
    for m in super::modules() {
        for f in &m.fields {
            add(&m.rows, &f.path, &f.label);
        }
        add(&m.rows, &m.label, "Row label");
        for f in named(&m.name).iter().chain(m.filter.iter().map(|f| &f.field)) {
            add(&m.rows, f, "");
        }
        if let Some(sc) = &m.scope {
            add(&sc.rows, &sc.label, "Row label");
            for f in named(&sc.name).iter().chain(sc.filter.iter().map(|f| &f.field)) {
                add(&sc.rows, f, "");
            }
        }
        for r in m.fields.iter().filter_map(|f| f.refs.as_ref()) {
            add(&r.rows, &r.label, "Row label");
            for f in named(&r.name).iter().chain(r.filter.iter().map(|f| &f.field)) {
                add(&r.rows, f, "");
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_embedded_catalogue() {
        let s = schema();
        assert!(s.metas.len() > 16_000, "{}", s.metas.len());
        let f = s
            .families
            .iter()
            .find(|f| f.pattern == "entrants.drivers.driver_N")
            .unwrap();
        assert_eq!(f.fields[0], "_metadata_");
        assert!(
            f.fields
                .iter()
                .any(|x| x == "driver_strategy.safety_car_pit_min_lap")
        );
        assert!(!f.meaning.is_empty());
        assert_eq!(part("{0043bb1eadb80344}N"), Part::HashTable(0x0043bb1eadb80344));
        assert_eq!(part("{01eca4484267ca0c}"), Part::HashObject(0x01eca4484267ca0c));
        assert_eq!(part("Track_N"), Part::Table("Track".into()));
        assert_eq!(
            pattern("a.b[_metadata_={scope}].c_3.d").as_deref(),
            Some("a.b_N.c_N.d")
        );
        assert!(
            s.families.iter().any(|f| f.pattern == "entrants.drivers.driver_N"
                && f.fields.iter().any(|x| x == "date_of_birth.day"))
        );
    }
}
