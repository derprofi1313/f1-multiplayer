//! The teams' league pace and R&D in the real car (W2-10): `vehicle.handling` layers that set
//! `chassis.mass` in the team archives, the lever Classic Seasons already sets each team's pace
//! with and the one the AI's per-track tables follow (`classic-seasons/tools/pace.py`).
//!
//! Classic Seasons builds each car at `798 + max(gap − head_start, 0) / percent_per_kg` kg: `gap`
//! is the team's real qualifying gap for the year, `head_start` how much slower the stock 2025
//! team's AI data it drives with (its AI slot, `tools/aislots.py`) is than the fastest team's.
//! The overlay sets the same mass from the league's pace for the next round instead
//! ([`Season::pace_at`]: the year's gap, the real in-season development, what the team carried
//! over and the parts the players' resource points paid for), so a team that develops in the
//! league is lighter in the game, and its AI driver with it.
//!
//! - Classic Seasons years (2016 to 2024) set the mass in the season addon's team archives: a
//!   layer on `swofty.classic-seasons-<year>/<team>` with the archive the addon ships
//!   (`cars/teams/<folder>/wep/<folder>.erp`, `cars/extra/…` for a team in another team's slot).
//! - The game's own years race the stock cars, so their layers target the stock archives
//!   (`vtf.stock`): 2025 the `f1_2025_vehicle_package`, 2026 the `f1_2026_vehicle_package` with its
//!   eleventh team. A stock car drives with its own AI data, so its head start is its own gap for
//!   the year and it only moves with the season's development and the R&D. A 2025 car gets the
//!   mass set like a Classic Seasons car (the AI slot gaps were measured at one mass); a 2026 car
//!   has its own stock mass (771 kg, the AI slots weren't measured for it), so it gets the
//!   difference added.
//! - Equal car performance in the game cancels all of it, so the overlay writes no cars then.
//! - [`Options::game_parts`](crate::league::Options) (experimental): the aero and powertrain
//!   parts of a Classic Seasons car become more downforce (`F1Aero` lift multipliers) and more
//!   power (`FuelECU` power scales), and their expected lap time goes back onto the mass, so the
//!   car's pace stays the same. The scales per percent of lap time are a guess until a
//!   calibration run in the game measures them; the AI team-mate follows only the mass.
//!   Durability has no proven effect in a Grand Prix and stays a simulation value.

use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::{Map, Value, json};

use super::{CarOverride, File, HANDLING_REGISTRY, Overlay, SyncContext, json_file, season_addon};
use crate::league::{League, Season, catalog_team, catalog_year, rnd};

/// The mass of the fastest car (Classic Seasons' `TOP_MASS`), kg.
pub const TOP_MASS: f32 = 798.0;
/// The heaviest car the overlay makes, kg: about 4.7 % of a lap behind the fastest.
pub const MAX_MASS: f32 = 950.0;
/// Lap time each kilogram costs in the AI's tables, in percent, when `ai_slots.json` doesn't say.
pub const PERCENT_PER_KG: f32 = 0.0308;
/// A round keeps the previous round's masses while every team's moved by less than this, kg.
pub const KEEP_BELOW_KG: f32 = 1.0;
/// The lift multipliers Classic Seasons gives every season car (`pace.py` `LIFT_VALUE`).
pub const LIFT: f32 = 0.95;
/// Experimental: the lift multiplier's relative rise per percent of lap time an aero part gains.
pub const LIFT_PER_PERCENT: f32 = 0.06;
/// Experimental: the engine power's relative rise per percent of lap time a powertrain part gains.
pub const POWER_PER_PERCENT: f32 = 0.035;
/// The `FuelECU` power scale of the lean mode on a 2025 car (the other modes are 1).
const LEAN_POWER: f32 = 0.9;
/// The stock team folders by league slot.
pub const STOCK_FOLDERS: &[(u8, &str)] = &[
    (1, "mercedes"),
    (2, "ferrari"),
    (3, "redbull"),
    (4, "mclaren"),
    (5, "force_india"),
    (6, "lotus"),
    (7, "williams"),
    (8, "toro_rosso"),
    (9, "sauber"),
    (10, "haas"),
    (11, "gm_cadillac"),
];

/// The experimental per-part feel of a car: the lift multipliers it gets (both axles) and the
/// scale on its engine power.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct PartScales {
    pub lift: f32,
    pub power: f32,
}

/// The cars whose pace the league sets for the next round, slot order; none with equal car
/// performance, outside a season or for a Classic Seasons year without its AI slots.
pub(super) fn build(league: &League, ctx: &SyncContext) -> Vec<CarOverride> {
    if league.options.rules.equal_performance == Some(true) {
        return Vec::new();
    }
    let season = league.season();
    let Some(round) = season.next_round().map(|r| r.round) else {
        return Vec::new();
    };
    let Some(masses) = masses_at(league, ctx, round) else {
        return Vec::new();
    };
    cars(season, ctx)
        .into_iter()
        .filter_map(|car| {
            let mass = *masses.get(&car.slot)?;
            let parts = (league.options.game_parts && !car.stock)
                .then(|| part_scales(season, car.slot, round))
                .flatten();
            Some(CarOverride {
                team_key: car.key,
                slot: car.slot,
                mass_kg: mass,
                erp: car.erp,
                stock: car.stock,
                parts,
            })
        })
        .collect()
}

/// `registry/handling.json`: one `vehicle.handling` layer per car. A Classic Seasons car's layer
/// is on `swofty.classic-seasons-<year>/<team>` and names the archive in that addon (`vtf` as a
/// path); a stock car's is a record of the overlay's own on the game's archive (`vtf.stock`).
pub(super) fn files(overlay: &Overlay, _ctx: &SyncContext) -> Vec<File> {
    let mut layers = Map::new();
    for car in &overlay.cars {
        let mass = car.mass_kg.round();
        let edit = if car.stock && adds_mass(overlay.year) {
            json!({ "path": "chassis.mass", "op": "add", "value": (mass - TOP_MASS) as i64 })
        } else {
            json!({ "path": "chassis.mass", "op": "set", "value": mass as i64 })
        };
        let mut layer = json!({ "edits": [edit] });
        let key = if car.stock {
            layer["vtf"] = json!({ "stock": car.erp });
            car.team_key.clone()
        } else {
            let Some(addon) = season_addon(overlay.year) else {
                continue;
            };
            layer["vtf"] = json!(car.erp);
            format!("{addon}/{}", car.team_key)
        };
        if let Some(p) = car.parts {
            // Text values: the VTF holds text, and canonical JSON keeps one decimal of a number.
            let lift = scale_text(p.lift);
            layer["f1_aero"] =
                json!({ "front_coeff_lift_multiplier": lift, "rear_coeff_lift_multiplier": lift });
            layer["fuel_ecu"] = json!({
                "fuel_power_scale_lean": scale_text(LEAN_POWER * p.power),
                "fuel_power_scale_standard": scale_text(p.power),
                "fuel_power_scale_rich": scale_text(p.power),
                "fuel_power_scale_max": scale_text(p.power),
            });
        }
        layers.insert(key, layer);
    }
    if layers.is_empty() {
        return Vec::new();
    }
    vec![json_file(
        HANDLING_REGISTRY,
        &json!({ "vehicle.handling": Value::Object(layers) }),
    )]
}

fn scale_text(v: f32) -> String {
    format!("{v:.4}")
}

/// A stock car of `year` keeps its own mass and gets the league's difference added (2026);
/// otherwise the mass is set.
fn adds_mass(year: u16) -> bool {
    year > 2025
}

/// Each team's car mass in `round`, kg, by slot: the league's pace in that round less the AI
/// slot's head start, at [`percent_per_kg`], from [`TOP_MASS`] for the fastest, clamped to
/// [`MAX_MASS`] and rounded to the kilogram. A team faster than its slot lets it be (its gap left
/// for the mass below zero) makes every other car heavier instead, so the gaps stay the league's.
/// Rounds walk from the season's first: a round keeps the masses before it while every team moved
/// by less than [`KEEP_BELOW_KG`], so the game's build of the team archives can be reused.
/// `None` for a Classic Seasons year without the AI slots.
pub fn masses_at(league: &League, ctx: &SyncContext, round: u8) -> Option<BTreeMap<u8, f32>> {
    let raw = Raw::new(league, ctx)?;
    let mut kept: Option<BTreeMap<u8, f32>> = None;
    for q in raw.season.rounds.iter().map(|r| r.round).filter(|q| *q <= round) {
        let now = raw.masses(q);
        kept = Some(match kept {
            Some(prev) if keeps(&prev, &now) => prev,
            _ => now,
        });
    }
    let kept = kept.unwrap_or_else(|| raw.masses(round));
    Some(rounded(kept))
}

/// The masses of `round` alone, as [`masses_at`] makes them when that round doesn't keep the
/// masses before it.
pub fn pace_masses(league: &League, ctx: &SyncContext, round: u8) -> Option<BTreeMap<u8, f32>> {
    Some(rounded(Raw::new(league, ctx)?.masses(round)))
}

fn rounded(masses: BTreeMap<u8, f32>) -> BTreeMap<u8, f32> {
    masses.into_iter().map(|(s, m)| (s, m.round())).collect()
}

/// What the masses of a round are made from.
struct Raw<'a> {
    season: &'a Season,
    head: BTreeMap<u8, f32>,
    ppk: f32,
    parts: bool,
}

impl<'a> Raw<'a> {
    fn new(league: &'a League, ctx: &SyncContext) -> Option<Self> {
        let season = league.season();
        let mut head = BTreeMap::new();
        for t in &season.teams {
            head.insert(t.slot, head_start(season.year, t.slot, ctx.ai_slots)?);
        }
        Some(Raw {
            season,
            head,
            ppk: percent_per_kg(ctx.ai_slots),
            parts: league.options.game_parts && season_addon(season.year).is_some(),
        })
    }

    fn masses(&self, round: u8) -> BTreeMap<u8, f32> {
        raw_masses(self.season, &self.head, self.ppk, self.parts, round)
    }
}

/// The unrounded masses of `round` (see [`masses_at`]).
fn raw_masses(
    season: &Season,
    head: &BTreeMap<u8, f32>,
    ppk: f32,
    parts: bool,
    round: u8,
) -> BTreeMap<u8, f32> {
    let pace: Vec<(u8, f32)> = season
        .teams
        .iter()
        .map(|t| (t.slot, season.pace_at(t.slot, round)))
        .collect();
    let best = pace.iter().map(|(_, p)| *p).fold(f32::INFINITY, f32::min);
    let left: Vec<(u8, f32)> = pace
        .iter()
        .map(|(slot, p)| {
            let mut x = p - best - head.get(slot).copied().unwrap_or(0.0);
            if parts {
                let profile = rnd::car_profile(season, *slot, round);
                x += profile.aero + profile.powertrain;
            }
            (*slot, x)
        })
        .collect();
    let lift = left.iter().map(|(_, x)| *x).fold(0.0_f32, f32::min);
    left.into_iter()
        .map(|(slot, x)| (slot, (TOP_MASS + (x - lift) / ppk).clamp(TOP_MASS, MAX_MASS)))
        .collect()
}

/// Whether a round keeps `prev`: the same teams, each moved by less than [`KEEP_BELOW_KG`].
pub fn keeps(prev: &BTreeMap<u8, f32>, now: &BTreeMap<u8, f32>) -> bool {
    prev.len() == now.len()
        && prev
            .iter()
            .all(|(slot, m)| now.get(slot).is_some_and(|n| (n - m).abs() < KEEP_BELOW_KG))
}

/// Lap time each kilogram costs in the AI's tables, in percent (`ai_slots.json`).
pub fn percent_per_kg(ai_slots: &Value) -> f32 {
    ai_slots
        .get("percent_per_kg")
        .and_then(Value::as_f64)
        .filter(|p| *p > 0.0)
        .map_or(PERCENT_PER_KG, |p| p as f32)
}

/// How much slower, in percent, the AI data team `slot` drives with in `year` is than the fastest
/// team's: `aislots.head_start` of Classic Seasons. Its `assign` gives the year's teams the AI
/// slots in pace order (the fastest team the fastest slot, teams beyond ten the slowest); the
/// teams' real gaps are the league catalogue's, which are Classic Seasons' `performance.json`.
/// A year the game has itself drives its own cars with their own AI data: their head start is
/// their gap for the year. `None` for a Classic Seasons year without the slots, or a team the
/// year doesn't have.
pub fn head_start(year: u16, slot: u8, ai_slots: &Value) -> Option<f32> {
    let cat = catalog_year(year)?;
    let gap = cat.teams.iter().find(|t| t.slot == slot)?.pace;
    if season_addon(year).is_none() {
        let best = cat.teams.iter().map(|t| t.pace).fold(f32::INFINITY, f32::min);
        return Some(gap - best);
    }
    let mut order: Vec<(&str, f64)> = ai_slots
        .get("slots")?
        .as_object()?
        .iter()
        .filter_map(|(folder, g)| Some((folder.as_str(), g.as_f64()?)))
        .collect();
    order.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(b.0)));
    let first = order.first()?.1;
    let mut teams: Vec<(f32, u8)> = cat.teams.iter().map(|t| (t.pace, t.slot)).collect();
    teams.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    let rank = teams.iter().position(|(_, s)| *s == slot)?;
    Some((order[rank.min(order.len() - 1)].1 - first) as f32)
}

/// The experimental part feel of team `slot`'s car in `round`; `None` without aero or
/// powertrain parts on it.
fn part_scales(season: &Season, slot: u8, round: u8) -> Option<PartScales> {
    let p = rnd::car_profile(season, slot, round);
    (p.aero > 0.0 || p.powertrain > 0.0).then_some(PartScales {
        lift: LIFT * (1.0 + LIFT_PER_PERCENT * p.aero),
        power: 1.0 + POWER_PER_PERCENT * p.powertrain,
    })
}

/// A team's car in the game: its key, the archive with its handling file and whether that is the
/// game's own.
#[derive(Debug, Clone, PartialEq)]
struct Car {
    slot: u8,
    key: String,
    erp: String,
    stock: bool,
}

/// The season's cars: the stock teams' archives for a year the game has itself, else each team
/// of the Classic Seasons box (matched by its drivers) with the archive its addon ships. A team
/// the box doesn't have is left out.
fn cars(season: &Season, ctx: &SyncContext) -> Vec<Car> {
    let year = season.year;
    if season_addon(year).is_none() {
        return season
            .teams
            .iter()
            .filter_map(|t| {
                let folder = STOCK_FOLDERS.iter().find(|(s, _)| *s == t.slot)?.1;
                Some(Car {
                    slot: t.slot,
                    key: folder.to_owned(),
                    erp: format!(
                        "2025_asset_groups/f1_{year}_vehicle_package/teams/{folder}/wep/{folder}.erp"
                    ),
                    stock: true,
                })
            })
            .collect();
    }
    let Some(teams) = ctx
        .content
        .get("content.package")
        .and_then(|p| p.get(format!("f1_{year}")))
        .and_then(|b| b.get("teams"))
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };
    season
        .teams
        .iter()
        .filter_map(|t| {
            let mut drivers: Vec<&str> = catalog_team(year, t.slot)
                .map(|c| c.drivers.iter().map(|d| d.key.as_str()).collect())
                .unwrap_or_default();
            drivers.extend(
                season
                    .entries
                    .iter()
                    .filter(|e| e.slot == t.slot && e.player.is_none())
                    .map(|e| e.id.as_str()),
            );
            let key = teams
                .iter()
                .find(|team| {
                    team.get("drivers")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(|d| d.get("driver").and_then(Value::as_str))
                        .any(|d| drivers.contains(&d.rsplit('/').next().unwrap_or(d)))
                })?
                .get("key")?
                .as_str()?;
            Some(Car {
                slot: t.slot,
                key: key.to_owned(),
                erp: archive(ctx, year, key)?,
                stock: false,
            })
        })
        .collect()
}

/// The archive, in the season addon, that holds the handling file of the box's team `key`: its
/// vehicle's package path, with the asset file name for `[FILENAME]`, is the target of a
/// `file.put` of the addon's liveries; a team the liveries don't ship (one in another team's
/// slot, from `assets/cars.json`) has it where the addon's layout puts it (`cars/…` for the
/// `cs_<year>` package, `cars/extra/…` for `cx_<year>`).
fn archive(ctx: &SyncContext, year: u16, key: &str) -> Option<String> {
    let vehicle = ctx.content.get("vehicle")?.get(key)?;
    let package = vehicle.get("package")?.as_str()?;
    let package = package.rsplit('/').next().unwrap_or(package);
    let path = ctx
        .content
        .get("vehicle.package")?
        .get(package)?
        .get("resource_path")?
        .as_str()?;
    let file = vehicle
        .get("ai")
        .and_then(|a| a.get("asset_filename"))
        .and_then(Value::as_str)
        .unwrap_or(key);
    let path = path.replace("[FILENAME]", file);
    let target = format!("2025_asset_groups/{path}.erp");
    let shipped = ctx
        .liveries
        .get("ops")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|op| op.get("op").and_then(Value::as_str) == Some("file.put"))
        .find(|op| {
            op.get("target")
                .and_then(Value::as_str)
                .is_some_and(|t| t.eq_ignore_ascii_case(&target))
        })
        .and_then(|op| op.get("source")?.as_str())
        .map(str::to_owned);
    let erp = shipped.or_else(|| {
        let (pkg, rest) = path.split_once('/')?;
        if pkg == format!("cs_{year}_vehicle_package") {
            Some(format!("cars/{rest}.erp"))
        } else if pkg == format!("cx_{year}_vehicle_package") {
            Some(format!("cars/extra/{rest}.erp"))
        } else {
            None
        }
    })?;
    let safe = !erp.is_empty()
        && !erp.starts_with('/')
        && !erp.contains('\\')
        && erp.split('/').all(|p| !p.is_empty() && p != "." && p != "..");
    safe.then_some(erp)
}
