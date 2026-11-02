//! The AI seats a dynamic line-up renames (W4-1): `driver.identity` layers.
//!
//! The game's season box keeps each year's real grid. With [`LineupMode::Dynamic`] the league's
//! AI market ([`aimarket`](crate::league::aimarket)) seats other drivers in some of its seats and
//! keeps the moves in [`Season::renames`](crate::league::Season): each one renames the row of the
//! seat's box driver to the driver the league seats there, with their name, code, number and
//! nationality. The face (the driver's look) stays the box driver's.
//!
//! The `content.package` layer restating the renamed seats' teams is written here, with the
//! numbers of the players' seats too: one layer per record, so the identity's file leaves it out
//! while seats are renamed ([`identity::package_layer`]).
//!
//! [`LineupMode::Dynamic`]: crate::league::LineupMode::Dynamic

use serde_json::{Map, Value, json};

use super::identity::{self, Row};
use super::{MARKET_DB, MARKET_LANG, MARKET_REGISTRY, Overlay, SeatRename, SyncContext, json_file};
use crate::league::League;

/// The addon files for the renamed seats of `overlay`: [`MARKET_REGISTRY`] always, and the
/// database and lang fallback ([`MARKET_DB`], [`MARKET_LANG`]) when the core doesn't apply the
/// registries' rows itself.
pub(super) fn files(overlay: &Overlay, ctx: &SyncContext) -> Vec<(String, Vec<u8>)> {
    if overlay.renames.is_empty() {
        return Vec::new();
    }
    let mut out = vec![json_file(MARKET_REGISTRY, &registry(overlay, ctx))];
    if !ctx.loader_applies_rows {
        let rows = || overlay.renames.iter().map(|r| row(overlay.year, r));
        out.push(json_file(MARKET_DB, &identity::rows_database(rows())));
        out.push(json_file(MARKET_LANG, &identity::rows_lang(rows())));
    }
    out
}

/// The AI seats renamed for the next round: the running season's [`Season::renames`]
/// (`aimarket` made them when the season began).
///
/// [`Season::renames`]: crate::league::Season
pub(super) fn build(league: &League, _ctx: &SyncContext) -> Vec<SeatRename> {
    league.season().renames.clone()
}

/// A rename's full name as the game shows it: "Lewis HAMILTON".
pub fn full_name(r: &SeatRename) -> String {
    match r.first.is_empty() {
        true => r.last.clone(),
        false => format!("{} {}", r.first, r.last),
    }
}

fn row(year: u16, r: &SeatRename) -> Row<'_> {
    Row {
        seat: &r.seat_driver,
        year,
        first: &r.first,
        last: &r.last,
        full: full_name(r),
        code: &r.code,
        number: r.number,
        nationality: r.nationality.as_deref(),
    }
}

/// `registry/market.json`: a `driver.identity` layer per renamed seat on its box driver's record,
/// and the `content.package` layer with the numbers the game shows.
fn registry(overlay: &Overlay, ctx: &SyncContext) -> Value {
    let mut identities = Map::new();
    for r in &overlay.renames {
        identities.insert(
            identity::identity_target(ctx, &r.seat_driver),
            identity::identity_layer(
                &r.first,
                &r.last,
                &full_name(r),
                &r.code,
                r.number,
                r.nationality.as_deref(),
            ),
        );
    }
    let mut out = json!({ "driver.identity": identities });
    if let Some(packages) = identity::package_layer(overlay, ctx) {
        out["content.package"] = packages;
    }
    out
}
