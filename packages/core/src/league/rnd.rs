//! Research and development: the parts players order for their team with resource points, and
//! what a team carries into the next season.
//!
//! Each part of the car has a small development tree ([`TREE`]): a main branch of five steady
//! steps and an alternative branch of five bigger, riskier ones. A node takes two to four rounds
//! to build. When it arrives, a seeded roll can fail it: it gains nothing, and half of its
//! resource points go back to the player who paid for it. A team may spend [`COST_CAP`] resource
//! points a season, whoever of its drivers pays.
//!
//! Upgrades from before the tree (`node: None`) keep their linear gain ([`Part::gain`]) and count
//! as the first steps of the part's main branch.

use serde::{Deserialize, Serialize};

use super::progress::{Part, Upgrade};
use super::texts::{Text, t};
use super::{Actor, League, LeagueError, Phase, Season, refuse};

/// Resource points spent this season on a team (the cost cap).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spend {
    pub slot: u8,
    pub rp: u32,
}

/// A node of a part's development tree.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Node {
    /// Unique within the part: 1 to 5 the main branch, 6 to 10 the alternative one.
    pub id: u8,
    pub part: Part,
    /// 0 the main branch, 1 the alternative one.
    pub branch: u8,
    pub name: &'static str,
    pub cost: u32,
    /// Lap time gained, in percent; for durability, how much less often the car breaks down
    /// (0.12 = 12% fewer breakdowns).
    pub gain: f32,
    /// Rounds it takes to build: ordered before round `n`, it is on the car from `n + rounds`.
    pub rounds: u8,
    /// The chance it fails when it arrives, 0 to 1.
    pub fail: f32,
    /// The node it needs ordered first.
    pub requires: Option<u8>,
}

/// What a car's developed parts gain, per part: in percent of a lap for aero, chassis and
/// powertrain; for durability, how much less often the car breaks down (0.12 = 12% fewer).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize)]
pub struct CarProfile {
    pub aero: f32,
    pub chassis: f32,
    pub powertrain: f32,
    pub durability: f32,
}

impl CarProfile {
    /// The lap time all parts gain, in percent (durability gains none).
    pub fn lap_time(&self) -> f32 {
        self.aero + self.chassis + self.powertrain
    }
}

/// Resource points a team may spend in a season: a strong season (about 11,000 resource points
/// simulated, 15,000 to 17,000 driven with telemetry practice) finishes about one and a half main
/// branches (9,750) and has to choose which; two team-mates together hit it sooner.
pub const COST_CAP: u32 = 10_000;

/// The main branch's chance that a node fails.
pub const MAIN_FAIL: f32 = 0.05;
/// The alternative branch's chance that a node fails.
pub const ALTERNATIVE_FAIL: f32 = 0.20;
/// How much less often the car breaks down per legacy durability level.
const LEGACY_DURABILITY: f32 = 0.12;

/// What the next level of a part costs, with `level` levels ordered this season.
pub const fn upgrade_cost(level: u8) -> u32 {
    800 + 250 * level as u32
}

/// The ten nodes of a part: five main ones (gains `main`) and five alternative ones (gains
/// `alternative`), named by `names`.
const fn branches(part: Part, main: f32, alternative: f32, names: [&'static str; 10]) -> [Node; 10] {
    const MAIN_ROUNDS: [u8; 5] = [2, 2, 2, 3, 3];
    const ALTERNATIVE_ROUNDS: [u8; 5] = [3, 3, 3, 4, 4];
    let mut out = [Node {
        id: 0,
        part,
        branch: 0,
        name: "",
        cost: 0,
        gain: 0.0,
        rounds: 0,
        fail: 0.0,
        requires: None,
    }; 10];
    let mut i = 0;
    while i < 10 {
        let alt = i >= 5;
        let step = (i % 5) as u8;
        let id = i as u8 + 1;
        out[i] = Node {
            id,
            part,
            branch: alt as u8,
            name: names[i],
            cost: if alt {
                upgrade_cost(step + 1)
            } else {
                upgrade_cost(step)
            },
            gain: if alt { alternative } else { main },
            rounds: if alt {
                ALTERNATIVE_ROUNDS[step as usize]
            } else {
                MAIN_ROUNDS[step as usize]
            },
            fail: if alt { ALTERNATIVE_FAIL } else { MAIN_FAIL },
            requires: if step == 0 { None } else { Some(id - 1) },
        };
        i += 1;
    }
    out
}

/// The development trees, in [`Part::ALL`] order.
pub static TREE: [[Node; 10]; 4] = [
    branches(
        Part::Aero,
        0.07,
        0.12,
        [
            "Front wing endplates",
            "Floor edge",
            "Rear wing flap",
            "Diffuser strakes",
            "Bargeboards",
            "High-downforce front wing",
            "Radical floor",
            "Low-drag rear wing",
            "Double diffuser",
            "New aero concept",
        ],
    ),
    branches(
        Part::Chassis,
        0.05,
        0.09,
        [
            "Suspension geometry",
            "Brake cooling",
            "Weight saving",
            "Stiffer monocoque",
            "Ride height control",
            "Pull-rod rear suspension",
            "Carbon brake package",
            "Lightweight chassis",
            "Hydraulic suspension",
            "Chassis concept B",
        ],
    ),
    branches(
        Part::Powertrain,
        0.06,
        0.10,
        [
            "Engine mapping",
            "Turbo tune",
            "ERS harvesting",
            "Combustion upgrade",
            "Gearbox ratios",
            "New fuel blend",
            "Split turbo",
            "High-power ERS",
            "Combustion chamber redesign",
            "Power unit spec 2",
        ],
    ),
    branches(
        Part::Durability,
        0.12,
        0.20,
        [
            "Cooling louvres",
            "Gearbox seals",
            "Hydraulics reroute",
            "Electronics shielding",
            "Power unit cooling",
            "Lifed components",
            "Reinforced gearbox",
            "Redundant hydraulics",
            "Sealed electronics",
            "Bulletproof spec",
        ],
    ),
];

/// The development tree of `part`.
pub fn nodes(part: Part) -> &'static [Node] {
    let i = Part::ALL.iter().position(|&p| p == part).unwrap_or(0);
    &TREE[i]
}

/// The node `id` of `part`'s tree.
pub fn node(part: Part, id: u8) -> Option<&'static Node> {
    nodes(part).iter().find(|n| n.id == id)
}

/// The nodes of `part` team `slot` has ordered this season and that didn't fail: the nodes
/// ordered, and for each legacy level (an upgrade from before the tree) the next main node.
fn taken(season: &Season, slot: u8, part: Part) -> Vec<u8> {
    let mine = season
        .upgrades
        .iter()
        .filter(|u| u.slot == slot && u.part == part && !u.failed);
    let mut ids: Vec<u8> = mine.clone().filter_map(|u| u.node).collect();
    let legacy = mine.filter(|u| u.node.is_none()).count().min(5) as u8;
    ids.extend(1..=legacy);
    ids
}

/// The nodes of `part` team `slot` can order now: not ordered yet (or failed), with the node
/// they need ordered.
pub fn available(season: &Season, slot: u8, part: Part) -> Vec<&'static Node> {
    let taken = taken(season, slot, part);
    nodes(part)
        .iter()
        .filter(|n| !taken.contains(&n.id) && n.requires.is_none_or(|r| taken.contains(&r)))
        .collect()
}

/// The next node of `part`'s main branch team `slot` can order, or `None` when the main branch
/// is complete.
pub fn next_main(season: &Season, slot: u8, part: Part) -> Option<&'static Node> {
    let taken = taken(season, slot, part);
    nodes(part)
        .iter()
        .find(|n| n.branch == 0 && !taken.contains(&n.id))
}

/// What an upgrade does once it is on the car: its node's gain, or a legacy level's.
fn effect(u: &Upgrade) -> f32 {
    match u.node {
        Some(id) => node(u.part, id).map_or(0.0, |n| n.gain),
        None if u.part == Part::Durability => LEGACY_DURABILITY,
        None => u.part.gain(),
    }
}

/// What the parts on team `slot`'s car in `round` gain, per part: the arrived nodes that didn't
/// fail, and the legacy levels.
pub fn car_profile(season: &Season, slot: u8, round: u8) -> CarProfile {
    let of = |part: Part| -> f32 {
        season
            .upgrades
            .iter()
            .filter(|u| u.slot == slot && u.part == part && u.ready <= round && !u.failed)
            .map(effect)
            .sum()
    };
    CarProfile {
        aero: of(Part::Aero),
        chassis: of(Part::Chassis),
        powertrain: of(Part::Powertrain),
        durability: of(Part::Durability),
    }
}

/// What the upgrades on team `slot`'s car in `round` gain, in percent of a lap
/// ([`Season::upgrade_gain`]).
pub(super) fn gain(season: &Season, slot: u8, round: u8) -> f32 {
    car_profile(season, slot, round).lap_time()
}

/// What a failed part gives back to the player who paid for it.
fn refund(u: &Upgrade) -> u32 {
    u.cost / 2
}

/// The resource points team `slot` spent this season: what it ordered, less what failed parts
/// gave back.
pub fn spent(season: &Season, slot: u8) -> u32 {
    season
        .upgrades
        .iter()
        .filter(|u| u.slot == slot)
        .map(|u| if u.failed { u.cost - refund(u) } else { u.cost })
        .sum()
}

/// Keep [`Season::spent`] in step with the upgrades.
fn sync_spent(season: &mut Season) {
    let mut slots: Vec<u8> = season.upgrades.iter().map(|u| u.slot).collect();
    slots.sort_unstable();
    slots.dedup();
    season.spent = slots
        .into_iter()
        .map(|slot| Spend {
            slot,
            rp: spent(season, slot),
        })
        .filter(|s| s.rp > 0)
        .collect();
}

/// splitmix64 over the inputs: the same roll on every PC and after an undo.
fn roll(seed: u64, parts: &[u64]) -> f32 {
    let mut x = seed;
    for p in parts {
        x ^= *p;
        x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = x;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        x = z ^ (z >> 31);
    }
    (x >> 40) as f32 / (1u64 << 24) as f32
}

/// Whether `u` fails when it arrives, for the league's `seed` in `year`.
pub(super) fn fails(seed: u64, year: u16, u: &Upgrade) -> bool {
    let Some(id) = u.node else {
        return false;
    };
    let Some(n) = node(u.part, id) else {
        return false;
    };
    let part = Part::ALL.iter().position(|&p| p == u.part).unwrap_or(0) as u64;
    let by = u.by.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    });
    roll(
        seed ^ 0x5244_2d74_7265_6501,
        &[
            u64::from(year),
            u64::from(u.slot),
            part,
            u64::from(id),
            u64::from(u.round),
            by,
        ],
    ) < n.fail
}

/// Round `n` was completed: the nodes on the car from round `n + 1` arrive, and a seeded roll can
/// fail each one.
pub(super) fn after_round(league: &mut League, n: u8) {
    let (seed, year) = (league.seed, league.year());
    let season = league.season();
    let arriving: Vec<usize> = season
        .upgrades
        .iter()
        .enumerate()
        .filter(|(_, u)| u.node.is_some() && !u.failed && u.ready == n.saturating_add(1))
        .map(|(i, _)| i)
        .collect();
    if arriving.is_empty() {
        league.sync_spent();
        return;
    }
    let race = season
        .round(n.saturating_add(1))
        .map_or_else(String::new, |r| r.race.clone());
    let mut fitted = Vec::new();
    let mut failed = Vec::new();
    for i in arriving {
        let u = league.season().upgrades[i].clone();
        let team = league
            .season()
            .team(u.slot)
            .map_or_else(String::new, |t| t.name.clone());
        let name = u.node.and_then(|id| node(u.part, id)).map_or("", |n| n.name);
        if fails(seed, year, &u) {
            let back = refund(&u);
            league.season_mut().upgrades[i].failed = true;
            let payer = match league.player_mut(&u.by) {
                Some(p) => {
                    p.rp += back;
                    p.name.clone()
                }
                None => String::new(),
            };
            failed.push(t!(
                "{team}'s new {node} ({part}) fails its tests and gains nothing; {back} of its {cost} resource points go back to {payer}.",
                team,
                node = name,
                part = u.part.name(),
                back,
                cost = u.cost,
                payer
            ));
        } else {
            fitted.push(t!("{team}'s {node} ({part})", team, node = name, part = u.part.name()));
        }
    }
    let mut lines = Vec::new();
    if !fitted.is_empty() {
        lines.push(t!("On the car for the {race}: {fitted}.", race).list("fitted", fitted));
    }
    lines.extend(failed);
    league.say_lines(n, lines);
    league.sync_spent();
}

/// Rounds from `n` on were taken back: the nodes that arrived after round `n` haven't arrived
/// yet, so they haven't failed either (the snapshot of round `n` restores the points they gave
/// back).
pub(super) fn undo(league: &mut League, n: u8) {
    for u in &mut league.season_mut().upgrades {
        if u.node.is_some() && u.ready > n {
            u.failed = false;
        }
    }
    league.sync_spent();
}

/// `Action::DevelopNode`.
pub(super) fn develop_node(
    league: &mut League,
    actor: &Actor,
    part: Part,
    node: u8,
) -> Result<Vec<Text>, LeagueError> {
    league.order(&actor.id, part, Some(node))?;
    Ok(Vec::new())
}

impl League {
    fn sync_spent(&mut self) {
        sync_spent(self.season_mut());
    }

    /// Order the next node of `part`'s main branch for `player`'s team with their resource points
    /// (`Action::Develop`).
    pub(super) fn develop(&mut self, player: &str, part: Part) -> Result<Text, LeagueError> {
        self.order(player, part, None)
    }

    /// Order node `id` of `part` (the next main node with `None`) for `player`'s team.
    fn order(&mut self, player: &str, part: Part, id: Option<u8>) -> Result<Text, LeagueError> {
        if self.phase != Phase::Racing {
            return refuse("phase", t!("parts are developed while a season runs"));
        }
        let season = self.season();
        let Some(entry) = season.player_entry(player) else {
            return refuse("no_seat", t!("you have no seat this season"));
        };
        let slot = entry.slot;
        let Some(next) = season.next_round() else {
            return refuse("phase", t!("the season is over"));
        };
        let (n, started) = (next.round, next.started());
        let node = match id {
            None => match next_main(season, slot, part) {
                Some(node) => node,
                None => {
                    return refuse(
                        "max_level",
                        t!(
                            "{part}'s main branch is developed as far as it goes this season",
                            part = part.name()
                        ),
                    );
                }
            },
            Some(id) => match available(season, slot, part).into_iter().find(|x| x.id == id) {
                Some(node) => node,
                None => {
                    let text = match self::node(part, id) {
                        None => t!("{part} has no development node {id}", part = part.name(), id),
                        Some(x) => match x.requires.and_then(|r| self::node(part, r)) {
                            Some(r) if !taken(season, slot, part).contains(&r.id) => {
                                t!("{node} needs {needs} first", node = x.name, needs = r.name)
                            }
                            _ => t!("{node} is ordered already", node = x.name),
                        },
                    };
                    return refuse("node_locked", text);
                }
            },
        };
        let have = self.player(player).map_or(0, |p| p.rp);
        if have < node.cost {
            return refuse(
                "no_points",
                t!(
                    "{node} ({part}) costs {cost} resource points; you have {have}",
                    node = node.name,
                    part = part.name(),
                    cost = node.cost,
                    have
                ),
            );
        }
        let spent = spent(season, slot);
        if spent + node.cost > COST_CAP {
            return refuse(
                "cost_cap",
                t!(
                    "{node} would take the team over the cost cap: it has spent {spent} of {cap} resource points this season",
                    node = node.name,
                    spent,
                    cap = COST_CAP
                ),
            );
        }
        let ready = n + node.rounds;
        let last = season.rounds.last().map_or(0, |r| r.round);
        if ready > last {
            return refuse(
                "too_late",
                t!(
                    "{node} takes {rounds} rounds to build; it wouldn't be ready this season",
                    node = node.name,
                    rounds = node.rounds
                ),
            );
        }
        let level = season.part_level(slot, part);
        let team = season.team(slot).map_or_else(String::new, |t| t.name.clone());
        let race = season.round(ready).map_or_else(String::new, |r| r.race.clone());
        let name = self.player(player).map_or_else(String::new, |p| p.name.clone());
        self.player_mut(player).expect("seated").rp -= node.cost;
        self.season_mut().upgrades.push(Upgrade {
            slot,
            part,
            level: level + 1,
            by: player.to_owned(),
            cost: node.cost,
            round: n,
            started,
            ready,
            node: Some(node.id),
            failed: false,
        });
        self.sync_spent();
        let text = t!(
            "{name} has {team} develop {part} level {level} ({node}); it's on the car from round {ready}, the {race}.",
            name,
            team,
            part = part.name(),
            level = level + 1,
            node = node.name,
            ready,
            race
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
}
