//! The players' names, codes, numbers and nationalities in the game (W2-9): `driver.identity` and
//! `content.package` registry layers, with a DB/lang fallback.
//!
//! Each active player with a seat renames the row of the real driver whose seat they took (their
//! `replaces`): the grid, timing tower, results and menus then show the player. Names, codes and
//! numbers come from the league document alone ([`shown`]), so the briefing
//! ([`identity_of`](super::identity_of)) and the game agree and two PCs write the same files; the
//! installed Classic Seasons files only say which records the layers go on.
//!
//! - **Names.** The last name goes upper case, as Classic Seasons writes its drivers ("Felipe
//!   NASR"); a one-word name is a last name alone.
//! - **Code.** Three letters from the last name, unique on the year's grid.
//! - **Number.** The player's own, unless another driver of the year's box (the real line-up the
//!   game's season box holds) races with it; then the seat's number (`number_clash`).
//! - **Nationality.** The ISO 3166-1 alpha-3 code as the game's `entrants.nationality` label;
//!   a country the game has no flag for is `default` (the game's "other").
//!
//! Mid-season stand-ins are left alone: the game's box keeps the seat's driver, and
//! [`lineup_notes`](crate::league::lineup_notes) explains them.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};

use super::{GameIdentity, IDENTITY_DB, IDENTITY_LANG, IDENTITY_REGISTRY, Overlay, SyncContext, json_file};
use crate::league::{League, Player, Season, catalog_driver, catalog_year, words};

/// The addon files for the identities of `overlay`: [`IDENTITY_REGISTRY`] always, and the
/// database and lang fallback ([`IDENTITY_DB`], [`IDENTITY_LANG`]) when the core doesn't apply the
/// registries' rows itself.
pub(super) fn files(overlay: &Overlay, ctx: &SyncContext) -> Vec<(String, Vec<u8>)> {
    if overlay.identities.is_empty() {
        return Vec::new();
    }
    let mut out = vec![json_file(IDENTITY_REGISTRY, &registry(overlay, ctx))];
    if !ctx.loader_applies_rows {
        out.push(json_file(IDENTITY_DB, &database(overlay)));
        out.push(json_file(IDENTITY_LANG, &lang(overlay)));
    }
    out
}

/// The identities of the league's players for the next round: [`shown`], with each player's team
/// named by its key in the year's Classic Seasons content when `ctx` has it.
pub(super) fn build(league: &League, ctx: &SyncContext) -> Vec<GameIdentity> {
    let mut out = shown(league.season(), &league.players);
    for i in &mut out {
        if let Some((team, _, _)) = content_team(ctx, &i.replaces) {
            i.team_key = team;
        }
    }
    out
}

/// What the game shows for each active player with a seat in `season`, in the players' order:
/// from the league alone (no Classic Seasons files), so every PC and the briefing get the same.
/// `team_key` is the team's name as a key here; [`build`] names the Classic Seasons team.
pub(crate) fn shown(season: &Season, players: &[Player]) -> Vec<GameIdentity> {
    let year = season.year;
    let Some(cat) = catalog_year(year) else {
        return Vec::new();
    };
    let seated: Vec<(&Player, u8, &str)> = players
        .iter()
        .filter(|p| !p.left && season.player_entry(&p.id).is_some())
        .filter_map(|p| p.seat.as_ref().map(|s| (p, s.slot, s.replaces.as_str())))
        .filter(|(_, slot, key)| catalog_driver(year, *slot, key).is_some())
        .collect();
    let replaced: BTreeSet<&str> = seated.iter().map(|(_, _, key)| *key).collect();
    let renamed: BTreeSet<&str> = season.renames.iter().map(|r| r.seat_driver.as_str()).collect();
    // The codes the rest of the grid shows: the box's drivers whose rows stay theirs, and the AI
    // seats a dynamic line-up renamed.
    let mut codes: BTreeSet<String> = cat
        .teams
        .iter()
        .flat_map(|t| &t.drivers)
        .filter(|d| !replaced.contains(d.key.as_str()) && !renamed.contains(d.key.as_str()))
        .map(|d| {
            let (first, last) = split_name(&d.name);
            base_code(&first, &last)
        })
        .chain(season.renames.iter().map(|r| r.code.clone()))
        .collect();
    let mut out = Vec::new();
    for (p, slot, key) in seated {
        let seat = catalog_driver(year, slot, key).expect("filtered");
        // Every number of the box but the seat's own is another driver's, and so are the numbers
        // of renamed AI seats; a player's own number is free when it is none of them.
        let taken = cat
            .teams
            .iter()
            .flat_map(|t| t.drivers.iter().map(move |d| (t.slot, d)))
            .filter(|(s, d)| (*s, d.key.as_str()) != (slot, key))
            .map(|(_, d)| d.number)
            .chain(season.renames.iter().map(|r| r.number))
            .any(|n| n == p.number);
        // The seat's number, unless a renamed AI seat shows it (a dynamic line-up's driver who
        // keeps it): then the lowest number nobody in the box, the market or the career has.
        let seat_number = match season.renames.iter().any(|r| r.number == seat.number) {
            false => seat.number,
            true => (2..=99)
                .find(|n| {
                    !cat.teams.iter().flat_map(|t| &t.drivers).any(|d| d.number == *n)
                        && !season.renames.iter().any(|r| r.number == *n)
                        && !players.iter().any(|o| o.number == *n)
                })
                .unwrap_or(seat.number),
        };
        let (number, number_clash) = match taken {
            true => (seat_number, true),
            false => (p.number, false),
        };
        let (first, last) = split_name(&p.name);
        let code = unique_code(&first, &last, &codes);
        codes.insert(code.clone());
        let full = match first.is_empty() {
            true => last.clone(),
            false => format!("{first} {last}"),
        };
        let team = season.team(slot).map(|t| t.name.clone()).unwrap_or_default();
        out.push(GameIdentity {
            player: p.id.clone(),
            first,
            last,
            full,
            code,
            number,
            nationality: p
                .nationality
                .as_deref()
                .and_then(game_nationality)
                .map(str::to_owned),
            replaces: key.to_owned(),
            year,
            slot,
            team_key: key_of(&team),
            number_clash,
        });
    }
    out
}

/// A team name as a key: "Red Bull Racing" → `red_bull_racing`.
fn key_of(name: &str) -> String {
    words(name).join("_")
}

/// Most a name part keeps, as Driver Manager limits its names.
const NAME_MAX: usize = 24;

/// Lower-case words that belong to the last name that follows them ("Nyck de Vries").
const PARTICLES: [&str; 16] = [
    "da", "das", "de", "del", "della", "den", "der", "di", "do", "dos", "du", "la", "le", "van", "von", "zu",
];

/// A player's name as the game shows it: the first name as written and the last name upper case
/// (Classic Seasons writes "Felipe NASR"). The last word is the last name, with the lower-case
/// particles before it; a one-word name is a last name alone.
pub(crate) fn split_name(name: &str) -> (String, String) {
    let parts: Vec<&str> = name.split_whitespace().collect();
    let Some((last_word, rest)) = parts.split_last() else {
        return (String::new(), String::new());
    };
    let mut at = rest.len();
    while at > 1 && PARTICLES.contains(&rest[at - 1]) {
        at -= 1;
    }
    let first = rest[..at].join(" ");
    let last = rest[at..]
        .iter()
        .chain(std::iter::once(last_word))
        .copied()
        .collect::<Vec<_>>()
        .join(" ");
    let cut = |s: String| -> String { s.chars().take(NAME_MAX).collect::<String>().trim().to_owned() };
    (cut(first), cut(last.to_uppercase()))
}

/// The letters A to Z of a name part, accents folded.
fn letters(part: &str) -> Vec<char> {
    words(part)
        .concat()
        .chars()
        .filter(char::is_ascii_alphabetic)
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

/// The first three letters of the last name, filled from the first name and `X`.
pub(crate) fn base_code(first: &str, last: &str) -> String {
    letters(last)
        .into_iter()
        .chain(letters(first))
        .chain("XXX".chars())
        .take(3)
        .collect()
}

/// A three-letter code for the name that `taken` doesn't hold: the first three letters of the
/// last name, else its first letter with two later ones in order, else its first two with the
/// first name's initial, else its first two with a digit.
pub(crate) fn unique_code(first: &str, last: &str, taken: &BTreeSet<String>) -> String {
    let base = base_code(first, last);
    let l = letters(last);
    let f = letters(first);
    let mut candidates = vec![base.clone()];
    for j in 1..l.len() {
        for k in j + 1..l.len() {
            candidates.push([l[0], l[j], l[k]].iter().collect());
        }
    }
    let two: String = base.chars().take(2).collect();
    if let Some(i) = f.first() {
        candidates.push(format!("{two}{i}"));
    }
    candidates.extend((1..=9).map(|d| format!("{two}{d}")));
    let one: String = base.chars().take(1).collect();
    candidates.extend((10..=99).map(|d| format!("{one}{d}")));
    candidates
        .into_iter()
        .find(|c| !taken.contains(c))
        .unwrap_or(base)
}

/// The Classic Seasons team (its key in the year's content package), the package's id and the
/// driver record of the seat `replaces` drives in it.
fn content_team(ctx: &SyncContext, replaces: &str) -> Option<(String, String, String)> {
    let (package, value) = content_package(ctx)?;
    for team in value.get("teams")?.as_array()? {
        for seat in team
            .get("drivers")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(driver) = seat.get("driver").and_then(Value::as_str) else {
                continue;
            };
            if driver.rsplit_once('/').map_or(driver, |(_, k)| k) == replaces {
                let key = team.get("key")?.as_str()?.to_owned();
                return Some((key, package, driver.to_owned()));
            }
        }
    }
    None
}

/// The year's content package in `ctx.content` (the one with a `season`): its full id and value.
fn content_package<'a>(ctx: &SyncContext<'a>) -> Option<(String, &'a Value)> {
    let content: &'a Value = ctx.content;
    content
        .get("content.package")?
        .as_object()?
        .iter()
        .find_map(|(key, value)| {
            let year = u16::try_from(value.get("season")?.as_u64()?).ok()?;
            Some((format!("{}/{key}", super::season_addon(year)?), value))
        })
}

/// The Classic Seasons core, which adds the drivers the game doesn't have (`ctx.cs_drivers`).
const CLASSIC_SEASONS_CORE: &str = "swofty.classic-seasons";

/// The `driver.identity` record of the seat driver `replaces`: the record the year's content seats
/// in that seat, else the Classic Seasons core's record when it adds that driver, else the game's.
pub(super) fn identity_target(ctx: &SyncContext, replaces: &str) -> String {
    if let Some((_, _, driver)) = content_team(ctx, replaces) {
        return driver;
    }
    let cs = ctx
        .cs_drivers
        .get("driver.identity")
        .and_then(Value::as_object)
        .is_some_and(|m| m.contains_key(replaces));
    match cs {
        true => format!("{CLASSIC_SEASONS_CORE}/{replaces}"),
        false => format!("game/{replaces}"),
    }
}

/// `registry/identity.json`: a `driver.identity` layer per player with literal names, code,
/// number and nationality, and a `content.package` layer restating the seats of every team a
/// player drives for, with the players' numbers ([`package_layer`]). With renamed AI seats
/// (a dynamic line-up) the market's file carries that layer instead: one layer per record, and
/// it restates the numbers of both.
fn registry(overlay: &Overlay, ctx: &SyncContext) -> Value {
    let mut identities = Map::new();
    for i in &overlay.identities {
        identities.insert(
            identity_target(ctx, &i.replaces),
            identity_layer(
                &i.first,
                &i.last,
                &i.full,
                &i.code,
                i.number,
                i.nationality.as_deref(),
            ),
        );
    }
    let mut out = json!({ "driver.identity": identities });
    if overlay.renames.is_empty()
        && let Some(packages) = package_layer(overlay, ctx)
    {
        out["content.package"] = packages;
    }
    out
}

/// A `driver.identity` layer with literal names, the code, the number and the nationality label.
pub(super) fn identity_layer(
    first: &str,
    last: &str,
    full: &str,
    code: &str,
    number: u8,
    nationality: Option<&str>,
) -> Value {
    let mut layer = json!({
        "names": {
            "first": { "literal": first },
            "last": { "literal": last },
            "full": { "literal": full },
            "code": code,
        },
        "number": number,
    });
    if let Some(n) = nationality {
        layer["nationality"] = json!(n);
    }
    layer
}

/// The `content.package` layers (`{package id: {"teams": [...]}}`) restating the seats of every
/// team with a player's identity or a renamed AI seat in `overlay`, each seat with the number the
/// game shows for it; `None` when `ctx` has no such team.
pub(super) fn package_layer(overlay: &Overlay, ctx: &SyncContext) -> Option<Value> {
    let numbers: Vec<(&str, u8)> = overlay
        .identities
        .iter()
        .map(|i| (i.replaces.as_str(), i.number))
        .chain(overlay.renames.iter().map(|r| (r.seat_driver.as_str(), r.number)))
        .collect();
    let (package, value) = content_package(ctx)?;
    let mut teams: BTreeMap<String, Value> = BTreeMap::new();
    for (seat, _) in &numbers {
        let Some((key, _, _)) = content_team(ctx, seat) else {
            continue;
        };
        if teams.contains_key(&key) {
            continue;
        }
        let team = value["teams"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|t| t.get("key").and_then(Value::as_str) == Some(key.as_str()))
            .expect("found the team");
        let drivers: Vec<Value> = team
            .get("drivers")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(|seat| {
                let mut seat = seat.clone();
                let driver = seat.get("driver").and_then(Value::as_str).unwrap_or_default();
                let key = driver.rsplit_once('/').map_or(driver, |(_, k)| k);
                if let Some((_, number)) = numbers.iter().find(|(k, _)| *k == key) {
                    seat["number"] = json!(number);
                }
                seat
            })
            .collect();
        teams.insert(key.clone(), json!({ "key": key, "drivers": drivers }));
    }
    if teams.is_empty() {
        return None;
    }
    let mut packages = Map::new();
    packages.insert(
        package,
        json!({ "teams": teams.into_values().collect::<Vec<_>>() }),
    );
    Some(Value::Object(packages))
}

/// One renamed row of the game's database: the seat driver's key and what the row shows.
pub(super) struct Row<'a> {
    pub seat: &'a str,
    pub year: u16,
    pub first: &'a str,
    pub last: &'a str,
    pub full: String,
    pub code: &'a str,
    pub number: u8,
    pub nationality: Option<&'a str>,
}

impl<'a> Row<'a> {
    fn of(i: &'a GameIdentity) -> Self {
        Row {
            seat: &i.replaces,
            year: i.year,
            first: &i.first,
            last: &i.last,
            full: i.full.clone(),
            code: &i.code,
            number: i.number,
            nationality: i.nationality.as_deref(),
        }
    }
}

/// The lang key of a renamed row's name part (`first`, `last` or `full`).
fn lang_key(seat: &str, part: &str) -> String {
    format!("lng_f1mp_{seat}_{part}")
}

/// `db/identity.json`: the player's names (as lang keys), code, nationality and number on the
/// seat driver's row, and the number on their entrant in the Classic Seasons year's list.
fn database(overlay: &Overlay) -> Value {
    rows_database(overlay.identities.iter().map(Row::of))
}

/// The database ops renaming `rows`: [`database`]'s, and the market's fallback.
pub(super) fn rows_database<'a>(rows: impl Iterator<Item = Row<'a>>) -> Value {
    let mut ops = Vec::new();
    let set = |select: &str, field: &str, value: String| json!({ "op": "set", "select": select, "field": field, "value": value });
    for i in rows {
        let row = format!("entrants.drivers.driver[_metadata_={}]", i.seat);
        for part in ["first", "last", "full"] {
            ops.push(set(&row, &format!("{part}_name"), lang_key(i.seat, part)));
        }
        ops.push(set(&row, "initials", i.code.to_owned()));
        if let Some(n) = i.nationality {
            ops.push(set(&row, "nationality", n.to_owned()));
        }
        ops.push(set(&row, "car_number", i.number.to_string()));
        if super::season_addon(i.year).is_some() {
            let entrant = format!(
                "entrants.entrant_lists.entrants.entrant[_metadata_=cs{}-{}]",
                i.year, i.seat
            );
            ops.push(set(&entrant, "car_number", i.number.to_string()));
        }
    }
    json!({ "schema_version": 1, "ops": ops })
}

/// `lang/identity.json`: the names behind [`database`]'s lang keys, in every language.
fn lang(overlay: &Overlay) -> Value {
    rows_lang(overlay.identities.iter().map(Row::of))
}

/// The lang ops behind [`rows_database`]'s lang keys.
pub(super) fn rows_lang<'a>(rows: impl Iterator<Item = Row<'a>>) -> Value {
    let mut ops = Vec::new();
    for i in rows {
        for (part, value) in [("first", i.first), ("last", i.last), ("full", i.full.as_str())] {
            ops.push(
                json!({ "op": "lang.set", "language": "*", "key": lang_key(i.seat, part), "value": value }),
            );
        }
    }
    json!({ "schema_version": 1, "ops": ops })
}

/// Test fixture: a game database with the rows [`database`]'s ops set for the box drivers `keys`
/// of `year`, as the game (a stock driver) or Classic Seasons (an added driver, the year's
/// entrants) has them.
#[cfg(test)]
pub(crate) fn box_rows(year: u16, keys: &[&str]) -> swofty_formats::gvdb::Gvdb {
    use swofty_formats::gvdb::{Gvdb, Record};
    let (int, text, id) = (0, 2, 4);
    let mut records = vec![
        Record::new(
            "entrants.drivers._driver_instancecount_",
            int,
            keys.len().to_string(),
        ),
        Record::new(
            "entrants.entrant_lists.entrants._entrant_instancecount_",
            int,
            keys.len().to_string(),
        ),
    ];
    for (i, key) in keys.iter().enumerate() {
        let row = format!("entrants.drivers.driver_{i}");
        let field = |f: &str| format!("{row}.{f}");
        records.push(Record::new(&field("_metadata_"), text, *key));
        for part in ["first", "last", "full"] {
            records.push(Record::new(
                &field(&format!("{part}_name")),
                text,
                format!("lng_{key}_{part}"),
            ));
        }
        records.push(Record::new(&field("initials"), text, "XXX"));
        records.push(Record::new(&field("nationality"), id, "finland"));
        records.push(Record::new(&field("car_number"), int, "1"));
        let entrant = format!("entrants.entrant_lists.entrants.entrant_{i}");
        records.push(Record::new(
            &format!("{entrant}._metadata_"),
            text,
            format!("cs{year}-{key}"),
        ));
        records.push(Record::new(&format!("{entrant}.car_number"), int, "1"));
    }
    Gvdb { records }
}

/// The game's nationality label for an ISO 3166-1 alpha-3 code (any case): its flag's label, or
/// `default` (the game's "other") for a country the game has no flag for. `None` for a code
/// that is no country.
pub fn game_nationality(iso: &str) -> Option<&'static str> {
    let iso = iso.trim().to_ascii_uppercase();
    if let Some((_, label)) = FLAGS.iter().find(|(code, _)| *code == iso) {
        return Some(label);
    }
    ISO_CODES
        .split_whitespace()
        .any(|c| c == iso)
        .then_some(DEFAULT_NATIONALITY)
}

/// The label of the game's "other" nationality.
pub const DEFAULT_NATIONALITY: &str = "default";

/// Every label of the game's `entrants.nationality` table (F1 25), in its row order.
pub const GAME_NATIONALITIES: [&str; 90] = [
    "default",
    "usa",
    "algeria",
    "argentina",
    "australia",
    "austria",
    "azerbaijan",
    "bahrain",
    "barbados",
    "belgium",
    "bolivia",
    "bosnia",
    "brazil",
    "britain",
    "bulgaria",
    "cameroon",
    "canada",
    "chile",
    "china",
    "columbia",
    "costa_rica",
    "croatia",
    "cyprus",
    "czech_republic",
    "denmark",
    "netherlands",
    "ecuador",
    "england",
    "uae",
    "estonia",
    "finland",
    "france",
    "germany",
    "ghana",
    "greece",
    "guatemala",
    "honduras",
    "hong_kong",
    "hungary",
    "iceland",
    "india",
    "indonesia",
    "ireland",
    "israel",
    "italy",
    "jamaica",
    "japan",
    "jordan",
    "kuwait",
    "latvia",
    "lebanon",
    "lithuania",
    "luxembourg",
    "malaysia",
    "malta",
    "mexico",
    "monaco",
    "new_zealand",
    "nicaragua",
    "northern_ireland",
    "norway",
    "oman",
    "pakistan",
    "panama",
    "paraguay",
    "peru",
    "phillipines",
    "poland",
    "portugal",
    "qatar",
    "romania",
    "el_salvador",
    "saudi_arabia",
    "scotland",
    "serbia",
    "singapore",
    "slovakia",
    "slovenia",
    "south_korea",
    "south_africa",
    "spain",
    "sweden",
    "switzerland",
    "thailand",
    "turkey",
    "uruguay",
    "ukraine",
    "venezuela",
    "vietnam",
    "wales",
];

/// The countries the game has a flag for, by ISO 3166-1 alpha-3 code; England, Scotland, Wales
/// and Northern Ireland, which have no ISO code, by their sporting codes.
pub const FLAGS: &[(&str, &str)] = &[
    ("USA", "usa"),
    ("DZA", "algeria"),
    ("ARG", "argentina"),
    ("AUS", "australia"),
    ("AUT", "austria"),
    ("AZE", "azerbaijan"),
    ("BHR", "bahrain"),
    ("BRB", "barbados"),
    ("BEL", "belgium"),
    ("BOL", "bolivia"),
    ("BIH", "bosnia"),
    ("BRA", "brazil"),
    ("GBR", "britain"),
    ("BGR", "bulgaria"),
    ("CMR", "cameroon"),
    ("CAN", "canada"),
    ("CHL", "chile"),
    ("CHN", "china"),
    ("COL", "columbia"),
    ("CRI", "costa_rica"),
    ("HRV", "croatia"),
    ("CYP", "cyprus"),
    ("CZE", "czech_republic"),
    ("DNK", "denmark"),
    ("NLD", "netherlands"),
    ("ECU", "ecuador"),
    ("ENG", "england"),
    ("ARE", "uae"),
    ("EST", "estonia"),
    ("FIN", "finland"),
    ("FRA", "france"),
    ("DEU", "germany"),
    ("GHA", "ghana"),
    ("GRC", "greece"),
    ("GTM", "guatemala"),
    ("HND", "honduras"),
    ("HKG", "hong_kong"),
    ("HUN", "hungary"),
    ("ISL", "iceland"),
    ("IND", "india"),
    ("IDN", "indonesia"),
    ("IRL", "ireland"),
    ("ISR", "israel"),
    ("ITA", "italy"),
    ("JAM", "jamaica"),
    ("JPN", "japan"),
    ("JOR", "jordan"),
    ("KWT", "kuwait"),
    ("LVA", "latvia"),
    ("LBN", "lebanon"),
    ("LTU", "lithuania"),
    ("LUX", "luxembourg"),
    ("MYS", "malaysia"),
    ("MLT", "malta"),
    ("MEX", "mexico"),
    ("MCO", "monaco"),
    ("NZL", "new_zealand"),
    ("NIC", "nicaragua"),
    ("NIR", "northern_ireland"),
    ("NOR", "norway"),
    ("OMN", "oman"),
    ("PAK", "pakistan"),
    ("PAN", "panama"),
    ("PRY", "paraguay"),
    ("PER", "peru"),
    ("PHL", "phillipines"),
    ("POL", "poland"),
    ("PRT", "portugal"),
    ("QAT", "qatar"),
    ("ROU", "romania"),
    ("SLV", "el_salvador"),
    ("SAU", "saudi_arabia"),
    ("SCO", "scotland"),
    ("SRB", "serbia"),
    ("SGP", "singapore"),
    ("SVK", "slovakia"),
    ("SVN", "slovenia"),
    ("KOR", "south_korea"),
    ("ZAF", "south_africa"),
    ("ESP", "spain"),
    ("SWE", "sweden"),
    ("CHE", "switzerland"),
    ("THA", "thailand"),
    ("TUR", "turkey"),
    ("URY", "uruguay"),
    ("UKR", "ukraine"),
    ("VEN", "venezuela"),
    ("VNM", "vietnam"),
    ("WAL", "wales"),
];

/// Every ISO 3166-1 alpha-3 code, and Kosovo's `XKX`: the countries without a flag in [`FLAGS`]
/// show the game's `default`.
pub const ISO_CODES: &str = "\
    ABW AFG AGO AIA ALA ALB AND ARE ARG ARM ASM ATA ATF ATG AUS AUT AZE BDI BEL BEN BES BFA BGD BGR BHR \
    BHS BIH BLM BLR BLZ BMU BOL BRA BRB BRN BTN BVT BWA CAF CAN CCK CHE CHL CHN CIV CMR COD COG COK COL \
    COM CPV CRI CUB CUW CXR CYM CYP CZE DEU DJI DMA DNK DOM DZA ECU EGY ERI ESH ESP EST ETH FIN FJI FLK \
    FRA FRO FSM GAB GBR GEO GGY GHA GIB GIN GLP GMB GNB GNQ GRC GRD GRL GTM GUF GUM GUY HKG HMD HND HRV \
    HTI HUN IDN IMN IND IOT IRL IRN IRQ ISL ISR ITA JAM JEY JOR JPN KAZ KEN KGZ KHM KIR KNA KOR KWT LAO \
    LBN LBR LBY LCA LIE LKA LSO LTU LUX LVA MAC MAF MAR MCO MDA MDG MDV MEX MHL MKD MLI MLT MMR MNE MNG \
    MNP MOZ MRT MSR MTQ MUS MWI MYS MYT NAM NCL NER NFK NGA NIC NIU NLD NOR NPL NRU NZL OMN PAK PAN PCN \
    PER PHL PLW PNG POL PRI PRK PRT PRY PSE PYF QAT REU ROU RUS RWA SAU SDN SEN SGP SGS SHN SJM SLB SLE \
    SLV SMR SOM SPM SRB SSD STP SUR SVK SVN SWE SWZ SXM SYC SYR TCA TCD TGO THA TJK TKL TKM TLS TON TTO \
    TUN TUR TUV TWN TZA UGA UKR UMI URY USA UZB VAT VCT VEN VGB VIR VNM VUT WLF WSM XKX YEM ZAF ZMB ZWE";
