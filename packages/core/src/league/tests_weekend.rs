//! Tests of the league's weekend (W2-8): the rules sheet, the check of the game's settings and
//! session formats against it, and the sprint shootout.

use super::weekend::{self, LABEL_AI, LABEL_DAMAGE, LABEL_PARC_FERME, LABEL_QUALI};
use super::*;

fn anna() -> Actor {
    Actor {
        id: "u-anna".into(),
        name: "Anna".into(),
        owner: true,
    }
}

fn ben() -> Actor {
    Actor {
        id: "u-ben".into(),
        name: "Ben".into(),
        owner: false,
    }
}

/// A league from `first_year` with Anna and Ben both at Ferrari.
fn ferrari(first_year: u16) -> League {
    let setup = Setup {
        first_year,
        ..Setup::default()
    };
    let mut l = League::start(setup, &anna(), Some(7), 42).unwrap();
    apply(&mut l, &ben(), Action::Enter { number: Some(8) }).unwrap();
    for who in [anna(), ben()] {
        apply(
            &mut l,
            &who,
            Action::Seat {
                slot: 2,
                replaces: None,
            },
        )
        .unwrap();
    }
    apply(&mut l, &anna(), Action::Begin).unwrap();
    l
}

/// The game's classification of a session of the next round: the season's grid, the reporter
/// first, the other player second, the AI cars in grid order.
fn report_of(l: &League, session: u64, session_type: u8, reporter: &str) -> RaceReport {
    let season = l.season();
    let track = season.next_round().and_then(|r| r.track);
    let mut cars: Vec<ReportCar> = season
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let mine = e.player.as_deref() == Some(reporter);
            ReportCar {
                name: if mine { "Player".into() } else { e.name.to_uppercase() },
                number: e.number,
                team: e.slot,
                ai: e.player.is_none(),
                mine,
                position: 0,
                grid: i as u8 + 1,
                laps: 10,
                status: 3,
                best_lap_ms: 90_000 + i as u32 * 10,
                time_ms: 900_000 + i as u64 * 1000,
                penalty_s: 0,
            }
        })
        .collect();
    cars.sort_by_key(|c| (!c.mine, c.ai, c.grid));
    for (i, c) in cars.iter_mut().enumerate() {
        c.position = i as u8 + 1;
    }
    RaceReport {
        session,
        track,
        session_type,
        cars,
        ..RaceReport::default()
    }
}

fn report(l: &mut League, who: &Actor, report: RaceReport) -> Result<Vec<String>, LeagueError> {
    apply(
        l,
        who,
        Action::Report {
            round: None,
            kind: None,
            force: false,
            report,
        },
    )
}

fn set_rules(l: &mut League, rules: WeekendRules) -> Result<Vec<String>, LeagueError> {
    let options = Options {
        rules,
        ..l.options.clone()
    };
    apply(l, &anna(), Action::SetOptions { options })
}

fn experience(l: &League, id: &str) -> f32 {
    let p = l.player(id).unwrap();
    p.attributes
        .unwrap_or_else(|| Attributes::from_rating(p.rating))
        .experience
}

/// Every rule agreed.
fn agreed() -> WeekendRules {
    WeekendRules {
        ai_difficulty: Some(90),
        quali: Some(QualiFormat::Short),
        practice: Some(PracticeFormat::OneSession),
        damage: Some(3),
        safety_car: Some(2),
        parc_ferme: Some(true),
        equal_performance: Some(false),
        weather: Some(0),
        rule_set: Some(1),
    }
}

#[test]
fn rules_are_validated() {
    let mut l = ferrari(2016);
    let bad = [
        WeekendRules {
            ai_difficulty: Some(111),
            ..WeekendRules::default()
        },
        WeekendRules {
            damage: Some(4),
            ..WeekendRules::default()
        },
        WeekendRules {
            safety_car: Some(4),
            ..WeekendRules::default()
        },
        WeekendRules {
            weather: Some(6),
            ..WeekendRules::default()
        },
        WeekendRules {
            rule_set: Some(3),
            ..WeekendRules::default()
        },
    ];
    for rules in bad {
        let err = set_rules(&mut l, rules.clone()).unwrap_err();
        assert_eq!(err.reason, "bad_rules", "{rules:?}");
    }
    assert_eq!(l.options.rules, WeekendRules::default(), "nothing was stored");
    // Only the owner sets them.
    let err = apply(
        &mut l,
        &ben(),
        Action::SetOptions {
            options: Options {
                rules: agreed(),
                ..Options::default()
            },
        },
    )
    .unwrap_err();
    assert_eq!(err.reason, "owner");
    // The edges of the game's ranges, and every rule agreed.
    let edges = WeekendRules {
        ai_difficulty: Some(110),
        damage: Some(3),
        safety_car: Some(3),
        weather: Some(5),
        rule_set: Some(11),
        ..WeekendRules::default()
    };
    set_rules(&mut l, edges.clone()).unwrap();
    assert_eq!(l.options.rules, edges);
    set_rules(&mut l, agreed()).unwrap();
    assert_eq!(l.options.rules, agreed());
    // An unknown format can't be sent at all, and the rules round-trip as JSON.
    let json = serde_json::to_string(&agreed()).unwrap();
    assert!(json.contains("\"quali\":\"short\"") && json.contains("\"practice\":\"one_session\""));
    assert_eq!(serde_json::from_str::<WeekendRules>(&json).unwrap(), agreed());
    assert!(serde_json::from_str::<WeekendRules>(r#"{"quali":"two_shot"}"#).is_err());
    assert_eq!(
        serde_json::from_str::<WeekendRules>("{}").unwrap(),
        WeekendRules::default()
    );
}

#[test]
fn a_mismatch_warns_but_counts() {
    let mut l = ferrari(2016);
    set_rules(
        &mut l,
        WeekendRules {
            ai_difficulty: Some(90),
            damage: Some(3),
            parc_ferme: Some(true),
            safety_car: Some(2),
            equal_performance: Some(false),
            ..WeekendRules::default()
        },
    )
    .unwrap();
    let mut race = report_of(&l, 0x51, 15, "u-anna");
    race.settings = Some(GameSettings {
        ai_difficulty: 85,
        car_damage: 2,
        parc_ferme: 1,
        safety_car: 2,
        equal_performance: 1,
        ..GameSettings::default()
    });
    let warnings = report(&mut l, &anna(), race).unwrap();
    assert!(
        warnings.contains(&"AI difficulty: the game ran 85, the agreed rules say 90".to_owned()),
        "{warnings:?}"
    );
    assert!(
        warnings.contains(&"Car damage: the game ran Standard, the agreed rules say Simulation".to_owned()),
        "{warnings:?}"
    );
    assert!(
        warnings.contains(&"Equal car performance: the game ran On, the agreed rules say Off".to_owned()),
        "{warnings:?}"
    );
    assert!(
        !warnings
            .iter()
            .any(|w| w.starts_with(LABEL_PARC_FERME) || w.starts_with("Safety car")),
        "these match: {warnings:?}"
    );
    // The race counts all the same.
    let r = l.season().round(1).unwrap();
    assert_eq!(r.state, RoundState::Raced);
    assert_eq!(r.race_result[0].points, 25.0);
}

#[test]
fn zero_settings_are_not_checked() {
    let rules = agreed();
    let mut r = RaceReport {
        session_type: 15,
        settings: Some(GameSettings::default()),
        ..RaceReport::default()
    };
    assert!(weekend::check(&rules, &r).is_empty(), "every byte 0: not sent");
    r.settings = None;
    assert!(weekend::check(&rules, &r).is_empty(), "no settings at all");
    // Rules not agreed aren't checked, whatever the game ran.
    r.settings = Some(GameSettings {
        ai_difficulty: 37,
        car_damage: 1,
        safety_car: 1,
        parc_ferme: 1,
        equal_performance: 1,
        weather: 4,
        rule_set: 1,
        ..GameSettings::default()
    });
    r.session_type = 9;
    assert!(weekend::check(&WeekendRules::default(), &r).is_empty());
    // An agreed "off" warns only when the game sends "on".
    let off = WeekendRules {
        parc_ferme: Some(false),
        ..WeekendRules::default()
    };
    assert_eq!(
        weekend::check(&off, &r),
        vec!["Parc fermé: the game ran On, the agreed rules say Off".to_owned()]
    );
    // Reports from older launchers carry no settings and still count without a warning.
    let mut l = ferrari(2016);
    set_rules(&mut l, agreed()).unwrap();
    let race = report_of(&l, 0x52, 15, "u-anna");
    assert!(report(&mut l, &anna(), race).unwrap().is_empty());
}

#[test]
fn the_quali_format_is_checked() {
    let short = WeekendRules {
        quali: Some(QualiFormat::Short),
        ..WeekendRules::default()
    };
    let session = |session_type: u8| RaceReport {
        session_type,
        ..RaceReport::default()
    };
    for t in [5, 6, 7] {
        assert_eq!(
            weekend::check(&short, &session(t)),
            vec!["Qualifying: the game ran Full (Q1, Q2, Q3), the agreed rules say Short".to_owned()],
            "Q{}",
            t - 4
        );
    }
    assert!(weekend::check(&short, &session(8)).is_empty());
    assert_eq!(weekend::check(&short, &session(9)).len(), 1, "one-shot");
    assert!(weekend::check(&short, &session(13)).is_empty(), "short sprint shootout");
    assert_eq!(weekend::check(&short, &session(14)).len(), 1, "one-shot sprint shootout");
    assert!(weekend::check(&short, &session(15)).is_empty(), "a race has no format");
    let one_shot = WeekendRules {
        quali: Some(QualiFormat::OneShot),
        ..WeekendRules::default()
    };
    assert!(weekend::check(&one_shot, &session(9)).is_empty());
    // The practice format, from the session type and the weekend structure.
    let one = WeekendRules {
        practice: Some(PracticeFormat::OneSession),
        ..WeekendRules::default()
    };
    assert!(weekend::check(&one, &session(1)).is_empty(), "P1 alone may be the one session");
    assert_eq!(weekend::check(&one, &session(2)).len(), 1, "P2 means three sessions");
    assert_eq!(
        weekend::check(&one, &session(4)),
        vec!["Practice: the game ran Short, the agreed rules say One session".to_owned()]
    );
    let mut p1 = session(1);
    p1.weekend = vec![1, 2, 3, 5, 6, 7, 15];
    assert_eq!(weekend::check(&one, &p1).len(), 1, "the weekend has three practices");
    p1.weekend = vec![1, 5, 6, 7, 15];
    assert!(weekend::check(&one, &p1).is_empty());
    let none = WeekendRules {
        practice: Some(PracticeFormat::None),
        ..WeekendRules::default()
    };
    assert_eq!(weekend::check(&none, &p1).len(), 1, "no practice was agreed");
    assert!(weekend::check(&none, &session(15)).is_empty());

    // Through a report: a Q1 when short qualifying was agreed warns and counts.
    let mut l = ferrari(2016);
    set_rules(&mut l, short).unwrap();
    let q1 = report_of(&l, 0x61, 5, "u-anna");
    let warnings = report(&mut l, &anna(), q1).unwrap();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].starts_with(LABEL_QUALI));
    assert!(!l.season().round(1).unwrap().quali_result.is_empty());
}

/// A league from 2023 at its first sprint weekend (driven in the game).
fn at_sprint_weekend() -> (League, u8) {
    let mut l = ferrari(2023);
    let n = l
        .season()
        .rounds
        .iter()
        .find(|r| r.sprint && r.track.is_some())
        .expect("a sprint weekend")
        .round;
    while l.season().next_round().unwrap().round < n {
        super::tests::agreed(&mut l, &anna(), Action::Simulate { round: None });
    }
    (l, n)
}

/// A sprint shootout where an AI car takes the sprint pole, Ben is second and Anna third.
fn shootout_of(l: &League, session: u64, session_type: u8) -> (RaceReport, String) {
    let mut r = report_of(l, session, session_type, "u-anna");
    // Positions 1, 2, 3 are Anna, Ben and the first AI car: make it AI, Ben, Anna.
    for c in &mut r.cars {
        c.position = match c.position {
            1 => 3,
            3 => 1,
            p => p,
        };
    }
    let ai = r.cars.iter().find(|c| c.position == 1).unwrap();
    let pole = l
        .season()
        .entries
        .iter()
        .find(|e| e.player.is_none() && e.number == ai.number)
        .unwrap()
        .id
        .clone();
    (r, pole)
}

#[test]
fn the_shootout_sets_the_sprint_grid() {
    let (mut l, n) = at_sprint_weekend();
    let (anna_exp, ben_exp) = (experience(&l, "u-anna"), experience(&l, "u-ben"));
    let rp = l.player("u-anna").unwrap().rp;
    let (sq3, pole) = shootout_of(&l, 0x5a, 12);
    let reply = report(&mut l, &anna(), sq3.clone()).unwrap();
    assert_eq!(reply[0], "sprint shootout counted", "{reply:?}");
    let r = l.season().round(n).unwrap().clone();
    assert_eq!(r.shootout_result.len(), l.season().entries.len());
    assert_eq!(r.shootout_result[0].entry, pole);
    assert!(r.shootout_result.iter().all(|c| c.points == 0.0 && !c.fastest));
    assert!(r.sprint_result.is_empty() && r.quali_result.is_empty());
    assert_eq!(r.state, RoundState::Pending);
    assert!(r.started(), "a counted shootout starts the round");
    assert!(r.before.is_some(), "an undo restores what it gave");
    assert_eq!(
        r.counted
            .iter()
            .map(|c| (c.kind, c.session_type, c.session))
            .collect::<Vec<_>>(),
        vec![(SessionKind::Sprint, 12, 0x5a)]
    );
    // Experience only: both players drove it, nobody earns resource points.
    assert!((experience(&l, "u-anna") - anna_exp - 0.1).abs() < 1e-4);
    assert!((experience(&l, "u-ben") - ben_exp - 0.1).abs() < 1e-4);
    assert_eq!(l.player("u-anna").unwrap().rp, rp);
    assert!(
        l.news
            .iter()
            .any(|x| x.text.contains("sprint shootout") && x.text.contains("Ben P2, Anna P3")),
        "{:?}",
        l.news.last()
    );
    // The same session again changes nothing; nor does an earlier part after a later one.
    let again = report(&mut l, &anna(), sq3).unwrap();
    assert!(again[0].contains("counted already"), "{again:?}");
    let (sq1, _) = shootout_of(&l, 0x5b, 10);
    let earlier = apply(
        &mut l,
        &ben(),
        Action::Report {
            round: Some(n),
            kind: None,
            force: false,
            report: sq1,
        },
    )
    .unwrap();
    assert_eq!(earlier, vec!["a later part of this sprint shootout is counted already".to_owned()]);
    assert!((experience(&l, "u-anna") - anna_exp - 0.1).abs() < 1e-4, "given once");

    // Simulated, the sprint starts from the shootout's grid, the race from qualifying.
    let mut sim = l.clone();
    super::tests::agreed(&mut sim, &anna(), Action::Simulate { round: None });
    let r = sim.season().round(n).unwrap();
    for c in &r.sprint_result {
        let shot = r.shootout_result.iter().find(|s| s.entry == c.entry).unwrap();
        assert_eq!(c.grid, shot.position, "{}", c.entry);
    }
    let quali_pole = &r.quali_result[0].entry;
    assert_eq!(&r.race_result.iter().find(|c| c.grid == 1).unwrap().entry, quali_pole);

    // Driven, the sprint counts with its points on top.
    let mut driven = l.clone();
    let mut sprint = report_of(&driven, 0x5c, 15, "u-anna");
    sprint.weekend = vec![1, 10, 11, 12, 15, 5, 6, 7, 16];
    report(&mut driven, &anna(), sprint).unwrap();
    let r = driven.season().round(n).unwrap();
    assert!(!r.sprint_result.is_empty() && r.sprint_result[0].points > 0.0);
    // Once the sprint is counted, a shootout is no longer counted.
    let (mut late, _) = shootout_of(&driven, 0x5d, 12);
    late.cars[0].best_lap_ms += 1; // another session's classification
    let err = apply(
        &mut driven,
        &anna(),
        Action::Report {
            round: Some(n),
            kind: None,
            force: false,
            report: late,
        },
    )
    .unwrap_err();
    assert_eq!(err.reason, "not_counted");

    // An undo takes the shootout back with what it gave.
    super::tests::agreed(&mut l, &anna(), Action::Undo);
    let r = l.season().round(n).unwrap();
    assert!(r.shootout_result.is_empty() && r.counted.is_empty() && !r.started());
    assert!((experience(&l, "u-anna") - anna_exp).abs() < 1e-4);

    // A weekend without a sprint doesn't count a shootout.
    let mut plain = ferrari(2016);
    let shootout = report_of(&plain, 0x5e, 12, "u-anna");
    let err = report(&mut plain, &anna(), shootout).unwrap_err();
    assert_eq!(err.reason, "not_counted");
}

#[test]
fn a_skipped_round_drops_its_shootout() {
    let (mut l, n) = at_sprint_weekend();
    let exp = experience(&l, "u-anna");
    let (sq3, _) = shootout_of(&l, 0x6a, 12);
    report(&mut l, &anna(), sq3).unwrap();
    super::tests::agreed(&mut l, &anna(), Action::Skip { round: None });
    let r = l.season().round(n).unwrap();
    assert_eq!(r.state, RoundState::Skipped);
    assert!(r.shootout_result.is_empty());
    assert!((experience(&l, "u-anna") - exp).abs() < 1e-4);
}

#[test]
fn rules_show_in_the_briefing() {
    let mut l = ferrari(2016);
    assert_eq!(l.briefing("u-ben").unwrap().rules, WeekendRules::default());
    assert!(weekend::describe(&WeekendRules::default()).is_empty(), "nothing agreed");
    let rules = WeekendRules {
        ai_difficulty: Some(95),
        quali: Some(QualiFormat::OneShot),
        damage: Some(1),
        parc_ferme: Some(true),
        ..WeekendRules::default()
    };
    set_rules(&mut l, rules.clone()).unwrap();
    assert_eq!(l.briefing("u-ben").unwrap().rules, rules);
    assert_eq!(
        weekend::describe(&rules),
        vec![
            (LABEL_AI, "95".to_owned()),
            (LABEL_QUALI, "One-shot".to_owned()),
            (LABEL_DAMAGE, "Reduced".to_owned()),
            (LABEL_PARC_FERME, "On".to_owned()),
        ]
    );
    // Every rule agreed: one line each, in the sheet's order.
    let all = weekend::describe(&agreed());
    assert_eq!(
        all.iter().map(|(label, _)| *label).collect::<Vec<_>>(),
        vec![
            "AI difficulty",
            "Qualifying",
            "Practice",
            "Car damage",
            "Safety car",
            "Parc fermé",
            "Equal car performance",
            "Weather",
            "Rule set",
        ]
    );
    assert_eq!(all[8].1, "Race");
    assert_eq!(all[7].1, "Clear");
    // The briefing serialises the rules for the launcher.
    let json = serde_json::to_value(l.briefing("u-anna").unwrap()).unwrap();
    assert_eq!(json["rules"]["ai_difficulty"], 95);
    assert_eq!(json["rules"]["quali"], "one_shot");
}

#[test]
fn the_league_pace_in_the_game_agrees_equal_performance_off() {
    // W2-10: with the league's car pace in the game, equal performance is agreed off...
    let mut l = ferrari(2016);
    let options = Options {
        game_performance: true,
        ..l.options.clone()
    };
    assert!(apply(&mut l, &anna(), Action::SetOptions { options }).unwrap().is_empty());
    assert_eq!(l.options.rules.equal_performance, Some(false));
    // ... even when the owner asked for it on.
    let options = Options {
        game_performance: true,
        rules: WeekendRules {
            equal_performance: Some(true),
            ..WeekendRules::default()
        },
        ..l.options.clone()
    };
    let warnings = apply(&mut l, &anna(), Action::SetOptions { options }).unwrap();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert_eq!(l.options.rules.equal_performance, Some(false));
    // A session where the game ran equal performance warns, practice included (W2-8).
    let mut p1 = report_of(&l, 0x71, 1, "u-anna");
    p1.settings = p1_settings();
    let warnings = report(&mut l, &anna(), p1).unwrap();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].starts_with(weekend::LABEL_EQUAL), "{warnings:?}");
    assert_eq!(l.season().round(1).unwrap().practice.len(), 1, "it counts all the same");
    let mut q = report_of(&l, 0x72, 7, "u-anna");
    q.settings = p1_settings();
    let warnings = report(&mut l, &anna(), q).unwrap();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    // Without the option nothing is agreed, and nothing is checked.
    let mut plain = ferrari(2016);
    let mut p1 = report_of(&plain, 0x73, 1, "u-anna");
    p1.settings = p1_settings();
    assert!(report(&mut plain, &anna(), p1).unwrap().is_empty());
}

fn p1_settings() -> Option<GameSettings> {
    Some(GameSettings {
        equal_performance: 1,
        ..GameSettings::default()
    })
}
