use std::collections::{BTreeMap, BTreeSet};

use crate::builtin::CORE_ID;
use crate::manifest::AddonManifest;
use crate::profile::Profile;
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub order: Vec<String>,
    pub warnings: Vec<String>,
}

fn glob_match(pattern: &str, id: &str) -> bool {
    match pattern.strip_suffix('*') {
        Some(prefix) => id.starts_with(prefix),
        None => pattern == id,
    }
}

pub fn resolve(profile: &Profile, installed: &BTreeMap<String, AddonManifest>) -> Result<Plan> {
    let mut warnings = Vec::new();
    if profile.is_vanilla() {
        return Ok(Plan {
            order: Vec::new(),
            warnings,
        });
    }
    let enabled: Vec<&str> = profile
        .enabled()
        .filter(|id| {
            let there = installed.contains_key(*id);
            if !there {
                warnings.push(format!(
                    "{id} is on in profile {} but isn't installed, so it is left out; install it in the Addon \
                     Browser or turn it off on the Addons page",
                    profile.name
                ));
            }
            there
        })
        .collect();
    let launcher = crate::update::current_version();
    let mut left_out: BTreeSet<&str> = BTreeSet::new();
    for id in &enabled {
        let m = &installed[*id];
        if let Some(req) = launcher_cap_passed(m, &launcher) {
            warnings.push(format!(
                "{} is left out: it is made for F1 Multiplayer {req} and this is {launcher}. Its author \
                 needs to publish an update; turn it off on the Addons page to stop this message",
                m.name
            ));
            left_out.insert(*id);
        }
    }
    while let Some((id, dep)) = enabled
        .iter()
        .filter(|id| !left_out.contains(**id))
        .find_map(|id| {
            installed[*id]
                .requires
                .iter()
                .find(|r| left_out.contains(r.id.as_str()))
                .map(|r| (*id, r.id.as_str()))
        })
    {
        warnings.push(format!(
            "{} is left out: it needs {}, which is left out",
            installed[id].name, installed[dep].name
        ));
        left_out.insert(id);
    }
    let enabled: Vec<&str> = enabled.into_iter().filter(|id| !left_out.contains(id)).collect();
    let mut addons = BTreeMap::new();
    for id in &enabled {
        addons.insert(*id, &installed[*id]);
    }
    if !addons.contains_key(CORE_ID) {
        return Err(Error::Resolve(format!("{CORE_ID} is missing")));
    }
    check_launcher(addons.values().copied(), &launcher)?;
    for m in addons.values() {
        for r in &m.requires {
            match (addons.get(r.id.as_str()), installed.get(&r.id)) {
                (None, Some(dep)) => {
                    return Err(Error::Resolve(format!(
                        "{} needs {}, which is off in profile {}. Turn {} on on the Addons page, or turn {} off",
                        m.name, dep.name, profile.name, dep.name, m.name
                    )));
                }
                (None, None) => {
                    return Err(Error::Resolve(format!(
                        "{} needs {}, which isn't installed. Install it in the Addon Browser, or turn {} off",
                        m.name, r.id, m.name
                    )));
                }
                (Some(dep), _) if !r.version.matches(&dep.version) => {
                    return Err(Error::Resolve(format!(
                        "{} needs {} {}, and {} is installed. Update {} in the Addon Browser, or turn {} off",
                        m.name, dep.name, r.version, dep.version, dep.name, m.name
                    )));
                }
                (Some(_), _) => {}
            }
        }
        for c in m.conflicts.iter().filter(|c| c.touches.is_none()) {
            if let Some(other) = addons.keys().find(|o| **o != m.id && glob_match(&c.id, o)) {
                return Err(Error::Resolve(format!("{} conflicts with {other}", m.id)));
            }
        }
        if m.conflicts.iter().any(|c| c.touches.is_some()) {
            warnings.push(format!("{}: touch conflicts are checked at build time", m.id));
        }
    }
    let name = |id: &str| {
        installed
            .get(id)
            .map_or_else(|| id.to_owned(), |m| m.name.clone())
    };
    for p in crate::order::problems(profile, installed, &crate::order::is_pinned)
        .iter()
        .filter(|p| p.kind == crate::order::Kind::Below)
    {
        warnings.push(format!(
            "{}: {}; the build loads it first anyway. Order for me on the Addons page puts it there in the list",
            name(&p.addon),
            p.line(&name)
        ));
    }
    let position: BTreeMap<&str, usize> = enabled.iter().enumerate().map(|(i, id)| (*id, i)).collect();
    let mut after: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for m in addons.values() {
        let deps = after.entry(m.id.as_str()).or_default();
        for r in &m.requires {
            deps.insert(r.id.as_str());
        }
        for pat in &m.load_after {
            deps.extend(addons.keys().filter(|o| **o != m.id && glob_match(pat, o)));
        }
        if m.id != CORE_ID {
            deps.insert(CORE_ID);
        }
    }
    let mut order = Vec::new();
    let mut done = BTreeSet::new();
    while done.len() < addons.len() {
        let next = addons
            .keys()
            .filter(|id| !done.contains(*id) && after[*id].iter().all(|d| done.contains(d)))
            .min_by_key(|id| position[*id]);
        let Some(&id) = next else {
            let stuck: Vec<&str> = addons.keys().filter(|id| !done.contains(*id)).copied().collect();
            return Err(Error::Resolve(format!(
                "load order cycle among {}",
                stuck.join(", ")
            )));
        };
        done.insert(id);
        order.push(id.to_owned());
    }
    Ok(Plan { order, warnings })
}

/// The addons on in `profile` that two or more catalog addons build on, with
/// none of those on: the shared part of a family such as Classic Seasons,
/// which adds nothing to pick on its own. When the base has a catalog
/// category, its family is the addons of that category that need it, so a
/// career addon that needs Classic Seasons doesn't count as a season. Each
/// comes as its name and the names of its family.
pub fn lonely_bases(
    profile: &Profile,
    installed: &BTreeMap<String, AddonManifest>,
    catalog: &[crate::catalog::Addon],
) -> Vec<(String, Vec<String>)> {
    let enabled: Vec<&str> = profile.enabled().collect();
    let mut out = Vec::new();
    for id in enabled.iter().filter(|id| **id != CORE_ID) {
        let Some(base) = installed.get(*id) else {
            continue;
        };
        let category = catalog
            .iter()
            .find(|a| a.id == *id)
            .and_then(|a| a.category.as_deref());
        let needs = |a: &&crate::catalog::Addon| {
            a.id != *id
                && a.newest()
                    .and_then(|v| v.requires.as_ref())
                    .is_some_and(|rs| rs.iter().any(|r| r.id == *id))
        };
        let family: Vec<&crate::catalog::Addon> = catalog
            .iter()
            .filter(needs)
            .filter(|a| category.is_none() || a.category.as_deref() == category)
            .collect();
        if family.len() < 2 {
            continue;
        }
        let used = enabled.iter().filter_map(|e| installed.get(*e)).any(|m| {
            m.requires.iter().any(|r| r.id == *id)
                && (family.iter().any(|a| a.id == m.id) || !catalog.iter().any(|a| a.id == m.id))
        });
        if !used {
            let mut names: Vec<String> = family.iter().map(|a| a.name.clone()).collect();
            names.sort();
            out.push((base.name.clone(), names));
        }
    }
    out
}

fn launcher_cap_passed(m: &AddonManifest, launcher: &semver::Version) -> Option<semver::VersionReq> {
    use semver::Op;
    let req = m.compat.launcher.as_ref().filter(|r| !r.matches(launcher))?;
    let failed: Vec<&semver::Comparator> = req.comparators.iter().filter(|c| !c.matches(launcher)).collect();
    let too_new = !failed.is_empty()
        && failed.iter().all(|c| match c.op {
            Op::Greater | Op::GreaterEq => false,
            Op::Less | Op::LessEq => true,
            _ => *launcher > semver::Version::new(c.major, c.minor.unwrap_or(0), c.patch.unwrap_or(0)),
        });
    (m.id != CORE_ID && too_new).then(|| req.clone())
}

pub fn launcher_refusal(m: &AddonManifest, launcher: &semver::Version) -> Option<String> {
    let req = m.compat.launcher.as_ref().filter(|r| !r.matches(launcher))?;
    Some(format!(
        "{} needs F1 Multiplayer {req}; this is {launcher}. Update the launcher, or turn the addon off",
        m.id
    ))
}

fn check_launcher<'a>(
    addons: impl Iterator<Item = &'a AddonManifest>,
    launcher: &semver::Version,
) -> Result<()> {
    for m in addons {
        if let Some(why) = launcher_refusal(m, launcher) {
            return Err(Error::Resolve(why));
        }
    }
    Ok(())
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

    #[test]
    fn core_first_then_load_after() {
        let installed = BTreeMap::from([
            m(CORE_ID, ""),
            m("a.grid", r#","load_after":["a.track.*"]"#),
            m("a.track.okayama", ""),
        ]);
        let plan = resolve(&profile(&["a.grid", "a.track.okayama"]), &installed).unwrap();
        assert_eq!(plan.order, [CORE_ID, "a.track.okayama", "a.grid"]);
        assert_eq!(plan.warnings.len(), 1);
        assert!(
            plan.warnings[0].starts_with("a.grid: Needs a.track.okayama above it; the build loads it first")
        );
    }

    #[test]
    fn moved_addons_load_in_their_new_place() {
        let installed = BTreeMap::from([m(CORE_ID, ""), m("dbedit.a", ""), m("dbedit.b", "")]);
        let mut p = profile(&["dbedit.a", "dbedit.b"]);
        assert_eq!(
            resolve(&p, &installed).unwrap().order,
            [CORE_ID, "dbedit.a", "dbedit.b"]
        );
        p.move_addon("dbedit.b", -1).unwrap();
        assert_eq!(
            resolve(&p, &installed).unwrap().order,
            [CORE_ID, "dbedit.b", "dbedit.a"]
        );
    }

    #[test]
    fn missing_requirement_fails() {
        let installed = BTreeMap::from([
            m(CORE_ID, ""),
            m("a.x", r#","requires":[{"id":"a.y"}]"#),
            m("a.y", ""),
        ]);
        assert!(resolve(&profile(&["a.x"]), &installed).is_err());
        assert!(resolve(&profile(&["a.x", "a.y"]), &installed).is_ok());
    }

    #[test]
    fn requirement_errors_name_the_addons_and_the_fix() {
        let (_, mut x) = m("a.x", r#","requires":[{"id":"a.y"}]"#);
        x.name = "Season 2019".into();
        let (_, mut y) = m("a.y", "");
        y.name = "Classic Seasons".into();
        let installed = BTreeMap::from([m(CORE_ID, ""), ("a.x".into(), x.clone()), ("a.y".into(), y)]);
        let off = resolve(&profile(&["a.x"]), &installed).unwrap_err().to_string();
        assert!(
            off.contains(
                "Season 2019 needs Classic Seasons, which is off in profile t. Turn Classic Seasons on"
            ),
            "{off}"
        );
        let installed = BTreeMap::from([m(CORE_ID, ""), ("a.x".into(), x)]);
        let missing = resolve(&profile(&["a.x"]), &installed).unwrap_err().to_string();
        assert!(
            missing.contains("Season 2019 needs a.y, which isn't installed"),
            "{missing}"
        );
    }

    #[test]
    fn an_addon_on_but_not_installed_is_left_out() {
        let installed = BTreeMap::from([m(CORE_ID, ""), m("a.x", "")]);
        let plan = resolve(&profile(&["a.x", "a.gone"]), &installed).unwrap();
        assert_eq!(plan.order, [CORE_ID, "a.x"]);
        assert!(
            plan.warnings
                .iter()
                .any(|w| w.starts_with("a.gone is on in profile t but isn't installed"))
        );
    }

    #[test]
    fn a_base_with_none_of_its_family_on_is_lonely() {
        let catalog: Vec<crate::catalog::Addon> = serde_json::from_value(serde_json::json!([
            {"id": "cs.2016", "name": "F1 2016", "author": "s",
             "versions": [{"version": "1.0.0", "size": 1, "sha256": "x", "url": "u", "requires": [{"id": "cs.base"}]}]},
            {"id": "cs.2017", "name": "F1 2017", "author": "s",
             "versions": [{"version": "1.0.0", "size": 1, "sha256": "x", "url": "u", "requires": [{"id": "cs.base"}]}]},
            {"id": "a.other", "name": "Other", "author": "s",
             "versions": [{"version": "1.0.0", "size": 1, "sha256": "x", "url": "u"}]}
        ]))
        .unwrap();
        let (_, mut base) = m("cs.base", "");
        base.name = "Classic Seasons".into();
        let installed = BTreeMap::from([
            m(CORE_ID, ""),
            ("cs.base".into(), base),
            m("cs.2016", r#","requires":[{"id":"cs.base"}]"#),
            m("a.other", ""),
        ]);
        assert_eq!(
            lonely_bases(&profile(&["cs.base", "a.other"]), &installed, &catalog),
            [(
                "Classic Seasons".to_owned(),
                vec!["F1 2016".to_owned(), "F1 2017".to_owned()]
            )]
        );
        assert!(lonely_bases(&profile(&["cs.base", "cs.2016"]), &installed, &catalog).is_empty());
        assert!(lonely_bases(&profile(&["a.other"]), &installed, &catalog).is_empty());
    }

    #[test]
    fn a_career_addon_on_the_base_is_not_one_of_its_seasons() {
        let catalog: Vec<crate::catalog::Addon> = serde_json::from_value(serde_json::json!([
            {"id": "cs.base", "name": "Classic Seasons", "author": "s", "category": "seasons",
             "versions": [{"version": "1.0.0", "size": 1, "sha256": "x", "url": "u"}]},
            {"id": "cs.2016", "name": "F1 2016", "author": "s", "category": "seasons",
             "versions": [{"version": "1.0.0", "size": 1, "sha256": "x", "url": "u", "requires": [{"id": "cs.base"}]}]},
            {"id": "cs.2017", "name": "F1 2017", "author": "s", "category": "seasons",
             "versions": [{"version": "1.0.0", "size": 1, "sha256": "x", "url": "u", "requires": [{"id": "cs.base"}]}]},
            {"id": "c.career", "name": "Classic Career", "author": "c", "category": "career",
             "versions": [{"version": "1.0.0", "size": 1, "sha256": "x", "url": "u", "requires": [{"id": "cs.base"}]}]}
        ]))
        .unwrap();
        let (_, mut base) = m("cs.base", "");
        base.name = "Classic Seasons".into();
        let installed = BTreeMap::from([
            m(CORE_ID, ""),
            ("cs.base".into(), base),
            m("cs.2016", r#","requires":[{"id":"cs.base"}]"#),
            m("c.career", r#","requires":[{"id":"cs.base"}]"#),
            m("a.local", r#","requires":[{"id":"cs.base"}]"#),
        ]);
        assert_eq!(
            lonely_bases(&profile(&["c.career", "cs.base"]), &installed, &catalog),
            [(
                "Classic Seasons".to_owned(),
                vec!["F1 2016".to_owned(), "F1 2017".to_owned()]
            )]
        );
        assert!(
            lonely_bases(
                &profile(&["c.career", "cs.base", "cs.2016"]),
                &installed,
                &catalog
            )
            .is_empty()
        );
        assert!(lonely_bases(&profile(&["cs.base", "a.local"]), &installed, &catalog).is_empty());
    }

    #[test]
    fn launcher_range_is_enforced() {
        let newer = AddonManifest::parse(
            r#"{"schema_version":1,"id":"a.newer","name":"a.newer","version":"1.0.0","author":"t",
            "compat":{"game":"f1_25","launcher":">=1.0.9"}}"#,
        )
        .unwrap();
        let (_, any) = m("a.any", "");
        let old = semver::Version::new(1, 0, 8);
        let new = semver::Version::new(1, 0, 9);
        assert!(check_launcher([&newer, &any].into_iter(), &new).is_ok());
        let err = check_launcher([&any, &newer].into_iter(), &old)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("a.newer needs F1 Multiplayer >=1.0.9; this is 1.0.8"),
            "{err}"
        );
    }

    #[test]
    fn an_addon_capped_below_this_launcher_is_left_out_with_its_dependents() {
        let capped = r#","compat":{"game":"f1_25","launcher":">=1.6.11, <1.6.13"}"#;
        let installed = BTreeMap::from([
            m(CORE_ID, ""),
            (
                "j.orders".to_owned(),
                AddonManifest::parse(&format!(
                    r#"{{"schema_version":1,"id":"j.orders","name":"Team Orders","version":"0.2.0","author":"t"{capped}}}"#
                ))
                .unwrap(),
            ),
            m("a.uses", r#","requires":[{"id":"j.orders","version":">=0.1.0"}]"#),
            m("a.other", ""),
        ]);
        let launcher = crate::update::current_version();
        assert!(launcher >= semver::Version::new(1, 6, 13));
        let plan = resolve(&profile(&["j.orders", "a.uses", "a.other"]), &installed).unwrap();
        assert!(
            !plan.order.iter().any(|id| id == "j.orders" || id == "a.uses"),
            "{:?}",
            plan.order
        );
        assert!(plan.order.iter().any(|id| id == "a.other"));
        assert!(
            plan.warnings.iter().any(|w| w
                .starts_with("Team Orders is left out: it is made for F1 Multiplayer >=1.6.11, <1.6.13")),
            "{:?}",
            plan.warnings
        );
        assert!(
            plan.warnings
                .iter()
                .any(|w| w == "a.uses is left out: it needs Team Orders, which is left out"),
            "{:?}",
            plan.warnings
        );
    }

    #[test]
    fn only_a_launcher_above_the_range_counts_as_past_the_cap() {
        let with = |req: &str| {
            AddonManifest::parse(&format!(
                r#"{{"schema_version":1,"id":"a.r","name":"a.r","version":"1.0.0","author":"t",
                "compat":{{"game":"f1_25","launcher":"{req}"}}}}"#
            ))
            .unwrap()
        };
        let v = |s: &str| semver::Version::parse(s).unwrap();
        assert!(launcher_cap_passed(&with(">=1.6.11, <1.6.13"), &v("1.7.0")).is_some());
        assert!(launcher_cap_passed(&with("<=1.6.12"), &v("1.7.0")).is_some());
        assert!(launcher_cap_passed(&with("^1.5"), &v("2.0.0")).is_some());
        assert!(launcher_cap_passed(&with("~1.6.11"), &v("1.7.0")).is_some());
        assert!(launcher_cap_passed(&with("^1.8"), &v("1.7.0")).is_none());
        assert!(launcher_cap_passed(&with(">=1.8.0"), &v("1.7.0")).is_none());
        assert!(launcher_cap_passed(&with(">=1.8.0, <2.0.0"), &v("1.7.0")).is_none());
        assert!(launcher_cap_passed(&with(">=1.6.0"), &v("1.7.0")).is_none());
    }

    #[test]
    fn an_addon_that_needs_a_newer_launcher_still_stops_the_build() {
        let installed = BTreeMap::from([
            m(CORE_ID, ""),
            (
                "a.future".to_owned(),
                AddonManifest::parse(
                    r#"{"schema_version":1,"id":"a.future","name":"a.future","version":"1.0.0","author":"t",
                    "compat":{"game":"f1_25","launcher":">=99.0.0, <100.0.0"}}"#,
                )
                .unwrap(),
            ),
        ]);
        let err = resolve(&profile(&["a.future"]), &installed)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("a.future needs F1 Multiplayer >=99.0.0, <100.0.0"),
            "{err}"
        );
    }

    #[test]
    fn cycles_fail() {
        let installed = BTreeMap::from([
            m(CORE_ID, ""),
            m("a.x", r#","load_after":["a.y"]"#),
            m("a.y", r#","load_after":["a.x"]"#),
        ]);
        assert!(resolve(&profile(&["a.x", "a.y"]), &installed).is_err());
    }
}
