//! Tests of the league's consent (W2-1).

use super::*;

fn actor(id: &str, name: &str, owner: bool) -> Actor {
    Actor {
        id: id.into(),
        name: name.into(),
        owner,
    }
}

fn anna() -> Actor {
    actor("u-anna", "Anna", true)
}

fn ben() -> Actor {
    actor("u-ben", "Ben", false)
}

fn cleo() -> Actor {
    actor("u-cleo", "Cleo", false)
}

/// A 2016 career of the owner Anna and `others`, Anna and the first other at Ferrari, the next at
/// Mercedes, racing.
fn career(others: &[Actor]) -> League {
    let mut l = League::start(Setup::default(), &anna(), Some(7), 42).unwrap();
    for (i, who) in others.iter().enumerate() {
        apply(
            &mut l,
            who,
            Action::Enter {
                number: Some(8 + i as u8),
            },
        )
        .unwrap();
    }
    apply(
        &mut l,
        &anna(),
        Action::Seat {
            slot: 2,
            replaces: None,
        },
    )
    .unwrap();
    for (i, who) in others.iter().enumerate() {
        let slot = if i == 0 { 2 } else { 1 };
        apply(&mut l, who, Action::Seat { slot, replaces: None }).unwrap();
    }
    apply(&mut l, &anna(), Action::Begin).unwrap();
    l
}

/// The game's report of the next round's race (`session_type` 15) or qualifying (5) from
/// `reporter`'s PC: the grid in grid order, the reporter winning.
fn report(l: &League, session: u64, session_type: u8, reporter: &str) -> Action {
    let season = l.season();
    let mut cars: Vec<ReportCar> = season
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| ReportCar {
            name: e.name.to_uppercase(),
            number: e.number,
            team: e.slot,
            ai: e.player.is_none(),
            mine: e.player.as_deref() == Some(reporter),
            position: 0,
            grid: i as u8 + 1,
            laps: 30,
            status: 3,
            best_lap_ms: 90_000 + i as u32 * 10,
            time_ms: 3_600_000 + i as u64 * 1000,
            penalty_s: 0,
        })
        .collect();
    cars.sort_by_key(|c| (!c.mine, c.grid));
    for (i, c) in cars.iter_mut().enumerate() {
        c.position = i as u8 + 1;
    }
    Action::Report {
        round: None,
        kind: None,
        force: true,
        report: RaceReport {
            session,
            track: season.next_round().and_then(|r| r.track),
            session_type,
            cars,
            ..RaceReport::default()
        },
    }
}

fn state(l: &League, round: u8) -> RoundState {
    l.season().round(round).unwrap().state
}

fn next(l: &League) -> u8 {
    l.season().next_round().unwrap().round
}

fn news_has(l: &League, text: &str) -> bool {
    l.news.iter().any(|n| n.text.contains(text))
}

#[test]
fn a_solo_career_simulates_at_once() {
    let mut l = career(&[]);
    assert!(!l.needs_consent());
    let warnings = apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(state(&l, 1), RoundState::Simulated);
    apply(&mut l, &anna(), Action::Skip { round: None }).unwrap();
    assert_eq!(state(&l, 2), RoundState::Skipped);
    apply(&mut l, &anna(), Action::Undo).unwrap();
    assert_eq!(state(&l, 2), RoundState::Pending);
    assert!(l.vote.is_none());
}

#[test]
fn simulate_in_a_shared_career_waits_for_the_other_player() {
    let mut l = career(&[ben()]);
    assert!(l.needs_consent());
    let warnings = apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    assert_eq!(warnings, vec!["Waiting for Ben to agree.".to_owned()]);
    assert_eq!(state(&l, 1), RoundState::Pending);
    let vote = l.vote.clone().unwrap();
    assert_eq!(
        (vote.proposal, vote.round, vote.by.as_str(), vote.by_name.as_str()),
        (Proposal::Simulate, 1, "u-anna", "Anna")
    );
    assert_eq!(vote.yes, vec!["u-anna".to_owned()]);
    let waiting: Vec<&str> = l.waiting_for().iter().map(|p| p.name.as_str()).collect();
    assert_eq!(waiting, ["Ben"]);
    assert!(news_has(
        &l,
        "Anna asks to simulate round 1, the Australian Grand Prix"
    ));

    // Ben can't ask for something else while Anna's proposal is open; what couldn't run anyway is
    // refused for that.
    let err = apply(&mut l, &ben(), Action::Skip { round: None }).unwrap_err();
    assert_eq!(err.reason, "vote_open", "{err}");
    let err = apply(&mut l, &ben(), Action::Undo).unwrap_err();
    assert_eq!(err.reason, "nothing", "{err}");
    // Anna asking again changes nothing; asking for a round that isn't next is refused at once.
    let before = l.clone();
    apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    assert_eq!(l, before);
    let err = apply(&mut l, &anna(), Action::Simulate { round: Some(3) }).unwrap_err();
    assert_eq!(err.reason, "not_next");
    // The proposer changes their own proposal.
    apply(&mut l, &anna(), Action::Skip { round: None }).unwrap();
    assert_eq!(l.vote.as_ref().unwrap().proposal, Proposal::Skip);
    assert_eq!(state(&l, 1), RoundState::Pending);
}

#[test]
fn agreeing_runs_it_once() {
    let mut l = career(&[ben()]);
    apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    let warnings = apply(
        &mut l,
        &ben(),
        Action::Vote {
            yes: true,
            force: false,
        },
    )
    .unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(state(&l, 1), RoundState::Simulated);
    assert_eq!(next(&l), 2);
    assert!(l.vote.is_none());
    let err = apply(
        &mut l,
        &ben(),
        Action::Vote {
            yes: true,
            force: false,
        },
    )
    .unwrap_err();
    assert_eq!(err.reason, "no_vote");

    // An older launcher agrees by asking for the same thing: it runs once, not twice.
    apply(&mut l, &ben(), Action::Simulate { round: None }).unwrap();
    assert_eq!(state(&l, 2), RoundState::Pending);
    apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    assert_eq!(state(&l, 2), RoundState::Simulated);
    assert_eq!(next(&l), 3, "round 3 isn't simulated with it");
    assert!(l.vote.is_none());

    // Taking back by consent takes back what the proposer saw.
    apply(&mut l, &ben(), Action::Undo).unwrap();
    let vote = l.vote.clone().unwrap();
    assert_eq!((vote.proposal, vote.round), (Proposal::Undo, 2));
    assert_eq!(state(&l, 2), RoundState::Simulated);
    apply(
        &mut l,
        &anna(),
        Action::Vote {
            yes: true,
            force: false,
        },
    )
    .unwrap();
    assert_eq!(state(&l, 2), RoundState::Pending);
    assert_eq!(state(&l, 1), RoundState::Simulated);
    assert!(l.vote.is_none());
}

#[test]
fn declining_or_cancelling_clears_the_vote() {
    let mut l = career(&[ben()]);
    apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    apply(
        &mut l,
        &ben(),
        Action::Vote {
            yes: false,
            force: false,
        },
    )
    .unwrap();
    assert!(l.vote.is_none());
    assert_eq!(state(&l, 1), RoundState::Pending);
    assert!(news_has(&l, "Ben doesn't agree to simulate round 1"));

    apply(&mut l, &ben(), Action::Skip { round: None }).unwrap();
    apply(
        &mut l,
        &ben(),
        Action::Vote {
            yes: false,
            force: false,
        },
    )
    .unwrap();
    assert!(l.vote.is_none());
    assert_eq!(state(&l, 1), RoundState::Pending);
    assert!(news_has(&l, "Ben withdrew the proposal to leave out round 1"));
    let err = apply(
        &mut l,
        &anna(),
        Action::Vote {
            yes: false,
            force: false,
        },
    )
    .unwrap_err();
    assert_eq!(err.reason, "no_vote");
}

#[test]
fn the_owner_can_force() {
    let mut l = career(&[ben()]);
    apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    let err = apply(
        &mut l,
        &ben(),
        Action::Vote {
            yes: true,
            force: true,
        },
    )
    .unwrap_err();
    assert_eq!(err.reason, "owner");
    assert_eq!(state(&l, 1), RoundState::Pending);
    apply(
        &mut l,
        &anna(),
        Action::Vote {
            yes: true,
            force: false,
        },
    )
    .unwrap();
    assert_eq!(
        state(&l, 1),
        RoundState::Pending,
        "Anna agreed already; Ben hasn't"
    );

    // Ben's proposal, which the owner decides alone.
    apply(
        &mut l,
        &anna(),
        Action::Vote {
            yes: false,
            force: false,
        },
    )
    .unwrap();
    apply(&mut l, &ben(), Action::Skip { round: None }).unwrap();
    apply(
        &mut l,
        &anna(),
        Action::Vote {
            yes: true,
            force: true,
        },
    )
    .unwrap();
    assert_eq!(state(&l, 1), RoundState::Skipped);
    assert!(l.vote.is_none());

    apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    apply(
        &mut l,
        &anna(),
        Action::Vote {
            yes: true,
            force: true,
        },
    )
    .unwrap();
    assert_eq!(state(&l, 2), RoundState::Simulated);
    assert!(news_has(
        &l,
        "Anna decided to simulate round 2, the Bahrain Grand Prix without waiting for Ben."
    ));
}

#[test]
fn a_counted_race_clears_a_stale_vote() {
    let mut l = career(&[ben()]);
    apply(&mut l, &ben(), Action::Simulate { round: None }).unwrap();
    let race = report(&l, 0x1001, 15, "u-anna");
    apply(&mut l, &anna(), race.clone()).unwrap();
    assert_eq!(state(&l, 1), RoundState::Raced);
    assert!(
        l.vote.is_none(),
        "round 1 was driven: simulating round 2 nobody asked for"
    );
    assert!(news_has(&l, "The proposal to simulate round 1"));

    // An undo vote names what it takes back: the same race reported again changes nothing ...
    apply(&mut l, &ben(), Action::Undo).unwrap();
    let before = l.clone();
    let warnings = apply(&mut l, &anna(), race).unwrap();
    assert!(
        warnings.iter().any(|w| w.contains("counted already")),
        "{warnings:?}"
    );
    assert_eq!(l, before);
    // ... but qualifying of round 2 would be taken back instead of round 1.
    let quali = report(&l, 0x1002, 5, "u-anna");
    apply(&mut l, &anna(), quali).unwrap();
    assert!(!l.season().round(2).unwrap().quali_result.is_empty());
    assert!(l.vote.is_none());
    let err = apply(
        &mut l,
        &anna(),
        Action::Vote {
            yes: true,
            force: false,
        },
    )
    .unwrap_err();
    assert_eq!(err.reason, "no_vote");
    assert_eq!(state(&l, 1), RoundState::Raced);
}

#[test]
fn left_players_are_not_asked() {
    // Ben left: Anna is alone and goes ahead.
    let mut l = career(&[ben()]);
    apply(
        &mut l,
        &ben(),
        Action::RemovePlayer {
            player: "u-ben".into(),
        },
    )
    .unwrap();
    assert!(!l.needs_consent());
    apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    assert_eq!(state(&l, 1), RoundState::Simulated);

    // Cleo leaves while Anna's proposal waits for her and Ben: Ben agreeing runs it.
    let mut l = career(&[ben(), cleo()]);
    apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    assert_eq!(l.waiting_for().len(), 2);
    apply(
        &mut l,
        &anna(),
        Action::RemovePlayer {
            player: "u-cleo".into(),
        },
    )
    .unwrap();
    let waiting: Vec<&str> = l.waiting_for().iter().map(|p| p.name.as_str()).collect();
    assert_eq!(waiting, ["Ben"]);
    let err = apply(
        &mut l,
        &cleo(),
        Action::Vote {
            yes: true,
            force: false,
        },
    )
    .unwrap_err();
    assert_eq!(err.reason, "left");
    apply(
        &mut l,
        &ben(),
        Action::Vote {
            yes: true,
            force: false,
        },
    )
    .unwrap();
    assert_eq!(state(&l, 1), RoundState::Simulated);

    // Ben agreed, then Cleo left: nobody is left to wait for, Anna asking again runs it.
    let mut l = career(&[ben(), cleo()]);
    apply(&mut l, &anna(), Action::Skip { round: None }).unwrap();
    apply(
        &mut l,
        &ben(),
        Action::Vote {
            yes: true,
            force: false,
        },
    )
    .unwrap();
    assert_eq!(state(&l, 1), RoundState::Pending);
    apply(
        &mut l,
        &cleo(),
        Action::RemovePlayer {
            player: "u-cleo".into(),
        },
    )
    .unwrap();
    assert!(l.waiting_for().is_empty());
    apply(&mut l, &anna(), Action::Skip { round: None }).unwrap();
    assert_eq!(state(&l, 1), RoundState::Skipped);
    assert!(l.vote.is_none());
}

#[test]
fn ready_marks_belong_to_the_next_round_and_go_with_skip() {
    let mut setup = League::start(Setup::default(), &anna(), Some(7), 42).unwrap();
    let err = apply(&mut setup, &anna(), Action::Ready { ready: true }).unwrap_err();
    assert_eq!(err.reason, "phase");

    let mut l = career(&[ben()]);
    let ready: Action = serde_json::from_str(r#"{"op":"ready"}"#).unwrap();
    assert_eq!(ready, Action::Ready { ready: true });
    apply(&mut l, &anna(), ready.clone()).unwrap();
    apply(&mut l, &anna(), ready).unwrap();
    apply(&mut l, &ben(), Action::Ready { ready: true }).unwrap();
    assert_eq!(l.season().round(1).unwrap().ready, ["u-anna", "u-ben"]);
    assert_eq!(l.briefing("u-ben").unwrap().ready, ["Anna", "Ben"]);
    apply(&mut l, &anna(), Action::Ready { ready: false }).unwrap();
    assert_eq!(l.season().round(1).unwrap().ready, ["u-ben"]);
    assert!(l.season().round(2).unwrap().ready.is_empty());

    apply(&mut l, &anna(), Action::Ready { ready: true }).unwrap();
    apply(&mut l, &anna(), Action::Skip { round: None }).unwrap();
    assert_eq!(
        l.season().round(1).unwrap().ready.len(),
        2,
        "ready-up doesn't agree to a proposal"
    );
    apply(
        &mut l,
        &ben(),
        Action::Vote {
            yes: true,
            force: false,
        },
    )
    .unwrap();
    assert_eq!(state(&l, 1), RoundState::Skipped);
    assert!(l.season().round(1).unwrap().ready.is_empty());
    assert!(l.briefing("u-anna").unwrap().ready.is_empty());
    apply(&mut l, &ben(), Action::Ready { ready: true }).unwrap();
    assert_eq!(l.season().round(2).unwrap().ready, ["u-ben"]);
}

#[test]
fn an_old_launchers_simulate_becomes_a_proposal() {
    let mut l = career(&[ben()]);
    let old: Action = serde_json::from_str(r#"{"op":"simulate"}"#).unwrap();
    let seasons = l.seasons.clone();
    let players = l.players.clone();
    let warnings = apply(&mut l, &ben(), old).unwrap();
    assert_eq!(warnings, vec!["Waiting for Anna to agree.".to_owned()]);
    assert_eq!(l.seasons, seasons);
    assert_eq!(l.players, players);
    assert_eq!(l.vote.as_ref().map(|v| v.proposal), Some(Proposal::Simulate));
    // A league with an open vote round-trips; one without has no `vote` key.
    let text = serde_json::to_string(&l).unwrap();
    assert_eq!(serde_json::from_str::<League>(&text).unwrap(), l);
    l.vote = None;
    assert!(!serde_json::to_string(&l).unwrap().contains("\"vote\""));
}

#[test]
fn a_removed_proposers_vote_lapses() {
    // Cleo proposes to simulate, then leaves: her proposal goes with her, and Anna and Ben are
    // free to ask for something else.
    let mut l = career(&[ben(), cleo()]);
    apply(&mut l, &cleo(), Action::Simulate { round: None }).unwrap();
    assert_eq!(l.vote.as_ref().map(|v| v.by.as_str()), Some("u-cleo"));
    apply(
        &mut l,
        &anna(),
        Action::RemovePlayer {
            player: "u-cleo".into(),
        },
    )
    .unwrap();
    assert!(l.vote.is_none());
    assert!(news_has(&l, "lapsed"));
    assert_eq!(state(&l, 1), RoundState::Pending);
    apply(&mut l, &ben(), Action::Skip { round: None }).unwrap();
    assert_eq!(l.vote.as_ref().map(|v| v.by.as_str()), Some("u-ben"));
}
