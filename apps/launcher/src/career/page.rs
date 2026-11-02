//! The Career page's frame: the switcher, the telemetry line, the cards and the career view.

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{Sizable, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use swofty_core::league::solo;
use swofty_core::league::{Phase, udp};

use crate::i18n::{tr, tr_owned};
use crate::pages::GUTTER;
use crate::shell::{Page, Shell};
use crate::theme::{Palette, ScrollArea};

impl Shell {
    pub(crate) fn career_page(&mut self, p: Palette, cx: &mut Context<Self>) -> AnyElement {
        let subtitle =
            tr("Your own career from 2016 to 2026, alone or with friends: qualifying and races count from the \
                        game by themselves, contracts move you between teams each winter.")
                .to_owned();
        let signed_in = self.career_api().is_some();
        let solo_shown = self.career.solo() && self.career.league().is_some();
        let body: AnyElement = if solo_shown && !self.career.creating {
            self.career_view(p, cx).into_any_element()
        } else if !signed_in || self.career.creating && self.career.solo_form {
            self.career_start_cards(p, cx).into_any_element()
        } else if !self.career.loaded {
            div()
                .text_size(px(13.))
                .text_color(p.text_muted)
                .child(
                    self.career
                        .error
                        .clone()
                        .unwrap_or_else(|| tr("Loading your careers…").into()),
                )
                .into_any_element()
        } else if self.career.league().is_none() || self.career.creating {
            self.career_start_cards(p, cx).into_any_element()
        } else {
            self.career_view(p, cx).into_any_element()
        };
        v_flex()
            .size_full()
            .child(self.header(Page::Career.label(), subtitle, self.career_switcher(p, cx), p))
            .child(
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .scroll_y("career")
                    .px(px(GUTTER))
                    .pt(px(4.))
                    .pb(px(24.))
                    .gap(px(16.))
                    .child(self.career_telemetry_line(p, cx))
                    .when_some(self.career.note.clone(), |d, n| {
                        d.child(div().text_size(px(13.)).text_color(p.warning).child(n))
                    })
                    .when_some(self.career_unstarted(p, cx), |d, u| d.child(u))
                    .child(body),
            )
            .into_any_element()
    }

    pub(super) fn career_switcher(&self, _p: Palette, cx: &mut Context<Self>) -> Div {
        let c = &self.career;
        let mut row = h_flex().gap(px(6.)).flex_wrap();
        let all: Vec<(String, String)> = c
            .solos
            .iter()
            .map(|s| (s.id.clone(), tr_owned("{name} (solo)").replace("{name}", &s.name)))
            .chain(c.careers.iter().map(|s| (s.id.clone(), s.name.clone())))
            .collect();
        if all.len() > 1 {
            for (i, (id, name)) in all.into_iter().enumerate() {
                let on = c.selected.as_deref() == Some(id.as_str());
                let b = Button::new(("career-pick", i))
                    .small()
                    .label(name)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.career.selected = Some(id.clone());
                        this.career.state = None;
                        this.career.creating = false;
                        this.career.league_at = None;
                        this.career.refused = None;
                        if solo::is_solo(&id) {
                            let root = this.store.root.clone();
                            if let Ok(s) = solo::load(&root, &id) {
                                this.career.show_solo(s, Vec::new(), &root.join(udp::RACES_DIR));
                            }
                            cx.notify();
                        } else {
                            this.career_load(cx);
                        }
                    }));
                row = row.child(if on { b.primary() } else { b.outline() });
            }
        }
        if c.league().is_some() {
            let creating = c.creating;
            row = row.child(
                Button::new("career-new")
                    .outline()
                    .small()
                    .icon(if creating { IconName::X } else { IconName::Plus })
                    .label(if creating {
                        tr("Back to the career")
                    } else {
                        tr("New career")
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.career.creating = !creating;
                        cx.notify();
                    })),
            );
        }
        row
    }

    pub(super) fn career_telemetry_line(&self, p: Palette, cx: &mut Context<Self>) -> Div {
        let c = &self.career;
        let text = match (&c.telemetry, &c.listener, &c.listen_error) {
            (_, _, Some(e)) => e.clone(),
            (_, Some(l), _) => {
                let s = l.status();
                if let Some((id, size)) = s.unread {
                    tr_owned(
                        "Race results: the game sends a telemetry packet this launcher can't read (packet {id}, \
                         {size} bytes). Set the telemetry format to 2025 in the game (Settings > Telemetry).",
                    )
                    .replace("{id}", &id.to_string())
                    .replace("{size}", &size.to_string())
                } else if s.datagrams > 0 {
                    tr_owned("Race results: the game is sending telemetry to {addr}.")
                        .replace("{addr}", &l.addr().to_string())
                } else {
                    tr_owned("Race results: listening on {addr} for the game's telemetry.")
                        .replace("{addr}", &l.addr().to_string())
                }
            }
            (Some(s), None, None) if s.usable() => tr("Race results: listening starts with your first league career.").into(),
            (Some(_), None, None) | (None, None, None) => tr(
                "Race results need the game's UDP telemetry. Turn it on here (with the game closed) or in the game \
                 under Settings > Telemetry.",
            )
            .into(),
        };
        let usable = c.telemetry.as_ref().is_some_and(udp::Settings::usable);
        h_flex()
            .gap(px(10.))
            .items_center()
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_size(px(12.))
                    .text_color(if c.listen_error.is_some() {
                        p.error
                    } else {
                        p.text_muted
                    })
                    .child(text),
            )
            .when(!usable, |d| {
                d.child(
                    Button::new("career-telemetry")
                        .outline()
                        .small()
                        .label(tr("Turn telemetry on"))
                        .on_click(cx.listener(|this, _, _, cx| this.career_enable_telemetry(cx))),
                )
            })
    }

    pub(super) fn career_card(&self, title: &'static str, body: &'static str, p: Palette) -> Div {
        v_flex()
            .flex_1()
            .min_w(px(300.))
            .p(px(16.))
            .gap(px(10.))
            .rounded(px(6.))
            .border_1()
            .border_color(p.border)
            .child(
                div()
                    .text_size(px(14.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(title),
            )
            .child(div().text_size(px(13.)).text_color(p.text_muted).child(body))
    }

    pub(super) fn career_view(&self, p: Palette, cx: &mut Context<Self>) -> Div {
        let c = &self.career;
        let league = c.league().expect("checked");
        let me = if c.solo() {
            solo::ME.to_owned()
        } else {
            c.me.clone().unwrap_or_default()
        };
        let owner = c.solo() || c.career().is_some_and(|x| x.owner == me);
        v_flex()
            .gap(px(16.))
            .when_some(c.error.clone(), |d, e| {
                d.child(div().text_size(px(13.)).text_color(p.error).child(e))
            })
            .child(self.career_banner(league, p, cx))
            .child(match league.phase {
                Phase::Setup => self.career_seats(league, &me, owner, p, cx),
                Phase::Racing => self.career_next(league, &me, p, cx),
                Phase::SeasonEnd => self.career_offers(league, &me, owner, p, cx),
                Phase::Finished => self.career_finished(league, p),
            })
            .when_some(self.career_profile_card(league, &me, p, cx), |d, card| {
                d.child(card)
            })
            .when_some(self.career_approaches(league, &me, p, cx), |d, card| {
                d.child(card)
            })
            .when(league.phase == Phase::Racing, |d| {
                d.child(self.career_weekend(league, &me, p))
            })
            .child(self.career_game_sync(league, &me, owner, p, cx))
            .child(self.career_options(league, owner, p, cx))
            .when(league.phase != Phase::Setup, |d| {
                d.child(self.career_tabs(league, &me, p, cx))
            })
    }
}
