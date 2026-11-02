//! The weekend (W2-8): the rules sheet the players agree on, the check of the game's settings
//! against it from the telemetry, the sprint shootout and the session formats.
//!
//! - [`validate`]: the rules the owner sets (`Action::SetOptions`) are values the game has.
//! - [`check`]: each counted report's settings bytes (from the session packet, see
//!   [`udp`](super::udp)) and session type against the agreed rules. A difference warns; the
//!   session counts all the same.
//! - [`shootout`]: a sprint weekend's sprint shootout (session types 10 to 14) sets the sprint's
//!   grid, with no points.
//! - [`describe`]: the rules sheet as (label, value) lines.

use serde::{Deserialize, Serialize};

use super::texts::{Text, t};
use super::{Actor, Classified, League, LeagueError, RaceReport, Round, SessionKind, classify, refuse};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualiFormat {
    Full,
    Short,
    OneShot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PracticeFormat {
    Full,
    OneSession,
    Short,
    None,
}

/// The weekend's rules the players agreed on. `None` everywhere: not agreed, not checked.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WeekendRules {
    /// 0 to 110.
    pub ai_difficulty: Option<u8>,
    pub quali: Option<QualiFormat>,
    pub practice: Option<PracticeFormat>,
    /// The game's `carDamage` value.
    pub damage: Option<u8>,
    pub safety_car: Option<u8>,
    pub parc_ferme: Option<bool>,
    pub equal_performance: Option<bool>,
    pub weather: Option<u8>,
    pub rule_set: Option<u8>,
}

/// The settings bytes of the game's session packet (F1 24 offsets). 0 can mean "not sent".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GameSettings {
    pub ai_difficulty: u8,
    pub game_mode: u8,
    pub rule_set: u8,
    pub session_length: u8,
    pub equal_performance: u8,
    pub car_damage: u8,
    pub safety_car: u8,
    pub parc_ferme: u8,
    pub weather: u8,
    pub network_game: u8,
    pub formation_lap: u8,
    pub red_flags: u8,
}

/// One lap of the reporting player's, with what the car used on it (practice programmes).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LapTelemetry {
    pub lap: u8,
    pub ms: u32,
    #[serde(default)]
    pub fuel_used_kg: f32,
    #[serde(default)]
    pub ers_deployed_mj: f32,
    #[serde(default)]
    pub ers_store_end_mj: f32,
    #[serde(default)]
    pub tyre_wear_pct: f32,
    #[serde(default)]
    pub valid: bool,
}

/// The sprint shootout's session types: SQ1, SQ2, SQ3, short and one-shot.
const SHOOTOUT: std::ops::RangeInclusive<u8> = 10..=14;

/// What a counted sprint shootout teaches a driver who drove it (the game gives sprint
/// qualifying no resource points, only experience). A seat the game's AI drove gets half.
const SHOOTOUT_EXPERIENCE: f32 = 0.1;

/// The highest AI difficulty the game offers.
pub const AI_DIFFICULTY_MAX: u8 = 110;

/// The game's `ruleSet` values (F1 24 and F1 25 UDP specification).
const RULE_SETS: [(u8, &str); 9] = [
    (0, "Practice and qualifying"),
    (1, "Race"),
    (2, "Time trial"),
    (4, "Time attack"),
    (6, "Checkpoint challenge"),
    (8, "Autocross"),
    (9, "Drift"),
    (10, "Average speed zone"),
    (11, "Rival duel"),
];
/// The game's `carDamage`, `safetyCarSetting` and `weather` values, by their number.
const DAMAGE: [&str; 4] = ["Off", "Reduced", "Standard", "Simulation"];
const SAFETY_CAR: [&str; 4] = ["Off", "Reduced", "Standard", "Increased"];
const WEATHER: [&str; 6] = ["Clear", "Light cloud", "Overcast", "Light rain", "Heavy rain", "Storm"];

/// The labels of the rules sheet (the launcher translates them).
pub const LABEL_AI: &str = "AI difficulty";
pub const LABEL_QUALI: &str = "Qualifying";
pub const LABEL_PRACTICE: &str = "Practice";
pub const LABEL_DAMAGE: &str = "Car damage";
pub const LABEL_SAFETY_CAR: &str = "Safety car";
pub const LABEL_PARC_FERME: &str = "Parc fermé";
pub const LABEL_EQUAL: &str = "Equal car performance";
pub const LABEL_WEATHER: &str = "Weather";
pub const LABEL_RULE_SET: &str = "Rule set";

impl QualiFormat {
    pub fn name(self) -> &'static str {
        match self {
            QualiFormat::Full => "Full (Q1, Q2, Q3)",
            QualiFormat::Short => "Short",
            QualiFormat::OneShot => "One-shot",
        }
    }

    /// The format a qualifying (5 to 9) or sprint shootout (10 to 14) session type belongs to.
    pub fn of_session(session_type: u8) -> Option<QualiFormat> {
        match session_type {
            5..=7 | 10..=12 => Some(QualiFormat::Full),
            8 | 13 => Some(QualiFormat::Short),
            9 | 14 => Some(QualiFormat::OneShot),
            _ => None,
        }
    }
}

impl PracticeFormat {
    pub fn name(self) -> &'static str {
        match self {
            PracticeFormat::Full => "Full (P1, P2, P3)",
            PracticeFormat::OneSession => "One session",
            PracticeFormat::Short => "Short",
            PracticeFormat::None => "None",
        }
    }
}

/// The labels and values [`check`]'s warnings name (the launcher translates them).
pub(super) fn words() -> Vec<&'static str> {
    let mut words = vec![
        LABEL_AI,
        LABEL_QUALI,
        LABEL_PRACTICE,
        LABEL_DAMAGE,
        LABEL_SAFETY_CAR,
        LABEL_PARC_FERME,
        LABEL_EQUAL,
        LABEL_WEATHER,
        LABEL_RULE_SET,
        "On",
        "Off",
    ];
    words.extend([QualiFormat::Full, QualiFormat::Short, QualiFormat::OneShot].map(QualiFormat::name));
    words.extend(
        [PracticeFormat::Full, PracticeFormat::OneSession, PracticeFormat::Short, PracticeFormat::None]
            .map(PracticeFormat::name),
    );
    words.extend(DAMAGE.iter().chain(&SAFETY_CAR).chain(&WEATHER));
    words.extend(RULE_SETS.iter().map(|(_, name)| *name));
    words.sort_unstable();
    words.dedup();
    words
}

fn on_off(on: bool) -> &'static str {
    if on { "On" } else { "Off" }
}

fn rule_set_name(value: u8) -> Option<&'static str> {
    RULE_SETS.iter().find(|(v, _)| *v == value).map(|(_, name)| *name)
}

/// A setting's name from its table, or the raw value for one the table doesn't know.
fn named(table: &[&'static str], value: u8) -> String {
    table
        .get(usize::from(value))
        .map_or_else(|| value.to_string(), |name| (*name).to_owned())
}

fn rule_set_named(value: u8) -> String {
    rule_set_name(value).map_or_else(|| value.to_string(), str::to_owned)
}

/// Called by `report()` before the session's kind is guessed. A sprint shootout (10 to 14) of a
/// round whose sprint is still open sets the sprint's grid: its classification is kept as the
/// round's `shootout_result`, with no points, and the drivers in it gain a little experience.
/// A later part of the shootout (SQ3 after SQ1) only updates the grid. `Ok(None)` for anything
/// else, which `report()` handles as before (a shootout without an open sprint isn't counted).
pub(super) fn shootout(
    league: &mut League,
    n: u8,
    actor: &Actor,
    report: &RaceReport,
) -> Result<Option<Vec<Text>>, LeagueError> {
    if !SHOOTOUT.contains(&report.session_type) {
        return Ok(None);
    }
    let Some(r) = league.season().round(n).cloned() else {
        return Ok(None);
    };
    if !(r.sprint && league.setup.sprints && r.sprint_result.is_empty()) {
        return Ok(None);
    }
    let again = !r.shootout_result.is_empty();
    if again {
        let stored = r
            .counted
            .iter()
            .filter(|c| SHOOTOUT.contains(&c.session_type))
            .map(|c| c.session_type)
            .max()
            .unwrap_or(0);
        if report.session_type < stored {
            return Ok(Some(vec![t!("a later part of this sprint shootout is counted already")]));
        }
    }
    let mine = league
        .player(&actor.id)
        .filter(|p| p.seat.is_some())
        .map(|p| p.id.clone());
    let (mut result, mut warnings) = classify(
        league.season(),
        &league.players,
        mine.as_deref(),
        report,
        SessionKind::Qualifying,
        league.year(),
        league.setup.points,
        n,
    );
    if result.is_empty() {
        return refuse(
            "no_match",
            t!("none of the cars in this sprint shootout belong to this season's grid"),
        );
    }
    for c in &mut result {
        c.points = 0.0;
        c.fastest = false;
    }
    warnings.extend(check(&league.options.rules, report));
    let line = shootout_line(league, &r, &result, again);
    if !again {
        league.keep_progress(n);
        let season = league.season();
        let learners: Vec<(String, f32)> = league
            .players
            .iter()
            .filter_map(|p| {
                let entry = season.player_entry(&p.id)?;
                let c = result.iter().find(|c| c.entry == entry.id)?;
                Some((p.id.clone(), if c.ai_driven { 0.5 } else { 1.0 }))
            })
            .collect();
        for (id, weight) in learners {
            league.train(&id, |a| a.experience += SHOOTOUT_EXPERIENCE * weight);
        }
        league.sync_ratings();
    }
    let round = league
        .season_mut()
        .rounds
        .iter_mut()
        .find(|x| x.round == n)
        .expect("the round");
    round.shootout_result = result;
    if report.session != 0 && !round.sessions.contains(&report.session) {
        round.sessions.push(report.session);
    }
    // `SessionKind` has no shootout; the session type (10 to 14) tells it from the sprint.
    league.count_session(n, actor, report, SessionKind::Sprint);
    league.say(n, line);
    let mut out = vec![t!("sprint shootout counted")];
    out.extend(warnings);
    Ok(Some(out))
}

/// "Round 6, Miami Grand Prix sprint shootout: Lando Norris on sprint pole; Anna P4."
fn shootout_line(league: &League, round: &Round, result: &[Classified], again: bool) -> Text {
    let season = league.season();
    let name = |id: &str| season.entry(id).map_or_else(|| id.to_owned(), |e| e.name.clone());
    let (n, race) = (round.round, &round.race);
    let head = match (result.iter().find(|c| c.position == 1), again) {
        (Some(c), false) => t!(
            "Round {n}, {race} sprint shootout: {winner} on sprint pole",
            n,
            race,
            winner = name(&c.entry)
        ),
        (Some(c), true) => t!(
            "Round {n}, {race} sprint shootout updated: {winner} on sprint pole",
            n,
            race,
            winner = name(&c.entry)
        ),
        (None, false) => t!("Round {n}, {race} sprint shootout: no sprint pole time", n, race),
        (None, true) => t!("Round {n}, {race} sprint shootout updated: no sprint pole time", n, race),
    };
    let mut placed: Vec<(u8, &str)> = league
        .players
        .iter()
        .filter_map(|p| {
            let e = season.player_entry(&p.id)?;
            let c = result.iter().find(|c| c.entry == e.id)?;
            (c.position != 1).then_some((c.position, p.name.as_str()))
        })
        .collect();
    placed.sort_unstable();
    let parts: Vec<Text> = placed
        .iter()
        .map(|(position, name)| t!("{name} P{position}", name, position))
        .collect();
    if parts.is_empty() {
        t!("{result}.").list("result", vec![head])
    } else {
        t!("{result}; {players}.")
            .list("result", vec![head])
            .list("players", parts)
    }
}

/// Whether `rules` can be agreed on (`Action::SetOptions`): values the game has. Refuses
/// `bad_rules`.
pub(super) fn validate(rules: &WeekendRules) -> Result<(), LeagueError> {
    let within = |value: Option<u8>, len: usize| value.is_none_or(|v| usize::from(v) < len);
    if rules.ai_difficulty.is_some_and(|v| v > AI_DIFFICULTY_MAX) {
        return refuse(
            "bad_rules",
            t!("the AI difficulty goes from 0 to {max}", max = AI_DIFFICULTY_MAX),
        );
    }
    if !within(rules.damage, DAMAGE.len()) {
        return refuse(
            "bad_rules",
            t!("car damage is off (0), reduced (1), standard (2) or simulation (3)"),
        );
    }
    if !within(rules.safety_car, SAFETY_CAR.len()) {
        return refuse(
            "bad_rules",
            t!("the safety car is off (0), reduced (1), standard (2) or increased (3)"),
        );
    }
    if !within(rules.weather, WEATHER.len()) {
        return refuse("bad_rules", t!("the weather goes from clear (0) to storm (5)"));
    }
    if rules.rule_set.is_some_and(|v| rule_set_name(v).is_none()) {
        return refuse("bad_rules", t!("the game has no such rule set"));
    }
    Ok(())
}

/// Warnings for what the game ran in `report` that differs from the agreed `rules`. Fields not
/// agreed aren't checked, nor settings bytes of 0: the game may not have sent them (0 also means
/// "off", so an agreed "on" is checked only when the game sends something else). The qualifying
/// format is checked against the session type, the practice format against the session type and
/// the weekend structure. The session counts all the same.
pub fn check(rules: &WeekendRules, report: &RaceReport) -> Vec<Text> {
    let mut warnings = Vec::new();
    let mut differs = |label: &str, ran: String, agreed: String| {
        warnings.push(t!("{label}: the game ran {ran}, the agreed rules say {agreed}", label, ran, agreed));
    };
    if let Some(s) = report.settings {
        let sent = |v: u8| (v != 0).then_some(v);
        if let (Some(want), Some(got)) = (rules.ai_difficulty, sent(s.ai_difficulty))
            && want != got
        {
            differs(LABEL_AI, got.to_string(), want.to_string());
        }
        if let (Some(want), Some(got)) = (rules.damage, sent(s.car_damage))
            && want != got
        {
            differs(LABEL_DAMAGE, named(&DAMAGE, got), named(&DAMAGE, want));
        }
        if let (Some(want), Some(got)) = (rules.safety_car, sent(s.safety_car))
            && want != got
        {
            differs(LABEL_SAFETY_CAR, named(&SAFETY_CAR, got), named(&SAFETY_CAR, want));
        }
        if let (Some(want), Some(got)) = (rules.parc_ferme, sent(s.parc_ferme))
            && want != (got == 1)
        {
            differs(LABEL_PARC_FERME, on_off(got == 1).into(), on_off(want).into());
        }
        if let (Some(want), Some(got)) = (rules.equal_performance, sent(s.equal_performance))
            && want != (got == 1)
        {
            differs(LABEL_EQUAL, on_off(got == 1).into(), on_off(want).into());
        }
        if let (Some(want), Some(got)) = (rules.weather, sent(s.weather))
            && want != got
        {
            differs(LABEL_WEATHER, named(&WEATHER, got), named(&WEATHER, want));
        }
        if let (Some(want), Some(got)) = (rules.rule_set, sent(s.rule_set))
            && want != got
        {
            differs(LABEL_RULE_SET, rule_set_named(got), rule_set_named(want));
        }
    }
    if let (Some(want), Some(got)) = (rules.quali, QualiFormat::of_session(report.session_type))
        && want != got
    {
        differs(LABEL_QUALI, got.name().into(), want.name().into());
    }
    if let Some(want) = rules.practice
        && let Some(ran) = practice_ran(report)
        && !ran.contains(&want)
    {
        differs(LABEL_PRACTICE, ran[0].name().into(), want.name().into());
    }
    warnings
}

/// The practice formats a practice session (1 to 4) fits; `None` for other sessions.
fn practice_ran(report: &RaceReport) -> Option<Vec<PracticeFormat>> {
    let practices = report.weekend.iter().filter(|t| (1..=3).contains(*t)).count();
    Some(match report.session_type {
        4 => vec![PracticeFormat::Short],
        2 | 3 => vec![PracticeFormat::Full],
        1 if practices >= 2 => vec![PracticeFormat::Full],
        1 if practices == 1 => vec![PracticeFormat::OneSession],
        // P1 without a weekend structure: of three sessions, or the only one.
        1 => vec![PracticeFormat::Full, PracticeFormat::OneSession],
        _ => return None,
    })
}

/// The agreed rules as (label, value) lines for the rules sheet, in a fixed order. Rules not
/// agreed are left out, so an empty sheet means nothing is agreed. Labels are the `LABEL_*`
/// constants; values are numbers or the game's English option names.
pub fn describe(rules: &WeekendRules) -> Vec<(&'static str, String)> {
    let mut lines = Vec::new();
    if let Some(v) = rules.ai_difficulty {
        lines.push((LABEL_AI, v.to_string()));
    }
    if let Some(v) = rules.quali {
        lines.push((LABEL_QUALI, v.name().to_owned()));
    }
    if let Some(v) = rules.practice {
        lines.push((LABEL_PRACTICE, v.name().to_owned()));
    }
    if let Some(v) = rules.damage {
        lines.push((LABEL_DAMAGE, named(&DAMAGE, v)));
    }
    if let Some(v) = rules.safety_car {
        lines.push((LABEL_SAFETY_CAR, named(&SAFETY_CAR, v)));
    }
    if let Some(v) = rules.parc_ferme {
        lines.push((LABEL_PARC_FERME, on_off(v).to_owned()));
    }
    if let Some(v) = rules.equal_performance {
        lines.push((LABEL_EQUAL, on_off(v).to_owned()));
    }
    if let Some(v) = rules.weather {
        lines.push((LABEL_WEATHER, named(&WEATHER, v)));
    }
    if let Some(v) = rules.rule_set {
        lines.push((LABEL_RULE_SET, rule_set_named(v)));
    }
    lines
}
