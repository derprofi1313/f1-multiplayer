//! Practice: the programmes a player drives in the game's practice sessions (or lets simulate),
//! graded from the game's telemetry, and the resource points they earn.

use super::progress::{Programme, ProgrammeResult, lap_text, mix};
use super::texts::{Text, t};
use super::{
    Classified, LapTelemetry, League, LeagueError, RaceReport, Season, SessionKind, classify, refuse,
    team_mate_at,
};

pub(super) fn grade(x: f32) -> u8 {
    x.clamp(0.0, 100.0).round() as u8
}

/// The practice programmes `report` (a practice session the game reported) shows for the car of
/// `entry`: laps for the track, the best lap against the team-mate's, and the long run.
pub(super) fn programmes_of(
    season: &Season,
    round: u8,
    entry: &str,
    result: &[Classified],
    report: &RaceReport,
) -> Option<(Vec<ProgrammeResult>, u8, u32)> {
    let mine = result.iter().find(|c| c.entry == entry)?;
    let laps = mine.laps.max(report.laps.len() as u8);
    if laps == 0 && mine.best_lap_ms == 0 {
        return None;
    }
    let mut out = Vec::new();
    // Track acclimatisation: clean laps (all laps when the game sent no lap list).
    let clean = if report.laps.is_empty() {
        u32::from(laps)
    } else {
        report.laps.iter().filter(|l| l.valid && l.ms > 0).count() as u32
    };
    out.push(ProgrammeResult {
        programme: Programme::TrackAcclimatisation,
        grade: grade(clean as f32 * 12.5),
    });
    // Qualifying pace: against the team-mate, else against the fastest car.
    if mine.best_lap_ms > 0 {
        let mate = team_mate_at(season, entry, round)
            .and_then(|m| result.iter().find(|c| c.entry == m.id))
            .map(|c| c.best_lap_ms)
            .filter(|&ms| ms > 0);
        let best = result
            .iter()
            .map(|c| c.best_lap_ms)
            .filter(|&ms| ms > 0)
            .min()
            .unwrap_or(mine.best_lap_ms);
        let g = match mate {
            Some(m) => 75.0 - (mine.best_lap_ms as f32 / m as f32 - 1.0) * 100.0 * 50.0,
            None => 100.0 - (mine.best_lap_ms as f32 / best as f32 - 1.0) * 100.0 * 25.0,
        };
        out.push(ProgrammeResult {
            programme: Programme::QualifyingPace,
            grade: grade(g),
        });
    }
    // Tyre management: four or more clean laps within 107% of the best.
    let best_lap = report
        .laps
        .iter()
        .filter(|l| l.valid && l.ms > 0)
        .map(|l| l.ms)
        .min();
    if let Some(best_lap) = best_lap {
        let run: Vec<u32> = report
            .laps
            .iter()
            .filter(|l| l.valid && l.ms > 0 && (l.ms as f32) <= best_lap as f32 * 1.07)
            .map(|l| l.ms)
            .collect();
        if run.len() >= 4 {
            let mean = run.iter().map(|&ms| ms as f32).sum::<f32>() / run.len() as f32;
            out.push(ProgrammeResult {
                programme: Programme::TyreManagement,
                grade: grade(100.0 - (mean / best_lap as f32 - 1.0) * 100.0 * 30.0),
            });
        }
    }
    out.extend(telemetry_programmes(report));
    Some((out, laps, mine.best_lap_ms))
}

/// The ERS a lap may deploy from the store, MJ.
pub(super) const ERS_PER_LAP_MJ: f32 = 4.0;
/// The ERS store a lap must keep, MJ (10% of the 4 MJ store).
pub(super) const ERS_RESERVE_MJ: f32 = 0.4;
/// The fuel target: kg per km of track (about 5% below a race's normal use).
pub(super) const FUEL_KG_PER_KM: f32 = 0.32;
/// The tyre wear target of a long run: percent per km of track (about 2% a lap at 5.3 km).
pub(super) const WEAR_PCT_PER_KM: f32 = 0.38;
/// Laps a fuel or ERS programme needs, and a race-strategy run.
const PROGRAMME_LAPS: usize = 3;
const STRATEGY_RUN: usize = 5;
/// A lap counts for the programmes within this of the best clean lap (no saving by crawling).
const PACE_WINDOW: f32 = 1.07;

/// The length of the game's track `track` in km (F1 25 track ids; the short layouts aren't known).
pub(super) fn track_km(track: u8) -> Option<f32> {
    Some(match track {
        0 => 5.278,
        1 => 5.842,
        2 => 5.451,
        3 => 5.412,
        4 => 4.657,
        5 => 3.337,
        6 => 4.361,
        7 | 39 => 5.891,
        8 => 4.574,
        9 => 4.381,
        10 => 7.004,
        11 => 5.793,
        12 => 4.940,
        13 => 5.807,
        14 => 5.281,
        15 => 5.513,
        16 => 4.309,
        17 | 40 => 4.318,
        18 => 5.848,
        19 => 4.304,
        20 => 6.003,
        21 => 3.543,
        25 => 5.607,
        26 | 41 => 4.259,
        27 => 4.909,
        28 => 4.653,
        29 => 6.174,
        30 => 5.412,
        31 => 6.201,
        32 => 5.419,
        _ => return None,
    })
}

/// The ERS, fuel and race-strategy programmes from the laps' telemetry. A programme the
/// telemetry can't show (an older launcher sent none, or too few laps) isn't graded.
pub(super) fn telemetry_programmes(report: &RaceReport) -> Vec<ProgrammeResult> {
    let mut out = Vec::new();
    let clean = |l: &&LapTelemetry| l.valid && l.ms > 0;
    let Some(best) = report.lap_telemetry.iter().filter(clean).map(|l| l.ms).min() else {
        return out;
    };
    let at_pace: Vec<&LapTelemetry> = report
        .lap_telemetry
        .iter()
        .filter(clean)
        .filter(|l| l.ms as f32 <= best as f32 * PACE_WINDOW)
        .collect();
    // The track's length, else what the best lap covers at an average 210 km/h.
    let km = report
        .track
        .and_then(track_km)
        .unwrap_or(best as f32 / 1000.0 * 0.0585);
    let mean = |xs: &[f32]| xs.iter().sum::<f32>() / xs.len() as f32;

    // ERS: deploy the full 4 MJ each lap with the store above 10% at the line.
    let ers: Vec<f32> = at_pace
        .iter()
        .filter(|l| l.ers_deployed_mj > 0.0 || l.ers_store_end_mj > 0.0)
        .map(|l| {
            let close = 1.0 - (l.ers_deployed_mj - ERS_PER_LAP_MJ).abs() / ERS_PER_LAP_MJ;
            let kept = if l.ers_store_end_mj >= ERS_RESERVE_MJ {
                1.0
            } else {
                0.5
            };
            close.clamp(0.0, 1.0) * kept
        })
        .collect();
    if ers.len() >= PROGRAMME_LAPS {
        out.push(ProgrammeResult {
            programme: Programme::ErsManagement,
            grade: grade(mean(&ers) * 100.0),
        });
    }

    // Fuel: laps at pace within the target; a lap 15% over it scores nothing.
    let target = km * FUEL_KG_PER_KM;
    let fuel: Vec<f32> = at_pace
        .iter()
        .filter(|l| l.fuel_used_kg > 0.0)
        .map(|l| (1.0 - (l.fuel_used_kg / target - 1.0).max(0.0) / 0.15).clamp(0.0, 1.0))
        .collect();
    if fuel.len() >= PROGRAMME_LAPS {
        out.push(ProgrammeResult {
            programme: Programme::FuelManagement,
            grade: grade(mean(&fuel) * 100.0),
        });
    }

    // Race strategy: the longest run of consecutive laps at pace with their tyre wear, at least
    // five; 80% of the target wear per lap or less is full marks, 180% nothing.
    let mut runs: Vec<Vec<f32>> = Vec::new();
    let mut last = None;
    for l in at_pace.iter().filter(|l| l.tyre_wear_pct > 0.0) {
        match runs.last_mut() {
            Some(run) if last.is_some_and(|n: u8| l.lap == n.wrapping_add(1)) => run.push(l.tyre_wear_pct),
            _ => runs.push(vec![l.tyre_wear_pct]),
        }
        last = Some(l.lap);
    }
    if let Some(run) = runs
        .into_iter()
        .max_by_key(Vec::len)
        .filter(|r| r.len() >= STRATEGY_RUN)
    {
        let ratio = mean(&run) / (km * WEAR_PCT_PER_KM);
        out.push(ProgrammeResult {
            programme: Programme::RaceStrategy,
            grade: grade(100.0 - (ratio - 0.8).max(0.0) * 100.0),
        });
    }
    out
}

pub(super) fn average(programmes: &[ProgrammeResult]) -> u8 {
    if programmes.is_empty() {
        return 0;
    }
    (programmes.iter().map(|p| u32::from(p.grade)).sum::<u32>() / programmes.len() as u32) as u8
}

/// Resource points per grade point of a programme: a full set of six programmes at 100 earns 1.3
/// times what the three programmes earned at most before the telemetry ones (50 + 300 × 1.2).
pub(super) const RP_PER_GRADE: f32 = (1.3 * (50.0 + 300.0 * 1.2) - 50.0) / 600.0;

pub(super) fn practice_rp(programmes: &[ProgrammeResult]) -> f32 {
    50.0 + programmes.iter().map(|p| f32::from(p.grade)).sum::<f32>() * RP_PER_GRADE
}

impl League {
    /// Count `player`'s practice of round `n`: from the game's report of a practice session, or
    /// simulated. Returns news lines.
    pub(super) fn count_practice(
        &mut self,
        player: &str,
        n: u8,
        report: Option<&RaceReport>,
    ) -> Result<Vec<Text>, LeagueError> {
        let season = self.season().clone();
        let Some(entry) = season.player_entry(player).cloned() else {
            return refuse("no_seat", t!("you have no seat this season"));
        };
        let round = season.round(n).expect("pending");
        if round.practice.iter().any(|l| l.player == player) {
            return refuse("round_done", t!("your practice of round {n} is counted already", n));
        }
        let name = self.player(player).map_or_else(String::new, |p| p.name.clone());
        let line = match report {
            Some(report) => {
                let (result, _) = classify(
                    &season,
                    &self.players,
                    Some(player),
                    report,
                    SessionKind::Practice,
                    season.year,
                    self.setup.points,
                    n,
                );
                let Some((programmes, laps, best)) = programmes_of(&season, n, &entry.id, &result, report)
                else {
                    return refuse("no_laps", t!("you drove no lap in this practice session"));
                };
                let rp = practice_rp(&programmes);
                super::progress::PracticeLine {
                    player: player.to_owned(),
                    driven: true,
                    grade: average(&programmes),
                    rp: rp.round() as u32,
                    programmes,
                    laps,
                    best_lap_ms: best,
                    session: report.session,
                }
            }
            None => {
                let rating = self.player(player).map_or(super::START_RATING, |p| p.rating);
                let seed = mix(&[
                    self.seed,
                    u64::from(season.year),
                    u64::from(n),
                    player.len() as u64,
                ]) ^ player.bytes().fold(0u64, |h, b| h.rotate_left(5) ^ u64::from(b));
                let programmes: Vec<ProgrammeResult> = [
                    Programme::TrackAcclimatisation,
                    Programme::QualifyingPace,
                    Programme::TyreManagement,
                ]
                .into_iter()
                .enumerate()
                .map(|(i, programme)| {
                    let luck = (mix(&[seed, i as u64]) >> 40) as f32 / (1u64 << 24) as f32;
                    ProgrammeResult {
                        programme,
                        grade: grade(45.0 + (rating - 70.0) * 1.2 + (luck - 0.5) * 30.0),
                    }
                })
                .collect();
                // Like the game, a simulated programme earns less than a driven one.
                let rp = practice_rp(&programmes) * 0.6;
                super::progress::PracticeLine {
                    player: player.to_owned(),
                    driven: false,
                    grade: average(&programmes),
                    rp: rp.round() as u32,
                    programmes,
                    laps: 0,
                    best_lap_ms: 0,
                    session: 0,
                }
            }
        };
        self.keep_progress(n);
        let mut lines = Vec::new();
        let rp = self.earn(player, line.rp as f32);
        let driven = if line.driven { 1.0 } else { 0.5 };
        // Only a practice driven shows the driver's pace and long runs.
        let skill = if line.driven { 1.0 } else { 0.0 };
        let of = |p: Programme| {
            line.programmes
                .iter()
                .find(|x| x.programme == p)
                .map(|x| f32::from(x.grade))
        };
        let (quali, tyres, done) = (
            of(Programme::QualifyingPace),
            of(Programme::TyreManagement),
            line.programmes.len() as f32,
        );
        self.train(player, |a| {
            a.experience += 0.1 * done * driven;
            if let Some(g) = quali {
                a.pace += (g - 60.0) / 250.0 * skill;
            }
            if let Some(g) = tyres {
                a.racecraft += (g - 60.0) / 300.0 * skill;
            }
        });
        if line.grade >= 75 {
            self.recognise(player, 1.0);
        } else if line.grade < 35 {
            self.recognise(player, -1.0);
        }
        self.sync_ratings();
        lines.extend(self.update_status());
        let (race, grade) = (&round.race, line.grade);
        let text = match (line.driven, line.best_lap_ms) {
            (true, 0) => t!("Round {n}, {race} practice: {name} grade {grade}, {rp} resource points.", n, race, name, grade, rp),
            (true, best) => t!(
                "Round {n}, {race} practice: {name} grade {grade}, {rp} resource points, best lap {lap}.",
                n,
                race,
                name,
                grade,
                rp,
                lap = lap_text(best)
            ),
            (false, 0) => t!(
                "Round {n}, {race} practice (simulated): {name} grade {grade}, {rp} resource points.",
                n,
                race,
                name,
                grade,
                rp
            ),
            (false, best) => t!(
                "Round {n}, {race} practice (simulated): {name} grade {grade}, {rp} resource points, best lap {lap}.",
                n,
                race,
                name,
                grade,
                rp,
                lap = lap_text(best)
            ),
        };
        lines.insert(0, text);
        let line = super::progress::PracticeLine { rp, ..line };
        let grade = line.grade;
        let season = self.season_mut();
        let round = season.rounds.iter_mut().find(|r| r.round == n).expect("pending");
        round.practice.push(line);
        super::objectives::grade_practice(self, n, player, grade);
        Ok(lines)
    }
}
