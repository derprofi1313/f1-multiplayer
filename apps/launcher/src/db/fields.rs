use std::collections::HashMap;

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState};
use gpui_kit::component::input::{Input, InputEvent, InputState, NumberInput, NumberInputEvent};
use gpui_kit::component::menu::{DropdownMenu, PopupMenuItem};
use gpui_kit::component::select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Disableable, Icon, Sizable, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use swofty_core::dbedit::{Backing, Kind, NodeId};

use super::{Nav, bool_text, clean, hex, kind_of, type_name};
use crate::pages::{fact, label};
use crate::shell::Shell;
use crate::theme::{Palette, ScrollArea};

#[derive(Clone)]
pub(crate) struct Choice {
    value: String,
    title: SharedString,
}

impl SelectItem for Choice {
    type Value = String;

    fn title(&self) -> SharedString {
        self.title.clone()
    }

    fn value(&self) -> &String {
        &self.value
    }
}

type Choices = SearchableVec<Choice>;

struct InputSlot {
    key: u64,
    input: Entity<InputState>,
    synced: String,
    seen: bool,
    _subs: Vec<Subscription>,
}

struct SelectSlot {
    key: u64,
    state: Entity<SelectState<Choices>>,
    synced: String,
    seen: bool,
    _sub: Subscription,
}

struct ColorSlot {
    key: u64,
    state: Entity<ColorPickerState>,
    synced: String,
    seen: bool,
    _sub: Subscription,
}

struct CopySlot {
    state: Entity<SelectState<Choices>>,
    seen: bool,
    _sub: Subscription,
}

#[derive(Default)]
pub(crate) struct Slots {
    inputs: HashMap<String, InputSlot>,
    selects: HashMap<String, SelectSlot>,
    colors: HashMap<String, ColorSlot>,
    copies: HashMap<String, CopySlot>,
}

impl Slots {
    pub(crate) fn forget(&mut self, key: u64) {
        self.inputs.retain(|_, s| s.key != key);
        self.selects.retain(|_, s| s.key != key);
        self.colors.retain(|_, s| s.key != key);
    }
}

/// One editable value of the detail panel: a module field of the selected row, or a raw key.
#[derive(Clone)]
pub(crate) struct FieldModel {
    pub id: String,
    pub key: Option<u64>,
    pub tag: u8,
    pub kind: Kind,
    pub label: String,
    pub section: Option<String>,
    pub help: Option<String>,
    pub value: String,
    pub stock: Option<String>,
    pub edited: bool,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub unit: Option<String>,
    pub options: Vec<(String, String)>,
    pub text: Option<String>,
    pub readonly: bool,
    pub optional: bool,
    pub shared: bool,
    pub others: Option<String>,
    /// For a field read from another table's row picked by this row's label: the row spec and where the row comes from.
    pub row: Option<(usize, Backing)>,
    /// The page field's index, when one value can be set on every listed row.
    pub all: Option<usize>,
}

struct Curve {
    head: Vec<(String, String)>,
    points: Vec<(String, String)>,
}

fn parse_curve(v: &str) -> Option<Curve> {
    let rest = v.strip_prefix("@v2@")?;
    let mut c = Curve {
        head: Vec::new(),
        points: Vec::new(),
    };
    for part in rest.split(',').filter(|p| !p.is_empty()) {
        let (a, b) = part.split_once(':')?;
        if a.starts_with("min") || a.starts_with("max") {
            c.head.push((a.to_owned(), b.to_owned()));
        } else {
            c.points.push((a.to_owned(), b.to_owned()));
        }
    }
    Some(c)
}

impl Curve {
    fn head(&self, name: &str, or: f64) -> f64 {
        self.head
            .iter()
            .find(|(a, _)| a == name)
            .and_then(|(_, b)| b.parse().ok())
            .unwrap_or(or)
    }

    fn span(&self) -> (f64, f64) {
        (self.head("miny", 0.), self.head("maxy", 1.))
    }
}

fn num(v: f64) -> String {
    let t = format!("{v:.4}");
    let t = t.trim_end_matches('0').trim_end_matches('.');
    if t == "-0" { "0".into() } else { t.to_owned() }
}

fn shown_y(c: &Curve, y: &str) -> String {
    let (lo, hi) = c.span();
    y.parse::<f64>()
        .map_or_else(|_| y.to_owned(), |f| num(lo + f * (hi - lo)))
}

fn stored_y(c: &Curve, shown: &str) -> Option<String> {
    let (lo, hi) = c.span();
    let v: f64 = shown.trim().parse().ok()?;
    (hi != lo).then(|| num((v - lo) / (hi - lo)))
}

/// A rating curve as a table cell: its first point on the curve's value scale.
pub(crate) fn curve_rating(v: &str) -> Option<String> {
    let c = parse_curve(v)?;
    let (_, y) = c.points.first()?;
    Some(shown_y(&c, y))
}

pub(crate) fn sep_of(v: &str) -> &'static str {
    if v.contains(", ") { ", " } else { "," }
}

fn curve_text(c: &Curve) -> String {
    let parts: Vec<String> = c
        .head
        .iter()
        .chain(&c.points)
        .map(|(a, b)| format!("{a}:{b}"))
        .collect();
    format!("@v2@{}", parts.join(","))
}

fn argb(v: &str) -> Option<u32> {
    v.trim().parse::<i64>().ok().map(|n| n as u32)
}

pub(crate) fn swatch_color(v: &str) -> Option<Hsla> {
    let c = argb(v)?;
    Some(rgba(((c & 0x00ff_ffff) << 8) | (c >> 24)).into())
}

fn color_text(h: Hsla) -> String {
    let c = Rgba::from(h);
    let byte = |f: f32| (f.clamp(0., 1.) * 255.).round() as u32;
    let v = (byte(c.a) << 24) | (byte(c.r) << 16) | (byte(c.g) << 8) | byte(c.b);
    (v as i32).to_string()
}

pub(crate) fn hex_rgb(v: &str) -> Option<String> {
    argb(v).map(|c| format!("#{:06X}", c & 0x00ff_ffff))
}

pub(crate) fn is_on(v: &str) -> bool {
    matches!(v.trim().to_ascii_lowercase().as_str(), "true" | "1")
}

fn range(text: &str) -> (Option<f64>, Option<f64>) {
    match text.split_once("..") {
        Some((a, b)) => (a.trim().parse().ok(), b.trim().parse().ok()),
        None => (None, None),
    }
}

pub(crate) fn list_items(v: &str) -> Vec<String> {
    v.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

impl Shell {
    pub(crate) fn db_row_fields(&self) -> Vec<FieldModel> {
        let (Some(slot), Some(row)) = (self.db.view.as_ref(), self.db.row.as_ref()) else {
            return Vec::new();
        };
        if self.db.nav != Nav::Module(slot.module.clone()) {
            return Vec::new();
        }
        let Some(module) = self.db.module(&slot.module) else {
            return Vec::new();
        };
        let Some(r) = slot.view.rows.iter().find(|r| r.label == *row) else {
            return Vec::new();
        };
        let rows = module.field_rows();
        module
            .fields
            .iter()
            .zip(&r.cells)
            .enumerate()
            .map(|(i, (f, c))| {
                let row = rows
                    .get(i)
                    .copied()
                    .flatten()
                    .and_then(|spec| Some((spec, r.backing.get(spec)?.clone())));
                let kind = kind_of(f.kind, c.tag);
                let options = match kind {
                    Kind::Choice => f
                        .options
                        .iter()
                        .map(|o| (o.value.clone(), o.label.clone()))
                        .collect(),
                    Kind::Ref | Kind::RefList => slot.choices.get(&i).cloned().unwrap_or_default(),
                    _ => Vec::new(),
                };
                FieldModel {
                    id: c.key.map_or_else(|| format!("m{i}"), |k| format!("m{i}-{k:x}")),
                    key: c.key,
                    tag: c.tag,
                    kind,
                    label: f.label.clone(),
                    section: f.section.clone(),
                    help: f.help.clone(),
                    value: c.value.clone(),
                    stock: Some(c.stock.clone()),
                    edited: c.edited,
                    min: f.min,
                    max: f.max,
                    step: f.step,
                    unit: f.unit.clone(),
                    options,
                    text: c.text.as_deref().map(clean),
                    readonly: f.readonly,
                    optional: f.optional,
                    shared: f.path.contains('[') || f.path.contains('{'),
                    others: c.key.and_then(|k| self.db_others(k)),
                    row,
                    all: (c.key.is_some() && module.sets_all(i, c.tag)).then_some(i),
                }
            })
            .collect()
    }

    pub(crate) fn db_key_field(&self, node: NodeId) -> Option<FieldModel> {
        let catalog = self.db.catalog()?;
        let record = catalog.record(catalog.node(node).record?);
        let key = record.key;
        let meta = catalog.meta(key);
        let (min, max) = meta.map_or((None, None), |m| range(&m.range));
        let edit = self.db.edit(key);
        let value = edit
            .map(|e| e.value.clone().unwrap_or_default())
            .unwrap_or_else(|| record.value.to_owned());
        let name = &catalog.node(node).name;
        Some(FieldModel {
            id: format!("k{key:x}"),
            key: Some(key),
            tag: record.tag,
            kind: kind_of(None, record.tag),
            label: "Value".into(),
            section: None,
            help: None,
            text: name
                .starts_with("lng_")
                .then(|| catalog.text(&value).map(clean))
                .flatten(),
            value,
            stock: Some(record.value.to_owned()),
            edited: edit.is_some(),
            min,
            max,
            step: None,
            unit: meta.map(|m| m.unit.clone()).filter(|u| !u.is_empty()),
            options: Vec::new(),
            readonly: false,
            optional: false,
            shared: false,
            others: self.db_others(key),
            row: None,
            all: None,
        })
    }

    fn db_detail_fields(&self) -> Vec<FieldModel> {
        match (&self.db.nav, self.db.key) {
            (Nav::Module(_), _) => self.db_row_fields(),
            (Nav::Raw(_) | Nav::Unnamed, Some(node)) => self.db_key_field(node).into_iter().collect(),
            _ => Vec::new(),
        }
    }

    pub(crate) fn sync_fields(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for s in self.db.slots.inputs.values_mut() {
            s.seen = false;
        }
        for s in self.db.slots.selects.values_mut() {
            s.seen = false;
        }
        for s in self.db.slots.colors.values_mut() {
            s.seen = false;
        }
        for s in self.db.slots.copies.values_mut() {
            s.seen = false;
        }
        let fields = self.db_detail_fields();
        let mut missing: Vec<usize> = fields
            .iter()
            .filter_map(|f| match &f.row {
                Some((spec, Backing::Missing)) => Some(*spec),
                _ => None,
            })
            .collect();
        missing.dedup();
        for spec in missing {
            self.sync_copy(spec, window, cx);
        }
        for f in fields {
            let Some(key) = f.key else {
                continue;
            };
            match f.kind {
                Kind::Int | Kind::Float => {
                    let (min, max, step) = (f.min, f.max, f.step);
                    self.sync_input(&f.id, key, &f.value, window, cx, move |s| {
                        let mut s = s.step(step.unwrap_or(1.));
                        if let Some(m) = min {
                            s = s.min(m);
                        }
                        if let Some(m) = max {
                            s = s.max(m);
                        }
                        s
                    });
                }
                Kind::Text | Kind::Time => {
                    let ph = if f.kind == Kind::Time { "HH,MM" } else { "" };
                    self.sync_input(&f.id, key, &f.value, window, cx, move |s| s.placeholder(ph));
                }
                Kind::List | Kind::Vector => {
                    for (i, v) in list_items(&f.value).iter().enumerate() {
                        self.sync_input(&format!("{}/i{i}", f.id), key, v, window, cx, |s| s);
                    }
                }
                Kind::Color => {
                    self.sync_input(&f.id, key, &f.value, window, cx, |s| s);
                    self.sync_color(&f.id, key, &f.value, window, cx);
                }
                Kind::Choice | Kind::Ref => {
                    self.sync_select(&f.id, key, &f.value, &f.value, &f.options, window, cx);
                }
                Kind::RefList => {
                    let have = list_items(&f.value);
                    let rest: Vec<(String, String)> = f
                        .options
                        .iter()
                        .filter(|(v, _)| !have.contains(v))
                        .cloned()
                        .collect();
                    self.sync_select(&format!("{}/add", f.id), key, "", &f.value, &rest, window, cx);
                }
                Kind::Curve => match parse_curve(&f.value) {
                    Some(c) => {
                        for (i, (x, y)) in c.points.iter().enumerate() {
                            let y = shown_y(&c, y);
                            self.sync_input(&format!("{}/x{i}", f.id), key, x, window, cx, |s| s);
                            self.sync_input(&format!("{}/y{i}", f.id), key, &y, window, cx, |s| s);
                        }
                    }
                    None => self.sync_input(&f.id, key, &f.value, window, cx, |s| s),
                },
                Kind::Bool => {}
            }
        }
        self.db.slots.inputs.retain(|_, s| s.seen);
        self.db.slots.selects.retain(|_, s| s.seen);
        self.db.slots.colors.retain(|_, s| s.seen);
        self.db.slots.copies.retain(|_, s| s.seen);
    }

    fn copy_id(&self, spec: usize) -> Option<String> {
        let slot = self.db.view.as_ref()?;
        let row = self.db.row.as_ref()?;
        Some(format!(
            "{}/{}/{row}/{spec}",
            slot.module,
            slot.scope.as_deref().unwrap_or_default()
        ))
    }

    /// The "Copy from…" picker of a row spec: every entity with a row of its own, and the game default.
    fn sync_copy(&mut self, spec: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.copy_id(spec) else {
            return;
        };
        if let Some(s) = self.db.slots.copies.get_mut(&id) {
            s.seen = true;
            return;
        }
        let (Some(catalog), Some(slot)) = (self.db.catalog().cloned(), self.db.view.as_ref()) else {
            return;
        };
        let Some(module) = self.db.module(&slot.module) else {
            return;
        };
        let (labels, default) = module.row_sources(&catalog, spec, slot.scope.as_deref());
        let names: HashMap<&str, &str> = slot
            .view
            .rows
            .iter()
            .filter(|r| !r.name.is_empty())
            .map(|r| (r.label.as_str(), r.name.as_str()))
            .collect();
        let mut items: Vec<Choice> = labels
            .into_iter()
            .map(|l| Choice {
                title: SharedString::from(names.get(l.as_str()).map_or_else(|| l.clone(), |n| n.to_string())),
                value: l,
            })
            .collect();
        items.sort_by_key(|c| c.title.to_lowercase());
        if default {
            items.insert(
                0,
                Choice {
                    value: String::new(),
                    title: "Game default".into(),
                },
            );
        }
        let state =
            cx.new(|cx| SelectState::new(SearchableVec::new(items), None, window, cx).searchable(true));
        let sub = cx.subscribe_in(
            &state,
            window,
            move |this, _, ev: &SelectEvent<Choices>, _, cx| {
                if let SelectEvent::Confirm(Some(v)) = ev {
                    let from = (!v.is_empty()).then(|| v.clone());
                    this.db_copy_row(spec, from, cx);
                }
            },
        );
        self.db.slots.copies.insert(
            id,
            CopySlot {
                state,
                seen: true,
                _sub: sub,
            },
        );
    }

    fn sync_input(
        &mut self,
        id: &str,
        key: u64,
        value: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
        build: impl FnOnce(InputState) -> InputState,
    ) {
        if let Some(slot) = self.db.slots.inputs.get_mut(id) {
            slot.seen = true;
            let focused = slot.input.read(cx).focus_handle(cx).is_focused(window);
            if slot.synced != value && !focused {
                slot.synced = value.to_owned();
                let text = value.to_owned();
                slot.input.update(cx, |s, cx| s.set_value(text, window, cx));
            }
            return;
        }
        let text = value.to_owned();
        let input = cx.new(|cx| build(InputState::new(window, cx).default_value(text)));
        let slot_id = id.to_owned();
        let commit = slot_id.clone();
        let sub = cx.subscribe_in(&input, window, move |this, _, ev: &InputEvent, _, cx| {
            if matches!(ev, InputEvent::PressEnter { .. } | InputEvent::Blur) {
                this.db_commit_input(&commit, cx);
            }
        });
        let step = cx.subscribe_in(&input, window, move |this, _, _: &NumberInputEvent, _, cx| {
            this.db_commit_input(&slot_id, cx);
        });
        self.db.slots.inputs.insert(
            id.to_owned(),
            InputSlot {
                key,
                input,
                synced: value.to_owned(),
                seen: true,
                _subs: vec![sub, step],
            },
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn sync_select(
        &mut self,
        id: &str,
        key: u64,
        value: &str,
        whole: &str,
        options: &[(String, String)],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let synced = format!("{whole}\u{0}{}", options.len());
        if let Some(slot) = self.db.slots.selects.get_mut(id) {
            slot.seen = true;
            if slot.synced == synced {
                return;
            }
        }
        let items: Vec<Choice> = options
            .iter()
            .map(|(v, name)| Choice {
                value: v.clone(),
                title: SharedString::from(if name.is_empty() { v.clone() } else { name.clone() }),
            })
            .collect();
        let at = items
            .iter()
            .position(|c| c.value == value)
            .map(|r| gpui_kit::component::IndexPath::default().row(r));
        let state = cx.new(|cx| SelectState::new(SearchableVec::new(items), at, window, cx).searchable(true));
        let adds = id.ends_with("/add");
        let slot_id = id.to_owned();
        let sub = cx.subscribe_in(
            &state,
            window,
            move |this, _, ev: &SelectEvent<Choices>, _, cx| {
                let SelectEvent::Confirm(Some(v)) = ev else {
                    return;
                };
                let Some(key) = this.db_slot_key(&slot_id) else {
                    return;
                };
                if adds {
                    let current = this.db.current(key);
                    let mut items = list_items(&current);
                    items.push(v.clone());
                    this.db_set(key, &items.join(sep_of(&current)), cx);
                } else {
                    this.db_set(key, v, cx);
                }
            },
        );
        self.db.slots.selects.insert(
            id.to_owned(),
            SelectSlot {
                key,
                state,
                synced,
                seen: true,
                _sub: sub,
            },
        );
    }

    fn sync_color(&mut self, id: &str, key: u64, value: &str, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(slot) = self.db.slots.colors.get_mut(id) {
            slot.seen = true;
            if slot.synced != value {
                slot.synced = value.to_owned();
                if let Some(c) = swatch_color(value) {
                    slot.state.update(cx, |s, cx| s.set_value(c, window, cx));
                }
            }
            return;
        }
        let color = swatch_color(value);
        let state = cx.new(|cx| {
            let s = ColorPickerState::new(window, cx);
            match color {
                Some(c) => s.default_value(c),
                None => s,
            }
        });
        let slot_id = id.to_owned();
        let sub = cx.subscribe_in(&state, window, move |this, _, ev: &ColorPickerEvent, _, cx| {
            let ColorPickerEvent::Change(Some(c)) = ev else {
                return;
            };
            if let Some(key) = this.db_slot_key(&slot_id) {
                this.db_set(key, &color_text(*c), cx);
            }
        });
        self.db.slots.colors.insert(
            id.to_owned(),
            ColorSlot {
                key,
                state,
                synced: value.to_owned(),
                seen: true,
                _sub: sub,
            },
        );
    }

    fn db_slot_key(&self, id: &str) -> Option<u64> {
        let base = id.split_once('/').map_or(id, |(b, _)| b);
        self.db_detail_fields()
            .into_iter()
            .find(|f| f.id == base)
            .and_then(|f| f.key)
    }

    fn db_commit_input(&mut self, id: &str, cx: &mut Context<Self>) {
        let Some(key) = self.db_slot_key(id) else {
            return;
        };
        let Some(slot) = self.db.slots.inputs.get(id) else {
            return;
        };
        let text = slot.input.read(cx).value().trim().to_owned();
        let part = id
            .rsplit_once('/')
            .filter(|(_, t)| t.starts_with(['x', 'y', 'i']));
        let Some((base, tail)) = part else {
            if text != slot.synced {
                self.db_set(key, &text, cx);
            }
            return;
        };
        let current = self.db.current(key);
        let edited = |slot: &str| {
            self.db
                .slots
                .inputs
                .get(slot)
                .map(|s| (s.input.read(cx).value().trim().to_owned(), s.synced.clone()))
                .filter(|(now, was)| now != was)
                .map(|(now, _)| now)
        };
        if tail.starts_with('i') {
            let mut items = list_items(&current);
            for (i, item) in items.iter_mut().enumerate() {
                if let Some(v) = edited(&format!("{base}/i{i}")) {
                    *item = v;
                }
            }
            let next = items.join(sep_of(&current));
            if next != current {
                self.db_set(key, &next, cx);
            }
            return;
        }
        let Some(stock) = parse_curve(&current) else {
            return;
        };
        let Some(mut curve) = parse_curve(&current) else {
            return;
        };
        let mut bad = None;
        for (i, (x, y)) in curve.points.iter_mut().enumerate() {
            if let Some(v) = edited(&format!("{base}/x{i}")) {
                *x = v;
            }
            if let Some(v) = edited(&format!("{base}/y{i}")) {
                match stored_y(&stock, &v) {
                    Some(stored) => *y = stored,
                    None => bad = Some(format!("{v} is not a number")),
                }
            }
        }
        if let Some(e) = bad {
            self.db.errors.insert(key, e);
            cx.notify();
            return;
        }
        let next = curve_text(&curve);
        if next != current {
            self.db_set(key, &next, cx);
        }
    }

    fn db_curve_edit(
        &mut self,
        key: u64,
        f: impl FnOnce(&mut Vec<(String, String)>),
        cx: &mut Context<Self>,
    ) {
        let current = self.db.current(key);
        if let Some(mut c) = parse_curve(&current) {
            f(&mut c.points);
            self.db.slots.forget(key);
            self.db_set(key, &curve_text(&c), cx);
        }
    }

    fn db_list_edit(&mut self, key: u64, f: impl FnOnce(&mut Vec<String>), cx: &mut Context<Self>) {
        let current = self.db.current(key);
        let mut items = list_items(&current);
        f(&mut items);
        self.db_set(key, &items.join(sep_of(&current)), cx);
    }

    pub(crate) fn db_fields_panel(
        &self,
        fields: &[FieldModel],
        p: Palette,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let mut sections: Vec<(Option<String>, Vec<&FieldModel>)> = Vec::new();
        for f in fields {
            match sections.iter_mut().find(|(s, _)| *s == f.section) {
                Some((_, list)) => list.push(f),
                None => sections.push((f.section.clone(), vec![f])),
            }
        }
        let mut shown: Vec<usize> = Vec::new();
        sections
            .into_iter()
            .map(|(title, list)| {
                let strip = list
                    .iter()
                    .filter_map(|f| f.row.as_ref())
                    .find(|(spec, b)| *b != Backing::Stock && !shown.contains(spec))
                    .cloned();
                if let Some((spec, _)) = &strip {
                    shown.push(*spec);
                }
                v_flex()
                    .flex_shrink_0()
                    .gap(px(14.))
                    .when_some(title, |d, t| d.child(label(t.to_uppercase(), p)))
                    .when_some(strip, |d, (spec, b)| d.child(self.db_row_strip(spec, &b, p, cx)))
                    .children(list.into_iter().map(|f| self.db_field(f, p, cx)))
                    .into_any_element()
            })
            .collect()
    }

    /// Above the fields of another table's row: a picker to copy a row when this
    /// entity has none, or where its own row came from with a way to remove it.
    fn db_row_strip(&self, spec: usize, backing: &Backing, p: Palette, cx: &mut Context<Self>) -> AnyElement {
        let locked = self.db_locked();
        match backing {
            Backing::Copied(from) => {
                let from = match from {
                    Some(l) => self
                        .db
                        .view
                        .as_ref()
                        .and_then(|v| v.view.rows.iter().find(|r| r.label == *l))
                        .map(|r| r.name.clone())
                        .filter(|n| !n.is_empty())
                        .unwrap_or_else(|| l.clone()),
                    None => "the game default".into(),
                };
                h_flex()
                    .gap(px(8.))
                    .min_h(px(22.))
                    .child(Icon::new(IconName::Copy).size(px(13.)).text_color(p.accent))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_size(px(12.))
                            .text_color(p.text_muted)
                            .truncate()
                            .child(format!("Copied from {from}")),
                    )
                    .child(
                        Button::new(SharedString::from(format!("db-row-remove-{spec}")))
                            .ghost()
                            .xsmall()
                            .icon(IconName::Trash)
                            .label("Remove own row")
                            .disabled(locked)
                            .on_click(cx.listener(move |this, _, _, cx| this.db_remove_row(spec, cx))),
                    )
                    .into_any_element()
            }
            _ => {
                let picker = self
                    .copy_id(spec)
                    .and_then(|id| self.db.slots.copies.get(&id))
                    .map(|s| {
                        Select::new(&s.state)
                            .id(SharedString::from(format!("db-row-copy-{spec}")))
                            .small()
                            .w_full()
                            .menu_max_h(px(320.))
                            .search_placeholder("Search")
                            .placeholder("Copy from…")
                            .disabled(locked)
                    });
                v_flex()
                    .gap(px(8.))
                    .px(px(10.))
                    .py(px(9.))
                    .rounded(px(6.))
                    .border_1()
                    .border_color(p.border_variant)
                    .bg(p.surface)
                    .child(
                        h_flex()
                            .gap(px(8.))
                            .items_start()
                            .text_size(px(12.))
                            .line_height(px(17.))
                            .text_color(p.text)
                            .child(
                                Icon::new(IconName::Info)
                                    .size(px(13.))
                                    .mt(px(2.))
                                    .text_color(p.text_muted),
                            )
                            .child(div().flex_1().min_w_0().child(
                                "These values come from the game default. Copy someone's row to edit them.",
                            )),
                    )
                    .children(picker)
                    .into_any_element()
            }
        }
    }

    fn db_set_all_menu(&self, f: &FieldModel, locked: bool, cx: &mut Context<Self>) -> Option<AnyElement> {
        let field = f.all.filter(|_| !locked)?;
        let n = self.db_listed(field);
        if n < 2 {
            return None;
        }
        let mut picks: Vec<(String, String, String)> = Vec::new();
        if !f.value.is_empty() {
            let shown = shown_value(f, &f.value);
            picks.push((format!("Set all {n} to {shown}"), f.value.clone(), shown));
        }
        for (end, limit) in [("max", f.max), ("min", f.min)] {
            if let Some(v) = limit {
                picks.push((
                    format!("Set all {n} to the {end}, {v}"),
                    format!("{v}"),
                    format!("{v}"),
                ));
            }
        }
        if picks.is_empty() {
            return None;
        }
        let shell = cx.entity().downgrade();
        Some(
            Button::new(SharedString::from(format!("all-{}", f.id)))
                .ghost()
                .xsmall()
                .icon(IconName::ListChecks)
                .label("All rows")
                .dropdown_menu_with_anchor(Anchor::TopRight, move |mut menu, _, _| {
                    for (text, value, shown) in &picks {
                        let (shell, value, shown) = (shell.clone(), value.clone(), shown.clone());
                        menu = menu.item(PopupMenuItem::new(text.clone()).on_click(move |_, window, cx| {
                            shell
                                .update(cx, |s, cx| {
                                    s.db_confirm_set_all(field, value.clone(), shown.clone(), window, cx)
                                })
                                .ok();
                        }));
                    }
                    menu
                })
                .into_any_element(),
        )
    }

    pub(crate) fn db_field(&self, f: &FieldModel, p: Palette, cx: &mut Context<Self>) -> AnyElement {
        let Some(key) = f.key else {
            return v_flex()
                .gap(px(4.))
                .child(
                    div()
                        .text_size(px(12.))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(p.text_muted)
                        .child(f.label.clone()),
                )
                .child(div().text_size(px(13.)).text_color(p.text_placeholder).child(
                    match (&f.row, f.optional) {
                        (Some((_, Backing::Copied(_))), _) => "Not in the copied row",
                        (_, true) => "Inherited",
                        (_, false) => "Not in this row",
                    },
                ))
                .into_any_element();
        };
        let locked = f.readonly || self.db_locked();
        let error = self.db.errors.get(&key).cloned();
        let revert = f.edited.then(|| {
            Button::new(SharedString::from(format!("revert-{}", f.id)))
                .ghost()
                .xsmall()
                .icon(IconName::Undo2)
                .label("Revert")
                .disabled(self.db_locked())
                .on_click(cx.listener(move |this, _, _, cx| this.db_revert(key, cx)))
        });
        let all = self.db_set_all_menu(f, locked, cx);
        let control = self.db_control(f, key, locked, p, cx);
        let stock = f.stock.as_ref().filter(|_| f.edited).map(|s| shown_value(f, s));
        v_flex()
            .flex_shrink_0()
            .gap(px(6.))
            .child(
                h_flex()
                    .gap(px(6.))
                    .min_h(px(22.))
                    .child(
                        div()
                            .text_size(px(12.))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(if f.edited { p.accent } else { p.text_muted })
                            .truncate()
                            .child(f.label.clone()),
                    )
                    .when(f.edited, |d| {
                        d.child(div().size(px(6.)).flex_shrink_0().rounded_full().bg(p.accent))
                    })
                    .when(f.readonly, |d| {
                        d.child(
                            Icon::new(IconName::Lock)
                                .size(px(11.))
                                .text_color(p.text_placeholder),
                        )
                    })
                    .child(div().flex_1())
                    .children(all)
                    .children(revert),
            )
            .child(control)
            .when_some(
                f.text
                    .clone()
                    .filter(|_| !matches!(f.kind, Kind::Ref | Kind::RefList | Kind::Choice)),
                |d, t| {
                    d.child(
                        h_flex()
                            .gap(px(6.))
                            .text_size(px(12.))
                            .text_color(p.text_muted)
                            .child(
                                Icon::new(IconName::Languages)
                                    .size(px(12.))
                                    .text_color(p.text_placeholder),
                            )
                            .child(div().min_w_0().truncate().child(t)),
                    )
                },
            )
            .when_some(error, |d, e| {
                d.child(
                    h_flex()
                        .gap(px(6.))
                        .items_start()
                        .text_size(px(12.))
                        .line_height(px(17.))
                        .text_color(p.error)
                        .child(Icon::new(IconName::CircleAlert).size(px(12.)).mt(px(2.)))
                        .child(div().flex_1().min_w_0().child(e)),
                )
            })
            .when_some(stock, |d, s| {
                d.child(
                    div()
                        .text_size(px(12.))
                        .text_color(p.text_placeholder)
                        .truncate()
                        .flex_shrink_0()
                        .child(match &f.row {
                            Some((_, Backing::Copied(_))) => format!("Copied: {s}"),
                            _ => format!("Stock: {s}"),
                        }),
                )
            })
            .when_some(f.others.clone(), |d, t| {
                d.child(
                    h_flex()
                        .gap(px(6.))
                        .items_start()
                        .text_size(px(12.))
                        .line_height(px(17.))
                        .text_color(p.text_muted)
                        .child(
                            Icon::new(IconName::Layers)
                                .size(px(12.))
                                .mt(px(2.))
                                .text_color(p.warning),
                        )
                        .child(div().flex_1().min_w_0().child(t)),
                )
            })
            .when_some(f.help.clone(), |d, h| {
                if f.shared {
                    d.child(
                        h_flex()
                            .gap(px(8.))
                            .items_start()
                            .px(px(10.))
                            .py(px(7.))
                            .rounded(px(6.))
                            .bg(p.warning.opacity(0.1))
                            .text_size(px(12.))
                            .line_height(px(17.))
                            .text_color(p.text)
                            .child(
                                Icon::new(IconName::TriangleAlert)
                                    .size(px(13.))
                                    .mt(px(2.))
                                    .text_color(p.warning),
                            )
                            .child(div().flex_1().min_w_0().child(h)),
                    )
                } else {
                    d.child(
                        div()
                            .text_size(px(12.))
                            .line_height(px(17.))
                            .text_color(p.text_placeholder)
                            .child(h),
                    )
                }
            })
            .into_any_element()
    }

    fn db_control(
        &self,
        f: &FieldModel,
        key: u64,
        locked: bool,
        p: Palette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let slots = &self.db.slots;
        let unit = f
            .unit
            .clone()
            .map(|u| div().text_size(px(12.)).text_color(p.text_placeholder).child(u));
        match f.kind {
            Kind::Int | Kind::Float => match slots.inputs.get(&f.id) {
                Some(s) => NumberInput::new(&s.input)
                    .small()
                    .disabled(locked)
                    .when_some(unit, |n, u| n.suffix(u))
                    .into_any_element(),
                None => plain(&f.value, p),
            },
            Kind::List | Kind::Vector => {
                let items = list_items(&f.value);
                let wide = items.iter().any(|v| v.len() > 6);
                h_flex()
                    .flex_wrap()
                    .gap(px(6.))
                    .children(items.iter().enumerate().map(|(i, v)| {
                        let input = slots
                            .inputs
                            .get(&format!("{}/i{i}", f.id))
                            .map(|s| Input::new(&s.input).xsmall().disabled(locked).into_any_element())
                            .unwrap_or_else(|| plain(v, p));
                        div().w(px(if wide { 120. } else { 58. })).child(input)
                    }))
                    .when(items.is_empty(), |d| d.child(plain("empty", p)))
                    .into_any_element()
            }
            Kind::Text | Kind::Time => match slots.inputs.get(&f.id) {
                Some(s) => Input::new(&s.input)
                    .small()
                    .disabled(locked)
                    .when_some(unit, |n, u| n.suffix(u))
                    .into_any_element(),
                None => plain(&f.value, p),
            },
            Kind::Bool => {
                let on = is_on(&f.value);
                let (tag, stock) = (f.tag, f.stock.clone().unwrap_or_else(|| f.value.clone()));
                h_flex()
                    .h(px(28.))
                    .gap(px(10.))
                    .child(
                        Switch::new(SharedString::from(format!("switch-{}", f.id)))
                            .checked(on)
                            .small()
                            .disabled(locked)
                            .on_change(cx.listener(move |this, on: &bool, _, cx| {
                                this.db_set(key, &bool_text(tag, &stock, *on), cx);
                            })),
                    )
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(if on { p.text } else { p.text_muted })
                            .child(if on { "On" } else { "Off" }),
                    )
                    .into_any_element()
            }
            Kind::Choice | Kind::Ref => match slots.selects.get(&f.id) {
                Some(s) => {
                    let shown = f
                        .options
                        .iter()
                        .find(|(v, _)| *v == f.value)
                        .map(|(v, n)| if n.is_empty() { v.clone() } else { n.clone() })
                        .or_else(|| f.text.clone())
                        .unwrap_or_else(|| f.value.clone());
                    Select::new(&s.state)
                        .id(SharedString::from(format!("select-{}", f.id)))
                        .small()
                        .w_full()
                        .menu_max_h(px(320.))
                        .search_placeholder("Search")
                        .placeholder(shown)
                        .disabled(locked)
                        .into_any_element()
                }
                None => plain(&f.value, p),
            },
            Kind::Color => {
                let picker = slots
                    .colors
                    .get(&f.id)
                    .map(|s| ColorPicker::new(&s.state).small());
                let raw = slots
                    .inputs
                    .get(&f.id)
                    .map(|s| Input::new(&s.input).small().disabled(locked));
                h_flex()
                    .gap(px(8.))
                    .children(picker.filter(|_| !locked))
                    .when(locked, |d| d.child(swatch(&f.value, 22., p)))
                    .child(div().flex_1().min_w_0().children(raw))
                    .child(
                        div()
                            .w(px(64.))
                            .flex_shrink_0()
                            .text_size(px(12.))
                            .text_color(p.text_muted)
                            .child(hex_rgb(&f.value).unwrap_or_default()),
                    )
                    .into_any_element()
            }
            Kind::RefList => self.db_ref_list(f, key, locked, p, cx),
            Kind::Curve => self.db_curve(f, key, locked, p, cx),
        }
    }

    fn db_ref_list(
        &self,
        f: &FieldModel,
        key: u64,
        locked: bool,
        p: Palette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let items = list_items(&f.value);
        let n = items.len();
        let name = |v: &str| {
            f.options
                .iter()
                .find(|(o, _)| o == v)
                .map(|(_, n)| n.clone())
                .filter(|n| !n.is_empty())
        };
        let rows = items.iter().enumerate().map(|(i, v)| {
            let small = |id: &str, icon: IconName| {
                Button::new(SharedString::from(format!("{}-{id}-{i}", f.id)))
                    .ghost()
                    .xsmall()
                    .icon(icon)
                    .disabled(locked)
            };
            h_flex()
                .h(px(32.))
                .px(px(8.))
                .gap(px(8.))
                .when(i > 0, |d| d.border_t_1().border_color(p.border_variant))
                .child(
                    div()
                        .w(px(20.))
                        .flex_shrink_0()
                        .text_size(px(12.))
                        .text_color(p.text_placeholder)
                        .child((i + 1).to_string()),
                )
                .child(
                    v_flex().flex_1().min_w_0().child(
                        div()
                            .text_size(px(13.))
                            .truncate()
                            .child(name(v).unwrap_or_else(|| v.clone())),
                    ),
                )
                .child(
                    small("up", IconName::ArrowUp)
                        .disabled(locked || i == 0)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.db_list_edit(key, |l| l.swap(i, i - 1), cx)
                        })),
                )
                .child(
                    small("down", IconName::ArrowDown)
                        .disabled(locked || i + 1 == n)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.db_list_edit(key, |l| l.swap(i, i + 1), cx)
                        })),
                )
                .child(
                    small("remove", IconName::X).on_click(cx.listener(move |this, _, _, cx| {
                        this.db_list_edit(
                            key,
                            |l| {
                                l.remove(i);
                            },
                            cx,
                        )
                    })),
                )
        });
        let add = self.db.slots.selects.get(&format!("{}/add", f.id)).map(|s| {
            Select::new(&s.state)
                .id(SharedString::from(format!("add-{}", f.id)))
                .small()
                .w_full()
                .menu_max_h(px(320.))
                .search_placeholder("Search")
                .placeholder("Add…")
                .disabled(locked)
        });
        v_flex()
            .gap(px(8.))
            .child(
                v_flex()
                    .rounded(px(6.))
                    .border_1()
                    .border_color(p.border_variant)
                    .bg(p.background)
                    .children(rows)
                    .when(n == 0, |d| {
                        d.child(
                            div()
                                .px(px(10.))
                                .py(px(8.))
                                .text_size(px(13.))
                                .text_color(p.text_placeholder)
                                .child("Empty"),
                        )
                    }),
            )
            .children(add)
            .into_any_element()
    }

    fn db_curve(
        &self,
        f: &FieldModel,
        key: u64,
        locked: bool,
        p: Palette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(c) = parse_curve(&f.value) else {
            return match self.db.slots.inputs.get(&f.id) {
                Some(s) => Input::new(&s.input).small().disabled(locked).into_any_element(),
                None => plain(&f.value, p),
            };
        };
        let head: Vec<String> = c.head.iter().map(|(a, b)| format!("{a} {b}")).collect();
        let n = c.points.len();
        let pts: Vec<(f32, f32)> = c
            .points
            .iter()
            .filter_map(|(x, y)| Some((x.parse::<f32>().ok()?, y.parse::<f32>().ok()?)))
            .collect();
        let line = p.accent;
        let grid = p.border_variant;
        let spark = canvas(
            |_, _, _| {},
            move |b, _, window, _| {
                let (x0, x1) = pts
                    .iter()
                    .fold((f32::MAX, f32::MIN), |(a, z), (x, _)| (a.min(*x), z.max(*x)));
                let (y0, y1) = pts
                    .iter()
                    .fold((0f32, 1f32), |(a, z), (_, y)| (a.min(*y), z.max(*y)));
                let at = |x: f32, y: f32| {
                    let fx = if x1 > x0 { (x - x0) / (x1 - x0) } else { 0. };
                    let fy = if y1 > y0 { (y - y0) / (y1 - y0) } else { 0. };
                    point(
                        b.origin.x + b.size.width * fx,
                        b.origin.y + b.size.height * (1. - fy),
                    )
                };
                let mut base = PathBuilder::stroke(px(1.));
                base.move_to(point(b.origin.x, b.origin.y + b.size.height));
                base.line_to(point(b.origin.x + b.size.width, b.origin.y + b.size.height));
                if let Ok(path) = base.build() {
                    window.paint_path(path, grid);
                }
                let mut path = PathBuilder::stroke(px(1.5));
                for (i, (x, y)) in pts.iter().enumerate() {
                    if i == 0 {
                        path.move_to(at(*x, *y));
                    } else {
                        path.line_to(at(*x, *y));
                    }
                }
                if pts.len() > 1
                    && let Ok(path) = path.build()
                {
                    window.paint_path(path, line);
                }
            },
        )
        .w_full()
        .h(px(44.));
        let rows = (0..n).map(|i| {
            let input = |axis: &str| {
                self.db
                    .slots
                    .inputs
                    .get(&format!("{}/{axis}{i}", f.id))
                    .map(|s| Input::new(&s.input).xsmall().disabled(locked))
            };
            h_flex()
                .gap(px(6.))
                .child(
                    div()
                        .w(px(18.))
                        .text_size(px(11.))
                        .text_color(p.text_placeholder)
                        .child((i + 1).to_string()),
                )
                .child(div().flex_1().min_w_0().children(input("x")))
                .child(div().flex_1().min_w_0().children(input("y")))
                .child(
                    Button::new(SharedString::from(format!("{}-pt-{i}", f.id)))
                        .ghost()
                        .xsmall()
                        .icon(IconName::X)
                        .disabled(locked || n <= 2)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.db_curve_edit(
                                key,
                                |pts| {
                                    pts.remove(i);
                                },
                                cx,
                            )
                        })),
                )
        });
        v_flex()
            .gap(px(6.))
            .child(
                div()
                    .px(px(8.))
                    .py(px(6.))
                    .rounded(px(6.))
                    .border_1()
                    .border_color(p.border_variant)
                    .bg(p.background)
                    .child(spark),
            )
            .child(
                h_flex()
                    .gap(px(6.))
                    .text_size(px(11.))
                    .text_color(p.text_placeholder)
                    .child(div().w(px(18.)))
                    .child(div().flex_1().child("X"))
                    .child(
                        div()
                            .flex_1()
                            .child(format!("VALUE {}–{}", num(c.span().0), num(c.span().1))),
                    )
                    .child(div().w(px(22.))),
            )
            .children(rows)
            .child(
                h_flex()
                    .gap(px(8.))
                    .child(
                        Button::new(SharedString::from(format!("{}-add-pt", f.id)))
                            .ghost()
                            .xsmall()
                            .icon(IconName::Plus)
                            .label("Add point")
                            .disabled(locked)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.db_curve_edit(
                                    key,
                                    |pts| {
                                        let last = pts.last().cloned().unwrap_or(("1".into(), "1".into()));
                                        pts.push(last);
                                    },
                                    cx,
                                )
                            })),
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(p.text_placeholder)
                            .child(head.join(" · ")),
                    ),
            )
            .into_any_element()
    }

    pub(crate) fn db_key_detail(&self, node: NodeId, p: Palette, cx: &mut Context<Self>) -> AnyElement {
        let Some(catalog) = self.db.catalog().cloned() else {
            return div().into_any_element();
        };
        let n = catalog.node(node);
        let Some(field) = self.db_key_field(node) else {
            return div().into_any_element();
        };
        let key = field.key.unwrap_or_default();
        let meta = catalog.meta(key).cloned().unwrap_or_default();
        let facts = [
            ("Key hash", hex(key)),
            ("Type", type_name(field.tag)),
            ("Meaning", meta.meaning.clone()),
            ("Unit", meta.unit.clone()),
            ("Range", meta.range.clone()),
            ("Example", meta.example.clone()),
            ("Evidence", meta.evidence.clone()),
        ];
        detail_shell(
            "db-key-detail",
            n.name.clone(),
            catalog.path(node),
            p,
            cx.listener(|this, _, _, cx| {
                this.db.key = None;
                cx.notify();
            }),
        )
        .child(
            v_flex()
                .flex_1()
                .min_h_0()
                .scroll_y("db-key-body")
                .px(px(16.))
                .py(px(14.))
                .gap(px(18.))
                .child(self.db_field(&field, p, cx))
                .child(
                    v_flex().children(
                        facts
                            .into_iter()
                            .filter(|(_, v)| !v.is_empty())
                            .map(|(l, v)| fact(l, v, p)),
                    ),
                ),
        )
        .into_any_element()
    }
}

fn shown_value(f: &FieldModel, v: &str) -> String {
    match f.kind {
        Kind::Choice | Kind::Ref => f
            .options
            .iter()
            .find(|(o, _)| o == v)
            .map(|(_, n)| n.clone())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| v.to_owned()),
        Kind::Color => hex_rgb(v).unwrap_or_else(|| v.to_owned()),
        _ if v.is_empty() => "empty".into(),
        _ => v.to_owned(),
    }
}

fn plain(v: &str, p: Palette) -> AnyElement {
    div()
        .text_size(px(13.))
        .text_color(p.text)
        .truncate()
        .child(v.to_owned())
        .into_any_element()
}

pub(crate) fn swatch(v: &str, size: f32, p: Palette) -> Div {
    div()
        .size(px(size))
        .flex_shrink_0()
        .rounded(px(4.))
        .border_1()
        .border_color(p.border)
        .when_some(swatch_color(v), |d, c| d.bg(c))
}

pub(crate) fn detail_shell(
    id: &'static str,
    title: String,
    subtitle: String,
    p: Palette,
    close: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    v_flex()
        .id(id)
        .w(px(380.))
        .flex_shrink_0()
        .h_full()
        .rounded(px(8.))
        .border_1()
        .border_color(p.border_variant)
        .bg(p.surface)
        .overflow_hidden()
        .child(
            h_flex()
                .px(px(16.))
                .py(px(12.))
                .gap(px(10.))
                .flex_shrink_0()
                .border_b_1()
                .border_color(p.border_variant)
                .child(
                    v_flex()
                        .flex_1()
                        .min_w_0()
                        .gap(px(2.))
                        .child(
                            div()
                                .text_size(px(15.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .truncate()
                                .child(title),
                        )
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(p.text_muted)
                                .truncate()
                                .child(subtitle),
                        ),
                )
                .child(
                    Button::new(SharedString::from(format!("{id}-close")))
                        .ghost()
                        .small()
                        .icon(IconName::X)
                        .tooltip("Close")
                        .on_click(close),
                ),
        )
}
