//! The career's texts as templates (W4-2).

use std::collections::HashSet;

use super::texts::{self, TEXTS, Text, t};
use super::*;

/// The league's modules that make texts with `t!`.
const SOURCES: &[(&str, &str)] = &[
    ("mod.rs", include_str!("mod.rs")),
    ("aimarket.rs", include_str!("aimarket.rs")),
    ("consent.rs", include_str!("consent.rs")),
    ("driver.rs", include_str!("driver.rs")),
    ("inbox.rs", include_str!("inbox.rs")),
    ("lineup.rs", include_str!("lineup.rs")),
    ("market.rs", include_str!("market.rs")),
    ("objectives.rs", include_str!("objectives.rs")),
    ("practice.rs", include_str!("practice.rs")),
    ("progress.rs", include_str!("progress.rs")),
    ("rnd.rs", include_str!("rnd.rs")),
    ("sim.rs", include_str!("sim.rs")),
    ("solo.rs", include_str!("solo.rs")),
    ("udp.rs", include_str!("udp.rs")),
    ("weekend.rs", include_str!("weekend.rs")),
    ("gamesync/mod.rs", include_str!("gamesync/mod.rs")),
    ("gamesync/identity.rs", include_str!("gamesync/identity.rs")),
    ("gamesync/market.rs", include_str!("gamesync/market.rs")),
    ("gamesync/performance.rs", include_str!("gamesync/performance.rs")),
];

/// The string literal each `t!(` in `source` starts with.
fn templates_in(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = source;
    while let Some(at) = rest.find("t!(") {
        let before = rest[..at].chars().next_back();
        rest = &rest[at + 3..];
        if before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
            continue;
        }
        let Some(body) = rest.trim_start().strip_prefix('"') else {
            continue;
        };
        let mut text = String::new();
        let mut chars = body.chars();
        while let Some(c) = chars.next() {
            match c {
                '"' => break,
                '\\' => text.push(chars.next().expect("an escaped character")),
                c => text.push(c),
            }
        }
        out.push(text);
    }
    out
}

/// The placeholders of `template`.
fn placeholders(template: &str) -> Vec<&str> {
    template
        .split('{')
        .skip(1)
        .filter_map(|part| part.split_once('}').map(|(name, _)| name))
        .collect()
}

#[test]
fn every_template_is_listed_in_texts() {
    let mut found = HashSet::new();
    for (file, source) in SOURCES {
        for template in templates_in(source) {
            assert!(TEXTS.contains(&template.as_str()), "{file}: {template:?} is not in TEXTS");
            found.insert(template);
        }
    }
    for template in TEXTS {
        assert!(found.contains(*template), "{template:?} is in TEXTS but no t! makes it");
    }
    let unique: HashSet<&str> = TEXTS.iter().copied().collect();
    assert_eq!(unique.len(), TEXTS.len(), "a template is listed twice");
    // Templates of the other lists are keys of their own.
    for template in inbox::TEMPLATES.iter().chain(market::TEMPLATES).chain(aimarket::TEMPLATES) {
        assert!(!unique.contains(template), "{template} is listed twice");
    }
}

#[test]
fn rendering_fills_placeholders() {
    let (name, n) = ("Anna", 3);
    let text = t!("Round {n}, the {race}, is left out.", n, race = "Bahrain Grand Prix");
    assert_eq!(text.text, "Round 3, the Bahrain Grand Prix, is left out.");
    assert_eq!(text.template, "Round {n}, the {race}, is left out.");
    assert_eq!(text.args, vec![("n".to_owned(), "3".to_owned()), ("race".to_owned(), "Bahrain Grand Prix".to_owned())]);
    assert_eq!(text.to_string(), text.text);
    assert_eq!(text, "Round 3, the Bahrain Grand Prix, is left out.");
    // A placeholder named twice is filled twice.
    let twice = t!("{name} and {name}", name);
    assert_eq!(twice.text, "Anna and Anna");
    // Lists are filled from their texts, joined with ", ".
    let list = t!("Objectives: {counts}.").list(
        "counts",
        vec![
            t!("{name} {met} of {count}", name, met = 2, count = 3),
            t!("{name} {met} of {count}", name = "Ben", met = 1, count = 3),
        ],
    );
    assert_eq!(list.text, "Objectives: Anna 2 of 3, Ben 1 of 3.");
    assert_eq!(list.args, vec![("counts".to_owned(), "Anna 2 of 3, Ben 1 of 3".to_owned())]);
    // The English comes back from the template when the text was left out.
    let bare = Text {
        text: String::new(),
        ..t!("{name} left the career.", name)
    };
    assert_eq!(bare.english(), "Anna left the career.");
    // A news item of one sentence keeps its template; of several, each sentence's.
    let one = News::of(2016, 1, vec![t!("{name} left the career.", name)]);
    assert_eq!(
        (one.text.as_str(), one.template.as_deref(), one.parts.len()),
        ("Anna left the career.", Some("{name} left the career."), 0)
    );
    let two = News::of(2016, 1, vec![t!("The {year} season begins.", year = 2016), t!("{name} left the career.", name)]);
    assert_eq!(two.text, "The 2016 season begins. Anna left the career.");
    assert_eq!((two.template, two.parts.len()), (None, 2));
    let listed = News::of(2016, 1, vec![list]);
    assert_eq!((listed.template, listed.parts.len()), (None, 1));
    // A text without a template is shown as it is.
    let plain = News::of(2016, 1, vec![Text::plain("Plain")]);
    assert_eq!((plain.text.as_str(), plain.template), ("Plain", None));
}

#[test]
fn old_news_without_template_still_show() {
    // A news item from before ids, kinds and templates existed.
    let old: News = serde_json::from_str(r#"{"year":2016,"round":3,"text":"Round 3, Bahrain Grand Prix: Anna wins."}"#)
        .expect("old news");
    assert_eq!(old.text, "Round 3, Bahrain Grand Prix: Anna wins.");
    assert_eq!((old.template, old.args.len(), old.parts.len()), (None, 0, 0));
    // A news item written by this version reads back the same, parts and all.
    let news = News::of(2016, 1, vec![t!("The {year} season begins.", year = 2016), t!("(simulated)")]);
    let json = serde_json::to_string(&news).expect("json");
    assert_eq!(serde_json::from_str::<News>(&json).expect("news"), news);
    // The warnings an older host server sent come without templates.
    let state: crate::multiplayer::rooms::LeagueState =
        serde_json::from_str(r#"{"league":null,"version":3,"warnings":["matched X to Anna by elimination"]}"#)
            .expect("state");
    let texts = state.warning_texts();
    assert_eq!(texts.len(), 1);
    assert_eq!((texts[0].text.as_str(), texts[0].template.as_str()), ("matched X to Anna by elimination", ""));
}

fn anna() -> Actor {
    Actor {
        id: "u-anna".into(),
        name: "Anna".into(),
        owner: true,
    }
}

/// Each sentence of `t` (and of its lists) has a listed template whose placeholders it fills.
fn check_text(t: &Text, context: &str) {
    if t.template.is_empty() {
        return;
    }
    assert!(
        TEXTS.contains(&t.template.as_str())
            || inbox::TEMPLATES.contains(&t.template.as_str())
            || market::TEMPLATES.contains(&t.template.as_str())
            || aimarket::TEMPLATES.contains(&t.template.as_str()),
        "{context}: {:?} isn't listed",
        t.template
    );
    for name in placeholders(&t.template) {
        let filled = t.args.iter().any(|(k, _)| k == name) || t.lists.iter().any(|(k, _)| k == name);
        assert!(filled, "{context}: {{{name}}} of {:?} has no value", t.template);
    }
    assert_eq!(t.english(), texts::render(&t.template, &t.args), "{context}");
    for (_, items) in &t.lists {
        for item in items {
            check_text(item, context);
        }
    }
}

#[test]
fn every_news_of_a_played_career_has_its_templates() {
    let setup = Setup {
        first_year: 2016,
        last_year: 2017,
        ..Setup::default()
    };
    let mut l = League::start(setup, &anna(), Some(7), 7).expect("start");
    apply(&mut l, &anna(), Action::Seat { slot: 2, replaces: None }).expect("seat");
    apply(&mut l, &anna(), Action::Begin).expect("begin");
    let mut warnings = Vec::new();
    while l.phase != Phase::Finished {
        match l.phase {
            Phase::Racing => {
                let _ = apply_t(&mut l, &anna(), Action::Develop { part: Part::Aero });
                warnings.extend(apply_t(&mut l, &anna(), Action::Practice { round: None }).unwrap_or_default());
                warnings.extend(apply_t(&mut l, &anna(), Action::Simulate { round: None }).expect("simulate"));
            }
            Phase::SeasonEnd => {
                let slot = l.offers("u-anna")[0].slot;
                apply(&mut l, &anna(), Action::Sign { slot, replaces: None, focus: None }).expect("sign");
            }
            _ => unreachable!("set up"),
        }
    }
    assert!(l.news.len() > 20, "{}", l.news.len());
    for n in &l.news {
        let context = n.text.as_str();
        assert!(!n.text.contains('{'), "{context}");
        match (&n.template, n.parts.as_slice()) {
            (Some(template), []) => check_text(
                &Text {
                    text: n.text.clone(),
                    template: template.clone(),
                    args: n.args.clone(),
                    lists: Vec::new(),
                },
                context,
            ),
            (None, parts) if !parts.is_empty() => {
                assert_eq!(texts::english(parts).join(" "), n.text, "{context}");
                for p in parts {
                    check_text(p, context);
                }
            }
            _ => panic!("news without a template: {context}"),
        }
    }
    for w in &warnings {
        check_text(w, &w.text);
    }
    // A refusal carries its template.
    let e = apply_t(&mut l, &anna(), Action::Simulate { round: None }).expect_err("finished");
    assert!(!e.template.is_empty(), "{e:?}");
    assert_eq!(e.message, texts::render(&e.template, &e.args));
}

#[test]
fn translated_values_are_words() {
    let words = texts::words();
    for part in Part::ALL {
        assert!(words.contains(&part.name()));
    }
    assert!(words.contains(&"Germany"));
    assert!(words.contains(&weekend::LABEL_PARC_FERME));
    assert!(words.contains(&"Q3"));
}
