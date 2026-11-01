//! Shared careers on a host server: the career saves two (or more) players continue together.
//!
//! A career has an owner, its members, a career mode (for example `swofty.progressive`) and a
//! chain of saves. Each save is an opaque document (base64 text, as the game's online career
//! serialises it, or a league document) with the version it replaces, so two players can't
//! overwrite each other's progress: a save that doesn't name the current version is refused with
//! 409 and the latest one. The last [`KEEP`] saves stay as backups.
//!
//! Layout under `<dir>/<id>/`: `meta.json`, and `v<N>.json` per kept save.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::lobby::Request;
use crate::league;

/// Saves kept per career, the newest included.
pub const KEEP: u64 = 10;
const NAME_MAX: usize = 48;
const MEMBERS_MAX: usize = 22;

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
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn error(status: u16, reason: &str, text: &str) -> (u16, Value) {
    (status, json!({"error": text, "reason": reason}))
}

fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 32 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

impl Careers {
    pub fn new(dir: PathBuf) -> Careers {
        Careers {
            dir,
            lock: Mutex::new(()),
        }
    }

    fn career_dir(&self, id: &str) -> PathBuf {
        self.dir.join(id)
    }

    fn load(&self, id: &str) -> Option<Meta> {
        if !valid_id(id) {
            return None;
        }
        let bytes = std::fs::read(self.career_dir(id).join("meta.json")).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    fn store(&self, meta: &Meta) -> std::io::Result<()> {
        let dir = self.career_dir(&meta.id);
        std::fs::create_dir_all(&dir)?;
        write_atomic(
            &dir.join("meta.json"),
            &serde_json::to_vec_pretty(meta).unwrap_or_default(),
        )
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
            ("POST", ["careers", id, "leave"]) => self.leave(id, user),
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
        let actor = league::Actor {
            id: user.to_owned(),
            name: name.to_owned(),
            owner: meta.owner == user,
        };
        let result = match (self.load_league(&meta), action) {
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
            (Ok(Some(mut l)), action) => league::apply(&mut l, &actor, action).map(|w| (l, w)),
            (Err(e), _) => return e,
        };
        let (league, warnings) = match result {
            Ok(r) => r,
            Err(e) => return error(409, e.reason, &e.message),
        };
        let data = serde_json::to_string(&league).unwrap_or_default();
        match self.store_save(&mut meta, user, &data, None) {
            Ok((version, _)) => (
                200,
                json!({"league": league, "version": version, "warnings": warnings}),
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

    fn load_league(&self, meta: &Meta) -> Result<Option<league::League>, (u16, Value)> {
        if meta.version == 0 {
            return Ok(None);
        }
        let file = self.career_dir(&meta.id).join(format!("v{}.json", meta.version));
        let save = std::fs::read(&file)
            .ok()
            .and_then(|b| serde_json::from_slice::<Save>(&b).ok())
            .ok_or_else(|| error(500, "storage", "the career's newest save is missing"))?;
        serde_json::from_str(&save.data).map(Some).map_err(|e| {
            error(
                500,
                "storage",
                &format!("the career's newest save is damaged: {e}"),
            )
        })
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
        let code = body["code"].as_str().unwrap_or("").trim().to_ascii_uppercase();
        let Some(mut meta) = self.all().into_iter().find(|m| m.code == code) else {
            return error(404, "not_found", "no career has that code");
        };
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

    fn leave(&self, id: &str, user: &str) -> (u16, Value) {
        let Some(mut meta) = self.load(id).filter(|m| is_member(m, user)) else {
            return error(404, "not_found", "no career with that id");
        };
        if meta.owner == user {
            return error(409, "owner", "the owner can't leave their career");
        }
        meta.members.retain(|m| m.id != user);
        match self.store(&meta) {
            Ok(()) => (204, Value::Null),
            Err(e) => error(500, "storage", &format!("couldn't store the career: {e}")),
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
        if version > KEEP {
            let _ = std::fs::remove_file(dir.join(format!("v{}.json", version - KEEP)));
        }
        Ok((version, hash))
    }
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

fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)
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
        let (s, sim) = act(ben, json!({"op": "simulate"}));
        assert_eq!((s, sim["version"].as_u64()), (200, Some(6)));
        assert_eq!(sim["league"]["seasons"][0]["rounds"][0]["state"], "simulated");
        let (s, refused) = act(ben, json!({"op": "begin"}));
        assert_eq!((s, refused["reason"].as_str()), (409, Some("owner")));
        let (s, bad) = act(ben, json!({"op": "fly"}));
        assert_eq!((s, bad["reason"].as_str()), (400, Some("bad_action")));

        // A league is never overwritten by a raw save.
        let save = format!("/v1/mp/careers/{id}/save");
        let (s, _) = c.route(&req("POST", json!({"base": 6, "data": "{}"})), &save, anna);
        assert_eq!(s, 409);
        let (_, read) = c.route(&req("GET", json!(null)), &path, ben);
        assert_eq!(read["version"].as_u64(), Some(6));
        let league: league::League = serde_json::from_value(read["league"].clone()).unwrap();
        assert_eq!(league.players.len(), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
