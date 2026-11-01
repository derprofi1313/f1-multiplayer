//! A lobby server the host runs on their own machine, so friends play together without
//! SwoftyLauncherServer: HTTPS (lobby, sign-in, TURN credentials, shared careers) on one TCP port
//! and a TURN relay on the same UDP port number.
//!
//! The host shares a host code (`<public address>:<port>#<pin>`, see
//! [`crate::multiplayer::pinned`]) or its six-digit server code ([`servercode`]); their own launcher
//! talks to `127.0.0.1`. Only that one port has
//! to be reachable from the internet, TCP and UDP; [`upnp`] asks the router to forward it.

pub mod careers;
mod lobby;
mod tls;
pub mod turn;
pub mod upnp;

use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

use super::pinned::Endpoint;
use super::servercode;

pub use super::pinned::DEFAULT_PORT;

/// Where a host server keeps its certificate, sessions and careers, under the launcher's data
/// folder.
pub const DATA_DIR: &str = "multiplayer/host";

#[derive(Clone, Debug)]
pub struct Config {
    /// The data folder (see [`DATA_DIR`]).
    pub dir: PathBuf,
    /// TCP for HTTPS and UDP for TURN; 0 picks a free one (tests).
    pub port: u16,
    /// What joiners see as the server's name.
    pub name: String,
    /// Asked of everyone who signs in, when set.
    pub password: Option<String>,
    /// Listen on every interface (`true`) or only on loopback (tests).
    pub public: bool,
    /// The UDP port to answer six-digit code lookups on ([`servercode::DISCOVERY_PORT`]; 0 picks a
    /// free one, for tests), or `None` for no lookups.
    pub discovery: Option<u16>,
    /// Replace the saved six-digit code with a new one.
    pub fresh_code: bool,
}

pub struct HostServer {
    lobby: Arc<lobby::Lobby>,
    turn: turn::TurnServer,
    pin: [u8; 32],
    port: u16,
    code: servercode::Saved,
    responder: Option<servercode::Responder>,
    stop: Arc<AtomicBool>,
    accept: Option<JoinHandle<()>>,
}

impl HostServer {
    pub fn start(config: Config) -> io::Result<HostServer> {
        std::fs::create_dir_all(&config.dir)?;
        let identity = tls::Identity::load_or_create(&config.dir)?;
        let ip = if config.public {
            Ipv4Addr::UNSPECIFIED
        } else {
            Ipv4Addr::LOCALHOST
        };
        let listener = TcpListener::bind(SocketAddrV4::new(ip, config.port))?;
        let port = listener.local_addr()?.port();
        let mut secret = vec![0u8; 32];
        crate::mpauth::getrandom(&mut secret);
        let turn = turn::TurnServer::start(SocketAddr::from((ip, port)), secret.clone())?;
        let lobby = Arc::new(lobby::Lobby::new(
            config.name.clone(),
            config.password.clone(),
            secret,
            port,
            &config.dir,
        ));
        let code = servercode::Saved::load_or_create(&config.dir, config.fresh_code)?;
        // Another host server on this PC may hold the lookup port; the long host code still works.
        let responder = config.discovery.and_then(|d| {
            servercode::Responder::start(&code.code, d, config.public, port, identity.pin()).ok()
        });
        let stop = Arc::new(AtomicBool::new(false));
        let accept = {
            let (lobby, stop, tls) = (lobby.clone(), stop.clone(), identity.server_config()?);
            std::thread::Builder::new()
                .name("host-accept".into())
                .spawn(move || lobby::accept_loop(lobby, listener, tls, stop))?
        };
        Ok(HostServer {
            lobby,
            turn,
            pin: identity.pin(),
            port,
            code,
            responder,
            stop,
            accept: Some(accept),
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn pin(&self) -> [u8; 32] {
        self.pin
    }

    pub fn name(&self) -> &str {
        &self.lobby.name
    }

    /// The six-digit server code friends can type instead of the host code.
    pub fn server_code(&self) -> &str {
        &self.code.code
    }

    /// Whether this PC and the local network can look the server code up (another host server on
    /// this PC may hold the lookup port).
    pub fn answers_lookups(&self) -> bool {
        self.responder.is_some()
    }

    /// The UDP port the server answers code lookups on.
    pub fn discovery_port(&self) -> Option<u16> {
        self.responder.as_ref().map(servercode::Responder::port)
    }

    /// Publish `host_code` under the server code at the code directory, when one is set (see
    /// [`servercode::publish`]).
    pub fn publish_code(&self, host_code: &str) -> Result<bool, String> {
        servercode::publish(&self.code, host_code)
    }

    /// How the host's own launcher reaches the server.
    pub fn local_endpoint(&self) -> Endpoint {
        Endpoint {
            base: format!("https://127.0.0.1:{}", self.port),
            pin: self.pin,
        }
    }

    /// The host code friends use, for the host's public address `host` (an IP or a name).
    pub fn code_for(&self, host: &str) -> String {
        let host = if host.contains(':') && !host.starts_with('[') {
            format!("[{host}]")
        } else {
            host.to_owned()
        };
        Endpoint {
            base: format!("https://{host}:{}", self.port),
            pin: self.pin,
        }
        .code()
    }

    /// Open rooms and live relays, for the status line.
    pub fn load(&self) -> (usize, usize) {
        (self.lobby.room_count(), self.turn.allocations())
    }

    /// Close every room (members get `closed`) and stop serving.
    pub fn stop(&mut self) {
        self.lobby.close_all();
        self.responder = None;
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.accept.take() {
            let _ = t.join();
        }
    }
}

impl Drop for HostServer {
    fn drop(&mut self) {
        self.stop();
    }
}
