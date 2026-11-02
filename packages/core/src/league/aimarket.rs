//! The optional dynamic AI driver market (W4-1, [`LineupMode::Dynamic`], off by default): at
//! each season's end the AI drivers move between teams, retire and make way for rookies, as the
//! game's own career simulates them. The historic grid stays the default.
//!
//! [`plan`] runs when the next season is built (`League::next_season`):
//! - It starts from last season's AI line-up at its last round, not from the season data.
//! - Drivers retire after the last year the season data has them in.
//! - A team keeps as many AI drivers as it has seats the players didn't take; the lowest-rated
//!   go. A team that is gone leaves its drivers without a seat.
//! - The lowest-rated AI driver of each team loses the seat with a chance (more often when rated
//!   below the field's middle).
//! - A driver rated well above their car moves up with a chance: into a free seat of a faster
//!   team, or swapping with a faster team's lowest-rated driver who is clearly slower.
//! - Free seats go to the best of the drivers without a seat and the rookies: drivers of the next
//!   few years' season data who were never on the career's grid, at their rating then less
//!   [`ROOKIE_MALUS`], else made-up drivers ([`GENERATED`]).
//!
//! Every roll is seeded by the league's seed and the year, so every PC makes the same market.
//! The players' seats are untouched, and the real mid-season line-up changes are off in a
//! dynamic season (the season data's changes are about the real drivers).
//!
//! The game's season box keeps the real grid, so the league seats its drivers in the box's
//! seats: a seat whose box driver isn't the league's driver is renamed
//! ([`Season::renames`](super::Season)) and the game shows the league's driver through
//! [`gamesync::market`](super::gamesync::market). The season's entries carry the shown driver's
//! key, name and number, so a report matches as usual; a report from a game without the
//! renames is matched by seat ([`box_shown`]).

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::OnceLock;

use super::gamesync::SeatRename;
use super::gamesync::identity::{base_code, game_nationality, split_name, unique_code};
use super::inbox::render;
use super::progress::mix;
use super::{
    Entry, League, LeagueError, NEWS_MAX, News, Player, ReportCar, Season, Seat, build_season, catalog,
    catalog_driver, catalog_year, same_driver, words,
};

/// A driver rated this many team places above their car moves up, with [`MOVE_CHANCE`].
pub const MOVE_GAP: usize = 3;
/// A move up goes at least this many team places up.
pub const MOVE_STEP: usize = 2;
/// The chance a driver rated well above their car moves up.
pub const MOVE_CHANCE: f64 = 0.35;
/// A move into a full team swaps with its lowest-rated driver when they are rated this much lower.
pub const SWAP_GAP: f32 = 2.0;
/// The chance a team's lowest-rated AI driver rated below the field's middle loses the seat.
pub const DROP_CHANCE_WEAK: f64 = 0.25;
/// The chance the lower-rated of a team's two AI drivers otherwise loses the seat.
pub const DROP_CHANCE: f64 = 0.05;
/// Rookies drive rated this much below their rating in the season data.
pub const ROOKIE_MALUS: f32 = 3.0;
/// Rookies come from the season data of the next season and this many years after it.
pub const ROOKIE_YEARS: u16 = 2;
/// The chance a team with a free seat hires a driver of a slower team rated [`SWAP_GAP`] above
/// the best newcomer.
pub const PROMOTE_CHANCE: f64 = 0.6;
/// A made-up driver's rating in the first season, give or take 2.
pub const GENERATED_RATING: f32 = 71.0;
/// A made-up driver's rating grows this much a season, up to [`GENERATED_PEAK`].
pub const GENERATED_GROWTH: f32 = 1.0;
pub const GENERATED_PEAK: f32 = 80.0;

/// What made the seeds of the market differ from other seeded rolls of the league.
const SALT: u64 = 0x4149_4d41_524b_4554; // "AIMARKET"

// The news templates: constants named T_*, translation keys of the launcher.
pub const T_RETIRED: &str = "{driver} retires from Formula 1 after the {last} season.";
pub const T_DROPPED: &str = "{team} doesn't keep {driver} for {year}.";
pub const T_NO_SEAT: &str = "{driver} has no seat for {year}.";
pub const T_TRANSFER: &str = "{driver} moves from {from} to {team} for {year}.";
pub const T_ROOKIE: &str = "{team} gives {driver} a Formula 1 debut in {year}.";
pub const T_RETURN: &str = "{driver} is back in Formula 1 with {team} for {year}.";

/// The English templates of the market's news (translation keys), with `{placeholders}` filled
/// from `News.args`.
pub const TEMPLATES: &[&str] = &[T_RETIRED, T_DROPPED, T_NO_SEAT, T_TRANSFER, T_ROOKIE, T_RETURN];

/// Made-up drivers for free seats when the season data has no rookie left: a name and an ISO
/// 3166-1 alpha-3 nationality. No first or last name is a real driver's of the season data, so a
/// report never takes one for the other.
pub const GENERATED: [(&str, &str); 24] = [
    ("Luca Ferraro", "ITA"),
    ("Jonas Brandt", "DEU"),
    ("Mathis Lefevre", "FRA"),
    ("Harvey Hayes", "GBR"),
    ("Diego Salvatierra", "ESP"),
    ("Kenji Morimoto", "JPN"),
    ("Tomas Novak", "CZE"),
    ("Rafael Moura", "BRA"),
    ("Erik Lindqvist", "SWE"),
    ("Sean Callaghan", "IRL"),
    ("Mateo Quiroga", "ARG"),
    ("Noah Fischer", "CHE"),
    ("Daan Visser", "NLD"),
    ("Callum Reid", "AUS"),
    ("Marco Bellini", "ITA"),
    ("Felix Aaltonen", "FIN"),
    ("Hugo Marchand", "FRA"),
    ("Tyler Brooks", "USA"),
    ("Arjun Mehta", "IND"),
    ("Kacper Wojcik", "POL"),
    ("Leo Santana", "MEX"),
    ("Emil Kjaer", "DNK"),
    ("Owen Fletcher", "CAN"),
    ("Victor Ionescu", "ROU"),
];

/// The nationalities (ISO 3166-1 alpha-3) of the season data's drivers, by key.
pub const NATIONALITIES: &[(&str, &str)] = &[
    ("alex_albon", "THA"),
    ("andrea-kimi_antonelli", "ITA"),
    ("antonio_giovinazzi", "ITA"),
    ("arvid_lindblad", "GBR"),
    ("brendon_hartley", "NZL"),
    ("carlos_sainz-jr", "ESP"),
    ("charles_leclerc", "MCO"),
    ("daniel_ricciardo", "AUS"),
    ("daniil_kvyat", "RUS"),
    ("esteban_gutierrez", "MEX"),
    ("esteban_ocon", "FRA"),
    ("felipe_massa", "BRA"),
    ("felipe_nasr", "BRA"),
    ("fernando_alonso", "ESP"),
    ("franco_colapinto", "ARG"),
    ("gabriel_bortoleto", "BRA"),
    ("george_russell", "GBR"),
    ("guanyu_zhou", "CHN"),
    ("isack_hadjar", "FRA"),
    ("jack_aitken", "GBR"),
    ("jack_doohan", "AUS"),
    ("jenson_button", "GBR"),
    ("jolyon_palmer", "GBR"),
    ("kevin_magnussen", "DNK"),
    ("kimi_raikkonen", "FIN"),
    ("lance_stroll", "CAN"),
    ("lando_norris", "GBR"),
    ("lewis_hamilton", "GBR"),
    ("liam_lawson", "NZL"),
    ("logan_sargeant", "USA"),
    ("marcus_ericsson", "SWE"),
    ("max_verstappen", "NLD"),
    ("mick_schumacher", "DEU"),
    ("nicholas_latifi", "CAN"),
    ("nico_hulkenberg", "DEU"),
    ("nico_rosberg", "DEU"),
    ("nikita_mazepin", "RUS"),
    ("nyck_de-vries", "NLD"),
    ("oliver_bearman", "GBR"),
    ("oscar_piastri", "AUS"),
    ("pascal_wehrlein", "DEU"),
    ("pierre_gasly", "FRA"),
    ("rio_haryanto", "IDN"),
    ("robert_kubica", "POL"),
    ("romain_grosjean", "FRA"),
    ("sebastian_vettel", "DEU"),
    ("sergey_sirotkin", "RUS"),
    ("sergio_perez", "MEX"),
    ("stoffel_vandoorne", "BEL"),
    ("valtteri_bottas", "FIN"),
    ("yuki_tsunoda", "JPN"),
];

/// What happened to a driver in the market.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveKind {
    /// Past the last year of the season data.
    Retired,
    /// The team didn't keep them.
    Dropped,
    /// Their team is gone or full of players, and no free seat was left.
    NoSeat,
    /// To another team.
    Transfer,
    /// A first season in the career: a rookie or a made-up driver.
    Rookie,
    /// A driver of the season data who wasn't on the career's grid before.
    Return,
}

/// A driver's move, for the news.
#[derive(Debug, Clone, PartialEq)]
pub struct Move {
    pub kind: MoveKind,
    pub key: String,
    pub name: String,
    /// Last season's team.
    pub from: Option<u8>,
    /// Next season's team.
    pub to: Option<u8>,
}

/// The next season of a dynamic line-up, and the moves that made it.
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub season: Season,
    pub moves: Vec<Move>,
}

/// A driver in the market.
#[derive(Debug, Clone)]
struct Driver {
    key: String,
    name: String,
    number: u8,
    rating: f32,
    /// Last season's team.
    from: Option<u8>,
    /// Next season's team, `None` while without a seat.
    slot: Option<u8>,
    moved: bool,
    /// New in the career (a rookie, a made-up driver, a driver back).
    new: Option<MoveKind>,
}

/// One year of a driver in the season data.
#[derive(Debug, Clone, Copy)]
struct Seen {
    name: &'static str,
    number: u8,
    rating: f32,
}

/// Every driver of the season data with the years they are in it (a team's driver, else a
/// mid-season change's).
fn careers() -> &'static BTreeMap<&'static str, BTreeMap<u16, Seen>> {
    static CAREERS: OnceLock<BTreeMap<&'static str, BTreeMap<u16, Seen>>> = OnceLock::new();
    CAREERS.get_or_init(|| {
        let mut out: BTreeMap<&'static str, BTreeMap<u16, Seen>> = BTreeMap::new();
        for (&year, y) in &catalog().years {
            for d in y.teams.iter().flat_map(|t| &t.drivers) {
                out.entry(d.key.as_str()).or_default().insert(
                    year,
                    Seen {
                        name: &d.name,
                        number: d.number,
                        rating: d.rating,
                    },
                );
            }
            for c in &y.changes {
                out.entry(c.driver.as_str())
                    .or_default()
                    .entry(year)
                    .or_insert(Seen {
                        name: &c.name,
                        number: c.number,
                        rating: c.rating,
                    });
            }
        }
        out
    })
}

/// The key of a made-up driver: `gen_luca_ferraro`.
fn generated_key(name: &str) -> String {
    format!("gen_{}", words(name).join("_"))
}

fn is_generated(key: &str) -> bool {
    key.starts_with("gen_")
}

/// A driver's nationality (ISO 3166-1 alpha-3): of the season data's drivers and the made-up ones.
pub fn nationality_of(key: &str) -> Option<&'static str> {
    NATIONALITIES
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, n)| *n)
        .or_else(|| {
            GENERATED
                .iter()
                .find(|(name, _)| generated_key(name) == key)
                .map(|(_, n)| *n)
        })
}

/// The name of the box driver of a seat (`slot`, `key`) in `year`'s season data.
pub fn seat_driver_name(year: u16, slot: u8, key: &str) -> Option<&'static str> {
    catalog_driver(year, slot, key).map(|d| d.name.as_str())
}

fn hash(s: &str) -> u64 {
    s.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// The rating a driver of last season's grid has next season: the season data's for that year,
/// a made-up driver's grown, else last season's.
fn next_rating(e: &Entry, next: u16) -> f32 {
    if let Some(s) = careers().get(e.id.as_str()).and_then(|c| c.get(&next)) {
        return s.rating;
    }
    if is_generated(&e.id) {
        return (e.rating + GENERATED_GROWTH).min(GENERATED_PEAK.max(e.rating));
    }
    e.rating
}

/// The dynamic line-up of season `next`: the season with the players in `seats` (as
/// `build_season` makes it, without the real mid-season changes), its AI seats filled by the
/// market from `league`'s current season, the renamed seats in [`Season::renames`], and the
/// moves.
pub(super) fn plan(league: &League, next: u16, seats: &[(Player, Seat)]) -> Result<Plan, LeagueError> {
    let mut season = build_season(next, seats, false)?;
    let cat = catalog_year(next).expect("build_season found the year");
    let last = league.season();
    let final_round = last.rounds.last().map_or(1, |r| r.round);
    let seed = mix(&[league.seed, u64::from(next), SALT]);
    let roll = |what: u64, key: &str| (mix(&[seed, what, hash(key)]) >> 11) as f64 / (1u64 << 53) as f64;

    // The box's seats the AI drives, and how many each team has.
    let player_seat = |slot: u8, key: &str| seats.iter().any(|(_, s)| s.slot == slot && s.replaces == key);
    let ai_seats: Vec<(u8, &str)> = cat
        .teams
        .iter()
        .flat_map(|t| t.drivers.iter().map(move |d| (t.slot, d.key.as_str())))
        .filter(|(slot, key)| !player_seat(*slot, key))
        .collect();
    let capacity: BTreeMap<u8, usize> = cat
        .teams
        .iter()
        .map(|t| (t.slot, ai_seats.iter().filter(|(s, _)| *s == t.slot).count()))
        .collect();
    // The teams, the fastest first.
    let mut ranked: Vec<(u8, f32)> = cat.teams.iter().map(|t| (t.slot, t.pace)).collect();
    ranked.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
    let rank_of = |slot: u8| {
        ranked
            .iter()
            .position(|(s, _)| *s == slot)
            .map_or(ranked.len(), |i| i + 1)
    };

    // Last season's AI line-up at its end.
    let mut pool: Vec<Driver> = last
        .lineup(final_round)
        .into_iter()
        .filter(|(e, _)| e.player.is_none())
        .map(|(e, slot)| Driver {
            key: e.id.clone(),
            name: e.name.clone(),
            number: e.number,
            rating: next_rating(e, next),
            from: Some(slot),
            slot: capacity.contains_key(&slot).then_some(slot),
            moved: false,
            new: None,
        })
        .collect();
    pool.sort_by(|a, b| a.key.cmp(&b.key));
    pool.dedup_by(|a, b| a.key == b.key);
    let mut moves = Vec::new();
    let gone = |d: &Driver, kind: MoveKind| Move {
        kind,
        key: d.key.clone(),
        name: d.name.clone(),
        from: d.from,
        to: None,
    };

    // Retirements: past the last year of the season data.
    pool.retain(|d| {
        let retires = careers()
            .get(d.key.as_str())
            .and_then(|c| c.keys().next_back())
            .is_some_and(|&y| y < next);
        if retires {
            moves.push(gone(d, MoveKind::Retired));
        }
        !retires
    });

    // A team keeps as many drivers as it has AI seats: the best-rated.
    let by_rating = |pool: &[Driver], idx: &mut Vec<usize>| {
        idx.sort_by(|&a, &b| {
            pool[b]
                .rating
                .total_cmp(&pool[a].rating)
                .then(pool[a].key.cmp(&pool[b].key))
        });
    };
    let at = |pool: &[Driver], slot: u8| -> Vec<usize> {
        (0..pool.len()).filter(|&i| pool[i].slot == Some(slot)).collect()
    };
    for (&slot, &cap) in &capacity {
        let mut stay = at(&pool, slot);
        by_rating(&pool, &mut stay);
        for &i in stay.iter().skip(cap) {
            pool[i].slot = None;
        }
    }

    // Each team's lowest-rated driver may lose the seat.
    let median = {
        let mut r: Vec<f32> = pool.iter().map(|d| d.rating).collect();
        r.sort_by(f32::total_cmp);
        r.get(r.len() / 2).copied().unwrap_or(0.0)
    };
    let mut dropped = Vec::new();
    for &slot in capacity.keys() {
        let mut stay = at(&pool, slot);
        by_rating(&pool, &mut stay);
        let Some(&i) = stay.last() else { continue };
        // A team's only AI driver keeps the seat unless rated below the middle.
        let chance = match (pool[i].rating < median, stay.len()) {
            (true, _) => DROP_CHANCE_WEAK,
            (false, 1) => continue,
            (false, _) => DROP_CHANCE,
        };
        if roll(1, &pool[i].key) < chance {
            dropped.push(i);
        }
    }
    dropped.sort_unstable();
    for i in dropped.into_iter().rev() {
        let d = pool.remove(i);
        moves.push(gone(&d, MoveKind::Dropped));
    }

    // Drivers rated well above their car move up.
    let mut order: Vec<usize> = (0..pool.len()).collect();
    by_rating(&pool, &mut order);
    for (place, &i) in order.iter().enumerate() {
        let Some(own) = pool[i].slot else { continue };
        if pool[i].moved {
            continue;
        }
        let deserved = place / 2 + 1;
        let own_rank = rank_of(own);
        if own_rank < deserved + MOVE_GAP || roll(2, &pool[i].key) >= MOVE_CHANCE {
            continue;
        }
        for &(target, _) in ranked.iter().filter(|(s, _)| {
            let r = rank_of(*s);
            r + MOVE_STEP <= own_rank && r >= deserved
        }) {
            let mut stay = at(&pool, target);
            if stay.len() < capacity[&target] {
                pool[i].slot = Some(target);
                pool[i].moved = true;
                break;
            }
            by_rating(&pool, &mut stay);
            let Some(&low) = stay.iter().rev().find(|&&j| !pool[j].moved) else {
                continue;
            };
            if pool[low].rating + SWAP_GAP <= pool[i].rating {
                pool[low].slot = Some(own);
                pool[low].moved = true;
                pool[i].slot = Some(target);
                pool[i].moved = true;
                break;
            }
        }
    }

    // Free seats, the fastest team's first: to the drivers without a seat, then to drivers of
    // the season data who race that year and were never on the career's grid (rookies and
    // drivers back), then to the rookies of the next years, then to made-up drivers. Each
    // group the best-rated first.
    let ever: HashSet<&str> = league
        .seasons
        .iter()
        .flat_map(|s| &s.entries)
        .filter(|e| e.player.is_none())
        .map(|e| e.id.as_str())
        .collect();
    let mut candidates: Vec<((u16, f32), Driver)> = (0..pool.len())
        .filter(|&i| pool[i].slot.is_none())
        .map(|i| ((0, pool[i].rating), pool[i].clone()))
        .collect();
    for (&key, years) in careers() {
        let (Some((&first, _)), Some((&last_year, _))) = (years.first_key_value(), years.last_key_value())
        else {
            continue;
        };
        if first > next + ROOKIE_YEARS
            || last_year < next
            || ever.contains(key)
            || pool.iter().any(|d| d.key == key)
        {
            continue;
        }
        let (_, seen) = years.range(..=next.max(first)).next_back().expect("first <= max");
        let rookie = first >= next;
        let rating = seen.rating - if rookie { ROOKIE_MALUS } else { 0.0 };
        // This year's drivers first, then the later years' rookies year by year.
        let group = if years.contains_key(&next) {
            1
        } else {
            first.max(next) - next + 1
        };
        candidates.push((
            (group, rating),
            Driver {
                key: key.to_owned(),
                name: seen.name.to_owned(),
                number: seen.number,
                rating,
                from: None,
                slot: None,
                moved: false,
                new: Some(if rookie {
                    MoveKind::Rookie
                } else {
                    MoveKind::Return
                }),
            },
        ));
    }
    candidates.sort_by(|a, b| {
        a.0.0
            .cmp(&b.0.0)
            .then(b.0.1.total_cmp(&a.0.1))
            .then(a.1.key.cmp(&b.1.key))
    });
    let mut candidates = candidates.into_iter().map(|(_, d)| d).peekable();
    let start = (seed % GENERATED.len() as u64) as usize;
    let made_up: Vec<&str> = (0..GENERATED.len())
        .map(|i| GENERATED[(start + i) % GENERATED.len()].0)
        .filter(|name| {
            let key = generated_key(name);
            !ever.contains(key.as_str()) && !pool.iter().any(|d| d.key == key)
        })
        .collect();
    let mut made_up = made_up.into_iter();
    for &(slot, _) in &ranked {
        let open = capacity[&slot].saturating_sub(at(&pool, slot).len());
        for _ in 0..open {
            // A team hires a proven driver of a slower team over a clearly lower-rated newcomer,
            // with a chance; that driver's seat is free for the slower team then.
            let newcomer = candidates.peek().map_or(GENERATED_RATING, |d| d.rating);
            let mut promote: Vec<usize> = (0..pool.len())
                .filter(|&j| {
                    !pool[j].moved
                        && pool[j]
                            .slot
                            .is_some_and(|s| rank_of(s) >= rank_of(slot) + MOVE_STEP)
                        && pool[j].rating >= newcomer + SWAP_GAP
                })
                .collect();
            by_rating(&pool, &mut promote);
            if let Some(&j) = promote.first()
                && roll(4, &pool[j].key) < PROMOTE_CHANCE
            {
                pool[j].slot = Some(slot);
                pool[j].moved = true;
                continue;
            }
            let d = match candidates.next() {
                Some(d) => d,
                None => match made_up.next() {
                    Some(name) => {
                        let key = generated_key(name);
                        Driver {
                            rating: GENERATED_RATING - 2.0 + (mix(&[seed, 3, hash(&key)]) % 5) as f32,
                            key,
                            name: name.to_owned(),
                            number: 0,
                            from: None,
                            slot: None,
                            moved: false,
                            new: Some(MoveKind::Rookie),
                        }
                    }
                    // More free seats than made-up names: never with the season data's grids.
                    None => break,
                },
            };
            match pool.iter_mut().find(|p| p.key == d.key) {
                Some(p) => p.slot = Some(slot),
                None => pool.push(Driver {
                    slot: Some(slot),
                    ..d
                }),
            }
        }
    }

    // Numbers: everyone keeps theirs, the best-rated first; a clash or a newcomer without one
    // gets a free number (a newcomer none of the players has either).
    let player_numbers: BTreeSet<u8> = seats.iter().map(|(p, _)| p.number).collect();
    let mut order: Vec<usize> = (0..pool.len()).filter(|&i| pool[i].slot.is_some()).collect();
    order.sort_by(|&a, &b| {
        (pool[a].new.is_some(), pool[a].number == 0)
            .cmp(&(pool[b].new.is_some(), pool[b].number == 0))
            .then(pool[b].rating.total_cmp(&pool[a].rating))
            .then(pool[a].key.cmp(&pool[b].key))
    });
    let mut used: BTreeSet<u8> = BTreeSet::new();
    for i in order {
        let d = &mut pool[i];
        let clash = d.number == 0
            || used.contains(&d.number)
            || (d.new.is_some() && player_numbers.contains(&d.number));
        if clash {
            let from = (hash(&d.key) % 90) as u8 + 10;
            d.number = (0..98u8)
                .map(|k| (from - 2 + k) % 98 + 2)
                .find(|n| !used.contains(n) && !player_numbers.contains(n))
                .unwrap_or(99);
        }
        used.insert(d.number);
    }

    // Each team's drivers in its AI seats: a driver in their own box seat first, then the
    // best-rated in the box's order.
    let mut seat_of: BTreeMap<(u8, &str), usize> = BTreeMap::new();
    for &slot in capacity.keys() {
        let mut drivers = at(&pool, slot);
        by_rating(&pool, &mut drivers);
        let mut open: Vec<&str> = ai_seats
            .iter()
            .filter(|(s, _)| *s == slot)
            .map(|(_, k)| *k)
            .collect();
        drivers.retain(|&i| match open.iter().position(|k| *k == pool[i].key) {
            Some(pos) => {
                seat_of.insert((slot, open.remove(pos)), i);
                false
            }
            None => true,
        });
        for (key, i) in open.into_iter().zip(drivers) {
            seat_of.insert((slot, key), i);
        }
    }

    // The entries, and the renamed seats with codes unique on the grid.
    let mut codes: BTreeSet<String> = ai_seats
        .iter()
        .filter(|(slot, key)| seat_of.get(&(*slot, *key)).is_some_and(|&i| pool[i].key == *key))
        .filter_map(|(slot, key)| catalog_driver(next, *slot, key))
        .map(|d| {
            let (first, last) = split_name(&d.name);
            base_code(&first, &last)
        })
        .collect();
    let mut renames = Vec::new();
    for e in season.entries.iter_mut().filter(|e| e.player.is_none()) {
        let Some(&i) = seat_of.get(&(e.slot, e.id.as_str())) else {
            continue;
        };
        let d = &pool[i];
        if d.key != e.id {
            let (first, last) = split_name(&d.name);
            let code = unique_code(&first, &last, &codes);
            codes.insert(code.clone());
            renames.push(SeatRename {
                slot: e.slot,
                seat_driver: e.id.clone(),
                key: d.key.clone(),
                first,
                last,
                code,
                number: d.number,
                nationality: nationality_of(&d.key)
                    .and_then(game_nationality)
                    .map(str::to_owned),
            });
        }
        *e = Entry {
            id: d.key.clone(),
            name: d.name.clone(),
            number: d.number,
            slot: e.slot,
            rating: d.rating,
            player: None,
        };
    }
    renames.sort_by(|a, b| (a.slot, &a.seat_driver).cmp(&(b.slot, &b.seat_driver)));
    season.renames = renames;

    // What happened, for the news.
    let seated: HashSet<usize> = seat_of.values().copied().collect();
    for (i, d) in pool.iter().enumerate() {
        let kind = match (seated.contains(&i), d.new, d.from) {
            (true, Some(kind), _) => kind,
            (true, None, Some(from)) if d.slot != Some(from) => MoveKind::Transfer,
            (false, None, _) => MoveKind::NoSeat,
            _ => continue,
        };
        moves.push(Move {
            kind,
            key: d.key.clone(),
            name: d.name.clone(),
            from: d.from,
            to: d.slot.filter(|_| seated.contains(&i)),
        });
    }
    moves.sort_by_key(|m| (order_of(m.kind), m.to.or(m.from), m.key.clone()));
    Ok(Plan { season, moves })
}

fn order_of(kind: MoveKind) -> u8 {
    match kind {
        MoveKind::Retired => 0,
        MoveKind::Dropped => 1,
        MoveKind::Transfer => 2,
        MoveKind::Rookie => 3,
        MoveKind::Return => 4,
        MoveKind::NoSeat => 5,
    }
}

/// The news of the market's `moves`, for the season that just began (templated, with the
/// English text for older launchers).
pub(super) fn announce(league: &mut League, moves: &[Move]) {
    let season = league.season();
    let year = season.year;
    let previous = league.seasons.len().checked_sub(2).map(|i| &league.seasons[i]);
    let team = |slot: Option<u8>, s: Option<&Season>| {
        slot.and_then(|slot| s?.team(slot))
            .map_or_else(String::new, |t| t.name.clone())
    };
    let arg = |k: &str, v: String| (k.to_owned(), v);
    let mut items = Vec::new();
    for m in moves {
        let driver = arg("driver", m.name.clone());
        let (template, args) = match m.kind {
            MoveKind::Retired => (T_RETIRED, vec![driver, arg("last", (year - 1).to_string())]),
            MoveKind::Dropped => (
                T_DROPPED,
                vec![
                    driver,
                    arg("team", team(m.from, previous)),
                    arg("year", year.to_string()),
                ],
            ),
            MoveKind::NoSeat => (T_NO_SEAT, vec![driver, arg("year", year.to_string())]),
            MoveKind::Transfer => (
                T_TRANSFER,
                vec![
                    driver,
                    arg("from", team(m.from, previous)),
                    arg("team", team(m.to, Some(season))),
                    arg("year", year.to_string()),
                ],
            ),
            MoveKind::Rookie | MoveKind::Return => (
                if m.kind == MoveKind::Rookie {
                    T_ROOKIE
                } else {
                    T_RETURN
                },
                vec![
                    driver,
                    arg("team", team(m.to, Some(season))),
                    arg("year", year.to_string()),
                ],
            ),
        };
        let mut news = News::plain(year, 0, render(template, &args));
        news.template = Some(template.to_owned());
        news.args = args;
        items.push(news);
    }
    league.news.extend(items);
    if league.news.len() > NEWS_MAX {
        let extra = league.news.len() - NEWS_MAX;
        league.news.drain(..extra);
    }
}

/// The seat (team, box driver's key) of `entry` in `season`: the renamed seat it drives, else
/// its own.
pub fn seat_of<'a>(season: &'a Season, entry: &'a Entry) -> (u8, &'a str) {
    season
        .renames
        .iter()
        .find(|r| r.key == entry.id)
        .map_or((entry.slot, entry.id.as_str()), |r| {
            (r.slot, r.seat_driver.as_str())
        })
}

/// Whether `car` shows the box driver of the renamed seat `r`.
fn shows_box(year: u16, r: &SeatRename, car: &ReportCar) -> bool {
    catalog_driver(year, r.slot, &r.seat_driver)
        .is_some_and(|d| d.number == car.number && same_driver(&car.name, &d.name))
}

/// Whether `car` shows the driver `r` renames its seat to.
fn shows_rename(r: &SeatRename, car: &ReportCar) -> bool {
    car.number == r.number && same_driver(&car.name, &format!("{} {}", r.first, r.last))
}

/// Whether a report's AI `cars` show the box's real drivers in the seats the market renamed:
/// more cars carry a renamed seat's box name and number (and no renamed driver's) than a renamed
/// driver's (and no box driver's). The game then didn't have the market's overlay.
pub(super) fn box_shown(season: &Season, cars: &[&ReportCar]) -> bool {
    if season.renames.is_empty() {
        return false;
    }
    let (mut boxed, mut renamed) = (0, 0);
    for c in cars.iter().filter(|c| c.ai) {
        let as_box = season.renames.iter().any(|r| shows_box(season.year, r, c));
        let as_rename = season.renames.iter().any(|r| shows_rename(r, c));
        match (as_box, as_rename) {
            (true, false) => boxed += 1,
            (false, true) => renamed += 1,
            _ => {}
        }
    }
    boxed > renamed
}

/// The league's driver of the renamed seat whose box driver `car` shows.
pub(super) fn box_holder<'a>(season: &'a Season, car: &ReportCar) -> Option<&'a Entry> {
    season
        .renames
        .iter()
        .find(|r| shows_box(season.year, r, car))
        .and_then(|r| season.entry(&r.key))
}
