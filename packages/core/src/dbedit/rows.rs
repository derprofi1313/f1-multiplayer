//! Rows a DB Editor addon adds: a copy of another entity's row of a table, or
//! of the table's game default row, with the entity's label in the row.

use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};
use swofty_formats::gvdb::Gvdb;
use swofty_formats::hash::{fnv1a64, fnv1a64_from};

use super::Catalog;
use crate::ops::DbOp;

pub const METADATA: &str = "_metadata_";

/// A row the addon adds at the end of `table`, copied from the row whose
/// `field` is `from`, with `field` set to `to`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RowCopy {
    /// The rows in selector form without the row number:
    /// `driver_statistics.driver_statistics.DriverStatisticsHolder`.
    pub table: String,
    /// The field naming the entity a row belongs to, such as `driver`.
    pub field: String,
    /// `field` of the row copied; `None` copies the game default row, the one row whose `field` is empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    pub to: String,
    /// `_metadata_` of the new row, unique in the table, when `field` is another field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// A key of a copied row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Copied<'a> {
    pub row: &'a RowCopy,
    /// The path under the row: `StatisticsDataSet_0.Statistic_3.evolution`.
    pub rel: String,
    /// The key in the draft, named after the row's selector.
    pub key: u64,
    /// The source row's key the value comes from.
    pub source: u64,
}

/// (`container`, `word`) of a table path `container.word`.
pub(crate) fn split(table: &str) -> Option<(&str, &str)> {
    table
        .rsplit_once('.')
        .filter(|(c, w)| !c.is_empty() && !w.is_empty() && !w.contains(['[', ']', '{', '}', '=']))
}

impl RowCopy {
    /// The new row in selector form: `…DriverStatisticsHolder[driver=romain_grosjean]`.
    pub fn select(&self) -> String {
        format!("{}[{}={}]", self.table, self.field, self.to)
    }

    /// The source row in selector form; `[driver=]` for the game default row.
    pub fn source(&self) -> String {
        format!(
            "{}[{}={}]",
            self.table,
            self.field,
            self.from.as_deref().unwrap_or("")
        )
    }

    pub fn is(&self, table: &str, field: &str, to: &str) -> bool {
        self.table == table && self.field == field && self.to == to
    }

    /// The draft key of `rel`, a path under the row such as `StatisticsDataSet_0.Statistic_3.evolution`.
    pub fn key(&self, rel: &str) -> u64 {
        fnv1a64(self.path(rel))
    }

    /// The part of `path` under this row.
    pub fn rel<'a>(&self, path: &'a str) -> Option<&'a str> {
        path.strip_prefix(self.select().as_str())?.strip_prefix('.')
    }

    /// The path of `rel` under this row, which [`RowCopy::key`] hashes.
    pub fn path(&self, rel: &str) -> String {
        format!("{}.{rel}", self.select())
    }

    /// The `add-row` op that adds this row with `values` (relative path → value) on top of the copy.
    pub(crate) fn op(&self, values: BTreeMap<String, String>) -> Option<DbOp> {
        let (container, _) = split(&self.table)?;
        let mut set = values;
        set.insert(self.field.clone(), self.to.clone());
        if let Some(l) = &self.label {
            set.insert(METADATA.into(), l.clone());
        }
        Some(DbOp::AddRow {
            table: container.to_owned(),
            from: self.source(),
            set,
            copy: None,
            stat_id: None,
        })
    }
}

/// The rows of `table` in the stock database, by hash.
pub(crate) fn rows(cat: &Catalog, table: &str) -> Vec<u64> {
    let Some((container, word)) = split(table) else {
        return Vec::new();
    };
    let Ok(base) = cat.state(None, container) else {
        return Vec::new();
    };
    cat.rows_of(base, word)
        .into_iter()
        .filter(|&r| cat.value_of(fnv1a64_from(r, b"._metadata_")).is_some())
        .collect()
}

pub(crate) fn value_in<'a>(cat: &'a Catalog, row: u64, field: &str) -> Option<&'a str> {
    cat.state(Some(row), field).ok().and_then(|k| cat.value_of(k))
}

/// The stock rows of `table` whose `field` is `want`.
pub(crate) fn matching(cat: &Catalog, table: &str, field: &str, want: &str) -> Vec<u64> {
    rows(cat, table)
        .into_iter()
        .filter(|&r| value_in(cat, r, field) == Some(want))
        .collect()
}

/// (path under the row, key) of every named key of the stock row `row`.
pub(crate) fn row_keys(cat: &Catalog, row: u64) -> Vec<(String, u64)> {
    let Some(node) = cat.lookup(row) else {
        return Vec::new();
    };
    let base = format!("{}.", cat.path(node));
    let mut out = Vec::new();
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        for &c in cat.children(n) {
            match cat.node(c).record {
                Some(r) => {
                    if let Some(rel) = cat.path(c).strip_prefix(&base) {
                        out.push((rel.to_owned(), cat.record(r).key));
                    }
                }
                None => stack.push(c),
            }
        }
    }
    out
}

/// The path of `key` under the stock row `row`.
pub(crate) fn rel_of(cat: &Catalog, row: u64, key: u64) -> Option<String> {
    let base = format!("{}.", cat.path(cat.lookup(row)?));
    cat.path(cat.node_of_key(key)?)
        .strip_prefix(&base)
        .map(str::to_owned)
}

/// The one stock row a copy reads from.
pub(crate) fn source_row(cat: &Catalog, row: &RowCopy) -> Option<u64> {
    match matching(cat, &row.table, &row.field, row.from.as_deref().unwrap_or("")).as_slice() {
        [one] => Some(*one),
        _ => None,
    }
}

/// A new `_metadata_` for a copy into `table`: `to`, else `to_2`, `to_3`… that
/// no stock row and no other copy in `copies` uses.
pub(crate) fn free_label(cat: &Catalog, table: &str, to: &str, copies: &[RowCopy]) -> String {
    let taken: HashSet<&str> = rows(cat, table)
        .into_iter()
        .filter_map(|r| cat.value_of(fnv1a64_from(r, b"._metadata_")))
        .chain(
            copies
                .iter()
                .filter(|c| c.table == table)
                .filter_map(|c| c.label.as_deref()),
        )
        .collect();
    std::iter::once(to.to_owned())
        .chain((2..).map(|n| format!("{to}_{n}")))
        .find(|l| !taken.contains(l.as_str()))
        .expect("a free label")
}

/// Checks that `table` takes added rows and that `field` names its rows.
pub(crate) fn check_table(cat: &Catalog, table: &str, field: &str) -> Result<(), String> {
    let (container, word) = split(table).ok_or_else(|| format!("{table} is not a table path"))?;
    let count = format!("{container}._{word}_instancecount_");
    if cat
        .state(None, &count)
        .ok()
        .and_then(|k| cat.value_of(k))
        .is_none()
    {
        return Err(format!("{table} has no row count, so rows can't be added to it"));
    }
    if field.is_empty() || field.contains(['[', ']', '=']) {
        return Err(format!("{field:?} is not a field name"));
    }
    if !rows(cat, table)
        .iter()
        .any(|&r| value_in(cat, r, field).is_some())
    {
        return Err(format!("the rows of {table} have no {field}"));
    }
    Ok(())
}

/// A row count raised past the last row the database holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortTable {
    /// The `_<word>_instancecount_` key.
    pub key: u64,
    /// `schedule.track_lists.track_event_list_19.fixed_event_reference`.
    pub table: String,
    pub count: u32,
    pub rows: u32,
}

impl ShortTable {
    /// The warning naming `by`, the addon that raised the count.
    pub fn line(&self, by: &str) -> String {
        format!(
            "{by}: {} has {} rows, not {}; the count stays at {} (the game crashes on rows that don't exist)",
            self.table, self.rows, self.count, self.rows
        )
    }
}

/// Lowers each row count `db` raises above stock to the rows `db` holds.
/// The game reads rows 0 to count−1, and a row without keys crashes it.
pub fn trim_row_counts(cat: &Catalog, db: &mut Gvdb) -> Vec<ShortTable> {
    let keys: HashSet<u64> = db.records.iter().map(|r| r.key).collect();
    let mut out = Vec::new();
    for r in &mut db.records {
        let Some(stock) = cat.record_by_key(r.key) else {
            continue;
        };
        if stock.value.as_bytes() == r.value.as_slice() {
            continue;
        }
        let Some((container, word)) = cat.instance_count(r.key) else {
            continue;
        };
        let (Ok(from), Ok(count)) = (stock.value.trim().parse::<u32>(), r.text().trim().parse::<u32>())
        else {
            continue;
        };
        let missing = |i: u32| {
            let row = cat.row_keys(container, word, i..i + 1);
            !row.is_empty() && !row.iter().any(|(k, _)| keys.contains(k))
        };
        let Some(rows) = (from..count).find(|&i| missing(i)) else {
            continue;
        };
        r.value = rows.to_string().into_bytes();
        out.push(ShortTable {
            key: r.key,
            table: cat.table_path(container, word),
            count,
            rows,
        });
    }
    out
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::dbedit::package::{self, Package};
    use crate::dbedit::{Edits, module};
    use crate::install::Game;
    use crate::ops::DbReport;
    use crate::rowkeys::{Names, RowKeys};
    use crate::store::Store;
    use swofty_formats::gvdb::Record;

    pub(crate) const HOLDERS: &str = "driver_statistics.driver_statistics.DriverStatisticsHolder";
    const H: &str = "driver_statistics.driver_statistics";
    const TEAMS: &str = "entrants.teams.team";
    const MODES: &str = "game_modes.ModeChampionship";

    fn curve(v: f32) -> String {
        format!("@v2@minx:0,maxx:10,miny:0,maxy:100,0:{v},1:{v}")
    }

    fn pace(who: &str, set: &str) -> String {
        format!(
            "{HOLDERS}[driver={who}].StatisticsDataSet[_metadata_={set}].Statistic[_metadata_=pace].evolution"
        )
    }

    pub(crate) fn sample() -> Gvdb {
        let mut r = vec![
            Record::new(&format!("{H}._DriverStatisticsHolder_instancecount_"), 0, "3"),
            Record::new("entrants.drivers._driver_instancecount_", 0, "3"),
            Record::new("entrants.teams._team_instancecount_", 0, "2"),
            Record::new("game_modes._ModeChampionship_instancecount_", 0, "1"),
            Record::new("game_modes.ModeChampionship_0._metadata_", 2, "career"),
            Record::new(
                "game_modes.ModeChampionship_0.allowedComponentSaveGroups",
                12,
                "-1",
            ),
            Record::new("game_modes.ModeChampionship_0.name", 2, "lng_career"),
        ];
        for (i, label) in ["lewis_hamilton", "max_verstappen", "romain_grosjean"]
            .iter()
            .enumerate()
        {
            r.push(Record::new(
                &format!("entrants.drivers.driver_{i}._metadata_"),
                2,
                *label,
            ));
            r.push(Record::new(
                &format!("entrants.drivers.driver_{i}.race_number"),
                0,
                "7",
            ));
        }
        for (i, (label, id)) in [("mclaren_26", "100"), ("ferrari_26", "101")].iter().enumerate() {
            r.push(Record::new(&format!("{TEAMS}_{i}._metadata_"), 4, *label));
            r.push(Record::new(&format!("{TEAMS}_{i}.stat_id"), 0, *id));
            r.push(Record::new(&format!("{TEAMS}_{i}.display_name_code"), 2, "XXX"));
        }
        for (i, (label, driver, sets)) in [
            ("default_player", "", &["default"][..]),
            ("lewis_hamilton", "lewis_hamilton", &["default", "2026"][..]),
            ("max_verstappen", "max_verstappen", &["default"][..]),
        ]
        .into_iter()
        .enumerate()
        {
            let row = format!("{HOLDERS}_{i}");
            r.push(Record::new(&format!("{row}._metadata_"), 2, label));
            r.push(Record::new(&format!("{row}.driver"), 4, driver));
            r.push(Record::new(
                &format!("{row}._StatisticsDataSet_instancecount_"),
                0,
                sets.len().to_string(),
            ));
            for (j, set) in sets.iter().enumerate() {
                let ds = format!("{row}.StatisticsDataSet_{j}");
                r.push(Record::new(&format!("{ds}._metadata_"), 2, *set));
                r.push(Record::new(&format!("{ds}._Statistic_instancecount_"), 0, "2"));
                r.push(Record::new(&format!("{ds}.is_invalid"), 5, "False"));
                r.push(Record::new(&format!("{ds}.seasons"), 13, ""));
                for (k, stat) in ["pace", "focus"].iter().enumerate() {
                    let st = format!("{ds}.Statistic_{k}");
                    r.push(Record::new(&format!("{st}._metadata_"), 2, *stat));
                    r.push(Record::new(&format!("{st}.info"), 4, *stat));
                    let v = 0.5 + 0.1 * i as f32 + 0.01 * (j * 2 + k) as f32;
                    r.push(Record::new(&format!("{st}.evolution"), 9, curve(v)));
                }
            }
        }
        Gvdb { records: r }
    }

    fn cat() -> Catalog {
        Catalog::load(&sample().to_bytes()).unwrap()
    }

    fn build(edits: &Edits) -> (Gvdb, DbReport) {
        let mut db = sample();
        let stock = db.records.iter().map(|r| r.key).collect();
        let fields = BTreeMap::new();
        let names = Names {
            fields: &fields,
            keys: RowKeys::builtin(),
            stock: Some(&stock),
            tracks: None,
        };
        let mut report = DbReport::default();
        let (_, failed) = crate::ops::apply_db_ops_lenient(&mut db, &edits.to_ops(), &names, &mut report);
        assert!(failed.is_empty(), "{failed:?}");
        (db, report)
    }

    fn under(db: &Gvdb, row: &str) -> BTreeMap<String, (u8, String)> {
        let cat = Catalog::load(&db.to_bytes()).unwrap();
        let node = cat.find(row).unwrap();
        let base = cat.state(None, row).unwrap();
        row_keys(&cat, base)
            .into_iter()
            .map(|(rel, k)| {
                let r = cat.record_by_key(k).unwrap();
                (rel, (r.tag, r.value.to_owned()))
            })
            .chain(std::iter::once((String::from("."), (0, cat.path(node)))))
            .collect()
    }

    #[test]
    fn copies_a_row_with_every_key_and_an_edit_on_top() {
        let cat = cat();
        let mut e = Edits::default();
        assert!(e.copied_path(&cat, &pace("romain_grosjean", "default")).is_none());
        e.copy_row(&cat, HOLDERS, "driver", Some("lewis_hamilton"), "romain_grosjean")
            .unwrap();
        assert_eq!(e.rows[0].label.as_deref(), Some("romain_grosjean"));
        assert_eq!(e.len(), 1);
        let c = e.copied_path(&cat, &pace("romain_grosjean", "default")).unwrap();
        let lewis = cat.resolve(&pace("lewis_hamilton", "default")).unwrap();
        assert_eq!(
            (c.source, c.rel.as_str()),
            (lewis, "StatisticsDataSet_0.Statistic_0.evolution")
        );
        let key = c.key;
        assert_eq!(e.value(&cat, key), cat.value_of(lewis));
        assert_eq!(e.copied(&cat, key).unwrap().source, lewis);
        assert_eq!(
            e.resolve(
                &cat,
                &format!("{HOLDERS}[driver=romain_grosjean].StatisticsDataSet_0.Statistic_0.evolution")
            ),
            Ok(key)
        );
        assert!(e.set(&cat, key, "fast").is_err());
        e.set(&cat, key, &curve(0.9)).unwrap();
        assert_eq!(e.value(&cat, key), Some(curve(0.9).as_str()));
        assert!(e.remove_key(&cat, key).is_err());

        let (db, report) = build(&e);
        assert_eq!(db.instance_count(H, "DriverStatisticsHolder"), Some(4));
        let (src, new) = (
            under(&db, &format!("{HOLDERS}_1")),
            under(&db, &format!("{HOLDERS}_3")),
        );
        let rels = |m: &BTreeMap<String, (u8, String)>| m.keys().cloned().collect::<Vec<_>>();
        assert_eq!(rels(&src), rels(&new), "{report:?}");
        assert_eq!(src.len(), 4 + 2 * (4 + 2 * 3));
        for (rel, (tag, v)) in &new {
            let want = match rel.as_str() {
                "." => format!("{HOLDERS}_3"),
                "driver" | "_metadata_" => "romain_grosjean".into(),
                "StatisticsDataSet_0.Statistic_0.evolution" => curve(0.9),
                _ => src[rel].1.clone(),
            };
            assert_eq!((v, *tag), (&want, src[rel].0), "{rel}");
        }
        assert!(
            Catalog::load(&db.to_bytes())
                .unwrap()
                .resolve(&pace("romain_grosjean", "2026"))
                .is_ok()
        );

        e.set(&cat, key, cat.value_of(lewis).unwrap()).unwrap();
        assert_eq!(e.edits.len(), 0, "a value equal to the source's is no change");
        e.set(&cat, key, &curve(0.8)).unwrap();
        assert!(e.remove_row(HOLDERS, "driver", "romain_grosjean"));
        assert!(e.is_empty() && !e.remove_row(HOLDERS, "driver", "romain_grosjean"));
    }

    #[test]
    fn copies_the_game_default_and_takes_the_drafts_changes_to_the_source() {
        let cat = cat();
        let mut e = Edits::default();
        let lewis = cat.resolve(&pace("lewis_hamilton", "default")).unwrap();
        e.set(&cat, lewis, &curve(0.99)).unwrap();
        e.copy_row(&cat, HOLDERS, "driver", Some("lewis_hamilton"), "romain_grosjean")
            .unwrap();
        let key = e
            .copied_path(&cat, &pace("romain_grosjean", "default"))
            .unwrap()
            .key;
        assert_eq!(e.value(&cat, key), Some(curve(0.99).as_str()));
        e.revert(lewis);
        assert_eq!(
            e.value(&cat, key),
            Some(curve(0.99).as_str()),
            "the copy keeps it"
        );

        let mut d = Edits::default();
        d.copy_row(&cat, HOLDERS, "driver", None, "romain_grosjean")
            .unwrap();
        assert_eq!(d.rows[0].source(), format!("{HOLDERS}[driver=]"));
        assert!(d.copied_path(&cat, &pace("romain_grosjean", "2026")).is_none());
        let (db, _) = build(&d);
        let built = Catalog::load(&db.to_bytes()).unwrap();
        let own = built.resolve(&pace("romain_grosjean", "default")).unwrap();
        let default = cat
            .resolve(&format!("{HOLDERS}_0.StatisticsDataSet_0.Statistic_0.evolution"))
            .unwrap();
        assert_eq!(built.value_of(own), cat.value_of(default));

        for (from, to, why) in [
            (
                Some("lewis_hamilton"),
                "max_verstappen",
                "already has its own row",
            ),
            (
                Some("nobody"),
                "romain_grosjean",
                "has no row whose driver is nobody",
            ),
            (
                Some("romain_grosjean"),
                "romain_grosjean",
                "can't copy its own row",
            ),
            (Some("lewis_hamilton"), "", "can't name a row"),
        ] {
            let err = Edits::default()
                .copy_row(&cat, HOLDERS, "driver", from, to)
                .unwrap_err();
            assert!(err.contains(why), "{err}");
        }
        assert!(
            d.copy_row(&cat, HOLDERS, "driver", Some("lewis_hamilton"), "romain_grosjean")
                .is_err()
        );
        let err = Edits::default()
            .copy_row(&cat, TEAMS, "_metadata_", None, "haas_26")
            .unwrap_err();
        assert!(err.contains("no game default row"), "{err}");
        assert!(
            Edits::default()
                .copy_row(&cat, "entrants.nothing", "x", None, "y")
                .is_err()
        );
    }

    #[test]
    fn copies_get_their_own_stat_id_and_keep_signed_t12_text() {
        let cat = cat();
        let mut e = Edits::default();
        e.copy_row(&cat, TEAMS, "_metadata_", Some("ferrari_26"), "swofty_26")
            .unwrap();
        e.copy_row(&cat, MODES, "_metadata_", Some("career"), "career_copy")
            .unwrap();
        assert_eq!(e.rows[0].label, None);
        let (db, report) = build(&e);
        assert_eq!(
            db.get_text(&format!("{TEAMS}_2._metadata_")).as_deref(),
            Some("swofty_26")
        );
        assert_eq!(
            db.get_text(&format!("{TEAMS}_2.stat_id")).as_deref(),
            Some("102"),
            "{report:?}"
        );
        let t12 = db
            .get("game_modes.ModeChampionship_1.allowedComponentSaveGroups")
            .unwrap();
        assert_eq!((t12.tag, t12.value.as_slice()), (12, &b"-1"[..]));
        assert_eq!(db.instance_count("game_modes", "ModeChampionship"), Some(2));
    }

    #[test]
    fn the_drivers_page_shows_a_copied_row_as_its_own() {
        let cat = cat();
        let drivers = module("drivers").unwrap();
        let specs = drivers.row_specs();
        assert_eq!(
            specs,
            [crate::dbedit::RowSpec {
                table: HOLDERS.into(),
                field: "driver".into()
            }]
        );
        let pace = drivers.fields.iter().position(|f| f.id == "pace").unwrap();
        let pace26 = drivers.fields.iter().position(|f| f.id == "pace_2026").unwrap();
        assert_eq!(drivers.field_row(pace), Some(0));
        assert_eq!(drivers.field_row(0), None);
        let (labels, default) = drivers.row_sources(&cat, 0, None);
        assert_eq!(
            (labels, default),
            (vec!["lewis_hamilton".to_owned(), "max_verstappen".into()], true)
        );

        let mut e = Edits::default();
        let row = |e: &Edits| {
            let v = drivers.view(&cat, e, None).unwrap();
            v.rows.into_iter().find(|r| r.label == "romain_grosjean").unwrap()
        };
        let before = row(&e);
        assert_eq!(before.backing, [crate::dbedit::Backing::Missing]);
        assert_eq!(before.cells[pace].key, None);
        e.copy_row(&cat, HOLDERS, "driver", Some("lewis_hamilton"), "romain_grosjean")
            .unwrap();
        let after = row(&e);
        assert_eq!(
            after.backing,
            [crate::dbedit::Backing::Copied(Some("lewis_hamilton".into()))]
        );
        let cell = &after.cells[pace];
        assert_eq!((cell.tag, cell.edited), (9, false));
        assert_eq!(cell.value, cell.stock);
        assert!(after.cells[pace26].key.is_some());
        e.set(&cat, cell.key.unwrap(), &curve(0.7)).unwrap();
        let edited = row(&e);
        assert!(edited.cells[pace].edited);
        assert_eq!(edited.cells[pace].value, curve(0.7));
        let lewis = drivers
            .view(&cat, &e, None)
            .unwrap()
            .rows
            .into_iter()
            .find(|r| r.label == "lewis_hamilton")
            .unwrap();
        assert_eq!(lewis.backing, [crate::dbedit::Backing::Stock]);
    }

    #[test]
    fn copies_a_real_driver_rating_row() {
        let Some(game) = std::env::var_os("F1_25_DIR").map(std::path::PathBuf::from) else {
            eprintln!("F1_25_DIR is not set; skipping");
            return;
        };
        let bytes = std::fs::read(game.join(crate::build::GVDB)).unwrap();
        let cat = Catalog::load(&bytes).unwrap();
        let mut e = Edits::default();
        e.copy_row(&cat, HOLDERS, "driver", Some("lewis_hamilton"), "romain_grosjean")
            .unwrap();
        let key = e
            .copied_path(&cat, &pace("romain_grosjean", "default"))
            .unwrap()
            .key;
        e.set(&cat, key, "@v2@minx:0,maxx:10,miny:0,maxy:100,0:0.95,1:0.9")
            .unwrap();
        let mut db = Gvdb::parse(&bytes).unwrap();
        let stock = db.records.iter().map(|r| r.key).collect();
        let fields = BTreeMap::new();
        let names = Names {
            fields: &fields,
            keys: RowKeys::builtin(),
            stock: Some(&stock),
            tracks: None,
        };
        let mut report = DbReport::default();
        crate::ops::apply_db_ops_report(&mut db, &e.to_ops(), &names, &mut report).unwrap();
        assert!(report.warnings.is_empty(), "{report:?}");
        let n = db.instance_count(H, "DriverStatisticsHolder").unwrap();
        assert_eq!(n, 118);
        let built = Catalog::load(&db.to_bytes()).unwrap();
        let lewis = matching(&built, HOLDERS, "driver", "lewis_hamilton")[0];
        let romain = matching(&built, HOLDERS, "driver", "romain_grosjean");
        assert_eq!(romain.len(), 1);
        let keys = |row| {
            row_keys(&built, row)
                .into_iter()
                .map(|(r, _)| r)
                .collect::<std::collections::BTreeSet<_>>()
        };
        assert_eq!(keys(lewis), keys(romain[0]));
        assert_eq!(matching(&built, HOLDERS, METADATA, "romain_grosjean").len(), 1);
        assert_eq!(
            built.value_of(built.resolve(&pace("romain_grosjean", "default")).unwrap()),
            Some("@v2@minx:0,maxx:10,miny:0,maxy:100,0:0.95,1:0.9")
        );
    }

    pub(crate) fn game(s: &Store, db: &Gvdb) -> Game {
        let dir = s.root.join("game");
        let path = dir.join(crate::build::GVDB);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, db.to_bytes()).unwrap();
        Game {
            dir,
            user_reg: None,
            identity: crate::baseline::Identity {
                build_id: "25251167".into(),
                exe_sha256: "ab".repeat(32),
            },
        }
    }

    #[test]
    fn copied_rows_save_export_import_and_stack() {
        let root = std::env::temp_dir().join(format!("swofty-dbrows-{}", std::process::id()));
        std::fs::remove_dir_all(&root).ok();
        let s = Store::new(root.clone());
        let cat = cat();
        let mut p = Package::create(&s, "Ratings").unwrap();
        p.edits
            .copy_row(&cat, HOLDERS, "driver", Some("lewis_hamilton"), "romain_grosjean")
            .unwrap();
        let key = p
            .edits
            .copied_path(&cat, &pace("romain_grosjean", "default"))
            .unwrap()
            .key;
        p.edits.set(&cat, key, &curve(0.9)).unwrap();
        p.save(&s, &cat).unwrap();
        let dir = s.addons_dir().join(&p.id);
        let marker: serde_json::Value = crate::read_json(&dir.join(package::MARKER)).unwrap();
        assert_eq!(marker["format_version"], 2);
        assert_eq!(marker["rows"][0]["to"], "romain_grosjean");
        assert_eq!(marker["rows"][0]["from"], "lewis_hamilton");
        let m = crate::manifest::AddonManifest::load(&dir).unwrap();
        let req = m.compat.launcher.unwrap();
        assert!(!req.matches(&semver::Version::new(1, 3, 0)), "{req}");
        assert!(req.matches(&crate::update::current_version()), "{req}");
        let again = Package::open(&s, &p.id).unwrap();
        assert_eq!(again.edits, p.edits);
        assert_eq!(again.info().changes, 2);

        let zip = root.join("ratings.zip");
        Package::export_zip(&s, &p.id, &zip).unwrap();
        let got = package::import(&s, &cat, &zip).unwrap();
        assert_eq!(Package::open(&s, &got.package.id).unwrap().edits, p.edits);
        let json = root.join("ratings.json");
        std::fs::write(&json, p.edits.export(&cat)).unwrap();
        let got = package::import(&s, &cat, &json).unwrap();
        assert_eq!(
            (got.report.imported, got.report.skipped.len()),
            (2, 0),
            "{:?}",
            got.report
        );
        let imported = Package::open(&s, &got.package.id).unwrap().edits;
        assert_eq!(imported.rows, p.edits.rows);
        assert_eq!(imported.value(&cat, key), Some(curve(0.9).as_str()));

        let mut other = Package::create(&s, "Teams").unwrap();
        other
            .edits
            .copy_row(&cat, HOLDERS, "driver", None, "lewis_2")
            .unwrap();
        other.save(&s, &cat).unwrap();
        let game = game(&s, &sample());
        let out = root.join("game.zip");
        let ids = [other.id.clone(), p.id.clone()];
        let done = package::export_game_files(&s, &game, &ids, &out).unwrap();
        assert!(done.skipped.is_empty(), "{:?}", done.skipped);
        let mut z = zip::ZipArchive::new(std::fs::File::open(&out).unwrap()).unwrap();
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut z.by_name(crate::build::GVDB).unwrap(), &mut bytes).unwrap();
        let built = Catalog::load(&bytes).unwrap();
        assert_eq!(
            built.find(&format!("{HOLDERS}_3.driver")).map(|n| built.path(n)),
            Some(format!("{HOLDERS}_3.driver"))
        );
        let romain = built.resolve(&pace("romain_grosjean", "default")).unwrap();
        assert_eq!(built.value_of(romain), Some(curve(0.9).as_str()));
        assert_eq!(
            built.path(built.node_of_key(romain).unwrap()),
            format!("{HOLDERS}_4.StatisticsDataSet_0.Statistic_0.evolution")
        );
        std::fs::remove_dir_all(&root).ok();
    }

    const LISTS: &str = "schedule.track_lists";

    fn calendar(events: &[&str]) -> Gvdb {
        let list = format!("{LISTS}.track_event_list_0");
        let mut r = vec![
            Record::new(&format!("{LISTS}._track_event_list_instancecount_"), 0, "1"),
            Record::new(&format!("{list}._metadata_"), 2, "F1_2025_16"),
            Record::new(
                &format!("{list}._fixed_event_reference_instancecount_"),
                0,
                events.len().to_string(),
            ),
        ];
        for (i, e) in events.iter().enumerate() {
            let row = format!("{list}.fixed_event_reference_{i}");
            r.push(Record::new(&format!("{row}._metadata_"), 2, format!("ref_{e}")));
            r.push(Record::new(&format!("{row}.event_name"), 4, *e));
        }
        Gvdb { records: r }
    }

    fn count_of(db: &Gvdb) -> Option<String> {
        db.get_text(&format!(
            "{LISTS}.track_event_list_0._fixed_event_reference_instancecount_"
        ))
    }

    #[test]
    fn row_counts_past_the_last_row_come_down_to_the_rows_there() {
        let stock = calendar(&["rw_bahrain", "rw_suzuka"]);
        let cat = Catalog::load(&stock.to_bytes()).unwrap();
        let mut db = stock.clone();
        let count = format!("{LISTS}.track_event_list_0._fixed_event_reference_instancecount_");
        db.set(&count, "5", None).unwrap();
        let row = format!("{LISTS}.track_event_list_0.fixed_event_reference_2");
        db.set(&format!("{row}.event_name"), "rw_miami", Some(4)).unwrap();
        let short = trim_row_counts(&cat, &mut db);
        assert_eq!(
            short,
            vec![ShortTable {
                key: fnv1a64(&count),
                table: format!("{LISTS}.track_event_list_0.fixed_event_reference"),
                count: 5,
                rows: 3,
            }]
        );
        assert_eq!(count_of(&db).as_deref(), Some("3"));
        assert!(trim_row_counts(&cat, &mut db.clone()).is_empty());
    }

    #[test]
    fn row_counts_with_their_rows_or_lowered_stay() {
        let stock = calendar(&["rw_bahrain", "rw_suzuka"]);
        let cat = Catalog::load(&stock.to_bytes()).unwrap();
        let mut grown = calendar(&["rw_bahrain", "rw_suzuka", "rw_miami"]);
        assert!(trim_row_counts(&cat, &mut grown).is_empty());
        assert_eq!(count_of(&grown).as_deref(), Some("3"));
        let mut cut = calendar(&["rw_bahrain"]);
        assert!(trim_row_counts(&cat, &mut cut).is_empty());
        assert_eq!(count_of(&cut).as_deref(), Some("1"));
    }

    #[test]
    fn exported_game_files_keep_a_row_count_at_the_rows_there() {
        let root = std::env::temp_dir().join(format!("swofty-dbrows-short-{}", std::process::id()));
        std::fs::remove_dir_all(&root).ok();
        let s = Store::new(root.clone());
        let stock = calendar(&["rw_bahrain", "rw_suzuka"]);
        let cat = Catalog::load(&stock.to_bytes()).unwrap();
        let list = format!("{LISTS}.track_event_list_0");
        let count = format!("{list}._fixed_event_reference_instancecount_");
        let second = format!("{list}.fixed_event_reference_1.event_name");
        let mut p = Package::create(&s, "2017 v2").unwrap();
        p.edits.set(&cat, fnv1a64(&count), "20").unwrap();
        p.edits.set(&cat, fnv1a64(&second), "rw_bahrain").unwrap();
        p.save(&s, &cat).unwrap();
        let game = game(&s, &stock);
        let out = root.join("game.zip");
        let done = package::export_game_files(&s, &game, &[p.id.clone()], &out).unwrap();
        let short = ShortTable {
            key: fnv1a64(&count),
            table: format!("{list}.fixed_event_reference"),
            count: 20,
            rows: 2,
        };
        assert_eq!(done.skipped, vec![short.line("2017 v2")]);
        let mut z = zip::ZipArchive::new(std::fs::File::open(&out).unwrap()).unwrap();
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut z.by_name(crate::build::GVDB).unwrap(), &mut bytes).unwrap();
        let db = Gvdb::parse(&bytes).unwrap();
        assert_eq!(count_of(&db).as_deref(), Some("2"));
        assert_eq!(db.get_text(&second).as_deref(), Some("rw_bahrain"));
        std::fs::remove_dir_all(&root).ok();
    }
}
