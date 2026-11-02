//! The banner on top of a career: its name, season, phase, members and who is ready; the owner
//! removes players here, the others leave. Also the panel for a career that was created on the
//! host server but didn't start.

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{Disableable, Sizable, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use swofty_core::league::{League, Phase, progress};

use crate::i18n::{tr, tr_owned};
use crate::pages::label;
use crate::shell::Shell;
use crate::theme::Palette;

use super::next::ready_line;

impl Shell {
    pub(super) fn career_banner(&self, league: &League, p: Palette, cx: &mut Context<Self>) -> Div {
        let season = league.season();
        let done = season.rounds.iter().filter(|r| r.done()).count();
        let career = self.career.career();
        let code = career.map(|c| c.code.clone()).unwrap_or_default();
        let solo_career = self.career.solo();
        let me = self.career.me_id();
        let owner = self.career.owner();
        let busy = self.career.busy.is_some();
        let phase = match league.phase {
            Phase::Setup => tr("picking seats").to_owned(),
            Phase::Racing => tr_owned("round {round} of {rounds}")
                .replace("{round}", &(done + 1).min(season.rounds.len()).to_string())
                .replace("{rounds}", &season.rounds.len().to_string()),
            Phase::SeasonEnd => tr("contracts for next season").to_owned(),
            Phase::Finished => tr("complete").to_owned(),
        };
        let players: Vec<(String, String, String, bool)> = league
            .players
            .iter()
            .map(|pl| {
                let team = season
                    .player_entry(&pl.id)
                    .and_then(|e| season.team(e.slot))
                    .map_or_else(|| tr("no seat yet").to_owned(), |t| t.name.clone());
                let titles = if pl.titles.is_empty() {
                    String::new()
                } else {
                    tr_owned(" · {n}x champion").replace("{n}", &pl.titles.len().to_string())
                };
                let contract =
                    pl.contract_until
                        .filter(|_| league.phase != Phase::Setup)
                        .map_or(String::new(), |y| {
                            tr_owned(" · {salary} a season to {year}")
                                .replace("{salary}", &progress::money(u64::from(pl.salary_k)))
                                .replace("{year}", &y.to_string())
                        });
                let status = if pl.left {
                    tr(" · left the career")
                } else if pl.number_one {
                    tr(" · number one driver")
                } else {
                    ""
                };
                let text = tr_owned("{name} #{number} · {team} · rating {rating}{status}{contract}{titles}")
                    .replace("{name}", &pl.name)
                    .replace("{number}", &pl.number.to_string())
                    .replace("{team}", &team)
                    .replace("{rating}", &format!("{:.0}", pl.rating))
                    .replace("{status}", status)
                    .replace("{contract}", &contract)
                    .replace("{titles}", &titles);
                (pl.id.clone(), pl.name.clone(), text, pl.left)
            })
            .collect();
        // Members of the career on the host server who have no driver in it (their Enter failed).
        let driverless: Vec<(String, String)> = career
            .map(|c| {
                c.members
                    .iter()
                    .filter(|m| league.player(&m.id).is_none() && m.id != c.owner)
                    .map(|m| (m.id.clone(), m.name.clone()))
                    .collect()
            })
            .unwrap_or_default();
        let ready = (league.phase == Phase::Racing && league.active_players().len() >= 2)
            .then(|| season.next_round().map(|r| ready_line(league, r.round)))
            .flatten();
        let rows = players
            .into_iter()
            .map(|(id, name, text, left)| {
                let removable = !solo_career && owner && id != me && !left;
                h_flex()
                    .gap(px(8.))
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(13.))
                            .when(left, |d| d.text_color(p.text_muted))
                            .child(text),
                    )
                    .when(removable, |d| {
                        d.child(self.career_remove_button(id, name, busy, cx))
                    })
            })
            .collect::<Vec<_>>();
        let driverless_rows =
            driverless
                .into_iter()
                .map(|(id, name)| {
                    h_flex()
                        .gap(px(8.))
                        .items_center()
                        .child(
                            div().flex_1().text_size(px(13.)).text_color(p.text_muted).child(
                                tr_owned("{name} joined but has no driver yet").replace("{name}", &name),
                            ),
                        )
                        .when(owner, |d| d.child(self.career_remove_button(id, name, busy, cx)))
                })
                .collect::<Vec<_>>();
        let leaving = self.career.confirm.as_deref() == Some("leave");
        v_flex()
            .p(px(14.))
            .gap(px(8.))
            .rounded(px(6.))
            .border_1()
            .border_color(p.border)
            .bg(p.surface)
            .child(
                h_flex()
                    .gap(px(10.))
                    .items_center()
                    .child(
                        div()
                            .text_size(px(22.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(p.accent)
                            .child(season.year.to_string()),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .text_size(px(15.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(self.career.career_name()),
                            )
                            .child(
                                div().text_size(px(12.)).text_color(p.text_muted).child(
                                    tr_owned("{first} to {last} · {phase}{solo}")
                                        .replace("{first}", &league.setup.first_year.to_string())
                                        .replace("{last}", &league.setup.last_year.to_string())
                                        .replace("{phase}", &phase)
                                        .replace("{solo}", if solo_career { tr(" · solo") } else { "" }),
                                ),
                            ),
                    )
                    .when(!solo_career, |d| {
                        d.child(label(tr("CODE"), p))
                            .child(
                                div()
                                    .text_size(px(15.))
                                    .font_family("monospace")
                                    .child(code.clone()),
                            )
                            .child(
                                Button::new("career-copy-code")
                                    .outline()
                                    .small()
                                    .icon(IconName::Copy)
                                    .label(tr("Copy"))
                                    .on_click(
                                        cx.listener(move |this, _, _, cx| {
                                            this.mp_copy_code(code.clone(), cx)
                                        }),
                                    ),
                            )
                    })
                    .when(!solo_career && !owner, |d| {
                        d.child(
                            Button::new("career-leave")
                                .small()
                                .icon(IconName::LogOut)
                                .label(if leaving {
                                    tr("Really leave?")
                                } else {
                                    tr("Leave career")
                                })
                                .disabled(busy)
                                .map(|b| if leaving { b.danger() } else { b.outline() })
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    if leaving {
                                        this.career_leave(cx)
                                    } else {
                                        this.career.confirm = Some("leave".into());
                                        cx.notify();
                                    }
                                })),
                        )
                    }),
            )
            .when(leaving, |d| {
                d.child(
                    div().text_size(px(12.)).text_color(p.text_muted).child(tr(
                        "Your seat stays in the career, driven by the game's AI. You can join again with the code.",
                    )),
                )
            })
            .children(rows)
            .children(driverless_rows)
            .when_some(ready, |d, r| {
                d.child(div().text_size(px(12.)).text_color(p.text_muted).child(r))
            })
    }

    /// The owner's button to remove a player; the first click asks to confirm.
    pub(super) fn career_remove_button(
        &self,
        id: String,
        name: String,
        busy: bool,
        cx: &mut Context<Self>,
    ) -> Button {
        let key = format!("remove:{id}");
        let asking = self.career.confirm.as_deref() == Some(key.as_str());
        Button::new(SharedString::from(format!("career-remove-{id}")))
            .small()
            .icon(IconName::X)
            .label(if asking {
                tr_owned("Really remove {name}?").replace("{name}", &name)
            } else {
                tr("Remove").to_owned()
            })
            .disabled(busy)
            .map(|b| if asking { b.danger() } else { b.ghost() })
            .on_click(cx.listener(move |this, _, _, cx| {
                if asking {
                    this.career_remove(id.clone(), name.clone(), cx)
                } else {
                    this.career.confirm = Some(key.clone());
                    cx.notify();
                }
            }))
    }

    /// A career on the host server that was created but didn't start (its Start failed): try the
    /// start again with the form's settings, or (the owner) delete it.
    pub(super) fn career_unstarted(&self, p: Palette, cx: &mut Context<Self>) -> Option<Div> {
        let c = &self.career;
        if c.solo() || self.career_api().is_none() {
            return None;
        }
        let id = c.unstarted.clone().or_else(|| {
            let me = c.me.as_deref()?;
            c.career()
                .filter(|x| x.version == 0 && x.owner == me && c.league().is_none())
                .map(|x| x.id.clone())
        })?;
        let name = c.name_of(&id);
        let busy = c.busy.is_some();
        let deleting = c.confirm.as_deref() == Some("delete");
        let retry_id = id.clone();
        Some(
            v_flex()
                .p(px(12.))
                .gap(px(8.))
                .rounded(px(6.))
                .border_1()
                .border_color(p.warning)
                .child(
                    div().text_size(px(13.)).child(
                        tr_owned(
                            "The career {name} was made on the host server, but its season didn't start. Try \
                             again with the settings below, or delete it.",
                        )
                        .replace("{name}", &name),
                    ),
                )
                .child(
                    h_flex()
                        .gap(px(8.))
                        .child(
                            Button::new("career-retry-start")
                                .primary()
                                .small()
                                .icon(IconName::RotateCw)
                                .label(tr("Try again"))
                                .loading(c.busy == Some("start"))
                                .disabled(busy)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.career_retry_start(retry_id.clone(), cx)
                                })),
                        )
                        .child(
                            Button::new("career-delete-unstarted")
                                .small()
                                .icon(IconName::Trash)
                                .label(if deleting {
                                    tr("Really delete?")
                                } else {
                                    tr("Delete")
                                })
                                .disabled(busy)
                                .map(|b| if deleting { b.danger() } else { b.outline() })
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    if deleting {
                                        this.career_delete(id.clone(), cx)
                                    } else {
                                        this.career.confirm = Some("delete".into());
                                        cx.notify();
                                    }
                                })),
                        ),
                ),
        )
    }
}
