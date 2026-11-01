use std::collections::BTreeMap;

use md5::{Digest as _, Md5};
use serde_json::{Value as Json, json};
use swofty_formats::gvdb::Gvdb;
use swofty_formats::pkg::{Node, Value};

use crate::ops::PkgEdit;
use crate::pkgedit::{apply_edits, value_text};
use crate::{Error, Result};

pub const GRID_LIST: &str = "entrants.entrant_lists.entrant_list_34";
const KINDS: [(&str, u32); 4] = [("PitGarage", 2), ("F1Garage", 1), ("PitBox", 1), ("GridSlot", 2)];

pub fn grid_size(db: &Gvdb) -> u32 {
    db.instance_count(GRID_LIST, "entrant").map_or(0, |n| n as u32)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Room {
    pub grid_slots: u32,
    pub garages: u32,
    pub pit_boxes: u32,
}

impl Room {
    pub fn holds(&self, cars: u32) -> bool {
        self.grid_slots >= cars && self.garages >= cars.div_ceil(2) && self.pit_boxes >= cars.div_ceil(2)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extension {
    pub layer: usize,
    pub before: Room,
    pub after: Room,
}

struct Spot {
    layer: usize,
    name: String,
    uid: String,
    transform: [f32; 16],
}

type Spots = BTreeMap<(&'static str, u32), Spot>;

fn wanted(kind: &str, cars: u32) -> u32 {
    match kind {
        "F1Garage" | "PitBox" => cars.div_ceil(2),
        _ => cars,
    }
}

fn kind_of(name: &str) -> Option<(&'static str, u32)> {
    let tail = name.rsplit('$').next()?;
    KINDS.iter().find_map(|(kind, _)| {
        let n = tail.strip_prefix(kind)?;
        let n = n.bytes().all(|b| b.is_ascii_digit()).then_some(n)?;
        Some((*kind, n.parse().ok()?))
    })
}

fn transform(entry: &Node) -> Option<[f32; 16]> {
    let m = entry
        .as_obj()?
        .get("overrides")?
        .as_obj()?
        .get("WorldObject")?
        .as_obj()?
        .get("transform")?
        .as_val()?;
    let Value::Raw(b) = &m.value else {
        return None;
    };
    let mut out = [0f32; 16];
    for (i, v) in out.iter_mut().enumerate() {
        *v = f32::from_le_bytes(b.get(4 * i..4 * i + 4)?.try_into().ok()?);
    }
    Some(out)
}

fn spots(layers: &[Node]) -> Spots {
    let mut out = Spots::new();
    for (layer, root) in layers.iter().enumerate() {
        let entries = root
            .as_obj()
            .and_then(|o| o.get("instances"))
            .and_then(Node::as_arr);
        for entry in entries.into_iter().flat_map(|a| &a.items) {
            let Ok(name) = value_text(entry, "name") else {
                continue;
            };
            let (Some(key), Some(transform), Ok(uid)) =
                (kind_of(&name), transform(entry), value_text(entry, "uniqueid"))
            else {
                continue;
            };
            out.insert(
                key,
                Spot {
                    layer,
                    name,
                    uid,
                    transform,
                },
            );
        }
    }
    out
}

fn top(spots: &Spots, kind: &str) -> u32 {
    spots
        .keys()
        .filter(|(k, _)| *k == kind)
        .map(|(_, n)| *n)
        .max()
        .unwrap_or(0)
}

fn room(spots: &Spots) -> Room {
    Room {
        grid_slots: top(spots, "GridSlot"),
        garages: top(spots, "F1Garage"),
        pit_boxes: top(spots, "PitBox"),
    }
}

/// The grid slots, garages and pit boxes the world layers `layers` hold.
pub fn room_of(layers: &[Node]) -> Room {
    room(&spots(layers))
}

fn uid(code: &str, name: &str) -> String {
    crate::baseline::hex(&Md5::digest(format!("grid:{code}:{name}").as_bytes()))
}

pub fn extend(code: &str, layers: &mut [Node], cars: u32) -> Result<Option<Extension>> {
    let mut spots = spots(layers);
    let before = room(&spots);
    if before.holds(cars) {
        return Ok(None);
    }
    let last = spots
        .get(&("GridSlot", before.grid_slots))
        .ok_or_else(|| Error::Map("the world has no grid slots".into()))?;
    let layer = last.layer;
    let prefix = last.name[..last.name.rfind('$').map_or(0, |i| i + 1)].to_owned();
    let mut edits = Vec::new();
    for (kind, step) in KINDS {
        for n in top(&spots, kind) + 1..=wanted(kind, cars) {
            let missing = |m: u32| Error::Map(format!("the world has no {kind}{m} to space {kind}{n} from"));
            let near = |back: u32| {
                n.checked_sub(back)
                    .and_then(|m| spots.get(&(kind, m)))
                    .ok_or_else(|| missing(n.saturating_sub(back)))
            };
            let (a, b) = (near(step)?, near(2 * step)?);
            let src = spots
                .iter()
                .filter(|((k, m), s)| *k == kind && m % step == n % step && s.layer == layer)
                .map(|(_, s)| s.name.clone())
                .next_back()
                .ok_or_else(|| Error::Map(format!("the grid's layer has no {kind} to copy")))?;
            let mut t = a.transform;
            for ((v, a), b) in t[12..15]
                .iter_mut()
                .zip(&a.transform[12..15])
                .zip(&b.transform[12..15])
            {
                *v = 2.0 * a - b;
            }
            let name = format!("{prefix}{kind}{n}");
            let id = uid(code, &name);
            let mut set = BTreeMap::from([
                ("uniqueid".to_owned(), Json::from(id.clone())),
                ("overrides.WorldObject.transform".to_owned(), json!(t)),
            ]);
            if kind == "F1Garage" {
                let car = |m: u32| {
                    spots
                        .get(&("PitGarage", m))
                        .map(|s| Json::from(s.uid.clone()))
                        .ok_or_else(|| missing(m))
                };
                set.insert("overrides.F1Garage.id".into(), n.into());
                set.insert("overrides.F1Garage.left car placement".into(), car(2 * n - 1)?);
                set.insert("overrides.F1Garage.right car placement".into(), car(2 * n)?);
            } else {
                set.insert("overrides.VehPlacement.Id".into(), n.into());
            }
            edits.push(PkgEdit::CloneEntry {
                at: "instances".into(),
                from: None,
                src: format!("[name={src}]"),
                name: name.clone(),
                set,
            });
            spots.insert(
                (kind, n),
                Spot {
                    layer,
                    name,
                    uid: id,
                    transform: t,
                },
            );
        }
    }
    apply_edits(&mut layers[layer], &edits, &|_| None)?;
    Ok(Some(Extension {
        layer,
        before,
        after: room(&spots),
    }))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use swofty_formats::pkg::{Arr, Obj, Pkg, Val};

    fn entry(name: &str, at: [f32; 3], fields: Vec<(&str, Val)>) -> Node {
        let mut m = [0f32; 16];
        m[0] = 1.0;
        m[5] = 1.0;
        m[10] = 1.0;
        m[12..15].copy_from_slice(&at);
        m[15] = 1.0;
        let m: Vec<u8> = m.iter().flat_map(|f| f.to_le_bytes()).collect();
        let group = if name.contains("$F1Garage") {
            "F1Garage"
        } else {
            "VehPlacement"
        };
        let fields = fields
            .into_iter()
            .map(|(k, v)| (Val::str(k), Node::Val(v)))
            .collect();
        Node::Obj(Obj::new(vec![
            (Val::str("name"), Node::Val(Val::str(name))),
            (
                Val::str("overrides"),
                Node::Obj(Obj::new(vec![
                    (Val::str(group), Node::Obj(Obj::new(fields))),
                    (
                        Val::str("WorldObject"),
                        Node::Obj(Obj::new(vec![(
                            Val::str("transform"),
                            Node::Val(Val::raw(*b"mat4", m)),
                        )])),
                    ),
                ])),
            ),
            (
                Val::str("uniqueid"),
                Node::Val(Val::raw(*b"woid", Md5::digest(name.as_bytes()).to_vec())),
            ),
        ]))
    }

    fn placement(prefix: &str, kind: &str, n: u32, at: [f32; 3]) -> Node {
        entry(&format!("{prefix}${kind}{n}"), at, vec![("Id", Val::u32(n))])
    }

    fn garage(prefix: &str, n: u32, x: f32) -> Node {
        let woid = |m: u32| {
            Val::raw(
                *b"woid",
                Md5::digest(format!("{prefix}$PitGarage{m}").as_bytes()).to_vec(),
            )
        };
        entry(
            &format!("{prefix}$F1Garage{n}"),
            [x, 0.0, 10.0],
            vec![
                ("id", Val::raw(*b"si32", (n as i32).to_le_bytes())),
                ("left car placement", woid(2 * n - 1)),
                ("right car placement", woid(2 * n)),
            ],
        )
    }

    fn layer(entries: Vec<Node>) -> Node {
        Node::Obj(Obj::new(vec![(
            Val::str("instances"),
            Node::Arr(Arr::new(entries)),
        )]))
    }

    pub(crate) fn world() -> Vec<Node> {
        let base = "VehiclePlacement";
        let mut common = Vec::new();
        for n in 1..=22 {
            let row = ((n - 1) / 2) as f32;
            let side = if n % 2 == 1 { 0.0 } else { 6.0 };
            common.push(placement(base, "GridSlot", n, [side, 0.0, -8.0 * row]));
        }
        for g in 1..=11 {
            let x = 15.0 * g as f32;
            common.push(placement(base, "PitGarage", 2 * g - 1, [x - 3.0, 0.0, 12.0]));
            common.push(placement(base, "PitGarage", 2 * g, [x + 3.0, 0.0, 12.0]));
            common.push(garage(base, g, x));
            common.push(placement(base, "PitBox", g, [x, 0.0, 4.0]));
        }
        let top = "vehicles_current_myteam";
        let myteam = vec![
            placement(top, "GridSlot", 23, [0.0, 0.0, -88.0]),
            placement(top, "GridSlot", 24, [6.0, 0.0, -88.0]),
            placement(top, "PitGarage", 23, [177.0, 0.0, 12.0]),
            placement(top, "PitGarage", 24, [183.0, 0.0, 12.0]),
            garage(top, 12, 180.0),
            placement(top, "PitBox", 12, [180.0, 0.0, 4.0]),
        ];
        vec![layer(common), layer(myteam)]
    }

    fn at(layers: &[Node], name: &str) -> [f32; 3] {
        let t = spots(layers)
            .into_values()
            .find(|s| s.name == name)
            .unwrap_or_else(|| panic!("{name}"))
            .transform;
        [t[12], t[13], t[14]]
    }

    #[test]
    fn measures_a_world_across_its_layers() {
        let room = room(&spots(&world()));
        assert_eq!(
            room,
            Room {
                grid_slots: 24,
                garages: 12,
                pit_boxes: 12
            }
        );
        assert!(room.holds(24));
        assert!(!room.holds(26));
    }

    #[test]
    fn leaves_a_world_with_room_alone() {
        let mut layers = world();
        assert_eq!(extend("map", &mut layers, 24).unwrap(), None);
        assert_eq!(extend("map", &mut layers, 22).unwrap(), None);
    }

    #[test]
    fn adds_slots_garages_and_boxes_one_row_and_bay_on() {
        let mut layers = world();
        let ext = extend("map", &mut layers, 28).unwrap().unwrap();
        assert_eq!(ext.layer, 1);
        assert_eq!(
            ext.after,
            Room {
                grid_slots: 28,
                garages: 14,
                pit_boxes: 14
            }
        );
        let p = "vehicles_current_myteam$";
        assert_eq!(at(&layers, &format!("{p}GridSlot25")), [0.0, 0.0, -96.0]);
        assert_eq!(at(&layers, &format!("{p}GridSlot28")), [6.0, 0.0, -104.0]);
        assert_eq!(at(&layers, &format!("{p}F1Garage14")), [210.0, 0.0, 10.0]);
        assert_eq!(at(&layers, &format!("{p}PitGarage25")), [192.0, 0.0, 12.0]);
        assert_eq!(at(&layers, &format!("{p}PitGarage28")), [213.0, 0.0, 12.0]);
        assert_eq!(at(&layers, &format!("{p}PitBox13")), [195.0, 0.0, 4.0]);
        let mut pkg = Pkg {
            name: *b"INST",
            root: layers.remove(1),
            vca: Vec::new(),
            layout: None,
        };
        let bytes = pkg.to_bytes().unwrap();
        pkg = Pkg::parse(&bytes).unwrap();
        let g = |path: &str| value_text(&pkg.root, &format!("instances[name={p}F1Garage13].{path}")).unwrap();
        let slot =
            |m: u32| value_text(&pkg.root, &format!("instances[name={p}PitGarage{m}].uniqueid")).unwrap();
        assert_eq!(g("overrides.F1Garage.id"), "0d000000");
        assert_eq!(g("overrides.F1Garage.left car placement"), slot(25));
        assert_eq!(g("overrides.F1Garage.right car placement"), slot(26));
        assert_eq!(g("uniqueid"), uid("map", &format!("{p}F1Garage13")));
        assert_eq!(
            value_text(
                &pkg.root,
                &format!("instances[name={p}GridSlot27].overrides.VehPlacement.Id")
            )
            .unwrap(),
            "1b000000"
        );
    }

    #[test]
    fn refuses_a_world_it_cannot_space() {
        let mut layers = vec![layer(vec![placement("v", "GridSlot", 1, [0.0; 3])])];
        assert!(extend("map", &mut layers, 4).is_err());
    }
}
