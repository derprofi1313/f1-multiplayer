//! Who drives which car in each round, and how fast each car is.
//!
//! A season starts with its real grid, the players in their seats. The real mid-season line-up
//! changes ([`Change`]) follow from the round they happened in: a stand-in for an injured or
//! ill driver, a driver who moved to another team, a driver who was replaced. A change that would
//! touch a player's seat is left out. The game's season box keeps its grid all year, so a car the
//! game reports is the seat it starts the season in, driven by whoever has that seat in the
//! round ([`Season::seat_holder`]).
//!
//! A car's gap to the fastest car moves over the season as it did for real (the
//! [`Checkpoint`](super::Checkpoint)s of the season data), with what the team carries over from
//! the years before ([`Team::carry`](super::Team::carry)) and the upgrades the players' resource
//! points paid for ([`Upgrade`](super::progress::Upgrade)).

use serde::{Deserialize, Serialize};

use super::progress::Part;
use super::{Entry, Player, Season, Seat, catalog_team, catalog_year};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Change {
    /// The first round of the change.
    pub round: u8,
    /// The round `out` is back in the seat, or `None` for good.
    pub until: Option<u8>,
    pub slot: u8,
    /// The entry id of the driver who loses the seat.
    pub out: String,
    /// The entry id of the driver who takes it.
    pub driver: String,
    /// `stand_in`, `move` or `replace`.
    pub kind: String,
    /// The news told about it.
    #[serde(default)]
    pub announced: bool,
}

impl Change {
    pub fn active(&self, round: u8) -> bool {
        self.round <= round && self.until.is_none_or(|u| round < u)
    }
}

/// Of `year`'s line-up changes, the ones that don't touch the players' seats, from round `from`
/// on (a change that started before is still in force from `from`, one that ended is left out),
/// and the drivers they bring who aren't on the starting grid.
pub(super) fn changes_for(year: u16, seats: &[(Player, Seat)], from: u8) -> (Vec<Change>, Vec<Entry>) {
    let Some(cat) = catalog_year(year) else {
        return (Vec::new(), Vec::new());
    };
    let taken: Vec<&str> = seats.iter().map(|(_, s)| s.replaces.as_str()).collect();
    let on_grid = |key: &str| cat.teams.iter().flat_map(|t| &t.drivers).any(|d| d.key == key);
    let mut changes = Vec::new();
    let mut reserves: Vec<Entry> = Vec::new();
    for c in &cat.changes {
        if taken.contains(&c.out.as_str()) || taken.contains(&c.driver.as_str()) {
            continue;
        }
        if c.until.is_some_and(|u| u <= from) {
            continue;
        }
        changes.push(Change {
            round: c.round.max(from),
            until: c.until,
            slot: c.slot,
            out: c.out.clone(),
            driver: c.driver.clone(),
            kind: c.kind.clone(),
            announced: false,
        });
        if !on_grid(&c.driver) && !reserves.iter().any(|e| e.id == c.driver) {
            reserves.push(Entry {
                id: c.driver.clone(),
                name: c.name.clone(),
                number: c.number,
                slot: c.slot,
                rating: c.rating,
                player: None,
            });
        }
    }
    (changes, reserves)
}

impl Season {
    pub fn changes(&self) -> &[Change] {
        self.changes.as_deref().unwrap_or(&[])
    }

    /// The round the season is at: the next round to drive, or the last one once all are done.
    pub fn current_round(&self) -> u8 {
        self.next_round().or(self.rounds.last()).map_or(1, |r| r.round)
    }

    /// The team `entry` races for in `round`, or `None` when it doesn't race then.
    pub fn slot_at(&self, entry: &str, round: u8) -> Option<u8> {
        let changes = self.changes();
        if let Some(c) = changes.iter().find(|c| c.active(round) && c.driver == entry) {
            return Some(c.slot);
        }
        if changes.iter().any(|c| c.active(round) && c.out == entry) {
            return None;
        }
        self.entries.iter().find(|e| e.id == entry).map(|e| e.slot)
    }

    /// The cars of `round`: every entry that races then, with its team.
    pub fn lineup(&self, round: u8) -> Vec<(&Entry, u8)> {
        self.entries
            .iter()
            .chain(&self.reserves)
            .filter_map(|e| Some((e, self.slot_at(&e.id, round)?)))
            .collect()
    }

    /// Who drives, in `round`, the seat `entry` starts the season in: the game's season box keeps
    /// its grid, so its car of `entry` is that seat.
    pub fn seat_holder<'a>(&'a self, entry: &'a Entry, round: u8) -> Option<&'a Entry> {
        if self.reserves.iter().any(|e| e.id == entry.id) {
            return self.slot_at(&entry.id, round).map(|_| entry);
        }
        match self
            .changes()
            .iter()
            .find(|c| c.active(round) && c.out == entry.id && c.slot == entry.slot)
        {
            Some(c) => self.entry(&c.driver),
            None => Some(entry),
        }
    }

    /// The car's gap to the fastest car of the year in `round`, in percent: its gap for the year,
    /// what it carries over, its real development so far and the upgrades on it.
    pub fn pace_at(&self, slot: u8, round: u8) -> f32 {
        let Some(team) = self.team(slot) else {
            return 3.0;
        };
        let rounds = self.rounds.len().max(1) as f32;
        let at = (f32::from(round.max(1)) - 0.5) / rounds;
        let development = catalog_team(self.year, slot).map_or(0.0, |t| interpolate(&t.development, at));
        team.pace + team.carry + development - self.upgrade_gain(slot, round)
    }

    /// The car's gap now (at the round the season is at).
    pub fn pace_now(&self, slot: u8) -> f32 {
        self.pace_at(slot, self.current_round())
    }

    /// What the upgrades on the car in `round` gain, in percent of a lap: the arrived parts that
    /// didn't fail, nodes of the development tree with their gain and older linear levels with
    /// [`Part::gain`] ([`super::rnd::gain`]).
    pub fn upgrade_gain(&self, slot: u8, round: u8) -> f32 {
        super::rnd::gain(self, slot, round)
    }

    /// The level of `part` the team has ordered this season: every part it ordered, failed ones
    /// too (what it can order next is [`super::rnd::available`]).
    pub fn part_level(&self, slot: u8, part: Part) -> u8 {
        self.upgrades
            .iter()
            .filter(|u| u.slot == slot && u.part == part)
            .count() as u8
    }

    /// How often the car breaks down in `round`, relative to an undeveloped one: less with each
    /// durability part on the car that didn't fail ([`super::rnd::car_profile`]), never below 30%.
    pub fn breakdowns(&self, slot: u8, round: u8) -> f32 {
        (1.0 - super::rnd::car_profile(self, slot, round).durability).max(0.3)
    }
}

/// The development delta at `at` (0 to 1 of the season), linear between checkpoints and flat
/// before the first and after the last.
fn interpolate(points: &[super::Checkpoint], at: f32) -> f32 {
    let (Some(first), Some(last)) = (points.first(), points.last()) else {
        return 0.0;
    };
    if at <= first.at {
        return first.delta;
    }
    if at >= last.at {
        return last.delta;
    }
    points
        .windows(2)
        .find(|w| at >= w[0].at && at <= w[1].at)
        .map_or(last.delta, |w| {
            let t = (at - w[0].at) / (w[1].at - w[0].at).max(1e-6);
            w[0].delta + (w[1].delta - w[0].delta) * t
        })
}
