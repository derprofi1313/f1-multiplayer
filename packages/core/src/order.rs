use std::collections::{BTreeMap, BTreeSet};

use crate::builtin::CORE_ID;
use crate::manifest::AddonManifest;
use crate::profile::Profile;
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Missing,
    Off,
    Below,
    Version { wanted: String, found: String },
    Conflict,
    Waits,
    Loses { items: usize },
    Career { label: String, why: Box<Kind> },
    Clash { why: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Red,
    Amber,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub addon: String,
    pub other: String,
    pub kind: Kind,
}

impl Problem {
    pub fn blocks_play(&self) -> bool {
        matches!(
            self.kind,
            Kind::Missing | Kind::Off | Kind::Version { .. } | Kind::Conflict
        )
    }

    pub fn severity(&self) -> Severity {
        match self.kind {
            Kind::Loses { .. } | Kind::Clash { .. } => Severity::Amber,
            _ => Severity::Red,
        }
    }

    pub fn is_order(&self) -> bool {
        matches!(self.kind, Kind::Below | Kind::Waits | Kind::Loses { .. })
    }

    pub fn line(&self, name: &dyn Fn(&str) -> String) -> String {
        let other = name(&self.other);
        match &self.kind {
            Kind::Missing => format!("Needs {other}, which isn't installed"),
            Kind::Off => format!("Needs {other}, which is off"),
            Kind::Below => format!("Needs {other} above it"),
            Kind::Version { wanted, found } => format!("Needs {other} {wanted}; {found} is installed"),
            Kind::Conflict => format!("Can't be on with {other}"),
            Kind::Waits => format!("Changes rows that {other} adds, so it needs {other} above it"),
            Kind::Loses { items } => format!("{other} loads below it and replaces {items} of its changes"),
            Kind::Career { label, why } => {
                let me = name(&self.addon);
                match &**why {
                    Kind::Missing => format!("Your career {label} needs {me}, which isn't installed"),
                    Kind::Off => format!("Your career {label} needs {me}, which is off"),
                    Kind::Version { wanted, found } => {
                        format!("Your career {label} needs {me} {wanted}; {found} is installed")
                    }
                    _ => format!("Your career {label} needs {me}"),
                }
            }
            Kind::Clash { why } => format!("{why} (with {other})"),
        }
    }
}

fn glob_match(pattern: &str, id: &str) -> bool {
    match pattern.strip_suffix('*') {
        Some(prefix) => id.starts_with(prefix),
        None => pattern == id,
    }
}

pub fn is_pinned(id: &str) -> bool {
    id == CORE_ID || crate::managed::is_managed(id)
}

fn enabled_ids(profile: &Profile) -> Vec<&str> {
    match profile.is_vanilla() {
        true => Vec::new(),
        false => profile.enabled().collect(),
    }
}

pub fn problems(
    profile: &Profile,
    installed: &BTreeMap<String, AddonManifest>,
    pinned: &dyn Fn(&str) -> bool,
) -> Vec<Problem> {
    let on = enabled_ids(profile);
    let place: BTreeMap<&str, usize> = on.iter().enumerate().map(|(i, id)| (*id, i)).collect();
    let mut out = Vec::new();
    for id in &on {
        let Some(m) = installed.get(*id) else {
            continue;
        };
        let at = place[*id];
        let movable = !pinned(id);
        let mut push = |other: &str, kind| {
            out.push(Problem {
                addon: (*id).to_owned(),
                other: other.to_owned(),
                kind,
            });
        };
        for r in &m.requires {
            match (place.get(r.id.as_str()), installed.get(&r.id)) {
                (None, Some(_)) => push(&r.id, Kind::Off),
                (None, None) => push(&r.id, Kind::Missing),
                (Some(_), None) => push(&r.id, Kind::Missing),
                (Some(_), Some(dep)) if !r.version.matches(&dep.version) => push(
                    &r.id,
                    Kind::Version {
                        wanted: r.version.to_string(),
                        found: dep.version.to_string(),
                    },
                ),
                (Some(&dep_at), Some(_)) if movable && dep_at > at => push(&r.id, Kind::Below),
                _ => {}
            }
        }
        for pat in m.load_after.iter().filter(|_| movable) {
            for (other, &other_at) in &place {
                let listed_requirement = m.requires.iter().any(|r| r.id == *other);
                if *other != *id && !listed_requirement && glob_match(pat, other) && other_at > at {
                    push(other, Kind::Below);
                }
            }
        }
        for c in m.conflicts.iter().filter(|c| c.touches.is_none()) {
            for other in place.keys().filter(|o| **o != *id && glob_match(&c.id, o)) {
                push(other, Kind::Conflict);
            }
        }
    }
    out
}

pub fn from_report(
    profile: &Profile,
    report: &crate::conflicts::Report,
    pinned: &dyn Fn(&str) -> bool,
) -> Vec<Problem> {
    if profile.is_vanilla() || !report.is_current(profile) {
        return Vec::new();
    }
    let at: BTreeMap<&str, usize> = profile.enabled().enumerate().map(|(i, id)| (id, i)).collect();
    let mut out = Vec::new();
    for w in report.waits.iter().filter(|w| !pinned(&w.addon)) {
        if let (Some(a), Some(o)) = (at.get(w.addon.as_str()), at.get(w.on.as_str()))
            && o > a
        {
            out.push(Problem {
                addon: w.addon.clone(),
                other: w.on.clone(),
                kind: Kind::Waits,
            });
        }
    }
    for p in &report.pairs {
        let items = p.database + p.strings + p.files + p.resources;
        if items > 0 {
            out.push(Problem {
                addon: p.loser.clone(),
                other: p.winner.clone(),
                kind: Kind::Loses { items },
            });
        }
    }
    out
}

pub fn careers(
    profile: &Profile,
    installed: &BTreeMap<String, AddonManifest>,
    list: &[crate::stdlib::careers::Career],
) -> Vec<Problem> {
    if profile.is_vanilla() {
        return Vec::new();
    }
    let order: Vec<String> = profile.enabled().map(str::to_owned).collect();
    let mut out = Vec::new();
    for c in list.iter().filter(|c| c.status() != "missing") {
        if c.profile.as_deref().is_some_and(|p| p != profile.name) {
            continue;
        }
        let label = match c.label.is_empty() {
            true => c.career.clone(),
            false => c.label.clone(),
        };
        for a in &c.addons {
            let range = a
                .need
                .as_deref()
                .and_then(|n| crate::stdlib::range::Range::parse(n).ok())
                .unwrap_or_else(crate::stdlib::range::Range::any);
            let why = match installed.get(&a.id) {
                None => Kind::Missing,
                Some(_) if !order.contains(&a.id) => Kind::Off,
                Some(m) if !range.matches(&m.version) => Kind::Version {
                    wanted: range.to_string(),
                    found: m.version.to_string(),
                },
                Some(_) => continue,
            };
            out.push(Problem {
                addon: a.id.clone(),
                other: c.career.clone(),
                kind: Kind::Career {
                    label: label.clone(),
                    why: Box::new(why),
                },
            });
        }
    }
    out
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ClashRule {
    a: String,
    b: String,
    why: String,
}

const CLASHES: &str = include_str!("order_clashes.json");

pub fn clashes(
    profile: &Profile,
    installed: &BTreeMap<String, AddonManifest>,
    is_track: &dyn Fn(&str) -> bool,
) -> Vec<Problem> {
    let rules: Vec<ClashRule> = serde_json::from_str(CLASHES).unwrap_or_default();
    let on: Vec<&str> = enabled_ids(profile);
    let says = |id: &str, what: &str| match what {
        "track" => is_track(id),
        key => {
            let name = installed
                .get(id)
                .map(|m| m.name.to_lowercase())
                .unwrap_or_default();
            id.to_lowercase().contains(key) || name.contains(key)
        }
    };
    let mut out = Vec::new();
    for r in &rules {
        for a in on.iter().filter(|id| says(id, &r.a)) {
            for b in on.iter().filter(|id| **id != *a && says(id, &r.b)) {
                out.push(Problem {
                    addon: (*a).to_owned(),
                    other: (*b).to_owned(),
                    kind: Kind::Clash { why: r.why.clone() },
                });
            }
        }
    }
    out
}

pub fn fix(
    profile: &mut Profile,
    installed: &BTreeMap<String, AddonManifest>,
    learned: &[Problem],
    pinned: &dyn Fn(&str) -> bool,
) -> Result<bool> {
    if profile.is_vanilla() {
        return Ok(false);
    }
    let slots: Vec<usize> = profile
        .addons
        .iter()
        .enumerate()
        .filter(|(_, a)| (a.enabled || a.id == CORE_ID) && installed.contains_key(&a.id))
        .map(|(i, _)| i)
        .collect();
    let ids: Vec<String> = slots.iter().map(|&i| profile.addons[i].id.clone()).collect();
    let index: BTreeMap<&str, usize> = ids.iter().enumerate().map(|(i, id)| (id.as_str(), i)).collect();
    let mut before: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); ids.len()];
    for (i, id) in ids.iter().enumerate().filter(|(_, id)| !pinned(id)) {
        let m = &installed[id];
        for r in &m.requires {
            if let Some(&d) = index.get(r.id.as_str()) {
                before[i].insert(d);
            }
        }
        for pat in &m.load_after {
            before[i].extend(
                ids.iter()
                    .enumerate()
                    .filter(|(j, o)| *j != i && glob_match(pat, o))
                    .map(|(j, _)| j),
            );
        }
    }
    for p in learned
        .iter()
        .filter(|p| p.kind == Kind::Waits && !pinned(&p.addon))
    {
        if let (Some(&a), Some(&o)) = (index.get(p.addon.as_str()), index.get(p.other.as_str()))
            && a != o
            && !reaches(&before, o, a)
        {
            before[a].insert(o);
        }
    }
    let mut order: Vec<usize> = Vec::with_capacity(ids.len());
    let mut done = vec![false; ids.len()];
    while order.len() < ids.len() {
        let Some(next) = (0..ids.len()).find(|&i| !done[i] && before[i].iter().all(|&d| done[d])) else {
            let stuck: Vec<String> = (0..ids.len())
                .filter(|&i| !done[i])
                .map(|i| installed[&ids[i]].name.clone())
                .collect();
            return Err(Error::Resolve(format!(
                "These addons need each other above themselves, so no order works: {}. Turn one of them off.",
                stuck.join(", ")
            )));
        };
        done[next] = true;
        order.push(next);
    }
    if order.iter().enumerate().all(|(i, &o)| i == o) {
        return Ok(false);
    }
    let mut taken: Vec<Option<crate::profile::ProfileAddon>> =
        slots.iter().map(|&i| Some(profile.addons[i].clone())).collect();
    for (slot, &o) in slots.iter().zip(&order) {
        profile.addons[*slot] = taken[o].take().expect("each addon is placed once");
    }
    Ok(true)
}

fn reaches(before: &[BTreeSet<usize>], from: usize, to: usize) -> bool {
    let mut seen = BTreeSet::new();
    let mut stack = vec![from];
    while let Some(n) = stack.pop() {
        if n == to {
            return true;
        }
        if seen.insert(n) {
            stack.extend(before[n].iter().copied());
        }
    }
    false
}

pub fn move_above(profile: &mut Profile, id: &str, anchor: &str) -> Result<()> {
    if id == CORE_ID {
        return Err(Error::Other(format!("{CORE_ID} always loads first")));
    }
    let from = profile.addons.iter().position(|a| a.id == id);
    let anchor_at = profile.addons.iter().position(|a| a.id == anchor);
    let (Some(from), Some(anchor_at)) = (from, anchor_at) else {
        return Err(Error::Other(format!(
            "{id} and {anchor} must both be in {}",
            profile.name
        )));
    };
    if from > anchor_at {
        let a = profile.addons.remove(from);
        profile.addons.insert(anchor_at, a);
    }
    Ok(())
}

pub fn lines(profile: &Profile, installed: &BTreeMap<String, AddonManifest>) -> Vec<String> {
    let name = |id: &str| {
        installed
            .get(id)
            .map_or_else(|| id.to_owned(), |m| m.name.clone())
    };
    problems(profile, installed, &is_pinned)
        .iter()
        .map(|p| format!("{}: {}", name(&p.addon), p.line(&name)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(id: &str, extra: &str) -> (String, AddonManifest) {
        let json = format!(
            r#"{{"schema_version":1,"id":"{id}","name":"{id}","version":"1.0.0","author":"t",
            "compat":{{"game":"f1_25"}}{extra}}}"#
        );
        (id.to_owned(), AddonManifest::parse(&json).unwrap())
    }

    fn profile(ids: &[&str]) -> Profile {
        let mut p = Profile::new("t");
        for id in ids {
            p.set_enabled(id, true).unwrap();
        }
        p
    }

    fn ids(p: &Profile) -> Vec<&str> {
        p.addons.iter().map(|a| a.id.as_str()).collect()
    }

    fn installed() -> BTreeMap<String, AddonManifest> {
        BTreeMap::from([
            m(CORE_ID, ""),
            m("cs.base", ""),
            m("cs.2016", r#","requires":[{"id":"cs.base"}]"#),
            m(
                "c.career",
                r#","requires":[{"id":"cs.base","version":">=2.0.0"}]"#,
            ),
            m("c.grid", r#","load_after":["track.*"]"#),
            m("track.a", ""),
            m("track.b", ""),
            m("c.rival", r#","conflicts":[{"id":"cs.*"}]"#),
        ])
    }

    fn kinds(p: &Profile) -> Vec<(String, String, Kind)> {
        problems(p, &installed(), &is_pinned)
            .into_iter()
            .map(|x| (x.addon, x.other, x.kind))
            .collect()
    }

    #[test]
    fn a_dependency_below_is_reported() {
        let p = profile(&["cs.2016", "cs.base"]);
        assert_eq!(kinds(&p), [("cs.2016".into(), "cs.base".into(), Kind::Below)]);
        assert!(!problems(&p, &installed(), &is_pinned)[0].blocks_play());
        assert!(problems(&profile(&["cs.base", "cs.2016"]), &installed(), &is_pinned).is_empty());
    }

    #[test]
    fn a_dependency_that_is_off_or_missing_is_reported() {
        let mut p = profile(&["cs.2016"]);
        p.set_enabled("cs.base", false).unwrap();
        assert_eq!(kinds(&p), [("cs.2016".into(), "cs.base".into(), Kind::Off)]);
        let mut gone = installed();
        gone.remove("cs.base");
        let found = problems(&profile(&["cs.2016"]), &gone, &is_pinned);
        assert_eq!(found[0].kind, Kind::Missing);
        assert!(found[0].blocks_play());
    }

    #[test]
    fn a_version_outside_the_range_is_reported() {
        let p = profile(&["cs.base", "c.career"]);
        let found = kinds(&p);
        assert_eq!(found.len(), 1);
        assert!(matches!(&found[0].2, Kind::Version { found, .. } if found == "1.0.0"));
    }

    #[test]
    fn load_after_below_and_conflicts_are_reported() {
        let p = profile(&["c.grid", "track.a", "track.b"]);
        assert_eq!(
            kinds(&p),
            [
                ("c.grid".into(), "track.a".into(), Kind::Below),
                ("c.grid".into(), "track.b".into(), Kind::Below)
            ]
        );
        assert_eq!(
            kinds(&profile(&["c.rival", "cs.base"])),
            [("c.rival".into(), "cs.base".into(), Kind::Conflict)]
        );
    }

    #[test]
    fn a_built_in_addon_that_loads_after_others_is_not_asked_to_move() {
        let mut all = installed();
        all.extend([m(
            crate::managed::ID,
            r#","load_after":["cs.*","track.*"],"requires":[{"id":"cs.base"}]"#,
        )]);
        let mut p = profile(&[crate::managed::ID, "cs.base", "cs.2016", "track.a"]);
        let report = crate::conflicts::Report {
            order: p.enabled().map(str::to_owned).collect(),
            waits: vec![crate::conflicts::Wait {
                addon: crate::managed::ID.into(),
                on: "track.a".into(),
            }],
            ..Default::default()
        };
        let mut found = problems(&p, &all, &is_pinned);
        found.extend(from_report(&p, &report, &is_pinned));
        assert!(found.is_empty(), "{found:?}");
        let before = ids(&p).join(",");
        assert!(!fix(&mut p, &all, &found, &is_pinned).unwrap());
        assert_eq!(ids(&p).join(","), before);
        let plan = crate::resolver::resolve(&p, &all).unwrap();
        let at = |id: &str| plan.order.iter().position(|x| x == id).unwrap();
        assert!(at("cs.2016") < at(crate::managed::ID) && at("track.a") < at(crate::managed::ID));
        assert!(
            plan.warnings.iter().all(|w| !w.contains("above it")),
            "{:?}",
            plan.warnings
        );
    }

    #[test]
    fn nothing_is_reported_for_addons_that_are_off_or_in_vanilla() {
        let mut p = profile(&["cs.2016"]);
        p.set_enabled("cs.2016", false).unwrap();
        assert!(problems(&p, &installed(), &is_pinned).is_empty());
        assert!(problems(&Profile::vanilla(), &installed(), &is_pinned).is_empty());
    }

    #[test]
    fn fix_moves_only_what_the_dependencies_need() {
        let mut p = profile(&["cs.2016", "track.a", "cs.base", "c.grid", "track.b"]);
        assert!(fix(&mut p, &installed(), &[], &is_pinned).unwrap());
        let order = ids(&p);
        let at = |id: &str| order.iter().position(|x| *x == id).unwrap();
        assert!(at("cs.base") < at("cs.2016"));
        assert!(at("track.a") < at("c.grid") && at("track.b") < at("c.grid"));
        assert!(
            problems(&p, &installed(), &is_pinned)
                .iter()
                .all(|x| x.kind != Kind::Below)
        );
        let mut fine = profile(&["cs.base", "cs.2016"]);
        assert!(!fix(&mut fine, &installed(), &[], &is_pinned).unwrap());
    }

    #[test]
    fn fix_keeps_off_addons_where_they_are() {
        let mut p = profile(&["cs.2016", "track.a", "cs.base"]);
        p.set_enabled("track.a", false).unwrap();
        let off_at = ids(&p).iter().position(|x| *x == "track.a").unwrap();
        fix(&mut p, &installed(), &[], &is_pinned).unwrap();
        assert_eq!(ids(&p)[off_at], "track.a");
    }

    #[test]
    fn fix_refuses_cycles_with_a_clear_message() {
        let all = BTreeMap::from([
            m(CORE_ID, ""),
            m("x.a", r#","requires":[{"id":"x.b"}]"#),
            m("x.b", r#","requires":[{"id":"x.a"}]"#),
        ]);
        let mut p = profile(&["x.a", "x.b"]);
        let err = fix(&mut p, &all, &[], &is_pinned).unwrap_err().to_string();
        assert!(
            err.contains("need each other") && err.contains("x.a, x.b"),
            "{err}"
        );
        assert_eq!(ids(&p)[p.addons.len() - 2..], ["x.a", "x.b"]);
    }

    #[test]
    fn fix_result_survives_folder_grouping() {
        use crate::folders::{Key, flatten, group};
        let mut p = profile(&["cs.2016", "track.a", "cs.base", "c.grid"]);
        fix(&mut p, &installed(), &[], &is_pinned).unwrap();
        let place = |id: &str| id.starts_with("track.").then(|| Key::Own("tracks".into()));
        let grouped = flatten(&group(p.addons.iter().map(|a| a.id.clone()), place));
        let mut regrouped = p.clone();
        crate::folders::reorder(&mut regrouped, &grouped, &|_| false);
        assert!(
            problems(&regrouped, &installed(), &is_pinned)
                .iter()
                .all(|x| x.kind != Kind::Below)
        );
    }

    #[test]
    fn move_above_puts_a_dependency_directly_above() {
        let mut p = profile(&["cs.2016", "track.a", "cs.base"]);
        move_above(&mut p, "cs.base", "cs.2016").unwrap();
        let order = ids(&p);
        let base = order.iter().position(|x| *x == "cs.base").unwrap();
        assert_eq!(order[base + 1], "cs.2016");
        let before = ids(&p).join(",");
        move_above(&mut p, "cs.base", "cs.2016").unwrap();
        assert_eq!(ids(&p).join(","), before);
    }

    #[test]
    fn lines_use_names() {
        let (_, mut base) = m("cs.base", "");
        base.name = "Classic Seasons (shared)".into();
        let mut all = installed();
        all.insert("cs.base".into(), base);
        let l = lines(&profile(&["cs.2016", "cs.base"]), &all);
        assert_eq!(l, ["cs.2016: Needs Classic Seasons (shared) above it"]);
    }

    #[test]
    fn rows_another_addon_adds_put_the_waiter_below_it() {
        use crate::conflicts::{Report, Wait};
        let mut p = profile(&["c.grid", "cs.base"]);
        let report = Report {
            order: p.enabled().map(str::to_owned).collect(),
            waits: vec![Wait {
                addon: "c.grid".into(),
                on: "cs.base".into(),
            }],
            pairs: vec![crate::conflicts::Pair {
                winner: "cs.base".into(),
                loser: "c.grid".into(),
                database: 2,
                ..Default::default()
            }],
            ..Default::default()
        };
        let found = from_report(&p, &report, &is_pinned);
        assert_eq!(found[0].kind, Kind::Waits);
        assert_eq!(found[1].kind, Kind::Loses { items: 2 });
        assert_eq!(found[1].severity(), Severity::Amber);
        assert_eq!(found[0].severity(), Severity::Red);
        fix(&mut p, &installed(), &found, &is_pinned).unwrap();
        let order = ids(&p);
        let at = |id: &str| order.iter().position(|x| *x == id).unwrap();
        assert!(at("cs.base") < at("c.grid"));
        let stale = Report {
            order: vec![],
            ..report
        };
        assert!(from_report(&p, &stale, &is_pinned).is_empty());
    }

    #[test]
    fn learned_orders_that_would_make_a_cycle_are_skipped() {
        let mut p = profile(&["cs.base", "cs.2016"]);
        let learned = [Problem {
            addon: "cs.base".into(),
            other: "cs.2016".into(),
            kind: Kind::Waits,
        }];
        fix(&mut p, &installed(), &learned, &is_pinned).unwrap();
        let order = ids(&p);
        let at = |id: &str| order.iter().position(|x| *x == id).unwrap();
        assert!(at("cs.base") < at("cs.2016"));
    }

    #[test]
    fn a_career_needing_an_off_addon_is_reported_on_that_addon() {
        let list = crate::stdlib::careers::parse(
            r#"{"careers":[{"career":"k1","label":"My Classic","addons":[{"id":"cs.base"},{"id":"cs.2016","need":">=1.0.0"},{"id":"gone.addon"}]}]}"#,
        );
        let p = profile(&["cs.2016"]);
        let found = careers(&p, &installed(), &list);
        let kinds: Vec<(&str, Option<&Kind>)> = found
            .iter()
            .map(|x| match &x.kind {
                Kind::Career { why, .. } => (x.addon.as_str(), Some(&**why)),
                _ => (x.addon.as_str(), None),
            })
            .collect();
        assert_eq!(
            kinds,
            [
                ("cs.base", Some(&Kind::Off)),
                ("gone.addon", Some(&Kind::Missing))
            ]
        );
        let name = |id: &str| installed().get(id).map_or(id.to_owned(), |m| m.name.clone());
        assert_eq!(
            found[0].line(&name),
            "Your career My Classic needs cs.base, which is off"
        );
    }

    #[test]
    fn known_clashes_are_amber() {
        let mut all = installed();
        all.extend([m("f.supercars", ""), m("t.mugello", "")]);
        let p = profile(&["f.supercars", "t.mugello", "cs.base"]);
        let found = clashes(&p, &all, &|id| id.starts_with("t."));
        assert_eq!(found.len(), 1);
        assert_eq!(
            (found[0].addon.as_str(), found[0].other.as_str()),
            ("f.supercars", "t.mugello")
        );
        assert_eq!(found[0].severity(), Severity::Amber);
        assert!(clashes(&profile(&["f.supercars", "cs.base"]), &all, &|_| false).is_empty());
    }
}
