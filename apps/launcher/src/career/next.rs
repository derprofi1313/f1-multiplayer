//! The next round's card: what to drive and how it counts, ready marks, and the proposals to
//! simulate, leave out or take back a round that everyone in a shared career agrees to.

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{Disableable, Sizable, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use swofty_core::league::{self, Action, League, Proposal, SessionKind, progress};

use crate::i18n::{tr, tr_owned};
use crate::pages::label;
use crate::shell::Shell;
use crate::theme::Palette;

use super::{
    capitalized, duplicate_text, kind_text, looks_counted, names_text, reason_text, session_text, track_text,
    undo_lines,
};

/// A muted line of text.
fn muted(text: impl Into<SharedString>, p: Palette) -> Div {
    div()
        .text_size(px(12.))
        .text_color(p.text_muted)
        .child(text.into())
}

/// "Anna proposes to simulate round 4, the Russian Grand Prix." (or "You propose …").
fn proposal_text(proposal: Proposal, mine: bool, name: &str, round: u8, race: &str) -> String {
    let template = match (proposal, mine) {
        (Proposal::Simulate, false) => tr("{name} proposes to simulate round {round}, the {race}."),
        (Proposal::Skip, false) => tr("{name} proposes to leave out round {round}, the {race}."),
        (Proposal::Undo, false) => tr("{name} proposes to take back round {round}, the {race}."),
        (Proposal::Simulate, true) => tr("You propose to simulate round {round}, the {race}."),
        (Proposal::Skip, true) => tr("You propose to leave out round {round}, the {race}."),
        (Proposal::Undo, true) => tr("You propose to take back round {round}, the {race}."),
    };
    template
        .replace("{name}", name)
        .replace("{round}", &round.to_string())
        .replace("{race}", race)
}

impl Shell {
    pub(super) fn career_next(&self, league: &League, me: &str, p: Palette, cx: &mut Context<Self>) -> Div {
        let busy = self.career.busy.is_some();
        let Some(b) = league.briefing(me) else {
            return v_flex();
        };
        let shared = league.needs_consent();
        let vote_open = league.vote.is_some();
        let sprint_next = b.sprint && !b.sprint_done;
        let can_undo = league
            .season()
            .rounds
            .iter()
            .any(|r| (r.done() && !r.auto) || r.started());
        let what = match (b.quali_done, sprint_next) {
            (false, true) => tr("qualifying, the sprint, then the race"),
            (false, false) => tr("qualifying, then the race"),
            (true, true) => tr("the sprint, then the race"),
            (true, false) => tr("the race (qualifying is counted)"),
        };
        let what = if b.practice_done || b.quali_done {
            what.to_owned()
        } else {
            tr_owned("a practice session, {what}").replace("{what}", what)
        };
        let me_player = league.player(me);
        let practice = league
            .season()
            .round(b.round)
            .and_then(|r| r.practice.iter().find(|l| l.player == me).cloned());
        let practice_text = match &practice {
            Some(l) => {
                let grades: Vec<String> = l
                    .programmes
                    .iter()
                    .map(|x| format!("{} {}", tr(x.programme.name()), x.grade))
                    .collect();
                let template = if l.driven {
                    tr("Practice: grade {grade} ({grades}), {rp} resource points{best}.")
                } else {
                    tr("Practice (simulated): grade {grade} ({grades}), {rp} resource points{best}.")
                };
                let best = if l.best_lap_ms > 0 {
                    tr_owned(", best lap {lap}").replace("{lap}", &progress::lap_text(l.best_lap_ms))
                } else {
                    String::new()
                };
                template
                    .replace("{grade}", &l.grade.to_string())
                    .replace("{grades}", &grades.join(", "))
                    .replace("{rp}", &l.rp.to_string())
                    .replace("{best}", &best)
            }
            None if b.quali_done => tr("Practice: left out this round.").to_owned(),
            None => tr(
                "Practice: drive a practice session first for resource points. It grades your laps \
                        (track acclimatisation), your best lap against your team-mate (qualifying pace) and \
                        a long run (tyre management). Or simulate it, for fewer points.",
            )
            .to_owned(),
        };
        let rp_text = me_player.map(|pl| {
            tr_owned("{rp} resource points to spend on the car (R&D tab) · recognition {recognition}{status}")
                .replace("{rp}", &pl.rp.to_string())
                .replace("{recognition}", &format!("{:.0}", pl.recognition))
                .replace(
                    "{status}",
                    if pl.number_one {
                        tr(" · number one driver")
                    } else {
                        ""
                    },
                )
        });
        let seat = match (&b.team, &b.replaces) {
            (Some(t), Some(r)) => tr_owned("You drive for {team}: pick {driver} in the game.")
                .replace("{team}", t)
                .replace("{driver}", r),
            _ => tr("You have no seat this season; you get offers when it ends.").to_owned(),
        };
        let mut aims = Vec::new();
        if let Some(g) = b.goal {
            aims.push(tr_owned("Team goal: P{place} or better").replace("{place}", &g.to_string()));
        }
        if let Some(m) = &b.mate {
            aims.push(tr_owned("team-mate {name}").replace("{name}", m));
        }
        if let Some(r) = &b.rival {
            aims.push(tr_owned("rival {name}").replace("{name}", r));
        }
        let howto = if b.solo {
            tr("In the game: F1 World > Grand Prix, the F1 {year} box, the {track} track, {distance}% race distance; \
                drive {what}. Each session counts by itself when it ends, also when you let the game simulate it.")
        } else {
            tr("In the game: open a LAN lobby (Multiplayer page), choose the {year} season box and the {track} \
                track, {distance}% race distance, and drive {what}. Each session counts by itself when it ends.")
        }
        .replace("{year}", &b.year.to_string())
        .replace("{track}", &track_text(b.track))
        .replace("{distance}", &b.distance.to_string())
        .replace("{what}", &what);
        let mut card = v_flex()
            .p(px(14.))
            .gap(px(8.))
            .rounded(px(6.))
            .border_1()
            .border_color(p.accent)
            .child(label(
                tr_owned("NEXT · ROUND {round} OF {rounds}")
                    .replace("{round}", &b.round.to_string())
                    .replace("{rounds}", &b.rounds.to_string()),
                p,
            ))
            .child(
                div()
                    .text_size(px(18.))
                    .font_weight(FontWeight::BOLD)
                    .child(format!("{} {}", b.year, b.race)),
            )
            .child(div().text_size(px(13.)).child(howto))
            .child(div().text_size(px(13.)).text_color(p.text_muted).child(seat))
            .when(!aims.is_empty(), |d| {
                d.child(
                    div()
                        .text_size(px(13.))
                        .text_color(p.text_muted)
                        .child(aims.join(" · ")),
                )
            })
            .when(!b.lineup.is_empty(), |d| {
                d.child(label(tr("DRIVER CHANGES THIS ROUND"), p))
                    .children(b.lineup.iter().map(|n| {
                        div()
                            .text_size(px(12.))
                            .text_color(p.text_muted)
                            .child(format!("{}.", super::texts::text(n)))
                    }))
            })
            .when(!b.solo, |d| d.child(self.career_ready(league, me, b.round, p, cx)))
            .child(
                h_flex()
                    .gap(px(8.))
                    .items_center()
                    .child(div().flex_1().text_size(px(13.)).child(practice_text))
                    .when(practice.is_none() && !b.quali_done && b.team.is_some(), |d| {
                        d.child(
                            Button::new("career-practice")
                                .outline()
                                .small()
                                .icon(IconName::Gauge)
                                .label(tr("Simulate practice"))
                                .disabled(busy)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.career_act(Action::Practice { round: None }, cx)
                                })),
                        )
                    }),
            )
            .when_some(rp_text, |d, t| d.child(muted(t, p)))
            .when_some(self.career_vote(league, me, p, cx), |d, v| d.child(v))
            .child(
                h_flex()
                    .gap(px(8.))
                    .child(
                        Button::new("career-simulate")
                            .outline()
                            .small()
                            .icon(IconName::Dices)
                            .label(if shared {
                                tr("Propose to simulate this round")
                            } else {
                                tr("Simulate this round")
                            })
                            .disabled(busy || vote_open)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.career_act(Action::Simulate { round: None }, cx)
                            })),
                    )
                    .child(
                        Button::new("career-skip")
                            .outline()
                            .small()
                            .icon(IconName::SkipForward)
                            .label(if shared {
                                tr("Propose to leave it out")
                            } else {
                                tr("Leave it out")
                            })
                            .disabled(busy || vote_open)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.career_act(Action::Skip { round: None }, cx)
                            })),
                    )
                    .child(
                        Button::new("career-undo")
                            .outline()
                            .small()
                            .icon(IconName::Undo)
                            .label(tr("Take back the last round"))
                            .disabled(busy || !can_undo || vote_open)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.career.confirm = Some("undo".into());
                                cx.notify();
                            })),
                    ),
            )
            .when(shared && !vote_open, |d| {
                d.child(muted(
                    tr("In a shared career, everyone agrees before a round is simulated, left out or taken back."),
                    p,
                ))
            })
            .when(self.career.confirm.as_deref() == Some("undo") && !vote_open, |d| {
                d.child(self.career_undo_confirm(league, me, shared, p, cx))
            });
        if let Some(refused) = self.career.refused.clone() {
            card = card.child(self.career_refused(league, refused, p, cx));
        }
        let waiting = self.career.races.iter().rev().take(4).map(|r| {
            let mine = r
                .cars
                .iter()
                .find(|c| c.mine)
                .map(|c| tr_owned(", you P{place}").replace("{place}", &c.position.to_string()));
            let quali = league::session_kind(r.session_type) == Some(SessionKind::Qualifying);
            let practice = league::session_kind(r.session_type) == Some(SessionKind::Practice);
            let mut text = tr_owned("{what} · {track} · {cars} cars{you}")
                .replace("{what}", &capitalized(tr(session_text(r.session_type))))
                .replace("{track}", &track_text(r.track))
                .replace("{cars}", &r.cars.len().to_string())
                .replace("{you}", &mine.unwrap_or_default());
            let duplicate = looks_counted(league, r);
            if let Some((round, kind, by)) = &duplicate {
                text = format!("{text} · {}", duplicate_text(*round, *kind, by));
            }
            let (a, b2, session) = (r.clone(), r.clone(), r.session);
            h_flex()
                .gap(px(8.))
                .items_center()
                .child(
                    div()
                        .flex_1()
                        .text_size(px(13.))
                        .when(duplicate.is_some(), |d| d.text_color(p.warning))
                        .child(text),
                )
                .child(
                    Button::new(SharedString::from(format!("count-race-{:x}", r.session)))
                        .outline()
                        .small()
                        .label(if practice {
                            tr("Count practice")
                        } else if quali {
                            tr("Count qualifying")
                        } else {
                            tr("Count as race")
                        })
                        .disabled(busy)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            let kind = if practice {
                                SessionKind::Practice
                            } else if quali {
                                SessionKind::Qualifying
                            } else {
                                SessionKind::Race
                            };
                            this.career_count(a.clone(), Some(kind), false, cx)
                        })),
                )
                .when(sprint_next && !quali && !practice, |d| {
                    d.child(
                        Button::new(SharedString::from(format!("count-sprint-{:x}", r.session)))
                            .outline()
                            .small()
                            .label(tr("Count as sprint"))
                            .disabled(busy)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.career_count(b2.clone(), Some(SessionKind::Sprint), false, cx)
                            })),
                    )
                })
                .child(
                    Button::new(SharedString::from(format!("discard-race-{:x}", r.session)))
                        .outline()
                        .small()
                        .icon(IconName::X)
                        .label(tr("Discard"))
                        .on_click(cx.listener(move |this, _, _, cx| this.career_discard(session, cx))),
                )
        });
        let waiting: Vec<_> = waiting.collect();
        if !waiting.is_empty() {
            card = card
                .child(label(tr("SESSIONS THE GAME REPORTED, NOT COUNTED YET"), p))
                .children(waiting);
        }
        card
    }

    /// The "Ready for round N" toggle and who is ready, as in the game's two-player career.
    fn career_ready(&self, league: &League, me: &str, round: u8, p: Palette, cx: &mut Context<Self>) -> Div {
        let ready = league
            .season()
            .round(round)
            .is_some_and(|r| r.ready.iter().any(|id| id == me));
        let b = Button::new("career-ready")
            .small()
            .icon(if ready { IconName::Check } else { IconName::Flag })
            .label(tr_owned("I'm ready for round {round}").replace("{round}", &round.to_string()))
            .disabled(self.career.busy.is_some())
            .on_click(
                cx.listener(move |this, _, _, cx| this.career_act(Action::Ready { ready: !ready }, cx)),
            );
        h_flex()
            .gap(px(10.))
            .items_center()
            .child(if ready { b.primary() } else { b.outline() })
            .child(muted(ready_line(league, round), p))
    }

    /// The open proposal: who asks for what, who it waits for, and the buttons to agree,
    /// decline, withdraw it or (the owner) go ahead without waiting.
    fn career_vote(&self, league: &League, me: &str, p: Palette, cx: &mut Context<Self>) -> Option<Div> {
        let vote = league.vote.as_ref()?;
        let busy = self.career.busy.is_some();
        let owner = self.career.owner();
        let mine = vote.by == me;
        let race = league
            .season()
            .round(vote.round)
            .map_or_else(String::new, |r| r.race.clone());
        let waiting: Vec<String> = league.waiting_for().iter().map(|p| p.name.clone()).collect();
        let asked = league.waiting_for().iter().any(|p| p.id == me);
        let me_name = league.player(me).map_or("", |p| p.name.as_str());
        let preview = (vote.proposal == Proposal::Undo)
            .then(|| league::undo_preview(league))
            .flatten()
            .map(|pr| undo_lines(&pr, me_name))
            .unwrap_or_default();
        Some(
            v_flex()
                .p(px(10.))
                .gap(px(6.))
                .rounded(px(6.))
                .border_1()
                .border_color(p.warning)
                .child(
                    div()
                        .text_size(px(13.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(proposal_text(
                            vote.proposal,
                            mine,
                            &vote.by_name,
                            vote.round,
                            &race,
                        )),
                )
                .children(preview.into_iter().map(|l| muted(l, p)))
                .when(!waiting.is_empty(), |d| {
                    d.child(muted(
                        tr_owned("Waiting for {names} to agree.").replace("{names}", &names_text(&waiting)),
                        p,
                    ))
                })
                .child(
                    h_flex()
                        .gap(px(8.))
                        .when(asked, |d| {
                            d.child(
                                Button::new("career-vote-yes")
                                    .primary()
                                    .small()
                                    .icon(IconName::Check)
                                    .label(tr("Agree"))
                                    .disabled(busy)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.career_act(
                                            Action::Vote {
                                                yes: true,
                                                force: false,
                                            },
                                            cx,
                                        )
                                    })),
                            )
                            .child(
                                Button::new("career-vote-no")
                                    .outline()
                                    .small()
                                    .icon(IconName::X)
                                    .label(tr("Decline"))
                                    .disabled(busy)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.career_act(
                                            Action::Vote {
                                                yes: false,
                                                force: false,
                                            },
                                            cx,
                                        )
                                    })),
                            )
                        })
                        .when(mine, |d| {
                            d.child(
                                Button::new("career-vote-cancel")
                                    .outline()
                                    .small()
                                    .icon(IconName::X)
                                    .label(tr("Cancel"))
                                    .disabled(busy)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.career_act(
                                            Action::Vote {
                                                yes: false,
                                                force: false,
                                            },
                                            cx,
                                        )
                                    })),
                            )
                        })
                        .when(owner && !waiting.is_empty(), |d| {
                            d.child(
                                Button::new("career-vote-force")
                                    .ghost()
                                    .small()
                                    .label(tr("Do it now"))
                                    .disabled(busy)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.career_act(
                                            Action::Vote {
                                                yes: true,
                                                force: true,
                                            },
                                            cx,
                                        )
                                    })),
                            )
                        }),
                ),
        )
    }

    /// What taking back the last round takes back, to confirm it (or, in a shared career, to
    /// propose it).
    fn career_undo_confirm(
        &self,
        league: &League,
        me: &str,
        shared: bool,
        p: Palette,
        cx: &mut Context<Self>,
    ) -> Div {
        let me_name = league.player(me).map_or("", |p| p.name.as_str());
        let lines = league::undo_preview(league)
            .map(|pr| undo_lines(&pr, me_name))
            .unwrap_or_default();
        v_flex()
            .p(px(10.))
            .gap(px(6.))
            .rounded(px(6.))
            .border_1()
            .border_color(p.border)
            .children(lines.into_iter().map(|l| div().text_size(px(13.)).child(l)))
            .child(
                h_flex()
                    .gap(px(8.))
                    .child(
                        Button::new("career-undo-yes")
                            .primary()
                            .small()
                            .icon(IconName::Undo)
                            .label(if shared {
                                tr("Propose to take it back")
                            } else {
                                tr("Take it back")
                            })
                            .disabled(self.career.busy.is_some())
                            .on_click(cx.listener(|this, _, _, cx| this.career_act(Action::Undo, cx))),
                    )
                    .child(
                        Button::new("career-undo-no")
                            .outline()
                            .small()
                            .label(tr("Keep it"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.career.confirm = None;
                                cx.notify();
                            })),
                    ),
            )
    }

    /// A race the career refused to count, with what to do about it: discard a duplicate (or
    /// count it after all), say whether it was the sprint or the Grand Prix, or count a race at
    /// another track anyway.
    fn career_refused(
        &self,
        league: &League,
        refused: super::Refused,
        p: Palette,
        cx: &mut Context<Self>,
    ) -> Div {
        let busy = self.career.busy.is_some();
        let super::Refused {
            report,
            kind,
            force,
            reason,
            message,
        } = refused;
        let session = report.session;
        let what = tr(session_text(report.session_type));
        let duplicate = looks_counted(league, &report);
        let discard = Button::new("career-refused-discard")
            .small()
            .icon(IconName::X)
            .label(tr("Discard"))
            .on_click(cx.listener(move |this, _, _, cx| this.career_discard(session, cx)));
        let row = |text: String, warn: bool| {
            div()
                .flex_1()
                .text_size(px(13.))
                .text_color(if warn { p.warning } else { p.text })
                .child(text)
        };
        match reason.as_str() {
            "duplicate" => {
                let text = match &duplicate {
                    Some((round, k, by)) => duplicate_text(*round, *k, by),
                    None => capitalized(&message),
                };
                let again = report.clone();
                let other_kind = kind.unwrap_or(match league::session_kind(report.session_type) {
                    Some(k) => k,
                    None => SessionKind::Race,
                });
                v_flex().gap(px(6.)).child(row(text, true)).child(
                    h_flex().gap(px(8.)).child(discard.primary()).child(
                        Button::new("career-refused-other")
                            .ghost()
                            .small()
                            .label(tr_owned("It was another {kind}").replace("{kind}", kind_text(other_kind)))
                            .disabled(busy)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.career_count(again.clone(), kind, true, cx)
                            })),
                    ),
                )
            }
            "ask_kind" => {
                let (sprint, race) = (report.clone(), report.clone());
                v_flex()
                    .gap(px(6.))
                    .child(row(
                        tr_owned("{track}: {question}")
                            .replace("{track}", &track_text(report.track))
                            .replace("{question}", tr(reason_text("ask_kind").unwrap_or_default())),
                        true,
                    ))
                    .child(
                        h_flex()
                            .gap(px(8.))
                            .child(
                                Button::new("career-refused-sprint")
                                    .outline()
                                    .small()
                                    .label(tr("Sprint"))
                                    .disabled(busy)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.career_count(
                                            sprint.clone(),
                                            Some(SessionKind::Sprint),
                                            force,
                                            cx,
                                        )
                                    })),
                            )
                            .child(
                                Button::new("career-refused-race")
                                    .outline()
                                    .small()
                                    .label(tr("Grand Prix"))
                                    .disabled(busy)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.career_count(race.clone(), Some(SessionKind::Race), force, cx)
                                    })),
                            )
                            .child(discard.outline()),
                    )
            }
            _ => {
                // A race at another track than the round's: counting it anyway also skips the
                // duplicate check, so say when it looks like a session counted already.
                let anyway = report.clone();
                let text = tr_owned("The {what} at {track} wasn't counted: {why}.")
                    .replace("{what}", what)
                    .replace("{track}", &track_text(report.track))
                    .replace("{why}", &message);
                let force_button = Button::new("career-force")
                    .small()
                    .label(tr("Count it anyway"))
                    .disabled(busy)
                    .on_click(
                        cx.listener(move |this, _, _, cx| this.career_count(anyway.clone(), kind, true, cx)),
                    );
                v_flex()
                    .gap(px(6.))
                    .child(row(text, true))
                    .when_some(duplicate.as_ref(), |d, (round, k, by)| {
                        d.child(row(duplicate_text(*round, *k, by), true))
                    })
                    .child(
                        h_flex()
                            .gap(px(8.))
                            .child(if duplicate.is_some() {
                                force_button.ghost()
                            } else {
                                force_button.outline()
                            })
                            .child(if duplicate.is_some() {
                                discard.primary()
                            } else {
                                discard.outline()
                            }),
                    )
            }
        }
    }
}

/// "Anna ready · Ben not ready" for the players still in the career.
pub(super) fn ready_line(league: &League, round: u8) -> String {
    let ready: &[String] = league.season().round(round).map_or(&[], |r| &r.ready);
    league
        .active_players()
        .iter()
        .map(|pl| {
            if ready.contains(&pl.id) {
                tr_owned("{name} ready").replace("{name}", &pl.name)
            } else {
                tr_owned("{name} not ready").replace("{name}", &pl.name)
            }
        })
        .collect::<Vec<_>>()
        .join(" · ")
}
