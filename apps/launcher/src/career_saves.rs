use std::path::{Path, PathBuf};

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::menu::{DropdownMenu, PopupMenuItem};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{Disableable, Icon, Sizable, WindowExt, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use swofty_core::install::Game;
use swofty_core::launcher_saves;
use swofty_core::profile::VANILLA;
use swofty_core::store::Store;
use swofty_core::vault::career_names;
use swofty_core::vault::careers::{self, Brief, Diff, Listed, Row, State as CareerState};
use swofty_core::vault::restore::Running;
use swofty_core::vault::{self, Location, Notice, Reason, Snapshot, Vault};

use crate::addons::ago;
use crate::library::plural;
use crate::pages::{GUTTER, label};
use crate::shell::{Page, Prompt, Shell};
use crate::theme::{Palette, ScrollArea};

const MODE_W: f32 = 124.;
const PLAYED_W: f32 = 104.;
const CHIP_W: f32 = 112.;
const WHEN_W: f32 = 104.;
const SIZE_W: f32 = 64.;
const ACTIONS_W: f32 = 164.;
const PIN_W: f32 = 18.;

pub(crate) const NOTICE_TITLE: &str = "Your careers have their own saves now";
pub(crate) const NOTICE_BODY: &str = "Each profile now has its own save, starting from a copy of your Steam save. Playing from Steam still uses your Steam save, and Vanilla plays on it too.";
const LISTED_LATER: &str = "careers listed after you play this profile once";

#[derive(Default)]
pub(crate) struct SavesPage {
    pub view: Option<SavesView>,
    pub confirm: Option<String>,
    pub banner: Option<Banner>,
    pub game_running: bool,
    notice_done: bool,
    generation: u64,
    pub loading: bool,
    pub banner_profile: String,
}

pub(crate) struct Built {
    banner: Option<Banner>,
    running: bool,
    view: SavesView,
}

pub(crate) enum Banner {
    Dropped {
        profile: String,
        source: String,
        snapshot: String,
        kept_at: i64,
        names: Vec<String>,
    },
    Hint(Notice),
}

struct Version {
    snap: Snapshot,
    held: Option<Vec<String>>,
    diff: Option<Diff>,
}

struct Entry {
    l: Listed,
    place: Option<String>,
}

impl Entry {
    fn mode(&self) -> String {
        self.l.mode_label()
    }
}

struct Imported {
    title: String,
    at: Option<i64>,
}

pub(crate) struct SavesView {
    profile: String,
    loc: Option<Location>,
    entries: Vec<Entry>,
    imported: Option<Imported>,
    has_save: bool,
    others: Vec<String>,
    versions: Vec<Version>,
    dir: PathBuf,
    usage: u64,
    cap: u64,
    steam: Option<Location>,
    steam_running: bool,
    awaiting_import: bool,
    steam_missing: bool,
}

impl SavesView {
    pub(crate) fn awaiting_import_for(&self, profile: &str) -> bool {
        self.awaiting_import && self.profile == profile
    }

    fn vanilla(&self) -> bool {
        self.profile == VANILLA
    }

    fn layout(&self) -> vault::Layout {
        self.loc
            .as_ref()
            .map(|l| l.layout)
            .or_else(|| self.versions.first().map(|v| v.snap.layout))
            .unwrap_or(vault::Layout::Launcher)
    }
}

impl SavesPage {
    pub(crate) fn banner_for(root: &Path, profile: &str) -> Option<Banner> {
        banner_from(root, source_of_profile(root, profile)?, profile.to_owned())
    }

    fn apply(&mut self, built: Built) {
        self.loading = false;
        self.banner_profile = built.view.profile.clone();
        self.banner = built.banner;
        self.game_running = built.running;
        self.view = Some(built.view);
    }

    pub(crate) fn take_notice(&mut self, root: &Path) -> bool {
        if self.notice_done || launcher_saves::marker(root).is_none() {
            return false;
        }
        self.notice_done = true;
        let due = launcher_saves::notice_due(root);
        if due && let Err(e) = launcher_saves::mark_seen(root) {
            crate::app_log::action("record the saves notice").failed(&e);
        }
        due
    }
}

pub(crate) const EMPTY_SAVE: &str =
    "This profile's launcher save is empty. Import from Steam brings your careers in.";
pub(crate) const EMPTY_SAVE_SHORT: &str = "Launcher save is empty · import from Steam on the Saves page";

pub(crate) const STEAM_SAVE_MISSING: &str =
    "Couldn't find your Steam save, so this profile's launcher saves are empty.";

pub(crate) fn awaiting_import(vanilla: bool, has_save: bool, steam_has_save: bool) -> bool {
    !vanilla && !has_save && steam_has_save
}

pub(crate) fn steam_save_missing(vanilla: bool, has_save: bool, steam_has_save: bool, pending: bool) -> bool {
    !vanilla && !has_save && !steam_has_save && pending
}

pub(crate) fn steam_save_in(folder: &Path) -> Option<Location> {
    let app = swofty_core::STEAM_APP_ID.to_string();
    let accounts = |dir: PathBuf| {
        std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|u| u.path().join(&app).join("remote"))
            .collect::<Vec<_>>()
    };
    [
        folder.to_path_buf(),
        folder.join("remote"),
        folder.join(&app).join("remote"),
    ]
    .into_iter()
    .chain(accounts(folder.to_path_buf()))
    .chain(accounts(folder.join("userdata")))
    .filter_map(|d| Location::steam(&d))
    .find(launcher_saves::has_files)
}

pub(crate) fn source_of_profile(root: &Path, profile: &str) -> Option<String> {
    match profile == VANILLA {
        true => Location::active(root).map(|l| l.source),
        false => Some(Location::launcher(root, profile).source),
    }
}

fn profile_of_source(source: &str) -> String {
    match source.strip_prefix("launcher-") {
        Some(p) => p.to_owned(),
        None => VANILLA.to_owned(),
    }
}

fn whose(profile: &str) -> String {
    match profile == VANILLA {
        true => "your Steam save".to_owned(),
        false => format!("profile {profile}'s save"),
    }
}

fn banner_from(root: &Path, source: String, profile: String) -> Option<Banner> {
    let v = Vault::open(root, &source).ok()?;
    let table = v.careers();
    let shown = table.drop_to_show();
    if let Some(newest) = shown.iter().max_by_key(|r| r.last_seen_at) {
        return Some(Banner::Dropped {
            profile,
            source,
            snapshot: newest.last_seen.clone(),
            kept_at: newest.last_seen_at,
            names: shown.iter().map(|r| row_name(r)).collect(),
        });
    }
    Notice::read(root).map(Banner::Hint)
}

fn row_name(r: &Row) -> String {
    let mut out = r.name().to_owned();
    if let Some(p) = r.place() {
        out.push_str(&format!(", {p}"));
    }
    out
}

fn mb(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
}

fn mb_whole(bytes: u64) -> String {
    format!("{} MB", (bytes as f64 / (1024.0 * 1024.0)).round() as u64)
}

fn list(items: &[Brief]) -> String {
    items
        .iter()
        .map(|b| match &b.place {
            Some(p) => format!("{} ({p})", b.name),
            None => b.name.clone(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn version_label(s: &Snapshot) -> &'static str {
    match s.pinned.iter().any(|p| p == "before-restore") {
        true => "Before restore",
        false => reason_label(s.reason),
    }
}

fn reason_label(r: Reason) -> &'static str {
    match r {
        Reason::BeforeSession => "Before Play",
        Reason::AfterSession => "After Play",
        Reason::Imported => "Imported from Steam",
        Reason::ImportedProfile => "Copied from another profile",
        Reason::LauncherStart => "Launcher start",
        Reason::ChangedOutside => "Changed outside the launcher",
        Reason::ForeignLaunch => "F1 25 started outside the launcher",
        Reason::ForeignExit => "F1 25 closed outside the launcher",
        Reason::BeforeRestore => "Before restore",
        Reason::Restored => "Restored",
        Reason::Migrated => "Old backup",
        Reason::Manual => "Backup",
        Reason::Other => "Saved",
    }
}

fn careers_held(held: &[String]) -> String {
    plural(held.len(), "career", "careers")
}

fn imported_from(snaps: &[Snapshot]) -> Imported {
    let found = snaps
        .iter()
        .find(|s| matches!(s.reason, Reason::Imported | Reason::ImportedProfile));
    let title = match found {
        Some(s) if s.reason == Reason::ImportedProfile => {
            let from = s
                .note
                .as_deref()
                .and_then(|n| n.strip_prefix("copied from "))
                .and_then(|d| Path::new(d).parent())
                .and_then(|d| d.file_name())
                .map(|n| n.to_string_lossy().into_owned());
            match from {
                Some(f) => format!("Copied from {f}"),
                None => "Copied from another profile".to_owned(),
            }
        }
        _ => "Imported Steam save".to_owned(),
    };
    Imported {
        title,
        at: found.map(|s| s.at),
    }
}

fn card_careers(root: &Path, store: &Store, game: Option<&Game>, profile: &str) -> Vec<Entry> {
    let listing = match game {
        Some(g) => career_names::read_named_listing(root, store, g, Some(profile)),
        None => careers::read_listing(root, Some(profile)),
    };
    listing
        .into_iter()
        .filter(|l| !l.hidden)
        .filter(|l| l.profile.as_deref().is_none_or(|p| p == profile))
        .map(|l| Entry {
            place: season_round(&l),
            l,
        })
        .collect()
}

fn season_round(l: &Listed) -> Option<String> {
    match (l.season_no, l.weekend) {
        (Some(s), Some(w)) => Some(format!("Season {s} · Round {}", w + 1)),
        (Some(s), None) => Some(format!("Season {s}")),
        _ => None,
    }
}

fn short_block(layout: vault::Layout, r: Running) -> Option<&'static str> {
    match (r.game, layout, r.steam) {
        (true, _, _) => Some("Close F1 25 first"),
        (false, vault::Layout::Steam, true) => Some("Close Steam first"),
        _ => None,
    }
}

type Indexed = (
    Option<Vec<String>>,
    Option<Vec<swofty_core::stdlib::careers::Career>>,
);

static INDEXES: std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<PathBuf, Indexed>>> =
    std::sync::LazyLock::new(Default::default);

fn indexed(v: &Vault, snap: &Snapshot) -> Indexed {
    let key = v.dir.join(&snap.id);
    if let Some(hit) = INDEXES.lock().ok().and_then(|m| m.get(&key).cloned()) {
        return hit;
    }
    let index = v.index_of(snap);
    let held = index.as_ref().map(|i| {
        vault::careers::in_save(i)
            .map(|c| match c.label.is_empty() {
                true => c.career.clone(),
                false => c.label.clone(),
            })
            .collect()
    });
    let out = (held, index);
    if let Ok(mut m) = INDEXES.lock() {
        m.insert(key, out.clone());
    }
    out
}

fn build_saves(store: &Store, install: Option<&crate::game::Install>, name: &str) -> Built {
    let started = std::time::Instant::now();
    let built = build_saves_untimed(store, install, name);
    crate::app_log::note(&format!(
        "saves: page loaded in {} ms ({} snapshots)",
        started.elapsed().as_millis(),
        built.view.versions.len()
    ));
    built
}

fn build_saves_untimed(store: &Store, install: Option<&crate::game::Install>, name: &str) -> Built {
    let root = store.root.clone();
    let name = name.to_owned();
    let banner = SavesPage::banner_for(&root, &name);
    let running = Running::now();
    let active_loc = Location::active(&root);
    let vanilla = name == VANILLA;
    let loc = match vanilla {
        true => active_loc.clone(),
        false => Some(Location::launcher(&root, &name)),
    };
    let vault = loc.as_ref().and_then(|l| Vault::of(&root, l).ok());
    let game = crate::sections::game(store, install);
    let entries = card_careers(&root, store, game.as_ref(), &name);
    let snaps = vault.as_ref().and_then(|v| v.list().ok()).unwrap_or_default();
    let has_save = loc.as_ref().is_some_and(launcher_saves::has_files);
    let imported = (has_save && entries.is_empty()).then(|| match vanilla {
        true => Imported {
            title: "Your Steam save".to_owned(),
            at: None,
        },
        false => imported_from(&snaps),
    });
    let (versions, dir, usage, cap) = match vault {
        Some(v) => {
            let now = swofty_core::stdlib::careers::read(&swofty_core::stdlib::careers::data_path(&root));
            let versions = snaps
                .into_iter()
                .map(|snap| {
                    let (held, index) = indexed(&v, &snap);
                    let diff = index
                        .filter(|_| !now.is_empty())
                        .map(|i| vault::careers::diff(&now, &i));
                    Version { snap, held, diff }
                })
                .collect();
            (versions, v.dir.clone(), v.usage(), v.policy.cap_bytes)
        }
        None => (Vec::new(), root.clone(), 0, 0),
    };
    let others = store
        .profile_names()
        .unwrap_or_default()
        .into_iter()
        .filter(|n| n != VANILLA && *n != name)
        .collect();
    let steam = active_loc.filter(|l| l.layout == vault::Layout::Steam);
    let awaiting_import = awaiting_import(
        vanilla,
        has_save,
        steam.as_ref().is_some_and(launcher_saves::has_files),
    );
    let steam_missing = steam_save_missing(
        vanilla,
        has_save,
        steam.as_ref().is_some_and(launcher_saves::has_files),
        launcher_saves::import_pending(&root),
    ) && launcher_saves::steam_save_pending(&root, &name);
    let view = SavesView {
        steam,
        profile: name,
        loc,
        entries,
        imported,
        has_save,
        others,
        versions,
        dir,
        usage,
        cap,
        steam_running: running.steam,
        awaiting_import,
        steam_missing,
    };
    Built {
        banner,
        running: running.game,
        view,
    }
}

impl Shell {
    pub(crate) fn refresh_saves(&mut self) {
        let built = build_saves(&self.store, self.install.as_ref(), &self.library.profile.name);
        self.saves.apply(built);
    }

    pub(crate) fn refresh_saves_in_background(&mut self, cx: &mut Context<Self>) {
        self.saves.generation += 1;
        self.saves.loading = true;
        let generation = self.saves.generation;
        let store = self.store.clone();
        let install = self.install.clone();
        let name = self.library.profile.name.clone();
        cx.spawn(async move |this, cx| {
            let built = cx
                .background_executor()
                .spawn(async move { build_saves(&store, install.as_ref(), &name) })
                .await;
            this.update(cx, |s, cx| {
                if s.saves.generation == generation {
                    s.saves.apply(built);
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    pub(crate) fn refresh_banner_in_background(&mut self, cx: &mut Context<Self>) {
        self.saves.generation += 1;
        let generation = self.saves.generation;
        let root = self.store.root.clone();
        let name = self.library.profile.name.clone();
        let name_done = name.clone();
        cx.spawn(async move |this, cx| {
            let banner = cx
                .background_executor()
                .spawn(async move { SavesPage::banner_for(&root, &name) })
                .await;
            this.update(cx, |s, cx| {
                if s.saves.generation == generation {
                    s.saves.banner_profile = name_done;
                    s.saves.banner = banner;
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    pub(crate) fn open_saves(&mut self, confirm: Option<String>, cx: &mut Context<Self>) {
        self.saves.confirm = confirm;
        self.open_page(Page::Careers, cx);
    }

    pub(crate) fn show_saves_notice(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let shell = cx.entity().downgrade();
        window.open_alert_dialog(cx, move |alert, _, _| {
            let shell = shell.clone();
            alert
                .width(px(480.))
                .title(NOTICE_TITLE)
                .description(NOTICE_BODY)
                .ok_text("Show me")
                .cancel_text("OK")
                .show_cancel(true)
                .on_ok(move |_, _, cx| {
                    shell.update(cx, |s, cx| s.open_saves(None, cx)).ok();
                    true
                })
        });
    }

    fn ask_copy(
        &mut self,
        from: Location,
        from_name: String,
        to: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let shell = cx.entity().downgrade();
        let text = format!(
            "{} replaces the save of {to}. Its current save is kept as a version.",
            match from_name == VANILLA {
                true => "Your Steam save".to_owned(),
                false => from_name.clone(),
            }
        );
        let steam = from_name == VANILLA;
        let title = match steam {
            true => format!("Import from Steam into {to}?"),
            false => format!("Copy careers to {to}?"),
        };
        window.open_alert_dialog(cx, move |alert, _, _| {
            let (shell, from, to, from_name) = (shell.clone(), from.clone(), to.clone(), from_name.clone());
            alert
                .width(px(480.))
                .title(title.clone())
                .description(text.clone())
                .ok_text(if steam { "Import" } else { "Copy" })
                .show_cancel(true)
                .on_ok(move |_, window, cx| {
                    shell
                        .update(cx, |s, cx| s.copy_save(&from, &from_name, &to, window, cx))
                        .ok();
                    true
                })
        });
    }

    fn copy_save(
        &mut self,
        from: &Location,
        from_name: &str,
        to: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let root = self.store.root.clone();
        let done = launcher_saves::copy_to(&root, from, to, Running::now(), vault::now());
        let note = match done {
            Ok((files, bytes, _)) => {
                crate::app_log::note(&format!(
                    "saves: copied {files} file(s), {bytes} bytes, from {from_name} into the launcher saves of profile {to}"
                ));
                Notification::success(format!("Copied the save of {from_name} to {to}."))
            }
            Err(e) => {
                crate::app_log::action("copy a save").failed(&e);
                Notification::warning(format!("The save was not copied: {e}"))
            }
        };
        window.push_notification(note, cx);
        self.refresh_saves();
        cx.notify();
    }

    fn ask_export(&mut self, profile: String, window: &mut Window, cx: &mut Context<Self>) {
        let shell = cx.entity().downgrade();
        let text =
            format!("The save of {profile} replaces your Steam save. Your Steam save is kept as a version.");
        let title = format!("Export {profile} to Steam?");
        window.open_alert_dialog(cx, move |alert, _, _| {
            let (shell, profile) = (shell.clone(), profile.clone());
            alert
                .width(px(480.))
                .title(title.clone())
                .description(text.clone())
                .ok_text("Export")
                .show_cancel(true)
                .on_ok(move |_, window, cx| {
                    shell.update(cx, |s, cx| s.export_save(&profile, window, cx)).ok();
                    true
                })
        });
    }

    fn export_save(&mut self, profile: &str, window: &mut Window, cx: &mut Context<Self>) {
        let root = self.store.root.clone();
        let done = match Location::active(&root).filter(|l| l.layout == vault::Layout::Steam) {
            Some(steam) => launcher_saves::export(&root, profile, &steam, Running::now(), vault::now())
                .map_err(|e| e.to_string()),
            None => Err("no Steam save folder of F1 25 found".to_owned()),
        };
        let note = match done {
            Ok(r) => {
                crate::app_log::note(&r.log_line());
                Notification::success(format!("Exported the save of {profile} to Steam."))
            }
            Err(e) => {
                crate::app_log::action("export a save").failed(&e);
                Notification::warning(format!("The save was not exported: {e}"))
            }
        };
        window.push_notification(note, cx);
        self.refresh_saves();
        cx.notify();
    }

    pub(crate) fn saves_banner(&self, on_page: bool, p: Palette, cx: &mut Context<Self>) -> Option<Div> {
        let banner = self.saves.banner.as_ref()?;
        let (text, profile, source, snapshot, kept_at) = match banner {
            Banner::Dropped {
                profile,
                source,
                snapshot,
                kept_at,
                names,
            } => (
                format!(
                    "{} missing from {}: {}. {} last in the save on {}. Restoring that version brings {} back; the \
                     restore shows what else changes first.",
                    match names.len() {
                        1 => "1 career is".to_owned(),
                        n => format!("{n} careers are"),
                    },
                    whose(profile),
                    names.join("; "),
                    match names.len() {
                        1 => "It was",
                        _ => "They were",
                    },
                    vault::human(*kept_at),
                    match names.len() {
                        1 => "it",
                        _ => "them",
                    }
                ),
                profile.clone(),
                source.clone(),
                snapshot.clone(),
                *kept_at,
            ),
            Banner::Hint(n) => (
                n.headline(),
                profile_of_source(&n.source),
                n.source.clone(),
                n.snapshot.clone(),
                n.kept_at,
            ),
        };
        let hint = matches!(banner, Banner::Hint(_));
        let review = snapshot.clone();
        let _ = kept_at;
        let _ = profile;
        let dismiss_source = source;
        Some(
            h_flex()
                .px(px(GUTTER))
                .pb(px(12.))
                .gap(px(8.))
                .flex_shrink_0()
                .items_center()
                .text_size(px(13.))
                .text_color(p.text_muted)
                .child(
                    Icon::new(IconName::CircleAlert)
                        .size(px(14.))
                        .text_color(p.warning),
                )
                .child(div().flex_1().min_w_0().child(text))
                .child(
                    Button::new("saves-banner-review")
                        .outline()
                        .small()
                        .label(match on_page {
                            true => "Review the restore",
                            false => "Open Career saves",
                        })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.open_saves(Some(review.clone()), cx);
                        })),
                )
                .child(
                    Button::new("saves-banner-dismiss")
                        .ghost()
                        .small()
                        .label("Dismiss")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            let root = this.store.root.clone();
                            if hint {
                                Notice::clear(&root);
                            }
                            if let Ok(v) = Vault::open(&root, &dismiss_source) {
                                v.dismiss_drop().ok();
                            }
                            this.refresh_saves();
                            cx.notify();
                        })),
                ),
        )
    }

    pub(crate) fn restore_version(
        &mut self,
        source: &str,
        name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let root = self.store.root.clone();
        let when = Vault::open(&root, source)
            .and_then(|v| v.find(name))
            .ok()
            .map(|s| s.at);
        let restored = Vault::open(&root, source).and_then(|v| {
            let snap = v.find(name)?;
            let loc = Location::of_source(&root, source)
                .or_else(|| Location::active(&root).filter(|l| l.source == source))
                .unwrap_or(Location {
                    source: snap.source.clone(),
                    layout: snap.layout,
                    dir: snap.save_dir.clone(),
                });
            vault::restore::restore(&v, name, &loc, vault::restore::Running::now(), vault::now())
        });
        let note = match restored {
            Ok(r) => {
                crate::app_log::note(&r.log_line());
                self.saves.confirm = None;
                Notification::success(match (when, &r.before) {
                    (Some(at), Some(_)) => format!(
                        "Restored your save from {}. The save it replaced is kept as a version.",
                        ago(at as f64)
                    ),
                    (Some(at), None) => format!("Restored your save from {}.", ago(at as f64)),
                    (None, _) => "Restored your save.".to_owned(),
                })
            }
            Err(e) => {
                crate::app_log::action("restore a save").failed(&e);
                Notification::warning(format!("The save was not restored: {e}"))
            }
        };
        window.push_notification(note, cx);
        self.refresh_saves();
        cx.notify();
    }

    fn pin_version(&mut self, source: &str, id: &str, pinned: bool, cx: &mut Context<Self>) {
        let root = self.store.root.clone();
        let done = Vault::open(&root, source).and_then(|v| match pinned {
            true => v.unpin_why(id, "pinned", vault::now()),
            false => v.pin(id, "pinned", vault::now()),
        });
        if let Err(e) = done {
            crate::app_log::action("pin a version").failed(&e);
        }
        self.refresh_saves();
        cx.notify();
    }

    fn cap_prompt(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let initial = self
            .store
            .config()
            .vault_cap_mb
            .unwrap_or(vault::DEFAULT_CAP_MB)
            .to_string();
        self.prompt(
            Prompt {
                title: "Career saves storage",
                description: "The most space, in MB, the saved versions may take. The oldest unpinned versions go first."
                    .into(),
                initial,
                placeholder: "500",
                ok: "Save",
                browse: None,
                action: std::rc::Rc::new(|store, value| {
                    let mb: u64 = value.parse().map_err(|_| "Enter a number of MB.".to_owned())?;
                    if mb == 0 {
                        return Err("The limit must be at least 1 MB.".to_owned());
                    }
                    let mut c = store.config();
                    c.vault_cap_mb = Some(mb);
                    store.save_config(&c).map_err(|e| e.to_string())?;
                    Ok(format!("The career saves keep at most {mb} MB"))
                }),
            },
            window,
            cx,
        );
    }

    fn restore_panel(
        &self,
        v: &Version,
        layout: vault::Layout,
        blocker: Option<String>,
        (id, source): (String, String),
        p: Palette,
        cx: &mut Context<Self>,
    ) -> Div {
        let when = vault::human(v.snap.at);
        let line = |head: &'static str, tone: Hsla, text: String| {
            h_flex()
                .gap(px(8.))
                .items_start()
                .text_size(px(13.))
                .child(
                    div()
                        .w(px(100.))
                        .flex_shrink_0()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(tone)
                        .child(head),
                )
                .child(div().flex_1().min_w_0().text_color(p.text_muted).child(text))
        };
        let mut lines: Vec<Div> = Vec::new();
        match &v.diff {
            Some(d) => {
                lines.push(match d.get_back.is_empty() {
                    true => line("You get back", p.text_placeholder, "No careers.".into()),
                    false => line("You get back", p.success, list(&d.get_back)),
                });
                let mut lose = Vec::new();
                if !d.lose.is_empty() {
                    lose.push(format!("{} (not in that version)", list(&d.lose)));
                }
                for (now, then) in &d.progress {
                    lose.push(format!(
                        "progress in {}: {} goes back to {}",
                        now.name,
                        now.place.as_deref().unwrap_or("now"),
                        then.place.as_deref().unwrap_or("then")
                    ));
                }
                lines.push(match lose.is_empty() {
                    true => line("You lose", p.text_placeholder, "No careers.".into()),
                    false => line("You lose", p.error, lose.join("; ")),
                });
            }
            None => lines.push(line(
                "Careers",
                p.text_placeholder,
                "No career list to compare.".into(),
            )),
        }
        lines.push(line(
            "Also",
            p.text_placeholder,
            format!("Settings and setups go back to {when}. The save as it is now is kept as a version."),
        ));
        let close = match layout {
            vault::Layout::Steam => Some("Steam uploads the restored save when it starts."),
            _ => None,
        };
        v_flex()
            .gap(px(8.))
            .p(px(12.))
            .rounded(px(5.))
            .bg(p.surface)
            .border_1()
            .border_color(p.border_variant)
            .children(lines)
            .children(close.map(|c| div().text_size(px(12.)).text_color(p.text_placeholder).child(c)))
            .child(
                h_flex()
                    .gap(px(8.))
                    .child(
                        Button::new("restore-confirm")
                            .primary()
                            .small()
                            .label("Restore")
                            .disabled(blocker.is_some())
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.restore_version(&source, &id, window, cx)
                            })),
                    )
                    .child(
                        Button::new("restore-cancel")
                            .ghost()
                            .small()
                            .label("Cancel")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.saves.confirm = None;
                                cx.notify();
                            })),
                    ),
            )
    }

    pub(crate) fn careers_page(&mut self, p: Palette, _wide: bool, cx: &mut Context<Self>) -> AnyElement {
        let banner = self.saves_banner(true, p, cx);
        let current = self.library.profile.name.as_str();
        let loading = self.saves.loading && self.saves.view.as_ref().is_none_or(|v| v.profile != current);
        let Some(view) = self.saves.view.as_ref().filter(|_| !loading) else {
            let empty = match loading {
                true => "Loading saves…",
                false => "No F1 25 save found yet.",
            };
            return v_flex()
                .size_full()
                .child(self.header("Saves", String::new(), div(), p))
                .children(banner)
                .child(
                    div()
                        .px(px(GUTTER))
                        .text_size(px(13.))
                        .text_color(p.text_muted)
                        .child(empty),
                )
                .into_any_element();
        };
        let title = match view.vanilla() {
            true => "Vanilla saves".to_owned(),
            false => format!("{} saves", view.profile),
        };
        let subtitle = match view.vanilla() {
            true => "Plays on your Steam save".to_owned(),
            false => "This profile's own save".to_owned(),
        };
        let actions = self.saves_actions(view, cx);
        let careers = self.careers_block(view, p, cx);
        let versions = self.versions_block(view, p, cx);
        let footer = self.storage_line(view, p, cx);
        let empty = self.empty_save_notice(view, p, cx);
        let missing = self.steam_missing_notice(view, p, cx);
        v_flex()
            .size_full()
            .child(self.header(title, subtitle, actions, p))
            .children(banner)
            .children(empty)
            .children(missing)
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .scroll_y("career-saves")
                    .px(px(GUTTER))
                    .pt(px(4.))
                    .pb(px(16.))
                    .child(v_flex().gap(px(22.)).child(careers).child(versions)),
            )
            .child(footer)
            .into_any_element()
    }

    fn empty_save_notice(&self, view: &SavesView, p: Palette, cx: &mut Context<Self>) -> Option<Div> {
        if !view.awaiting_import {
            return None;
        }
        let steam = view.steam.clone()?;
        let name = view.profile.clone();
        let blocked = self.saves.game_running;
        Some(
            h_flex()
                .mx(px(GUTTER))
                .mb(px(8.))
                .px(px(12.))
                .py(px(8.))
                .gap(px(12.))
                .items_center()
                .rounded(px(8.))
                .border_1()
                .border_color(p.border)
                .text_size(px(13.))
                .text_color(p.text_muted)
                .child(Icon::new(IconName::Info).size(px(15.)))
                .child(div().flex_1().min_w_0().child(EMPTY_SAVE))
                .child(
                    Button::new("saves-empty-import")
                        .small()
                        .icon(IconName::Download)
                        .label("Import from Steam")
                        .disabled(blocked)
                        .when(blocked, |b| b.tooltip("Close F1 25 first"))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.ask_copy(steam.clone(), VANILLA.to_owned(), name.clone(), window, cx);
                        })),
                ),
        )
    }

    fn steam_missing_notice(&self, view: &SavesView, p: Palette, cx: &mut Context<Self>) -> Option<Div> {
        if !view.steam_missing {
            return None;
        }
        let blocked = self.saves.game_running;
        let name = view.profile.clone();
        let on = self.library.profile.launcher_saves;
        Some(
            h_flex()
                .mx(px(GUTTER))
                .mb(px(8.))
                .px(px(12.))
                .py(px(8.))
                .gap(px(8.))
                .items_center()
                .flex_wrap()
                .rounded(px(8.))
                .border_1()
                .border_color(p.border)
                .text_size(px(13.))
                .text_color(p.text_muted)
                .child(Icon::new(IconName::Info).size(px(15.)))
                .child(div().flex_1().min_w(px(240.)).child(STEAM_SAVE_MISSING))
                .when(on, |row| {
                    row.child(
                        Button::new("saves-missing-steam")
                            .small()
                            .label("Play with your Steam saves")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.set_launcher_saves(false, window, cx);
                                this.refresh_saves();
                            })),
                    )
                })
                .child(
                    Button::new("saves-missing-retry")
                        .small()
                        .icon(IconName::Download)
                        .label("Import from Steam")
                        .disabled(blocked)
                        .when(blocked, |b| b.tooltip("Close F1 25 first"))
                        .on_click(cx.listener(|this, _, window, cx| this.retry_steam_import(window, cx))),
                )
                .child(
                    Button::new("saves-missing-pick")
                        .small()
                        .outline()
                        .icon(IconName::Folder)
                        .label("Choose the folder…")
                        .disabled(blocked)
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.pick_steam_save(name.clone(), window, cx);
                        })),
                ),
        )
    }

    fn retry_steam_import(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        crate::game::import_now(&self.store);
        self.refresh_saves();
        let found = !self.saves.view.as_ref().is_some_and(|v| v.steam_missing);
        window.push_notification(
            match found {
                true => Notification::success("Imported your Steam save."),
                false => Notification::warning(
                    "Still couldn't find your Steam save. The log lists where it looked.",
                ),
            },
            cx,
        );
        cx.notify();
    }

    fn pick_steam_save(&mut self, profile: String, window: &mut Window, cx: &mut Context<Self>) {
        let rx = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Choose".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = rx.await else { return };
            let Some(folder) = paths.into_iter().next() else {
                return;
            };
            this.update_in(cx, |s, window, cx| match steam_save_in(&folder) {
                Some(from) => s.ask_copy(from, VANILLA.to_owned(), profile, window, cx),
                None => {
                    crate::app_log::note(&format!(
                        "saves: {} is not a Steam save folder (userdata/<id>/{}/remote)",
                        folder.display(),
                        swofty_core::STEAM_APP_ID
                    ));
                    window.push_notification(
                        Notification::warning(
                            "That isn't a Steam save folder. Pick the one named remote under userdata.",
                        ),
                        cx,
                    );
                }
            })
            .ok();
        })
        .detach();
    }

    fn saves_actions(&self, view: &SavesView, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let game = self.saves.game_running;
        let steam = view.steam.clone();
        let name = view.profile.clone();
        let vanilla = view.vanilla();
        let why_import = match () {
            _ if vanilla => Some("Vanilla uses the Steam save"),
            _ if steam.is_none() => Some("No Steam save found"),
            _ if game => Some("Close F1 25 first"),
            _ => None,
        };
        let why_export = match () {
            _ if vanilla => Some("Vanilla uses the Steam save"),
            _ if steam.is_none() => Some("No Steam save found"),
            _ if !view.has_save => Some("Nothing to export yet"),
            _ if game => Some("Close F1 25 first"),
            _ if view.steam_running => Some("Close Steam first"),
            _ => None,
        };
        let why_copy = match () {
            _ if view.others.is_empty() => Some("No other profiles"),
            _ if !view.has_save => Some("Nothing to copy yet"),
            _ if game => Some("Close F1 25 first"),
            _ => None,
        };
        let (from, from_name) = (view.loc.clone(), name.clone());
        let others = view.others.clone();
        let shell = cx.entity().downgrade();
        let copy = Button::new("saves-copy")
            .outline()
            .small()
            .icon(IconName::Copy)
            .label("Copy careers to…")
            .dropdown_caret(true)
            .disabled(why_copy.is_some())
            .when_some(why_copy, |b, t| b.tooltip(t))
            .dropdown_menu_with_anchor(Anchor::TopRight, move |mut menu, _, _| {
                for target in &others {
                    let (shell, from, from_name, to) =
                        (shell.clone(), from.clone(), from_name.clone(), target.clone());
                    menu = menu.item(PopupMenuItem::new(target.clone()).on_click(move |_, window, cx| {
                        let Some(from) = from.clone() else { return };
                        let (from_name, to) = (from_name.clone(), to.clone());
                        shell
                            .update(cx, |s, cx| s.ask_copy(from, from_name, to, window, cx))
                            .ok();
                    }));
                }
                menu
            });
        let import_name = name.clone();
        let import = Button::new("saves-import")
            .outline()
            .small()
            .icon(IconName::Download)
            .label("Import from Steam")
            .disabled(why_import.is_some())
            .when_some(why_import, |b, t| b.tooltip(t))
            .on_click(cx.listener(move |this, _, window, cx| {
                if let Some(steam) = steam.clone() {
                    this.ask_copy(steam, VANILLA.to_owned(), import_name.clone(), window, cx);
                }
            }));
        let export = Button::new("saves-export")
            .outline()
            .small()
            .icon(IconName::Upload)
            .label("Export to Steam")
            .disabled(why_export.is_some())
            .when_some(why_export, |b, t| b.tooltip(t))
            .on_click(cx.listener(move |this, _, window, cx| this.ask_export(name.clone(), window, cx)));
        h_flex()
            .gap(px(8.))
            .flex_shrink_0()
            .child(copy)
            .child(import)
            .child(export)
    }

    fn careers_block(&self, view: &SavesView, p: Palette, _cx: &mut Context<Self>) -> Div {
        let rows: Vec<AnyElement> = match (&view.imported, view.entries.is_empty()) {
            (Some(i), true) => vec![self.imported_row(i, p)],
            (None, true) => vec![
                div()
                    .px(px(12.))
                    .py(px(14.))
                    .text_size(px(13.))
                    .text_color(p.text_muted)
                    .child("No careers yet. They appear after you play this profile.")
                    .into_any_element(),
            ],
            _ => view.entries.iter().map(|e| self.career_row(e, p)).collect(),
        };
        v_flex()
            .gap(px(8.))
            .child(label("CAREERS", p))
            .child(self.rows_box(rows, p))
    }

    fn rows_box(&self, rows: Vec<AnyElement>, p: Palette) -> Div {
        v_flex()
            .rounded(px(8.))
            .border_1()
            .border_color(p.border)
            .overflow_hidden()
            .children(rows.into_iter().enumerate().map(|(i, r)| {
                div()
                    .w_full()
                    .when(i > 0, |d| d.border_t_1().border_color(p.border_variant))
                    .child(r)
            }))
    }

    fn imported_row(&self, i: &Imported, p: Palette) -> AnyElement {
        h_flex()
            .min_h(px(52.))
            .px(px(12.))
            .py(px(8.))
            .gap(px(10.))
            .items_center()
            .child(
                div().w(px(MODE_W)).flex_shrink_0().child(
                    h_flex()
                        .gap(px(6.))
                        .items_center()
                        .text_size(px(13.))
                        .text_color(p.text_muted)
                        .child(Icon::new(IconName::Save).size(px(15.)))
                        .child("Saved game"),
                ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap(px(2.))
                    .child(
                        div()
                            .truncate()
                            .text_size(px(14.))
                            .font_weight(FontWeight::MEDIUM)
                            .child(i.title.clone()),
                    )
                    .child(
                        div()
                            .truncate()
                            .text_size(px(12.))
                            .text_color(p.text_placeholder)
                            .child(LISTED_LATER),
                    ),
            )
            .child(
                div()
                    .w(px(PLAYED_W + CHIP_W))
                    .flex_shrink_0()
                    .truncate()
                    .text_right()
                    .text_size(px(12.))
                    .text_color(p.text_placeholder)
                    .children(i.at.map(|at| format!("imported {}", ago(at as f64)))),
            )
            .into_any_element()
    }

    fn career_row(&self, e: &Entry, p: Palette) -> AnyElement {
        let chip = match e.l.state {
            CareerState::Present => None,
            CareerState::Missing => Some(("Missing", p.warning)),
            CareerState::DeletedInGame => Some(("Deleted in game", p.text_placeholder)),
        };
        let dim = chip.is_some();
        h_flex()
            .min_h(px(52.))
            .px(px(12.))
            .py(px(8.))
            .gap(px(10.))
            .items_center()
            .child(
                div()
                    .w(px(MODE_W))
                    .flex_shrink_0()
                    .truncate()
                    .text_size(px(13.))
                    .text_color(p.text_muted)
                    .child(e.mode()),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap(px(2.))
                    .child(
                        div()
                            .truncate()
                            .text_size(px(14.))
                            .font_weight(FontWeight::MEDIUM)
                            .when(dim, |d| d.text_color(p.text_muted))
                            .child(e.l.title()),
                    )
                    .children(e.place.clone().map(|pl| {
                        div()
                            .truncate()
                            .text_size(px(12.))
                            .text_color(p.text_placeholder)
                            .child(pl)
                    })),
            )
            .child(
                div()
                    .w(px(PLAYED_W))
                    .flex_shrink_0()
                    .truncate()
                    .text_right()
                    .text_size(px(12.))
                    .text_color(p.text_placeholder)
                    .children(e.l.last_played.map(|at| ago(at as f64))),
            )
            .child(
                h_flex()
                    .w(px(CHIP_W))
                    .flex_shrink_0()
                    .justify_end()
                    .children(chip.map(|(t, c)| {
                        div()
                            .px(px(6.))
                            .h(px(20.))
                            .flex()
                            .items_center()
                            .rounded(px(4.))
                            .border_1()
                            .border_color(c)
                            .text_size(px(11.))
                            .text_color(c)
                            .whitespace_nowrap()
                            .child(t)
                    })),
            )
            .into_any_element()
    }

    fn versions_block(&self, view: &SavesView, p: Palette, cx: &mut Context<Self>) -> Div {
        let rows: Vec<AnyElement> = match view.versions.is_empty() {
            true => vec![
                div()
                    .px(px(12.))
                    .py(px(14.))
                    .text_size(px(13.))
                    .text_color(p.text_muted)
                    .child(match view.steam_missing {
                        true => STEAM_SAVE_MISSING,
                        false => "No versions yet.",
                    })
                    .into_any_element(),
            ],
            false => view
                .versions
                .iter()
                .map(|v| self.version_row(v, view, p, cx))
                .collect(),
        };
        v_flex()
            .gap(px(8.))
            .child(label("VERSIONS", p))
            .child(self.rows_box(rows, p))
    }

    fn version_row(&self, v: &Version, view: &SavesView, p: Palette, cx: &mut Context<Self>) -> AnyElement {
        let s = &v.snap;
        let id = s.id.clone();
        let pinned = !s.pinned.is_empty();
        let own_pin = s.pinned.iter().any(|x| x == "pinned");
        let confirming = self.saves.confirm.as_deref() == Some(s.id.as_str());
        let layout = view.layout();
        let running = Running {
            game: self.saves.game_running,
            steam: view.steam_running,
            ea_app: false,
        };
        let blocker = vault::restore::blocker(layout, running);
        let why = short_block(layout, running);
        let source = s.source.clone();
        let (restore_id, restore_source) = (id.clone(), source.clone());
        let (pin_id, pin_source) = (id.clone(), source);
        let ask_id = id.clone();
        let full = vault::human(s.at);
        let held = v.held.as_ref().map(|h| careers_held(h));
        v_flex()
            .w_full()
            .child(
                h_flex()
                    .h(px(44.))
                    .px(px(12.))
                    .gap(px(10.))
                    .items_center()
                    .child(div().w(px(PIN_W)).flex_shrink_0().when(pinned, |d| {
                        d.child(Icon::new(IconName::Star).size(px(13.)).text_color(p.accent))
                    }))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .truncate()
                                    .text_size(px(14.))
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(version_label(s)),
                            )
                            .children(held.map(|h| {
                                div()
                                    .truncate()
                                    .text_size(px(12.))
                                    .text_color(p.text_placeholder)
                                    .child(h)
                            })),
                    )
                    .child(
                        div()
                            .id(SharedString::from(format!("when-{id}")))
                            .w(px(WHEN_W))
                            .flex_shrink_0()
                            .truncate()
                            .text_size(px(13.))
                            .text_color(p.text_muted)
                            .child(ago(s.at as f64))
                            .tooltip(move |window, cx| Tooltip::new(full.clone()).build(window, cx)),
                    )
                    .child(
                        div()
                            .w(px(SIZE_W))
                            .flex_shrink_0()
                            .text_right()
                            .text_size(px(13.))
                            .text_color(p.text_muted)
                            .child(mb(s.total())),
                    )
                    .child(
                        h_flex()
                            .w(px(ACTIONS_W))
                            .flex_shrink_0()
                            .gap(px(4.))
                            .justify_end()
                            .child(
                                Button::new(SharedString::from(format!("pin-{id}")))
                                    .ghost()
                                    .small()
                                    .icon(IconName::Star)
                                    .label(if own_pin { "Unpin" } else { "Pin" })
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.pin_version(&pin_source, &pin_id, own_pin, cx)
                                    })),
                            )
                            .child(
                                Button::new(SharedString::from(format!("restore-{id}")))
                                    .outline()
                                    .small()
                                    .icon(IconName::Undo2)
                                    .label("Restore")
                                    .disabled(why.is_some())
                                    .when_some(why, |b, t| b.tooltip(t))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.saves.confirm =
                                            match this.saves.confirm.as_deref() == Some(ask_id.as_str()) {
                                                true => None,
                                                false => Some(ask_id.clone()),
                                            };
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            .when(confirming, |d| {
                d.child(div().px(px(12.)).pb(px(12.)).child(self.restore_panel(
                    v,
                    layout,
                    blocker,
                    (restore_id, restore_source),
                    p,
                    cx,
                )))
            })
            .into_any_element()
    }

    fn storage_line(&self, view: &SavesView, p: Palette, cx: &mut Context<Self>) -> Div {
        let dir = view.dir.clone();
        h_flex()
            .px(px(GUTTER))
            .py(px(8.))
            .gap(px(4.))
            .flex_shrink_0()
            .items_center()
            .border_t_1()
            .border_color(p.border_variant)
            .text_size(px(13.))
            .text_color(p.text_muted)
            .child(format!(
                "Versions use {} of {}",
                mb(view.usage),
                mb_whole(view.cap)
            ))
            .child("·")
            .child(
                Button::new("saves-storage-change")
                    .ghost()
                    .small()
                    .label("Change")
                    .on_click(cx.listener(|this, _, window, cx| this.cap_prompt(window, cx))),
            )
            .child("·")
            .child(
                Button::new("saves-storage-open")
                    .ghost()
                    .small()
                    .label("Open folder")
                    .on_click(cx.listener(move |_, _, _, cx| crate::reveal::show(&dir, cx))),
            )
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use swofty_core::vault::careers::{Brief, Row, Status};
    use swofty_core::vault::{self, Vault};

    use swofty_core::launcher_saves::{self, Marker};

    use super::{
        Banner, SavesPage, awaiting_import, banner_from, build_saves, list, reason_label, steam_save_in,
        steam_save_missing, whose,
    };

    #[test]
    fn a_saves_view_builds_without_the_shell_so_it_can_run_off_the_ui_thread() {
        fn is_send<T: Send>() {}
        is_send::<super::Built>();
        let root = std::env::temp_dir().join(format!("swofty-build-saves-{}", std::process::id()));
        std::fs::remove_dir_all(&root).ok();
        std::fs::create_dir_all(&root).unwrap();
        let store = swofty_core::store::Store::new(&root);
        let built = build_saves(&store, None, "Race");
        assert_eq!(built.view.profile, "Race");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn an_empty_launcher_save_with_a_steam_save_waits_for_an_import() {
        assert!(awaiting_import(false, false, true));
        assert!(!awaiting_import(true, false, true));
        assert!(!awaiting_import(false, true, true));
        assert!(!awaiting_import(false, false, false));
    }

    #[test]
    fn a_missed_steam_save_is_told_apart_from_an_empty_one() {
        assert!(steam_save_missing(false, false, false, true));
        assert!(!steam_save_missing(false, false, false, false));
        assert!(!steam_save_missing(false, false, true, true));
        assert!(!steam_save_missing(false, true, false, true));
        assert!(!steam_save_missing(true, false, false, true));
    }

    #[test]
    fn a_picked_folder_may_be_remote_the_app_folder_or_the_steam_root() {
        let base = std::env::temp_dir().join(format!("swofty-pick-{}", std::process::id()));
        let remote = base.join("Steam Ünï/userdata/843502855/3059520/remote");
        std::fs::create_dir_all(&remote).unwrap();
        assert!(steam_save_in(&remote).is_none());
        std::fs::write(remote.join("savegame@profile#X"), b"x").unwrap();
        for pick in [
            remote.clone(),
            remote.parent().unwrap().to_path_buf(),
            base.join("Steam Ünï/userdata"),
            base.join("Steam Ünï"),
        ] {
            assert_eq!(
                steam_save_in(&pick).map(|l| l.dir),
                Some(remote.clone()),
                "{pick:?}"
            );
        }
        assert!(steam_save_in(&base).is_none());
        std::fs::remove_dir_all(&base).ok();
    }

    fn row(id: &str, status: Status, at: i64) -> Row {
        Row {
            career: id.into(),
            mode: None,
            label: format!("{id} label"),
            driver: None,
            team: None,
            season: Some(2016),
            round: Some(7),
            profile: None,
            addons: Vec::new(),
            first_seen: "a".into(),
            first_seen_at: 1,
            last_seen: format!("snap-{at}"),
            last_seen_at: at,
            last_played: None,
            status,
            found_at: None,
            dismissed: false,
        }
    }

    fn data(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("swofty-launcher-saves-{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&d).ok();
        std::fs::create_dir_all(d.join("career-vault/steam-1")).unwrap();
        d
    }

    #[test]
    fn the_banner_names_the_missing_careers_and_the_version_to_restore() {
        let d = data("drop");
        let table = vault::careers::Careers {
            version: 1,
            careers: vec![
                row("a", Status::MissingSince("later".into()), 10),
                row("b", Status::MissingSince("later".into()), 20),
                row("c", Status::Present, 30),
            ],
        };
        table.write(&d.join("career-vault/steam-1")).unwrap();
        match banner_from(&d, "steam-1".into(), "vanilla".into()) {
            Some(Banner::Dropped {
                snapshot,
                names,
                kept_at,
                profile,
                ..
            }) => {
                assert_eq!(profile, "vanilla");
                assert_eq!(snapshot, "snap-20");
                assert_eq!(kept_at, 20);
                assert_eq!(
                    names,
                    ["a label, season 2016, round 7", "b label, season 2016, round 7"]
                );
            }
            _ => panic!("expected a drop banner"),
        }
        Vault::open(&d, "steam-1").unwrap().dismiss_drop().unwrap();
        assert!(banner_from(&d, "steam-1".into(), "vanilla".into()).is_none());
    }

    #[test]
    fn no_careers_and_no_notice_means_no_banner() {
        let d = data("none");
        assert!(banner_from(&d, "steam-1".into(), "vanilla".into()).is_none());
    }

    #[test]
    fn mode_names_read_like_the_game() {
        use swofty_core::vault::careers::mode_label;
        assert_eq!(mode_label(Some("driver_career")), "Driver Career");
        assert_eq!(mode_label(Some("my_team")), "My Team");
        assert_eq!(mode_label(Some("progressive")), "Progressive");
        assert_eq!(mode_label(Some("sprint_cup")), "Sprint Cup");
        assert_eq!(mode_label(None), "Career");
    }

    #[test]
    fn versions_and_places_read_short() {
        use super::{careers_held, season_round};
        use swofty_core::vault::careers::listing;
        assert_eq!(careers_held(&vec![String::new(); 10]), "10 careers");
        let l = listing(
            &swofty_core::stdlib::careers::parse(
                r#"{"careers":[{"career":"f1_career_2026","slot":"0","slot_season":1,"slot_weekend":2}]}"#,
            ),
            None,
        );
        assert_eq!(season_round(&l[0]).as_deref(), Some("Season 1 · Round 3"));
        assert_eq!(l[0].title(), "Driver Career · 2026 season");
    }

    #[test]
    fn lists_name_the_place() {
        let b = |n: &str, p: Option<&str>| Brief {
            career: n.into(),
            name: n.into(),
            place: p.map(str::to_owned),
        };
        assert_eq!(
            list(&[b("A", Some("season 2016, round 7")), b("B", None)]),
            "A (season 2016, round 7), B"
        );
    }

    fn marker(profiles: &[&str], seen: bool) -> Marker {
        Marker {
            at: 1,
            launcher: "1.7.0".into(),
            profiles: profiles.iter().map(|p| p.to_string()).collect(),
            note: String::new(),
            seen,
        }
    }

    #[test]
    fn the_first_start_notice_shows_once_and_is_remembered() {
        let d = data("notice");
        let mut page = SavesPage::default();
        assert!(!page.take_notice(&d));
        swofty_core::write_json(&launcher_saves::marker_path(&d), &marker(&["default"], false)).unwrap();
        assert!(page.take_notice(&d));
        assert!(!page.take_notice(&d));
        assert!(launcher_saves::marker(&d).unwrap().seen);
        assert!(!SavesPage::default().take_notice(&d));
    }

    #[test]
    fn no_notice_when_the_import_copied_nothing_or_was_seen() {
        let d = data("notice-none");
        swofty_core::write_json(&launcher_saves::marker_path(&d), &marker(&[], false)).unwrap();
        assert!(!SavesPage::default().take_notice(&d));
        swofty_core::write_json(&launcher_saves::marker_path(&d), &marker(&["default"], true)).unwrap();
        assert!(!SavesPage::default().take_notice(&d));
    }

    #[test]
    fn versions_read_in_plain_words() {
        use swofty_core::vault::Reason;
        assert_eq!(reason_label(Reason::BeforeSession), "Before Play");
        assert_eq!(reason_label(Reason::Migrated), "Old backup");
        assert_eq!(reason_label(Reason::Imported), "Imported from Steam");
        assert_eq!(whose("vanilla"), "your Steam save");
        assert_eq!(whose("default"), "profile default's save");
    }

    #[test]
    fn an_imported_save_with_no_listing_gets_a_titled_placeholder() {
        use swofty_core::vault::{Meta, Reason, Taken};
        let d = data("imported");
        let loc = swofty_core::vault::Location::launcher(&d, "default");
        std::fs::create_dir_all(&loc.dir).unwrap();
        std::fs::write(loc.dir.join("savegame@profile#NXDSMWW.BWW"), [1u8; 64]).unwrap();
        std::fs::write(loc.dir.join("savegame@profile#QKRHMYXE"), [2u8; 16]).unwrap();
        let v = Vault::of(&d, &loc).unwrap();
        let note = format!("copied from {}", d.join("saves/progressive/remote").display());
        let Taken::New(_) = v
            .snapshot(&loc, Meta::new(Reason::ImportedProfile).note(note), 700)
            .unwrap()
        else {
            panic!()
        };
        let i = super::imported_from(&v.list().unwrap());
        assert_eq!(i.title, "Copied from progressive");
        assert_eq!(i.at, Some(700));
        assert_eq!(super::imported_from(&[]).title, "Imported Steam save");
    }

    #[test]
    fn the_fixed_columns_leave_room_at_the_narrowest_window() {
        let content = 960. - 220. - 2. * 28.;
        let career_fixed = super::MODE_W + super::PLAYED_W + super::CHIP_W + 3. * 10. + 24.;
        let version_fixed = super::PIN_W + super::WHEN_W + super::SIZE_W + super::ACTIONS_W + 4. * 10. + 24.;
        assert!(content - career_fixed >= 150., "{}", content - career_fixed);
        assert!(content - version_fixed >= 150., "{}", content - version_fixed);
    }

    #[test]
    fn restore_waits_for_what_is_running() {
        use super::short_block;
        use swofty_core::vault::Layout;
        let run = |game, steam| swofty_core::vault::restore::Running {
            game,
            steam,
            ea_app: false,
        };
        assert_eq!(
            short_block(Layout::Launcher, run(true, false)),
            Some("Close F1 25 first")
        );
        assert_eq!(short_block(Layout::Launcher, run(false, true)), None);
        assert_eq!(
            short_block(Layout::Steam, run(false, true)),
            Some("Close Steam first")
        );
        assert_eq!(short_block(Layout::Steam, run(false, false)), None);
    }
}
