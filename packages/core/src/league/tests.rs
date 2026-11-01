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

/// A league (from 2016 by default) with Anna and Ben both at Ferrari.
fn ferrari_2016(setup: Setup) -> League {
    let mut l = League::start(setup, &anna(), Some(7), 42).unwrap();
    apply(&mut l, &ben(), Action::Enter { number: Some(8) }).unwrap();
    apply(
        &mut l,
        &anna(),
        Action::Seat {
            slot: 2,
            replaces: None,
        },
    )
    .unwrap();
    apply(
        &mut l,
        &ben(),
        Action::Seat {
            slot: 2,
            replaces: None,
        },
    )
    .unwrap();
    apply(&mut l, &anna(), Action::Begin).unwrap();
    l
}

/// The game's report of a race on `track`: the season's grid in grid order, the reporter's car
/// first, `others` the other human cars (named as the game shows them).
fn report_of(season: &Season, session: u64, track: u8, reporter: &str) -> RaceReport {
    let mut cars = Vec::new();
    for (i, e) in season.entries.iter().enumerate() {
        let human = e.player.is_some();
        cars.push(ReportCar {
            name: if human {
                "Player".into()
            } else {
                e.name.to_uppercase()
            },
            number: e.number,
            team: e.slot,
            ai: !human,
            mine: e.player.as_deref() == Some(reporter),
            position: 0,
            grid: i as u8 + 1,
            laps: 30,
            status: 3,
            best_lap_ms: if e.player.as_deref() == Some(reporter) {
                89_000
            } else {
                90_000 + i as u32 * 10
            },
            time_ms: 3_600_000 + i as u64 * 1000,
            penalty_s: 0,
        });
    }
    // The reporter wins, the other player is second, the rest in grid order.
    cars.sort_by_key(|c| (!c.mine, c.ai, c.grid));
    for (i, c) in cars.iter_mut().enumerate() {
        c.position = i as u8 + 1;
    }
    RaceReport {
        session,
        track: Some(track),
        session_type: 15,
        cars,
        at: 0,
        laps: Vec::new(),
    }
}

#[test]
fn points_follow_each_years_rules() {
    use SessionKind::*;
    assert_eq!(points(2016, PointsRule::Historic, Race, 1, true), 25.0);
    assert_eq!(points(2019, PointsRule::Historic, Race, 1, true), 26.0);
    assert_eq!(points(2019, PointsRule::Historic, Race, 11, true), 0.0);
    assert_eq!(points(2024, PointsRule::Historic, Race, 10, true), 2.0);
    assert_eq!(points(2025, PointsRule::Historic, Race, 1, true), 25.0);
    assert_eq!(points(2021, PointsRule::Historic, Sprint, 1, false), 3.0);
    assert_eq!(points(2021, PointsRule::Historic, Sprint, 4, false), 0.0);
    assert_eq!(points(2021, PointsRule::Current, Sprint, 1, false), 8.0);
    assert_eq!(points(2022, PointsRule::Historic, Sprint, 8, false), 1.0);
    assert_eq!(points(2020, PointsRule::Current, Race, 1, true), 25.0);
    assert_eq!(points(2016, PointsRule::Historic, Race, 0, false), 0.0);
}

#[test]
fn the_catalog_has_every_season_with_full_teams() {
    for year in FIRST_YEAR..=LAST_YEAR {
        let y = catalog_year(year).unwrap_or_else(|| panic!("{year} missing"));
        assert!(y.teams.len() >= 10, "{year}");
        assert!(y.teams.iter().all(|t| t.drivers.len() == 2), "{year}");
        assert!(y.calendar.len() >= 17, "{year}");
    }
    assert_eq!(catalog_year(2026).unwrap().teams.len(), 11);
    assert_eq!(track_name(3), Some("Bahrain Grand Prix"));
}

#[test]
fn players_take_the_seats_they_pick() {
    let l = ferrari_2016(Setup::default());
    let season = l.season();
    let anna = season.player_entry("u-anna").unwrap();
    let ben = season.player_entry("u-ben").unwrap();
    assert_eq!((anna.slot, anna.number), (2, 7));
    assert_eq!((ben.slot, ben.number), (2, 8));
    // Anna took the lower-rated seat (Räikkönen), Ben the one left (Vettel).
    assert_eq!(
        l.player("u-anna").unwrap().seat.as_ref().unwrap().replaces,
        "kimi_raikkonen"
    );
    assert_eq!(
        l.player("u-ben").unwrap().seat.as_ref().unwrap().replaces,
        "sebastian_vettel"
    );
    assert!(season.entry("kimi_raikkonen").is_none() && season.entry("sebastian_vettel").is_none());
    assert_eq!(season.entries.len(), 22);
    assert_eq!(l.phase, Phase::Racing);
    let b = l.briefing("u-anna").unwrap();
    assert_eq!(
        (b.round, b.race.as_str(), b.track),
        (1, "Australian Grand Prix", Some(0))
    );
    assert_eq!(b.team.as_deref(), Some("Ferrari"));
    assert_eq!(b.replaces.as_deref(), Some("Kimi Räikkönen"));
}

#[test]
fn a_third_player_cant_take_a_full_team_and_rookies_start_at_the_back() {
    let mut l = League::start(Setup::default(), &anna(), None, 1).unwrap();
    let cleo = Actor {
        id: "u-cleo".into(),
        name: "Cleo".into(),
        owner: false,
    };
    apply(&mut l, &ben(), Action::Enter { number: None }).unwrap();
    apply(&mut l, &cleo, Action::Enter { number: None }).unwrap();
    apply(
        &mut l,
        &anna(),
        Action::Seat {
            slot: 1,
            replaces: None,
        },
    )
    .unwrap();
    apply(
        &mut l,
        &ben(),
        Action::Seat {
            slot: 1,
            replaces: None,
        },
    )
    .unwrap();
    let err = apply(
        &mut l,
        &cleo,
        Action::Seat {
            slot: 1,
            replaces: None,
        },
    )
    .unwrap_err();
    assert_eq!(err.reason, "team_full");
    let err = apply(&mut l, &cleo, Action::Begin).unwrap_err();
    assert_eq!(err.reason, "owner");
    let err = apply(&mut l, &anna(), Action::Begin).unwrap_err();
    assert_eq!(err.reason, "no_seat");

    let rookie = Setup {
        team_choice: TeamChoice::Rookie,
        ..Setup::default()
    };
    let mut r = League::start(rookie, &anna(), None, 1).unwrap();
    assert_eq!(
        apply(
            &mut r,
            &anna(),
            Action::Seat {
                slot: 1,
                replaces: None
            }
        )
        .unwrap_err()
        .reason,
        "rookie"
    );
    apply(
        &mut r,
        &anna(),
        Action::Seat {
            slot: 11,
            replaces: None,
        },
    )
    .unwrap();
}

#[test]
fn a_reported_race_counts_for_the_right_drivers() {
    let mut l = ferrari_2016(Setup::default());
    let report = report_of(l.season(), 0xabc, 0, "u-anna");
    let warnings = apply(
        &mut l,
        &anna(),
        Action::Report {
            round: None,
            kind: None,
            force: false,
            report: report.clone(),
        },
    )
    .unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let season = l.season();
    assert_eq!(season.rounds[0].state, RoundState::Raced);
    assert_eq!(season.rounds[0].race_result.len(), 22);
    let table = driver_standings(season);
    assert_eq!((table[0].name.as_str(), table[0].points), ("Anna", 25.0));
    assert_eq!((table[1].name.as_str(), table[1].points), ("Ben", 18.0));
    // Hamilton (#44) started first of the AI cars and finished third.
    assert_eq!(
        (table[2].entry.as_str(), table[2].points),
        ("lewis_hamilton", 15.0)
    );
    assert_eq!(team_standings(season)[0].name, "Ferrari");
    assert!(l.news.iter().any(|n| n.text.contains("Anna wins; Ben P2")));
    // Like the game's career: first win and podium, and the team's goal (Ferrari, the second
    // fastest car at the start of 2016: P4 or better).
    let last = &l
        .news
        .iter()
        .find(|n| n.text.contains("for the first time"))
        .unwrap()
        .text;
    assert!(
        last.contains("Anna wins a Grand Prix for the first time!"),
        "{last}"
    );
    assert!(
        last.contains("Ben stands on the podium for the first time."),
        "{last}"
    );
    assert!(
        last.contains("Anna meets the team's goal (P4 or better)."),
        "{last}"
    );

    // The same race again (Ben's launcher reports it too) changes nothing.
    let again = apply(
        &mut l,
        &ben(),
        Action::Report {
            round: None,
            kind: None,
            force: false,
            report,
        },
    )
    .unwrap();
    assert_eq!(again, ["this race is counted already"]);
    assert!(l.season().rounds[1].race_result.is_empty());
}

#[test]
fn a_race_at_another_track_is_refused_unless_forced() {
    let mut l = ferrari_2016(Setup::default());
    let report = report_of(l.season(), 1, 3, "u-ben");
    let err = apply(
        &mut l,
        &ben(),
        Action::Report {
            round: None,
            kind: None,
            force: false,
            report: report.clone(),
        },
    )
    .unwrap_err();
    assert_eq!(err.reason, "wrong_track");
    assert!(err.message.contains("Bahrain"), "{}", err.message);
    apply(
        &mut l,
        &ben(),
        Action::Report {
            round: None,
            kind: None,
            force: true,
            report,
        },
    )
    .unwrap();
    // Ben reported it, so Ben's car is the winner.
    assert_eq!(driver_standings(l.season())[0].name, "Ben");
}

#[test]
fn unknown_cars_are_warned_about_not_counted() {
    let mut l = ferrari_2016(Setup::default());
    let mut report = report_of(l.season(), 2, 0, "u-anna");
    let stranger = report.cars.iter_mut().find(|c| c.ai).unwrap();
    stranger.name = "NOBODY".into();
    stranger.number = 0;
    let warnings = apply(
        &mut l,
        &anna(),
        Action::Report {
            round: Some(1),
            kind: Some(SessionKind::Race),
            force: false,
            report,
        },
    )
    .unwrap();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("NOBODY"));
    assert_eq!(l.season().rounds[0].race_result.len(), 21);
}

#[test]
fn circuits_the_game_lacks_are_simulated_on_the_way() {
    let mut l = ferrari_2016(Setup::default());
    // Drive rounds 1 to 11 by simulating them; round 12 (Hockenheim) follows on its own.
    for _ in 0..11 {
        apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    }
    let season = l.season();
    assert_eq!(season.rounds[11].circuit, "hockenheimring");
    assert_eq!(season.rounds[11].state, RoundState::Simulated);
    assert!(season.rounds[11].auto);
    assert_eq!(season.next_round().unwrap().circuit, "spa");

    // Taking back round 11 takes back the automatic round after it too.
    apply(&mut l, &ben(), Action::Undo).unwrap();
    let season = l.season();
    assert_eq!(season.next_round().unwrap().round, 11);
    assert_eq!(season.rounds[11].state, RoundState::Pending);

    let skipping = Setup {
        untracked: Untracked::Skip,
        ..Setup::default()
    };
    let mut s = ferrari_2016(skipping);
    for _ in 0..11 {
        apply(&mut s, &anna(), Action::Simulate { round: None }).unwrap();
    }
    assert_eq!(s.season().rounds[11].state, RoundState::Skipped);
}

#[test]
fn simulations_are_the_same_everywhere() {
    let a = ferrari_2016(Setup::default());
    let mut x = a.clone();
    let mut y = a.clone();
    apply(&mut x, &anna(), Action::Simulate { round: Some(5) }).unwrap();
    apply(&mut y, &ben(), Action::Simulate { round: Some(5) }).unwrap();
    assert_eq!(x.season().rounds[4].race_result, y.season().rounds[4].race_result);
    let r = &x.season().rounds[4].race_result;
    assert_eq!(r.len(), 22);
    assert_eq!(r.iter().filter(|c| c.fastest).count(), 1);
    let total: f32 = r.iter().map(|c| c.points).sum();
    assert!(total == 101.0, "{total}");
}

#[test]
fn sprints_count_at_sprint_weekends() {
    let setup = Setup {
        first_year: 2021,
        ..Setup::default()
    };
    let mut l = ferrari_2016(setup);
    let silverstone = l
        .season()
        .rounds
        .iter()
        .find(|r| r.circuit == "silverstone")
        .unwrap()
        .round;
    while l.season().next_round().unwrap().round < silverstone {
        apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    }
    let report = report_of(l.season(), 10, 7, "u-anna");
    apply(
        &mut l,
        &anna(),
        Action::Report {
            round: None,
            kind: None,
            force: false,
            report,
        },
    )
    .unwrap();
    let r = l.season().round(silverstone).unwrap();
    assert_eq!(r.state, RoundState::Pending, "the sprint came first");
    assert_eq!(r.sprint_result[0].points, 3.0);
    let mut race = report_of(l.season(), 11, 7, "u-anna");
    race.session_type = 16;
    apply(
        &mut l,
        &anna(),
        Action::Report {
            round: None,
            kind: None,
            force: false,
            report: race,
        },
    )
    .unwrap();
    let r = l.season().round(silverstone).unwrap();
    assert_eq!(r.state, RoundState::Raced);
    assert_eq!(r.race_result[0].points, 26.0, "the win and the fastest lap");
}

#[test]
fn a_season_ends_with_offers_and_the_next_one_starts_with_the_signed_seats() {
    let setup = Setup {
        first_year: 2025,
        last_year: 2026,
        ..Setup::default()
    };
    let mut l = ferrari_2016(setup);
    while l.phase == Phase::Racing {
        apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    }
    assert_eq!(l.phase, Phase::SeasonEnd);
    assert!(l.news.iter().any(|n| n.text.contains("2025 world champion")));
    let offers: Vec<(u8, OfferKind)> = l.offers("u-anna").iter().map(|o| (o.slot, o.kind)).collect();
    assert_eq!(offers[0], (2, OfferKind::Renew));
    assert!(offers.len() >= 2);
    let mv = offers.iter().find(|o| o.1 == OfferKind::Move).unwrap().0;
    assert_eq!(
        apply(
            &mut l,
            &anna(),
            Action::Sign {
                slot: 99,
                replaces: None,
                focus: None,
            }
        )
        .unwrap_err()
        .reason,
        "no_offer"
    );
    apply(
        &mut l,
        &anna(),
        Action::Sign {
            slot: mv,
            replaces: None,
            focus: None,
        },
    )
    .unwrap();
    assert_eq!(l.phase, Phase::SeasonEnd, "Ben hasn't signed");
    apply(
        &mut l,
        &ben(),
        Action::Sign {
            slot: 2,
            replaces: Some("lewis_hamilton".into()),
            focus: None,
        },
    )
    .unwrap();
    assert_eq!(l.phase, Phase::Racing);
    assert_eq!(l.year(), 2026);
    let season = l.season();
    assert_eq!(season.player_entry("u-anna").unwrap().slot, mv);
    assert_eq!(season.player_entry("u-ben").unwrap().slot, 2);
    assert!(season.entry("lewis_hamilton").is_none());
    assert_eq!(season.entries.len(), 22);
    // 2026's Madrid round isn't in the game: it's simulated when the season gets there.
    while l.phase == Phase::Racing {
        apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    }
    assert_eq!(l.phase, Phase::Finished);
    assert_eq!(l.seasons.len(), 2);
    assert!(
        l.seasons[1]
            .rounds
            .iter()
            .any(|r| r.circuit == "madring" && r.auto)
    );
}

#[test]
fn the_owner_can_move_on_for_players_who_dont_sign() {
    let setup = Setup {
        first_year: 2024,
        last_year: 2025,
        ..Setup::default()
    };
    let mut l = ferrari_2016(setup);
    while l.phase == Phase::Racing {
        apply(&mut l, &ben(), Action::Simulate { round: None }).unwrap();
    }
    assert_eq!(
        apply(&mut l, &ben(), Action::NextSeason).unwrap_err().reason,
        "owner"
    );
    apply(&mut l, &anna(), Action::NextSeason).unwrap();
    assert_eq!(l.year(), 2025);
    assert_eq!(l.season().player_entry("u-anna").unwrap().slot, 2);
    assert_eq!(l.season().player_entry("u-ben").unwrap().slot, 2);
}

#[test]
fn undo_at_season_end_reopens_the_last_round_and_restores_ratings() {
    let setup = Setup {
        first_year: 2025,
        last_year: 2026,
        ..Setup::default()
    };
    let mut l = ferrari_2016(setup);
    while l.phase == Phase::Racing {
        apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    }
    assert!(l.ratings_before.is_some());
    let before = l.season().rounds[23].before.clone().unwrap();
    let was = before.iter().find(|p| p.player == "u-anna").unwrap();
    assert_ne!(l.player("u-anna").unwrap().rating, was.rating);
    apply(&mut l, &anna(), Action::Undo).unwrap();
    assert_eq!(l.phase, Phase::Racing);
    assert!(l.season().offers.is_empty());
    // Ratings, resource points and pay are as before the last round.
    let anna = l.player("u-anna").unwrap();
    assert_eq!(
        (anna.rating, anna.attributes, anna.rp, anna.earnings_k),
        (was.rating, was.attributes, was.rp, was.earnings_k)
    );
    assert_eq!(l.season().next_round().unwrap().round, 24);
}

#[test]
fn the_document_round_trips_as_json() {
    let mut l = ferrari_2016(Setup::default());
    apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    let text = serde_json::to_string(&l).unwrap();
    let back: League = serde_json::from_str(&text).unwrap();
    assert_eq!(back, l);
    let action: Action = serde_json::from_str(r#"{"op":"seat","slot":3}"#).unwrap();
    assert_eq!(
        action,
        Action::Seat {
            slot: 3,
            replaces: None
        }
    );
}

// ---------------------------------------------------------------------------------------------
// UDP telemetry

fn packet(id: u8, session: u64, player_car: u8, body: &[u8]) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&2025u16.to_le_bytes());
    b.extend_from_slice(&[25, 1, 0, 1, id]);
    b.extend_from_slice(&session.to_le_bytes());
    b.extend_from_slice(&0f32.to_le_bytes());
    b.extend_from_slice(&0u32.to_le_bytes());
    b.extend_from_slice(&0u32.to_le_bytes());
    b.extend_from_slice(&[player_car, 255]);
    assert_eq!(b.len(), udp::HEADER_LEN);
    b.extend_from_slice(body);
    b
}

fn session_body(track: i8, session_type: u8) -> Vec<u8> {
    let mut b = vec![0u8; 724];
    b[3] = 10;
    b[6] = session_type;
    b[7] = track as u8;
    b
}

fn participants_body(cars: &[(bool, u8, &str)]) -> Vec<u8> {
    participants_body_of(cars, udp::MAX_CARS)
}

/// Participant records of format 2025 (57 bytes) for 22 cars, or of format 2026 (60 bytes, three
/// more bytes before the name) for 24.
fn participants_body_of(cars: &[(bool, u8, &str)], records: usize) -> Vec<u8> {
    let (size, team, number_at, name_at) = if records == udp::MAX_CARS_2026 {
        (60, 5, 8, 10)
    } else {
        (57, 3, 5, 7)
    };
    let mut b = vec![cars.len() as u8];
    for i in 0..records {
        let mut r = vec![0u8; size];
        if let Some(&(ai, number, name)) = cars.get(i) {
            r[0] = u8::from(ai);
            r[team] = i as u8;
            r[number_at] = number;
            r[name_at..name_at + name.len()].copy_from_slice(name.as_bytes());
        }
        b.extend_from_slice(&r);
    }
    b
}

fn final_body(cars: &[(u8, u8, u32)]) -> Vec<u8> {
    final_body_of(cars, udp::MAX_CARS)
}

fn final_body_of(cars: &[(u8, u8, u32)], records: usize) -> Vec<u8> {
    let mut b = vec![cars.len() as u8];
    for i in 0..records {
        let mut r = [0u8; 46];
        if let Some(&(position, status, best)) = cars.get(i) {
            r[0] = position;
            r[1] = 10;
            r[2] = i as u8 + 1;
            r[5] = status;
            r[6] = 2;
            r[7..11].copy_from_slice(&best.to_le_bytes());
            r[11..19].copy_from_slice(&1234.5f64.to_le_bytes());
            r[19] = 5;
        }
        b.extend_from_slice(&r);
    }
    b
}

#[test]
fn the_collector_reports_a_race_from_the_games_packets() {
    let mut c = udp::Collector::new();
    let uid = 0x1122_3344_5566_7788;
    assert!(c.feed(&packet(1, uid, 1, &session_body(3, 15)), 1).is_none());
    let p = packet(
        4,
        uid,
        1,
        &participants_body(&[(true, 44, "HAMILTON"), (false, 7, "Player"), (true, 5, "VETTEL")]),
    );
    assert_eq!(p.len(), 1284);
    assert!(c.feed(&p, 2).is_none());
    let f = packet(
        8,
        uid,
        1,
        &final_body(&[(2, 3, 91_000), (1, 3, 90_500), (3, 4, 0)]),
    );
    assert_eq!(f.len(), 1042);
    let r = c.feed(&f, 3).expect("a race report");
    assert_eq!((r.session, r.track, r.session_type, r.at), (uid, Some(3), 15, 3));
    assert_eq!(r.cars.len(), 3);
    assert_eq!(r.cars[1].name, "Player");
    assert!(r.cars[1].mine && !r.cars[1].ai);
    assert_eq!(
        (
            r.cars[1].position,
            r.cars[1].best_lap_ms,
            r.cars[1].time_ms,
            r.cars[1].penalty_s
        ),
        (1, 90_500, 1_234_500, 5)
    );
    assert_eq!((r.cars[0].name.as_str(), r.cars[0].number), ("HAMILTON", 44));
    assert_eq!(r.cars[2].status, 4);
    // The game sends the classification more than once; it's reported once.
    assert!(c.feed(&f, 4).is_none());
    // Qualifying is reported too (short qualifying, type 8), and practice (P1, type 1).
    let q = 9;
    c.feed(&packet(1, q, 0, &session_body(3, 8)), 5);
    c.feed(&packet(4, q, 0, &participants_body(&[(false, 7, "Player")])), 5);
    let r = c
        .feed(&packet(8, q, 0, &final_body(&[(1, 3, 1)])), 5)
        .expect("qualifying");
    assert_eq!(
        (r.session_type, session_kind(r.session_type)),
        (8, Some(SessionKind::Qualifying))
    );
    let p = 10;
    c.feed(&packet(1, p, 0, &session_body(3, 1)), 6);
    c.feed(&packet(4, p, 0, &participants_body(&[(false, 7, "Player")])), 6);
    let r = c
        .feed(&packet(8, p, 0, &final_body(&[(1, 3, 1)])), 6)
        .expect("practice");
    assert_eq!(session_kind(r.session_type), Some(SessionKind::Practice));
}

#[test]
fn format_2026_sends_24_cars() {
    let as_2026 = |mut p: Vec<u8>| {
        p[..2].copy_from_slice(&2026u16.to_le_bytes());
        p
    };
    let mut c = udp::Collector::new();
    let grid: Vec<(bool, u8, &str)> = (0..24).map(|i| (i != 23, i as u8 + 1, "DRIVER")).collect();
    let order: Vec<(u8, u8, u32)> = (0..24).map(|i| (i as u8 + 1, 3, 90_000 + i)).collect();
    c.feed(&as_2026(packet(1, 5, 23, &session_body(32, 15))), 1);
    let p = as_2026(packet(4, 5, 23, &participants_body_of(&grid, 24)));
    assert_eq!(p.len(), 1470);
    c.feed(&p, 1);
    let r = c
        .feed(&as_2026(packet(8, 5, 23, &final_body_of(&order, 24))), 2)
        .expect("a 24-car race");
    assert_eq!(r.cars.len(), 24);
    assert!(r.cars[23].mine && !r.cars[23].ai);
    assert_eq!(
        (r.cars[23].position, r.cars[23].number, r.track),
        (24, 24, Some(32))
    );
    assert_eq!(r.cars[0].name, "DRIVER");
    assert!(c.unread.is_none());
    // A layout nobody knows is reported, not misread.
    c.feed(&as_2026(packet(8, 6, 0, &[0u8; 101])), 3);
    assert_eq!(c.unread, Some((8, 130)));
}

/// Packets F1 25 sent in format 2026 (`packages/core/tests/udp-2026`): the 2016 box's Australian
/// Grand Prix, a one-shot qualifying the game simulated, then the race it simulated (classified
/// with session uid 0), and an earlier race the player retired from.
mod real_2026 {
    pub const QUALI_SESSION: &[u8] = include_bytes!("../../tests/udp-2026/quali-session.bin");
    pub const QUALI_PARTICIPANTS: &[u8] = include_bytes!("../../tests/udp-2026/quali-participants.bin");
    pub const QUALI_CLASSIFICATION: &[u8] = include_bytes!("../../tests/udp-2026/quali-classification.bin");
    pub const RACE_SIMULATED: &[u8] =
        include_bytes!("../../tests/udp-2026/race-simulated-classification.bin");
    pub const RACE_DNF: &[u8] = include_bytes!("../../tests/udp-2026/race-dnf-classification.bin");
}

#[test]
fn the_games_own_2026_packets_read() {
    use real_2026::*;
    let h = udp::header(QUALI_PARTICIPANTS).unwrap();
    assert_eq!((h.format, h.packet_id, h.player_car), (2026, 4, 21));
    let cars = udp::participants(QUALI_PARTICIPANTS).unwrap();
    assert_eq!(cars.len(), 22);
    let line = |i: usize| (cars[i].ai, cars[i].team, cars[i].number, cars[i].name.as_str());
    assert_eq!(line(0), (true, 254, 8, "GROSJEAN"));
    assert_eq!(line(2), (true, 244, 44, "HAMILTON"));
    assert_eq!(line(8), (true, 248, 27, "HÜLKENBERG"));
    assert_eq!(line(21), (false, 246, 7, "RÄIKKÖNEN"));
    let s = udp::session(QUALI_SESSION).unwrap();
    assert_eq!((s.track, s.session_type), (Some(0), 9));
    // The simulated race's classification: Hamilton wins from third, Räikkönen (the player) P15.
    let race = udp::final_classification(RACE_SIMULATED).unwrap();
    assert_eq!(
        (race[2].position, race[2].grid, race[2].points, race[2].laps),
        (1, 3, 25, 5)
    );
    assert_eq!((race[21].position, race[21].grid, race[21].status), (15, 22, 3));
    assert_eq!(race[2].best_lap_ms, 77_867);
    let dnf = udp::final_classification(RACE_DNF).unwrap();
    assert!(dnf.iter().filter(|c| c.position > 0).count() == 22);
}

#[test]
fn a_race_the_game_simulated_is_reported_with_the_weekends_cars() {
    use real_2026::*;
    let mut c = udp::Collector::new();
    c.feed(QUALI_SESSION, 1);
    c.feed(QUALI_PARTICIPANTS, 1);
    let quali = c.feed(QUALI_CLASSIFICATION, 2).expect("the one-shot qualifying");
    assert_eq!((quali.session_type, quali.cars.len()), (9, 22));
    let r = c.feed(RACE_SIMULATED, 3).expect("the simulated race");
    assert_eq!((r.track, r.session_type, r.cars.len()), (Some(0), 15, 22));
    assert_ne!(r.session, 0);
    let winner = r.cars.iter().find(|c| c.position == 1).unwrap();
    assert_eq!(
        (winner.name.as_str(), winner.number, winner.grid),
        ("HAMILTON", 44, 3)
    );
    let me = r.cars.iter().find(|c| c.mine).unwrap();
    assert_eq!((me.name.as_str(), me.position, me.ai), ("RÄIKKÖNEN", 15, false));
    // The results screen keeps sending it: it counts once.
    assert!(c.feed(RACE_SIMULATED, 4).is_none());

    // Counted in a 2016 career where the player drives Räikkönen's Ferrari, every car matches.
    let me_actor = solo::actor("Robin");
    let mut l = League::start(Setup::default(), &me_actor, Some(27), 1).unwrap();
    apply(
        &mut l,
        &me_actor,
        Action::Seat {
            slot: 2,
            replaces: Some("kimi_raikkonen".into()),
        },
    )
    .unwrap();
    apply(&mut l, &me_actor, Action::Begin).unwrap();
    for report in [quali, r] {
        let warnings = apply(
            &mut l,
            &me_actor,
            Action::Report {
                round: None,
                kind: None,
                force: false,
                report,
            },
        )
        .unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
    }
    let round = &l.season().rounds[0];
    // The game's Verstappen took pole in the Red Bull, which Daniil Kvyat drove in Australia 2016
    // (Verstappen joined Red Bull in Spain): the pole is Kvyat's. The player set no time.
    let pole = round.quali_result.iter().find(|c| c.position == 1).unwrap();
    assert_eq!(pole.entry, "daniil_kvyat");
    let mine = round
        .quali_result
        .iter()
        .find(|c| c.entry.starts_with("p:"))
        .unwrap();
    assert_eq!((mine.position, mine.status), (22, Finish::Dnf));
    assert!(l.news.iter().any(
        |n| n.text == "Round 1, Australian Grand Prix qualifying: Daniil Kvyat on pole; Robin no time."
    ));
    assert_eq!((round.state, round.race_result.len()), (RoundState::Raced, 22));
    let standings = driver_standings(l.season());
    assert_eq!(
        (standings[0].name.as_str(), standings[0].points),
        ("Lewis Hamilton", 25.0)
    );
    let mine = standings.iter().find(|d| d.player.is_some()).unwrap();
    assert_eq!((mine.name.as_str(), mine.finishes[0]), ("Robin", Some(15)));
    // Palmer is BRAHMS in the 2016 box; his number finds him.
    assert!(round.race_result.iter().any(|c| c.entry == "jolyon_palmer"));
}

#[test]
fn reports_survive_on_disk() {
    let dir = std::env::temp_dir().join(format!("f1mp-races-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut l = ferrari_2016(Setup::default());
    let report = report_of(l.season(), 77, 0, "u-anna");
    udp::save(&dir, &report).unwrap();
    let back = udp::saved(&dir);
    assert_eq!(back, std::slice::from_ref(&report));
    apply(
        &mut l,
        &anna(),
        Action::Report {
            round: None,
            kind: None,
            force: false,
            report: back[0].clone(),
        },
    )
    .unwrap();
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_telemetry_setting_is_read_and_turned_on() {
    let xml = "<a>\n\t<motion>\n\t\t<udp enabled=\"false\" broadcast=\"false\" ip=\"127.0.0.1\" port=\"20780\" sendRate=\"60\" format=\"2023\" yourTelemetry=\"restricted\" />\n\t</motion>\n</a>";
    let s = udp::parse_settings(xml).unwrap();
    assert_eq!((s.enabled, s.port, s.format.as_str()), (false, 20780, "2023"));
    assert!(!s.usable());
    let on = udp::enable_in(xml).unwrap();
    let s = udp::parse_settings(&on).unwrap();
    assert!(s.enabled && s.usable());
    assert_eq!((s.port, s.format.as_str()), (20780, "2025"));
    assert!(on.contains("yourTelemetry=\"restricted\""));
    assert_eq!(s.listen_addr(), "127.0.0.1:20780".parse().unwrap());
    let newer = xml.replace("2023", "2026");
    assert_eq!(
        udp::parse_settings(&udp::enable_in(&newer).unwrap())
            .unwrap()
            .format,
        "2026"
    );
}

#[test]
fn a_solo_career_is_driven_alone_and_kept_on_disk() {
    let root = std::env::temp_dir().join(format!("f1mp-solo-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let setup = Setup {
        first_year: 2016,
        last_year: 2017,
        ..Setup::default()
    };
    let c = solo::create(&root, "Alone", "Robin", setup, Some(5)).unwrap();
    assert!(solo::is_solo(&c.id));
    assert_eq!(c.league.players.len(), 1);
    // Seat, begin, drive one race, simulate the rest of the season.
    solo::act(
        &root,
        &c.id,
        Action::Seat {
            slot: 2,
            replaces: None,
        },
    )
    .unwrap();
    let (c2, _) = solo::act(&root, &c.id, Action::Begin).unwrap();
    assert_eq!(c2.league.phase, Phase::Racing);
    let report = report_of(c2.league.season(), 9, 0, solo::ME);
    let (c3, warnings) = solo::act(
        &root,
        &c.id,
        Action::Report {
            round: None,
            kind: None,
            force: false,
            report,
        },
    )
    .unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let me = c3.league.season().player_entry(solo::ME).unwrap().id.clone();
    assert_eq!(c3.league.season().rounds[0].race_result[0].entry, me);
    while solo::load(&root, &c.id).unwrap().league.phase == Phase::Racing {
        solo::act(&root, &c.id, Action::Simulate { round: None }).unwrap();
    }
    let ended = solo::load(&root, &c.id).unwrap();
    assert_eq!(ended.league.phase, Phase::SeasonEnd);
    // Alone, signing an offer starts the next season straight away.
    let slot = ended.league.offers(solo::ME)[0].slot;
    let (next, _) = solo::act(
        &root,
        &c.id,
        Action::Sign {
            slot,
            replaces: None,
            focus: None,
        },
    )
    .unwrap();
    assert_eq!((next.league.year(), next.league.phase), (2017, Phase::Racing));
    assert!(next.version > c3.version);
    // The list finds it, earlier versions are kept, a refused action changes nothing.
    assert_eq!(solo::list(&root)[0].id, c.id);
    let history = std::fs::read_dir(solo::dir(&root).join("history").join(&c.id))
        .unwrap()
        .count();
    assert!((1..=10).contains(&history), "{history}");
    let err = solo::act(&root, &c.id, Action::Begin).unwrap_err();
    assert_eq!(err.reason(), "phase");
    assert_eq!(solo::load(&root, &c.id).unwrap().version, next.version);
    solo::delete(&root, &c.id).unwrap();
    assert!(solo::list(&root).is_empty());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_career_has_rivals_duels_goals_contracts_and_a_record() {
    let me = solo::actor("Robin");
    let setup = Setup {
        first_year: 2016,
        last_year: 2018,
        ..Setup::default()
    };
    let mut l = League::start(setup, &me, Some(27), 7).unwrap();
    // Williams: the fifth fastest car of 2016.
    apply(
        &mut l,
        &me,
        Action::Seat {
            slot: 7,
            replaces: None,
        },
    )
    .unwrap();
    apply(&mut l, &me, Action::Begin).unwrap();
    let p = l.player(solo::ME).unwrap();
    assert_eq!(p.contract_until, Some(2016));
    // A rival from another team with a car and rating like the player's.
    let rival = p.rival.clone().expect("a rival");
    let season = l.season();
    let entry = season.player_entry(solo::ME).unwrap().clone();
    assert_ne!(season.entry(&rival).unwrap().slot, entry.slot);
    assert!(
        l.news
            .iter()
            .any(|n| n.text.starts_with("Robin's rival this season: "))
    );
    let b = l.briefing(solo::ME).unwrap();
    assert!(b.solo && !b.quali_done);
    // Williams start 2016 as the fourth fastest car (Red Bull found its pace later): P8.
    assert_eq!(b.goal, Some(8));
    assert_eq!(b.mate.as_deref(), Some("Valtteri Bottas"));
    assert!(b.rival.is_some());

    // Simulated rounds have a qualifying too: poles count.
    apply(&mut l, &me, Action::Simulate { round: None }).unwrap();
    let r1 = &l.season().rounds[0];
    assert_eq!(r1.quali_result.len(), 22);
    let pole = r1.quali_result[0].entry.clone();
    // The race starts from that qualifying.
    assert_eq!(r1.race_result.iter().find(|c| c.entry == pole).unwrap().grid, 1);
    let standings = driver_standings(l.season());
    assert_eq!(standings.iter().map(|d| d.poles).sum::<u32>(), 1);
    assert_eq!(standings.iter().find(|d| d.entry == pole).unwrap().poles, 1);
    assert_eq!(standings.iter().map(|d| d.fastest_laps).sum::<u32>(), 1);

    // A qualifying reported on its own can be taken back.
    let mut quali = report_of(l.season(), 31, 17, solo::ME);
    quali.session_type = 8;
    apply(
        &mut l,
        &me,
        Action::Report {
            round: None,
            kind: None,
            force: true,
            report: quali,
        },
    )
    .unwrap();
    let r2 = l.season().next_round().unwrap();
    assert_eq!((r2.round, r2.quali_result.len(), r2.started()), (2, 22, true));
    assert_eq!(r2.quali_result[0].entry, entry.id);
    assert!(l.briefing(solo::ME).unwrap().quali_done);
    assert!(
        l.news
            .iter()
            .any(|n| n.text == "Robin takes the first pole position of their career!")
    );
    apply(&mut l, &me, Action::Undo).unwrap();
    assert!(l.season().round(2).unwrap().quali_result.is_empty());
    // A sprint shootout isn't counted.
    let mut shootout = report_of(l.season(), 32, 17, solo::ME);
    shootout.session_type = 12;
    let err = apply(
        &mut l,
        &me,
        Action::Report {
            round: None,
            kind: None,
            force: true,
            report: shootout,
        },
    )
    .unwrap_err();
    assert_eq!(err.reason, "not_counted");

    // A highly rated driver, so three teams make offers.
    l.players[0].attributes = Some(Attributes::from_rating(92.0));
    l.players[0].rating = 92.0;
    while l.phase == Phase::Racing {
        apply(&mut l, &me, Action::Simulate { round: None }).unwrap();
    }
    assert_eq!(l.phase, Phase::SeasonEnd);
    // The duels are in the news.
    assert!(
        l.news
            .iter()
            .any(|n| n.text.contains("against team-mate Valtteri Bottas: qualifying "))
    );
    assert!(l.news.iter().any(|n| n.text.contains(" the rivalry with ")));
    let h = head_to_head(l.season(), &entry.id, "valtteri_bottas").unwrap();
    assert_eq!(h.race.0 + h.race.1, 21);
    assert_eq!(h.quali.0 + h.quali.1, 21);

    // The contract ran out: the own team renews, three others offer one, two and three years.
    let offers: Vec<Offer> = l.offers(solo::ME).into_iter().cloned().collect();
    assert_eq!(offers[0].kind, OfferKind::Renew);
    let moves: Vec<u8> = offers
        .iter()
        .filter(|o| o.kind == OfferKind::Move)
        .map(|o| o.years)
        .collect();
    assert_eq!(moves, [1, 2, 3]);
    // Sign the three-year deal: in 2017 the player is under contract until 2019.
    let three = offers.iter().find(|o| o.years == 3).unwrap().clone();
    apply(
        &mut l,
        &me,
        Action::Sign {
            slot: three.slot,
            replaces: None,
            focus: None,
        },
    )
    .unwrap();
    assert_eq!(l.year(), 2017);
    let p = l.player(solo::ME).unwrap();
    assert_eq!((p.contract_until, p.signed_until), (Some(2019), None));
    assert!(p.rival.is_some());
    while l.phase == Phase::Racing {
        apply(&mut l, &me, Action::Simulate { round: None }).unwrap();
    }
    // Under contract: the team keeps the player, only faster teams may buy them out.
    let season = l.season().clone();
    let own = season.player_entry(solo::ME).unwrap().slot;
    let offers: Vec<Offer> = l.offers(solo::ME).into_iter().cloned().collect();
    assert_eq!(
        (offers[0].kind, offers[0].slot, offers[0].years),
        (OfferKind::Contract, own, 2)
    );
    let cat = catalog_year(2018).unwrap();
    let rank = |slot: u8| pace_rank(cat.teams.iter().map(|t| (t.slot, t.pace)), slot);
    for o in offers.iter().skip(1) {
        assert!(rank(o.slot) < rank(own), "{o:?}");
    }
    apply(&mut l, &me, Action::NextSeason).unwrap();
    assert_eq!(l.year(), 2018);
    assert_eq!(l.player(solo::ME).unwrap().contract_until, Some(2019));

    // The career record adds the seasons up.
    let stats = career_stats(&l, solo::ME);
    assert_eq!(stats.seasons.len(), 3);
    assert_eq!(stats.seasons[0].starts, 21);
    assert!(stats.seasons[0].complete && !stats.seasons[2].complete);
    let wins: u32 = stats.seasons.iter().map(|s| s.wins).sum();
    assert_eq!(stats.wins, wins);
    assert!(stats.starts >= 41);
}

// ---------------------------------------------------------------------------------------------
// Practice, development, salary, recognition and line-up changes

/// The practice session the game reports for `reporter`, with `laps` of theirs.
fn practice_of(season: &Season, session: u64, track: u8, reporter: &str, laps: &[(u32, bool)]) -> RaceReport {
    let mut r = report_of(season, session, track, reporter);
    r.session_type = 1;
    r.laps = laps.iter().map(|&(ms, valid)| Lap { ms, valid }).collect();
    r
}

fn player<'a>(l: &'a League, id: &str) -> &'a Player {
    l.player(id).unwrap()
}

#[test]
fn practice_programmes_earn_resource_points_and_train_the_driver() {
    let mut l = ferrari_2016(Setup::default());
    // Anna lets her practice be simulated: three programmes, fewer points than driven.
    apply(&mut l, &anna(), Action::Practice { round: None }).unwrap();
    let line = l.season().rounds[0].practice[0].clone();
    assert_eq!(
        (line.player.as_str(), line.driven, line.programmes.len()),
        ("u-anna", false, 3)
    );
    assert!(line.rp > 0 && player(&l, "u-anna").rp == line.rp);
    assert!(l.briefing("u-anna").unwrap().practice_done);
    assert!(!l.briefing("u-ben").unwrap().practice_done);
    assert_eq!(
        apply(&mut l, &anna(), Action::Practice { round: None })
            .unwrap_err()
            .reason,
        "round_done"
    );
    // Ben drives his in the game: ten laps close to his best, one off the track, faster than Anna.
    let laps: Vec<(u32, bool)> = (0..10).map(|i| (89_000 + i * 150, i != 3)).collect();
    let report = practice_of(l.season(), 70, 0, "u-ben", &laps);
    apply(
        &mut l,
        &ben(),
        Action::Report {
            round: None,
            kind: None,
            force: false,
            report: report.clone(),
        },
    )
    .unwrap();
    let ben_line = l.season().rounds[0]
        .practice
        .iter()
        .find(|p| p.player == "u-ben")
        .unwrap()
        .clone();
    assert!(ben_line.driven);
    assert_eq!((ben_line.laps, ben_line.best_lap_ms), (30, 89_000));
    let grade = |p: Programme| {
        ben_line
            .programmes
            .iter()
            .find(|x| x.programme == p)
            .unwrap()
            .grade
    };
    assert_eq!(grade(Programme::TrackAcclimatisation), 100);
    assert!(grade(Programme::QualifyingPace) >= 90, "{ben_line:?}");
    assert!(grade(Programme::TyreManagement) > 70, "{ben_line:?}");
    assert!(ben_line.rp > line.rp);
    assert!(
        player(&l, "u-ben").attributes.unwrap().experience > Attributes::from_rating(START_RATING).experience
    );
    assert!(l.news.iter().any(|n| n.text.contains("practice: Ben grade")));
    // The same session again counts once; the round isn't over, it has started.
    let again = apply(
        &mut l,
        &ben(),
        Action::Report {
            round: None,
            kind: None,
            force: false,
            report,
        },
    )
    .unwrap();
    assert_eq!(again, ["this practice is counted already"]);
    assert!(l.season().rounds[0].started());
    // Taking it back gives the points back.
    apply(&mut l, &anna(), Action::Undo).unwrap();
    assert!(l.season().rounds[0].practice.is_empty());
    assert_eq!((player(&l, "u-anna").rp, player(&l, "u-ben").rp), (0, 0));
    // Simulating a round simulates the practice of everyone who didn't drive it.
    apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    assert_eq!(l.season().rounds[0].practice.len(), 2);
}

#[test]
fn resource_points_develop_the_car_and_new_parts_arrive_two_rounds_later() {
    let mut l = ferrari_2016(Setup::default());
    let err = apply(&mut l, &anna(), Action::Develop { part: Part::Aero }).unwrap_err();
    assert_eq!(err.reason, "no_points");
    l.players[0].rp = 5_000;
    let before = (l.season().pace_at(2, 2), l.season().pace_at(2, 3));
    apply(&mut l, &anna(), Action::Develop { part: Part::Aero }).unwrap();
    assert_eq!(player(&l, "u-anna").rp, 5_000 - progress::upgrade_cost(0));
    let season = l.season();
    assert_eq!((season.upgrades[0].level, season.upgrades[0].ready), (1, 3));
    assert_eq!(season.part_level(2, Part::Aero), 1);
    // On the car from round 3: the Ferrari is 0.07% faster from then on.
    assert_eq!(season.pace_at(2, 2), before.0);
    assert!((before.1 - season.pace_at(2, 3) - Part::Aero.gain()).abs() < 1e-5);
    assert!(
        l.news
            .iter()
            .any(|n| n.text.contains("develop Aerodynamics level 1"))
    );
    // Durability makes the car break down less often.
    apply(
        &mut l,
        &anna(),
        Action::Develop {
            part: Part::Durability,
        },
    )
    .unwrap();
    assert!(l.season().breakdowns(2, 3) < 1.0);
    // Ben spends his own points on the same car.
    l.players[1].rp = progress::upgrade_cost(1);
    apply(&mut l, &ben(), Action::Develop { part: Part::Aero }).unwrap();
    assert_eq!(
        (l.season().part_level(2, Part::Aero), player(&l, "u-ben").rp),
        (2, 0)
    );

    // A part ordered before a round survives taking that round back; one ordered after it
    // started goes, and its points come back.
    apply(&mut l, &anna(), Action::Practice { round: None }).unwrap();
    let rp = player(&l, "u-anna").rp;
    apply(&mut l, &anna(), Action::Develop { part: Part::Chassis }).unwrap();
    assert_eq!(l.season().upgrades.len(), 4);
    apply(&mut l, &anna(), Action::Undo).unwrap();
    assert_eq!(l.season().upgrades.len(), 3);
    assert!(player(&l, "u-anna").rp < rp, "the practice's points went too");

    // Late in the season a new part wouldn't be ready.
    while l.season().next_round().unwrap().round < 20 {
        apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    }
    l.players[0].rp = 50_000;
    let err = apply(
        &mut l,
        &anna(),
        Action::Develop {
            part: Part::Powertrain,
        },
    )
    .unwrap_err();
    assert_eq!(err.reason, "too_late");

    // What a team developed carries into next season: a sixth in 2017, when the rules changed.
    while l.phase == Phase::Racing {
        apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    }
    let gain = l.season().upgrade_gain(2, u8::MAX);
    apply(&mut l, &anna(), Action::NextSeason).unwrap();
    let ferrari = l.season().team(2).unwrap();
    assert!(
        (ferrari.carry + gain / 6.0).abs() < 1e-5,
        "{} {gain}",
        ferrari.carry
    );
    assert!(l.season().upgrades.is_empty());
}

#[test]
fn a_part_can_be_developed_ten_times_a_season() {
    let mut l = ferrari_2016(Setup::default());
    l.players[0].rp = 1_000_000;
    for _ in 0..progress::UPGRADE_LEVELS {
        apply(&mut l, &anna(), Action::Develop { part: Part::Chassis }).unwrap();
    }
    let err = apply(&mut l, &anna(), Action::Develop { part: Part::Chassis }).unwrap_err();
    assert_eq!(err.reason, "max_level");
}

#[test]
fn salaries_are_paid_each_round_with_a_bonus_per_point() {
    let mut l = ferrari_2016(Setup::default());
    let start = player(&l, "u-anna").clone();
    assert!(start.salary_k >= 1_000 && start.bonus_k > 0, "{start:?}");
    let report = report_of(l.season(), 0xabc, 0, "u-anna");
    apply(
        &mut l,
        &anna(),
        Action::Report {
            round: None,
            kind: None,
            force: false,
            report,
        },
    )
    .unwrap();
    let rounds = l.season().rounds.len() as u64;
    let earned = player(&l, "u-anna").earnings_k;
    assert_eq!(
        earned,
        u64::from(start.salary_k) / rounds + 25 * u64::from(start.bonus_k)
    );
    // A round left out pays the salary, no bonus.
    apply(&mut l, &anna(), Action::Skip { round: None }).unwrap();
    assert_eq!(
        player(&l, "u-anna").earnings_k,
        earned + u64::from(start.salary_k) / rounds
    );
    assert_eq!(progress::money(3_650), "$3.65M");
    assert_eq!(progress::money(10_000), "$10M");
    assert_eq!(progress::money(850), "$850k");

    // Offers come with a salary; asking for more salary gets a quarter more.
    let setup = Setup {
        first_year: 2024,
        last_year: 2025,
        ..Setup::default()
    };
    let mut l = ferrari_2016(setup);
    while l.phase == Phase::Racing {
        apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    }
    assert!(l.news.iter().any(|n| n.text.contains("earned $")));
    let offer = l.offers("u-anna")[0].clone();
    assert!(offer.salary_k > 0 && offer.bonus_k > 0);
    apply(
        &mut l,
        &anna(),
        Action::Sign {
            slot: offer.slot,
            replaces: None,
            focus: Some(Focus::Salary),
        },
    )
    .unwrap();
    let expected = (offer.salary_k as f32 * 1.25 / 50.0).round() as u32 * 50;
    assert_eq!(player(&l, "u-anna").signed_salary_k, expected);
    apply(&mut l, &anna(), Action::NextSeason).unwrap();
    let a = player(&l, "u-anna");
    assert_eq!((a.salary_k, a.focus), (expected, Some(Focus::Salary)));
}

#[test]
fn recognition_makes_a_number_one_driver_and_a_team_can_let_a_driver_go() {
    let mut l = ferrari_2016(Setup::default());
    l.players[0].recognition = 64.0;
    let report = report_of(l.season(), 0xabc, 0, "u-anna");
    apply(
        &mut l,
        &anna(),
        Action::Report {
            round: None,
            kind: None,
            force: false,
            report,
        },
    )
    .unwrap();
    assert!(player(&l, "u-anna").recognition >= progress::NUMBER_ONE);
    assert!(
        l.news
            .iter()
            .any(|n| n.text.contains("Anna is now Ferrari's number one driver."))
    );
    // A driver the team doesn't value gets no new contract when theirs runs out.
    let setup = Setup {
        first_year: 2024,
        last_year: 2025,
        ..Setup::default()
    };
    let mut l = ferrari_2016(setup);
    while l.season().next_round().is_some_and(|r| r.round < 24) {
        apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    }
    l.players[0].recognition = 5.0;
    apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    assert_eq!(l.phase, Phase::SeasonEnd);
    assert!(l.offers("u-anna").iter().all(|o| o.kind == OfferKind::Move));
    assert!(
        l.news
            .iter()
            .any(|n| n.text.contains("Ferrari doesn't renew the contract."))
    );
    // Moving to another team starts over with its recognition.
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
    apply(&mut l, &anna(), Action::NextSeason).unwrap();
    assert_eq!(player(&l, "u-anna").recognition, progress::RECOGNITION_NEW_TEAM);
}

#[test]
fn the_real_line_up_changes_happen_during_the_season() {
    let mut l = ferrari_2016(Setup::default());
    let season = l.season();
    // 2016: Kvyat starts at Red Bull and Verstappen at Toro Rosso until Spain, Vandoorne stands
    // in for Alonso in Bahrain, Ocon replaces Haryanto from Spa.
    assert_eq!(season.changes().len(), 4);
    assert_eq!(season.reserves.len(), 2);
    assert_eq!(season.slot_at("daniil_kvyat", 1), Some(3));
    assert_eq!(season.slot_at("max_verstappen", 1), Some(8));
    assert_eq!(season.slot_at("max_verstappen", 5), Some(3));
    assert_eq!(season.slot_at("fernando_alonso", 2), None);
    assert_eq!(season.slot_at("stoffel_vandoorne", 2), Some(4));
    assert_eq!(season.slot_at("stoffel_vandoorne", 3), None);
    assert_eq!(season.slot_at("esteban_ocon", 13), Some(11));
    for round in [1, 2, 13] {
        assert_eq!(season.lineup(round).len(), 22, "round {round}");
    }
    assert!(
        l.news
            .iter()
            .any(|n| n.text.contains("Daniil Kvyat drives for Red Bull Racing"))
    );
    let b = l.briefing("u-anna").unwrap();
    assert!(
        b.lineup
            .iter()
            .any(|n| n
                == "Daniil Kvyat drives Max Verstappen's Red Bull Racing (the game shows Max Verstappen)"),
        "{:?}",
        b.lineup
    );
    apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    assert!(l.news.iter().any(|n| n.text.contains(
        "Stoffel Vandoorne stands in for Fernando Alonso at McLaren until Fernando Alonso is back at the Chinese Grand Prix"
    )));
    // The game's season box still has Alonso in Bahrain: his car is Vandoorne's.
    let mut report = report_of(l.season(), 0xb, 3, "u-anna");
    let alonso = report.cars.iter_mut().find(|c| c.number == 14).unwrap();
    assert_eq!(alonso.name, "FERNANDO ALONSO");
    let alonso_place = alonso.position;
    apply(
        &mut l,
        &anna(),
        Action::Report {
            round: None,
            kind: None,
            force: false,
            report,
        },
    )
    .unwrap();
    let r2 = &l.season().rounds[1];
    let v = r2
        .race_result
        .iter()
        .find(|c| c.entry == "stoffel_vandoorne")
        .unwrap();
    assert_eq!(v.position, alonso_place);
    assert!(r2.race_result.iter().all(|c| c.entry != "fernando_alonso"));
    // Vandoorne is in the standings, his points McLaren's.
    let drivers = driver_standings(l.season());
    let vd = drivers.iter().find(|d| d.entry == "stoffel_vandoorne").unwrap();
    assert_eq!(vd.slot, 4);
    let mclaren = team_standings(l.season())
        .into_iter()
        .find(|t| t.slot == 4)
        .unwrap();
    let mclaren_drivers: f32 = drivers
        .iter()
        .filter(|d| ["fernando_alonso", "jenson_button", "stoffel_vandoorne"].contains(&d.entry.as_str()))
        .map(|d| d.points)
        .sum();
    assert_eq!(mclaren.points, mclaren_drivers);

    // A change that would touch a player's seat doesn't happen.
    let mut solo_l = League::start(Setup::default(), &anna(), None, 3).unwrap();
    apply(
        &mut solo_l,
        &anna(),
        Action::Seat {
            slot: 4,
            replaces: Some("fernando_alonso".into()),
        },
    )
    .unwrap();
    assert!(
        solo_l
            .season()
            .changes()
            .iter()
            .all(|c| c.out != "fernando_alonso")
    );
    // ... and none happen when the career leaves them out.
    let fixed = ferrari_2016(Setup {
        lineup_changes: false,
        ..Setup::default()
    });
    assert!(fixed.season().changes().is_empty() && fixed.season().reserves.is_empty());
}

#[test]
fn a_career_from_before_gets_ratings_a_salary_and_line_up_changes_from_the_next_round() {
    let mut l = ferrari_2016(Setup::default());
    apply(&mut l, &anna(), Action::Simulate { round: None }).unwrap();
    // As an older version saved it.
    for p in &mut l.players {
        p.attributes = None;
        p.salary_k = 0;
        p.bonus_k = 0;
    }
    let season = l.seasons.last_mut().unwrap();
    season.changes = None;
    season.reserves.clear();
    let text = serde_json::to_string(&l).unwrap();
    let mut old: League = serde_json::from_str(&text).unwrap();
    assert!(old.season().changes.is_none());
    apply(&mut old, &anna(), Action::Practice { round: None }).unwrap();
    let a = player(&old, "u-anna");
    assert!(a.attributes.is_some() && a.salary_k > 0);
    // The Kvyat-Verstappen swap is in force from round 2, the round the career is at.
    assert!(old.season().changes().iter().all(|c| c.round >= 2));
    assert_eq!(old.season().slot_at("daniil_kvyat", 2), Some(3));
    assert_eq!(old.season().slot_at("daniil_kvyat", 1), Some(8));
}

/// A session history packet body (format 2025: 14-byte laps, 3-byte stints) for `car`.
fn history_body(car: u8, laps: &[(u32, bool)]) -> Vec<u8> {
    let mut b = vec![car, laps.len() as u8, 1, 1, 1, 1, 1];
    for i in 0..100 {
        let mut r = [0u8; 14];
        if let Some(&(ms, valid)) = laps.get(i) {
            r[..4].copy_from_slice(&ms.to_le_bytes());
            r[13] = u8::from(valid);
        }
        b.extend_from_slice(&r);
    }
    b.extend_from_slice(&[0u8; 24]);
    b
}

#[test]
fn the_collector_reports_a_practice_session_from_the_laps() {
    let mut c = udp::Collector::new();
    let uid = 0x77;
    c.feed(&packet(1, uid, 1, &session_body(3, 2)), 1);
    c.feed(
        &packet(
            4,
            uid,
            1,
            &participants_body(&[(true, 44, "HAMILTON"), (false, 7, "Player")]),
        ),
        1,
    );
    let mine = [(91_000, true), (90_200, false), (90_400, true), (0, true)];
    let h = packet(11, uid, 1, &history_body(1, &mine));
    assert_eq!(h.len(), 1460);
    let (car, laps) = udp::session_history(&h).unwrap();
    assert_eq!((car, laps.len()), (1, 3), "the lap being driven has no time");
    c.feed(&h, 2);
    c.feed(&packet(11, uid, 1, &history_body(0, &[(90_100, true)])), 2);
    let mut send = vec![0u8; 16];
    send[..4].copy_from_slice(b"SEND");
    let r = c.feed(&packet(3, uid, 1, &send), 3).expect("the practice");
    assert_eq!((r.session, r.session_type, r.track), (uid, 2, Some(3)));
    assert_eq!(r.laps.len(), 3);
    let me = r.cars.iter().find(|c| c.mine).unwrap();
    // The best clean lap counts (90.2 s was off the track).
    assert_eq!((me.position, me.best_lap_ms, me.laps), (2, 90_400, 3));
    assert!(c.feed(&packet(3, uid, 1, &send), 4).is_none());
    // Leaving a practice for the next session reports it too.
    let p2 = 0x78;
    c.feed(&packet(1, p2, 0, &session_body(3, 3)), 5);
    c.feed(&packet(4, p2, 0, &participants_body(&[(false, 7, "Player")])), 5);
    c.feed(&packet(11, p2, 0, &history_body(0, &[(92_000, true)])), 5);
    let r = c
        .feed(&packet(1, 0x79, 0, &session_body(3, 8)), 6)
        .expect("the practice left");
    assert_eq!((r.session, r.session_type), (p2, 3));
    // A practice the player didn't drive isn't reported.
    let p3 = 0x80;
    c.feed(&packet(1, p3, 0, &session_body(3, 1)), 7);
    c.feed(&packet(4, p3, 0, &participants_body(&[(false, 7, "Player")])), 7);
    assert!(c.feed(&packet(3, p3, 0, &send), 8).is_none());
}
