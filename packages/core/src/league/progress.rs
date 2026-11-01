//! The driver's side of a career, as in the game's: four ratings that move with every session,
//! the team's recognition, practice programmes, resource points that pay for the team's
//! development (R&D), and a salary with a bonus for each point.
//!
//! Everything a round changes for the players is kept before the round is counted
//! ([`Progress`]), so taking the round back restores it.

use serde::{Deserialize, Serialize};

use super::{
    Classified, Finish, League, LeagueError, Phase, RaceReport, Season, SessionKind, classify, refuse,
    team_goal_at, team_mate_at,
};

pub const ATTRIBUTE_MIN: f32 = 40.0;
pub const ATTRIBUTE_MAX: f32 = 99.0;
/// A player's recognition in a new career.
pub const RECOGNITION_START: f32 = 50.0;
/// A player's recognition at a team they moved to.
pub const RECOGNITION_NEW_TEAM: f32 = 45.0;
/// From this recognition on, a player is their team's number one driver: they earn more
/// resource points, and the team pays more to keep them.
pub const NUMBER_ONE: f32 = 65.0;
/// Below this, a team doesn't renew a contract that runs out.
pub const RECOGNITION_RENEW: f32 = 20.0;
/// Levels of each part a team can develop in a season.
pub const UPGRADE_LEVELS: u8 = 10;
/// Rounds a new part takes: ordered before round `n`, it is on the car from round
/// `n + UPGRADE_ROUNDS`.
pub const UPGRADE_ROUNDS: u8 = 2;

/// A driver's ratings, as the game rates them.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Attributes {
    pub experience: f32,
    pub racecraft: f32,
    pub awareness: f32,
    pub pace: f32,
}

impl Attributes {
    /// The overall rating, weighed as the game weighs it: pace twice, racecraft once, experience
    /// and awareness a quarter each.
    pub fn overall(&self) -> f32 {
        (0.25 * self.experience + self.racecraft + 0.25 * self.awareness + 2.0 * self.pace) / 3.5
    }

    /// Ratings with the overall `rating`: less experience, a little more awareness and pace.
    pub fn from_rating(rating: f32) -> Attributes {
        let at = |x: f32| x.clamp(ATTRIBUTE_MIN, ATTRIBUTE_MAX);
        Attributes {
            experience: at(rating - 18.0),
            racecraft: at(rating),
            awareness: at(rating + 2.0),
            pace: at(rating + 2.0),
        }
    }

    /// From the season data's `[experience, racecraft, awareness, pace]`.
    pub fn from_array(a: [f32; 4]) -> Attributes {
        Attributes {
            experience: a[0],
            racecraft: a[1],
            awareness: a[2],
            pace: a[3],
        }
    }

    fn clamp(&mut self) {
        for x in [
            &mut self.experience,
            &mut self.racecraft,
            &mut self.awareness,
            &mut self.pace,
        ] {
            *x = x.clamp(ATTRIBUTE_MIN, ATTRIBUTE_MAX);
        }
    }

    /// Move every rating by `by`, so the overall moves by `by`.
    pub(super) fn shift(&mut self, by: f32) {
        self.experience += by;
        self.racecraft += by;
        self.awareness += by;
        self.pace += by;
        self.clamp();
    }
}

/// A part of the car a team develops.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Part {
    Aero,
    Chassis,
    Powertrain,
    Durability,
}

impl Part {
    pub const ALL: [Part; 4] = [Part::Aero, Part::Chassis, Part::Powertrain, Part::Durability];

    pub fn name(self) -> &'static str {
        match self {
            Part::Aero => "Aerodynamics",
            Part::Chassis => "Chassis",
            Part::Powertrain => "Powertrain",
            Part::Durability => "Durability",
        }
    }

    /// The lap time a level gains, in percent; durability gains none but makes the car break
    /// down less often.
    pub fn gain(self) -> f32 {
        match self {
            Part::Aero => 0.07,
            Part::Chassis => 0.05,
            Part::Powertrain => 0.06,
            Part::Durability => 0.0,
        }
    }

    pub fn parse(s: &str) -> Option<Part> {
        match s.to_ascii_lowercase().as_str() {
            "aero" | "aerodynamics" => Some(Part::Aero),
            "chassis" => Some(Part::Chassis),
            "powertrain" | "engine" | "power" => Some(Part::Powertrain),
            "durability" | "reliability" => Some(Part::Durability),
            _ => None,
        }
    }
}

/// What the next level of a part costs, with `level` levels ordered this season.
pub fn upgrade_cost(level: u8) -> u32 {
    800 + 250 * u32::from(level)
}

/// A part a player ordered for their team.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Upgrade {
    pub slot: u8,
    pub part: Part,
    /// The level it brings the part to.
    pub level: u8,
    /// The player who paid for it.
    pub by: String,
    pub cost: u32,
    /// The round that was next when it was ordered, and whether that round had started.
    pub round: u8,
    #[serde(default)]
    pub started: bool,
    /// The first round it is on the car.
    pub ready: u8,
}

/// What a player asks for in a new contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Focus {
    /// A quarter more salary.
    Salary,
    /// The team listens more: a fifth more resource points.
    Development,
}

/// A practice programme, as the game's practice sessions offer them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Programme {
    /// Laps to learn the track: eight clean laps make it.
    TrackAcclimatisation,
    /// The best lap against the team-mate's.
    QualifyingPace,
    /// A long run: how close the laps stay to the best one.
    TyreManagement,
}

impl Programme {
    pub fn name(self) -> &'static str {
        match self {
            Programme::TrackAcclimatisation => "Track acclimatisation",
            Programme::QualifyingPace => "Qualifying pace",
            Programme::TyreManagement => "Tyre management",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgrammeResult {
    pub programme: Programme,
    /// 0 to 100.
    pub grade: u8,
}

/// A player's practice of a round.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PracticeLine {
    pub player: String,
    /// Driven in the game (from its telemetry), not simulated.
    pub driven: bool,
    pub programmes: Vec<ProgrammeResult>,
    /// The average grade, 0 to 100.
    pub grade: u8,
    /// Resource points earned.
    pub rp: u32,
    #[serde(default)]
    pub laps: u8,
    #[serde(default)]
    pub best_lap_ms: u32,
    /// The game session counted, 0 for a simulated practice.
    #[serde(default)]
    pub session: u64,
}

/// What a player had before a round was counted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Progress {
    pub player: String,
    pub rating: f32,
    pub attributes: Option<Attributes>,
    pub recognition: f32,
    pub rp: u32,
    pub earnings_k: u64,
    #[serde(default)]
    pub number_one: bool,
}

/// A season's salary and the bonus per point, in thousands of dollars, for a driver rated
/// `rating` with `titles` titles at the team with pace rank `rank` of `teams`; `recognition` for
/// a renewal (a team pays more to keep a driver it values).
pub fn salary(rating: f32, titles: usize, rank: usize, teams: usize, recognition: Option<f32>) -> (u32, u32) {
    let skill = ((rating - 65.0).max(0.0) / 10.0).powi(2);
    let mut millions = 0.3 + skill * 2.0;
    let spread = (teams.max(2) - 1) as f32;
    millions *= 1.35 - 0.7 * (rank.max(1) - 1) as f32 / spread;
    millions *= 1.0 + 0.1 * titles.min(4) as f32;
    if let Some(r) = recognition {
        millions *= 0.85 + r / 200.0;
    }
    let salary_k = ((millions * 1000.0 / 50.0).round() as u32 * 50).max(100);
    let bonus_k = ((salary_k as f32 / 100.0 / 5.0).round() as u32 * 5).max(5);
    (salary_k, bonus_k)
}

/// "$3.65M" or "$850k".
pub fn money(k: u64) -> String {
    if k >= 1000 {
        let m = format!("{:.2}", k as f64 / 1000.0);
        format!("${}M", m.trim_end_matches('0').trim_end_matches('.'))
    } else {
        format!("${k}k")
    }
}

/// splitmix64 over a few values, for the luck of a simulated practice.
fn mix(values: &[u64]) -> u64 {
    let mut z: u64 = 0x9e37_79b9_7f4a_7c15;
    for v in values {
        z ^= v.wrapping_add(0x9e37_79b9_7f4a_7c15);
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^= z >> 31;
    }
    z
}

fn grade(x: f32) -> u8 {
    x.clamp(0.0, 100.0).round() as u8
}

/// The practice programmes `report` (a practice session the game reported) shows for the car of
/// `entry`: laps for the track, the best lap against the team-mate's, and the long run.
fn programmes_of(
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
    Some((out, laps, mine.best_lap_ms))
}

/// Where the car of `entry` should finish in `round`, from its car's pace then and its driver's
/// rating.
pub fn expected_place(season: &Season, entry: &str, round: u8) -> u8 {
    let lineup = season.lineup(round);
    let strength = |e: &super::Entry, slot: u8| super::sim::base(season.pace_at(slot, round), e.rating);
    let Some(mine) = lineup
        .iter()
        .find(|(e, _)| e.id == entry)
        .map(|(e, s)| strength(e, *s))
    else {
        return lineup.len() as u8;
    };
    1 + lineup
        .iter()
        .filter(|(e, s)| e.id != entry && strength(e, *s) < mine)
        .count() as u8
}

fn average(programmes: &[ProgrammeResult]) -> u8 {
    if programmes.is_empty() {
        return 0;
    }
    (programmes.iter().map(|p| u32::from(p.grade)).sum::<u32>() / programmes.len() as u32) as u8
}

fn practice_rp(programmes: &[ProgrammeResult]) -> f32 {
    50.0 + programmes.iter().map(|p| f32::from(p.grade)).sum::<f32>() * 1.2
}

impl League {
    /// Keep what the players have before round `n` changes, once.
    pub(super) fn keep_progress(&mut self, n: u8) {
        let snapshot: Vec<Progress> = self
            .players
            .iter()
            .map(|p| Progress {
                player: p.id.clone(),
                rating: p.rating,
                attributes: p.attributes,
                recognition: p.recognition,
                rp: p.rp,
                earnings_k: p.earnings_k,
                number_one: p.number_one,
            })
            .collect();
        if let Some(r) = self.season_mut().rounds.iter_mut().find(|r| r.round == n)
            && r.before.is_none()
        {
            r.before = Some(snapshot);
        }
    }

    /// Give the players back what they had before a round, and drop the parts ordered after it
    /// (round `n` and later are taken back).
    pub(super) fn restore_progress(&mut self, snapshot: Vec<Progress>, n: u8) {
        for s in snapshot {
            if let Some(p) = self.player_mut(&s.player) {
                p.rating = s.rating;
                p.attributes = s.attributes;
                p.recognition = s.recognition;
                p.rp = s.rp;
                p.earnings_k = s.earnings_k;
                p.number_one = s.number_one;
            }
        }
        self.season_mut()
            .upgrades
            .retain(|u| u.round < n || (u.round == n && !u.started));
        self.sync_ratings();
    }

    /// Each player's overall rating from their ratings, and their car's rating with it.
    pub(super) fn sync_ratings(&mut self) {
        let mut ratings = Vec::new();
        for p in &mut self.players {
            if let Some(a) = p.attributes {
                p.rating = (a.overall() * 10.0).round() / 10.0;
            }
            ratings.push((p.id.clone(), p.rating));
        }
        let season = self.season_mut();
        for (id, rating) in ratings {
            if let Some(e) = season
                .entries
                .iter_mut()
                .find(|e| e.player.as_deref() == Some(&id))
            {
                e.rating = rating;
            }
        }
    }

    /// Resource points earned, with the number-one driver's and the contract's bonus.
    fn earn(&mut self, player: &str, rp: f32) -> u32 {
        let Some(p) = self.player_mut(player) else {
            return 0;
        };
        let mut rp = rp;
        if p.number_one {
            rp *= 1.1;
        }
        if p.focus == Some(Focus::Development) {
            rp *= 1.2;
        }
        let rp = rp.max(0.0).round() as u32;
        p.rp += rp;
        rp
    }

    /// Change a player's recognition.
    fn recognise(&mut self, player: &str, by: f32) {
        if let Some(p) = self.player_mut(player) {
            p.recognition = (p.recognition + by).clamp(0.0, 100.0);
        }
    }

    /// Each player's number-one status now, with news for those who became (or stopped being)
    /// their team's number one driver.
    pub(super) fn update_status(&mut self) -> Vec<String> {
        let now: Vec<bool> = self.players.iter().map(|p| number_one(self, &p.id)).collect();
        let season = self.season().clone();
        let mut lines = Vec::new();
        for (p, now) in self.players.iter_mut().zip(now) {
            if p.number_one == now {
                continue;
            }
            p.number_one = now;
            let Some(team) = season.player_entry(&p.id).and_then(|e| season.team(e.slot)) else {
                continue;
            };
            lines.push(if now {
                format!("{} is now {}'s number one driver.", p.name, team.name)
            } else {
                format!("{} is no longer {}'s number one driver.", p.name, team.name)
            });
        }
        lines
    }

    fn train(&mut self, player: &str, f: impl FnOnce(&mut Attributes)) {
        if let Some(p) = self.player_mut(player) {
            let a = p
                .attributes
                .get_or_insert_with(|| Attributes::from_rating(p.rating));
            f(a);
            a.clamp();
        }
    }

    /// What players earn and learn from the `kind` session of round `n`, just counted (`driven`
    /// in the game, else simulated). Returns news lines.
    pub(super) fn grow(&mut self, n: u8, kind: SessionKind, driven: bool) -> Vec<String> {
        let season = self.season().clone();
        let Some(round) = season.round(n) else {
            return Vec::new();
        };
        let list = match kind {
            SessionKind::Qualifying => &round.quali_result,
            SessionKind::Sprint => &round.sprint_result,
            SessionKind::Race => &round.race_result,
            SessionKind::Practice => return Vec::new(),
        };
        let mut lines = Vec::new();
        let ids: Vec<String> = self.players.iter().map(|p| p.id.clone()).collect();
        for id in ids {
            let Some(entry) = season.player_entry(&id) else {
                continue;
            };
            let Some(c) = list.iter().find(|c| c.entry == entry.id) else {
                continue;
            };
            let finished = c.status == Finish::Finished;
            let mate =
                team_mate_at(&season, &entry.id, n).and_then(|m| list.iter().find(|x| x.entry == m.id));
            // Ahead of the team-mate: a finished car beats one that didn't, else the position.
            let ahead = mate.map(|m| (!finished, c.position) < (m.status != Finish::Finished, m.position));
            // A simulated session teaches half the experience, and nothing about pace or
            // racecraft: its result comes from the ratings.
            let weight = if driven { 1.0 } else { 0.5 };
            let skill = if driven { 1.0 } else { 0.0 };
            // What the duel with the team-mate tells the team: driven, who was ahead; simulated, only
            // a result against the ratings.
            let mate_rating = team_mate_at(&season, &entry.id, n).map(|m| m.rating);
            let duel = |ahead: bool| -> f32 {
                match (driven, mate_rating) {
                    (true, _) => {
                        if ahead {
                            1.0
                        } else {
                            -1.0
                        }
                    }
                    (false, Some(r)) if ahead && entry.rating < r => 0.5,
                    (false, Some(r)) if !ahead && entry.rating > r => -0.5,
                    _ => 0.0,
                }
            };
            match kind {
                SessionKind::Qualifying => {
                    let mut rp = 60.0;
                    if c.position == 1 && finished {
                        rp += 80.0;
                    }
                    match ahead {
                        Some(true) => {
                            rp += 60.0;
                            self.train(&id, |a| a.pace += 0.2 * skill);
                            self.recognise(&id, duel(true));
                        }
                        Some(false) => {
                            self.train(&id, |a| a.pace -= 0.1 * skill);
                            self.recognise(&id, 0.5 * duel(false));
                        }
                        None => {}
                    }
                    self.train(&id, |a| a.experience += 0.15 * weight);
                    self.earn(&id, rp);
                }
                SessionKind::Sprint => {
                    let gained = f32::from(c.grid) - f32::from(c.position);
                    self.train(&id, |a| {
                        a.experience += 0.1 * weight;
                        if finished && c.grid > 0 {
                            a.racecraft += (gained * 0.03).clamp(-0.2, 0.2) * skill;
                        }
                    });
                    self.earn(&id, 30.0 + c.points * 5.0);
                }
                SessionKind::Race => {
                    let goal = team_goal_at(&season, entry.slot, n);
                    let mut rp = if finished { 100.0 + c.points * 5.0 } else { 0.0 };
                    if finished && c.position <= goal {
                        rp += 150.0;
                    }
                    if !driven {
                        // A simulated race tells the team only how the car and the driver's
                        // rating did against what they should have done.
                        if finished {
                            let expected = f32::from(expected_place(&season, &entry.id, n));
                            let by = ((expected - f32::from(c.position)) * 0.25).clamp(-1.0, 1.0);
                            self.recognise(&id, by);
                        }
                    } else if finished && c.position <= goal {
                        let by = 2.0 + (f32::from(goal - c.position) * 0.5).min(2.0);
                        self.recognise(&id, by);
                    } else if finished {
                        let by = 1.0 + (f32::from(c.position - goal) * 0.5).min(2.0);
                        self.recognise(&id, -by);
                    } else {
                        self.recognise(&id, -3.0);
                    }
                    match ahead {
                        Some(true) => {
                            rp += 80.0;
                            self.train(&id, |a| a.pace += 0.15 * skill);
                            self.recognise(&id, duel(true));
                        }
                        Some(false) => {
                            self.train(&id, |a| a.pace -= 0.1 * skill);
                            self.recognise(&id, duel(false));
                        }
                        None => {}
                    }
                    let gained = f32::from(c.grid) - f32::from(c.position);
                    let penalty = f32::from(c.penalty_s);
                    self.train(&id, |a| {
                        a.experience += 0.3 * weight;
                        if finished && c.grid > 0 {
                            a.racecraft += (gained * 0.05).clamp(-0.4, 0.4) * skill;
                        }
                        // Awareness comes from clean races driven, and goes with penalties and
                        // crashes (a simulated car that broke down isn't the driver's fault).
                        if driven {
                            if finished && penalty == 0.0 {
                                a.awareness += 0.2;
                            }
                            a.awareness -= (penalty * 0.05).min(0.6);
                            if !finished {
                                a.awareness -= 0.4;
                            }
                        }
                    });
                    self.earn(&id, rp);
                }
                SessionKind::Practice => {}
            }
        }
        self.sync_ratings();
        if kind == SessionKind::Race {
            self.pay(n, true);
        }
        lines.extend(self.update_status());
        lines
    }

    /// Pay the players their salary for round `n` (a share of the season's), and with `bonus`
    /// the bonus for the points they scored in it.
    pub(super) fn pay(&mut self, n: u8, bonus: bool) {
        let season = self.season().clone();
        let rounds = season.rounds.len().max(1) as u64;
        let points = |entry: &str| -> f32 {
            season.round(n).map_or(0.0, |r| {
                r.race_result
                    .iter()
                    .chain(&r.sprint_result)
                    .filter(|c| c.entry == entry)
                    .map(|c| c.points)
                    .sum()
            })
        };
        for p in &mut self.players {
            let Some(entry) = season.player_entry(&p.id) else {
                continue;
            };
            let mut k = u64::from(p.salary_k) / rounds;
            if bonus {
                k += (points(&entry.id) * p.bonus_k as f32).round() as u64;
            }
            p.earnings_k += k;
        }
    }

    /// Count `player`'s practice of round `n`: from the game's report of a practice session, or
    /// simulated. Returns news lines.
    pub(super) fn count_practice(
        &mut self,
        player: &str,
        n: u8,
        report: Option<&RaceReport>,
    ) -> Result<Vec<String>, LeagueError> {
        let season = self.season().clone();
        let Some(entry) = season.player_entry(player).cloned() else {
            return refuse("no_seat", "you have no seat this season");
        };
        let round = season.round(n).expect("pending");
        if round.practice.iter().any(|l| l.player == player) {
            return refuse(
                "round_done",
                format!("your practice of round {n} is counted already"),
            );
        }
        let name = self.player(player).map_or_else(String::new, |p| p.name.clone());
        let line = match report {
            Some(report) => {
                let (result, _) = classify(
                    &season,
                    Some(player),
                    report,
                    SessionKind::Practice,
                    season.year,
                    self.setup.points,
                    n,
                );
                let Some((programmes, laps, best)) = programmes_of(&season, n, &entry.id, &result, report)
                else {
                    return refuse("no_laps", "you drove no lap in this practice session");
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
        let what = if line.driven { "" } else { " (simulated)" };
        let mut text = format!(
            "Round {n}, {} practice{what}: {name} grade {}, {rp} resource points",
            round.race, line.grade
        );
        if line.best_lap_ms > 0 {
            text.push_str(&format!(", best lap {}", lap_text(line.best_lap_ms)));
        }
        text.push('.');
        lines.insert(0, text);
        let line = super::progress::PracticeLine { rp, ..line };
        let season = self.season_mut();
        let round = season.rounds.iter_mut().find(|r| r.round == n).expect("pending");
        round.practice.push(line);
        Ok(lines)
    }

    /// Order the next level of `part` for `player`'s team with their resource points.
    pub(super) fn develop(&mut self, player: &str, part: Part) -> Result<String, LeagueError> {
        if self.phase != Phase::Racing {
            return refuse("phase", "parts are developed while a season runs");
        }
        let season = self.season();
        let Some(entry) = season.player_entry(player) else {
            return refuse("no_seat", "you have no seat this season");
        };
        let slot = entry.slot;
        let Some(next) = season.next_round() else {
            return refuse("phase", "the season is over");
        };
        let (n, started) = (next.round, next.started());
        let ready = n + UPGRADE_ROUNDS;
        let last = season.rounds.last().map_or(0, |r| r.round);
        if ready > last {
            return refuse(
                "too_late",
                "a new part takes two rounds to build; it wouldn't be ready this season",
            );
        }
        let level = season.part_level(slot, part);
        if level >= UPGRADE_LEVELS {
            return refuse(
                "max_level",
                format!("{} is developed as far as it goes this season", part.name()),
            );
        }
        let cost = upgrade_cost(level);
        let have = self.player(player).map_or(0, |p| p.rp);
        if have < cost {
            return refuse(
                "no_points",
                format!(
                    "the next {} level costs {cost} resource points; you have {have}",
                    part.name()
                ),
            );
        }
        let team = season.team(slot).map_or_else(String::new, |t| t.name.clone());
        let race = season.round(ready).map_or_else(String::new, |r| r.race.clone());
        let name = self.player(player).map_or_else(String::new, |p| p.name.clone());
        self.player_mut(player).expect("seated").rp -= cost;
        self.season_mut().upgrades.push(Upgrade {
            slot,
            part,
            level: level + 1,
            by: player.to_owned(),
            cost,
            round: n,
            started,
            ready,
        });
        let text = format!(
            "{name} has {team} develop {} level {}; it's on the car from round {ready}, the {race}.",
            part.name(),
            level + 1
        );
        self.say(0, text.clone());
        Ok(text)
    }

    /// The pace each team carries into next season from what it developed this one: half of it,
    /// a sixth after the rules changed (2017, 2022, 2026).
    pub(super) fn carry_over(&self, next: u16) -> Vec<(u8, f32)> {
        let season = self.season();
        let keep = if matches!(next, 2017 | 2022 | 2026) {
            1.0 / 6.0
        } else {
            0.5
        };
        season
            .teams
            .iter()
            .map(|t| (t.slot, (t.carry - season.upgrade_gain(t.slot, u8::MAX)) * keep))
            .filter(|(_, c)| c.abs() > 1e-4)
            .collect()
    }

    /// A seated player's first salary, at the team they start with.
    pub(super) fn first_salary(&mut self) {
        let season = self.season().clone();
        let teams = season.teams.len();
        for p in &mut self.players {
            let Some(e) = season.player_entry(&p.id) else {
                continue;
            };
            if p.salary_k == 0 {
                let (s, b) = salary(
                    p.rating,
                    p.titles.len(),
                    season.pace_rank(e.slot),
                    teams,
                    Some(p.recognition),
                );
                p.salary_k = s;
                p.bonus_k = b;
            }
        }
    }
}

/// Whether `player` is their team's number one driver: the team values them (recognition from
/// [`NUMBER_ONE`] on) and they aren't behind their team-mate in the championship.
pub fn number_one(league: &League, player: &str) -> bool {
    let Some(p) = league.player(player) else {
        return false;
    };
    let season = league.season();
    let Some(entry) = season.player_entry(player) else {
        return false;
    };
    if p.recognition < NUMBER_ONE {
        return false;
    }
    let Some(mate) = super::team_mate(season, &entry.id) else {
        return true;
    };
    let drivers = super::driver_standings(season);
    let points = |id: &str| drivers.iter().find(|d| d.entry == id).map_or(0.0, |d| d.points);
    points(&entry.id) >= points(&mate.id)
}

/// "1:31.457".
pub fn lap_text(ms: u32) -> String {
    format!("{}:{:02}.{:03}", ms / 60_000, ms / 1000 % 60, ms % 1000)
}
