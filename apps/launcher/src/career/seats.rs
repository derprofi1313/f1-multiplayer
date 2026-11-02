//! The seat choice before a season starts.

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{Disableable, Sizable, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use swofty_core::league::{self, Action, League, TeamChoice};

use crate::i18n::{tr, tr_owned};
use crate::shell::Shell;
use crate::theme::Palette;

impl Shell {
    pub(super) fn career_seats(
        &self,
        league: &League,
        me: &str,
        owner: bool,
        p: Palette,
        cx: &mut Context<Self>,
    ) -> Div {
        let season = league.season();
        let Some(cat) = league::catalog_year(season.year) else {
            return v_flex();
        };
        let busy = self.career.busy.is_some();
        let mut teams: Vec<&league::CatalogTeam> = cat.teams.iter().collect();
        teams.sort_by(|a, b| a.pace.total_cmp(&b.pace));
        let n = teams.len();
        let rows = teams
            .into_iter()
            .enumerate()
            .map(|(rank, t)| {
                let slot = t.slot;
                let locked = league.setup.team_choice == TeamChoice::Rookie && (rank + 1) * 2 <= n;
                h_flex()
                    .gap(px(10.))
                    .py(px(6.))
                    .items_center()
                    .border_t_1()
                    .border_color(p.border)
                    .child(
                        div()
                            .w(px(28.))
                            .text_size(px(12.))
                            .text_color(p.text_muted)
                            .child(format!("{}", rank + 1)),
                    )
                    .child(
                        div()
                            .w(px(150.))
                            .text_size(px(13.))
                            .font_weight(FontWeight::MEDIUM)
                            .child(t.name.clone()),
                    )
                    .child(
                        div()
                            .w(px(70.))
                            .text_size(px(12.))
                            .text_color(p.text_muted)
                            .child(format!("+{:.2}%", t.pace)),
                    )
                    .children(t.drivers.iter().map(|d| {
                        let holder = league.players.iter().find(|pl| {
                            pl.seat
                                .as_ref()
                                .is_some_and(|s| s.slot == slot && s.replaces == d.key)
                        });
                        let key = d.key.clone();
                        let text = match holder {
                            Some(pl) => tr_owned("{driver} (now {name})")
                                .replace("{driver}", &d.name)
                                .replace("{name}", &pl.name),
                            None => d.name.clone(),
                        };
                        let mine = holder.is_some_and(|pl| pl.id == me);
                        let b = Button::new(SharedString::from(format!("seat-{slot}-{}", d.key)))
                            .small()
                            .label(text)
                            .disabled(busy || locked || (holder.is_some() && !mine))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.career_act(
                                    Action::Seat {
                                        slot,
                                        replaces: Some(key.clone()),
                                    },
                                    cx,
                                )
                            }));
                        if mine { b.primary() } else { b.outline() }
                    }))
            })
            .collect::<Vec<_>>();
        let ready = league.players.iter().all(|pl| pl.seat.is_some());
        // Players without a seat hold up the start; the owner can take them out of the career.
        let unseated: Vec<(String, String)> = league
            .players
            .iter()
            .filter(|pl| pl.seat.is_none() && !pl.left)
            .map(|pl| (pl.id.clone(), pl.name.clone()))
            .collect();
        let solo_career = self.career.solo();
        let unseated_rows = unseated
            .into_iter()
            .map(|(id, name)| {
                let mine = id == me;
                h_flex()
                    .gap(px(8.))
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(12.))
                            .text_color(p.text_muted)
                            .child(if mine {
                                tr("You haven't picked a seat yet.").to_owned()
                            } else {
                                tr_owned("{name} hasn't picked a seat yet.").replace("{name}", &name)
                            }),
                    )
                    .when(owner && !mine && !solo_career, |d| {
                        d.child(self.career_remove_button(id, name, busy, cx))
                    })
            })
            .collect::<Vec<_>>();
        v_flex()
            .gap(px(8.))
            .child(
                div().text_size(px(14.)).font_weight(FontWeight::SEMIBOLD).child(
                    tr_owned("Pick your seat for {year}: you take that driver's place for the season")
                        .replace("{year}", &season.year.to_string()),
                ),
            )
            .when(league.setup.team_choice == TeamChoice::Rookie, |d| {
                d.child(div().text_size(px(12.)).text_color(p.text_muted).child(tr(
                    "Rookie start: the faster half of the grid opens up through contract offers later.",
                )))
            })
            .child(v_flex().children(rows))
            .children(unseated_rows)
            .child(
                h_flex()
                    .gap(px(10.))
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(12.))
                            .text_color(p.text_muted)
                            .child(if owner {
                                if ready && league.players.len() == 1 {
                                    tr("Your seat is set. Start the season when you're ready.")
                                } else if ready {
                                    tr("Everyone has a seat. Start the season when your friends are ready.")
                                } else {
                                    tr("Waiting for everyone to pick a seat.")
                                }
                            } else {
                                tr("The career's owner starts the season once everyone has a seat.")
                            }),
                    )
                    .when(owner, |d| {
                        d.child(
                            Button::new("career-begin")
                                .primary()
                                .small()
                                .icon(IconName::Flag)
                                .label(
                                    tr_owned("Start the {year} season")
                                        .replace("{year}", &season.year.to_string()),
                                )
                                .disabled(busy || !ready)
                                .on_click(cx.listener(|this, _, _, cx| this.career_act(Action::Begin, cx))),
                        )
                    }),
            )
    }
}
