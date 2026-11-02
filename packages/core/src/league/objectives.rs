//! Per-weekend objectives, rival challenges and the rivalry meter (W2-2), as the game's career
//! sets them before each round.
//!
//! - [`prepare`] sets each active, seated player's objectives when a round becomes the next one.
//! - [`grade`] (after each counted or simulated session) and [`grade_practice`] mark them met or
//!   missed and pay the rewards that `grow` doesn't pay already.
//! - The rivalry meter (`Player::rival_score`) is the season's head-to-head against the rival in
//!   qualifying and races: +1 ahead, −1 behind. It is worked out from the season's results, so a
//!   round taken back or left out takes its head-to-heads with it.

use serde::{Deserialize, Serialize};

use super::texts::{Text, t};
use super::{
    Classified, Finish, League, Round, RoundState, Season, SessionKind, is_false, team_goal_at, team_mate_at,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveKind {
    QualiPosition,
    RacePosition,
    BeatTeamMateQuali,
    BeatTeamMateRace,
    BeatRival,
    CleanRace,
    PracticeGrade,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Objective {
    pub player: String,
    pub kind: ObjectiveKind,
    /// A position or a grade; 0 when it doesn't apply.
    #[serde(default)]
    pub target: u8,
    #[serde(default)]
    pub reward_rp: u32,
    #[serde(default)]
    pub reward_recognition: f32,
    /// `None` open, `Some(true)` met, `Some(false)` missed.
    #[serde(default)]
    pub met: Option<bool>,
    /// The reward is paid by [`League::grow`](super::League) already (shown only).
    #[serde(default, skip_serializing_if = "is_false")]
    pub paid_elsewhere: bool,
}

/// The practice grade the team asks for.
pub const PRACTICE_TARGET: u8 = 70;

/// Rewards of the objectives this module pays.
const QUALI_RP: u32 = 40;
const QUALI_RECOGNITION: f32 = 0.5;
const RIVAL_RP: u32 = 60;
const RIVAL_RECOGNITION: f32 = 1.0;
const CLEAN_RP: u32 = 30;
const PRACTICE_RP: u32 = 20;
/// What `grow` pays for the objectives it pays (shown only): the race's team goal and beating
/// the team-mate in qualifying and in the race.
const RACE_GOAL_RP: u32 = 150;
const MATE_QUALI_RP: u32 = 60;
const MATE_RACE_RP: u32 = 80;

impl Objective {
    fn new(player: &str, kind: ObjectiveKind, target: u8, reward_rp: u32, reward_recognition: f32) -> Self {
        Self {
            player: player.to_owned(),
            kind,
            target,
            reward_rp,
            reward_recognition,
            met: None,
            paid_elsewhere: false,
        }
    }

    fn paid_by_grow(self) -> Self {
        Self {
            paid_elsewhere: true,
            ..self
        }
    }
}

/// The objectives of `player` (seated in `season`) for round `n`, in the order of the weekend.
fn objectives_for(season: &Season, player: &str, rival: Option<&str>, n: u8) -> Vec<Objective> {
    let Some(entry) = season.player_entry(player) else {
        return Vec::new();
    };
    let goal = team_goal_at(season, entry.slot, n);
    let mate = team_mate_at(season, &entry.id, n).is_some();
    let lineup = season.lineup(n);
    let rival = rival.is_some_and(|r| lineup.iter().any(|(e, _)| e.id == r));
    let mut list = vec![
        Objective::new(
            player,
            ObjectiveKind::PracticeGrade,
            PRACTICE_TARGET,
            PRACTICE_RP,
            0.0,
        ),
        Objective::new(
            player,
            ObjectiveKind::QualiPosition,
            goal,
            QUALI_RP,
            QUALI_RECOGNITION,
        ),
    ];
    if mate {
        list.push(
            Objective::new(player, ObjectiveKind::BeatTeamMateQuali, 0, MATE_QUALI_RP, 0.0).paid_by_grow(),
        );
    }
    list.push(Objective::new(player, ObjectiveKind::RacePosition, goal, RACE_GOAL_RP, 0.0).paid_by_grow());
    if mate {
        list.push(
            Objective::new(player, ObjectiveKind::BeatTeamMateRace, 0, MATE_RACE_RP, 0.0).paid_by_grow(),
        );
    }
    if rival {
        list.push(Objective::new(
            player,
            ObjectiveKind::BeatRival,
            0,
            RIVAL_RP,
            RIVAL_RECOGNITION,
        ));
    }
    list.push(Objective::new(player, ObjectiveKind::CleanRace, 0, CLEAN_RP, 0.0));
    list
}

/// Called by `settle()` whenever round `round` is the next one (more than once per round: it
/// keeps what it set). Sets the objectives of every active, seated player who has none, as long
/// as no session of the round is counted yet; drops those of rounds left out; and works out the
/// rivalry meter again (a round taken back takes its head-to-heads with it).
pub(super) fn prepare(league: &mut League, round: u8) {
    let season = league.season().clone();
    let players: Vec<(String, Option<String>)> = league
        .active_players()
        .into_iter()
        .map(|p| (p.id.clone(), p.rival.clone()))
        .collect();
    let season_mut = league.season_mut();
    // A round left out has nothing to meet (the rewards went with its snapshot).
    for r in &mut season_mut.rounds {
        if r.state == RoundState::Skipped {
            r.objectives.clear();
        }
    }
    if let Some(r) = season_mut.rounds.iter_mut().find(|r| r.round == round)
        && !r.done()
        && !r.started()
    {
        for (id, rival) in &players {
            if r.objectives.iter().any(|o| &o.player == id) {
                continue;
            }
            r.objectives
                .extend(objectives_for(&season, id, rival.as_deref(), round));
        }
    }
    rivalry(league);
}

/// Ahead of `other`: a finished car beats one that didn't, else the position (as `grow` judges
/// the team-mate duel).
fn ahead(c: &Classified, other: &Classified) -> bool {
    (c.status != Finish::Finished, c.position) < (other.status != Finish::Finished, other.position)
}

/// Each player's head-to-head against the rival in this season's qualifying sessions and races.
fn rivalry(league: &mut League) {
    let season = league.season();
    let scores: Vec<i32> = league
        .players
        .iter()
        .map(|p| {
            let (Some(entry), Some(rival)) = (season.player_entry(&p.id), p.rival.as_deref()) else {
                return 0;
            };
            season
                .rounds
                .iter()
                .flat_map(|r| [&r.quali_result, &r.race_result])
                .map(|list| {
                    let me = list.iter().find(|c| c.entry == entry.id);
                    let them = list.iter().find(|c| c.entry == rival);
                    match (me, them) {
                        (Some(me), Some(them)) if ahead(me, them) => 1,
                        (Some(_), Some(_)) => -1,
                        _ => 0,
                    }
                })
                .sum()
        })
        .collect();
    for (p, score) in league.players.iter_mut().zip(scores) {
        p.rival_score = score;
    }
}

/// Called at the end of `grow()`, after the `kind` session of round `n` was counted (`driven` in
/// the game, else simulated: both are graded).
pub(super) fn grade(league: &mut League, n: u8, kind: SessionKind, _driven: bool, lines: &mut Vec<Text>) {
    if matches!(kind, SessionKind::Qualifying | SessionKind::Race) {
        rivalry(league);
    }
    let season = league.season().clone();
    let Some(round) = season.round(n) else {
        return;
    };
    let list = match kind {
        SessionKind::Qualifying => &round.quali_result,
        SessionKind::Race => &round.race_result,
        SessionKind::Sprint | SessionKind::Practice => return,
    };
    let mut verdicts: Vec<(usize, bool)> = Vec::new();
    for (i, o) in round.objectives.iter().enumerate() {
        if o.met.is_some() {
            continue;
        }
        let Some(entry) = season.player_entry(&o.player) else {
            continue;
        };
        let mine = list.iter().find(|c| c.entry == entry.id);
        let finished = mine.is_some_and(|c| c.status == Finish::Finished);
        // Beating a car that isn't classified: finishing is enough.
        let beat = |other: Option<&str>| -> bool {
            match (mine, other.and_then(|id| list.iter().find(|c| c.entry == id))) {
                (Some(me), Some(them)) => ahead(me, them),
                (Some(_), None) => finished,
                (None, _) => false,
            }
        };
        let mate = team_mate_at(&season, &entry.id, n).map(|m| m.id.as_str());
        let rival = league.player(&o.player).and_then(|p| p.rival.as_deref());
        let met = match (kind, o.kind) {
            (SessionKind::Qualifying, ObjectiveKind::QualiPosition) => {
                finished && mine.is_some_and(|c| c.position <= o.target)
            }
            (SessionKind::Qualifying, ObjectiveKind::BeatTeamMateQuali) => beat(mate),
            (SessionKind::Race, ObjectiveKind::RacePosition) => {
                finished && mine.is_some_and(|c| c.position <= o.target)
            }
            (SessionKind::Race, ObjectiveKind::BeatTeamMateRace) => beat(mate),
            (SessionKind::Race, ObjectiveKind::BeatRival) => beat(rival),
            (SessionKind::Race, ObjectiveKind::CleanRace) => {
                finished && mine.is_some_and(|c| c.penalty_s == 0)
            }
            // The race ends the weekend: what is still open (a practice or qualifying not
            // counted) is missed.
            (SessionKind::Race, _) => false,
            _ => continue,
        };
        verdicts.push((i, met));
    }
    settle_verdicts(league, n, &verdicts);
    if kind == SessionKind::Race {
        let round = league.season().round(n).expect("graded round");
        let mut counts: Vec<Text> = Vec::new();
        for p in &league.players {
            let mine = of(round, &p.id);
            if mine.is_empty() {
                continue;
            }
            let met = mine.iter().filter(|o| o.met == Some(true)).count();
            counts.push(t!("{name} {met} of {count}", name = p.name, met, count = mine.len()));
        }
        if !counts.is_empty() {
            lines.push(t!("Objectives: {counts}.").list("counts", counts));
        }
    }
}

/// Mark the objectives (by index into round `n`'s list) met or missed, and pay the rewards of
/// those met that `grow` doesn't pay.
fn settle_verdicts(league: &mut League, n: u8, verdicts: &[(usize, bool)]) {
    let mut pay: Vec<(String, u32, f32)> = Vec::new();
    if let Some(r) = league.season_mut().rounds.iter_mut().find(|r| r.round == n) {
        for &(i, met) in verdicts {
            let Some(o) = r.objectives.get_mut(i) else {
                continue;
            };
            if o.met.is_some() {
                continue;
            }
            o.met = Some(met);
            if met && !o.paid_elsewhere {
                pay.push((o.player.clone(), o.reward_rp, o.reward_recognition));
            }
        }
    }
    for (player, rp, recognition) in pay {
        if rp > 0 {
            league.earn(&player, rp as f32);
        }
        if recognition != 0.0 {
            league.recognise(&player, recognition);
        }
    }
}

/// Called at the end of `count_practice()` with the practice's grade.
pub(super) fn grade_practice(league: &mut League, n: u8, player: &str, grade: u8) {
    let Some(round) = league.season().round(n) else {
        return;
    };
    let verdicts: Vec<(usize, bool)> = round
        .objectives
        .iter()
        .enumerate()
        .filter(|(_, o)| o.player == player && o.kind == ObjectiveKind::PracticeGrade && o.met.is_none())
        .map(|(i, o)| (i, grade >= o.target))
        .collect();
    settle_verdicts(league, n, &verdicts);
}

/// Called by `undo()` for each round taken back: its objectives are open again. The rewards come
/// back with the round's snapshot, the rivalry meter with the next `prepare`.
pub(super) fn reset(round: &mut Round) {
    for o in &mut round.objectives {
        o.met = None;
    }
}

/// `player`'s objectives of `round`.
pub fn of<'a>(round: &'a Round, player: &str) -> Vec<&'a Objective> {
    round.objectives.iter().filter(|o| o.player == player).collect()
}
