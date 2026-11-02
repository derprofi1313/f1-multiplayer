//! What the Career page does: apply actions, count races, create, join, start, leave and delete
//! careers.

use gpui_kit::*;
use swofty_core::league::solo;
use swofty_core::league::{
    self, Action, Phase, PointsRule, RaceReport, SessionKind, Setup, TeamChoice, Untracked, udp,
};
use swofty_core::multiplayer::rooms::{Api, ApiError};

use crate::i18n::{tr, tr_owned};
use crate::shell::Shell;

use super::{
    Refused, api_refusal, capitalized, league_refusal, refusal_message, session_text, texts, track_text, uncounted,
    value,
};

/// A refused count the page explains with its own card (the rest go to the note).
fn has_card(reason: &str) -> bool {
    matches!(reason, "wrong_track" | "duplicate" | "ask_kind")
}

/// What went wrong with a solo career, a league's refusal in the launcher's language.
fn solo_refusal(e: &solo::SoloError) -> String {
    match e {
        solo::SoloError::League(e) => league_refusal(e),
        e => e.to_string(),
    }
}

/// "The race is counted." with the warnings that came with it.
fn counted_note(what: &str, counted_in: Option<&str>, warnings: &[String]) -> String {
    let mut note = match counted_in {
        Some(career) => tr_owned("Counted in {career}: the {what} is counted.")
            .replace("{career}", career)
            .replace("{what}", what),
        None => tr_owned("The {what} is counted.").replace("{what}", what),
    };
    if !warnings.is_empty() {
        note.push(' ');
        note.push_str(&warnings.join(" · "));
    }
    note
}

impl Shell {
    /// Apply `action` to the selected solo career, on this PC. Returns the warnings in the
    /// launcher's language.
    pub(super) fn career_act_solo(&mut self, action: Action) -> Result<Vec<String>, solo::SoloError> {
        let id = self.career.selected.clone().unwrap_or_default();
        let root = self.store.root.clone();
        let (career, warnings) = solo::act(&root, &id, action)?;
        let shown = warnings.iter().map(texts::text).collect();
        self.career
            .show_solo(career, warnings, &root.join(udp::RACES_DIR));
        self.career.error = None;
        Ok(shown)
    }

    /// Apply `action` to the selected career. On a host server it is decided on the version on
    /// show (`base`): when the career moved on meanwhile, it is refused, and the page reloads.
    pub(super) fn career_act(&mut self, action: Action, cx: &mut Context<Self>) {
        self.career.confirm = None;
        if self.career.solo() {
            self.career.note = match self.career_act_solo(action) {
                Ok(w) if w.is_empty() => None,
                Ok(w) => Some(w.join(" · ")),
                Err(solo::SoloError::League(e)) => Some(league_refusal(&e)),
                Err(e) => Some(e.to_string()),
            };
            cx.notify();
            return;
        }
        if self.career.busy.is_some() {
            self.career.note = Some(tr("Still working on the last step…").into());
            cx.notify();
            return;
        }
        let Some(id) = self.career.selected.clone() else {
            return;
        };
        let Some(api) = self.career_api() else {
            self.career.note = Some(tr("Sign in to the host server first.").into());
            cx.notify();
            return;
        };
        // Race reports go through `career_count`, without a base: a race counts whenever it ends.
        let base = self.career.state.as_ref().map(|s| s.version);
        self.career.busy = Some("act");
        self.career.note = None;
        let races_dir = self.store.root.join(udp::RACES_DIR);
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let state = api.league_action_at(&id, &action, base)?;
                    let races = uncounted(state.league.as_ref(), udp::saved(&races_dir));
                    Ok::<_, ApiError>((state, races))
                })
                .await;
            this.update(cx, |s, cx| {
                s.career.busy = None;
                match result {
                    Ok((state, races)) => {
                        let c = &mut s.career;
                        if !state.warnings.is_empty() {
                            c.note = Some(texts::joined(&state.warning_texts(), " · "));
                        }
                        c.state = Some(state);
                        c.races = races;
                        c.error = None;
                    }
                    Err(e) if e.reason == "stale" => {
                        s.career.note = Some(api_refusal(&e));
                        s.career.league_at = None;
                        s.career_load(cx);
                    }
                    Err(e) => s.career.note = Some(api_refusal(&e)),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    /// Count a race the game reported in the selected career.
    pub(super) fn career_count(
        &mut self,
        report: RaceReport,
        kind: Option<SessionKind>,
        force: bool,
        cx: &mut Context<Self>,
    ) {
        self.career_count_in(report, kind, force, None, cx);
    }

    /// Count a race in the selected career; `counted_in` names the career when the race was
    /// routed there by itself, for the note.
    pub(super) fn career_count_in(
        &mut self,
        report: RaceReport,
        kind: Option<SessionKind>,
        force: bool,
        counted_in: Option<String>,
        cx: &mut Context<Self>,
    ) {
        // A race that came by itself while another step runs waits its turn.
        if self.career.busy.is_some() {
            if kind.is_none() && !force {
                if !self.career.queue.iter().any(|r| r.session == report.session) {
                    self.career.queue.push(report);
                }
            } else {
                self.career.note = Some(tr("Still working on the last step…").into());
            }
            cx.notify();
            return;
        }
        if self.career.selected.is_none() {
            return;
        }
        let what = tr(session_text(report.session_type));
        if self.career.league().is_none_or(|l| l.phase != Phase::Racing) {
            self.career.note = Some(
                tr_owned("{what} at {track} ended; it waits below until the career has a round for it.")
                    .replace("{what}", &capitalized(what))
                    .replace("{track}", &track_text(report.track)),
            );
            cx.notify();
            return;
        }
        let action = Action::Report {
            round: None,
            kind,
            force,
            report: report.clone(),
        };
        if self.career.solo() {
            self.career.refused = None;
            match self.career_act_solo(action) {
                Ok(w) => self.career.note = Some(counted_note(what, counted_in.as_deref(), &w)),
                Err(solo::SoloError::League(e)) if has_card(e.reason) => {
                    self.career.refused = Some(Refused {
                        report,
                        kind,
                        force,
                        reason: e.reason.to_owned(),
                        message: refusal_message(&e.message, texts::refusal(&e.template, &e.args)),
                    })
                }
                Err(solo::SoloError::League(e)) => {
                    self.career.note = Some(
                        tr_owned("The {what} wasn't counted: {why}")
                            .replace("{what}", what)
                            .replace("{why}", &league_refusal(&e)),
                    )
                }
                Err(e) => {
                    self.career.note = Some(
                        tr_owned("The {what} wasn't counted: {why}")
                            .replace("{what}", what)
                            .replace("{why}", &e.to_string()),
                    )
                }
            }
            cx.notify();
            return;
        }
        let Some(id) = self.career.selected.clone() else {
            return;
        };
        let Some(api) = self.career_api() else {
            // It stays queued and counts once signed in.
            if kind.is_none() && !force && !self.career.queue.iter().any(|r| r.session == report.session) {
                self.career.queue.push(report);
            }
            self.career.note =
                Some(tr("Sign in to the host server to count this race; it waits here.").into());
            cx.notify();
            return;
        };
        self.career.busy = Some("count");
        self.career.refused = None;
        let races_dir = self.store.root.join(udp::RACES_DIR);
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let r = api.league_action(&id, &action);
                    let league = r.as_ref().ok().and_then(|s| s.league.clone());
                    (r, udp::saved(&races_dir), league)
                })
                .await;
            this.update(cx, |s, cx| {
                let c = &mut s.career;
                c.busy = None;
                match result {
                    (Ok(state), saved, league) => {
                        let warnings: Vec<String> = state.warning_texts().iter().map(texts::text).collect();
                        c.note = Some(counted_note(what, counted_in.as_deref(), &warnings));
                        c.races = uncounted(league.as_ref(), saved);
                        c.state = Some(state);
                    }
                    (Err(e), saved, _) => {
                        c.races = uncounted(c.league(), saved);
                        if has_card(&e.reason) {
                            c.refused = Some(Refused {
                                report,
                                kind,
                                force,
                                message: refusal_message(&e.message, e.text.as_deref().map(texts::text)),
                                reason: e.reason,
                            });
                        } else {
                            c.note = Some(
                                tr_owned("The {what} wasn't counted: {why}")
                                    .replace("{what}", what)
                                    .replace("{why}", &api_refusal(&e)),
                            );
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
    pub(super) fn career_discard(&mut self, session: u64, cx: &mut Context<Self>) {
        let _ = udp::discard(&self.store.root.join(udp::RACES_DIR), session);
        self.career.races.retain(|r| r.session != session);
        self.career.queue.retain(|r| r.session != session);
        if self
            .career
            .refused
            .as_ref()
            .is_some_and(|r| r.report.session == session)
        {
            self.career.refused = None;
        }
        cx.notify();
    }

    /// The setup the start form describes, with the player's race number; `Err` with what to
    /// fix.
    fn career_setup(&self, cx: &App) -> Result<(Setup, Option<u8>), &'static str> {
        let number = match value(&self.career.number, cx) {
            n if n.is_empty() => None,
            n => match n.parse::<u8>() {
                Ok(n) => Some(n),
                Err(_) => return Err(tr("The race number is a number from 1 to 99.")),
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
        Ok((setup, number))
    }

    pub(super) fn career_create(&mut self, cx: &mut Context<Self>) {
        let api = self.career_api();
        let solo_career = self.career.solo_form || api.is_none();
        let name = value(&self.career.name, cx);
        if name.is_empty() {
            self.career.error = Some(tr("Give the career a name.").into());
            cx.notify();
            return;
        }
        let (setup, number) = match self.career_setup(cx) {
            Ok(x) => x,
            Err(e) => {
                self.career.error = Some(e.into());
                cx.notify();
                return;
            }
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
        // A new career has the players' names and the league's car pace in the game.
        let created = solo::create(&root, &name, &player, setup, number).map(|career| {
            let id = career.id.clone();
            solo::act(&root, &id, super::gamesync::new_career_options()).map_or(career, |(c, _)| c)
        });
        match created {
            Ok(career) => {
                let c = &mut self.career;
                c.creating = false;
                c.error = None;
                c.show_solo(career, Vec::new(), &root.join(udp::RACES_DIR));
                // The driver picked in the form (W3-2).
                if let Some(profile) = self.career.profile.action()
                    && let Err(e) = self.career_act_solo(profile)
                {
                    self.career.note = Some(solo_refusal(&e));
                }
            }
            Err(e) => self.career.error = Some(solo_refusal(&e)),
        }
        cx.notify();
    }

    /// Create a career on the host server and start it. When the start fails after the career
    /// was made, the page remembers it, to try the start again or delete it.
    pub(super) fn career_create_on_server(
        &mut self,
        api: Api,
        name: String,
        setup: Setup,
        number: Option<u8>,
        cx: &mut Context<Self>,
    ) {
        let profile = self.career.profile.action();
        self.career_start_or_join(
            move || {
                let career = api
                    .create_career(&name, league::MODE, serde_json::Value::Null)
                    .map_err(|e| (api_refusal(&e), None))?;
                api.league_action(&career.id, &Action::Start { setup, number })
                    .map_err(|e| (api_refusal(&e), Some(career.id.clone())))?;
                // The driver picked in the form (W3-2); a refusal leaves it to the driver card.
                if let Some(profile) = &profile {
                    api.league_action(&career.id, profile).ok();
                }
                // A new career has the players' names and the league's car pace in the game; an
                // older host server that refuses the options leaves them off.
                let _ = api.league_action(&career.id, &super::gamesync::new_career_options());
                Ok(career.id)
            },
            cx,
        );
    }

    /// Try the Start of a career that was created but didn't start, with the start form's setup.
    pub(super) fn career_retry_start(&mut self, id: String, cx: &mut Context<Self>) {
        let Some(api) = self.career_api() else { return };
        let profile = self.career.profile.action();
        let (setup, number) = match self.career_setup(cx) {
            Ok(x) => x,
            Err(e) => {
                self.career.error = Some(e.into());
                cx.notify();
                return;
            }
        };
        self.career_start_or_join(
            move || {
                api.league_action(&id, &Action::Start { setup, number })
                    .map_err(|e| (api_refusal(&e), Some(id.clone())))?;
                // The driver picked in the form (W3-2); a refusal leaves it to the driver card.
                if let Some(profile) = &profile {
                    api.league_action(&id, profile).ok();
                }
                let _ = api.league_action(&id, &super::gamesync::new_career_options());
                Ok(id)
            },
            cx,
        );
    }

    pub(super) fn career_join(&mut self, cx: &mut Context<Self>) {
        let Some(api) = self.career_api() else { return };
        let code = value(&self.career.code, cx);
        if code.is_empty() {
            self.career.error = Some(tr("Type the career code your friend sent you.").into());
            cx.notify();
            return;
        }
        let number = value(&self.career.number, cx).parse::<u8>().ok();
        let profile = self.career.profile.action();
        self.career_start_or_join(
            move || {
                let career = api.join_career(&code).map_err(|e| (api_refusal(&e), None))?;
                if career.mode != league::MODE {
                    return Err((
                        tr("That code belongs to a shared career that isn't a league career.").into(),
                        None,
                    ));
                }
                api.league_action(&career.id, &Action::Enter { number })
                    .map_err(|e| (api_refusal(&e), None))?;
                // The driver picked in the form (W3-2); a refusal leaves it to the driver card.
                if let Some(profile) = &profile {
                    api.league_action(&career.id, profile).ok();
                }
                Ok(career.id)
            },
            cx,
        );
    }

    /// Run `work` (create, start or join a career) in the background, then show the career it
    /// gives. Its error names a career that was created but didn't start, if any.
    pub(super) fn career_start_or_join(
        &mut self,
        work: impl FnOnce() -> Result<String, (String, Option<String>)> + Send + 'static,
        cx: &mut Context<Self>,
    ) {
        if self.career.busy.is_some() {
            self.career.note = Some(tr("Still working on the last step…").into());
            cx.notify();
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
                        s.career.unstarted = None;
                        s.career.creating = false;
                        s.career.state = None;
                        s.career.careers_at = None;
                        s.career.league_at = None;
                        s.career_load(cx);
                    }
                    Err((e, unstarted)) => {
                        if unstarted.is_some() {
                            s.career.unstarted = unstarted;
                            s.career.careers_at = None;
                        }
                        s.career.error = Some(e);
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    /// Run a membership call on the host server (leave, remove, delete) in the background; `done`
    /// updates the page with its answer.
    fn career_member_call<T: Send + 'static>(
        &mut self,
        work: impl FnOnce(Api) -> Result<T, ApiError> + Send + 'static,
        done: impl FnOnce(&mut Shell, T, &mut Context<Shell>) + 'static,
        cx: &mut Context<Self>,
    ) {
        self.career.confirm = None;
        if self.career.busy.is_some() {
            self.career.note = Some(tr("Still working on the last step…").into());
            cx.notify();
            return;
        }
        let Some(api) = self.career_api() else { return };
        self.career.busy = Some("member");
        cx.spawn(async move |this, cx| {
            let result = cx.background_executor().spawn(async move { work(api) }).await;
            this.update(cx, |s, cx| {
                s.career.busy = None;
                match result {
                    Ok(t) => done(s, t, cx),
                    Err(e) => s.career.note = Some(api_refusal(&e)),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    /// Forget the selected career on this page and load the list again (after leaving or
    /// deleting it).
    fn career_gone(&mut self, cx: &mut Context<Self>) {
        let c = &mut self.career;
        c.selected = None;
        c.state = None;
        c.unstarted = None;
        c.refused = None;
        c.careers_at = None;
        c.league_at = None;
        self.career_load(cx);
    }

    /// Leave the selected server career: its seat stays, driven by the game's AI.
    pub(super) fn career_leave(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.career.selected.clone().filter(|id| !solo::is_solo(id)) else {
            return;
        };
        self.career_member_call(
            move |api| api.leave_career(&id),
            |s, (), cx| {
                s.career.note =
                    Some(tr("You left the career. Your seat stays, driven by the game's AI.").into());
                s.career_gone(cx);
            },
            cx,
        );
    }

    /// Remove the player `user` from the selected server career (the owner): before the season
    /// they go, later their seat stays, driven by the game's AI.
    pub(super) fn career_remove(&mut self, user: String, name: String, cx: &mut Context<Self>) {
        let Some(id) = self.career.selected.clone().filter(|id| !solo::is_solo(id)) else {
            return;
        };
        let races_dir = self.store.root.join(udp::RACES_DIR);
        self.career_member_call(
            move |api| {
                let state = api.remove_member(&id, &user)?;
                let races = uncounted(state.league.as_ref(), udp::saved(&races_dir));
                Ok((state, races))
            },
            move |s, (state, races), cx| {
                let c = &mut s.career;
                c.note = Some(tr_owned("{name} is out of the career.").replace("{name}", &name));
                c.state = Some(state);
                c.races = races;
                c.careers_at = None;
                s.career_load(cx);
            },
            cx,
        );
    }

    /// Delete a server career (the owner): one that didn't start, or the selected one.
    pub(super) fn career_delete(&mut self, id: String, cx: &mut Context<Self>) {
        self.career_member_call(
            move |api| api.delete_career(&id),
            |s, (), cx| {
                s.career.note = Some(tr("The career is deleted.").into());
                s.career_gone(cx);
            },
            cx,
        );
    }

    pub(super) fn career_enable_telemetry(&mut self, cx: &mut Context<Self>) {
        if !self.processes.is_empty() {
            self.career.note = Some(tr("Close F1 25 first: it writes its settings when it quits.").into());
            cx.notify();
            return;
        }
        let Some(path) = udp::settings_path() else { return };
        match udp::enable(&path) {
            Ok(s) => {
                self.career.telemetry = Some(s);
                self.career.listen_error = None;
                self.career.note =
                    Some(tr("The game's UDP telemetry is on. Race results arrive by themselves now.").into());
            }
            Err(e) => {
                self.career.note = Some(
                    tr_owned("Couldn't turn the telemetry on: {error}").replace("{error}", &e.to_string()),
                )
            }
        }
        cx.notify();
    }
}
