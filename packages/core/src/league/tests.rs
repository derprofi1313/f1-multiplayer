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

/// `action` by `who`, then agreed to by every other active player: in a shared career Simulate,
/// Skip and Undo wait for everyone (W2-1).
pub(super) fn agreed(l: &mut League, who: &Actor, action: Action) -> Vec<String> {
    let warnings = apply(l, who, action).unwrap();
    let others: Vec<Actor> = l
        .waiting_for()
        .iter()
        .map(|p| Actor {
            id: p.id.clone(),
            name: p.name.clone(),
            owner: false,
        })
        .collect();
    for a in others {
        apply(l, &a, Action::Vote { yes: true, force: false }).unwrap();
    }
    warnings
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
            // The other players' cars carry their LAN names (the launcher sets the driver's name).
            name: if e.player.as_deref() == Some(reporter) {
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
        ..RaceReport::default()
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
        agreed(&mut l, &anna(), Action::Simulate { round: None });
    }
    let season = l.season();
    assert_eq!(season.rounds[11].circuit, "hockenheimring");
    assert_eq!(season.rounds[11].state, RoundState::Simulated);
    assert!(season.rounds[11].auto);
    assert_eq!(season.next_round().unwrap().circuit, "spa");

    // Taking back round 11 takes back the automatic round after it too.
    agreed(&mut l, &ben(), Action::Undo);
    let season = l.season();
    assert_eq!(season.next_round().unwrap().round, 11);
    assert_eq!(season.rounds[11].state, RoundState::Pending);

    let skipping = Setup {
        untracked: Untracked::Skip,
        ..Setup::default()
    };
    let mut s = ferrari_2016(skipping);
    for _ in 0..11 {
        agreed(&mut s, &anna(), Action::Simulate { round: None });
    }
    assert_eq!(s.season().rounds[11].state, RoundState::Skipped);
}

#[test]
fn simulations_are_the_same_everywhere() {
    // Rounds are counted in order: simulate rounds 1 to 4, then round 5 on two copies.
    let mut a = ferrari_2016(Setup::default());
    for _ in 0..4 {
        agreed(&mut a, &anna(), Action::Simulate { round: None });
    }
    let mut x = a.clone();
    let mut y = a.clone();
    agreed(&mut x, &anna(), Action::Simulate { round: Some(5) });
    agreed(&mut y, &ben(), Action::Simulate { round: Some(5) });
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
        agreed(&mut l, &anna(), Action::Simulate { round: None });
    }
    let mut report = report_of(l.season(), 10, 7, "u-anna");
    report.weekend = vec![1, 10, 11, 12, 15, 16];
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
        agreed(&mut l, &anna(), Action::Simulate { round: None });
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
        agreed(&mut l, &anna(), Action::Simulate { round: None });
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
        agreed(&mut l, &ben(), Action::Simulate { round: None });
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
        agreed(&mut l, &anna(), Action::Simulate { round: None });
    }
    assert!(l.ratings_before.is_some());
    let before = l.season().rounds[23].before.clone().unwrap();
    let was = before.iter().find(|p| p.player == "u-anna").unwrap();
    assert_ne!(l.player("u-anna").unwrap().rating, was.rating);
    agreed(&mut l, &anna(), Action::Undo);
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
    agreed(&mut l, &anna(), Action::Simulate { round: None });
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
    agreed(&mut l, &anna(), Action::Undo);
    assert!(l.season().rounds[0].practice.is_empty());
    assert_eq!((player(&l, "u-anna").rp, player(&l, "u-ben").rp), (0, 0));
    // Simulating a round simulates the practice of everyone who didn't drive it.
    agreed(&mut l, &anna(), Action::Simulate { round: None });
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
    agreed(&mut l, &anna(), Action::Undo);
    assert_eq!(l.season().upgrades.len(), 3);
    assert!(player(&l, "u-anna").rp < rp, "the practice's points went too");

    // Late in the season a new part wouldn't be ready.
    while l.season().next_round().unwrap().round < 20 {
        agreed(&mut l, &anna(), Action::Simulate { round: None });
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
        agreed(&mut l, &anna(), Action::Simulate { round: None });
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
fn develop_orders_the_main_branch_until_the_cost_cap() {
    // W2-6: a part's main branch has five nodes, and a team spends at most rnd::COST_CAP a
    // season; the chassis' main branch costs 6,500, three aero nodes 3,150 more (9,650 of
    // 10,000), and the fourth (1,550) would go over.
    let mut l = ferrari_2016(Setup::default());
    l.players[0].rp = 1_000_000;
    for _ in 0..5 {
        apply(&mut l, &anna(), Action::Develop { part: Part::Chassis }).unwrap();
    }
    let err = apply(&mut l, &anna(), Action::Develop { part: Part::Chassis }).unwrap_err();
    assert_eq!(err.reason, "max_level");
    for _ in 0..3 {
        apply(&mut l, &anna(), Action::Develop { part: Part::Aero }).unwrap();
    }
    assert_eq!(rnd::spent(l.season(), 2), 9_650);
    let err = apply(&mut l, &anna(), Action::Develop { part: Part::Aero }).unwrap_err();
    assert_eq!(err.reason, "cost_cap");
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
    agreed(&mut l, &anna(), Action::Skip { round: None });
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
        agreed(&mut l, &anna(), Action::Simulate { round: None });
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
        agreed(&mut l, &anna(), Action::Simulate { round: None });
    }
    l.players[0].recognition = 5.0;
    agreed(&mut l, &anna(), Action::Simulate { round: None });
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
    agreed(&mut l, &anna(), Action::Simulate { round: None });
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
    agreed(&mut l, &anna(), Action::Simulate { round: None });
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

// ---------------------------------------------------------------------------------------------
// The career contract: old documents, new actions, leaving players

#[test]
fn old_documents_still_load() {
    // A 2016 to 2017 career saved before the career contract existed: 2016 played through with
    // practice, qualifying, a reported race, an upgrade and offers, then three rounds of 2017.
    let text = include_str!("fixtures/league-v1.json");
    let mut l: League = serde_json::from_str(text).unwrap();
    assert_eq!((l.year(), l.phase), (2017, Phase::Racing));
    assert!(!l.seasons[0].offers.is_empty());
    assert!(!l.seasons[0].upgrades.is_empty() && !l.seasons[1].upgrades.is_empty());
    assert_eq!(l.seasons[0].offers[0].attempts, 3);
    assert_eq!(l.options, Options::default());
    assert!(l.news.iter().all(|n| n.id == 0 && n.kind == NewsKind::News));
    // It round-trips as it loaded.
    let back: League = serde_json::from_str(&serde_json::to_string(&l).unwrap()).unwrap();
    assert_eq!(back, l);
    agreed(&mut l, &anna(), Action::Simulate { round: None });
    assert_eq!(l.season().next_round().unwrap().round, 5);
}

#[test]
fn new_actions_round_trip_as_json() {
    let all = [
        Action::Ready { ready: true },
        Action::Vote {
            yes: false,
            force: true,
        },
        Action::RemovePlayer {
            player: "u-ben".into(),
        },
        Action::SetOptions {
            options: Options {
                rules: WeekendRules {
                    ai_difficulty: Some(90),
                    quali: Some(QualiFormat::OneShot),
                    ..WeekendRules::default()
                },
                game_identity: true,
                ..Options::default()
            },
        },
        Action::Profile {
            nationality: Some("DEU".into()),
            archetype: Some(Archetype::Veteran),
        },
        Action::Meet { slot: 3 },
        Action::AcceptApproach {
            slot: 3,
            replaces: Some("max_verstappen".into()),
        },
        Action::DeclineApproach { slot: 4 },
        Action::Negotiate {
            slot: 3,
            terms: Terms {
                salary_k: 2_000,
                years: 2,
                number_one: true,
            },
        },
        Action::Answer { news: 12, choice: 1 },
        Action::ReadMail { upto: 40 },
        Action::DevelopNode {
            part: Part::Aero,
            node: 2,
        },
    ];
    for a in all {
        let text = serde_json::to_string(&a).unwrap();
        let back: Action = serde_json::from_str(&text).unwrap();
        assert_eq!(back, a, "{text}");
    }
    // Defaults an older or shorter request leaves out.
    let a: Action = serde_json::from_str(r#"{"op":"ready"}"#).unwrap();
    assert_eq!(a, Action::Ready { ready: true });
    let a: Action = serde_json::from_str(r#"{"op":"vote","yes":true}"#).unwrap();
    assert_eq!(
        a,
        Action::Vote {
            yes: true,
            force: false
        }
    );
    let a: Action = serde_json::from_str(r#"{"op":"set_options","options":{}}"#).unwrap();
    assert_eq!(
        a,
        Action::SetOptions {
            options: Options::default()
        }
    );
    // A refusal carries a reason old launchers show as text.
    let mut l = ferrari_2016(Setup::default());
    let err = apply(&mut l, &ben(), Action::Meet { slot: 1 }).unwrap_err();
    assert_eq!(err.reason, "no_approach");
    assert!(!err.message.is_empty());
}

#[test]
fn removing_a_player_in_setup_lets_the_owner_begin() {
    let mut l = League::start(Setup::default(), &anna(), Some(7), 42).unwrap();
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
    assert!(l.season().player_entry("u-ben").is_some());
    // Only the owner removes someone else, and the owner can't leave.
    let cleo = Actor {
        id: "u-cleo".into(),
        name: "Cleo".into(),
        owner: false,
    };
    apply(&mut l, &cleo, Action::Enter { number: None }).unwrap();
    let err = apply(
        &mut l,
        &cleo,
        Action::RemovePlayer {
            player: "u-ben".into(),
        },
    )
    .unwrap_err();
    assert_eq!(err.reason, "owner");
    let err = apply(
        &mut l,
        &anna(),
        Action::RemovePlayer {
            player: "u-anna".into(),
        },
    )
    .unwrap_err();
    assert_eq!(err.reason, "owner");
    // Cleo never picked a seat and leaves herself; the owner removes Ben.
    apply(
        &mut l,
        &cleo,
        Action::RemovePlayer {
            player: "u-cleo".into(),
        },
    )
    .unwrap();
    apply(
        &mut l,
        &anna(),
        Action::RemovePlayer {
            player: "u-ben".into(),
        },
    )
    .unwrap();
    assert_eq!(l.players.len(), 1);
    assert!(l.season().player_entry("u-ben").is_none());
    assert!(
        l.season().entry("sebastian_vettel").is_some(),
        "Vettel is back in his seat"
    );
    apply(&mut l, &anna(), Action::Begin).unwrap();
    assert_eq!(l.phase, Phase::Racing);
}

#[test]
fn a_player_who_leaves_mid_season_keeps_the_seat_and_needs_not_sign() {
    let setup = Setup {
        first_year: 2024,
        last_year: 2025,
        ..Setup::default()
    };
    let mut l = ferrari_2016(setup);
    agreed(&mut l, &anna(), Action::Simulate { round: None });
    apply(
        &mut l,
        &ben(),
        Action::RemovePlayer {
            player: "u-ben".into(),
        },
    )
    .unwrap();
    let b = player(&l, "u-ben");
    assert!(b.left && b.seat.is_some());
    assert_eq!(l.active_players().len(), 1);
    assert!(l.season().player_entry("u-ben").is_some(), "the seat stays");
    assert!(l.briefing("u-anna").unwrap().solo);
    assert!(l.news.iter().any(|n| n.text.contains("Ben left the career")));
    // A player who left acts only to come back.
    let err = apply(&mut l, &ben(), Action::Simulate { round: None }).unwrap_err();
    assert_eq!(err.reason, "left");
    while l.phase == Phase::Racing {
        agreed(&mut l, &anna(), Action::Simulate { round: None });
    }
    // Ben's results count, but he gets no offers; Anna's signature alone starts the next season.
    assert!(l.offers("u-ben").is_empty());
    let mine = l.offers("u-anna")[0].slot;
    apply(
        &mut l,
        &anna(),
        Action::Sign {
            slot: mine,
            replaces: None,
            focus: None,
        },
    )
    .unwrap();
    assert_eq!((l.year(), l.phase), (2025, Phase::Racing));
    assert!(player(&l, "u-ben").seat.is_none());
    assert!(l.season().player_entry("u-ben").is_none());
    assert_eq!(
        career_stats(&l, "u-ben").seasons.len(),
        1,
        "Ben keeps his history"
    );
}

#[test]
fn a_left_player_who_enters_again_is_back() {
    let mut l = ferrari_2016(Setup::default());
    apply(
        &mut l,
        &anna(),
        Action::RemovePlayer {
            player: "u-ben".into(),
        },
    )
    .unwrap();
    assert!(player(&l, "u-ben").left);
    apply(&mut l, &ben(), Action::Enter { number: None }).unwrap();
    assert!(!player(&l, "u-ben").left);
    assert!(l.news.iter().any(|n| n.text.contains("Ben is back")));
    apply(&mut l, &ben(), Action::Practice { round: None }).unwrap();
}

#[test]
fn the_owner_sets_the_career_options() {
    let mut l = ferrari_2016(Setup::default());
    let options = Options {
        game_identity: true,
        ..Options::default()
    };
    let err = apply(
        &mut l,
        &ben(),
        Action::SetOptions {
            options: options.clone(),
        },
    )
    .unwrap_err();
    assert_eq!(err.reason, "owner");
    apply(
        &mut l,
        &anna(),
        Action::SetOptions {
            options: options.clone(),
        },
    )
    .unwrap();
    assert_eq!(l.options, options);
    assert!(
        l.news
            .iter()
            .any(|n| n.text == "Anna changed the career options.")
    );
}

// ---------------------------------------------------------------------------------------------
// Integrity: each session counts once, in the right round and as the right kind

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

#[test]
fn the_same_race_from_two_players_counts_once() {
    // (a) The same game session from Ben's launcher.
    let mut l = ferrari_2016(Setup::default());
    let anna_race = report_of(l.season(), 0xa1, 0, "u-anna");
    report(&mut l, &anna(), anna_race.clone()).unwrap();
    let again = report(&mut l, &ben(), anna_race.clone()).unwrap();
    assert_eq!(again, ["this race is counted already"]);
    assert!(l.season().rounds[1].race_result.is_empty());
    assert_eq!(l.season().rounds[0].counted.len(), 1);
    assert_eq!(l.season().rounds[0].counted[0].by, "u-anna");

    // (b) Another session uid, the same classification (Ben's game numbered the session itself).
    let mut ben_race = anna_race.clone();
    ben_race.session = 0xb1;
    for c in &mut ben_race.cars {
        c.mine = c.name == "BEN";
        c.name = if c.mine { "Player".into() } else { c.name.clone() };
    }
    assert_eq!(fingerprint(&ben_race), fingerprint(&anna_race));
    let again = report(&mut l, &ben(), ben_race).unwrap();
    assert_eq!(again, ["this race is counted already"]);
    assert!(l.season().rounds[1].race_result.is_empty());

    // (c) A 2020 double header at the Red Bull Ring: Ben's copy of round 1 (another uid, other
    // times) arrives a minute after Anna's and isn't counted as round 2.
    let setup = Setup {
        first_year: 2020,
        ..Setup::default()
    };
    let mut l = ferrari_2016(setup);
    let mut anna_race = report_of(l.season(), 0xa2, 17, "u-anna");
    anna_race.at = 1_000;
    report(&mut l, &anna(), anna_race).unwrap();
    let mut ben_race = report_of(l.season(), 0xb2, 17, "u-ben");
    ben_race.at = 1_060;
    let err = report(&mut l, &ben(), ben_race.clone()).unwrap_err();
    assert_eq!(err.reason, "duplicate");
    assert!(err.message.contains("round 1") && err.message.contains("Anna"), "{}", err.message);
    assert!(l.season().rounds[1].race_result.is_empty());
    // (e) Counting it anyway counts it as round 2.
    apply(
        &mut l,
        &ben(),
        Action::Report {
            round: None,
            kind: None,
            force: true,
            report: ben_race,
        },
    )
    .unwrap();
    assert_eq!(l.season().rounds[1].state, RoundState::Raced);

    // (d) A sprint weekend: Ben's copy of the sprint (no weekend structure) arrives after Anna's
    // sprint was counted; it isn't counted as the Grand Prix.
    let setup = Setup {
        first_year: 2021,
        ..Setup::default()
    };
    let mut l = ferrari_2016(setup);
    while l.season().next_round().unwrap().round < 10 {
        agreed(&mut l, &anna(), Action::Simulate { round: None });
    }
    let mut sprint = report_of(l.season(), 0xa3, 7, "u-anna");
    sprint.at = 5_000;
    apply(
        &mut l,
        &anna(),
        Action::Report {
            round: None,
            kind: Some(SessionKind::Sprint),
            force: false,
            report: sprint,
        },
    )
    .unwrap();
    let mut ben_sprint = report_of(l.season(), 0xb3, 7, "u-ben");
    ben_sprint.at = 5_030;
    let err = report(&mut l, &ben(), ben_sprint).unwrap_err();
    assert_eq!(err.reason, "duplicate");
    assert!(err.message.contains("sprint"), "{}", err.message);
    let r = l.season().round(10).unwrap();
    assert!(r.race_result.is_empty() && !r.sprint_result.is_empty());
}

#[test]
fn qualifying_rewards_once() {
    let mut l = ferrari_2016(Setup::default());
    let mut q1 = report_of(l.season(), 0x51, 0, "u-anna");
    q1.session_type = 5;
    report(&mut l, &anna(), q1).unwrap();
    let rp = (player(&l, "u-anna").rp, player(&l, "u-ben").rp);
    assert!(rp.0 > 0 && rp.1 > 0);
    let pole = |l: &League| {
        l.season().rounds[0]
            .quali_result
            .iter()
            .find(|c| c.position == 1)
            .unwrap()
            .entry
            .clone()
    };
    assert_eq!(pole(&l), "p:u-anna");
    // Q3 (Ben on pole) replaces the classification without paying again.
    let mut q3 = report_of(l.season(), 0x53, 0, "u-ben");
    q3.session_type = 7;
    report(&mut l, &ben(), q3.clone()).unwrap();
    assert_eq!(pole(&l), "p:u-ben");
    assert_eq!((player(&l, "u-anna").rp, player(&l, "u-ben").rp), rp);
    assert!(l.news.iter().any(|n| n.text.starts_with("Qualifying updated (Q3).")));
    // Anna's copy of Q3 is the same classification: counted already.
    let mut copy = q3.clone();
    copy.session = 0x63;
    let again = report(&mut l, &anna(), copy).unwrap();
    assert_eq!(again, ["this qualifying is counted already"]);
    // An earlier part after a later one isn't counted over it.
    let mut q2 = report_of(l.season(), 0x52, 0, "u-anna");
    q2.session_type = 6;
    let again = report(&mut l, &anna(), q2).unwrap();
    assert_eq!(again, ["a later part of this qualifying is counted already"]);
    assert_eq!(pole(&l), "p:u-ben");
}

#[test]
fn sprint_or_race_follows_the_weekend() {
    let setup = Setup {
        first_year: 2021,
        ..Setup::default()
    };
    let mut base = ferrari_2016(setup);
    while base.season().next_round().unwrap().round < 10 {
        agreed(&mut base, &anna(), Action::Simulate { round: None });
    }
    let race_of = |weekend: &[u8], session_type: u8| {
        let mut r = report_of(base.season(), 0x77, 7, "u-anna");
        r.session_type = session_type;
        r.weekend = weekend.to_vec();
        r
    };
    // 15 on a sprint weekend is the sprint.
    let mut l = base.clone();
    report(&mut l, &anna(), race_of(&[1, 10, 11, 12, 15, 16], 15)).unwrap();
    let r = l.season().round(10).unwrap();
    assert!(!r.sprint_result.is_empty() && r.state == RoundState::Pending);
    // 15 on a weekend without a second race is the Grand Prix.
    let mut l = base.clone();
    report(&mut l, &anna(), race_of(&[1, 2, 3, 5, 6, 7, 15], 15)).unwrap();
    let r = l.season().round(10).unwrap();
    assert!(r.sprint_result.is_empty() && r.state == RoundState::Raced);
    // 15 without a weekend structure on an open sprint: the player is asked.
    let mut l = base.clone();
    let err = report(&mut l, &anna(), race_of(&[], 15)).unwrap_err();
    assert_eq!(err.reason, "ask_kind");
    // 16 is always the race.
    let mut l = base.clone();
    report(&mut l, &anna(), race_of(&[], 16)).unwrap();
    assert_eq!(l.season().round(10).unwrap().state, RoundState::Raced);
}

#[test]
fn the_weekend_structure_and_settings_read_from_the_session_packet() {
    // The game's own Grand Prix packet: no weekend structure, game mode 4.
    let s = udp::session(real_2026::QUALI_SESSION).unwrap();
    assert!(s.weekend().is_empty());
    assert_eq!(s.settings.unwrap().game_mode, 4);
    assert_eq!(s.total_laps, real_2026::QUALI_SESSION[udp::HEADER_LEN + 3]);
    // A weekend of five sessions.
    let mut body = session_body(7, 15);
    body[703] = 5;
    body[704..709].copy_from_slice(&[1, 10, 11, 15, 16]);
    body[640] = 95;
    let s = udp::session(&packet(1, 1, 0, &body)).unwrap();
    assert_eq!(s.weekend(), [1, 10, 11, 15, 16]);
    assert_eq!(s.settings.unwrap().ai_difficulty, 95);
    // A count the packet can't hold is no structure.
    body[703] = 13;
    assert!(udp::session(&packet(1, 1, 0, &body)).unwrap().weekend().is_empty());
}

#[test]
fn a_simulated_session_on_a_sprint_weekend_is_the_race_after_the_sprint() {
    let mut c = udp::Collector::new();
    let uid = 0x5151;
    let mut body = session_body(7, 15);
    body[703] = 6;
    body[704..710].copy_from_slice(&[1, 10, 11, 12, 15, 16]);
    c.feed(&packet(1, uid, 1, &body), 1);
    let cars = [(true, 44, "HAMILTON"), (false, 7, "Player"), (true, 5, "VETTEL")];
    c.feed(&packet(4, uid, 1, &participants_body(&cars)), 1);
    let sprint = c
        .feed(
            &packet(8, uid, 1, &final_body(&[(2, 3, 91_000), (1, 3, 90_500), (3, 3, 92_000)])),
            2,
        )
        .expect("the sprint");
    assert_eq!((sprint.session_type, sprint.weekend.len(), sprint.total_laps), (15, 6, 10));
    // The game simulates the Grand Prix: its classification comes without a session.
    let race = c
        .feed(
            &packet(8, 0, 1, &final_body(&[(3, 3, 91_100), (1, 3, 90_400), (2, 3, 92_100)])),
            3,
        )
        .expect("the race");
    assert_eq!(race.session_type, 16);
    assert_ne!(race.session, 0);
}

#[test]
fn only_the_next_round_can_be_counted() {
    let mut l = ferrari_2016(Setup::default());
    let three = Some(3);
    let bahrain = report_of(l.season(), 3, 2, "u-anna");
    let refused = [
        apply(&mut l, &anna(), Action::Simulate { round: three }),
        apply(&mut l, &anna(), Action::Skip { round: three }),
        apply(&mut l, &anna(), Action::Practice { round: three }),
        apply(
            &mut l,
            &anna(),
            Action::Report {
                round: three,
                kind: Some(SessionKind::Race),
                force: true,
                report: bahrain,
            },
        ),
    ];
    for r in refused {
        assert_eq!(r.unwrap_err().reason, "not_next");
    }
    agreed(&mut l, &anna(), Action::Simulate { round: Some(1) });
    assert_eq!(
        apply(&mut l, &anna(), Action::Skip { round: Some(1) })
            .unwrap_err()
            .reason,
        "round_done"
    );
}

#[test]
fn skip_after_qualifying_gives_back_its_resource_points() {
    let mut l = ferrari_2016(Setup::default());
    let earned = player(&l, "u-anna").earnings_k;
    apply(&mut l, &anna(), Action::Practice { round: None }).unwrap();
    let mut q = report_of(l.season(), 0x55, 0, "u-anna");
    q.session_type = 7;
    report(&mut l, &anna(), q).unwrap();
    assert!(player(&l, "u-anna").rp > 0);
    agreed(&mut l, &anna(), Action::Skip { round: None });
    let r = &l.season().rounds[0];
    assert_eq!(r.state, RoundState::Skipped);
    assert!(r.practice.is_empty() && r.quali_result.is_empty() && r.counted.is_empty());
    assert_eq!((player(&l, "u-anna").rp, player(&l, "u-ben").rp), (0, 0));
    // The salary is paid for the round all the same.
    assert!(player(&l, "u-anna").earnings_k > earned);
}

#[test]
fn undo_preview_names_what_goes() {
    let mut l = ferrari_2016(Setup::default());
    assert_eq!(undo_preview(&l), None);
    apply(&mut l, &anna(), Action::Practice { round: None }).unwrap();
    apply(&mut l, &ben(), Action::Practice { round: None }).unwrap();
    let mut q = report_of(l.season(), 0x56, 0, "u-anna");
    q.session_type = 7;
    report(&mut l, &anna(), q).unwrap();
    let p = undo_preview(&l).unwrap();
    assert_eq!((p.round, p.race_counted, p.quali, p.sprint), (1, false, true, false));
    assert_eq!(p.practice_of, ["Anna", "Ben"]);
    assert!(p.by_others.contains(&"Anna".to_owned()) && p.by_others.contains(&"Ben".to_owned()));
    assert!(!p.season_end);
    let setup = Setup {
        first_year: 2025,
        last_year: 2026,
        ..Setup::default()
    };
    let mut l = ferrari_2016(setup);
    while l.phase == Phase::Racing {
        agreed(&mut l, &anna(), Action::Simulate { round: None });
    }
    let p = undo_preview(&l).unwrap();
    assert!(p.season_end && p.race_counted);
    assert_eq!(p.round, 24);
}

#[test]
fn undo_in_finished_reopens_the_last_round() {
    let setup = Setup {
        first_year: 2026,
        last_year: 2026,
        ..Setup::default()
    };
    let mut l = ferrari_2016(setup);
    while l.phase == Phase::Racing {
        agreed(&mut l, &anna(), Action::Simulate { round: None });
    }
    assert_eq!(l.phase, Phase::Finished);
    let last = l.season().rounds.last().unwrap().round;
    agreed(&mut l, &anna(), Action::Undo);
    assert_eq!(l.phase, Phase::Racing);
    assert_eq!(l.season().next_round().unwrap().round, last);
}

#[test]
fn undo_at_season_end_clears_the_signed_terms() {
    let setup = Setup {
        first_year: 2025,
        last_year: 2026,
        ..Setup::default()
    };
    let mut l = ferrari_2016(setup);
    while l.phase == Phase::Racing {
        agreed(&mut l, &anna(), Action::Simulate { round: None });
    }
    let slot = l.offers("u-anna")[0].slot;
    apply(
        &mut l,
        &anna(),
        Action::Sign {
            slot,
            replaces: None,
            focus: Some(Focus::Development),
        },
    )
    .unwrap();
    assert!(player(&l, "u-anna").signed_salary_k > 0);
    agreed(&mut l, &anna(), Action::Undo);
    let a = player(&l, "u-anna");
    assert_eq!(
        (a.signed.clone(), a.signed_salary_k, a.signed_bonus_k, a.signed_focus),
        (None, 0, 0, None)
    );
}

/// Anna's race report with Ben absent: the game's AI drove Ben's seat as Sebastian Vettel (#5).
fn race_without_ben(l: &League) -> RaceReport {
    let mut r = report_of(l.season(), 0xab, 0, "u-anna");
    let car = r.cars.iter_mut().find(|c| c.name == "BEN").unwrap();
    car.name = "VETTEL".into();
    car.number = 5;
    car.ai = true;
    r
}

#[test]
fn an_absent_players_seat_driven_by_the_ai_counts_for_them() {
    let mut l = ferrari_2016(Setup::default());
    let pace = player(&l, "u-ben").attributes.unwrap().pace;
    let race = race_without_ben(&l);
    let warnings = report(&mut l, &anna(), race).unwrap();
    assert!(
        warnings.iter().any(|w| w == "Ben's seat was driven by the game's AI"),
        "{warnings:?}"
    );
    let c = l.season().rounds[0]
        .race_result
        .iter()
        .find(|c| c.entry == "p:u-ben")
        .unwrap()
        .clone();
    assert!(c.ai_driven && c.position == 2 && c.points == 18.0);
    assert_eq!(driver_standings(l.season())[1].name, "Ben");
    // The AI's drive teaches Ben nothing about pace.
    assert_eq!(player(&l, "u-ben").attributes.unwrap().pace, pace);
}

#[test]
fn a_guest_is_not_matched_to_an_absent_player_whose_seat_the_ai_drove() {
    let mut l = ferrari_2016(Setup::default());
    let mut r = race_without_ben(&l);
    let mut guest = r.cars[0].clone();
    guest.name = "CLEO".into();
    guest.number = 33;
    guest.mine = false;
    guest.ai = false;
    guest.position = 23;
    r.cars.push(guest);
    let warnings = report(&mut l, &anna(), r).unwrap();
    assert!(warnings.iter().any(|w| w.contains("CLEO")), "{warnings:?}");
    assert!(!warnings.iter().any(|w| w.contains("by elimination")), "{warnings:?}");
    let result = &l.season().rounds[0].race_result;
    assert!(result.iter().any(|c| c.entry == "p:u-ben" && c.ai_driven));
    assert_eq!(result.len(), 22);
}

fn classified(entry: &str, position: u8, status: Finish, points: f32) -> Classified {
    Classified {
        entry: entry.into(),
        position,
        grid: position,
        laps: 0,
        status,
        best_lap_ms: 0,
        time_ms: 0,
        penalty_s: 0,
        points,
        fastest: false,
        ai_driven: false,
    }
}

#[test]
fn ties_use_full_countback() {
    use Finish::*;
    // Drivers: Button and Hamilton both have a second place; Button's other race ended in a
    // retirement classified 15th, ahead of Hamilton's 16th.
    let mut l = ferrari_2016(Setup::default());
    let season = l.seasons.last_mut().unwrap();
    season.rounds[0].race_result = vec![
        classified("jenson_button", 2, Finished, 18.0),
        classified("lewis_hamilton", 16, Finished, 0.0),
    ];
    season.rounds[1].race_result = vec![
        classified("lewis_hamilton", 2, Finished, 18.0),
        classified("jenson_button", 15, Dnf, 0.0),
    ];
    let table = driver_standings(l.season());
    assert_eq!(
        (table[0].entry.as_str(), table[1].entry.as_str()),
        ("jenson_button", "lewis_hamilton")
    );
    // Constructors: McLaren (2nd and 8th) and Mercedes (4th and 5th) both have 22 points; the
    // second place puts McLaren ahead of the faster car.
    let mut l = ferrari_2016(Setup::default());
    l.seasons.last_mut().unwrap().rounds[0].race_result = vec![
        classified("daniel_ricciardo", 1, Finished, 25.0),
        classified("fernando_alonso", 2, Finished, 18.0),
        classified("max_verstappen", 3, Finished, 15.0),
        classified("lewis_hamilton", 4, Finished, 12.0),
        classified("nico_rosberg", 5, Finished, 10.0),
        classified("jenson_button", 8, Finished, 4.0),
    ];
    let teams: Vec<String> = team_standings(l.season())
        .into_iter()
        .take(3)
        .map(|t| t.name)
        .collect();
    assert_eq!(teams, ["Red Bull Racing", "McLaren", "Mercedes"]);
}

#[test]
fn a_damaged_solo_save_loads_from_history() {
    let root = std::env::temp_dir().join(format!("f1mp-solo-damaged-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let c = solo::create(&root, "Alone", "Robin", Setup::default(), Some(5)).unwrap();
    solo::act(
        &root,
        &c.id,
        Action::Seat {
            slot: 7,
            replaces: None,
        },
    )
    .unwrap();
    let (good, _) = solo::act(&root, &c.id, Action::Begin).unwrap();
    let (_, _) = solo::act(&root, &c.id, Action::Simulate { round: None }).unwrap();
    // A crash left the newest save half written.
    let file = solo::dir(&root).join(format!("{}.json", c.id));
    std::fs::write(&file, b"{\"id\":").unwrap();
    let loaded = solo::load(&root, &c.id).unwrap();
    assert_eq!((loaded.version, loaded.league.clone()), (good.version, good.league));
    // The next change is saved over it.
    let (after, _) = solo::act(&root, &c.id, Action::Simulate { round: None }).unwrap();
    assert_eq!(after.version, good.version + 1);
    assert_eq!(solo::load(&root, &c.id).unwrap().version, after.version);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_full_2016_to_2026_career_rolls_over() {
    let mut l = ferrari_2016(Setup::default());
    let mut actions = 0;
    while l.phase != Phase::Finished {
        let action = match l.phase {
            Phase::Racing => Action::Simulate { round: None },
            _ => Action::NextSeason,
        };
        agreed(&mut l, &anna(), action);
        actions += 1;
        assert!(actions < 400, "the career doesn't end");
    }
    assert_eq!(l.seasons.len(), 11);
    assert_eq!(l.year(), LAST_YEAR);
    // Contracts may run past the career's last season; it ends all the same.
    for p in &l.players {
        assert!(p.contract_until.is_some_and(|y| y >= LAST_YEAR), "{p:?}");
        assert!(p.seat.is_some() && p.signed.is_none());
    }
    assert!(undo_preview(&l).unwrap().season_end);
}
