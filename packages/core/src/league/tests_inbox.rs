//! Tests of the league's inbox (W2-4).

use super::inbox::{self, TEMPLATES};
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

/// A 2016 league with Anna and Ben both at Ferrari.
fn ferrari_2016(seed: u64) -> League {
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

/// Simulate the next round (and whatever the game can't drive after it), as the consent of
/// every player would.
fn simulate_next(l: &mut League) -> u8 {
    let n = l.season().next_round().unwrap().round;
    l.simulate(Some(n), false).unwrap();
    l.settle();
    n
}

fn mails(l: &League) -> Vec<&News> {
    l.news.iter().filter(|n| n.kind != NewsKind::News).collect()
}

fn mails_of<'a>(l: &'a League, player: &str) -> Vec<&'a News> {
    mails(l)
        .into_iter()
        .filter(|n| n.to.as_deref() == Some(player))
        .collect()
}

/// Round `n`'s race ended for Anna and Ben as given; their round-`n` mails are made again.
fn race_again(l: &mut League, n: u8, anna_at: (u8, Finish), ben_at: (u8, Finish)) {
    let year = l.year();
    l.news
        .retain(|x| x.kind == NewsKind::News || x.year != year || x.round != n);
    let season = l.season().clone();
    let ids = [
        (season.player_entry("u-anna").unwrap().id.clone(), anna_at),
        (season.player_entry("u-ben").unwrap().id.clone(), ben_at),
    ];
    let round = l.season_mut().rounds.iter_mut().find(|r| r.round == n).unwrap();
    for (id, (position, status)) in ids {
        let c = round.race_result.iter_mut().find(|c| c.entry == id).unwrap();
        c.position = position;
        c.status = status;
        c.ai_driven = false;
    }
    inbox::after_round(l, n);
}

fn answer(l: &mut League, who: &Actor, news: u32, choice: u8) -> Result<Vec<String>, LeagueError> {
    apply(l, who, Action::Answer { news, choice })
}

#[test]
fn events_are_generated_deterministically() {
    let play = |seed: u64| {
        let mut l = ferrari_2016(seed);
        for _ in 0..10 {
            simulate_next(&mut l);
        }
        l
    };
    let (a, b) = (play(42), play(42));
    let (ma, mb) = (mails(&a), mails(&b));
    assert_eq!(ma, mb);
    assert!(!ma.is_empty(), "ten rounds bring some mail");
    // Ids count up from 1 without gaps, and the league remembers the last one.
    let ids: Vec<u32> = ma.iter().map(|n| n.id).collect();
    assert_eq!(ids, (1..=ids.len() as u32).collect::<Vec<_>>());
    assert_eq!(a.news_seq, ids.len() as u32);
    for m in &ma {
        // One item per player and round, for that player, with listed templates and its text.
        assert_eq!(
            ma.iter().filter(|x| x.round == m.round && x.to == m.to).count(),
            1
        );
        assert!(m.to.is_some() && m.answered.is_none());
        let template = m.template.as_deref().unwrap();
        assert!(TEMPLATES.contains(&template), "{template}");
        assert_eq!(m.text, inbox::render(template, &m.args));
        assert!(!m.text.contains('{'), "{}", m.text);
        assert!(m.choices.iter().all(|c| TEMPLATES.contains(&c.template.as_str())));
    }
    // The document still round-trips.
    let json = serde_json::to_string(&a).unwrap();
    assert_eq!(serde_json::from_str::<League>(&json).unwrap(), a);
}

#[test]
fn each_kind_of_mail_comes_when_it_should() {
    let mut l = ferrari_2016(42);
    // A podium brings a media question with three answers.
    simulate_next(&mut l);
    race_again(&mut l, 1, (2, Finish::Finished), (6, Finish::Dnf));
    let anna_mail = mails_of(&l, "u-anna")[0].clone();
    assert_eq!(anna_mail.kind, NewsKind::Media);
    assert_eq!(anna_mail.template.as_deref(), Some(inbox::T_MEDIA_PODIUM));
    assert!(anna_mail.text.contains("you finished P2"), "{}", anna_mail.text);
    assert_eq!(anna_mail.choices.len(), 3);
    // A retirement too.
    assert_eq!(
        mails_of(&l, "u-ben")[0].template.as_deref(),
        Some(inbox::T_MEDIA_DNF)
    );
    // Three races in a row behind the team-mate: the rivalry flares up.
    for n in 1..=3 {
        if n > 1 {
            simulate_next(&mut l);
        }
        race_again(&mut l, n, (8, Finish::Finished), (5, Finish::Finished));
    }
    let rivalry = mails_of(&l, "u-anna")
        .into_iter()
        .find(|m| m.round == 3)
        .unwrap()
        .clone();
    assert_eq!(rivalry.template.as_deref(), Some(inbox::T_RIVALRY));
    assert!(rivalry.text.contains("in the last 3 races"), "{}", rivalry.text);
    assert_eq!(rivalry.choices.len(), 3);
    assert!(rivalry.args.contains(&("count".into(), "3".into())));
    // New parts for the next round: the engineer reports, with no answer to give.
    let next = l.season().next_round().unwrap().round;
    l.season_mut().upgrades.push(Upgrade {
        slot: 2,
        part: Part::Aero,
        level: 1,
        by: "u-anna".into(),
        cost: 800,
        round: 2,
        started: false,
        ready: next + 1,
        node: None,
        failed: false,
    });
    let n = simulate_next(&mut l);
    assert_eq!(n, next);
    race_again(&mut l, n, (6, Finish::Finished), (7, Finish::Finished));
    let report = mails_of(&l, "u-anna")
        .into_iter()
        .find(|m| m.round == n)
        .unwrap()
        .clone();
    assert_eq!(report.kind, NewsKind::Mail);
    assert_eq!(report.template.as_deref(), Some(inbox::T_ENGINEER));
    assert!(report.choices.is_empty());
    // A department asks now and then, seeded: in about a third of the rounds.
    let mut asked = 0;
    for seed in 0..60 {
        l.seed = seed;
        race_again(&mut l, 1, (6, Finish::Finished), (7, Finish::Finished));
        let m = mails_of(&l, "u-anna").into_iter().find(|m| m.round == 1).cloned();
        if let Some(m) = m {
            assert_eq!(m.kind, NewsKind::Event);
            assert!(inbox::programme_of(m.template.as_deref().unwrap()).is_some());
            assert_eq!(m.choices.len(), 2);
            assert!(m.choices[0].effect.rp >= 30);
            asked += 1;
        }
    }
    assert!((10..=35).contains(&asked), "{asked}");
}

#[test]
fn answering_applies_the_effect_once() {
    let mut l = ferrari_2016(42);
    simulate_next(&mut l);
    race_again(&mut l, 1, (3, Finish::Finished), (4, Finish::Finished));
    let mail = mails_of(&l, "u-anna")[0].clone();
    assert_eq!(mail.template.as_deref(), Some(inbox::T_MEDIA_PODIUM));
    let before = l.player("u-anna").unwrap().clone();
    // Bold: more pace, less recognition.
    answer(&mut l, &anna(), mail.id, 1).unwrap();
    let after = l.player("u-anna").unwrap().clone();
    let (a, b) = (after.attributes.unwrap(), before.attributes.unwrap());
    assert!((a.pace - b.pace - 0.3).abs() < 1e-4);
    assert_eq!(a.racecraft, b.racecraft);
    assert!((after.recognition - (before.recognition - 1.0)).abs() < 1e-4);
    assert_eq!(after.rating, (a.overall() * 10.0).round() / 10.0);
    assert_eq!(l.news.iter().find(|n| n.id == mail.id).unwrap().answered, Some(1));
    // A second answer is refused and changes nothing.
    let err = answer(&mut l, &anna(), mail.id, 0).unwrap_err();
    assert_eq!(err.reason, "answered");
    assert_eq!(l.player("u-anna").unwrap(), &after);

    // Resource points come through `earn`.
    let mut l = ferrari_2016(42);
    simulate_next(&mut l);
    let dept = (0..200)
        .find_map(|seed| {
            l.seed = seed;
            race_again(&mut l, 1, (6, Finish::Finished), (7, Finish::Finished));
            mails_of(&l, "u-anna").into_iter().find(|m| m.round == 1).cloned()
        })
        .unwrap();
    let rp = l.player("u-anna").unwrap().rp;
    answer(&mut l, &anna(), dept.id, 0).unwrap();
    let gained = l.player("u-anna").unwrap().rp - rp;
    assert!(gained >= dept.choices[0].effect.rp as u32);
    // Declining does nothing, and can't come after an answer.
    assert_eq!(
        answer(&mut l, &anna(), dept.id, 1).unwrap_err().reason,
        "answered"
    );
}

#[test]
fn only_the_addressee_answers() {
    let mut l = ferrari_2016(42);
    simulate_next(&mut l);
    race_again(&mut l, 1, (1, Finish::Finished), (2, Finish::Finished));
    let mail = mails_of(&l, "u-anna")[0].clone();
    let ben_before = l.player("u-ben").unwrap().clone();
    let err = answer(&mut l, &ben(), mail.id, 0).unwrap_err();
    assert_eq!(err.reason, "not_yours");
    assert_eq!(l.player("u-ben").unwrap(), &ben_before);
    assert_eq!(l.news.iter().find(|n| n.id == mail.id).unwrap().answered, None);
    // No such mail, plain news, no such answer.
    assert_eq!(answer(&mut l, &anna(), 999, 0).unwrap_err().reason, "no_mail");
    assert_eq!(answer(&mut l, &anna(), 0, 0).unwrap_err().reason, "no_mail");
    assert_eq!(
        answer(&mut l, &anna(), mail.id, 3).unwrap_err().reason,
        "bad_choice"
    );
    // Someone outside the career can't either.
    let stranger = Actor {
        id: "u-carl".into(),
        name: "Carl".into(),
        owner: false,
    };
    assert_eq!(
        answer(&mut l, &stranger, mail.id, 0).unwrap_err().reason,
        "not_player"
    );
    answer(&mut l, &anna(), mail.id, 0).unwrap();
}

#[test]
fn undo_drops_the_rounds_events() {
    let mut l = ferrari_2016(42);
    simulate_next(&mut l);
    race_again(&mut l, 1, (2, Finish::Finished), (4, Finish::Finished));
    let first = mails_of(&l, "u-anna")[0].clone();
    let recognition = l.player("u-anna").unwrap().recognition;
    // Round 2 is counted before Anna answers the mail of round 1.
    let n = simulate_next(&mut l);
    race_again(&mut l, n, (1, Finish::Finished), (4, Finish::Finished));
    let second = mails_of(&l, "u-anna")
        .into_iter()
        .find(|m| m.round == n)
        .unwrap()
        .clone();
    answer(&mut l, &anna(), first.id, 0).unwrap();
    answer(&mut l, &anna(), second.id, 1).unwrap();
    let seq = l.news_seq;

    l.undo().unwrap();
    l.settle();
    // The round's mail goes, with what its answer did; the earlier answer stays, and so does
    // what it did, though it came after the round's snapshot.
    assert!(mails(&l).iter().all(|m| m.round < n));
    let kept = l.news.iter().find(|m| m.id == first.id).unwrap();
    assert_eq!(kept.answered, Some(0));
    let p = l.player("u-anna").unwrap();
    assert!(
        (p.recognition - (recognition + 1.0)).abs() < 1e-4,
        "{}",
        p.recognition
    );
    // Counted again, the round brings its mail again, with a new id.
    let again = simulate_next(&mut l);
    assert_eq!(again, n);
    race_again(&mut l, n, (1, Finish::Finished), (4, Finish::Finished));
    let fresh = mails_of(&l, "u-anna")
        .into_iter()
        .find(|m| m.round == n)
        .unwrap()
        .clone();
    assert!(fresh.id > seq);
    assert_eq!(fresh.answered, None);
    assert_eq!((fresh.template, fresh.args), (second.template, second.args));
}

#[test]
fn unread_counts_follow_mail_read() {
    let mut l = ferrari_2016(42);
    simulate_next(&mut l);
    race_again(&mut l, 1, (1, Finish::Finished), (5, Finish::Dnf));
    let n = simulate_next(&mut l);
    race_again(&mut l, n, (3, Finish::Finished), (5, Finish::Dnf));
    assert_eq!(inbox::unread(&l, "u-anna").len(), 2);
    assert_eq!(inbox::unread(&l, "u-ben").len(), 2);
    // Plain news never counts.
    assert!(
        inbox::unread(&l, "u-anna")
            .iter()
            .all(|m| m.kind != NewsKind::News)
    );
    let first = inbox::unread(&l, "u-anna")[0].id;
    apply(&mut l, &anna(), Action::ReadMail { upto: first }).unwrap();
    assert_eq!(inbox::unread(&l, "u-anna").len(), 1);
    assert_eq!(l.player("u-anna").unwrap().mail_read, first);
    // Reading never goes back, and never past the last mail.
    apply(&mut l, &anna(), Action::ReadMail { upto: 0 }).unwrap();
    assert_eq!(l.player("u-anna").unwrap().mail_read, first);
    apply(&mut l, &anna(), Action::ReadMail { upto: 9_999 }).unwrap();
    assert_eq!(l.player("u-anna").unwrap().mail_read, l.news_seq);
    assert!(inbox::unread(&l, "u-anna").is_empty());
    // Ben's inbox is his own.
    assert_eq!(inbox::unread(&l, "u-ben").len(), 2);
    // The next mail is unread again.
    let n = simulate_next(&mut l);
    race_again(&mut l, n, (2, Finish::Finished), (5, Finish::Finished));
    assert_eq!(inbox::unread(&l, "u-anna").len(), 1);
}

#[test]
#[allow(non_snake_case)]
fn every_template_is_listed_in_TEMPLATES() {
    let source = include_str!("inbox.rs");
    let mut found = Vec::new();
    // The template constants (their literal may be on the next line) and any literal given to a
    // `template:` field.
    for marker in ["const T_", "template: \""] {
        for (at, _) in source.match_indices(marker) {
            let rest = &source[at..];
            let start = rest.find('"').unwrap() + 1;
            let end = start + rest[start..].find("\"").unwrap();
            found.push(rest[start..end].to_owned());
        }
    }
    assert_eq!(found.len(), TEMPLATES.len(), "{found:?}");
    for t in &found {
        assert!(TEMPLATES.contains(&t.as_str()), "{t} is not in TEMPLATES");
    }
    let mut unique = TEMPLATES.to_vec();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), TEMPLATES.len(), "a template is listed twice");
    // Placeholders are plain words.
    for t in TEMPLATES {
        let mut rest = *t;
        while let Some(i) = rest.find('{') {
            let end = rest[i..].find('}').expect("closed placeholder") + i;
            assert!(rest[i + 1..end].chars().all(|c| c.is_ascii_lowercase()), "{t}");
            rest = &rest[end + 1..];
        }
    }
}
