//! The Career page: the league career 2016-2026 (see `swofty_core::league`), driven alone (a
//! solo career kept in the launcher's data folder) or with friends on a host server. While there
//! is a league career, the launcher listens to the game's UDP telemetry and counts each
//! qualifying and race in the selected career as it ends.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;
use swofty_core::league::solo::{self, SoloCareer};
use swofty_core::league::{self, Focus, League, Phase, RaceReport, SessionKind, UndoPreview, udp};
use swofty_core::multiplayer::rooms::{Api, ApiError, LeagueState, SharedCareer};

use crate::i18n::{tr, tr_owned};
use crate::shell::{Page, Shell};

mod actions;
mod banner;
mod development;
mod driver;
mod gamesync;
mod inbox;
mod next;
mod offers;
mod page;
mod seats;
mod start;
mod tabs;
pub(crate) mod texts;
#[cfg(test)]
mod tests_round;
mod weekend;

const CAREERS_EVERY: Duration = Duration::from_secs(20);
const LEAGUE_EVERY: Duration = Duration::from_secs(5);
const NOTE_FOR: Duration = Duration::from_secs(20);
/// The other league careers on the host server whose leagues are kept, to tell which career a
/// race the game reported belongs to.
const OTHERS_MAX: usize = 5;
/// A session counted from another PC this close (in seconds) to a report looks like the same one.
const DUPLICATE_WINDOW_S: u64 = 120;

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
    /// The player's mails, department events and media questions.
    Inbox,
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
    /// The race a count was refused for, with why, to offer what to do about it.
    refused: Option<Refused>,
    /// Races the game reported while listening that wait to be counted by themselves, oldest
    /// first (see [`route_report`]).
    queue: Vec<RaceReport>,
    /// The sessions the listener reported that were queued already (or were there before it
    /// started listening).
    seen: HashSet<u64>,
    /// The leagues of the other league careers on the host server (at most [`OTHERS_MAX`]), to
    /// route a race to the career it belongs to.
    others: Vec<(String, LeagueState)>,
    others_at: Option<Instant>,
    /// The host server had league careers at the last load; kept while signed out, so a race
    /// waits for the sign-in instead of being given up.
    had_server_careers: bool,
    /// A career on the host server whose Start failed after it was created, to try again or
    /// delete it.
    unstarted: Option<String>,
    /// A step that waits for a second click: `undo`, `leave`, `delete` or `remove:<user id>`.
    confirm: Option<String>,
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
    /// The driver picked in the start form, and the terms asked in contract talks (W3-2).
    profile: driver::ProfileForm,
    talks: offers::Talks,
    /// The game sync on this PC (W3-3).
    sync: gamesync::GameSync,
    /// The owner's rules sheet editor (W3-3).
    rules: weekend::RulesEditor,
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
                Ok("inbox") => Tab::Inbox,
                Ok("driver") => Tab::Driver,
                Ok("development") => Tab::Development,
                _ => Tab::Drivers,
            },
            creating: false,
            note_since: None,
            races: Vec::new(),
            refused: None,
            queue: Vec::new(),
            seen: HashSet::new(),
            others: Vec::new(),
            others_at: None,
            had_server_careers: false,
            unstarted: None,
            confirm: None,
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
            profile: driver::ProfileForm::default(),
            talks: offers::Talks::default(),
            sync: gamesync::GameSync::default(),
            rules: weekend::RulesEditor::default(),
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

    /// The name of the career `id`, server or solo.
    fn name_of(&self, id: &str) -> String {
        self.careers
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.name.clone())
            .or_else(|| self.solos.iter().find(|c| c.id == id).map(|c| c.name.clone()))
            .unwrap_or_else(|| id.to_owned())
    }

    /// This player in the selected career: the solo player, or the host server's user id.
    fn me_id(&self) -> String {
        if self.solo() {
            solo::ME.to_owned()
        } else {
            self.me.clone().unwrap_or_default()
        }
    }

    /// This player owns the selected career (a solo career is always their own).
    fn owner(&self) -> bool {
        self.solo()
            || self
                .me
                .as_ref()
                .is_some_and(|me| self.career().is_some_and(|c| &c.owner == me))
    }

    /// Show a solo career that was read or changed.
    fn show_solo(&mut self, career: SoloCareer, warnings: Vec<league::Warning>, races_dir: &std::path::Path) {
        self.races = uncounted(Some(&career.league), udp::saved(races_dir));
        self.state = Some(LeagueState {
            league: Some(career.league.clone()),
            version: career.version,
            warnings: league::texts::english(&warnings),
            warnings_t: warnings,
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

/// Races not counted in `league` yet, oldest first. A report of a session another player's PC
/// counted (the same classification of the same session type) counts as counted too.
fn uncounted(league: Option<&League>, saved: Vec<RaceReport>) -> Vec<RaceReport> {
    let rounds: Vec<&league::Round> = league
        .into_iter()
        .flat_map(|l| &l.seasons)
        .flat_map(|s| &s.rounds)
        .collect();
    let counted: Vec<u64> = rounds
        .iter()
        .flat_map(|r| {
            r.sessions
                .iter()
                .copied()
                .chain(r.practice.iter().map(|l| l.session))
                .chain(r.counted.iter().map(|c| c.session))
        })
        .filter(|s| *s != 0)
        .collect();
    let prints: Vec<(u8, u64)> = rounds
        .iter()
        .flat_map(|r| &r.counted)
        .filter(|c| c.kind != SessionKind::Practice && c.fingerprint != 0)
        .map(|c| (c.session_type, c.fingerprint))
        .collect();
    saved
        .into_iter()
        .filter(|r| !counted.contains(&r.session))
        .filter(|r| {
            league::session_kind(r.session_type) == Some(SessionKind::Practice)
                || !prints.contains(&(r.session_type, league::fingerprint(r)))
        })
        .collect()
}

fn track_text(track: Option<u8>) -> String {
    track
        .and_then(league::track_name)
        .map_or_else(|| tr("an unknown track").to_owned(), str::to_owned)
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

/// "race", "sprint", "qualifying" or "practice", in the launcher's language.
fn kind_text(kind: SessionKind) -> &'static str {
    match kind {
        SessionKind::Race => tr("race"),
        SessionKind::Sprint => tr("sprint"),
        SessionKind::Qualifying => tr("qualifying"),
        SessionKind::Practice => tr("practice"),
    }
}

/// "Ben", "Ben, Cleo and Dan".
fn names_text(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => tr_owned("{names} and {last}")
            .replace("{names}", &rest.join(", "))
            .replace("{last}", last),
    }
}

/// A race the selected career refused to count, and why.
#[derive(Clone)]
pub(super) struct Refused {
    report: RaceReport,
    /// The kind and force the count was asked with.
    kind: Option<SessionKind>,
    force: bool,
    reason: String,
    message: String,
}

/// Where a race the game reported is counted (see [`route_report`]).
#[derive(Debug, PartialEq, Eq)]
enum Route {
    /// In this career.
    Count(String),
    /// It fits each of these careers: the player picks one.
    Ask(Vec<String>),
    /// No career has a round for it now.
    Hold,
}

/// The career a race the game reported belongs to: among `candidates` (career id and league),
/// the careers racing whose next round is at the race's track (every racing career when the
/// game didn't say the track). The selected career wins when it fits; otherwise exactly one
/// fitting career is counted in, and several are asked about.
fn route_report(report: &RaceReport, candidates: &[(String, &League)], selected: Option<&str>) -> Route {
    let fits: Vec<&str> = candidates
        .iter()
        .filter(|(_, l)| l.phase == Phase::Racing)
        .filter(|(_, l)| {
            let next = l.season().next_round().map(|r| r.track);
            match (report.track, next) {
                (_, None) => false,
                (None, Some(_)) | (Some(_), Some(None)) => true,
                (Some(t), Some(Some(want))) => t == want,
            }
        })
        .map(|(id, _)| id.as_str())
        .collect();
    if let Some(sel) = selected
        && fits.contains(&sel)
    {
        return Route::Count(sel.to_owned());
    }
    match fits.as_slice() {
        [] => Route::Hold,
        [one] => Route::Count((*one).to_owned()),
        many => Route::Ask(many.iter().map(|s| (*s).to_owned()).collect()),
    }
}

/// Queue the reports in `saved` the listener hasn't queued yet (`seen`), keeping the queue
/// oldest first.
fn enqueue(queue: &mut Vec<RaceReport>, seen: &mut HashSet<u64>, saved: Vec<RaceReport>) {
    for r in saved {
        if seen.insert(r.session) {
            queue.push(r);
        }
    }
    queue.sort_by_key(|r| (r.at, r.session));
}

/// The queued report to count next: the oldest one.
fn next_to_count(queue: &[RaceReport]) -> Option<usize> {
    queue
        .iter()
        .enumerate()
        .min_by_key(|(_, r)| (r.at, r.session))
        .map(|(i, _)| i)
}

/// A session of the next round or the round before that another PC counted moments before
/// `report` (the same session type, within [`DUPLICATE_WINDOW_S`]): its round, kind and the
/// name of who counted it. Practice never is one: each player's practice counts.
fn looks_counted(league: &League, report: &RaceReport) -> Option<(u8, SessionKind, String)> {
    if report.at == 0 || league.phase == Phase::Setup {
        return None;
    }
    let season = league.season();
    let next = season
        .next_round()
        .map_or(season.rounds.len() as u8 + 1, |r| r.round);
    season
        .rounds
        .iter()
        .filter(|r| r.round == next || r.round + 1 == next)
        .flat_map(|r| r.counted.iter().map(move |c| (r.round, c)))
        .filter(|(_, c)| c.kind != SessionKind::Practice && c.at != 0)
        .find(|(_, c)| {
            c.session_type == report.session_type && c.at.abs_diff(report.at) <= DUPLICATE_WINDOW_S
        })
        .map(|(round, c)| {
            let by = league
                .player(&c.by)
                .map_or_else(|| tr("another player").to_owned(), |p| p.name.clone());
            (round, c.kind, by)
        })
}

/// "This looks like round 4's race, counted by Ben."
fn duplicate_text(round: u8, kind: SessionKind, by: &str) -> String {
    tr_owned("This looks like round {round}'s {kind}, counted by {name}.")
        .replace("{round}", &round.to_string())
        .replace("{kind}", kind_text(kind))
        .replace("{name}", by)
}

/// What taking back the round in `preview` takes back, line by line. `me` (this player's name)
/// is left out of the players whose sessions go too.
fn undo_lines(preview: &UndoPreview, me: &str) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    if preview.race_counted {
        parts.push(tr("the race").to_owned());
    }
    if preview.sprint {
        parts.push(tr("the sprint").to_owned());
    }
    if preview.quali {
        parts.push(tr("qualifying").to_owned());
    }
    if !preview.practice_of.is_empty() {
        parts.push(tr_owned("the practice of {names}").replace("{names}", &names_text(&preview.practice_of)));
    }
    let mut lines = vec![if parts.is_empty() {
        tr_owned("Round {round}, the {race}, goes back to the start.")
            .replace("{round}", &preview.round.to_string())
            .replace("{race}", &preview.race)
    } else {
        tr_owned("Round {round}, the {race}: {parts} go.")
            .replace("{round}", &preview.round.to_string())
            .replace("{race}", &preview.race)
            .replace("{parts}", &names_text(&parts))
    }];
    if preview.auto_after > 0 {
        lines.push(
            tr_owned("The {n} simulated round(s) after it are counted again.")
                .replace("{n}", &preview.auto_after.to_string()),
        );
    }
    let others: Vec<String> = preview
        .by_others
        .iter()
        .filter(|n| n.as_str() != me)
        .cloned()
        .collect();
    if !others.is_empty() {
        lines.push(tr_owned("Sessions {names} counted go too.").replace("{names}", &names_text(&others)));
    }
    if preview.season_end {
        lines.push(tr("The season opens again: contracts signed for next season are cleared.").to_owned());
    }
    lines
}

/// The refusal reasons of the league, the host server and its sign-in that the launcher explains
/// in its own words (see [`reason_text`]).
#[cfg(test)]
pub(crate) const REASONS: &[&str] = &[
    "duplicate",
    "ask_kind",
    "not_next",
    "stale",
    "vote_open",
    "no_vote",
    "not_voter",
    "owner",
    "left",
    "not_yet",
    "name_claimed",
    "secret_required",
    "too_many",
    "too_big",
    "bad_report",
    "rate_limited",
];

/// The English text for a refusal reason (a `tr` key; `{seconds}` stands for the wait of
/// `rate_limited`), or `None` for a reason whose message is shown as it is.
pub(crate) fn reason_text(reason: &str) -> Option<&'static str> {
    Some(match reason {
        "duplicate" => "This looks like a session another player counted moments ago.",
        "ask_kind" => "Sprint or Grand Prix? The game didn't say which race of the weekend this was.",
        "not_next" => "Only the next round can be counted, simulated, left out or practised.",
        "stale" => "The career changed meanwhile; check and try again.",
        "vote_open" => "Another player's proposal is open; agree to it or decline it first.",
        "no_vote" => "There is no open proposal any more.",
        "not_voter" => "You aren't asked in this vote.",
        "owner" => "Only the career's owner can do that.",
        "left" => "You left this career; join it again with its code first.",
        "not_yet" => "The host server doesn't know this yet; the host updates F1 Multiplayer.",
        "name_claimed" => {
            "Another PC claimed this driver name on the host server. Sign in from the PC you used before, or ask \
             the host to release the name."
        }
        "secret_required" => {
            "This driver name is claimed on the host server; update F1 Multiplayer to sign in."
        }
        "too_many" => "You own too many careers on this host server; delete one first.",
        "too_big" => "That is too much data for the host server.",
        "bad_report" => "The host server refused the race report: it has more cars or laps than a race has.",
        "rate_limited" => "Too many tries; wait {seconds} seconds and try again.",
        _ => return None,
    })
}

/// What to tell the player about a refused request, in the launcher's language: the launcher's
/// own words for a reason it knows, else the refusal's template (`translated`), else the
/// message that came with it (from an older host server).
fn refusal_text(reason: &str, message: &str, translated: Option<String>, retry_after: Option<u64>) -> String {
    match reason_text(reason) {
        Some(text) => tr(text).replace("{seconds}", &retry_after.unwrap_or(10).to_string()),
        None => translated.unwrap_or_else(|| message.to_owned()),
    }
}

/// [`refusal_text`] for a refused call to the host server.
fn api_refusal(e: &ApiError) -> String {
    refusal_text(&e.reason, &e.message, e.text.as_deref().map(texts::text), e.retry_after)
}

/// [`refusal_text`] for a league's refusal on this PC (a solo career).
fn league_refusal(e: &league::LeagueError) -> String {
    refusal_text(e.reason, &e.message, texts::refusal(&e.template, &e.args), None)
}

/// A refusal's own text in the launcher's language (not the launcher's words for its reason).
fn refusal_message(message: &str, translated: Option<String>) -> String {
    translated.unwrap_or_else(|| message.to_owned())
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
            c.others.clear();
            c.others_at = None;
            c.unstarted = None;
            c.confirm = None;
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
        self.career_sync_tick(cx);
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
            let dir = self.store.root.join(udp::RACES_DIR);
            match udp::Listener::start(settings.listen_addr(), dir.clone()) {
                Ok(l) => {
                    c.listener = Some(l);
                    // What was reported before stays below to count by hand; only new races
                    // count by themselves.
                    c.seen = udp::saved(&dir).into_iter().map(|r| r.session).collect();
                }
                Err(e) => c.listen_error = Some(
                    tr_owned(
                        "Couldn't listen for race results on {addr}: {error}. Another telemetry tool may use \
                             that port.",
                    )
                    .replace("{addr}", &settings.listen_addr().to_string())
                    .replace("{error}", &e.to_string()),
                ),
            }
            cx.notify();
        }
        let Some(status) = self.career.listener.as_ref().map(udp::Listener::status) else {
            return;
        };
        if status.races > self.career.races_seen {
            self.career.races_seen = status.races;
            let saved = udp::saved(&self.store.root.join(udp::RACES_DIR));
            let c = &mut self.career;
            enqueue(&mut c.queue, &mut c.seen, saved);
        }
        self.career_drain(cx);
    }

    /// Count the oldest race waiting in the queue, in the career it belongs to: one at a time,
    /// while nothing else runs.
    fn career_drain(&mut self, cx: &mut Context<Self>) {
        let c = &self.career;
        if c.busy.is_some() || c.queue.is_empty() {
            return;
        }
        let signed_in = self.career_api().is_some();
        let mut candidates: Vec<(String, &League)> =
            c.solos.iter().map(|s| (s.id.clone(), &s.league)).collect();
        if let (Some(id), Some(l)) = (c.selected.as_ref().filter(|id| !solo::is_solo(id)), c.league()) {
            candidates.push((id.clone(), l));
        }
        for (id, state) in &c.others {
            if c.selected.as_ref() != Some(id)
                && let Some(l) = &state.league
            {
                candidates.push((id.clone(), l));
            }
        }
        let mut pick = None;
        let mut ordered: Vec<usize> = (0..c.queue.len()).collect();
        ordered.sort_by_key(|&i| (c.queue[i].at, c.queue[i].session));
        for i in ordered {
            let route = route_report(&c.queue[i], &candidates, c.selected.as_deref());
            // A race that may belong to a server career waits for the sign-in.
            if route == Route::Hold && !signed_in && c.had_server_careers {
                continue;
            }
            pick = Some((i, route));
            break;
        }
        drop(candidates);
        let Some((i, route)) = pick else {
            let r = &self.career.queue[next_to_count(&self.career.queue).unwrap_or(0)];
            let note =
                tr_owned("{what} at {track} ended. Sign in to the host server to count it; it waits here.")
                    .replace("{what}", &capitalized(tr(session_text(r.session_type))))
                    .replace("{track}", &track_text(r.track));
            if self.career.note.as_ref() != Some(&note) {
                self.career.note = Some(note);
                cx.notify();
            }
            return;
        };
        let report = self.career.queue.remove(i);
        let what = capitalized(tr(session_text(report.session_type)));
        match route {
            Route::Count(id) => {
                let routed = self.career.selected.as_deref() != Some(id.as_str());
                if routed && !self.career_select_for_count(&id) {
                    return;
                }
                let name = routed.then(|| self.career.career_name());
                self.career_count_in(report, None, false, name, cx);
            }
            Route::Ask(ids) => {
                let names: Vec<String> = ids.iter().map(|id| self.career.name_of(id)).collect();
                self.career.note = Some(
                    tr_owned("{what} at {track} fits {careers}: pick the career above and count it below.")
                        .replace("{what}", &what)
                        .replace("{track}", &track_text(report.track))
                        .replace("{careers}", &names_text(&names)),
                );
                cx.notify();
            }
            Route::Hold => {
                self.career.note = Some(
                    tr_owned("{what} at {track} ended; it waits below until a career has a round for it.")
                        .replace("{what}", &what)
                        .replace("{track}", &track_text(report.track)),
                );
                cx.notify();
            }
        }
    }

    /// Show the career `id` to count a race in it; `false` when it can't be shown.
    fn career_select_for_count(&mut self, id: &str) -> bool {
        let root = self.store.root.clone();
        let c = &mut self.career;
        if solo::is_solo(id) {
            return match solo::load(&root, id) {
                Ok(s) => {
                    c.show_solo(s, Vec::new(), &root.join(udp::RACES_DIR));
                    c.refused = None;
                    c.confirm = None;
                    true
                }
                Err(_) => false,
            };
        }
        let Some(state) = c.others.iter().find(|(x, _)| x == id).map(|(_, s)| s.clone()) else {
            return false;
        };
        c.races = uncounted(state.league.as_ref(), udp::saved(&root.join(udp::RACES_DIR)));
        c.selected = Some(id.to_owned());
        c.state = Some(state);
        c.creating = false;
        c.refused = None;
        c.confirm = None;
        c.league_at = None;
        true
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
        // The other careers' leagues are needed to route races, so only while listening.
        let others_due = self.career.listener.is_some()
            && self
                .career
                .others_at
                .is_none_or(|at| at.elapsed() >= CAREERS_EVERY);
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
                    let others = others_due.then(|| {
                        let mut rest: Vec<&SharedCareer> = careers
                            .iter()
                            .filter(|c| Some(&c.id) != pick.as_ref() && c.version > 0)
                            .collect();
                        rest.sort_by_key(|c| std::cmp::Reverse(c.updated_at));
                        rest.into_iter()
                            .take(OTHERS_MAX)
                            .filter_map(|c| Some((c.id.clone(), api.league(&c.id).ok()?)))
                            .collect::<Vec<_>>()
                    });
                    Ok::<_, String>((me, careers, pick, state, races, others))
                })
                .await;
            this.update(cx, |s, cx| {
                let c = &mut s.career;
                c.polling = false;
                let now = Instant::now();
                c.careers_at = Some(now);
                c.league_at = Some(now);
                match result {
                    Ok((me, careers, pick, state, races, others)) => {
                        if me.is_some() {
                            c.me = me;
                        }
                        if let Some(others) = others {
                            c.others = others;
                            c.others_at = Some(now);
                        }
                        c.had_server_careers = !careers.is_empty();
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
}
