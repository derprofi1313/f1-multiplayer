//! Two players (and a third who is removed) play a league career on a real host server, as the
//! launchers do it over HTTP: practice, qualifying in parts with a duplicate from the other PC, a
//! sprint weekend reported by both PCs, votes to simulate and to take a round back, a mid-season
//! approach, the season's end with offers and a negotiation, and the next season's game overlay
//! prepared on both PCs from the shared league document (the same `hash8`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::Value;
use swofty_core::careersync;
use swofty_core::league::{
    self, Action, ApproachStatus, League, OfferStatus, Options, Phase, RaceReport, ReportCar, RoundState,
    Setup, Terms,
};
use swofty_core::multiplayer::host::{Config, HostServer};
use swofty_core::multiplayer::rooms::{Api, ApiError, LeagueState};
use swofty_core::store::Store;

const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/classic-seasons-mini");
const PROFILE: &str = "career";

/// A launcher: its API session, its data folder and its player id.
struct Pc {
    api: Api,
    id: String,
    store: Store,
}

impl Pc {
    fn act(&self, career: &str, action: Action) -> Result<LeagueState, ApiError> {
        self.api.league_action(career, &action)
    }

    fn ok(&self, career: &str, action: Action) -> LeagueState {
        let what = format!("{action:?}");
        self.act(career, action)
            .unwrap_or_else(|e| panic!("{what}: {} {}", e.reason, e.message))
    }

    fn league(&self, career: &str) -> League {
        self.api.league(career).unwrap().league.expect("a league")
    }

    fn report(&self, career: &str, report: RaceReport) -> Result<LeagueState, ApiError> {
        self.act(
            career,
            Action::Report {
                round: None,
                kind: None,
                force: false,
                report,
            },
        )
    }
}

/// The game's classification of the next round's session `session_type` as `reporter`'s PC
/// sees it: the reporter first, the other players next, the AI cars in grid order.
fn report_of(l: &League, session: u64, session_type: u8, reporter: &str) -> RaceReport {
    let season = l.season();
    let track = season.next_round().and_then(|r| r.track);
    let mut cars: Vec<ReportCar> = season
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let mine = e.player.as_deref() == Some(reporter);
            ReportCar {
                name: if mine { "Player".into() } else { e.name.to_uppercase() },
                number: e.number,
                team: e.slot,
                ai: e.player.is_none(),
                mine,
                position: 0,
                grid: i as u8 + 1,
                laps: 10,
                status: 3,
                // Every session has its own lap times (an identical classification is the same
                // session, counted already).
                best_lap_ms: 90_000 + i as u32 * 10 + (session % 997) as u32,
                time_ms: 900_000 + i as u64 * 1000,
                penalty_s: 0,
            }
        })
        .collect();
    cars.sort_by_key(|c| (!c.mine, c.ai, c.grid));
    for (i, c) in cars.iter_mut().enumerate() {
        c.position = i as u8 + 1;
    }
    RaceReport {
        session,
        track,
        session_type,
        cars,
        ..RaceReport::default()
    }
}

/// The same classification from another PC: only which car is "mine" differs.
fn seen_by(report: &RaceReport, l: &League, player: &str, session: u64) -> RaceReport {
    let number = l.season().player_entry(player).unwrap().number;
    let mut r = report.clone();
    r.session = session;
    for c in &mut r.cars {
        c.mine = c.number == number;
    }
    r
}

fn next(l: &League) -> u8 {
    l.season().next_round().expect("a next round").round
}

fn state(l: &League, round: u8) -> RoundState {
    l.season().round(round).unwrap().state
}

/// A data folder with the Classic Seasons core installed and on in [`PROFILE`].
fn data_folder(root: &Path) -> Store {
    let store = Store::new(root.join("data"));
    store
        .install_addon(&Path::new(FIXTURE).join(careersync::CLASSIC_SEASONS_CORE))
        .unwrap();
    store
        .update_profile(PROFILE, |p| {
            p.set_enabled(careersync::CLASSIC_SEASONS_CORE, true)?;
            Ok(())
        })
        .unwrap();
    store
}

/// Every file of the installed overlay addon, by relative path.
fn overlay_files(store: &Store) -> BTreeMap<String, Vec<u8>> {
    fn walk(dir: &Path, base: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for e in std::fs::read_dir(dir).unwrap() {
            let path = e.unwrap().path();
            if path.is_dir() {
                walk(&path, base, out);
            } else {
                let rel = path.strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/");
                out.insert(rel, std::fs::read(&path).unwrap());
            }
        }
    }
    let dir = store.addons_dir().join(league::gamesync::ADDON_ID);
    let mut out = BTreeMap::new();
    walk(&dir, &dir, &mut out);
    out
}

/// The game's weekend structures: a sprint weekend, and a plain one.
const SPRINT_WEEKEND: [u8; 9] = [1, 10, 11, 12, 15, 5, 6, 7, 16];
const PLAIN_WEEKEND: [u8; 7] = [1, 2, 3, 5, 6, 7, 15];

/// Anna reports the next round's qualifying, sprint (on a sprint weekend) and race from her PC,
/// winning them.
fn drive(anna: &Pc, career: &str, uid: &mut u64) {
    let l = anna.league(career);
    let n = next(&l);
    let sprint = l.season().round(n).unwrap().sprint;
    let weekend = if sprint { SPRINT_WEEKEND.to_vec() } else { PLAIN_WEEKEND.to_vec() };
    let mut sessions = vec![7];
    if sprint {
        sessions.push(15);
    }
    sessions.push(if sprint { 16 } else { 15 });
    for session_type in sessions {
        let l = anna.league(career);
        *uid += 1;
        let mut r = report_of(&l, *uid, session_type, &anna.id);
        r.weekend = weekend.clone();
        anna.report(career, r).unwrap();
    }
    let l = anna.league(career);
    assert_eq!(state(&l, n), RoundState::Raced, "round {n}");
}

/// `proposer` asks to simulate the next round and `other` agrees.
fn simulate_by_vote(proposer: &Pc, other: &Pc, career: &str) {
    let n = next(&proposer.league(career));
    let asked = proposer.ok(career, Action::Simulate { round: None });
    assert_eq!(asked.warnings.len(), 1, "{:?}", asked.warnings);
    assert_eq!(state(asked.league.as_ref().unwrap(), n), RoundState::Pending);
    let done = other.ok(career, Action::Vote { yes: true, force: false });
    let l = done.league.unwrap();
    assert_eq!(state(&l, n), RoundState::Simulated, "round {n}");
    assert!(l.vote.is_none());
}

#[test]
fn two_players_play_a_season_into_the_next() {
    let root = std::env::temp_dir().join(format!("f1mp-league-two-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let server = HostServer::start(Config {
        dir: root.join("server"),
        port: 0,
        name: "Test host".into(),
        password: None,
        public: false,
        discovery: None,
        fresh_code: false,
    })
    .unwrap();
    let code = server.code_for("127.0.0.1");
    let pc = |who: &str| -> Pc {
        let dir: PathBuf = root.join(who);
        let api = Api::for_session(&swofty_core::mpauth::sign_in_host(&dir, &code, who, None).unwrap());
        let id = api.me().unwrap().id;
        Pc {
            api,
            id,
            store: data_folder(&dir),
        }
    };
    let (anna, ben, cleo) = (pc("Anna"), pc("Ben"), pc("Cleo"));

    // Anna starts a 2024 career; Ben and Cleo join. Anna drives for the slowest team, Ben for the
    // next slowest, Cleo for the one before.
    let career = anna
        .api
        .create_career("Liga", league::MODE, Value::Null)
        .unwrap();
    let id = career.id.as_str();
    let setup = Setup {
        first_year: 2024,
        ..Setup::default()
    };
    anna.ok(
        id,
        Action::Start {
            setup,
            number: Some(7),
        },
    );
    for (who, number) in [(&ben, 8), (&cleo, 9)] {
        who.api.join_career(&career.code).unwrap();
        who.ok(id, Action::Enter { number: Some(number) });
    }
    let mut teams = anna.league(id).season().teams.clone();
    teams.sort_by(|a, b| b.pace.total_cmp(&a.pace));
    for (who, team) in [&anna, &ben, &cleo].into_iter().zip(&teams) {
        who.ok(
            id,
            Action::Seat {
                slot: team.slot,
                replaces: None,
            },
        );
    }
    anna.ok(id, Action::Begin);
    let options = Options {
        game_identity: true,
        game_performance: true,
        ..Options::default()
    };
    let set = anna.ok(id, Action::SetOptions { options });
    assert_eq!(
        set.league.unwrap().options.rules.equal_performance,
        Some(false),
        "the league's pace in the game agrees equal performance off"
    );
    let mut uid = 0x1000u64;

    // Round 1. Practice from both PCs, each their own session.
    let l = anna.league(id);
    assert_eq!(next(&l), 1);
    anna.report(id, report_of(&l, 0x101, 1, &anna.id)).unwrap();
    ben.report(id, report_of(&l, 0x102, 1, &ben.id)).unwrap();
    let l = anna.league(id);
    let practice: Vec<&str> = l.season().round(1).unwrap().practice.iter().map(|p| p.player.as_str()).collect();
    assert_eq!(practice, [anna.id.as_str(), ben.id.as_str()]);
    // Q1, then Q3 from Anna; Ben's PC sends Q3 again under another uid.
    anna.report(id, report_of(&l, 0x103, 5, &anna.id)).unwrap();
    let mut q3 = report_of(&l, 0x104, 7, &anna.id);
    q3.at = 1_800_000_000;
    let updated = anna.report(id, q3.clone()).unwrap();
    let l = updated.league.unwrap();
    let round = l.season().round(1).unwrap();
    assert!(!round.quali_result.is_empty());
    assert_eq!(round.counted.iter().map(|c| c.session_type).collect::<Vec<_>>(), [1, 1, 5, 7]);
    assert!(l.news.iter().any(|n| n.text.contains("Qualifying updated")));
    // The same classification: counted already, nothing changes.
    let same = ben.report(id, seen_by(&q3, &l, &ben.id, 0x204)).unwrap();
    assert!(same.warnings.iter().any(|w| w.contains("counted already")), "{:?}", same.warnings);
    let counted = same.league.unwrap().season().round(1).unwrap().counted.len();
    assert_eq!(counted, 4);
    // Moments later, a classification that differs a little (Ben's PC saw one lap time 1 ms
    // apart): refused as a duplicate, which the launcher can still count with force.
    let mut dup = seen_by(&q3, &l, &ben.id, 0x205);
    dup.at += 30;
    dup.cars[5].best_lap_ms += 1;
    let refused = ben.report(id, dup).unwrap_err();
    assert_eq!(refused.reason, "duplicate", "{}", refused.message);
    assert!(refused.message.contains("Anna"), "{}", refused.message);
    // The race from Anna, then from Ben's PC (the same LAN session): counted once.
    let race = report_of(&l, 0x105, 15, &anna.id);
    anna.report(id, race.clone()).unwrap();
    let again = ben.report(id, seen_by(&race, &anna.league(id), &ben.id, 0x105)).unwrap();
    assert!(again.warnings.iter().any(|w| w.contains("counted already")), "{:?}", again.warnings);
    let l = anna.league(id);
    assert_eq!(state(&l, 1), RoundState::Raced);
    assert_eq!(l.season().round(1).unwrap().race_result[0].entry, l.season().player_entry(&anna.id).unwrap().id);

    // Round 2: Anna asks to simulate; it runs once Ben and Cleo agree.
    let asked = anna.ok(id, Action::Simulate { round: None });
    assert!(asked.warnings[0].contains("Ben") && asked.warnings[0].contains("Cleo"), "{:?}", asked.warnings);
    ben.ok(id, Action::Vote { yes: true, force: false });
    assert_eq!(state(&anna.league(id), 2), RoundState::Pending, "Cleo hasn't agreed");
    cleo.ok(id, Action::Vote { yes: true, force: false });
    assert_eq!(state(&anna.league(id), 2), RoundState::Simulated);

    // Cleo proposes to leave out round 3, then Anna removes her: the proposal goes with her and
    // her seat goes to the game's AI.
    let n = next(&anna.league(id));
    cleo.ok(id, Action::Skip { round: None });
    assert!(anna.league(id).vote.is_some());
    let removed = anna.api.remove_member(id, &cleo.id).unwrap();
    let l = removed.league.unwrap();
    assert!(l.vote.is_none(), "{:?}", l.vote);
    assert!(l.player(&cleo.id).unwrap().left);
    assert_eq!(state(&l, n), RoundState::Pending);
    assert_eq!(l.active_players().len(), 2);
    assert_eq!(cleo.api.league(id).unwrap_err().reason, "not_found");

    // Anna drives round 3; it is taken back once Ben agrees, and driven again.
    drive(&anna, id, &mut uid);
    let asked = anna.ok(id, Action::Undo);
    assert_eq!(asked.warnings, ["Waiting for Ben to agree."]);
    assert_eq!(state(&anna.league(id), n), RoundState::Raced);
    ben.ok(id, Action::Vote { yes: true, force: false });
    let l = anna.league(id);
    assert_eq!(state(&l, n), RoundState::Pending);
    assert!(l.season().round(n).unwrap().counted.is_empty());
    drive(&anna, id, &mut uid);

    // On to the first sprint weekend; its sprint and race come from both PCs.
    let sprint_round = anna
        .league(id)
        .season()
        .rounds
        .iter()
        .find(|r| r.sprint && r.track.is_some() && r.state == RoundState::Pending)
        .expect("a sprint weekend")
        .round;
    while next(&anna.league(id)) < sprint_round {
        drive(&anna, id, &mut uid);
    }
    let l = anna.league(id);
    let mut sprint = report_of(&l, 0x301, 15, &anna.id);
    sprint.weekend = SPRINT_WEEKEND.to_vec();
    anna.report(id, sprint.clone()).unwrap();
    let again = ben.report(id, seen_by(&sprint, &l, &ben.id, 0x301)).unwrap();
    assert!(again.warnings.iter().any(|w| w.contains("counted already")), "{:?}", again.warnings);
    let l = anna.league(id);
    let round = l.season().round(sprint_round).unwrap();
    assert!(!round.sprint_result.is_empty() && round.race_result.is_empty());
    let mut race = report_of(&l, 0x302, 16, &anna.id);
    race.weekend = sprint.weekend.clone();
    anna.report(id, race.clone()).unwrap();
    let again = ben.report(id, seen_by(&race, &l, &ben.id, 0x302)).unwrap();
    assert!(again.warnings.iter().any(|w| w.contains("counted already")), "{:?}", again.warnings);
    assert_eq!(state(&anna.league(id), sprint_round), RoundState::Raced);

    // Anna leads the championship at the season's first approach point: a faster team wants
    // her; she meets them and signs early.
    let rounds = anna.league(id).season().rounds.len();
    let first_approach = league::market::approach_rounds(rounds)[0];
    while next(&anna.league(id)) <= first_approach {
        drive(&anna, id, &mut uid);
    }
    let l = anna.league(id);
    let approach = league::market::approaches_of(&l, &anna.id)
        .first()
        .map(|a| (*a).clone())
        .expect("an approach for the championship leader");
    assert_eq!(approach.status, ApproachStatus::Interest);
    assert!(league::inbox::unread(&l, &anna.id).iter().any(|m| m.text.contains("would like to meet you")));
    let met = anna.ok(id, Action::Meet { slot: approach.slot });
    assert!(met.warnings[0].contains("offers"), "{:?}", met.warnings);
    anna.ok(
        id,
        Action::AcceptApproach {
            slot: approach.slot,
            replaces: None,
        },
    );
    let l = anna.league(id);
    assert_eq!(l.player(&anna.id).unwrap().signed.as_ref().map(|s| s.slot), Some(approach.slot));

    // The rest of the season, each round simulated by vote, Ben and Anna taking turns to ask.
    let mut turn = false;
    while anna.league(id).phase == Phase::Racing {
        let (asker, other) = if turn { (&ben, &anna) } else { (&anna, &ben) };
        simulate_by_vote(asker, other, id);
        turn = !turn;
    }

    // The season's end: Anna's early signing is her only offer; Ben negotiates and signs.
    let l = anna.league(id);
    assert_eq!(l.phase, Phase::SeasonEnd);
    let mine: Vec<_> = l.season().offers.iter().filter(|o| o.player == anna.id).collect();
    assert!(mine.iter().all(|o| o.slot == approach.slot), "{mine:?}");
    let offer = l
        .season()
        .offers
        .iter()
        .find(|o| o.player == ben.id)
        .expect("an offer for Ben")
        .clone();
    let asked = Terms {
        salary_k: offer.salary_k * 3,
        years: 2,
        number_one: false,
    };
    ben.ok(
        id,
        Action::Negotiate {
            slot: offer.slot,
            terms: asked,
        },
    );
    let l = anna.league(id);
    let talked = l
        .season()
        .offers
        .iter()
        .find(|o| o.player == ben.id && o.slot == offer.slot)
        .unwrap();
    assert_ne!(talked.status, OfferStatus::Open, "{talked:?}");
    assert!(talked.attempts < offer.attempts || talked.status == OfferStatus::Agreed, "{talked:?}");
    let sign = |who: &Pc, slot: u8| {
        who.ok(
            id,
            Action::Sign {
                slot,
                replaces: None,
                focus: None,
            },
        )
    };
    sign(&ben, offer.slot);
    if anna.league(id).phase == Phase::SeasonEnd {
        sign(&anna, approach.slot);
    }

    // The 2025 season: both PCs prepare the game overlay from the shared document and get the
    // same addon, byte for byte.
    let l = anna.league(id);
    assert_eq!((l.phase, l.year()), (Phase::Racing, 2025));
    assert_eq!(l.season().player_entry(&anna.id).unwrap().slot, approach.slot);
    let mut hashes = Vec::new();
    let mut contents = Vec::new();
    for who in [&anna, &ben] {
        let l = who.league(id);
        let ctx = careersync::context(&who.store, l.year()).unwrap();
        let overlay = league::gamesync::overlay(&l, id, &career.name, &ctx.get()).expect("an overlay");
        assert_eq!((overlay.year, overlay.round), (2025, 1));
        assert_eq!(overlay.identities.len(), 2, "{:?}", overlay.identities);
        assert!(!overlay.cars.is_empty());
        let state = careersync::prepare_as(&who.store, PROFILE, &overlay, false).unwrap();
        assert!(state.enabled);
        let prepared = state.prepared.expect("prepared");
        assert_eq!((prepared.career.as_str(), prepared.year, prepared.round), (id, 2025, 1));
        hashes.push(prepared.hash);
        contents.push(overlay_files(&who.store));
    }
    assert_eq!(hashes[0].len(), 8);
    assert_eq!(hashes[0], hashes[1]);
    assert_eq!(contents[0], contents[1]);

    drop(server);
    let _ = std::fs::remove_dir_all(&root);
}
