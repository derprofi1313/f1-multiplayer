//! The ids UDP telemetry reports for rows an addon added, in `swofty/udp_ids.json`.
//!
//! An added team keeps its stock ancestor's stat id. An added driver keeps its stock ancestor's id
//! when its `first_name`, `last_name` and `full_name` equal the ancestor's, and is 255 otherwise;
//! so is any driver id above 254 with no stock ancestor. The 2024 and 2025 formats carry one byte
//! per id, so `teams_byte` and `drivers_byte` hold only the ids whose low byte no stock row and no
//! other mapped id shares; the 2026 format carries 16 bits and reads `teams` and `drivers`.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};
use swofty_formats::gvdb::Gvdb;

use crate::dbedit::deps::Copy;
use crate::ops::stat_id_field;

pub const FILE: &str = "udp_ids.json";
pub const NETWORK_DRIVER: i64 = 255;
const VERSION: u32 = 1;
const TEAMS: (&str, &str) = ("entrants.teams", "team");
const DRIVERS: (&str, &str) = ("entrants.drivers", "driver");
const NAME_FIELDS: [&str; 3] = ["first_name", "last_name", "full_name"];
const DEPTH: usize = 64;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UdpIds {
    pub version: u32,
    pub teams: BTreeMap<i64, i64>,
    pub drivers: BTreeMap<i64, i64>,
    pub teams_byte: BTreeMap<i64, i64>,
    pub drivers_byte: BTreeMap<i64, i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<String>,
}

impl UdpIds {
    pub fn is_empty(&self) -> bool {
        self.teams.is_empty() && self.drivers.is_empty()
    }
}

struct Table {
    stock: HashSet<i64>,
    ids: HashMap<String, i64>,
}

fn id_of(db: &Gvdb, table: (&str, &str), path: &str) -> Option<i64> {
    db.get_text(&format!("{path}.{}", stat_id_field(table.0, table.1)))?
        .trim()
        .parse()
        .ok()
}

fn read(db: &Gvdb, table: (&str, &str), copied: &HashMap<&str, &Copy>) -> Table {
    let n = db.instance_count(table.0, table.1).unwrap_or(0);
    let mut out = Table {
        stock: HashSet::new(),
        ids: HashMap::new(),
    };
    for i in 0..n {
        let path = format!("{}.{}_{i}", table.0, table.1);
        let Some(id) = id_of(db, table, &path) else {
            continue;
        };
        if !copied.contains_key(path.as_str()) {
            out.stock.insert(id);
        }
        out.ids.insert(path, id);
    }
    out
}

fn ancestor<'a>(copied: &HashMap<&'a str, &'a Copy>, row: &'a str) -> Option<&'a str> {
    let mut at = row;
    for _ in 0..DEPTH {
        at = copied.get(at)?.from.as_str();
        if !copied.contains_key(at) {
            return Some(at);
        }
    }
    None
}

fn same_person(db: &Gvdb, row: &str, other: &str) -> bool {
    NAME_FIELDS
        .iter()
        .all(|f| db.get_text(&format!("{row}.{f}")) == db.get_text(&format!("{other}.{f}")))
}

fn narrow(full: &BTreeMap<i64, i64>, stock: &HashSet<i64>) -> BTreeMap<i64, i64> {
    let mut by_byte: HashMap<i64, usize> = HashMap::new();
    for id in full.keys() {
        *by_byte.entry(id & 0xff).or_default() += 1;
    }
    full.iter()
        .filter(|(id, _)| **id < 256 || (!stock.contains(&(**id & 0xff)) && by_byte[&(**id & 0xff)] == 1))
        .map(|(id, to)| (id & 0xff, *to))
        .collect()
}

pub fn build(db: &Gvdb, copies: &[Copy]) -> UdpIds {
    let name = |path: &str| {
        db.get_text(&format!("{path}._metadata_"))
            .unwrap_or_else(|| path.to_owned())
    };
    let copied: HashMap<&str, &Copy> = copies.iter().map(|c| (c.row.as_str(), c)).collect();
    let mut out = UdpIds {
        version: VERSION,
        ..UdpIds::default()
    };
    let teams = read(db, TEAMS, &copied);
    let drivers = read(db, DRIVERS, &copied);
    for (path, &id) in &teams.ids {
        if !copied.contains_key(path.as_str()) {
            continue;
        }
        let Some(from) = ancestor(&copied, path) else {
            continue;
        };
        let Some(&stock) = teams.ids.get(from) else {
            continue;
        };
        if stock != id && teams.stock.contains(&stock) {
            out.teams.insert(id, stock);
            out.lines.push(format!("team {id} -> {stock} ({})", name(path)));
        }
    }
    for (path, &id) in &drivers.ids {
        let target = match copied.contains_key(path.as_str()) {
            true => match ancestor(&copied, path).and_then(|from| Some((from, *drivers.ids.get(from)?))) {
                Some((from, stock)) if drivers.stock.contains(&stock) && same_person(db, path, from) => stock,
                _ => NETWORK_DRIVER,
            },
            false => NETWORK_DRIVER,
        };
        let needs = match copied.contains_key(path.as_str()) {
            true => target != id,
            false => id > 254,
        };
        if needs {
            let why = match target {
                NETWORK_DRIVER => "no stock driver".to_owned(),
                _ => name(path),
            };
            out.drivers.insert(id, target);
            out.lines.push(format!("driver {id} -> {target} ({why})"));
        }
    }
    out.teams_byte = narrow(&out.teams, &teams.stock);
    out.drivers_byte = narrow(&out.drivers, &drivers.stock);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use swofty_formats::gvdb::Record;

    fn row(records: &mut Vec<Record>, table: (&str, &str), i: usize, id: i64, extra: &[(&str, &str)]) {
        let p = format!("{}.{}_{i}", table.0, table.1);
        records.push(Record::new(
            &format!("{p}._metadata_"),
            4,
            format!("{}_{i}", table.1),
        ));
        records.push(Record::new(&format!("{p}.stat_id"), 0, id.to_string()));
        for (f, v) in extra {
            records.push(Record::new(&format!("{p}.{f}"), 4, *v));
        }
    }

    fn count(records: &mut Vec<Record>, table: (&str, &str), n: usize) {
        records.push(Record::new(
            &format!("{}._{}_instancecount_", table.0, table.1),
            0,
            n.to_string(),
        ));
    }

    fn copy(from: &str, row: &str) -> Copy {
        Copy {
            addon: "cs".into(),
            from: from.into(),
            row: row.into(),
            ..Copy::default()
        }
    }

    const NORRIS: [(&str, &str); 3] = [
        ("first_name", "lng_lando"),
        ("last_name", "lng_norris"),
        ("full_name", "lng_lando_norris"),
    ];
    const KIMI: [(&str, &str); 3] = [
        ("first_name", "lng_kimi"),
        ("last_name", "lng_raikkonen"),
        ("full_name", "lng_kimi_raikkonen"),
    ];

    fn fixture() -> (Gvdb, Vec<Copy>) {
        let mut r = Vec::new();
        count(&mut r, TEAMS, 5);
        row(&mut r, TEAMS, 0, 4, &[]);
        row(&mut r, TEAMS, 1, 1, &[]);
        row(&mut r, TEAMS, 2, 507, &[]);
        row(&mut r, TEAMS, 3, 518, &[]);
        row(&mut r, TEAMS, 4, 509, &[]);
        count(&mut r, DRIVERS, 6);
        row(&mut r, DRIVERS, 0, 8, &NORRIS);
        row(&mut r, DRIVERS, 1, 197, &KIMI);
        row(&mut r, DRIVERS, 2, 198, &NORRIS);
        row(&mut r, DRIVERS, 3, 300, &KIMI);
        row(&mut r, DRIVERS, 4, 301, &KIMI);
        row(&mut r, DRIVERS, 5, 44, &KIMI);
        let copies = vec![
            copy("entrants.teams.team_0", "entrants.teams.team_2"),
            copy("entrants.teams.team_1", "entrants.teams.team_3"),
            copy("entrants.teams.team_2", "entrants.teams.team_4"),
            copy("entrants.drivers.driver_0", "entrants.drivers.driver_1"),
            copy("entrants.drivers.driver_0", "entrants.drivers.driver_2"),
            copy("entrants.drivers.driver_0", "entrants.drivers.driver_3"),
            copy("entrants.drivers.driver_3", "entrants.drivers.driver_4"),
        ];
        (Gvdb { records: r }, copies)
    }

    fn built() -> UdpIds {
        let (db, copies) = fixture();
        build(&db, &copies)
    }

    #[test]
    fn added_teams_take_their_stock_ancestor() {
        let ids = built();
        assert_eq!(ids.teams, BTreeMap::from([(507, 4), (509, 4), (518, 1)]));
    }

    #[test]
    fn a_chain_of_copies_ends_at_the_stock_row() {
        assert_eq!(built().teams.get(&509), Some(&4));
    }

    #[test]
    fn drivers_keep_stock_only_for_the_same_person() {
        let ids = built();
        assert_eq!(ids.drivers.get(&197), Some(&255));
        assert_eq!(ids.drivers.get(&198), Some(&8));
        assert_eq!(ids.drivers.get(&300), Some(&255));
        assert_eq!(ids.drivers.get(&301), Some(&255));
        assert_eq!(ids.drivers.get(&8), None);
        assert_eq!(ids.drivers.get(&44), None);
    }

    #[test]
    fn a_driver_above_254_that_no_copy_made_is_255() {
        let mut r = Vec::new();
        count(&mut r, DRIVERS, 2);
        row(&mut r, DRIVERS, 0, 8, &NORRIS);
        row(&mut r, DRIVERS, 1, 400, &KIMI);
        let ids = build(&Gvdb { records: r }, &[]);
        assert_eq!(ids.drivers, BTreeMap::from([(400, 255)]));
    }

    #[test]
    fn byte_tables_drop_ids_whose_low_byte_is_shared() {
        let ids = built();
        assert_eq!(ids.drivers_byte.get(&197), Some(&255));
        assert_eq!(ids.drivers_byte.get(&198), Some(&8));
        assert_eq!(ids.drivers_byte.get(&(300 & 0xff)), None);
        assert_eq!(ids.drivers_byte.get(&(301 & 0xff)), Some(&255));
        assert_eq!(ids.teams_byte.get(&(507 & 0xff)), Some(&4));
        assert_eq!(ids.teams_byte.get(&(518 & 0xff)), Some(&1));
    }

    #[test]
    fn a_chain_through_rows_of_other_addons_ends_at_the_stock_row() {
        let mut r = Vec::new();
        count(&mut r, TEAMS, 4);
        row(&mut r, TEAMS, 0, 6, &[]);
        row(&mut r, TEAMS, 1, 700, &[]);
        row(&mut r, TEAMS, 2, 701, &[]);
        row(&mut r, TEAMS, 3, 702, &[]);
        let mut a = copy("entrants.teams.team_0", "entrants.teams.team_1");
        a.addon = "bdx-base".into();
        let mut b = copy("entrants.teams.team_1", "entrants.teams.team_2");
        b.addon = "bdx-teams".into();
        let mut c = copy("entrants.teams.team_2", "entrants.teams.team_3");
        c.addon = "season-2012".into();
        let ids = build(&Gvdb { records: r }, &[c, a, b]);
        assert_eq!(ids.teams, BTreeMap::from([(700, 6), (701, 6), (702, 6)]));
    }

    #[test]
    fn a_copy_that_keeps_its_source_stat_id_is_not_rewritten() {
        let mut r = Vec::new();
        count(&mut r, TEAMS, 2);
        row(&mut r, TEAMS, 0, 6, &[]);
        row(&mut r, TEAMS, 1, 6, &[]);
        let ids = build(
            &Gvdb { records: r },
            &[copy("entrants.teams.team_0", "entrants.teams.team_1")],
        );
        assert!(ids.is_empty());
    }

    #[test]
    fn nothing_added_gives_nothing() {
        let mut r = Vec::new();
        count(&mut r, TEAMS, 1);
        row(&mut r, TEAMS, 0, 4, &[]);
        count(&mut r, DRIVERS, 1);
        row(&mut r, DRIVERS, 0, 8, &NORRIS);
        assert!(build(&Gvdb { records: r }, &[]).is_empty());
    }

    #[test]
    fn the_file_round_trips() {
        let ids = built();
        let text = serde_json::to_string(&ids).unwrap();
        assert_eq!(serde_json::from_str::<UdpIds>(&text).unwrap(), ids);
    }
}
