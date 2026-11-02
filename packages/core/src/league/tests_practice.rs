//! Tests of the league's practice (W2-7): the practice programmes graded from the game's
//! telemetry (lap data, car status and car damage packets, laid out as EA's F1 25 UDP
//! specification gives them, in format 2025 with 22 cars and format 2026 with 24).

use super::practice::{
    ERS_PER_LAP_MJ, FUEL_KG_PER_KM, RP_PER_GRADE, WEAR_PCT_PER_KM, practice_rp, telemetry_programmes,
    track_km,
};
use super::progress::{Programme, ProgrammeResult};
use super::*;

fn anna() -> Actor {
    Actor {
        id: "u-anna".into(),
        name: "Anna".into(),
        owner: true,
    }
}

/// A 2016 league of Anna's alone, at Ferrari (round 1 at Melbourne, game track 0).
fn anna_at_ferrari() -> League {
    let mut l = League::start(Setup::default(), &anna(), Some(7), 42).unwrap();
    apply(
        &mut l,
        &anna(),
        Action::Seat {
            slot: 2,
            replaces: None,
        },
    )
    .unwrap();
    apply(&mut l, &anna(), Action::Begin).unwrap();
    l
}

fn report(l: &mut League, report: RaceReport) -> Vec<String> {
    apply(
        l,
        &anna(),
        Action::Report {
            round: None,
            kind: None,
            force: false,
            report,
        },
    )
    .unwrap()
}

fn practice_line(l: &League) -> progress::PracticeLine {
    l.season().rounds[0].practice[0].clone()
}

fn kinds(programmes: &[ProgrammeResult]) -> Vec<Programme> {
    programmes.iter().map(|p| p.programme).collect()
}

fn grade_of(programmes: &[ProgrammeResult], p: Programme) -> Option<u8> {
    programmes.iter().find(|x| x.programme == p).map(|x| x.grade)
}

// ---------------------------------------------------------------------------------------------
// The game's packets

/// One lap of the player's as the game drives it: its time, the fuel it burns (kg), the ERS it
/// deploys and the store left at the line (MJ), the tyre wear it adds (percent) and whether it
/// stays within track limits.
#[derive(Clone, Copy)]
struct Drive {
    ms: u32,
    fuel: f32,
    ers: f32,
    store: f32,
    wear: f32,
    valid: bool,
}

const fn lap(ms: u32, fuel: f32, ers: f32, store: f32, wear: f32) -> Drive {
    Drive {
        ms,
        fuel,
        ers,
        store,
        wear,
        valid: true,
    }
}

struct Game {
    format: u16,
    cars: usize,
    session: u64,
    car: u8,
}

impl Game {
    fn packet(&self, id: u8, body: &[u8]) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&self.format.to_le_bytes());
        b.extend_from_slice(&[25, 1, 0, 1, id]);
        b.extend_from_slice(&self.session.to_le_bytes());
        b.extend_from_slice(&0f32.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&[self.car, 255]);
        assert_eq!(b.len(), udp::HEADER_LEN);
        b.extend_from_slice(body);
        b
    }

    /// The session packet: practice 1 at `track`.
    fn session(&self, track: u8) -> Vec<u8> {
        let mut b = vec![0u8; 724];
        b[3] = 10;
        b[6] = 1;
        b[7] = track;
        self.packet(1, &b)
    }

    /// Participants: the player's car named as the launcher names it, AI cars around it.
    fn participants(&self, team: u8, number: u8) -> Vec<u8> {
        let (size, team_at, number_at, name_at) = if self.format >= 2026 {
            (60, 5, 8, 10)
        } else {
            (57, 3, 5, 7)
        };
        let mut b = vec![self.cars as u8];
        for i in 0..self.cars {
            let mut r = vec![0u8; size];
            let mine = i == usize::from(self.car);
            r[0] = u8::from(!mine);
            r[team_at] = if mine { team } else { 9 };
            r[number_at] = if mine { number } else { 50 + i as u8 };
            let name: &[u8] = if mine { b"Player" } else { b"AI DRIVER" };
            r[name_at..name_at + name.len()].copy_from_slice(name);
            b.extend_from_slice(&r);
        }
        self.packet(4, &b)
    }

    /// Lap data (57-byte records, then two bytes): the player's lap number, last lap time,
    /// invalid flag and driver status (3 out lap, 4 on track).
    fn lap_data(&self, lap: u8, last_ms: u32, invalid: bool, driver: u8) -> Vec<u8> {
        let mut b = vec![0u8; self.cars * 57 + 2];
        let r = usize::from(self.car) * 57;
        b[r..r + 4].copy_from_slice(&last_ms.to_le_bytes());
        b[r + 33] = lap;
        b[r + 37] = u8::from(invalid);
        b[r + 44] = driver;
        // Another car's record, to make sure the player's is read.
        let other = (usize::from(self.car) + 1) % self.cars * 57;
        b[other + 33] = 99;
        self.packet(2, &b)
    }

    /// Car status (55-byte records): fuel in tank (kg), ERS store and ERS deployed this lap (J).
    fn status(&self, fuel: f32, store_j: f32, deployed_j: f32) -> Vec<u8> {
        let mut b = vec![0u8; self.cars * 55];
        let r = usize::from(self.car) * 55;
        b[r + 5..r + 9].copy_from_slice(&fuel.to_le_bytes());
        b[r + 9..r + 13].copy_from_slice(&110f32.to_le_bytes());
        b[r + 37..r + 41].copy_from_slice(&store_j.to_le_bytes());
        b[r + 50..r + 54].copy_from_slice(&deployed_j.to_le_bytes());
        self.packet(7, &b)
    }

    /// Car damage (46-byte records): the four tyres' wear in percent first.
    fn damage(&self, wear: [f32; 4]) -> Vec<u8> {
        let mut b = vec![0u8; self.cars * 46];
        let r = usize::from(self.car) * 46;
        for (i, w) in wear.iter().enumerate() {
            b[r + i * 4..r + i * 4 + 4].copy_from_slice(&w.to_le_bytes());
        }
        self.packet(10, &b)
    }

    /// The player's laps (session history, 14-byte lap records).
    fn history(&self, laps: &[(u32, bool)]) -> Vec<u8> {
        let mut b = vec![self.car, laps.len() as u8, 1, 1, 1, 1, 1];
        for i in 0..100 {
            let mut r = [0u8; 14];
            if let Some(&(ms, valid)) = laps.get(i) {
                r[..4].copy_from_slice(&ms.to_le_bytes());
                r[13] = u8::from(valid);
            }
            b.extend_from_slice(&r);
        }
        b.extend_from_slice(&[0u8; 24]);
        self.packet(11, &b)
    }

    fn session_end(&self) -> Vec<u8> {
        let mut b = vec![0u8; 16];
        b[..4].copy_from_slice(b"SEND");
        self.packet(3, &b)
    }

    /// A practice session at `track`: an out lap from the garage, then `laps`, each sent as the
    /// game does, a few times a lap; the session ends with its history and `SEND`.
    fn practice(&self, track: u8, team: u8, number: u8, laps: &[Drive]) -> RaceReport {
        let mut c = udp::Collector::new();
        c.feed(&self.session(track), 1);
        c.feed(&self.participants(team, number), 1);
        let (mut fuel, mut wear, mut store) = (30.0f32, 2.0f32, 4.0e6f32);
        // The out lap: a slow lap from the pit lane.
        let out = Drive {
            ms: 120_000,
            fuel: 1.2,
            ers: 0.5,
            store: 4.0,
            wear: 0.6,
            valid: true,
        };
        let all: Vec<Drive> = std::iter::once(out).chain(laps.iter().copied()).collect();
        let mut last_ms = 0;
        for (i, d) in all.iter().enumerate() {
            let n = i as u8 + 1;
            let driver = if i == 0 { 3 } else { 4 };
            let (fuel0, wear0, store0) = (fuel, wear, store);
            for t in 0..=4 {
                let f = t as f32 / 4.0;
                c.feed(&self.lap_data(n, last_ms, !d.valid && t > 1, driver), 2);
                fuel = fuel0 - d.fuel * f;
                wear = wear0 + d.wear * f;
                store = store0 + (d.store * 1e6 - store0) * f;
                c.feed(&self.status(fuel, store, d.ers * 1e6 * f), 2);
                // The front tyres wear a little more than the rears.
                c.feed(&self.damage([wear - 0.1, wear - 0.1, wear + 0.1, wear + 0.1]), 2);
            }
            last_ms = d.ms;
        }
        // Crossing the line after the last lap ends it.
        c.feed(&self.lap_data(all.len() as u8 + 1, last_ms, false, 4), 3);
        assert!(c.unread.is_none(), "every packet read: {:?}", c.unread);
        let history: Vec<(u32, bool)> = all.iter().map(|d| (d.ms, d.valid)).collect();
        c.feed(&self.history(&history), 3);
        c.feed(&self.session_end(), 4).expect("the practice")
    }
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

#[test]
fn car_status_and_damage_packets_make_lap_telemetry() {
    // The packet sizes of EA's specification: format 2025 sends 22 cars (lap data 1285 bytes, car
    // status 1239, car damage 1041); format 2026 sends 24 (its lap data 1399 bytes, as the game's
    // `Game_TelemetryUdp_Send2026` builds it).
    for (format, cars, car, lap_data_len) in [(2025u16, 22usize, 19u8, 1285usize), (2026, 24, 21, 1399)] {
        let game = Game {
            format,
            cars,
            session: 0x5150 + u64::from(format),
            car,
        };
        assert_eq!(game.lap_data(1, 0, false, 4).len(), lap_data_len);
        if format == 2025 {
            assert_eq!(game.status(1.0, 1.0, 1.0).len(), 1239);
            assert_eq!(game.damage([0.0; 4]).len(), 1041);
        }
        let s = udp::car_status(&game.status(20.5, 3.0e6, 1.5e6)).unwrap();
        assert_eq!((s.fuel_kg, s.ers_store_j, s.ers_deployed_j), (20.5, 3.0e6, 1.5e6));
        assert_eq!(
            udp::car_damage(&game.damage([1.0, 2.0, 3.0, 4.0])),
            Some([1.0, 2.0, 3.0, 4.0])
        );
        let l = udp::lap_data(&game.lap_data(3, 91_000, true, 4)).unwrap();
        assert_eq!((l.lap, l.last_lap_ms, l.invalid, l.pit), (3, 91_000, true, false));
        assert!(
            udp::lap_data(&game.lap_data(1, 0, false, 3)).unwrap().pit,
            "an out lap"
        );

        let mut off = lap(91_500, 1.9, 3.0, 0.2, 2.0);
        off.valid = false;
        let r = game.practice(
            0,
            2,
            7,
            &[
                lap(91_000, 1.6, 4.0, 1.0, 1.8),
                off,
                lap(90_500, 1.5, 3.8, 0.6, 1.9),
            ],
        );
        // The out lap began before the first packet (in the garage): it isn't a whole lap.
        let t = &r.lap_telemetry;
        assert_eq!(
            t.iter().map(|l| (l.lap, l.ms, l.valid)).collect::<Vec<_>>(),
            [
                (2, 91_000, true),
                (3, 91_500, false), // off the track
                (4, 90_500, true),
            ],
            "{format}"
        );
        assert!(
            close(t[0].fuel_used_kg, 1.6) && close(t[2].fuel_used_kg, 1.5),
            "{t:?}"
        );
        assert!(
            close(t[0].ers_deployed_mj, 4.0) && close(t[1].ers_deployed_mj, 3.0),
            "{t:?}"
        );
        assert!(
            close(t[0].ers_store_end_mj, 1.0) && close(t[1].ers_store_end_mj, 0.2),
            "{t:?}"
        );
        assert!(
            close(t[0].tyre_wear_pct, 1.8) && close(t[2].tyre_wear_pct, 1.9),
            "{t:?}"
        );
        // The session's laps and best lap come from its history, as before.
        assert_eq!(r.laps.len(), 4);
        assert_eq!(r.cars.iter().find(|c| c.mine).unwrap().best_lap_ms, 90_500);
        // The telemetry survives the launcher's saved report.
        let back: RaceReport = serde_json::from_str(&serde_json::to_string(&r).unwrap()).unwrap();
        assert_eq!(back.lap_telemetry, r.lap_telemetry);
    }

    // A flashback across the line drives the lap again; the lap telemetry follows.
    let game = Game {
        format: 2025,
        cars: 22,
        session: 0x99,
        car: 0,
    };
    let mut c = udp::Collector::new();
    c.feed(&game.session(3), 1);
    for (n, last, fuel) in [(1u8, 0u32, 20.0f32), (2, 95_000, 18.0), (3, 92_000, 16.0)] {
        c.feed(&game.lap_data(n, last, false, 4), 1);
        c.feed(&game.status(fuel, 2.0e6, 0.0), 1);
    }
    // Lap 1 began before the first packet, so only lap 2 (line to line) was known. Back into lap 2: its telemetry goes (where the lap began is gone with it), then over the
    // line again; lap 3 driven from the line counts.
    c.feed(&game.lap_data(2, 95_000, false, 4), 2);
    c.feed(&game.status(17.0, 2.0e6, 0.0), 2);
    c.feed(&game.lap_data(3, 91_000, false, 4), 2);
    c.feed(&game.status(16.5, 2.0e6, 0.0), 2);
    c.feed(&game.status(15.0, 2.0e6, 0.0), 2);
    c.feed(&game.lap_data(4, 90_000, false, 4), 2);
    c.feed(
        &game.history(&[(95_000, true), (91_000, true), (90_000, true)]),
        2,
    );
    let r = c.feed(&game.session_end(), 3).unwrap();
    assert_eq!(
        r.lap_telemetry
            .iter()
            .map(|l| (l.lap, l.ms, l.fuel_used_kg))
            .collect::<Vec<_>>(),
        [(3, 90_000, 2.0)]
    );
}

// ---------------------------------------------------------------------------------------------
// Grading

/// A practice report at Melbourne (5.278 km: fuel target 1.69 kg a lap, wear target 2.0%).
fn melbourne(laps: &[Drive]) -> RaceReport {
    RaceReport {
        session: 1,
        track: Some(0),
        session_type: 1,
        lap_telemetry: laps
            .iter()
            .enumerate()
            .map(|(i, d)| LapTelemetry {
                lap: i as u8 + 2,
                ms: d.ms,
                fuel_used_kg: d.fuel,
                ers_deployed_mj: d.ers,
                ers_store_end_mj: d.store,
                tyre_wear_pct: d.wear,
                valid: d.valid,
            })
            .collect(),
        ..RaceReport::default()
    }
}

#[test]
fn ers_fuel_and_strategy_are_graded() {
    let km = track_km(0).unwrap();
    assert!(close(km * FUEL_KG_PER_KM, 1.689) && close(km * WEAR_PCT_PER_KM, 2.006));
    // Six laps at pace: the full 4 MJ each with the store kept, under the fuel target, tyres
    // within the wear target.
    let good: Vec<Drive> = (0..6)
        .map(|i| lap(90_000 + i * 120, 1.62, ERS_PER_LAP_MJ, 1.2, 1.55))
        .collect();
    let p = telemetry_programmes(&melbourne(&good));
    assert_eq!(
        kinds(&p),
        [
            Programme::ErsManagement,
            Programme::FuelManagement,
            Programme::RaceStrategy
        ]
    );
    assert!(p.iter().all(|x| x.grade == 100), "{p:?}");

    // Half the ERS deployed or the battery run flat, 10% over the fuel target, twice the wear.
    let poor: Vec<Drive> = (0..6)
        .map(|i| {
            let (ers, store) = if i % 2 == 0 { (2.0, 1.0) } else { (4.0, 0.1) };
            lap(90_000 + i * 120, km * FUEL_KG_PER_KM * 1.1, ers, store, 4.0)
        })
        .collect();
    let p = telemetry_programmes(&melbourne(&poor));
    assert_eq!(grade_of(&p, Programme::ErsManagement), Some(50));
    assert_eq!(grade_of(&p, Programme::FuelManagement), Some(33));
    assert_eq!(grade_of(&p, Programme::RaceStrategy), Some(0));

    // Saving fuel or tyres by crawling doesn't count: laps beyond 107% of the best are left out,
    // and so are laps off the track.
    let mut mixed = good[..2].to_vec();
    mixed.push(lap(99_000, 1.0, 4.0, 1.0, 0.5));
    let mut off = lap(90_100, 1.0, 4.0, 1.0, 0.5);
    off.valid = false;
    mixed.push(off);
    let p = telemetry_programmes(&melbourne(&mixed));
    assert!(p.is_empty(), "two laps at pace grade nothing: {p:?}");

    // A race-strategy run needs five consecutive laps at pace; a break in it starts a new run.
    let mut broken = good.clone();
    broken[2].valid = false;
    let p = telemetry_programmes(&melbourne(&broken));
    assert_eq!(kinds(&p), [Programme::ErsManagement, Programme::FuelManagement]);

    // Without ERS or tyre data (a car damage or status packet the game didn't send), only what
    // the telemetry shows is graded.
    let fuel_only: Vec<Drive> = (0..4)
        .map(|i| lap(90_000 + i * 100, 1.7, 0.0, 0.0, 0.0))
        .collect();
    let p = telemetry_programmes(&melbourne(&fuel_only));
    assert_eq!(kinds(&p), [Programme::FuelManagement]);
    assert!(grade_of(&p, Programme::FuelManagement).unwrap() > 90);

    // An unknown track takes its length from the best lap (90 s at 210 km/h is 5.27 km).
    let mut unknown = melbourne(&good);
    unknown.track = Some(70);
    assert!(track_km(70).is_none());
    assert_eq!(
        telemetry_programmes(&unknown),
        telemetry_programmes(&melbourne(&good))
    );
}

#[test]
fn reports_without_telemetry_grade_three_programmes() {
    // A practice report as an older launcher sends it: laps, no lap telemetry.
    let old = r#"{"session": 77, "track": 0, "session_type": 1, "cars": [
        {"name": "Player", "number": 7, "team": 2, "ai": false, "mine": true, "position": 1,
         "grid": 0, "laps": 10, "status": 3, "best_lap_ms": 89000, "time_ms": 0, "penalty_s": 0}],
        "laps": [{"ms": 89000, "valid": true}, {"ms": 89200, "valid": true},
                 {"ms": 89300, "valid": true}, {"ms": 89400, "valid": true},
                 {"ms": 89500, "valid": true}, {"ms": 89600, "valid": true},
                 {"ms": 89700, "valid": true}, {"ms": 89800, "valid": true}]}"#;
    let old: RaceReport = serde_json::from_str(old).unwrap();
    assert!(old.lap_telemetry.is_empty());
    let mut l = anna_at_ferrari();
    report(&mut l, old);
    let line = practice_line(&l);
    assert!(line.driven);
    assert_eq!(
        kinds(&line.programmes),
        [
            Programme::TrackAcclimatisation,
            Programme::QualifyingPace,
            Programme::TyreManagement
        ]
    );
    assert_eq!(line.rp, practice_rp(&line.programmes).round() as u32);
    // The resource points per grade point: six programmes at 100 earn 1.3 times the old
    // three-programme maximum (410).
    assert!(close(50.0 + 600.0 * RP_PER_GRADE, 533.0));
    // A simulated practice keeps its three programmes.
    let mut l = anna_at_ferrari();
    apply(&mut l, &anna(), Action::Practice { round: None }).unwrap();
    assert_eq!(practice_line(&l).programmes.len(), 3);
    // Practice lines saved before the new programmes still load, and the new ones round-trip.
    let saved = r#"{"player": "u-anna", "driven": true, "grade": 80, "rp": 300,
        "programmes": [{"programme": "track_acclimatisation", "grade": 100},
                       {"programme": "ers_management", "grade": 70},
                       {"programme": "fuel_management", "grade": 60},
                       {"programme": "race_strategy", "grade": 50}]}"#;
    let saved: progress::PracticeLine = serde_json::from_str(saved).unwrap();
    assert_eq!(saved.programmes[3].programme, Programme::RaceStrategy);
    assert_eq!(Programme::ErsManagement.name(), "ERS management");
}

#[test]
fn a_simulated_practice_grades_all_six() {
    // Anna drives P1 at Melbourne in the game (format 2026, 24 cars): an out lap, then ten laps
    // close to her best, full ERS, fuel and tyres within their targets. The launcher's collector
    // makes the report from the game's packets; the league grades all six programmes.
    let game = Game {
        format: 2026,
        cars: 24,
        session: 0xabcd,
        car: 22,
    };
    let laps: Vec<Drive> = (0..10)
        .map(|i| lap(89_000 + i * 60, 1.6, 3.9, 0.9, 1.7))
        .collect();
    let r = game.practice(0, 2, 7, &laps);
    assert_eq!(r.lap_telemetry.len(), 10);
    let mut l = anna_at_ferrari();
    report(&mut l, r);
    let line = practice_line(&l);
    assert!(line.driven);
    assert_eq!(
        kinds(&line.programmes),
        [
            Programme::TrackAcclimatisation,
            Programme::QualifyingPace,
            Programme::TyreManagement,
            Programme::ErsManagement,
            Programme::FuelManagement,
            Programme::RaceStrategy,
        ],
        "{line:?}"
    );
    for p in [
        Programme::ErsManagement,
        Programme::FuelManagement,
        Programme::RaceStrategy,
    ] {
        assert!(grade_of(&line.programmes, p).unwrap() >= 90, "{p:?}: {line:?}");
    }
    assert_eq!(line.rp, practice_rp(&line.programmes).round() as u32);
    // More than the old three-programme maximum: the telemetry programmes pay.
    assert!(line.rp > 410, "{line:?}");
    // The practice objective (W2-2) pays its reward on top, as the grade reached its target.
    let objective: u32 = objectives::of(&l.season().rounds[0], "u-anna")
        .iter()
        .filter(|o| o.kind == ObjectiveKind::PracticeGrade && o.met == Some(true))
        .map(|o| o.reward_rp)
        .sum();
    assert_eq!(objective, 20, "{:?}", l.season().rounds[0].objectives);
    assert_eq!(l.player("u-anna").unwrap().rp, line.rp + objective);
    assert!(l.news.iter().any(|n| n.text.contains("practice: Anna grade")));
}
