//! A stand-in game for a simulated second player (`f1mp mp test-friend`), so two-player sessions
//! can be tested on one PC with one copy of F1 25: the real game hosts, and the test friend's
//! session joins the room through the host server, its relay and a bridge like any other member's.
//!
//! F1 25 refuses a second copy on the same licence, so the stand-in doesn't play: it answers the
//! session's game commands the way a game on its main menu would and counts the game datagrams
//! that reach it from the host's game (proof that the host game's traffic crosses the server, the
//! relay and the bridge). It sends the host's game nothing but the optional [`SimGame::probe`].

use std::net::{Ipv4Addr, UdpSocket};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use super::bridge;
use super::game::{self, Category, GameStatus, Role, Screen};
use super::session::GameControl;

/// What reached the stand-in from the other members' games.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Traffic {
    pub datagrams: u64,
    pub bytes: u64,
}

/// What [`SimGame::probe`] sends: not a game packet, so the game drops it; the host's socket trace
/// (`SWOFTY_NET_TRACE`) logs it as received from the test friend's lobby address.
pub const PROBE: &[u8] = b"SWOFTY-TEST-FRIEND-PROBE";

pub struct SimGame {
    dir: PathBuf,
    port: u16,
    socket: Arc<UdpSocket>,
    status: Mutex<GameStatus>,
    datagrams: Arc<AtomicU64>,
    bytes: Arc<AtomicU64>,
    stop: Arc<AtomicBool>,
    receiver: Mutex<Option<JoinHandle<()>>>,
    log: Mutex<Vec<String>>,
}

impl SimGame {
    /// A stand-in whose "game folder" is `dir` (the bridge reads `swofty/netplay.json` there).
    pub fn new(dir: &Path) -> std::io::Result<Arc<SimGame>> {
        std::fs::create_dir_all(dir.join("swofty"))?;
        let socket = Arc::new(UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))?);
        socket.set_read_timeout(Some(Duration::from_millis(200)))?;
        let port = socket.local_addr()?.port();
        let (datagrams, bytes, stop) = (
            Arc::new(AtomicU64::new(0)),
            Arc::new(AtomicU64::new(0)),
            Arc::new(AtomicBool::new(false)),
        );
        let receiver = {
            let (socket, datagrams, bytes, stop) =
                (socket.clone(), datagrams.clone(), bytes.clone(), stop.clone());
            std::thread::Builder::new()
                .name("test-friend-game".into())
                .spawn(move || {
                    let mut buf = [0u8; 2048];
                    while !stop.load(Ordering::Relaxed) {
                        if let Ok((n, _)) = socket.recv_from(&mut buf) {
                            datagrams.fetch_add(1, Ordering::Relaxed);
                            bytes.fetch_add(n as u64, Ordering::Relaxed);
                        }
                    }
                })?
        };
        Ok(Arc::new(SimGame {
            dir: dir.to_path_buf(),
            port,
            socket,
            status: Mutex::new(GameStatus {
                screen: Screen::MainMenu,
                in_session: false,
                role: Role::None,
                configured: false,
                self_member: 0,
                base: 0,
                cover: None,
                room_failed: None,
            }),
            datagrams,
            bytes,
            stop,
            receiver: Mutex::new(Some(receiver)),
            log: Mutex::new(Vec::new()),
        }))
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn traffic(&self) -> Traffic {
        Traffic {
            datagrams: self.datagrams.load(Ordering::Relaxed),
            bytes: self.bytes.load(Ordering::Relaxed),
        }
    }

    /// Send [`PROBE`] `count` times to lobby member `member`'s game, the way the game would send to
    /// 10.77.0.<member>: to this member's bridge port for it. Needs the room configured.
    pub fn probe(&self, member: u8, count: u32) -> std::io::Result<()> {
        let base = lock(&self.status).base;
        if base == 0 {
            return Err(std::io::Error::other("the stand-in isn't in a room yet"));
        }
        for _ in 0..count {
            self.socket
                .send_to(PROBE, (Ipv4Addr::LOCALHOST, base + u16::from(member)))?;
            std::thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    }

    /// The game commands the session sent, oldest first, once each; clears them.
    pub fn take_log(&self) -> Vec<String> {
        std::mem::take(&mut *lock(&self.log))
    }

    fn note(&self, line: String) {
        lock(&self.log).push(line);
    }

    fn enter(&self, op: String, role: Role) -> Result<GameStatus, game::Error> {
        let mut s = lock(&self.status);
        if !s.configured || s.in_session {
            return Err(game::Error::Refused {
                op: op.clone(),
                why: "the test friend is not on its main menu".into(),
                status: s.clone(),
            });
        }
        s.screen = Screen::LanLobby;
        s.in_session = true;
        s.role = role;
        let status = s.clone();
        drop(s);
        self.note(op);
        Ok(status)
    }
}

impl Drop for SimGame {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = lock(&self.receiver).take() {
            let _ = t.join();
        }
    }
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

impl GameControl for SimGame {
    fn status(&self) -> Result<GameStatus, game::Error> {
        Ok(lock(&self.status).clone())
    }

    fn configure(&self, self_member: u8, base: u16, name: Option<&str>) -> Result<GameStatus, game::Error> {
        let mut s = lock(&self.status);
        s.configured = true;
        s.self_member = self_member;
        s.base = base;
        let netplay = serde_json::json!({
            "pid": std::process::id(), "self": self_member, "base": base, "game_port": self.port,
        });
        let path = self.dir.join(bridge::STATUS_FILE);
        std::fs::write(&path, netplay.to_string()).map_err(|source| game::Error::Write { path, source })?;
        let status = s.clone();
        drop(s);
        self.note(format!(
            "configure: member {self_member}, bridge base port {base}, driver {}",
            name.unwrap_or("-")
        ));
        Ok(status)
    }

    fn host(&self, category: Category) -> Result<GameStatus, game::Error> {
        self.enter(format!("host {category}"), Role::Host)
    }

    fn join(&self, ip: Ipv4Addr) -> Result<GameStatus, game::Error> {
        self.enter(format!("join the host's game at {ip}"), Role::Client)
    }

    fn leave(&self) -> Result<GameStatus, game::Error> {
        let mut s = lock(&self.status);
        s.screen = Screen::MainMenu;
        s.in_session = false;
        s.role = Role::None;
        let status = s.clone();
        drop(s);
        self.note("leave".into());
        Ok(status)
    }

    fn clear(&self) -> Result<GameStatus, game::Error> {
        let mut s = lock(&self.status);
        s.configured = false;
        s.self_member = 0;
        s.base = 0;
        let _ = std::fs::remove_file(self.dir.join(bridge::STATUS_FILE));
        Ok(s.clone())
    }

    fn cancel(&self) -> Result<GameStatus, game::Error> {
        Ok(lock(&self.status).clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_what_reaches_its_game_port() {
        let dir = std::env::temp_dir().join(format!("swofty-test-friend-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let sim = SimGame::new(&dir).unwrap();
        assert!(
            sim.join(Ipv4Addr::new(10, 77, 0, 1)).is_err(),
            "not configured yet"
        );
        sim.configure(2, 40000, Some("Testfreund")).unwrap();
        let written: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join(bridge::STATUS_FILE)).unwrap()).unwrap();
        let port = written["game_port"].as_u64().unwrap() as u16;
        assert_eq!(
            sim.join(Ipv4Addr::new(10, 77, 0, 1)).unwrap().screen,
            Screen::LanLobby
        );
        let out = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        out.send_to(b"hello", (Ipv4Addr::LOCALHOST, port)).unwrap();
        let until = std::time::Instant::now() + Duration::from_secs(3);
        while sim.traffic().datagrams == 0 && std::time::Instant::now() < until {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert_eq!(
            sim.traffic(),
            Traffic {
                datagrams: 1,
                bytes: 5
            }
        );
        assert_eq!(sim.take_log().len(), 2);
        drop(sim);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
