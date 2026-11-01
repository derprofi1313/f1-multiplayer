//! What a room join looks like to the player: one step with a percentage, built from the session's state and
//! the game's own report, and each member's stage as the game reports it.

use serde::Serialize;

use super::game::GameStatus;
use super::session::{Phase, SessionState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberStage {
    Starting,
    Loading,
    InLobby,
}

impl MemberStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Starting => "starting",
            Self::Loading => "loading",
            Self::InLobby => "lobby",
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Self::Starting => "Starting the game",
            Self::Loading => "Loading",
            Self::InLobby => "In the lobby",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "starting" => Some(Self::Starting),
            "loading" => Some(Self::Loading),
            "lobby" => Some(Self::InLobby),
            _ => None,
        }
    }
}

/// Where this player's own game is: not answering yet, answering but outside the game's lobby, or in it.
pub fn own_stage(game: Option<&GameStatus>) -> MemberStage {
    match game {
        None => MemberStage::Starting,
        Some(s) if s.in_session => MemberStage::InLobby,
        Some(_) => MemberStage::Loading,
    }
}

/// Another member's stage: the server's ready flag says they are in the game's lobby, their announced
/// stage says the rest. `None` for a launcher that announces none.
pub fn member_stage(ready: bool, announced: Option<&str>) -> Option<MemberStage> {
    if ready {
        return Some(MemberStage::InLobby);
    }
    announced
        .and_then(MemberStage::parse)
        .filter(|s| *s != MemberStage::InLobby)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    SigningIn,
    Joining,
    Connecting,
    StartingGame,
    Loading,
    InLobby,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Progress {
    pub step: Step,
    pub label: String,
    pub percent: u8,
}

const SIGNING_IN: u8 = 4;
const JOINING: u8 = 12;
const CONNECTING: u8 = 22;
const STARTING: u8 = 30;
const LOADING_FROM: u8 = 40;
const LOADING_SPAN: u8 = 55;

fn connecting(step: &str, host: bool) -> Progress {
    let (kind, label, percent) = if step.contains("signing in") {
        (Step::SigningIn, "Signing in", SIGNING_IN)
    } else if step.contains("creating the room") || step.contains("joining ") || step.contains("earlier room")
    {
        (
            Step::Joining,
            if host {
                "Opening the room"
            } else {
                "Joining the room"
            },
            JOINING,
        )
    } else {
        (Step::Connecting, "Connecting to the other players", CONNECTING)
    };
    Progress {
        step: kind,
        label: label.into(),
        percent,
    }
}

/// The one step the room page shows, from the session's phase and, once the game runs, the game's cover.
pub fn progress(state: &SessionState) -> Progress {
    let game = state.game.as_ref();
    match &state.phase {
        Phase::Connecting(step) => connecting(step, state.host),
        Phase::InLobby | Phase::Racing => Progress {
            step: Step::InLobby,
            label: "In the lobby".into(),
            percent: 100,
        },
        Phase::WaitingForGame(_) | Phase::Ended(_) | Phase::Failed(_) => match game {
            Some(g) if g.in_session => Progress {
                step: Step::InLobby,
                label: "In the lobby".into(),
                percent: 100,
            },
            Some(g) => {
                let (label, cover) = match &g.cover {
                    Some(c) => (c.step.clone(), u32::from(c.percent.min(100))),
                    None => ("Loading F1 25".to_owned(), 0),
                };
                let percent = u32::from(LOADING_FROM) + cover * u32::from(LOADING_SPAN) / 100;
                Progress {
                    step: Step::Loading,
                    label,
                    percent: u8::try_from(percent).unwrap_or(LOADING_FROM + LOADING_SPAN),
                }
            }
            None if state.members.is_empty() => connecting("", state.host),
            None => Progress {
                step: Step::StartingGame,
                label: "Starting F1 25".into(),
                percent: STARTING,
            },
        },
    }
}

/// Something the player has to act on, as one plain sentence and the one action that goes with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub text: String,
    pub retry: bool,
}

/// What stopped the join, or `None` while it goes on. Retry is offered when driving the game again can help;
/// a room that ended or failed leaves only Leave.
pub fn problem(state: &SessionState) -> Option<Problem> {
    let sentence = |text: &str, retry: bool| {
        Some(Problem {
            text: text.to_owned(),
            retry,
        })
    };
    match &state.phase {
        Phase::Ended(_) => None,
        Phase::Failed(_) => sentence("The room stopped unexpectedly.", false),
        Phase::WaitingForGame(_) if !state.mismatches.is_empty() => {
            let names: Vec<&str> = state.mismatches.iter().filter_map(|m| m.name()).collect();
            let text = match names.as_slice() {
                [] => "Your setup doesn't match the host's.".to_owned(),
                [one] => format!("{one} doesn't match the host's."),
                [first, rest @ ..] => format!("{first} and {} more don't match the host's.", rest.len()),
            };
            Some(Problem { text, retry: false })
        }
        Phase::WaitingForGame(why) => {
            let has = |s: &str| why.contains(s);
            if let Some(reason) = why.split_once("own room start failed: ").map(|(_, r)| r) {
                let reason = reason
                    .split(';')
                    .next()
                    .unwrap_or(reason)
                    .trim()
                    .trim_end_matches('.');
                sentence(&format!("F1 25 couldn't join the room: {reason}."), true)
            } else if has("didn't host") || has("didn't join") {
                sentence("F1 25 couldn't open the lobby.", true)
            } else if has("left the LAN session") {
                sentence("F1 25 left the lobby.", true)
            } else if has("room settings") {
                sentence("F1 25 didn't take the room's settings.", true)
            } else if has("didn't start for the room") {
                sentence("F1 25 didn't start.", true)
            } else if has("restart F1 25") {
                sentence("F1 25 has to restart for this session.", true)
            } else {
                None
            }
        }
        Phase::Connecting(_) | Phase::InLobby | Phase::Racing => state.last_error.as_ref().map(|_| Problem {
            text: "Having trouble reaching the other players.".into(),
            retry: true,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multiplayer::game::{Cover, Role, Screen};
    use crate::multiplayer::session::MemberState;

    fn game(in_session: bool, cover: Option<(&str, u8)>) -> GameStatus {
        GameStatus {
            screen: if in_session {
                Screen::LanLobby
            } else {
                Screen::Other
            },
            in_session,
            role: Role::None,
            configured: true,
            self_member: 2,
            base: 50000,
            room_failed: None,
            cover: cover.map(|(step, percent)| Cover {
                step: step.into(),
                percent,
            }),
        }
    }

    fn state(phase: Phase, game: Option<GameStatus>) -> SessionState {
        let mut s = SessionState::new(false);
        s.phase = phase;
        s.game = game;
        s
    }

    fn member() -> MemberState {
        MemberState {
            id: "1".into(),
            name: "Ana".into(),
            avatar: None,
            number: 1,
            ip: "10.77.0.1".into(),
            host: true,
            me: false,
            connected: true,
            fingerprint: crate::multiplayer::session::PrintMatch::Matches,
            relay: crate::multiplayer::session::RelayStatus::Ready,
            stage: None,
            traffic: None,
        }
    }

    #[test]
    fn the_session_steps_come_before_the_game() {
        let at = |step: &str| progress(&state(Phase::Connecting(step.into()), None));
        assert_eq!(at("signing in").step, Step::SigningIn);
        assert_eq!(at("joining F1-ABC123").step, Step::Joining);
        assert_eq!(at("connecting to the relay").step, Step::Connecting);
        assert!(at("signing in").percent < at("joining F1-ABC123").percent);
        assert!(at("joining F1-ABC123").percent < at("connecting to the relay").percent);
    }

    #[test]
    fn a_game_that_has_not_answered_is_starting() {
        let mut s = state(Phase::WaitingForGame("starting F1 25 for the room".into()), None);
        s.members.push(member());
        let p = progress(&s);
        assert_eq!((p.step, p.label.as_str()), (Step::StartingGame, "Starting F1 25"));
    }

    #[test]
    fn the_covers_step_and_percent_drive_the_loading_bar() {
        let loading = |pct: u8| {
            progress(&state(
                Phase::WaitingForGame("F1 25 is starting".into()),
                Some(game(false, Some(("Joining the lobby", pct)))),
            ))
        };
        let (a, b) = (loading(0), loading(90));
        assert_eq!((a.step, a.label.as_str()), (Step::Loading, "Joining the lobby"));
        assert!(a.percent >= LOADING_FROM && b.percent > a.percent && b.percent < 100);
    }

    #[test]
    fn only_the_games_lobby_completes_the_bar() {
        let waiting = progress(&state(
            Phase::InLobby,
            Some(game(false, Some(("Opening the lobby", 97)))),
        ));
        assert_eq!(waiting.step, Step::InLobby);
        let lobby = progress(&state(Phase::WaitingForGame("x".into()), Some(game(true, None))));
        assert_eq!((lobby.step, lobby.percent), (Step::InLobby, 100));
    }

    #[test]
    fn members_are_in_the_lobby_only_when_the_game_says_so() {
        assert_eq!(own_stage(None), MemberStage::Starting);
        assert_eq!(own_stage(Some(&game(false, None))), MemberStage::Loading);
        assert_eq!(own_stage(Some(&game(true, None))), MemberStage::InLobby);
        assert_eq!(member_stage(true, Some("loading")), Some(MemberStage::InLobby));
        assert_eq!(member_stage(false, Some("loading")), Some(MemberStage::Loading));
        assert_eq!(member_stage(false, Some("lobby")), None);
        assert_eq!(member_stage(false, None), None);
    }

    #[test]
    fn a_refused_game_is_one_sentence_with_retry_and_a_wrapper_error_is_not_shown() {
        let p = problem(&state(
            Phase::WaitingForGame(
                "F1 25 didn't start for the room: the wrapper exited (exit code: 1) before the game started; retry or leave the room"
                    .into(),
            ),
            None,
        ))
        .expect("a problem");
        assert_eq!(p.text, "F1 25 didn't start.");
        assert!(p.retry);
        assert!(!p.text.contains("wrapper"));
    }

    #[test]
    fn a_running_join_has_no_problem_and_a_dead_room_only_leaves() {
        assert_eq!(
            problem(&state(Phase::WaitingForGame("F1 25 is starting".into()), None)),
            None
        );
        let failed = problem(&state(Phase::Failed("x".into()), None)).expect("a problem");
        assert!(!failed.retry);
        assert_eq!(
            problem(&state(Phase::Ended("you left the room".into()), None)),
            None
        );
    }
}
