//! Tests of the league's aimarket and gamesync::market (W4-1): the dynamic AI driver market,
//! its renamed seats in the league and in the game, and the historic grid when it is off.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::Path;

use serde_json::Value;

use super::aimarket::{self, MoveKind};
use super::gamesync::identity::{base_code, game_nationality, split_name};
use super::gamesync::{self, IDENTITY_REGISTRY, MARKET_DB, MARKET_LANG, MARKET_REGISTRY, SyncContext};
use super::tests::agreed;
use super::*;

const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/classic-seasons-mini");

fn actor(id: &str, name: &str) -> Actor {
    Actor {
        id: id.into(),
        name: name.into(),
        owner: id == "u-anna",
    }
}

fn anna() -> Actor {
    actor("u-anna", "Anna Müller")
}

fn ben() -> Actor {
    actor("u-ben", "Ben Becker")
}

/// A career from `year` (to 2026) with the seed `seed`: Anna (#42) in Kimi Räikkonen's Ferrari,
/// Ben (#31) in Felipe Massa's Williams (in years without them, the slower driver's seat), with
/// the dynamic line-up on.
fn career(year: u16, seed: u64) -> League {
    let setup = Setup {
        first_year: year,
        last_year: 2026,
        ..Setup::default()
    };
    let mut l = League::start(setup, &anna(), Some(42), seed).unwrap();
    apply(&mut l, &ben(), Action::Enter { number: Some(31) }).unwrap();
    for (a, slot, key) in [(anna(), 2, "kimi_raikkonen"), (ben(), 7, "felipe_massa")] {
        let replaces = catalog_driver(year, slot, key).map(|d| d.key.clone());
        apply(&mut l, &a, Action::Seat { slot, replaces }).unwrap();
    }
    apply(&mut l, &anna(), Action::Begin).unwrap();
    let options = Options {
        lineups: LineupMode::Dynamic,
        ..l.options.clone()
    };
    apply(&mut l, &anna(), Action::SetOptions { options }).unwrap();
    l
}

/// The players' seats for `next`: the same team, the slower box driver's seat.
fn seats_for(l: &League, next: u16) -> Vec<(Player, Seat)> {
    let mut taken: Vec<(u8, String)> = Vec::new();
    l.players
        .iter()
        .filter_map(|p| {
            let slot = p.seat.as_ref()?.slot;
            let team = catalog_team(next, slot)?;
            let others: Vec<&str> = taken
                .iter()
                .filter(|(s, _)| *s == slot)
                .map(|(_, k)| k.as_str())
                .collect();
            let replaces = choose_replaced(team, None, &others).ok()?;
            taken.push((slot, replaces.clone()));
            Some((p.clone(), Seat { slot, replaces }))
        })
        .collect()
}

/// The market from `l`'s season to the next year, as `next_season` makes it.
fn plan_next(l: &League) -> aimarket::Plan {
    let next = l.year() + 1;
    aimarket::plan(l, next, &seats_for(l, next)).unwrap()
}

/// `l` moved on to the next season without racing: the players stay in their teams.
fn move_on(l: &mut League) {
    let next = l.year() + 1;
    for (p, seat) in seats_for(l, next) {
        let me = l.players.iter_mut().find(|q| q.id == p.id).unwrap();
        me.signed = Some(seat);
        me.signed_until = Some(next);
    }
    l.next_season().unwrap();
}

/// The box seats (team, box driver's key) of `season`'s year, in the order of its entries.
fn box_seats(year: u16) -> Vec<(u8, &'static str)> {
    catalog_year(year)
        .unwrap()
        .teams
        .iter()
        .flat_map(|t| t.drivers.iter().map(move |d| (t.slot, d.key.as_str())))
        .collect()
}

/// Every check a dynamic season must pass: each box seat once, each driver once, unique
/// numbers among the AI, no retired driver, and renames exactly where an AI seat's driver isn't
/// its box driver, with codes unique on the grid.
fn check_season(season: &Season, players: &[Player]) {
    let year = season.year;
    let seats = box_seats(year);
    assert_eq!(season.entries.len(), seats.len(), "{year}: one entry per seat");
    let ids: HashSet<&str> = season.entries.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(ids.len(), seats.len(), "{year}: every driver once");
    let numbers: HashSet<u8> = season
        .entries
        .iter()
        .filter(|e| e.player.is_none())
        .map(|e| e.number)
        .collect();
    assert_eq!(
        numbers.len(),
        season.entries.iter().filter(|e| e.player.is_none()).count(),
        "{year}: AI numbers are unique"
    );
    assert!(
        season.changes().is_empty() && season.reserves.is_empty(),
        "{year}: no real changes"
    );
    let mut renamed = 0;
    for (e, (slot, key)) in season.entries.iter().zip(&seats) {
        assert_eq!(e.slot, *slot);
        if e.player.is_some() {
            assert!(
                !season.renames.iter().any(|r| r.seat_driver == *key),
                "a player's seat isn't renamed"
            );
            continue;
        }
        let last = catalog()
            .years
            .iter()
            .filter(|(_, y)| {
                y.teams.iter().flat_map(|t| &t.drivers).any(|d| d.key == e.id)
                    || y.changes.iter().any(|c| c.driver == e.id)
            })
            .map(|(y, _)| *y)
            .max();
        assert!(
            last.is_none_or(|l| l >= year),
            "{year}: {} retired in {last:?}",
            e.id
        );
        let rename = season.renames.iter().find(|r| r.seat_driver == *key);
        if e.id == *key {
            assert!(rename.is_none(), "{year}: {key} drives their own seat");
            continue;
        }
        renamed += 1;
        let r = rename.unwrap_or_else(|| panic!("{year}: {} in {key}'s seat isn't renamed", e.id));
        let (first, last) = split_name(&e.name);
        assert_eq!(
            (
                r.slot,
                r.key.as_str(),
                r.number,
                r.first.as_str(),
                r.last.as_str()
            ),
            (e.slot, e.id.as_str(), e.number, first.as_str(), last.as_str())
        );
        assert_eq!(
            r.nationality.as_deref(),
            aimarket::nationality_of(&e.id).and_then(game_nationality),
            "{}",
            e.id
        );
        assert_eq!(aimarket::seat_of(season, e), (*slot, *key));
    }
    assert_eq!(
        renamed,
        season.renames.len(),
        "{year}: no rename without its entry"
    );
    // Codes: the renamed drivers', the box drivers' who keep their seat, the players'.
    let mut codes: Vec<String> = season.renames.iter().map(|r| r.code.clone()).collect();
    for (e, (slot, key)) in season.entries.iter().zip(&seats) {
        if e.player.is_none() && e.id == *key {
            let (first, last) = split_name(&catalog_driver(year, *slot, key).unwrap().name);
            codes.push(base_code(&first, &last));
        }
    }
    codes.extend(
        gamesync::identity::shown(season, players)
            .into_iter()
            .map(|i| i.code),
    );
    let unique: BTreeSet<&String> = codes.iter().collect();
    assert_eq!(unique.len(), codes.len(), "{year}: codes are unique: {codes:?}");
    // The players' shown numbers clash with no AI car's (each AI car shows its entry's number).
    for i in gamesync::identity::shown(season, players) {
        assert!(
            !numbers.contains(&i.number),
            "{year}: {} shows #{}",
            i.player,
            i.number
        );
    }
}

#[test]
fn the_market_is_deterministic() {
    let l = career(2016, 42);
    let a = plan_next(&l);
    let b = plan_next(&l.clone());
    assert_eq!(a, b);
    // Through the whole season change too, on two PCs.
    let (mut one, mut two) = (l.clone(), l.clone());
    move_on(&mut one);
    move_on(&mut two);
    assert_eq!(one, two);
    assert_eq!(one.season().entries, a.season.entries);
    assert_eq!(one.season().renames, a.season.renames);
    // Another seed makes another market.
    assert!(
        (0..10).any(|seed| plan_next(&career(2016, seed)).season.entries != a.season.entries),
        "the seed decides the moves"
    );
}

#[test]
fn every_seat_is_filled_once() {
    for seed in 0..12 {
        let mut l = career(2016, seed);
        // Ten seasons of markets, from 2016's grid to 2026's.
        while l.year() < 2026 {
            move_on(&mut l);
            check_season(l.season(), &l.players);
        }
        // Drivers who left the grid don't come back as rookies.
        let mut seen: BTreeMap<&str, Vec<u16>> = BTreeMap::new();
        for s in &l.seasons {
            for e in s.entries.iter().filter(|e| e.player.is_none()) {
                seen.entry(e.id.as_str()).or_default().push(s.year);
            }
        }
        for (id, years) in seen {
            assert!(
                years.windows(2).all(|w| w[1] == w[0] + 1),
                "seed {seed}: {id} raced {years:?}"
            );
        }
    }
    // From each year's real grid too.
    for year in 2016..2026 {
        for seed in [1, 7] {
            let l = career(year, seed);
            let plan = plan_next(&l);
            check_season(&plan.season, &l.players);
        }
    }
}

#[test]
fn players_keep_their_signed_seats() {
    let setup = Setup {
        first_year: 2016,
        last_year: 2017,
        ..Setup::default()
    };
    let mut l = League::start(setup, &anna(), Some(42), 9).unwrap();
    apply(&mut l, &ben(), Action::Enter { number: Some(31) }).unwrap();
    apply(
        &mut l,
        &anna(),
        Action::Seat {
            slot: 2,
            replaces: Some("kimi_raikkonen".into()),
        },
    )
    .unwrap();
    apply(
        &mut l,
        &ben(),
        Action::Seat {
            slot: 7,
            replaces: Some("felipe_massa".into()),
        },
    )
    .unwrap();
    apply(&mut l, &anna(), Action::Begin).unwrap();
    let warnings = apply(
        &mut l,
        &anna(),
        Action::SetOptions {
            options: Options {
                lineups: LineupMode::Dynamic,
                ..Options::default()
            },
        },
    )
    .unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    assert!(
        l.options.game_identity,
        "a dynamic line-up shows the players in the game"
    );
    // The first season is the real grid.
    assert!(l.season().renames.is_empty());
    while l.phase == Phase::Racing {
        agreed(&mut l, &anna(), Action::Simulate { round: None });
    }
    assert_eq!(l.phase, Phase::SeasonEnd);
    let mv = l
        .offers("u-anna")
        .iter()
        .find(|o| o.kind == OfferKind::Move)
        .map(|o| o.slot)
        .unwrap();
    for (a, slot, replaces) in [
        (anna(), mv, None),
        (ben(), 2, Some("sebastian_vettel".to_owned())),
    ] {
        let offered = l.offers(&a.id).iter().any(|o| o.slot == slot);
        if !offered {
            continue;
        }
        apply(
            &mut l,
            &a,
            Action::Sign {
                slot,
                replaces,
                focus: None,
            },
        )
        .unwrap();
    }
    if l.phase == Phase::SeasonEnd {
        apply(&mut l, &anna(), Action::NextSeason).unwrap();
    }
    assert_eq!(l.year(), 2017);
    let season = l.season();
    for p in &l.players {
        let seat = p.seat.as_ref().expect("a seat for 2017");
        let e = season.player_entry(&p.id).expect("the player's car");
        assert_eq!(e.slot, seat.slot, "{}", p.name);
        let at = box_seats(2017)
            .iter()
            .position(|(s, k)| *s == seat.slot && *k == seat.replaces)
            .unwrap();
        assert_eq!(season.entries[at].id, e.id, "{} sits in the seat signed", p.name);
        assert!(!season.renames.iter().any(|r| r.seat_driver == seat.replaces));
    }
    check_season(season, &l.players);
    // The market's moves are in the news, as templates with their English text.
    let market: Vec<&News> = l
        .news
        .iter()
        .filter(|n| {
            n.year == 2017
                && n.template
                    .as_deref()
                    .is_some_and(|t| aimarket::TEMPLATES.contains(&t))
        })
        .collect();
    assert!(!market.is_empty(), "the market made news");
    for n in &market {
        assert_eq!(n.text, inbox::render(n.template.as_deref().unwrap(), &n.args));
        assert!(!n.text.contains('{'), "{}", n.text);
    }
    let begins = l
        .news
        .iter()
        .position(|n| n.text.starts_with("The 2017 season begins"))
        .unwrap();
    assert!(
        l.news.iter().position(|n| std::ptr::eq(n, market[0])).unwrap() < begins,
        "the moves come before the season begins"
    );
}

#[test]
fn the_news_name_every_move() {
    let l = career(2016, 3);
    let plan = plan_next(&l);
    let mut moved = l.clone();
    move_on(&mut moved);
    let texts: Vec<&str> = moved
        .news
        .iter()
        .filter(|n| n.template.is_some() && n.year == 2017)
        .map(|n| n.text.as_str())
        .collect();
    assert_eq!(texts.len(), plan.moves.len());
    // Manor is gone in 2017: its drivers at the end of 2016 moved or found no seat.
    for key in ["pascal_wehrlein", "esteban_ocon"] {
        let in_2017 = plan.season.entries.iter().any(|e| e.id == key);
        let m = plan.moves.iter().find(|m| m.key == key);
        match (in_2017, m.map(|m| m.kind)) {
            (true, Some(MoveKind::Transfer))
            | (false, Some(MoveKind::NoSeat | MoveKind::Retired | MoveKind::Dropped)) => {}
            other => panic!("{key}: {other:?}"),
        }
    }
    // Nico Rosberg retired after 2016.
    let rosberg = plan.moves.iter().find(|m| m.key == "nico_rosberg").unwrap();
    assert_eq!(rosberg.kind, MoveKind::Retired);
    assert!(texts.contains(&"Nico Rosberg retires from Formula 1 after the 2016 season."));
}

#[test]
fn renames_match_the_entries() {
    for seed in 0..8 {
        let l = career(2018, seed);
        let plan = plan_next(&l);
        check_season(&plan.season, &l.players);
    }
    // Rosberg's 2016 Mercedes seat goes to someone else in 2017 (the 2017 box has Bottas there,
    // and keeps him only if the market does).
    let l = career(2016, 42);
    let plan = plan_next(&l);
    let at = box_seats(2017)
        .iter()
        .position(|(_, k)| *k == "valtteri_bottas")
        .unwrap();
    let e = &plan.season.entries[at];
    if e.id != "valtteri_bottas" {
        let r = plan
            .season
            .renames
            .iter()
            .find(|r| r.seat_driver == "valtteri_bottas")
            .unwrap();
        assert_eq!(r.key, e.id);
    }
}

#[test]
fn historic_mode_is_unchanged() {
    let mut l = career(2016, 42);
    l.options.lineups = LineupMode::Historic;
    let before = l.clone();
    move_on(&mut l);
    let seats: Vec<(Player, Seat)> = l
        .players
        .iter()
        .filter_map(|p| p.seat.clone().map(|s| (p.clone(), s)))
        .collect();
    let expected = build_season(2017, &seats, l.setup.lineup_changes).unwrap();
    let season = l.season();
    assert_eq!(season.entries, expected.entries, "the real grid");
    assert_eq!(season.reserves, expected.reserves);
    let plain = |s: &Season| -> Vec<(u8, Option<u8>, u8, String, String)> {
        s.changes()
            .iter()
            .map(|c| (c.round, c.until, c.slot, c.out.clone(), c.driver.clone()))
            .collect()
    };
    assert!(!plain(season).is_empty());
    assert_eq!(
        plain(season),
        plain(&expected),
        "with the real mid-season changes"
    );
    assert!(season.renames.is_empty());
    assert!(
        !l.news[before.news.len()..].iter().any(|n| n.template.is_some()),
        "no market news"
    );
    // A league document from before the market loads and has no renames.
    let mut doc = serde_json::to_value(&before).unwrap();
    for s in doc["seasons"].as_array_mut().unwrap() {
        s.as_object_mut().unwrap().remove("renames");
    }
    doc["options"].as_object_mut().unwrap().remove("lineups");
    let old: League = serde_json::from_value(doc).unwrap();
    assert_eq!(old.options.lineups, LineupMode::Historic);
    assert!(old.seasons.iter().all(|s| s.renames.is_empty()));
}

#[test]
fn set_options_keeps_the_names_in_the_game_with_a_dynamic_line_up() {
    let mut l = career(2016, 1);
    assert!(l.options.game_identity);
    let mut options = l.options.clone();
    options.game_identity = false;
    let warnings = apply(&mut l, &anna(), Action::SetOptions { options }).unwrap();
    assert!(l.options.game_identity);
    assert_eq!(
        warnings,
        ["the players' names stay in the game while AI drivers move between teams"]
    );
    // Back to the historic grid, the names may go.
    let options = Options {
        lineups: LineupMode::Historic,
        game_identity: false,
        ..l.options.clone()
    };
    apply(&mut l, &anna(), Action::SetOptions { options }).unwrap();
    assert!(!l.options.game_identity);
}

/// A dynamic 2017 season to race: `l`'s 2016 career moved on by the market.
fn dynamic_2017(seed: u64) -> League {
    let mut l = career(2016, seed);
    move_on(&mut l);
    assert!(!l.season().renames.is_empty(), "seed {seed} renames seats");
    l
}

/// The game's race of `l`'s next round as Anna's game reports it: every car in grid order, Anna
/// first; each AI car under the name and number `shown` gives it.
fn report_of(l: &League, shown: impl Fn(&Entry) -> (String, u8)) -> RaceReport {
    let season = l.season();
    let round = season.next_round().unwrap();
    let mut cars: Vec<ReportCar> = season
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let mine = e.player.as_deref() == Some("u-anna");
            let (name, number) = if mine {
                ("Player".to_owned(), e.number)
            } else {
                shown(e)
            };
            ReportCar {
                name,
                number,
                team: e.slot,
                ai: !mine,
                mine,
                position: 0,
                grid: i as u8 + 1,
                laps: 30,
                status: 3,
                best_lap_ms: 90_000 + i as u32 * 10,
                time_ms: 3_600_000 + i as u64 * 1000,
                penalty_s: 0,
            }
        })
        .collect();
    cars.sort_by_key(|c| (!c.mine, c.grid));
    for (i, c) in cars.iter_mut().enumerate() {
        c.position = i as u8 + 1;
    }
    RaceReport {
        session: 0x2017,
        track: round.track,
        session_type: 15,
        cars,
        ..RaceReport::default()
    }
}

/// The name and number the game shows for `e` with the overlay: a renamed seat's driver's, a
/// player's identity, else the box driver's (who is the entry).
fn with_overlay(l: &League) -> impl Fn(&Entry) -> (String, u8) + '_ {
    let shown = gamesync::identity::shown(l.season(), &l.players);
    move |e: &Entry| {
        if let Some(p) = e.player.as_deref() {
            let i = shown.iter().find(|i| i.player == p).unwrap();
            return (i.full.clone(), i.number);
        }
        match l.season().renames.iter().find(|r| r.key == e.id) {
            Some(r) => (r.last.clone(), r.number),
            None => (e.name.to_uppercase(), e.number),
        }
    }
}

/// The name and number the game shows for `e` without the overlay: its seat's box driver's.
fn without_overlay(l: &League) -> impl Fn(&Entry) -> (String, u8) + '_ {
    let year = l.year();
    let seats = box_seats(year);
    move |e: &Entry| {
        let at = l.season().entries.iter().position(|x| x.id == e.id).unwrap();
        let (slot, key) = seats[at];
        let d = catalog_driver(year, slot, key).unwrap();
        (d.name.to_uppercase(), d.number)
    }
}

fn count(l: &mut League, report: RaceReport) -> Vec<String> {
    apply(
        l,
        &anna(),
        Action::Report {
            round: None,
            kind: None,
            force: true,
            report,
        },
    )
    .unwrap()
}

#[test]
fn classify_counts_a_renamed_ai_car_for_its_new_driver() {
    for overlay in [true, false] {
        let mut l = dynamic_2017(42);
        let report = match overlay {
            true => report_of(&l, with_overlay(&l)),
            false => report_of(&l, without_overlay(&l)),
        };
        let n = l.season().next_round().unwrap().round;
        let warnings = count(&mut l, report.clone());
        assert!(
            !warnings.iter().any(|w| w.contains("matched no driver")),
            "overlay {overlay}: {warnings:?}"
        );
        assert_eq!(
            warnings.iter().any(|w| w.contains("seats the AI market changed")),
            !overlay,
            "{warnings:?}"
        );
        let season = l.season();
        let result = &season.round(n).unwrap().race_result;
        assert_eq!(
            result.len(),
            season.entries.len(),
            "overlay {overlay}: every car counts"
        );
        // Each car counts for the league's driver of its seat: the car at grid place i is entry i.
        for c in result {
            let e = season.entry(&c.entry).unwrap();
            let at = season.entries.iter().position(|x| x.id == e.id).unwrap();
            let car = report.cars.iter().find(|x| x.grid as usize == at + 1).unwrap();
            assert_eq!(
                c.position, car.position,
                "overlay {overlay}: {} in its own car",
                e.id
            );
        }
        for r in &season.renames {
            assert!(result.iter().any(|c| c.entry == r.key), "{} counted", r.key);
        }
    }
}

/// The fixture's Classic Seasons 2016 files.
struct Ctx {
    content: Value,
    cs_drivers: Value,
    liveries: Value,
    ai_slots: Value,
}

impl Ctx {
    fn new() -> Self {
        let read = |rel: &str| -> Value {
            serde_json::from_slice(&std::fs::read(Path::new(FIXTURE).join(rel)).unwrap()).unwrap()
        };
        Self {
            content: read("swofty.classic-seasons-2016/registry/content.json"),
            cs_drivers: read("swofty.classic-seasons/registry/drivers.json"),
            liveries: read("swofty.classic-seasons-2016/assets/liveries.json"),
            ai_slots: read("swofty.classic-seasons/data/ai_slots.json"),
        }
    }

    fn get(&self, loader_applies_rows: bool) -> SyncContext<'_> {
        SyncContext {
            content: &self.content,
            cs_drivers: &self.cs_drivers,
            liveries: &self.liveries,
            ai_slots: &self.ai_slots,
            loader_applies_rows,
        }
    }
}

/// A dynamic 2016 career for the fixture (which has Classic Seasons 2016 only): the market run
/// over 2016's own grid, with a renamed seat of a driver Classic Seasons adds.
fn dynamic_2016() -> League {
    for seed in 0..200 {
        let mut l = career(2016, seed);
        let plan = aimarket::plan(&l, 2016, &l.seats()).unwrap();
        let renames = &plan.season.renames;
        let cs = ["felipe_nasr", "esteban_gutierrez", "rio_haryanto"];
        if renames.iter().any(|r| cs.contains(&r.seat_driver.as_str())) {
            let season = l.seasons.last_mut().unwrap();
            season.entries = plan.season.entries;
            season.renames = plan.season.renames;
            season.changes = Some(Vec::new());
            season.reserves.clear();
            return l;
        }
    }
    panic!("no seed renames a Classic Seasons driver's seat");
}

#[test]
fn market_files_restate_the_seats_once() {
    let ctx = Ctx::new();
    let l = dynamic_2016();
    check_season(l.season(), &l.players);
    let overlay = gamesync::overlay(&l, "c-1", "Anna & Ben", &ctx.get(true)).unwrap();
    assert_eq!(overlay.renames, l.season().renames);
    let files = gamesync::files(&overlay, &ctx.get(true));
    let json = |path: &str| -> Value {
        serde_json::from_slice(&files.iter().find(|(p, _)| p == path).unwrap().1).unwrap()
    };
    let market = json(MARKET_REGISTRY);
    let identity = json(IDENTITY_REGISTRY);
    assert!(identity.get("content.package").is_none(), "one layer per record");
    let layers = market["driver.identity"].as_object().unwrap();
    assert_eq!(layers.len(), overlay.renames.len());
    for r in &overlay.renames {
        let target = if ["felipe_nasr", "esteban_gutierrez", "rio_haryanto"].contains(&r.seat_driver.as_str())
        {
            format!("swofty.classic-seasons/{}", r.seat_driver)
        } else {
            format!("game/{}", r.seat_driver)
        };
        let layer = &layers[&target];
        assert_eq!(layer["names"]["last"]["literal"], r.last.as_str());
        assert_eq!(layer["names"]["code"], r.code.as_str());
        assert_eq!(layer["number"], r.number);
    }
    // The content restates every seat of the teams with a renamed seat or a player, with the
    // numbers the game shows.
    let package = market["content.package"]["swofty.classic-seasons-2016/f1_2016"]["teams"]
        .as_array()
        .unwrap();
    let original = ctx.content["content.package"]["f1_2016"]["teams"]
        .as_array()
        .unwrap();
    let key_of = |d: &Value| {
        d["driver"]
            .as_str()
            .unwrap()
            .rsplit('/')
            .next()
            .unwrap()
            .to_owned()
    };
    let mut restated = BTreeSet::new();
    for team in package {
        let name = team["key"].as_str().unwrap();
        let before = original.iter().find(|t| t["key"] == name).unwrap();
        assert_eq!(
            team["drivers"].as_array().unwrap().len(),
            before["drivers"].as_array().unwrap().len()
        );
        for (seat, was) in team["drivers"]
            .as_array()
            .unwrap()
            .iter()
            .zip(before["drivers"].as_array().unwrap())
        {
            let key = key_of(seat);
            assert_eq!(key, key_of(was), "the seats in their order");
            let shown = overlay
                .identities
                .iter()
                .find(|i| i.replaces == key)
                .map(|i| i.number)
                .or_else(|| {
                    overlay
                        .renames
                        .iter()
                        .find(|r| r.seat_driver == key)
                        .map(|r| r.number)
                });
            match shown {
                Some(n) => {
                    restated.insert(name.to_owned());
                    assert_eq!(seat["number"], n, "{key}");
                }
                None => assert_eq!(seat["number"], was["number"], "{key}"),
            }
        }
    }
    assert_eq!(
        restated.len(),
        package.len(),
        "only teams with a player or a renamed seat"
    );
    assert!(
        restated.contains("ferrari") && restated.contains("williams"),
        "the players' teams"
    );
    let anna = gamesync::identity_of(&l, "u-anna").unwrap();
    assert_eq!(anna.number, 42);
    // Without the loader, the database fallback comes along; the same league writes the same
    // bytes on every PC.
    let fallback = gamesync::files(&overlay, &ctx.get(false));
    let paths: Vec<&str> = fallback.iter().map(|(p, _)| p.as_str()).collect();
    assert!(
        paths.contains(&MARKET_DB) && paths.contains(&MARKET_LANG),
        "{paths:?}"
    );
    assert_eq!(fallback, gamesync::files(&overlay, &ctx.get(false)));
}

#[test]
fn market_files_load_through_the_registry_index() {
    use crate::careersync;
    use crate::stdlib::{data, deps, index::Index};
    use crate::store::Store;

    let _pin = crate::builtin::LoaderPin::none();
    let root = std::env::temp_dir().join(format!("f1mp-market-index-{}", std::process::id()));
    std::fs::remove_dir_all(&root).ok();
    let store = Store::new(root.join("data"));
    let season = "swofty.classic-seasons-2016";
    for id in ["swofty.classic-seasons", season] {
        store.install_addon(&Path::new(FIXTURE).join(id)).unwrap();
    }
    store
        .update_profile("career", |p| {
            p.set_enabled("swofty.classic-seasons", true)?;
            p.set_enabled(season, true)
        })
        .unwrap();
    let l = dynamic_2016();
    let ctx = careersync::context(&store, 2016).unwrap();
    let overlay = gamesync::overlay(&l, "c-1", "Anna & Ben", &ctx.get()).unwrap();
    careersync::prepare_as(&store, "career", &overlay, false).unwrap();

    let installed = store.addons().unwrap();
    let order: Vec<String> = store
        .profile("career")
        .unwrap()
        .addons
        .iter()
        .filter(|a| a.enabled)
        .map(|a| a.id.clone())
        .collect();
    let std = deps::read_all(&store, &installed);
    let placed = Index::plan("career", &order, &installed, &std, &store.addons_dir()).expect("an index");
    let entry = placed
        .index
        .addons
        .iter()
        .find(|a| a.id == gamesync::ADDON_ID)
        .expect("the overlay is in the index");
    let prefix = format!("addons/{}/", gamesync::ADDON_ID);
    assert_eq!(
        entry.data,
        [
            format!("{prefix}{IDENTITY_REGISTRY}"),
            format!("{prefix}{MARKET_REGISTRY}")
        ]
    );
    let report = deps::for_order("career", &order, &installed, &std);
    assert!(!report.stops(), "{:?}", report.stop_text());

    // Every layer of both files goes on a record the game or an addon before it has, and no
    // record gets two layers.
    let mut records: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for id in &order[..order.len() - 1] {
        let dir = store.addons_dir().join(id);
        for rel in data::files(&dir) {
            for r in data::records(id, &std::fs::read_to_string(dir.join(rel)).unwrap()) {
                records.entry(r.registry).or_default().insert(r.id);
            }
        }
    }
    let mut targets = BTreeSet::new();
    for file in [IDENTITY_REGISTRY, MARKET_REGISTRY] {
        let (_, src) = placed.files.iter().find(|(rel, _)| rel.ends_with(file)).unwrap();
        let layers: Value = serde_json::from_slice(&std::fs::read(src).unwrap()).unwrap();
        for (registry, by_target) in layers.as_object().unwrap() {
            for target in by_target.as_object().unwrap().keys() {
                assert!(
                    target.starts_with("game/") || records.get(registry).is_some_and(|r| r.contains(target)),
                    "{registry} {target} resolves to no record"
                );
                assert!(
                    targets.insert(format!("{registry} {target}")),
                    "{registry} {target} twice"
                );
            }
        }
    }
    assert_eq!(
        targets.len(),
        2 + l.season().renames.len() + 1,
        "two players, the renamed seats and the season's package"
    );
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn market_fallback_ops_build_into_the_game_database() {
    use crate::baseline::Identity;
    use crate::careersync;
    use crate::store::Store;
    use swofty_formats::gvdb::Gvdb;

    let _pin = crate::builtin::LoaderPin::none();
    let root = std::env::temp_dir().join(format!("f1mp-market-build-{}", std::process::id()));
    std::fs::remove_dir_all(&root).ok();
    let store = Store::new(root.join("data"));
    let season = "swofty.classic-seasons-2016";
    for id in ["swofty.classic-seasons", season] {
        store.install_addon(&Path::new(FIXTURE).join(id)).unwrap();
    }
    store
        .update_profile("career", |p| {
            p.set_enabled("swofty.classic-seasons", true)?;
            p.set_enabled(season, true)
        })
        .unwrap();
    let l = dynamic_2016();
    let renamed: Vec<&SeatRename> = l.season().renames.iter().collect();
    // The renamed seats, the players' and one nobody renames.
    let players = ["kimi_raikkonen", "felipe_massa"];
    let plain = box_seats(2016)
        .into_iter()
        .map(|(_, k)| k)
        .find(|k| !players.contains(k) && !renamed.iter().any(|r| r.seat_driver == *k))
        .unwrap();
    let keys: Vec<&str> = renamed
        .iter()
        .map(|r| r.seat_driver.as_str())
        .chain(players)
        .chain([plain])
        .collect();
    let dir = root.join("game");
    std::fs::create_dir_all(dir.join("game_package/gamedatabases")).unwrap();
    std::fs::write(dir.join(crate::GAME_EXE), "exe").unwrap();
    let rows = gamesync::identity::box_rows(2016, &keys);
    std::fs::write(dir.join(crate::build::GVDB), rows.to_bytes()).unwrap();
    let game = crate::install::Game {
        identity: Identity::detect(&dir, Some(1)).unwrap(),
        dir,
        user_reg: None,
    };
    let ctx = careersync::context(&store, 2016).unwrap();
    assert!(
        !ctx.loader_applies_rows,
        "no loader here: the fallback comes along"
    );
    let overlay = gamesync::overlay(&l, "c-1", "Anna & Ben", &ctx.get()).unwrap();
    careersync::prepare_as(&store, "career", &overlay, false).unwrap();
    let built = crate::build::build(&store, "career", &game).unwrap();
    assert!(built.order.iter().any(|id| id == gamesync::ADDON_ID));

    let db = Gvdb::parse(&std::fs::read(built.overlay.join(crate::build::GVDB)).unwrap()).unwrap();
    let row = |i: usize, f: &str| db.get_text(&format!("entrants.drivers.driver_{i}.{f}")).unwrap();
    for (i, key) in keys.iter().enumerate() {
        match renamed.iter().find(|r| r.seat_driver == *key) {
            Some(r) => {
                assert_eq!(row(i, "first_name"), format!("lng_f1mp_{key}_first"));
                assert_eq!(row(i, "initials"), r.code);
                assert_eq!(row(i, "car_number"), r.number.to_string());
                assert_eq!(
                    db.get_text(&format!("entrants.entrant_lists.entrants.entrant_{i}.car_number"))
                        .unwrap(),
                    r.number.to_string()
                );
                if let Some(n) = &r.nationality {
                    assert_eq!(&row(i, "nationality"), n);
                }
            }
            None if players.contains(key) => {
                assert_eq!(
                    row(i, "first_name"),
                    format!("lng_f1mp_{key}_first"),
                    "a player's seat"
                )
            }
            None => assert_eq!(
                row(i, "first_name"),
                format!("lng_{key}_first"),
                "a seat nobody renames"
            ),
        }
    }
    std::fs::remove_dir_all(&root).ok();
}
