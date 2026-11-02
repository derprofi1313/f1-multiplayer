//! Consent and ready-up (W2-1): in a shared career with two or more active players, simulating,
//! skipping or taking back a round needs everyone's agreement, and players ready up for a round
//! like in the game's two-player career.
//!
//! A Simulate, Skip or Undo in a shared career becomes a proposal ([`Vote`]); the others agree
//! with [`Action::Vote`](super::Action::Vote), and the last one to agree runs it. An older
//! launcher that knows no votes agrees by asking for the same thing (its player presses Simulate
//! too), so a mixed pair of launchers still gets on.

use serde::{Deserialize, Serialize};

use super::texts::{Text, t};
use super::{Actor, League, LeagueError, Phase, Player, refuse, undo_preview};

/// What a vote is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Proposal {
    Simulate,
    Skip,
    Undo,
}

/// An open proposal the other active players agree to (or not).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vote {
    pub proposal: Proposal,
    /// The round it is about (the next round, or the round Undo takes back).
    pub round: u8,
    /// The proposer's id and name.
    pub by: String,
    pub by_name: String,
    /// The ids that agreed, the proposer first.
    pub yes: Vec<String>,
    /// For Undo, a fingerprint of everything it takes back ([`undo_preview`]): the players agreed
    /// to take back exactly that, so the vote lapses when a session is counted meanwhile. 0 for
    /// Simulate and Skip.
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub target: u64,
}

fn is_zero_u64(n: &u64) -> bool {
    *n == 0
}

/// Called before Simulate, Skip and Undo. `Ok(None)` lets the action go ahead; `Ok(Some(..))`
/// stops it with those warnings (the action became a proposal, or a vote on one).
pub(super) fn gate(
    league: &mut League,
    actor: &Actor,
    proposal: Proposal,
    round: Option<u8>,
) -> Result<Option<Vec<Text>>, LeagueError> {
    if !league.needs_consent() {
        // A proposal left from when more players were in the career goes with the action.
        league.vote = None;
        return Ok(None);
    }
    // Refuse what couldn't run anyway, before anyone is asked.
    let (round, target) = match proposal {
        Proposal::Simulate | Proposal::Skip => (league.pending_round(round)?, 0),
        Proposal::Undo => undo_target(league)?,
    };
    if let Some(open) = league.vote.clone()
        && !stale(league, &open)
    {
        if open.proposal == proposal && open.round == round {
            // Asking for what another player proposed is agreeing to it (an older launcher has
            // no other way to agree); the proposer asking again changes nothing, unless the
            // last player who hadn't agreed left the career meanwhile.
            return vote(league, actor, true, false).map(Some);
        }
        if open.by != actor.id {
            let (name, round) = (&open.by_name, open.round);
            let race = race_of(league, round).unwrap_or("?");
            return refuse(
                "vote_open",
                match open.proposal {
                    Proposal::Simulate => t!(
                        "{name} asked to simulate round {round}, the {race} first; agree to it or decline it before asking for something else",
                        name,
                        round,
                        race
                    ),
                    Proposal::Skip => t!(
                        "{name} asked to leave out round {round}, the {race} first; agree to it or decline it before asking for something else",
                        name,
                        round,
                        race
                    ),
                    Proposal::Undo => t!(
                        "{name} asked to take back round {round}, the {race} first; agree to it or decline it before asking for something else",
                        name,
                        round,
                        race
                    ),
                },
            );
        }
    }
    league.vote = Some(Vote {
        proposal,
        round,
        by: actor.id.clone(),
        by_name: actor.name.clone(),
        yes: vec![actor.id.clone()],
        target,
    });
    let text = t!("{name} asks to {what}; it happens once everyone agrees.", name = actor.name)
        .list("what", vec![what(league, proposal, round)]);
    league.say(round, text);
    Ok(Some(vec![waiting_line(league)]))
}

/// Called after a Report, Practice, Simulate, Skip or Undo changed the league: an open vote whose
/// round isn't the round it named any more (or, for Undo, that would take back something else
/// now) lapses.
pub(super) fn changed(league: &mut League) {
    let Some(open) = league.vote.clone() else {
        return;
    };
    if !stale(league, &open) {
        return;
    }
    league.vote = None;
    let text = t!("The proposal to {what} lapsed: the career moved on meanwhile.")
        .list("what", vec![what(league, open.proposal, open.round)]);
    league.say(open.round, text);
}

/// `Action::Ready`: ready (or not) for the next round.
pub(super) fn ready(league: &mut League, actor: &Actor, ready: bool) -> Result<Vec<Text>, LeagueError> {
    if league.phase != Phase::Racing {
        return refuse("phase", t!("no season is running"));
    }
    let Some(round) = league.season_mut().rounds.iter_mut().find(|r| !r.done()) else {
        return refuse("round_done", t!("every round is counted"));
    };
    let marked = round.ready.contains(&actor.id);
    if ready && !marked {
        round.ready.push(actor.id.clone());
    } else if !ready && marked {
        round.ready.retain(|id| *id != actor.id);
    }
    Ok(Vec::new())
}

/// `Action::Vote`: agree to the open proposal (the last one to agree runs it), or decline it (the
/// proposer withdraws it). `force` (the owner) runs it without waiting for the others.
pub(super) fn vote(
    league: &mut League,
    actor: &Actor,
    yes: bool,
    force: bool,
) -> Result<Vec<Text>, LeagueError> {
    let Some(open) = league.vote.clone() else {
        return refuse("no_vote", t!("there is no open proposal to vote on"));
    };
    if !league.active_players().iter().any(|p| p.id == actor.id) {
        return refuse("not_voter", t!("you aren't asked in this vote"));
    }
    if force && yes && !actor.owner {
        return refuse("owner", t!("only the career's owner goes ahead without the others"));
    }
    if stale(league, &open) {
        league.vote = None;
        return Ok(vec![
            t!("the proposal to {what} lapsed: the career moved on meanwhile")
                .list("what", vec![what(league, open.proposal, open.round)]),
        ]);
    }
    let what = what(league, open.proposal, open.round);
    let name = &actor.name;
    if !yes {
        league.vote = None;
        let text = if open.by == actor.id {
            t!("{name} withdrew the proposal to {what}.", name)
        } else {
            t!("{name} doesn't agree to {what}; it doesn't happen.", name)
        };
        league.say(open.round, text.list("what", vec![what]));
        return Ok(Vec::new());
    }
    if let Some(v) = league.vote.as_mut()
        && !v.yes.contains(&actor.id)
    {
        v.yes.push(actor.id.clone());
    }
    let waiting = league.waiting_for();
    if !waiting.is_empty() {
        if !force {
            return Ok(vec![waiting_line(league)]);
        }
        let text = t!("{name} decided to {what} without waiting for {waiting}.", name)
            .list("what", vec![what])
            .list("waiting", vec![names(&waiting)]);
        league.say(open.round, text);
    }
    run(league, &open)?;
    Ok(Vec::new())
}

/// Run an agreed proposal, as `apply` runs the action when nobody needs to agree.
fn run(league: &mut League, open: &Vote) -> Result<(), LeagueError> {
    league.vote = None;
    match open.proposal {
        Proposal::Simulate => league.simulate(Some(open.round), false)?,
        Proposal::Skip => league.skip(Some(open.round), false)?,
        Proposal::Undo => league.undo()?,
    }
    league.settle();
    changed(league);
    Ok(())
}

/// The round Undo takes back now, and a fingerprint of everything it takes back.
fn undo_target(league: &League) -> Result<(u8, u64), LeagueError> {
    if league.phase == Phase::Setup {
        return refuse("phase", t!("nothing to take back"));
    }
    let Some(preview) = undo_preview(league) else {
        return refuse("nothing", t!("no round of this season to take back"));
    };
    let bytes = serde_json::to_vec(&preview).unwrap_or_default();
    // FNV-1a; never 0, which stands for "no fingerprint".
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h = (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3);
    }
    Ok((preview.round, h.max(1)))
}

/// Whether `vote` no longer names what running it would do.
fn stale(league: &League, vote: &Vote) -> bool {
    // A proposer who left the career no longer asks for anything.
    if !league.active_players().iter().any(|p| p.id == vote.by) {
        return true;
    }
    match vote.proposal {
        Proposal::Simulate | Proposal::Skip => {
            league.phase != Phase::Racing || league.season().next_round().map(|r| r.round) != Some(vote.round)
        }
        Proposal::Undo => match undo_target(league) {
            Ok((round, target)) => round != vote.round || (vote.target != 0 && target != vote.target),
            Err(_) => true,
        },
    }
}

/// The race of `round` of the current season.
fn race_of(league: &League, round: u8) -> Option<&str> {
    league
        .seasons
        .last()
        .and_then(|s| s.round(round))
        .map(|r| r.race.as_str())
}

/// "simulate round 3, the Bahrain Grand Prix".
fn what(league: &League, proposal: Proposal, round: u8) -> Text {
    match (proposal, race_of(league, round)) {
        (Proposal::Simulate, Some(race)) => t!("simulate round {round}, the {race}", round, race),
        (Proposal::Skip, Some(race)) => t!("leave out round {round}, the {race}", round, race),
        (Proposal::Undo, Some(race)) => t!("take back round {round}, the {race}", round, race),
        (Proposal::Simulate, None) => t!("simulate round {round}", round),
        (Proposal::Skip, None) => t!("leave out round {round}", round),
        (Proposal::Undo, None) => t!("take back round {round}", round),
    }
}

/// "Waiting for Ben to agree."
fn waiting_line(league: &League) -> Text {
    t!("Waiting for {names} to agree.").list("names", vec![names(&league.waiting_for())])
}

/// "Ben", "Ben and Cleo", "Ben, Cleo and Dan".
fn names(players: &[&Player]) -> Text {
    match players {
        [] => Text::plain(""),
        [one] => Text::plain(one.name.clone()),
        [rest @ .., last] => t!(
            "{names} and {last}",
            names = rest.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", "),
            last = last.name
        ),
    }
}

impl League {
    /// The active players who haven't agreed to the open vote yet (none without a vote).
    pub fn waiting_for(&self) -> Vec<&Player> {
        let Some(vote) = &self.vote else {
            return Vec::new();
        };
        self.active_players()
            .into_iter()
            .filter(|p| !vote.yes.contains(&p.id))
            .collect()
    }

    /// Whether Simulate, Skip and Undo need everyone's agreement: two or more active players.
    pub fn needs_consent(&self) -> bool {
        self.active_players().len() >= 2
    }
}
