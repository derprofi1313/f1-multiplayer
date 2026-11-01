use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use swofty_formats::bxml::{Document, Element, Kind};

// The stock values of every track's ai_track_config.space_lines generation settings.
const ABSOLUTE_SPACE: f32 = 2.2;
const CENTRE_SHARE: f32 = 0.35;
const SIDE_SHARE: f32 = 0.15;
const MIXED: u8 = 4;
const WAYPOINT_TYPES: [&str; 9] = [
    "left_racing_limit",
    "right_racing_limit",
    "left_track_limit",
    "right_track_limit",
    "racing_line",
    "left_white_line",
    "right_white_line",
    "left_run_off",
    "right_run_off",
];

#[derive(Debug, Clone, PartialEq)]
pub enum Spacing {
    Fine,
    Fixed { doc: Document, routes: Vec<String> },
    Unfixable(Vec<String>),
}

struct Route {
    kind: String,
    gates: Vec<u32>,
}

struct Line {
    widths: BTreeMap<u32, (f32, f32)>,
    routes: Vec<Route>,
}

fn below(a: f32, b: f32) -> bool {
    !matches!(a.partial_cmp(&b), Some(Ordering::Greater | Ordering::Equal))
}

fn space(left: f32, right: f32) -> u8 {
    let (l, r) = (left.abs(), right.abs());
    if !below(ABSOLUTE_SPACE, l) && r > ABSOLUTE_SPACE {
        return 0;
    }
    let sum = l + r;
    let fl = l / sum;
    if SIDE_SHARE > fl {
        return 0;
    }
    if !below(ABSOLUTE_SPACE, r) && l > ABSOLUTE_SPACE {
        return 2;
    }
    let fr = r / sum;
    if SIDE_SHARE > fr {
        return 2;
    }
    if below(fr, CENTRE_SHARE) || below(fl, CENTRE_SHARE) {
        MIXED
    } else {
        1
    }
}

fn num<T: std::str::FromStr>(e: &Element, key: &[u8]) -> Option<T> {
    std::str::from_utf8(e.attr(key)?).ok()?.trim().parse().ok()
}

fn lengths(gate: &Element) -> BTreeMap<u32, f32> {
    gate.child(b"waypoints")
        .into_iter()
        .flat_map(|w| w.children_named(b"waypoint"))
        .filter_map(|w| Some((num(w, b"id")?, num(w, b"length")?)))
        .collect()
}

fn widths(gate: &Element) -> Option<(f32, f32)> {
    let l = lengths(gate);
    let left = l.get(&5).or_else(|| l.get(&2))?;
    let right = l.get(&6).or_else(|| l.get(&3))?;
    Some((*left, *right))
}

fn read(doc: &Document) -> Option<Line> {
    let root = &doc.root;
    let mut widths_of = BTreeMap::new();
    for g in root.child(b"gates")?.children_named(b"gate") {
        widths_of.insert(num(g, b"id")?, widths(g)?);
    }
    let mut links = BTreeMap::new();
    for l in root.child(b"links")?.children_named(b"link") {
        links.insert(
            num::<u32>(l, b"id")?,
            (num::<u32>(l, b"from_gate")?, num::<u32>(l, b"to_gate")?),
        );
    }
    let mut routes = Vec::new();
    for r in root.child(b"routes")?.children_named(b"route") {
        let (from, to) = (num::<u32>(r, b"from_link")?, num::<u32>(r, b"to_link")?);
        let mut gates = vec![links.get(&from)?.0];
        for id in from..=to {
            gates.push(links.get(&id)?.1);
        }
        if gates.iter().any(|g| !widths_of.contains_key(g)) {
            return None;
        }
        let kind = String::from_utf8_lossy(r.attr(b"type").unwrap_or(b"route")).into_owned();
        routes.push(Route { kind, gates });
    }
    Some(Line {
        widths: widths_of,
        routes,
    })
}

fn space_gates(line: &Line) -> BTreeSet<u32> {
    let mut out = BTreeSet::new();
    for route in &line.routes {
        let types: Vec<u8> = route
            .gates
            .iter()
            .map(|g| {
                let (l, r) = line.widths[g];
                space(l, r)
            })
            .collect();
        let Some(&first) = types.first() else {
            continue;
        };
        let (mut cur, mut last) = (first, 0);
        for (u, &t) in types.iter().enumerate().skip(1) {
            if t == cur {
                last = u;
            } else if t != MIXED {
                if cur != MIXED {
                    out.insert(route.gates[last]);
                }
                out.insert(route.gates[u]);
                cur = t;
                last = u;
            }
        }
    }
    out
}

fn unspaced(line: &Line) -> Vec<usize> {
    let with = space_gates(line);
    (0..line.routes.len())
        .filter(|&i| line.routes[i].gates.iter().all(|g| !with.contains(g)))
        .collect()
}

fn changed(left: f32, right: f32, kind: u8) -> (f32, f32) {
    let (l, r) = (left.abs(), right.abs());
    let sign = |old: f32, fallback: f32| if old == 0.0 { fallback } else { old.signum() };
    let (sl, sr) = (sign(left, -1.0), sign(right, 1.0));
    let (wide, narrow) = (l.max(r), l.min(r));
    let new = match kind {
        MIXED | 1 if wide > ABSOLUTE_SPACE => narrow.min(ABSOLUTE_SPACE) * 0.9,
        MIXED | 1 => wide * SIDE_SHARE * 0.5,
        _ => wide,
    };
    match r <= l {
        true => (left, sr * new),
        false => (sl * new, right),
    }
}

fn gate_mut(doc: &mut Document, id: u32) -> Option<&mut Element> {
    doc.root
        .child_mut(b"gates")?
        .children
        .iter_mut()
        .find(|g| g.kind == Kind::Element && g.name == b"gate" && num::<u32>(g, b"id") == Some(id))
}

fn set_widths(gate: &mut Element, left: f32, right: f32) -> Option<()> {
    let waypoints = gate.child_mut(b"waypoints")?;
    let have = waypoints.children_named(b"waypoint").count();
    let template = waypoints
        .children
        .iter()
        .find(|w| w.kind == Kind::Element && w.name == b"waypoint")?
        .clone();
    if have < 5 {
        return None;
    }
    if have < 7 {
        for (id, kind) in WAYPOINT_TYPES.iter().enumerate().take(7).skip(have) {
            let mut w = template.clone();
            w.children.clear();
            w.set_attr(b"id", id.to_string().as_bytes());
            w.set_attr(b"type", kind.as_bytes());
            w.set_attr(b"length", b"0");
            waypoints.children.push(w);
        }
        waypoints.set_attr(b"num_waypoints", b"7");
    }
    for (id, value) in [(5u32, left), (6, right)] {
        let w = waypoints
            .children
            .iter_mut()
            .find(|w| w.kind == Kind::Element && num::<u32>(w, b"id") == Some(id))?;
        w.set_attr(b"length", value.to_string().as_bytes());
    }
    Some(())
}

fn respace(doc: &mut Document, line: &mut Line, route: usize) -> Option<()> {
    let mut uses = BTreeMap::<u32, usize>::new();
    for r in &line.routes {
        for g in r.gates.iter().collect::<BTreeSet<_>>() {
            *uses.entry(*g).or_default() += 1;
        }
    }
    let gates = line.routes[route].gates.clone();
    let types: Vec<u8> = gates
        .iter()
        .map(|g| {
            let (l, r) = line.widths[g];
            space(l, r)
        })
        .collect();
    let run = types.iter().copied().find(|&t| t != MIXED).unwrap_or(MIXED);
    let own = |k: usize| uses[&gates[k]] == 1;
    let k = (1..gates.len().saturating_sub(1))
        .find(|&k| own(k - 1) && own(k) && own(k + 1) && types[k - 1] != MIXED && types[k + 1] != MIXED)
        .or_else(|| (1..gates.len().saturating_sub(1)).find(|&k| own(k - 1) && own(k) && own(k + 1)))?;
    let (l, r) = line.widths[&gates[k]];
    let (nl, nr) = changed(l, r, run);
    let kind = space(nl, nr);
    if kind == MIXED || kind == run {
        return None;
    }
    set_widths(gate_mut(doc, gates[k])?, nl, nr)?;
    line.widths.insert(gates[k], (nl, nr));
    Some(())
}

pub fn give_routes_space(data: &[u8]) -> Spacing {
    let Ok(mut doc) = Document::parse(data) else {
        return Spacing::Fine;
    };
    let Some(mut line) = read(&doc) else {
        return Spacing::Fine;
    };
    let broken = unspaced(&line);
    if broken.is_empty() {
        return Spacing::Fine;
    }
    let names: Vec<String> = broken.iter().map(|&i| line.routes[i].kind.clone()).collect();
    for &i in &broken {
        if respace(&mut doc, &mut line, i).is_none() {
            return Spacing::Unfixable(names);
        }
    }
    let Some(again) = read(&doc) else {
        return Spacing::Unfixable(names);
    };
    match unspaced(&again).is_empty() {
        true => Spacing::Fixed { doc, routes: names },
        false => Spacing::Unfixable(names),
    }
}

fn route_words(routes: &[String]) -> String {
    let mut words: Vec<&str> = routes
        .iter()
        .map(|r| match r.as_str() {
            "pit" => "pit lane",
            _ => "racing line",
        })
        .collect();
    words.dedup();
    words.join(" and ")
}

pub fn fixed_note(who: &str, routes: &[String]) -> String {
    format!(
        "fixed the AI {} of {who} so F1 25 doesn't crash when an AI car drives it",
        route_words(routes)
    )
}

pub fn unfixable_message(who: &str, routes: &[String], fix: &str) -> String {
    format!(
        "{who} would crash F1 25 when an AI car drives its {}: the AI racing line has no change of passing \
         space there, and the launcher can't find a gate to change. {fix}",
        route_words(routes)
    )
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    fn el(name: &str, attrs: &[(&str, String)], children: Vec<Element>) -> Element {
        Element {
            kind: Kind::Element,
            pad: 0,
            name: name.as_bytes().to_vec(),
            attrs: attrs
                .iter()
                .map(|(k, v)| (k.as_bytes().to_vec(), v.as_bytes().to_vec()))
                .collect(),
            children,
            end_pad: 0,
        }
    }

    fn gate(id: u32, left: f32, right: f32, waypoints: usize) -> Element {
        let lengths = [
            left + 0.5,
            right - 0.5,
            left - 0.2,
            right + 0.2,
            0.0,
            left,
            right,
            left - 1.0,
            right + 1.0,
        ];
        let wps = (0..waypoints)
            .map(|i| {
                el(
                    "waypoint",
                    &[
                        ("id", i.to_string()),
                        ("type", WAYPOINT_TYPES[i].to_string()),
                        ("length", lengths[i].to_string()),
                    ],
                    Vec::new(),
                )
            })
            .collect();
        el(
            "gate",
            &[("id", id.to_string()), ("name", format!("ai_gate_track_{id:03}"))],
            vec![
                el(
                    "position",
                    &[("x", "0".into()), ("y", "0".into()), ("z", "0".into())],
                    Vec::new(),
                ),
                el("waypoints", &[("num_waypoints", waypoints.to_string())], wps),
            ],
        )
    }

    // A 40-gate lap and a 12-gate pit lane from gate 32 back onto gate 7; gates 5-9 and 30-34 are as
    // wide as the first pit gate.
    pub(crate) fn aispline(pit: impl Fn(u32) -> (f32, f32), waypoints: usize) -> Document {
        let mut gates = Vec::new();
        for id in 0..40u32 {
            let side = match id / 5 {
                1 | 6 => pit(0),
                b if b % 2 == 0 => (-6.0, 1.5),
                _ => (-1.5, 6.0),
            };
            gates.push(gate(id, side.0, side.1, 9));
        }
        for n in 0..12u32 {
            let (l, r) = pit(n);
            gates.push(gate(40 + n, l, r, waypoints));
        }
        let mut links = Vec::new();
        let mut link = |from: u32, to: u32| {
            let id = links.len();
            links.push(el(
                "link",
                &[
                    ("id", id.to_string()),
                    ("from_gate", from.to_string()),
                    ("to_gate", to.to_string()),
                ],
                Vec::new(),
            ));
        };
        link(39, 0);
        for g in 1..40 {
            link(g - 1, g);
        }
        link(32, 40);
        for g in 41..52 {
            link(g - 1, g);
        }
        link(51, 7);
        let routes = vec![
            el(
                "route",
                &[
                    ("id", "0".into()),
                    ("type", "track".into()),
                    ("from_link", "0".into()),
                    ("to_link", "39".into()),
                ],
                Vec::new(),
            ),
            el(
                "route",
                &[
                    ("id", "1".into()),
                    ("type", "pit".into()),
                    ("from_link", "40".into()),
                    ("to_link", "52".into()),
                ],
                Vec::new(),
            ),
        ];
        Document {
            root: el(
                "track",
                &[("name", "default".into())],
                vec![
                    el("gates", &[("num_gates", "52".into())], gates),
                    el("links", &[("num_links", "53".into())], links),
                    el("routes", &[("num_routes", "2".into())], routes),
                ],
            ),
            tail: Vec::new(),
        }
    }

    #[test]
    fn space_types_follow_the_game() {
        assert_eq!(space(-4.1, 3.0), 1);
        assert_eq!(space(-1.5, 6.0), 0);
        assert_eq!(space(-6.0, 1.5), 2);
        assert_eq!(space(-6.0, 2.5), MIXED);
        assert_eq!(space(-2.0, 2.0), 1);
        assert_eq!(space(-2.0, 0.1), 2);
        assert_eq!(space(0.0, 0.0), MIXED);
    }

    #[test]
    fn a_line_with_space_changes_on_every_route_is_left_alone() {
        let doc = aispline(|n| if n < 6 { (-1.2, 6.0) } else { (-6.0, 1.2) }, 9);
        assert_eq!(give_routes_space(&doc.to_bytes()), Spacing::Fine);
        assert_eq!(give_routes_space(b"!pkg"), Spacing::Fine);
    }

    #[test]
    fn a_pit_lane_without_space_changes_gets_one_on_its_own_gates() {
        for (widths, waypoints) in [
            ((-4.1, 3.0), 7),
            ((-2.0, 1.8), 9),
            ((-1.5, 6.0), 7),
            ((-3.0, 3.0), 5),
        ] {
            let doc = aispline(|_| widths, waypoints);
            let before = read(&doc).unwrap();
            assert_eq!(unspaced(&before), [1], "{widths:?}");
            let Spacing::Fixed { doc: fixed, routes } = give_routes_space(&doc.to_bytes()) else {
                panic!("{widths:?} should be fixed");
            };
            assert_eq!(routes, ["pit"]);
            let bytes = fixed.to_bytes();
            assert_eq!(Document::parse(&bytes).unwrap(), fixed);
            let after = read(&fixed).unwrap();
            assert!(unspaced(&after).is_empty());
            let moved: Vec<u32> = after
                .widths
                .iter()
                .filter(|(g, w)| before.widths[g] != **w)
                .map(|(g, _)| *g)
                .collect();
            assert_eq!(moved, [41], "{widths:?}");
            let track: BTreeSet<u32> = after.routes[0].gates.iter().copied().collect();
            let spaced = space_gates(&after);
            assert_eq!(
                spaced.intersection(&track).collect::<Vec<_>>(),
                space_gates(&before).intersection(&track).collect::<Vec<_>>()
            );
            let gate = fixed
                .root
                .child(b"gates")
                .unwrap()
                .children_named(b"gate")
                .nth(41)
                .unwrap();
            assert_eq!(lengths(gate).len(), waypoints.max(7));
            assert_eq!(give_routes_space(&bytes), Spacing::Fine);
        }
    }

    #[test]
    fn a_pit_lane_without_gates_of_its_own_is_reported() {
        let mut doc = aispline(|_| (-4.1, 3.0), 9);
        doc.root.child_mut(b"links").unwrap().children[40].set_attr(b"to_gate", b"33");
        doc.root.child_mut(b"routes").unwrap().children[1].set_attr(b"to_link", b"40");
        assert_eq!(
            give_routes_space(&doc.to_bytes()),
            Spacing::Unfixable(vec!["pit".into()])
        );
    }
}
