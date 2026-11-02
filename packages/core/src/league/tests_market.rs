//! Tests of the league's market (W2-3).

use super::market::{MAIL_MEET, approach_rounds, approaches_of};
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

/// A career from 2016 with Anna, rated 95, at Sauber (the slowest team of 2017), and with
/// `with_ben` Ben, rated 70, beside her.
fn sauber(seed: u64, with_ben: bool) -> League {
    let setup = Setup {
        first_year: 2016,
        last_year: 2018,
        lineup_changes: false,
        ..Setup::default()
    };
    let mut l = League::start(setup, &anna(), Some(7), seed).unwrap();
    let seat = Action::Seat {
        slot: 9,
        replaces: None,
    };
    apply(&mut l, &anna(), seat.clone()).unwrap();
    if with_ben {
        apply(&mut l, &ben(), Action::Enter { number: Some(8) }).unwrap();
        apply(&mut l, &ben(), seat).unwrap();
    }
    apply(&mut l, &anna(), Action::Begin).unwrap();
    rate(&mut l, "u-anna", 95.0);
    if with_ben {
        rate(&mut l, "u-ben", 70.0);
    }
    l
}

fn rate(l: &mut League, id: &str, rating: f32) {
    l.player_mut(id).unwrap().attributes = Some(Attributes::from_rating(rating));
    l.sync_ratings();
}

fn player<'a>(l: &'a League, id: &str) -> &'a Player {
    l.player(id).unwrap()
}

/// The next round is left out, as everyone agreed (the vote is W2-1's).
fn next(l: &mut League) {
    l.skip(None, false).unwrap();
    l.settle();
}

/// Play the season out.
fn finish(l: &mut League) {
    while l.phase == Phase::Racing {
        next(l);
    }
}

/// `slot`'s interest in Anna, after round 8.
fn interest(l: &mut League, slot: u8) {
    l.season_mut().approaches.push(Approach {
        player: "u-anna".into(),
        slot,
        round: 8,
        status: ApproachStatus::Interest,
        offer: None,
        leaked: false,
        replaces: None,
        old_until: None,
    });
}

fn approach(l: &League, slot: u8) -> Approach {
    approaches_of(l, "u-anna")
        .into_iter()
        .find(|a| a.slot == slot)
        .unwrap()
        .clone()
}

/// The market's unread mails to `id` (other packages send mails too).
fn meet_mails<'a>(l: &'a League, id: &str) -> Vec<&'a News> {
    inbox::unread(l, id)
        .into_iter()
        .filter(|n| n.template.as_deref() == Some(MAIL_MEET))
        .collect()
}

fn reason(r: Result<Vec<String>, LeagueError>) -> &'static str {
    r.unwrap_err().reason
}

fn ask(salary_k: u32, years: u8, number_one: bool) -> Terms {
    Terms {
        salary_k,
        years,
        number_one,
    }
}

fn negotiate(l: &mut League, slot: u8, terms: Terms) -> Result<Vec<String>, LeagueError> {
    apply(l, &anna(), Action::Negotiate { slot, terms })
}

fn offer(l: &League, slot: u8) -> Option<Offer> {
    l.offers("u-anna").into_iter().find(|o| o.slot == slot).cloned()
}

#[test]
fn approaches_come_at_set_points_of_the_season_for_good_drivers() {
    let mut l = sauber(42, true);
    l.player_mut("u-anna").unwrap().recognition = 90.0;
    let points = approach_rounds(l.season().rounds.len());
    assert_eq!(points, [8, 13, 17]);
    for _ in 0..7 {
        next(&mut l);
    }
    assert!(
        l.season().approaches.is_empty(),
        "nothing before 40% of the season"
    );
    for _ in 7..17 {
        next(&mut l);
    }
    let ranked = {
        let mut t: Vec<&CatalogTeam> = catalog_year(2017).unwrap().teams.iter().collect();
        t.sort_by(|a, b| a.pace.total_cmp(&b.pace));
        t
    };
    let sauber_rank = ranked.iter().position(|t| t.slot == 9).unwrap();
    let anna = approaches_of(&l, "u-anna");
    assert!(anna.len() >= 2, "a 95-rated driver at Sauber is wanted: {anna:?}");
    for a in &anna {
        assert!(points.contains(&a.round), "{a:?}");
        assert_eq!(a.status, ApproachStatus::Interest);
        assert!(a.offer.is_none(), "the offer comes at the meeting");
        let rank = ranked.iter().position(|t| t.slot == a.slot).unwrap();
        assert!(rank < sauber_rank, "only faster teams approach: {a:?}");
        assert_eq!(
            anna.iter().filter(|b| b.round == a.round).count(),
            1,
            "one team at a time"
        );
        assert_eq!(
            anna.iter().filter(|b| b.slot == a.slot).count(),
            1,
            "a team asks once"
        );
        // A private mail for each.
        let team = &ranked[rank].name;
        let mail = l
            .news
            .iter()
            .find(|n| {
                n.to.as_deref() == Some("u-anna")
                    && n.round == a.round
                    && n.template.as_deref() == Some(MAIL_MEET)
            })
            .unwrap();
        assert_eq!(mail.kind, NewsKind::Mail);
        assert_eq!(mail.template.as_deref(), Some(MAIL_MEET));
        assert_eq!(mail.args, vec![("team".to_owned(), team.clone())]);
        assert_eq!(mail.text, format!("{team} would like to meet you."));
        assert!(mail.id > 0);
    }
    assert_eq!(meet_mails(&l, "u-anna").len(), anna.len());
    // A 70-rated driver reaches no team faster than Sauber.
    assert!(approaches_of(&l, "u-ben").is_empty());
    assert!(meet_mails(&l, "u-ben").is_empty());
    // Mail ids are unique.
    let mut ids: Vec<u32> = meet_mails(&l, "u-anna").iter().map(|n| n.id).collect();
    ids.dedup();
    assert_eq!(ids.len(), anna.len());
    // At the season's end, open approaches expire.
    finish(&mut l);
    assert_eq!(l.phase, Phase::SeasonEnd);
    assert!(
        l.season()
            .approaches
            .iter()
            .all(|a| a.status == ApproachStatus::Expired)
    );
}

#[test]
fn a_meeting_reveals_an_offer_and_can_leak() {
    let base = sauber(1, false);
    assert_eq!(
        reason(apply(&mut base.clone(), &anna(), Action::Meet { slot: 1 })),
        "no_approach"
    );
    let (mut leaks, mut quiet) = (0, 0);
    for seed in 0..60 {
        let mut l = base.clone();
        l.seed = seed;
        interest(&mut l, 1);
        let lines = apply(&mut l, &anna(), Action::Meet { slot: 1 }).unwrap();
        let a = approach(&l, 1);
        assert_eq!(a.status, ApproachStatus::Met);
        let o = a.offer.clone().unwrap();
        assert_eq!((o.slot, o.kind, o.years, o.early), (1, OfferKind::Move, 2, true));
        assert!(
            o.salary_k > player(&l, "u-anna").salary_k,
            "Mercedes pays more than Sauber"
        );
        assert!(
            lines[0].starts_with("Mercedes offers 2 years from 2017"),
            "{lines:?}"
        );
        let seen = l
            .news
            .iter()
            .any(|n| n.text.contains("Anna was seen meeting Mercedes"));
        if a.leaked {
            leaks += 1;
            assert_eq!(
                player(&l, "u-anna").recognition,
                progress::RECOGNITION_START - 5.0
            );
            assert!(seen);
        } else {
            quiet += 1;
            assert_eq!(player(&l, "u-anna").recognition, progress::RECOGNITION_START);
            assert!(!seen);
        }
        // Meeting again changes nothing.
        let again = l.clone();
        assert!(
            apply(&mut l, &anna(), Action::Meet { slot: 1 })
                .unwrap()
                .is_empty()
        );
        assert_eq!(l, again);
    }
    assert!(leaks > 0 && quiet > leaks * 2, "{leaks} leaks of 60");
    // A declined approach is over; meeting is for the season only.
    let mut l = base.clone();
    interest(&mut l, 1);
    apply(&mut l, &anna(), Action::DeclineApproach { slot: 1 }).unwrap();
    assert_eq!(approach(&l, 1).status, ApproachStatus::Declined);
    assert_eq!(
        reason(apply(&mut l, &anna(), Action::Meet { slot: 1 })),
        "no_approach"
    );
    let mut l = base;
    interest(&mut l, 1);
    finish(&mut l);
    assert_eq!(reason(apply(&mut l, &anna(), Action::Meet { slot: 1 })), "phase");
}

#[test]
fn accepting_early_signs_and_carries_into_next_season() {
    let mut l = sauber(3, false);
    interest(&mut l, 3);
    interest(&mut l, 1);
    assert_eq!(
        reason(apply(
            &mut l,
            &anna(),
            Action::AcceptApproach {
                slot: 3,
                replaces: None
            }
        )),
        "no_approach",
        "meet them first"
    );
    apply(&mut l, &anna(), Action::Meet { slot: 3 }).unwrap();
    apply(&mut l, &anna(), Action::Meet { slot: 1 }).unwrap();
    let o = approach(&l, 3).offer.unwrap();
    apply(
        &mut l,
        &anna(),
        Action::AcceptApproach {
            slot: 3,
            replaces: Some("max_verstappen".into()),
        },
    )
    .unwrap();
    let a = player(&l, "u-anna");
    assert_eq!(
        a.signed,
        Some(Seat {
            slot: 3,
            replaces: "max_verstappen".into()
        })
    );
    assert_eq!(a.signed_until, Some(2018));
    assert_eq!((a.signed_salary_k, a.signed_bonus_k), (o.salary_k, o.bonus_k));
    assert_eq!(
        a.contract_until,
        Some(2016),
        "the current contract runs out as it was"
    );
    assert_eq!(approach(&l, 3).status, ApproachStatus::Accepted);
    assert_eq!(
        approach(&l, 1).status,
        ApproachStatus::Declined,
        "every other approach is declined"
    );
    assert!(l.news.iter().any(|n| {
        n.text
            .starts_with("Anna signs early with Red Bull Racing for 2017 to 2018")
    }));
    assert_eq!(
        reason(apply(
            &mut l,
            &anna(),
            Action::AcceptApproach {
                slot: 1,
                replaces: None
            }
        )),
        "no_approach"
    );
    // The season's end offers only the signed contract.
    finish(&mut l);
    assert_eq!(l.phase, Phase::SeasonEnd);
    let offers = l.offers("u-anna");
    assert_eq!(offers.len(), 1, "{offers:?}");
    let c = offers[0].clone();
    assert_eq!(
        (c.slot, c.kind, c.years, c.early, c.status),
        (3, OfferKind::Contract, 2, true, OfferStatus::Agreed)
    );
    assert!(l.news.iter().any(|n| {
        n.text
            .contains("Anna has offers from Red Bull Racing (under contract).")
    }));
    assert_eq!(player(&l, "u-anna").contract_until, Some(2018));
    assert_eq!(reason(negotiate(&mut l, 3, ask(9_000, 3, false))), "bad_offer");
    // Taking the last round back clears the signings; the early one comes back with the season's end.
    apply(&mut l, &anna(), Action::Undo).unwrap();
    assert_eq!(l.phase, Phase::Racing);
    assert_eq!(approach(&l, 3).status, ApproachStatus::Accepted);
    finish(&mut l);
    assert_eq!(
        player(&l, "u-anna").signed.as_ref().unwrap().replaces,
        "max_verstappen"
    );
    assert_eq!(l.offers("u-anna").len(), 1);
    // Already signed: the owner moves on, and she races for Red Bull in 2017.
    apply(&mut l, &anna(), Action::NextSeason).unwrap();
    assert_eq!(l.year(), 2017);
    let a = player(&l, "u-anna");
    assert_eq!(l.season().player_entry("u-anna").unwrap().slot, 3);
    assert!(l.season().entry("max_verstappen").is_none());
    assert_eq!((a.contract_until, a.salary_k), (Some(2018), o.salary_k));
    assert_eq!(a.recognition, progress::RECOGNITION_NEW_TEAM);
    // An approach from before the new fields loads.
    let old: Approach =
        serde_json::from_str(r#"{"player":"u-anna","slot":3,"round":8,"status":"met"}"#).unwrap();
    assert_eq!((old.replaces, old.old_until), (None, None));
}

#[test]
fn a_buy_out_moves_a_driver_under_contract() {
    let mut l = sauber(5, false);
    l.player_mut("u-anna").unwrap().contract_until = Some(2017);
    interest(&mut l, 1);
    apply(&mut l, &anna(), Action::Meet { slot: 1 }).unwrap();
    let before = player(&l, "u-anna").recognition;
    apply(
        &mut l,
        &anna(),
        Action::AcceptApproach {
            slot: 1,
            replaces: None,
        },
    )
    .unwrap();
    assert_eq!(player(&l, "u-anna").recognition, before - 10.0);
    assert!(l.news.iter().any(|n| {
        n.text
            .starts_with("Mercedes buys Anna out of the contract with Sauber")
    }));
    finish(&mut l);
    // Sauber's contract is gone: only Mercedes' remains.
    let offers: Vec<(u8, OfferKind)> = l.offers("u-anna").iter().map(|o| (o.slot, o.kind)).collect();
    assert_eq!(offers, vec![(1, OfferKind::Contract)]);
    apply(
        &mut l,
        &anna(),
        Action::Sign {
            slot: 1,
            replaces: None,
            focus: Some(Focus::Development),
        },
    )
    .unwrap();
    assert_eq!(l.year(), 2017);
    assert_eq!(l.season().player_entry("u-anna").unwrap().slot, 1);
    let a = player(&l, "u-anna");
    assert_eq!(
        a.contract_until,
        Some(2018),
        "the new contract's two years, not Sauber's"
    );
    assert_eq!(a.focus, Some(Focus::Development));
}

#[test]
fn negotiation_accepts_counters_and_walks_away_deterministically() {
    let mut l = sauber(7, false);
    finish(&mut l);
    let moves: Vec<Offer> = l
        .offers("u-anna")
        .into_iter()
        .filter(|o| o.kind == OfferKind::Move)
        .cloned()
        .collect();
    assert!(moves.len() >= 2, "{moves:?}");
    let start = l.clone();
    let run = |l: &mut League| -> Vec<String> {
        let mut said = Vec::new();
        let (m1, m2) = (&moves[0], &moves[1]);
        assert_eq!(
            reason(negotiate(l, m1.slot, ask(m1.salary_k, 4, false))),
            "bad_offer"
        );
        // Too much: the team counters halfway, within its limit.
        said.extend(negotiate(l, m1.slot, ask(m1.salary_k * 3, 2, false)).unwrap());
        let o = offer(l, m1.slot).unwrap();
        assert_eq!((o.status, o.attempts), (OfferStatus::Countered, 2));
        let c = o.counter.unwrap();
        assert!(c.salary_k > m1.salary_k && c.salary_k < m1.salary_k * 2, "{c:?}");
        assert_eq!((c.years, c.number_one), (2, false));
        // The counter is within the limit.
        said.extend(negotiate(l, m1.slot, c).unwrap());
        let o = offer(l, m1.slot).unwrap();
        assert_eq!((o.status, o.agreed, o.years), (OfferStatus::Agreed, Some(c), 2));
        assert_eq!(reason(negotiate(l, m1.slot, c)), "bad_offer");
        // Asking too much three times: the other team walks away.
        for left in [2, 1] {
            said.extend(negotiate(l, m2.slot, ask(m2.salary_k * 5, 3, false)).unwrap());
            assert_eq!(offer(l, m2.slot).unwrap().attempts, left);
        }
        said.extend(negotiate(l, m2.slot, ask(m2.salary_k * 5, 3, false)).unwrap());
        assert!(said.last().unwrap().contains("walks away"), "{said:?}");
        assert!(offer(l, m2.slot).is_none(), "a withdrawn offer can't be signed");
        assert_eq!(
            reason(negotiate(l, m2.slot, ask(m2.salary_k, 1, false))),
            "no_offer"
        );
        said
    };
    let said = run(&mut l);
    let mut again = start.clone();
    assert_eq!(run(&mut again), said);
    assert_eq!(again, l, "the same for a seed");
    // The agreed terms are signed.
    let m1 = &moves[0];
    let agreed = offer(&l, m1.slot).unwrap().agreed.unwrap();
    apply(
        &mut l,
        &anna(),
        Action::Sign {
            slot: m1.slot,
            replaces: None,
            focus: None,
        },
    )
    .unwrap();
    let a = player(&l, "u-anna");
    assert_eq!(
        (l.year(), a.salary_k, a.contract_until),
        (2017, agreed.salary_k, Some(2018))
    );
    // A mid-season offer that walked away can't be accepted.
    let mut l = sauber(7, false);
    interest(&mut l, 1);
    apply(&mut l, &anna(), Action::Meet { slot: 1 }).unwrap();
    let first = approach(&l, 1).offer.unwrap().salary_k;
    for _ in 0..3 {
        negotiate(&mut l, 1, ask(first * 5, 2, false)).unwrap();
    }
    assert_eq!(approach(&l, 1).offer.unwrap().status, OfferStatus::Withdrawn);
    assert_eq!(
        reason(apply(
            &mut l,
            &anna(),
            Action::AcceptApproach {
                slot: 1,
                replaces: None
            }
        )),
        "walked_away"
    );
    assert_eq!(reason(negotiate(&mut l, 1, ask(first, 2, false))), "walked_away");
}

#[test]
fn the_own_team_never_withdraws_completely() {
    let mut l = sauber(11, false);
    finish(&mut l);
    let renew = offer(&l, 9).unwrap();
    assert_eq!(renew.kind, OfferKind::Renew);
    for _ in 0..3 {
        negotiate(&mut l, 9, ask(renew.salary_k * 5, 3, false)).unwrap();
    }
    let o = offer(&l, 9).unwrap();
    assert_eq!(
        (o.status, o.attempts, o.counter, o.agreed),
        (OfferStatus::Open, 0, None, None)
    );
    assert_eq!(o.salary_k, renew.salary_k, "the first terms stand");
    assert_eq!(
        reason(negotiate(&mut l, 9, ask(renew.salary_k, 1, false))),
        "walked_away"
    );
    // Nor does the last team that made an offer.
    let mut last = l.clone();
    last.season_mut().offers.retain(|o| o.kind == OfferKind::Move);
    last.season_mut().offers.truncate(1);
    let m = last.season().offers[0].clone();
    for _ in 0..3 {
        negotiate(&mut last, m.slot, ask(m.salary_k * 5, 3, false)).unwrap();
    }
    assert_eq!(offer(&last, m.slot).unwrap().status, OfferStatus::Open);
    // The renewal is signed on its first terms.
    apply(
        &mut l,
        &anna(),
        Action::Sign {
            slot: 9,
            replaces: None,
            focus: None,
        },
    )
    .unwrap();
    assert_eq!(l.year(), 2017);
    assert_eq!(player(&l, "u-anna").salary_k, renew.salary_k);
}

#[test]
fn a_number_one_clause_makes_the_number_one() {
    let mut l = sauber(13, false);
    finish(&mut l);
    // Sauber doesn't value her enough to make her its number one on its own.
    l.player_mut("u-anna").unwrap().recognition = 30.0;
    let renew = offer(&l, 9).unwrap();
    // Too slow for the clause: the team counters without it.
    let mut slow = l.clone();
    slow.player_mut("u-anna").unwrap().rating = 75.0;
    let lines = negotiate(&mut slow, 9, ask(renew.salary_k / 2, 2, true)).unwrap();
    let o = offer(&slow, 9).unwrap();
    assert_eq!(o.status, OfferStatus::Countered);
    assert!(!o.counter.unwrap().number_one);
    assert!(lines[0].contains("without a number-one clause"), "{lines:?}");
    // The clause costs a fifth of the limit: at the full salary it isn't agreed, below it is.
    let mut greedy = l.clone();
    negotiate(&mut greedy, 9, ask(renew.salary_k * 3 / 2, 2, true)).unwrap();
    assert_eq!(offer(&greedy, 9).unwrap().status, OfferStatus::Countered);
    negotiate(&mut l, 9, ask(renew.salary_k * 3 / 4, 2, true)).unwrap();
    let o = offer(&l, 9).unwrap();
    assert_eq!(o.status, OfferStatus::Agreed);
    apply(
        &mut l,
        &anna(),
        Action::Sign {
            slot: 9,
            replaces: None,
            focus: None,
        },
    )
    .unwrap();
    assert_eq!(l.year(), 2017);
    let a = player(&l, "u-anna");
    assert_eq!((a.number_one_until, a.contract_until), (Some(2018), Some(2018)));
    assert!(a.recognition < progress::NUMBER_ONE);
    assert!(a.number_one, "the clause makes her Sauber's number one");
    assert!(progress::number_one(&l, "u-anna"));
}

#[test]
fn undo_before_the_approach_round_removes_it() {
    let mut l = sauber(42, false);
    l.player_mut("u-anna").unwrap().recognition = 90.0;
    for _ in 0..8 {
        next(&mut l);
    }
    let came = l.season().approaches.clone();
    assert!(!came.is_empty() && came.iter().all(|a| a.round == 8), "{came:?}");
    let mails = meet_mails(&l, "u-anna").len();
    let slot = came[0].slot;
    apply(&mut l, &anna(), Action::Meet { slot }).unwrap();
    apply(&mut l, &anna(), Action::AcceptApproach { slot, replaces: None }).unwrap();
    // Taking back a later round keeps it.
    next(&mut l);
    apply(&mut l, &anna(), Action::Undo).unwrap();
    assert_eq!(l.season().next_round().unwrap().round, 9);
    assert_eq!(approach(&l, slot).status, ApproachStatus::Accepted);
    assert!(player(&l, "u-anna").signed.is_some());
    // Taking back round 8 takes the approach, its mail and the early signing with it.
    let before = l.season().round(8).unwrap().before.clone().unwrap();
    apply(&mut l, &anna(), Action::Undo).unwrap();
    assert_eq!(l.season().next_round().unwrap().round, 8);
    assert!(l.season().approaches.is_empty());
    assert!(meet_mails(&l, "u-anna").is_empty());
    let a = player(&l, "u-anna");
    assert_eq!(
        (
            a.signed.clone(),
            a.signed_until,
            a.signed_salary_k,
            a.signed_bonus_k
        ),
        (None, None, 0, 0)
    );
    let was = before.iter().find(|p| p.player == "u-anna").unwrap();
    assert_eq!(a.recognition, was.recognition);
    // Counting round 8 again brings the same interest back.
    next(&mut l);
    assert_eq!(l.season().approaches, came);
    assert_eq!(meet_mails(&l, "u-anna").len(), mails);
}
