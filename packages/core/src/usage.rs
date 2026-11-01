//! Anonymised usage reports.
//!
//! Release builds report, every five minutes while the launcher is open and
//! during each modded session, which launcher version is running on which
//! kind of system, which store the game copy is from (Steam, the EA app or
//! unknown), and how each session ended: quit, crash, stopped by the player,
//! or unknown, with the exit code when the session could read it.
//! A report carries a random id
//! made for this install and nothing that identifies the player. Nothing is
//! sent until the offline play terms, which say so, are accepted; builds from
//! source have no address to send to.

use std::hash::{BuildHasher, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::install::Source;
use crate::session::SessionEnd;

pub const INTERVAL: Duration = Duration::from_secs(300);
const TIMEOUT: Duration = Duration::from_secs(10);
const END_TIMEOUT: Duration = Duration::from_secs(5);

/// Where reports go: `SWOFTY_USAGE_URL` at run time, else the address baked in
/// at build time. `SWOFTY_USAGE=0` turns reporting off.
pub fn endpoint() -> Option<String> {
    if std::env::var_os("SWOFTY_USAGE").is_some_and(|v| v == "0") {
        return None;
    }
    std::env::var("SWOFTY_USAGE_URL")
        .ok()
        .or_else(|| option_env!("SWOFTY_USAGE_URL").map(str::to_owned))
        .map(|u| u.trim().trim_end_matches('/').to_owned())
        .filter(|u| u.starts_with("https://") || u.starts_with("http://"))
}

pub fn enabled(data: &Path) -> bool {
    endpoint().is_some() && crate::offline::is_accepted(data)
}

fn id_path(data: &Path) -> PathBuf {
    data.join("state/install-id")
}

/// This install's random id, made on first use.
pub fn install_id(data: &Path) -> Option<String> {
    let path = id_path(data);
    if let Ok(text) = std::fs::read_to_string(&path) {
        let id = text.trim();
        if id.len() == 32
            && id
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Some(id.to_owned());
        }
    }
    let id = random_hex(2);
    std::fs::create_dir_all(path.parent()?).ok()?;
    std::fs::write(&path, format!("{id}\n")).ok()?;
    Some(id)
}

pub(crate) fn random_hex(words: usize) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    (0..words)
        .map(|i| {
            let mut h = std::collections::hash_map::RandomState::new().build_hasher();
            h.write_u128(nanos);
            h.write_u32(std::process::id());
            h.write_usize(i);
            format!("{:016x}", h.finish())
        })
        .collect()
}

pub fn os() -> &'static str {
    match std::env::consts::OS {
        "windows" => "windows",
        "macos" => "macos",
        "linux" => {
            let release = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
            if release.lines().any(|l| l.trim() == "ID=steamos") {
                "steamos"
            } else {
                "linux"
            }
        }
        _ => "other",
    }
}

pub fn platform(source: Source) -> &'static str {
    match source {
        Source::Steam => "steam",
        Source::EaApp => "ea",
        Source::Unknown => "unknown",
    }
}

/// The store of the copy Play uses, or unknown when no copy is found.
pub fn active_platform(data: &Path) -> &'static str {
    crate::copies::find_active(&crate::store::Store::new(data).config())
        .map_or("unknown", |c| platform(c.source))
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Event {
    pub kind: &'static str,
    pub install: String,
    pub version: String,
    pub os: &'static str,
    pub platform: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crash_code: Option<u32>,
}

impl Event {
    pub fn new(kind: &'static str, install: &str, platform: &'static str) -> Self {
        Self {
            kind,
            install: install.to_owned(),
            version: crate::update::current_version().to_string(),
            os: os(),
            platform,
            session: None,
            exit_code: None,
            seconds: None,
            end: None,
            crash_code: None,
        }
    }

    /// A session's last report. Servers before `end` was added require an
    /// integer exit code, so a code the session could not read goes as -1,
    /// the value they always had for it.
    pub fn session_end(
        install: &str,
        platform: &'static str,
        session: &str,
        seconds: u64,
        end: &SessionEnd,
    ) -> Self {
        Self {
            session: Some(session.to_owned()),
            exit_code: Some(end.exit_code.unwrap_or(-1)),
            seconds: Some(seconds),
            end: Some(end.end.as_str()),
            crash_code: end.crash_code,
            ..Self::new("session_end", install, platform)
        }
    }
}

/// Post one report; failures are dropped.
pub fn send(url: &str, event: &Event, timeout: Duration) -> bool {
    let Ok(body) = serde_json::to_vec(event) else {
        return false;
    };
    ureq::post(&format!("{url}/v1/events"))
        .header(
            "User-Agent",
            concat!("swofty-launcher/", env!("CARGO_PKG_VERSION")),
        )
        .header("Content-Type", "application/json")
        .config()
        .timeout_global(Some(timeout))
        .http_status_as_error(false)
        .build()
        .send(&body[..])
        .is_ok_and(|r| r.status().is_success())
}

/// Report that the launcher is open, now and every [INTERVAL] while the
/// process runs. Each report checks the terms first, so accepting them later
/// starts reporting without a restart.
pub fn launcher_heartbeats(data: PathBuf) {
    if endpoint().is_none() {
        return;
    }
    std::thread::Builder::new()
        .name("usage".into())
        .spawn(move || {
            loop {
                if let (Some(url), true) = (endpoint(), crate::offline::is_accepted(&data))
                    && let Some(id) = install_id(&data)
                {
                    let event = Event::new("launcher", &id, active_platform(&data));
                    send(&url, &event, TIMEOUT);
                }
                std::thread::sleep(INTERVAL);
            }
        })
        .ok();
}

/// One modded session of the copy in `game`: reported when it starts, every
/// [INTERVAL] while it runs, and when the game exits.
pub struct Session {
    url: String,
    install: String,
    platform: &'static str,
    id: String,
    started: Instant,
    stop: Arc<AtomicBool>,
}

impl Session {
    pub fn start(data: &Path, game: &Path) -> Option<Self> {
        if !enabled(data) {
            return None;
        }
        let session = Self {
            url: endpoint()?,
            install: install_id(data)?,
            platform: platform(Source::detect(game)),
            id: random_hex(1),
            started: Instant::now(),
            stop: Arc::new(AtomicBool::new(false)),
        };
        let (url, install, platform, id, stop) = (
            session.url.clone(),
            session.install.clone(),
            session.platform,
            session.id.clone(),
            session.stop.clone(),
        );
        std::thread::Builder::new()
            .name("usage-session".into())
            .spawn(move || {
                let event = |kind| Event {
                    session: Some(id.clone()),
                    ..Event::new(kind, &install, platform)
                };
                send(&url, &event("session_start"), TIMEOUT);
                let mut last = Instant::now();
                while !stop.load(Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_secs(1));
                    if last.elapsed() >= INTERVAL && !stop.load(Ordering::Relaxed) {
                        send(&url, &event("session"), TIMEOUT);
                        last = Instant::now();
                    }
                }
            })
            .ok()?;
        Some(session)
    }

    pub fn end(self, end: &SessionEnd) {
        self.stop.store(true, Ordering::Relaxed);
        let event = Event::session_end(
            &self.install,
            self.platform,
            &self.id,
            self.started.elapsed().as_secs(),
            end,
        );
        send(&self.url, &event, END_TIMEOUT);
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    use super::*;
    use crate::session::Ending;

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("swofty-usage-{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&d).ok();
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn install_id_is_made_once() {
        let d = temp("id");
        let id = install_id(&d).unwrap();
        assert_eq!(id.len(), 32);
        assert!(id.bytes().all(|b| b.is_ascii_hexdigit()));
        assert_eq!(install_id(&d).unwrap(), id);
        std::fs::write(id_path(&d), "not an id\n").unwrap();
        assert_ne!(install_id(&d).unwrap(), id);
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn ids_differ() {
        assert_ne!(random_hex(2), random_hex(2));
        assert_eq!(random_hex(1).len(), 16);
    }

    #[test]
    fn events_carry_only_what_the_server_reads() {
        let crash = SessionEnd {
            end: Ending::Crash,
            exit_code: Some(-1),
            crash_code: Some(0xc000_0005),
            reason: "the mod loader saw an unhandled exception".into(),
        };
        let event = Event::session_end(&"a".repeat(32), "ea", "0123456789abcdef", 42, &crash);
        let json: serde_json::Value = serde_json::to_value(&event).unwrap();
        let mut keys: Vec<&str> = json.as_object().unwrap().keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "crash_code",
                "end",
                "exit_code",
                "install",
                "kind",
                "os",
                "platform",
                "seconds",
                "session",
                "version"
            ]
        );
        assert_eq!(
            (&json["end"], &json["exit_code"], &json["crash_code"]),
            (&"crash".into(), &(-1).into(), &0xc000_0005u32.into())
        );
        assert_eq!(json["platform"], "ea");
        let launcher = serde_json::to_value(Event::new("launcher", &"b".repeat(32), "steam")).unwrap();
        assert_eq!(launcher.as_object().unwrap().len(), 5);
        assert_eq!(launcher["os"], os());
        assert_eq!(launcher["platform"], "steam");
    }

    #[test]
    fn an_unread_exit_code_goes_as_minus_one_beside_its_end() {
        let unknown = SessionEnd {
            end: Ending::Unknown,
            exit_code: None,
            crash_code: None,
            reason: "the EA app started F1 25".into(),
        };
        let json = serde_json::to_value(Event::session_end(
            &"a".repeat(32),
            "unknown",
            "0123456789abcdef",
            7,
            &unknown,
        ))
        .unwrap();
        assert_eq!(
            (&json["exit_code"], &json["end"]),
            (&(-1).into(), &"unknown".into())
        );
        assert!(json.get("crash_code").is_none());
        assert!(json.get("reason").is_none());
        let stopped = SessionEnd {
            end: Ending::Stopped,
            exit_code: Some(1),
            ..unknown
        };
        let json = serde_json::to_value(Event::session_end(
            &"a".repeat(32),
            "unknown",
            "0123456789abcdef",
            7,
            &stopped,
        ))
        .unwrap();
        assert_eq!((&json["exit_code"], &json["end"]), (&1.into(), &"stopped".into()));
    }

    #[test]
    fn nothing_is_sent_before_the_terms_are_accepted() {
        let d = temp("terms");
        assert!(!enabled(&d));
        assert!(Session::start(&d, &d).is_none());
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn send_posts_json() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = vec![0u8; 8192];
            let mut got = Vec::new();
            loop {
                let n = stream.read(&mut buf).unwrap();
                got.extend_from_slice(&buf[..n]);
                let text = String::from_utf8_lossy(&got);
                if let Some(head) = text.find("\r\n\r\n") {
                    let len = text[..head]
                        .lines()
                        .find_map(|l| {
                            l.to_ascii_lowercase()
                                .strip_prefix("content-length:")
                                .map(|v| v.trim().parse::<usize>().unwrap())
                        })
                        .unwrap_or(0);
                    if got.len() >= head + 4 + len {
                        break;
                    }
                }
                if n == 0 {
                    break;
                }
            }
            stream
                .write_all(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n")
                .unwrap();
            String::from_utf8(got).unwrap()
        });
        assert!(send(
            &url,
            &Event::new("launcher", &"c".repeat(32), "ea"),
            TIMEOUT
        ));
        let request = server.join().unwrap();
        assert!(request.starts_with("POST /v1/events "));
        let body: serde_json::Value =
            serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(body["kind"], "launcher");
        assert_eq!(body["install"], "c".repeat(32));
        assert_eq!(body["platform"], "ea");
    }

    #[test]
    fn the_platform_is_the_store_of_the_copy_play_uses() {
        let d = temp("platform");
        let (ea, other) = (d.join("ea/F1 25"), d.join("other/F1 25"));
        for dir in [&ea, &other] {
            std::fs::create_dir_all(dir).unwrap();
            std::fs::write(dir.join(crate::GAME_EXE), b"MZ").unwrap();
        }
        std::fs::create_dir_all(ea.join("__Installer")).unwrap();
        std::fs::write(ea.join(crate::ea_app::MARKER), "<contentID>1</contentID>").unwrap();
        assert_eq!(platform(Source::detect(&ea)), "ea");
        assert_eq!(platform(Source::detect(&other)), "unknown");
        assert_eq!(platform(Source::Steam), "steam");
        let data = d.join("data");
        let store = crate::store::Store::new(&data);
        for (dir, want) in [(&ea, "ea"), (&other, "unknown")] {
            let mut config = store.config();
            crate::copies::add(&mut config, dir).unwrap();
            store.save_config(&config).unwrap();
            assert_eq!(active_platform(&data), want);
        }
        std::fs::remove_dir_all(&d).ok();
    }
}
