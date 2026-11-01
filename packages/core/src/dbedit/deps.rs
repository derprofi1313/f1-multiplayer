//! What a row's copy under a new name does not bring along: rows of other
//! tables keyed by the row's label or a name derived from it, and values
//! elsewhere that pick the row by its label or `stat_id`.
//!
//! Lookups the game is known to make ([`RULES`]) give warnings. Name and value
//! matches with no known reader are findings to review: the build counts them
//! in one line per addon and lists them on request. Matches in tables of game
//! areas a copy does not reach by default ([`AREAS`]) and matches an op file's
//! `side_data.ignore` acknowledges are listed only.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use swofty_formats::gvdb::Gvdb;

use super::{Catalog, NodeId, NodeKind};
use crate::ops::row_shape;

const FIELDS_SHOWN: usize = 6;
const NAMES_SHOWN: usize = 6;
const PLACES_SHOWN: usize = 3;

/// An `add-row` an addon applied: the source row and the new row, both plain
/// paths, and the `side_data.ignore` patterns of its op file. A `quiet` copy
/// gets no findings; its source still counts as a row copied on its own.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Copy {
    pub addon: String,
    pub from: String,
    pub row: String,
    pub ignore: Vec<String>,
    pub quiet: bool,
}

/// How a known lookup builds its name from the row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleName {
    Label,
    /// The `n`th folder (from 1) of a path field, without an `eaid://` prefix.
    Folder(&'static str, usize),
}

/// A row lookup the game is known to make by a name of a row of `source`.
#[derive(Debug, PartialEq, Eq)]
pub struct Rule {
    /// The row shape of the rows looked up for.
    pub source: &'static str,
    pub what: &'static str,
    pub name: RuleName,
    /// The table searched, as `<container>.<row word>`.
    pub table: &'static str,
    /// The field of the table's rows whose value or comma-separated list
    /// names the row; `None` when the rows are labelled with the name.
    pub field: Option<&'static str>,
    pub reader: &'static str,
    pub without: &'static str,
    /// How to give a copy what the lookup needs; `{table}` and `{example}`
    /// stand for the table and a row the source has there.
    pub fix: &'static str,
}

/// The lookups the reverse-engineered readers confirm.
pub const RULES: &[Rule] = &[
    Rule {
        source: "entrants.vehicles.vehicle_packages.vehicle_package_N",
        what: "vehicle package",
        name: RuleName::Folder("resource_path", 1),
        table: "car_setup.presets.vehicle_package_setups",
        field: None,
        reader: "Car_Load_SetupPreset (0x140c04fd0) reads the package's car setups from that row on session start",
        without: "without it the AI drives setups that ground the car",
        fix: "Add a row per copy, such as an add-row into {table} from {example} that sets _metadata_",
    },
    Rule {
        source: "game_setup.game_setup_modes.game_setup_mode_N",
        what: "game setup mode",
        name: RuleName::Label,
        table: "game_setup.branding.game_setup_branding",
        field: Some("game_setup_modes"),
        reader: "the branding reader (0x142a12f80) keeps the rows whose game_setup_modes list the mode",
        without: "without one the mode has no branding: logos and flags such as car_setup_available and mfd_enabled",
        fix: "Add each copy to game_setup_modes, such as with a list-append on {example}",
    },
    Rule {
        source: "game_setup.game_setup_modes.game_setup_mode_N",
        what: "game setup mode",
        name: RuleName::Label,
        table: "track_data.game_setup.TrackDataSet",
        field: Some("game_setup_modes"),
        reader: "the track data reader (0x142e44b50) keeps the data sets whose game_setup_modes list the mode",
        without: "without one the mode gets no per-track data set",
        fix: "Add each copy to game_setup_modes, such as with a list-append on {example}",
    },
    Rule {
        source: "entrants.teams.team_N",
        what: "team",
        name: RuleName::Label,
        table: "pit_lane.pit_times.team_pit_times",
        field: Some("team"),
        reader: "the pit lane system (0x1434c8bf0) reads each team's pit stop times from the row whose team names it",
        without: "without one the team's pit stops use other times (the consumer is not traced)",
        fix: "Add a row per copy, such as an add-row into {table} from {example} that sets _metadata_ and team",
    },
];

impl Rule {
    pub fn applies(&self, cat: &Catalog, row: NodeId) -> bool {
        row_shape(&cat.path(row)) == self.source
    }

    /// How the name comes from the row: `the label`, `folder 1 of resource_path`.
    pub fn how(&self) -> String {
        match self.name {
            RuleName::Label => "the label".into(),
            RuleName::Folder(field, n) => format!("folder {n} of {field}"),
        }
    }

    /// A row of the rule's table that the lookup finds for `name`.
    fn finds(&self, cat: &Catalog, index: &Index, name: &str) -> Option<NodeId> {
        let Some(field) = self.field else {
            return index.row_in(cat, self.table, name);
        };
        let shape = format!("{}_N.{field}", self.table);
        index
            .values(name)
            .iter()
            .map(|&r| cat.record_node(r))
            .find(|&id| row_shape(&cat.path(id)) == shape)
            .and_then(|id| parent(cat, id))
    }

    /// The name the lookup uses for `row`.
    pub fn name_of(&self, cat: &Catalog, row: NodeId) -> Option<String> {
        match self.name {
            RuleName::Label => cat.node(row).label.clone().filter(|l| !l.is_empty()),
            RuleName::Folder(field, n) => {
                let id = cat.find_in(row, field)?;
                let value = cat.record(cat.node(id).record?).value;
                let path = value.strip_prefix("eaid://").unwrap_or(value);
                if !path.contains('/') {
                    return None;
                }
                path.split('/')
                    .nth(n.checked_sub(1)?)
                    .filter(|f| !f.is_empty())
                    .map(str::to_owned)
            }
        }
    }
}

/// Table-path prefixes of game areas a copy does not reach by default, and the
/// area's name. A copy into a table of an area brings the area into reach for
/// its addon.
pub const AREAS: &[(&str, &str)] = &[
    ("accolades", "career"),
    ("bank", "career"),
    ("career", "career"),
    ("challenge_career", "career"),
    ("driver_academies", "career"),
    ("driver_contracts", "career"),
    ("driver_market", "career"),
    ("emails", "career"),
    ("engine_management", "career"),
    ("facilities", "career"),
    ("invitational_events", "career"),
    ("news", "career"),
    ("personal_development", "career"),
    ("pit_crew_training", "career"),
    ("press_interviews", "career"),
    ("recognition", "career"),
    ("resource_points", "career"),
    ("rivals", "career"),
    ("specialist_manager", "career"),
    ("sponsors", "career"),
    ("story_mode", "story mode"),
    ("driver_statistics.story_mode_driver_statistics", "story mode"),
    ("f1_world", "F1 World"),
    ("second_worlds", "F1 World"),
    ("f1cg", "F1 World"),
    ("auction", "F1 World"),
    ("leaderboards", "online"),
    ("online_systems", "online"),
    ("entitlements", "online"),
    ("paddock_pass", "online"),
    ("options_system.options.multiplayer", "online"),
    ("options_system.context", "online"),
    ("options_system.restrictions", "online"),
    ("showroom", "front end"),
    ("f1_life", "front end"),
    ("videos", "front end"),
    ("customisation", "customisation"),
    ("unlockable_content", "customisation"),
];

/// The area of a table or field path, `unnamed` for top-level names the
/// catalogue does not know.
pub fn area(path: &str) -> Option<&'static str> {
    if path.starts_with('{') {
        return Some("unnamed");
    }
    AREAS
        .iter()
        .find(|(prefix, _)| {
            path.strip_prefix(prefix)
                .is_some_and(|rest| rest.is_empty() || rest.starts_with('.'))
        })
        .map(|(_, a)| *a)
}

/// A table or field path without row numbers, `_N` and selectors:
/// `accolades.AccoladeDataGroup.driver_data.AccoladeDriver`.
pub fn plain(path: &str) -> String {
    path.split('.')
        .map(|seg| {
            let seg = seg.split('[').next().unwrap_or(seg);
            if let Some(end) = seg.find('}') {
                return &seg[..=end];
            }
            match seg.rsplit_once('_') {
                Some((word, n)) if n == "N" || (!n.is_empty() && n.bytes().all(|b| b.is_ascii_digit())) => {
                    word
                }
                _ => seg,
            }
        })
        .collect::<Vec<_>>()
        .join(".")
}

fn glob(pattern: &str, text: &str) -> bool {
    let (p, t) = (pattern.as_bytes(), text.as_bytes());
    let (mut i, mut j, mut star, mut mark) = (0, 0, None, 0);
    while j < t.len() {
        if i < p.len() && p[i] == b'*' {
            star = Some(i);
            mark = j;
            i += 1;
        } else if i < p.len() && p[i] == t[j] {
            i += 1;
            j += 1;
        } else if let Some(s) = star {
            i = s + 1;
            mark += 1;
            j = mark;
        } else {
            return false;
        }
    }
    p[i..].iter().all(|&c| c == b'*')
}

/// Whether the `side_data.ignore` pattern covers a finding at any of `places`.
pub fn ignores(pattern: &str, places: &[String]) -> bool {
    let pattern = plain(pattern);
    places.iter().any(|p| glob(&pattern, &plain(p)))
}

/// A name the game can derive from a row.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Derived {
    pub value: String,
    /// `label`, `label+_<word>`, a field path, or `<field> folder <n>` for a part of a path value.
    pub from: String,
}

impl Derived {
    /// How the name comes from the row: `the label`, `folder 1 of resource_path`.
    pub fn how(&self) -> String {
        match self.from.as_str() {
            "label" => "the label".into(),
            f => match (f.strip_prefix("label+"), f.split_once(" folder ")) {
                (Some(suffix), _) => format!("the label + {suffix}"),
                (None, Some((field, n))) => format!("folder {n} of {field}"),
                (None, None) => format!("the {f} value"),
            },
        }
    }
}

/// Rows of one table family labelled with a derived name of the row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keyed {
    pub name: Derived,
    pub family: String,
    /// The tables of the family holding such a row.
    pub tables: Vec<NodeId>,
    /// The known lookup that finds the row there.
    pub rule: Option<&'static Rule>,
    pub area: Option<&'static str>,
}

/// Values of one field pattern that pick the row by its label or `stat_id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    pub by: By,
    pub field: String,
    pub count: usize,
    pub area: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum By {
    Label,
    StatId,
}

impl By {
    fn word(self) -> &'static str {
        match self {
            By::Label => "label",
            By::StatId => "stat_id",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Deps {
    pub names: Vec<Derived>,
    pub keyed: Vec<Keyed>,
    pub references: Vec<Reference>,
    /// Values naming the row's label under keys the catalogue does not name.
    pub unnamed: usize,
    /// The known lookups for the row: the rule, the name and the row found.
    pub rules: Vec<(&'static Rule, String, Option<NodeId>)>,
}

fn identifier(v: &str) -> bool {
    v.len() >= 3 && label_like(v)
}

fn label_like(v: &str) -> bool {
    let b = v.as_bytes();
    b.len() >= 2
        && (b[0].is_ascii_alphabetic() || b[0] == b'_')
        && b.iter().all(|c| c.is_ascii_alphanumeric() || *c == b'_')
        && b.iter().any(u8::is_ascii_lowercase)
        && !matches!(v, "True" | "False" | "true" | "false")
}

fn parent(cat: &Catalog, id: NodeId) -> Option<NodeId> {
    cat.node(id).parent
}

fn under(cat: &Catalog, mut id: NodeId, row: NodeId) -> bool {
    loop {
        if id == row {
            return true;
        }
        match parent(cat, id) {
            Some(p) => id = p,
            None => return false,
        }
    }
}

fn under_any(cat: &Catalog, mut id: NodeId, rows: &HashSet<NodeId>) -> bool {
    loop {
        if rows.contains(&id) {
            return true;
        }
        match parent(cat, id) {
            Some(p) => id = p,
            None => return false,
        }
    }
}

fn unnamed(cat: &Catalog, id: NodeId) -> bool {
    parent(cat, id).is_some_and(|p| cat.node(p).kind == NodeKind::Unnamed)
}

fn table_word(cat: &Catalog, row: NodeId) -> Option<&str> {
    match &cat.node(parent(cat, row)?).kind {
        NodeKind::Table { word, .. } => Some(word.as_str()),
        _ => None,
    }
}

fn family(cat: &Catalog, table: NodeId) -> String {
    cat.family(table)
        .map(str::to_owned)
        .unwrap_or_else(|| format!("{}_N", row_shape(&cat.path(table))))
}

fn rel(cat: &Catalog, row: NodeId, id: NodeId) -> String {
    let mut parts = Vec::new();
    let mut at = id;
    while at != row {
        parts.push(cat.node(at).name.as_str());
        match parent(cat, at) {
            Some(p) => at = p,
            None => break,
        }
    }
    parts.reverse();
    parts.join(".")
}

/// The fields of `row` outside its nested tables, as (relative path, value).
fn fields(cat: &Catalog, row: NodeId) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut stack = vec![row];
    while let Some(id) = stack.pop() {
        for &c in cat.children(id) {
            let n = cat.node(c);
            match n.kind {
                NodeKind::Key => {
                    if let Some(r) = n.record {
                        out.push((rel(cat, row, c), cat.record(r).value.to_owned()));
                    }
                }
                NodeKind::Container => stack.push(c),
                _ => {}
            }
        }
    }
    out.sort();
    out
}

/// The names the game can derive from `row`: its label, `<label>_<row word>`,
/// identifier values of its fields and the folders of its path values that
/// hold the label or a digit.
pub fn derived(cat: &Catalog, row: NodeId) -> Vec<Derived> {
    derived_with(cat, row, false)
}

fn derived_with(cat: &Catalog, row: NodeId, every_folder: bool) -> Vec<Derived> {
    let mut out = Vec::new();
    let label = cat.node(row).label.clone().unwrap_or_default();
    if label_like(&label) {
        out.push(Derived {
            value: label.clone(),
            from: "label".into(),
        });
        if let Some(word) = table_word(cat, row) {
            out.push(Derived {
                value: format!("{label}_{word}"),
                from: format!("label+_{word}"),
            });
        }
    }
    for (field, value) in fields(cat, row) {
        if field == "_metadata_" || field.ends_with("stat_id") {
            continue;
        }
        if identifier(&value) {
            out.push(Derived { value, from: field });
            continue;
        }
        let path = value.strip_prefix("eaid://").unwrap_or(&value);
        if !path.contains('/') {
            continue;
        }
        for (i, part) in path.split('/').enumerate() {
            let names_row = every_folder
                || (!label.is_empty() && part.contains(label.as_str()))
                || part.bytes().any(|b| b.is_ascii_digit());
            if identifier(part) && names_row {
                out.push(Derived {
                    value: part.to_owned(),
                    from: format!("{field} folder {}", i + 1),
                });
            }
        }
    }
    out
}

fn stat_id(cat: &Catalog, row: NodeId) -> Option<String> {
    let id = cat.find_in(row, "stat_id")?;
    let v = cat.record(cat.node(id).record?).value.trim();
    v.parse::<i64>().is_ok().then(|| v.to_owned())
}

/// Whether a field named `leaf` can hold a `stat_id` of a row of the table `word`.
fn id_field(leaf: &str, word: &str) -> bool {
    let leaf = leaf.to_ascii_lowercase();
    let word = word.to_ascii_lowercase();
    let stem = word.rsplit('_').next().unwrap_or(&word);
    leaf.ends_with("id") && leaf != "stat_id" && (leaf.contains(&word) || leaf.contains(stem))
}

/// Rows labelled and records holding each needle, found in one pass.
struct Index {
    rows: HashMap<String, Vec<NodeId>>,
    values: HashMap<String, Vec<u32>>,
}

impl Index {
    fn new(cat: &Catalog, labels: &HashSet<String>, values: &HashSet<String>) -> Self {
        let mut rows: HashMap<String, Vec<NodeId>> = HashMap::new();
        for id in 0..cat.len() as NodeId {
            let n = cat.node(id);
            if let (NodeKind::Row { .. }, Some(l)) = (&n.kind, &n.label)
                && labels.contains(l)
            {
                rows.entry(l.clone()).or_default().push(id);
            }
        }
        let mut found: HashMap<String, Vec<u32>> = HashMap::new();
        for i in 0..cat.records() as u32 {
            let v = cat.record(i).value;
            if values.contains(v) {
                found.entry(v.to_owned()).or_default().push(i);
            } else if v.contains(',') {
                for item in v.split(',').map(str::trim) {
                    if values.contains(item) {
                        found.entry(item.to_owned()).or_default().push(i);
                    }
                }
            }
        }
        Self { rows, values: found }
    }

    fn rows(&self, label: &str) -> &[NodeId] {
        self.rows.get(label).map_or(&[], Vec::as_slice)
    }

    fn values(&self, v: &str) -> &[u32] {
        self.values.get(v).map_or(&[], Vec::as_slice)
    }

    /// A row of the table `table` (`<container>.<row word>`) labelled `label`.
    fn row_in(&self, cat: &Catalog, table: &str, label: &str) -> Option<NodeId> {
        self.rows(label)
            .iter()
            .copied()
            .find(|&r| parent(cat, r).is_some_and(|t| cat.select_path(t) == table))
    }
}

struct Skip {
    rows: Vec<NodeId>,
    family: Option<String>,
}

impl Skip {
    fn row(&self, cat: &Catalog, id: NodeId) -> bool {
        self.rows.iter().any(|&r| under(cat, id, r))
    }

    fn table(&self, cat: &Catalog, table: NodeId) -> bool {
        self.family.as_deref() == Some(family(cat, table).as_str())
    }
}

fn keyed(cat: &Catalog, index: &Index, names: &[Derived], skip: &Skip) -> Vec<Keyed> {
    let mut by: BTreeMap<(String, Derived), BTreeSet<NodeId>> = BTreeMap::new();
    for name in names {
        for &r in index.rows(&name.value) {
            let Some(t) = parent(cat, r) else { continue };
            if skip.row(cat, r) || skip.table(cat, t) {
                continue;
            }
            by.entry((family(cat, t), name.clone())).or_default().insert(t);
        }
    }
    by.into_iter()
        .map(|((family, name), tables)| Keyed {
            area: area(&family),
            name,
            family,
            tables: tables.into_iter().collect(),
            rule: None,
        })
        .collect()
}

/// The field pattern of a record, or `None` for `_metadata_`, unnamed and skipped keys.
fn field_of(cat: &Catalog, record: u32, skip: &Skip) -> Option<String> {
    let id = cat.record_node(record);
    let n = cat.node(id);
    if n.name == "_metadata_" || unnamed(cat, id) || skip.row(cat, id) {
        return None;
    }
    Some(row_shape(&cat.path(id)))
}

/// The row a value can pick: its label and `stat_id`, and whether rows of
/// other tables share the label.
struct Target<'a> {
    label: &'a str,
    stat_id: Option<&'a str>,
    word: &'a str,
    container: String,
    shared: bool,
}

impl<'a> Target<'a> {
    fn new(cat: &'a Catalog, index: &Index, row: NodeId, label: &'a str, stat_id: Option<&'a str>) -> Self {
        let table = parent(cat, row);
        let own = table.map(|t| family(cat, t));
        let shared = index
            .rows(label)
            .iter()
            .any(|&r| parent(cat, r).map(|t| family(cat, t)) != own);
        Self {
            label,
            stat_id,
            word: table_word(cat, row).unwrap_or_default(),
            container: table
                .and_then(|t| parent(cat, t))
                .map(|c| cat.path(c))
                .unwrap_or_default(),
            shared,
        }
    }

    /// Whether the field of `record` names rows of this table: by its name, or
    /// by its catalogue meaning.
    fn named_by(&self, cat: &Catalog, record: u32, field: &str) -> bool {
        let leaf = field.rsplit('.').next().unwrap_or(field);
        (!self.word.is_empty() && leaf.contains(self.word))
            || (!self.container.is_empty()
                && cat
                    .meta(cat.record(record).key)
                    .is_some_and(|m| m.meaning.contains(&self.container)))
    }
}

/// The records of `target` that pick it, by field pattern: (by, field) -> records.
fn picks(
    cat: &Catalog,
    index: &Index,
    target: &Target,
    skip: &Skip,
) -> (BTreeMap<(By, String), Vec<u32>>, usize) {
    let mut by: BTreeMap<(By, String), Vec<u32>> = BTreeMap::new();
    let mut hidden = 0;
    for &r in index.values(target.label) {
        match field_of(cat, r, skip) {
            Some(f) if target.shared && !target.named_by(cat, r, &f) => {}
            Some(f) => by.entry((By::Label, f)).or_default().push(r),
            None if unnamed(cat, cat.record_node(r)) => hidden += 1,
            None => {}
        }
    }
    if let Some(id) = target.stat_id {
        for &r in index.values(id) {
            if let Some(f) = field_of(cat, r, skip)
                && id_field(f.rsplit('.').next().unwrap_or(&f), target.word)
            {
                by.entry((By::StatId, f)).or_default().push(r);
            }
        }
    }
    (by, hidden)
}

fn references(cat: &Catalog, index: &Index, target: &Target, skip: &Skip) -> (Vec<Reference>, usize) {
    let (by, hidden) = picks(cat, index, target, skip);
    let refs = by
        .into_iter()
        .map(|((by, field), records)| Reference {
            area: area(&field),
            by,
            field,
            count: records.len(),
        })
        .collect();
    (refs, hidden)
}

/// The label-keyed side data of `row` in the stock database `cat`.
pub fn deps(cat: &Catalog, row: NodeId) -> Deps {
    let mut names = derived(cat, row);
    let rules: Vec<(&'static Rule, String)> = RULES
        .iter()
        .filter(|r| r.applies(cat, row))
        .filter_map(|r| Some((r, r.name_of(cat, row)?)))
        .collect();
    for (rule, value) in &rules {
        if !names.iter().any(|n| n.value == *value) {
            names.push(Derived {
                value: value.clone(),
                from: match rule.name {
                    RuleName::Label => "label".into(),
                    RuleName::Folder(field, n) => format!("{field} folder {n}"),
                },
            });
        }
    }
    let label = cat.node(row).label.clone().unwrap_or_default();
    let id = stat_id(cat, row);
    let mut labels: HashSet<String> = names.iter().map(|n| n.value.clone()).collect();
    labels.insert(label.clone());
    let values: HashSet<String> = [Some(label.clone()), id.clone()].into_iter().flatten().collect();
    let index = Index::new(cat, &labels, &values);
    let skip = Skip {
        rows: vec![row],
        family: parent(cat, row).map(|t| family(cat, t)),
    };
    let mut keyed = keyed(cat, &index, &names, &skip);
    for k in &mut keyed {
        k.rule = rules
            .iter()
            .find(|(r, v)| {
                r.field.is_none()
                    && *v == k.name.value
                    && k.tables.iter().any(|&t| cat.select_path(t) == r.table)
            })
            .map(|(r, _)| *r);
    }
    let rules = rules
        .into_iter()
        .map(|(r, v)| {
            let found = r.finds(cat, &index, &v);
            (r, v, found)
        })
        .collect();
    let target = Target::new(cat, &index, row, &label, id.as_deref());
    let (references, unnamed) = references(cat, &index, &target, &skip);
    Deps {
        names,
        keyed,
        references,
        unnamed,
        rules,
    }
}

/// Whether a derived name is a plain field value, which may name another row.
fn field_value(d: &Derived) -> bool {
    d.from != "label" && !d.from.starts_with("label+") && !d.from.contains(" folder ")
}

struct Pair {
    src: NodeId,
    dst: NodeId,
    /// The derived names of the source the copy changed.
    names: Vec<Derived>,
    /// The copy's derived names by how they come about.
    ours: HashMap<String, String>,
    /// For each derived name of the source, the copy's names from the same
    /// fields: the rows a reader finds for the source by that name, it can
    /// find for the copy by one of these.
    alias: HashMap<String, BTreeSet<String>>,
    label: (String, String),
    stat_id: Option<(String, String)>,
    /// (rule, source name, copy name) for each known lookup whose name the copy changed.
    rules: Vec<(&'static Rule, String, String)>,
}

fn pair(cat: &Catalog, src: NodeId, dst: NodeId) -> Pair {
    let ours: HashMap<String, String> = derived_with(cat, dst, true)
        .into_iter()
        .map(|d| (d.from, d.value))
        .collect();
    let mut names = Vec::new();
    let mut alias: HashMap<String, BTreeSet<String>> = HashMap::new();
    for d in derived(cat, src) {
        let Some(new) = ours.get(&d.from) else { continue };
        alias.entry(d.value.clone()).or_default().insert(new.clone());
        if *new != d.value {
            names.push(d);
        }
    }
    let rules = RULES
        .iter()
        .filter(|r| r.applies(cat, src))
        .filter_map(|r| Some((r, r.name_of(cat, src)?, r.name_of(cat, dst)?)))
        .filter(|(_, old, new)| old != new)
        .collect();
    let label = |r: NodeId| cat.node(r).label.clone().unwrap_or_default();
    Pair {
        src,
        dst,
        names,
        ours,
        alias,
        label: (label(src), label(dst)),
        stat_id: stat_id(cat, src).zip(stat_id(cat, dst)).filter(|(a, b)| a != b),
        rules,
    }
}

fn list(names: &BTreeSet<String>, shown: usize) -> String {
    let v: Vec<&str> = names.iter().take(shown).map(String::as_str).collect();
    match names.len() > v.len() {
        true => format!("{} and {} more", v.join(", "), names.len() - v.len()),
        false => v.join(", "),
    }
}

/// What the build reports about the side data of the `add-row` copies.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    /// Known lookups the copies miss, one line per addon counting the findings
    /// to review, and `side_data.ignore` patterns that match nothing.
    pub warnings: Vec<String>,
    /// Every finding, tagged `[known]`, `[review]`, `[<area>]` or `[ignored]`.
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Tag {
    Known,
    Review,
    Area(&'static str),
    Ignored,
}

impl Tag {
    fn text(self) -> &'static str {
        match self {
            Tag::Known => "known",
            Tag::Review => "review",
            Tag::Area(a) => a,
            Tag::Ignored => "ignored",
        }
    }
}

#[derive(Default)]
struct Missing {
    olds: BTreeSet<String>,
    hows: BTreeSet<String>,
    sources: BTreeSet<String>,
    tables: BTreeSet<NodeId>,
    lacking: BTreeMap<String, BTreeSet<NodeId>>,
    example: Option<NodeId>,
}

#[derive(Default)]
struct Refs {
    news: BTreeSet<String>,
    sources: BTreeSet<String>,
    fields: BTreeMap<String, BTreeMap<NodeId, usize>>,
}

/// Per addon, the ignore patterns of its copies and the ones that matched.
#[derive(Default)]
struct Acks {
    patterns: BTreeSet<String>,
    used: HashSet<String>,
}

impl Acks {
    fn tag(&mut self, copy: &Copy, places: &[String], tag: Tag) -> Tag {
        let mut hit = false;
        for p in &copy.ignore {
            if ignores(p, places) {
                self.used.insert(p.clone());
                hit = true;
            }
        }
        if hit { Tag::Ignored } else { tag }
    }
}

/// The review tag, or the area of `path` when the addon's copies don't reach it.
fn scope(path: &str, reach: &HashSet<&'static str>) -> Tag {
    match area(path) {
        Some(a) if !reach.contains(a) => Tag::Area(a),
        _ => Tag::Review,
    }
}

/// Build warnings and findings for the side data `copies` lack in the built
/// database `db`. `name` gives an addon's display name from its id.
pub fn check(db: &Gvdb, copies: &[Copy], name: &dyn Fn(&str) -> String) -> Report {
    if copies.is_empty() {
        return Report::default();
    }
    let cat = Catalog::from_gvdb(db.clone(), String::new());
    check_with(&cat, copies, name)
}

/// [`check`] on a database already catalogued.
pub fn check_with(cat: &Catalog, copies: &[Copy], name: &dyn Fn(&str) -> String) -> Report {
    let pairs: Vec<(&Copy, Pair)> = copies
        .iter()
        .filter_map(|c| Some((c, pair(cat, cat.find(&c.from)?, cat.find(&c.row)?))))
        .filter(|(_, p)| p.label.0 != p.label.1)
        .collect();
    let mut labels = HashSet::new();
    let mut values = HashSet::new();
    let mut copied: HashMap<&str, HashSet<NodeId>> = HashMap::new();
    let mut reach: HashMap<&str, HashSet<&'static str>> = HashMap::new();
    let mut acks: BTreeMap<String, Acks> = BTreeMap::new();
    for c in copies {
        reach.entry(&c.addon).or_default().extend(area(&c.row));
        if let Some(row) = cat.find(&c.row) {
            copied.entry(&c.addon).or_default().insert(row);
        }
        acks.entry(c.addon.clone())
            .or_default()
            .patterns
            .extend(c.ignore.iter().cloned());
    }
    for (_, p) in &pairs {
        for d in &p.names {
            labels.insert(d.value.clone());
        }
        for names in p.alias.values() {
            labels.extend(names.iter().cloned());
        }
        for (_, old, new) in &p.rules {
            labels.extend([old.clone(), new.clone()]);
        }
        labels.insert(p.label.0.clone());
        values.extend([p.label.0.clone(), p.label.1.clone()]);
        if let Some((a, b)) = &p.stat_id {
            values.extend([a.clone(), b.clone()]);
        }
    }
    let index = Index::new(cat, &labels, &values);
    let sources: HashSet<(NodeId, String)> = pairs
        .iter()
        .filter_map(|(_, p)| Some((parent(cat, p.src)?, p.label.0.clone())))
        .collect();
    let families = |v: &str| -> HashSet<String> {
        index
            .rows(v)
            .iter()
            .filter_map(|&r| parent(cat, r))
            .map(|t| family(cat, t))
            .collect()
    };
    let none = HashSet::new();
    let no_rows = HashSet::new();
    let mut known: BTreeMap<(String, Tag, usize), Missing> = BTreeMap::new();
    let mut missing: BTreeMap<(String, Tag, String), Missing> = BTreeMap::new();
    let mut refs: BTreeMap<(String, Tag, String, By), Refs> = BTreeMap::new();
    for (copy, p) in pairs.iter().filter(|(c, _)| !c.quiet) {
        let acks = acks.get_mut(&copy.addon).expect("every copy's addon");
        let reach = reach.get(copy.addon.as_str()).unwrap_or(&none);
        for (rule, old, new) in &p.rules {
            let Some(example) = rule.finds(cat, &index, old) else {
                continue;
            };
            if rule.finds(cat, &index, new).is_some() {
                continue;
            }
            let tag = acks.tag(copy, &[rule.table.to_owned()], Tag::Known);
            let i = RULES.iter().position(|r| std::ptr::eq(r, *rule)).expect("a rule");
            let m = known.entry((copy.addon.clone(), tag, i)).or_default();
            m.olds.insert(old.clone());
            m.sources.insert(cat.select_path(p.src));
            m.lacking.entry(new.clone()).or_default();
            m.example.get_or_insert(example);
        }
        let ruled: Vec<&str> = RULES
            .iter()
            .filter(|r| r.applies(cat, p.src))
            .map(|r| r.table)
            .collect();
        let skip = Skip {
            rows: vec![p.src, p.dst],
            family: parent(cat, p.dst).map(|t| family(cat, t)),
        };
        let refers = |d: &Derived| {
            field_value(d)
                && p.ours
                    .get(&d.from)
                    .is_some_and(|new| !families(new).is_disjoint(&families(&d.value)))
        };
        let own = parent(cat, p.src).map(|t| family(cat, t)).unwrap_or_default();
        for k in keyed(cat, &index, &p.names, &skip) {
            if refers(&k.name) {
                continue;
            }
            let by_label = p
                .names
                .iter()
                .any(|d| d.value == k.name.value && (d.from == "label" || d.from.starts_with("label+")));
            let alts = &p.alias[&k.name.value];
            let has: HashSet<NodeId> = alts
                .iter()
                .filter(|a| !by_label || **a != k.name.value)
                .flat_map(|a| index.rows(a))
                .filter_map(|&r| parent(cat, r))
                .collect();
            let lacking: BTreeSet<NodeId> = k
                .tables
                .iter()
                .copied()
                .filter(|t| {
                    !has.contains(t)
                        && !ruled.contains(&cat.select_path(*t).as_str())
                        && !sources.contains(&(*t, k.name.value.clone()))
                })
                .collect();
            if lacking.is_empty() {
                continue;
            }
            let mut places: Vec<String> = lacking.iter().map(|&t| cat.select_path(t)).collect();
            places.push(k.family.clone());
            let tag = acks.tag(copy, &places, scope(&k.family, reach));
            let m = missing
                .entry((copy.addon.clone(), tag, k.family.clone()))
                .or_default();
            m.olds.insert(k.name.value.clone());
            m.hows.insert(k.name.how());
            m.sources.insert(cat.select_path(p.src));
            m.tables.extend(k.tables.iter().copied());
            let shown = p
                .names
                .iter()
                .filter(|d| d.value == k.name.value && !refers(d))
                .filter_map(|d| p.ours.get(&d.from));
            for new in shown {
                m.lacking
                    .entry(new.clone())
                    .or_default()
                    .extend(lacking.iter().copied());
            }
            if m.example.is_none() {
                m.example = index
                    .rows(&k.name.value)
                    .iter()
                    .copied()
                    .find(|&r| parent(cat, r).is_some_and(|t| lacking.contains(&t)));
            }
        }
        let stat_ids = p.stat_id.as_ref();
        let target = Target::new(cat, &index, p.src, &p.label.0, stat_ids.map(|(a, _)| a.as_str()));
        let (ours, _) = picks(cat, &index, &target, &skip);
        let theirs = Target {
            label: &p.label.1,
            stat_id: stat_ids.map(|(_, b)| b.as_str()),
            shared: false,
            ..target
        };
        let (theirs, _) = picks(cat, &index, &theirs, &skip);
        let rows = copied.get(copy.addon.as_str()).unwrap_or(&no_rows);
        for ((by, field), records) in ours {
            if theirs.contains_key(&(by, field.clone())) {
                continue;
            }
            if records.iter().any(|&r| under_any(cat, cat.record_node(r), rows)) {
                continue;
            }
            let tag = acks.tag(copy, std::slice::from_ref(&field), scope(&field, reach));
            let new = match by {
                By::Label => p.label.1.clone(),
                By::StatId => stat_ids.map(|(_, b)| b.clone()).unwrap_or_default(),
            };
            let r = refs
                .entry((copy.addon.clone(), tag, own.clone(), by))
                .or_default();
            r.news.insert(new);
            r.sources.insert(cat.select_path(p.src));
            let n = r.fields.entry(field).or_default().entry(p.src).or_default();
            *n = (*n).max(records.len());
        }
    }
    let mut report = Report::default();
    let mut review: BTreeMap<String, (BTreeSet<String>, BTreeSet<String>, usize)> = BTreeMap::new();
    let push = |report: &mut Report, addon: &str, tag: Tag, line: String| {
        report
            .findings
            .push(format!("{}: [{}] {line}", name(addon), tag.text()));
        if tag == Tag::Known {
            report.warnings.push(format!("{}: {line}", name(addon)));
        }
    };
    for ((addon, tag, i), m) in &known {
        let rule = &RULES[*i];
        let news = list(&m.lacking.keys().cloned().collect(), NAMES_SHOWN);
        let olds = list(&m.olds, NAMES_SHOWN);
        let example = m.example.map(|r| cat.select_path(r)).unwrap_or_default();
        let (what, sources) = match m.sources.len() {
            1 => (rule.what.to_owned(), list(&m.sources, NAMES_SHOWN)),
            n => (
                format!("{}s", rule.what),
                format!("({n}) {}", list(&m.sources, PLACES_SHOWN)),
            ),
        };
        let found = match rule.field {
            None => format!(
                "{} has a row labelled {olds} and none labelled {news}",
                rule.table
            ),
            Some(field) => format!(
                "no {} row names {news} in {field}, while rows such as {example} name {olds}",
                rule.table
            ),
        };
        let is = if m.olds.len() == 1 { "is" } else { "are" };
        let line = format!(
            "{found}; {olds} {is} {} of the add-row {what} {sources}, and {}: {}. {}",
            rule.how(),
            rule.reader,
            rule.without,
            rule.fix
                .replace("{table}", rule.table)
                .replace("{example}", &example),
        );
        push(&mut report, addon, *tag, line);
    }
    for ((addon, tag, family), m) in &missing {
        let lacking: BTreeSet<String> = m.lacking.keys().cloned().collect();
        let worst = m.lacking.values().map(BTreeSet::len).max().unwrap_or(0);
        let example = m.example.map(|r| cat.select_path(r)).unwrap_or_default();
        let table = m
            .example
            .and_then(|r| parent(cat, r))
            .map(|t| cat.select_path(t))
            .unwrap_or_else(|| family.clone());
        let olds = list(&m.olds, NAMES_SHOWN);
        let (rows, are, sources) = match (m.olds.len(), m.sources.len()) {
            (1, 1) => ("a row", "is", "source"),
            (1, _) => ("a row", "is", "sources"),
            _ => ("rows", "are", "sources"),
        };
        let place = match m.tables.len() {
            1 => format!(
                "{table} has {rows} labelled {olds} and none labelled {}",
                list(&lacking, NAMES_SHOWN)
            ),
            n => format!(
                "{family} has rows labelled {olds} in {n} tables and none labelled {} in up to {worst} of them",
                list(&lacking, NAMES_SHOWN)
            ),
        };
        let line = format!(
            "{place}; {olds} {are} {} of the add-row {sources} {}. If a reader finds the source's row by that name, add a row per copy, such as an add-row into {table} from {example} that sets _metadata_",
            m.hows.iter().cloned().collect::<Vec<_>>().join(" and "),
            list(&m.sources, NAMES_SHOWN),
        );
        if *tag == Tag::Review {
            let r = review.entry(addon.clone()).or_default();
            r.0.insert(if m.tables.len() == 1 {
                table
            } else {
                family.clone()
            });
            r.2 += 1;
        }
        push(&mut report, addon, *tag, line);
    }
    for ((addon, tag, _, by), r) in &refs {
        let count = |f: &BTreeMap<NodeId, usize>| f.values().sum::<usize>();
        let shown: Vec<String> = r
            .fields
            .iter()
            .take(FIELDS_SHOWN)
            .map(|(f, n)| format!("{f} ({})", count(n)))
            .collect();
        let more = match r.fields.len() > shown.len() {
            true => format!(" and {} more field(s)", r.fields.len() - shown.len()),
            false => String::new(),
        };
        let (source, copy) = match r.sources.len() {
            1 => ("source", "copy"),
            _ => ("sources", "copies"),
        };
        let line = format!(
            "{} value(s) pick the add-row {source} {} by {} and none pick the {copy} {}: {}{more}",
            r.fields.values().map(count).sum::<usize>(),
            list(&r.sources, NAMES_SHOWN),
            match (by, r.sources.len()) {
                (_, 1) => format!("its {}", by.word()),
                _ => format!("their {}", by.word()),
            },
            list(&r.news, NAMES_SHOWN),
            shown.join(", "),
        );
        if *tag == Tag::Review {
            let e = review.entry(addon.clone()).or_default();
            e.1.extend(r.fields.keys().cloned());
            e.2 += 1;
        }
        push(&mut report, addon, *tag, line);
    }
    for (addon, (tables, fields, n)) in review {
        let mut parts = Vec::new();
        if !tables.is_empty() {
            parts.push(format!(
                "rows labelled with a source's name in {} and none for the copy",
                list(&tables, PLACES_SHOWN)
            ));
        }
        if !fields.is_empty() {
            parts.push(format!(
                "values naming a source in {} field(s) and none naming the copy",
                fields.len()
            ));
        }
        report.warnings.push(format!(
            "{}: {n} side-data finding(s) to review, matched by name with no known reader: {}. f1mp build --side-data lists them; list reviewed ones in the op file's side_data.ignore",
            name(&addon),
            parts.join("; "),
        ));
    }
    for (addon, a) in acks {
        for p in a.patterns.iter().filter(|p| !a.used.contains(*p)) {
            report.warnings.push(format!(
                "{}: side_data.ignore pattern {p} matches no side-data finding",
                name(&addon)
            ));
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use swofty_formats::gvdb::Record;

    use super::*;
    use crate::ops::{DbOps, DbReport, apply_db_ops_report};
    use crate::rowkeys::{Names, RowKeys};

    const PACKAGES: &str = "entrants.vehicles.vehicle_packages";
    const PRESETS: &str = "car_setup.presets";

    fn stock() -> Gvdb {
        let package = format!("{PACKAGES}.vehicle_package_0");
        let records = vec![
            Record::new(&format!("{PACKAGES}._vehicle_package_instancecount_"), 0, "1"),
            Record::new(&format!("{package}._metadata_"), 4, "f1_2025"),
            Record::new(&format!("{package}.stat_id"), 0, "24"),
            Record::new(
                &format!("{package}.ai_package_folder"),
                2,
                "f1_2025_vehicle_package",
            ),
            Record::new(
                &format!("{package}.resource_path"),
                2,
                "f1_2025_vehicle_package/teams/[FILENAME]/wep/[FILENAME]",
            ),
            Record::new(
                &format!("{PRESETS}._vehicle_package_setups_instancecount_"),
                0,
                "1",
            ),
            Record::new(
                &format!("{PRESETS}.vehicle_package_setups_0._metadata_"),
                4,
                "f1_2025_vehicle_package",
            ),
            Record::new("entrants.vehicles._vehicle_instancecount_", 0, "1"),
            Record::new("entrants.vehicles.vehicle_0._metadata_", 4, "mercedes_2025"),
            Record::new("entrants.vehicles.vehicle_0.vehicle_package", 4, "f1_2025"),
            Record::new("entrants.drivers._driver_instancecount_", 0, "1"),
            Record::new("entrants.drivers.driver_0._metadata_", 4, "lando_norris"),
            Record::new("entrants.drivers.driver_0.head", 4, "lando_norris"),
            Record::new(
                "entrants.animation.driver_heads._driver_head_instancecount_",
                0,
                "2",
            ),
            Record::new(
                "entrants.animation.driver_heads.driver_head_0._metadata_",
                4,
                "lando_norris",
            ),
            Record::new(
                "entrants.animation.driver_heads.driver_head_1._metadata_",
                4,
                "male_ls212",
            ),
            Record::new(
                "entrants.expression_label._expression_label_instancecount_",
                0,
                "1",
            ),
            Record::new(
                "entrants.expression_label.expression_label_0._metadata_",
                4,
                "lando_norris",
            ),
            Record::new("bank.initial_team_cash._InitialTeamCash_instancecount_", 0, "2"),
            Record::new(
                "bank.initial_team_cash.InitialTeamCash_0._metadata_",
                4,
                "lando_norris",
            ),
            Record::new(
                "bank.initial_team_cash.InitialTeamCash_1._metadata_",
                4,
                "williams",
            ),
        ];
        Gvdb { records }
    }

    fn build_with(ops: &str, ignore: &[&str]) -> Report {
        let mut db = stock();
        let ops: DbOps = serde_json::from_str(ops).unwrap();
        let none = BTreeMap::new();
        let names = Names {
            fields: &none,
            keys: RowKeys::builtin(),
            stock: None,
            tracks: None,
        };
        let mut report = DbReport::default();
        apply_db_ops_report(&mut db, &ops, &names, &mut report).unwrap();
        let copies: Vec<Copy> = report
            .added_rows
            .into_iter()
            .map(|r| Copy {
                addon: "seasons".into(),
                from: r.from,
                row: r.row,
                ignore: ignore.iter().map(|p| (*p).to_owned()).collect(),
                quiet: false,
            })
            .collect();
        check(&db, &copies, &|id| format!("[{id}]"))
    }

    fn build(ops: &str) -> Vec<String> {
        build_with(ops, &[]).warnings
    }

    fn file(ops: &[&str]) -> String {
        format!(r#"{{"schema_version":1,"ops":[{}]}}"#, ops.join(","))
    }

    const PACKAGE_COPY: &str = r#"{ "op": "add-row", "table": "entrants.vehicles.vehicle_packages",
        "from": "entrants.vehicles.vehicle_packages.vehicle_package[_metadata_=f1_2025]",
        "set": { "_metadata_": "cs_2019", "resource_path": "cs_2019_vehicle_package/teams/[FILENAME]/wep/[FILENAME]" } }"#;

    const PRESET_COPY: &str = r#"{ "op": "add-row", "table": "car_setup.presets",
        "from": "car_setup.presets.vehicle_package_setups[_metadata_=f1_2025_vehicle_package]",
        "set": { "_metadata_": "cs_2019_vehicle_package" } }"#;

    const DRIVER_COPY: &str = r#"{ "op": "add-row", "table": "entrants.drivers",
        "from": "entrants.drivers.driver[_metadata_=lando_norris]",
        "set": { "_metadata_": "felipe_nasr", "head": "male_ls212" } }"#;

    #[test]
    fn a_relabelled_package_copy_lacks_its_setup_presets() {
        let report = build_with(&file(&[PACKAGE_COPY]), &[]);
        let presets = report
            .warnings
            .iter()
            .find(|w| w.contains("car_setup.presets.vehicle_package_setups has a row"))
            .unwrap_or_else(|| panic!("{report:#?}"));
        assert!(presets.starts_with("[seasons]: "), "{presets}");
        assert!(
            presets.contains("labelled f1_2025_vehicle_package and none labelled cs_2019_vehicle_package"),
            "{presets}"
        );
        assert!(presets.contains("folder 1 of resource_path"), "{presets}");
        assert!(
            presets.contains("Car_Load_SetupPreset (0x140c04fd0)"),
            "{presets}"
        );
        assert!(
            presets.contains(
                "from car_setup.presets.vehicle_package_setups[_metadata_=f1_2025_vehicle_package]"
            ),
            "{presets}"
        );
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.starts_with("[seasons]: [known] car_setup.presets")),
            "{report:#?}"
        );
        let refs = report
            .findings
            .iter()
            .find(|w| w.contains("pick the add-row source"))
            .unwrap_or_else(|| panic!("{report:#?}"));
        assert!(
            refs.contains(
                "[review] 1 value(s) pick the add-row source entrants.vehicles.vehicle_packages.vehicle_package[_metadata_=f1_2025] by its label and none pick the copy cs_2019: entrants.vehicles.vehicle_N.vehicle_package (1)"
            ),
            "{refs}"
        );
        assert!(
            !report
                .warnings
                .iter()
                .any(|w| w.contains("pick the add-row source")),
            "{report:#?}"
        );
    }

    #[test]
    fn a_preset_row_for_the_copy_clears_the_warning() {
        let report = build_with(&file(&[PACKAGE_COPY, PRESET_COPY]), &[]);
        assert!(
            !report
                .findings
                .iter()
                .any(|w| w.contains("car_setup.presets.vehicle_package_setups has")),
            "{report:#?}"
        );
        assert!(
            !report.findings.iter().any(|w| w.contains("ai_package_folder")),
            "the package copy still names f1_2025_vehicle_package by ai_package_folder: {report:#?}"
        );
    }

    #[test]
    fn a_kept_derived_value_keeps_its_rows_reachable() {
        let kept = r#"{ "op": "add-row", "table": "entrants.vehicles.vehicle_packages",
            "from": "entrants.vehicles.vehicle_packages.vehicle_package[_metadata_=f1_2025]",
            "set": { "_metadata_": "cs_2019" } }"#;
        let report = build_with(&file(&[kept]), &[]);
        assert!(
            !report.findings.iter().any(|w| w.contains("car_setup.presets")),
            "{report:#?}"
        );
    }

    #[test]
    fn name_matches_without_a_known_reader_are_counted_once_per_addon() {
        let report = build_with(&file(&[DRIVER_COPY]), &[]);
        let wear = report
            .findings
            .iter()
            .find(|f| f.contains("entrants.expression_label.expression_label has a row"))
            .unwrap_or_else(|| panic!("{report:#?}"));
        assert!(wear.starts_with("[seasons]: [review] "), "{wear}");
        assert!(
            !report
                .findings
                .iter()
                .any(|f| f.contains("entrants.animation.driver_heads")),
            "the copy's head male_ls212 has a driver_head row: {report:#?}"
        );
        let bank = report
            .findings
            .iter()
            .find(|f| f.contains("bank.initial_team_cash"))
            .unwrap_or_else(|| panic!("{report:#?}"));
        assert!(bank.starts_with("[seasons]: [career] "), "{bank}");
        assert_eq!(report.warnings.len(), 1, "{report:#?}");
        assert!(
            report.warnings[0].starts_with("[seasons]: 1 side-data finding(s) to review")
                && report.warnings[0].contains("entrants.expression_label.expression_label")
                && report.warnings[0].contains("f1mp build --side-data"),
            "{report:#?}"
        );
    }

    #[test]
    fn ignore_patterns_acknowledge_findings() {
        let report = build_with(
            &file(&[DRIVER_COPY]),
            &["entrants.expression_label.expression_label", "bank.*"],
        );
        assert!(report.warnings.is_empty(), "{report:#?}");
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.starts_with("[seasons]: [ignored] entrants.expression_label")),
            "{report:#?}"
        );
        let stale = build_with(&file(&[DRIVER_COPY]), &["bank.*", "tracks.*"]);
        assert!(
            stale
                .warnings
                .iter()
                .any(|w| w == "[seasons]: side_data.ignore pattern tracks.* matches no side-data finding"),
            "{stale:#?}"
        );
        let known = build_with(&file(&[PACKAGE_COPY]), &["car_setup.presets.*"]);
        assert!(
            !known.warnings.iter().any(|w| w.contains("car_setup.presets")),
            "{known:#?}"
        );
    }

    #[test]
    fn a_copy_into_an_area_brings_it_into_reach() {
        let cash = r#"{ "op": "add-row", "table": "bank.initial_team_cash",
            "from": "bank.initial_team_cash.InitialTeamCash[_metadata_=williams]",
            "set": { "_metadata_": "williams_b" } }"#;
        let report = build_with(&file(&[DRIVER_COPY, cash]), &[]);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.starts_with("[seasons]: [review] bank.initial_team_cash")),
            "{report:#?}"
        );
    }

    #[test]
    fn a_copy_under_its_source_label_brings_everything() {
        let same = r#"{ "op": "add-row", "table": "entrants.vehicles.vehicle_packages",
            "from": "entrants.vehicles.vehicle_packages.vehicle_package_0", "set": {} }"#;
        assert_eq!(build(&file(&[same])), Vec::<String>::new());
    }

    #[test]
    fn plain_paths_drop_row_numbers_and_selectors() {
        assert_eq!(
            plain("accolades.AccoladeDataGroup_N.driver_data.AccoladeDriver_N"),
            "accolades.AccoladeDataGroup.driver_data.AccoladeDriver"
        );
        assert_eq!(
            plain("accolades.AccoladeDataGroup[_metadata_=f1_career_2025].team_data.AccoladeTeam_3"),
            "accolades.AccoladeDataGroup.team_data.AccoladeTeam"
        );
        assert_eq!(plain("{e204b954fab7202d}0.vehicle"), "{e204b954fab7202d}.vehicle");
        assert!(ignores(
            "bank.*",
            &["bank.initial_team_cash.InitialTeamCash".into()]
        ));
        assert!(ignores(
            "entrants.vehicles.vehicle_packages.vehicle_package_N.ai_package_folder",
            &["entrants.vehicles.vehicle_packages.vehicle_package_N.ai_package_folder".into()]
        ));
        assert!(!ignores(
            "bank",
            &["bank.initial_team_cash.InitialTeamCash".into()]
        ));
        assert_eq!(area("career.my_team_sponsors.x"), Some("career"));
        assert_eq!(area("careers"), None);
        assert_eq!(area("{2985d484cb6942f1}.ChapterContent_N"), Some("unnamed"));
        assert_eq!(area("engine_management.driver_wear_rates"), Some("career"));
        assert_eq!(area("pit_lane.pit_times"), None);
    }

    #[test]
    fn derived_names_skip_generic_folders() {
        let cat = Catalog::from_gvdb(stock(), String::new());
        let row = cat
            .find("entrants.vehicles.vehicle_packages.vehicle_package_0")
            .unwrap();
        let names: Vec<(String, String)> = derived(&cat, row)
            .into_iter()
            .map(|d| (d.how(), d))
            .map(|(h, d)| (d.value, h))
            .collect();
        assert!(names.contains(&(
            "f1_2025_vehicle_package".into(),
            "folder 1 of resource_path".into()
        )));
        assert!(names.contains(&(
            "f1_2025_vehicle_package".into(),
            "the label + _vehicle_package".into()
        )));
        assert!(names.contains(&(
            "f1_2025_vehicle_package".into(),
            "the ai_package_folder value".into()
        )));
        assert!(
            !names.iter().any(|(v, _)| v == "teams" || v == "wep"),
            "{names:?}"
        );
        let d = deps(&cat, row);
        let presets = d
            .keyed
            .iter()
            .find(|k| k.family.starts_with("car_setup.presets"))
            .unwrap_or_else(|| panic!("{d:#?}"));
        assert!(presets.rule.is_some(), "{d:#?}");
    }
}
