//! The tabs below the career: drivers, constructors and the calendar.

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{Sizable, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use swofty_core::league::{self, Action, League, NewsKind, RoundState, inbox};

use crate::i18n::{tr, tr_owned};
use crate::shell::Shell;
use crate::theme::Palette;

use super::Tab;

/// "1 win" or "3 wins" (`one` and `many` are `tr` keys; `{n}` stands for the count).
fn count(n: u32, one: &'static str, many: &'static str) -> String {
    if n == 1 {
        tr(one).to_owned()
    } else {
        tr_owned(many).replace("{n}", &n.to_string())
    }
}
use super::inbox::{news_text, read_upto};

impl Shell {
    pub(super) fn career_tabs(&self, league: &League, me: &str, p: Palette, cx: &mut Context<Self>) -> Div {
        let tab = self.career.tab;
        let season = league.season();
        let body = match tab {
            Tab::Drivers => self.drivers_table(season, me, p),
            Tab::Teams => self.teams_table(season, p),
            Tab::Calendar => self.calendar(season, me, p),
            Tab::Driver => self.career_driver(league, me, p),
            Tab::Development => self.career_development(league, me, p, cx),
            Tab::Record => self.career_record(league, me, p),
            // Mails are the addressee's own: they show in the inbox, not here.
            Tab::News => v_flex().children(
                league
                    .news
                    .iter()
                    .rev()
                    .filter(|n| n.kind == NewsKind::News)
                    .take(40)
                    .map(|n| {
                        h_flex()
                            .gap(px(10.))
                            .py(px(4.))
                            .border_t_1()
                            .border_color(p.border)
                            .child(
                                div()
                                    .w(px(40.))
                                    .text_size(px(12.))
                                    .text_color(p.text_muted)
                                    .child(n.year.to_string()),
                            )
                            .child(div().flex_1().text_size(px(13.)).child(news_text(n)))
                    }),
            ),
            Tab::Inbox => self.career_inbox(league, me, p, cx),
        };
        let button = |id: &'static str, text: &'static str, t: Tab| {
            let b = Button::new(id)
                .small()
                .label(text)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.career.tab = t;
                    cx.notify();
                }));
            if tab == t { b.primary() } else { b.outline() }
        };
        let unread = inbox::unread(league, me).len();
        let inbox_button = {
            let upto = read_upto(league, me);
            let b = Button::new("career-tab-inbox")
                .small()
                .label(tr("Inbox"))
                .when(unread > 0, |b| {
                    b.child(
                        div()
                            .ml(px(6.))
                            .h(px(16.))
                            .min_w(px(16.))
                            .px(px(4.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(8.))
                            .text_size(px(10.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .bg(if tab == Tab::Inbox { p.on_accent } else { p.accent })
                            .text_color(if tab == Tab::Inbox { p.accent } else { p.on_accent })
                            .child(unread.to_string()),
                    )
                })
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.career.tab = Tab::Inbox;
                    // Opening the inbox reads it.
                    if let Some(upto) = upto {
                        this.career_act(Action::ReadMail { upto }, cx);
                    }
                    cx.notify();
                }));
            if tab == Tab::Inbox {
                b.primary()
            } else {
                b.outline()
            }
        };
        v_flex()
            .gap(px(10.))
            .child(
                h_flex()
                    .gap(px(6.))
                    .flex_wrap()
                    .child(button("career-tab-drivers", tr("Drivers"), Tab::Drivers))
                    .child(button("career-tab-teams", tr("Constructors"), Tab::Teams))
                    .child(button("career-tab-calendar", tr("Calendar"), Tab::Calendar))
                    .child(button("career-tab-driver", tr("Your driver"), Tab::Driver))
                    .child(button("career-tab-development", tr("R&D"), Tab::Development))
                    .child(button("career-tab-record", tr("Your career"), Tab::Record))
                    .child(button("career-tab-news", tr("News"), Tab::News))
                    .child(inbox_button),
            )
            .child(body)
    }

    pub(super) fn drivers_table(&self, season: &league::Season, me: &str, p: Palette) -> Div {
        let rows = league::driver_standings(season);
        v_flex().children(rows.into_iter().enumerate().map(|(i, d)| {
            let team = season.team(d.slot).map_or(String::new(), |t| t.name.clone());
            let human = d.player.is_some();
            let mine = d.player.as_deref() == Some(me);
            h_flex()
                .gap(px(10.))
                .py(px(4.))
                .px(px(6.))
                .border_t_1()
                .border_color(p.border)
                .when(human, |d| d.bg(p.selected))
                .child(
                    div()
                        .w(px(28.))
                        .text_size(px(13.))
                        .text_color(p.text_muted)
                        .child(format!("{}", i + 1)),
                )
                .child(
                    div()
                        .w(px(190.))
                        .text_size(px(13.))
                        .when(human, |d| d.font_weight(FontWeight::SEMIBOLD))
                        .when(mine, |d| d.text_color(p.accent))
                        .child(d.name.clone()),
                )
                .child(
                    div()
                        .w(px(150.))
                        .text_size(px(12.))
                        .text_color(p.text_muted)
                        .child(team),
                )
                .child(div().w(px(60.)).text_size(px(13.)).child(format!("{}", d.points)))
                .child(super::driver::form_cell(season, &d.entry, p))
                .child(
                    div()
                        .w(px(70.))
                        .text_size(px(12.))
                        .text_color(p.text_muted)
                        .child(count(d.wins, "1 win", "{n} wins")),
                )
                .child(
                    div()
                        .w(px(80.))
                        .text_size(px(12.))
                        .text_color(p.text_muted)
                        .child(count(d.podiums, "1 podium", "{n} podiums")),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(p.text_muted)
                        .child(count(d.poles, "1 pole", "{n} poles")),
                )
        }))
    }

    pub(super) fn teams_table(&self, season: &league::Season, p: Palette) -> Div {
        v_flex().children(
            league::team_standings(season)
                .into_iter()
                .enumerate()
                .map(|(i, t)| {
                    h_flex()
                        .gap(px(10.))
                        .py(px(4.))
                        .px(px(6.))
                        .border_t_1()
                        .border_color(p.border)
                        .child(
                            div()
                                .w(px(28.))
                                .text_size(px(13.))
                                .text_color(p.text_muted)
                                .child(format!("{}", i + 1)),
                        )
                        .child(div().w(px(190.)).text_size(px(13.)).child(t.name.clone()))
                        .child(div().w(px(60.)).text_size(px(13.)).child(format!("{}", t.points)))
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(p.text_muted)
                                .child(count(t.wins, "1 win", "{n} wins")),
                        )
                }),
        )
    }

    pub(super) fn calendar(&self, season: &league::Season, me: &str, p: Palette) -> Div {
        v_flex().children(season.rounds.iter().map(|r| {
            let winner = r
                .race_result
                .iter()
                .find(|c| c.position == 1)
                .and_then(|c| season.entry(&c.entry))
                .map(|e| e.name.clone());
            let practice = r
                .practice
                .iter()
                .find(|l| l.player == me)
                .map(|l| format!(" · {}", tr_owned("practice {grade}").replace("{grade}", &l.grade.to_string())))
                .unwrap_or_default();
            let state = match r.state {
                RoundState::Pending if !r.sprint_result.is_empty() => tr("sprint counted").to_owned(),
                RoundState::Pending if !r.quali_result.is_empty() => tr("qualifying counted").to_owned(),
                RoundState::Pending if !r.practice.is_empty() => tr("practice counted").to_owned(),
                RoundState::Pending => String::new(),
                RoundState::Raced => match &r.by {
                    Some(by) => tr_owned("raced ({name})").replace("{name}", by),
                    None => tr("raced").to_owned(),
                },
                RoundState::Simulated if r.auto => tr("simulated (not in the game)").to_owned(),
                RoundState::Simulated => tr("simulated").to_owned(),
                RoundState::Skipped => tr("left out").to_owned(),
            };
            h_flex()
                .gap(px(10.))
                .py(px(4.))
                .px(px(6.))
                .border_t_1()
                .border_color(p.border)
                .child(
                    div()
                        .w(px(28.))
                        .text_size(px(13.))
                        .text_color(p.text_muted)
                        .child(r.round.to_string()),
                )
                .child(div().w(px(230.)).text_size(px(13.)).child(if r.sprint {
                    format!("{} · {}", r.race, tr("sprint"))
                } else {
                    r.race.clone()
                }))
                .child(
                    div()
                        .w(px(260.))
                        .text_size(px(12.))
                        .text_color(p.text_muted)
                        .child(format!("{state}{practice}")),
                )
                .child(div().text_size(px(13.)).child(winner.unwrap_or_default()))
        }))
    }
}
