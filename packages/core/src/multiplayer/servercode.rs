//! Six-digit server codes: the short code a host server gets, which friends type instead of the
//! long host code (`<address>:<port>#<pin>`, see [`super::pinned`]).
//!
//! Six digits can't carry an address and a certificate pin, so a code is looked up:
//! - **On this PC and the local network**: the joiner broadcasts `SWOFTY-FIND 1 <code>` to UDP
//!   [`DISCOVERY_PORT`] (and sends it to loopback); the host server with that code answers
//!   `SWOFTY-HERE 1 <code> <port> <pin>` from its address. The answer's source address and port
//!   make the host code.
//! - **Over the internet**, only when a directory is set ([`DIRECTORY_ENV`]): the host publishes its
//!   host code under the code (`PUT <directory>/v1/codes/<code>`, with the server's own key so no one
//!   else can take the code over) and joiners read it back (`GET`). The directory is a small service
//!   of the players' own (`apps/mp-directory`); nothing is published without one.
//!
//! Whoever answers a lookup chooses the pin the joiner then trusts, so a code is only as good as the
//! network or directory that answered it; the host server's password still guards the sign-in.

use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use super::pinned::Endpoint;

/// The UDP port host servers answer code lookups on.
pub const DISCOVERY_PORT: u16 = 47799;
/// The base URL of a code directory for lookups over the internet, like `https://codes.example.org`.
pub const DIRECTORY_ENV: &str = "SWOFTY_MP_DIRECTORY";
/// Where a host server keeps its code and directory key, in its data folder.
pub const FILE: &str = "server_code.json";

const FIND: &str = "SWOFTY-FIND 1 ";
const HERE: &str = "SWOFTY-HERE 1 ";

/// The six digits of `text` when it is a server code: `123456`, `123 456` or `123-456`.
pub fn parse(text: &str) -> Option<String> {
    let text = text.trim();
    if !text.chars().all(|c| c.is_ascii_digit() || c == ' ' || c == '-') {
        return None;
    }
    let digits: String = text.chars().filter(char::is_ascii_digit).collect();
    (digits.len() == 6).then_some(digits)
}

/// A code as people read it: `123 456`.
pub fn display(code: &str) -> String {
    match code.len() {
        6 => format!("{} {}", &code[..3], &code[3..]),
        _ => code.to_owned(),
    }
}

/// A host server's code and the key that owns it at the directory.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Saved {
    pub code: String,
    pub key: String,
}

impl Saved {
    /// The server's code from its data folder, made on first use; `fresh` replaces it.
    pub fn load_or_create(dir: &Path, fresh: bool) -> io::Result<Saved> {
        let path = dir.join(FILE);
        if !fresh
            && let Ok(text) = std::fs::read_to_string(&path)
            && let Ok(saved) = serde_json::from_str::<Saved>(&text)
            && parse(&saved.code).as_deref() == Some(saved.code.as_str())
        {
            return Ok(saved);
        }
        let mut raw = [0u8; 4];
        crate::mpauth::getrandom(&mut raw);
        let mut key = [0u8; 24];
        crate::mpauth::getrandom(&mut key);
        let saved = Saved {
            code: format!("{:06}", u32::from_le_bytes(raw) % 1_000_000),
            key: crate::mpauth::base64url(&key),
        };
        std::fs::write(&path, serde_json::to_string_pretty(&saved)?)?;
        Ok(saved)
    }
}

/// Answers code lookups for one host server until dropped.
pub struct Responder {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    port: u16,
}

impl Responder {
    /// Answer lookups of `code` on UDP `port` (0 picks a free one, for tests), every interface when
    /// `public`, else loopback only. Answers carry the host server's `server_port` and `pin`.
    pub fn start(
        code: &str,
        port: u16,
        public: bool,
        server_port: u16,
        pin: [u8; 32],
    ) -> io::Result<Responder> {
        let ip = if public {
            Ipv4Addr::UNSPECIFIED
        } else {
            Ipv4Addr::LOCALHOST
        };
        let socket = UdpSocket::bind(SocketAddrV4::new(ip, port))?;
        socket.set_read_timeout(Some(Duration::from_millis(250)))?;
        let port = socket.local_addr()?.port();
        let stop = Arc::new(AtomicBool::new(false));
        let ask = format!("{FIND}{code}");
        let answer = format!("{HERE}{code} {server_port} {}", crate::mpauth::base64url(&pin));
        let thread = {
            let stop = stop.clone();
            std::thread::Builder::new()
                .name("host-code".into())
                .spawn(move || {
                    let mut buf = [0u8; 64];
                    while !stop.load(Ordering::Relaxed) {
                        let Ok((n, from)) = socket.recv_from(&mut buf) else {
                            continue;
                        };
                        if buf[..n].trim_ascii() == ask.as_bytes() {
                            let _ = socket.send_to(answer.as_bytes(), from);
                        }
                    }
                })?
        };
        Ok(Responder {
            stop,
            thread: Some(thread),
            port,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Drop for Responder {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// The host code behind `code`: on this PC or the local network first, then at the directory.
pub fn resolve(code: &str) -> Result<Endpoint, String> {
    if let Some(found) = find_nearby(code, DISCOVERY_PORT, Duration::from_millis(1500)) {
        return Ok(found);
    }
    match directory() {
        Some(base) => lookup(&base, code),
        None => Err(format!(
            "no host server with the code {} answered on this PC or your local network. Over the \
             internet, use the long host code your friend's launcher shows (or set up a code directory, \
             {DIRECTORY_ENV})",
            display(code)
        )),
    }
}

/// Ask this PC and the local network for `code` on UDP `port`, for up to `wait`.
pub fn find_nearby(code: &str, port: u16, wait: Duration) -> Option<Endpoint> {
    let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).ok()?;
    let _ = socket.set_broadcast(true);
    socket.set_read_timeout(Some(Duration::from_millis(200))).ok()?;
    let ask = format!("{FIND}{code}");
    let targets = [Ipv4Addr::LOCALHOST, Ipv4Addr::BROADCAST];
    let until = Instant::now() + wait;
    let mut next_send = Instant::now();
    let mut buf = [0u8; 128];
    while Instant::now() < until {
        if Instant::now() >= next_send {
            for ip in targets {
                let _ = socket.send_to(ask.as_bytes(), SocketAddrV4::new(ip, port));
            }
            next_send = Instant::now() + Duration::from_millis(400);
        }
        if let Ok((n, from)) = socket.recv_from(&mut buf)
            && let Some(found) = answer(&buf[..n], code, from)
        {
            return Some(found);
        }
    }
    None
}

/// The host code in a `SWOFTY-HERE` answer to a lookup of `code` that came from `from`.
fn answer(data: &[u8], code: &str, from: SocketAddr) -> Option<Endpoint> {
    let text = std::str::from_utf8(data).ok()?.trim();
    let rest = text.strip_prefix(HERE)?;
    let mut words = rest.split(' ');
    if words.next()? != code {
        return None;
    }
    let port: u16 = words.next()?.parse().ok()?;
    let pin = super::rooms::decode_key(words.next()?)?;
    Some(Endpoint {
        base: format!("https://{}:{port}", from.ip()),
        pin,
    })
}

fn directory() -> Option<String> {
    std::env::var(DIRECTORY_ENV)
        .ok()
        .map(|s| s.trim().trim_end_matches('/').to_owned())
        .filter(|s| !s.is_empty())
}

fn lookup(base: &str, code: &str) -> Result<Endpoint, String> {
    let url = format!("{base}/v1/codes/{code}");
    let response = ureq::get(&url)
        .header("User-Agent", crate::catalog::agent_name())
        .config()
        .timeout_global(Some(Duration::from_secs(15)))
        .http_status_as_error(false)
        .build()
        .call()
        .map_err(|e| format!("couldn't reach the code directory: {e}"))?;
    let status = response.status().as_u16();
    let text = response
        .into_body()
        .with_config()
        .limit(16 * 1024)
        .read_to_string()
        .map_err(|e| format!("the code directory's answer broke off: {e}"))?;
    match status {
        200 => {}
        404 => return Err(format!("no host server has the code {}", display(code))),
        _ => return Err(format!("the code directory refused the lookup (HTTP {status})")),
    }
    let doc: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
    Endpoint::parse(doc["host"].as_str().unwrap_or(""))
}

/// Publish `host_code` under the server's code at the directory, when one is set. `Ok(false)` when
/// there is no directory.
pub fn publish(saved: &Saved, host_code: &str) -> Result<bool, String> {
    let Some(base) = directory() else {
        return Ok(false);
    };
    let url = format!("{base}/v1/codes/{}", saved.code);
    let body = serde_json::json!({ "host": host_code, "key": saved.key }).to_string();
    let response = ureq::put(&url)
        .header("User-Agent", crate::catalog::agent_name())
        .header("Content-Type", "application/json")
        .config()
        .timeout_global(Some(Duration::from_secs(15)))
        .http_status_as_error(false)
        .build()
        .send(body)
        .map_err(|e| format!("couldn't reach the code directory: {e}"))?;
    match response.status().as_u16() {
        200 | 201 | 204 => Ok(true),
        409 => Err(format!(
            "another server already has the code {} at the directory; make a new code",
            display(&saved.code)
        )),
        status => Err(format!("the code directory refused the code (HTTP {status})")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_codes_people_type() {
        assert_eq!(parse("123456").as_deref(), Some("123456"));
        assert_eq!(parse(" 123 456 ").as_deref(), Some("123456"));
        assert_eq!(parse("123-456").as_deref(), Some("123456"));
        assert_eq!(parse("12345"), None);
        assert_eq!(parse("1234567"), None);
        assert_eq!(parse("203.0.113.7:47800#abc"), None);
        assert_eq!(display("012345"), "012 345");
    }

    #[test]
    fn keeps_the_code_until_a_fresh_one_is_asked_for() {
        let dir = std::env::temp_dir().join(format!("swofty-code-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let first = Saved::load_or_create(&dir, false).unwrap();
        assert_eq!(parse(&first.code).as_deref(), Some(first.code.as_str()));
        assert_eq!(Saved::load_or_create(&dir, false).unwrap(), first);
        assert_ne!(Saved::load_or_create(&dir, true).unwrap().key, first.key);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn finds_a_server_on_this_pc_by_its_code() {
        let pin = [7u8; 32];
        let responder = Responder::start("042424", 0, false, 47800, pin).unwrap();
        let found = find_nearby("042424", responder.port(), Duration::from_secs(3)).unwrap();
        assert_eq!(
            found,
            Endpoint {
                base: "https://127.0.0.1:47800".into(),
                pin
            }
        );
        assert!(find_nearby("999999", responder.port(), Duration::from_millis(600)).is_none());
    }

    #[test]
    fn ignores_answers_for_other_codes_and_garbage() {
        let from: SocketAddr = "192.168.1.20:47799".parse().unwrap();
        let pin = crate::mpauth::base64url(&[1u8; 32]);
        assert!(answer(format!("{HERE}111111 47800 {pin}").as_bytes(), "222222", from).is_none());
        assert!(answer(b"SWOFTY-HERE 1 111111 x y", "111111", from).is_none());
        let ok = answer(format!("{HERE}111111 47800 {pin}").as_bytes(), "111111", from).unwrap();
        assert_eq!(ok.base, "https://192.168.1.20:47800");
    }
}
