//! The Career page: the league career 2016-2026 (see `swofty_core::league`), driven alone (a
//! solo career kept in the launcher's data folder) or with friends on a host server. While there
//! is a league career, the launcher listens to the game's UDP telemetry and counts each
//! qualifying and race in the selected career as it ends.

use std::time::{Duration, Instant};

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{Disableable, Sizable, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use swofty_core::league::solo::{self, SoloCareer};
use swofty_core::league::{
    self, Action, Focus, League, OfferKind, Part, Phase, PointsRule, RaceReport, RoundState, SessionKind,
    Setup, TeamChoice, Untracked, progress, udp,
};
use swofty_core::multiplayer::rooms::{Api, LeagueState, SharedCareer};

use crate::pages::{GUTTER, label};
use crate::shell::{Page, Shell};
use crate::theme::{Palette, ScrollArea};

const CAREERS_EVERY: Duration = Duration::from_secs(20);
const LEAGUE_EVERY: Duration = Duration::from_secs(5);
const NOTE_FOR: Duration = Duration::from_secs(20);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tab {
    Drivers,
    Teams,
    Calendar,
    /// The player's driver: ratings, recognition, contract.
    Driver,
    /// The team's development with the player's resource points.
    Development,
    Record,
    News,
}

pub(crate) struct CareerPage {
    careers: Vec<SharedCareer>,
    /// Solo careers on this PC, the most recently changed first.
    solos: Vec<SoloCareer>,
    solos_read: bool,
    /// The start form makes a solo career (else one on the host server).
    solo_form: bool,
    loaded: bool,
    selected: Option<String>,
    state: Option<LeagueState>,
    /// This player's user id on the host server.
    me: Option<String>,
    /// The sign-in the lists belong to.
    signed_in: Option<String>,
    careers_at: Option<Instant>,
    league_at: Option<Instant>,
    polling: bool,
    busy: Option<&'static str>,
    error: Option<String>,
    note: Option<String>,
    tab: Tab,
    /// The forms to start or join another career are open.
    creating: bool,
    /// The note on show and since when, so it goes away after a while.
    note_since: Option<(String, Instant)>,
    /// Races the game reported that the selected career hasn't counted, oldest first.
    races: Vec<RaceReport>,
    /// The race a count was refused for, with why, to offer counting it anyway.
    refused: Option<(RaceReport, String)>,
    telemetry: Option<udp::Settings>,
    telemetry_read: bool,
    listener: Option<udp::Listener>,
    listen_error: Option<String>,
    races_seen: u64,
    // The forms.
    name: Entity<InputState>,
    /// The player's name in a solo career.
    player: Entity<InputState>,
    number: Entity<InputState>,
    code: Entity<InputState>,
    first_year: u16,
    last_year: u16,
    sprints: bool,
    simulate_missing: bool,
    historic: bool,
    rookie: bool,
    lineup_changes: bool,
    distance: u8,
    /// What the player asks for in the contract they sign.
    focus: Option<Focus>,
    _inputs: [Subscription; 4],
}

impl CareerPage {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Shell>) -> Self {
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("Career name, like 2016 to 2026"));
        let player = cx.new(|cx| InputState::new(window, cx).placeholder("Your name"));
        let number = cx.new(|cx| InputState::new(window, cx).placeholder("Your race number (optional)"));
        let code = cx.new(|cx| InputState::new(window, cx).placeholder("Career code (K-XXXXXX)"));
        let changed = |cx: &mut Context<Shell>, input: &Entity<InputState>| {
            cx.subscribe(input, |_, _, e: &InputEvent, cx| {
                if matches!(e, InputEvent::Change) {
                    cx.notify();
                }
            })
        };
        let inputs = [
            changed(cx, &name),
            changed(cx, &player),
            changed(cx, &number),
            changed(cx, &code),
        ];
        Self {
            careers: Vec::new(),
            solos: Vec::new(),
            solos_read: false,
            solo_form: true,
            loaded: false,
            selected: None,
            state: None,
            me: None,
            signed_in: None,
            careers_at: None,
            league_at: None,
            polling: false,
            busy: None,
            error: None,
            note: None,
            // F1MP_CAREER_TAB opens a tab at start, for screenshots and tests (like F1MP_START_PAGE).
            tab: match std::env::var("F1MP_CAREER_TAB").as_deref() {
                Ok("constructors") => Tab::Teams,
                Ok("calendar") => Tab::Calendar,
                Ok("record") => Tab::Record,
                Ok("news") => Tab::News,
                Ok("driver") => Tab::Driver,
                Ok("development") => Tab::Development,
                _ => Tab::Drivers,
            },
            creating: false,
            note_since: None,
            races: Vec::new(),
            refused: None,
            telemetry: None,
            telemetry_read: false,
            listener: None,
            listen_error: None,
            races_seen: 0,
            name,
            player,
            number,
            code,
            first_year: league::FIRST_YEAR,
            last_year: league::LAST_YEAR,
            sprints: true,
            simulate_missing: true,
            historic: true,
            rookie: false,
            lineup_changes: true,
            distance: 50,
            focus: None,
            _inputs: inputs,
        }
    }

    fn league(&self) -> Option<&League> {
        self.state.as_ref()?.league.as_ref()
    }

    fn career(&self) -> Option<&SharedCareer> {
        let id = self.selected.as_deref()?;
        self.careers.iter().find(|c| c.id == id)
    }

    /// The selected career is a solo one.
    fn solo(&self) -> bool {
        self.selected.as_deref().is_some_and(solo::is_solo)
    }

    fn solo_career(&self) -> Option<&SoloCareer> {
        let id = self.selected.as_deref()?;
        self.solos.iter().find(|c| c.id == id)
    }

    /// The selected career's name.
    fn career_name(&self) -> String {
        self.career()
            .map(|c| c.name.clone())
            .or_else(|| self.solo_career().map(|c| c.name.clone()))
            .unwrap_or_default()
    }

    /// Show a solo career that was read or changed.
    fn show_solo(&mut self, career: SoloCareer, warnings: Vec<String>, races_dir: &std::path::Path) {
        self.races = uncounted(Some(&career.league), udp::saved(races_dir));
        self.state = Some(LeagueState {
            league: Some(career.league.clone()),
            version: career.version,
            warnings,
        });
        self.selected = Some(career.id.clone());
        match self.solos.iter_mut().find(|c| c.id == career.id) {
            Some(c) => *c = career,
            None => self.solos.insert(0, career),
        }
    }
}

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().trim().to_owned()
}

/// Races not counted in `league` yet, oldest first.
fn uncounted(league: Option<&League>, saved: Vec<RaceReport>) -> Vec<RaceReport> {
    let counted: Vec<u64> = league
        .into_iter()
        .flat_map(|l| &l.seasons)
        .flat_map(|s| &s.rounds)
        .flat_map(|r| {
            r.sessions
                .iter()
                .copied()
                .chain(r.practice.iter().map(|l| l.session))
        })
        .collect();
    saved
        .into_iter()
        .filter(|r| !counted.contains(&r.session))
        .collect()
}

fn track_text(track: Option<u8>) -> String {
    track
        .and_then(league::track_name)
        .map_or_else(|| "an unknown track".to_owned(), str::to_owned)
}

/// "race", "qualifying" or "practice", for a game session type.
fn session_text(session_type: u8) -> &'static str {
    match league::session_kind(session_type) {
        Some(SessionKind::Qualifying) => "qualifying",
        Some(SessionKind::Practice) => "practice",
        _ => "race",
    }
}

/// "1st", "2nd", "11th".
fn ordinal(n: usize) -> String {
    let suffix = match (n % 10, n % 100) {
        (1, x) if x != 11 => "st",
        (2, x) if x != 12 => "nd",
        (3, x) if x != 13 => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

fn capitalized(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().chain(c).collect())
        .unwrap_or_default()
}

impl Shell {
    fn career_api(&self) -> Option<Api> {
        self.mp
            .as_ref()
            .filter(|s| s.server.is_some())
            .map(Api::for_session)
    }

    /// Runs with the multiplayer tick: keeps the career lists fresh while the page is open, and
    /// listens to the game's telemetry while there is a league career to count races in.
    pub(crate) fn career_tick(&mut self, cx: &mut Context<Self>) {
        let signed_in = self
            .mp
            .as_ref()
            .filter(|s| s.server.is_some())
            .map(|s| s.token.clone());
        if signed_in != self.career.signed_in {
            let c = &mut self.career;
            c.signed_in = signed_in.clone();
            c.careers.clear();
            c.loaded = false;
            if !c.solo() {
                c.selected = None;
                c.state = None;
            }
            c.me = None;
            c.careers_at = None;
            c.league_at = None;
            cx.notify();
        }
        if !self.career.solos_read {
            self.career_read_solos(cx);
        }
        let c = &mut self.career;
        match (&c.note, &c.note_since) {
            (Some(n), Some((seen, at))) if n == seen => {
                if at.elapsed() > NOTE_FOR {
                    c.note = None;
                    c.note_since = None;
                    cx.notify();
                }
            }
            (Some(n), _) => c.note_since = Some((n.clone(), Instant::now())),
            (None, _) => c.note_since = None,
        }
        if !self.career.telemetry_read {
            self.career.telemetry_read = true;
            self.career.telemetry = udp::settings_path().and_then(|p| udp::read_settings(&p));
        }
        self.career_listen(cx);
        if signed_in.is_none() {
            return;
        }
        let now = Instant::now();
        let visible = self.page == Page::Career;
        let c = &self.career;
        if c.polling {
            return;
        }
        let careers_due = c
            .careers_at
            .is_none_or(|at| now.duration_since(at) >= CAREERS_EVERY);
        let league_due = visible
            && c.league_at
                .is_none_or(|at| now.duration_since(at) >= LEAGUE_EVERY);
        if (visible || !c.loaded) && careers_due || league_due {
            self.career_load(cx);
        }
    }

    /// Read the solo careers on this PC; without a selected career, pick the newest one.
    fn career_read_solos(&mut self, cx: &mut Context<Self>) {
        let root = self.store.root.clone();
        let races_dir = root.join(udp::RACES_DIR);
        let c = &mut self.career;
        c.solos_read = true;
        c.solos = solo::list(&root);
        let keep = c.selected.clone().filter(|id| solo::is_solo(id));
        let pick = match keep {
            Some(id) => c.solos.iter().find(|s| s.id == id).cloned(),
            None if c.selected.is_none() => c.solos.first().cloned(),
            None => None,
        };
        if let Some(s) = pick {
            c.show_solo(s, Vec::new(), &races_dir);
        }
        cx.notify();
    }

    fn career_listen(&mut self, cx: &mut Context<Self>) {
        let c = &mut self.career;
        let has_league = c.careers.iter().any(|x| x.mode == league::MODE) || !c.solos.is_empty();
        if c.listener.is_none()
            && c.listen_error.is_none()
            && has_league
            && let Some(settings) = c.telemetry.clone().filter(udp::Settings::usable)
        {
            match udp::Listener::start(settings.listen_addr(), self.store.root.join(udp::RACES_DIR)) {
                Ok(l) => c.listener = Some(l),
                Err(e) => {
                    c.listen_error = Some(format!(
                        "Couldn't listen for race results on {}: {e}. Another telemetry tool may use that port.",
                        settings.listen_addr()
                    ))
                }
            }
            cx.notify();
        }
        let Some(status) = self.career.listener.as_ref().map(udp::Listener::status) else {
            return;
        };
        if status.races > self.career.races_seen {
            self.career.races_seen = status.races;
            if let Some(report) = status.last {
                self.career_count(report, None, false, cx);
            }
        }
    }

    fn career_load(&mut self, cx: &mut Context<Self>) {
        if self.career.solo() {
            // A solo career is read from disk; the server's list still refreshes below.
            let root = self.store.root.clone();
            if let Some(id) = self.career.selected.clone()
                && let Ok(s) = solo::load(&root, &id)
            {
                self.career.show_solo(s, Vec::new(), &root.join(udp::RACES_DIR));
            }
        }
        let Some(api) = self.career_api() else { return };
        // The server's league is fetched only for a server career.
        let selected = self.career.selected.clone().filter(|id| !solo::is_solo(id));
        let solo_selected = self.career.solo();
        let need_me = self.career.me.is_none();
        let races_dir = self.store.root.join(udp::RACES_DIR);
        let races_dir_ui = races_dir.clone();
        self.career.polling = true;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let me = if need_me {
                        Some(api.me().map_err(|e| e.message)?.id)
                    } else {
                        None
                    };
                    let careers: Vec<SharedCareer> = api
                        .careers()
                        .map_err(|e| e.message)?
                        .into_iter()
                        .filter(|c| c.mode == league::MODE)
                        .collect();
                    let pick = selected
                        .filter(|id| careers.iter().any(|c| &c.id == id))
                        .or_else(|| {
                            (!solo_selected)
                                .then(|| careers.first().map(|c| c.id.clone()))
                                .flatten()
                        });
                    let state = match &pick {
                        Some(id) => Some(api.league(id).map_err(|e| e.message)?),
                        None => None,
                    };
                    let races = uncounted(
                        state.as_ref().and_then(|s| s.league.as_ref()),
                        udp::saved(&races_dir),
                    );
                    Ok::<_, String>((me, careers, pick, state, races))
                })
                .await;
            this.update(cx, |s, cx| {
                let c = &mut s.career;
                c.polling = false;
                let now = Instant::now();
                c.careers_at = Some(now);
                c.league_at = Some(now);
                match result {
                    Ok((me, careers, pick, state, races)) => {
                        if me.is_some() {
                            c.me = me;
                        }
                        c.careers = careers;
                        c.loaded = true;
                        c.error = None;
                        // A solo career picked meanwhile stays on show.
                        if c.solo() {
                            cx.notify();
                            return;
                        }
                        if pick.is_none()
                            && let Some(s) = c.solos.first().cloned()
                        {
                            c.show_solo(s, Vec::new(), &races_dir_ui);
                            cx.notify();
                            return;
                        }
                        c.selected = pick;
                        // An action that answered meanwhile may be newer than this read.
                        let newer = c
                            .state
                            .as_ref()
                            .zip(state.as_ref())
                            .is_some_and(|(a, b)| a.version > b.version);
                        if !newer {
                            c.state = state;
                            c.races = races;
                        }
                        c.error = None;
                    }
                    // A solo career on show doesn't need the server.
                    Err(_) if c.solo() => c.loaded = true,
                    Err(e) => c.error = Some(e),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Apply `action` to the selected solo career, on this PC.
    fn career_act_solo(&mut self, action: Action) -> Result<Vec<String>, solo::SoloError> {
        let id = self.career.selected.clone().unwrap_or_default();
        let root = self.store.root.clone();
        let (career, warnings) = solo::act(&root, &id, action)?;
        self.career
            .show_solo(career, warnings.clone(), &root.join(udp::RACES_DIR));
        self.career.error = None;
        Ok(warnings)
    }

    fn career_act(&mut self, action: Action, cx: &mut Context<Self>) {
        if self.career.solo() {
            self.career.note = match self.career_act_solo(action) {
                Ok(w) if w.is_empty() => None,
                Ok(w) => Some(w.join(" · ")),
                Err(e) => Some(e.to_string()),
            };
            cx.notify();
            return;
        }
        let (Some(api), Some(id)) = (self.career_api(), self.career.selected.clone()) else {
            return;
        };
        if self.career.busy.is_some() {
            return;
        }
        self.career.busy = Some("act");
        self.career.note = None;
        let races_dir = self.store.root.join(udp::RACES_DIR);
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let state = api.league_action(&id, &action).map_err(|e| e.message)?;
                    let races = uncounted(state.league.as_ref(), udp::saved(&races_dir));
                    Ok::<_, String>((state, races))
                })
                .await;
            this.update(cx, |s, cx| {
                let c = &mut s.career;
                c.busy = None;
                match result {
                    Ok((state, races)) => {
                        if !state.warnings.is_empty() {
                            c.note = Some(state.warnings.join(" · "));
                        }
                        c.state = Some(state);
                        c.races = races;
                        c.error = None;
                    }
                    Err(e) => c.note = Some(e),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    /// Count a race the game reported in the selected career.
    fn career_count(
        &mut self,
        report: RaceReport,
        kind: Option<SessionKind>,
        force: bool,
        cx: &mut Context<Self>,
    ) {
        if self.career.selected.is_none() {
            return;
        }
        let what = session_text(report.session_type);
        if self.career.league().is_none_or(|l| l.phase != Phase::Racing) {
            self.career.note = Some(format!(
                "{} at {} ended; it waits below until the career has a round for it.",
                capitalized(what),
                track_text(report.track)
            ));
            cx.notify();
            return;
        }
        if self.career.solo() {
            self.career.refused = None;
            let action = Action::Report {
                round: None,
                kind,
                force,
                report: report.clone(),
            };
            match self.career_act_solo(action) {
                Ok(w) => {
                    self.career.note = Some(if w.is_empty() {
                        format!("The {what} is counted.")
                    } else {
                        format!("The {what} is counted. {}", w.join(" · "))
                    })
                }
                Err(solo::SoloError::League(e)) if e.reason == "wrong_track" => {
                    self.career.refused = Some((report, e.message))
                }
                Err(e) => self.career.note = Some(format!("The {what} wasn't counted: {e}")),
            }
            cx.notify();
            return;
        }
        let (Some(api), Some(id)) = (self.career_api(), self.career.selected.clone()) else {
            return;
        };
        self.career.busy = Some("count");
        self.career.refused = None;
        let races_dir = self.store.root.join(udp::RACES_DIR);
        cx.spawn(async move |this, cx| {
            let action = Action::Report {
                round: None,
                kind,
                force,
                report: report.clone(),
            };
            let result = cx
                .background_executor()
                .spawn(async move {
                    let r = api.league_action(&id, &action).map_err(|e| (e.reason, e.message));
                    let league = r.as_ref().ok().and_then(|s| s.league.clone());
                    (r, udp::saved(&races_dir), league)
                })
                .await;
            this.update(cx, |s, cx| {
                let c = &mut s.career;
                c.busy = None;
                match result {
                    (Ok(state), saved, league) => {
                        c.note = Some(if state.warnings.is_empty() {
                            format!("The {what} is counted.")
                        } else {
                            format!("The {what} is counted. {}", state.warnings.join(" · "))
                        });
                        c.races = uncounted(league.as_ref(), saved);
                        c.state = Some(state);
                    }
                    (Err((reason, message)), saved, _) => {
                        c.races = uncounted(c.league(), saved);
                        if reason == "wrong_track" {
                            c.refused = Some((report, message));
                        } else {
                            c.note = Some(format!("The {what} wasn't counted: {message}"));
                        }
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    /// Forget a race the game reported that won't be counted.
    fn career_discard(&mut self, session: u64, cx: &mut Context<Self>) {
        let _ = udp::discard(&self.store.root.join(udp::RACES_DIR), session);
        self.career.races.retain(|r| r.session != session);
        if self
            .career
            .refused
            .as_ref()
            .is_some_and(|(r, _)| r.session == session)
        {
            self.career.refused = None;
        }
        cx.notify();
    }

    fn career_create(&mut self, cx: &mut Context<Self>) {
        let api = self.career_api();
        let solo_career = self.career.solo_form || api.is_none();
        let name = value(&self.career.name, cx);
        if name.is_empty() {
            self.career.error = Some("Give the career a name.".into());
            cx.notify();
            return;
        }
        let number = match value(&self.career.number, cx) {
            n if n.is_empty() => None,
            n => match n.parse::<u8>() {
                Ok(n) => Some(n),
                Err(_) => {
                    self.career.error = Some("The race number is a number from 1 to 99.".into());
                    cx.notify();
                    return;
                }
            },
        };
        let c = &self.career;
        let setup = Setup {
            first_year: c.first_year,
            last_year: c.last_year,
            sprints: c.sprints,
            untracked: if c.simulate_missing {
                Untracked::Simulate
            } else {
                Untracked::Skip
            },
            points: if c.historic {
                PointsRule::Historic
            } else {
                PointsRule::Current
            },
            distance: c.distance,
            team_choice: if c.rookie {
                TeamChoice::Rookie
            } else {
                TeamChoice::Free
            },
            lineup_changes: c.lineup_changes,
        };
        if !solo_career && let Some(api) = api {
            return self.career_create_on_server(api, name, setup, number, cx);
        }
        let player = Some(value(&self.career.player, cx))
            .filter(|n| !n.is_empty())
            .or_else(|| self.mp.as_ref().map(|s| s.name.clone()))
            .or_else(|| std::env::var("USERNAME").ok())
            .unwrap_or_else(|| "Player".to_owned());
        let root = self.store.root.clone();
        match solo::create(&root, &name, &player, setup, number) {
            Ok(career) => {
                let c = &mut self.career;
                c.creating = false;
                c.error = None;
                c.show_solo(career, Vec::new(), &root.join(udp::RACES_DIR));
            }
            Err(e) => self.career.error = Some(e.to_string()),
        }
        cx.notify();
    }

    fn career_create_on_server(
        &mut self,
        api: Api,
        name: String,
        setup: Setup,
        number: Option<u8>,
        cx: &mut Context<Self>,
    ) {
        self.career_start_or_join(
            move || {
                let career = api
                    .create_career(&name, league::MODE, serde_json::Value::Null)
                    .map_err(|e| e.message)?;
                api.league_action(&career.id, &Action::Start { setup, number })
                    .map_err(|e| e.message)?;
                Ok(career.id)
            },
            cx,
        );
    }

    fn career_join(&mut self, cx: &mut Context<Self>) {
        let Some(api) = self.career_api() else { return };
        let code = value(&self.career.code, cx);
        if code.is_empty() {
            self.career.error = Some("Type the career code your friend sent you.".into());
            cx.notify();
            return;
        }
        let number = value(&self.career.number, cx).parse::<u8>().ok();
        self.career_start_or_join(
            move || {
                let career = api.join_career(&code).map_err(|e| e.message)?;
                if career.mode != league::MODE {
                    return Err("That code belongs to a shared career that isn't a league career.".into());
                }
                api.league_action(&career.id, &Action::Enter { number })
                    .map_err(|e| e.message)?;
                Ok(career.id)
            },
            cx,
        );
    }

    fn career_start_or_join(
        &mut self,
        work: impl FnOnce() -> Result<String, String> + Send + 'static,
        cx: &mut Context<Self>,
    ) {
        if self.career.busy.is_some() {
            return;
        }
        self.career.busy = Some("start");
        self.career.error = None;
        cx.spawn(async move |this, cx| {
            let result = cx.background_executor().spawn(async move { work() }).await;
            this.update(cx, |s, cx| {
                s.career.busy = None;
                match result {
                    Ok(id) => {
                        s.career.selected = Some(id);
                        s.career.creating = false;
                        s.career.state = None;
                        s.career.careers_at = None;
                        s.career.league_at = None;
                        s.career_load(cx);
                    }
                    Err(e) => s.career.error = Some(e),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    fn career_enable_telemetry(&mut self, cx: &mut Context<Self>) {
        if !self.processes.is_empty() {
            self.career.note = Some("Close F1 25 first: it writes its settings when it quits.".into());
            cx.notify();
            return;
        }
        let Some(path) = udp::settings_path() else { return };
        match udp::enable(&path) {
            Ok(s) => {
                self.career.telemetry = Some(s);
                self.career.listen_error = None;
                self.career.note =
                    Some("The game's UDP telemetry is on. Race results arrive by themselves now.".into());
            }
            Err(e) => self.career.note = Some(format!("Couldn't turn the telemetry on: {e}")),
        }
        cx.notify();
    }

    // -----------------------------------------------------------------------------------------
    // The page

    pub(crate) fn career_page(&mut self, p: Palette, cx: &mut Context<Self>) -> AnyElement {
        let subtitle =
            "Your own career from 2016 to 2026, alone or with friends: qualifying and races count from the \
                        game by themselves, contracts move you between teams each winter."
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
                        .unwrap_or_else(|| "Loading your careers…".into()),
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
                    .child(body),
            )
            .into_any_element()
    }

    fn career_switcher(&self, _p: Palette, cx: &mut Context<Self>) -> Div {
        let c = &self.career;
        let mut row = h_flex().gap(px(6.)).flex_wrap();
        let all: Vec<(String, String)> = c
            .solos
            .iter()
            .map(|s| (s.id.clone(), format!("{} (solo)", s.name)))
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
                        "Back to the career"
                    } else {
                        "New career"
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.career.creating = !creating;
                        cx.notify();
                    })),
            );
        }
        row
    }

    fn career_telemetry_line(&self, p: Palette, cx: &mut Context<Self>) -> Div {
        let c = &self.career;
        let text = match (&c.telemetry, &c.listener, &c.listen_error) {
            (_, _, Some(e)) => e.clone(),
            (_, Some(l), _) => {
                let s = l.status();
                if let Some((id, size)) = s.unread {
                    format!(
                        "Race results: the game sends a telemetry packet this launcher can't read (packet {id}, \
                         {size} bytes). Set the telemetry format to 2025 in the game (Settings > Telemetry)."
                    )
                } else if s.datagrams > 0 {
                    format!("Race results: the game is sending telemetry to {}.", l.addr())
                } else {
                    format!("Race results: listening on {} for the game's telemetry.", l.addr())
                }
            }
            (Some(s), None, None) if s.usable() => "Race results: listening starts with your first league career.".into(),
            (Some(_), None, None) | (None, None, None) => {
                "Race results need the game's UDP telemetry. Turn it on here (with the game closed) or in the game \
                 under Settings > Telemetry."
                    .into()
            }
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
                        .label("Turn telemetry on")
                        .on_click(cx.listener(|this, _, _, cx| this.career_enable_telemetry(cx))),
                )
            })
    }

    fn career_card(&self, title: &'static str, body: &'static str, p: Palette) -> Div {
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

    fn career_start_cards(&self, p: Palette, cx: &mut Context<Self>) -> Div {
        let c = &self.career;
        let busy = c.busy.is_some();
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
            .child(div().w(px(110.)).text_size(px(13.)).child("Race distance"))
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
                    .label("Alone")
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
                    .label("With friends")
                    .disabled(!signed_in)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.career.solo_form = false;
                        cx.notify();
                    }));
                if solo_form { b.outline() } else { b.primary() }
            });
        let start = self
            .career_card(
                "Start a career",
                if solo_form {
                    "Just you: kept on this PC, no server needed. Every round is a Grand Prix in the game (F1 World > \
                     Grand Prix) with that year's season box."
                } else {
                    "On your host server: your friends join with its code, everyone picks a seat, then you start the \
                     first season. Every round is a LAN race."
                },
                p,
            )
            .child(mode)
            .child(Input::new(&c.name).small())
            .when(solo_form, |d| d.child(Input::new(&c.player).small()))
            .child(Input::new(&c.number).small())
            .child(year("career-from", format!("From {}", c.first_year), |c, d| {
                let y = (c.first_year as i32 + d).clamp(league::FIRST_YEAR as i32, c.last_year as i32);
                c.first_year = y as u16;
            }))
            .child(year("career-to", format!("To {}", c.last_year), |c, d| {
                let y = (c.last_year as i32 + d).clamp(c.first_year as i32, league::LAST_YEAR as i32);
                c.last_year = y as u16;
            }))
            .child(distances)
            .child(
                h_flex()
                    .gap(px(6.))
                    .flex_wrap()
                    .child(toggle("career-sprints", "Sprints", c.sprints, |c| c.sprints = !c.sprints))
                    .child(toggle(
                        "career-historic",
                        "Each year's points",
                        c.historic,
                        |c| c.historic = !c.historic,
                    ))
                    .child(toggle(
                        "career-simulate",
                        "Simulate missing circuits",
                        c.simulate_missing,
                        |c| c.simulate_missing = !c.simulate_missing,
                    ))
                    .child(toggle("career-rookie", "Rookie start", c.rookie, |c| c.rookie = !c.rookie))
                    .child(toggle(
                        "career-lineups",
                        "Real driver changes",
                        c.lineup_changes,
                        |c| c.lineup_changes = !c.lineup_changes,
                    )),
            )
            .child(
                Button::new("career-start")
                    .primary()
                    .small()
                    .icon(IconName::Flag)
                    .label("Start career")
                    .loading(c.busy == Some("start"))
                    .disabled(busy)
                    .on_click(cx.listener(|this, _, _, cx| this.career_create(cx))),
            );
        let join = if signed_in {
            self.career_card(
                "Join a friend's career",
                "Type the career code your friend sent you (K-XXXXXX). You need to be signed in to their server.",
                p,
            )
            .child(Input::new(&c.code).small())
            .child(Input::new(&c.number).small())
            .child(
                Button::new("career-join")
                    .primary()
                    .small()
                    .label("Join career")
                    .loading(c.busy == Some("start"))
                    .disabled(busy)
                    .on_click(cx.listener(|this, _, _, cx| this.career_join(cx))),
            )
        } else {
            self.career_card(
                "Drive with friends",
                "A career with friends lives on a host server. Host one or join a friend's on the Multiplayer page, \
                 then start or join it here.",
                p,
            )
            .child(
                Button::new("career-to-mp")
                    .outline()
                    .small()
                    .icon(IconName::Users)
                    .label("Open Multiplayer")
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

    fn career_view(&self, p: Palette, cx: &mut Context<Self>) -> Div {
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
            .when(league.phase != Phase::Setup, |d| {
                d.child(self.career_tabs(league, &me, p, cx))
            })
    }

    fn career_banner(&self, league: &League, p: Palette, cx: &mut Context<Self>) -> Div {
        let season = league.season();
        let done = season.rounds.iter().filter(|r| r.done()).count();
        let career = self.career.career();
        let code = career.map(|c| c.code.clone()).unwrap_or_default();
        let solo_career = self.career.solo();
        let phase = match league.phase {
            Phase::Setup => "picking seats".to_owned(),
            Phase::Racing => format!(
                "round {} of {}",
                (done + 1).min(season.rounds.len()),
                season.rounds.len()
            ),
            Phase::SeasonEnd => "contracts for next season".to_owned(),
            Phase::Finished => "complete".to_owned(),
        };
        let players: Vec<String> = league
            .players
            .iter()
            .map(|pl| {
                let team = season
                    .player_entry(&pl.id)
                    .and_then(|e| season.team(e.slot))
                    .map_or("no seat yet".to_owned(), |t| t.name.clone());
                let titles = if pl.titles.is_empty() {
                    String::new()
                } else {
                    format!(" · {}x champion", pl.titles.len())
                };
                let contract = pl
                    .contract_until
                    .filter(|_| league.phase != Phase::Setup)
                    .map_or(String::new(), |y| {
                        format!(" · {} a season to {y}", progress::money(u64::from(pl.salary_k)))
                    });
                let status = if pl.number_one {
                    " · number one driver"
                } else {
                    ""
                };
                format!(
                    "{} #{} · {team} · rating {:.0}{status}{contract}{titles}",
                    pl.name, pl.number, pl.rating
                )
            })
            .collect();
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
                            .child(div().text_size(px(12.)).text_color(p.text_muted).child(format!(
                                "{} to {} · {phase}{}",
                                league.setup.first_year,
                                league.setup.last_year,
                                if solo_career { " · solo" } else { "" }
                            ))),
                    )
                    .when(!solo_career, |d| {
                        d.child(label("CODE", p))
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
                                    .label("Copy")
                                    .on_click(
                                        cx.listener(move |this, _, _, cx| {
                                            this.mp_copy_code(code.clone(), cx)
                                        }),
                                    ),
                            )
                    }),
            )
            .children(players.into_iter().map(|t| div().text_size(px(13.)).child(t)))
    }

    fn career_seats(
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
        let rows = teams.into_iter().enumerate().map(|(rank, t)| {
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
                        Some(pl) => format!("{} (now {})", d.name, pl.name),
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
        });
        let ready = league.players.iter().all(|pl| pl.seat.is_some());
        v_flex()
            .gap(px(8.))
            .child(
                div()
                    .text_size(px(14.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(format!(
                        "Pick your seat for {}: you take that driver's place for the season",
                        season.year
                    )),
            )
            .when(league.setup.team_choice == TeamChoice::Rookie, |d| {
                d.child(div().text_size(px(12.)).text_color(p.text_muted).child(
                    "Rookie start: the faster half of the grid opens up through contract offers later.",
                ))
            })
            .child(v_flex().children(rows))
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
                                    "Your seat is set. Start the season when you're ready."
                                } else if ready {
                                    "Everyone has a seat. Start the season when your friends are ready."
                                } else {
                                    "Waiting for everyone to pick a seat."
                                }
                            } else {
                                "The career's owner starts the season once everyone has a seat."
                            }),
                    )
                    .when(owner, |d| {
                        d.child(
                            Button::new("career-begin")
                                .primary()
                                .small()
                                .icon(IconName::Flag)
                                .label(format!("Start the {} season", season.year))
                                .disabled(busy || !ready)
                                .on_click(cx.listener(|this, _, _, cx| this.career_act(Action::Begin, cx))),
                        )
                    }),
            )
    }

    fn career_next(&self, league: &League, me: &str, p: Palette, cx: &mut Context<Self>) -> Div {
        let busy = self.career.busy.is_some();
        let Some(b) = league.briefing(me) else {
            return v_flex();
        };
        let sprint_next = b.sprint && !b.sprint_done;
        let can_undo = league
            .season()
            .rounds
            .iter()
            .any(|r| (r.done() && !r.auto) || r.started());
        let what = match (b.quali_done, sprint_next) {
            (false, true) => "qualifying, the sprint, then the race",
            (false, false) => "qualifying, then the race",
            (true, true) => "the sprint, then the race",
            (true, false) => "the race (qualifying is counted)",
        };
        let what = if b.practice_done || b.quali_done {
            what.to_owned()
        } else {
            format!("a practice session, {what}")
        };
        let me_player = league.player(me);
        let practice = league
            .season()
            .round(b.round)
            .and_then(|r| r.practice.iter().find(|l| l.player == me).cloned());
        let practice_text = match &practice {
            Some(l) => {
                let grades: Vec<String> = l
                    .programmes
                    .iter()
                    .map(|x| format!("{} {}", x.programme.name(), x.grade))
                    .collect();
                format!(
                    "Practice{}: grade {} ({}), {} resource points{}.",
                    if l.driven { "" } else { " (simulated)" },
                    l.grade,
                    grades.join(", "),
                    l.rp,
                    if l.best_lap_ms > 0 {
                        format!(", best lap {}", progress::lap_text(l.best_lap_ms))
                    } else {
                        String::new()
                    }
                )
            }
            None if b.quali_done => "Practice: left out this round.".to_owned(),
            None => "Practice: drive a practice session first for resource points. It grades your laps \
                     (track acclimatisation), your best lap against your team-mate (qualifying pace) and \
                     a long run (tyre management). Or simulate it, for fewer points."
                .to_owned(),
        };
        let rp_text = me_player.map(|pl| {
            format!(
                "{} resource points to spend on the car (R&D tab) · recognition {:.0}{}",
                pl.rp,
                pl.recognition,
                if pl.number_one {
                    " · number one driver"
                } else {
                    ""
                }
            )
        });
        let seat = match (&b.team, &b.replaces) {
            (Some(t), Some(r)) => format!("You drive for {t}: pick {r} in the game."),
            _ => "You have no seat this season; you get offers when it ends.".to_owned(),
        };
        let mut aims = Vec::new();
        if let Some(g) = b.goal {
            aims.push(format!("Team goal: P{g} or better"));
        }
        if let Some(m) = &b.mate {
            aims.push(format!("team-mate {m}"));
        }
        if let Some(r) = &b.rival {
            aims.push(format!("rival {r}"));
        }
        let howto = if b.solo {
            format!(
                "In the game: F1 World > Grand Prix, the F1 {} box, the {} track, {}% race distance; drive {what}. \
                 Each session counts by itself when it ends, also when you let the game simulate it.",
                b.year,
                track_text(b.track),
                b.distance
            )
        } else {
            format!(
                "In the game: open a LAN lobby (Multiplayer page), choose the {} season box and the {} track, \
                 {}% race distance, and drive {what}. Each session counts by itself when it ends.",
                b.year,
                track_text(b.track),
                b.distance
            )
        };
        let mut card = v_flex()
            .p(px(14.))
            .gap(px(8.))
            .rounded(px(6.))
            .border_1()
            .border_color(p.accent)
            .child(label(format!("NEXT · ROUND {} OF {}", b.round, b.rounds), p))
            .child(
                div()
                    .text_size(px(18.))
                    .font_weight(FontWeight::BOLD)
                    .child(format!("{} {}", b.year, b.race)),
            )
            .child(div().text_size(px(13.)).child(howto))
            .child(div().text_size(px(13.)).text_color(p.text_muted).child(seat))
            .when(!aims.is_empty(), |d| {
                d.child(
                    div()
                        .text_size(px(13.))
                        .text_color(p.text_muted)
                        .child(aims.join(" · ")),
                )
            })
            .when(!b.lineup.is_empty(), |d| {
                d.child(label("DRIVER CHANGES THIS ROUND", p))
                    .children(b.lineup.iter().map(|n| {
                        div()
                            .text_size(px(12.))
                            .text_color(p.text_muted)
                            .child(format!("{n}."))
                    }))
            })
            .child(
                h_flex()
                    .gap(px(8.))
                    .items_center()
                    .child(div().flex_1().text_size(px(13.)).child(practice_text))
                    .when(practice.is_none() && !b.quali_done && b.team.is_some(), |d| {
                        d.child(
                            Button::new("career-practice")
                                .outline()
                                .small()
                                .icon(IconName::Gauge)
                                .label("Simulate practice")
                                .disabled(busy)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.career_act(Action::Practice { round: None }, cx)
                                })),
                        )
                    }),
            )
            .when_some(rp_text, |d, t| {
                d.child(div().text_size(px(12.)).text_color(p.text_muted).child(t))
            })
            .child(
                h_flex()
                    .gap(px(8.))
                    .child(
                        Button::new("career-simulate")
                            .outline()
                            .small()
                            .icon(IconName::Dices)
                            .label("Simulate this round")
                            .disabled(busy)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.career_act(Action::Simulate { round: None }, cx)
                            })),
                    )
                    .child(
                        Button::new("career-skip")
                            .outline()
                            .small()
                            .icon(IconName::SkipForward)
                            .label("Leave it out")
                            .disabled(busy)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.career_act(Action::Skip { round: None }, cx)
                            })),
                    )
                    .child(
                        Button::new("career-undo")
                            .outline()
                            .small()
                            .icon(IconName::Undo)
                            .label("Take back the last round")
                            .disabled(busy || !can_undo)
                            .on_click(cx.listener(|this, _, _, cx| this.career_act(Action::Undo, cx))),
                    ),
            );
        if let Some((report, why)) = self.career.refused.clone() {
            card = card.child(
                h_flex()
                    .gap(px(8.))
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(13.))
                            .text_color(p.warning)
                            .child(format!(
                                "The race at {} wasn't counted: {why}.",
                                track_text(report.track)
                            )),
                    )
                    .child(
                        Button::new("career-force")
                            .outline()
                            .small()
                            .label("Count it anyway")
                            .disabled(busy)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.career_count(report.clone(), None, true, cx)
                            })),
                    ),
            );
        }
        let waiting = self.career.races.iter().rev().take(4).map(|r| {
            let mine = r
                .cars
                .iter()
                .find(|c| c.mine)
                .map(|c| format!(", you P{}", c.position));
            let quali = league::session_kind(r.session_type) == Some(SessionKind::Qualifying);
            let practice = league::session_kind(r.session_type) == Some(SessionKind::Practice);
            let text = format!(
                "{} · {} · {} cars{}",
                capitalized(session_text(r.session_type)),
                track_text(r.track),
                r.cars.len(),
                mine.unwrap_or_default()
            );
            let (a, b2, session) = (r.clone(), r.clone(), r.session);
            h_flex()
                .gap(px(8.))
                .items_center()
                .child(div().flex_1().text_size(px(13.)).child(text))
                .child(
                    Button::new(SharedString::from(format!("count-race-{:x}", r.session)))
                        .outline()
                        .small()
                        .label(if practice {
                            "Count practice"
                        } else if quali {
                            "Count qualifying"
                        } else {
                            "Count as race"
                        })
                        .disabled(busy)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            let kind = if practice {
                                SessionKind::Practice
                            } else if quali {
                                SessionKind::Qualifying
                            } else {
                                SessionKind::Race
                            };
                            this.career_count(a.clone(), Some(kind), false, cx)
                        })),
                )
                .when(sprint_next && !quali && !practice, |d| {
                    d.child(
                        Button::new(SharedString::from(format!("count-sprint-{:x}", r.session)))
                            .outline()
                            .small()
                            .label("Count as sprint")
                            .disabled(busy)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.career_count(b2.clone(), Some(SessionKind::Sprint), false, cx)
                            })),
                    )
                })
                .child(
                    Button::new(SharedString::from(format!("discard-race-{:x}", r.session)))
                        .outline()
                        .small()
                        .icon(IconName::X)
                        .label("Discard")
                        .on_click(cx.listener(move |this, _, _, cx| this.career_discard(session, cx))),
                )
        });
        let waiting: Vec<_> = waiting.collect();
        if !waiting.is_empty() {
            card = card
                .child(label("SESSIONS THE GAME REPORTED, NOT COUNTED YET", p))
                .children(waiting);
        }
        card
    }

    fn career_offers(
        &self,
        league: &League,
        me: &str,
        owner: bool,
        p: Palette,
        cx: &mut Context<Self>,
    ) -> Div {
        let busy = self.career.busy.is_some();
        let next = league.year() + 1;
        let cat = league::catalog_year(next);
        let signed = league.player(me).and_then(|pl| pl.signed.as_ref());
        let focus_row = self.career_focus(p, cx);
        let offers = league.offers(me).into_iter().map(|o| {
            let team = cat.and_then(|c| c.teams.iter().find(|t| t.slot == o.slot));
            let slot = o.slot;
            let name = team.map_or("?".to_owned(), |t| t.name.clone());
            let years = |n: u8| format!("{n} year{}", if n == 1 { "" } else { "s" });
            let pay = format!(
                "{} a season + {} a point",
                progress::money(u64::from(o.salary_k)),
                progress::money(u64::from(o.bonus_k))
            );
            let kind = match o.kind {
                OfferKind::Contract => format!("Under contract: {} more · {pay}", years(o.years)),
                OfferKind::Renew => format!("Your team keeps you · {} · {pay}", years(o.years)),
                OfferKind::Move => format!("Offer · {} · {pay}", years(o.years)),
            };
            let focus = self.career.focus;
            let taken: Vec<&str> = league
                .players
                .iter()
                .filter(|pl| pl.id != me)
                .filter_map(|pl| pl.signed.as_ref())
                .filter(|s| s.slot == slot)
                .map(|s| s.replaces.as_str())
                .collect();
            let drivers = team.map(|t| t.drivers.clone()).unwrap_or_default();
            h_flex()
                .gap(px(10.))
                .py(px(6.))
                .items_center()
                .border_t_1()
                .border_color(p.border)
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
                .children(drivers.into_iter().map(|d| {
                    let key = d.key.clone();
                    let mine = signed.is_some_and(|s| s.slot == slot && s.replaces == d.key);
                    let b = Button::new(SharedString::from(format!("sign-{slot}-{}", d.key)))
                        .small()
                        .label(format!("Take {}'s seat", d.name))
                        .disabled(busy || taken.contains(&d.key.as_str()))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.career_act(
                                Action::Sign {
                                    slot,
                                    replaces: Some(key.clone()),
                                    focus,
                                },
                                cx,
                            )
                        }));
                    if mine { b.primary() } else { b.outline() }
                }))
        });
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
                    .child(format!("The {} season is over. Sign for {next}:", league.year())),
            )
            .child(div().text_size(px(12.)).text_color(p.text_muted).child(
                "Offers follow how you did against your car and your team-mate, and how much your team values \
                 you: a team that doesn't value you lets your contract run out. Under contract, only faster teams can buy \
                 you out; longer deals keep you from the market.",
            ))
            .child(focus_row)
            .child(v_flex().children(offers))
            .child(
                h_flex()
                    .gap(px(10.))
                    .items_center()
                    .child(div().flex_1().text_size(px(12.)).text_color(p.text_muted).child(
                        if league.players.len() == 1 {
                            format!("Sign an offer and the {next} season starts.")
                        } else if waiting.is_empty() {
                            "Everyone signed.".to_owned()
                        } else {
                            format!(
                                "Waiting for {} to sign. The next season starts when everyone has.",
                                waiting.join(", ")
                            )
                        },
                    ))
                    .when(owner && !waiting.is_empty() && league.players.len() > 1, |d| {
                        d.child(
                            Button::new("career-next-season")
                                .outline()
                                .small()
                                .label("Start next season anyway")
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
                            .label("Take back the last round")
                            .disabled(busy)
                            .on_click(cx.listener(|this, _, _, cx| this.career_act(Action::Undo, cx))),
                    ),
            )
    }

    /// What the player asks for in the contract they sign.
    fn career_focus(&self, p: Palette, cx: &mut Context<Self>) -> Div {
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
                    .child("In the new contract, ask for:"),
            )
            .child(choice("career-focus-none", "Nothing extra", None))
            .child(choice(
                "career-focus-salary",
                "More salary (+25%)",
                Some(Focus::Salary),
            ))
            .child(choice(
                "career-focus-development",
                "A say in the development (+20% resource points)",
                Some(Focus::Development),
            ))
    }

    fn career_finished(&self, league: &League, p: Palette) -> Div {
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
                    .child("The career is complete"),
            )
            .children(champions.into_iter().map(|t| div().text_size(px(13.)).child(t)))
    }

    fn career_tabs(&self, league: &League, me: &str, p: Palette, cx: &mut Context<Self>) -> Div {
        let tab = self.career.tab;
        let season = league.season();
        let body = match tab {
            Tab::Drivers => self.drivers_table(season, me, p),
            Tab::Teams => self.teams_table(season, p),
            Tab::Calendar => self.calendar(season, me, p),
            Tab::Driver => self.career_driver(league, me, p),
            Tab::Development => self.career_development(league, me, p, cx),
            Tab::Record => self.career_record(league, me, p),
            Tab::News => v_flex().children(league.news.iter().rev().take(40).map(|n| {
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
                    .child(div().flex_1().text_size(px(13.)).child(n.text.clone()))
            })),
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
        v_flex()
            .gap(px(10.))
            .child(
                h_flex()
                    .gap(px(6.))
                    .flex_wrap()
                    .child(button("career-tab-drivers", "Drivers", Tab::Drivers))
                    .child(button("career-tab-teams", "Constructors", Tab::Teams))
                    .child(button("career-tab-calendar", "Calendar", Tab::Calendar))
                    .child(button("career-tab-driver", "Your driver", Tab::Driver))
                    .child(button("career-tab-development", "R&D", Tab::Development))
                    .child(button("career-tab-record", "Your career", Tab::Record))
                    .child(button("career-tab-news", "News", Tab::News)),
            )
            .child(body)
    }

    fn drivers_table(&self, season: &league::Season, me: &str, p: Palette) -> Div {
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
                .child(
                    div()
                        .w(px(70.))
                        .text_size(px(12.))
                        .text_color(p.text_muted)
                        .child(format!("{} win{}", d.wins, if d.wins == 1 { "" } else { "s" })),
                )
                .child(
                    div()
                        .w(px(80.))
                        .text_size(px(12.))
                        .text_color(p.text_muted)
                        .child(format!(
                            "{} podium{}",
                            d.podiums,
                            if d.podiums == 1 { "" } else { "s" }
                        )),
                )
                .child(div().text_size(px(12.)).text_color(p.text_muted).child(format!(
                    "{} pole{}",
                    d.poles,
                    if d.poles == 1 { "" } else { "s" }
                )))
        }))
    }

    /// The player's career: totals, every season, and this season's duels.
    fn career_record(&self, league: &League, me: &str, p: Palette) -> Div {
        let s = league::career_stats(league, me);
        let tile = |label_text: &'static str, value: String| {
            v_flex()
                .min_w(px(90.))
                .p(px(10.))
                .gap(px(2.))
                .rounded(px(6.))
                .border_1()
                .border_color(p.border)
                .child(
                    div()
                        .text_size(px(18.))
                        .font_weight(FontWeight::BOLD)
                        .child(value),
                )
                .child(label(label_text, p))
        };
        let totals = h_flex()
            .gap(px(8.))
            .flex_wrap()
            .child(tile("STARTS", s.starts.to_string()))
            .child(tile("WINS", s.wins.to_string()))
            .child(tile("PODIUMS", s.podiums.to_string()))
            .child(tile("POLES", s.poles.to_string()))
            .child(tile("FASTEST LAPS", s.fastest_laps.to_string()))
            .child(tile("POINTS", format!("{}", s.points)))
            .child(tile("TITLES", s.titles.to_string()))
            .child(tile(
                "BEST FINISH",
                if s.best_finish > 0 {
                    format!("P{}", s.best_finish)
                } else {
                    "-".to_owned()
                },
            ));
        let seasons = v_flex().children(s.seasons.iter().map(|l| {
            h_flex()
                .gap(px(10.))
                .py(px(4.))
                .px(px(6.))
                .border_t_1()
                .border_color(p.border)
                .child(div().w(px(50.)).text_size(px(13.)).child(l.year.to_string()))
                .child(div().w(px(170.)).text_size(px(13.)).child(l.team.clone()))
                .child(
                    div()
                        .w(px(60.))
                        .text_size(px(13.))
                        .child(format!("P{}", l.position)),
                )
                .child(
                    div()
                        .w(px(70.))
                        .text_size(px(13.))
                        .child(format!("{} pts", l.points)),
                )
                .child(div().text_size(px(12.)).text_color(p.text_muted).child(format!(
                    "{} wins · {} podiums · {} poles · {} starts{}",
                    l.wins,
                    l.podiums,
                    l.poles,
                    l.starts,
                    if l.complete { "" } else { " · running" }
                )))
        }));
        let season = league.season();
        let duel = |h: league::HeadToHead, what: &str| {
            div().text_size(px(13.)).child(format!(
                "{what} {}: qualifying {}:{}, races {}:{}, points {}:{}",
                h.other_name, h.quali.0, h.quali.1, h.race.0, h.race.1, h.points.0, h.points.1
            ))
        };
        let entry = season.player_entry(me);
        let mate = entry
            .and_then(|e| league::team_mate(season, &e.id).map(|m| (e, m)))
            .and_then(|(e, m)| league::head_to_head(season, &e.id, &m.id));
        let rival = entry
            .zip(league.player(me).and_then(|pl| pl.rival.clone()))
            .and_then(|(e, r)| league::head_to_head(season, &e.id, &r));
        v_flex()
            .gap(px(12.))
            .child(totals)
            .when(mate.is_some() || rival.is_some(), |d| {
                d.child(label(format!("{} DUELS", season.year), p))
                    .when_some(mate, |d, h| d.child(duel(h, "Team-mate")))
                    .when_some(rival, |d, h| d.child(duel(h, "Rival")))
            })
            .child(label("SEASONS", p))
            .child(seasons)
    }

    /// The player's driver: the four ratings next to the team-mate's and the rival's, recognition,
    /// the contract and the earnings.
    fn career_driver(&self, league: &League, me: &str, p: Palette) -> Div {
        let Some(pl) = league.player(me) else {
            return v_flex();
        };
        let season = league.season();
        let mine = pl
            .attributes
            .unwrap_or_else(|| league::Attributes::from_rating(pl.rating));
        let entry = season.player_entry(me);
        let others: Vec<(String, league::Attributes)> = entry
            .and_then(|e| league::team_mate(season, &e.id))
            .into_iter()
            .chain(pl.rival.as_deref().and_then(|r| season.entry(r)))
            .filter_map(|e| {
                let a = match &e.player {
                    Some(id) => league.player(id).and_then(|x| x.attributes)?,
                    None => league::driver_attributes(season.year, &e.id)?,
                };
                Some((e.name.clone(), a))
            })
            .collect();
        let bar = |value: f32| {
            let w = ((value - 40.0) / 59.0).clamp(0.0, 1.0) * 160.0;
            div()
                .w(px(160.))
                .h(px(8.))
                .rounded(px(4.))
                .bg(p.border)
                .child(div().w(px(w)).h(px(8.)).rounded(px(4.)).bg(p.accent))
        };
        let row = |name: &'static str, get: fn(&league::Attributes) -> f32| {
            h_flex()
                .gap(px(10.))
                .py(px(4.))
                .items_center()
                .border_t_1()
                .border_color(p.border)
                .child(div().w(px(110.)).text_size(px(13.)).child(name))
                .child(bar(get(&mine)))
                .child(
                    div()
                        .w(px(50.))
                        .text_size(px(13.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(format!("{:.1}", get(&mine))),
                )
                .children(others.iter().map(|(_, a)| {
                    div()
                        .w(px(150.))
                        .text_size(px(12.))
                        .text_color(p.text_muted)
                        .child(format!("{:.1}", get(a)))
                }))
        };
        let header = h_flex()
            .gap(px(10.))
            .child(div().w(px(110.)))
            .child(div().w(px(160.)))
            .child(div().w(px(50.)).child(label("YOU", p)))
            .children(others.iter().map(|(n, _)| {
                div()
                    .w(px(150.))
                    .text_size(px(12.))
                    .text_color(p.text_muted)
                    .child(n.clone())
            }));
        let team = entry
            .and_then(|e| season.team(e.slot))
            .map_or("no team".to_owned(), |t| t.name.clone());
        let status = if pl.number_one {
            format!("{team}'s number one driver: a tenth more resource points.")
        } else if pl.recognition >= progress::NUMBER_ONE {
            "Valued highly; score more points than your team-mate to become the number one driver.".to_owned()
        } else if pl.recognition < progress::RECOGNITION_RENEW {
            format!(
                "{team} isn't happy: below {:.0}, a team lets the contract run out.",
                progress::RECOGNITION_RENEW
            )
        } else {
            format!(
                "Meet the team's goals and beat your team-mate; from {:.0} on, and ahead of your team-mate, you're \
                 the number one driver.",
                progress::NUMBER_ONE
            )
        };
        let focus = match pl.focus {
            Some(Focus::Salary) => " · you asked for more salary",
            Some(Focus::Development) => " · you have a say in the development (+20% resource points)",
            None => "",
        };
        v_flex()
            .gap(px(12.))
            .child(
                h_flex()
                    .gap(px(16.))
                    .items_center()
                    .child(
                        div()
                            .text_size(px(28.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(p.accent)
                            .child(format!("{:.1}", pl.rating)),
                    )
                    .child(
                        v_flex()
                            .child(div().text_size(px(15.)).font_weight(FontWeight::SEMIBOLD).child(format!(
                                "{} #{} · {team}",
                                pl.name, pl.number
                            )))
                            .child(div().text_size(px(12.)).text_color(p.text_muted).child(
                                "Overall rating, weighed as the game weighs it: pace twice, racecraft once, \
                                 experience and awareness a quarter each.",
                            )),
                    ),
            )
            .child(
                v_flex()
                    .child(header)
                    .child(row("Experience", |a| a.experience))
                    .child(row("Racecraft", |a| a.racecraft))
                    .child(row("Awareness", |a| a.awareness))
                    .child(row("Pace", |a| a.pace)),
            )
            .child(div().text_size(px(12.)).text_color(p.text_muted).child(
                "Every session you drive moves them: experience with each one, pace with qualifying and practice \
                 against your team-mate, racecraft with places gained, awareness with clean races (penalties and \
                 crashes cost it). Simulated sessions only add experience.",
            ))
            .child(label("TEAM", p))
            .child(
                h_flex()
                    .gap(px(10.))
                    .items_center()
                    .child(div().w(px(110.)).text_size(px(13.)).child("Recognition"))
                    .child(bar(40.0 + pl.recognition / 100.0 * 59.0))
                    .child(div().text_size(px(13.)).child(format!("{:.0} of 100", pl.recognition))),
            )
            .child(div().text_size(px(12.)).text_color(p.text_muted).child(status))
            .child(label("CONTRACT", p))
            .child(div().text_size(px(13.)).child(format!(
                "{} a season + {} a point, until {}{focus}",
                progress::money(u64::from(pl.salary_k)),
                progress::money(u64::from(pl.bonus_k)),
                pl.contract_until.map_or("?".to_owned(), |y| y.to_string())
            )))
            .child(div().text_size(px(13.)).child(format!(
                "Earned in the career: {}",
                progress::money(pl.earnings_k)
            )))
    }

    /// The team's development: the player's resource points, the four parts with their levels
    /// and the next one's cost, the parts on order and the car's pace.
    fn career_development(&self, league: &League, me: &str, p: Palette, cx: &mut Context<Self>) -> Div {
        let season = league.season();
        let (Some(pl), Some(entry)) = (league.player(me), season.player_entry(me)) else {
            return v_flex().child(
                div()
                    .text_size(px(13.))
                    .text_color(p.text_muted)
                    .child("You have no seat this season."),
            );
        };
        let slot = entry.slot;
        let Some(team) = season.team(slot) else {
            return v_flex();
        };
        let busy = self.career.busy.is_some();
        let racing = league.phase == Phase::Racing;
        let round = season.current_round();
        let last = season.rounds.last().map_or(0, |r| r.round);
        let late = season
            .next_round()
            .is_none_or(|r| r.round + progress::UPGRADE_ROUNDS > last);
        let parts = Part::ALL.into_iter().map(|part| {
            let level = season.part_level(slot, part);
            let cost = progress::upgrade_cost(level);
            let maxed = level >= progress::UPGRADE_LEVELS;
            let effect = if part == Part::Durability {
                "fewer breakdowns".to_owned()
            } else {
                format!("{:.2}% a lap a level", part.gain())
            };
            let pips = h_flex()
                .gap(px(3.))
                .children((0..progress::UPGRADE_LEVELS).map(|i| {
                    div().w(px(10.)).h(px(10.)).rounded(px(2.)).bg(if i < level {
                        p.accent
                    } else {
                        p.border
                    })
                }));
            h_flex()
                .gap(px(10.))
                .py(px(6.))
                .items_center()
                .border_t_1()
                .border_color(p.border)
                .child(
                    v_flex()
                        .w(px(150.))
                        .child(
                            div()
                                .text_size(px(13.))
                                .font_weight(FontWeight::MEDIUM)
                                .child(part.name()),
                        )
                        .child(div().text_size(px(11.)).text_color(p.text_muted).child(effect)),
                )
                .child(pips)
                .child(
                    div()
                        .w(px(70.))
                        .text_size(px(12.))
                        .text_color(p.text_muted)
                        .child(format!("level {level}")),
                )
                .child(
                    Button::new(SharedString::from(format!("develop-{part:?}")))
                        .outline()
                        .small()
                        .icon(IconName::Wrench)
                        .label(if maxed {
                            "Fully developed".to_owned()
                        } else {
                            format!("Develop · {cost} points")
                        })
                        .disabled(busy || !racing || late || maxed || pl.rp < cost)
                        .on_click(
                            cx.listener(move |this, _, _, cx| this.career_act(Action::Develop { part }, cx)),
                        ),
                )
        });
        let on_order: Vec<String> = season
            .upgrades
            .iter()
            .filter(|u| u.slot == slot && u.ready > round)
            .map(|u| {
                let race = season.round(u.ready).map_or("", |r| r.race.as_str());
                format!(
                    "{} level {}: on the car from round {} ({race})",
                    u.part.name(),
                    u.level,
                    u.ready
                )
            })
            .collect();
        let start = team.pace + team.carry;
        let now = season.pace_now(slot);
        v_flex()
            .gap(px(12.))
            .child(
                h_flex()
                    .gap(px(16.))
                    .items_center()
                    .child(
                        div()
                            .text_size(px(28.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(p.accent)
                            .child(pl.rp.to_string()),
                    )
                    .child(
                        v_flex()
                            .child(
                                div()
                                    .text_size(px(15.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(format!("resource points for {}", team.name)),
                            )
                            .child(div().text_size(px(12.)).text_color(p.text_muted).child(
                                "Earned in practice programmes, with qualifying, results, team goals and beating \
                                 your team-mate. A new part takes two rounds to build.",
                            )),
                    ),
            )
            .child(v_flex().children(parts))
            .when(late && racing, |d| {
                d.child(div().text_size(px(12.)).text_color(p.warning).child(
                    "Too late in the season: a new part wouldn't be ready. Your points carry over to next season.",
                ))
            })
            .when(!on_order.is_empty(), |d| {
                d.child(label("ON ORDER", p)).children(
                    on_order
                        .into_iter()
                        .map(|t| div().text_size(px(13.)).child(t)),
                )
            })
            .child(label("THE CAR", p))
            .child(div().text_size(px(13.)).child(format!(
                "Gap to the fastest car: {now:.2}% now, {start:.2}% at the start of {}. The {} fastest car of {}.",
                season.year,
                ordinal(season.pace_rank(slot)),
                season.teams.len()
            )))
            .child(div().text_size(px(12.)).text_color(p.text_muted).child(format!(
                "Every team's car also moves over the season as it did for real. Half of what a team develops carries \
                 into next season (a sixth when the rules change, 2017, 2022 and 2026){}. The upgrades make the car \
                 faster in simulated rounds and raise the team's goals; the game's own car stays as the season box \
                 has it.",
                if team.carry.abs() > 1e-4 {
                    format!("; {} carries {:+.2}% from earlier seasons", team.name, team.carry)
                } else {
                    String::new()
                }
            )))
    }

    fn teams_table(&self, season: &league::Season, p: Palette) -> Div {
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
                                .child(format!("{} wins", t.wins)),
                        )
                }),
        )
    }

    fn calendar(&self, season: &league::Season, me: &str, p: Palette) -> Div {
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
                .map(|l| format!(" · practice {}", l.grade))
                .unwrap_or_default();
            let state = match r.state {
                RoundState::Pending if !r.sprint_result.is_empty() => "sprint counted".to_owned(),
                RoundState::Pending if !r.quali_result.is_empty() => "qualifying counted".to_owned(),
                RoundState::Pending if !r.practice.is_empty() => "practice counted".to_owned(),
                RoundState::Pending => String::new(),
                RoundState::Raced => format!(
                    "raced{}",
                    r.by.as_ref().map_or(String::new(), |b| format!(" ({b})"))
                ),
                RoundState::Simulated if r.auto => "simulated (not in the game)".to_owned(),
                RoundState::Simulated => "simulated".to_owned(),
                RoundState::Skipped => "left out".to_owned(),
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
                .child(div().w(px(230.)).text_size(px(13.)).child(format!(
                    "{}{}",
                    r.race,
                    if r.sprint { " · sprint" } else { "" }
                )))
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
