use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::{Arc, Mutex, OnceLock};

use swofty_formats::gvdb::Gvdb;
use swofty_formats::hash::{fnv1a64, fnv1a64_from};

use super::schema::{Part, schema};
use crate::install::Game;
use crate::ops::first_segment;
use crate::store::Store;
use crate::{Error, Result, io_err};

/// The synthetic root child holding records no known name reaches.
pub const UNNAMED: &str = "Unnamed";

pub type NodeId = u32;

const NONE: u32 = u32::MAX;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Root,
    /// A path part holding keys, sub-objects or tables (`tracks`, `ai_vehicle_config`).
    Container,
    /// Synthetic: the rows `word_0..word_{rows-1}` of one table.
    Table {
        word: String,
        rows: u32,
    },
    Row {
        index: u32,
    },
    /// A record; `Node::record` is set.
    Key,
    Unnamed,
}

#[derive(Debug, Clone)]
pub struct Node {
    /// The path segment: `tracks`, `Track`, `Track_3`, `numberOfCorners`; `0x…` for unnamed records.
    pub name: String,
    pub parent: Option<NodeId>,
    pub kind: NodeKind,
    pub record: Option<u32>,
    /// Rows: the `_metadata_` value.
    pub label: Option<String>,
    /// Rows: an English name found through a `lng_*` field of the row.
    pub display: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct RecordView<'a> {
    pub index: u32,
    pub key: u64,
    pub tag: u8,
    pub value: &'a str,
}

/// Schema facts for one field of a table family, from the catalogue.
#[derive(Debug, Clone, Default)]
pub struct FieldMeta {
    pub meaning: String,
    pub unit: String,
    pub range: String,
    pub example: String,
    pub evidence: String,
}

/// Record keys are FNV hashes already; hashing them again only costs time.
#[derive(Default)]
pub(crate) struct IdHasher(u64);

impl Hasher for IdHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0 = fnv1a64_from(self.0, bytes);
    }

    fn write_u64(&mut self, n: u64) {
        self.0 = n;
    }
}

pub(crate) type IdMap<V> = HashMap<u64, V, BuildHasherDefault<IdHasher>>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Lookup {
    State,
    Table,
}

struct Pending {
    name: String,
    kind: NodeKind,
    hash: u64,
    lookup: Lookup,
    family: Option<u32>,
    node: Option<NodeId>,
}

pub struct Catalog {
    build: String,
    index: IdMap<u32>,
    keys: Vec<u64>,
    tags: Vec<u8>,
    spans: Vec<(u32, u32)>,
    values: String,
    nodes: Vec<Node>,
    hashes: Vec<u64>,
    child_start: Vec<u32>,
    child_list: Vec<NodeId>,
    rec_node: Vec<NodeId>,
    rec_meta: Vec<u32>,
    by_state: IdMap<NodeId>,
    tables_by: IdMap<NodeId>,
    table_family: HashMap<NodeId, u32>,
    dup_labels: HashSet<NodeId>,
    unnamed: NodeId,
    named: usize,
    lang: HashMap<String, String>,
    pub(crate) search: OnceLock<super::search::Index>,
}

fn child_hash(state: Option<u64>, name: &str) -> u64 {
    match state {
        Some(s) => fnv1a64_from(fnv1a64_from(s, b"."), name),
        None => fnv1a64(name),
    }
}

fn head_hash(name: &str) -> u64 {
    if let Some(inner) = name.strip_prefix('{')
        && let Some((hex, rest)) = inner.split_once('}')
        && let Ok(h) = u64::from_str_radix(hex, 16)
    {
        return fnv1a64_from(h, rest);
    }
    fnv1a64(name)
}

/// English text without its `{o:upper}…{/o}` markup.
pub(crate) fn plain(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find('{') {
        out.push_str(&rest[..i]);
        match rest[i..].find('}') {
            Some(j) => rest = &rest[i + j + 1..],
            None => {
                rest = &rest[i..];
                break;
            }
        }
    }
    out.push_str(rest);
    out.trim().to_owned()
}

/// `0x` and 16 hex digits.
pub(crate) fn parse_key(s: &str) -> Option<u64> {
    s.strip_prefix("0x")
        .filter(|h| h.len() == 16)
        .and_then(|h| u64::from_str_radix(h, 16).ok())
}

impl Catalog {
    pub fn load(db: &[u8]) -> Result<Self> {
        let gvdb = Gvdb::parse(db)?;
        Ok(Self::from_gvdb(
            gvdb,
            crate::baseline::sha256_bytes(db)[..16].to_owned(),
        ))
    }

    /// A parsed database; `build` is what [`Catalog::build_id`] returns.
    pub fn from_gvdb(gvdb: Gvdb, build: String) -> Self {
        let n = gvdb.records.len();
        let bytes: usize = gvdb.records.iter().map(|r| r.value.len()).sum();
        let mut cat = Catalog {
            build,
            index: IdMap::with_capacity_and_hasher(n, Default::default()),
            keys: Vec::with_capacity(n),
            tags: Vec::with_capacity(n),
            spans: Vec::with_capacity(n),
            values: String::with_capacity(bytes),
            nodes: Vec::with_capacity(n + n / 8),
            hashes: Vec::with_capacity(n + n / 8),
            child_start: Vec::new(),
            child_list: Vec::new(),
            rec_node: vec![NONE; n],
            rec_meta: vec![NONE; n],
            by_state: IdMap::default(),
            tables_by: IdMap::default(),
            table_family: HashMap::new(),
            dup_labels: HashSet::new(),
            unnamed: 0,
            named: 0,
            lang: HashMap::new(),
            search: OnceLock::new(),
        };
        for (i, r) in gvdb.records.into_iter().enumerate() {
            let start = cat.values.len() as u32;
            match String::from_utf8(r.value) {
                Ok(s) => cat.values.push_str(&s),
                Err(e) => cat.values.push_str(&String::from_utf8_lossy(e.as_bytes())),
            }
            cat.spans.push((start, cat.values.len() as u32));
            cat.keys.push(r.key);
            cat.tags.push(r.tag);
            cat.index.entry(r.key).or_insert(i as u32);
        }
        cat.push_node(
            Node {
                name: String::new(),
                parent: None,
                kind: NodeKind::Root,
                record: None,
                label: None,
                display: None,
            },
            0,
        );
        let sc = schema();
        let (named, hashed): (Vec<usize>, Vec<usize>) = sc.nodes[0]
            .children
            .iter()
            .partition(|&&c| matches!(sc.nodes[c].part, Some(Part::Name(_) | Part::Table(_))));
        let mut stack = Vec::new();
        for c in named.into_iter().chain(hashed) {
            cat.walk_one(c, None, &mut stack);
        }
        cat.unnamed = cat.push_node(
            Node {
                name: UNNAMED.into(),
                parent: Some(0),
                kind: NodeKind::Unnamed,
                record: None,
                label: None,
                display: None,
            },
            0,
        );
        let mut left: Vec<u32> = (0..n as u32)
            .filter(|&i| cat.rec_node[i as usize] == NONE)
            .collect();
        left.sort_by_key(|&i| cat.keys[i as usize]);
        for i in left {
            let key = cat.keys[i as usize];
            let id = cat.push_node(
                Node {
                    name: format!("0x{key:016x}"),
                    parent: Some(cat.unnamed),
                    kind: NodeKind::Key,
                    record: Some(i),
                    label: None,
                    display: None,
                },
                key,
            );
            cat.rec_node[i as usize] = id;
        }
        cat.link();
        cat.mark_duplicate_labels();
        cat
    }

    fn push_node(&mut self, node: Node, hash: u64) -> NodeId {
        self.nodes.push(node);
        self.hashes.push(hash);
        (self.nodes.len() - 1) as NodeId
    }

    pub(crate) fn value_of(&self, key: u64) -> Option<&str> {
        self.index.get(&key).map(|&i| self.value_at(i))
    }

    fn value_at(&self, i: u32) -> &str {
        let (a, b) = self.spans[i as usize];
        &self.values[a as usize..b as usize]
    }

    fn probe(&self, base: u64) -> u32 {
        let mut n = 0u32;
        while self
            .index
            .contains_key(&fnv1a64_from(fnv1a64_from(base, n.to_string()), b"._metadata_"))
        {
            n += 1;
        }
        n
    }

    fn walk(&mut self, s: usize, state: Option<u64>, stack: &mut Vec<Pending>) {
        let sc = schema();
        for &c in &sc.nodes[s].children {
            self.walk_one(c, state, stack);
        }
    }

    fn walk_one(&mut self, c: usize, state: Option<u64>, stack: &mut Vec<Pending>) {
        let sc = schema();
        let sn = &sc.nodes[c];
        match sn.part.as_ref().expect("only the root has no part") {
            Part::Name(name) => {
                let h = child_hash(state, name);
                if let Some(leaf) = sn.leaf {
                    self.claim(h, name, leaf, stack);
                }
                if !sn.children.is_empty() {
                    stack.push(Pending {
                        name: name.clone(),
                        kind: NodeKind::Container,
                        hash: h,
                        lookup: Lookup::State,
                        family: None,
                        node: None,
                    });
                    self.walk(c, Some(h), stack);
                    stack.pop();
                }
            }
            Part::Table(word) => {
                let base = child_hash(state, &format!("{word}_"));
                let count = self
                    .value_of(child_hash(state, &format!("_{word}_instancecount_")))
                    .and_then(|v| v.trim().parse::<u32>().ok())
                    .unwrap_or(0);
                let rows = count.max(self.probe(base));
                let key = child_hash(state, word);
                self.table(
                    c,
                    word.clone(),
                    (key, base),
                    rows,
                    |i| format!("{word}_{i}"),
                    stack,
                );
            }
            Part::HashTable(h) => {
                let rows = self.probe(*h);
                let word = format!("{{{h:016x}}}");
                self.table(c, word.clone(), (*h, *h), rows, |i| format!("{word}{i}"), stack);
            }
            Part::HashObject(h) => {
                stack.push(Pending {
                    name: format!("{{{h:016x}}}"),
                    kind: NodeKind::Container,
                    hash: *h,
                    lookup: Lookup::State,
                    family: None,
                    node: None,
                });
                self.walk(c, Some(*h), stack);
                stack.pop();
            }
        }
    }

    fn table(
        &mut self,
        c: usize,
        word: String,
        (key, base): (u64, u64),
        rows: u32,
        row_name: impl Fn(u32) -> String,
        stack: &mut Vec<Pending>,
    ) {
        stack.push(Pending {
            name: word.clone(),
            kind: NodeKind::Table { word, rows },
            hash: key,
            lookup: Lookup::Table,
            family: schema().nodes[c].family,
            node: None,
        });
        for i in 0..rows {
            let h = fnv1a64_from(base, i.to_string());
            stack.push(Pending {
                name: row_name(i),
                kind: NodeKind::Row { index: i },
                hash: h,
                lookup: Lookup::State,
                family: None,
                node: None,
            });
            self.walk(c, Some(h), stack);
            stack.pop();
        }
        stack.pop();
    }

    fn claim(&mut self, h: u64, name: &str, leaf: u32, stack: &mut [Pending]) {
        let Some(&rec) = self.index.get(&h) else {
            return;
        };
        if self.rec_node[rec as usize] != NONE {
            return;
        }
        let parent = self.materialise(stack);
        let id = self.push_node(
            Node {
                name: name.to_owned(),
                parent: Some(parent),
                kind: NodeKind::Key,
                record: Some(rec),
                label: None,
                display: None,
            },
            h,
        );
        self.rec_node[rec as usize] = id;
        self.rec_meta[rec as usize] = leaf;
        self.named += 1;
    }

    fn materialise(&mut self, stack: &mut [Pending]) -> NodeId {
        let mut parent = 0;
        for p in stack.iter_mut() {
            if let Some(id) = p.node {
                parent = id;
                continue;
            }
            let found = match p.lookup {
                Lookup::State => self.by_state.get(&p.hash),
                Lookup::Table => self.tables_by.get(&p.hash),
            };
            let id = match found {
                Some(&id) => id,
                None => {
                    let label = match p.kind {
                        NodeKind::Row { .. } => self
                            .value_of(fnv1a64_from(p.hash, b"._metadata_"))
                            .map(str::to_owned),
                        _ => None,
                    };
                    let id = self.push_node(
                        Node {
                            name: p.name.clone(),
                            parent: Some(parent),
                            kind: p.kind.clone(),
                            record: None,
                            label,
                            display: None,
                        },
                        p.hash,
                    );
                    match p.lookup {
                        Lookup::State => self.by_state.insert(p.hash, id),
                        Lookup::Table => self.tables_by.insert(p.hash, id),
                    };
                    if let Some(f) = p.family {
                        self.table_family.insert(id, f);
                    }
                    id
                }
            };
            p.node = Some(id);
            parent = id;
        }
        parent
    }

    fn rank(&self, id: NodeId) -> (u8, u64) {
        let n = &self.nodes[id as usize];
        match n.kind {
            NodeKind::Row { index } => (0, index as u64),
            NodeKind::Key if n.parent == Some(self.unnamed) => (0, self.hashes[id as usize]),
            NodeKind::Key => (1, 0),
            NodeKind::Container => (2, 0),
            NodeKind::Table { .. } => (3, 0),
            NodeKind::Root | NodeKind::Unnamed => (4, 0),
        }
    }

    fn link(&mut self) {
        let n = self.nodes.len();
        let mut start = vec![0u32; n + 1];
        for node in &self.nodes {
            if let Some(p) = node.parent {
                start[p as usize + 1] += 1;
            }
        }
        for i in 0..n {
            start[i + 1] += start[i];
        }
        let mut fill = start.clone();
        let mut list = vec![0; start[n] as usize];
        for (id, node) in self.nodes.iter().enumerate() {
            if let Some(p) = node.parent {
                list[fill[p as usize] as usize] = id as NodeId;
                fill[p as usize] += 1;
            }
        }
        for p in 0..n {
            let slice = &mut list[start[p] as usize..start[p + 1] as usize];
            if slice.len() > 1 {
                slice.sort_by(|&a, &b| {
                    self.rank(a)
                        .cmp(&self.rank(b))
                        .then_with(|| self.nodes[a as usize].name.cmp(&self.nodes[b as usize].name))
                });
            }
        }
        self.child_start = start;
        self.child_list = list;
    }

    fn mark_duplicate_labels(&mut self) {
        let tables: Vec<NodeId> = self.tables_by.values().copied().collect();
        for t in tables {
            let mut seen: HashMap<&str, NodeId> = HashMap::new();
            let mut dups = Vec::new();
            for &r in self.children(t) {
                if let Some(l) = self.nodes[r as usize].label.as_deref() {
                    match seen.get(l) {
                        Some(&first) => dups.extend([first, r]),
                        None => {
                            seen.insert(l, r);
                        }
                    }
                }
            }
            self.dup_labels.extend(dups);
        }
    }

    /// English text for `lng_*` keys, from `language_english.lng`.
    pub fn set_lang(&mut self, english: &[u8]) -> Result<()> {
        let lng = swofty_formats::lng::Lng::parse(english)?;
        self.lang = lng.entries.into_iter().map(|e| (e.key, e.value)).collect();
        let rows: Vec<NodeId> = self
            .by_state
            .values()
            .copied()
            .filter(|&id| matches!(self.nodes[id as usize].kind, NodeKind::Row { .. }))
            .collect();
        for row in rows {
            let display = self.display_of(row);
            self.nodes[row as usize].display = display;
        }
        self.search = OnceLock::new();
        Ok(())
    }

    fn display_of(&self, row: NodeId) -> Option<String> {
        let mut best: Option<(u8, &str)> = None;
        let (mut first, mut last) = (None, None);
        for &c in self.children(row) {
            let n = &self.nodes[c as usize];
            let Some(rec) = n.record else { continue };
            let v = self.value_at(rec);
            if !v.starts_with("lng_") {
                continue;
            }
            let Some(text) = self.text(v).filter(|t| !t.trim().is_empty()) else {
                continue;
            };
            let field = n.name.to_ascii_lowercase();
            let score = match field.as_str() {
                "full_name" => 0,
                "display_name" | "name" => 1,
                "first_name" => {
                    first = Some(text);
                    continue;
                }
                "last_name" => {
                    last = Some(text);
                    continue;
                }
                f if f.contains("name") && !f.contains("short") && !f.contains("audio") => 2,
                f if f.contains("title") || f.contains("label") => 3,
                f if !f.contains("desc") && text.chars().count() <= 40 => 5,
                _ => continue,
            };
            if best.is_none_or(|(s, _)| score < s) {
                best = Some((score, text));
            }
        }
        match (best, first, last) {
            (Some((s, t)), _, _) if s <= 1 => Some(plain(t)),
            (_, Some(f), Some(l)) => Some(format!("{} {}", plain(f), plain(l))),
            (Some((_, t)), _, _) => Some(plain(t)),
            _ => None,
        }
    }

    /// The stock database of `game` ([`super::stock_database`]),
    /// named, with English text; cached in-process per database hash.
    pub fn open(store: &Store, game: &Game) -> Result<Arc<Self>> {
        static CACHE: Mutex<Option<(String, Arc<Catalog>)>> = Mutex::new(None);
        let path = super::stock_database(store, game)?;
        let bytes = std::fs::read(&path).map_err(io_err(&path))?;
        let id = crate::baseline::sha256_bytes(&bytes)[..16].to_owned();
        let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((have, cat)) = cache.as_ref()
            && *have == id
        {
            return Ok(cat.clone());
        }
        let mut cat = Catalog::load(&bytes)?;
        if let Ok(p) = crate::baseline::original_lang(&store.root, &game.dir, &game.identity, "eng")
            && let Ok(english) = std::fs::read(&p)
        {
            cat.set_lang(&english)?;
        }
        let cat = Arc::new(cat);
        *cache = Some((id, cat.clone()));
        Ok(cat)
    }

    /// First 16 hex digits of the database's SHA-256.
    pub fn build_id(&self) -> &str {
        &self.build
    }

    pub fn records(&self) -> usize {
        self.keys.len()
    }

    pub fn named(&self) -> usize {
        self.named
    }

    pub fn root(&self) -> NodeId {
        0
    }

    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id as usize]
    }

    pub fn children(&self, id: NodeId) -> &[NodeId] {
        let (a, b) = (self.child_start[id as usize], self.child_start[id as usize + 1]);
        &self.child_list[a as usize..b as usize]
    }

    /// The number of nodes; ids run from 0 to `len() - 1`.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// The dotted path of a node: `tracks.Track_3.numberOfCorners`; a table
    /// node gives `tracks.Track`.
    pub fn path(&self, id: NodeId) -> String {
        let mut parts = Vec::new();
        let mut at = Some(id);
        while let Some(i) = at {
            let n = &self.nodes[i as usize];
            match n.kind {
                NodeKind::Root | NodeKind::Unnamed => {}
                NodeKind::Table { .. } if i != id => {}
                _ => parts.push(n.name.as_str()),
            }
            at = n.parent;
        }
        parts.reverse();
        parts.join(".")
    }

    /// The node at a dotted path, or at a selector path (`entrants.teams.team[_metadata_=ferrari_26].name`).
    pub fn find(&self, path: &str) -> Option<NodeId> {
        let path = path.trim();
        match path {
            "" => return Some(0),
            UNNAMED => return Some(self.unnamed),
            _ => {}
        }
        if let Some(k) = parse_key(path) {
            return self.node_of_key(k);
        }
        self.lookup(self.state(None, path).ok()?)
    }

    /// `rel` (dotted, may name a sub-object) under `from`.
    pub fn find_in(&self, from: NodeId, rel: &str) -> Option<NodeId> {
        self.lookup(self.state(self.base_of(from), rel).ok()?)
    }

    fn base_of(&self, id: NodeId) -> Option<u64> {
        match self.nodes[id as usize].kind {
            NodeKind::Root | NodeKind::Unnamed => None,
            NodeKind::Table { .. } => self.nodes[id as usize].parent.and_then(|p| self.base_of(p)),
            _ => Some(self.hashes[id as usize]),
        }
    }

    pub(crate) fn lookup(&self, h: u64) -> Option<NodeId> {
        if let Some(&rec) = self.index.get(&h) {
            return Some(self.rec_node[rec as usize]);
        }
        self.by_state.get(&h).or_else(|| self.tables_by.get(&h)).copied()
    }

    /// The hash a path names under `from`, rows picked through selectors.
    pub(crate) fn state(&self, from: Option<u64>, path: &str) -> std::result::Result<u64, String> {
        let mut state = from;
        let mut rest = Some(path);
        while let Some(p) = rest {
            let (seg, tail) = first_segment(p).map_err(|e| e.to_string())?;
            if seg.name.is_empty() {
                return Err(format!("{path}: empty segment"));
            }
            state = Some(match (state, seg.select) {
                (None, None) => head_hash(seg.name),
                (Some(s), None) => child_hash(Some(s), seg.name),
                (s, Some((field, want))) => {
                    let upto = &path[..path.len() - tail.map_or(0, |t| t.len() + 1)];
                    let rows = match s {
                        Some(s) => self.rows_of(s, seg.name),
                        None => match super::schema::part(seg.name) {
                            Part::HashObject(h) => self.scan(h),
                            _ => return Err(format!("{upto}: a selector needs a table path before it")),
                        },
                    };
                    let hits: Vec<u64> = rows
                        .into_iter()
                        .filter(|&r| {
                            self.index.contains_key(&fnv1a64_from(r, b"._metadata_"))
                                && self.state(Some(r), field).ok().and_then(|k| self.value_of(k))
                                    == Some(want)
                        })
                        .collect();
                    match hits.as_slice() {
                        [h] => *h,
                        [] => return Err(format!("{upto}: no row matches")),
                        many => return Err(format!("{upto}: {} rows match", many.len())),
                    }
                }
            });
            rest = tail;
        }
        state.ok_or_else(|| format!("bad path {path}"))
    }

    /// The row hashes of `table.word_N`: the instance count when there is one, else
    /// every row up to the first without `_metadata_`.
    pub(crate) fn rows_of(&self, table: u64, word: &str) -> Vec<u64> {
        let base = child_hash(Some(table), &format!("{word}_"));
        let count = self
            .value_of(child_hash(Some(table), &format!("_{word}_instancecount_")))
            .and_then(|v| v.trim().parse::<u32>().ok());
        match count {
            Some(n) => (0..n).map(|i| fnv1a64_from(base, i.to_string())).collect(),
            None => self.scan(base),
        }
    }

    /// For an `_<Word>_instancecount_` key: the container holding the table, and the word.
    pub(crate) fn instance_count(&self, key: u64) -> Option<(NodeId, &str)> {
        let node = &self.nodes[self.node_of_key(key)? as usize];
        let word = node.name.strip_prefix('_')?.strip_suffix("_instancecount_")?;
        Some((node.parent?, word))
    }

    /// The dotted path of the table `word` under `container`: `entrants.drivers.driver`.
    pub(crate) fn table_path(&self, container: NodeId, word: &str) -> String {
        match self.path(container) {
            p if p.is_empty() => word.to_owned(),
            p => format!("{p}.{word}"),
        }
    }

    /// Keys and paths of the fields of rows `rows` of the table `word` under
    /// `container`, from the table's schema family; for rows the stock database lacks.
    pub(crate) fn row_keys(
        &self,
        container: NodeId,
        word: &str,
        rows: std::ops::Range<u32>,
    ) -> Vec<(u64, String)> {
        let base = self.base_of(container);
        let Some(&table) = self.tables_by.get(&child_hash(base, word)) else {
            return Vec::new();
        };
        let (prefix, fields) = (self.table_path(container, word), self.fields(table));
        let row_base = child_hash(base, &format!("{word}_"));
        let mut out = Vec::new();
        for i in rows {
            let row = fnv1a64_from(row_base, i.to_string());
            for f in &fields {
                if let Ok(key) = self.state(Some(row), f) {
                    out.push((key, format!("{prefix}_{i}.{f}")));
                }
            }
        }
        out
    }

    fn scan(&self, base: u64) -> Vec<u64> {
        (0..self.probe(base).min(4096))
            .map(|i| fnv1a64_from(base, i.to_string()))
            .collect()
    }

    pub fn record(&self, index: u32) -> RecordView<'_> {
        RecordView {
            index,
            key: self.keys[index as usize],
            tag: self.tags[index as usize],
            value: self.value_at(index),
        }
    }

    pub(crate) fn record_node(&self, index: u32) -> NodeId {
        self.rec_node[index as usize]
    }

    pub(crate) fn record_meta(&self, index: u32) -> Option<u32> {
        Some(self.rec_meta[index as usize]).filter(|&m| m != NONE)
    }

    pub(crate) fn lang_entries(&self) -> impl Iterator<Item = (&str, &str)> {
        self.lang.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    pub fn record_by_key(&self, key: u64) -> Option<RecordView<'_>> {
        self.index.get(&key).map(|&i| self.record(i))
    }

    pub fn node_of_key(&self, key: u64) -> Option<NodeId> {
        self.index.get(&key).map(|&i| self.rec_node[i as usize])
    }

    /// Schema facts for the record's field.
    pub fn meta(&self, key: u64) -> Option<&FieldMeta> {
        let &i = self.index.get(&key)?;
        let m = self.rec_meta[i as usize];
        (m != NONE).then(|| &schema().metas[m as usize])
    }

    /// The family meaning of a table node.
    pub fn table_meaning(&self, table: NodeId) -> Option<&str> {
        let f = self.table_family.get(&table)?;
        Some(schema().families[*f as usize].meaning.as_str()).filter(|m| !m.is_empty())
    }

    /// The catalogue family of a table node: `entrants.drivers.driver_N`.
    pub fn family(&self, table: NodeId) -> Option<&str> {
        let f = self.table_family.get(&table)?;
        Some(schema().families[*f as usize].pattern.as_str())
    }

    /// Relative leaf paths of the fields the rows of `table` hold, in schema order.
    pub fn fields(&self, table: NodeId) -> Vec<String> {
        self.table_family
            .get(&table)
            .map(|f| schema().families[*f as usize].fields.clone())
            .unwrap_or_default()
    }

    pub fn text(&self, lng_key: &str) -> Option<&str> {
        self.lang.get(lng_key).map(String::as_str)
    }

    /// Whether English text is loaded.
    pub fn has_text(&self) -> bool {
        !self.lang.is_empty()
    }

    /// The key a selector path names (the `swofty_addon::ops` selector syntax).
    pub fn resolve(&self, selector: &str) -> Result<u64> {
        let selector = selector.trim();
        if let Some(k) = parse_key(selector) {
            return Ok(k);
        }
        self.state(None, selector).map_err(Error::Resolve)
    }

    /// The label selector form of a key: (`entrants.drivers.driver[_metadata_=max_verstappen]`, `race_number`)
    /// when its row has a unique label.
    pub fn selector(&self, key: u64) -> Option<(String, String)> {
        let node = self.node_of_key(key)?;
        let mut rel = vec![self.nodes[node as usize].name.as_str()];
        let mut at = self.nodes[node as usize].parent?;
        loop {
            let n = &self.nodes[at as usize];
            match n.kind {
                NodeKind::Row { .. } => break,
                NodeKind::Container => {
                    rel.push(n.name.as_str());
                    at = n.parent?;
                }
                _ => return None,
            }
        }
        if !self.labelled(at) {
            return None;
        }
        rel.reverse();
        Some((self.select_path(at), rel.join(".")))
    }

    fn labelled(&self, row: NodeId) -> bool {
        self.nodes[row as usize]
            .label
            .as_deref()
            .is_some_and(|l| !l.is_empty() && !l.contains(']'))
            && !self.dup_labels.contains(&row)
    }

    /// Like `path`, with every uniquely labelled row written as a selector.
    pub(crate) fn select_path(&self, id: NodeId) -> String {
        let n = &self.nodes[id as usize];
        let parent = n.parent.unwrap_or(0);
        let prefix = |p: NodeId| match self.nodes[p as usize].kind {
            NodeKind::Root | NodeKind::Unnamed => String::new(),
            _ => format!("{}.", self.select_path(p)),
        };
        match &n.kind {
            NodeKind::Root | NodeKind::Unnamed => String::new(),
            NodeKind::Row { .. } if self.labelled(id) => {
                let table = &self.nodes[parent as usize];
                let label = n.label.as_deref().unwrap_or_default();
                format!(
                    "{}{}[_metadata_={label}]",
                    prefix(table.parent.unwrap_or(0)),
                    table.name
                )
            }
            NodeKind::Row { .. } => {
                let table = self.nodes[parent as usize].parent.unwrap_or(0);
                format!("{}{}", prefix(table), n.name)
            }
            _ => format!("{}{}", prefix(parent), n.name),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use swofty_formats::gvdb::Record;

    pub(crate) fn sample() -> Vec<u8> {
        let mut records = vec![
            Record::new("entrants.drivers._driver_instancecount_", 0, "2"),
            Record::new("entrants.teams._team_instancecount_", 0, "1"),
            Record::new("entrants.teams.team_0._metadata_", 2, "ferrari_26"),
            Record::new("entrants.teams.team_0.display_name", 2, "lng_team_fer_title"),
            Record::new("entrants.teams.team_0.prestige", 0, "18"),
            Record::new("tracks._Track_instancecount_", 0, "1"),
            Record::new("tracks.Track_0._metadata_", 2, "melbourne"),
            Record::new("tracks.Track_0.PoleOnRight", 5, "False"),
            Record::new("nobody.knows.this", 1, "1.5"),
        ];
        for (i, (label, first, num)) in [
            ("max_verstappen", "lng_max", "1"),
            ("lando_norris", "lng_lando", "4"),
        ]
        .iter()
        .enumerate()
        {
            for (f, tag, v) in [
                ("_metadata_", 2, *label),
                ("first_name", 2, *first),
                ("car_number", 0, *num),
                ("available", 5, "True"),
                ("driver_strategy.safety_car_pit_min_lap", 0, "1"),
            ] {
                records.push(Record::new(&format!("entrants.drivers.driver_{i}.{f}"), tag, v));
            }
        }
        Gvdb { records }.to_bytes()
    }

    #[test]
    fn names_walks_and_resolves_a_small_database() {
        let cat = Catalog::load(&sample()).unwrap();
        assert_eq!(cat.records(), 19);
        assert_eq!(cat.named(), 18);
        let t = cat.find("entrants.drivers.driver").unwrap();
        assert!(matches!(cat.node(t).kind, NodeKind::Table { rows: 2, .. }));
        assert_eq!(cat.path(t), "entrants.drivers.driver");
        assert_eq!(cat.table_meaning(t).map(|m| !m.is_empty()), Some(true));
        assert_eq!(cat.fields(t)[0], "_metadata_");
        let row = cat.children(t)[1];
        assert_eq!(cat.node(row).label.as_deref(), Some("lando_norris"));
        assert_eq!(cat.path(row), "entrants.drivers.driver_1");
        let k = cat
            .find("entrants.drivers.driver[_metadata_=lando_norris].car_number")
            .unwrap();
        assert_eq!(cat.path(k), "entrants.drivers.driver_1.car_number");
        let key = fnv1a64("entrants.drivers.driver_1.car_number");
        assert_eq!(cat.record_by_key(key).unwrap().value, "4");
        assert_eq!(cat.node_of_key(key), Some(k));
        assert_eq!(
            cat.selector(key),
            Some((
                "entrants.drivers.driver[_metadata_=lando_norris]".into(),
                "car_number".into()
            ))
        );
        let nested = fnv1a64("entrants.drivers.driver_0.driver_strategy.safety_car_pit_min_lap");
        assert_eq!(
            cat.selector(nested).unwrap().1,
            "driver_strategy.safety_car_pit_min_lap"
        );
        assert_eq!(
            cat.find_in(row, "driver_strategy.safety_car_pit_min_lap"),
            cat.node_of_key(fnv1a64(
                "entrants.drivers.driver_1.driver_strategy.safety_car_pit_min_lap"
            ))
        );
        assert!(cat.meta(key).is_some());
        assert_eq!(
            cat.resolve("entrants.drivers.driver[_metadata_=max_verstappen].nope")
                .unwrap(),
            fnv1a64("entrants.drivers.driver_0.nope")
        );
        assert!(
            cat.resolve("entrants.drivers.driver[_metadata_=nobody].car_number")
                .is_err()
        );
        let count = cat.find("tracks._Track_instancecount_").unwrap();
        assert_eq!(cat.node(cat.node(count).parent.unwrap()).name, "tracks");
        assert_eq!(cat.selector(fnv1a64("tracks._Track_instancecount_")), None);
        let unnamed = cat.find(UNNAMED).unwrap();
        let lost = cat.children(unnamed)[0];
        assert_eq!(cat.path(lost), format!("0x{:016x}", fnv1a64("nobody.knows.this")));
        assert_eq!(cat.find(&cat.path(lost)), Some(lost));
        let top: Vec<&str> = cat
            .children(0)
            .iter()
            .map(|&c| cat.node(c).name.as_str())
            .collect();
        assert_eq!(top, ["entrants", "tracks", UNNAMED]);
    }

    #[test]
    fn rows_take_english_names() {
        let mut cat = Catalog::load(&sample()).unwrap();
        let lng = swofty_formats::lng::Lng {
            entries: vec![swofty_formats::lng::Entry {
                key: "lng_team_fer_title".into(),
                value: "Ferrari".into(),
            }],
        };
        cat.set_lang(&lng.to_bytes()).unwrap();
        let row = cat.find("entrants.teams.team_0").unwrap();
        assert_eq!(cat.node(row).display.as_deref(), Some("Ferrari"));
        assert_eq!(cat.text("lng_team_fer_title"), Some("Ferrari"));
    }

    pub(crate) fn real() -> Option<Catalog> {
        let game = std::path::PathBuf::from(std::env::var_os("F1_25_DIR")?);
        let db = std::fs::read(game.join(crate::build::GVDB)).ok()?;
        let mut cat = Catalog::load(&db).unwrap();
        let lang = std::path::Path::new("/mnt/work/.dbedit/lang/language_eng.lng");
        if let Ok(english) = std::fs::read(lang) {
            cat.set_lang(&english).unwrap();
        }
        Some(cat)
    }

    #[test]
    fn names_the_real_database() {
        let t = std::time::Instant::now();
        let Some(cat) = real() else {
            eprintln!("F1_25_DIR is not set; skipping");
            return;
        };
        eprintln!(
            "loaded {} records, named {} in {:?}, {} nodes",
            cat.records(),
            cat.named(),
            t.elapsed(),
            cat.len()
        );
        assert_eq!(cat.records(), 742_857);
        assert!(cat.named() > 560_000, "{}", cat.named());
        let count = cat.find("tracks._Track_instancecount_").unwrap();
        let rows: u32 = cat.record(cat.node(count).record.unwrap()).value.parse().unwrap();
        assert!(rows > 20, "{rows}");
        let k = cat
            .find("entrants.drivers.driver[_metadata_=max_verstappen].car_number")
            .unwrap();
        let row = cat.node(k).parent.unwrap();
        assert_eq!(cat.node(row).label.as_deref(), Some("max_verstappen"));
        if cat.has_text() {
            eprintln!("max_verstappen shows as {:?}", cat.node(row).display);
            assert!(cat.node(row).display.is_some());
        }
        let key = cat.node(k).record.map(|r| cat.record(r).key).unwrap();
        assert_eq!(cat.selector(key).unwrap().1, "car_number");
        assert_eq!(cat.find(&cat.path(k)), Some(k));
        let t = cat.find("tracks.Track").unwrap();
        assert!(cat.table_meaning(t).is_some());
    }
}
