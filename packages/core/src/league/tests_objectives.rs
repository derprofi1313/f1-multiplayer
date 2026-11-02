//! Tests of the league's objectives (W2-2): set when a round comes up, graded for driven and
//! simulated sessions, paid once, taken back with the round, and the rivalry meter.

use super::objectives::{self, PRACTICE_TARGET};
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

fn seat(l: &mut League, who: &Actor, slot: u8) {
    apply(l, who, Action::Seat { slot, replaces: None }).unwrap();
}

/// Anna alone at Ferrari: Simulate, Skip and Undo need nobody's consent.
fn solo(setup: Setup) -> League {
    let mut l = League::start(setup, &anna(), Some(7), 42).unwrap();
    seat(&mut l, &anna(), 2);
    apply(&mut l, &anna(), Action::Begin).unwrap();
    l
}

/// Anna and Ben, both at Ferrari, in 2016.
fn ferrari_2016() -> League {
    let mut l = League::start(Setup::default(), &anna(), Some(7), 42).unwrap();
    apply(&mut l, &ben(), Action::Enter { number: Some(8) }).unwrap();
    seat(&mut l, &anna(), 2);
    seat(&mut l, &ben(), 2);
    apply(&mut l, &anna(), Action::Begin).unwrap();
    l
}

/// The game's report of the next round's session: the reporter first, the other players next,
/// the AI in grid order.
fn report_of(l: &League, session: u64, session_type: u8, reporter: &str) -> RaceReport {
    let season = l.season();
    let track = season.next_round().and_then(|r| r.track);
    let mut cars = Vec::new();
    for (i, e) in season.entries.iter().enumerate() {
        let human = e.player.is_some();
        let mine = e.player.as_deref() == Some(reporter);
        cars.push(ReportCar {
            name: if mine {
                "Player".into()
            } else {
                e.name.to_uppercase()
            },
            number: e.number,
            team: e.slot,
            ai: !human,
            mine,
            position: 0,
            grid: i as u8 + 1,
            laps: 30,
            status: 3,
            best_lap_ms: if mine { 89_000 } else { 90_000 + i as u32 * 10 },
            time_ms: 3_600_000 + i as u64 * 1000,
            penalty_s: 0,
        });
    }
    cars.sort_by_key(|c| (!c.mine, c.ai, c.grid));
    for (i, c) in cars.iter_mut().enumerate() {
        c.position = i as u8 + 1;
    }
    RaceReport {
        session,
        track,
        session_type,
        cars,
        at: 0,
        laps: Vec::new(),
        ..RaceReport::default()
    }
}

fn count(l: &mut League, who: &Actor, report: RaceReport) {
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
    .unwrap();
}

fn player<'a>(l: &'a League, id: &str) -> &'a Player {
    l.player(id).unwrap()
}

fn find(l: &League, n: u8, player: &str, kind: ObjectiveKind) -> Option<Objective> {
    let round = l.season().round(n).unwrap();
    objectives::of(round, player)
        .into_iter()
        .find(|o| o.kind == kind)
        .cloned()
}

/// The same league with no objectives in round `n`: what `grow` alone pays.
fn without_objectives(l: &League, n: u8) -> League {
    let mut plain = l.clone();
    plain
        .season_mut()
        .rounds
        .iter_mut()
        .find(|r| r.round == n)
        .unwrap()
        .objectives
        .clear();
    plain
}

#[test]
fn objectives_are_set_when_a_round_comes_up() {
    let l = ferrari_2016();
    let season = l.season();
    let goal = team_goal_at(season, 2, 1);
    for id in ["u-anna", "u-ben"] {
        let mine = objectives::of(&season.rounds[0], id);
        let kinds: Vec<ObjectiveKind> = mine.iter().map(|o| o.kind).collect();
        assert_eq!(
            kinds,
            [
                ObjectiveKind::PracticeGrade,
                ObjectiveKind::QualiPosition,
                ObjectiveKind::BeatTeamMateQuali,
                ObjectiveKind::RacePosition,
                ObjectiveKind::BeatTeamMateRace,
                ObjectiveKind::BeatRival,
                ObjectiveKind::CleanRace,
            ],
            "{id}"
        );
        assert!(mine.iter().all(|o| o.met.is_none()));
        let target = |k: ObjectiveKind| mine.iter().find(|o| o.kind == k).unwrap().target;
        assert_eq!(target(ObjectiveKind::PracticeGrade), PRACTICE_TARGET);
        assert_eq!(target(ObjectiveKind::QualiPosition), goal);
        assert_eq!(target(ObjectiveKind::RacePosition), goal);
        let paid: Vec<ObjectiveKind> = mine.iter().filter(|o| o.paid_elsewhere).map(|o| o.kind).collect();
        assert_eq!(
            paid,
            [
                ObjectiveKind::BeatTeamMateQuali,
                ObjectiveKind::RacePosition,
                ObjectiveKind::BeatTeamMateRace
            ]
        );
    }
    // Only the next round has them; the briefing shows them.
    assert!(season.rounds[1].objectives.is_empty());
    assert_eq!(
        l.briefing("u-anna").unwrap().objectives,
        objectives::of(&season.rounds[0], "u-anna")
            .into_iter()
            .cloned()
            .collect::<Vec<_>>()
    );
    // The same season gives the same objectives on every PC.
    assert_eq!(
        ferrari_2016().season().rounds[0].objectives,
        season.rounds[0].objectives
    );
    // Once round 1 is counted, round 2 gets its own.
    let mut l = l;
    let race = report_of(&l, 0x1, 15, "u-anna");
    count(&mut l, &anna(), race);
    assert_eq!(l.season().rounds[1].objectives.len(), 14);
    assert_eq!(l.briefing("u-ben").unwrap().objectives.len(), 7);
    // A player without a rival has no rival challenge.
    let mut l = solo(Setup::default());
    l.players[0].rival = None;
    l.season_mut().rounds[1].objectives.clear();
    objectives::prepare(&mut l, 2);
    assert!(find(&l, 2, "u-anna", ObjectiveKind::BeatRival).is_none());
    assert!(find(&l, 2, "u-anna", ObjectiveKind::CleanRace).is_some());
}

#[test]
fn a_met_quali_target_pays_once() {
    let mut l = ferrari_2016();
    let mut plain = without_objectives(&l, 1);
    // Anna takes pole: her target (P{goal}) is met, worth 40 resource points and half a point of
    // recognition on top of what qualifying pays anyway.
    let q1 = report_of(&l, 0x51, 5, "u-anna");
    count(&mut l, &anna(), q1.clone());
    count(&mut plain, &anna(), q1);
    let quali = find(&l, 1, "u-anna", ObjectiveKind::QualiPosition).unwrap();
    assert_eq!(quali.met, Some(true));
    assert_eq!(
        find(&l, 1, "u-anna", ObjectiveKind::BeatTeamMateQuali)
            .unwrap()
            .met,
        Some(true)
    );
    assert_eq!(
        find(&l, 1, "u-ben", ObjectiveKind::BeatTeamMateQuali)
            .unwrap()
            .met,
        Some(false)
    );
    assert_eq!(player(&l, "u-anna").rp, player(&plain, "u-anna").rp + 40);
    let recognition = player(&l, "u-anna").recognition - player(&plain, "u-anna").recognition;
    assert!((recognition - 0.5).abs() < 1e-4, "{recognition}");
    // The race isn't counted yet: its objectives are open.
    assert_eq!(
        find(&l, 1, "u-anna", ObjectiveKind::RacePosition).unwrap().met,
        None
    );
    // A later part of qualifying only moves the grid: nothing is paid again.
    let rp = player(&l, "u-anna").rp;
    let q3 = report_of(&l, 0x53, 7, "u-ben");
    count(&mut l, &ben(), q3);
    assert_eq!(player(&l, "u-anna").rp, rp);
    assert_eq!(
        find(&l, 1, "u-anna", ObjectiveKind::QualiPosition).unwrap().met,
        Some(true)
    );
    // Grading again pays nothing either.
    let mut lines = Vec::new();
    objectives::grade(&mut l, 1, SessionKind::Qualifying, true, &mut lines);
    assert_eq!(player(&l, "u-anna").rp, rp);
}

#[test]
fn simulated_sessions_are_graded_too() {
    let mut l = solo(Setup::default());
    apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    let round = &l.season().rounds[0];
    assert_eq!(round.state, RoundState::Simulated);
    let mine = objectives::of(round, "u-anna");
    assert!(!mine.is_empty());
    assert!(mine.iter().all(|o| o.met.is_some()), "{mine:?}");
    // Each is graded from the simulated results.
    let entry = l.season().player_entry("u-anna").unwrap().id.clone();
    let race = round.race_result.iter().find(|c| c.entry == entry).unwrap();
    let goal = find(&l, 1, "u-anna", ObjectiveKind::RacePosition).unwrap();
    assert_eq!(
        goal.met,
        Some(race.status == Finish::Finished && race.position <= goal.target)
    );
    let practice = round.practice.iter().find(|p| p.player == "u-anna").unwrap();
    assert_eq!(
        find(&l, 1, "u-anna", ObjectiveKind::PracticeGrade).unwrap().met,
        Some(practice.grade >= PRACTICE_TARGET)
    );
    let met = mine.iter().filter(|o| o.met == Some(true)).count();
    let line = format!("Objectives: Anna {met} of {}.", mine.len());
    assert!(l.news.iter().any(|n| n.text.contains(&line)), "{line}");
    // The next round has its objectives, still open.
    assert!(
        objectives::of(&l.season().rounds[1], "u-anna")
            .iter()
            .all(|o| o.met.is_none())
    );
}

#[test]
fn undo_reopens_objectives_and_takes_the_reward_back() {
    let mut l = solo(Setup::default());
    let (rp, recognition) = (player(&l, "u-anna").rp, player(&l, "u-anna").recognition);
    // A pole: the quali target is met and paid.
    let q = report_of(&l, 0x61, 5, "u-anna");
    count(&mut l, &anna(), q);
    assert_eq!(
        find(&l, 1, "u-anna", ObjectiveKind::QualiPosition).unwrap().met,
        Some(true)
    );
    assert!(player(&l, "u-anna").rp > rp);
    assert_ne!(player(&l, "u-anna").rival_score, 0);
    apply(&mut l, &anna(), Action::Undo).unwrap();
    assert!(l.season().rounds[0].objectives.iter().all(|o| o.met.is_none()));
    assert_eq!(player(&l, "u-anna").rp, rp);
    assert_eq!(player(&l, "u-anna").recognition, recognition);
    assert_eq!(player(&l, "u-anna").rival_score, 0);
    // Counted again, it pays again, once.
    let q = report_of(&l, 0x62, 5, "u-anna");
    count(&mut l, &anna(), q);
    let paid = player(&l, "u-anna").rp;
    assert!(paid > rp);
    // A whole simulated round taken back reopens everything.
    apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    assert!(l.season().rounds[0].objectives.iter().all(|o| o.met.is_some()));
    apply(&mut l, &anna(), Action::Undo).unwrap();
    assert!(l.season().rounds[0].objectives.iter().all(|o| o.met.is_none()));
    assert_eq!(player(&l, "u-anna").rp, rp);
    // A round left out has no objectives to show; its rewards went with it.
    let q = report_of(&l, 0x63, 5, "u-anna");
    count(&mut l, &anna(), q);
    apply(&mut l, &anna(), Action::Skip { round: None }).unwrap();
    assert!(l.season().rounds[0].objectives.is_empty());
    assert_eq!(player(&l, "u-anna").rp, rp);
    assert!(!l.season().rounds[1].objectives.is_empty());
}

/// `report` with the rival's car ahead of the reporter's.
fn rival_ahead(mut report: RaceReport, l: &League, player: &str) -> RaceReport {
    let rival = l.player(player).unwrap().rival.clone().unwrap();
    let number = l.season().entry(&rival).unwrap().number;
    let mine = report.cars.iter().position(|c| c.mine).unwrap();
    let theirs = report.cars.iter().position(|c| c.number == number).unwrap();
    let (a, b) = (report.cars[mine].position, report.cars[theirs].position);
    report.cars[mine].position = b;
    report.cars[theirs].position = a;
    report
}

#[test]
fn the_rival_meter_counts_head_to_heads_and_resets_each_season() {
    let setup = Setup {
        first_year: 2025,
        last_year: 2026,
        ..Setup::default()
    };
    let mut l = solo(setup);
    assert!(player(&l, "u-anna").rival.is_some());
    assert_eq!(player(&l, "u-anna").rival_score, 0);
    // Round 1: ahead of the rival in qualifying and the race.
    let q = report_of(&l, 0x71, 5, "u-anna");
    count(&mut l, &anna(), q);
    assert_eq!(player(&l, "u-anna").rival_score, 1);
    let race = report_of(&l, 0x72, 15, "u-anna");
    count(&mut l, &anna(), race);
    assert_eq!(player(&l, "u-anna").rival_score, 2);
    assert_eq!(
        find(&l, 1, "u-anna", ObjectiveKind::BeatRival).unwrap().met,
        Some(true)
    );
    // Round 2: the rival wins the race, Anna is behind.
    let n = l.season().next_round().unwrap().round;
    // (A sprint weekend: the Grand Prix is the second race, type 16.)
    let race = rival_ahead(report_of(&l, 0x73, 16, "u-anna"), &l, "u-anna");
    count(&mut l, &anna(), race);
    assert_eq!(player(&l, "u-anna").rival_score, 1);
    assert_eq!(
        find(&l, n, "u-anna", ObjectiveKind::BeatRival).unwrap().met,
        Some(false)
    );
    // Taking round 2 back takes its head-to-head with it.
    apply(&mut l, &anna(), Action::Undo).unwrap();
    assert_eq!(player(&l, "u-anna").rival_score, 2);
    // The rest of the season is simulated; the meter keeps counting.
    while l.phase == Phase::Racing {
        apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    }
    let season = l.season();
    let me = season.player_entry("u-anna").unwrap().id.clone();
    let rival = player(&l, "u-anna").rival.clone().unwrap();
    let expected: i32 = season
        .rounds
        .iter()
        .flat_map(|r| [&r.quali_result, &r.race_result])
        .filter_map(|list| {
            let a = list.iter().find(|c| c.entry == me)?;
            let b = list.iter().find(|c| c.entry == rival)?;
            let ahead =
                (a.status != Finish::Finished, a.position) < (b.status != Finish::Finished, b.position);
            Some(if ahead { 1 } else { -1 })
        })
        .sum();
    assert_eq!(player(&l, "u-anna").rival_score, expected);
    // A new season, a new rival, a meter at zero.
    let slot = l.offers("u-anna")[0].slot;
    apply(
        &mut l,
        &anna(),
        Action::Sign {
            slot,
            replaces: None,
            focus: None,
        },
    )
    .unwrap();
    assert_eq!(l.year(), 2026);
    assert_eq!(l.phase, Phase::Racing);
    assert_eq!(player(&l, "u-anna").rival_score, 0);
    assert!(
        l.season()
            .next_round()
            .unwrap()
            .objectives
            .iter()
            .all(|o| o.met.is_none())
    );
}

#[test]
fn objectives_already_paid_by_grow_are_not_paid_twice() {
    let mut l = ferrari_2016();
    let mut plain = without_objectives(&l, 1);
    // Anna wins the race, with no practice or qualifying counted.
    let race = report_of(&l, 0x81, 15, "u-anna");
    count(&mut l, &anna(), race.clone());
    count(&mut plain, &anna(), race);
    let mine: Vec<Objective> = l.season().rounds[0]
        .objectives
        .iter()
        .filter(|o| o.player == "u-anna")
        .cloned()
        .collect();
    let met = |k: ObjectiveKind| mine.iter().find(|o| o.kind == k).unwrap().met;
    assert_eq!(met(ObjectiveKind::RacePosition), Some(true));
    assert_eq!(met(ObjectiveKind::BeatTeamMateRace), Some(true));
    assert_eq!(met(ObjectiveKind::BeatRival), Some(true));
    assert_eq!(met(ObjectiveKind::CleanRace), Some(true));
    // The race ended the weekend: practice and qualifying weren't counted, so they're missed.
    assert_eq!(met(ObjectiveKind::PracticeGrade), Some(false));
    assert_eq!(met(ObjectiveKind::QualiPosition), Some(false));
    assert_eq!(met(ObjectiveKind::BeatTeamMateQuali), Some(false));
    // Only the rival challenge and the clean race pay on top of what the race pays.
    let extra: u32 = mine
        .iter()
        .filter(|o| o.met == Some(true) && !o.paid_elsewhere)
        .map(|o| o.reward_rp)
        .sum();
    assert_eq!(extra, 60 + 30);
    assert_eq!(player(&l, "u-anna").rp, player(&plain, "u-anna").rp + extra);
    let recognition = player(&l, "u-anna").recognition - player(&plain, "u-anna").recognition;
    assert!((recognition - 1.0).abs() < 1e-4, "{recognition}");
    assert!(
        l.news
            .iter()
            .any(|n| n.text.contains("Objectives: Anna 4 of 7, Ben"))
    );
}

#[test]
fn objectives_round_trip_and_old_rounds_have_none() {
    let mut l = ferrari_2016();
    let race = report_of(&l, 0x91, 15, "u-anna");
    count(&mut l, &anna(), race);
    let json = serde_json::to_string(&l).unwrap();
    let back: League = serde_json::from_str(&json).unwrap();
    assert_eq!(
        back.season().rounds[0].objectives,
        l.season().rounds[0].objectives
    );
    // A round of an old document has no objectives field and loads with none.
    let mut value = serde_json::to_value(&l).unwrap();
    let rounds = value["seasons"][0]["rounds"].as_array_mut().unwrap();
    for r in rounds {
        r.as_object_mut().unwrap().remove("objectives");
    }
    let old: League = serde_json::from_value(value).unwrap();
    assert!(old.season().rounds.iter().all(|r| r.objectives.is_empty()));
}
