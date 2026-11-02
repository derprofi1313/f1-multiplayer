//! The cards to start a new career or join one.

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::Input;
use gpui_kit::component::{Disableable, Sizable, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use swofty_core::league::{self};

use crate::i18n::{tr, tr_owned};
use crate::shell::{Page, Shell};
use crate::theme::Palette;

use super::CareerPage;
use super::driver::Target;

impl Shell {
    pub(super) fn career_start_cards(&self, p: Palette, cx: &mut Context<Self>) -> Div {
        let c = &self.career;
        let busy = c.busy.is_some();
        // The driver to create, in the start and the join form.
        let start_driver = self.career_profile_picker(Target::Form, "career-start", c.profile, p, cx);
        let join_driver = self.career_profile_picker(Target::Form, "career-join", c.profile, p, cx);
        let toggle = |id: &'static str, text: &'static str, on: bool, f: fn(&mut CareerPage)| {
            let b = Button::new(id)
                .small()
                .label(text)
                .when(on, |b| b.icon(IconName::Check))
                .on_click(cx.listener(move |this, _, _, cx| {
                    f(&mut this.career);
                    cx.notify();
                }));
            if on { b.primary() } else { b.outline() }
        };
        let year = |id: &'static str, text: String, f: fn(&mut CareerPage, i32)| {
            h_flex()
                .gap(px(6.))
                .items_center()
                .child(div().w(px(110.)).text_size(px(13.)).child(text))
                .child(
                    Button::new((id, 0usize))
                        .outline()
                        .small()
                        .icon(IconName::Minus)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            f(&mut this.career, -1);
                            cx.notify();
                        })),
                )
                .child(
                    Button::new((id, 1usize))
                        .outline()
                        .small()
                        .icon(IconName::Plus)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            f(&mut this.career, 1);
                            cx.notify();
                        })),
                )
        };
        let distances = h_flex()
            .gap(px(6.))
            .items_center()
            .child(div().w(px(110.)).text_size(px(13.)).child(tr("Race distance")))
            .children([25u8, 50, 100].into_iter().map(|d| {
                let b = Button::new(("career-distance", d as usize))
                    .small()
                    .label(format!("{d}%"))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.career.distance = d;
                        cx.notify();
                    }));
                if c.distance == d { b.primary() } else { b.outline() }
            }));
        let signed_in = self.career_api().is_some();
        let solo_form = c.solo_form || !signed_in;
        let mode = h_flex()
            .gap(px(6.))
            .child({
                let b = Button::new("career-mode-solo")
                    .small()
                    .icon(IconName::User)
                    .label(tr("Alone"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.career.solo_form = true;
                        cx.notify();
                    }));
                if solo_form { b.primary() } else { b.outline() }
            })
            .child({
                let b = Button::new("career-mode-friends")
                    .small()
                    .icon(IconName::Users)
                    .label(tr("With friends"))
                    .disabled(!signed_in)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.career.solo_form = false;
                        cx.notify();
                    }));
                if solo_form { b.outline() } else { b.primary() }
            });
        let start = self
            .career_card(
                tr("Start a career"),
                if solo_form {
                    tr("Just you: kept on this PC, no server needed. Every round is a Grand Prix in the game (F1 World > \
                        Grand Prix) with that year's season box.")
                } else {
                    tr("On your host server: your friends join with its code, everyone picks a seat, then you start \
                        the first season. Every round is a LAN race.")
                },
                p,
            )
            .child(mode)
            .child(Input::new(&c.name).small())
            .when(solo_form, |d| d.child(Input::new(&c.player).small()))
            .child(Input::new(&c.number).small())
            .child(start_driver)
            .child(year("career-from", tr_owned("From {year}").replace("{year}", &c.first_year.to_string()), |c, d| {
                let y = (c.first_year as i32 + d).clamp(league::FIRST_YEAR as i32, c.last_year as i32);
                c.first_year = y as u16;
            }))
            .child(year("career-to", tr_owned("To {year}").replace("{year}", &c.last_year.to_string()), |c, d| {
                let y = (c.last_year as i32 + d).clamp(c.first_year as i32, league::LAST_YEAR as i32);
                c.last_year = y as u16;
            }))
            .child(distances)
            .child(
                h_flex()
                    .gap(px(6.))
                    .flex_wrap()
                    .child(toggle("career-sprints", tr("Sprints"), c.sprints, |c| c.sprints = !c.sprints))
                    .child(toggle(
                        "career-historic",
                        tr("Each year's points"),
                        c.historic,
                        |c| c.historic = !c.historic,
                    ))
                    .child(toggle(
                        "career-simulate",
                        tr("Simulate missing circuits"),
                        c.simulate_missing,
                        |c| c.simulate_missing = !c.simulate_missing,
                    ))
                    .child(toggle("career-rookie", tr("Rookie start"), c.rookie, |c| c.rookie = !c.rookie))
                    .child(toggle(
                        "career-lineups",
                        tr("Real driver changes"),
                        c.lineup_changes,
                        |c| c.lineup_changes = !c.lineup_changes,
                    )),
            )
            .child(
                Button::new("career-start")
                    .primary()
                    .small()
                    .icon(IconName::Flag)
                    .label(tr("Start career"))
                    .loading(c.busy == Some("start"))
                    .disabled(busy)
                    .on_click(cx.listener(|this, _, _, cx| this.career_create(cx))),
            );
        let join = if signed_in {
            self.career_card(
                tr("Join a friend's career"),
                tr("Type the career code your friend sent you (K-XXXXXX). You need to be signed in to their server."),
                p,
            )
            .child(Input::new(&c.code).small())
            .child(Input::new(&c.number).small())
            .child(join_driver)
            .child(
                Button::new("career-join")
                    .primary()
                    .small()
                    .label(tr("Join career"))
                    .loading(c.busy == Some("start"))
                    .disabled(busy)
                    .on_click(cx.listener(|this, _, _, cx| this.career_join(cx))),
            )
        } else {
            self.career_card(
                tr("Drive with friends"),
                tr("A career with friends lives on a host server. Host one or join a friend's on the Multiplayer \
                    page, then start or join it here."),
                p,
            )
            .child(
                Button::new("career-to-mp")
                    .outline()
                    .small()
                    .icon(IconName::Users)
                    .label(tr("Open Multiplayer"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.page = Page::Multiplayer;
                        cx.notify();
                    })),
            )
        };
        v_flex()
            .gap(px(12.))
            .when_some(c.error.clone(), |d, e| {
                d.child(div().text_size(px(13.)).text_color(p.error).child(e))
            })
            .child(
                h_flex()
                    .gap(px(16.))
                    .items_start()
                    .flex_wrap()
                    .child(start)
                    .child(join),
            )
    }
}
