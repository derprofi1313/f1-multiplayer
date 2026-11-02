//! Tests of the league's rnd (W2-6).

use super::rnd::{self, COST_CAP, Spend};
use super::*;

fn anna() -> Actor {
    Actor {
        id: "u-anna".into(),
        name: "Anna".into(),
        owner: true,
    }
}

fn ben() -> Actor {
    Actor {
        id: "u-ben".into(),
        name: "Ben".into(),
        owner: false,
    }
}

/// 2016 with Anna and Ben both at Ferrari (slot 2), with `seed`.
fn ferrari(seed: u64) -> League {
    let mut l = League::start(Setup::default(), &anna(), Some(7), seed).unwrap();
    apply(&mut l, &ben(), Action::Enter { number: Some(8) }).unwrap();
    for who in [anna(), ben()] {
        apply(
            &mut l,
            &who,
            Action::Seat {
                slot: 2,
                replaces: None,
            },
        )
        .unwrap();
    }
    apply(&mut l, &anna(), Action::Begin).unwrap();
    l
}

fn rp(l: &League, id: &str) -> u32 {
    l.player(id).unwrap().rp
}

fn node(l: &mut League, who: &Actor, part: Part, node: u8) -> Result<Vec<String>, LeagueError> {
    apply(l, who, Action::DevelopNode { part, node })
}

fn ids(nodes: &[&rnd::Node]) -> Vec<u8> {
    nodes.iter().map(|n| n.id).collect()
}

fn simulate(l: &mut League) {
    super::tests::agreed(l, &anna(), Action::Simulate { round: None });
}

#[test]
fn the_tree_has_two_branches_of_five_per_part() {
    for part in Part::ALL {
        let nodes = rnd::nodes(part);
        assert_eq!(nodes.len(), 10);
        for (i, n) in nodes.iter().enumerate() {
            let step = i % 5;
            assert_eq!((n.id, n.part, n.branch), (i as u8 + 1, part, (i / 5) as u8));
            assert_eq!(n.requires, (step > 0).then_some(n.id - 1));
            assert!((2..=4).contains(&n.rounds), "{n:?}");
            if n.branch == 0 {
                assert_eq!(n.cost, progress::upgrade_cost(step as u8));
                assert!((n.fail - 0.05).abs() < 1e-6);
            } else {
                assert!((n.fail - 0.20).abs() < 1e-6);
                assert!(n.gain > nodes[step].gain, "the alternative gains more: {n:?}");
                assert!(n.cost > nodes[step].cost);
            }
        }
        // The main branch gains what a linear level gained.
        if part != Part::Durability {
            assert!(nodes[..5].iter().all(|n| (n.gain - part.gain()).abs() < 1e-6));
        }
    }
}

#[test]
fn the_tree_unlocks_in_order() {
    let mut l = ferrari(42);
    l.players[0].rp = 20_000;
    assert_eq!(ids(&rnd::available(l.season(), 2, Part::Aero)), [1, 6]);
    let err = node(&mut l, &anna(), Part::Aero, 2).unwrap_err();
    assert_eq!(err.reason, "node_locked");
    assert!(err.message.contains("needs"), "{}", err.message);
    let err = node(&mut l, &anna(), Part::Aero, 11).unwrap_err();
    assert_eq!(err.reason, "node_locked");
    assert_eq!(rp(&l, "u-anna"), 20_000, "a refusal costs nothing");

    node(&mut l, &anna(), Part::Aero, 1).unwrap();
    assert_eq!(ids(&rnd::available(l.season(), 2, Part::Aero)), [2, 6]);
    let err = node(&mut l, &anna(), Part::Aero, 1).unwrap_err();
    assert_eq!(err.reason, "node_locked", "a node is ordered once");
    // The next node can be ordered while the one before is still built.
    node(&mut l, &anna(), Part::Aero, 2).unwrap();
    node(&mut l, &anna(), Part::Aero, 6).unwrap();
    assert_eq!(ids(&rnd::available(l.season(), 2, Part::Aero)), [3, 7]);
    // Other parts and other teams have their own trees.
    assert_eq!(ids(&rnd::available(l.season(), 2, Part::Chassis)), [1, 6]);
    assert_eq!(ids(&rnd::available(l.season(), 1, Part::Aero)), [1, 6]);

    let season = l.season();
    let u = &season.upgrades[2];
    assert_eq!(
        (u.node, u.level, u.cost, u.ready),
        (Some(6), 3, rnd::nodes(Part::Aero)[5].cost, 1 + 3)
    );
    assert!(
        l.news
            .iter()
            .any(|n| n.text.contains("High-downforce front wing"))
    );
}

#[test]
fn the_cost_cap_stops_spending() {
    let mut l = ferrari(42);
    l.players[0].rp = 4_000;
    l.players[1].rp = 10_000;
    // Anna and Ben pay for the same car: 800 + 1050 + 1300 = 3150 from Anna...
    node(&mut l, &anna(), Part::Aero, 1).unwrap();
    node(&mut l, &anna(), Part::Aero, 2).unwrap();
    node(&mut l, &anna(), Part::Aero, 3).unwrap();
    // ... and 800 + 1050 + 1300 + 1550 (powertrain) + 800 (chassis) = 5500 from Ben make 8650.
    for n in 1..=4 {
        node(&mut l, &ben(), Part::Powertrain, n).unwrap();
    }
    node(&mut l, &ben(), Part::Chassis, 1).unwrap();
    assert_eq!(rnd::spent(l.season(), 2), 8_650);
    assert_eq!(l.season().spent, [Spend { slot: 2, rp: 8_650 }]);
    // Points come first: Anna has 850 left for a node of 1550.
    let err = node(&mut l, &anna(), Part::Aero, 4).unwrap_err();
    assert_eq!(err.reason, "no_points");
    // Ben has the points, but 1550 more would take the team over the cap.
    let err = node(&mut l, &ben(), Part::Aero, 4).unwrap_err();
    assert_eq!(err.reason, "cost_cap");
    assert!(err.message.contains("8650 of 10000"), "{}", err.message);
    assert_eq!(rp(&l, "u-ben"), 10_000 - 5_500);
    // 800 more fits.
    node(&mut l, &ben(), Part::Durability, 1).unwrap();
    assert_eq!(rnd::spent(l.season(), 2), 9_450);
    let err = apply(
        &mut l,
        &ben(),
        Action::Develop {
            part: Part::Durability,
        },
    )
    .unwrap_err();
    assert_eq!(err.reason, "cost_cap");
    assert!(rnd::spent(l.season(), 2) <= COST_CAP);

    // A part ordered after the round started goes with its undo, and the spending with it.
    let mut fresh = ferrari(42);
    fresh.players[0].rp = 5_000;
    node(&mut fresh, &anna(), Part::Aero, 1).unwrap();
    apply(&mut fresh, &anna(), Action::Practice { round: None }).unwrap();
    node(&mut fresh, &anna(), Part::Chassis, 1).unwrap();
    assert_eq!(fresh.season().spent, [Spend { slot: 2, rp: 1_600 }]);
    super::tests::agreed(&mut fresh, &anna(), Action::Undo);
    assert_eq!(fresh.season().spent, [Spend { slot: 2, rp: 800 }]);

    // The cap is per season.
    while l.phase == Phase::Racing {
        simulate(&mut l);
    }
    apply(&mut l, &anna(), Action::NextSeason).unwrap();
    assert_eq!(rnd::spent(l.season(), 2), 0);
    assert!(l.season().spent.is_empty());
}

#[test]
fn a_part_too_late_for_the_season_is_refused_after_the_cap() {
    let mut l = ferrari(42);
    l.players[0].rp = 100_000;
    while l.season().next_round().unwrap().round < 19 {
        simulate(&mut l);
    }
    // Round 19 of 21: a main node (2 rounds) is ready for round 21, an alternative one (3) isn't.
    node(&mut l, &anna(), Part::Aero, 1).unwrap();
    let err = node(&mut l, &anna(), Part::Aero, 6).unwrap_err();
    assert_eq!(err.reason, "too_late");
    assert!(err.message.contains("3 rounds"), "{}", err.message);
}

/// A seed with which `part`'s node `id`, ordered by Anna for Ferrari before round 1, fails (or
/// doesn't).
fn seed_where(part: Part, id: u8, fail: bool) -> u64 {
    let n = rnd::node(part, id).unwrap();
    let u = progress::Upgrade {
        slot: 2,
        part,
        level: 1,
        by: "u-anna".into(),
        cost: n.cost,
        round: 1,
        started: false,
        ready: 1 + n.rounds,
        node: Some(id),
        failed: false,
    };
    (0..10_000)
        .find(|&s| rnd::fails(s, 2016, &u) == fail)
        .expect("a seed")
}

#[test]
fn failures_are_seeded_and_about_as_likely_as_the_node_says() {
    let n = rnd::node(Part::Aero, 6).unwrap();
    let mut u = progress::Upgrade {
        slot: 2,
        part: Part::Aero,
        level: 1,
        by: "u-anna".into(),
        cost: n.cost,
        round: 1,
        started: false,
        ready: 4,
        node: Some(6),
        failed: false,
    };
    let alt = (0..4_000).filter(|&s| rnd::fails(s, 2016, &u)).count();
    assert!((600..1000).contains(&alt), "{alt} of 4000 at 20%");
    u.node = Some(1);
    let main = (0..4_000).filter(|&s| rnd::fails(s, 2016, &u)).count();
    assert!((120..280).contains(&main), "{main} of 4000 at 5%");
    assert_eq!(rnd::fails(7, 2016, &u), rnd::fails(7, 2016, &u.clone()));
    // Legacy levels never fail.
    u.node = None;
    assert!((0..1_000).all(|s| !rnd::fails(s, 2016, &u)));
}

#[test]
fn a_failed_part_refunds_half() {
    let mut l = ferrari(seed_where(Part::Aero, 6, true));
    l.players[0].rp = 5_000;
    let cost = rnd::node(Part::Aero, 6).unwrap().cost;
    let before = l.season().pace_at(2, 4);
    node(&mut l, &anna(), Part::Aero, 6).unwrap();
    assert_eq!(rp(&l, "u-anna"), 5_000 - cost);
    simulate(&mut l);
    simulate(&mut l);
    assert!(!l.season().upgrades[0].failed, "not arrived yet");
    let points = rp(&l, "u-anna");
    // Round 3 done: the part arrives for round 4 and fails.
    simulate(&mut l);
    let season = l.season();
    assert!(season.upgrades[0].failed);
    assert_eq!(season.upgrade_gain(2, 4), 0.0);
    assert_eq!(season.pace_at(2, 4), before);
    assert_eq!(rnd::spent(season, 2), cost - cost / 2);
    assert_eq!(
        season.spent,
        [Spend {
            slot: 2,
            rp: cost - cost / 2
        }]
    );
    let gained_in_round_3 = rp(&l, "u-anna") - points - cost / 2;
    assert!(
        l.news
            .iter()
            .any(|n| n.text.contains("fails its tests")
                && n.text.contains(&format!("{} of its {cost}", cost / 2))),
        "{:?}",
        l.news.last()
    );
    // The node can be ordered again.
    assert_eq!(ids(&rnd::available(l.season(), 2, Part::Aero)), [1, 6]);

    // Taking round 3 back: the part hasn't arrived, the points it gave back go too; counting the
    // round again fails it the same way.
    super::tests::agreed(&mut l, &anna(), Action::Undo);
    assert!(!l.season().upgrades[0].failed);
    assert_eq!(rp(&l, "u-anna"), points);
    assert_eq!(rnd::spent(l.season(), 2), cost);
    simulate(&mut l);
    assert!(l.season().upgrades[0].failed);
    assert_eq!(rp(&l, "u-anna"), points + gained_in_round_3 + cost / 2);
}

#[test]
fn a_part_that_passes_gains_from_its_round() {
    let mut l = ferrari(seed_where(Part::Aero, 6, false));
    l.players[0].rp = 5_000;
    let before = l.season().pace_at(2, 4);
    node(&mut l, &anna(), Part::Aero, 6).unwrap();
    for _ in 0..3 {
        simulate(&mut l);
    }
    let season = l.season();
    assert!(!season.upgrades[0].failed);
    let gain = rnd::node(Part::Aero, 6).unwrap().gain;
    assert!((before - season.pace_at(2, 4) - gain).abs() < 1e-5);
    assert_eq!(rnd::spent(season, 2), rnd::node(Part::Aero, 6).unwrap().cost);
    assert!(l.news.iter().any(|n| n.text.contains("On the car for the")));
}

#[test]
fn legacy_upgrades_keep_their_gain() {
    // A career saved before the tree: its upgrades are linear levels.
    let l: League = serde_json::from_str(include_str!("fixtures/league-v1.json")).unwrap();
    for season in &l.seasons {
        for slot in season.teams.iter().map(|t| t.slot) {
            let linear: f32 = season
                .upgrades
                .iter()
                .filter(|u| u.slot == slot)
                .map(|u| u.part.gain())
                .sum();
            assert!((season.upgrade_gain(slot, u8::MAX) - linear).abs() < 1e-6);
        }
    }

    let mut l = ferrari(42);
    let legacy = |part, level| progress::Upgrade {
        slot: 2,
        part,
        level,
        by: "u-anna".into(),
        cost: progress::upgrade_cost(level - 1),
        round: 1,
        started: false,
        ready: 3,
        node: None,
        failed: false,
    };
    l.season_mut().upgrades = vec![
        legacy(Part::Aero, 1),
        legacy(Part::Aero, 2),
        legacy(Part::Durability, 1),
    ];
    let season = l.season();
    assert!((season.upgrade_gain(2, 3) - 2.0 * Part::Aero.gain()).abs() < 1e-6);
    assert_eq!(season.upgrade_gain(2, 2), 0.0);
    assert!((season.breakdowns(2, 3) - 0.88).abs() < 1e-6);
    // The legacy levels are the main branch's first steps, and count against the cap.
    assert_eq!(ids(&rnd::available(season, 2, Part::Aero)), [3, 6]);
    assert_eq!(rnd::spent(season, 2), 800 + 1050 + 800);
    // They arrive without a roll.
    for _ in 0..3 {
        simulate(&mut l);
    }
    assert!(l.season().upgrades.iter().all(|u| !u.failed));
    assert!(!l.news.iter().any(|n| n.text.contains("On the car for the")));
}

#[test]
fn car_profile_sums_parts() {
    let mut l = ferrari(42);
    l.players[0].rp = 20_000;
    node(&mut l, &anna(), Part::Aero, 1).unwrap();
    node(&mut l, &anna(), Part::Aero, 6).unwrap();
    node(&mut l, &anna(), Part::Chassis, 1).unwrap();
    node(&mut l, &anna(), Part::Durability, 6).unwrap();
    let season = l.season();
    // Before round 3 nothing is on the car; the alternative nodes (3 rounds) come a round later.
    assert_eq!(rnd::car_profile(season, 2, 2), rnd::CarProfile::default());
    let at3 = rnd::car_profile(season, 2, 3);
    assert!((at3.aero - 0.07).abs() < 1e-6 && (at3.chassis - 0.05).abs() < 1e-6);
    assert_eq!((at3.powertrain, at3.durability), (0.0, 0.0));
    let p = rnd::car_profile(season, 2, 4);
    assert!((p.aero - 0.19).abs() < 1e-6, "{p:?}");
    assert!((p.chassis - 0.05).abs() < 1e-6 && p.powertrain == 0.0);
    assert!((p.durability - 0.20).abs() < 1e-6);
    // The lap time is the sum of the parts; durability makes the car break down less instead.
    assert!((season.upgrade_gain(2, 4) - (p.aero + p.chassis + p.powertrain)).abs() < 1e-6);
    assert!((season.breakdowns(2, 4) - 0.8).abs() < 1e-6);
    // Another team's car is undeveloped.
    assert_eq!(rnd::car_profile(season, 1, u8::MAX), rnd::CarProfile::default());
    // A failed part gains nothing.
    let mut failed = season.clone();
    failed.upgrades[1].failed = true;
    assert!((rnd::car_profile(&failed, 2, 4).aero - 0.07).abs() < 1e-6);
}

#[test]
fn develop_picks_the_next_main_node() {
    let mut l = ferrari(42);
    l.players[0].rp = 20_000;
    apply(&mut l, &anna(), Action::Develop { part: Part::Aero }).unwrap();
    node(&mut l, &anna(), Part::Aero, 6).unwrap();
    apply(&mut l, &anna(), Action::Develop { part: Part::Aero }).unwrap();
    let ordered: Vec<Option<u8>> = l.season().upgrades.iter().map(|u| u.node).collect();
    assert_eq!(ordered, [Some(1), Some(6), Some(2)]);
    assert_eq!(l.season().upgrades[2].cost, progress::upgrade_cost(1));
    assert!(
        l.news
            .iter()
            .any(|n| n.text.contains("develop Aerodynamics level 3 (Floor edge)"))
    );

    // With the main branch complete (here from five older levels), Develop has nothing left.
    let mut l = ferrari(42);
    l.players[0].rp = 20_000;
    for level in 1..=5 {
        l.season_mut().upgrades.push(progress::Upgrade {
            slot: 2,
            part: Part::Chassis,
            level,
            by: "u-anna".into(),
            cost: 0,
            round: 1,
            started: false,
            ready: 3,
            node: None,
            failed: false,
        });
    }
    assert_eq!(rnd::next_main(l.season(), 2, Part::Chassis), None);
    let err = apply(&mut l, &anna(), Action::Develop { part: Part::Chassis }).unwrap_err();
    assert_eq!(err.reason, "max_level");
    node(&mut l, &anna(), Part::Chassis, 6).unwrap();
}
