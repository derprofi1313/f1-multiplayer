//! The game-sync panel: the career in the game itself, on this PC. A generated addon
//! (`swofty_core::careersync`, made by `league::gamesync`) carries the players' names and numbers
//! and the teams' league pace for the next round. The panel shows what the game will show,
//! prepares the addon for the next round (also by itself when the round moves on, once it was
//! prepared for this career), restores the normal game, and lets the owner switch the parts on
//! and off.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Disableable, Sizable, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use swofty_core::careersync::{self, Prepared, SyncState};
use swofty_core::league::gamesync::{self, Overlay, performance};
use swofty_core::league::{Action, League, LineupMode, Options, Phase, aimarket, solo};
use swofty_core::store::Store;

use crate::i18n::{tr, tr_owned};
use crate::pages::label;
use crate::shell::Shell;
use crate::theme::Palette;

use super::capitalized;

/// How often the panel looks again at what this PC has, while nothing changed.
const LOOK_EVERY: Duration = Duration::from_secs(30);

/// The game sync on this PC, as the Career page keeps it.
#[derive(Default)]
pub(crate) struct GameSync {
    /// What the last look found.
    status: Option<Status>,
    /// What the last look was for; a new key looks again.
    key: Option<Key>,
    at: Option<Instant>,
    /// "look", "prepare" or "restore" while that runs in the background.
    busy: Option<&'static str>,
    /// A prepare asked for while the game ran, for this career: it runs when the game closes.
    waiting: Option<String>,
    /// The overlay (its hash) a prepare was tried for on its own, so a failure isn't retried on
    /// every tick.
    tried: Option<String>,
    error: Option<String>,
    /// The details (identities, car pace, options) are open.
    open: bool,
}

/// What a look is for: the selected career at a version, the launcher's profile, and whether the
/// game runs.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Key {
    career: String,
    version: u64,
    profile: String,
    running: bool,
}

/// What a look found.
#[derive(Debug, Clone, Default)]
struct Status {
    state: SyncState,
    /// The overlay the next round wants, as its `league.json` says; `None` without one.
    wanted: Option<Prepared>,
    overlay: Option<Overlay>,
    pace: Option<Pace>,
    /// Why the overlay can't be made on this PC (Classic Seasons missing, say).
    problem: Option<String>,
}

/// The teams' car masses for the next round.
#[derive(Debug, Clone, PartialEq)]
struct Pace {
    round: u8,
    masses: BTreeMap<u8, f32>,
    percent_per_kg: f32,
    /// The masses are the last round's: the game reuses its build of the team archives.
    same_as_last: bool,
}

/// What the tick does by itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Auto {
    Prepare,
    Restore,
}

/// How a status line reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tone {
    Good,
    Warn,
    Muted,
}

/// The career wants an overlay: a running season with a round to come and a game option on.
fn wants(league: &League) -> bool {
    league.phase == Phase::Racing && any_option(&league.options) && league.season().next_round().is_some()
}

fn any_option(o: &Options) -> bool {
    o.game_identity || o.game_performance || o.lineups == LineupMode::Dynamic
}

/// The options a new career starts with: the players' names and the league's car pace in the
/// game (the per-part feel stays off).
pub(super) fn new_career_options() -> Action {
    Action::SetOptions {
        options: Options {
            game_identity: true,
            game_performance: true,
            ..Options::default()
        },
    }
}

/// Look at what this PC has of the overlay, and what `league`'s next round wants. Runs in the
/// background: it reads the profile, the installed addon and the Classic Seasons files.
fn look(store: &Store, profile: &str, league: &League, career: &str, name: &str) -> Status {
    let mut status = Status {
        state: careersync::state(store, profile),
        ..Status::default()
    };
    if !wants(league) {
        return status;
    }
    let ctx = match careersync::context(store, league.season().year) {
        Ok(ctx) => ctx,
        Err(e) => {
            status.problem = Some(e);
            return status;
        }
    };
    let ctx = ctx.get();
    let Some(overlay) = gamesync::overlay(league, career, name, &ctx) else {
        return status;
    };
    status.wanted = wanted_of(&gamesync::files(&overlay, &ctx));
    if league.options.game_performance {
        let round = overlay.round;
        let last = league
            .season()
            .rounds
            .iter()
            .map(|r| r.round)
            .filter(|r| *r < round)
            .max();
        if let Some(masses) = performance::masses_at(league, &ctx, round) {
            let same_as_last =
                last.is_some_and(|l| performance::masses_at(league, &ctx, l).as_ref() == Some(&masses));
            status.pace = Some(Pace {
                round,
                masses,
                percent_per_kg: performance::percent_per_kg(ctx.ai_slots),
                same_as_last,
            });
        }
    }
    status.overlay = Some(overlay);
    status
}

/// What the overlay's files are made for, from their `league.json`.
fn wanted_of(files: &[gamesync::File]) -> Option<Prepared> {
    files
        .iter()
        .find(|(path, _)| path == gamesync::LEAGUE_FILE)
        .and_then(|(_, bytes)| Prepared::parse(bytes))
}

/// Write the overlay for `league`'s next round and turn it on in `profile`. Returns the state
/// and what was wanted, so a prepare the running game held back shows.
fn prepare(
    store: &Store,
    profile: &str,
    league: &League,
    career: &str,
    name: &str,
) -> Result<(SyncState, Option<Prepared>), String> {
    let ctx = careersync::context(store, league.season().year)?;
    let overlay = gamesync::overlay(league, career, name, &ctx.get()).ok_or_else(|| {
        tr("There is nothing to prepare: the game sync needs a running season and one of its options on.")
            .to_owned()
    })?;
    let wanted = wanted_of(&gamesync::files(&overlay, &ctx.get()));
    let state = careersync::prepare(store, profile, &overlay)?;
    Ok((state, wanted))
}

/// What the tick does by itself for `career`, from a fresh look:
/// - nothing while the game runs;
/// - prepare when a prepare waits for the game to close, or when this PC has the overlay of
///   this career on and it is out of date (the round moved on, the league changed), unless that
///   very overlay was tried already;
/// - restore when this PC has the overlay of this career and the career wants none any more
///   (`ended`: finished, or every game option off), or of a solo career that is gone
///   (`orphan`).
fn auto_step(
    status: &Status,
    career: &str,
    ended: bool,
    orphan: bool,
    waiting: bool,
    tried: Option<&str>,
) -> Option<Auto> {
    if status.state.game_running {
        return None;
    }
    let prepared = status.state.prepared.as_ref();
    if orphan && prepared.is_some() {
        return Some(Auto::Restore);
    }
    let fresh = |w: &Prepared| tried != Some(w.hash.as_str());
    if waiting && status.wanted.as_ref().is_some_and(fresh) {
        return Some(Auto::Prepare);
    }
    let prepared = prepared.filter(|p| p.career == career)?;
    match &status.wanted {
        None if ended => Some(Auto::Restore),
        Some(w) if status.state.enabled && prepared != w && fresh(w) => Some(Auto::Prepare),
        _ => None,
    }
}

/// The status line: the overlay's state for `career` on this PC.
fn status_line(status: &Status, career: &str, waiting: bool) -> (Tone, String) {
    let prepared = status.state.prepared.as_ref();
    let round = |r: u8| r.to_string();
    if waiting && let Some(w) = &status.wanted {
        return (
            Tone::Warn,
            tr_owned("Prepared for round {round} after the game closes").replace("{round}", &round(w.round)),
        );
    }
    let Some(p) = prepared else {
        return (Tone::Muted, tr("Off").to_owned());
    };
    if p.career != career {
        return (
            Tone::Muted,
            tr("Off: the game is prepared for another career").to_owned(),
        );
    }
    if !status.state.enabled {
        return (
            Tone::Muted,
            tr("Off: the addon is turned off in the profile").to_owned(),
        );
    }
    match &status.wanted {
        Some(w) if p == w => (
            Tone::Good,
            tr_owned("Ready for round {round}").replace("{round}", &round(w.round)),
        ),
        Some(w) => (
            Tone::Warn,
            tr_owned("Out of date: prepare for round {round}").replace("{round}", &round(w.round)),
        ),
        None => (
            Tone::Warn,
            tr("On, but this career wants nothing in the game now: restore the normal game").to_owned(),
        ),
    }
}

/// The AI drivers the market seats in other drivers' seats this season, as the game shows them.
fn market_seats(league: &League, p: Palette) -> Div {
    let season = league.season();
    let mut out = v_flex()
        .gap(px(2.))
        .child(label(tr("AI DRIVER MARKET IN THE GAME"), p));
    if season.renames.is_empty() {
        return out.child(note(
            tr("This season every AI driver races in their own seat; the market moves them at the season's end."),
            p.text_muted,
        ));
    }
    out = out.child(note(
        tr("The game shows each of these drivers in the seat of the year's real driver, with that driver's face."),
        p.text_muted,
    ));
    for line in market_lines(league) {
        out = out.child(div().text_size(px(12.)).child(line));
    }
    out
}

/// "Max Verstappen #33 in Lewis Hamilton's seat at Mercedes" for each renamed seat.
fn market_lines(league: &League) -> Vec<String> {
    let season = league.season();
    season
        .renames
        .iter()
        .map(|r| {
            let seat =
                aimarket::seat_driver_name(season.year, r.slot, &r.seat_driver).unwrap_or(&r.seat_driver);
            let driver = season
                .entry(&r.key)
                .map_or_else(|| gamesync::market::full_name(r), |e| e.name.clone());
            tr_owned("{driver} #{number} in {seat}'s seat at {team}")
                .replace("{driver}", &driver)
                .replace("{number}", &r.number.to_string())
                .replace("{seat}", seat)
                .replace(
                    "{team}",
                    &season.team(r.slot).map_or(String::new(), |t| t.name.clone()),
                )
        })
        .collect()
}

/// "+0.42%" a car's lap time over the fastest, from its mass.
fn gap_text(mass: f32, percent_per_kg: f32) -> String {
    format!(
        "+{:.2}%",
        (mass - performance::TOP_MASS).max(0.0) * percent_per_kg
    )
}

impl Shell {
    /// Runs with the Career page's tick: looks again when the career, its version, the profile or
    /// the game's state changed (or after a while), and prepares or restores by itself.
    pub(super) fn career_sync_tick(&mut self, cx: &mut Context<Self>) {
        let c = &self.career;
        let (Some(league), Some(career)) = (c.league(), c.selected.clone()) else {
            return;
        };
        if c.sync.busy.is_some() {
            return;
        }
        let key = Key {
            career: career.clone(),
            version: c.state.as_ref().map_or(0, |s| s.version),
            profile: self.library.profile.name.clone(),
            running: !self.processes.is_empty(),
        };
        let stale =
            c.sync.key.as_ref() != Some(&key) || c.sync.at.is_none_or(|at| at.elapsed() >= LOOK_EVERY);
        if stale {
            self.career_sync_look(key, cx);
            return;
        }
        let Some(status) = &c.sync.status else { return };
        let ended = league.phase == Phase::Finished || !any_option(&league.options);
        // A solo career's overlay whose career is gone (a server career may be on another server).
        let orphan = status.state.prepared.as_ref().is_some_and(|p| {
            c.solos_read && solo::is_solo(&p.career) && !c.solos.iter().any(|s| s.id == p.career)
        });
        let waiting = c.sync.waiting.as_deref() == Some(career.as_str());
        let step = auto_step(status, &career, ended, orphan, waiting, c.sync.tried.as_deref());
        let wanted = status.wanted.as_ref().map(|w| w.hash.clone());
        match step {
            Some(Auto::Prepare) => {
                self.career.sync.tried = wanted;
                self.career_sync_prepare(cx);
            }
            Some(Auto::Restore) => self.career_sync_restore(cx),
            None => {}
        }
    }

    fn career_sync_look(&mut self, key: Key, cx: &mut Context<Self>) {
        let Some(league) = self.career.league().cloned() else {
            return;
        };
        let store = self.store.clone();
        let name = self.career.career_name();
        let s = &mut self.career.sync;
        s.busy = Some("look");
        s.key = Some(key.clone());
        cx.spawn(async move |this, cx| {
            let status = cx
                .background_executor()
                .spawn(async move { look(&store, &key.profile, &league, &key.career, &name) })
                .await;
            this.update(cx, |shell, cx| {
                let s = &mut shell.career.sync;
                s.busy = None;
                s.at = Some(Instant::now());
                // A prepare held back by the game is done once the overlay is what was wanted.
                if status.wanted.is_some() && status.state.prepared == status.wanted {
                    s.waiting = None;
                }
                s.status = Some(status);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Prepare the game for the selected career's next round, in the background.
    pub(super) fn career_sync_prepare(&mut self, cx: &mut Context<Self>) {
        let (Some(league), Some(career)) = (self.career.league().cloned(), self.career.selected.clone())
        else {
            return;
        };
        if self.career.sync.busy.is_some() {
            return;
        }
        let store = self.store.clone();
        let profile = self.library.profile.name.clone();
        let name = self.career.career_name();
        let s = &mut self.career.sync;
        s.busy = Some("prepare");
        s.error = None;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { prepare(&store, &profile, &league, &career, &name).map(|r| (r, career)) })
                .await;
            this.update(cx, |shell, cx| {
                let s = &mut shell.career.sync;
                s.busy = None;
                s.key = None;
                match result {
                    Ok(((state, wanted), career)) => {
                        let held = state.game_running && wanted.is_some() && state.prepared != wanted;
                        s.waiting = held.then_some(career);
                    }
                    Err(e) => s.error = Some(e),
                }
                // The profile's addons changed.
                shell.reload_in_background(cx);
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    /// Take the overlay out of the game, in the background.
    pub(super) fn career_sync_restore(&mut self, cx: &mut Context<Self>) {
        if self.career.sync.busy.is_some() {
            return;
        }
        let store = self.store.clone();
        let profile = self.library.profile.name.clone();
        let s = &mut self.career.sync;
        s.busy = Some("restore");
        s.error = None;
        s.waiting = None;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { careersync::restore(&store, &profile) })
                .await;
            this.update(cx, |shell, cx| {
                let s = &mut shell.career.sync;
                s.busy = None;
                s.key = None;
                if let Err(e) = result {
                    s.error = Some(e);
                }
                shell.reload_in_background(cx);
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    /// The owner changes one game option; the rest of the options stay.
    fn career_sync_option(&mut self, set: fn(&mut Options, bool), on: bool, cx: &mut Context<Self>) {
        let Some(league) = self.career.league() else {
            return;
        };
        let mut options = league.options.clone();
        set(&mut options, on);
        self.career_act(Action::SetOptions { options }, cx);
    }

    /// The game-sync panel.
    pub(super) fn career_game_sync(
        &self,
        league: &League,
        me: &str,
        owner: bool,
        p: Palette,
        cx: &mut Context<Self>,
    ) -> Div {
        let c = &self.career;
        let s = &c.sync;
        let career = c.selected.clone().unwrap_or_default();
        let working = s.busy.is_some_and(|b| b != "look") || c.busy.is_some();
        let next = league.season().next_round().map(|r| r.round);
        let (tone, line) = match (&s.status, s.busy) {
            (_, Some("prepare")) => (Tone::Muted, tr("Preparing the game…").to_owned()),
            (_, Some("restore")) => (Tone::Muted, tr("Restoring the normal game…").to_owned()),
            (Some(status), _) => status_line(status, &career, s.waiting.as_deref() == Some(career.as_str())),
            (None, _) => (Tone::Muted, tr("Looking at the game…").to_owned()),
        };
        let colour = match tone {
            Tone::Good => p.success,
            Tone::Warn => p.warning,
            Tone::Muted => p.text_muted,
        };
        let can_prepare = wants(league) && !self.library.profile.is_vanilla();
        let installed = s
            .status
            .as_ref()
            .is_some_and(|st| st.state.prepared.is_some() || st.state.enabled);
        let open = s.open;
        let head = h_flex()
            .gap(px(8.))
            .items_center()
            .flex_wrap()
            .child(label(tr("THE CAREER IN THE GAME"), p))
            .child(
                div()
                    .flex_1()
                    .min_w(px(160.))
                    .text_size(px(13.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(colour)
                    .child(line),
            )
            .when_some(next.filter(|_| can_prepare), |d, n| {
                d.child(
                    Button::new("career-sync-prepare")
                        .outline()
                        .small()
                        .icon(IconName::Gamepad2)
                        .label(
                            tr_owned("Prepare the game for round {round}").replace("{round}", &n.to_string()),
                        )
                        .disabled(working)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.career.sync.tried = None;
                            this.career_sync_prepare(cx)
                        })),
                )
            })
            .when(installed, |d| {
                d.child(
                    Button::new("career-sync-restore")
                        .outline()
                        .small()
                        .icon(IconName::RotateCcw)
                        .label(tr("Restore the normal game"))
                        .disabled(working)
                        .on_click(cx.listener(|this, _, _, cx| this.career_sync_restore(cx))),
                )
            })
            .child(
                Button::new("career-sync-details")
                    .ghost()
                    .small()
                    .icon(if open {
                        IconName::ChevronUp
                    } else {
                        IconName::ChevronDown
                    })
                    .label(if open { tr("Less") } else { tr("Details") })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.career.sync.open = !open;
                        cx.notify();
                    })),
            );
        let mut panel = v_flex()
            .p(px(14.))
            .gap(px(8.))
            .rounded(px(6.))
            .border_1()
            .border_color(p.border)
            .child(head)
            .child(self.career_sync_identity(league, me, p));
        if self.library.profile.is_vanilla() && wants(league) {
            panel = panel.child(note(
                tr("The profile on the play bar plays without addons: pick another one to prepare the game."),
                p.warning,
            ));
        }
        if let Some(problem) = s.status.as_ref().and_then(|st| st.problem.clone()) {
            panel = panel.child(note(capitalized(&problem), p.warning));
        }
        if let Some(e) = &s.error {
            panel = panel.child(note(
                tr_owned("Couldn't change the game: {error}").replace("{error}", e),
                p.error,
            ));
        }
        if !open {
            return panel;
        }
        let o = &league.options;
        panel
            .child(note(
                tr("Preparing writes a small generated addon (League career sync) into your addons and turns it on in \
                    the profile on the play bar; the next Play builds the game with it. It follows the career round \
                    by round by itself. Restoring takes it out again, and the game is as it was."),
                p.text_muted,
            ))
            .when(!c.solo(), |d| {
                d.child(note(
                    tr("Both players must prepare the same round; the lobby lists a mismatch as an addon difference."),
                    p.text_muted,
                ))
            })
            .when(o.game_performance, |d| {
                d.child(note(
                    tr("Set Car performance to Realistic in the lobby (or the Grand Prix settings): equal performance \
                        cancels the league's car pace."),
                    p.warning,
                ))
            })
            .when_some(
                s.status
                    .as_ref()
                    .and_then(|st| st.pace.clone())
                    .filter(|_| o.game_performance),
                |d, pace| d.child(self.career_sync_pace(league, me, &pace, p)),
            )
            .when(o.lineups == LineupMode::Dynamic, |d| d.child(market_seats(league, p)))
            .child(label(tr("WHAT GOES INTO THE GAME"), p))
            .child(option_row(
                "career-sync-identity",
                tr("Your names in the game"),
                if o.lineups == LineupMode::Dynamic {
                    tr("Each player's name, number and nationality replace the driver of their seat, in the menus \
                        and on the timing screens. It stays on with the AI driver market: the seat's real driver may \
                        race for another team.")
                } else {
                    tr("Each player's name, number and nationality replace the driver of their seat, in the menus and \
                        on the timing screens. In a LAN race the game may still show a human car's network name.")
                },
                o.game_identity,
                owner && o.lineups != LineupMode::Dynamic,
                |o, on| o.game_identity = on,
                p,
                cx,
            ))
            .child(option_row(
                "career-sync-performance",
                tr("The league's car pace in the game"),
                tr("Each team's car weighs what its league pace says, development included, so a team that develops \
                    is faster in the game, its AI driver too. It needs equal car performance off."),
                o.game_performance,
                owner,
                |o, on| o.game_performance = on,
                p,
                cx,
            ))
            .child(option_row(
                "career-sync-parts",
                tr("Per-part feel (experimental)"),
                tr("Aero parts add downforce and powertrain parts add power, with the car's pace kept the same. The \
                    strengths are a guess until they are measured in the game; the AI team-mate follows only the pace."),
                o.game_parts,
                owner && o.game_performance,
                |o, on| o.game_parts = on,
                p,
                cx,
            ))
            .when(!owner, |d| {
                d.child(note(tr("Only the career's owner changes these."), p.text_muted))
            })
    }

    /// "The game shows you as …" and the other players' names in the game.
    fn career_sync_identity(&self, league: &League, me: &str, p: Palette) -> Div {
        let mut out = v_flex().gap(px(2.));
        let Some(b) = league.briefing(me).filter(|_| league.phase == Phase::Racing) else {
            return out.child(note(
                tr("The game sync works while a season runs: it prepares the game for each round."),
                p.text_muted,
            ));
        };
        let seat = b.replaces.clone().unwrap_or_default();
        let team = b.team.clone().unwrap_or_default();
        match &b.game {
            Some(g) => {
                out = out.child(
                    div().text_size(px(13.)).child(
                        tr_owned("The game shows you as {name} #{number} in {driver}'s {team}.")
                            .replace("{name}", &g.full)
                            .replace("{number}", &g.number.to_string())
                            .replace("{driver}", &seat)
                            .replace("{team}", &team),
                    ),
                );
                if g.number_clash {
                    let own = league.player(me).map_or(0, |pl| pl.number);
                    out = out.child(note(
                        tr_owned("Your number #{own} belongs to another driver in {year}, so the game shows #{number}.")
                            .replace("{own}", &own.to_string())
                            .replace("{year}", &g.year.to_string())
                            .replace("{number}", &g.number.to_string()),
                        p.warning,
                    ));
                }
            }
            None if b.team.is_some() => {
                out = out.child(note(
                    tr_owned("Your name isn't in the game: it shows {driver} in your seat.")
                        .replace("{driver}", &seat),
                    p.text_muted,
                ));
            }
            None => {}
        }
        let season = league.season();
        let others: Vec<String> = league
            .active_players()
            .into_iter()
            .filter(|pl| pl.id != me)
            .filter_map(|pl| gamesync::identity_of(league, &pl.id))
            .map(|g| {
                tr_owned("{name} #{number} in the {team}")
                    .replace("{name}", &g.full)
                    .replace("{number}", &g.number.to_string())
                    .replace(
                        "{team}",
                        &season.team(g.slot).map_or(String::new(), |t| t.name.clone()),
                    )
            })
            .collect();
        if !others.is_empty() {
            out = out.child(note(
                tr_owned("Also in the game: {players}.").replace("{players}", &others.join(" · ")),
                p.text_muted,
            ));
        }
        out
    }

    /// The teams' car masses the game gets for the next round.
    fn career_sync_pace(&self, league: &League, me: &str, pace: &Pace, p: Palette) -> Div {
        let season = league.season();
        let mine = season.player_entry(me).map(|e| e.slot);
        let mut rows: Vec<(u8, f32)> = pace.masses.iter().map(|(s, m)| (*s, *m)).collect();
        rows.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
        v_flex()
            .gap(px(2.))
            .child(label(
                tr_owned("CAR PACE IN THE GAME · ROUND {round}").replace("{round}", &pace.round.to_string()),
                p,
            ))
            .child(note(
                tr_owned(
                    "The game gets each team's league pace through its car's weight, the lever Classic Seasons uses \
                     and the AI follows: each kilogram costs about {percent}% of a lap.",
                )
                .replace("{percent}", &format!("{:.3}", pace.percent_per_kg)),
                p.text_muted,
            ))
            .children(rows.into_iter().map(|(slot, mass)| {
                let name = season.team(slot).map_or(String::new(), |t| t.name.clone());
                h_flex()
                    .gap(px(10.))
                    .px(px(6.))
                    .py(px(2.))
                    .when(mine == Some(slot), |d| d.bg(p.selected))
                    .child(div().w(px(190.)).text_size(px(12.)).child(name))
                    .child(
                        div()
                            .w(px(70.))
                            .text_size(px(12.))
                            .child(tr_owned("{mass} kg").replace("{mass}", &format!("{mass:.0}"))),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(p.text_muted)
                            .child(tr_owned("{gap} a lap").replace("{gap}", &gap_text(mass, pace.percent_per_kg))),
                    )
            }))
            .when(pace.same_as_last, |d| {
                d.child(note(
                    tr("No team moved by a kilogram since the last round: the game reuses its last build of the cars."),
                    p.text_muted,
                ))
            })
    }
}

fn note(text: impl Into<SharedString>, colour: Hsla) -> Div {
    div().text_size(px(12.)).text_color(colour).child(text.into())
}

/// One game option: a switch for the owner (off when `can` is false), its name and what it does.
#[allow(clippy::too_many_arguments, reason = "one row of the panel")]
fn option_row(
    id: &'static str,
    name: &'static str,
    what: &'static str,
    on: bool,
    can: bool,
    set: fn(&mut Options, bool),
    p: Palette,
    cx: &mut Context<Shell>,
) -> Div {
    h_flex()
        .gap(px(10.))
        .items_start()
        .py(px(4.))
        .border_t_1()
        .border_color(p.border)
        .child(
            Switch::new(id)
                .checked(on)
                .small()
                .disabled(!can)
                .on_click(cx.listener(move |this, on: &bool, _, cx| this.career_sync_option(set, *on, cx))),
        )
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .text_size(px(13.))
                        .font_weight(FontWeight::MEDIUM)
                        .child(name),
                )
                .child(div().text_size(px(12.)).text_color(p.text_muted).child(what)),
        )
}

#[cfg(test)]
mod tests {
    use swofty_core::careersync::{Prepared, SyncState};
    use swofty_core::league::{Action, Actor, League, LineupMode, SeatRename, Setup, apply};

    use super::{Auto, Status, Tone, auto_step, gap_text, market_lines, status_line};

    #[test]
    fn the_market_seats_name_the_driver_the_seat_and_the_team() {
        let anna = Actor {
            id: "u-anna".into(),
            name: "Anna".into(),
            owner: true,
        };
        let mut l = League::start(Setup::default(), &anna, Some(7), 1).unwrap();
        apply(
            &mut l,
            &anna,
            Action::Seat {
                slot: 2,
                replaces: None,
            },
        )
        .unwrap();
        apply(&mut l, &anna, Action::Begin).unwrap();
        l.options.lineups = LineupMode::Dynamic;
        assert!(market_lines(&l).is_empty());
        // Max Verstappen in Lewis Hamilton's Mercedes seat, as the market would seat him.
        let season = l.seasons.last_mut().unwrap();
        season.renames.push(SeatRename {
            slot: 1,
            seat_driver: "lewis_hamilton".into(),
            key: "max_verstappen".into(),
            first: "Max".into(),
            last: "VERSTAPPEN".into(),
            code: "VER".into(),
            number: 33,
            nationality: Some("netherlands".into()),
        });
        crate::i18n::set(None);
        assert_eq!(
            market_lines(&l),
            ["Max Verstappen #33 in Lewis Hamilton's seat at Mercedes"]
        );
        crate::i18n::set(Some("de"));
        assert_eq!(
            market_lines(&l),
            ["Max Verstappen #33 im Cockpit von Lewis Hamilton bei Mercedes"]
        );
        crate::i18n::set(None);
    }

    fn prepared(career: &str, round: u8, hash: &str) -> Prepared {
        Prepared {
            career: career.to_owned(),
            year: 2016,
            round,
            hash: hash.to_owned(),
        }
    }

    fn status(installed: Option<Prepared>, enabled: bool, running: bool, wanted: Option<Prepared>) -> Status {
        Status {
            state: SyncState {
                prepared: installed,
                enabled,
                game_running: running,
            },
            wanted,
            ..Status::default()
        }
    }

    #[test]
    fn sync_prepares_again_when_the_round_moves_on() {
        let s = status(
            Some(prepared("k1", 3, "aaaa")),
            true,
            false,
            Some(prepared("k1", 4, "bbbb")),
        );
        assert_eq!(
            auto_step(&s, "k1", false, false, false, None),
            Some(Auto::Prepare)
        );
        // Not the same overlay twice after a failure.
        assert_eq!(auto_step(&s, "k1", false, false, false, Some("bbbb")), None);
        // Up to date: nothing to do.
        let s = status(
            Some(prepared("k1", 4, "bbbb")),
            true,
            false,
            Some(prepared("k1", 4, "bbbb")),
        );
        assert_eq!(auto_step(&s, "k1", false, false, false, None), None);
    }

    #[test]
    fn sync_leaves_other_careers_and_switched_off_addons_alone() {
        let wanted = Some(prepared("k1", 4, "bbbb"));
        // Never prepared on this PC, or for another career: the player prepares it.
        assert_eq!(
            auto_step(
                &status(None, false, false, wanted.clone()),
                "k1",
                false,
                false,
                false,
                None
            ),
            None
        );
        let other = status(Some(prepared("k2", 7, "cccc")), true, false, wanted.clone());
        assert_eq!(auto_step(&other, "k1", false, false, false, None), None);
        // Turned off on the Addons page: it stays off.
        let off = status(Some(prepared("k1", 3, "aaaa")), false, false, wanted);
        assert_eq!(auto_step(&off, "k1", false, false, false, None), None);
    }

    #[test]
    fn sync_waits_while_the_game_runs() {
        let wanted = Some(prepared("k1", 4, "bbbb"));
        let running = status(Some(prepared("k1", 3, "aaaa")), true, true, wanted.clone());
        assert_eq!(auto_step(&running, "k1", false, false, true, None), None);
        let closed = status(Some(prepared("k1", 3, "aaaa")), true, false, wanted);
        assert_eq!(
            auto_step(&closed, "k1", false, false, true, None),
            Some(Auto::Prepare)
        );
    }

    #[test]
    fn sync_restores_an_ended_or_orphaned_career() {
        let s = status(Some(prepared("k1", 22, "aaaa")), true, false, None);
        assert_eq!(auto_step(&s, "k1", true, false, false, None), Some(Auto::Restore));
        // The season break keeps it, for the next season's first round.
        assert_eq!(auto_step(&s, "k1", false, false, false, None), None);
        let gone = status(Some(prepared("solo-x", 2, "aaaa")), true, false, None);
        assert_eq!(
            auto_step(&gone, "k1", false, true, false, None),
            Some(Auto::Restore)
        );
    }

    #[test]
    fn status_line_names_the_round() {
        let ready = status(
            Some(prepared("k1", 4, "bbbb")),
            true,
            false,
            Some(prepared("k1", 4, "bbbb")),
        );
        let (tone, text) = status_line(&ready, "k1", false);
        assert_eq!(tone, Tone::Good);
        assert!(text.contains('4'), "{text}");
        let old = status(
            Some(prepared("k1", 3, "aaaa")),
            true,
            false,
            Some(prepared("k1", 4, "bbbb")),
        );
        let (tone, text) = status_line(&old, "k1", false);
        assert_eq!(tone, Tone::Warn);
        assert!(text.contains('4'), "{text}");
        let (tone, _) = status_line(&old, "k1", true);
        assert_eq!(tone, Tone::Warn);
        let (tone, _) = status_line(&status(None, false, false, None), "k1", false);
        assert_eq!(tone, Tone::Muted);
    }

    #[test]
    fn gap_text_reads_percent_of_a_lap() {
        assert_eq!(gap_text(798.0, 0.0308), "+0.00%");
        assert_eq!(gap_text(830.0, 0.0308), "+0.99%");
    }
}
