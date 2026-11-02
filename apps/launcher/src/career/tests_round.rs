//! Tests of the round flow's pure parts: routing a race to its career, the queue, duplicates,
//! the undo preview and the refusal texts.

use std::collections::HashSet;

use swofty_core::league::{
    self, Action, Actor, League, Phase, RaceReport, ReportCar, SessionKind, Setup, UndoPreview,
};

use super::{
    REASONS, Route, enqueue, looks_counted, next_to_count, reason_text, route_report, uncounted, undo_lines,
};

fn anna() -> Actor {
    Actor {
        id: "u-anna".into(),
        name: "Anna".into(),
        owner: true,
    }
}

/// A one-player league in its first round.
fn racing(seed: u64) -> League {
    let mut l = League::start(Setup::default(), &anna(), None, seed).expect("starts");
    let slot = league::catalog_year(l.year()).expect("year").teams[0].slot;
    league::apply(&mut l, &anna(), Action::Seat { slot, replaces: None }).expect("seat");
    league::apply(&mut l, &anna(), Action::Begin).expect("begin");
    assert_eq!(l.phase, Phase::Racing);
    l
}

/// A league whose next round is at another track than `other`'s.
fn racing_elsewhere(other: &League) -> League {
    let mut l = racing(2);
    let want = track(other);
    while track(&l) == want {
        league::apply(&mut l, &anna(), Action::Simulate { round: None }).expect("simulate");
    }
    l
}

fn track(l: &League) -> Option<u8> {
    l.season().next_round().and_then(|r| r.track)
}

fn report(session: u64, track: Option<u8>, at: u64) -> RaceReport {
    RaceReport {
        session,
        track,
        session_type: 15,
        cars: Vec::new(),
        at,
        laps: Vec::new(),
        total_laps: 0,
        weekend: Vec::new(),
        settings: None,
        lap_telemetry: Vec::new(),
    }
}

/// The 2016 grid of `l` as the game classifies it, in entry order.
fn classified(l: &League, mine: usize) -> Vec<ReportCar> {
    l.season()
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| ReportCar {
            name: e.name.clone(),
            number: e.number,
            team: e.slot,
            ai: i != mine,
            mine: i == mine,
            position: i as u8 + 1,
            grid: i as u8 + 1,
            laps: 20,
            status: 3,
            best_lap_ms: 90_000 + i as u32 * 100,
            time_ms: 1_800_000 + i as u64 * 1_000,
            penalty_s: 0,
        })
        .collect()
}

#[test]
fn route_report_picks_the_career_at_that_track() {
    let here = racing(1);
    let there = racing_elsewhere(&here);
    let careers = vec![("a".to_owned(), &here), ("b".to_owned(), &there)];
    let r = report(1, track(&there), 10);
    assert_eq!(route_report(&r, &careers, Some("a")), Route::Count("b".into()));
    assert_eq!(route_report(&r, &careers, None), Route::Count("b".into()));
    // A track no career races at next waits.
    let nowhere = (0..40u8).find(|t| Some(*t) != track(&here) && Some(*t) != track(&there));
    assert_eq!(
        route_report(&report(2, nowhere, 10), &careers, Some("a")),
        Route::Hold
    );
    // A career still picking seats counts nothing.
    let setup = League::start(Setup::default(), &anna(), None, 3).expect("starts");
    assert_eq!(route_report(&r, &[("c".to_owned(), &setup)], None), Route::Hold);
}

#[test]
fn route_report_prefers_the_selected_career() {
    let a = racing(1);
    let b = racing(1);
    let careers = vec![("a".to_owned(), &a), ("b".to_owned(), &b)];
    let r = report(1, track(&a), 10);
    assert_eq!(route_report(&r, &careers, Some("b")), Route::Count("b".into()));
    // Without a track every racing career fits; the selected one still wins.
    assert_eq!(
        route_report(&report(2, None, 10), &careers, Some("a")),
        Route::Count("a".into())
    );
}

#[test]
fn route_report_holds_when_two_match() {
    let a = racing(1);
    let b = racing(1);
    let other = racing_elsewhere(&a);
    let careers = vec![
        ("a".to_owned(), &a),
        ("b".to_owned(), &b),
        ("o".to_owned(), &other),
    ];
    let r = report(1, track(&a), 10);
    // Two careers at that track and neither on show: the player picks one.
    assert_eq!(
        route_report(&r, &careers, Some("o")),
        Route::Ask(vec!["a".into(), "b".into()])
    );
    assert_eq!(
        route_report(&r, &careers, None),
        Route::Ask(vec!["a".into(), "b".into()])
    );
}

#[test]
fn queued_reports_are_counted_oldest_first() {
    let mut queue = Vec::new();
    // What was there before listening started isn't queued.
    let mut seen: HashSet<u64> = [7].into_iter().collect();
    enqueue(
        &mut queue,
        &mut seen,
        vec![report(3, None, 300), report(1, None, 100), report(7, None, 50)],
    );
    assert_eq!(queue.iter().map(|r| r.session).collect::<Vec<_>>(), [1, 3]);
    // A report queued already isn't queued twice; one that came meanwhile takes its place.
    enqueue(
        &mut queue,
        &mut seen,
        vec![report(1, None, 100), report(2, None, 200)],
    );
    assert_eq!(queue.iter().map(|r| r.session).collect::<Vec<_>>(), [1, 2, 3]);
    let mut order = Vec::new();
    while let Some(i) = next_to_count(&queue) {
        order.push(queue.remove(i).session);
    }
    assert_eq!(order, [1, 2, 3]);
    assert_eq!(next_to_count(&queue), None);
}

#[test]
fn reason_texts_exist_for_every_new_reason() {
    for reason in REASONS {
        assert!(reason_text(reason).is_some(), "no text for {reason}");
    }
    // A reason without a text of its own shows the message that came with it.
    assert_eq!(reason_text("wrong_track"), None);
}

#[test]
fn a_race_another_pc_counted_is_neither_offered_again_nor_counted_twice() {
    let mut l = racing(1);
    let mut anna_race = report(11, track(&l), 1_000);
    anna_race.cars = classified(&l, 0);
    league::apply(
        &mut l,
        &anna(),
        Action::Report {
            round: None,
            kind: None,
            force: false,
            report: anna_race.clone(),
        },
    )
    .expect("counts");
    // Ben's PC reports the same race with its own session uid, moments later.
    let mut ben_race = anna_race.clone();
    ben_race.session = 12;
    ben_race.at = 1_030;
    ben_race.cars = classified(&l, 1);
    assert!(uncounted(Some(&l), vec![anna_race, ben_race.clone()]).is_empty());
    // A different race at the same time of the next round looks like it.
    let mut next = ben_race;
    next.session = 13;
    next.cars.swap(0, 1);
    next.cars[0].position = 1;
    next.cars[1].position = 2;
    assert_eq!(uncounted(Some(&l), vec![next.clone()]).len(), 1);
    let (round, kind, by) = looks_counted(&l, &next).expect("looks counted");
    assert_eq!((round, kind, by.as_str()), (1, SessionKind::Race, "Anna"));
    // Two minutes later it doesn't any more.
    next.at = 1_000 + 121;
    assert_eq!(looks_counted(&l, &next), None);
}

#[test]
fn the_undo_preview_names_what_goes_without_this_player() {
    let preview = UndoPreview {
        round: 3,
        race: "Bahrain Grand Prix".into(),
        race_counted: true,
        sprint: false,
        quali: true,
        practice_of: vec!["Anna".into(), "Ben".into()],
        by_others: vec!["Anna".into(), "Ben".into()],
        auto_after: 1,
        season_end: true,
    };
    assert_eq!(
        undo_lines(&preview, "Anna"),
        [
            "Round 3, the Bahrain Grand Prix: the race, qualifying and the practice of Anna and Ben go.",
            "The 1 simulated round(s) after it are counted again.",
            "Sessions Ben counted go too.",
            "The season opens again: contracts signed for next season are cleared.",
        ]
    );
    let nothing = UndoPreview {
        race_counted: false,
        quali: false,
        practice_of: Vec::new(),
        by_others: vec!["Anna".into()],
        auto_after: 0,
        season_end: false,
        ..preview
    };
    assert_eq!(
        undo_lines(&nothing, "Anna"),
        ["Round 3, the Bahrain Grand Prix, goes back to the start."]
    );
}
