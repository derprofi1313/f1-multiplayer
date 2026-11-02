//! Contract offers between seasons, mid-season approaches, contract talks, the focus choice and
//! the finished career.

use std::collections::HashMap;

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{Disableable, Sizable, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use swofty_core::league::{
    self, Action, ApproachStatus, Focus, League, Offer, OfferKind, OfferStatus, Phase, Terms, market,
    progress,
};

use crate::i18n::{tr, tr_owned};
use crate::shell::Shell;
use crate::theme::Palette;

/// The lowest salary a player can ask for, in thousands (the teams' own floor).
const SALARY_MIN: u32 = 100;
/// The highest, in thousands.
const SALARY_MAX: u32 = 50_000;
/// The steps the salary moves in, in thousands.
const SALARY_STEPS: [i32; 4] = [-250, -50, 50, 250];

/// The terms a player asks for in contract talks, per offer, until they ask.
#[derive(Default)]
pub(super) struct Talks {
    asks: HashMap<TalkKey, Terms>,
}

/// An offer in talks: the season it came in, the team's slot, and whether it is a mid-season
/// approach's.
type TalkKey = (u16, u8, bool);

impl Talks {
    /// The terms asked for `key`: the changed ones, else `default` (see [`ask_of`]).
    fn ask(&self, key: TalkKey, default: Terms) -> Terms {
        self.asks.get(&key).copied().unwrap_or(default)
    }

    fn change(&mut self, key: TalkKey, default: Terms, change: Change) {
        let terms = self.asks.get(&key).copied().unwrap_or(default);
        self.asks.insert(key, changed(terms, change));
    }
}

/// What the player asks for before changing anything: what was agreed, the team's counter, or
/// the team's first terms.
pub(super) fn ask_of(offer: &Offer, edited: Option<&Terms>) -> Terms {
    edited.copied().unwrap_or_else(|| {
        offer.agreed.or(offer.counter).unwrap_or(Terms {
            salary_k: offer.salary_k,
            years: offer.years.clamp(*market::YEARS.start(), *market::YEARS.end()),
            number_one: false,
        })
    })
}

/// One change to the terms asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Change {
    /// More or less salary, in thousands.
    Salary(i32),
    Years(u8),
    NumberOne,
}

/// `terms` with `change`: the salary in steps of 50k between the floor and the ceiling, the
/// length within what a contract can run.
pub(super) fn changed(terms: Terms, change: Change) -> Terms {
    match change {
        Change::Salary(d) => {
            let salary = (i64::from(terms.salary_k) + i64::from(d))
                .clamp(i64::from(SALARY_MIN), i64::from(SALARY_MAX));
            Terms {
                salary_k: (salary as u32) / 50 * 50,
                ..terms
            }
        }
        Change::Years(y) => Terms {
            years: y.clamp(*market::YEARS.start(), *market::YEARS.end()),
            ..terms
        },
        Change::NumberOne => Terms {
            number_one: !terms.number_one,
            ..terms
        },
    }
}

/// The offer can still be talked about: not a contract that runs on, not agreed or withdrawn,
/// and the team still talks.
fn negotiable(offer: &Offer) -> bool {
    offer.kind != OfferKind::Contract
        && offer.attempts > 0
        && matches!(offer.status, OfferStatus::Open | OfferStatus::Countered)
}

/// "1 year", "2 years".
fn years_text(n: u8) -> String {
    if n == 1 {
        tr("1 year").to_owned()
    } else {
        tr_owned("{n} years").replace("{n}", &n.to_string())
    }
}

/// "$2.5M a season for 2 years", with the number-one clause when asked.
fn terms_text(t: Terms) -> String {
    let template = if t.number_one {
        tr("{salary} a season for {years}, with a number-one clause")
    } else {
        tr("{salary} a season for {years}")
    };
    template
        .replace("{salary}", &progress::money(u64::from(t.salary_k)))
        .replace("{years}", &years_text(t.years))
}

/// "$2.5M a season + $25k a point", with the agreed salary once there is one.
fn pay_text(o: &Offer) -> String {
    let salary = o.agreed.map_or(o.salary_k, |t| t.salary_k);
    tr_owned("{salary} a season + {bonus} a point")
        .replace("{salary}", &progress::money(u64::from(salary)))
        .replace("{bonus}", &progress::money(u64::from(o.bonus_k)))
}

/// Where the talks stand, for the line under an offer; `None` while nothing was asked.
fn talks_status(o: &Offer) -> Option<String> {
    match o.status {
        OfferStatus::Agreed if o.kind != OfferKind::Contract => o.agreed.map(|t| {
            tr_owned("Agreed: {terms}. Sign it to make it count.").replace("{terms}", &terms_text(t))
        }),
        OfferStatus::Agreed => None,
        OfferStatus::Withdrawn => {
            Some(tr("They walked away from the talks and withdrew the offer.").to_owned())
        }
        OfferStatus::Countered => o.counter.map(|t| {
            let left = if o.attempts == 1 {
                tr("1 more attempt before they walk away.").to_owned()
            } else {
                tr_owned("{n} more attempts before they walk away.").replace("{n}", &o.attempts.to_string())
            };
            tr_owned("They counter: {terms}. {left} Signing now takes their first offer.")
                .replace("{terms}", &terms_text(t))
                .replace("{left}", &left)
        }),
        OfferStatus::Open if o.kind != OfferKind::Contract && o.attempts == 0 => {
            Some(tr("They won't talk any more; their first offer stands.").to_owned())
        }
        OfferStatus::Open => None,
    }
}

impl Shell {
    /// The talks about one offer: the terms to ask for (salary, length, number-one clause), where
    /// the talks stand, and **Ask**.
    fn career_talks(&self, year: u16, o: &Offer, p: Palette, cx: &mut Context<Self>) -> Div {
        let busy = self.career.busy.is_some();
        let key: TalkKey = (year, o.slot, o.early);
        let default = ask_of(o, None);
        let ask = self.career.talks.ask(key, default);
        let slot = o.slot;
        let id = |what: &str| SharedString::from(format!("career-talk-{year}-{slot}-{}-{what}", o.early));
        let step = |what: String, change: Change, label: String| {
            Button::new(id(&what))
                .small()
                .label(label)
                .disabled(busy)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.career.talks.change(key, default, change);
                    cx.notify();
                }))
        };
        let status = talks_status(o);
        let counter = o.counter.filter(|_| o.status == OfferStatus::Countered);
        let controls = negotiable(o).then(|| {
            h_flex()
                .gap(px(6.))
                .items_center()
                .flex_wrap()
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(p.text_muted)
                        .child(tr("Ask for:")),
                )
                .children(SALARY_STEPS.iter().take(2).map(|&d| {
                    step(
                        format!("salary{d}"),
                        Change::Salary(d),
                        format!("−{}", progress::money(d.unsigned_abs().into())),
                    )
                    .outline()
                }))
                .child(
                    div()
                        .min_w(px(70.))
                        .text_size(px(13.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(progress::money(u64::from(ask.salary_k))),
                )
                .children(SALARY_STEPS.iter().skip(2).map(|&d| {
                    step(
                        format!("salary{d}"),
                        Change::Salary(d),
                        format!("+{}", progress::money(d.unsigned_abs().into())),
                    )
                    .outline()
                }))
                .children(market::YEARS.map(|y| {
                    let b = step(format!("years{y}"), Change::Years(y), years_text(y));
                    if ask.years == y { b.primary() } else { b.outline() }
                }))
                .child({
                    let b = step(
                        "number-one".to_owned(),
                        Change::NumberOne,
                        tr("Number-one clause").to_owned(),
                    )
                    .when(ask.number_one, |b| b.icon(IconName::Check));
                    if ask.number_one { b.primary() } else { b.outline() }
                })
                .child(
                    Button::new(id("ask"))
                        .primary()
                        .small()
                        .label(tr("Ask for these terms"))
                        .disabled(busy)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            let terms = this.career.talks.ask(key, default);
                            this.career_act(Action::Negotiate { slot, terms }, cx)
                        })),
                )
                .when_some(counter, |d, t| {
                    d.child(
                        Button::new(id("counter"))
                            .outline()
                            .small()
                            .label(tr("Take their counter"))
                            .disabled(busy)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.career_act(Action::Negotiate { slot, terms: t }, cx)
                            })),
                    )
                })
        });
        v_flex()
            .gap(px(4.))
            .when_some(status, |d, s| {
                let color = match o.status {
                    OfferStatus::Agreed => p.success,
                    OfferStatus::Withdrawn => p.error,
                    _ => p.warning,
                };
                d.child(div().text_size(px(12.)).text_color(color).child(s))
            })
            .when_some(controls, |d, c| d.child(c))
    }

    /// The seat buttons for a team next season: "Take Vettel's seat" for each driver, the seats
    /// other players signed for already greyed out.
    fn career_seat_buttons(
        &self,
        league: &League,
        me: &str,
        slot: u8,
        id: &str,
        act: fn(u8, String, Option<Focus>) -> Action,
        cx: &mut Context<Self>,
    ) -> Vec<Button> {
        let busy = self.career.busy.is_some();
        let team =
            league::catalog_year(league.year() + 1).and_then(|c| c.teams.iter().find(|t| t.slot == slot));
        let signed = league.player(me).and_then(|pl| pl.signed.as_ref());
        let taken: Vec<&str> = league
            .players
            .iter()
            .filter(|pl| pl.id != me)
            .filter_map(|pl| pl.signed.as_ref())
            .filter(|s| s.slot == slot)
            .map(|s| s.replaces.as_str())
            .collect();
        let focus = self.career.focus;
        team.map(|t| t.drivers.clone())
            .unwrap_or_default()
            .into_iter()
            .map(|d| {
                let key = d.key.clone();
                let mine = signed.is_some_and(|s| s.slot == slot && s.replaces == d.key);
                let b = Button::new(SharedString::from(format!("{id}-{slot}-{}", d.key)))
                    .small()
                    .label(tr_owned("Take {name}'s seat").replace("{name}", &d.name))
                    .disabled(busy || taken.contains(&d.key.as_str()))
                    .on_click(
                        cx.listener(move |this, _, _, cx| this.career_act(act(slot, key.clone(), focus), cx)),
                    );
                if mine { b.primary() } else { b.outline() }
            })
            .collect()
    }

    pub(super) fn career_offers(
        &self,
        league: &League,
        me: &str,
        owner: bool,
        p: Palette,
        cx: &mut Context<Self>,
    ) -> Div {
        let busy = self.career.busy.is_some();
        let year = league.year();
        let next = year + 1;
        let cat = league::catalog_year(next);
        let focus_row = self.career_focus(p, cx);
        let offers: Vec<Div> = league
            .offers(me)
            .into_iter()
            .map(|o| {
                let team = cat.and_then(|c| c.teams.iter().find(|t| t.slot == o.slot));
                let name = team.map_or("?".to_owned(), |t| t.name.clone());
                let pay = pay_text(o);
                let kind = match o.kind {
                    OfferKind::Contract if o.early => tr("Signed during the season · {years} · {pay}"),
                    OfferKind::Contract => tr("Under contract: {years} more · {pay}"),
                    OfferKind::Renew => tr("Your team keeps you · {years} · {pay}"),
                    OfferKind::Move => tr("Offer · {years} · {pay}"),
                }
                .replace("{years}", &years_text(o.years))
                .replace("{pay}", &pay);
                let seats = self.career_seat_buttons(league, me, o.slot, "sign", sign, cx);
                v_flex()
                    .gap(px(4.))
                    .py(px(6.))
                    .border_t_1()
                    .border_color(p.border)
                    .child(
                        h_flex()
                            .gap(px(10.))
                            .items_center()
                            .child(
                                v_flex()
                                    .w(px(260.))
                                    .child(
                                        div()
                                            .text_size(px(13.))
                                            .font_weight(FontWeight::MEDIUM)
                                            .child(name),
                                    )
                                    .child(div().text_size(px(11.)).text_color(p.text_muted).child(kind)),
                            )
                            .children(seats),
                    )
                    .child(self.career_talks(year, o, p, cx))
            })
            .collect();
        let waiting: Vec<String> = league
            .players
            .iter()
            .filter(|pl| pl.signed.is_none())
            .map(|pl| pl.name.clone())
            .collect();
        v_flex()
            .gap(px(8.))
            .child(
                div()
                    .text_size(px(14.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(
                        tr_owned("The {year} season is over. Sign for {next}:")
                            .replace("{year}", &year.to_string())
                            .replace("{next}", &next.to_string()),
                    ),
            )
            .child(div().text_size(px(12.)).text_color(p.text_muted).child(tr(
                "Offers follow how you did against your car and your team-mate, and how much your team values \
                 you: a team that doesn't value you lets your contract run out. Under contract, only faster teams can buy \
                 you out; longer deals keep you from the market.",
            )))
            .child(div().text_size(px(12.)).text_color(p.text_muted).child(talks_help()))
            .child(focus_row)
            .child(v_flex().children(offers))
            .child(
                h_flex()
                    .gap(px(10.))
                    .items_center()
                    .child(div().flex_1().text_size(px(12.)).text_color(p.text_muted).child(
                        if league.players.len() == 1 {
                            tr_owned("Sign an offer and the {next} season starts.").replace("{next}", &next.to_string())
                        } else if waiting.is_empty() {
                            tr("Everyone signed.").to_owned()
                        } else {
                            tr_owned("Waiting for {names} to sign. The next season starts when everyone has.")
                                .replace("{names}", &waiting.join(", "))
                        },
                    ))
                    .when(owner && !waiting.is_empty() && league.players.len() > 1, |d| {
                        d.child(
                            Button::new("career-next-season")
                                .outline()
                                .small()
                                .label(tr("Start next season anyway"))
                                .disabled(busy)
                                .on_click(
                                    cx.listener(|this, _, _, cx| this.career_act(Action::NextSeason, cx)),
                                ),
                        )
                    })
                    .child(
                        Button::new("career-undo-end")
                            .outline()
                            .small()
                            .icon(IconName::Undo)
                            .label(tr("Take back the last round"))
                            .disabled(busy)
                            .on_click(cx.listener(|this, _, _, cx| this.career_act(Action::Undo, cx))),
                    ),
            )
    }

    /// Teams that approached the player this season: meet them in secret to hear their offer,
    /// talk about it, sign early, or turn them down. `None` without any approach.
    pub(super) fn career_approaches(
        &self,
        league: &League,
        me: &str,
        p: Palette,
        cx: &mut Context<Self>,
    ) -> Option<Div> {
        if league.phase != Phase::Racing {
            return None;
        }
        let approaches: Vec<&league::Approach> = market::approaches_of(league, me)
            .into_iter()
            .filter(|a| a.status != ApproachStatus::Expired)
            .collect();
        if approaches.is_empty() {
            return None;
        }
        let busy = self.career.busy.is_some();
        let year = league.year();
        let next = year + 1;
        let cat = league::catalog_year(next);
        let under_contract = league
            .player(me)
            .and_then(|pl| pl.contract_until)
            .is_some_and(|y| y > year);
        let signed_early = approaches.iter().any(|a| a.status == ApproachStatus::Accepted);
        let rows = approaches.into_iter().map(|a| {
            let slot = a.slot;
            let team = cat.and_then(|c| c.teams.iter().find(|t| t.slot == slot));
            let name = team.map_or_else(|| "?".to_owned(), |t| t.name.clone());
            let decline = Button::new(SharedString::from(format!("career-approach-decline-{slot}")))
                .outline()
                .small()
                .label(tr("Not interested"))
                .disabled(busy)
                .on_click(cx.listener(move |this, _, _, cx| this.career_act(Action::DeclineApproach { slot }, cx)));
            let row = v_flex().gap(px(4.)).py(px(6.)).border_t_1().border_color(p.border);
            match a.status {
                ApproachStatus::Interest => row.child(
                    h_flex()
                        .gap(px(10.))
                        .items_center()
                        .flex_wrap()
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(220.))
                                .text_size(px(13.))
                                .child(tr_owned(market::MAIL_MEET).replace("{team}", &name)),
                        )
                        .child(
                            Button::new(SharedString::from(format!("career-approach-meet-{slot}")))
                                .primary()
                                .small()
                                .label(tr("Meet them in secret"))
                                .disabled(busy)
                                .on_click(cx.listener(move |this, _, _, cx| this.career_act(Action::Meet { slot }, cx))),
                        )
                        .child(decline),
                ),
                ApproachStatus::Met => {
                    let offer = a.offer.as_ref();
                    let text = offer.map_or_else(
                        || tr_owned("You met {team}.").replace("{team}", &name),
                        |o| {
                            tr_owned("{team} offers {years} from {next}: {pay}.")
                                .replace("{team}", &name)
                                .replace("{years}", &years_text(o.years))
                                .replace("{next}", &next.to_string())
                                .replace("{pay}", &pay_text(o))
                        },
                    );
                    let open = offer.is_some_and(|o| o.status != OfferStatus::Withdrawn);
                    let seats = if open {
                        self.career_seat_buttons(league, me, slot, "career-approach-sign", accept_early, cx)
                    } else {
                        Vec::new()
                    };
                    row.child(div().text_size(px(13.)).font_weight(FontWeight::MEDIUM).child(text))
                        .when(a.leaked, |d| {
                            d.child(div().text_size(px(12.)).text_color(p.warning).child(tr(
                                "Word of the meeting got out, and your team isn't pleased.",
                            )))
                        })
                        .when_some(offer, |d, o| d.child(self.career_talks(year, o, p, cx)))
                        .when(open && under_contract, |d| {
                            d.child(div().text_size(px(12.)).text_color(p.warning).child(
                                tr_owned(
                                    "You're under contract beyond {year}: signing is a buy-out, and your team will \
                                     think less of you.",
                                )
                                .replace("{year}", &year.to_string()),
                            ))
                        })
                        .child(
                            h_flex()
                                .gap(px(6.))
                                .items_center()
                                .flex_wrap()
                                .when(open, |d| {
                                    d.child(
                                        div()
                                            .text_size(px(12.))
                                            .text_color(p.text_muted)
                                            .child(tr("Sign early:")),
                                    )
                                })
                                .children(seats)
                                .child(decline),
                        )
                }
                ApproachStatus::Accepted => row.child(
                    div().text_size(px(13.)).text_color(p.success).child(
                        tr_owned("You signed early with {team} for {next}.")
                            .replace("{team}", &name)
                            .replace("{next}", &next.to_string()),
                    ),
                ),
                ApproachStatus::Declined | ApproachStatus::Expired => row.child(
                    div()
                        .text_size(px(12.))
                        .text_color(p.text_muted)
                        .child(tr_owned("You turned {team} down.").replace("{team}", &name)),
                ),
            }
        });
        Some(
            v_flex()
                .p(px(14.))
                .gap(px(6.))
                .rounded(px(6.))
                .border_1()
                .border_color(p.accent)
                .child(
                    div()
                        .text_size(px(14.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(tr_owned("Teams want you for {next}").replace("{next}", &next.to_string())),
                )
                .when(!signed_early, |d| {
                    d.child(
                        div()
                            .text_size(px(12.))
                            .text_color(p.text_muted)
                            .child(leak_warning()),
                    )
                })
                .children(rows),
        )
    }

    /// What the player asks for in the contract they sign.
    pub(super) fn career_focus(&self, p: Palette, cx: &mut Context<Self>) -> Div {
        let focus = self.career.focus;
        let choice = |id: &'static str, text: &'static str, f: Option<Focus>| {
            let b = Button::new(id)
                .small()
                .label(text)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.career.focus = f;
                    cx.notify();
                }));
            if focus == f { b.primary() } else { b.outline() }
        };
        h_flex()
            .gap(px(6.))
            .items_center()
            .flex_wrap()
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(p.text_muted)
                    .child(tr("In the new contract, ask for:")),
            )
            .child(choice("career-focus-none", tr("Nothing extra"), None))
            .child(choice(
                "career-focus-salary",
                tr("More salary (+25%)"),
                Some(Focus::Salary),
            ))
            .child(choice(
                "career-focus-development",
                tr("A say in the development (+20% resource points)"),
                Some(Focus::Development),
            ))
    }

    pub(super) fn career_finished(&self, league: &League, p: Palette) -> Div {
        let champions: Vec<String> = league
            .seasons
            .iter()
            .filter_map(|s| {
                let d = league::driver_standings(s);
                d.first().map(|c| format!("{}: {}", s.year, c.name))
            })
            .collect();
        v_flex()
            .p(px(14.))
            .gap(px(6.))
            .rounded(px(6.))
            .border_1()
            .border_color(p.accent)
            .child(
                div()
                    .text_size(px(16.))
                    .font_weight(FontWeight::BOLD)
                    .child(tr("The career is complete")),
            )
            .children(champions.into_iter().map(|t| div().text_size(px(13.)).child(t)))
    }
}

fn sign(slot: u8, replaces: String, focus: Option<Focus>) -> Action {
    Action::Sign {
        slot,
        replaces: Some(replaces),
        focus,
    }
}

fn accept_early(slot: u8, replaces: String, _focus: Option<Focus>) -> Action {
    Action::AcceptApproach {
        slot,
        replaces: Some(replaces),
    }
}

/// How the talks work.
fn talks_help() -> String {
    tr_owned(
        "You can ask for other terms: a team pays more the better you are against its drivers, the more \
         you're valued and the more offers you have. Ask too much and it counters; after {attempts} tries it \
         walks away, though your own team keeps its first offer. A number-one clause costs {cost}% of what a team \
         would pay and needs a rating at least your team-mate's.",
    )
    .replace("{attempts}", &Offer::fresh().attempts.to_string())
    .replace("{cost}", &format!("{:.0}", market::NUMBER_ONE_COST * 100.0))
}

/// The warning before a secret meeting.
fn leak_warning() -> String {
    tr_owned(
        "Meet a team in secret to hear its offer. Word of a meeting gets out {chance}% of the time, and your \
         team then thinks less of you.",
    )
    .replace("{chance}", &format!("{:.0}", market::LEAK * 100.0))
}

#[cfg(test)]
mod tests {
    use swofty_core::league::{Offer, OfferKind, OfferStatus, Terms};

    use super::{Change, Talks, ask_of, changed, negotiable};

    fn offer() -> Offer {
        Offer {
            slot: 4,
            kind: OfferKind::Move,
            salary_k: 2_000,
            bonus_k: 20,
            years: 2,
            ..Offer::fresh()
        }
    }

    #[test]
    fn negotiation_inputs_build_terms() {
        let o = offer();
        let first = Terms {
            salary_k: 2_000,
            years: 2,
            number_one: false,
        };
        assert_eq!(ask_of(&o, None), first);
        // The salary moves in steps of 50k, never below the teams' floor.
        let more = changed(first, Change::Salary(250));
        assert_eq!(more.salary_k, 2_250);
        assert_eq!(changed(more, Change::Salary(-50)).salary_k, 2_200);
        assert_eq!(changed(first, Change::Salary(-5_000)).salary_k, 100);
        assert_eq!(changed(first, Change::Salary(99_000)).salary_k, 50_000);
        // The length stays within 1 to 3 years.
        assert_eq!(changed(first, Change::Years(3)).years, 3);
        assert_eq!(changed(first, Change::Years(9)).years, 3);
        assert_eq!(changed(first, Change::Years(0)).years, 1);
        // The clause toggles.
        let one = changed(first, Change::NumberOne);
        assert!(one.number_one);
        assert!(!changed(one, Change::NumberOne).number_one);
        // A counter is what the player asks for next, until they change it.
        let counter = Terms {
            salary_k: 2_400,
            years: 3,
            number_one: true,
        };
        let countered = Offer {
            status: OfferStatus::Countered,
            counter: Some(counter),
            attempts: 2,
            ..o.clone()
        };
        assert_eq!(ask_of(&countered, None), counter);
        assert_eq!(ask_of(&countered, Some(&more)), more);
        // The talks keep each offer's terms apart, by season, team and kind.
        let mut talks = Talks::default();
        talks.change((2017, 4, false), first, Change::Salary(250));
        talks.change((2017, 4, false), first, Change::Years(3));
        assert_eq!(
            talks.ask((2017, 4, false), first),
            Terms {
                salary_k: 2_250,
                years: 3,
                number_one: false,
            }
        );
        assert_eq!(talks.ask((2017, 4, true), first), first);
        assert_eq!(talks.ask((2018, 4, false), first), first);
    }

    #[test]
    fn only_open_talks_take_terms() {
        let o = offer();
        assert!(negotiable(&o));
        assert!(!negotiable(&Offer {
            kind: OfferKind::Contract,
            ..o.clone()
        }));
        assert!(!negotiable(&Offer {
            attempts: 0,
            ..o.clone()
        }));
        assert!(!negotiable(&Offer {
            status: OfferStatus::Agreed,
            ..o.clone()
        }));
        assert!(!negotiable(&Offer {
            status: OfferStatus::Withdrawn,
            ..o
        }));
    }
}
