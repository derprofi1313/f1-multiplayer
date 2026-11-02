//! The inbox (W2-4): mails, department events and media questions with choices.
//!
//! After each completed round every active player may get one item (at most one per round),
//! picked in this order:
//! - a media question after a podium or a retirement (humble, bold or deflect);
//! - a flare-up of the rivalry with the team-mate after every third race in a row lost to them;
//! - the engineer's feedback when an upgrade of the team arrives for the next round (no choice);
//! - a department asking for a practice programme at the next race (one round in three, seeded).
//!
//! Every item carries an English `template` (a translation key, listed in [`TEMPLATES`]) with
//! `args` for its `{placeholders}`, and `text` as the English rendering for older launchers.
//! Choice templates have no placeholders. Answering applies the choice's [`Effect`] once.
//!
//! Taking a round back drops the items of that round and later ones; what their answers did goes
//! with the round's snapshot. An answer given after a round's snapshot was kept is written into
//! that snapshot too, so taking that round back (or leaving it out) keeps the answer and what it
//! did together.

use serde::{Deserialize, Serialize};

use super::progress::{ATTRIBUTE_MAX, ATTRIBUTE_MIN, Attributes, Programme, mix};
use super::texts::{Text, t};
use super::{Actor, Finish, League, LeagueError, NEWS_MAX, News, Season, refuse, team_mate_at};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NewsKind {
    #[default]
    News,
    Mail,
    Event,
    Media,
}

/// An answer a mail offers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Choice {
    pub template: String,
    #[serde(default)]
    pub effect: Effect,
}

/// What an answer does to the player.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Effect {
    pub rp: i32,
    pub recognition: f32,
    pub experience: f32,
    pub racecraft: f32,
    pub awareness: f32,
    pub pace: f32,
    pub focus: f32,
}

// The templates: constants named T_*, which a test reads from this file.
pub const T_ENGINEER: &str = "Engineering report: the new {part} parts are on the car from the {race}. The first simulator runs match what the factory promised.";
pub const T_DEPT_TRACK: &str = "The simulator team asks you to put Track acclimatisation first at the {race}. If you agree, they add {rp} resource points of their own work.";
pub const T_DEPT_QUALI: &str = "The aerodynamics department asks you to put Qualifying pace first at the {race}. If you agree, they add {rp} resource points of their own work.";
pub const T_DEPT_TYRES: &str = "The vehicle dynamics group asks you to put Tyre management first at the {race}. If you agree, they add {rp} resource points of their own work.";
pub const T_DEPT_ACCEPT: &str = "Agree to the programme";
pub const T_DEPT_DECLINE: &str = "Decline: you have your own plan";
pub const T_MEDIA_PODIUM: &str = "Media after the {race}: you finished P{position}. A reporter asks whether {team} can fight at the front all season.";
pub const T_MEDIA_PODIUM_HUMBLE: &str = "Stay humble: the team gave you a great car";
pub const T_MEDIA_PODIUM_BOLD: &str = "Be bold: nobody out there was faster than you";
pub const T_MEDIA_DNF: &str =
    "Media after the {race}: your race ended early. A reporter asks what went wrong.";
pub const T_MEDIA_DNF_HUMBLE: &str = "Take it on the chin: the team will find the cause together";
pub const T_MEDIA_DNF_BOLD: &str = "Be bold: you would have been on the podium";
pub const T_MEDIA_DEFLECT: &str = "Deflect: you only think about the next race";
pub const T_RIVALRY: &str = "{mate} has finished ahead of you in the last {count} races, and the press talks of a feud inside {team}.";
pub const T_RIVALRY_DEMAND: &str = "Demand equal treatment from the team";
pub const T_RIVALRY_STUDY: &str = "Study your team-mate's data with the engineers";
pub const T_RIVALRY_CALM: &str = "Keep it inside the team";

/// The English templates of the inbox's texts (translation keys): those of `News.template`, with
/// `{placeholders}` filled from `News.args`, and those of `Choice.template`, which have none.
pub const TEMPLATES: &[&str] = &[
    T_ENGINEER,
    T_DEPT_TRACK,
    T_DEPT_QUALI,
    T_DEPT_TYRES,
    T_DEPT_ACCEPT,
    T_DEPT_DECLINE,
    T_MEDIA_PODIUM,
    T_MEDIA_PODIUM_HUMBLE,
    T_MEDIA_PODIUM_BOLD,
    T_MEDIA_DNF,
    T_MEDIA_DNF_HUMBLE,
    T_MEDIA_DNF_BOLD,
    T_MEDIA_DEFLECT,
    T_RIVALRY,
    T_RIVALRY_DEMAND,
    T_RIVALRY_STUDY,
    T_RIVALRY_CALM,
];

/// A department asks for a programme in one round of this many.
const DEPARTMENT_ODDS: u64 = 3;
/// A rivalry flares up after every this many races in a row lost to the team-mate.
const DUELS_LOST: usize = 3;

/// `template` with its `{placeholders}` filled from `args`: the English text.
pub fn render(template: &str, args: &[(String, String)]) -> String {
    super::texts::render(template, args)
}

/// `Action::Answer`: `actor` answers their mail `news` with choice `choice`.
pub(super) fn answer(
    league: &mut League,
    actor: &Actor,
    news: u32,
    choice: u8,
) -> Result<Vec<Text>, LeagueError> {
    let Some(item) = league
        .news
        .iter()
        .find(|n| news != 0 && n.id == news && n.kind != NewsKind::News)
    else {
        return refuse("no_mail", t!("there is no mail {news}", news));
    };
    if item.to.as_deref().is_some_and(|to| to != actor.id) {
        return refuse("not_yours", t!("only the player it is for answers this mail"));
    }
    if item.answered.is_some() {
        return refuse("answered", t!("this mail is answered already"));
    }
    let Some(effect) = item.choices.get(usize::from(choice)).map(|c| c.effect) else {
        return refuse("bad_choice", t!("this mail has no such answer"));
    };
    let (year, round) = (item.year, item.round);
    let before = state(league, &actor.id);
    apply_effect(league, &actor.id, &effect);
    let after = state(league, &actor.id);
    if let (Some(before), Some(after)) = (before, after) {
        carry_into_snapshots(league, &actor.id, year, round, &before, &after);
    }
    if let Some(item) = league.news.iter_mut().find(|n| n.id == news) {
        item.answered = Some(choice);
    }
    let lines = league.update_status();
    let at = if year == league.year() { round } else { 0 };
    league.say_lines(at, lines);
    Ok(Vec::new())
}

/// `Action::ReadMail`: `actor` read their mails up to id `upto` (it never goes back).
pub(super) fn read(league: &mut League, actor: &Actor, upto: u32) -> Result<Vec<Text>, LeagueError> {
    let upto = upto.min(league.news_seq);
    if let Some(p) = league.player_mut(&actor.id) {
        p.mail_read = p.mail_read.max(upto);
    }
    Ok(Vec::new())
}

/// Called after round `n` was completed: each active player's mail of the round, if any.
pub(super) fn after_round(league: &mut League, n: u8) {
    let season = league.season().clone();
    let ids: Vec<String> = league.active_players().iter().map(|p| p.id.clone()).collect();
    for id in ids {
        let had = league.news.iter().any(|x| {
            x.kind != NewsKind::News && x.year == season.year && x.round == n && x.to.as_deref() == Some(&id)
        });
        if had {
            continue;
        }
        if let Some(mail) = mail_for(league.seed, &season, &id, n) {
            post(league, &id, n, mail);
        }
    }
}

/// Called by `undo()` for each round taken back: the mails of round `n` and later go (what their
/// answers did goes with the round's snapshot).
pub(super) fn undo(league: &mut League, n: u8) {
    let year = league.year();
    league
        .news
        .retain(|x| x.kind == NewsKind::News || x.year != year || x.round < n);
}

/// `player`'s unread mails, events and media questions.
pub fn unread<'a>(league: &'a League, player: &str) -> Vec<&'a News> {
    let read = league.player(player).map_or(0, |p| p.mail_read);
    league
        .news
        .iter()
        .filter(|n| n.kind != NewsKind::News && n.id > read)
        .filter(|n| n.to.as_deref().is_none_or(|to| to == player))
        .collect()
}

/// A mail before it gets its id.
struct Mail {
    kind: NewsKind,
    template: &'static str,
    args: Vec<(String, String)>,
    choices: Vec<Choice>,
}

fn arg(k: &str, v: impl ToString) -> (String, String) {
    (k.to_owned(), v.to_string())
}

fn choice(template: &str, effect: Effect) -> Choice {
    Choice {
        template: template.to_owned(),
        effect,
    }
}

/// FNV-1a over a player id, for the seed.
fn hash(id: &str) -> u64 {
    id.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// What `player` gets after round `n` of `season`, if anything.
fn mail_for(seed: u64, season: &Season, player: &str, n: u8) -> Option<Mail> {
    let entry = season.player_entry(player)?;
    let round = season.round(n)?;
    let team = season
        .team(entry.slot)
        .map_or_else(String::new, |t| t.name.clone());
    let race = round.race.clone();

    // A media question after a podium or a retirement the player drove (or had simulated).
    if let Some(c) = round
        .race_result
        .iter()
        .find(|c| c.entry == entry.id && !c.ai_driven)
    {
        let deflect = choice(T_MEDIA_DEFLECT, Effect::default());
        let humble = Effect {
            recognition: 1.0,
            ..Effect::default()
        };
        let bold = Effect {
            pace: 0.3,
            recognition: -1.0,
            ..Effect::default()
        };
        if c.status == Finish::Finished && (1..=3).contains(&c.position) {
            return Some(Mail {
                kind: NewsKind::Media,
                template: T_MEDIA_PODIUM,
                args: vec![
                    arg("race", &race),
                    arg("position", c.position),
                    arg("team", &team),
                ],
                choices: vec![
                    choice(T_MEDIA_PODIUM_HUMBLE, humble),
                    choice(T_MEDIA_PODIUM_BOLD, bold),
                    deflect,
                ],
            });
        }
        if c.status == Finish::Dnf {
            return Some(Mail {
                kind: NewsKind::Media,
                template: T_MEDIA_DNF,
                args: vec![arg("race", &race)],
                choices: vec![
                    choice(T_MEDIA_DNF_HUMBLE, humble),
                    choice(T_MEDIA_DNF_BOLD, bold),
                    deflect,
                ],
            });
        }
    }

    // The rivalry with the team-mate flares up after every third race in a row lost to them.
    let lost = duels_lost(season, &entry.id, n);
    if lost > 0
        && lost.is_multiple_of(DUELS_LOST)
        && let Some(mate) = team_mate_at(season, &entry.id, n)
    {
        return Some(Mail {
            kind: NewsKind::Event,
            template: T_RIVALRY,
            args: vec![arg("mate", &mate.name), arg("count", lost), arg("team", &team)],
            choices: vec![
                choice(
                    T_RIVALRY_DEMAND,
                    Effect {
                        recognition: -1.0,
                        racecraft: 0.4,
                        ..Effect::default()
                    },
                ),
                choice(
                    T_RIVALRY_STUDY,
                    Effect {
                        experience: 0.3,
                        pace: 0.2,
                        ..Effect::default()
                    },
                ),
                choice(
                    T_RIVALRY_CALM,
                    Effect {
                        recognition: 1.0,
                        ..Effect::default()
                    },
                ),
            ],
        });
    }

    // The engineer's feedback when the team's new parts arrive for the next round.
    let next = n.checked_add(1)?;
    let mut parts: Vec<&str> = season
        .upgrades
        .iter()
        .filter(|u| u.slot == entry.slot && u.ready == next && !u.failed)
        .map(|u| u.part.name())
        .collect();
    parts.dedup();
    if !parts.is_empty()
        && let Some(r) = season.round(next)
    {
        return Some(Mail {
            kind: NewsKind::Mail,
            template: T_ENGINEER,
            args: vec![arg("part", parts.join(", ")), arg("race", &r.race)],
            choices: Vec::new(),
        });
    }

    // A department asks for a programme at the next race the game drives.
    let luck = mix(&[seed, u64::from(season.year), u64::from(n), hash(player), 0x1b0c]);
    let next_driven = season.rounds.iter().find(|r| r.round > n && r.track.is_some())?;
    if !luck.is_multiple_of(DEPARTMENT_ODDS) {
        return None;
    }
    let template = match (luck >> 8) % 3 {
        0 => T_DEPT_TRACK,
        1 => T_DEPT_QUALI,
        _ => T_DEPT_TYRES,
    };
    let rp = 30 + 10 * ((luck >> 16) % 4) as i32;
    Some(Mail {
        kind: NewsKind::Event,
        template,
        args: vec![arg("race", &next_driven.race), arg("rp", rp)],
        choices: vec![
            choice(
                T_DEPT_ACCEPT,
                Effect {
                    rp,
                    ..Effect::default()
                },
            ),
            choice(T_DEPT_DECLINE, Effect::default()),
        ],
    })
}

/// The programmes a department asks for, by template (for the launcher's practice view).
pub fn programme_of(template: &str) -> Option<Programme> {
    match template {
        T_DEPT_TRACK => Some(Programme::TrackAcclimatisation),
        T_DEPT_QUALI => Some(Programme::QualifyingPace),
        T_DEPT_TYRES => Some(Programme::TyreManagement),
        _ => None,
    }
}

/// How many races in a row up to round `n` the car `entry` finished behind its team-mate (rounds
/// without a race result don't break the run).
fn duels_lost(season: &Season, entry: &str, n: u8) -> usize {
    let mut lost = 0;
    for r in season.rounds.iter().rev().filter(|r| r.round <= n) {
        if r.race_result.is_empty() {
            continue;
        }
        let Some(me) = r.race_result.iter().find(|c| c.entry == entry) else {
            break;
        };
        let Some(mate) =
            team_mate_at(season, entry, r.round).and_then(|m| r.race_result.iter().find(|c| c.entry == m.id))
        else {
            break;
        };
        let key = |c: &super::Classified| (c.status != Finish::Finished, c.position);
        if key(me) <= key(mate) {
            break;
        }
        lost += 1;
    }
    lost
}

/// Post `mail` to `player` for round `n`, with the next id.
fn post(league: &mut League, player: &str, n: u8, mail: Mail) {
    league.news_seq += 1;
    let year = league.year();
    league.news.push(News {
        year,
        round: n,
        text: render(mail.template, &mail.args),
        id: league.news_seq,
        kind: mail.kind,
        to: Some(player.to_owned()),
        template: Some(mail.template.to_owned()),
        args: mail.args,
        parts: Vec::new(),
        choices: mail.choices,
        answered: None,
    });
    if league.news.len() > NEWS_MAX {
        let extra = league.news.len() - NEWS_MAX;
        league.news.drain(..extra);
    }
}

/// Apply an answer's effect to `player`: resource points through `earn` (a negative amount is
/// taken off), recognition through `recognise`, the ratings through `train`.
fn apply_effect(league: &mut League, player: &str, e: &Effect) {
    if e.rp > 0 {
        league.earn(player, e.rp as f32);
    } else if e.rp < 0
        && let Some(p) = league.player_mut(player)
    {
        p.rp = p.rp.saturating_sub(e.rp.unsigned_abs());
    }
    if e.recognition != 0.0 {
        league.recognise(player, e.recognition);
    }
    if [e.experience, e.racecraft, e.awareness, e.pace, e.focus]
        .iter()
        .any(|x| *x != 0.0)
    {
        league.train(player, |a| add(a, e));
        league.sync_ratings();
    }
}

/// Add an effect's ratings to `a`, within the ratings' bounds.
fn add(a: &mut Attributes, e: &Effect) {
    a.experience += e.experience;
    a.racecraft += e.racecraft;
    a.awareness += e.awareness;
    a.pace += e.pace;
    if e.focus != 0.0 {
        let base = a.focus.unwrap_or_else(|| a.overall());
        a.focus = Some((base + e.focus).clamp(ATTRIBUTE_MIN, ATTRIBUTE_MAX));
    }
    a.clamp();
}

/// What an answer can change of a player.
#[derive(Debug, Clone, Copy)]
struct State {
    rp: u32,
    recognition: f32,
    attributes: Attributes,
}

fn state(league: &League, player: &str) -> Option<State> {
    let p = league.player(player)?;
    Some(State {
        rp: p.rp,
        recognition: p.recognition,
        attributes: p.attributes.unwrap_or_else(|| Attributes::from_rating(p.rating)),
    })
}

/// Write what an answer just changed into the snapshots kept after the mail came (those of this
/// season's rounds after round `round` of `year`), so restoring one keeps the answer's effect,
/// as the mail stays answered.
fn carry_into_snapshots(
    league: &mut League,
    player: &str,
    year: u16,
    round: u8,
    before: &State,
    after: &State,
) {
    let rp = i64::from(after.rp) - i64::from(before.rp);
    let recognition = after.recognition - before.recognition;
    let (b, a) = (&before.attributes, &after.attributes);
    let delta = Effect {
        rp: 0,
        recognition: 0.0,
        experience: a.experience - b.experience,
        racecraft: a.racecraft - b.racecraft,
        awareness: a.awareness - b.awareness,
        pace: a.pace - b.pace,
        focus: a.focus.unwrap_or(0.0) - b.focus.unwrap_or(0.0),
    };
    let focus_set = a.focus.is_some() && b.focus.is_none();
    let season_year = league.year();
    for r in &mut league.season_mut().rounds {
        if season_year == year && r.round <= round {
            continue;
        }
        let Some(snapshot) = r.before.as_mut() else {
            continue;
        };
        for s in snapshot.iter_mut().filter(|s| s.player == player) {
            s.rp = (i64::from(s.rp) + rp).clamp(0, i64::from(u32::MAX)) as u32;
            s.recognition = (s.recognition + recognition).clamp(0.0, 100.0);
            let mut at = s.attributes.unwrap_or_else(|| Attributes::from_rating(s.rating));
            at.experience += delta.experience;
            at.racecraft += delta.racecraft;
            at.awareness += delta.awareness;
            at.pace += delta.pace;
            if focus_set {
                at.focus = a.focus;
            } else if let Some(f) = at.focus.as_mut() {
                *f = (*f + delta.focus).clamp(ATTRIBUTE_MIN, ATTRIBUTE_MAX);
            }
            at.clamp();
            s.attributes = Some(at);
            s.rating = (at.overall() * 10.0).round() / 10.0;
        }
    }
}
