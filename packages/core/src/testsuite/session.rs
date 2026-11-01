//! The `session` environment: the test agent starts a Time Trial or Grand
//! Prix session straight from the main menu, on a box's track, with the
//! values the test names.

use serde_json::{Value, json};

use super::suite::Env;

pub const MODES: [&str; 2] = ["time_trial", "grand_prix"];
pub const SESSIONS: [&str; 23] = [
    "practice1",
    "practice2",
    "practice3",
    "qualifying",
    "sprint_shootout",
    "practice_1",
    "practice_2",
    "practice_3",
    "practice_s",
    "qualifying_1",
    "qualifying_2",
    "qualifying_3",
    "qualifying_s",
    "qualifying_o",
    "sprint_shootout_1",
    "sprint_shootout_2",
    "sprint_shootout_3",
    "sprint_shootout_s",
    "sprint_shootout_o",
    "race",
    "race_2",
    "race_3",
    "time_trial",
];
pub const WEATHER: [&str; 3] = ["clear", "wet", "rain"];
pub const LAPS_MAX: u64 = 100;
pub const AI_MAX: u64 = 110;

/// The agent logs this once the session's load has started:
/// `[test] session.start time_trial melbourne`.
pub const READY_PREFIX: &str = "[test] session.start ";

pub fn mode(env: &Env) -> &str {
    env.str("mode").unwrap_or(MODES[0])
}

/// Why a value is not one the environment takes, or `None`.
pub fn check(key: &str, value: &Value) -> Option<String> {
    let names = |list: &[&str]| format!("expected one of: {}", list.join(", "));
    match (key, value) {
        ("mode", Value::String(s)) if !MODES.contains(&s.as_str()) => Some(names(&MODES)),
        ("session", Value::String(s)) if !SESSIONS.contains(&s.as_str()) => Some(names(&SESSIONS)),
        ("weather", Value::String(s)) if !WEATHER.contains(&s.as_str()) => Some(names(&WEATHER)),
        ("laps", Value::Number(n)) if n.as_u64().is_some_and(|n| !(1..=LAPS_MAX).contains(&n)) => {
            Some(format!("expected 1 to {LAPS_MAX}, got {n}"))
        }
        ("ai_difficulty", Value::Number(n)) if n.as_u64().is_some_and(|n| n > AI_MAX) => {
            Some(format!("expected 0 to {AI_MAX}, got {n}"))
        }
        _ => None,
    }
}

/// The agent command for the environment: `session.start` with the
/// environment's own keys.
pub fn agent_command(env: &Env) -> (String, Value) {
    let mut args = json!({ "mode": mode(env) });
    for key in [
        "box",
        "track",
        "team",
        "driver",
        "session",
        "weather",
        "laps",
        "ai_difficulty",
    ] {
        if let Some(v) = env.params.get(key) {
            args[key] = v.clone();
        }
    }
    ("session.start".to_owned(), args)
}

/// What `session time_trial cs_f1_2017 melbourne` in `f1mp test shell`
/// means: the mode, then the box, then the track, then `key=value` pairs.
pub fn shell_args(words: &[&str]) -> Result<Value, String> {
    let usage = "session <time_trial|grand_prix> [box] [track] [team=..] [driver=..] [session=..] [weather=..] [laps=..] [ai_difficulty=..]";
    let Some((mode, rest)) = words.split_first() else {
        return Err(format!("usage: {usage}"));
    };
    if !MODES.contains(mode) {
        return Err(format!(
            "{mode}: expected time_trial or grand_prix; usage: {usage}"
        ));
    }
    let mut args = json!({ "mode": mode });
    let mut plain = ["box", "track"].into_iter();
    for w in rest {
        let (key, text) = match w.split_once('=') {
            Some((k, v)) => (k, v),
            None => (
                plain.next().ok_or_else(|| {
                    format!("{w}: only a box and a track go without a name; usage: {usage}")
                })?,
                *w,
            ),
        };
        let value = match key {
            "laps" | "ai_difficulty" => json!(
                text.parse::<u64>()
                    .map_err(|_| format!("{key}: expected a number, got {text}"))?
            ),
            "box" | "track" | "team" | "driver" | "session" | "weather" => json!(text),
            _ => return Err(format!("{key}: not a session key; usage: {usage}")),
        };
        if let Some(why) = check(key, &value) {
            return Err(format!("{key}: {why}"));
        }
        args[key] = value;
    }
    Ok(args)
}
