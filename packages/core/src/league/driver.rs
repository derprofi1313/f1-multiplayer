//! The driver (W2-5): the creation profile (nationality, archetype), the focus attribute and the
//! AI drivers' in-season rating development.
//!
//! - **Profile.** A player picks a nationality and an archetype before their first counted
//!   session. The archetype sets the starting rating (70, 78 or 84) and the focus.
//! - **Focus.** A fifth rating, outside the overall. It grows with clean races and drops with
//!   races the player didn't finish; in a simulated race it makes a retirement less likely.
//! - **AI form.** Each AI driver's rating moves during a season with how they finish against
//!   where their car and rating should be. A new season starts from the season data again,
//!   with a quarter of the form carried by drivers who stay with their team.
//!
//! The form is kept as each driver's total in [`Season::ai_form`], counted through a round
//! (`Form::through`). It is always computed again from the season's results, so taking a round
//! back and counting it again gives the same form, and a simulation of a round taken back uses
//! the form from before that round.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::progress::{ATTRIBUTE_MAX, ATTRIBUTE_MIN, Attributes};
use super::texts::{Text, t};
use super::{
    Actor, Entry, Finish, League, LeagueError, Phase, Player, Round, RoundState, START_RATING, Season, refuse,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Archetype {
    /// Starts rated 70.
    Rookie,
    /// Starts rated 78.
    Experienced,
    /// Starts rated 84.
    Veteran,
}

impl Archetype {
    pub fn name(self) -> &'static str {
        match self {
            Archetype::Rookie => "rookie",
            Archetype::Experienced => "experienced driver",
            Archetype::Veteran => "veteran",
        }
    }
}

/// An AI driver's rating change in a season.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Form {
    pub entry: String,
    /// The change from this season's races, within ±[`FORM_CAP`].
    pub delta: f32,
    /// What the driver carried over from last season at the same team.
    #[serde(default, skip_serializing_if = "is_zero_f32")]
    pub carry: f32,
    /// The last round whose race `delta` counts (0: none yet).
    #[serde(default, skip_serializing_if = "is_zero_u8")]
    pub through: u8,
}

impl Form {
    /// The whole change to the driver's rating from the season data.
    pub fn total(&self) -> f32 {
        self.carry + self.delta
    }
}

fn is_zero_f32(x: &f32) -> bool {
    *x == 0.0
}

fn is_zero_u8(x: &u8) -> bool {
    *x == 0
}

/// How far an AI driver's form moves in a season, either way.
pub const FORM_CAP: f32 = 4.0;
/// How far one race moves it, either way.
pub const FORM_STEP: f32 = 0.3;
/// What a place better (or worse) than expected moves it.
pub const FORM_PER_PLACE: f32 = 0.05;
/// The share of the form a driver who stays with the team keeps into next season.
pub const FORM_CARRY: f32 = 0.25;

/// Focus gained with a clean race (finished, no penalty).
pub const FOCUS_CLEAN: f32 = 0.2;
/// Focus lost with a race the player didn't finish.
pub const FOCUS_DNF: f32 = 0.3;

/// ISO 3166-1 alpha-3 codes and English names a player can pick, by name.
pub const NATIONALITIES: &[(&str, &str)] = &[
    ("ARG", "Argentina"),
    ("AUS", "Australia"),
    ("AUT", "Austria"),
    ("AZE", "Azerbaijan"),
    ("BHR", "Bahrain"),
    ("BEL", "Belgium"),
    ("BOL", "Bolivia"),
    ("BRA", "Brazil"),
    ("BGR", "Bulgaria"),
    ("CAN", "Canada"),
    ("CHL", "Chile"),
    ("CHN", "China"),
    ("COL", "Colombia"),
    ("CRI", "Costa Rica"),
    ("HRV", "Croatia"),
    ("CYP", "Cyprus"),
    ("CZE", "Czechia"),
    ("DNK", "Denmark"),
    ("DOM", "Dominican Republic"),
    ("ECU", "Ecuador"),
    ("EGY", "Egypt"),
    ("EST", "Estonia"),
    ("FIN", "Finland"),
    ("FRA", "France"),
    ("DEU", "Germany"),
    ("GHA", "Ghana"),
    ("GRC", "Greece"),
    ("GTM", "Guatemala"),
    ("HKG", "Hong Kong"),
    ("HUN", "Hungary"),
    ("ISL", "Iceland"),
    ("IND", "India"),
    ("IDN", "Indonesia"),
    ("IRL", "Ireland"),
    ("ISR", "Israel"),
    ("ITA", "Italy"),
    ("JAM", "Jamaica"),
    ("JPN", "Japan"),
    ("JOR", "Jordan"),
    ("KAZ", "Kazakhstan"),
    ("KEN", "Kenya"),
    ("KWT", "Kuwait"),
    ("LVA", "Latvia"),
    ("LBN", "Lebanon"),
    ("LIE", "Liechtenstein"),
    ("LTU", "Lithuania"),
    ("LUX", "Luxembourg"),
    ("MYS", "Malaysia"),
    ("MLT", "Malta"),
    ("MEX", "Mexico"),
    ("MCO", "Monaco"),
    ("MAR", "Morocco"),
    ("NLD", "Netherlands"),
    ("NZL", "New Zealand"),
    ("NGA", "Nigeria"),
    ("NOR", "Norway"),
    ("OMN", "Oman"),
    ("PAK", "Pakistan"),
    ("PAN", "Panama"),
    ("PRY", "Paraguay"),
    ("PER", "Peru"),
    ("PHL", "Philippines"),
    ("POL", "Poland"),
    ("PRT", "Portugal"),
    ("QAT", "Qatar"),
    ("ROU", "Romania"),
    ("RUS", "Russia"),
    ("SMR", "San Marino"),
    ("SAU", "Saudi Arabia"),
    ("SRB", "Serbia"),
    ("SGP", "Singapore"),
    ("SVK", "Slovakia"),
    ("SVN", "Slovenia"),
    ("ZAF", "South Africa"),
    ("KOR", "South Korea"),
    ("ESP", "Spain"),
    ("LKA", "Sri Lanka"),
    ("SWE", "Sweden"),
    ("CHE", "Switzerland"),
    ("TWN", "Taiwan"),
    ("THA", "Thailand"),
    ("TUR", "Turkey"),
    ("UKR", "Ukraine"),
    ("ARE", "United Arab Emirates"),
    ("GBR", "United Kingdom"),
    ("USA", "United States"),
    ("URY", "Uruguay"),
    ("VEN", "Venezuela"),
    ("VNM", "Vietnam"),
    ("ZWE", "Zimbabwe"),
];

/// The English name of a nationality code.
pub fn nationality_name(code: &str) -> Option<&'static str> {
    NATIONALITIES.iter().find(|(c, _)| *c == code).map(|(_, n)| *n)
}

/// The rating a new player starts with.
pub fn start_rating(a: Option<Archetype>) -> f32 {
    match a {
        Some(Archetype::Rookie) => 70.0,
        Some(Archetype::Experienced) => 78.0,
        Some(Archetype::Veteran) => 84.0,
        None => START_RATING,
    }
}

/// A player's focus: their own once they have one, else (a player of an older career) their
/// rating.
pub fn focus_of(player: &Player) -> f32 {
    player.attributes.and_then(|a| a.focus).unwrap_or(player.rating)
}

/// Whether `player` has a session counted in the career (practice, qualifying, a sprint or a
/// race, driven or simulated).
pub fn has_driven(league: &League, player: &str) -> bool {
    let entry = format!("p:{player}");
    league.seasons.iter().flat_map(|s| &s.rounds).any(|r| {
        r.practice.iter().any(|l| l.player == player)
            || [
                &r.quali_result,
                &r.sprint_result,
                &r.race_result,
                &r.shootout_result,
            ]
            .iter()
            .any(|list| list.iter().any(|c| c.entry == entry))
    })
}

/// `Action::Profile`: the player's nationality and archetype, until their first counted session.
/// `None` leaves a part as it is; an empty nationality clears it.
pub(super) fn profile(
    league: &mut League,
    actor: &Actor,
    nationality: Option<String>,
    archetype: Option<Archetype>,
) -> Result<Vec<Text>, LeagueError> {
    if league.player(&actor.id).is_none() {
        return refuse("not_player", t!("join the career first"));
    }
    if league.phase == Phase::Finished {
        return refuse("phase", t!("the career is complete"));
    }
    if has_driven(league, &actor.id) {
        return refuse("phase", t!("your driver is set once a session of yours is counted"));
    }
    let nationality = match nationality.map(|n| n.trim().to_ascii_uppercase()) {
        None => None,
        Some(n) if n.is_empty() => Some(None),
        Some(n) => match nationality_name(&n) {
            Some(_) => Some(Some(n)),
            None => return refuse("bad_profile", t!("{n} isn't a nationality to pick", n)),
        },
    };
    if nationality.is_none() && archetype.is_none() {
        return Ok(Vec::new());
    }
    let first_season = league.seasons.len() == 1 && league.phase == Phase::Racing;
    let seated = league.season().player_entry(&actor.id).is_some();
    let p = league.player_mut(&actor.id).expect("checked");
    if let Some(n) = nationality {
        p.nationality = n;
    }
    if let Some(a) = archetype {
        let rating = start_rating(Some(a));
        p.archetype = Some(a);
        let mut attributes = Attributes::from_rating(rating);
        attributes.focus = Some(rating);
        p.attributes = Some(attributes);
        p.rating = rating;
        // The first contract was set with the rating the career started with.
        if first_season && seated {
            p.salary_k = 0;
            p.bonus_k = 0;
        }
    }
    let p = league.player(&actor.id).expect("checked");
    let (name, rating) = (&p.name, format!("{:.0}", p.rating));
    let text = match (p.archetype, p.nationality.as_deref().and_then(nationality_name)) {
        (Some(Archetype::Experienced), Some(country)) => {
            t!("{name} races as an experienced driver from {country}, rated {rating}.", name, country, rating)
        }
        (Some(Archetype::Rookie), Some(country)) => {
            t!("{name} races as a rookie from {country}, rated {rating}.", name, country, rating)
        }
        (Some(Archetype::Veteran), Some(country)) => {
            t!("{name} races as a veteran from {country}, rated {rating}.", name, country, rating)
        }
        (Some(Archetype::Experienced), None) => t!("{name} races as an experienced driver, rated {rating}.", name, rating),
        (Some(Archetype::Rookie), None) => t!("{name} races as a rookie, rated {rating}.", name, rating),
        (Some(Archetype::Veteran), None) => t!("{name} races as a veteran, rated {rating}.", name, rating),
        (None, Some(country)) => t!("{name} races from {country}.", name, country),
        (None, None) => t!("{name} races.", name),
    };
    league.sync_ratings();
    if archetype.is_some() && first_season && seated {
        league.first_salary();
    }
    league.say(0, text);
    Ok(Vec::new())
}

/// Called after round `n` was completed: the AI drivers' form and the players' focus.
pub(super) fn after_round(league: &mut League, n: u8) {
    update_form(league.season_mut(), n);
    train_focus(league, n);
}

/// The form of every AI driver through round `n`, computed again from the season's results.
fn update_form(season: &mut Season, n: u8) {
    let form = history(season, n);
    let carry: HashMap<&str, f32> = season
        .ai_form
        .iter()
        .filter(|f| f.carry != 0.0)
        .map(|f| (f.entry.as_str(), f.carry))
        .collect();
    let mut forms: Vec<Form> = season
        .entries
        .iter()
        .chain(&season.reserves)
        .filter(|e| e.player.is_none())
        .filter_map(|e| {
            let delta = form.get(&e.id).copied();
            let carry = carry.get(e.id.as_str()).copied().unwrap_or(0.0);
            (delta.is_some() || carry != 0.0).then(|| Form {
                entry: e.id.clone(),
                delta: delta.unwrap_or(0.0),
                carry,
                through: n,
            })
        })
        .collect();
    forms.sort_by(|a, b| a.entry.cmp(&b.entry));
    forms.dedup_by(|a, b| a.entry == b.entry);
    season.ai_form = forms;
}

/// A player's focus moves with the race of round `n` they drove: up with a clean finish, down
/// when they didn't finish (a car that stopped before the last lap). Simulated races and seats
/// the game's AI drove don't count.
fn train_focus(league: &mut League, n: u8) {
    let season = league.season();
    let Some(round) = season.round(n) else {
        return;
    };
    if round.state != RoundState::Raced {
        return;
    }
    let total = round
        .race_result
        .iter()
        .filter(|c| c.status == Finish::Finished)
        .map(|c| c.laps)
        .max()
        .unwrap_or(0);
    let mut moves = Vec::new();
    for p in &league.players {
        let Some(entry) = season.player_entry(&p.id) else {
            continue;
        };
        let Some(c) = round.race_result.iter().find(|c| c.entry == entry.id) else {
            continue;
        };
        if c.ai_driven {
            continue;
        }
        let by = match c.status {
            Finish::Finished if c.penalty_s == 0 => FOCUS_CLEAN,
            Finish::Dnf if total == 0 || c.laps < total => -FOCUS_DNF,
            _ => continue,
        };
        moves.push((p.id.clone(), by));
    }
    for (id, by) in moves {
        league.train(&id, |a| {
            if let Some(f) = a.focus.as_mut() {
                *f += by;
            }
        });
    }
}

/// Called when a new season was added: a quarter of last season's form stays with the drivers
/// who drive for the same team again.
pub(super) fn new_season(league: &mut League) {
    let [.., last, next] = league.seasons.as_mut_slice() else {
        return;
    };
    let final_round = last.rounds.last().map_or(u8::MAX, |r| r.round);
    let mut forms: Vec<Form> = next
        .entries
        .iter()
        .chain(&next.reserves)
        .filter(|e| e.player.is_none())
        .filter_map(|e| {
            let total = last
                .ai_form
                .iter()
                .find(|f| f.entry == e.id)
                .map_or(0.0, |f| f.total());
            let stays = last.slot_at(&e.id, final_round) == Some(e.slot);
            let carry = round2(total * FORM_CARRY);
            (stays && carry != 0.0).then(|| Form {
                entry: e.id.clone(),
                delta: 0.0,
                carry,
                through: 0,
            })
        })
        .collect();
    forms.sort_by(|a, b| a.entry.cmp(&b.entry));
    forms.dedup_by(|a, b| a.entry == b.entry);
    next.ai_form = forms;
}

/// The rating `entry` drives with in `round` (the simulation and the expected places use it):
/// a player's own, an AI driver's from the season data with the form from the races before.
pub(super) fn rating_of(season: &Season, entry: &Entry, round: u8) -> f32 {
    if entry.player.is_some() || season.ai_form.is_empty() {
        return entry.rating;
    }
    let carry = season
        .ai_form
        .iter()
        .find(|f| f.entry == entry.id)
        .map_or(0.0, |f| f.carry);
    let through = season.ai_form.iter().map(|f| f.through).max().unwrap_or(0);
    let delta = if round > through {
        season
            .ai_form
            .iter()
            .find(|f| f.entry == entry.id)
            .map_or(0.0, |f| f.delta)
    } else {
        // A round counted again after an undo (or one in the past): the form from before it.
        history(season, round.saturating_sub(1))
            .get(&entry.id)
            .copied()
            .unwrap_or(0.0)
    };
    if carry + delta == 0.0 {
        return entry.rating;
    }
    (entry.rating + carry + delta).clamp(ATTRIBUTE_MIN, ATTRIBUTE_MAX)
}

/// The in-season form of the AI drivers who raced, after the races of rounds up to `upto`.
fn history(season: &Season, upto: u8) -> HashMap<String, f32> {
    let mut form = HashMap::new();
    for r in season
        .rounds
        .iter()
        .filter(|r| r.round <= upto && !r.race_result.is_empty())
    {
        race_form(season, r, &mut form);
    }
    form
}

/// Move the form with the race of round `r`: by a twentieth per place better or worse than
/// the car and the driver's rating then should have finished.
fn race_form(season: &Season, r: &Round, form: &mut HashMap<String, f32>) {
    let carry = |id: &str| {
        season
            .ai_form
            .iter()
            .find(|f| f.entry == id)
            .map_or(0.0, |f| f.carry)
    };
    let lineup = season.lineup(r.round);
    let strength: Vec<(&Entry, f32)> = lineup
        .iter()
        .map(|(e, slot)| {
            let rating = if e.player.is_some() {
                e.rating
            } else {
                let by = carry(&e.id) + form.get(&e.id).copied().unwrap_or(0.0);
                if by == 0.0 {
                    e.rating
                } else {
                    (e.rating + by).clamp(ATTRIBUTE_MIN, ATTRIBUTE_MAX)
                }
            };
            (*e, super::sim::base(season.pace_at(*slot, r.round), rating))
        })
        .collect();
    let mut steps = Vec::new();
    for c in r.race_result.iter().filter(|c| c.status == Finish::Finished) {
        let Some(&(e, mine)) = strength.iter().find(|(e, _)| e.id == c.entry) else {
            continue;
        };
        if e.player.is_some() {
            continue;
        }
        let expected = 1 + strength.iter().filter(|(o, s)| o.id != e.id && *s < mine).count();
        let step = ((expected as f32 - f32::from(c.position)) * FORM_PER_PLACE).clamp(-FORM_STEP, FORM_STEP);
        steps.push((e.id.clone(), step));
    }
    for (id, step) in steps {
        let now = form.entry(id).or_insert(0.0);
        *now = round2((*now + step).clamp(-FORM_CAP, FORM_CAP));
    }
}

fn round2(x: f32) -> f32 {
    (x * 100.0).round() / 100.0
}
