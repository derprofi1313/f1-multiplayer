//! The league career: a career from 2016 to 2026 that two (or more) friends drive together,
//! kept by the host server instead of the game.
//!
//! Every round is an ordinary LAN race in the game, with that year's Classic Seasons box and the
//! round's track. The game's own UDP telemetry ([`udp`]) reports the final classification; the
//! host server maps it onto the season's grid, scores it with that year's points rules and keeps
//! the standings. Rounds at circuits the game doesn't have are simulated (or skipped), like any
//! round the friends choose not to drive. When a season ends, each player gets contract offers
//! that depend on how they did, signs one, and the next season starts with that year's real
//! grid, the players in the seats they signed for.
//!
//! The league is one JSON document ([`League`]) stored as the save of a shared career
//! ([`MODE`], see [`crate::multiplayer::host::careers`]). Players change it only through
//! [`Action`]s the server applies with [`apply`], so both launchers always see the same career.
//! The season data comes from Progressive Career (`tools/league/gen_seasons.py`).

pub mod lineup;
pub mod progress;
mod sim;
pub mod solo;
pub mod udp;

pub use lineup::Change;
pub use progress::{Attributes, Focus, Part, PracticeLine, Programme, ProgrammeResult, Progress, Upgrade};

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

/// The shared-career mode of a league career.
pub const MODE: &str = "f1mp.league";
/// The document's schema version.
pub const SCHEMA: u32 = 1;
pub const FIRST_YEAR: u16 = 2016;
pub const LAST_YEAR: u16 = 2026;
/// News items kept, the newest included.
const NEWS_MAX: usize = 300;
const RATING_MIN: f32 = 60.0;
const RATING_MAX: f32 = 99.0;
/// The rating a player starts with.
pub const START_RATING: f32 = 78.0;

// ---------------------------------------------------------------------------------------------
// Season data

#[derive(Debug, Deserialize)]
pub struct Catalog {
    pub years: BTreeMap<u16, CatalogYear>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CatalogYear {
    pub teams: Vec<CatalogTeam>,
    pub calendar: Vec<CatalogRound>,
    /// The real mid-season line-up changes.
    #[serde(default)]
    pub changes: Vec<CatalogChange>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CatalogTeam {
    /// The team's lineage across the years (Toro Rosso, AlphaTauri, RB and Racing Bulls share one).
    pub slot: u8,
    pub name: String,
    pub code: String,
    /// The car's gap to the fastest car of the year, in percent.
    pub pace: f32,
    /// How the car's real pace moved over the season (see [`Season::pace_at`]).
    #[serde(default)]
    pub development: Vec<Checkpoint>,
    pub drivers: Vec<CatalogDriver>,
}

/// The car's qualifying gap at a point of the season (`at`, 0 to 1), relative to its gap for the
/// year: `delta` > 0 is slower.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Checkpoint {
    pub at: f32,
    pub delta: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CatalogDriver {
    pub key: String,
    pub name: String,
    pub number: u8,
    pub rating: f32,
    /// Experience, racecraft, awareness and pace that year, as the game rates them.
    #[serde(default)]
    pub attributes: Option<[f32; 4]>,
}

/// A real mid-season line-up change: from round `round`, `driver` races for team `slot` in place of
/// `out`, until round `until` (when `out` is back) or for good.
#[derive(Debug, Clone, Deserialize)]
pub struct CatalogChange {
    pub round: u8,
    pub until: Option<u8>,
    pub slot: u8,
    pub out: String,
    pub driver: String,
    pub name: String,
    pub number: u8,
    pub rating: f32,
    #[serde(default)]
    pub attributes: Option<[f32; 4]>,
    /// `stand_in`, `move` or `replace`.
    pub kind: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CatalogRound {
    pub round: u8,
    pub circuit: String,
    pub race: String,
    /// The game's track id (UDP `m_trackId`), or `None` for a circuit the game doesn't have.
    pub track: Option<u8>,
    pub sprint: bool,
}

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| serde_json::from_str(include_str!("seasons.json")).expect("seasons.json is valid"))
}

pub fn catalog_year(year: u16) -> Option<&'static CatalogYear> {
    catalog().years.get(&year)
}

/// A real driver's ratings in `year` (experience, racecraft, awareness, pace), as the game rates
/// them.
pub fn driver_attributes(year: u16, key: &str) -> Option<Attributes> {
    let cat = catalog_year(year)?;
    cat.teams
        .iter()
        .flat_map(|t| &t.drivers)
        .find(|d| d.key == key)
        .and_then(|d| d.attributes)
        .or_else(|| {
            cat.changes
                .iter()
                .find(|c| c.driver == key)
                .and_then(|c| c.attributes)
        })
        .map(Attributes::from_array)
}

/// The name of the circuit with game track id `track`, from any year's calendar.
pub fn track_name(track: u8) -> Option<&'static str> {
    catalog()
        .years
        .values()
        .rev()
        .flat_map(|y| &y.calendar)
        .find(|r| r.track == Some(track))
        .map(|r| r.race.as_str())
}

// ---------------------------------------------------------------------------------------------
// The document

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Untracked {
    /// Rounds at circuits the game doesn't have are simulated.
    #[default]
    Simulate,
    /// They're left out.
    Skip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PointsRule {
    /// Each year's own rules: the fastest-lap point from 2019 to 2024, 2021's three-two-one sprint.
    #[default]
    Historic,
    /// Today's rules for every year: no fastest-lap point, sprint points 8 to 1.
    Current,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamChoice {
    /// Any team in the first season.
    #[default]
    Free,
    /// The slower half of the grid in the first season.
    Rookie,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Setup {
    pub first_year: u16,
    pub last_year: u16,
    /// Score sprint races at sprint weekends (2021 on).
    pub sprints: bool,
    pub untracked: Untracked,
    pub points: PointsRule,
    /// The race distance the friends agreed on, in percent (shown with each round).
    pub distance: u8,
    pub team_choice: TeamChoice,
    /// The real mid-season line-up changes happen (stand-ins, moves, replaced drivers).
    pub lineup_changes: bool,
}

impl Default for Setup {
    fn default() -> Self {
        Setup {
            first_year: FIRST_YEAR,
            last_year: LAST_YEAR,
            sprints: true,
            untracked: Untracked::Simulate,
            points: PointsRule::Historic,
            distance: 50,
            team_choice: TeamChoice::Free,
            lineup_changes: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    /// Players join and pick their seats for the first season.
    Setup,
    Racing,
    /// The season is over: players sign for next season.
    SeasonEnd,
    /// The last season is over.
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Seat {
    pub slot: u8,
    /// The driver key of the real driver whose seat the player takes.
    pub replaces: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Player {
    /// The host server's user id.
    pub id: String,
    pub name: String,
    pub number: u8,
    pub rating: f32,
    /// This season's seat.
    pub seat: Option<Seat>,
    /// The seat signed for next season.
    #[serde(default)]
    pub signed: Option<Seat>,
    /// Years this player won the drivers' title.
    #[serde(default)]
    pub titles: Vec<u16>,
    /// The last season of the player's contract with their team.
    #[serde(default)]
    pub contract_until: Option<u16>,
    /// The last season of the contract signed for next season.
    #[serde(default)]
    pub signed_until: Option<u16>,
    /// This season's rival: the entry id of a driver of another team with a similar car and
    /// rating, as the game's career picks one.
    #[serde(default)]
    pub rival: Option<String>,
    /// Experience, racecraft, awareness and pace; the overall of them is `rating`.
    #[serde(default)]
    pub attributes: Option<Attributes>,
    /// How much the team values the player, 0 to 100 (see [`progress::NUMBER_ONE`]).
    #[serde(default = "recognition_start")]
    pub recognition: f32,
    /// Resource points to spend on the team's development.
    #[serde(default)]
    pub rp: u32,
    /// The contract's salary per season and bonus per point, in thousands of dollars.
    #[serde(default)]
    pub salary_k: u32,
    #[serde(default)]
    pub bonus_k: u32,
    /// What the player asked for in the contract.
    #[serde(default)]
    pub focus: Option<Focus>,
    /// Everything the player earned in the career, in thousands of dollars.
    #[serde(default)]
    pub earnings_k: u64,
    /// The contract signed for next season: salary, bonus and focus.
    #[serde(default)]
    pub signed_salary_k: u32,
    #[serde(default)]
    pub signed_bonus_k: u32,
    #[serde(default)]
    pub signed_focus: Option<Focus>,
    /// The team's number one driver (see [`progress::number_one`]): a tenth more resource points.
    #[serde(default)]
    pub number_one: bool,
}

fn recognition_start() -> f32 {
    progress::RECOGNITION_START
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Team {
    pub slot: u8,
    pub name: String,
    pub code: String,
    pub pace: f32,
    /// The gap the team carries over from what it developed in earlier seasons (< 0 faster).
    #[serde(default)]
    pub carry: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    /// The driver key, or `p:<user id>` for a player.
    pub id: String,
    pub name: String,
    pub number: u8,
    pub slot: u8,
    pub rating: f32,
    /// The player's user id, for a player's car.
    #[serde(default)]
    pub player: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoundState {
    Pending,
    /// The race was driven and reported.
    Raced,
    Simulated,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Finish {
    Finished,
    Dnf,
    Dsq,
    Dns,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Classified {
    pub entry: String,
    /// The finishing position the game gave; for a car that didn't finish, where it was classified.
    pub position: u8,
    #[serde(default)]
    pub grid: u8,
    #[serde(default)]
    pub laps: u8,
    pub status: Finish,
    /// 0 when unknown (a simulated race).
    #[serde(default)]
    pub best_lap_ms: u32,
    #[serde(default)]
    pub time_ms: u64,
    #[serde(default)]
    pub penalty_s: u16,
    pub points: f32,
    #[serde(default)]
    pub fastest: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Round {
    pub round: u8,
    pub circuit: String,
    pub race: String,
    pub track: Option<u8>,
    pub sprint: bool,
    pub state: RoundState,
    #[serde(default)]
    pub race_result: Vec<Classified>,
    #[serde(default)]
    pub sprint_result: Vec<Classified>,
    /// The qualifying classification (no points; position 1 is the pole).
    #[serde(default)]
    pub quali_result: Vec<Classified>,
    /// The game sessions (UDP session uids) counted for this round.
    #[serde(default)]
    pub sessions: Vec<u64>,
    #[serde(default)]
    pub by: Option<String>,
    /// Simulated or skipped on its own, because the game doesn't have the circuit.
    #[serde(default)]
    pub auto: bool,
    /// Each player's practice.
    #[serde(default)]
    pub practice: Vec<PracticeLine>,
    /// What the players had before the round was counted, to restore when it's taken back.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<Vec<Progress>>,
}

impl Round {
    pub fn done(&self) -> bool {
        self.state != RoundState::Pending
    }

    /// A session of this round (practice, qualifying or the sprint) is counted, but not the race.
    pub fn started(&self) -> bool {
        !self.done()
            && (!self.sprint_result.is_empty() || !self.quali_result.is_empty() || !self.practice.is_empty())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OfferKind {
    /// The player's own team offers a new contract.
    Renew,
    Move,
    /// The player is under contract with their team for next season already.
    Contract,
}

fn one_year() -> u8 {
    1
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Offer {
    pub player: String,
    pub slot: u8,
    pub kind: OfferKind,
    /// The contract's length in seasons (for a [`OfferKind::Contract`], what is left of it).
    #[serde(default = "one_year")]
    pub years: u8,
    /// The salary per season and the bonus per point, in thousands of dollars.
    #[serde(default)]
    pub salary_k: u32,
    #[serde(default)]
    pub bonus_k: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Season {
    pub year: u16,
    pub teams: Vec<Team>,
    /// The starting grid.
    pub entries: Vec<Entry>,
    pub rounds: Vec<Round>,
    /// Contracts for next season, once this one is over.
    #[serde(default)]
    pub offers: Vec<Offer>,
    /// Drivers who join during the season (see [`lineup`]).
    #[serde(default)]
    pub reserves: Vec<Entry>,
    /// The season's line-up changes; `None` in a season from before they existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changes: Option<Vec<Change>>,
    /// The parts the players ordered for their teams.
    #[serde(default)]
    pub upgrades: Vec<Upgrade>,
}

impl Season {
    pub fn team(&self, slot: u8) -> Option<&Team> {
        self.teams.iter().find(|t| t.slot == slot)
    }

    /// A driver of the starting grid or one who joined during the season.
    pub fn entry(&self, id: &str) -> Option<&Entry> {
        self.entries.iter().chain(&self.reserves).find(|e| e.id == id)
    }

    /// The player's car this season.
    pub fn player_entry(&self, player: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.player.as_deref() == Some(player))
    }

    /// The next round to drive.
    pub fn next_round(&self) -> Option<&Round> {
        self.rounds.iter().find(|r| !r.done())
    }

    pub fn round(&self, round: u8) -> Option<&Round> {
        self.rounds.iter().find(|r| r.round == round)
    }

    /// The team's pace rank now, 1 for the fastest car.
    pub fn pace_rank(&self, slot: u8) -> usize {
        self.pace_rank_at(slot, self.current_round())
    }

    /// The team's pace rank in `round`.
    pub fn pace_rank_at(&self, slot: u8, round: u8) -> usize {
        pace_rank(
            self.teams.iter().map(|t| (t.slot, self.pace_at(t.slot, round))),
            slot,
        )
    }
}

fn pace_rank(teams: impl Iterator<Item = (u8, f32)>, slot: u8) -> usize {
    let mut all: Vec<(u8, f32)> = teams.collect();
    all.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
    all.iter().position(|t| t.0 == slot).map_or(all.len(), |i| i + 1)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct News {
    pub year: u16,
    /// 0 for season news.
    pub round: u8,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct League {
    pub schema: u32,
    pub setup: Setup,
    /// Seeds the simulated races, so a simulation is the same wherever it runs.
    pub seed: u64,
    pub phase: Phase,
    pub players: Vec<Player>,
    /// Every season so far; the last one is the current one.
    pub seasons: Vec<Season>,
    #[serde(default)]
    pub news: Vec<News>,
    /// The players' ratings before the season-end update, so taking back the last round restores
    /// them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ratings_before: Option<Vec<(String, f32)>>,
}

/// Who applies an action: a signed-in member of the shared career.
#[derive(Debug, Clone)]
pub struct Actor {
    pub id: String,
    pub name: String,
    /// Whether they started the career (only they begin it and move it to the next season by
    /// force).
    pub owner: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeagueError {
    pub reason: &'static str,
    pub message: String,
}

impl fmt::Display for LeagueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for LeagueError {}

fn refuse<T>(reason: &'static str, message: impl Into<String>) -> Result<T, LeagueError> {
    Err(LeagueError {
        reason,
        message: message.into(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionKind {
    Race,
    Sprint,
    Qualifying,
    /// A practice session: the player's practice programmes.
    Practice,
}

/// The kind of session a game session type is, when it's counted: practice (1 to 4: P1, P2, P3
/// and short practice), qualifying (5 to 9: Q1, Q2, Q3, short and one-shot), or a race (15 to
/// 17). Sprint shootouts and time trial aren't.
pub fn session_kind(session_type: u8) -> Option<SessionKind> {
    match session_type {
        1..=4 => Some(SessionKind::Practice),
        5..=9 => Some(SessionKind::Qualifying),
        15..=17 => Some(SessionKind::Race),
        _ => None,
    }
}

/// A lap the reporting player drove in a practice session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lap {
    pub ms: u32,
    /// Within track limits.
    pub valid: bool,
}

/// A race as the game reported it over UDP telemetry (see [`udp::Collector`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RaceReport {
    /// The game's session uid.
    pub session: u64,
    pub track: Option<u8>,
    /// The game's session type (15 race, 16 race 2, 17 race 3).
    pub session_type: u8,
    pub cars: Vec<ReportCar>,
    /// When the classification arrived, unix seconds.
    #[serde(default)]
    pub at: u64,
    /// The reporting player's laps, for a practice session.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub laps: Vec<Lap>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportCar {
    pub name: String,
    pub number: u8,
    #[serde(default)]
    pub team: u8,
    pub ai: bool,
    /// The car of the player whose game sent the report.
    pub mine: bool,
    pub position: u8,
    pub grid: u8,
    pub laps: u8,
    /// The game's result status (3 finished, 4 did not finish, 5 disqualified, 6 not classified,
    /// 7 retired).
    pub status: u8,
    pub best_lap_ms: u32,
    pub time_ms: u64,
    pub penalty_s: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Action {
    /// Start the league (the career's owner, once). The owner joins as its first player.
    Start {
        #[serde(default)]
        setup: Setup,
        #[serde(default)]
        number: Option<u8>,
    },
    /// Join the league as a player.
    Enter {
        #[serde(default)]
        number: Option<u8>,
    },
    /// Pick the first season's seat: a team, and whose seat to take (default: the driver with
    /// the lower rating who isn't taken).
    Seat {
        slot: u8,
        #[serde(default)]
        replaces: Option<String>,
    },
    /// Start the first season (the owner).
    Begin,
    /// Count a race the game reported. `round` defaults to the next round, `kind` to a guess
    /// from the session type.
    Report {
        #[serde(default)]
        round: Option<u8>,
        #[serde(default)]
        kind: Option<SessionKind>,
        /// Count it although it was at another track than the round's.
        #[serde(default)]
        force: bool,
        report: RaceReport,
    },
    /// Simulate a round instead of driving it.
    Simulate {
        #[serde(default)]
        round: Option<u8>,
    },
    /// Leave a round out.
    Skip {
        #[serde(default)]
        round: Option<u8>,
    },
    /// Take back the last round that was counted.
    Undo,
    /// Simulate this player's practice of a round (default: the next) instead of driving it.
    Practice {
        #[serde(default)]
        round: Option<u8>,
    },
    /// Spend resource points on the next level of a part of this player's car.
    Develop { part: Part },
    /// Sign one of this player's offers for next season.
    Sign {
        slot: u8,
        #[serde(default)]
        replaces: Option<String>,
        /// What the player asks for: more salary or more say in the development.
        #[serde(default)]
        focus: Option<Focus>,
    },
    /// Go on to next season (the owner): players who haven't signed stay with their team, or take
    /// their first offer.
    NextSeason,
}

// ---------------------------------------------------------------------------------------------
// Points

/// Points for `position` in a race or sprint of `year`.
pub fn points(year: u16, rule: PointsRule, kind: SessionKind, position: u8, fastest: bool) -> f32 {
    const RACE: [f32; 10] = [25., 18., 15., 12., 10., 8., 6., 4., 2., 1.];
    const SPRINT: [f32; 8] = [8., 7., 6., 5., 4., 3., 2., 1.];
    const SPRINT_2021: [f32; 3] = [3., 2., 1.];
    let historic = rule == PointsRule::Historic;
    let table: &[f32] = match kind {
        SessionKind::Race => &RACE,
        SessionKind::Sprint if historic && year == 2021 => &SPRINT_2021,
        SessionKind::Sprint => &SPRINT,
        SessionKind::Qualifying | SessionKind::Practice => &[],
    };
    let Some(&base) = position.checked_sub(1).and_then(|i| table.get(i as usize)) else {
        return 0.0;
    };
    let bonus =
        kind == SessionKind::Race && fastest && historic && (2019..=2024).contains(&year) && position <= 10;
    base + if bonus { 1.0 } else { 0.0 }
}

// ---------------------------------------------------------------------------------------------
// Standings

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DriverStanding {
    pub entry: String,
    pub name: String,
    pub slot: u8,
    pub player: Option<String>,
    pub points: f32,
    pub wins: u32,
    pub podiums: u32,
    pub poles: u32,
    pub fastest_laps: u32,
    /// The race finish per round of the season, `None` before it or without a classified finish.
    pub finishes: Vec<Option<u8>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TeamStanding {
    pub slot: u8,
    pub name: String,
    pub code: String,
    pub points: f32,
    pub wins: u32,
}

/// The drivers' championship, leader first; ties go to the better finishes (wins, then seconds,
/// and so on), then to the faster car, so a new season lists the grid.
pub fn driver_standings(season: &Season) -> Vec<DriverStanding> {
    let now = season.current_round();
    // Drivers who joined during the season once they race.
    let raced: HashSet<&str> = season
        .rounds
        .iter()
        .flat_map(|r| {
            r.race_result
                .iter()
                .chain(&r.sprint_result)
                .chain(&r.quali_result)
        })
        .map(|c| c.entry.as_str())
        .collect();
    let slot_of = |e: &Entry| season.slot_at(&e.id, now).unwrap_or(e.slot);
    let mut order: Vec<&Entry> = season
        .entries
        .iter()
        .chain(
            season
                .reserves
                .iter()
                .filter(|e| raced.contains(e.id.as_str()) || season.slot_at(&e.id, now).is_some()),
        )
        .collect();
    let ranks: BTreeMap<u8, usize> = season
        .teams
        .iter()
        .map(|t| (t.slot, season.pace_rank(t.slot)))
        .collect();
    order.sort_by_key(|e| ranks.get(&slot_of(e)).copied().unwrap_or(usize::MAX));
    let mut rows: Vec<(DriverStanding, [u32; 24])> = order
        .into_iter()
        .map(|e| {
            (
                DriverStanding {
                    entry: e.id.clone(),
                    name: e.name.clone(),
                    slot: slot_of(e),
                    player: e.player.clone(),
                    points: 0.0,
                    wins: 0,
                    podiums: 0,
                    poles: 0,
                    fastest_laps: 0,
                    finishes: vec![None; season.rounds.len()],
                },
                [0; 24],
            )
        })
        .collect();
    for (i, round) in season.rounds.iter().enumerate() {
        for c in round.race_result.iter().chain(&round.sprint_result) {
            if let Some((row, _)) = rows.iter_mut().find(|x| x.0.entry == c.entry) {
                row.points += c.points;
            }
        }
        if let Some(pole) = round
            .quali_result
            .iter()
            .find(|c| c.position == 1 && c.status == Finish::Finished)
            && let Some((row, _)) = rows.iter_mut().find(|x| x.0.entry == pole.entry)
        {
            row.poles += 1;
        }
        if let Some(f) = round.race_result.iter().find(|c| c.fastest)
            && let Some((row, _)) = rows.iter_mut().find(|x| x.0.entry == f.entry)
        {
            row.fastest_laps += 1;
        }
        for c in &round.race_result {
            let Some((row, counts)) = rows.iter_mut().find(|x| x.0.entry == c.entry) else {
                continue;
            };
            if c.status == Finish::Finished && c.position > 0 {
                row.finishes[i] = Some(c.position);
                if let Some(n) = counts.get_mut(c.position as usize - 1) {
                    *n += 1;
                }
                row.wins += u32::from(c.position == 1);
                row.podiums += u32::from(c.position <= 3);
            }
        }
    }
    // A stable sort: equal rows keep the grid order.
    rows.sort_by(|(a, ca), (b, cb)| b.points.total_cmp(&a.points).then_with(|| cb.cmp(ca)));
    rows.into_iter().map(|(r, _)| r).collect()
}

/// The constructors' championship, leader first. Each result counts for the team the driver
/// raced for in that round.
pub fn team_standings(season: &Season) -> Vec<TeamStanding> {
    let mut rows: Vec<TeamStanding> = season
        .teams
        .iter()
        .map(|t| TeamStanding {
            slot: t.slot,
            name: t.name.clone(),
            code: t.code.clone(),
            points: 0.0,
            wins: 0,
        })
        .collect();
    for round in &season.rounds {
        let slot_of = |c: &Classified| {
            season
                .slot_at(&c.entry, round.round)
                .or_else(|| season.entry(&c.entry).map(|e| e.slot))
        };
        for c in round.race_result.iter().chain(&round.sprint_result) {
            if let Some(row) = slot_of(c).and_then(|s| rows.iter_mut().find(|t| t.slot == s)) {
                row.points += c.points;
            }
        }
        if let Some(c) = round
            .race_result
            .iter()
            .find(|c| c.position == 1 && c.status == Finish::Finished)
            && let Some(row) = slot_of(c).and_then(|s| rows.iter_mut().find(|t| t.slot == s))
        {
            row.wins += 1;
        }
    }
    rows.sort_by_key(|t| season.pace_rank(t.slot));
    rows.sort_by(|a, b| b.points.total_cmp(&a.points).then(b.wins.cmp(&a.wins)));
    rows
}

/// How two drivers did against each other this season, in qualifying and in the race: sessions
/// both were classified in, counted for the one ahead.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HeadToHead {
    /// The other driver's entry id and name.
    pub other: String,
    pub other_name: String,
    /// (this driver ahead, the other ahead).
    pub quali: (u32, u32),
    pub race: (u32, u32),
    pub points: (f32, f32),
}

pub fn head_to_head(season: &Season, entry: &str, other: &str) -> Option<HeadToHead> {
    let other_name = season.entry(other)?.name.clone();
    let ahead = |list: &[Classified]| -> Option<bool> {
        let a = list.iter().find(|c| c.entry == entry)?;
        let b = list.iter().find(|c| c.entry == other)?;
        // A car that finished beats one that didn't; else the better position.
        let rank = |c: &Classified| (c.status != Finish::Finished, c.position);
        Some(rank(a) < rank(b))
    };
    let mut h = HeadToHead {
        other: other.to_owned(),
        other_name,
        quali: (0, 0),
        race: (0, 0),
        points: (0.0, 0.0),
    };
    for r in &season.rounds {
        match ahead(&r.quali_result) {
            Some(true) => h.quali.0 += 1,
            Some(false) => h.quali.1 += 1,
            None => {}
        }
        match ahead(&r.race_result) {
            Some(true) => h.race.0 += 1,
            Some(false) => h.race.1 += 1,
            None => {}
        }
    }
    for d in driver_standings(season) {
        if d.entry == entry {
            h.points.0 = d.points;
        } else if d.entry == other {
            h.points.1 = d.points;
        }
    }
    Some(h)
}

/// The team-mate of `entry` now.
pub fn team_mate<'a>(season: &'a Season, entry: &str) -> Option<&'a Entry> {
    team_mate_at(season, entry, season.current_round())
}

/// The team-mate of `entry` in `round`.
pub fn team_mate_at<'a>(season: &'a Season, entry: &str, round: u8) -> Option<&'a Entry> {
    let lineup = season.lineup(round);
    let slot = lineup.iter().find(|(e, _)| e.id == entry)?.1;
    lineup
        .into_iter()
        .find(|(e, s)| *s == slot && e.id != entry)
        .map(|(e, _)| e)
}

/// The team's goal for a race now, as in the game's career: finish at this position or better
/// (P2 for the fastest car, two places further back for each slower one).
pub fn team_goal(season: &Season, slot: u8) -> u8 {
    team_goal_at(season, slot, season.current_round())
}

/// The team's goal for the race of `round`, with the cars as fast as they are then.
pub fn team_goal_at(season: &Season, slot: u8, round: u8) -> u8 {
    let rank = season.pace_rank_at(slot, round);
    (rank * 2).clamp(1, season.entries.len().max(1)) as u8
}

/// What the car and the driver are worth over a lap now: lower is faster (see the simulation).
fn strength(season: &Season, slot: u8, e: &Entry) -> f32 {
    sim::base(season.pace_now(slot), e.rating)
}

/// A rival for the player's car: the driver of another team closest to them in car and rating.
fn choose_rival(season: &Season, player: &str) -> Option<String> {
    let me = season.player_entry(player)?;
    let lineup = season.lineup(season.current_round());
    let mine = strength(season, me.slot, me);
    lineup
        .iter()
        .filter(|(e, slot)| *slot != me.slot && e.player.is_none())
        .min_by(|(a, sa), (b, sb)| {
            (strength(season, *sa, a) - mine)
                .abs()
                .total_cmp(&(strength(season, *sb, b) - mine).abs())
                .then(a.id.cmp(&b.id))
        })
        .map(|(e, _)| e.id.clone())
}

/// One season of a player's career.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SeasonLine {
    pub year: u16,
    pub team: String,
    /// The championship position (0 without a seat).
    pub position: usize,
    pub points: f32,
    pub wins: u32,
    pub podiums: u32,
    pub poles: u32,
    /// Rounds driven or simulated with a result for this player.
    pub starts: u32,
    pub complete: bool,
}

/// A player's whole career: every season and the totals.
#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct CareerStats {
    pub seasons: Vec<SeasonLine>,
    pub starts: u32,
    pub wins: u32,
    pub podiums: u32,
    pub poles: u32,
    pub fastest_laps: u32,
    pub points: f32,
    pub dnfs: u32,
    pub titles: u32,
    /// The best race finish, 0 before the first.
    pub best_finish: u8,
}

pub fn career_stats(league: &League, player: &str) -> CareerStats {
    let mut s = CareerStats::default();
    let last = league.seasons.len().saturating_sub(1);
    for (i, season) in league.seasons.iter().enumerate() {
        let Some(entry) = season.player_entry(player) else {
            continue;
        };
        let drivers = driver_standings(season);
        let Some((place, row)) = drivers.iter().enumerate().find(|(_, d)| d.entry == entry.id) else {
            continue;
        };
        let starts = season
            .rounds
            .iter()
            .filter(|r| r.race_result.iter().any(|c| c.entry == entry.id))
            .count() as u32;
        let dnfs = season
            .rounds
            .iter()
            .flat_map(|r| &r.race_result)
            .filter(|c| c.entry == entry.id && c.status != Finish::Finished)
            .count() as u32;
        let best = row.finishes.iter().flatten().min().copied().unwrap_or(0);
        s.seasons.push(SeasonLine {
            year: season.year,
            team: season
                .team(entry.slot)
                .map_or_else(String::new, |t| t.name.clone()),
            position: place + 1,
            points: row.points,
            wins: row.wins,
            podiums: row.podiums,
            poles: row.poles,
            starts,
            complete: i < last || league.phase != Phase::Racing,
        });
        s.starts += starts;
        s.wins += row.wins;
        s.podiums += row.podiums;
        s.poles += row.poles;
        s.fastest_laps += row.fastest_laps;
        s.points += row.points;
        s.dnfs += dnfs;
        if best > 0 && (s.best_finish == 0 || best < s.best_finish) {
            s.best_finish = best;
        }
    }
    s.titles = league.player(player).map_or(0, |p| p.titles.len() as u32);
    s
}

// ---------------------------------------------------------------------------------------------
// The league

/// What a player has to set up in the game for the next round.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Briefing {
    pub year: u16,
    pub round: u8,
    pub rounds: usize,
    pub race: String,
    pub track: Option<u8>,
    pub sprint: bool,
    /// The sprint of this round is counted already.
    pub sprint_done: bool,
    pub distance: u8,
    /// The player's team and the driver whose seat they drive, when they have a seat.
    pub team: Option<String>,
    pub replaces: Option<String>,
    /// This round's qualifying is counted already.
    pub quali_done: bool,
    /// The team's goal for the race: finish at this position or better.
    pub goal: Option<u8>,
    pub mate: Option<String>,
    pub rival: Option<String>,
    /// Only one player drives this career: rounds are Grand Prix races, not LAN lobbies.
    pub solo: bool,
    /// This player's practice of this round is counted already.
    pub practice_done: bool,
    /// The line-up changes in force this round, which the game's season box doesn't show.
    pub lineup: Vec<String>,
}

/// "Stoffel Vandoorne drives Fernando Alonso's McLaren (the game shows Fernando Alonso)" for each
/// line-up change in force in `round`.
pub fn lineup_notes(season: &Season, round: u8) -> Vec<String> {
    season
        .changes()
        .iter()
        .filter(|c| c.active(round))
        .filter_map(|c| {
            let driver = season.entry(&c.driver)?;
            let out = season.entry(&c.out)?;
            let team = season.team(c.slot)?;
            Some(format!(
                "{} drives {}'s {} (the game shows {})",
                driver.name, out.name, team.name, out.name
            ))
        })
        .collect()
}

impl League {
    /// A new league, with the owner as its first player.
    pub fn start(setup: Setup, owner: &Actor, number: Option<u8>, seed: u64) -> Result<League, LeagueError> {
        if setup.first_year > setup.last_year
            || catalog_year(setup.first_year).is_none()
            || catalog_year(setup.last_year).is_none()
        {
            return refuse(
                "bad_setup",
                format!("the career runs from {FIRST_YEAR} to {LAST_YEAR}; pick years in between"),
            );
        }
        if !(5..=100).contains(&setup.distance) {
            return refuse("bad_setup", "the race distance is a percentage from 5 to 100");
        }
        let first = build_season(setup.first_year, &[], setup.lineup_changes)?;
        let mut league = League {
            schema: SCHEMA,
            setup,
            seed,
            phase: Phase::Setup,
            players: Vec::new(),
            seasons: vec![first],
            news: Vec::new(),
            ratings_before: None,
        };
        league.enter(owner, number)?;
        let first_year = league.setup.first_year;
        league.say(
            0,
            format!(
                "{} started the career, {first_year} to {}.",
                owner.name, league.setup.last_year
            ),
        );
        Ok(league)
    }

    pub fn season(&self) -> &Season {
        self.seasons.last().expect("a league has a season")
    }

    fn season_mut(&mut self) -> &mut Season {
        self.seasons.last_mut().expect("a league has a season")
    }

    pub fn year(&self) -> u16 {
        self.season().year
    }

    pub fn player(&self, id: &str) -> Option<&Player> {
        self.players.iter().find(|p| p.id == id)
    }

    fn player_mut(&mut self, id: &str) -> Option<&mut Player> {
        self.players.iter_mut().find(|p| p.id == id)
    }

    /// This player's offers for next season.
    pub fn offers(&self, player: &str) -> Vec<&Offer> {
        self.season()
            .offers
            .iter()
            .filter(|o| o.player == player)
            .collect()
    }

    /// What `player` sets up in the game for the next round.
    pub fn briefing(&self, player: &str) -> Option<Briefing> {
        if self.phase != Phase::Racing {
            return None;
        }
        let season = self.season();
        let round = season.next_round()?;
        let seat = self.player(player).and_then(|p| p.seat.as_ref());
        let entry = season.player_entry(player);
        Some(Briefing {
            practice_done: round.practice.iter().any(|l| l.player == player),
            lineup: lineup_notes(season, round.round),
            quali_done: !round.quali_result.is_empty(),
            goal: entry.map(|e| team_goal(season, e.slot)),
            mate: entry
                .and_then(|e| team_mate(season, &e.id))
                .map(|m| m.name.clone()),
            rival: self
                .player(player)
                .and_then(|p| p.rival.as_deref())
                .and_then(|id| season.entry(id))
                .map(|e| e.name.clone()),
            solo: self.players.len() == 1,
            year: season.year,
            round: round.round,
            rounds: season.rounds.len(),
            race: round.race.clone(),
            track: round.track,
            sprint: round.sprint && self.setup.sprints,
            sprint_done: !round.sprint_result.is_empty(),
            distance: self.setup.distance,
            team: seat.and_then(|s| season.team(s.slot)).map(|t| t.name.clone()),
            replaces: seat
                .and_then(|s| catalog_driver(season.year, s.slot, &s.replaces))
                .map(|d| d.name.clone()),
        })
    }

    fn say(&mut self, round: u8, text: String) {
        let year = self.seasons.last().map_or(self.setup.first_year, |s| s.year);
        self.news.push(News { year, round, text });
        if self.news.len() > NEWS_MAX {
            let extra = self.news.len() - NEWS_MAX;
            self.news.drain(..extra);
        }
    }

    fn enter(&mut self, actor: &Actor, number: Option<u8>) -> Result<(), LeagueError> {
        if self.player(&actor.id).is_some() {
            return Ok(());
        }
        if self.players.len() >= 22 {
            return refuse("full", "this career has no free seat");
        }
        let taken: HashSet<u8> = self.players.iter().map(|p| p.number).collect();
        let number = match number {
            Some(n) if (1..=99).contains(&n) && !taken.contains(&n) => n,
            Some(n) if taken.contains(&n) => {
                return refuse("number_taken", format!("another player races with number {n}"));
            }
            Some(_) => return refuse("bad_number", "pick a race number from 1 to 99"),
            None => (2..=99).rev().find(|n| !taken.contains(n)).unwrap_or(99),
        };
        self.players.push(Player {
            id: actor.id.clone(),
            name: actor.name.clone(),
            number,
            rating: START_RATING,
            seat: None,
            signed: None,
            titles: Vec::new(),
            contract_until: None,
            signed_until: None,
            rival: None,
            attributes: Some(Attributes::from_rating(START_RATING)),
            recognition: progress::RECOGNITION_START,
            rp: 0,
            salary_k: 0,
            bonus_k: 0,
            focus: None,
            earnings_k: 0,
            signed_salary_k: 0,
            signed_bonus_k: 0,
            signed_focus: None,
            number_one: false,
        });
        if self.phase != Phase::Setup {
            self.say(
                0,
                format!(
                    "{} joined the career and gets offers when the season ends.",
                    actor.name
                ),
            );
        }
        Ok(())
    }

    /// The seats players hold this season.
    fn seats(&self) -> Vec<(Player, Seat)> {
        self.players
            .iter()
            .filter_map(|p| p.seat.clone().map(|s| (p.clone(), s)))
            .collect()
    }

    fn pick_seat(&mut self, actor: &Actor, slot: u8, replaces: Option<String>) -> Result<(), LeagueError> {
        if self.phase != Phase::Setup {
            return refuse(
                "phase",
                "seats are picked before the first season; later, sign an offer",
            );
        }
        if self.player(&actor.id).is_none() {
            return refuse("not_player", "join the career first");
        }
        let year = self.setup.first_year;
        let cat = catalog_year(year).expect("checked at start");
        let Some(team) = cat.teams.iter().find(|t| t.slot == slot) else {
            return refuse("bad_team", format!("no team {slot} in {year}"));
        };
        if self.setup.team_choice == TeamChoice::Rookie {
            let rank = pace_rank(cat.teams.iter().map(|t| (t.slot, t.pace)), slot);
            if rank * 2 <= cat.teams.len() {
                return refuse(
                    "rookie",
                    format!(
                        "as a rookie you start in the slower half of the grid; {} is too fast",
                        team.name
                    ),
                );
            }
        }
        let others: Vec<&str> = self
            .players
            .iter()
            .filter(|p| p.id != actor.id)
            .filter_map(|p| p.seat.as_ref())
            .filter(|s| s.slot == slot)
            .map(|s| s.replaces.as_str())
            .collect();
        let replaces = choose_replaced(team, replaces.as_deref(), &others)?;
        self.player_mut(&actor.id).expect("checked").seat = Some(Seat { slot, replaces });
        let seats = self.seats();
        *self.season_mut() = build_season(year, &seats, self.setup.lineup_changes)?;
        Ok(())
    }

    fn begin(&mut self, actor: &Actor) -> Result<(), LeagueError> {
        if !actor.owner {
            return refuse("owner", "only the career's owner starts the first season");
        }
        if self.phase != Phase::Setup {
            return refuse("phase", "the career has started already");
        }
        if let Some(p) = self.players.iter().find(|p| p.seat.is_none()) {
            return refuse("no_seat", format!("{} hasn't picked a seat yet", p.name));
        }
        self.phase = Phase::Racing;
        // The first contract runs for the first season.
        let year = self.year();
        for p in &mut self.players {
            p.contract_until = Some(year);
        }
        self.first_salary();
        let line = self.lineup_line();
        self.say(0, format!("The {} season begins. {line}", self.year()));
        self.assign_rivals();
        self.settle();
        Ok(())
    }

    /// Each player's rival for the season, with a news line.
    fn assign_rivals(&mut self) {
        let season = self.season().clone();
        let mut lines = Vec::new();
        for p in &mut self.players {
            p.rival = choose_rival(&season, &p.id);
            if let Some(r) = p.rival.as_deref().and_then(|id| season.entry(id)) {
                let team = season.team(r.slot).map_or("", |t| t.name.as_str());
                lines.push(format!("{}'s rival this season: {} ({team}).", p.name, r.name));
            }
        }
        if !lines.is_empty() {
            self.say(0, lines.join(" "));
        }
    }

    fn lineup_line(&self) -> String {
        let season = self.season();
        self.players
            .iter()
            .filter_map(|p| {
                let e = season.player_entry(&p.id)?;
                let team = season.team(e.slot)?;
                Some(format!("{} drives for {} (#{}).", p.name, team.name, e.number))
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn pending_round(&self, round: Option<u8>) -> Result<u8, LeagueError> {
        if self.phase != Phase::Racing {
            return refuse("phase", "no season is running");
        }
        let season = self.season();
        match round {
            Some(n) => match season.round(n) {
                Some(r) if !r.done() => Ok(n),
                Some(_) => refuse("round_done", format!("round {n} is counted already")),
                None => refuse("bad_round", format!("{} has no round {n}", season.year)),
            },
            None => season
                .next_round()
                .map(|r| r.round)
                .map_or_else(|| refuse("round_done", "every round is counted"), Ok),
        }
    }

    fn report(
        &mut self,
        actor: &Actor,
        round: Option<u8>,
        kind: Option<SessionKind>,
        force: bool,
        report: RaceReport,
    ) -> Result<Vec<String>, LeagueError> {
        if self.phase == Phase::Racing
            && report.session != 0
            && self
                .season()
                .rounds
                .iter()
                .any(|r| r.sessions.contains(&report.session))
        {
            return Ok(vec!["this race is counted already".into()]);
        }
        let n = self.pending_round(round)?;
        let (year, rule, sprints) = (self.year(), self.setup.points, self.setup.sprints);
        let r = self.season().round(n).expect("pending").clone();
        if let (Some(at), Some(want)) = (report.track, r.track)
            && at != want
            && !force
        {
            return refuse(
                "wrong_track",
                format!(
                    "this race was at {}, but round {n} is the {}",
                    track_name(at).unwrap_or("another track"),
                    r.race
                ),
            );
        }
        let sprint_open = r.sprint && sprints && r.sprint_result.is_empty();
        if kind.is_none() && (10..=14).contains(&report.session_type) {
            return refuse(
                "not_counted",
                "a sprint shootout isn't counted; the sprint and the race are",
            );
        }
        let kind = kind.unwrap_or(match session_kind(report.session_type) {
            Some(SessionKind::Qualifying) => SessionKind::Qualifying,
            Some(SessionKind::Practice) => SessionKind::Practice,
            _ if report.session_type == 15 && sprint_open => SessionKind::Sprint,
            _ => SessionKind::Race,
        });
        if kind == SessionKind::Practice {
            if report.session != 0
                && r.practice
                    .iter()
                    .any(|l| l.player == actor.id && l.session == report.session)
            {
                return Ok(vec!["this practice is counted already".into()]);
            }
            let lines = self.count_practice(&actor.id, n, Some(&report))?;
            self.say(n, lines.join(" "));
            return Ok(Vec::new());
        }
        if kind == SessionKind::Sprint && !(r.sprint && sprints) {
            return refuse("no_sprint", format!("round {n} has no sprint"));
        }
        if kind == SessionKind::Sprint && !r.sprint_result.is_empty() {
            return refuse(
                "round_done",
                format!("the sprint of round {n} is counted already"),
            );
        }
        let mine = self
            .player(&actor.id)
            .filter(|p| p.seat.is_some())
            .map(|p| p.id.clone());
        let (result, warnings) = classify(self.season(), mine.as_deref(), &report, kind, year, rule, n);
        if result.is_empty() {
            return refuse(
                "no_match",
                "none of the cars in this race belong to this season's grid",
            );
        }
        let text = result_line(self.season(), &r, kind, &result, &self.players);
        self.keep_progress(n);
        let before = self.clone();
        let season = self.season_mut();
        let round = season.rounds.iter_mut().find(|x| x.round == n).expect("pending");
        match kind {
            SessionKind::Sprint => round.sprint_result = result,
            SessionKind::Qualifying => round.quali_result = result,
            SessionKind::Race => {
                round.race_result = result;
                round.state = RoundState::Raced;
                round.by = Some(actor.name.clone());
            }
            SessionKind::Practice => unreachable!("counted above"),
        }
        if report.session != 0 {
            round.sessions.push(report.session);
        }
        self.say(n, text);
        self.milestones(&before, n);
        let lines = self.grow(n, kind, true);
        if !lines.is_empty() {
            self.say(n, lines.join(" "));
        }
        self.settle();
        Ok(warnings)
    }

    /// News for what players did for the first time (and every tenth win), the team goal of the
    /// race and the rivalry, comparing with the league before round `n` was counted.
    fn milestones(&mut self, before: &League, n: u8) {
        let season = self.season().clone();
        let Some(round) = season.round(n).cloned() else {
            return;
        };
        let mut lines = Vec::new();
        for p in &self.players {
            let Some(entry) = season.player_entry(&p.id) else {
                continue;
            };
            let (was, now) = (career_stats(before, &p.id), career_stats(self, &p.id));
            let name = &p.name;
            if now.poles > was.poles {
                lines.push(match now.poles {
                    1 => format!("{name} takes the first pole position of their career!"),
                    k => format!("{name}'s pole position number {k}."),
                });
            }
            if now.wins > was.wins {
                lines.push(match now.wins {
                    1 => format!("{name} wins a Grand Prix for the first time!"),
                    k if k % 10 == 0 => format!("{name}'s Grand Prix win number {k}!"),
                    _ => String::new(),
                });
            } else if now.podiums > was.podiums && now.podiums == 1 {
                lines.push(format!("{name} stands on the podium for the first time."));
            }
            if now.points > 0.0 && was.points == 0.0 && now.wins == 0 && now.podiums == 0 {
                lines.push(format!("{name} scores the first points of their career."));
            }
            // The race's team goal, once the race is counted.
            if round.done()
                && let Some(c) = round.race_result.iter().find(|c| c.entry == entry.id)
            {
                let goal = team_goal_at(&season, entry.slot, n);
                if c.status == Finish::Finished && c.position <= goal {
                    lines.push(format!("{name} meets the team's goal (P{goal} or better)."));
                } else {
                    lines.push(format!("{name} misses the team's goal of P{goal}."));
                }
            }
        }
        let lines: Vec<String> = lines.into_iter().filter(|l| !l.is_empty()).collect();
        if !lines.is_empty() {
            self.say(n, lines.join(" "));
        }
    }

    fn simulate(&mut self, round: Option<u8>, auto: bool) -> Result<(), LeagueError> {
        let n = self.pending_round(round)?;
        let (year, rule, seed, sprints) = (self.year(), self.setup.points, self.seed, self.setup.sprints);
        self.keep_progress(n);
        // The practice of every seated player who didn't drive it.
        let mut lines = Vec::new();
        let unpractised: Vec<String> = self
            .players
            .iter()
            .filter(|p| self.season().player_entry(&p.id).is_some())
            .filter(|p| {
                !self
                    .season()
                    .round(n)
                    .is_some_and(|r| r.practice.iter().any(|l| l.player == p.id))
            })
            .map(|p| p.id.clone())
            .collect();
        for id in unpractised {
            if let Ok(l) = self.count_practice(&id, n, None) {
                lines.extend(l);
            }
        }
        if !lines.is_empty() {
            self.say(n, lines.join(" "));
        }
        let r = self.season().round(n).expect("pending").clone();
        // Qualifying sets the grid, unless it was driven.
        let quali_driven = !r.quali_result.is_empty();
        let quali = if r.quali_result.is_empty() {
            sim::simulate(self.season(), n, seed, SessionKind::Qualifying, year, rule, None)
        } else {
            r.quali_result.clone()
        };
        let sprint = (r.sprint && sprints && r.sprint_result.is_empty()).then(|| {
            sim::simulate(
                self.season(),
                n,
                seed,
                SessionKind::Sprint,
                year,
                rule,
                Some(&quali),
            )
        });
        let race = sim::simulate(
            self.season(),
            n,
            seed,
            SessionKind::Race,
            year,
            rule,
            Some(&quali),
        );
        let mut text = result_line(self.season(), &r, SessionKind::Race, &race, &self.players);
        text.push_str(if auto {
            " (simulated: the game doesn't have this circuit)"
        } else {
            " (simulated)"
        });
        let before = self.clone();
        let season = self.season_mut();
        let round = season.rounds.iter_mut().find(|x| x.round == n).expect("pending");
        let sprint_simulated = sprint.is_some();
        if let Some(s) = sprint {
            round.sprint_result = s;
        }
        round.quali_result = quali;
        round.race_result = race;
        round.state = RoundState::Simulated;
        round.auto = auto;
        self.say(n, text);
        self.milestones(&before, n);
        let mut lines = Vec::new();
        if !quali_driven {
            lines.extend(self.grow(n, SessionKind::Qualifying, false));
        }
        if sprint_simulated {
            lines.extend(self.grow(n, SessionKind::Sprint, false));
        }
        lines.extend(self.grow(n, SessionKind::Race, false));
        if !lines.is_empty() {
            self.say(n, lines.join(" "));
        }
        Ok(())
    }

    fn skip(&mut self, round: Option<u8>, auto: bool) -> Result<(), LeagueError> {
        let n = self.pending_round(round)?;
        self.keep_progress(n);
        // The salary is paid for a round left out too.
        self.pay(n, false);
        let season = self.season_mut();
        let round = season.rounds.iter_mut().find(|x| x.round == n).expect("pending");
        round.state = RoundState::Skipped;
        round.auto = auto;
        round.sprint_result.clear();
        round.quali_result.clear();
        let race = round.race.clone();
        self.say(n, format!("Round {n}, the {race}, is left out."));
        Ok(())
    }

    fn undo(&mut self) -> Result<(), LeagueError> {
        if self.phase == Phase::Setup {
            return refuse("phase", "nothing to take back");
        }
        let season = self.season();
        let Some(last) = season
            .rounds
            .iter()
            .rposition(|r| (r.done() && !r.auto) || r.started())
        else {
            return refuse("nothing", "no round of this season to take back");
        };
        let name = season.rounds[last].race.clone();
        let n = season.rounds[last].round;
        let snapshot = season.rounds[last].before.clone();
        let season = self.season_mut();
        for r in &mut season.rounds[last..] {
            if r.done() || r.started() {
                r.state = RoundState::Pending;
                r.race_result.clear();
                r.sprint_result.clear();
                r.quali_result.clear();
                r.practice.clear();
                r.sessions.clear();
                r.by = None;
                r.auto = false;
                r.before = None;
            }
        }
        season.offers.clear();
        if self.phase != Phase::Racing {
            self.phase = Phase::Racing;
            let year = self.year();
            for p in &mut self.players {
                p.signed = None;
                p.signed_until = None;
                p.titles.retain(|&y| y != year);
            }
            if let Some(ratings) = self.ratings_before.take() {
                for (id, rating) in ratings {
                    if let Some(p) = self.player_mut(&id) {
                        p.rating = rating;
                    }
                }
            }
        }
        // Resource points, ratings, recognition and pay as before the round.
        if let Some(snapshot) = snapshot {
            self.restore_progress(snapshot, n);
        }
        self.say(n, format!("Round {n}, the {name}, was taken back."));
        // Rounds the game doesn't have, right after it, are simulated again with the same result.
        Ok(())
    }

    /// Simulate or skip the rounds the game can't drive, and end the season after its last round.
    fn settle(&mut self) {
        while self.phase == Phase::Racing {
            let Some(next) = self.season().next_round().cloned() else {
                self.end_season();
                return;
            };
            self.announce_changes(next.round);
            if next.track.is_some() {
                return;
            }
            let _ = match self.setup.untracked {
                Untracked::Simulate => self.simulate(Some(next.round), true),
                Untracked::Skip => self.skip(Some(next.round), true),
            };
        }
    }

    /// News for the line-up changes that start by round `next`.
    fn announce_changes(&mut self, next: u8) {
        let season = self.season();
        let mut lines = Vec::new();
        let mut told = Vec::new();
        for (i, c) in season.changes().iter().enumerate() {
            if c.announced || c.round > next {
                continue;
            }
            told.push(i);
            let (Some(driver), Some(out), Some(team)) =
                (season.entry(&c.driver), season.entry(&c.out), season.team(c.slot))
            else {
                continue;
            };
            let race = season.round(c.round).map_or("", |r| r.race.as_str());
            let back = c
                .until
                .and_then(|u| season.round(u))
                .map(|r| format!(" until {} is back at the {}", out.name, r.race));
            lines.push(match c.kind.as_str() {
                "stand_in" => format!(
                    "From the {race}, {} stands in for {} at {}{}.",
                    driver.name,
                    out.name,
                    team.name,
                    back.unwrap_or_default()
                ),
                "move" => format!(
                    "From the {race}, {} drives for {} in place of {}{}.",
                    driver.name,
                    team.name,
                    out.name,
                    back.map(|_| " for a while".to_owned()).unwrap_or_default()
                ),
                _ => format!(
                    "From the {race}, {} replaces {} at {}.",
                    driver.name, out.name, team.name
                ),
            });
        }
        if told.is_empty() {
            return;
        }
        if let Some(changes) = self.season_mut().changes.as_mut() {
            for i in told {
                changes[i].announced = true;
            }
        }
        if !lines.is_empty() {
            lines.push("The game's season box still shows its own line-up; the career counts each car for the driver of that seat.".into());
            self.say(next, lines.join(" "));
        }
    }

    fn end_season(&mut self) {
        let year = self.year();
        let drivers = driver_standings(self.season());
        let teams = team_standings(self.season());
        if let (Some(d), Some(t)) = (drivers.first(), teams.first()) {
            let (d_name, d_points, t_name) = (d.name.clone(), d.points, t.name.clone());
            self.say(
                0,
                format!("{d_name} is the {year} world champion with {d_points} points; {t_name} win the constructors' title."),
            );
        }
        // Ratings move with how each player did against their car and their team-mate.
        let mut before = Vec::new();
        let season = self.season().clone();
        for p in &mut self.players {
            let Some(entry) = season.player_entry(&p.id) else {
                continue;
            };
            before.push((p.id.clone(), p.rating));
            let place = drivers
                .iter()
                .position(|d| d.entry == entry.id)
                .map_or(drivers.len(), |i| i + 1);
            let expected = 2.0 * season.pace_rank(entry.slot) as f32 - 0.5;
            let own = drivers
                .iter()
                .find(|d| d.entry == entry.id)
                .map_or(0.0, |d| d.points);
            let mate = drivers
                .iter()
                .find(|d| d.slot == entry.slot && d.entry != entry.id)
                .map(|d| d.points);
            let mate_bonus = match mate {
                Some(m) if own > m => 1.0,
                Some(m) if own < m => -1.0,
                _ => 0.0,
            };
            let delta = ((expected - place as f32) / 2.0).clamp(-4.0, 4.0) + mate_bonus;
            match &mut p.attributes {
                Some(a) => a.shift(delta),
                None => p.rating = (p.rating + delta).clamp(RATING_MIN, RATING_MAX),
            }
            if place == 1 {
                p.titles.push(year);
            }
        }
        self.ratings_before = Some(before);
        self.sync_ratings();
        // What each player earned this season.
        let earned: Vec<String> = self
            .players
            .iter()
            .filter(|p| season.player_entry(&p.id).is_some())
            .map(|p| {
                let start = season
                    .rounds
                    .first()
                    .and_then(|r| r.before.as_ref())
                    .and_then(|b| b.iter().find(|s| s.player == p.id))
                    .map_or(0, |s| s.earnings_k);
                format!(
                    "{} earned {} this season ({} in the career).",
                    p.name,
                    progress::money(p.earnings_k.saturating_sub(start)),
                    progress::money(p.earnings_k)
                )
            })
            .collect();
        if !earned.is_empty() {
            self.say(0, earned.join(" "));
        }
        // How each player did against their team-mate and their rival.
        let mut duels = Vec::new();
        for p in &self.players {
            let Some(entry) = season.player_entry(&p.id) else {
                continue;
            };
            if let Some(h) =
                team_mate(&season, &entry.id).and_then(|m| head_to_head(&season, &entry.id, &m.id))
            {
                duels.push(format!(
                    "{} against team-mate {}: qualifying {}:{}, races {}:{}.",
                    p.name, h.other_name, h.quali.0, h.quali.1, h.race.0, h.race.1
                ));
            }
            if let Some(h) = p
                .rival
                .as_deref()
                .and_then(|r| head_to_head(&season, &entry.id, r))
            {
                let verdict = if h.race.0 > h.race.1 {
                    "won"
                } else if h.race.0 < h.race.1 {
                    "lost"
                } else {
                    "drew"
                };
                duels.push(format!(
                    "{} {verdict} the rivalry with {} {}:{}.",
                    p.name, h.other_name, h.race.0, h.race.1
                ));
            }
        }
        if !duels.is_empty() {
            self.say(0, duels.join(" "));
        }
        if year >= self.setup.last_year {
            self.phase = Phase::Finished;
            self.say(0, "The career is complete.".into());
            return;
        }
        self.phase = Phase::SeasonEnd;
        let offers = make_offers(&self.players, &season, &drivers);
        let lines: Vec<String> = self
            .players
            .iter()
            .map(|p| {
                let names: Vec<String> = offers
                    .iter()
                    .filter(|o| o.player == p.id)
                    .filter_map(|o| {
                        let team = catalog_team(year + 1, o.slot)?;
                        Some(match o.kind {
                            OfferKind::Contract => format!("{} (under contract)", team.name),
                            _ => format!(
                                "{} ({} year{}, {} a season)",
                                team.name,
                                o.years,
                                if o.years == 1 { "" } else { "s" },
                                progress::money(u64::from(o.salary_k))
                            ),
                        })
                    })
                    .collect();
                let renewed = offers
                    .iter()
                    .any(|o| o.player == p.id && o.kind != OfferKind::Move);
                let dropped = if renewed || season.player_entry(&p.id).is_none() {
                    String::new()
                } else {
                    let team = season
                        .player_entry(&p.id)
                        .and_then(|e| season.team(e.slot))
                        .map_or("", |t| t.name.as_str());
                    format!(" {team} doesn't renew the contract.")
                };
                format!("{} has offers from {}.{dropped}", p.name, names.join(", "))
            })
            .collect();
        self.season_mut().offers = offers;
        self.say(0, format!("Contracts for {}: {}", year + 1, lines.join(" ")));
    }

    fn sign(
        &mut self,
        actor: &Actor,
        slot: u8,
        replaces: Option<String>,
        focus: Option<Focus>,
    ) -> Result<(), LeagueError> {
        if self.phase != Phase::SeasonEnd {
            return refuse("phase", "contracts are signed when a season is over");
        }
        let Some(offer) = self
            .season()
            .offers
            .iter()
            .find(|o| o.player == actor.id && o.slot == slot)
            .cloned()
        else {
            return refuse("no_offer", "that team made you no offer");
        };
        let next = self.year() + 1;
        let until = self.contract_end(&actor.id, &offer);
        let team = catalog_team(next, slot).expect("offers name next year's teams");
        let others: Vec<&str> = self
            .players
            .iter()
            .filter(|p| p.id != actor.id)
            .filter_map(|p| p.signed.as_ref())
            .filter(|s| s.slot == slot)
            .map(|s| s.replaces.as_str())
            .collect();
        let replaces = choose_replaced(team, replaces.as_deref(), &others)?;
        let name = team.name.clone();
        let p = self.player_mut(&actor.id).expect("has offers");
        p.signed = Some(Seat { slot, replaces });
        p.signed_until = Some(until);
        let terms = signed_terms(p, &offer, focus);
        let text = if offer.kind == OfferKind::Contract {
            format!(
                "{} stays with {name} for {next}, under contract until {until}.",
                actor.name
            )
        } else if until > next {
            format!(
                "{} signs with {name} for {next} to {until}, for {} a season.",
                actor.name,
                progress::money(u64::from(terms))
            )
        } else {
            format!(
                "{} signs with {name} for {next}, for {}.",
                actor.name,
                progress::money(u64::from(terms))
            )
        };
        self.say(0, text);
        if self.players.iter().all(|p| p.signed.is_some()) {
            self.next_season()?;
        }
        Ok(())
    }

    fn force_next_season(&mut self, actor: &Actor) -> Result<(), LeagueError> {
        if !actor.owner {
            return refuse("owner", "only the career's owner moves on before everyone signed");
        }
        if self.phase != Phase::SeasonEnd {
            return refuse("phase", "the season isn't over yet");
        }
        let next = self.year() + 1;
        let unsigned: Vec<String> = self
            .players
            .iter()
            .filter(|p| p.signed.is_none())
            .map(|p| p.id.clone())
            .collect();
        for id in unsigned {
            let offers = self.offers(&id);
            let pick = offers
                .iter()
                .find(|o| o.kind == OfferKind::Contract)
                .or_else(|| offers.iter().find(|o| o.kind == OfferKind::Renew))
                .or(offers.first())
                .map(|o| (*o).clone());
            let Some(offer) = pick else { continue };
            let slot = offer.slot;
            let until = self.contract_end(&id, &offer);
            let team = catalog_team(next, slot).expect("offers name next year's teams");
            let others: Vec<&str> = self
                .players
                .iter()
                .filter_map(|p| p.signed.as_ref())
                .filter(|s| s.slot == slot)
                .map(|s| s.replaces.as_str())
                .collect();
            if let Ok(replaces) = choose_replaced(team, None, &others) {
                let p = self.player_mut(&id).expect("listed");
                p.signed = Some(Seat { slot, replaces });
                p.signed_until = Some(until);
                signed_terms(p, &offer, None);
            }
        }
        self.next_season()
    }

    /// The last season of the contract `offer` gives `player`.
    fn contract_end(&self, player: &str, offer: &Offer) -> u16 {
        let next = self.year() + 1;
        match offer.kind {
            OfferKind::Contract => self
                .player(player)
                .and_then(|p| p.contract_until)
                .unwrap_or(next)
                .max(next),
            _ => next + u16::from(offer.years.max(1)) - 1,
        }
    }

    fn next_season(&mut self) -> Result<(), LeagueError> {
        let next = self.year() + 1;
        let carry = self.carry_over(next);
        for p in &mut self.players {
            let moved = p.seat.as_ref().map(|s| s.slot) != p.signed.as_ref().map(|s| s.slot);
            p.seat = p.signed.take();
            if let Some(until) = p.signed_until.take() {
                p.contract_until = Some(until);
            }
            if p.signed_salary_k > 0 {
                p.salary_k = std::mem::take(&mut p.signed_salary_k);
                p.bonus_k = std::mem::take(&mut p.signed_bonus_k);
                p.focus = p.signed_focus.take();
            }
            // A new team has to get to know its driver.
            if moved && p.seat.is_some() {
                p.recognition = progress::RECOGNITION_NEW_TEAM;
            }
        }
        let seats = self.seats();
        let mut season = build_season(next, &seats, self.setup.lineup_changes)?;
        for (slot, c) in carry {
            if let Some(t) = season.teams.iter_mut().find(|t| t.slot == slot) {
                t.carry = c;
            }
        }
        self.seasons.push(season);
        self.first_salary();
        // A new season starts level with the team-mate.
        let _ = self.update_status();
        self.ratings_before = None;
        self.phase = Phase::Racing;
        let line = self.lineup_line();
        self.say(0, format!("The {next} season begins. {line}"));
        self.assign_rivals();
        self.settle();
        Ok(())
    }
}

/// Apply `action` by `actor` to the league. Returns warnings worth showing (cars of a reported
/// race that matched no driver, say).
pub fn apply(league: &mut League, actor: &Actor, action: Action) -> Result<Vec<String>, LeagueError> {
    if league.schema > SCHEMA {
        return refuse(
            "newer",
            "this career was saved by a newer F1 Multiplayer; update it first",
        );
    }
    let is_player = league.player(&actor.id).is_some();
    let needs_player = !matches!(action, Action::Enter { .. } | Action::Start { .. });
    if needs_player && !is_player {
        return refuse("not_player", "join the career first");
    }
    // Careers started before rivals and contract lengths existed get them now.
    if league.phase == Phase::Racing {
        let year = league.year();
        let seated: Vec<bool> = league
            .players
            .iter()
            .map(|p| league.season().player_entry(&p.id).is_some())
            .collect();
        for (p, seated) in league.players.iter_mut().zip(&seated) {
            if *seated && p.contract_until.is_none() {
                p.contract_until = Some(year);
            }
        }
        if league
            .players
            .iter()
            .zip(&seated)
            .any(|(p, seated)| *seated && p.rival.is_none())
        {
            league.assign_rivals();
        }
        // ... and ratings, a salary and the season's line-up changes from the next round on.
        for p in &mut league.players {
            if p.attributes.is_none() {
                p.attributes = Some(Attributes::from_rating(p.rating));
            }
        }
        league.sync_ratings();
        league.first_salary();
        if league.season().changes.is_none() {
            let from = league.season().current_round();
            let (changes, reserves) = if league.setup.lineup_changes {
                lineup::changes_for(year, &league.seats(), from)
            } else {
                (Vec::new(), Vec::new())
            };
            let season = league.season_mut();
            season.changes = Some(changes);
            season.reserves = reserves;
            league.announce_changes(from);
        }
    }
    match action {
        Action::Start { .. } => refuse("started", "this career has started already"),
        Action::Enter { number } => league.enter(actor, number).map(|()| Vec::new()),
        Action::Seat { slot, replaces } => league.pick_seat(actor, slot, replaces).map(|()| Vec::new()),
        Action::Begin => league.begin(actor).map(|()| Vec::new()),
        Action::Report {
            round,
            kind,
            force,
            report,
        } => league.report(actor, round, kind, force, report),
        Action::Simulate { round } => league.simulate(round, false).map(|()| {
            league.settle();
            Vec::new()
        }),
        Action::Skip { round } => league.skip(round, false).map(|()| {
            league.settle();
            Vec::new()
        }),
        Action::Undo => league.undo().map(|()| {
            league.settle();
            Vec::new()
        }),
        Action::Practice { round } => {
            let n = league.pending_round(round)?;
            let lines = league.count_practice(&actor.id, n, None)?;
            league.say(n, lines.join(" "));
            Ok(Vec::new())
        }
        Action::Develop { part } => league.develop(&actor.id, part).map(|_| Vec::new()),
        Action::Sign {
            slot,
            replaces,
            focus,
        } => league.sign(actor, slot, replaces, focus).map(|()| Vec::new()),
        Action::NextSeason => league.force_next_season(actor).map(|()| Vec::new()),
    }
}

/// Settle the terms of the contract `p` signs with `offer`: the salary (a quarter more when they
/// ask for it), the bonus and what they asked for. Returns the salary.
fn signed_terms(p: &mut Player, offer: &Offer, focus: Option<Focus>) -> u32 {
    if offer.kind == OfferKind::Contract {
        p.signed_salary_k = p.salary_k.max(offer.salary_k);
        p.signed_bonus_k = p.bonus_k.max(offer.bonus_k);
        p.signed_focus = focus.or(p.focus);
    } else {
        let more = if focus == Some(Focus::Salary) { 1.25 } else { 1.0 };
        p.signed_salary_k = ((offer.salary_k as f32 * more / 50.0).round() as u32 * 50).max(100);
        p.signed_bonus_k = offer.bonus_k;
        p.signed_focus = focus;
    }
    p.signed_salary_k
}

fn catalog_team(year: u16, slot: u8) -> Option<&'static CatalogTeam> {
    catalog_year(year)?.teams.iter().find(|t| t.slot == slot)
}

fn catalog_driver(year: u16, slot: u8, key: &str) -> Option<&'static CatalogDriver> {
    catalog_team(year, slot)?.drivers.iter().find(|d| d.key == key)
}

/// The driver of `team` whose seat a player takes: the one asked for, or the lower-rated one
/// nobody else took.
fn choose_replaced(team: &CatalogTeam, asked: Option<&str>, taken: &[&str]) -> Result<String, LeagueError> {
    match asked {
        Some(key) => {
            if !team.drivers.iter().any(|d| d.key == key) {
                return refuse("bad_driver", format!("{key} doesn't drive for {}", team.name));
            }
            if taken.contains(&key) {
                return refuse("seat_taken", "another player has that seat");
            }
            Ok(key.to_owned())
        }
        None => {
            let mut free: Vec<&CatalogDriver> = team
                .drivers
                .iter()
                .filter(|d| !taken.contains(&d.key.as_str()))
                .collect();
            free.sort_by(|a, b| a.rating.total_cmp(&b.rating));
            free.first().map(|d| d.key.clone()).map_or_else(
                || refuse("team_full", format!("both seats at {} are taken", team.name)),
                Ok,
            )
        }
    }
}

/// The season `year` with its real grid, the players in their seats, and with `lineup_changes`
/// the real mid-season line-up changes that don't touch the players' seats.
fn build_season(year: u16, seats: &[(Player, Seat)], lineup_changes: bool) -> Result<Season, LeagueError> {
    let Some(cat) = catalog_year(year) else {
        return refuse("bad_year", format!("no season data for {year}"));
    };
    for (p, s) in seats {
        if catalog_driver(year, s.slot, &s.replaces).is_none() {
            return refuse("bad_seat", format!("{}'s seat doesn't exist in {year}", p.name));
        }
    }
    let mut entries = Vec::new();
    for t in &cat.teams {
        for d in &t.drivers {
            match seats
                .iter()
                .find(|(_, s)| s.slot == t.slot && s.replaces == d.key)
            {
                Some((p, _)) => entries.push(Entry {
                    id: format!("p:{}", p.id),
                    name: p.name.clone(),
                    number: p.number,
                    slot: t.slot,
                    rating: p.rating,
                    player: Some(p.id.clone()),
                }),
                None => entries.push(Entry {
                    id: d.key.clone(),
                    name: d.name.clone(),
                    number: d.number,
                    slot: t.slot,
                    rating: d.rating,
                    player: None,
                }),
            }
        }
    }
    let (changes, reserves) = if lineup_changes {
        lineup::changes_for(year, seats, 1)
    } else {
        (Vec::new(), Vec::new())
    };
    Ok(Season {
        year,
        teams: cat
            .teams
            .iter()
            .map(|t| Team {
                slot: t.slot,
                name: t.name.clone(),
                code: t.code.clone(),
                pace: t.pace,
                carry: 0.0,
            })
            .collect(),
        entries,
        rounds: cat
            .calendar
            .iter()
            .map(|r| Round {
                round: r.round,
                circuit: r.circuit.clone(),
                race: r.race.clone(),
                track: r.track,
                sprint: r.sprint,
                state: RoundState::Pending,
                race_result: Vec::new(),
                sprint_result: Vec::new(),
                quali_result: Vec::new(),
                sessions: Vec::new(),
                by: None,
                auto: false,
                practice: Vec::new(),
                before: None,
            })
            .collect(),
        offers: Vec::new(),
        reserves,
        changes: Some(changes),
        upgrades: Vec::new(),
    })
}

/// Next season's offers: the player's own team, and up to three of the best teams their rating
/// reaches (any team for the champion). A player whose contract runs on stays with their team
/// unless one of the teams faster than theirs buys them out. The best team reached offers one
/// year, the next ones two and three; the own team two years after a season above the car's
/// level, else one.
fn make_offers(players: &[Player], season: &Season, drivers: &[DriverStanding]) -> Vec<Offer> {
    let next = season.year + 1;
    let Some(cat) = catalog_year(next) else {
        return Vec::new();
    };
    let mut ranked: Vec<&CatalogTeam> = cat.teams.iter().collect();
    ranked.sort_by(|a, b| a.pace.total_cmp(&b.pace).then(a.slot.cmp(&b.slot)));
    let mut offers = Vec::new();
    for p in players {
        let champion = drivers.first().and_then(|d| d.player.as_deref()) == Some(p.id.as_str());
        let reach = if champion {
            1
        } else {
            ((11.0 - (p.rating - 70.0) / 2.5).ceil() as usize).clamp(1, ranked.len())
        };
        let own = season
            .player_entry(&p.id)
            .map(|e| e.slot)
            .filter(|&slot| cat.teams.iter().any(|t| t.slot == slot));
        let bound = own.is_some() && p.contract_until.is_some_and(|y| y >= next);
        let own_rank = own.map_or(ranked.len(), |slot| {
            ranked.iter().position(|t| t.slot == slot).unwrap_or(ranked.len())
        });
        let teams = ranked.len();
        let titles = p.titles.len();
        let renew = |slot: u8, years: u8| {
            let (salary_k, bonus_k) =
                progress::salary(p.rating, titles, own_rank + 1, teams, Some(p.recognition));
            Offer {
                player: p.id.clone(),
                slot,
                kind: OfferKind::Renew,
                years,
                salary_k,
                bonus_k,
            }
        };
        let mut mine = Vec::new();
        if let Some(slot) = own {
            let place = drivers
                .iter()
                .position(|d| d.player.as_deref() == Some(p.id.as_str()))
                .map_or(drivers.len(), |i| i + 1);
            let expected = 2 * season.pace_rank(slot);
            if bound {
                mine.push(Offer {
                    player: p.id.clone(),
                    slot,
                    kind: OfferKind::Contract,
                    years: (p.contract_until.unwrap_or(next) - next + 1) as u8,
                    salary_k: p.salary_k,
                    bonus_k: p.bonus_k,
                });
            } else if p.recognition >= progress::RECOGNITION_RENEW {
                // A team that doesn't value its driver lets the contract run out.
                mine.push(renew(slot, if place <= expected { 2 } else { 1 }));
            }
        }
        let moves = ranked
            .iter()
            .enumerate()
            .skip(reach - 1)
            .filter(|(_, t)| Some(t.slot) != own)
            // Under contract, only a faster team buys a player out.
            .filter(|(rank, _)| !bound || *rank < own_rank)
            .take(3)
            .enumerate()
            .map(|(i, (rank, t))| {
                let (salary_k, bonus_k) = progress::salary(p.rating, titles, rank + 1, teams, None);
                Offer {
                    player: p.id.clone(),
                    slot: t.slot,
                    kind: OfferKind::Move,
                    years: i as u8 + 1,
                    // A team that wants a driver pays a little more.
                    salary_k: salary_k / 50 * 50 + salary_k / 20 / 50 * 50,
                    bonus_k,
                }
            });
        mine.extend(moves);
        // Nobody is left without a seat: the own team keeps them after all.
        if mine.is_empty()
            && let Some(slot) = own
        {
            mine.push(renew(slot, 1));
        }
        offers.extend(mine);
    }
    offers
}

// ---------------------------------------------------------------------------------------------
// Matching a reported race onto the grid

/// Lower-case words of a name, accents folded.
fn words(name: &str) -> Vec<String> {
    let folded: String = name
        .chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'ö' | 'õ' | 'ø' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ñ' => 'n',
            'ç' => 'c',
            c if c.is_alphanumeric() => c,
            _ => ' ',
        })
        .collect();
    folded.split_whitespace().map(str::to_owned).collect()
}

/// Whether a name the game shows (often just the surname, upper case) is this driver.
fn same_driver(shown: &str, name: &str) -> bool {
    let shown = words(shown);
    !shown.is_empty() && words(name).iter().any(|w| w.len() >= 3 && shown.contains(w))
}

fn finish(status: u8) -> Finish {
    match status {
        2 | 3 => Finish::Finished,
        5 => Finish::Dsq,
        0 | 1 => Finish::Dns,
        _ => Finish::Dnf,
    }
}

/// The report as round `round`'s classification: the reporter's car is their entry, other human
/// cars are the other players (by name, or the only one left), AI cars are matched by race
/// number and name to the seat they start the season in, and count for whoever has that seat in
/// the round (see [`lineup`]).
fn classify(
    season: &Season,
    reporter: Option<&str>,
    report: &RaceReport,
    kind: SessionKind,
    year: u16,
    rule: PointsRule,
    round: u8,
) -> (Vec<Classified>, Vec<String>) {
    let mut cars: Vec<&ReportCar> = report.cars.iter().filter(|c| c.position > 0).collect();
    cars.sort_by_key(|c| c.position);
    let mut used: HashSet<&str> = HashSet::new();
    let mut matched: Vec<Option<&Entry>> = vec![None; cars.len()];
    let humans: Vec<&Entry> = season.entries.iter().filter(|e| e.player.is_some()).collect();
    let ai: Vec<&Entry> = season
        .entries
        .iter()
        .chain(&season.reserves)
        .filter(|e| e.player.is_none())
        .collect();
    // The reporter's own car first.
    if let Some(me) = reporter.and_then(|id| season.player_entry(id))
        && let Some(i) = cars.iter().position(|c| c.mine)
    {
        matched[i] = Some(me);
        used.insert(me.id.as_str());
    }
    // Other humans by name, then by elimination.
    for (i, c) in cars.iter().enumerate() {
        if matched[i].is_some() || c.ai {
            continue;
        }
        if let Some(e) = humans
            .iter()
            .copied()
            .find(|e| !used.contains(e.id.as_str()) && same_driver(&c.name, &e.name))
        {
            matched[i] = Some(e);
            used.insert(e.id.as_str());
        }
    }
    let open_cars: Vec<usize> = (0..cars.len())
        .filter(|&i| matched[i].is_none() && !cars[i].ai)
        .collect();
    let open_humans: Vec<&Entry> = humans
        .iter()
        .copied()
        .filter(|e| !used.contains(e.id.as_str()))
        .collect();
    if open_cars.len() == 1 && open_humans.len() == 1 {
        matched[open_cars[0]] = Some(open_humans[0]);
        used.insert(open_humans[0].id.as_str());
    }
    // AI cars: number and name, then number alone, then name alone.
    for pass in 0..3 {
        for (i, c) in cars.iter().enumerate() {
            if matched[i].is_some() || !c.ai {
                continue;
            }
            let found: Vec<&Entry> = ai
                .iter()
                .copied()
                .filter(|e| !used.contains(e.id.as_str()))
                .filter(|e| match pass {
                    0 => e.number == c.number && same_driver(&c.name, &e.name),
                    1 => e.number == c.number,
                    _ => same_driver(&c.name, &e.name),
                })
                .collect();
            if let [e] = found[..] {
                matched[i] = Some(e);
                used.insert(e.id.as_str());
            }
        }
    }
    // Each AI car counts for the driver of its seat in this round.
    let mut holders: HashSet<&str> = HashSet::new();
    for m in matched.iter_mut() {
        let Some(e) = *m else { continue };
        let holder = if e.player.is_some() {
            Some(e)
        } else {
            season.seat_holder(e, round)
        };
        *m = holder.filter(|h| holders.insert(h.id.as_str()));
    }
    let fastest = cars
        .iter()
        .enumerate()
        .filter(|(_, c)| c.best_lap_ms > 0)
        .min_by_key(|(_, c)| c.best_lap_ms)
        .map(|(i, _)| i);
    let mut warnings = Vec::new();
    let mut out = Vec::new();
    for (i, c) in cars.iter().enumerate() {
        let Some(e) = matched[i] else {
            warnings.push(format!(
                "P{} {} (#{}) matched no driver of {} and isn't counted",
                c.position,
                if c.name.is_empty() { "?" } else { &c.name },
                c.number,
                season.year
            ));
            continue;
        };
        let status = finish(c.status);
        let is_fastest = matches!(kind, SessionKind::Race | SessionKind::Sprint) && fastest == Some(i);
        let pts = if status == Finish::Finished {
            points(year, rule, kind, c.position, is_fastest)
        } else {
            0.0
        };
        out.push(Classified {
            entry: e.id.clone(),
            position: c.position,
            grid: c.grid,
            laps: c.laps,
            status,
            best_lap_ms: c.best_lap_ms,
            time_ms: c.time_ms,
            penalty_s: c.penalty_s,
            points: pts,
            fastest: is_fastest,
        });
    }
    (out, warnings)
}

/// "Round 3, Bahrain Grand Prix: Lewis Hamilton wins; Anna P4, Ben P9."
fn result_line(
    season: &Season,
    round: &Round,
    kind: SessionKind,
    result: &[Classified],
    players: &[Player],
) -> String {
    let name = |id: &str| season.entry(id).map_or(id.to_owned(), |e| e.name.clone());
    let winner = result
        .iter()
        .find(|c| c.position == 1 && c.status == Finish::Finished)
        .map(|c| name(&c.entry));
    let mut parts = Vec::new();
    for p in players {
        let Some(e) = season.player_entry(&p.id) else {
            continue;
        };
        match result.iter().find(|c| c.entry == e.id) {
            Some(c) if c.position == 1 && c.status == Finish::Finished => {}
            Some(c) if c.status == Finish::Finished => parts.push(format!("{} P{}", p.name, c.position)),
            Some(_) if kind == SessionKind::Qualifying => parts.push(format!("{} no time", p.name)),
            Some(_) => parts.push(format!("{} out", p.name)),
            None => {}
        }
    }
    let what = match kind {
        SessionKind::Race => "",
        SessionKind::Sprint => " sprint",
        SessionKind::Qualifying => " qualifying",
        SessionKind::Practice => " practice",
    };
    let mut line = format!("Round {}, {}{what}: ", round.round, round.race);
    match (winner, kind) {
        (Some(w), SessionKind::Qualifying) => line.push_str(&format!("{w} on pole")),
        (Some(w), _) => line.push_str(&format!("{w} wins")),
        (None, SessionKind::Qualifying) => line.push_str("no pole time"),
        (None, _) => line.push_str("no winner classified"),
    }
    if !parts.is_empty() {
        line.push_str("; ");
        line.push_str(&parts.join(", "));
    }
    line.push('.');
    line
}

#[cfg(test)]
mod tests;
