//! The driver market (W2-3): mid-season approaches, secret meetings, negotiation, buy-outs and the
//! number-one clause.
//!
//! - At 40%, 60% and 80% of the calendar, teams faster than a player's own may show interest
//!   ([`after_round`]); the player gets a mail.
//! - A meeting ([`meet`]) reveals the team's offer for next season. Word of it can get out (15%),
//!   which costs recognition with the current team.
//! - Accepting ([`accept`]) signs early. A player under contract beyond this season is bought
//!   out: the new team pays, and the current team thinks less of the player.
//! - Any offer, mid-season or at the season's end, can be negotiated ([`negotiate`]): within the
//!   team's limit it is agreed, else the team counters, until it walks away. The own team never
//!   leaves a player without a seat.

use serde::{Deserialize, Serialize};

use super::progress::{self, mix};
use super::texts::{Text, t};
use super::{
    Actor, CatalogTeam, Focus, League, LeagueError, News, NewsKind, Offer, OfferKind, Phase, Player, Seat,
    catalog_team, catalog_year, choose_replaced, driver_standings, head_to_head, is_false, refuse, team_mate,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OfferStatus {
    #[default]
    Open,
    Countered,
    Agreed,
    Withdrawn,
}

/// What a player asks for, or a team offers, in a negotiation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Terms {
    pub salary_k: u32,
    pub years: u8,
    #[serde(default)]
    pub number_one: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApproachStatus {
    Interest,
    Met,
    Accepted,
    Declined,
    Expired,
}

/// A team's mid-season interest in a player.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Approach {
    pub player: String,
    /// Next season's team slot.
    pub slot: u8,
    /// The round after which it came.
    pub round: u8,
    pub status: ApproachStatus,
    /// The offer, revealed at the meeting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offer: Option<Offer>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub leaked: bool,
    /// The seat the player took when they accepted (the driver key they replace next season).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replaces: Option<String>,
    /// The end of the contract the player had when they accepted, given back when the early
    /// signing is taken back.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_until: Option<u16>,
}

/// The mail a player gets when a team wants to meet them (`{team}`).
pub const MAIL_MEET: &str = "{team} would like to meet you.";
/// The English templates of the market's mails (translation keys).
pub const TEMPLATES: &[&str] = &[MAIL_MEET];

/// The share of meetings word gets out of.
pub const LEAK: f64 = 0.15;
/// Recognition a leaked meeting costs with the current team.
pub const LEAK_RECOGNITION: f32 = 5.0;
/// Recognition a buy-out costs with the current team.
pub const BUY_OUT_RECOGNITION: f32 = 10.0;
/// How much more than its offer a team pays per point of leverage.
pub const LEVERAGE_STEP: f32 = 0.15;
/// The share of the salary limit a number-one clause costs.
pub const NUMBER_ONE_COST: f32 = 0.2;
/// Contract lengths a player may ask for.
pub const YEARS: std::ops::RangeInclusive<u8> = 1..=3;
/// The length of a mid-season offer.
const EARLY_YEARS: u8 = 2;

// Salts that keep the rolls of the market apart.
const SALT_LEAK: u64 = 0x6c65_616b;
const SALT_LIMIT: u64 = 0x6c69_6d69;

/// The rounds after which teams approach players: 40%, 60% and 80% into a calendar of `rounds`.
pub fn approach_rounds(rounds: usize) -> [u8; 3] {
    let rounds = rounds.max(1);
    [2, 3, 4].map(|k| ((2 * rounds * k + 5) / 10).clamp(1, rounds) as u8)
}

/// How far down next season's teams, ranked by pace, a player's rating reaches (1: the fastest
/// team); the champion reaches every team.
pub(super) fn reach(rating: f32, champion: bool, teams: usize) -> usize {
    if champion {
        1
    } else {
        ((11.0 - (rating - 70.0) / 2.5).ceil() as usize).clamp(1, teams.max(1))
    }
}

/// Next season's teams, the fastest first.
fn ranked(year: u16) -> Vec<&'static CatalogTeam> {
    let Some(cat) = catalog_year(year) else {
        return Vec::new();
    };
    let mut ranked: Vec<&CatalogTeam> = cat.teams.iter().collect();
    ranked.sort_by(|a, b| a.pace.total_cmp(&b.pace).then(a.slot.cmp(&b.slot)));
    ranked
}

/// A number from 0 to 1 for a seeded roll.
fn roll(values: &[u64]) -> f64 {
    (mix(values) >> 11) as f64 / (1u64 << 53) as f64
}

/// A player id as a number for [`mix`].
fn key(id: &str) -> u64 {
    id.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

fn racing(league: &League) -> Result<(), LeagueError> {
    if league.phase == Phase::Racing {
        Ok(())
    } else {
        refuse("phase", t!("teams approach drivers during the season"))
    }
}

fn team_name(year: u16, slot: u8) -> String {
    catalog_team(year, slot).map_or_else(|| format!("team {slot}"), |t| t.name.clone())
}

/// The index of `player`'s approach by `slot` this season.
fn find(league: &League, player: &str, slot: u8) -> Result<usize, LeagueError> {
    league
        .season()
        .approaches
        .iter()
        .position(|a| a.player == player && a.slot == slot)
        .map_or_else(
            || {
                refuse(
                    "no_approach",
                    t!("{team} hasn't approached you", team = team_name(league.year() + 1, slot)),
                )
            },
            Ok,
        )
}

/// A mail to `to`: `template` with its `{placeholders}` filled from `args`.
fn mail(league: &mut League, round: u8, to: &str, template: &str, args: &[(&str, String)]) {
    let mut text = template.to_owned();
    for (k, v) in args {
        text = text.replace(&format!("{{{k}}}"), v);
    }
    league.news_seq += 1;
    let year = league.year();
    league.news.push(News {
        id: league.news_seq,
        kind: NewsKind::Mail,
        to: Some(to.to_owned()),
        template: Some(template.to_owned()),
        args: args.iter().map(|(k, v)| ((*k).to_owned(), v.clone())).collect(),
        ..News::plain(year, round, text)
    });
    if league.news.len() > super::NEWS_MAX {
        let extra = league.news.len() - super::NEWS_MAX;
        league.news.drain(..extra);
    }
}

/// How likely a team is to approach `p` after a round, from how the player does: against the
/// team-mate in the championship, the team's recognition and the rivalry.
fn interest(league: &League, p: &Player, entry: &str) -> f64 {
    let season = league.season();
    let drivers = driver_standings(season);
    let points = |id: &str| drivers.iter().find(|d| d.entry == id).map_or(0.0, |d| d.points);
    let mate = match team_mate(season, entry).map(|m| points(entry) - points(&m.id)) {
        Some(d) if d > 0.0 => 0.15,
        Some(d) if d < 0.0 => -0.15,
        _ => 0.0,
    };
    let recognition = f64::from((p.recognition - 50.0) / 100.0 * 0.3);
    let rivalry = if p.rival_score != 0 {
        p.rival_score.signum()
    } else {
        p.rival
            .as_deref()
            .and_then(|r| head_to_head(season, entry, r))
            .map_or(0, |h| (h.race.0 as i32 - h.race.1 as i32).signum())
    };
    (0.3 + mate + recognition + 0.05 * f64::from(rivalry)).clamp(0.05, 0.75)
}

/// Called after round `n` was completed (raced, simulated or skipped): at the set points of the
/// season, teams faster than a player's own may approach them (one team per point at most).
pub(super) fn after_round(league: &mut League, n: u8) {
    restore_early(league);
    if league.phase != Phase::Racing {
        return;
    }
    let season = league.season();
    let year = season.year;
    if !approach_rounds(season.rounds.len()).contains(&n) || year >= league.setup.last_year {
        return;
    }
    let ranked = ranked(year + 1);
    if ranked.is_empty() {
        return;
    }
    let leader = driver_standings(season).first().and_then(|d| d.player.clone());
    let mut new = Vec::new();
    for p in league.players.iter().filter(|p| !p.left) {
        let Some(entry) = season.player_entry(&p.id) else {
            continue;
        };
        let mine: Vec<&Approach> = season.approaches.iter().filter(|a| a.player == p.id).collect();
        if mine.iter().any(|a| a.status == ApproachStatus::Accepted) {
            continue;
        }
        let reach = reach(p.rating, leader.as_deref() == Some(p.id.as_str()), ranked.len());
        let own_rank = ranked
            .iter()
            .position(|t| t.slot == entry.slot)
            .unwrap_or(ranked.len());
        let chance = interest(league, p, &entry.id);
        let pick = ranked
            .iter()
            .take(own_rank)
            .skip(reach - 1)
            .filter(|t| !mine.iter().any(|a| a.slot == t.slot))
            .find(|t| {
                roll(&[
                    league.seed,
                    u64::from(year),
                    u64::from(n),
                    key(&p.id),
                    u64::from(t.slot),
                ]) < chance
            });
        if let Some(t) = pick {
            new.push((p.id.clone(), t.slot, t.name.clone()));
        }
    }
    for (player, slot, team) in new {
        league.season_mut().approaches.push(Approach {
            player: player.clone(),
            slot,
            round: n,
            status: ApproachStatus::Interest,
            offer: None,
            leaked: false,
            replaces: None,
            old_until: None,
        });
        mail(league, n, &player, MAIL_MEET, &[("team", team)]);
    }
}

/// `Action::Meet`: a secret meeting. The team reveals its offer for next season; word of the
/// meeting may get out.
pub(super) fn meet(league: &mut League, actor: &Actor, slot: u8) -> Result<Vec<Text>, LeagueError> {
    racing(league)?;
    let i = find(league, &actor.id, slot)?;
    let year = league.year();
    let next = year + 1;
    let team = team_name(next, slot);
    let a = league.season().approaches[i].clone();
    match a.status {
        ApproachStatus::Interest => {}
        ApproachStatus::Met => return Ok(Vec::new()),
        _ => {
            return refuse("no_approach", t!("{team} isn't waiting to meet you any more", team));
        }
    }
    let p = league.player(&actor.id).expect("a player");
    let ranked = ranked(next);
    let rank = ranked.iter().position(|t| t.slot == slot).unwrap_or(ranked.len());
    let (salary_k, bonus_k) = progress::salary(p.rating, p.titles.len(), rank + 1, ranked.len(), None);
    let offer = Offer {
        player: actor.id.clone(),
        slot,
        kind: OfferKind::Move,
        years: EARLY_YEARS,
        // A team that comes for a driver pays a little more, as at the season's end.
        salary_k: salary_k / 50 * 50 + salary_k / 20 / 50 * 50,
        bonus_k,
        early: true,
        ..Offer::fresh()
    };
    let leaked = roll(&[
        league.seed,
        u64::from(year),
        u64::from(a.round),
        key(&actor.id),
        u64::from(slot),
        SALT_LEAK,
    ]) < LEAK;
    let line = t!(
        "{team} offers {years} from {next}, {salary} a season.",
        team,
        next,
        salary = progress::money(u64::from(offer.salary_k))
    )
    .list("years", vec![years_text(offer.years)]);
    let approach = &mut league.season_mut().approaches[i];
    approach.status = ApproachStatus::Met;
    approach.offer = Some(offer);
    approach.leaked = leaked;
    if leaked {
        league.recognise(&actor.id, -LEAK_RECOGNITION);
        let own = league
            .season()
            .player_entry(&actor.id)
            .and_then(|e| league.season().team(e.slot))
            .map_or_else(String::new, |t| t.name.clone());
        league.say(
            a.round,
            t!("{name} was seen meeting {team}; {own} isn't pleased.", name = actor.name, team, own),
        );
    }
    Ok(vec![line])
}

/// `Action::AcceptApproach`: sign early for next season. A player under contract beyond this
/// season is bought out.
pub(super) fn accept(
    league: &mut League,
    actor: &Actor,
    slot: u8,
    replaces: Option<String>,
) -> Result<Vec<Text>, LeagueError> {
    racing(league)?;
    let i = find(league, &actor.id, slot)?;
    let year = league.year();
    let next = year + 1;
    let a = league.season().approaches[i].clone();
    let Some(t) = catalog_team(next, slot) else {
        return refuse("bad_offer", t!("there is no team {slot} in {next} any more", slot, next));
    };
    match a.status {
        ApproachStatus::Met => {}
        ApproachStatus::Accepted => return Ok(Vec::new()),
        ApproachStatus::Interest => {
            return refuse("no_approach", t!("meet {team} first to hear their offer", team = t.name));
        }
        _ => return refuse("no_approach", t!("{team}'s approach is over", team = t.name)),
    }
    let Some(offer) = a.offer.clone() else {
        return refuse("no_approach", t!("meet {team} first to hear their offer", team = t.name));
    };
    if offer.status == OfferStatus::Withdrawn {
        return refuse("walked_away", t!("{team} walked away from the talks", team = t.name));
    }
    let others: Vec<&str> = league
        .players
        .iter()
        .filter(|p| p.id != actor.id)
        .filter_map(|p| p.signed.as_ref())
        .filter(|s| s.slot == slot)
        .map(|s| s.replaces.as_str())
        .collect();
    let replaces = choose_replaced(t, replaces.as_deref(), &others)?;
    let terms = agreed_or_first(&offer);
    let until = next + u16::from(terms.years.max(1)) - 1;
    let own = league
        .season()
        .player_entry(&actor.id)
        .and_then(|e| league.season().team(e.slot))
        .map(|t| t.name.clone());
    let p = league.player_mut(&actor.id).expect("a player");
    let old_until = p.contract_until;
    let bought = old_until.is_some_and(|y| y > year);
    p.signed = Some(Seat {
        slot,
        replaces: replaces.clone(),
    });
    p.signed_until = Some(until);
    let salary = settle_terms(p, &offer, None, false);
    for (j, other) in league.season_mut().approaches.iter_mut().enumerate() {
        if j == i {
            other.status = ApproachStatus::Accepted;
            other.replaces = Some(replaces.clone());
            other.old_until = old_until;
        } else if other.player == actor.id
            && matches!(other.status, ApproachStatus::Interest | ApproachStatus::Met)
        {
            other.status = ApproachStatus::Declined;
        }
    }
    let salary = progress::money(u64::from(salary));
    let (team, name) = (&t.name, &actor.name);
    let text = match (bought, own) {
        (true, Some(own)) => {
            league.recognise(&actor.id, -BUY_OUT_RECOGNITION);
            if until > next {
                t!(
                    "{team} buys {name} out of the contract with {own}: {name} drives for {team} from {next} to {until}, for {salary} a season.",
                    team,
                    name,
                    own,
                    next,
                    until,
                    salary
                )
            } else {
                t!(
                    "{team} buys {name} out of the contract with {own}: {name} drives for {team} from {next}, for {salary} a season.",
                    team,
                    name,
                    own,
                    next,
                    salary
                )
            }
        }
        _ if until > next => t!(
            "{name} signs early with {team} for {next} to {until}, for {salary} a season.",
            name,
            team,
            next,
            until,
            salary
        ),
        _ => t!("{name} signs early with {team} for {next}, for {salary} a season.", name, team, next, salary),
    };
    league.say(league.season().current_round(), text);
    Ok(Vec::new())
}

/// `Action::DeclineApproach`.
pub(super) fn decline(league: &mut League, actor: &Actor, slot: u8) -> Result<Vec<Text>, LeagueError> {
    racing(league)?;
    let i = find(league, &actor.id, slot)?;
    let a = &mut league.season_mut().approaches[i];
    match a.status {
        ApproachStatus::Interest | ApproachStatus::Met => {
            a.status = ApproachStatus::Declined;
            Ok(Vec::new())
        }
        ApproachStatus::Declined => Ok(Vec::new()),
        _ => refuse("no_approach", t!("that approach is over")),
    }
}

/// Where an offer being negotiated is: a mid-season approach's, or one of the season's end.
#[derive(Clone, Copy)]
enum Place {
    Approach(usize),
    End(usize),
}

fn offer_at(league: &mut League, at: Place) -> &mut Offer {
    let season = league.season_mut();
    match at {
        Place::Approach(i) => season.approaches[i].offer.as_mut().expect("met"),
        Place::End(i) => &mut season.offers[i],
    }
}

/// The rating a number-one clause has to match at `slot` next season: the better of the team's
/// drivers, or of other players signed there.
fn mate_rating(league: &League, player: &str, slot: u8, next: u16) -> f32 {
    let drivers = catalog_team(next, slot)
        .into_iter()
        .flat_map(|t| t.drivers.iter().map(|d| d.rating));
    let players = league
        .players
        .iter()
        .filter(|p| p.id != player && !p.left)
        .filter(|p| p.signed.as_ref().is_some_and(|s| s.slot == slot))
        .map(|p| p.rating);
    drivers.chain(players).fold(f32::MIN, f32::max)
}

/// `Action::Negotiate`: ask `slot`'s team for `terms`.
///
/// The team's limit is its offer times `1 + 0.15 × leverage`; leverage comes from the player's
/// rating against the team's drivers, the recognition and the competing offers. A number-one
/// clause costs a fifth of the limit and needs a rating at least the team-mate's. Within the
/// limit the terms are agreed; else the team counters halfway, until its attempts run out and it
/// walks away. The own team then keeps its first offer instead, and so does the last team a
/// player has an offer from, so nobody ends without a seat.
pub(super) fn negotiate(
    league: &mut League,
    actor: &Actor,
    slot: u8,
    terms: Terms,
) -> Result<Vec<Text>, LeagueError> {
    if !YEARS.contains(&terms.years) || terms.salary_k == 0 {
        return refuse("bad_offer", t!("ask for a salary and a contract of 1 to 3 years"));
    }
    let year = league.year();
    let next = year + 1;
    let team = team_name(next, slot);
    let at = match league.phase {
        Phase::Racing => {
            let i = find(league, &actor.id, slot)?;
            if league.season().approaches[i].status != ApproachStatus::Met {
                return refuse("no_approach", t!("meet {team} first to hear their offer", team));
            }
            Place::Approach(i)
        }
        Phase::SeasonEnd => match league
            .season()
            .offers
            .iter()
            .position(|o| o.player == actor.id && o.slot == slot)
        {
            Some(i) => Place::End(i),
            None => return refuse("no_offer", t!("that team made you no offer")),
        },
        _ => {
            return refuse(
                "phase",
                t!("contracts are negotiated during and at the end of a season"),
            );
        }
    };
    let offer = offer_at(league, at).clone();
    if offer.kind == OfferKind::Contract {
        return refuse("bad_offer", t!("a contract that runs on isn't negotiated"));
    }
    match offer.status {
        OfferStatus::Agreed => {
            return refuse("bad_offer", t!("you agreed terms with {team} already; sign them", team));
        }
        OfferStatus::Withdrawn => return refuse("walked_away", t!("{team} walked away from the talks", team)),
        _ => {}
    }
    if offer.attempts == 0 {
        return refuse(
            "walked_away",
            t!("{team} won't talk any more; their first offer stands", team),
        );
    }
    let p = league.player(&actor.id).expect("a player");
    // The other offers the player could take instead.
    let competing = match at {
        Place::Approach(i) => league
            .season()
            .approaches
            .iter()
            .enumerate()
            .filter(|(j, a)| *j != i && a.player == actor.id && a.status == ApproachStatus::Met)
            .filter(|(_, a)| {
                a.offer
                    .as_ref()
                    .is_some_and(|o| o.status != OfferStatus::Withdrawn)
            })
            .count(),
        Place::End(i) => league
            .season()
            .offers
            .iter()
            .enumerate()
            .filter(|(j, o)| *j != i && o.player == actor.id && o.status != OfferStatus::Withdrawn)
            .count(),
    };
    let mate = mate_rating(league, &actor.id, slot, next);
    let edge = ((p.rating - mate) / 10.0).clamp(-0.5, 1.0);
    let weight = if offer.kind == OfferKind::Renew { 1.0 } else { 0.5 };
    let recognition = ((p.recognition - 50.0) / 50.0).clamp(-1.0, 1.0) * weight;
    let leverage = (1.0 + edge + recognition + 0.5 * competing.min(3) as f32).clamp(0.0, 3.0);
    // A team's limit isn't known to the cent: a few percent either way, the same for a seed.
    let noise = 0.97
        + 0.06
            * roll(&[
                league.seed,
                u64::from(year),
                key(&actor.id),
                u64::from(slot),
                SALT_LIMIT,
            ]) as f32;
    let limit = (offer.salary_k as f32 * (1.0 + LEVERAGE_STEP * leverage) * noise).max(offer.salary_k as f32);
    let clause_ok = !terms.number_one || p.rating >= mate;
    let allowed = if terms.number_one {
        limit * (1.0 - NUMBER_ONE_COST)
    } else {
        limit
    };
    let within = clause_ok && terms.salary_k as f32 <= allowed;
    let o = offer_at(league, at);
    if within {
        o.status = OfferStatus::Agreed;
        o.agreed = Some(terms);
        o.counter = None;
        // The contract runs as long as agreed.
        o.years = terms.years;
        let clause = if terms.number_one {
            " with a number-one clause"
        } else {
            ""
        };
        let salary = progress::money(u64::from(terms.salary_k));
        let text = if clause.is_empty() {
            t!("{team} agrees: {salary} a season for {years}. Sign it to make it count.", team, salary)
        } else {
            t!(
                "{team} agrees: {salary} a season for {years} with a number-one clause. Sign it to make it count.",
                team,
                salary
            )
        };
        return Ok(vec![text.list("years", vec![years_text(terms.years)])]);
    }
    o.attempts -= 1;
    if o.attempts > 0 {
        let current = o.counter.map_or(o.salary_k, |c| c.salary_k);
        let number_one = terms.number_one && clause_ok;
        let cap = if number_one { allowed } else { limit };
        let half = (current as f32 + terms.salary_k as f32) / 2.0;
        let salary_k = ((half.min(cap) / 50.0).floor() as u32 * 50).max(100);
        let counter = Terms {
            salary_k,
            years: terms.years,
            number_one,
        };
        o.counter = Some(counter);
        o.status = OfferStatus::Countered;
        let attempts = o.attempts;
        let salary = progress::money(u64::from(salary_k));
        let head = if number_one {
            t!("{team} counters: {salary} a season for {years} with a number-one clause.", team, salary)
        } else if terms.number_one {
            t!("{team} counters: {salary} a season for {years}, without a number-one clause.", team, salary)
        } else {
            t!("{team} counters: {salary} a season for {years}.", team, salary)
        };
        let left = if attempts == 1 {
            t!("1 more attempt before they walk away.")
        } else {
            t!("{attempts} more attempts before they walk away.", attempts)
        };
        return Ok(vec![
            t!("{counter} {left}")
                .list("counter", vec![head.list("years", vec![years_text(counter.years)])])
                .list("left", vec![left]),
        ]);
    }
    // The talks are over.
    o.counter = None;
    let last = match at {
        Place::End(i) => !league
            .season()
            .offers
            .iter()
            .enumerate()
            .any(|(j, x)| j != i && x.player == actor.id && x.status != OfferStatus::Withdrawn),
        Place::Approach(_) => false,
    };
    let o = offer_at(league, at);
    if o.kind == OfferKind::Renew || last {
        o.status = OfferStatus::Open;
        return Ok(vec![t!("{team} ends the talks; their first offer stands.", team)]);
    }
    match at {
        Place::Approach(_) => o.status = OfferStatus::Withdrawn,
        // A season-end offer that is gone can't be signed.
        Place::End(i) => {
            league.season_mut().offers.remove(i);
        }
    }
    Ok(vec![t!("{team} walks away from the talks and withdraws the offer.", team)])
}

/// "1 year" or "3 years".
fn years_text(years: u8) -> Text {
    if years == 1 {
        t!("1 year")
    } else {
        t!("{years} years", years)
    }
}

/// `offer` is the contract the player is under already (not one signed early during the season).
fn runs_on(offer: &Offer) -> bool {
    offer.kind == OfferKind::Contract && !offer.early
}

/// The terms of `offer`: the agreed ones, else its first.
fn agreed_or_first(offer: &Offer) -> Terms {
    offer.agreed.unwrap_or(Terms {
        salary_k: offer.salary_k,
        years: offer.years,
        number_one: false,
    })
}

/// Early signings the season's end took back (an undo from there clears every signing) are
/// signed again.
fn restore_early(league: &mut League) {
    let next = league.year() + 1;
    let accepted: Vec<Approach> = league
        .season()
        .approaches
        .iter()
        .filter(|a| a.status == ApproachStatus::Accepted)
        .cloned()
        .collect();
    for a in accepted {
        let Some(offer) = a.offer.as_ref() else {
            continue;
        };
        let replaces = match a.replaces.clone() {
            Some(r) => r,
            None => {
                let Some(t) = catalog_team(next, a.slot) else {
                    continue;
                };
                let others: Vec<&str> = league
                    .players
                    .iter()
                    .filter(|p| p.id != a.player)
                    .filter_map(|p| p.signed.as_ref())
                    .filter(|s| s.slot == a.slot)
                    .map(|s| s.replaces.as_str())
                    .collect();
                let Ok(r) = choose_replaced(t, None, &others) else {
                    continue;
                };
                r
            }
        };
        let until = next + u16::from(agreed_or_first(offer).years.max(1)) - 1;
        let Some(p) = league.player_mut(&a.player) else {
            continue;
        };
        if p.left || p.signed.is_some() {
            continue;
        }
        p.signed = Some(Seat {
            slot: a.slot,
            replaces,
        });
        p.signed_until = Some(until);
        settle_terms(p, offer, None, false);
    }
}

/// Called by `undo()` for each round taken back: the approaches that came after it go, and an
/// early signing with them.
pub(super) fn undo(league: &mut League, n: u8) {
    let year = league.year();
    let season_end = league.phase != Phase::Racing;
    let season = league.season_mut();
    let (gone, kept): (Vec<Approach>, Vec<Approach>) = std::mem::take(&mut season.approaches)
        .into_iter()
        .partition(|a| a.round >= n);
    season.approaches = kept;
    for a in &gone {
        if a.status == ApproachStatus::Accepted
            && let Some(p) = league.player_mut(&a.player)
        {
            p.signed = None;
            p.signed_until = None;
            p.signed_salary_k = 0;
            p.signed_bonus_k = 0;
            p.signed_focus = None;
            if a.old_until.is_some() {
                p.contract_until = a.old_until;
            }
        }
    }
    league.news.retain(|m| {
        !(m.kind == NewsKind::Mail
            && m.year == year
            && m.round >= n
            && m.template.as_deref() == Some(MAIL_MEET)
            && gone
                .iter()
                .any(|a| Some(&a.player) == m.to.as_ref() && a.round == m.round))
    });
    // The season's end is taken back: the number-one clauses are those of the contracts the
    // players race under this season again.
    if season_end {
        let ids: Vec<String> = league.players.iter().map(|p| p.id.clone()).collect();
        for id in ids {
            let clause = clause_before(league, &id);
            if let Some(p) = league.player_mut(&id) {
                p.number_one_until = clause;
            }
        }
    }
}

/// The number-one clause of the contract `player` races under this season, from the contracts
/// signed at the end of earlier seasons.
fn clause_before(league: &League, player: &str) -> Option<u16> {
    let mut clause = None;
    for w in league.seasons.windows(2) {
        let Some(e) = w[1].player_entry(player) else {
            continue;
        };
        let Some(o) = w[0]
            .offers
            .iter()
            .find(|o| o.player == player && o.slot == e.slot)
        else {
            continue;
        };
        // A contract that runs on keeps its clause.
        if runs_on(o) {
            continue;
        }
        clause = o
            .agreed
            .filter(|t| t.number_one)
            .map(|t| w[1].year + u16::from(t.years.max(1)) - 1);
    }
    clause
}

/// Called by `end_season()` before the season-end offers are made: open approaches expire, and
/// early signings become the players' contracts for next season.
pub(super) fn season_end(league: &mut League) {
    restore_early(league);
    let accepted: Vec<Approach> = league
        .season()
        .approaches
        .iter()
        .filter(|a| a.status == ApproachStatus::Accepted)
        .cloned()
        .collect();
    for a in &mut league.season_mut().approaches {
        if matches!(a.status, ApproachStatus::Interest | ApproachStatus::Met) {
            a.status = ApproachStatus::Expired;
        }
    }
    for a in accepted {
        let Some(p) = league.player_mut(&a.player) else {
            continue;
        };
        if p.left || p.signed.as_ref().is_none_or(|s| s.slot != a.slot) {
            continue;
        }
        // The signed contract is the one the player is under for next season (`make_offers`
        // offers only it).
        p.contract_until = p.signed_until;
        let number_one = a
            .offer
            .as_ref()
            .and_then(|o| o.agreed)
            .is_some_and(|t| t.number_one);
        p.number_one_until = if number_one { p.signed_until } else { None };
    }
}

/// Settle the terms of the contract `p` signs with `offer` (see `signed_terms`): the agreed terms
/// when there are some. A number-one clause runs as long as the contract. Returns the salary.
pub(super) fn terms(p: &mut Player, offer: &Offer, focus: Option<Focus>) -> u32 {
    settle_terms(p, offer, focus, true)
}

/// [`terms`]; with `clause`, the number-one clause is set too (an early signing gets it when
/// the season ends, so it doesn't count for the current team).
fn settle_terms(p: &mut Player, offer: &Offer, focus: Option<Focus>, clause: bool) -> u32 {
    let salary = match offer.agreed {
        Some(t) => {
            p.signed_salary_k = t.salary_k;
            p.signed_bonus_k = offer.bonus_k;
            p.signed_focus = focus;
            t.salary_k
        }
        None => super::signed_terms(p, offer, focus),
    };
    // A contract that runs on keeps its clause; a new one brings its own, or none.
    if clause && !runs_on(offer) {
        let number_one = offer.agreed.is_some_and(|t| t.number_one);
        p.number_one_until = if number_one { p.signed_until } else { None };
    }
    salary
}

/// The early-signed contract `make_offers` offers `p` alone, as [`OfferKind::Contract`].
pub(super) fn early_offer(season: &super::Season, p: &Player) -> Option<Offer> {
    let a = season
        .approaches
        .iter()
        .find(|a| a.player == p.id && a.status == ApproachStatus::Accepted)?;
    let offer = a.offer.as_ref()?;
    let terms = agreed_or_first(offer);
    Some(Offer {
        player: p.id.clone(),
        slot: a.slot,
        kind: OfferKind::Contract,
        years: terms.years.max(1),
        salary_k: terms.salary_k,
        bonus_k: offer.bonus_k,
        status: OfferStatus::Agreed,
        agreed: Some(terms),
        early: true,
        ..Offer::fresh()
    })
}

/// `player`'s approaches this season.
pub fn approaches_of<'a>(league: &'a League, player: &str) -> Vec<&'a Approach> {
    league
        .season()
        .approaches
        .iter()
        .filter(|a| a.player == player)
        .collect()
}
