use std::ops::Range;

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::Input;
use gpui_kit::component::menu::{DropdownMenu, PopupMenuItem};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Disableable, Icon, Sizable, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use swofty_core::dbedit::{HitKind, Kind, LinkSpec, Module};

use super::fields::{curve_rating, detail_shell, hex_rgb, is_on, list_items, swatch};
use super::{LinkState, Load, Nav, bool_text, clean, icon_named, kind_of, swap_field};
use crate::library::plural;
use crate::pages::{GUTTER, badge, label};
use crate::shell::Shell;
use crate::theme::{Palette, SCROLLBAR_GAP, ScrollArea, palette};

const ROW_H: f32 = 40.;
const NAME_W: f32 = 220.;
const MOVE_W: f32 = 56.;
const NARROW: f32 = 1100.;

pub(crate) fn notice(
    id: &'static str,
    icon: IconName,
    color: Hsla,
    text: String,
    p: Palette,
) -> Stateful<Div> {
    h_flex()
        .id(id)
        .px(px(12.))
        .py(px(8.))
        .gap(px(10.))
        .flex_shrink_0()
        .rounded(px(6.))
        .border_1()
        .border_color(p.border_variant)
        .bg(p.surface)
        .text_size(px(13.))
        .child(Icon::new(icon).size(px(14.)).text_color(color))
        .child(div().flex_1().min_w_0().text_color(p.text).child(text))
}

fn hit_group(kind: HitKind) -> &'static str {
    match kind {
        HitKind::ModuleRow => "Drivers, teams and tracks",
        HitKind::Added => "Driver Manager drivers",
        HitKind::Table => "Tables",
        HitKind::Row => "Rows",
        HitKind::Key => "Keys",
        HitKind::Value => "Values",
        HitKind::Text => "English text",
    }
}

fn hit_icon(kind: HitKind) -> IconName {
    match kind {
        HitKind::ModuleRow => IconName::User,
        HitKind::Added => IconName::Users,
        HitKind::Table => IconName::Table2,
        HitKind::Row => IconName::Rows3,
        HitKind::Key => IconName::KeyRound,
        HitKind::Value => IconName::Hash,
        HitKind::Text => IconName::Languages,
    }
}

pub(crate) fn section_head(icon: IconName, title: &str, subtitle: &str, p: Palette) -> Div {
    h_flex()
        .flex_1()
        .min_w_0()
        .gap(px(12.))
        .child(badge(icon, 36., p))
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap(px(2.))
                .child(
                    div()
                        .text_size(px(15.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(title.to_owned()),
                )
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(p.text_muted)
                        .truncate()
                        .child(subtitle.to_owned()),
                ),
        )
}

pub(crate) fn nav_row(
    id: impl Into<SharedString>,
    icon: IconName,
    title: impl Into<SharedString>,
    count: Option<String>,
    active: bool,
    p: Palette,
) -> Stateful<Div> {
    h_flex()
        .id(ElementId::Name(id.into()))
        .h(px(32.))
        .px(px(10.))
        .gap(px(9.))
        .flex_shrink_0()
        .rounded(px(6.))
        .cursor_pointer()
        .text_size(px(13.))
        .map(|d| {
            if active {
                d.bg(p.selected)
                    .text_color(p.text)
                    .font_weight(FontWeight::MEDIUM)
            } else {
                d.text_color(p.text_muted)
                    .hover(move |s| s.bg(p.hover).text_color(p.text))
            }
        })
        .child(
            Icon::new(icon)
                .size(px(15.))
                .text_color(if active { p.accent } else { p.text_placeholder }),
        )
        .child(div().flex_1().min_w_0().truncate().child(title.into()))
        .children(count.map(|n| div().text_size(px(12.)).text_color(p.text_placeholder).child(n)))
}

impl Shell {
    fn db_header(&self, p: Palette, cx: &mut Context<Self>) -> AnyElement {
        let profile = self.library.profile.name.clone();
        let ready = self.db.catalog().is_some();
        let n = self.db.draft.edits.len();
        let unsaved = self.db.draft.unsaved();
        let bound = self.db.draft.addon.clone();
        let info = self.db_bound().cloned();
        let on = bound.as_deref().is_some_and(|id| self.db_enabled(id));
        let broken = self.db_locked();
        let target = match (&bound, &info) {
            (Some(_), Some(i)) => format!("{} {}", i.name, i.version),
            (Some(id), None) => id.clone(),
            (None, _) => "New addon".to_owned(),
        };
        let place = match &bound {
            Some(_) if broken => Some("Could not be read".to_owned()),
            Some(_) if on => Some(format!("on in {profile}")),
            Some(_) => Some(format!("off in {profile}")),
            None => None,
        };
        let (state, color) = match (unsaved, &bound) {
            (0, Some(_)) => ("Saved".to_owned(), p.success),
            (0, None) => ("No changes yet".to_owned(), p.text_placeholder),
            (k, Some(_)) => (format!("{k} not saved to the addon"), p.warning),
            (k, None) => (format!("{k} not saved yet"), p.warning),
        };
        let pill = h_flex()
            .h(px(22.))
            .px(px(8.))
            .gap(px(5.))
            .flex_shrink_0()
            .rounded(px(11.))
            .bg(color.opacity(0.13))
            .text_size(px(12.))
            .font_weight(FontWeight::MEDIUM)
            .text_color(color)
            .child(div().size(px(6.)).rounded_full().bg(color))
            .child(state);
        let turn_on = bound.clone().filter(|_| !on && !broken).map(|id| {
            Button::new("db-turn-on")
                .ghost()
                .xsmall()
                .icon(IconName::Power)
                .label("Turn on")
                .when(self.vanilla(), |b| b.tooltip("Vanilla stays stock"))
                .disabled(self.vanilla())
                .on_click(
                    cx.listener(move |this, _, window, cx| this.set_addon_enabled(&id, true, window, cx)),
                )
        });
        let details = bound.as_ref().filter(|_| !broken).map(|_| {
            Button::new("db-details")
                .ghost()
                .xsmall()
                .icon(IconName::SquarePen)
                .label("Details…")
                .on_click(cx.listener(|this, _, window, cx| this.db_details(window, cx)))
        });
        let fresh = bound.as_ref().map(|_| {
            Button::new("db-new-draft")
                .ghost()
                .xsmall()
                .icon(IconName::Plus)
                .label("New addon")
                .tooltip("Start a new addon from the game's stock values")
                .on_click(cx.listener(|this, _, window, cx| this.db_new_draft(window, cx)))
        });
        let discard = (unsaved > 0 && !broken).then(|| {
            Button::new("db-discard")
                .ghost()
                .xsmall()
                .icon(IconName::Undo2)
                .label("Discard changes")
                .on_click(cx.listener(|this, _, window, cx| this.db_discard(window, cx)))
        });
        let save = match &bound {
            Some(_) => Button::new("db-save")
                .primary()
                .icon(IconName::Save)
                .label("Save")
                .disabled(!ready || broken || unsaved == 0),
            None => Button::new("db-save")
                .primary()
                .icon(IconName::PackagePlus)
                .label("Save as Addon")
                .disabled(!ready || n == 0),
        }
        .on_click(cx.listener(|this, _, window, cx| this.db_save(window, cx)));
        let actions = h_flex()
            .gap(px(8.))
            .child(
                Button::new("db-import")
                    .outline()
                    .icon(IconName::FileDown)
                    .label("Import…")
                    .loading(self.db.importing)
                    .disabled(!ready || self.db.importing)
                    .on_click(cx.listener(|this, _, window, cx| this.db_import(window, cx))),
            )
            .child(self.db_export_menu(cx))
            .child(
                Button::new("db-changes")
                    .outline()
                    .icon(IconName::ListChecks)
                    .label(format!("All changes ({n})"))
                    .disabled(!ready)
                    .on_click(cx.listener(|this, _, _, cx| this.db_go(Nav::Changes, cx))),
            )
            .child(save);
        let title = div()
            .text_size(px(18.))
            .font_weight(FontWeight::SEMIBOLD)
            .child("DB Editor");
        let line = h_flex()
            .gap(px(10.))
            .min_w_0()
            .child(
                Icon::new(if bound.is_some() {
                    IconName::Pencil
                } else {
                    IconName::PackagePlus
                })
                .size(px(14.))
                .text_color(p.text_muted),
            )
            .child(
                div()
                    .min_w(px((target.chars().count() as f32 * 7.).min(160.)))
                    .flex_shrink(1.)
                    .text_size(px(13.))
                    .font_weight(FontWeight::MEDIUM)
                    .truncate()
                    .child(target),
            )
            .children(place.map(|t| {
                div()
                    .text_size(px(13.))
                    .flex_shrink_0()
                    .text_color(if broken { p.error } else { p.text_muted })
                    .child(t)
            }))
            .when(!broken, |d| d.child(pill))
            .children(turn_on)
            .children(discard)
            .children(details)
            .children(fresh);
        if self.content_width < NARROW {
            return v_flex()
                .px(px(GUTTER))
                .pt(px(22.))
                .pb(px(16.))
                .gap(px(10.))
                .flex_shrink_0()
                .child(
                    h_flex()
                        .gap(px(16.))
                        .items_center()
                        .child(title.flex_1())
                        .child(actions),
                )
                .child(line)
                .into_any_element();
        }
        h_flex()
            .px(px(GUTTER))
            .pt(px(22.))
            .pb(px(16.))
            .gap(px(16.))
            .items_end()
            .flex_shrink_0()
            .child(v_flex().flex_1().min_w_0().gap(px(8.)).child(title).child(line))
            .child(h_flex().gap(px(8.)).pb(px(2.)).child(actions))
            .into_any_element()
    }

    pub(crate) fn db_page(&mut self, p: Palette, cx: &mut Context<Self>) -> AnyElement {
        let career = Self::career_link();
        let career_label = match self.db_link_state(&career) {
            LinkState::Open => "Open Career Mode Editor".to_owned(),
            LinkState::Off(_) => "Turn on Career Mode Editor".to_owned(),
            LinkState::Missing => "Get Career Mode Editor".to_owned(),
        };
        let strip = notice(
            "db-career-notice",
            IconName::Info,
            p.accent,
            "Changes apply when you press Play, to careers you start afterwards. To edit a career in \
             progress, use Career Mode Editor."
                .into(),
            p,
        )
        .mx(px(GUTTER))
        .mb(px(10.))
        .child(
            Button::new("db-career-link")
                .ghost()
                .small()
                .label(career_label)
                .when(matches!(self.db_link_state(&career), LinkState::Missing), |b| {
                    b.icon(IconName::ExternalLink)
                })
                .disabled(self.vanilla() && matches!(self.db_link_state(&career), LinkState::Off(_)))
                .on_click(cx.listener(move |this, _, window, cx| this.db_follow(&career, window, cx))),
        );
        let vanilla = self.vanilla().then(|| {
            notice(
                "db-vanilla-notice",
                IconName::Lock,
                p.warning,
                "Vanilla stays stock. You can still edit and save addons here; turn them on in another profile."
                    .into(),
                p,
            )
            .mx(px(GUTTER))
            .mb(px(10.))
        });
        let broken = self.db.addon_error.clone().map(|e| {
            notice(
                "db-addon-error",
                IconName::CircleAlert,
                p.error,
                format!("This addon could not be read, so it can't be edited: {e}"),
                p,
            )
            .mx(px(GUTTER))
            .mb(px(10.))
            .child(
                Button::new("db-addon-retry")
                    .outline()
                    .small()
                    .icon(IconName::RefreshCw)
                    .label("Try again")
                    .on_click(cx.listener(|this, _, _, cx| this.db_retry(cx))),
            )
        });
        let body = match &self.db.load {
            Load::Ready(_) => self.db_body(p, cx),
            Load::Loading => self.db_loading("Reading the game database…", p),
            Load::Failed(e) => self.empty(
                IconName::CircleAlert,
                "The database could not be read",
                e.clone(),
                Button::new("db-retry")
                    .outline()
                    .icon(IconName::RefreshCw)
                    .label("Try again")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.db.load = Load::Idle;
                        this.db.game = None;
                        cx.notify();
                    })),
                p,
            ),
            Load::Idle if !self.detected => self.db_loading("Looking for F1 25…", p),
            Load::Idle => self.empty(
                IconName::Database,
                "F1 25 not found",
                "The DB Editor reads the game's database. Show the launcher where F1 25 is installed.",
                Button::new("db-locate")
                    .outline()
                    .icon(IconName::FolderOpen)
                    .label("Locate F1 25…")
                    .on_click(cx.listener(|this, _, window, cx| this.locate_game(window, cx))),
                p,
            ),
        };
        let focus = self.db.focus.clone();
        v_flex()
            .id("db-page")
            .size_full()
            .when_some(focus, |d, f| d.track_focus(&f))
            .capture_key_down(cx.listener(|this, ev: &KeyDownEvent, window, cx| {
                let k = &ev.keystroke;
                let ctrl = k.modifiers.control || k.modifiers.platform;
                let typing = this
                    .db
                    .focus
                    .as_ref()
                    .is_some_and(|f| !f.is_focused(window) && f.contains_focused(window, cx));
                if (ctrl && k.key == "f") || (k.key == "/" && !typing) {
                    this.db_focus_search(window, cx);
                    cx.stop_propagation();
                } else if k.key == "escape" && this.db.hits_open {
                    this.db.hits_open = false;
                    cx.notify();
                }
            }))
            .child(self.db_header(p, cx))
            .child(strip)
            .children(vanilla)
            .children(broken)
            .children(self.db.save_error.clone().map(|e| {
                notice("db-save-error", IconName::CircleAlert, p.error, e, p)
                    .mx(px(GUTTER))
                    .mb(px(10.))
            }))
            .children(self.db.newer.then(|| {
                notice(
                    "db-newer-draft",
                    IconName::TriangleAlert,
                    p.warning,
                    "Another launcher window saved a newer draft. Changes here are not saved until you pick one."
                        .into(),
                    p,
                )
                .mx(px(GUTTER))
                .mb(px(10.))
                .child(
                    Button::new("db-take-newer")
                        .outline()
                        .small()
                        .label("Load the newer draft")
                        .on_click(cx.listener(|this, _, _, cx| this.db_take_newer(cx))),
                )
                .child(
                    Button::new("db-keep-mine")
                        .ghost()
                        .small()
                        .label("Keep this window's changes")
                        .on_click(cx.listener(|this, _, _, cx| this.db_keep_mine(cx))),
                )
            }))
            .child(body)
            .into_any_element()
    }

    fn db_body(&mut self, p: Palette, cx: &mut Context<Self>) -> AnyElement {
        let main = match self.db.nav.clone() {
            Nav::Module(id) => self.db_module(&id, p, cx),
            Nav::Raw(node) => self.db_raw(node, p, cx),
            Nav::Unnamed => self.db_unnamed_view(p, cx),
            Nav::Changes => self.db_changes(p, cx),
        };
        v_flex()
            .flex_1()
            .min_h_0()
            .child(self.db_search_bar(p, cx))
            .child(
                h_flex()
                    .id("db-body")
                    .flex_1()
                    .min_h_0()
                    .items_start()
                    .px(px(GUTTER))
                    .pb(px(20.))
                    .gap(px(20.))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            if this.db.hits_open {
                                this.db.hits_open = false;
                                cx.notify();
                            }
                        }),
                    )
                    .child(self.db_nav(p, cx))
                    .child(div().flex_1().min_w_0().h_full().flex().flex_col().child(main)),
            )
            .into_any_element()
    }

    fn db_search_bar(&self, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(search) = &self.db.search else {
            return div().into_any_element();
        };
        let query = search.query.trim().to_owned();
        let open = self.db.hits_open && !query.is_empty();
        let results = open.then(|| {
            let mut rows: Vec<AnyElement> = Vec::new();
            let mut last = None;
            for (i, h) in self.db.hits.iter().enumerate() {
                if last != Some(h.kind) {
                    last = Some(h.kind);
                    rows.push(
                        div()
                            .px(px(12.))
                            .pt(px(if i == 0 { 8. } else { 12. }))
                            .pb(px(4.))
                            .child(label(hit_group(h.kind).to_uppercase(), p))
                            .into_any_element(),
                    );
                }
                let on = i == self.db.hit;
                rows.push(
                    h_flex()
                        .id(SharedString::from(format!("db-hit-{i}")))
                        .mx(px(4.))
                        .h(px(40.))
                        .px(px(8.))
                        .gap(px(10.))
                        .rounded(px(6.))
                        .cursor_pointer()
                        .when(on, |d| d.bg(p.selected))
                        .hover(move |s| s.bg(p.hover))
                        .child(
                            Icon::new(hit_icon(h.kind))
                                .size(px(14.))
                                .text_color(p.text_placeholder),
                        )
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .child(div().text_size(px(13.)).truncate().child(h.title.clone()))
                                .child(
                                    div()
                                        .text_size(px(11.))
                                        .text_color(p.text_muted)
                                        .truncate()
                                        .child(h.detail.clone()),
                                ),
                        )
                        .on_click(cx.listener(move |this, _, window, cx| this.db_open_hit(i, window, cx)))
                        .into_any_element(),
                );
            }
            let empty = self.db.hits.is_empty().then(|| {
                div()
                    .px(px(14.))
                    .py(px(14.))
                    .text_size(px(13.))
                    .text_color(p.text_muted)
                    .child(format!("Nothing matches “{query}”"))
            });
            deferred(
                v_flex()
                    .id("db-hits")
                    .occlude()
                    .absolute()
                    .top(px(40.))
                    .left_0()
                    .w(px(620.))
                    .max_h(px(440.))
                    .rounded(px(8.))
                    .border_1()
                    .border_color(p.border)
                    .bg(p.elevated)
                    .shadow_lg()
                    .overflow_hidden()
                    .child(
                        v_flex()
                            .flex_1()
                            .scroll_y("db-hits-list")
                            .pb(px(6.))
                            .children(rows)
                            .children(empty),
                    ),
            )
            .with_priority(1)
        });
        h_flex()
            .px(px(GUTTER))
            .pb(px(14.))
            .flex_shrink_0()
            .child(
                div()
                    .relative()
                    .w(px(620.))
                    .capture_key_down(cx.listener(|this, ev: &KeyDownEvent, _, cx| {
                        let n = this.db.hits.len();
                        match ev.keystroke.key.as_str() {
                            "down" if n > 0 => this.db.hit = (this.db.hit + 1) % n,
                            "up" if n > 0 => this.db.hit = (this.db.hit + n - 1) % n,
                            _ => return,
                        }
                        this.db.hits_open = true;
                        cx.stop_propagation();
                        cx.notify();
                    }))
                    .child(
                        Input::new(&search.input).cleanable(true).prefix(
                            Icon::new(IconName::Search)
                                .size(px(15.))
                                .text_color(p.text_placeholder),
                        ),
                    )
                    .children(results),
            )
            .child(div().flex_1())
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(p.text_placeholder)
                    .child("Ctrl+F or / to search"),
            )
            .into_any_element()
    }

    fn db_nav(&self, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        let mut groups: Vec<(&str, Vec<&Module>)> = Vec::new();
        for m in self.db.modules {
            match groups.iter_mut().find(|(g, _)| *g == m.group) {
                Some((_, list)) => list.push(m),
                None => groups.push((&m.group, vec![m])),
            }
        }
        let nav = &self.db.nav;
        let catalog = self.db.catalog().cloned();
        let item = |id: String,
                    icon: IconName,
                    title: String,
                    count: Option<String>,
                    to: Nav,
                    cx: &mut Context<Self>| {
            let active = match (&to, nav) {
                (Nav::Raw(_), Nav::Raw(_)) => true,
                (a, b) => a == b,
            };
            nav_row(id, icon, title, count, active, p)
                .on_click(cx.listener(move |this, _, _, cx| this.db_go(to.clone(), cx)))
                .into_any_element()
        };
        let mut sections: Vec<AnyElement> = groups
            .into_iter()
            .map(|(g, list)| {
                let rows: Vec<AnyElement> = list
                    .into_iter()
                    .map(|m| {
                        item(
                            format!("db-nav-{}", m.id),
                            icon_named(&m.icon),
                            m.title.clone(),
                            None,
                            Nav::Module(m.id.clone()),
                            cx,
                        )
                    })
                    .collect();
                v_flex()
                    .gap(px(2.))
                    .child(label(g.to_uppercase(), p).px(px(10.)).pb(px(4.)))
                    .children(rows)
                    .into_any_element()
            })
            .collect();
        let root = catalog.as_ref().map_or(0, |c| c.root());
        let unnamed = catalog.as_ref().map(|c| c.records().saturating_sub(c.named()));
        let advanced: Vec<AnyElement> = vec![
            item(
                "db-nav-raw".into(),
                IconName::FolderTree,
                "All tables".into(),
                None,
                Nav::Raw(root),
                cx,
            ),
            item(
                "db-nav-unnamed".into(),
                IconName::Binary,
                "Unnamed keys".into(),
                unnamed.map(|n| n.to_string()),
                Nav::Unnamed,
                cx,
            ),
            item(
                "db-nav-changes".into(),
                IconName::ListChecks,
                "Changes".into(),
                Some(self.db.draft.edits.len().to_string()),
                Nav::Changes,
                cx,
            ),
        ];
        sections.push(
            v_flex()
                .gap(px(2.))
                .child(label("ADVANCED", p).px(px(10.)).pb(px(4.)))
                .children(advanced)
                .into_any_element(),
        );
        v_flex()
            .w(px(220.))
            .flex_shrink_0()
            .h_full()
            .scroll_y("db-nav")
            .gap(px(18.))
            .pr(px(SCROLLBAR_GAP))
            .pb(px(12.))
            .children(sections)
    }

    pub(crate) fn db_link_button(&self, i: usize, link: &LinkSpec, cx: &mut Context<Self>) -> Button {
        let state = self.db_link_state(link);
        let label = match &state {
            LinkState::Open | LinkState::Missing => link.label.clone(),
            LinkState::Off(name) => format!("Turn on {name}"),
        };
        let link = link.clone();
        Button::new(SharedString::from(format!("db-link-{i}")))
            .outline()
            .small()
            .label(label)
            .tooltip(link.text.clone())
            .map(|b| match state {
                LinkState::Missing => b.icon(IconName::ExternalLink),
                LinkState::Off(_) => b.icon(IconName::Power).disabled(self.vanilla()),
                LinkState::Open => b.icon(IconName::ArrowUpRight),
            })
            .on_click(cx.listener(move |this, _, window, cx| this.db_follow(&link, window, cx)))
    }

    fn db_module(&mut self, id: &str, p: Palette, cx: &mut Context<Self>) -> AnyElement {
        let Some(module) = self.db.module(id) else {
            return div().into_any_element();
        };
        let detail_w = if self.db.row.is_some() { 396. } else { 0. };
        let moves_w = if swap_field(module).is_some() { 68. } else { 0. };
        let room = self.content_width - 240. - detail_w - moves_w - NAME_W - 26.;
        self.db.max_cols = ((room / 124.).floor() as usize).max(1);
        let slot = self.db.view.as_ref().filter(|v| v.module == id);
        let links: Vec<Button> = module
            .links
            .iter()
            .enumerate()
            .map(|(i, l)| self.db_link_button(i, l, cx))
            .collect();
        let head = h_flex()
            .gap(px(12.))
            .flex_shrink_0()
            .child(section_head(
                icon_named(&module.icon),
                &module.title,
                &module.summary,
                p,
            ))
            .children(links);
        let notes = module.notes.iter().enumerate().map(|(i, n)| {
            notice("db-note", IconName::Info, p.text_placeholder, n.clone(), p)
                .id(SharedString::from(format!("db-note-{i}")))
                .into_any_element()
        });
        let warnings: Vec<AnyElement> = slot
            .map(|s| s.view.warnings.clone())
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(i, w)| {
                notice("db-warning", IconName::TriangleAlert, p.warning, w, p)
                    .id(SharedString::from(format!("db-warning-{i}")))
                    .into_any_element()
            })
            .collect();
        let toolbar = slot.map(|slot| {
            let scope = module.scope.as_ref().map(|sc| {
                let current = slot.scope.clone().unwrap_or_default();
                let scopes = slot.view.scopes.clone();
                let shown = scopes
                    .iter()
                    .find(|(l, _)| *l == current)
                    .map(|(l, n)| if n.is_empty() { l.clone() } else { n.clone() })
                    .unwrap_or_else(|| current.clone());
                let shell = cx.entity().downgrade();
                h_flex()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(p.text_muted)
                            .child(sc.title.clone()),
                    )
                    .child(
                        Button::new("db-scope")
                            .outline()
                            .small()
                            .min_w(px(200.))
                            .label(shown)
                            .dropdown_caret(true)
                            .dropdown_menu_with_anchor(Anchor::TopLeft, move |mut menu, _, _| {
                                menu = menu.max_h(px(360.)).scrollable(true).min_w(px(260.));
                                for (l, n) in &scopes {
                                    let (shell, value) = (shell.clone(), l.clone());
                                    let text = if n.is_empty() { l.clone() } else { n.clone() };
                                    menu =
                                        menu.item(PopupMenuItem::new(text).checked(*l == current).on_click(
                                            move |_, _, cx| {
                                                shell
                                                    .update(cx, |s, cx| s.db_set_scope(value.clone(), cx))
                                                    .ok();
                                            },
                                        ));
                                }
                                menu
                            }),
                    )
            });
            let total = slot.view.rows.len();
            let count = if slot.shown.len() == total {
                plural(total, "row", "rows")
            } else {
                format!("{} of {total} rows", slot.shown.len())
            };
            h_flex()
                .gap(px(16.))
                .flex_shrink_0()
                .children(scope)
                .children(self.db.filter.as_ref().map(|f| {
                    div().w(px(280.)).child(
                        Input::new(&f.input).small().cleanable(true).prefix(
                            Icon::new(IconName::ListFilter)
                                .size(px(14.))
                                .text_color(p.text_placeholder),
                        ),
                    )
                }))
                .child(div().flex_1())
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(p.text_placeholder)
                        .child(count),
                )
        });
        let has_slot = slot.is_some();
        let content = match (has_slot, self.db.view_error.clone()) {
            (_, Some(e)) => self.empty(IconName::CircleAlert, "This module could not load", e, div(), p),
            (false, None) => self.db_loading("Loading rows…", p),
            (true, None) => self.db_table(module, p, cx),
        };
        let detail = self
            .db
            .row
            .clone()
            .filter(|_| has_slot)
            .map(|row| self.db_row_detail(module, &row, p, cx));
        v_flex()
            .size_full()
            .gap(px(14.))
            .child(head)
            .children(notes)
            .children(warnings)
            .children(toolbar)
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .gap(px(16.))
                    .items_start()
                    .child(div().flex_1().min_w_0().h_full().flex().flex_col().child(content))
                    .children(detail),
            )
            .into_any_element()
    }

    fn db_table(&mut self, module: &'static Module, p: Palette, cx: &mut Context<Self>) -> AnyElement {
        let Some(slot) = self.db.view.as_ref() else {
            return div().into_any_element();
        };
        if slot.shown.is_empty() {
            let filtered = !slot.view.rows.is_empty();
            return self.empty(
                IconName::Search,
                if filtered { "No matching rows" } else { "No rows" },
                if filtered {
                    "Nothing in this list matches the filter."
                } else {
                    "This table has no rows in the stock database."
                },
                div(),
                p,
            );
        }
        let columns: Vec<(usize, String)> = slot
            .view
            .columns
            .iter()
            .take(self.db.max_cols)
            .map(|c| (c.field, c.label.clone()))
            .collect();
        let moves = swap_field(module).is_some() && !self.db_locked();
        let header = h_flex()
            .h(px(32.))
            .px(px(12.))
            .gap(px(12.))
            .flex_shrink_0()
            .border_b_1()
            .border_color(p.border_variant)
            .text_size(px(11.))
            .font_weight(FontWeight::MEDIUM)
            .text_color(p.text_placeholder)
            .child(div().w(px(NAME_W)).flex_shrink_0().child("NAME"))
            .children(columns.iter().map(|(f, l)| {
                let editable = module.fields.get(*f).is_some_and(|s| !s.readonly);
                h_flex()
                    .flex_1()
                    .flex_basis(px(0.))
                    .min_w_0()
                    .gap(px(4.))
                    .child(div().truncate().child(l.to_uppercase()))
                    .when(editable && !self.db_locked(), |d| {
                        d.child(
                            Icon::new(IconName::Pencil)
                                .size(px(10.))
                                .text_color(p.text_placeholder),
                        )
                    })
            }))
            .when(moves, |d| d.child(div().w(px(MOVE_W)).flex_shrink_0()));
        let count = slot.shown.len();
        let list = uniform_list(
            "db-rows",
            count,
            cx.processor(move |this, range: Range<usize>, _, cx| this.db_rows(module, range, cx)),
        )
        .flex_1()
        .track_scroll(&self.db.rows_scroll);
        v_flex()
            .size_full()
            .rounded(px(6.))
            .border_1()
            .border_color(p.border_variant)
            .overflow_hidden()
            .child(header)
            .child(self.db.rows_motion.wrap(list, &self.db.rows_scroll, cx))
            .into_any_element()
    }

    fn db_rows(
        &mut self,
        module: &'static Module,
        range: Range<usize>,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let p = palette(cx);
        let Some(slot) = self.db.view.as_ref() else {
            return Vec::new();
        };
        let view = slot.view.clone();
        let rows: Vec<usize> = range.filter_map(|i| slot.shown.get(i).copied()).collect();
        let columns: Vec<usize> = view
            .columns
            .iter()
            .take(self.db.max_cols)
            .map(|c| c.field)
            .collect();
        let moves = swap_field(module).is_some() && !self.db_locked();
        let total = view.rows.len();
        rows.into_iter()
            .enumerate()
            .map(|(n, ri)| {
                let r = &view.rows[ri];
                let selected = self.db.row.as_deref() == Some(r.label.as_str());
                let label = r.label.clone();
                let edited = r.cells.iter().any(|c| c.edited);
                let shared = r.cells.iter().any(|c| c.key.is_some_and(|k| self.db_shared(k)));
                h_flex()
                    .id(SharedString::from(format!("db-row-{}", r.label)))
                    .w_full()
                    .h(px(ROW_H))
                    .px(px(12.))
                    .gap(px(12.))
                    .when(n > 0 || ri > 0, |d| d.border_t_1().border_color(p.border_variant))
                    .cursor_pointer()
                    .map(|d| {
                        if selected {
                            d.bg(p.selected)
                        } else {
                            d.hover(move |s| s.bg(p.hover))
                        }
                    })
                    .child(
                        h_flex()
                            .w(px(NAME_W))
                            .flex_shrink_0()
                            .gap(px(8.))
                            .child(
                                v_flex().flex_1().min_w_0().child(
                                    div()
                                        .text_size(px(13.))
                                        .font_weight(FontWeight::MEDIUM)
                                        .truncate()
                                        .child(if r.name.is_empty() {
                                            r.label.clone()
                                        } else {
                                            r.name.clone()
                                        }),
                                ),
                            )
                            .when(shared, |d| {
                                d.child(
                                    div()
                                        .id(SharedString::from(format!("db-row-shared-{ri}")))
                                        .size(px(7.))
                                        .flex_shrink_0()
                                        .rounded_full()
                                        .border_1()
                                        .border_color(p.warning)
                                        .tooltip(|window, cx| {
                                            gpui_kit::component::tooltip::Tooltip::new(
                                                "Another DB Editor addon on in this profile changes this row",
                                            )
                                            .build(window, cx)
                                        }),
                                )
                            })
                            .when(edited, |d| {
                                d.child(div().size(px(6.)).flex_shrink_0().rounded_full().bg(p.accent))
                            }),
                    )
                    .children(columns.iter().map(|&f| {
                        div()
                            .flex_1()
                            .flex_basis(px(0.))
                            .min_w_0()
                            .child(self.db_cell(module, ri, f, p, cx))
                    }))
                    .when(moves, |d| {
                        let (up, down) = (label.clone(), label.clone());
                        d.child(
                            h_flex()
                                .w(px(MOVE_W))
                                .flex_shrink_0()
                                .justify_end()
                                .child(
                                    Button::new(SharedString::from(format!("db-up-{ri}")))
                                        .ghost()
                                        .xsmall()
                                        .icon(IconName::ArrowUp)
                                        .tooltip("Move up")
                                        .disabled(ri == 0)
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            cx.stop_propagation();
                                            this.db_move(&up, false, cx);
                                        })),
                                )
                                .child(
                                    Button::new(SharedString::from(format!("db-down-{ri}")))
                                        .ghost()
                                        .xsmall()
                                        .icon(IconName::ArrowDown)
                                        .tooltip("Move down")
                                        .disabled(ri + 1 == total)
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            cx.stop_propagation();
                                            this.db_move(&down, true, cx);
                                        })),
                                ),
                        )
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.db.row = (this.db.row.as_ref() != Some(&label)).then(|| label.clone());
                        this.db.cell = None;
                        cx.notify();
                    }))
                    .into_any_element()
            })
            .collect()
    }

    fn db_cell(
        &self,
        module: &'static Module,
        ri: usize,
        f: usize,
        p: Palette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(slot) = self.db.view.as_ref() else {
            return div().into_any_element();
        };
        let (Some(cell), Some(spec)) = (slot.view.rows[ri].cells.get(f), module.fields.get(f)) else {
            return div().into_any_element();
        };
        let Some(key) = cell.key else {
            return div()
                .text_size(px(13.))
                .text_color(p.text_placeholder)
                .whitespace_nowrap()
                .truncate()
                .child(if spec.optional { "Default" } else { "—" })
                .into_any_element();
        };
        let kind = kind_of(spec.kind, cell.tag);
        let locked = spec.readonly || self.db_locked();
        let error = self.db.errors.contains_key(&key);
        let color = if error {
            p.error
        } else if cell.edited {
            p.accent
        } else {
            p.text
        };
        if let Some(edit) = self.db.cell.as_ref().filter(|c| c.key == key) {
            let input = edit.input.clone();
            return div()
                .id(SharedString::from(format!("db-cell-edit-{key:x}")))
                .w_full()
                .on_click(|_, _, cx| cx.stop_propagation())
                .capture_key_down(cx.listener(|this, ev: &KeyDownEvent, _, cx| {
                    if ev.keystroke.key == "escape" {
                        this.db.cell = None;
                        cx.notify();
                    }
                }))
                .child(Input::new(&input).xsmall())
                .into_any_element();
        }
        let choices: Vec<(String, String)> = match kind {
            Kind::Choice => spec
                .options
                .iter()
                .map(|o| (o.value.clone(), o.label.clone()))
                .collect(),
            Kind::Ref | Kind::RefList => slot.choices.get(&f).cloned().unwrap_or_default(),
            _ => Vec::new(),
        };
        let shown = match kind {
            Kind::Choice | Kind::Ref => choices
                .iter()
                .find(|(v, _)| *v == cell.value)
                .map(|(_, n)| n.clone())
                .filter(|n| !n.is_empty())
                .or_else(|| cell.text.as_deref().map(clean))
                .unwrap_or_else(|| cell.value.clone()),
            Kind::RefList => plural(list_items(&cell.value).len(), "item", "items"),
            Kind::Curve => curve_rating(&cell.value).unwrap_or_else(|| "curve".into()),
            Kind::Color => hex_rgb(&cell.value).unwrap_or_else(|| cell.value.clone()),
            _ => cell
                .text
                .as_deref()
                .map(clean)
                .unwrap_or_else(|| cell.value.clone()),
        };
        let revert = (cell.edited && !self.db_locked()).then(|| {
            div()
                .id(SharedString::from(format!("db-cell-revert-{key:x}")))
                .flex_shrink_0()
                .p(px(3.))
                .rounded(px(4.))
                .cursor_pointer()
                .text_color(p.text_placeholder)
                .hover(move |s| s.bg(p.hover).text_color(p.text))
                .child(Icon::new(IconName::Undo2).size(px(12.)))
                .tooltip(move |window, cx| {
                    gpui_kit::component::tooltip::Tooltip::new("Revert to stock").build(window, cx)
                })
                .on_click(cx.listener(move |this, _, _, cx| {
                    cx.stop_propagation();
                    this.db_revert(key, cx);
                }))
        });
        let text = div()
            .min_w_0()
            .truncate()
            .whitespace_nowrap()
            .text_size(px(13.))
            .text_color(color)
            .child(shown.clone());
        let body: AnyElement = match kind {
            Kind::Bool => {
                let on = is_on(&cell.value);
                let (tag, stock) = (cell.tag, cell.stock.clone());
                div()
                    .id(SharedString::from(format!("db-cell-bool-{key:x}")))
                    .on_click(|_, _, cx| cx.stop_propagation())
                    .child(
                        Switch::new(SharedString::from(format!("db-cell-switch-{key:x}")))
                            .checked(on)
                            .small()
                            .disabled(locked)
                            .on_change(cx.listener(move |this, on: &bool, _, cx| {
                                this.db_set(key, &bool_text(tag, &stock, *on), cx);
                            })),
                    )
                    .into_any_element()
            }
            Kind::Color => h_flex()
                .gap(px(8.))
                .min_w_0()
                .child(swatch(&cell.value, 14., p))
                .child(text)
                .into_any_element(),
            Kind::Choice | Kind::Ref if !locked && !choices.is_empty() => {
                let shell = cx.entity().downgrade();
                let current = cell.value.clone();
                let pick = Button::new(SharedString::from(format!("db-cell-pick-{key:x}")))
                    .ghost()
                    .xsmall()
                    .icon(IconName::ChevronDown)
                    .dropdown_menu_with_anchor(Anchor::TopLeft, move |mut menu, _, _| {
                        menu = menu.max_h(px(320.)).scrollable(true).min_w(px(220.));
                        for (v, n) in &choices {
                            let (shell, v2) = (shell.clone(), v.clone());
                            let t = if n.is_empty() { v.clone() } else { n.clone() };
                            menu = menu.item(PopupMenuItem::new(t).checked(*v == current).on_click(
                                move |_, _, cx| {
                                    shell.update(cx, |s, cx| s.db_set(key, &v2, cx)).ok();
                                },
                            ));
                        }
                        menu
                    });
                h_flex()
                    .gap(px(2.))
                    .min_w_0()
                    .child(text)
                    .child(
                        div()
                            .flex_shrink_0()
                            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                            .child(pick),
                    )
                    .into_any_element()
            }
            Kind::Int | Kind::Float | Kind::Text | Kind::Time | Kind::List | Kind::Vector if !locked => {
                let value = cell.value.clone();
                h_flex()
                    .id(SharedString::from(format!("db-cell-{key:x}")))
                    .min_w_0()
                    .h(px(26.))
                    .px(px(6.))
                    .mx(px(-6.))
                    .rounded(px(4.))
                    .cursor_text()
                    .hover(move |s| s.bg(p.hover))
                    .child(text)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        cx.stop_propagation();
                        this.db_start_cell(key, value.clone(), window, cx);
                    }))
                    .into_any_element()
            }
            _ => text.into_any_element(),
        };
        h_flex()
            .gap(px(2.))
            .min_w_0()
            .child(div().min_w_0().child(body))
            .children(revert)
            .into_any_element()
    }

    fn db_row_detail(
        &self,
        module: &'static Module,
        row: &str,
        p: Palette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(r) = self
            .db
            .view
            .as_ref()
            .and_then(|s| s.view.rows.iter().find(|r| r.label == row))
        else {
            return div().into_any_element();
        };
        let title = if r.name.is_empty() {
            r.label.clone()
        } else {
            r.name.clone()
        };
        let edited = r.cells.iter().filter(|c| c.edited).count();
        let subtitle = if edited > 0 {
            format!("{} · {} edited", r.label, edited)
        } else {
            r.label.clone()
        };
        let fields = self.db_row_fields();
        let raw = r.node;
        detail_shell(
            "db-row-detail",
            title,
            subtitle,
            p,
            cx.listener(|this, _, _, cx| {
                this.db.row = None;
                cx.notify();
            }),
        )
        .child(
            v_flex()
                .flex_1()
                .min_h_0()
                .scroll_y(SharedString::from(format!("db-row-body-{}-{row}", module.id)))
                .px(px(16.))
                .py(px(14.))
                .gap(px(22.))
                .children(self.db_fields_panel(&fields, p, cx))
                .child(
                    h_flex()
                        .pt(px(4.))
                        .gap(px(6.))
                        .when(swap_field(module).is_some() && !self.db_locked(), |d| {
                            let (up, down) = (row.to_owned(), row.to_owned());
                            d.child(
                                Button::new("db-row-up")
                                    .outline()
                                    .small()
                                    .icon(IconName::ArrowUp)
                                    .label("Move up")
                                    .on_click(
                                        cx.listener(move |this, _, _, cx| this.db_move(&up, false, cx)),
                                    ),
                            )
                            .child(
                                Button::new("db-row-down")
                                    .outline()
                                    .small()
                                    .icon(IconName::ArrowDown)
                                    .label("Move down")
                                    .on_click(
                                        cx.listener(move |this, _, _, cx| this.db_move(&down, true, cx)),
                                    ),
                            )
                            .child(div().flex_1())
                        })
                        .child(
                            Button::new("db-row-raw")
                                .ghost()
                                .small()
                                .icon(IconName::FolderTree)
                                .label("Show in All tables")
                                .on_click(cx.listener(move |this, _, _, cx| this.db_go(Nav::Raw(raw), cx))),
                        ),
                ),
        )
        .into_any_element()
    }
}
