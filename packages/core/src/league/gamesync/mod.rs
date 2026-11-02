//! Game sync: the league career in the game itself, through a generated per-career addon
//! ([`ADDON_ID`]) that both PCs write byte for byte from the shared league document.
//!
//! This module is pure (league → overlay → files); the I/O is in `crate::careersync`. W1-C owns
//! the bodies of [`overlay`], [`identity_of`] and [`files`]; the submodules fill the overlay's
//! parts: [`identity`] (W2-9) the players' names and numbers, [`performance`] (W2-10) the cars'
//! pace, [`market`] (W4-1) the renamed AI seats of a dynamic line-up.

pub mod identity;
pub mod market;
pub mod performance;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::{League, LineupMode, Phase};

/// The generated addon's id.
pub const ADDON_ID: &str = "f1mp.league";
/// The generator's version, written into the addon's `league.json` and its hash: bump it when
/// the same league makes other files.
pub const GENERATOR: u32 = 1;
/// The addon's name on the Addons page.
pub const ADDON_NAME: &str = "League career sync";
/// The addon loads after every Classic Seasons addon. Not `*`: Driver Manager loads after `*`,
/// and two such addons are a load-order cycle.
pub const LOAD_AFTER: &str = "swofty.classic-seasons*";
/// The addon's manifest.
pub const MANIFEST_FILE: &str = "swofty.json";
/// What the addon was made for: `{"career","year","round","hash","generator"}`.
pub const LEAGUE_FILE: &str = "league.json";

// The body files the submodules write, beside the two above. A submodule writes only its own
// paths, each a canonical JSON file ([`json_file`]) under `registry/`, `db/` or `lang/`; [`files`]
// leaves out anything else, lists `db/` files as the manifest's database operations and `lang/`
// files as its asset operations, and hashes them all into `hash8`. Registry data files need no
// listing: the build finds every `registry/*.json`.

/// W2-9: `driver.identity` and `content.package` layers.
pub const IDENTITY_REGISTRY: &str = "registry/identity.json";
/// W2-9: the database fallback, written only when `!ctx.loader_applies_rows`.
pub const IDENTITY_DB: &str = "db/identity.json";
/// W2-9: the lang fallback for [`IDENTITY_DB`].
pub const IDENTITY_LANG: &str = "lang/identity.json";
/// W2-10: `vehicle.handling` layers. The build reads them from `registry/` like every data
/// file (`activation::handling::read_maps` over `stdlib::data::files`), not from `data/`.
pub const HANDLING_REGISTRY: &str = "registry/handling.json";
/// W4-1: `driver.identity` layers renaming AI seats (with the `content.package` layer restating
/// their teams' seats, and the players' when the identity is on too).
pub const MARKET_REGISTRY: &str = "registry/market.json";
/// W4-1: the database fallback for [`MARKET_REGISTRY`], written only when `!ctx.loader_applies_rows`.
pub const MARKET_DB: &str = "db/market.json";
/// W4-1: the lang fallback for [`MARKET_DB`].
pub const MARKET_LANG: &str = "lang/market.json";
/// The last year Classic Seasons has a season addon for; later years race the game's own grid.
pub const LAST_CLASSIC_SEASON: u16 = 2024;

/// What the game shows for a player.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GameIdentity {
    pub player: String,
    pub first: String,
    pub last: String,
    pub full: String,
    /// Three letters, unique on the grid.
    pub code: String,
    /// The number shown: the player's, or the seat's when it would clash.
    pub number: u8,
    /// The game's nationality label (W2-9 maps ISO codes to labels).
    pub nationality: Option<String>,
    /// The seat driver's key whose row is renamed.
    pub replaces: String,
    pub year: u16,
    pub slot: u8,
    pub team_key: String,
    /// The player's own number was taken that year.
    pub number_clash: bool,
}

/// A team's car with the league's pace (see [`performance`]).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CarOverride {
    pub team_key: String,
    pub slot: u8,
    pub mass_kg: f32,
    pub erp: String,
    pub stock: bool,
    /// The experimental per-part feel (`Options::game_parts`, W2-10); `None` when off.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parts: Option<performance::PartScales>,
}

/// An AI seat renamed for a dynamic line-up (kept in [`Season::renames`](super::Season)).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeatRename {
    pub slot: u8,
    pub seat_driver: String,
    pub key: String,
    pub first: String,
    pub last: String,
    pub code: String,
    pub number: u8,
    pub nationality: Option<String>,
}

/// Everything the addon carries for one round.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Overlay {
    pub career: String,
    pub career_name: String,
    pub year: u16,
    pub round: u8,
    pub identities: Vec<GameIdentity>,
    pub cars: Vec<CarOverride>,
    pub renames: Vec<SeatRename>,
}

/// What the generator needs from the installed Classic Seasons addons (read by `careersync`,
/// fixtures in tests).
pub struct SyncContext<'a> {
    /// `classic-seasons-<year>/registry/content.json`.
    pub content: &'a serde_json::Value,
    /// `classic-seasons/registry/drivers.json` (the driver keys Classic Seasons adds).
    pub cs_drivers: &'a serde_json::Value,
    /// `classic-seasons-<year>/assets/liveries.json` (the team archive paths).
    pub liveries: &'a serde_json::Value,
    /// `classic-seasons/data/ai_slots.json`.
    pub ai_slots: &'a serde_json::Value,
    /// `stdlib::manifest::core_applies_rows`.
    pub loader_applies_rows: bool,
}

/// The Classic Seasons addon of `year`'s season box, `None` for a year the game has itself.
pub fn season_addon(year: u16) -> Option<String> {
    (super::FIRST_YEAR..=LAST_CLASSIC_SEASON)
        .contains(&year)
        .then(|| format!("swofty.classic-seasons-{year}"))
}

impl SyncContext<'static> {
    /// No Classic Seasons files: what [`identity_of`] hands [`identity::build`].
    pub fn empty() -> Self {
        static NULL: Value = Value::Null;
        SyncContext {
            content: &NULL,
            cs_drivers: &NULL,
            liveries: &NULL,
            ai_slots: &NULL,
            loader_applies_rows: true,
        }
    }
}

/// The overlay for the league's next round; `None` outside a running season, past its last
/// round, or with every game option off (`game_identity`, `game_performance`, a dynamic
/// line-up; `game_parts` only refines the performance). Each part comes from its submodule's
/// `build` while its option is on, sorted here so the order never depends on the submodule.
pub fn overlay(league: &League, career: &str, career_name: &str, ctx: &SyncContext) -> Option<Overlay> {
    if league.phase != Phase::Racing {
        return None;
    }
    let o = &league.options;
    let dynamic = o.lineups == LineupMode::Dynamic;
    if !o.game_identity && !o.game_performance && !dynamic {
        return None;
    }
    let season = league.season();
    let round = season.next_round()?.round;
    let (mut identities, mut cars, mut renames) = (Vec::new(), Vec::new(), Vec::new());
    if o.game_identity {
        identities = identity::build(league, ctx);
        identities.sort_by(|a, b| a.player.cmp(&b.player));
    }
    if o.game_performance {
        cars = performance::build(league, ctx);
        cars.sort_by(|a, b| (a.slot, &a.team_key).cmp(&(b.slot, &b.team_key)));
    }
    if dynamic {
        renames = market::build(league, ctx);
        renames.sort_by(|a, b| (a.slot, &a.seat_driver).cmp(&(b.slot, &b.seat_driver)));
    }
    Some(Overlay {
        career: career.to_owned(),
        career_name: career_name.to_owned(),
        year: season.year,
        round,
        identities,
        cars,
        renames,
    })
}

/// What the game shows for `player` (`Briefing::game`); `None` outside a running season or when
/// the identity option is off. It has no Classic Seasons files to hand, so it calls
/// [`identity::build`] with [`SyncContext::empty`]: `build` must take names, codes and numbers
/// from the league alone and use the context only for layer targets, so the briefing and the
/// game agree.
pub fn identity_of(league: &League, player: &str) -> Option<GameIdentity> {
    if league.phase != Phase::Racing || !league.options.game_identity {
        return None;
    }
    identity::build(league, &SyncContext::empty())
        .into_iter()
        .find(|i| i.player == player)
}

/// The addon's files, as (relative path, bytes), sorted by path: `swofty.json`, `league.json`
/// and the body files of the overlay's parts. A part that is empty writes nothing (its
/// submodule's `files` isn't called). Two PCs with the same overlay and the same Classic Seasons
/// files get the same bytes.
pub fn files(overlay: &Overlay, ctx: &SyncContext) -> Vec<File> {
    let mut body = Vec::new();
    if !overlay.identities.is_empty() {
        body.extend(identity::files(overlay, ctx));
    }
    if !overlay.cars.is_empty() {
        body.extend(performance::files(overlay, ctx));
    }
    if !overlay.renames.is_empty() {
        body.extend(market::files(overlay, ctx));
    }
    assemble(overlay, body)
}

/// One file of the addon: its path relative to the addon folder, with `/`, and its bytes.
pub type File = (String, Vec<u8>);

/// The addon's files from the submodules' `body` files: leaves out a path no submodule may write
/// ([`IDENTITY_REGISTRY`] and the others are fine) and a second file at one path, then adds
/// `league.json` and `swofty.json`. Sorted by path.
pub(super) fn assemble(overlay: &Overlay, body_files: Vec<File>) -> Vec<File> {
    let mut body: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for (path, bytes) in body_files {
        if body_path(&path) && !body.contains_key(&path) {
            body.insert(path, bytes);
        }
    }
    let hash = hash8(overlay, &body);
    let league = serde_json::json!({
        "career": overlay.career,
        "year": overlay.year,
        "round": overlay.round,
        "hash": hash,
        "generator": GENERATOR,
    });
    let manifest = manifest(overlay, &hash, body.keys().map(String::as_str));
    body.insert(LEAGUE_FILE.to_owned(), canonical_json(&league));
    body.insert(MANIFEST_FILE.to_owned(), canonical_json(&manifest));
    body.into_iter().collect()
}

/// A path a submodule may write: relative, `/`-separated, a `.json` file in `registry/`, `db/`
/// or `lang/`.
fn body_path(path: &str) -> bool {
    let mut parts = path.split('/');
    let top = parts.next().unwrap_or_default();
    let rest: Vec<&str> = parts.collect();
    matches!(top, "registry" | "db" | "lang")
        && !rest.is_empty()
        && rest.iter().all(|p| !p.is_empty() && !p.starts_with('.'))
        && path.ends_with(".json")
        && !path.contains('\\')
}

/// The first 8 hex digits of a SHA-256 over what the overlay was made for (generator, career,
/// year, round) and every body file, by path. `swofty.json` and `league.json` carry it, so they
/// stay out.
pub fn hash8(overlay: &Overlay, body: &BTreeMap<String, Vec<u8>>) -> String {
    let mut h = Sha256::new();
    h.update(format!(
        "{ADDON_ID} generator {GENERATOR}\0career {}\0year {}\0round {}\0",
        overlay.career, overlay.year, overlay.round
    ));
    for (path, bytes) in body {
        h.update(format!("{path}\0{}\0", bytes.len()));
        h.update(bytes);
    }
    let digest = h.finalize();
    digest[..4].iter().map(|b| format!("{b:02x}")).collect()
}

/// The overlay's `swofty.json`: named and versioned by year, round and hash, after Classic
/// Seasons, gameplay for a lobby (never cosmetic), with the body's `db/` files as database and
/// its `lang/` files as asset operations.
fn manifest<'a>(overlay: &Overlay, hash: &str, body: impl Iterator<Item = &'a str>) -> Value {
    let (mut database, mut assets) = (Vec::new(), Vec::new());
    for path in body {
        if path.starts_with("db/") {
            database.push(path);
        } else if path.starts_with("lang/") {
            assets.push(path);
        }
    }
    let mut kind = vec!["files"];
    if !database.is_empty() {
        kind.insert(0, "database");
    }
    let mut m = serde_json::json!({
        "schema_version": 1,
        "id": ADDON_ID,
        "name": ADDON_NAME,
        "version": version(overlay.year, overlay.round, hash),
        "author": "F1 Multiplayer",
        "description": format!(
            "Generated by F1 Multiplayer for career {}; don't edit",
            overlay.career_name
        ),
        "kind": kind,
        "compat": { "game": "f1_25" },
        "load_after": [LOAD_AFTER],
        "multiplayer": "gameplay",
    });
    if let Some(season) = season_addon(overlay.year) {
        m["requires"] = serde_json::json!([{ "id": season }]);
    }
    let mut ops = serde_json::Map::new();
    if !database.is_empty() {
        ops.insert("database".into(), serde_json::json!(database));
    }
    if !assets.is_empty() {
        ops.insert("assets".into(), serde_json::json!(assets));
    }
    if !ops.is_empty() {
        m["operations"] = Value::Object(ops);
    }
    m
}

/// `0.<year>.<round>+<hash8>`. The hash goes in the build metadata: as a pre-release it would be
/// invalid whenever it is all digits with a leading zero.
pub fn version(year: u16, round: u8, hash: &str) -> String {
    format!("0.{year}.{round}+{hash}")
}

/// `value` as canonical JSON: keys sorted, two-space indents, LF line ends and a final LF,
/// integers as they are and other numbers with one decimal (`{:.1}`). The same value gives the
/// same bytes on every PC.
pub fn canonical_json(value: &Value) -> Vec<u8> {
    let mut out = String::new();
    write_canonical(value, 0, &mut out);
    out.push('\n');
    out.into_bytes()
}

fn write_canonical(value: &Value, depth: usize, out: &mut String) {
    let pad = |out: &mut String, depth: usize| out.extend(std::iter::repeat_n("  ", depth));
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => match (n.as_i64(), n.as_u64(), n.as_f64()) {
            (Some(i), _, _) => out.push_str(&i.to_string()),
            (_, Some(u), _) => out.push_str(&u.to_string()),
            (_, _, Some(f)) => out.push_str(&format!("{f:.1}")),
            _ => out.push_str("null"),
        },
        Value::String(s) => out.push_str(&serde_json::to_string(s).expect("a string serialises")),
        Value::Array(items) if items.is_empty() => out.push_str("[]"),
        Value::Array(items) => {
            out.push_str("[\n");
            for (i, item) in items.iter().enumerate() {
                pad(out, depth + 1);
                write_canonical(item, depth + 1, out);
                out.push_str(if i + 1 < items.len() { ",\n" } else { "\n" });
            }
            pad(out, depth);
            out.push(']');
        }
        Value::Object(map) if map.is_empty() => out.push_str("{}"),
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            out.push_str("{\n");
            for (i, key) in keys.iter().enumerate() {
                pad(out, depth + 1);
                out.push_str(&serde_json::to_string(key).expect("a string serialises"));
                out.push_str(": ");
                write_canonical(&map[*key], depth + 1, out);
                out.push_str(if i + 1 < keys.len() { ",\n" } else { "\n" });
            }
            pad(out, depth);
            out.push('}');
        }
    }
}

/// A body file at `path` holding `value` as [`canonical_json`]: what the submodules' `files`
/// return.
pub fn json_file(path: &str, value: &Value) -> (String, Vec<u8>) {
    (path.to_owned(), canonical_json(value))
}
