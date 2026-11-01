use crate::ops::{DbOp, DbOps};
use std::path::Path;

use serde::{Deserialize, Serialize};
use swofty_formats::hash::{fnv1a64, fnv1a64_from};

use super::Catalog;
use super::rows::{self, Copied, METADATA, RowCopy};
use crate::{Error, Result, io_err};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Origin {
    #[default]
    Typed,
    /// Imported from this file name.
    Imported(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edit {
    /// The record hash, written `0x` and 16 hex digits.
    #[serde(with = "hex_key")]
    pub key: u64,
    /// The name at edit time, for display.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Label selector of the row and the field under it, when the row has a unique label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub select: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(rename = "type")]
    pub tag: u8,
    /// `None`: the key is removed.
    pub value: Option<String>,
    /// `None`: the key is not in the stock database (added).
    pub stock: Option<String>,
    #[serde(default)]
    pub origin: Origin,
}

mod hex_key {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(key: &u64, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!("0x{key:016x}"))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
        let s = String::deserialize(d)?;
        super::super::catalog::parse_key(&s)
            .ok_or_else(|| serde::de::Error::custom(format!("bad record key {s}; want 0x and 16 hex digits")))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Edits {
    pub schema_version: u32,
    pub edits: Vec<Edit>,
    /// Rows added as copies of other rows, in the order they were added.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rows: Vec<RowCopy>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ImportReport {
    /// `swofty`, `ego`, `package` or `bin`.
    pub format: &'static str,
    pub imported: usize,
    pub unchanged: usize,
    pub skipped: Vec<String>,
}

pub const EXPORT_FORMAT: &str = "swofty-db-edits";

#[derive(Serialize)]
struct Export<'a> {
    format: &'static str,
    format_version: u32,
    game_build: &'a str,
    schema_version: u32,
    edits: &'a [Edit],
    #[serde(skip_serializing_if = "<[RowCopy]>::is_empty")]
    rows: &'a [RowCopy],
}

/// Checks a value's text against its type tag; [`validate_like`] with no stock value.
pub fn validate(tag: u8, value: &str) -> std::result::Result<(), String> {
    validate_like(tag, None, value)
}

/// Checks a value's text against its type tag and, when given, the stock
/// value's shape: vectors keep their length, lists keep unpadded items unless
/// stock pads them, and values stock writes off-grammar must keep its shape.
pub fn validate_like(tag: u8, stock: Option<&str>, value: &str) -> std::result::Result<(), String> {
    if value.contains('\0') {
        return Err("values cannot hold a NUL character".into());
    }
    let same_shape = || stock.is_some_and(|s| separators(s) == separators(value));
    let check = |ok: bool, why: String| if ok { Ok(()) } else { Err(why) };
    match tag {
        0 | 12 => {
            let e_form = tag == 0 && stock.is_some_and(|s| !int(s) && whole(s).is_some());
            check(
                int(value) || (e_form && whole(value).is_some()),
                format!("{value:?} is not a whole number from -2147483648 to 2147483647"),
            )
        }
        1 => check(
            float(value) || (stock.is_some_and(|s| !float(s)) && same_shape()),
            format!("{value:?} is not a number"),
        ),
        5 | 6 => check(
            matches!(value, "True" | "False" | "true" | "false"),
            format!("{value:?} must be True or False"),
        ),
        7 => check(
            value
                .split(',')
                .enumerate()
                .all(|(i, p)| int(p.strip_prefix(' ').filter(|_| i > 0).unwrap_or(p))),
            format!("{value:?}: write whole numbers joined with commas"),
        ),
        8 => check(
            value.split(',').all(float),
            format!("{value:?}: write numbers joined with commas"),
        ),
        9 => match curve(value) {
            Err(_)
                if stock.is_some_and(|s| !s.starts_with('@'))
                    && same_shape()
                    && value.split(',').all(float) =>
            {
                Ok(())
            }
            r => r.map_err(|e| format!("{value:?} is not a curve: {e}")),
        },
        11 => check(
            value.is_empty() || int(value),
            format!("{value:?} is not a colour; write a signed 32-bit ARGB number"),
        ),
        13 => {
            let padded = |v: &str| v.split(',').any(|p| p.trim() != p);
            check(
                !padded(value) || stock.is_some_and(padded),
                format!("{value:?}: join list items with commas and no spaces around them"),
            )
        }
        14 => {
            let want = stock.map_or(3, |s| s.split(',').count());
            check(
                value.split(',').count() == want && value.split(',').all(float),
                format!("{value:?}: write {want} numbers joined with commas"),
            )
        }
        17 => check(
            stock.is_none() || same_shape(),
            format!("{value:?} does not match the shape of the stock value"),
        ),
        18 => check(
            value.is_empty()
                || value
                    .split(',')
                    .all(|p| !p.is_empty() && !p.contains(char::is_whitespace)),
            format!("{value:?}: join names with commas and no spaces"),
        ),
        _ => Ok(()),
    }
}

/// The text of a boolean in the stock value's style: `True`/`False` or `true`/`false`.
pub fn bool_text(stock: &str, on: bool) -> String {
    let lower = stock.starts_with(|c: char| c.is_ascii_lowercase());
    match (lower, on) {
        (true, true) => "true",
        (true, false) => "false",
        (false, true) => "True",
        (false, false) => "False",
    }
    .into()
}

fn int(s: &str) -> bool {
    let digits = s.strip_prefix('-').unwrap_or(s);
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) && s.parse::<i32>().is_ok()
}

/// A decimal number as the game writes them: `-1`, `17.5`, `.5`, `13.`, `1E+07`, `3.4e-05`.
fn number(s: &str) -> Option<f64> {
    let body = s.strip_prefix('-').unwrap_or(s);
    let (mant, exp) = match body.find(['e', 'E']) {
        Some(i) => (&body[..i], Some(&body[i + 1..])),
        None => (body, None),
    };
    let (int_part, frac) = mant.split_once('.').unwrap_or((mant, ""));
    let digits = |d: &str| d.bytes().all(|b| b.is_ascii_digit());
    let exp_ok = exp.is_none_or(|e| {
        let e = e.strip_prefix(['+', '-']).unwrap_or(e);
        !e.is_empty() && digits(e)
    });
    (digits(int_part) && digits(frac) && !(int_part.is_empty() && frac.is_empty()) && exp_ok)
        .then(|| s.parse::<f64>().ok())
        .flatten()
        .filter(|f| f.is_finite())
}

fn float(s: &str) -> bool {
    number(s).is_some() && s.parse::<f32>().is_ok_and(f32::is_finite)
}

fn whole(s: &str) -> Option<i32> {
    number(s)
        .filter(|f| f.fract() == 0.0 && *f >= i32::MIN as f64 && *f <= i32::MAX as f64)
        .map(|f| f as i32)
}

/// The separator characters of a value in order, for comparing shapes.
fn separators(s: &str) -> String {
    s.chars().filter(|c| ",|:;@/ ".contains(*c)).collect()
}

/// `@v2@minx:<n>,maxx:<n>,miny:<n>,maxy:<n>,<x>:<y>,…` with at least two points.
fn curve(value: &str) -> std::result::Result<(), String> {
    let body = value.strip_prefix("@v2@").ok_or("curves start with @v2@minx:")?;
    let mut parts = body.split(',');
    for name in ["minx", "maxx", "miny", "maxy"] {
        let p = parts.next().unwrap_or_default();
        let n = p
            .strip_prefix(name)
            .and_then(|r| r.strip_prefix(':'))
            .ok_or_else(|| format!("expected {name}:<number>, found {p:?}"))?;
        float(n)
            .then_some(())
            .ok_or_else(|| format!("{name} {n:?} is not a number"))?;
    }
    let mut points = 0;
    for p in parts {
        let ok = p.split_once(':').is_some_and(|(x, y)| float(x) && float(y));
        ok.then_some(())
            .ok_or_else(|| format!("point {p:?} is not <x>:<y>"))?;
        points += 1;
    }
    (points >= 2)
        .then_some(())
        .ok_or_else(|| "a curve needs at least two x:y points".into())
}

/// Equal texts, or the same boolean written in another case.
fn same_value(tag: u8, a: &str, b: &str) -> bool {
    a == b || (matches!(tag, 5 | 6) && a.eq_ignore_ascii_case(b))
}

fn key_ref(e: &Edit) -> String {
    match &e.path {
        Some(p) if !p.contains(['{', '[']) && fnv1a64(p) == e.key => p.clone(),
        _ => format!("0x{:016x}", e.key),
    }
}

fn hex(value: &str) -> String {
    value.bytes().map(|b| format!("{b:02x}")).collect()
}

/// The path under `row` of an edit of one of its keys.
fn owner<'a>(row: &RowCopy, e: &'a Edit) -> Option<&'a str> {
    let rel = row.rel(e.path.as_deref()?)?;
    (e.stock.is_none() && row.key(rel) == e.key).then_some(rel)
}

impl Edit {
    /// Whether this edits a key of a copied row: named by the row's selector, not in the stock database.
    pub fn is_copied(&self) -> bool {
        self.stock.is_none()
            && self
                .path
                .as_deref()
                .is_some_and(|p| p.contains('[') && fnv1a64(p) == self.key)
    }
}

impl Edits {
    fn empty() -> Self {
        Edits {
            schema_version: 1,
            ..Edits::default()
        }
    }

    /// Reads an edits file; a missing file holds no edits. A file that does not
    /// parse stays in place and fails with [`Error::Json`] naming it.
    pub fn read(path: &Path) -> Result<Self> {
        match std::fs::read(path) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|source| Error::Json {
                path: path.into(),
                source,
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::empty()),
            Err(e) => Err(io_err(path)(e)),
        }
    }

    /// Replaces the file whole and durably, through a temporary name unique to
    /// this process and call.
    pub fn write(&self, path: &Path) -> Result<()> {
        crate::write_json_shared(path, self)
    }

    /// Changed keys and copied rows.
    pub fn len(&self) -> usize {
        self.edits.len() + self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.edits.is_empty() && self.rows.is_empty()
    }

    pub fn get(&self, key: u64) -> Option<&Edit> {
        self.edits.iter().find(|e| e.key == key)
    }

    /// The value in effect for `key`: the edit, else the stock value, else the
    /// value a copied row takes from its source row.
    pub fn value<'a>(&'a self, catalog: &'a Catalog, key: u64) -> Option<&'a str> {
        match self.get(key) {
            Some(e) => e.value.as_deref(),
            None => catalog
                .record_by_key(key)
                .map(|r| r.value)
                .or_else(|| catalog.value_of(self.copied(catalog, key)?.source)),
        }
    }

    /// The copied row and source key behind `key`, a key of a copied row.
    pub fn copied<'a>(&'a self, catalog: &Catalog, key: u64) -> Option<Copied<'a>> {
        self.rows.iter().find_map(|row| {
            let src = rows::source_row(catalog, row)?;
            rows::row_keys(catalog, src)
                .into_iter()
                .find(|(rel, _)| row.key(rel) == key)
                .map(|(rel, source)| Copied {
                    row,
                    rel,
                    key,
                    source,
                })
        })
    }

    /// The copied-row key a selector path names, such as
    /// `…DriverStatisticsHolder[driver=romain_grosjean].StatisticsDataSet[_metadata_=default].Statistic[_metadata_=pace].evolution`.
    pub fn copied_path<'a>(&'a self, catalog: &Catalog, path: &str) -> Option<Copied<'a>> {
        let path = path.trim();
        self.rows.iter().find_map(|row| {
            let tail = row.rel(path)?;
            let src = rows::source_row(catalog, row)?;
            let source = catalog.state(Some(src), tail).ok()?;
            catalog.record_by_key(source)?;
            let rel = rows::rel_of(catalog, src, source)?;
            Some(Copied {
                row,
                key: row.key(&rel),
                rel,
                source,
            })
        })
    }

    /// The key a path names: a stock key, else a key of a copied row.
    pub fn resolve(&self, catalog: &Catalog, path: &str) -> std::result::Result<u64, String> {
        catalog.resolve(path).or_else(|e| {
            self.copied_path(catalog, path)
                .map(|c| c.key)
                .ok_or(e.to_string())
        })
    }

    /// The copy of `table` whose `field` is `to`.
    pub fn row(&self, table: &str, field: &str, to: &str) -> Option<&RowCopy> {
        self.rows.iter().find(|r| r.is(table, field, to))
    }

    /// Adds a row to `table` for `to`: a copy of the stock row whose `field` is
    /// `from`, or of the game default row, the one whose `field` is empty.
    /// The copy takes this draft's changes to the source row.
    pub fn copy_row(
        &mut self,
        catalog: &Catalog,
        table: &str,
        field: &str,
        from: Option<&str>,
        to: &str,
    ) -> std::result::Result<(), String> {
        let row = self.new_row(catalog, table, field, from, to)?;
        let src = rows::source_row(catalog, &row).expect("new_row found it");
        let changed: Vec<(String, String)> = rows::row_keys(catalog, src)
            .into_iter()
            .filter(|(rel, _)| *rel != row.field && rel != METADATA)
            .filter_map(|(rel, k)| Some((rel, self.get(k)?.value.clone()?)))
            .collect();
        self.rows.push(row);
        for (rel, value) in changed {
            let key = self.rows.last().expect("pushed").key(&rel);
            self.set(catalog, key, &value).ok();
        }
        Ok(())
    }

    fn new_row(
        &self,
        catalog: &Catalog,
        table: &str,
        field: &str,
        from: Option<&str>,
        to: &str,
    ) -> std::result::Result<RowCopy, String> {
        let (table, field, to) = (table.trim(), field.trim(), to.trim());
        rows::check_table(catalog, table, field)?;
        if to.is_empty() || to.contains(['[', ']']) {
            return Err(format!("{to:?} can't name a row"));
        }
        let from = from.map(str::trim).filter(|f| !f.is_empty());
        if from == Some(to) {
            return Err(format!("{to} can't copy its own row"));
        }
        if self.row(table, field, to).is_some() || !rows::matching(catalog, table, field, to).is_empty() {
            return Err(format!("{to} already has its own row in {table}"));
        }
        let row = RowCopy {
            table: table.to_owned(),
            field: field.to_owned(),
            from: from.map(str::to_owned),
            to: to.to_owned(),
            label: None,
        };
        let Some(src) = rows::source_row(catalog, &row) else {
            let found = rows::matching(catalog, table, field, from.unwrap_or("")).len();
            return Err(match (from, found) {
                (Some(f), 0) => format!("{table} has no row whose {field} is {f}"),
                (None, 0) => format!("{table} has no game default row"),
                (Some(f), n) => format!("{n} rows of {table} have {field} {f}"),
                (None, n) => format!("{n} rows of {table} have no {field}; none is the game default"),
            });
        };
        let label = (field != METADATA && catalog.value_of(fnv1a64_from(src, b"._metadata_")).is_some())
            .then(|| rows::free_label(catalog, table, to, &self.rows));
        Ok(RowCopy { label, ..row })
    }

    /// Removes the copy of `table` whose `field` is `to`, with every change to it.
    pub fn remove_row(&mut self, table: &str, field: &str, to: &str) -> bool {
        let Some(i) = self.rows.iter().position(|r| r.is(table, field, to)) else {
            return false;
        };
        let row = self.rows.remove(i);
        self.edits.retain(|e| owner(&row, e).is_none());
        true
    }

    fn upsert(&mut self, edit: Edit) {
        match self.edits.iter_mut().find(|e| e.key == edit.key) {
            Some(e) => *e = edit,
            None => self.edits.push(edit),
        }
    }

    fn named(catalog: &Catalog, key: u64, tag: u8, value: Option<String>, stock: Option<String>) -> Edit {
        let (select, field) = catalog.selector(key).unzip();
        Edit {
            key,
            path: catalog.node_of_key(key).map(|n| catalog.path(n)),
            select,
            field,
            tag,
            value,
            stock,
            origin: Origin::Typed,
        }
    }

    /// Sets an existing key; a value equal to stock, or to the source row's
    /// for a copied row, drops the edit.
    pub fn set(&mut self, catalog: &Catalog, key: u64, value: &str) -> std::result::Result<(), String> {
        if catalog.record_by_key(key).is_none()
            && let Some(c) = self.copied(catalog, key)
        {
            let base = catalog.record_by_key(c.source).expect("a stock key");
            let path = c.row.path(&c.rel);
            if same_value(base.tag, base.value, value) {
                self.revert(key);
                return Ok(());
            }
            validate_like(base.tag, Some(base.value), value)?;
            self.upsert(Edit {
                key,
                path: Some(path),
                select: None,
                field: None,
                tag: base.tag,
                value: Some(value.to_owned()),
                stock: None,
                origin: Origin::Typed,
            });
            return Ok(());
        }
        let Some(stock) = catalog.record_by_key(key) else {
            let Some(e) = self.edits.iter_mut().find(|e| e.key == key && e.stock.is_none()) else {
                return Err(format!("0x{key:016x} is not in the game database"));
            };
            validate_like(e.tag, None, value)?;
            e.value = Some(value.to_owned());
            e.origin = Origin::Typed;
            return Ok(());
        };
        if same_value(stock.tag, stock.value, value) {
            self.revert(key);
            return Ok(());
        }
        validate_like(stock.tag, Some(stock.value), value)?;
        let edit = Self::named(
            catalog,
            key,
            stock.tag,
            Some(value.into()),
            Some(stock.value.into()),
        );
        self.upsert(edit);
        Ok(())
    }

    /// Adds a key the stock database lacks; returns its hash.
    pub fn add(
        &mut self,
        catalog: &Catalog,
        path: &str,
        tag: u8,
        value: &str,
    ) -> std::result::Result<u64, String> {
        let key = catalog.resolve(path).map_err(|e| e.to_string())?;
        if catalog.record_by_key(key).is_some() {
            return Err(format!(
                "{path} is already in the game database; change its value instead"
            ));
        }
        validate(tag, value)?;
        self.upsert(Edit {
            key,
            path: Some(path.trim().to_owned()),
            select: None,
            field: None,
            tag,
            value: Some(value.into()),
            stock: None,
            origin: Origin::Typed,
        });
        Ok(key)
    }

    pub fn remove_key(&mut self, catalog: &Catalog, key: u64) -> std::result::Result<(), String> {
        if catalog.record_by_key(key).is_none() && self.copied(catalog, key).is_some() {
            return Err("a copied row keeps every key of its source row; remove the row instead".into());
        }
        let Some(stock) = catalog.record_by_key(key) else {
            return match self.revert(key) {
                true => Ok(()),
                false => Err(format!("0x{key:016x} is not in the game database")),
            };
        };
        let edit = Self::named(catalog, key, stock.tag, None, Some(stock.value.into()));
        self.upsert(edit);
        Ok(())
    }

    /// An edit taking a modded database's record as it is, without validation;
    /// a changed type tag drops the selector so the record is written whole.
    pub(crate) fn imported(
        catalog: &Catalog,
        key: u64,
        tag: u8,
        value: Option<String>,
        path: Option<String>,
        origin: &Origin,
    ) -> Edit {
        let mut edit = match catalog.record_by_key(key) {
            Some(s) => {
                let mut e = Self::named(catalog, key, s.tag, value, Some(s.value.into()));
                if tag != s.tag {
                    (e.tag, e.select, e.field) = (tag, None, None);
                }
                e
            }
            None => Edit {
                key,
                path,
                select: None,
                field: None,
                tag,
                value,
                stock: None,
                origin: Origin::Typed,
            },
        };
        edit.origin = origin.clone();
        edit
    }

    pub fn revert(&mut self, key: u64) -> bool {
        let before = self.edits.len();
        self.edits.retain(|e| e.key != key);
        self.edits.len() != before
    }

    pub fn clear(&mut self) {
        self.edits.clear();
        self.rows.clear();
    }

    /// The database operations the build applies after every addon: the
    /// copied rows with their changes, then the other changes, relabels last.
    pub fn to_ops(&self) -> DbOps {
        let mut ops: Vec<DbOp> = self
            .rows
            .iter()
            .filter_map(|row| {
                let values = self
                    .edits
                    .iter()
                    .filter_map(|e| Some((owner(row, e)?.to_owned(), e.value.clone()?)))
                    .collect();
                row.op(values)
            })
            .collect();
        let relabels = |e: &&Edit| e.field.as_deref() == Some("_metadata_");
        let others = self.edits.iter().filter(|e| !e.is_copied());
        let ordered = others
            .clone()
            .filter(|e| !relabels(e))
            .chain(others.filter(relabels));
        ops.extend(ordered.map(|e| match (&e.value, &e.stock, &e.select, &e.field) {
            (None, _, _, _) => DbOp::RemoveRecord {
                key: key_ref(e),
                allow_stock_removal: false,
            },
            (Some(v), Some(_), Some(select), Some(field)) if field != "_metadata_" => DbOp::Set {
                select: select.clone(),
                field: field.clone(),
                value: Some(v.clone()),
                value_from: None,
                datasets: Vec::new(),
                create: false,
                tag: None,
            },
            (Some(v), _, _, _) => DbOp::PutRecord {
                key: key_ref(e),
                tag: e.tag,
                value: hex(v),
            },
        }));
        DbOps {
            strict: false,
            side_data: Default::default(),
            schema_version: 1,
            fields: Default::default(),
            ops,
        }
    }

    pub fn export(&self, catalog: &Catalog) -> String {
        serde_json::to_string_pretty(&Export {
            format: EXPORT_FORMAT,
            format_version: 1,
            game_build: catalog.build_id(),
            schema_version: 1,
            edits: &self.edits,
            rows: &self.rows,
        })
        .expect("serialisable")
    }

    /// Merges a Swofty export or an EGO GameDatabases Editor export; incoming values win.
    pub fn import(
        &mut self,
        catalog: &Catalog,
        bytes: &[u8],
        file_name: &str,
    ) -> std::result::Result<ImportReport, String> {
        let doc: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|e| format!("{file_name} is not JSON: {e}"))?;
        let format = doc
            .get("format")
            .and_then(|f| f.as_str())
            .unwrap_or(EXPORT_FORMAT);
        let mut report = ImportReport::default();
        let origin = Origin::Imported(file_name.to_owned());
        match format {
            super::ego::FORMAT => {
                report.format = "ego";
                for (key, value) in super::ego::modified(doc, file_name)? {
                    self.import_value(catalog, key, &value, None, &origin, &mut report);
                }
            }
            EXPORT_FORMAT | super::package::FORMAT if doc.get("edits").is_some() => {
                report.format = "swofty";
                let edits: Vec<Edit> =
                    serde_json::from_value(doc["edits"].clone()).map_err(|e| format!("{file_name}: {e}"))?;
                let rows: Vec<RowCopy> = match doc.get("rows") {
                    Some(r) => serde_json::from_value(r.clone()).map_err(|e| format!("{file_name}: {e}"))?,
                    None => Vec::new(),
                };
                for r in rows {
                    self.import_row(catalog, r, &mut report);
                }
                for e in edits {
                    match (&e.value, catalog.record_by_key(e.key)) {
                        (Some(v), _) => self.import_value(catalog, e.key, v, Some(&e), &origin, &mut report),
                        (None, Some(_)) => {
                            let was = self.get(e.key).map(|x| x.value.is_none());
                            self.remove_key(catalog, e.key)
                                .expect("the key is in the database");
                            match was {
                                Some(true) => report.unchanged += 1,
                                _ => report.imported += 1,
                            }
                        }
                        (None, None) => report.skipped.push(format!(
                            "{}: not in this game's database",
                            e.path.unwrap_or_else(|| format!("0x{:016x}", e.key))
                        )),
                    }
                }
            }
            other => return Err(format!("{file_name}: unknown format {other:?}")),
        }
        Ok(report)
    }

    fn import_row(&mut self, catalog: &Catalog, r: RowCopy, report: &mut ImportReport) {
        if let Some(have) = self.rows.iter_mut().find(|h| h.is(&r.table, &r.field, &r.to)) {
            match have.from == r.from {
                true => report.unchanged += 1,
                false => {
                    have.from = r.from;
                    report.imported += 1;
                }
            }
            return;
        }
        match self.new_row(catalog, &r.table, &r.field, r.from.as_deref(), &r.to) {
            Ok(row) => {
                self.rows.push(row);
                report.imported += 1;
            }
            Err(why) => report.skipped.push(format!("{}: {why}", r.select())),
        }
    }

    fn import_value(
        &mut self,
        catalog: &Catalog,
        key: u64,
        value: &str,
        from: Option<&Edit>,
        origin: &Origin,
        report: &mut ImportReport,
    ) {
        let name = || {
            from.and_then(|e| e.path.clone())
                .or_else(|| catalog.node_of_key(key).map(|n| catalog.path(n)))
                .unwrap_or_else(|| format!("0x{key:016x}"))
        };
        let tag = catalog.record_by_key(key).map(|r| r.tag).or(from.map(|e| e.tag));
        if self
            .value(catalog, key)
            .is_some_and(|v| tag.is_some_and(|t| same_value(t, v, value)))
        {
            report.unchanged += 1;
            return;
        }
        let result = match (catalog.record_by_key(key), from) {
            (Some(_), _) => self.set(catalog, key, value),
            (None, Some(e)) if e.stock.is_none() => validate_like(e.tag, None, value).map(|()| {
                self.upsert(Edit {
                    origin: Origin::Typed,
                    ..e.clone()
                })
            }),
            (None, _) => Err("not in this game's database".into()),
        };
        match result {
            Ok(()) => {
                if let Some(e) = self.edits.iter_mut().find(|e| e.key == key) {
                    e.origin = origin.clone();
                }
                report.imported += 1;
            }
            Err(why) => report.skipped.push(format!("{}: {why}", name())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dbedit::catalog::tests::{real, sample};

    fn cat() -> Catalog {
        Catalog::load(&sample()).unwrap()
    }

    #[test]
    fn validates_values_by_type() {
        fn ok(tag: u8, stock: Option<&str>, v: &str) -> bool {
            validate_like(tag, stock, v).is_ok()
        }
        assert!(ok(0, None, "-12") && ok(0, None, "2147483647"));
        for bad in ["1.5", "2147483648", "+1", " 1", "", "1E+07"] {
            assert!(!ok(0, None, bad), "int {bad:?}");
        }
        assert!(ok(0, Some("1.5E+07"), "2E+07") && !ok(0, Some("1.5E+07"), "1.5E+00"));
        for good in ["1.5", "-0.25", ".5", "13.", "1E+07", "3.4e-05", "17"] {
            assert!(ok(1, None, good), "float {good:?}");
        }
        for bad in ["fast", "1e39", "inf", "NaN", " 1", "1.5.", "-", ".", "1e"] {
            assert!(!ok(1, None, bad), "float {bad:?}");
        }
        let level = "Level_0|DevelopmentLevel|facilities.facilities.development_levels";
        assert!(ok(1, Some(level), &level.replace("_0", "_2")));
        assert!(!ok(1, Some(level), "Level_0|DevelopmentLevel") && !ok(1, Some("1.5"), level));
        for good in ["True", "False", "true", "false"] {
            assert!(ok(5, None, good) && ok(6, Some("false"), good));
        }
        assert!(!ok(5, None, "yes") && !ok(5, None, "TRUE") && !ok(6, Some("false"), "14,30"));
        assert!(ok(7, None, "06,45") && ok(7, None, "57, 57, 57") && ok(7, None, "-20,-10,0"));
        assert!(
            !ok(7, None, "6,4.5") && !ok(7, None, " 1,2") && !ok(7, None, "1 ,2") && !ok(7, None, "1,,2")
        );
        assert!(ok(8, None, "0.31,0.4,.99") && ok(8, None, "120.0"));
        assert!(!ok(8, None, "1, 2") && !ok(8, None, "1,x") && !ok(8, None, ""));
        let curve = "@v2@minx:0,maxx:100,miny:0,maxy:25,0:0.144,0.03:0.146,1:1";
        assert!(ok(9, None, curve) && ok(9, None, "@v2@minx:-1,maxx:1,miny:-1,maxy:1,0:1,1:0"));
        for bad in [
            "@v2@minx:0",
            "@v2@minx:0,maxx:1,miny:0,maxy:1,0:1",
            "@v2@minx:0,maxx:x,miny:0,maxy:1,0:1,1:1",
            "@v2@minx:0,maxx:1,miny:0,maxy:1,0:1,1",
            "@v2@maxx:0,minx:1,miny:0,maxy:1,0:1,1:1",
            "1,2,3,4",
        ] {
            assert!(!ok(9, None, bad), "curve {bad:?}");
        }
        assert!(ok(9, Some("1,0.9999996,1,1"), "1,1,1,0.5") && !ok(9, Some("1,1,1,1"), "1,1,1"));
        assert!(
            ok(11, None, "-13249833")
                && ok(11, None, "")
                && !ok(11, None, "#ff0000")
                && !ok(11, None, "4294967295")
        );
        assert!(ok(12, None, "32768") && !ok(12, None, "1,2"));
        assert!(ok(13, None, ",a,team badge") && !ok(13, None, "a, b") && !ok(13, None, "a ,b"));
        assert!(ok(
            13,
            Some(" DeptEvent_DriverTraining"),
            " DeptEvent_GymAccident02"
        ));
        assert!(ok(14, Some("0,-0.6,0"), "1.5,2,3.4e-05") && ok(14, None, "1,2,3"));
        assert!(!ok(14, Some("0,0,0"), "1,2") && !ok(14, None, "1,2,3,4") && !ok(14, None, "1, 2, 3"));
        assert!(ok(17, Some(""), "x") && !ok(17, Some(""), "a,b"));
        assert!(ok(18, None, "game_state,FrontEnd") && ok(18, None, ""));
        assert!(!ok(18, None, "a,,b") && !ok(18, None, "a, b"));
        assert!(validate(2, "anything at all").is_ok() && validate(2, "a\0b").is_err());
        assert_eq!(validate(9, "1,2"), validate_like(9, None, "1,2"));
    }

    #[test]
    fn bool_text_keeps_the_stock_case() {
        assert_eq!(bool_text("True", false), "False");
        assert_eq!(bool_text("False", true), "True");
        assert_eq!(bool_text("false", true), "true");
        assert_eq!(bool_text("true", false), "false");
    }

    #[test]
    fn every_stock_record_validates_against_its_own_tag() {
        let Some(cat) = real() else {
            eprintln!("F1_25_DIR is not set; skipping");
            return;
        };
        let mut per_tag = std::collections::BTreeMap::<u8, usize>::new();
        let mut failed = Vec::new();
        for i in 0..cat.records() as u32 {
            let r = cat.record(i);
            *per_tag.entry(r.tag).or_default() += 1;
            if let Err(e) = validate_like(r.tag, Some(r.value), r.value) {
                failed.push(format!("0x{:016x} (type {}): {e}", r.key, r.tag));
            }
        }
        eprintln!("{per_tag:?}");
        assert!(
            failed.is_empty(),
            "{} rejected:\n{}",
            failed.len(),
            failed[..failed.len().min(20)].join("\n")
        );
    }

    #[test]
    fn edits_files_write_whole_and_read_damage_as_an_error() {
        let dir = std::env::temp_dir().join(format!("swofty-dbedit-file-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        let path = dir.join("pkg/edits.json");
        assert!(Edits::read(&path).unwrap().is_empty());
        let cat = cat();
        let mut edits = Edits::empty();
        edits
            .set(&cat, fnv1a64("entrants.drivers.driver_0.car_number"), "33")
            .unwrap();
        std::thread::scope(|s| {
            for _ in 0..8 {
                s.spawn(|| {
                    for _ in 0..10 {
                        edits.write(&path).unwrap();
                    }
                });
            }
        });
        assert_eq!(Edits::read(&path).unwrap(), edits);
        let names: Vec<_> = std::fs::read_dir(path.parent().unwrap())
            .unwrap()
            .flatten()
            .map(|e| e.file_name())
            .collect();
        assert_eq!(names, ["edits.json"]);
        for damage in [&b""[..], b"\0\0\0\0", b"{\"schema_version\": 1, \"ed", b"[]"] {
            std::fs::write(&path, damage).unwrap();
            let err = Edits::read(&path).unwrap_err();
            assert!(matches!(&err, Error::Json { path: p, .. } if *p == path), "{err}");
            assert!(err.to_string().contains("edits.json"), "{err}");
            assert_eq!(std::fs::read(&path).unwrap(), damage, "the damaged file stays");
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn edits_round_trip_and_become_ops() {
        let cat = cat();
        let mut edits = Edits::empty();
        let num = fnv1a64("entrants.drivers.driver_0.car_number");
        assert!(edits.set(&cat, num, "x").is_err());
        edits.set(&cat, num, "33").unwrap();
        assert_eq!(edits.value(&cat, num), Some("33"));
        let e = edits.get(num).unwrap();
        assert_eq!(
            e.select.as_deref(),
            Some("entrants.drivers.driver[_metadata_=max_verstappen]")
        );
        assert_eq!(e.path.as_deref(), Some("entrants.drivers.driver_0.car_number"));
        assert_eq!(e.stock.as_deref(), Some("1"));
        edits.set(&cat, num, "1").unwrap();
        assert!(edits.is_empty());
        edits.set(&cat, num, "33").unwrap();
        let label = fnv1a64("entrants.drivers.driver_0._metadata_");
        edits.set(&cat, label, "max_33").unwrap();
        let added = edits
            .add(
                &cat,
                "entrants.drivers.driver[_metadata_=lando_norris].nickname",
                2,
                "Lando",
            )
            .unwrap();
        assert_eq!(added, fnv1a64("entrants.drivers.driver_1.nickname"));
        assert!(
            edits
                .add(&cat, "entrants.drivers.driver_1.car_number", 0, "4")
                .is_err()
        );
        let gone = fnv1a64("tracks.Track_0.PoleOnRight");
        edits.remove_key(&cat, gone).unwrap();
        let json = serde_json::to_string(&edits).unwrap();
        assert!(json.contains(&format!("\"key\":\"0x{num:016x}\"")), "{json}");
        assert!(json.contains("\"origin\":null"), "{json}");
        assert_eq!(serde_json::from_str::<Edits>(&json).unwrap(), edits);
        let ops = serde_json::to_value(edits.to_ops()).unwrap();
        let ops = ops["ops"].as_array().unwrap();
        assert_eq!(ops[0]["op"], "set");
        assert_eq!(ops[0]["field"], "car_number");
        assert_eq!(ops[1]["op"], "put-record");
        assert_eq!(ops[1]["key"], format!("0x{added:016x}"));
        assert_eq!(ops[2]["op"], "remove-record");
        assert_eq!(ops[2]["key"], "tracks.Track_0.PoleOnRight");
        assert_eq!(ops[3]["key"], "entrants.drivers.driver_0._metadata_");
        let mut db = swofty_formats::gvdb::Gvdb::parse(&sample()).unwrap();
        let names = std::collections::BTreeMap::new();
        crate::ops::apply_db_ops(&mut db, &edits.to_ops(), &names).unwrap();
        assert_eq!(
            db.get_text("entrants.drivers.driver_0.car_number").as_deref(),
            Some("33")
        );
        assert_eq!(
            db.get_text("entrants.drivers.driver_0._metadata_").as_deref(),
            Some("max_33")
        );
        assert_eq!(
            db.get_text("entrants.drivers.driver_1.nickname").as_deref(),
            Some("Lando")
        );
        assert!(db.get("tracks.Track_0.PoleOnRight").is_none());
        assert!(edits.revert(gone));
        assert!(!edits.revert(gone));
    }

    #[test]
    fn imports_exports_and_ego_files() {
        let cat = cat();
        let mut edits = Edits::empty();
        let num = fnv1a64("entrants.drivers.driver_1.car_number");
        edits.set(&cat, num, "44").unwrap();
        let out = edits.export(&cat);
        assert!(out.contains("\"format\": \"swofty-db-edits\""), "{out}");
        assert!(out.contains(cat.build_id()));
        let mut other = Edits::empty();
        let r = other.import(&cat, out.as_bytes(), "mine.json").unwrap();
        assert_eq!((r.format, r.imported, r.unchanged), ("swofty", 1, 0));
        assert_eq!(
            other.get(num).unwrap().origin,
            Origin::Imported("mine.json".into())
        );
        assert_eq!(
            other.import(&cat, out.as_bytes(), "mine.json").unwrap().unchanged,
            1
        );
        let lap = fnv1a64("entrants.drivers.driver_1.driver_strategy.safety_car_pit_min_lap");
        let ego = serde_json::json!({"format": "ego-gamedatabases-editor-records", "format_version": 1,
            "records": {"modified_records": {
                lap.to_string(): "33",
                num.to_string(): "3434",
                "12345": "31",
                fnv1a64("entrants.drivers.driver_0.available").to_string(): "6E6F",
            }, "added_records": []}});
        let r = other
            .import(&cat, ego.to_string().as_bytes(), "ego.json")
            .unwrap();
        assert_eq!(
            (r.format, r.imported, r.unchanged, r.skipped.len()),
            ("ego", 1, 1, 2),
            "{r:?}"
        );
        assert_eq!(other.value(&cat, lap), Some("3"));
        assert!(r.skipped.iter().any(|s| s.contains("True or False")), "{r:?}");
    }

    #[test]
    fn imports_skip_unknown_keys_and_count_stock_values_unchanged() {
        let cat = cat();
        let key = |p: &str| fnv1a64(p).to_string();
        let hex = |v: &str| hex(v);
        let ego = serde_json::json!({"format": "ego-gamedatabases-editor-records", "records": {
            "modified_records": {
                key("entrants.drivers.driver_0.car_number"): hex("1"),
                key("entrants.drivers.driver_0.available"): hex("true"),
                key("tracks.Track_0.PoleOnRight"): hex("true"),
                key("entrants.drivers.driver_1.car_number"): hex("4.5"),
                key("not.in.the.database"): hex("1"),
            }, "added_records": []}});
        let mut edits = Edits::empty();
        let r = edits
            .import(&cat, ego.to_string().as_bytes(), "ego.json")
            .unwrap();
        assert_eq!((r.imported, r.unchanged, r.skipped.len()), (1, 2, 2), "{r:?}");
        assert!(
            r.skipped.iter().any(|s| s.contains(&format!(
                "0x{:016x}: not in this game's database",
                fnv1a64("not.in.the.database")
            ))),
            "{r:?}"
        );
        assert!(r.skipped.iter().any(|s| s.starts_with("entrants.drivers.driver_1.car_number: \"4.5\" is not a whole number")), "{r:?}");
        assert_eq!(edits.len(), 1);
        assert_eq!(
            edits.value(&cat, fnv1a64("tracks.Track_0.PoleOnRight")),
            Some("true")
        );
        edits
            .set(&cat, fnv1a64("entrants.drivers.driver_0.available"), "true")
            .unwrap();
        assert_eq!(edits.len(), 1, "a case-only change of a boolean is no edit");
    }

    #[test]
    fn refuses_the_sample_ego_export_with_added_records() {
        let Some(cat) = real() else {
            eprintln!("F1_25_DIR is not set; skipping");
            return;
        };
        let Ok(bytes) = std::fs::read("/home/swofty/Downloads/Max33Verstappen.json") else {
            eprintln!("no sample EGO export; skipping");
            return;
        };
        let mut edits = Edits::empty();
        let e = edits.import(&cat, &bytes, "Max33Verstappen.json").unwrap_err();
        assert!(
            e.contains("adds 1185 new record(s) and changes 319 value(s)"),
            "{e}"
        );
        assert!(e.contains("only value changes import"), "{e}");
        assert!(edits.is_empty());
    }
}
