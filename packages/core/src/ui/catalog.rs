use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

use serde::Deserialize;
use swofty_formats::pkg::{Node, Pkg, Val, Value};

use super::idf;

const STOCK_JSON: &str = include_str!("../../../../tools/uicatalog/catalog.json");
const ENGINE_JSON: &str = include_str!("../../../../re/db/ui/loader_rules.json");

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Engine {
    pub schema: u32,
    pub build: String,
    pub error_codes: BTreeMap<String, String>,
    pub rules: Vec<EngineRule>,
    pub silently_ignored: Vec<String>,
    pub element_base_props: Vec<String>,
    pub classes: BTreeMap<String, Class>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct EngineRule {
    pub id: String,
    pub level: String,
    pub check: String,
    pub fail: Fail,
    #[serde(rename = "fn")]
    pub func: String,
    pub ev: String,
    pub applies_to: AppliesTo,
    pub props: Vec<PropRule>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct AppliesTo {
    pub classes: Vec<String>,
    pub behaviours: Vec<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct PropRule {
    pub key: String,
    pub required: bool,
    pub shape: String,
    pub values: Vec<String>,
    pub case_insensitive: bool,
}

impl AppliesTo {
    pub fn matches(&self, class: &str, behaviours: &[String]) -> bool {
        (self.classes.is_empty() || self.classes.iter().any(|c| c == class))
            && (self.behaviours.is_empty() || self.behaviours.iter().any(|b| behaviours.contains(b)))
            && !(self.classes.is_empty() && self.behaviours.is_empty())
    }
}

impl PropRule {
    pub fn problem(&self, prop: Option<&Node>) -> Option<String> {
        let key = &self.key;
        let Some(v) = prop else {
            return self.required.then(|| format!("{key} is missing"));
        };
        let obj = v.as_obj();
        match self.shape.as_str() {
            "object" if obj.is_none() => return Some(format!("{key} is not an object")),
            "value" if obj.and_then(|o| o.get("value")).is_none() => {
                return Some(format!("{key} is not a {{ value }} object"));
            }
            "binding" => {
                let db = obj
                    .and_then(|o| o.get("dbentry"))
                    .and_then(Node::as_val)
                    .and_then(Val::as_str);
                if db.is_none_or(str::is_empty) {
                    return Some(format!(
                        "{key} must be {{ value, dbentry }} with a non-empty dbentry"
                    ));
                }
            }
            _ => {}
        }
        if self.values.is_empty() {
            return None;
        }
        let value = obj
            .and_then(|o| o.get("value"))
            .and_then(Node::as_val)
            .and_then(Val::as_str)
            .unwrap_or_default();
        let ok = self.values.iter().any(|w| {
            if self.case_insensitive {
                w.eq_ignore_ascii_case(value)
            } else {
                w == value
            }
        });
        (!ok).then(|| format!("{key} {value:?} is not one of {}", self.values.join(", ")))
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct Fail {
    pub code: serde_json::Value,
    pub scope: String,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct Class {
    pub registry: String,
    pub init: String,
    pub deliver_pin: String,
    pub init_calls_element_base: bool,
    pub init_calls_list_base: bool,
    pub props_read_by_init: Vec<String>,
    pub native_pins: Vec<String>,
}

impl Engine {
    pub fn rule(&self, id: &str) -> Option<&EngineRule> {
        self.rules.iter().find(|r| r.id == id)
    }
}

impl EngineRule {
    pub fn code(&self) -> String {
        match &self.fail.code {
            serde_json::Value::Number(n) => match n.as_u64() {
                Some(v) if v > 9 => format!("{v:#x}"),
                Some(v) => v.to_string(),
                None => n.to_string(),
            },
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        }
    }

    pub fn cite(&self) -> String {
        format!(
            "{} {}: rc {}, takes down the {}; {}",
            self.id,
            self.level,
            self.code(),
            self.fail.scope,
            self.func
        )
    }
}

pub fn engine() -> &'static Engine {
    static ENGINE: OnceLock<Engine> = OnceLock::new();
    ENGINE.get_or_init(|| serde_json::from_str(ENGINE_JSON).expect("re/db/ui/loader_rules.json parses"))
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Catalog {
    pub node_types: BTreeMap<String, NodeType>,
    pub scene_elements: BTreeMap<String, ElementType>,
    pub root_names: BTreeMap<String, BTreeMap<String, Vec<String>>>,
    pub scene_ids: Vec<String>,
}

impl Catalog {
    pub fn has_scene(&self, id: u64) -> bool {
        self.scene_ids.binary_search(&format!("{id:016x}")).is_ok()
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct NodeType {
    pub count: u64,
    pub roles: BTreeMap<String, u64>,
    pub props: BTreeMap<String, Prop>,
    pub commands: BTreeMap<String, Named>,
    pub events: BTreeMap<String, Named>,
    pub pins_in: BTreeMap<String, Pin>,
    pub pins_out: BTreeMap<String, u64>,
    pub links: BTreeMap<String, u64>,
    pub children: BTreeMap<String, u64>,
    pub parents: BTreeMap<String, u64>,
    pub layout: BTreeMap<String, u64>,
    pub layout_elements: BTreeMap<String, u64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Prop {
    pub count: u64,
    pub shapes: BTreeMap<String, u64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Named {
    pub count: u64,
    pub no_signals: u64,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Pin {
    pub count: u64,
    pub attributes: BTreeMap<String, u64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct ElementType {
    pub count: u64,
    pub params: BTreeMap<String, Param>,
    pub parents: BTreeMap<String, u64>,
    pub children: BTreeMap<String, u64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Param {
    pub count: u64,
    pub types: BTreeMap<String, u64>,
}

pub fn stock() -> &'static Catalog {
    static STOCK: OnceLock<Catalog> = OnceLock::new();
    STOCK.get_or_init(|| serde_json::from_str(STOCK_JSON).expect("tools/uicatalog/catalog.json parses"))
}

pub fn tag(ty: [u8; 4]) -> String {
    String::from_utf8_lossy(&ty).into_owned()
}

pub fn shape(n: &Node) -> String {
    match n {
        Node::Val(v) => format!("={}", tag(v.ty)),
        Node::Arr(_) => "arr".into(),
        Node::Iar(_) => "Iar".into(),
        Node::Obj(o) => {
            let mut keys: Vec<&str> = o.keys().collect();
            match o.get("value") {
                Some(inner) => {
                    let base = match inner {
                        Node::Val(v) => tag(v.ty),
                        other => shape(other),
                    };
                    let mut extra: Vec<&str> = keys.into_iter().filter(|k| *k != "value").collect();
                    extra.sort_unstable();
                    if extra.is_empty() {
                        base
                    } else {
                        format!("{base}+{}", extra.join("+"))
                    }
                }
                None => {
                    keys.sort_unstable();
                    format!("obj{{{}}}", keys.join(","))
                }
            }
        }
    }
}

fn attr_paths(n: &Node, pre: &str, out: &mut Vec<String>) {
    match n {
        Node::Val(v) => out.push(format!("{pre}:{}", tag(v.ty))),
        Node::Obj(o) => {
            for (k, v) in &o.items {
                let key = k.as_str().unwrap_or_default();
                let p = if pre.is_empty() {
                    key.to_owned()
                } else {
                    format!("{pre}.{key}")
                };
                attr_paths(v, &p, out);
            }
        }
        Node::Arr(_) => out.push(format!("{pre}[]")),
        Node::Iar(_) => out.push(pre.to_owned()),
    }
}

pub fn attr_shape(n: &Node) -> String {
    let mut out = Vec::new();
    attr_paths(n, "", &mut out);
    out.sort_unstable();
    out.join(" ")
}

pub fn shnm(v: &Val) -> String {
    match &v.value {
        Value::Str(s) => s.clone(),
        Value::Raw(b) => {
            let end = b.iter().rposition(|c| *c != 0).map_or(0, |i| i + 1);
            String::from_utf8_lossy(&b[..end]).into_owned()
        }
    }
}

pub fn element_type(e: &Node) -> String {
    e.as_obj()
        .and_then(|o| o.get("typeName"))
        .and_then(Node::as_val)
        .map(shnm)
        .unwrap_or_else(|| "?".into())
}

#[derive(Debug, Default)]
pub struct Scene {
    pub elements: Vec<(String, String, Option<u64>, String)>,
    pub by_name: HashMap<String, Vec<usize>>,
    pub xrefs: Vec<u64>,
    pub lookup: Vec<u32>,
}

#[derive(Debug, Default)]
pub struct Scenes {
    pub scenes: HashMap<u64, Scene>,
}

type Hit<'a> = (&'a str, Option<u64>, &'a str, u64);

impl Scenes {
    pub fn add(&mut self, data: &[u8]) -> crate::Result<()> {
        let p = Pkg::parse(data)?;
        for entry in idf::scenes(&p)? {
            let (Some(sid), Some(root)) = (
                idf::scene_name(entry),
                entry.as_obj().and_then(|o| o.get("scene")),
            ) else {
                continue;
            };
            let mut sc = Scene::default();
            for (i, (path, e)) in idf::elements(root).into_iter().enumerate() {
                let name = idf::elem_name(e).to_owned();
                let xref = idf::xref(e);
                sc.by_name
                    .entry(name.clone())
                    .or_default()
                    .push(sc.elements.len());
                sc.xrefs.extend(xref);
                sc.elements.push((
                    name,
                    element_type(e),
                    xref,
                    if i == 0 { String::new() } else { path },
                ));
            }
            sc.lookup = entry
                .as_obj()
                .and_then(|o| o.get("nodeidlookup"))
                .and_then(Node::as_arr)
                .map(|a| {
                    a.items
                        .iter()
                        .filter_map(|v| v.as_val().and_then(Val::as_u32))
                        .collect()
                })
                .unwrap_or_default();
            self.scenes.insert(sid, sc);
        }
        Ok(())
    }

    fn hits(&self, sid: u64, name: &str, prefix: &str) -> Vec<Hit<'_>> {
        let Some(sc) = self.scenes.get(&sid) else {
            return Vec::new();
        };
        sc.by_name
            .get(name)
            .into_iter()
            .flatten()
            .map(|i| &sc.elements[*i])
            .filter(|(_, _, _, path)| path.starts_with(prefix))
            .map(|(_, ty, x, path)| (ty.as_str(), *x, path.as_str(), sid))
            .collect()
    }

    fn lookup(&self, name: &str, sid: u64, prefix: &str) -> Vec<Hit<'_>> {
        let hits = self.hits(sid, name, prefix);
        if !hits.is_empty() || !prefix.is_empty() {
            return hits;
        }
        let mut seen = Vec::new();
        let mut queue = std::collections::VecDeque::from([sid]);
        while let Some(s) = queue.pop_front() {
            if seen.contains(&s) {
                continue;
            }
            seen.push(s);
            let Some(sc) = self.scenes.get(&s) else { continue };
            let hits = self.hits(s, name, "");
            if !hits.is_empty() {
                return hits;
            }
            queue.extend(sc.xrefs.iter().copied());
        }
        Vec::new()
    }

    pub fn resolve_ids(&self, ids: &[u32], sid: u64) -> bool {
        let Some((first, rest)) = ids.split_first() else {
            return true;
        };
        let Some(sc) = self.scenes.get(&sid) else {
            return false;
        };
        let hits: Vec<&(String, String, Option<u64>, String)> = sc
            .elements
            .iter()
            .filter(|(name, ..)| swofty_formats::hash::djb2(name) == *first)
            .collect();
        if hits.is_empty() {
            return sc.xrefs.iter().any(|x| self.resolve_ids(ids, *x));
        }
        if rest.is_empty() {
            return true;
        }
        hits.iter()
            .any(|(_, _, x, _)| x.is_some_and(|x| self.resolve_ids(rest, x)))
    }

    pub fn resolve(&self, names: &[&str], sid: u64) -> Result<(Vec<String>, Option<u64>), String> {
        self.resolve_in(names, sid, "")
    }

    fn resolve_in(
        &self,
        names: &[&str],
        sid: u64,
        prefix: &str,
    ) -> Result<(Vec<String>, Option<u64>), String> {
        let Some((first, rest)) = names.split_first() else {
            return Err("empty layout path".into());
        };
        if !self.scenes.contains_key(&sid) {
            return Err(format!("scene {sid:#018x} is not loaded"));
        }
        let mut hits = self.lookup(first, sid, prefix);
        if hits.is_empty() {
            return Err(format!("no element {first}"));
        }
        if rest.is_empty() {
            return Ok((vec![hits[0].0.to_owned()], hits[0].1));
        }
        hits.sort_by_key(|h| h.1.is_none());
        let mut first_err = None;
        for (ty, xref, path, s) in hits {
            let next = match xref {
                Some(x) => self.resolve_in(rest, x, ""),
                None => self.resolve_in(rest, s, &format!("{path}.")),
            };
            match next {
                Ok((mut chain, last)) => {
                    chain.insert(0, ty.to_owned());
                    return Ok((chain, last));
                }
                Err(e) => {
                    first_err.get_or_insert(e);
                }
            }
        }
        Err(first_err.unwrap_or_default())
    }
}
