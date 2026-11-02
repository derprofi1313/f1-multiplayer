//! Launcher side of the Discord sign-in.
//!
//! The server (`/v1/auth/launcher/*`) runs the OAuth code exchange; the launcher
//! only opens the browser and catches the redirect back to a loopback port. The
//! GPUI layer supplies the browser-open (`cx.open_url`) and runs [`sign_in`] off
//! the UI thread; everything here is plain blocking I/O.

use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

const WAIT: Duration = Duration::from_secs(180);
const DONE_PAGE: &str = "<!doctype html><meta charset=utf-8><title>Signed in</title>\
<body style=\"font:16px system-ui;display:grid;place-items:center;height:100vh;margin:0;\
background:#0b0b0d;color:#eee\"><div>Signed in to F1 Multiplayer. You can close this tab.</div>";

/// A persisted sign-in. `allowed` is whether the server cleared this user for
/// multiplayer (beta-tester role); it is re-checked on each launch via [`refresh`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub name: String,
    pub allowed: bool,
    /// The host code of the host server this sign-in belongs to (see
    /// [`crate::multiplayer::host`]); `None` for the launcher's own server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
}

impl Session {
    /// The host server this sign-in belongs to.
    pub fn endpoint(&self) -> Option<crate::multiplayer::pinned::Endpoint> {
        crate::multiplayer::pinned::Endpoint::parse(self.server.as_deref()?).ok()
    }
}

/// Sign in to the host server with host code `code` (or its six-digit server code, see
/// [`crate::multiplayer::servercode`]) under the driver name `name`, with the server's `password`
/// when it has one, and persist the session.
pub fn sign_in_host(
    data_dir: &Path,
    code: &str,
    name: &str,
    password: Option<&str>,
) -> Result<Session, String> {
    let endpoint = match crate::multiplayer::servercode::parse(code) {
        Some(short) => crate::multiplayer::servercode::resolve(&short)?,
        None => crate::multiplayer::pinned::Endpoint::parse(code)?,
    };
    let secret = host_secret(data_dir, &endpoint.pin);
    let body = serde_json::json!({ "name": name, "password": password, "secret": secret }).to_string();
    let (status, text) = crate::multiplayer::pinned::request(
        &endpoint,
        "POST",
        "/v1/auth/launcher/direct",
        None,
        Some(&body),
        Duration::from_secs(15),
    )
    .map_err(|e| format!("couldn't reach the host server {}: {e}", endpoint.authority()))?;
    let doc: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
    if status != 200 {
        return Err(doc["error"].as_str().map_or_else(
            || format!("the host server refused the sign-in (HTTP {status})"),
            str::to_owned,
        ));
    }
    let session = Session {
        token: doc["token"]
            .as_str()
            .ok_or("the host server sent no session token")?
            .to_owned(),
        name: doc["user"]["name"].as_str().unwrap_or(name).to_owned(),
        allowed: true,
        server: Some(endpoint.code()),
    };
    if doc["claimed"].as_bool() == Some(true) {
        mark_claimed(data_dir, &endpoint.pin);
    }
    save(data_dir, &session).map_err(|e| format!("couldn't save the sign-in: {e}"))?;
    Ok(session)
}

/// This launcher's secret for one host server (by its certificate pin), made once and kept in
/// `<data>/multiplayer/host-keys.json`: the server lets only this secret sign in under the name
/// it claimed (see [`crate::multiplayer::host`]). Signing out keeps it.
pub fn host_secret(data_dir: &Path, pin: &[u8; 32]) -> String {
    let mut keys = load_keys(data_dir);
    let id = base64url(pin);
    if let Some(key) = keys.get(&id).filter(|k| k.secret.len() >= 43) {
        return key.secret.clone();
    }
    let mut raw = [0u8; 32];
    secure_random(&mut raw);
    let secret = base64url(&raw);
    keys.insert(
        id,
        HostKey {
            secret: secret.clone(),
            claimed: false,
        },
    );
    if let Err(e) = save_keys(data_dir, &keys) {
        crate::multiplayer::diag::line(format!("couldn't keep the host server key: {e}"));
    }
    secret
}

/// Claim the session's name on its host server with this launcher's secret, once (a sign-in from
/// before host servers knew secrets). `Ok(true)` once the name is claimed, `Ok(false)` for the
/// launcher's own server or a host server that doesn't know claims yet.
pub fn claim_host_name(data_dir: &Path, session: &Session) -> Result<bool, String> {
    let Some(endpoint) = session.endpoint() else {
        return Ok(false);
    };
    if load_keys(data_dir)
        .get(&base64url(&endpoint.pin))
        .is_some_and(|k| k.claimed)
    {
        return Ok(true);
    }
    let body = serde_json::json!({ "secret": host_secret(data_dir, &endpoint.pin) }).to_string();
    let (status, text) = crate::multiplayer::pinned::request(
        &endpoint,
        "POST",
        "/v1/auth/launcher/claim",
        Some(&session.token),
        Some(&body),
        Duration::from_secs(15),
    )
    .map_err(|e| format!("couldn't reach the host server {}: {e}", endpoint.authority()))?;
    match status {
        200 => {
            mark_claimed(data_dir, &endpoint.pin);
            Ok(true)
        }
        404 => Ok(false),
        _ => {
            let doc: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
            Err(doc["error"].as_str().map_or_else(
                || format!("the host server refused the name claim (HTTP {status})"),
                str::to_owned,
            ))
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct HostKey {
    secret: String,
    #[serde(default)]
    claimed: bool,
}

fn keys_path(data_dir: &Path) -> PathBuf {
    data_dir.join("multiplayer").join("host-keys.json")
}

fn load_keys(data_dir: &Path) -> std::collections::BTreeMap<String, HostKey> {
    std::fs::read(keys_path(data_dir))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

fn save_keys(data_dir: &Path, keys: &std::collections::BTreeMap<String, HostKey>) -> std::io::Result<()> {
    let path = keys_path(data_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(keys).unwrap_or_default())?;
    std::fs::rename(&tmp, &path)
}

fn mark_claimed(data_dir: &Path, pin: &[u8; 32]) {
    let mut keys = load_keys(data_dir);
    if let Some(key) = keys.get_mut(&base64url(pin))
        && !key.claimed
    {
        key.claimed = true;
        if let Err(e) = save_keys(data_dir, &keys) {
            crate::multiplayer::diag::line(format!("couldn't keep the host server key: {e}"));
        }
    }
}

/// Bytes from the operating system's random source (falls back to [`getrandom`]).
fn secure_random(buf: &mut [u8]) {
    use ring::rand::SecureRandom;
    if ring::rand::SystemRandom::new().fill(buf).is_err() {
        getrandom(buf);
    }
}

fn base_url() -> String {
    std::env::var(crate::catalog::URL_ENV)
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| crate::catalog::DEFAULT_URL.to_owned())
        .trim_end_matches('/')
        .to_owned()
}

fn session_path(data_dir: &Path) -> PathBuf {
    data_dir.join("multiplayer").join("session.json")
}

/// Load the saved session, if any.
pub fn load(data_dir: &Path) -> Option<Session> {
    let text = std::fs::read_to_string(session_path(data_dir)).ok()?;
    serde_json::from_str(&text).ok()
}

fn save(data_dir: &Path, session: &Session) -> std::io::Result<()> {
    let path = session_path(data_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_vec_pretty(session).unwrap_or_default())
}

/// Drop the saved session. The host server keys (see [`host_secret`]) stay: they are what proves
/// this PC's claim to its name.
fn forget(data_dir: &Path) {
    let _ = std::fs::remove_file(session_path(data_dir));
}

fn query_pairs(request_line: &str) -> Vec<(String, String)> {
    let path = request_line.split_whitespace().nth(1).unwrap_or("");
    let query = path.split_once('?').map(|(_, q)| q).unwrap_or("");
    query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .map(|(k, v)| (k.to_owned(), urldecode(v)))
        .collect()
}

fn urldecode(s: &str) -> String {
    let raw = s.replace('+', " ");
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = (bytes[i] == b'%' && i + 2 < bytes.len())
            .then(|| u8::from_str_radix(&raw[i + 1..i + 3], 16).ok())
            .flatten();
        match hex {
            Some(byte) => {
                out.push(byte);
                i += 3;
            }
            None => {
                out.push(bytes[i]);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Run the full sign-in: open the browser at the server's start URL and wait for
/// the loopback redirect, then persist and return the session. `open` is called
/// with the URL to open; it must not block (e.g. hand it to `cx.open_url`).
/// A loopback waiting for the sign-in redirect. [`begin`] returns it together
/// with the URL to open in the browser; [`Pending::wait`] blocks for the result,
/// so the browser-open can happen on the UI thread and the wait on a worker.
pub struct Pending {
    listener: TcpListener,
    nonce: String,
}

/// Open a loopback port and build the URL the browser should visit. Call this on
/// the UI thread, open the returned URL, then run [`Pending::wait`] off-thread.
pub fn begin() -> Result<(Pending, String), String> {
    let listener =
        TcpListener::bind("127.0.0.1:0").map_err(|e| format!("couldn't open a local port: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("couldn't read the local port: {e}"))?
        .port();
    listener.set_nonblocking(true).map_err(|e| format!("{e}"))?;
    let nonce = {
        let mut raw = [0u8; 18];
        getrandom(&mut raw);
        base64url(&raw)
    };
    let url = format!("{}/v1/auth/launcher/start?port={port}&nonce={nonce}", base_url());
    Ok((Pending { listener, nonce }, url))
}

/// Run the whole flow on one thread: open a port, hand the URL to `open`, and
/// block for the result. Convenient for the CLI; the GPUI layer uses
/// [`begin`] + [`Pending::wait`] instead.
pub fn sign_in(data_dir: &Path, open: impl FnOnce(&str)) -> Result<Session, String> {
    let (pending, url) = begin()?;
    open(&url);
    pending.wait(data_dir)
}

impl Pending {
    /// Block until the browser hits the loopback with a matching nonce, persist
    /// the session, and return it.
    pub fn wait(self, data_dir: &Path) -> Result<Session, String> {
        let Pending { listener, nonce } = self;
        let deadline = Instant::now() + WAIT;
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut buf = [0u8; 4096];
                    let n = stream.read(&mut buf).unwrap_or(0);
                    let request = String::from_utf8_lossy(&buf[..n]);
                    let first = request.lines().next().unwrap_or("");
                    let pairs = query_pairs(first);
                    let get = |k: &str| pairs.iter().find(|(p, _)| p == k).map(|(_, v)| v.clone());

                    let ok = get("nonce").as_deref() == Some(nonce.as_str());
                    let body = if ok {
                        DONE_PAGE
                    } else {
                        "sign-in could not be verified"
                    };
                    let _ = write!(
                        stream,
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.flush();
                    let _ = stream.shutdown(Shutdown::Both);

                    if !ok {
                        continue;
                    }
                    let token = get("token").ok_or("the server sent no session token")?;
                    let session = Session {
                        token,
                        name: get("name").unwrap_or_else(|| "Player".into()),
                        allowed: get("allowed").as_deref() == Some("1"),
                        server: None,
                    };
                    let _ = save(data_dir, &session);
                    return Ok(session);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        return Err("sign-in timed out; try again".into());
                    }
                    std::thread::sleep(Duration::from_millis(120));
                }
                Err(e) => return Err(format!("sign-in failed: {e}")),
            }
        }
    }
}

/// Re-check the current session against the server (user still valid, still
/// cleared for multiplayer). Returns the refreshed session, or `None` once the
/// server says the session is gone (401), in which case it's forgotten. When
/// the server can't be reached, answers with another error, or can't tell
/// whether the user has the role, the session stays as it was.
pub fn refresh(data_dir: &Path, session: &Session) -> Option<Session> {
    refresh_at(&base_url(), data_dir, session)
}

/// `GET /v1/auth/launcher/me` as `(status, body)`, or `None` when the server can't be reached.
fn me(base: &str, session: &Session) -> Option<(u16, String)> {
    if let Some(endpoint) = session.endpoint() {
        let path = "/v1/auth/launcher/me";
        return crate::multiplayer::pinned::request(
            &endpoint,
            "GET",
            path,
            Some(&session.token),
            None,
            Duration::from_secs(20),
        )
        .ok();
    }
    let url = format!("{base}/v1/auth/launcher/me");
    let mut response = ureq::get(&url)
        .header("Authorization", &format!("Bearer {}", session.token))
        .header("User-Agent", "f1-multiplayer")
        .config()
        .timeout_global(Some(Duration::from_secs(20)))
        .http_status_as_error(false)
        .build()
        .call()
        .ok()?;
    let status = response.status().as_u16();
    Some((status, response.body_mut().read_to_string().unwrap_or_default()))
}

fn refresh_at(base: &str, data_dir: &Path, session: &Session) -> Option<Session> {
    let Some((status, text)) = me(base, session) else {
        return Some(session.clone());
    };
    match status {
        200 => {}
        401 => {
            forget(data_dir);
            return None;
        }
        _ => return Some(session.clone()),
    }
    let Ok(body) = serde_json::from_str::<MeResponse>(&text) else {
        return Some(session.clone());
    };
    let fresh = Session {
        token: session.token.clone(),
        name: body.user.name,
        allowed: match body.multiplayer_known {
            Some(false) => session.allowed,
            _ => body.multiplayer_allowed,
        },
        server: session.server.clone(),
    };
    let _ = save(data_dir, &fresh);
    // A host server sign-in from before secrets claims its name now, before anyone else can.
    if fresh.server.is_some()
        && let Err(e) = claim_host_name(data_dir, &fresh)
    {
        crate::multiplayer::diag::line(format!("couldn't claim the name on the host server: {e}"));
    }
    Some(fresh)
}

/// End the session on the server and locally.
pub fn sign_out(data_dir: &Path, session: &Session) {
    if let Some(endpoint) = session.endpoint() {
        let _ = crate::multiplayer::pinned::request(
            &endpoint,
            "POST",
            "/v1/auth/launcher/logout",
            Some(&session.token),
            Some(""),
            Duration::from_secs(5),
        );
        forget(data_dir);
        return;
    }
    let url = format!("{}/v1/auth/launcher/logout", base_url());
    let _ = ureq::post(&url)
        .header("Authorization", &format!("Bearer {}", session.token))
        .config()
        .timeout_global(Some(Duration::from_secs(15)))
        .http_status_as_error(false)
        .build()
        .send_empty();
    forget(data_dir);
}

#[derive(Deserialize)]
struct MeResponse {
    user: MeUser,
    multiplayer_allowed: bool,
    /// Whether the server could look up the role; servers before this field
    /// always could.
    #[serde(default)]
    multiplayer_known: Option<bool>,
}

#[derive(Deserialize)]
struct MeUser {
    name: String,
}

pub(crate) fn base64url(bytes: &[u8]) -> String {
    const SET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..chunk.len() + 1 {
            out.push(SET[((n >> (18 - 6 * i)) & 0x3f) as usize] as char);
        }
    }
    out
}

pub(crate) fn getrandom(buf: &mut [u8]) {
    // Enough entropy for a one-shot loopback login nonce without adding a dependency:
    // RandomState is randomly seeded, mixed with the clock and a counter.
    use std::hash::{BuildHasher, Hasher, RandomState};
    let state = RandomState::new();
    let mut i = 0usize;
    let mut counter = 0u64;
    while i < buf.len() {
        let mut h = state.build_hasher();
        h.write_u64(counter);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        h.write_u128(now);
        for byte in h.finish().to_le_bytes() {
            if i < buf.len() {
                buf[i] = byte;
                i += 1;
            }
        }
        counter += 1;
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    use super::*;

    fn serve(status: &str, body: &str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let reply = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut request = [0u8; 4096];
                let _ = stream.read(&mut request);
                let _ = stream.write_all(reply.as_bytes());
            }
        });
        format!("http://{addr}")
    }

    fn me(allowed: bool, known: Option<bool>) -> String {
        let known = known.map_or(String::new(), |k| format!(r#","multiplayer_known":{k}"#));
        format!(
            r#"{{"user":{{"id":"1","name":"xsoul","avatar":null}},"multiplayer_allowed":{allowed}{known}}}"#
        )
    }

    #[test]
    fn only_a_401_ends_the_session_and_an_unknown_role_keeps_the_last_answer() {
        let dir = std::env::temp_dir().join(format!("swofty-mpauth-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        let session = Session {
            token: "t".into(),
            name: "xsoul".into(),
            allowed: true,
            server: None,
        };
        save(&dir, &session).unwrap();
        let closed = {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            format!("http://{}", listener.local_addr().unwrap())
        };

        assert!(refresh_at(&closed, &dir, &session).unwrap().allowed);
        assert!(
            refresh_at(&serve("500 Internal Server Error", ""), &dir, &session)
                .unwrap()
                .allowed
        );
        assert!(
            refresh_at(&serve("429 Too Many Requests", ""), &dir, &session)
                .unwrap()
                .allowed
        );
        assert!(
            refresh_at(&serve("200 OK", "not json"), &dir, &session)
                .unwrap()
                .allowed
        );
        assert!(
            refresh_at(&serve("200 OK", &me(false, Some(false))), &dir, &session)
                .unwrap()
                .allowed
        );
        assert!(load(&dir).is_some());

        assert!(
            refresh_at(&serve("200 OK", &me(true, None)), &dir, &session)
                .unwrap()
                .allowed
        );
        let denied = refresh_at(&serve("200 OK", &me(false, Some(true))), &dir, &session).unwrap();
        assert!(!denied.allowed);
        assert!(!load(&dir).unwrap().allowed);

        assert!(refresh_at(&serve("401 Unauthorized", "not signed in"), &dir, &session).is_none());
        assert!(load(&dir).is_none());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn host_keys_survive_sign_out_and_differ_per_server() {
        let dir = std::env::temp_dir().join(format!("f1mp-host-keys-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        let a = host_secret(&dir, &[1; 32]);
        assert!(a.len() >= 43, "32 bytes of base64url: {a}");
        assert_eq!(host_secret(&dir, &[1; 32]), a);
        assert_ne!(host_secret(&dir, &[2; 32]), a);
        let session = Session {
            token: "t".into(),
            name: "Max".into(),
            allowed: true,
            server: None,
        };
        save(&dir, &session).unwrap();
        forget(&dir);
        assert!(load(&dir).is_none());
        assert_eq!(host_secret(&dir, &[1; 32]), a, "signing out keeps the key");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// A launcher that signed in before host servers knew secrets claims its name on the next
    /// refresh; then only it signs in under that name.
    #[test]
    fn a_launcher_claims_its_name_on_refresh_and_keeps_it() {
        let dir = std::env::temp_dir().join(format!("f1mp-host-claim-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        let server = crate::multiplayer::host::HostServer::start(crate::multiplayer::host::Config {
            dir: dir.join("server"),
            port: 0,
            name: "Test host".into(),
            password: None,
            public: false,
            discovery: None,
            fresh_code: false,
        })
        .unwrap();
        let code = server.code_for("127.0.0.1");
        let endpoint = crate::multiplayer::pinned::Endpoint::parse(&code).unwrap();

        let fresh = sign_in_host(&dir.join("a"), &code, "Anna", None).unwrap();
        assert!(load_keys(&dir.join("a")).values().all(|k| k.claimed));
        let taken = sign_in_host(&dir.join("b"), &code, "anna", None).unwrap_err();
        assert!(taken.contains("taken"), "{taken}");
        assert!(sign_in_host(&dir.join("a"), &code, "Anna", None).is_ok());
        assert!(claim_host_name(&dir.join("a"), &fresh).unwrap());

        // An old launcher's sign-in: no secret.
        let (status, text) = crate::multiplayer::pinned::request(
            &endpoint,
            "POST",
            "/v1/auth/launcher/direct",
            None,
            Some(r#"{"name": "Ben"}"#),
            Duration::from_secs(15),
        )
        .unwrap();
        assert_eq!(status, 200);
        let doc: serde_json::Value = serde_json::from_str(&text).unwrap();
        let old = Session {
            token: doc["token"].as_str().unwrap().to_owned(),
            name: "Ben".into(),
            allowed: true,
            server: Some(code.clone()),
        };
        save(&dir.join("c"), &old).unwrap();
        assert!(refresh(&dir.join("c"), &old).is_some());
        assert!(load_keys(&dir.join("c")).values().all(|k| k.claimed));
        assert!(sign_in_host(&dir.join("d"), &code, "Ben", None).is_err());
        assert!(sign_in_host(&dir.join("c"), &code, "Ben", None).is_ok());
        drop(server);
        std::fs::remove_dir_all(&dir).ok();
    }
}
