//! Shared careers on a host server: the career saves two (or more) players continue together.
//!
//! A career has an owner, its members, a career mode (for example `swofty.progressive`) and a
//! chain of saves. Each save is an opaque document (base64 text, as the game's online career
//! serialises it, or a league document) with the version it replaces, so two players can't
//! overwrite each other's progress: a save that doesn't name the current version is refused with
//! 409 and the latest one. The last [`KEEP`] saves stay as backups.
//!
//! Layout under `<dir>/<id>/`: `meta.json` (and its copy `meta.json.bak`), and `v<N>.json` per
//! kept save. A deleted career moves to `<dir>/.deleted/<id>-<unix time>`.
//!
//! **Trust.** Race results are trusted among a career's members: the server checks a report's
//! size, not whether the race happened. Members are friends who share the career's code; the
//! limits below only keep one member from flooding the server.
//!
//! League actions may name the version they were decided on (`base`): an action decided on an
//! older version is refused with 409 `stale`, so a late click can't take back a friend's race.
//! Old launchers send no `base` and are served as before.

use std::collections::{HashMap, VecDeque};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::lobby::Request;
use crate::league;

/// Saves kept per career, the newest included.
pub const KEEP: u64 = 10;
const NAME_MAX: usize = 48;
const MEMBERS_MAX: usize = 22;
/// Careers one user owns at most.
pub const OWNED_MAX: usize = 20;
/// The largest `extra` a career keeps, in bytes of JSON.
pub const EXTRA_MAX: usize = 64 * 1024;
/// Failed joins (a wrong code) one user may make within [`JOIN_WINDOW`].
pub const JOINS_FAILED_MAX: usize = 10;
pub const JOIN_WINDOW: Duration = Duration::from_secs(600);
/// A race report's limits: cars, laps and telemetry laps, and a driver name's characters.
pub const REPORT_CARS_MAX: usize = 30;
pub const REPORT_LAPS_MAX: usize = 300;
pub const REPORT_NAME_MAX: usize = 64;
const DELETED_DIR: &str = ".deleted";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Member {
    id: String,
    name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Meta {
    id: String,
    name: String,
    mode: String,
    owner: String,
    members: Vec<Member>,
    /// The career's join code, for adding a member who isn't in it yet.
    code: String,
    version: u64,
    hash: Option<String>,
    created_at: u64,
    updated_at: u64,
    updated_by: Option<String>,
    /// Free data the launcher keeps with the career, like the addon fingerprint it started with.
    #[serde(default)]
    extra: Value,
}

#[derive(Serialize, Deserialize)]
struct Save {
    version: u64,
    hash: String,
    data: String,
    by: String,
    at: u64,
}

pub struct Careers {
    dir: PathBuf,
    lock: Mutex<()>,
    /// User id → when their recent joins failed, oldest first.
    failed_joins: Mutex<HashMap<String, VecDeque<Instant>>>,
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn error(status: u16, reason: &str, text: &str) -> (u16, Value) {
    (status, json!({"error": text, "reason": reason}))
}

/// A league's refusal, with its template for launchers to show it in their language (W4-2).
fn refused(e: &league::LeagueError) -> (u16, Value) {
    let (status, mut body) = error(409, e.reason, &e.message);
    if !e.template.is_empty() {
        body["template"] = json!(e.template);
        body["args"] = json!(e.args);
    }
    (status, body)
}

fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 32 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

impl Careers {
    pub fn new(dir: PathBuf) -> Careers {
        Careers {
            dir,
            lock: Mutex::new(()),
            failed_joins: Mutex::new(HashMap::new()),
        }
    }

    fn career_dir(&self, id: &str) -> PathBuf {
        self.dir.join(id)
    }

    /// The career's meta, from `meta.json` or, when that is damaged, its copy.
    fn load(&self, id: &str) -> Option<Meta> {
        if !valid_id(id) {
            return None;
        }
        let dir = self.career_dir(id);
        let read = |name: &str| -> Option<Meta> { serde_json::from_slice(&std::fs::read(dir.join(name)).ok()?).ok() };
        read("meta.json").or_else(|| {
            let meta = read("meta.json.bak")?;
            if dir.join("meta.json").exists() {
                crate::multiplayer::diag::line(format!("career {id}: meta.json is damaged; read its copy"));
            }
            Some(meta)
        })
    }

    /// Store the meta, with a copy written first, so one of the two is whole whatever happens.
    fn store(&self, meta: &Meta) -> std::io::Result<()> {
        let dir = self.career_dir(&meta.id);
        std::fs::create_dir_all(&dir)?;
        let bytes = serde_json::to_vec_pretty(meta).unwrap_or_default();
        write_atomic(&dir.join("meta.json.bak"), &bytes)?;
        write_atomic(&dir.join("meta.json"), &bytes)
    }

    fn all(&self) -> Vec<Meta> {
        let Ok(entries) = std::fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        entries
            .flatten()
            .filter_map(|e| self.load(&e.file_name().to_string_lossy()))
            .collect()
    }

    pub(super) fn route(&self, req: &Request, path: &str, (user, name): (&str, &str)) -> (u16, Value) {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        let body: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
        match (req.method.as_str(), &parts[2..]) {
            ("GET", ["careers"]) => {
                let mut mine: Vec<Value> = self
                    .all()
                    .into_iter()
                    .filter(|m| m.members.iter().any(|x| x.id == user))
                    .map(|m| view(&m))
                    .collect();
                mine.sort_by_key(|m| std::cmp::Reverse(m["updated_at"].as_u64().unwrap_or(0)));
                (200, json!({ "careers": mine }))
            }
            ("POST", ["careers"]) => self.create(&body, user, name),
            ("POST", ["careers", "join"]) => self.join(&body, user, name),
            ("GET", ["careers", id]) => match self.load(id).filter(|m| is_member(m, user)) {
                Some(m) => (200, json!({ "career": view(&m) })),
                None => error(404, "not_found", "no career with that id"),
            },
            ("POST", ["careers", id, "leave"]) => self.leave(id, (user, name)),
            ("POST", ["careers", id, "members", member, "remove"]) => self.remove(id, (user, name), member),
            ("POST", ["careers", id, "delete"]) => self.delete(id, user),
            ("GET", ["careers", id, "save"]) => self.read_save(id, user, None),
            ("GET", ["careers", id, "save", version]) => match version.parse() {
                Ok(v) => self.read_save(id, user, Some(v)),
                Err(_) => error(400, "bad_version", "not a save version"),
            },
            ("POST", ["careers", id, "save"]) => self.write_save(id, user, &body),
            ("GET", ["careers", id, "league"]) => self.league(id, user),
            ("POST", ["careers", id, "league"]) => self.league_action(id, (user, name), &body),
            _ => error(404, "not_found", "no route"),
        }
    }

    /// A league career's document and its save version (see [`crate::league`]).
    fn league(&self, id: &str, user: &str) -> (u16, Value) {
        let meta = match self.league_meta(id, user) {
            Ok(m) => m,
            Err(e) => return e,
        };
        match self.load_league(&meta) {
            Ok(league) => (200, json!({"league": league, "version": meta.version})),
            Err(e) => e,
        }
    }

    /// Apply a league action for the member and store the league as the career's next save. The
    /// server applies every change, so two launchers can't overwrite each other's results.
    fn league_action(&self, id: &str, (user, name): (&str, &str), body: &Value) -> (u16, Value) {
        let mut meta = match self.league_meta(id, user) {
            Ok(m) => m,
            Err(e) => return e,
        };
        let action: league::Action = match serde_json::from_value(body["action"].clone()) {
            Ok(a) => a,
            Err(e) => return error(400, "bad_action", &format!("not a career action: {e}")),
        };
        if let Some(base) = body.get("base").and_then(Value::as_u64)
            && base != meta.version
        {
            return (
                409,
                json!({"error": "the career changed meanwhile; look at it again and try again",
                       "reason": "stale", "version": meta.version}),
            );
        }
        if let league::Action::Report { report, .. } = &action
            && let Err(problem) = check_report(report)
        {
            return error(400, "bad_report", &problem);
        }
        let actor = league::Actor {
            id: user.to_owned(),
            name: name.to_owned(),
            owner: meta.owner == user,
        };
        let loaded = self.load_league(&meta);
        let before = match &loaded {
            Ok(Some(l)) => serde_json::to_string(l).ok(),
            _ => None,
        };
        let result = match (loaded, action) {
            (Ok(None), league::Action::Start { setup, number }) => {
                if !actor.owner {
                    return error(403, "owner", "only the career's owner starts it");
                }
                let mut seed = [0u8; 8];
                crate::mpauth::getrandom(&mut seed);
                league::League::start(setup, &actor, number, u64::from_le_bytes(seed))
                    .map(|l| (l, Vec::new()))
            }
            (Ok(None), _) => return error(409, "not_started", "the career hasn't started yet"),
            (Ok(Some(mut l)), action) => league::apply_t(&mut l, &actor, action).map(|w| (l, w)),
            (Err(e), _) => return e,
        };
        let (league, warnings_t) = match result {
            Ok(r) => r,
            Err(e) => return refused(&e),
        };
        let warnings = league::texts::english(&warnings_t);
        let data = serde_json::to_string(&league).unwrap_or_default();
        // An action that changed nothing (a race counted already, say) stores no new save.
        if before.as_deref() == Some(data.as_str()) {
            return (
                200,
                json!({"league": league, "version": meta.version, "warnings": warnings,
                       "warnings_t": warnings_t}),
            );
        }
        match self.store_save(&mut meta, user, &data, None) {
            Ok((version, _)) => (
                200,
                json!({"league": league, "version": version, "warnings": warnings,
                       "warnings_t": warnings_t}),
            ),
            Err(e) => e,
        }
    }

    fn league_meta(&self, id: &str, user: &str) -> Result<Meta, (u16, Value)> {
        let Some(meta) = self.load(id).filter(|m| is_member(m, user)) else {
            return Err(error(404, "not_found", "no career with that id"));
        };
        if meta.mode != league::MODE {
            return Err(error(
                409,
                "not_league",
                "this shared career isn't a league career",
            ));
        }
        Ok(meta)
    }

    /// The league of the career's newest save, or of the newest readable one before it when that
    /// is missing or damaged (the next action then stores `v{version + 1}`, so nothing kept is
    /// overwritten).
    fn load_league(&self, meta: &Meta) -> Result<Option<league::League>, (u16, Value)> {
        if meta.version == 0 {
            return Ok(None);
        }
        let dir = self.career_dir(&meta.id);
        let oldest = meta.version.saturating_sub(KEEP - 1).max(1);
        let mut problem = String::new();
        for version in (oldest..=meta.version).rev() {
            let read = std::fs::read(dir.join(format!("v{version}.json")))
                .map_err(|e| e.to_string())
                .and_then(|b| serde_json::from_slice::<Save>(&b).map_err(|e| e.to_string()))
                .and_then(|s| serde_json::from_str::<league::League>(&s.data).map_err(|e| e.to_string()));
            match read {
                Ok(league) => {
                    if version != meta.version {
                        crate::multiplayer::diag::line(format!(
                            "career {}: save v{} is missing or damaged ({problem}); using v{version}",
                            meta.id, meta.version
                        ));
                    }
                    return Ok(Some(league));
                }
                Err(e) if problem.is_empty() => problem = e,
                Err(_) => {}
            }
        }
        Err(error(
            500,
            "storage",
            &format!("the career's saves are missing or damaged: {problem}"),
        ))
    }

    fn create(&self, body: &Value, user: &str, name: &str) -> (u16, Value) {
        let title: String = body["name"]
            .as_str()
            .unwrap_or("")
            .trim()
            .chars()
            .take(NAME_MAX)
            .collect();
        let mode: String = body["mode"].as_str().unwrap_or("").chars().take(64).collect();
        if title.is_empty() || mode.is_empty() {
            return error(400, "bad_career", "a career needs a name and a mode");
        }
        if let Err(e) = check_extra(body.get("extra")) {
            return e;
        }
        if self.all().iter().filter(|m| m.owner == user).count() >= OWNED_MAX {
            return error(
                409,
                "too_many",
                &format!("you own {OWNED_MAX} careers on this host server already; delete one first"),
            );
        }
        let mut raw = [0u8; 9];
        crate::mpauth::getrandom(&mut raw);
        let id = format!("c-{}", crate::baseline::hex(&raw[..6]));
        let code = format!("K-{}", crate::baseline::hex(&raw[6..9]).to_ascii_uppercase());
        let now = now_secs();
        let meta = Meta {
            id,
            name: title,
            mode,
            owner: user.to_owned(),
            members: vec![Member {
                id: user.to_owned(),
                name: name.to_owned(),
            }],
            code,
            version: 0,
            hash: None,
            created_at: now,
            updated_at: now,
            updated_by: None,
            extra: body.get("extra").cloned().unwrap_or(Value::Null),
        };
        match self.store(&meta) {
            Ok(()) => (201, json!({ "career": view(&meta) })),
            Err(e) => error(500, "storage", &format!("couldn't store the career: {e}")),
        }
    }

    fn join(&self, body: &Value, user: &str, name: &str) -> (u16, Value) {
        let now = Instant::now();
        let mut failed = self.failed_joins.lock().unwrap_or_else(|e| e.into_inner());
        failed.retain(|_, times| {
            while times.front().is_some_and(|t| now.duration_since(*t) >= JOIN_WINDOW) {
                times.pop_front();
            }
            !times.is_empty()
        });
        if let Some(oldest) = failed
            .get(user)
            .filter(|t| t.len() >= JOINS_FAILED_MAX)
            .and_then(|t| t.front())
        {
            let wait = JOIN_WINDOW.saturating_sub(now.duration_since(*oldest)).as_secs().max(1);
            return (
                429,
                json!({"error": format!("too many wrong career codes; try again in {} minute(s)", wait.div_ceil(60)),
                       "reason": "rate_limited", "retry_after": wait}),
            );
        }
        let code = body["code"].as_str().unwrap_or("").trim().to_ascii_uppercase();
        let Some(mut meta) = self.all().into_iter().find(|m| m.code == code) else {
            failed.entry(user.to_owned()).or_default().push_back(now);
            return error(404, "not_found", "no career has that code");
        };
        drop(failed);
        if !is_member(&meta, user) {
            if meta.members.len() >= MEMBERS_MAX {
                return error(409, "full", "this career has no free seat");
            }
            meta.members.push(Member {
                id: user.to_owned(),
                name: name.to_owned(),
            });
            if let Err(e) = self.store(&meta) {
                return error(500, "storage", &format!("couldn't store the career: {e}"));
            }
        }
        (200, json!({ "career": view(&meta) }))
    }

    /// A member leaves. In a league that has started, they leave the league too (their seat
    /// stays, driven by the game's AI), so they can't hold the others up.
    fn leave(&self, id: &str, (user, name): (&str, &str)) -> (u16, Value) {
        let Some(mut meta) = self.load(id).filter(|m| is_member(m, user)) else {
            return error(404, "not_found", "no career with that id");
        };
        if meta.owner == user {
            return error(409, "owner", "the owner can't leave their career");
        }
        let actor = league::Actor {
            id: user.to_owned(),
            name: name.to_owned(),
            owner: false,
        };
        if let Err(e) = self.remove_from_league(&mut meta, &actor, user) {
            return e;
        }
        meta.members.retain(|m| m.id != user);
        match self.store(&meta) {
            Ok(()) => (204, Value::Null),
            Err(e) => error(500, "storage", &format!("couldn't store the career: {e}")),
        }
    }

    /// The owner removes a member: from the league, when it has started, and from the career.
    fn remove(&self, id: &str, (user, name): (&str, &str), member: &str) -> (u16, Value) {
        let Some(mut meta) = self.load(id).filter(|m| is_member(m, user)) else {
            return error(404, "not_found", "no career with that id");
        };
        if meta.owner != user {
            return error(403, "owner", "only the career's owner removes members");
        }
        if member == user {
            return error(409, "owner", "the owner can't remove themselves; delete the career instead");
        }
        if !is_member(&meta, member) {
            return error(404, "not_found", "that player isn't a member of this career");
        }
        let actor = league::Actor {
            id: user.to_owned(),
            name: name.to_owned(),
            owner: true,
        };
        let league = match self.remove_from_league(&mut meta, &actor, member) {
            Ok(l) => l,
            Err(e) => return e,
        };
        meta.members.retain(|m| m.id != member);
        if let Err(e) = self.store(&meta) {
            return error(500, "storage", &format!("couldn't store the career: {e}"));
        }
        (
            200,
            json!({"career": view(&meta), "league": league, "version": meta.version}),
        )
    }

    /// Apply `RemovePlayer { player }` as `actor` to a league career that has started and has
    /// `player` in it, and store the result. Answers the league as it is then (`None` for a
    /// career that isn't a league or hasn't started).
    fn remove_from_league(
        &self,
        meta: &mut Meta,
        actor: &league::Actor,
        player: &str,
    ) -> Result<Option<league::League>, (u16, Value)> {
        if meta.mode != league::MODE || meta.version == 0 {
            return Ok(None);
        }
        let Some(mut l) = self.load_league(meta)? else {
            return Ok(None);
        };
        if l.player(player).is_none() {
            return Ok(Some(l));
        }
        let before = serde_json::to_string(&l).unwrap_or_default();
        let action = league::Action::RemovePlayer {
            player: player.to_owned(),
        };
        match league::apply(&mut l, actor, action) {
            Ok(_) => {}
            // Gone from the league already.
            Err(e) if e.reason == "left" => return Ok(Some(l)),
            Err(e) => return Err(refused(&e)),
        }
        let data = serde_json::to_string(&l).unwrap_or_default();
        if data != before {
            self.store_save(meta, &actor.id, &data, None)?;
        }
        Ok(Some(l))
    }

    /// The owner deletes the career: its folder moves to `.deleted/<id>-<unix time>`, so the host
    /// can still bring it back by hand.
    fn delete(&self, id: &str, user: &str) -> (u16, Value) {
        let Some(meta) = self.load(id).filter(|m| is_member(m, user)) else {
            return error(404, "not_found", "no career with that id");
        };
        if meta.owner != user {
            return error(403, "owner", "only the career's owner deletes it");
        }
        let aside = self.dir.join(DELETED_DIR);
        let moved = std::fs::create_dir_all(&aside).and_then(|()| {
            std::fs::rename(
                self.career_dir(&meta.id),
                aside.join(format!("{}-{}", meta.id, now_secs())),
            )
        });
        match moved {
            Ok(()) => (204, Value::Null),
            Err(e) => error(500, "storage", &format!("couldn't delete the career: {e}")),
        }
    }

    fn read_save(&self, id: &str, user: &str, version: Option<u64>) -> (u16, Value) {
        let Some(meta) = self.load(id).filter(|m| is_member(m, user)) else {
            return error(404, "not_found", "no career with that id");
        };
        let version = version.unwrap_or(meta.version);
        if version == 0 {
            return error(404, "no_save", "this career has no save yet");
        }
        let file = self.career_dir(id).join(format!("v{version}.json"));
        match std::fs::read(&file)
            .ok()
            .and_then(|b| serde_json::from_slice::<Save>(&b).ok())
        {
            Some(save) => (
                200,
                json!({"version": save.version, "hash": save.hash, "data": save.data,
                       "by": save.by, "at": save.at}),
            ),
            None => error(404, "no_save", "that save is no longer kept"),
        }
    }

    fn write_save(&self, id: &str, user: &str, body: &Value) -> (u16, Value) {
        let Some(mut meta) = self.load(id).filter(|m| is_member(m, user)) else {
            return error(404, "not_found", "no career with that id");
        };
        let Some(data) = body["data"].as_str() else {
            return error(400, "bad_save", "a save needs its data");
        };
        if meta.mode == league::MODE {
            return error(409, "league", "a league career changes only through its actions");
        }
        if let Err(e) = check_extra(body.get("extra")) {
            return e;
        }
        let base = body["base"].as_u64().unwrap_or(u64::MAX);
        if base != meta.version {
            return (
                409,
                json!({"error": "the career moved on since this save started; load the newest save first",
                       "reason": "stale", "version": meta.version, "hash": meta.hash}),
            );
        }
        let hash = crate::baseline::hex(&Sha256::digest(data.as_bytes()));
        if let Some(claimed) = body["hash"].as_str()
            && claimed != hash
        {
            return error(400, "bad_hash", "the save arrived damaged");
        }
        match self.store_save(&mut meta, user, data, body.get("extra")) {
            Ok((version, hash)) => (200, json!({"version": version, "hash": hash})),
            Err(e) => e,
        }
    }

    /// Store `data` as the career's next save, keeping the last [`KEEP`].
    fn store_save(
        &self,
        meta: &mut Meta,
        user: &str,
        data: &str,
        extra: Option<&Value>,
    ) -> Result<(u64, String), (u16, Value)> {
        let id = meta.id.clone();
        let hash = crate::baseline::hex(&Sha256::digest(data.as_bytes()));
        let version = meta.version + 1;
        let save = Save {
            version,
            hash: hash.clone(),
            data: data.to_owned(),
            by: user.to_owned(),
            at: now_secs(),
        };
        let dir = self.career_dir(&id);
        if let Err(e) = write_atomic(
            &dir.join(format!("v{version}.json")),
            &serde_json::to_vec(&save).unwrap_or_default(),
        ) {
            return Err(error(500, "storage", &format!("couldn't store the save: {e}")));
        }
        meta.version = version;
        meta.hash = Some(hash.clone());
        meta.updated_at = save.at;
        meta.updated_by = Some(user.to_owned());
        if let Some(extra) = extra {
            meta.extra = extra.clone();
        }
        if let Err(e) = self.store(meta) {
            return Err(error(500, "storage", &format!("couldn't store the career: {e}")));
        }
        prune(&dir, version);
        Ok((version, hash))
    }
}

/// Remove the saves older than the newest [`KEEP`] (and any left from before a crash).
fn prune(dir: &Path, newest: u64) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(n) = name
            .to_str()
            .and_then(|n| n.strip_prefix('v')?.strip_suffix(".json")?.parse::<u64>().ok())
        else {
            continue;
        };
        if n + KEEP <= newest {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

/// A race report the server takes: [`REPORT_CARS_MAX`] cars, [`REPORT_LAPS_MAX`] laps and
/// names of [`REPORT_NAME_MAX`] characters at most.
fn check_report(report: &league::RaceReport) -> Result<(), String> {
    if report.cars.len() > REPORT_CARS_MAX {
        return Err(format!(
            "a race report lists {} cars; at most {REPORT_CARS_MAX} are taken",
            report.cars.len()
        ));
    }
    if report.laps.len() > REPORT_LAPS_MAX {
        return Err(format!(
            "a race report lists {} laps; at most {REPORT_LAPS_MAX} are taken",
            report.laps.len()
        ));
    }
    if report.lap_telemetry.len() > REPORT_LAPS_MAX {
        return Err(format!(
            "a race report has telemetry for {} laps; at most {REPORT_LAPS_MAX} are taken",
            report.lap_telemetry.len()
        ));
    }
    if let Some(car) = report.cars.iter().find(|c| c.name.chars().count() > REPORT_NAME_MAX) {
        return Err(format!(
            "a driver's name in the race report is longer than {REPORT_NAME_MAX} characters: {}…",
            car.name.chars().take(24).collect::<String>()
        ));
    }
    Ok(())
}

/// `extra` up to [`EXTRA_MAX`] bytes.
fn check_extra(extra: Option<&Value>) -> Result<(), (u16, Value)> {
    let size = extra.map_or(0, |e| serde_json::to_vec(e).map_or(0, |b| b.len()));
    if size > EXTRA_MAX {
        return Err(error(
            413,
            "too_big",
            &format!("the career's extra data is {size} bytes; at most {EXTRA_MAX} are kept"),
        ));
    }
    Ok(())
}

fn is_member(meta: &Meta, user: &str) -> bool {
    meta.members.iter().any(|m| m.id == user)
}

fn view(meta: &Meta) -> Value {
    json!({
        "id": meta.id, "name": meta.name, "mode": meta.mode, "owner": meta.owner,
        "members": meta.members.iter().map(|m| json!({"id": m.id, "name": m.name})).collect::<Vec<_>>(),
        "code": meta.code, "version": meta.version, "hash": meta.hash,
        "created_at": meta.created_at, "updated_at": meta.updated_at, "updated_by": meta.updated_by,
        "extra": meta.extra,
    })
}

/// Write `bytes` to `path` whole or not at all: a flushed temporary file renamed over it (and on
/// Unix the directory flushed too, so the rename survives a power cut).
pub(super) fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    std::fs::rename(&tmp, path)?;
    #[cfg(unix)]
    if let Some(dir) = path.parent() {
        std::fs::File::open(dir)?.sync_all()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn req(method: &str, body: Value) -> Request {
        Request {
            method: method.into(),
            path: String::new(),
            headers: HashMap::new(),
            body: serde_json::to_vec(&body).unwrap(),
        }
    }

    #[test]
    fn two_players_share_a_career_and_cant_overwrite_each_other() {
        let dir = std::env::temp_dir().join(format!("swofty-host-careers-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let c = Careers::new(dir.clone());
        let (s, made) = c.route(
            &req(
                "POST",
                json!({"name": "2016 bis 2026", "mode": "swofty.progressive"}),
            ),
            "/v1/mp/careers",
            ("u-a", "Anna"),
        );
        assert_eq!(s, 201);
        let id = made["career"]["id"].as_str().unwrap().to_owned();
        let code = made["career"]["code"].as_str().unwrap().to_owned();
        let save_path = format!("/v1/mp/careers/{id}/save");

        assert_eq!(
            c.route(&req("GET", json!(null)), &save_path, ("u-b", "Ben")).0,
            404
        );
        let (s, joined) = c.route(
            &req("POST", json!({"code": code})),
            "/v1/mp/careers/join",
            ("u-b", "Ben"),
        );
        assert_eq!(s, 200);
        assert_eq!(joined["career"]["members"].as_array().unwrap().len(), 2);

        let (s, v1) = c.route(
            &req("POST", json!({"base": 0, "data": "AAAA"})),
            &save_path,
            ("u-a", "Anna"),
        );
        assert_eq!((s, v1["version"].as_u64()), (200, Some(1)));
        let (s, stale) = c.route(
            &req("POST", json!({"base": 0, "data": "BBBB"})),
            &save_path,
            ("u-b", "Ben"),
        );
        assert_eq!((s, stale["reason"].as_str()), (409, Some("stale")));
        let (s, latest) = c.route(&req("GET", json!(null)), &save_path, ("u-b", "Ben"));
        assert_eq!((s, latest["data"].as_str()), (200, Some("AAAA")));
        let (s, _) = c.route(
            &req("POST", json!({"base": 1, "data": "BBBB"})),
            &save_path,
            ("u-b", "Ben"),
        );
        assert_eq!(s, 200);

        let (_, list) = c.route(&req("GET", json!(null)), "/v1/mp/careers", ("u-a", "Anna"));
        assert_eq!(list["careers"][0]["version"].as_u64(), Some(2));
        let (_, none) = c.route(&req("GET", json!(null)), "/v1/mp/careers", ("u-c", "Cleo"));
        assert!(none["careers"].as_array().unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_server_applies_league_actions_and_keeps_each_step_as_a_save() {
        let dir = std::env::temp_dir().join(format!("f1mp-league-careers-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let c = Careers::new(dir.clone());
        let anna = ("u-a", "Anna");
        let ben = ("u-b", "Ben");
        let (_, made) = c.route(
            &req("POST", json!({"name": "2016 bis 2026", "mode": league::MODE})),
            "/v1/mp/careers",
            anna,
        );
        let id = made["career"]["id"].as_str().unwrap().to_owned();
        let code = made["career"]["code"].as_str().unwrap().to_owned();
        let path = format!("/v1/mp/careers/{id}/league");
        let act = |who, action: Value| c.route(&req("POST", json!({ "action": action })), &path, who);

        let (s, state) = c.route(&req("GET", json!(null)), &path, anna);
        assert_eq!((s, state["league"].is_null()), (200, true));
        assert_eq!(act(anna, json!({"op": "enter"})).1["reason"], "not_started");
        c.route(&req("POST", json!({ "code": code })), "/v1/mp/careers/join", ben);
        assert_eq!(act(ben, json!({"op": "start"})).1["reason"], "owner");
        let (s, started) = act(anna, json!({"op": "start", "number": 7}));
        assert_eq!((s, started["version"].as_u64()), (200, Some(1)));
        act(ben, json!({"op": "enter", "number": 8}));
        act(anna, json!({"op": "seat", "slot": 2}));
        act(ben, json!({"op": "seat", "slot": 2}));
        let (s, begun) = act(anna, json!({"op": "begin"}));
        assert_eq!((s, begun["league"]["phase"].as_str()), (200, Some("racing")));
        // Ben asks to simulate; Anna asking for the same agrees (W2-1).
        let (s, asked) = act(ben, json!({"op": "simulate"}));
        assert_eq!((s, asked["version"].as_u64()), (200, Some(6)));
        assert_eq!(asked["league"]["seasons"][0]["rounds"][0]["state"], "pending");
        // The warnings come in English and with their templates (W4-2).
        assert_eq!(asked["warnings"], json!(["Waiting for Anna to agree."]));
        assert_eq!(asked["warnings_t"][0]["template"], "Waiting for {names} to agree.");
        assert_eq!(asked["warnings_t"][0]["text"], "Waiting for Anna to agree.");
        let (s, sim) = act(anna, json!({"op": "simulate"}));
        assert_eq!((s, sim["version"].as_u64()), (200, Some(7)));
        assert_eq!(sim["league"]["seasons"][0]["rounds"][0]["state"], "simulated");
        let (s, refused) = act(ben, json!({"op": "begin"}));
        assert_eq!((s, refused["reason"].as_str()), (409, Some("owner")));
        // A refusal carries its template; the client reads it back.
        assert_eq!(refused["template"], "only the career's owner starts the first season");
        let e = crate::multiplayer::rooms::ApiError::from_body(409, &refused.to_string());
        let text = e.text.expect("a template");
        assert_eq!((text.template.as_str(), text.english()), (
            "only the career's owner starts the first season",
            e.message.clone()
        ));
        let (s, bad) = act(ben, json!({"op": "fly"}));
        assert_eq!((s, bad["reason"].as_str()), (400, Some("bad_action")));

        // A league is never overwritten by a raw save.
        let save = format!("/v1/mp/careers/{id}/save");
        let (s, _) = c.route(&req("POST", json!({"base": 7, "data": "{}"})), &save, anna);
        assert_eq!(s, 409);
        let (_, read) = c.route(&req("GET", json!(null)), &path, ben);
        assert_eq!(read["version"].as_u64(), Some(7));
        let league: league::League = serde_json::from_value(read["league"].clone()).unwrap();
        assert_eq!(league.players.len(), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }
    const ANNA: (&str, &str) = ("u-a", "Anna");
    const BEN: (&str, &str) = ("u-b", "Ben");

    /// A league career Anna owns and Ben joined, in `dir`.
    struct Shared {
        c: Careers,
        dir: PathBuf,
        id: String,
        code: String,
    }

    impl Shared {
        fn new(tag: &str) -> Shared {
            let dir = std::env::temp_dir().join(format!("f1mp-careers-{tag}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            let c = Careers::new(dir.clone());
            let (_, made) = c.route(
                &req("POST", json!({"name": "Liga", "mode": league::MODE})),
                "/v1/mp/careers",
                ANNA,
            );
            let id = made["career"]["id"].as_str().unwrap().to_owned();
            let code = made["career"]["code"].as_str().unwrap().to_owned();
            c.route(&req("POST", json!({ "code": code })), "/v1/mp/careers/join", BEN);
            Shared { c, dir, id, code }
        }

        /// Started, both seated at Ferrari and begun: version 5.
        fn racing(tag: &str) -> Shared {
            let l = Shared::new(tag);
            assert_eq!(l.act(ANNA, json!({"op": "start", "number": 7}), None).0, 200);
            l.act(BEN, json!({"op": "enter", "number": 8}), None);
            l.act(ANNA, json!({"op": "seat", "slot": 2}), None);
            l.act(BEN, json!({"op": "seat", "slot": 2}), None);
            let (s, begun) = l.act(ANNA, json!({"op": "begin"}), None);
            assert_eq!((s, begun["version"].as_u64()), (200, Some(5)));
            l
        }

        fn path(&self, tail: &str) -> String {
            format!("/v1/mp/careers/{}{tail}", self.id)
        }

        fn act(&self, who: (&str, &str), action: Value, base: Option<u64>) -> (u16, Value) {
            let body = match base {
                Some(b) => json!({ "action": action, "base": b }),
                None => json!({ "action": action }),
            };
            self.c.route(&req("POST", body), &self.path("/league"), who)
        }

        fn get(&self, who: (&str, &str), tail: &str) -> (u16, Value) {
            self.c.route(&req("GET", Value::Null), &self.path(tail), who)
        }

        fn post(&self, who: (&str, &str), tail: &str) -> (u16, Value) {
            self.c.route(&req("POST", Value::Null), &self.path(tail), who)
        }

        fn saves(&self) -> Vec<u64> {
            let mut v: Vec<u64> = std::fs::read_dir(self.dir.join(&self.id))
                .unwrap()
                .flatten()
                .filter_map(|e| {
                    let n = e.file_name().to_string_lossy().into_owned();
                    n.strip_prefix('v')?.strip_suffix(".json")?.parse().ok()
                })
                .collect();
            v.sort();
            v
        }

        /// The game's report of the next round's race, Anna's car winning.
        fn report(&self, session: u64) -> Value {
            let doc = self.get(ANNA, "/league").1;
            let l: league::League = serde_json::from_value(doc["league"].clone()).unwrap();
            let season = l.season();
            let cars: Vec<Value> = season
                .entries
                .iter()
                .enumerate()
                .map(|(i, e)| {
                    let mine = e.player.as_deref() == Some(ANNA.0);
                    json!({"name": if e.player.is_some() { "Player".to_owned() } else { e.name.to_uppercase() },
                           "number": e.number, "team": e.slot, "ai": e.player.is_none(), "mine": mine,
                           "position": if mine { 1 } else { i as u8 + 2 }, "grid": i + 1, "laps": 30, "status": 3,
                           "best_lap_ms": 90_000 + i as u32, "time_ms": 3_600_000 + i as u64 * 1000, "penalty_s": 0})
                })
                .collect();
            let track = l.briefing(ANNA.0).and_then(|b| b.track);
            json!({"op": "report", "force": true,
                   "report": {"session": session, "track": track, "session_type": 15, "cars": cars}})
        }
    }

    impl Drop for Shared {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    #[test]
    fn a_stale_base_is_refused_with_the_current_version() {
        let l = Shared::racing("stale");
        let (s, refused) = l.act(BEN, json!({"op": "undo"}), Some(4));
        assert_eq!((s, refused["reason"].as_str(), refused["version"].as_u64()), (409, Some("stale"), Some(5)));
        assert_eq!(l.get(ANNA, "/league").1["version"].as_u64(), Some(5), "nothing stored");
        let (s, done) = l.act(BEN, json!({"op": "simulate"}), Some(5));
        assert_eq!((s, done["version"].as_u64()), (200, Some(6)));
    }

    #[test]
    fn no_base_is_accepted_for_old_launchers() {
        let l = Shared::racing("nobase");
        let (s, done) = l.act(BEN, json!({"op": "simulate"}), None);
        assert_eq!((s, done["version"].as_u64()), (200, Some(6)));
        // A base that isn't a number counts as none (Anna agrees to Ben's simulate, W2-1).
        let body = json!({"action": {"op": "simulate"}, "base": null});
        let (s, done) = l.c.route(&req("POST", body), &l.path("/league"), ANNA);
        assert_eq!((s, done["version"].as_u64()), (200, Some(7)));
    }

    #[test]
    fn an_action_that_changes_nothing_stores_no_version() {
        let l = Shared::racing("noop");
        let race = l.report(0xabc);
        let (s, counted) = l.act(ANNA, race.clone(), None);
        assert_eq!((s, counted["version"].as_u64()), (200, Some(6)), "{counted}");
        let (_, again) = l.act(ANNA, race, None);
        assert_eq!(again["version"].as_u64().or(Some(6)), Some(6), "{again}");
        assert_eq!(l.get(ANNA, "/league").1["version"].as_u64(), Some(6));
        assert_eq!(l.saves(), (1..=6).collect::<Vec<_>>());
    }

    #[test]
    fn a_damaged_newest_save_falls_back_to_the_one_before() {
        let l = Shared::new("damaged");
        l.act(ANNA, json!({"op": "start", "number": 7}), None);
        l.act(BEN, json!({"op": "enter", "number": 8}), None);
        let v2 = l.get(ANNA, "/league").1["league"].clone();
        l.act(ANNA, json!({"op": "seat", "slot": 2}), None);
        std::fs::write(l.dir.join(&l.id).join("v3.json"), b"{ torn").unwrap();
        let (s, read) = l.get(BEN, "/league");
        assert_eq!((s, read["version"].as_u64()), (200, Some(3)));
        assert_eq!(read["league"], v2);
        let (s, next) = l.act(BEN, json!({"op": "seat", "slot": 2}), Some(3));
        assert_eq!((s, next["version"].as_u64()), (200, Some(4)));
        assert!(l.dir.join(&l.id).join("v4.json").exists());
        assert_eq!(std::fs::read(l.dir.join(&l.id).join("v3.json")).unwrap(), b"{ torn", "v3 isn't overwritten");
        // With every kept save damaged, the server says so.
        for v in 1..=4 {
            std::fs::write(l.dir.join(&l.id).join(format!("v{v}.json")), b"").unwrap();
        }
        assert_eq!(l.get(BEN, "/league").0, 500);
    }

    #[test]
    fn a_damaged_meta_reads_its_backup() {
        let l = Shared::racing("meta");
        std::fs::write(l.dir.join(&l.id).join("meta.json"), b"{\"id\": ").unwrap();
        let (s, read) = l.get(BEN, "");
        assert_eq!((s, read["career"]["version"].as_u64()), (200, Some(5)));
        let (s, next) = l.act(BEN, json!({"op": "simulate"}), Some(5));
        assert_eq!((s, next["version"].as_u64()), (200, Some(6)));
        let (_, list) = l.c.route(&req("GET", Value::Null), "/v1/mp/careers", ANNA);
        assert_eq!(list["careers"][0]["version"].as_u64(), Some(6));
    }

    #[test]
    fn an_oversized_report_is_refused() {
        let l = Shared::racing("oversized");
        let mut race = l.report(0xdef);
        let car = race["report"]["cars"][0].clone();
        race["report"]["cars"] = Value::Array(vec![car.clone(); REPORT_CARS_MAX + 1]);
        let (s, refused) = l.act(ANNA, race.clone(), None);
        assert_eq!((s, refused["reason"].as_str()), (400, Some("bad_report")));
        let mut long = car;
        long["name"] = json!("X".repeat(REPORT_NAME_MAX + 1));
        race["report"]["cars"] = json!([long]);
        assert_eq!(l.act(ANNA, race.clone(), None).1["reason"], "bad_report");
        race["report"]["cars"] = json!([]);
        race["report"]["laps"] = Value::Array(vec![json!({"ms": 90000, "valid": true}); REPORT_LAPS_MAX + 1]);
        assert_eq!(l.act(ANNA, race.clone(), None).1["reason"], "bad_report");
        race["report"]["laps"] = json!([]);
        race["report"]["lap_telemetry"] = Value::Array(vec![json!({"lap": 1, "ms": 90000}); REPORT_LAPS_MAX + 1]);
        assert_eq!(l.act(ANNA, race, None).1["reason"], "bad_report");
        assert_eq!(l.get(ANNA, "/league").1["version"].as_u64(), Some(5));
    }

    #[test]
    fn owners_have_a_career_limit_and_extra_is_capped() {
        let dir = std::env::temp_dir().join(format!("f1mp-careers-limit-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let c = Careers::new(dir.clone());
        let create = |extra: Value, who| {
            c.route(
                &req("POST", json!({"name": "C", "mode": "swofty.progressive", "extra": extra})),
                "/v1/mp/careers",
                who,
            )
        };
        let big = json!("x".repeat(EXTRA_MAX + 1));
        let (s, refused) = create(big.clone(), ANNA);
        assert_eq!((s, refused["reason"].as_str()), (413, Some("too_big")));
        let mut first = String::new();
        for _ in 0..OWNED_MAX {
            let (s, made) = create(json!({"addons": 3}), ANNA);
            assert_eq!(s, 201);
            first = made["career"]["id"].as_str().unwrap().to_owned();
        }
        let (s, refused) = create(Value::Null, ANNA);
        assert_eq!((s, refused["reason"].as_str()), (409, Some("too_many")));
        assert_eq!(create(Value::Null, BEN).0, 201, "the limit is per owner");
        let save = format!("/v1/mp/careers/{first}/save");
        let (s, refused) = c.route(&req("POST", json!({"base": 0, "data": "AA", "extra": big})), &save, ANNA);
        assert_eq!((s, refused["reason"].as_str()), (413, Some("too_big")));
        // Deleting one makes room.
        assert_eq!(c.route(&req("POST", Value::Null), &format!("/v1/mp/careers/{first}/delete"), ANNA).0, 204);
        assert_eq!(create(Value::Null, ANNA).0, 201);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn failed_joins_are_rate_limited() {
        let l = Shared::new("joins");
        let join = |code: &str, who| l.c.route(&req("POST", json!({ "code": code })), "/v1/mp/careers/join", who);
        let cleo = ("u-c", "Cleo");
        for _ in 0..JOINS_FAILED_MAX {
            assert_eq!(join("K-000000", cleo).0, 404);
        }
        let (s, limited) = join(&l.code, cleo);
        assert_eq!((s, limited["reason"].as_str()), (429, Some("rate_limited")));
        assert!(limited["retry_after"].as_u64().is_some_and(|r| r > 0 && r <= JOIN_WINDOW.as_secs()));
        assert_eq!(join(&l.code, ("u-d", "Dana")).0, 200, "others still join");
    }

    #[test]
    fn pruned_saves_stay_at_keep() {
        let l = Shared::racing("prune");
        // Ben asks to simulate, Anna agrees (W2-1): each is a save.
        for i in 5..KEEP + 3 {
            let who = if i % 2 == 1 { BEN } else { ANNA };
            assert_eq!(l.act(who, json!({"op": "simulate"}), None).0, 200);
        }
        assert_eq!(l.get(ANNA, "/league").1["version"].as_u64(), Some(KEEP + 3));
        assert_eq!(l.saves(), (4..=KEEP + 3).collect::<Vec<_>>());
        assert_eq!(l.saves().len() as u64, KEEP);
    }

    #[test]
    fn career_ids_with_dots_or_slashes_are_not_found() {
        let l = Shared::racing("traversal");
        std::fs::create_dir_all(l.dir.join(DELETED_DIR)).unwrap();
        for id in ["..", ".", ".deleted", "..%2F..", "c-1.json"] {
            for tail in ["", "/league", "/save"] {
                let path = format!("/v1/mp/careers/{id}{tail}");
                assert_eq!(l.c.route(&req("GET", Value::Null), &path, ANNA).0, 404, "{path}");
            }
        }
        let path = format!("/v1/mp/careers/../{}/league", l.id);
        assert_eq!(l.c.route(&req("GET", Value::Null), &path, ANNA).0, 404);
        assert_eq!(l.get(ANNA, "/../league").0, 404);
    }
    #[test]
    fn leaving_in_setup_removes_the_league_player_so_the_owner_can_begin() {
        let l = Shared::new("leave-setup");
        l.act(ANNA, json!({"op": "start", "number": 7}), None);
        l.act(BEN, json!({"op": "enter", "number": 8}), None);
        l.act(ANNA, json!({"op": "seat", "slot": 2}), None);
        assert_eq!(l.act(ANNA, json!({"op": "begin"}), None).0, 409, "Ben has no seat");
        assert_eq!(l.post(ANNA, "/leave").1["reason"], "owner", "the owner can't leave");
        assert_eq!(l.post(BEN, "/leave").0, 204);
        let (_, read) = l.get(ANNA, "/league");
        assert_eq!(read["league"]["players"].as_array().unwrap().len(), 1);
        assert_eq!(read["version"].as_u64(), Some(4));
        assert_eq!(l.get(BEN, "").0, 404, "Ben is no member any more");
        assert_eq!(l.get(ANNA, "").1["career"]["members"].as_array().unwrap().len(), 1);
        let (s, begun) = l.act(ANNA, json!({"op": "begin"}), None);
        assert_eq!((s, begun["league"]["phase"].as_str()), (200, Some("racing")));
        // Leaving before the league started, or a career that isn't a league, is as before.
        let other = Shared::new("leave-plain");
        assert_eq!(other.post(BEN, "/leave").0, 204);
        assert_eq!(other.get(ANNA, "/league").1["league"], Value::Null);
    }

    #[test]
    fn the_owner_removes_a_member_mid_season() {
        let l = Shared::racing("remove");
        let cleo = ("u-c", "Cleo");
        l.c.route(&req("POST", json!({ "code": l.code })), "/v1/mp/careers/join", cleo);
        assert_eq!(l.post(BEN, "/members/u-a/remove").1["reason"], "owner");
        assert_eq!(l.post(ANNA, "/members/u-a/remove").0, 409);
        assert_eq!(l.post(ANNA, "/members/u-x/remove").0, 404);
        assert_eq!(l.post(cleo, "/members/u-b/remove").0, 403);
        let (s, removed) = l.post(ANNA, "/members/u-b/remove");
        assert_eq!((s, removed["version"].as_u64()), (200, Some(6)), "{removed}");
        let members: Vec<&str> = removed["career"]["members"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| m["id"].as_str().unwrap())
            .collect();
        assert_eq!(members, ["u-a", "u-c"]);
        let league: league::League = serde_json::from_value(removed["league"].clone()).unwrap();
        assert!(league.player(BEN.0).unwrap().left);
        assert!(league.season().player_entry(BEN.0).is_some(), "the seat stays");
        assert_eq!(l.get(BEN, "/league").0, 404);
        // The career goes on without Ben.
        assert_eq!(l.act(ANNA, json!({"op": "simulate"}), Some(6)).0, 200);
        // Removing a member who isn't in the league only drops the membership.
        let (s, removed) = l.post(ANNA, "/members/u-c/remove");
        assert_eq!((s, removed["version"].as_u64()), (200, Some(7)));
    }

    #[test]
    fn only_the_owner_deletes_and_the_career_is_moved_aside() {
        let l = Shared::racing("delete");
        assert_eq!(l.post(BEN, "/delete").1["reason"], "owner");
        assert_eq!(l.post(("u-c", "Cleo"), "/delete").0, 404);
        assert_eq!(l.post(ANNA, "/delete").0, 204);
        assert_eq!(l.get(ANNA, "").0, 404);
        assert_eq!(l.get(BEN, "/league").0, 404);
        let (_, list) = l.c.route(&req("GET", Value::Null), "/v1/mp/careers", ANNA);
        assert!(list["careers"].as_array().unwrap().is_empty());
        let aside: Vec<String> = std::fs::read_dir(l.dir.join(DELETED_DIR))
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(aside.len(), 1);
        assert!(aside[0].starts_with(&format!("{}-", l.id)));
        assert!(l.dir.join(DELETED_DIR).join(&aside[0]).join("v5.json").exists());
        // Its code joins nothing any more.
        let (s, _) = l.c.route(&req("POST", json!({ "code": l.code })), "/v1/mp/careers/join", ("u-c", "Cleo"));
        assert_eq!(s, 404);
    }
}
