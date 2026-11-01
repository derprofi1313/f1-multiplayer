//! Simulated rounds: for circuits the game doesn't have, and rounds the players don't drive.
//!
//! Each car's race comes from its team's pace that year, its driver's rating and luck. The luck is
//! seeded by the league's seed, the year, the round and the session, so the same round simulates
//! the same everywhere and after an undo.

use super::{Classified, Finish, PointsRule, Season, SessionKind, points};

/// splitmix64: small, fast and good enough for race luck.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Rng {
        Rng(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1).
    fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Standard normal (Box-Muller).
    fn normal(&mut self) -> f32 {
        let u = self.unit().max(1e-7);
        let v = self.unit();
        (-2.0 * u.ln()).sqrt() * (std::f32::consts::TAU * v).cos()
    }
}

/// How far behind the ideal a car is, in percent of a lap: the car's gap and what the driver loses.
pub(super) fn base(pace: f32, rating: f32) -> f32 {
    pace + (94.0 - rating).max(0.0) * 0.11
}

/// Simulate a session of `round`. A race or sprint starts from `grid` (a qualifying
/// classification) when given, else from a qualifying of its own.
pub(super) fn simulate(
    season: &Season,
    round: u8,
    seed: u64,
    kind: SessionKind,
    year: u16,
    rule: PointsRule,
    grid: Option<&[Classified]>,
) -> Vec<Classified> {
    let session = match kind {
        SessionKind::Race => 1u64,
        SessionKind::Sprint => 2,
        SessionKind::Qualifying => 3,
        SessionKind::Practice => 4,
    };
    let mut rng = Rng::new(seed ^ (u64::from(year) << 32) ^ (u64::from(round) << 8) ^ session);
    let dnf_chance = match kind {
        SessionKind::Race => 0.06,
        SessionKind::Sprint => 0.03,
        SessionKind::Qualifying | SessionKind::Practice => 0.0,
    };
    struct Car {
        index: usize,
        quali: f32,
        race: f32,
        out: bool,
    }
    // The cars of the round, each with its team's pace then.
    let entries: Vec<(&super::Entry, u8)> = season.lineup(round);
    let mut cars: Vec<Car> = entries
        .iter()
        .enumerate()
        .map(|(index, (e, slot))| {
            let b = base(season.pace_at(*slot, round), e.rating);
            Car {
                index,
                quali: b + rng.normal() * 0.25,
                race: b + rng.normal() * 0.45,
                out: rng.unit() < dnf_chance * season.breakdowns(*slot, round),
            }
        })
        .collect();
    let mut order: Vec<usize> = (0..cars.len()).collect();
    order.sort_by(|&a, &b| cars[a].quali.total_cmp(&cars[b].quali));
    if kind == SessionKind::Qualifying {
        return order
            .iter()
            .enumerate()
            .map(|(pos, &i)| Classified {
                entry: entries[i].0.id.clone(),
                position: pos as u8 + 1,
                grid: pos as u8 + 1,
                laps: 0,
                status: Finish::Finished,
                best_lap_ms: 0,
                time_ms: 0,
                penalty_s: 0,
                points: 0.0,
                fastest: false,
            })
            .collect();
    }
    // A given qualifying sets the grid; cars it doesn't list start behind, by this session's pace.
    if let Some(q) = grid.filter(|q| !q.is_empty()) {
        let place = |i: usize| {
            q.iter()
                .find(|c| c.entry == entries[i].0.id)
                .map_or(u8::MAX, |c| c.position)
        };
        order.sort_by_key(|&i| place(i));
    }
    let grid = order;
    let grid_of = |i: usize| grid.iter().position(|&g| g == i).map_or(0, |p| p as u8 + 1);
    // Starting further back costs a little, so the grid matters.
    for (pos, &i) in grid.iter().enumerate() {
        cars[i].race += pos as f32 * 0.02;
    }
    cars.sort_by(|a, b| a.out.cmp(&b.out).then(a.race.total_cmp(&b.race)));
    let finishers = cars.iter().filter(|c| !c.out).count();
    let fastest = cars
        .iter()
        .take(finishers.min(10))
        .map(|c| (c.index, c.race + rng.normal() * 0.5))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i);
    cars.iter()
        .enumerate()
        .map(|(pos, c)| {
            let position = pos as u8 + 1;
            let status = if c.out { Finish::Dnf } else { Finish::Finished };
            let is_fastest = kind == SessionKind::Race && fastest == Some(c.index);
            Classified {
                entry: entries[c.index].0.id.clone(),
                position,
                grid: grid_of(c.index),
                laps: 0,
                status,
                best_lap_ms: 0,
                time_ms: 0,
                penalty_s: 0,
                points: if c.out {
                    0.0
                } else {
                    points(year, rule, kind, position, is_fastest)
                },
                fastest: is_fastest,
            }
        })
        .collect()
}
