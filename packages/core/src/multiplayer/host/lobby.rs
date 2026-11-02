//! The host server's lobby: sign-in by name, rooms, invite codes, the members' websocket and TURN
//! credentials. It speaks the `/v1/auth/launcher` and `/v1/mp` API the launcher already uses with
//! SwoftyLauncherServer (see [`crate::multiplayer::rooms`] and [`crate::multiplayer::session`]),
//! with a name-and-password sign-in (`POST /v1/auth/launcher/direct`) instead of Discord.
//!
//! **Identity.** A user's id comes from their name, so the same name keeps its careers on any PC.
//! A launcher also sends a per-server secret (see [`crate::mpauth::host_secret`]); the first
//! secret that signs in under a name *claims* it, and from then on only that secret signs in under
//! it (`name_claimed` for another secret, `secret_required` for an old launcher that sends none).
//! A launcher that signed in before claims with its session token (`POST /v1/auth/launcher/claim`).
//! Unclaimed names sign in by name alone, as before. The host frees a name with [`release_name`].

use std::collections::HashMap;
use std::io::{self, Cursor, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tungstenite::Message;

use super::careers::Careers;

/// How long TURN credentials from `POST /v1/mp/turn` last.
const TURN_TTL: Duration = Duration::from_secs(24 * 3600);
const HEAD_LIMIT: usize = 16 * 1024;
const BODY_LIMIT: usize = 8 * 1024 * 1024;
const NAME_MAX: usize = 32;
const CODE_ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
/// Sessions one user keeps at once when signing in with their secret (the launcher and the CLI,
/// a reinstall); the oldest goes first.
const SESSIONS_PER_USER: usize = 8;
/// The shortest secret accepted: 32 bytes in unpadded base64url.
const SECRET_MIN: usize = 43;
const SECRET_MAX: usize = 256;

pub(super) type TlsStream = rustls::StreamOwned<rustls::ServerConnection, TcpStream>;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
    /// The order sessions were made in, so the oldest goes first; 0 for sessions from before.
    #[serde(default, skip_serializing_if = "is_zero")]
    since: u64,
}

fn is_zero(n: &u64) -> bool {
    *n == 0
}

/// Who holds a name: the hash of the secret that claimed it.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct UserRecord {
    /// The name the user signed in with last.
    name: String,
    /// SHA-256 of the user's secret, `None` once the host released the name.
    #[serde(default)]
    secret_hash: Option<String>,
    #[serde(default)]
    claimed_at: u64,
}

#[derive(Serialize, Deserialize, Default)]
struct Saved {
    /// SHA-256 of a session token → the user it signs in.
    sessions: HashMap<String, User>,
    /// User id → the record of who claimed the name.
    #[serde(default)]
    users: HashMap<String, UserRecord>,
}

impl Saved {
    /// The user who holds `name` with a secret: the one whose id it is, or the one who uses it as
    /// their display name now.
    fn holder(&self, name: &str) -> Option<&str> {
        let id = user_id(name);
        if self.users.get(&id).is_some_and(|r| r.secret_hash.is_some()) {
            return Some(self.users.get_key_value(&id)?.0);
        }
        let wanted = normal(name);
        self.users
            .iter()
            .find(|(_, r)| r.secret_hash.is_some() && normal(&r.name) == wanted)
            .map(|(id, _)| id.as_str())
    }

    /// The user whose secret hashes to `hash`.
    fn by_secret(&self, hash: &str) -> Option<String> {
        self.users
            .iter()
            .find(|(_, r)| r.secret_hash.as_deref() == Some(hash))
            .map(|(id, _)| id.clone())
    }

    /// Add a session for `user`, keeping at most [`SESSIONS_PER_USER`] of theirs.
    fn add_session(&mut self, token: &str, mut user: User) {
        let mine: Vec<(String, u64)> = self
            .sessions
            .iter()
            .filter(|(_, u)| u.id == user.id)
            .map(|(k, u)| (k.clone(), u.since))
            .collect();
        user.since = self.sessions.values().map(|u| u.since).max().unwrap_or(0) + 1;
        let mut mine = mine;
        mine.sort_by_key(|(_, since)| *since);
        let excess = (mine.len() + 1).saturating_sub(SESSIONS_PER_USER);
        for (key, _) in mine.into_iter().take(excess) {
            self.sessions.remove(&key);
        }
        self.sessions.insert(token_hash(token), user);
    }
}

/// A name as it is compared: trimmed and lowercase.
fn normal(name: &str) -> String {
    name.trim().to_lowercase()
}

struct Member {
    user: User,
    ip: u8,
    connected: bool,
    digest: Option<String>,
    blob: Option<Value>,
    relay: Option<String>,
    /// The member's game is in the room's lobby, as its `ready` message says. Launchers count only a
    /// ready member as joined; a reopened room and a lost connection clear it.
    ready: bool,
}

struct Room {
    code: String,
    host: String,
    name: String,
    public: bool,
    category: String,
    max: u64,
    started: bool,
    rev: u64,
    created_at: u64,
    key: String,
    members: Vec<Member>,
}

impl Room {
    fn next_ip(&self) -> Option<u8> {
        (1..=254u8).find(|n| !self.members.iter().any(|m| m.ip == *n))
    }

    fn has(&self, user: &str) -> bool {
        self.members.iter().any(|m| m.user.id == user)
    }
}

enum Out {
    Text(String),
    Close(u16),
}

#[derive(Default)]
struct State {
    saved: Saved,
    /// `sessions.json`'s modification time when this server last read or wrote it: a newer one
    /// was changed from outside (see [`release_name`]) and is read again.
    stamp: Option<(SystemTime, u64)>,
    rooms: HashMap<String, Room>,
    /// User id → the code of the room they are in.
    member_of: HashMap<String, String>,
    sockets: HashMap<String, mpsc::Sender<Out>>,
    /// User id → the number of the connection that owns their entry in `sockets`.
    connections: HashMap<String, u64>,
    next_connection: u64,
}

pub struct Lobby {
    pub(super) name: String,
    password: Option<String>,
    turn_secret: Vec<u8>,
    turn_port: u16,
    file: PathBuf,
    state: Mutex<State>,
    pub(super) careers: Careers,
}

pub(super) struct Request {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn random_text(bytes: usize) -> String {
    let mut raw = vec![0u8; bytes];
    crate::mpauth::getrandom(&mut raw);
    crate::mpauth::base64url(&raw)
}

fn token_hash(token: &str) -> String {
    crate::baseline::hex(&Sha256::digest(token.as_bytes()))
}

fn error(status: u16, reason: &str, text: &str) -> (u16, Value) {
    (status, json!({"error": text, "reason": reason}))
}

fn name_claimed() -> (u16, Value) {
    error(
        409,
        "name_claimed",
        "this name is taken on this host server; sign in from the PC you used before, or ask the host to release it",
    )
}

/// The request's session token.
fn bearer(req: &Request) -> Option<&str> {
    req.headers
        .get("authorization")
        .and_then(|a| a.strip_prefix("Bearer "))
}

/// The launcher's secret in a sign-in or claim body: `None` when it sends none (an older
/// launcher), an error when it isn't base64url of at least 32 bytes.
fn secret_of(doc: &Value) -> Result<Option<String>, (u16, Value)> {
    match doc.get("secret") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s))
            if (SECRET_MIN..=SECRET_MAX).contains(&s.len())
                && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_') =>
        {
            Ok(Some(s.clone()))
        }
        Some(_) => Err(error(
            400,
            "bad_secret",
            "the launcher's secret must be at least 32 bytes in base64url",
        )),
    }
}

const SESSIONS_FILE: &str = "sessions.json";

fn read_saved(file: &Path) -> Option<Saved> {
    serde_json::from_slice(&std::fs::read(file).ok()?).ok()
}

/// The file's modification time and length, to tell when something else changed it.
fn modified(file: &Path) -> Option<(SystemTime, u64)> {
    let meta = std::fs::metadata(file).ok()?;
    Some((meta.modified().ok()?, meta.len()))
}

/// Free the name `name` on the host server whose data folder is `dir` (see
/// [`super::DATA_DIR`]), so its user can claim it again from another PC: the claim's secret is
/// forgotten and the name's sessions end. A running server reads the change on its next request.
/// `false` when nobody had claimed the name.
pub fn release_name(dir: &Path, name: &str) -> io::Result<bool> {
    let file = dir.join(SESSIONS_FILE);
    let mut saved = match std::fs::read(&file) {
        Ok(bytes) => serde_json::from_slice::<Saved>(&bytes).map_err(io::Error::other)?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e),
    };
    let Some(id) = saved.holder(name).map(str::to_owned) else {
        return Ok(false);
    };
    if let Some(record) = saved.users.get_mut(&id) {
        record.secret_hash = None;
    }
    saved.sessions.retain(|_, u| u.id != id);
    let bytes = serde_json::to_vec_pretty(&saved).map_err(io::Error::other)?;
    super::careers::write_atomic(&file, &bytes)?;
    Ok(true)
}

/// A user's id from their name, so the same name signs in as the same player (and keeps their
/// careers) on any machine.
fn user_id(name: &str) -> String {
    let digest = Sha256::digest(name.trim().to_lowercase().as_bytes());
    format!("u-{}", &crate::baseline::hex(&digest)[..12])
}

fn lock(m: &Mutex<State>) -> std::sync::MutexGuard<'_, State> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl Lobby {
    pub(super) fn new(
        name: String,
        password: Option<String>,
        turn_secret: Vec<u8>,
        turn_port: u16,
        dir: &std::path::Path,
    ) -> Lobby {
        let file = dir.join(SESSIONS_FILE);
        let saved = read_saved(&file).unwrap_or_default();
        Lobby {
            name,
            password: password.filter(|p| !p.is_empty()),
            turn_secret,
            turn_port,
            state: Mutex::new(State {
                saved,
                stamp: modified(&file),
                ..State::default()
            }),
            file,
            careers: Careers::new(dir.join("careers")),
        }
    }

    fn save(&self, s: &mut State) {
        if let Ok(bytes) = serde_json::to_vec_pretty(&s.saved) {
            let _ = super::careers::write_atomic(&self.file, &bytes);
        }
        s.stamp = modified(&self.file);
    }

    /// Read `sessions.json` again when something else changed it since (the host releasing a
    /// name from the CLI while the server runs).
    fn fresh(&self, s: &mut State) {
        let stamp = modified(&self.file);
        if stamp.is_some() && stamp != s.stamp {
            if let Some(saved) = read_saved(&self.file) {
                s.saved = saved;
            }
            s.stamp = stamp;
        }
    }

    fn user(&self, req: &Request) -> Option<User> {
        let token = bearer(req)?;
        let mut s = lock(&self.state);
        self.fresh(&mut s);
        s.saved.sessions.get(&token_hash(token)).cloned()
    }

    /// Serve one connection: an HTTP request, or a websocket upgrade.
    pub(super) fn serve(self: &Arc<Self>, mut tls: TlsStream) {
        let Some((req, head)) = read_request(&mut tls) else {
            return;
        };
        let user = self.user(&req);
        if req.path.ends_with("/ws") && req.method == "GET" {
            let Some(user) = user else {
                return respond(
                    &mut tls,
                    401,
                    &json!({"error": "sign in", "reason": "signed_out"}),
                );
            };
            let code = req
                .path
                .trim_start_matches("/v1/mp/lobbies/")
                .trim_end_matches("/ws")
                .to_ascii_uppercase();
            let member = lock(&self.state)
                .rooms
                .get(&code)
                .is_some_and(|r| r.has(&user.id));
            if !member {
                return respond(
                    &mut tls,
                    404,
                    &json!({"error": "no open lobby has that code", "reason": "not_found"}),
                );
            }
            return self.websocket(tls, head, &code, &user);
        }
        let (status, body) = self.route(&req, user.as_ref());
        if status == 200 && body.get("__html").is_some() {
            return respond_html(&mut tls, body["__html"].as_str().unwrap_or(""));
        }
        respond(&mut tls, status, &body);
    }

    fn route(&self, req: &Request, user: Option<&User>) -> (u16, Value) {
        let path = req.path.split('?').next().unwrap_or("");
        match (req.method.as_str(), path) {
            ("GET", "/v1/host/info") => (
                200,
                json!({"name": self.name, "password": self.password.is_some(), "version": 2,
                       "identity": "secret"}),
            ),
            ("GET", "/v1/auth/launcher/start") => (
                200,
                json!({"__html": format!(
                    "<!doctype html><meta charset=utf-8><title>{name}</title><body style=\"font:16px system-ui\">\
                     <p>This is a F1 Multiplayer host server ({name}). Join it from the launcher's Multiplayer page \
                     with the host code the host gave you.</p>", name = html_escape(&self.name))}),
            ),
            ("POST", "/v1/auth/launcher/direct") => self.sign_in(&req.body),
            _ => {
                let Some(user) = user else {
                    return error(401, "signed_out", "sign in to this host server again");
                };
                self.route_signed_in(req, path, user)
            }
        }
    }

    fn sign_in(&self, body: &[u8]) -> (u16, Value) {
        let doc: Value = serde_json::from_slice(body).unwrap_or(Value::Null);
        let name: String = doc["name"]
            .as_str()
            .unwrap_or("")
            .trim()
            .chars()
            .filter(|c| !c.is_control())
            .take(NAME_MAX)
            .collect();
        if name.is_empty() {
            return error(400, "bad_name", "enter a driver name");
        }
        if let Some(password) = &self.password
            && doc["password"].as_str() != Some(password.as_str())
        {
            return error(403, "wrong_password", "the host server's password is wrong");
        }
        let secret = match secret_of(&doc) {
            Ok(s) => s,
            Err(e) => return e,
        };
        let token = random_text(32);
        let mut s = lock(&self.state);
        self.fresh(&mut s);
        let secret = secret.as_deref().map(token_hash).map(|h| (s.saved.by_secret(&h), h));
        let (id, claimed) = match secret {
            // (a) A known secret signs in as its user, under any name nobody else holds.
            Some((Some(id), _)) => {
                if s.saved.holder(&name).is_some_and(|h| h != id) {
                    return name_claimed();
                }
                if let Some(record) = s.saved.users.get_mut(&id) {
                    record.name = name.clone();
                }
                (id, true)
            }
            // (b) A new secret claims a name nobody holds; the name's older sessions go, as
            // they may be someone else's.
            Some((None, hash)) => {
                if s.saved.holder(&name).is_some() {
                    return name_claimed();
                }
                let id = user_id(&name);
                s.saved.sessions.retain(|_, u| u.id != id);
                s.saved.users.insert(
                    id.clone(),
                    UserRecord {
                        name: name.clone(),
                        secret_hash: Some(hash),
                        claimed_at: now_secs(),
                    },
                );
                (id, true)
            }
            // (d) An old launcher, under a claimed name.
            None if s.saved.holder(&name).is_some() => {
                return error(
                    409,
                    "secret_required",
                    "this name is claimed on this host server; update F1 Multiplayer to sign in",
                );
            }
            // (e) An old launcher under a free name: one session per name, as before.
            None => {
                let id = user_id(&name);
                s.saved.sessions.retain(|_, u| u.id != id);
                (id, false)
            }
        };
        let user = User { id, name, since: 0 };
        s.saved.add_session(&token, user.clone());
        self.save(&mut s);
        (
            200,
            json!({"token": token, "user": {"id": user.id, "name": user.name, "avatar": null},
                   "multiplayer_allowed": true, "claimed": claimed}),
        )
    }

    /// `POST /v1/auth/launcher/claim`: a user signed in from before claims their name with their
    /// launcher's secret. Their other sessions go, as they may be someone else's.
    fn claim(&self, req: &Request, user: &User) -> (u16, Value) {
        let doc: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
        let hash = match secret_of(&doc) {
            Ok(Some(secret)) => token_hash(&secret),
            Ok(None) => return error(400, "bad_secret", "a claim needs the launcher's secret"),
            Err(e) => return e,
        };
        let mut s = lock(&self.state);
        self.fresh(&mut s);
        if let Some(owner) = s.saved.by_secret(&hash) {
            return if owner == user.id {
                (200, json!({"claimed": true}))
            } else {
                name_claimed()
            };
        }
        if s.saved.holder(&user.name).is_some_and(|h| h != user.id)
            || s.saved.users.get(&user.id).is_some_and(|r| r.secret_hash.is_some())
        {
            return name_claimed();
        }
        s.saved.users.insert(
            user.id.clone(),
            UserRecord {
                name: user.name.clone(),
                secret_hash: Some(hash),
                claimed_at: now_secs(),
            },
        );
        let mine = bearer(req).map(token_hash);
        s.saved
            .sessions
            .retain(|k, u| u.id != user.id || Some(k) == mine.as_ref());
        self.save(&mut s);
        (200, json!({"claimed": true}))
    }

    fn route_signed_in(&self, req: &Request, path: &str, user: &User) -> (u16, Value) {
        match (req.method.as_str(), path) {
            ("GET", "/v1/auth/launcher/me") => (
                200,
                json!({"user": {"id": user.id, "name": user.name, "avatar": null},
                       "multiplayer_allowed": true, "multiplayer_known": true}),
            ),
            ("POST", "/v1/auth/launcher/claim") => self.claim(req, user),
            ("POST", "/v1/auth/launcher/logout") => {
                let mut s = lock(&self.state);
                self.fresh(&mut s);
                // Only this session ends: the user's other PCs stay signed in.
                if let Some(mine) = bearer(req).map(token_hash) {
                    s.saved.sessions.remove(&mine);
                }
                self.save(&mut s);
                (204, Value::Null)
            }
            ("POST", "/v1/mp/turn") => self.turn(req, user),
            ("GET", "/v1/mp/rooms") => (200, json!({"rooms": self.room_list()})),
            ("POST", "/v1/mp/lobbies") => self.create(&req.body, user),
            _ if path.starts_with("/v1/mp/careers") => self.careers.route(req, path, user_view(user)),
            _ => {
                let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
                let (code, action) = match parts.as_slice() {
                    ["v1", "mp", "lobbies", code] => (code.to_ascii_uppercase(), ""),
                    ["v1", "mp", "lobbies", code, action] => (code.to_ascii_uppercase(), *action),
                    _ => return error(404, "not_found", "no route"),
                };
                match (req.method.as_str(), action) {
                    ("GET", "") => self.get_lobby(&code, user),
                    ("POST", "join") => self.join(&code, user),
                    ("POST", "leave") => self.leave(&code, user),
                    _ => error(404, "not_found", "no route"),
                }
            }
        }
    }

    fn turn(&self, req: &Request, user: &User) -> (u16, Value) {
        let host = req
            .headers
            .get("host")
            .map(|h| host_of(h).to_owned())
            .unwrap_or_else(|| "127.0.0.1".into());
        let (username, credential, expires) = super::turn::credentials(&self.turn_secret, &user.id, TURN_TTL);
        let host = if host.contains(':') {
            format!("[{host}]")
        } else {
            host
        };
        (
            200,
            json!({
                "iceServers": [{
                    "urls": [format!("turn:{host}:{}?transport=udp", self.turn_port)],
                    "username": username,
                    "credential": credential,
                }],
                "expires_at": expires,
            }),
        )
    }

    fn room_list(&self) -> Vec<Value> {
        let s = lock(&self.state);
        let mut rooms: Vec<&Room> = s.rooms.values().filter(|r| r.public && !r.started).collect();
        rooms.sort_by_key(|r| r.created_at);
        rooms
            .iter()
            .map(|r| {
                let host = r.members.iter().find(|m| m.user.id == r.host);
                json!({
                    "code": r.code, "name": r.name,
                    "host": {"id": r.host, "name": host.map_or("Host", |h| h.user.name.as_str()), "avatar": null},
                    "members": r.members.len(), "max_members": r.max, "category": r.category,
                    "digest": host.and_then(|h| h.digest.clone()), "addons": host.and_then(|h| room_addons(h.blob.as_ref()?)),
                    "created_at": r.created_at,
                })
            })
            .collect()
    }

    fn create(&self, body: &[u8], user: &User) -> (u16, Value) {
        let mut s = lock(&self.state);
        if let Some(code) = s.member_of.get(&user.id) {
            return (
                409,
                json!({"error": format!("you are already in lobby {code}; leave it first"),
                       "reason": "in_lobby", "lobby": code}),
            );
        }
        let settings: Value = serde_json::from_slice(body).unwrap_or(json!({}));
        let code = loop {
            let mut raw = [0u8; 6];
            crate::mpauth::getrandom(&mut raw);
            let tail: String = raw
                .iter()
                .map(|b| CODE_ALPHABET[usize::from(*b) % CODE_ALPHABET.len()] as char)
                .collect();
            let code = format!("F1-{tail}");
            if !s.rooms.contains_key(&code) {
                break code;
            }
        };
        let name: String = settings["name"]
            .as_str()
            .map_or_else(|| format!("{}'s room", user.name), str::to_owned)
            .chars()
            .take(48)
            .collect();
        let room = Room {
            code: code.clone(),
            host: user.id.clone(),
            name,
            public: settings["public"].as_bool().unwrap_or(true),
            category: settings["category"].as_str().unwrap_or("f1-2026").to_owned(),
            max: settings["max_members"].as_u64().unwrap_or(20).clamp(2, 24),
            started: false,
            rev: 0,
            created_at: now_secs(),
            key: random_text(32),
            members: vec![Member {
                user: user.clone(),
                ip: 1,
                connected: false,
                ready: false,
                digest: None,
                blob: None,
                relay: None,
            }],
        };
        let lobby = view(&room);
        s.rooms.insert(code.clone(), room);
        s.member_of.insert(user.id.clone(), code);
        (201, json!({ "lobby": lobby }))
    }

    fn get_lobby(&self, code: &str, user: &User) -> (u16, Value) {
        let s = lock(&self.state);
        match s.rooms.get(code).filter(|r| r.has(&user.id)) {
            Some(room) => (200, json!({ "lobby": view(room) })),
            None => error(404, "not_found", "no open lobby has that code"),
        }
    }

    fn join(&self, code: &str, user: &User) -> (u16, Value) {
        let mut s = lock(&self.state);
        if let Some(other) = s.member_of.get(&user.id).filter(|c| *c != code) {
            return (
                409,
                json!({"error": format!("you are already in lobby {other}; leave it first"),
                       "reason": "in_lobby", "lobby": other}),
            );
        }
        let Some(room) = s.rooms.get_mut(code) else {
            return error(404, "not_found", "no open lobby has that code");
        };
        if !room.has(&user.id) {
            if room.started {
                return error(409, "started", "this room is in a race; join after it ends");
            }
            if room.members.len() as u64 >= room.max {
                return error(409, "full", "this room is full");
            }
            let Some(ip) = room.next_ip() else {
                return error(409, "full", "this room is full");
            };
            room.members.push(Member {
                user: user.clone(),
                ip,
                connected: false,
                ready: false,
                digest: None,
                blob: None,
                relay: None,
            });
            s.member_of.insert(user.id.clone(), code.to_owned());
        }
        broadcast(&mut s, code);
        let lobby = s.rooms.get(code).map(view).unwrap_or(Value::Null);
        (200, json!({ "lobby": lobby }))
    }

    fn leave(&self, code: &str, user: &User) -> (u16, Value) {
        let mut s = lock(&self.state);
        let Some(room) = s.rooms.get_mut(code).filter(|r| r.has(&user.id)) else {
            return error(404, "not_found", "no open lobby has that code");
        };
        if room.host == user.id {
            close_room(&mut s, code, "host_left");
        } else {
            room.members.retain(|m| m.user.id != user.id);
            s.member_of.remove(&user.id);
            if let Some(tx) = s.sockets.remove(&user.id) {
                let _ = tx.send(Out::Text(json!({"type": "left"}).to_string()));
                let _ = tx.send(Out::Close(4003));
            }
            broadcast(&mut s, code);
        }
        (204, Value::Null)
    }

    fn websocket(self: &Arc<Self>, tls: TlsStream, head: Vec<u8>, code: &str, user: &User) {
        let replay = Replay {
            head: Cursor::new(head),
            inner: tls,
        };
        let Ok(mut ws) = tungstenite::accept(replay) else {
            return;
        };
        // The loop below sends what the room queued between reads, so a read must give up quickly.
        // Set on the socket the TLS stream reads, not on a clone: on Windows a cloned socket's timeout
        // doesn't reach it, and the 10 s timeout of the request read held every lobby update back
        // until the client spoke.
        if ws.get_ref().inner.sock.set_read_timeout(Some(Duration::from_millis(20))).is_err() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        let connection;
        {
            let mut s = lock(&self.state);
            s.next_connection += 1;
            connection = s.next_connection;
            s.connections.insert(user.id.clone(), connection);
            if let Some(old) = s.sockets.insert(user.id.clone(), tx.clone()) {
                let _ = old.send(Out::Text(json!({"type": "replaced"}).to_string()));
                let _ = old.send(Out::Close(4004));
            }
            if let Some(room) = s.rooms.get_mut(code) {
                for m in room.members.iter_mut().filter(|m| m.user.id == user.id) {
                    m.connected = true;
                }
                let prints: Vec<String> = room
                    .members
                    .iter()
                    .filter(|m| m.user.id != user.id)
                    .filter_map(|m| {
                        Some(
                            json!({"type": "fingerprint", "member": m.user.id,
                                   "digest": m.digest.as_ref()?, "blob": m.blob.as_ref()?})
                            .to_string(),
                        )
                    })
                    .collect();
                for p in prints {
                    let _ = tx.send(Out::Text(p));
                }
            }
            broadcast(&mut s, code);
        }
        'conn: loop {
            while let Ok(out) = rx.try_recv() {
                match out {
                    Out::Text(text) => {
                        if ws.send(Message::text(text)).is_err() {
                            break 'conn;
                        }
                    }
                    Out::Close(c) => {
                        let _ = ws.close(Some(tungstenite::protocol::CloseFrame {
                            code: c.into(),
                            reason: "".into(),
                        }));
                        let _ = ws.flush();
                        return;
                    }
                }
            }
            match ws.read() {
                Ok(Message::Text(text)) => {
                    let msg: Value = serde_json::from_str(text.as_str()).unwrap_or(Value::Null);
                    self.handle(code, user, &msg);
                }
                Ok(Message::Close(_)) => break,
                Ok(_) => {}
                Err(tungstenite::Error::Io(e))
                    if matches!(e.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut) => {}
                Err(_) => break,
            }
        }
        let mut s = lock(&self.state);
        let ours = s.connections.get(&user.id) == Some(&connection);
        if ours {
            s.connections.remove(&user.id);
            s.sockets.remove(&user.id);
            if let Some(room) = s.rooms.get_mut(code) {
                for m in room.members.iter_mut().filter(|m| m.user.id == user.id) {
                    m.connected = false;
                    m.ready = false;
                }
            }
            broadcast(&mut s, code);
        }
    }

    fn handle(&self, code: &str, user: &User, msg: &Value) {
        let mut s = lock(&self.state);
        let Some(room) = s.rooms.get_mut(code) else {
            return;
        };
        let is_host = room.host == user.id;
        match msg["type"].as_str().unwrap_or("") {
            "fingerprint" => {
                let digest = msg["digest"].as_str().map(str::to_owned);
                let blob = msg.get("blob").cloned();
                if let Some(m) = room.members.iter_mut().find(|m| m.user.id == user.id) {
                    m.digest = digest.clone();
                    m.blob = blob.clone();
                }
                let out = json!({"type": "fingerprint", "member": user.id, "digest": digest, "blob": blob})
                    .to_string();
                let others: Vec<String> = room
                    .members
                    .iter()
                    .filter(|m| m.user.id != user.id)
                    .map(|m| m.user.id.clone())
                    .collect();
                for id in others {
                    if let Some(tx) = s.sockets.get(&id) {
                        let _ = tx.send(Out::Text(out.clone()));
                    }
                }
                broadcast(&mut s, code);
            }
            "ready" => {
                let ready = msg["ready"].as_bool().unwrap_or(false);
                let Some(m) = room.members.iter_mut().find(|m| m.user.id == user.id) else {
                    return;
                };
                if m.ready != ready {
                    m.ready = ready;
                    broadcast(&mut s, code);
                }
            }
            "relay" => {
                if let Some(m) = room.members.iter_mut().find(|m| m.user.id == user.id) {
                    m.relay = msg["relay"].as_str().map(|r| r.chars().take(64).collect());
                }
                broadcast(&mut s, code);
            }
            "start" if is_host && !room.started => {
                room.started = true;
                let ips: serde_json::Map<String, Value> = room
                    .members
                    .iter()
                    .map(|m| (m.user.id.clone(), Value::String(format!("10.77.0.{}", m.ip))))
                    .collect();
                let out =
                    json!({"type": "start", "host_ip": "10.77.0.1", "ips": ips, "key": room.key}).to_string();
                let ids: Vec<String> = room.members.iter().map(|m| m.user.id.clone()).collect();
                broadcast(&mut s, code);
                for id in ids {
                    if let Some(tx) = s.sockets.get(&id) {
                        let _ = tx.send(Out::Text(out.clone()));
                    }
                }
            }
            "reopen" if is_host && room.started => {
                room.started = false;
                for m in &mut room.members {
                    m.ready = false;
                }
                broadcast(&mut s, code);
            }
            "kick" if is_host => {
                let Some(target) = msg["member"]
                    .as_str()
                    .filter(|t| *t != user.id)
                    .map(str::to_owned)
                else {
                    return;
                };
                room.members.retain(|m| m.user.id != target);
                s.member_of.remove(&target);
                if let Some(tx) = s.sockets.remove(&target) {
                    let _ = tx.send(Out::Text(json!({"type": "kicked"}).to_string()));
                    let _ = tx.send(Out::Close(4005));
                }
                broadcast(&mut s, code);
            }
            "ping" => {
                if let Some(tx) = s.sockets.get(&user.id) {
                    let _ = tx.send(Out::Text(json!({"type": "pong"}).to_string()));
                }
            }
            _ => {}
        }
    }

    /// Close every room, as the server stops.
    pub(super) fn close_all(&self) {
        let mut s = lock(&self.state);
        let codes: Vec<String> = s.rooms.keys().cloned().collect();
        for code in codes {
            close_room(&mut s, &code, "server_stopped");
        }
    }

    pub(super) fn room_count(&self) -> usize {
        lock(&self.state).rooms.len()
    }
}

fn user_view(user: &User) -> (&str, &str) {
    (&user.id, &user.name)
}

fn close_room(s: &mut State, code: &str, reason: &str) {
    let Some(room) = s.rooms.remove(code) else {
        return;
    };
    for m in &room.members {
        s.member_of.remove(&m.user.id);
        if let Some(tx) = s.sockets.remove(&m.user.id) {
            let _ = tx.send(Out::Text(json!({"type": "closed", "reason": reason}).to_string()));
            let _ = tx.send(Out::Close(4002));
        }
    }
}

/// The room card's addons: every addon the host's fingerprint lists, by id and name.
fn room_addons(blob: &Value) -> Option<Value> {
    let list: Vec<Value> = blob["addons"]
        .as_array()?
        .iter()
        .filter_map(|a| {
            let id = a["id"].as_str()?;
            Some(json!({"id": id, "name": a["name"].as_str().unwrap_or(id)}))
        })
        .collect();
    Some(json!({"count": list.len(), "tracks": [], "list": list}))
}

fn view(room: &Room) -> Value {
    let host_digest = room
        .members
        .iter()
        .find(|m| m.user.id == room.host)
        .and_then(|h| h.digest.clone());
    json!({
        "code": room.code, "host": room.host, "state": if room.started { "started" } else { "open" },
        "rev": room.rev, "name": room.name, "public": room.public, "category": room.category,
        "created_at": room.created_at, "max_members": room.max, "can_start": !room.started, "blocker": null,
        "key": room.key,
        "members": room.members.iter().map(|m| json!({
            "id": m.user.id, "name": m.user.name, "avatar": null, "ip": format!("10.77.0.{}", m.ip),
            "host": m.user.id == room.host, "ready": m.ready, "connected": m.connected,
            "digest": m.digest, "matches": m.digest.is_some() && m.digest == host_digest, "relay": m.relay,
        })).collect::<Vec<_>>(),
    })
}

fn broadcast(s: &mut State, code: &str) {
    let Some(room) = s.rooms.get_mut(code) else {
        return;
    };
    room.rev += 1;
    let msg = json!({"type": "lobby", "lobby": view(room)}).to_string();
    let ids: Vec<String> = room.members.iter().map(|m| m.user.id.clone()).collect();
    for id in ids {
        if let Some(tx) = s.sockets.get(&id) {
            let _ = tx.send(Out::Text(msg.clone()));
        }
    }
}

/// The host of `host[:port]` or `[v6]:port`, without brackets.
fn host_of(authority: &str) -> &str {
    if let Some(v6) = authority.strip_prefix('[') {
        return v6.split(']').next().unwrap_or("");
    }
    authority.split(':').next().unwrap_or("")
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Bytes already read from a stream, then the stream.
struct Replay {
    head: Cursor<Vec<u8>>,
    inner: TlsStream,
}

impl Read for Replay {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if (self.head.position() as usize) < self.head.get_ref().len() {
            return self.head.read(buf);
        }
        self.inner.read(buf)
    }
}

impl Write for Replay {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// Read one request. Returns it with the raw bytes read (head and body) for a websocket upgrade.
fn read_request(tls: &mut TlsStream) -> Option<(Request, Vec<u8>)> {
    let mut raw = Vec::new();
    let mut buf = [0u8; 8192];
    let head_end = loop {
        if let Some(i) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
            break i + 4;
        }
        if raw.len() > HEAD_LIMIT {
            return None;
        }
        match tls.read(&mut buf) {
            Ok(0) | Err(_) => return None,
            Ok(n) => raw.extend_from_slice(&buf[..n]),
        }
    };
    let head = String::from_utf8_lossy(&raw[..head_end]).into_owned();
    let mut lines = head.lines();
    let mut first = lines.next().unwrap_or("").split_whitespace();
    let (method, path) = (
        first.next().unwrap_or("").to_owned(),
        first.next().unwrap_or("").to_owned(),
    );
    let headers: HashMap<String, String> = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_owned()))
        .collect();
    let length: usize = headers
        .get("content-length")
        .and_then(|l| l.parse().ok())
        .unwrap_or(0);
    if length > BODY_LIMIT {
        return None;
    }
    while raw.len() < head_end + length {
        match tls.read(&mut buf) {
            Ok(0) | Err(_) => return None,
            Ok(n) => raw.extend_from_slice(&buf[..n]),
        }
    }
    let body = raw[head_end..head_end + length].to_vec();
    Some((
        Request {
            method,
            path,
            headers,
            body,
        },
        raw,
    ))
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        409 => "Conflict",
        413 => "Payload Too Large",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        _ => "Error",
    }
}

fn respond(tls: &mut TlsStream, status: u16, body: &Value) {
    let text = if status == 204 {
        String::new()
    } else {
        body.to_string()
    };
    let answer = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
        reason_phrase(status),
        text.len()
    );
    let _ = tls.write_all(answer.as_bytes());
    let _ = tls.flush();
    tls.conn.send_close_notify();
    let _ = tls.flush();
}

fn respond_html(tls: &mut TlsStream, html: &str) {
    let answer = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{html}",
        html.len()
    );
    let _ = tls.write_all(answer.as_bytes());
    let _ = tls.flush();
    tls.conn.send_close_notify();
    let _ = tls.flush();
}

/// Serve every connection from `listener` on its own thread until `stop`.
pub(super) fn accept_loop(
    lobby: Arc<Lobby>,
    listener: std::net::TcpListener,
    config: Arc<rustls::ServerConfig>,
    stop: Arc<std::sync::atomic::AtomicBool>,
) {
    use std::sync::atomic::Ordering;
    let _ = listener.set_nonblocking(true);
    while !stop.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((tcp, _)) => {
                let (lobby, config) = (lobby.clone(), config.clone());
                let _ = thread::Builder::new().name("host-conn".into()).spawn(move || {
                    let _ = tcp.set_nonblocking(false);
                    let _ = tcp.set_read_timeout(Some(Duration::from_secs(10)));
                    let _ = tcp.set_write_timeout(Some(Duration::from_secs(10)));
                    let _ = tcp.set_nodelay(true);
                    let Ok(conn) = rustls::ServerConnection::new(config) else {
                        return;
                    };
                    lobby.serve(rustls::StreamOwned::new(conn, tcp));
                });
            }
            Err(_) => thread::sleep(Duration::from_millis(20)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lobby(tag: &str) -> (Lobby, PathBuf) {
        let dir = std::env::temp_dir().join(format!("f1mp-lobby-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        (Lobby::new("Test".into(), None, vec![0; 32], 1, &dir), dir)
    }

    fn secret(n: u8) -> String {
        crate::mpauth::base64url(&[n; 32])
    }

    fn sign(l: &Lobby, name: &str, secret: Option<&str>) -> (u16, Value) {
        let body = match secret {
            Some(s) => json!({"name": name, "secret": s}),
            None => json!({"name": name}),
        };
        l.sign_in(&serde_json::to_vec(&body).unwrap())
    }

    fn req(method: &str, path: &str, token: &str, body: Value) -> Request {
        Request {
            method: method.into(),
            path: path.into(),
            headers: HashMap::from([("authorization".to_owned(), format!("Bearer {token}"))]),
            body: serde_json::to_vec(&body).unwrap(),
        }
    }

    /// `GET /me` with `token`: its status and the user's id.
    fn me(l: &Lobby, token: &str) -> (u16, Option<String>) {
        let r = req("GET", "/v1/auth/launcher/me", token, Value::Null);
        let user = l.user(&r);
        let (status, body) = l.route(&r, user.as_ref());
        (status, body["user"]["id"].as_str().map(str::to_owned))
    }

    fn claim(l: &Lobby, token: &str, secret: &str) -> (u16, Value) {
        let r = req("POST", "/v1/auth/launcher/claim", token, json!({ "secret": secret }));
        let user = l.user(&r);
        l.route(&r, user.as_ref())
    }

    fn token(answer: &(u16, Value)) -> String {
        answer.1["token"].as_str().unwrap().to_owned()
    }

    #[test]
    fn a_secret_claims_the_name_and_keeps_the_old_id() {
        let (l, dir) = lobby("claim");
        let legacy = sign(&l, "Max", None);
        assert_eq!((legacy.0, legacy.1["claimed"].as_bool()), (200, Some(false)));
        let old_id = legacy.1["user"]["id"].as_str().unwrap().to_owned();
        let claimed = sign(&l, "Max", Some(&secret(1)));
        assert_eq!(claimed.0, 200);
        assert_eq!(claimed.1["user"]["id"].as_str(), Some(old_id.as_str()));
        assert_eq!(claimed.1["claimed"].as_bool(), Some(true));
        // The legacy session may have been someone else's: it ends.
        assert_eq!(me(&l, &token(&legacy)).0, 401);
        assert_eq!(me(&l, &token(&claimed)), (200, Some(old_id.clone())));

        let other = sign(&l, "Max", Some(&secret(2)));
        assert_eq!((other.0, other.1["reason"].as_str()), (409, Some("name_claimed")));
        let old = sign(&l, "Max", None);
        assert_eq!((old.0, old.1["reason"].as_str()), (409, Some("secret_required")));
        let bad = sign(&l, "Max", Some("short"));
        assert_eq!((bad.0, bad.1["reason"].as_str()), (400, Some("bad_secret")));
        // The claim survives a restart.
        drop(l);
        let l = Lobby::new("Test".into(), None, vec![0; 32], 1, &dir);
        assert_eq!(sign(&l, "Max", Some(&secret(2))).0, 409);
        assert_eq!(me(&l, &token(&claimed)), (200, Some(old_id)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn names_are_claimed_case_insensitively() {
        let (l, dir) = lobby("case");
        assert_eq!(sign(&l, "Max", Some(&secret(1))).0, 200);
        let other = sign(&l, " max ", Some(&secret(2)));
        assert_eq!((other.0, other.1["reason"].as_str()), (409, Some("name_claimed")));
        // The holder signs in under another spelling, and under a new name nobody holds; the
        // new name is then held too, and the id stays.
        let again = sign(&l, "MAX", Some(&secret(1)));
        assert_eq!(again.0, 200);
        let renamed = sign(&l, "Maxi", Some(&secret(1)));
        assert_eq!(renamed.1["user"]["id"], again.1["user"]["id"]);
        assert_eq!(sign(&l, "maxi", None).1["reason"].as_str(), Some("secret_required"));
        assert_eq!(sign(&l, "Ben", Some(&secret(3))).0, 200);
        assert_eq!(sign(&l, "Ben", Some(&secret(1))).1["reason"].as_str(), Some("name_claimed"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_same_secret_keeps_other_sessions_up_to_eight() {
        let (l, dir) = lobby("eight");
        let tokens: Vec<String> = (0..SESSIONS_PER_USER + 1)
            .map(|_| token(&sign(&l, "Max", Some(&secret(1)))))
            .collect();
        assert_eq!(me(&l, &tokens[0]).0, 401, "the oldest session goes");
        for t in &tokens[1..] {
            assert_eq!(me(&l, t).0, 200);
        }
        // Signing out ends only that session.
        let r = req("POST", "/v1/auth/launcher/logout", &tokens[1], Value::Null);
        let user = l.user(&r);
        assert_eq!(l.route(&r, user.as_ref()).0, 204);
        assert_eq!(me(&l, &tokens[1]).0, 401);
        assert_eq!(me(&l, &tokens[2]).0, 200);
        // Old launchers keep one session per name, as before.
        let a = token(&sign(&l, "Ben", None));
        let b = token(&sign(&l, "Ben", None));
        assert_eq!((me(&l, &a).0, me(&l, &b).0), (401, 200));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_signed_in_user_claims_with_their_token() {
        let (l, dir) = lobby("token");
        let mine = token(&sign(&l, "Max", None));
        let theirs = token(&sign(&l, "Ana", None));
        let (s, body) = claim(&l, &mine, &secret(1));
        assert_eq!((s, body["claimed"].as_bool()), (200, Some(true)));
        assert_eq!(claim(&l, &mine, &secret(1)).0, 200, "claiming again is fine");
        assert_eq!(me(&l, &mine).0, 200, "the claiming session stays");
        let again = sign(&l, "Max", Some(&secret(1)));
        assert_eq!((again.0, again.1["claimed"].as_bool()), (200, Some(true)));
        assert_eq!(sign(&l, "Max", None).1["reason"].as_str(), Some("secret_required"));
        assert_eq!(me(&l, &theirs).0, 200);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn claim_fails_when_someone_else_holds_the_name() {
        let (l, dir) = lobby("taken");
        let legacy = token(&sign(&l, "Max", None));
        assert_eq!(sign(&l, "Max", Some(&secret(1))).0, 200);
        // The legacy session ended with the claim; a session from before it can't claim either.
        assert_eq!(claim(&l, &legacy, &secret(2)).0, 401);
        let ana = token(&sign(&l, "Ana", None));
        let (s, body) = claim(&l, &ana, &secret(1));
        assert_eq!((s, body["reason"].as_str()), (409, Some("name_claimed")), "Max's secret isn't Ana's");
        assert_eq!(claim(&l, &ana, &secret(2)).0, 200);
        assert_eq!(claim(&l, &ana, &secret(3)).1["reason"].as_str(), Some("name_claimed"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_released_name_can_be_claimed_again() {
        let (l, dir) = lobby("release");
        let first = token(&sign(&l, "Max", Some(&secret(1))));
        let id = me(&l, &first).1;
        assert!(!release_name(&dir, "Nobody").unwrap());
        assert!(release_name(&dir, "max").unwrap());
        // The running server reads the change: the old sessions end, and a new PC claims.
        assert_eq!(me(&l, &first).0, 401);
        let second = sign(&l, "Max", Some(&secret(2)));
        assert_eq!(second.0, 200);
        assert_eq!(me(&l, &token(&second)).1, id, "the id stays, so the careers do");
        assert_eq!(sign(&l, "Max", Some(&secret(1))).1["reason"].as_str(), Some("name_claimed"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn sessions_from_before_keep_working() {
        let (_, dir) = lobby("upgrade");
        let token = "old-token";
        std::fs::write(
            dir.join(SESSIONS_FILE),
            json!({"sessions": {token_hash(token): {"id": "u-1", "name": "Max"}}}).to_string(),
        )
        .unwrap();
        let l = Lobby::new("Test".into(), None, vec![0; 32], 1, &dir);
        assert_eq!(me(&l, token), (200, Some("u-1".to_owned())));
        let r = req("GET", "/v1/host/info", token, Value::Null);
        let (s, info) = l.route(&r, None);
        assert_eq!((s, info["version"].as_u64(), info["identity"].as_str()), (200, Some(2), Some("secret")));
        let _ = std::fs::remove_dir_all(&dir);
    }
}