//! Tests of the league's driver (W2-5): the creation profile, the focus attribute and the AI
//! drivers' form.

use super::driver::{self, FORM_CAP, FORM_CARRY, FORM_STEP, NATIONALITIES};
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

/// Anna alone at Ferrari, in Setup.
fn solo_setup(setup: Setup) -> League {
    let mut l = League::start(setup, &anna(), Some(7), 42).unwrap();
    seat(&mut l, &anna(), 2);
    l
}

/// Anna alone at Ferrari, the season running (one player: no votes needed).
fn solo(setup: Setup) -> League {
    let mut l = solo_setup(setup);
    apply(&mut l, &anna(), Action::Begin).unwrap();
    l
}

fn profile(
    l: &mut League,
    who: &Actor,
    nationality: Option<&str>,
    archetype: Option<Archetype>,
) -> Result<Vec<String>, LeagueError> {
    apply(
        l,
        who,
        Action::Profile {
            nationality: nationality.map(str::to_owned),
            archetype,
        },
    )
}

fn simulate(l: &mut League) {
    apply(l, &anna(), Action::Simulate { round: None }).unwrap();
}

/// The game's race of the next round: Anna's car with `status` after `laps` of 30, the AI cars
/// finishing in grid order.
fn race(l: &League, session: u64, status: u8, laps: u8, penalty_s: u16) -> RaceReport {
    let season = l.season();
    let track = season.next_round().unwrap().track;
    let mut cars: Vec<ReportCar> = season
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let mine = e.player.as_deref() == Some("u-anna");
            ReportCar {
                name: if mine {
                    "Player".into()
                } else {
                    e.name.to_uppercase()
                },
                number: e.number,
                team: e.slot,
                ai: !mine,
                mine,
                position: 0,
                grid: i as u8 + 1,
                laps: if mine { laps } else { 30 },
                status: if mine { status } else { 3 },
                best_lap_ms: 90_000 + i as u32 * 10,
                time_ms: 3_600_000 + i as u64 * 1000,
                penalty_s: if mine { penalty_s } else { 0 },
            }
        })
        .collect();
    // Anna wins when she finishes, else she is classified last.
    cars.sort_by_key(|c| (c.mine != (status == 3), c.grid));
    for (i, c) in cars.iter_mut().enumerate() {
        c.position = i as u8 + 1;
    }
    RaceReport {
        session,
        track,
        session_type: 15,
        cars,
        total_laps: 30,
        ..RaceReport::default()
    }
}

fn report(l: &mut League, r: RaceReport) {
    apply(
        l,
        &anna(),
        Action::Report {
            round: None,
            kind: None,
            force: false,
            report: r,
        },
    )
    .unwrap();
}

fn focus(l: &League, id: &str) -> f32 {
    driver::focus_of(l.player(id).unwrap())
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

#[test]
fn a_rookie_starts_at_70_and_a_veteran_at_84() {
    assert_eq!(driver::start_rating(Some(Archetype::Rookie)), 70.0);
    assert_eq!(driver::start_rating(Some(Archetype::Experienced)), 78.0);
    assert_eq!(driver::start_rating(Some(Archetype::Veteran)), 84.0);
    assert_eq!(driver::start_rating(None), START_RATING);

    let mut l = solo_setup(Setup::default());
    apply(&mut l, &ben(), Action::Enter { number: Some(8) }).unwrap();
    seat(&mut l, &ben(), 2);
    profile(&mut l, &anna(), Some("deu"), Some(Archetype::Rookie)).unwrap();
    profile(&mut l, &ben(), None, Some(Archetype::Veteran)).unwrap();

    let a = l.player("u-anna").unwrap();
    assert_eq!(a.nationality.as_deref(), Some("DEU"));
    assert_eq!(a.archetype, Some(Archetype::Rookie));
    assert_eq!(a.rating, 70.0);
    assert!(close(a.attributes.unwrap().overall(), 70.0));
    assert_eq!(a.attributes.unwrap().focus, Some(70.0));
    let b = l.player("u-ben").unwrap();
    assert_eq!(b.rating, 84.0);
    assert_eq!(b.attributes.unwrap().focus, Some(84.0));
    assert_eq!(b.nationality, None);
    // The cars carry the new ratings.
    assert_eq!(l.season().player_entry("u-anna").unwrap().rating, 70.0);
    assert_eq!(l.season().player_entry("u-ben").unwrap().rating, 84.0);
    // The veteran starts with the bigger salary.
    apply(&mut l, &anna(), Action::Begin).unwrap();
    assert!(l.player("u-ben").unwrap().salary_k > l.player("u-anna").unwrap().salary_k);

    // Only listed nationalities; an empty one clears it.
    let err = profile(&mut l, &anna(), Some("XYZ"), None).unwrap_err();
    assert_eq!(err.reason, "bad_profile");
    profile(&mut l, &anna(), Some(""), None).unwrap();
    assert_eq!(l.player("u-anna").unwrap().nationality, None);
    assert!(NATIONALITIES.iter().all(|(c, n)| c.len() == 3 && !n.is_empty()));
    assert!(
        NATIONALITIES.windows(2).all(|w| w[0].1 < w[1].1),
        "sorted by name"
    );
}

#[test]
fn profile_is_locked_after_the_first_session() {
    let mut l = solo(Setup::default());
    // Before the first session, the profile can still change, and so does the first salary.
    let salary = l.player("u-anna").unwrap().salary_k;
    profile(&mut l, &anna(), Some("GBR"), Some(Archetype::Veteran)).unwrap();
    assert_eq!(l.player("u-anna").unwrap().rating, 84.0);
    assert!(l.player("u-anna").unwrap().salary_k > salary);
    assert!(
        l.news
            .iter()
            .any(|n| n.text.contains("Anna races as a veteran from United Kingdom"))
    );

    simulate(&mut l);
    let err = profile(&mut l, &anna(), Some("FRA"), Some(Archetype::Rookie)).unwrap_err();
    assert_eq!(err.reason, "phase");
    let a = l.player("u-anna").unwrap();
    assert_eq!(a.nationality.as_deref(), Some("GBR"));
    assert_eq!(a.archetype, Some(Archetype::Veteran));
    assert!(driver::has_driven(&l, "u-anna"));

    // A player who joins later has driven nothing yet.
    apply(&mut l, &ben(), Action::Enter { number: Some(8) }).unwrap();
    profile(&mut l, &ben(), Some("ITA"), Some(Archetype::Rookie)).unwrap();
    assert_eq!(l.player("u-ben").unwrap().rating, 70.0);
}

#[test]
fn focus_grows_with_clean_races() {
    let mut l = solo(Setup::default());
    profile(&mut l, &anna(), None, Some(Archetype::Experienced)).unwrap();
    assert!(close(focus(&l, "u-anna"), 78.0));

    let r = race(&l, 0xa1, 3, 30, 0);
    report(&mut l, r);
    assert!(close(focus(&l, "u-anna"), 78.2), "{}", focus(&l, "u-anna"));
    // Focus isn't part of the overall rating.
    let a = l.player("u-anna").unwrap().attributes.unwrap();
    let mut without = a;
    without.focus = None;
    assert_eq!(a.overall(), without.overall());

    // A penalty: finished, but not clean.
    let r = race(&l, 0xa2, 3, 30, 5);
    report(&mut l, r);
    assert!(close(focus(&l, "u-anna"), 78.2));

    // Out after ten laps: down.
    let r = race(&l, 0xa3, 7, 10, 0);
    report(&mut l, r);
    assert!(close(focus(&l, "u-anna"), 77.9), "{}", focus(&l, "u-anna"));

    // Taking the race back takes the focus back.
    apply(&mut l, &anna(), Action::Undo).unwrap();
    assert!(close(focus(&l, "u-anna"), 78.2));

    // A simulated race leaves it.
    simulate(&mut l);
    assert!(close(focus(&l, "u-anna"), 78.2));
}

#[test]
fn focus_makes_a_simulated_retirement_less_likely() {
    let l = solo(Setup::default());
    // The season as before round 1 was simulated, with Anna's focus then.
    let with_focus = |f: f32| {
        let mut s = l.season().clone();
        let round = s.rounds.iter_mut().find(|r| r.round == 1).unwrap();
        let mut attributes = Attributes::from_rating(78.0);
        attributes.focus = Some(f);
        round.before = Some(vec![Progress {
            player: "u-anna".into(),
            rating: 78.0,
            attributes: Some(attributes),
            recognition: 50.0,
            rp: 0,
            earnings_k: 0,
            number_one: false,
        }]);
        s
    };
    let entry = l.season().player_entry("u-anna").unwrap().clone();
    assert_eq!(super::sim::focus_before(&with_focus(90.0), &entry, 1), 90.0);
    // Over many seeds, the focused driver retires less often.
    let outs = |f: f32| {
        let s = with_focus(f);
        (0..400u64)
            .filter(|&seed| {
                super::sim::simulate(&s, 1, seed, SessionKind::Race, 2016, PointsRule::Historic, None)
                    .iter()
                    .any(|c| c.entry == "p:u-anna" && c.status == Finish::Dnf)
            })
            .count()
    };
    let (calm, nervous) = (outs(99.0), outs(45.0));
    assert!(calm < nervous, "{calm} vs {nervous}");
}

#[test]
fn ai_form_moves_with_results_and_is_capped() {
    // Simulated races move the form, a step per race at most.
    let mut l = solo(Setup::default());
    for _ in 0..3 {
        simulate(&mut l);
    }
    let season = l.season();
    assert!(!season.ai_form.is_empty());
    assert!(season.ai_form.iter().all(|f| f.through == 3));
    assert!(
        season
            .ai_form
            .iter()
            .all(|f| f.delta.abs() <= 3.0 * FORM_STEP + 1e-4)
    );
    assert!(season.ai_form.iter().any(|f| f.delta != 0.0));
    assert!(season.ai_form.iter().all(|f| !f.entry.starts_with("p:")));

    // A back-marker who wins every race gains the most a race gives, up to the cap.
    let mut l = solo(Setup::default());
    let last = {
        let s = l.season();
        s.entries
            .iter()
            .filter(|e| e.player.is_none())
            .max_by_key(|e| progress::expected_place(s, &e.id, 1))
            .unwrap()
            .id
            .clone()
    };
    let rounds: Vec<u8> = l.season().rounds.iter().map(|r| r.round).collect();
    for (i, &n) in rounds.iter().enumerate() {
        let s = l.season().clone();
        let mut result = super::sim::simulate(&s, n, 7, SessionKind::Race, 2016, PointsRule::Historic, None);
        for c in &mut result {
            c.status = Finish::Finished;
        }
        let at = result.iter().position(|c| c.entry == last).unwrap();
        let winner = result.remove(at);
        result.insert(0, winner);
        for (p, c) in result.iter_mut().enumerate() {
            c.position = p as u8 + 1;
        }
        let round = l
            .seasons
            .last_mut()
            .unwrap()
            .rounds
            .iter_mut()
            .find(|r| r.round == n)
            .unwrap();
        round.race_result = result;
        round.state = RoundState::Raced;
        driver::after_round(&mut l, n);
        let form = l.season().ai_form.iter().find(|f| f.entry == last).unwrap().delta;
        let expected = (FORM_STEP * (i + 1) as f32).min(FORM_CAP);
        assert!(close(form, expected), "round {n}: {form} vs {expected}");
    }
    let form = l.season().ai_form.iter().find(|f| f.entry == last).unwrap();
    assert_eq!(form.delta, FORM_CAP);
    let entry = l.season().entry(&last).unwrap().clone();
    assert!(close(
        driver::rating_of(l.season(), &entry, rounds.len() as u8 + 1),
        entry.rating + FORM_CAP
    ));
}

#[test]
fn simulations_use_ai_form() {
    let mut l = solo(Setup::default());
    let s = l.season().clone();
    // A midfield driver.
    let mut ai: Vec<&Entry> = s.entries.iter().filter(|e| e.player.is_none()).collect();
    ai.sort_by_key(|e| progress::expected_place(&s, &e.id, 1));
    let mid = ai[ai.len() / 2].clone();
    let place = progress::expected_place(&s, &mid.id, 1);
    let quali = |s: &Season| {
        super::sim::simulate(s, 1, 9, SessionKind::Qualifying, 2016, PointsRule::Historic, None)
            .iter()
            .find(|c| c.entry == mid.id)
            .unwrap()
            .position
    };
    let before = quali(&s);

    l.seasons.last_mut().unwrap().ai_form = vec![Form {
        entry: mid.id.clone(),
        delta: FORM_CAP,
        ..Form::default()
    }];
    let s = l.season();
    assert_eq!(driver::rating_of(s, &mid, 1), mid.rating + FORM_CAP);
    assert!(progress::expected_place(s, &mid.id, 1) < place);
    assert!(quali(s) < before, "{} vs {before}", quali(s));

    // Players drive with their own rating.
    let me = s.player_entry("u-anna").unwrap();
    assert_eq!(driver::rating_of(s, me, 1), me.rating);
}

#[test]
fn a_round_taken_back_simulates_the_same_again() {
    let mut l = solo(Setup::default());
    simulate(&mut l);
    simulate(&mut l);
    let form = l.season().ai_form.clone();
    simulate(&mut l);
    let third = l.season().rounds[2].race_result.clone();
    assert!(l.season().ai_form.iter().all(|f| f.through == 3));

    apply(&mut l, &anna(), Action::Undo).unwrap();
    // The stored form still counts round 3; round 3 itself uses the form from before it.
    let s = l.season();
    for e in s.entries.iter().filter(|e| e.player.is_none()) {
        let before = form.iter().find(|f| f.entry == e.id).map_or(0.0, |f| f.delta);
        let rating = driver::rating_of(s, e, 3);
        assert!(
            close(
                rating,
                if before == 0.0 {
                    e.rating
                } else {
                    e.rating + before
                }
            ),
            "{}",
            e.id
        );
    }
    simulate(&mut l);
    assert_eq!(l.season().rounds[2].race_result, third);
}

#[test]
fn old_players_get_a_focus_from_their_rating() {
    // A player saved before focus existed.
    let mut l = solo(Setup::default());
    {
        let p = l.players.iter_mut().find(|p| p.id == "u-anna").unwrap();
        p.attributes = None;
        p.rating = 81.0;
    }
    let json = serde_json::to_string(&l).unwrap();
    let mut l: League = serde_json::from_str(&json).unwrap();
    assert!(l.player("u-anna").unwrap().attributes.is_none());
    assert_eq!(focus(&l, "u-anna"), 81.0);

    // It is kept once it first moves.
    let r = race(&l, 0xb1, 3, 30, 0);
    report(&mut l, r);
    let a = l.player("u-anna").unwrap().attributes.unwrap();
    assert!(close(a.focus.unwrap(), 81.2), "{:?}", a.focus);
}

#[test]
fn drivers_who_stay_carry_a_quarter_of_their_form() {
    let setup = Setup {
        first_year: 2016,
        last_year: 2017,
        ..Setup::default()
    };
    let mut l = solo(setup);
    while l.phase == Phase::Racing {
        simulate(&mut l);
    }
    assert_eq!(l.phase, Phase::SeasonEnd);
    let last = l.season().clone();
    let final_round = last.rounds.last().unwrap().round;
    apply(&mut l, &anna(), Action::NextSeason).unwrap();
    let next = l.season();
    assert_eq!(next.year, 2017);
    let mut carried = 0;
    for e in next.entries.iter().filter(|e| e.player.is_none()) {
        let total = last
            .ai_form
            .iter()
            .find(|f| f.entry == e.id)
            .map_or(0.0, |f| f.total());
        let stays = last.slot_at(&e.id, final_round) == Some(e.slot);
        let carry = next
            .ai_form
            .iter()
            .find(|f| f.entry == e.id)
            .map_or(0.0, |f| f.carry);
        let want = if stays {
            (total * FORM_CARRY * 100.0).round() / 100.0
        } else {
            0.0
        };
        assert!(close(carry, want), "{}: {carry} vs {want}", e.id);
        if carry != 0.0 {
            carried += 1;
            // The season starts from the season data, with the carry on top.
            assert!(close(driver::rating_of(next, e, 1), e.rating + carry));
        }
    }
    assert!(carried > 0);
    assert!(next.ai_form.iter().all(|f| f.delta == 0.0 && f.through == 0));
}
