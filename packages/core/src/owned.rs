use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::Range;

use serde::{Deserialize, Serialize};
use swofty_formats::gvdb::{Gvdb, instance_count_path};
use swofty_formats::hash::fnv1a64;

use crate::settings::djb2;

pub const TYPE_NAMES: [&str; 28] = [
    "eLivery",
    "eAvatarHelmet",
    "eAvatarGlove",
    "eAvatarSuit",
    "eAvatarOutfit",
    "eDLC",
    "eVehicle",
    "eDriverHead",
    "eAvatarPodiumEmote",
    "eAvatarPose",
    "eCurrencyReward",
    "ePlayerBadgeBackground",
    "ePlayerBadgeEmblem",
    "eDecalSet",
    "eItemPack",
    "ePersonalSticker",
    "eVictoryVO",
    "eEyewear",
    "eCasualHeadwear",
    "eCasualFootwear",
    "eTops",
    "eTrousers",
    "eBoosts",
    "eDriver",
    "eSELivery",
    "eDriverNumber",
    "eTeam",
    "eDecalGroup",
];

pub const UNLOCK_ITEMS: &str = "unlockable_content.unlockable_items";
pub const UNLOCK_LISTS: &str = "unlockable_content.unlockable_items.unlockable_item_list_";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnedRow {
    pub row: &'static str,
    pub type_name: &'static str,
    pub items: &'static str,
    pub item_row: &'static str,
}

pub const OWNED_ROWS: &[OwnedRow] = &[OwnedRow {
    row: "unlockable_se_livery",
    type_name: TYPE_NAMES[0x18],
    items: "customisation.custom_se_liveries.se_liveries",
    item_row: "se_livery",
}];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OwnedItem {
    pub key: String,
    #[serde(alias = "mod")]
    pub addon: String,
    pub what: String,
}

pub type Counts = BTreeMap<(String, &'static str), usize>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Added {
    pub addon: String,
    pub table: String,
    pub kind: &'static OwnedRow,
    pub rows: Range<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Owned {
    pub items: Vec<OwnedItem>,
    pub lines: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn item_key(type_name: &str, label: &str) -> u32 {
    djb2(&format!("{type_name}{label}"))
}

pub fn key_text(key: u32) -> String {
    format!("{key:#010x}")
}

pub fn counts(db: &Gvdb) -> Counts {
    let mut index = HashMap::with_capacity(db.records.len());
    for r in &db.records {
        index.entry(r.key).or_insert(r);
    }
    let count = |path: String| -> Option<usize> { index.get(&fnv1a64(path))?.text().trim().parse().ok() };
    let rows = |table: &str, row: &str| {
        count(instance_count_path(table, row)).unwrap_or_else(|| {
            (0..)
                .take_while(|i| index.contains_key(&fnv1a64(format!("{table}.{row}_{i}._metadata_"))))
                .count()
        })
    };
    let mut out = Counts::new();
    for l in 0..rows(UNLOCK_ITEMS, "unlockable_item_list") {
        let list = format!("{UNLOCK_LISTS}{l}");
        for f in 0..rows(&list, "folder") {
            let table = format!("{list}.folder_{f}");
            for kind in OWNED_ROWS {
                if let Some(n) = count(instance_count_path(&table, kind.row)) {
                    out.insert((table.clone(), kind.row), n);
                }
            }
        }
    }
    out
}

pub fn added(db: &Gvdb, addon: &str, before: &Counts) -> Vec<Added> {
    counts(db)
        .into_iter()
        .filter_map(|((table, row), after)| {
            let from = before.get(&(table.clone(), row)).copied().unwrap_or(0);
            let kind = OWNED_ROWS.iter().find(|k| k.row == row)?;
            (after > from).then(|| Added {
                addon: addon.to_owned(),
                table,
                kind,
                rows: from..after,
            })
        })
        .collect()
}

pub fn stock_collisions(pristine: &Gvdb, keys: &HashSet<u32>) -> HashSet<u32> {
    let prefixes: Vec<u32> = TYPE_NAMES.iter().map(|t| djb2(t)).collect();
    let mut out = HashSet::new();
    if keys.is_empty() {
        return out;
    }
    for r in &pristine.records {
        let (scale, tail) = r.value.iter().fold((1u32, 0u32), |(s, t), &b| {
            (s.wrapping_mul(33), t.wrapping_mul(33).wrapping_add(u32::from(b)))
        });
        for p in &prefixes {
            let k = p.wrapping_mul(scale).wrapping_add(tail);
            if keys.contains(&k) {
                out.insert(k);
            }
        }
    }
    out
}

fn item_id(db: &Gvdb, kind: &OwnedRow, label: &str) -> Option<String> {
    let n = db.instance_count(kind.items, kind.item_row)?;
    (0..n).find_map(|i| {
        let row = format!("{}.{}_{i}", kind.items, kind.item_row);
        (db.get_text(&format!("{row}._metadata_")).as_deref() == Some(label))
            .then(|| db.get_text(&format!("{row}.id")))
            .flatten()
    })
}

pub fn owned_items(db: &Gvdb, pristine: &Gvdb, added: &[Added]) -> Owned {
    let mut found = Vec::new();
    for a in added {
        for i in a.rows.clone() {
            let row = format!("{}.{}_{i}", a.table, a.kind.row);
            let Some(label) = db.get_text(&format!("{row}._metadata_")) else {
                continue;
            };
            let item = db
                .get_text(&format!("{row}.customisation_item"))
                .unwrap_or_else(|| label.clone());
            let what = format!(
                "{} {}",
                a.kind.item_row,
                item_id(db, a.kind, &item).unwrap_or_else(|| label.clone())
            );
            let key = item_key(a.kind.type_name, &label);
            found.push((a, label, key, what));
        }
    }
    let keys: HashSet<u32> = found.iter().map(|f| f.2).collect();
    let stock = stock_collisions(pristine, &keys);
    let mut out = Owned::default();
    let mut seen = HashSet::new();
    for (a, label, key, what) in found {
        let k = key_text(key);
        if stock.contains(&key) {
            out.warnings.push(format!(
                "{}: unlock row {label} has key {k}, which a stock game row also gives; not reported as owned",
                a.addon
            ));
        } else if !seen.insert(key) {
            out.warnings.push(format!(
                "{}: unlock row {label} repeats key {k}; reported once",
                a.addon
            ));
        } else {
            out.lines.push(format!(
                "items: {} unlock row {label} ({what}) is reported as owned, key {k}",
                a.addon
            ));
            out.items.push(OwnedItem {
                key: k,
                addon: a.addon.clone(),
                what,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::{DbOps, apply_db_ops};
    use swofty_formats::gvdb::Record;

    const ADDON: &str = "rmdesigns.alphatauri-orlen-livery";
    const FOLDER: &str = "unlockable_content.unlockable_items.unlockable_item_list_2.folder_23";
    const SE: &str = "customisation.custom_se_liveries.se_liveries";

    fn pristine() -> Gvdb {
        let mut records = vec![
            Record::new(
                &format!("{UNLOCK_ITEMS}._unlockable_item_list_instancecount_"),
                0,
                "3",
            ),
            Record::new(&format!("{UNLOCK_LISTS}2._folder_instancecount_"), 0, "24"),
            Record::new(&format!("{FOLDER}._unlockable_se_livery_instancecount_"), 0, "1"),
            Record::new(&format!("{SE}._se_livery_instancecount_"), 0, "2"),
        ];
        let unlock = [
            ("_metadata_", 4, "toro_rosso_miami_26"),
            ("customisation_item", 4, "toro_rosso_miami_2026"),
            ("is_new_on_boot", 5, "1"),
            ("rarity", 3, "eRare"),
            ("statId", 0, "5200"),
            ("type_loctag", 4, "lng_se_livery"),
        ];
        for (f, tag, v) in unlock {
            records.push(Record::new(
                &format!("{FOLDER}.unlockable_se_livery_0.{f}"),
                tag,
                v,
            ));
        }
        for (i, (label, id)) in [("toro_rosso_suzuka_2026", "20"), ("toro_rosso_miami_2026", "21")]
            .iter()
            .enumerate()
        {
            let fields = [
                ("_metadata_", 4, *label),
                ("id", 0, *id),
                ("compatible_vehicle", 4, "toro_rosso_2026"),
                ("wep_override", 2, "_miami_26"),
                ("team_customisation", 4, "toro_rosso_26"),
                ("name_loc_tag", 4, "lng_se_livery_toro_rosso_miami_26"),
                ("desc_loc_tag", 4, "lng_se_livery_desc_toro_rosso_miami_26"),
                ("ui_priority", 0, *id),
            ];
            for (f, tag, v) in fields {
                records.push(Record::new(&format!("{SE}.se_livery_{i}.{f}"), tag, v));
            }
        }
        Gvdb { records }
    }

    fn ops(unlock_set: &str) -> DbOps {
        serde_json::from_str(&format!(
            r#"{{"schema_version":1,
            "fields":{{
              "{SE}.se_livery":["_metadata_","id","compatible_vehicle","wep_override","team_customisation","name_loc_tag","desc_loc_tag","ui_priority"],
              "{FOLDER}.unlockable_se_livery":["_metadata_","customisation_item","is_new_on_boot","rarity","statId","type_loctag"]
            }},
            "ops":[
              {{"op":"add-row","table":"{SE}","from":"{SE}.se_livery[_metadata_=toro_rosso_miami_2026]",
                "set":{{"_metadata_":"toro_rosso_rm_alphatauri_2026","id":"22","wep_override":"_barcelona_26",
                        "name_loc_tag":"lng_team_tor_hashtag","desc_loc_tag":"lng_team_tor_hashtag","ui_priority":"22"}}}},
              {{"op":"add-row","table":"{FOLDER}","from":"{FOLDER}.unlockable_se_livery[_metadata_=toro_rosso_miami_26]",
                "set":{{{unlock_set}"customisation_item":"toro_rosso_rm_alphatauri_2026","statId":"5280"}}}}
            ]}}"#
        ))
        .unwrap()
    }

    fn build(addons: &[(&str, DbOps)]) -> (Gvdb, Owned) {
        let base = pristine();
        let mut db = base.clone();
        let mut all = Vec::new();
        for (id, o) in addons {
            let before = counts(&db);
            apply_db_ops(&mut db, o, &BTreeMap::<String, Vec<String>>::new()).unwrap();
            all.extend(added(&db, id, &before));
        }
        let owned = owned_items(&db, &base, &all);
        (db, owned)
    }

    #[test]
    fn keys_match_the_game() {
        assert_eq!(TYPE_NAMES[0x18], "eSELivery");
        assert_eq!(item_key("eSELivery", "toro_rosso_rm_alphatauri_2026"), 0xf01d03c7);
        assert_eq!(item_key("eSELivery", "toro_rosso_miami_26"), 0x7f88c309);
        assert_eq!(key_text(0xf01d03c7), "0xf01d03c7");
        assert_eq!(key_text(0x1505), "0x00001505");
    }

    #[test]
    fn alphatauri_livery_is_reported_as_owned() {
        let o = ops(r#""_metadata_":"toro_rosso_rm_alphatauri_2026","#);
        let (_, owned) = build(&[(ADDON, o)]);
        assert_eq!(
            owned.items,
            vec![OwnedItem {
                key: "0xf01d03c7".into(),
                addon: ADDON.into(),
                what: "se_livery 22".into(),
            }]
        );
        assert!(owned.warnings.is_empty(), "{:?}", owned.warnings);
        assert_eq!(
            serde_json::to_value(&owned.items[0]).unwrap(),
            serde_json::json!({"key": "0xf01d03c7", "addon": ADDON, "what": "se_livery 22"})
        );
        let old = serde_json::json!({"key": "0xf01d03c7", "mod": ADDON, "what": "se_livery 22"});
        assert_eq!(serde_json::from_value::<OwnedItem>(old).unwrap(), owned.items[0]);
    }

    #[test]
    fn stock_keys_are_never_emitted() {
        let stock = key_text(item_key("eSELivery", "toro_rosso_miami_26"));
        let copied = ops("");
        let (_, owned) = build(&[("a.copy", copied)]);
        assert!(owned.items.is_empty(), "{:?}", owned.items);
        assert_eq!(owned.warnings.len(), 1);
        assert!(owned.warnings[0].contains(&stock));

        let reused = ops(r#""_metadata_":"toro_rosso_2026","#);
        let (_, owned) = build(&[("a.reuse", reused)]);
        assert!(owned.items.is_empty(), "{:?}", owned.items);

        let edit: DbOps = serde_json::from_str(&format!(
            r#"{{"schema_version":1,"ops":[{{"op":"set","select":"{FOLDER}.unlockable_se_livery[_metadata_=toro_rosso_miami_26]","field":"statId","value":"1"}}]}}"#
        ))
        .unwrap();
        let (_, owned) = build(&[("a.edit", edit)]);
        assert!(owned.items.is_empty());
        assert!(owned.warnings.is_empty());

        let mut base = pristine();
        base.records.push(Record::new(
            "elsewhere.row_0._metadata_",
            4,
            "toro_rosso_rm_alphatauri_2026",
        ));
        let o = ops(r#""_metadata_":"toro_rosso_rm_alphatauri_2026","#);
        let mut db = base.clone();
        let before = counts(&db);
        apply_db_ops(&mut db, &o, &BTreeMap::<String, Vec<String>>::new()).unwrap();
        let owned = owned_items(&db, &base, &added(&db, ADDON, &before));
        assert!(owned.items.is_empty(), "{:?}", owned.items);
    }

    #[test]
    fn stock_collisions_cover_every_type_name() {
        let mut base = Gvdb::default();
        base.records.push(Record::new("x._metadata_", 4, "label"));
        let keys: HashSet<u32> = TYPE_NAMES.iter().map(|t| item_key(t, "label")).collect();
        assert_eq!(stock_collisions(&base, &keys), keys);
        let fresh = HashSet::from([item_key("eSELivery", "other")]);
        assert!(stock_collisions(&base, &fresh).is_empty());
    }

    #[test]
    fn second_addon_rows_belong_to_that_addon() {
        let a = ops(r#""_metadata_":"toro_rosso_rm_alphatauri_2026","#);
        let b: DbOps = serde_json::from_str(&format!(
            r#"{{"schema_version":1,
            "fields":{{"{FOLDER}.unlockable_se_livery":["_metadata_","customisation_item","is_new_on_boot","rarity","statId","type_loctag"]}},
            "ops":[{{"op":"add-row","table":"{FOLDER}","from":"{FOLDER}.unlockable_se_livery[_metadata_=toro_rosso_miami_26]",
                    "set":{{"_metadata_":"b_livery","customisation_item":"b_livery","statId":"5281"}}}}]}}"#
        ))
        .unwrap();
        let (_, owned) = build(&[(ADDON, a), ("b.addon", b)]);
        let got: Vec<(&str, &str, &str)> = owned
            .items
            .iter()
            .map(|i| (i.key.as_str(), i.addon.as_str(), i.what.as_str()))
            .collect();
        let bkey = key_text(item_key("eSELivery", "b_livery"));
        assert_eq!(
            got,
            vec![
                ("0xf01d03c7", ADDON, "se_livery 22"),
                (bkey.as_str(), "b.addon", "se_livery b_livery")
            ]
        );
    }
}
