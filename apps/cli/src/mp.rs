//! The `f1mp mp` commands: run a host server on this machine, and sign in to one.
//!
//! See `swofty_core::multiplayer::host` for the server and `swofty_core::multiplayer::pinned` for
//! host codes.

use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use clap::Subcommand;
use swofty_core::install::Game;
use swofty_core::mpauth;
use swofty_core::multiplayer::host::{self, Config, HostServer, upnp};
use swofty_core::multiplayer::rooms::Api;
use swofty_core::multiplayer::servercode;
use swofty_core::multiplayer::session::{Local, Phase, Relay, RelayStatus, Session, Setup, stats_line};
use swofty_core::multiplayer::testfriend::{PROBE, SimGame};
use swofty_core::store::Store;

#[derive(Subcommand)]
pub enum MpCommand {
    #[command(
        about = "Run a host server on this PC so friends can join your rooms and shared careers",
        args_conflicts_with_subcommands = true
    )]
    Host {
        #[command(subcommand)]
        action: Option<HostAction>,
        #[arg(long, default_value_t = host::DEFAULT_PORT, help = "TCP and UDP port to listen on")]
        port: u16,
        #[arg(long, help = "Ask everyone who signs in for this password")]
        password: Option<String>,
        #[arg(
            long,
            default_value = "F1 Multiplayer host",
            help = "The server's name joiners see"
        )]
        name: String,
        #[arg(
            long,
            help = "Your public address (IP or DNS name) for the host code, if UPnP can't tell"
        )]
        address: Option<String>,
        #[arg(long, help = "Don't ask the router to forward the port")]
        no_upnp: bool,
        #[arg(long, help = "Give the server a new six-digit code")]
        new_code: bool,
    },
    #[command(about = "Sign this launcher in to a host server with its six-digit server code or host code")]
    Join {
        #[arg(help = "The six-digit server code (123 456) or the host code (address:port#pin)")]
        code: String,
        #[arg(long, help = "Your driver name")]
        name: String,
        #[arg(long, help = "The host server's password, if it has one")]
        password: Option<String>,
    },
    #[command(
        about = "Join a host server as a simulated second player, to test two-player sessions on one PC",
        long_about = "Join a host server as a simulated second player, to test two-player sessions on one PC \
                      with one copy of F1 25 (the game refuses a second copy on the same licence). The test \
                      friend signs in with the server code under its own name, keeps its own sign-in (your \
                      launcher's stays as it is), joins your room like a second launcher, with the server's \
                      relay and a bridge, and reports what your game sends it. It doesn't play: nothing goes \
                      back to your game."
    )]
    TestFriend {
        #[arg(help = "The six-digit server code (123 456) or the host code")]
        code: String,
        #[arg(long, default_value = "Testfreund", help = "The test friend's driver name")]
        name: String,
        #[arg(long, help = "The host server's password, if it has one")]
        password: Option<String>,
        #[arg(
            long,
            help = "The room to join (F1-...); default: the first open room, waiting for one"
        )]
        room: Option<String>,
        #[arg(long, help = "Also join this shared career by its code")]
        career: Option<String>,
        #[arg(long, default_value_t = 600, help = "Stay this many seconds, then leave")]
        seconds: u64,
        #[arg(
            long,
            help = "Once in the room, send the host's game a few probe datagrams (it drops them; its socket                     trace shows whether they arrived)"
        )]
        probe: bool,
        #[arg(
            long,
            help = "The F1 25 folder for the addon fingerprint (default: the active copy)"
        )]
        game: Option<std::path::PathBuf>,
    },
    #[command(
        about = "The league career 2016-2026 you drive with friends on the host server",
        subcommand
    )]
    Career(crate::mp_career::CareerCommand),
    #[command(about = "Show which server this launcher's multiplayer is signed in to")]
    Status,
    #[command(about = "Sign out of multiplayer")]
    Logout,
}

#[derive(Subcommand)]
pub enum HostAction {
    #[command(
        about = "Free a driver name claimed on your host server, so its driver can sign in again from another PC"
    )]
    Release {
        #[arg(help = "The driver name")]
        name: String,
        #[arg(long, help = "The host server's data folder (default: this launcher's)")]
        dir: Option<std::path::PathBuf>,
    },
}

pub fn command(store: &Store, profile: &str, cmd: MpCommand) -> Result<()> {
    match cmd {
        MpCommand::Host {
            action: Some(HostAction::Release { name, dir }),
            ..
        } => {
            let dir = dir.unwrap_or_else(|| store.root.join(host::DATA_DIR));
            let released = host::release_name(&dir, &name)
                .with_context(|| format!("reading the host server's sign-ins in {}", dir.display()))?;
            if released {
                println!("released \"{name}\": its driver can sign in again from any PC, which then claims it");
            } else {
                println!("nobody has claimed \"{name}\" on this host server");
            }
            Ok(())
        }
        MpCommand::Host {
            action: None,
            port,
            password,
            name,
            address,
            no_upnp,
            new_code,
        } => run_host(store, port, password, name, address, no_upnp, new_code),
        MpCommand::Join { code, name, password } => {
            let session = mpauth::sign_in_host(&store.root, &code, &name, password.as_deref())
                .map_err(|e| anyhow!(e))?;
            println!(
                "signed in to {} as {}",
                session.server.as_deref().unwrap_or("?"),
                session.name
            );
            println!("restart the launcher and open the Multiplayer page");
            Ok(())
        }
        MpCommand::TestFriend {
            code,
            name,
            password,
            room,
            career,
            seconds,
            probe,
            game,
        } => test_friend(
            store,
            &code,
            &name,
            password.as_deref(),
            room,
            career,
            Duration::from_secs(seconds),
            probe,
            game,
        ),
        MpCommand::Career(cmd) => crate::mp_career::command(store, profile, cmd),
        MpCommand::Status => {
            match mpauth::load(&store.root) {
                Some(s) => println!(
                    "signed in as {} to {}",
                    s.name,
                    s.server.as_deref().unwrap_or("the Swofty server")
                ),
                None => println!("not signed in"),
            }
            Ok(())
        }
        MpCommand::Logout => {
            if let Some(s) = mpauth::load(&store.root) {
                mpauth::sign_out(&store.root, &s);
            }
            println!("signed out");
            Ok(())
        }
    }
}

fn run_host(
    store: &Store,
    port: u16,
    password: Option<String>,
    name: String,
    address: Option<String>,
    no_upnp: bool,
    new_code: bool,
) -> Result<()> {
    let server = HostServer::start(Config {
        dir: store.root.join(host::DATA_DIR),
        port,
        name,
        password,
        public: true,
        discovery: Some(servercode::DISCOVERY_PORT),
        fresh_code: new_code,
    })
    .with_context(|| format!("starting the host server on port {port}"))?;
    println!(
        "host server \"{}\" listening on TCP and UDP port {}",
        server.name(),
        server.port()
    );
    let forwarded = if no_upnp {
        None
    } else {
        print!("asking the router to forward the port (UPnP)… ");
        match upnp::forward(server.port()) {
            Ok(f) => {
                println!(
                    "done{}",
                    f.external_ip
                        .map_or(String::new(), |ip| format!(", public address {ip}"))
                );
                if f.behind_another_nat() {
                    println!(
                        "warning: the router's own public address is private: your connection is behind a second \
                         router or carrier NAT, so friends can't reach this server from the internet"
                    );
                }
                Some(f)
            }
            Err(e) => {
                println!("not possible: {e}");
                println!(
                    "forward TCP and UDP port {} on your router to this PC by hand, and allow it in the firewall",
                    server.port()
                );
                None
            }
        }
    };
    let public = address.or_else(|| {
        forwarded
            .as_ref()
            .and_then(|f| f.external_ip)
            .map(|ip| ip.to_string())
    });
    println!(
        "\nserver code for your friends:  {}",
        servercode::display(server.server_code())
    );
    if !server.answers_lookups() {
        println!(
            "warning: UDP port {} is taken (another host server on this PC?), so the server code can't be \
             looked up here; use the host code",
            servercode::DISCOVERY_PORT
        );
    }
    match &public {
        Some(addr) => {
            let code = server.code_for(addr);
            match server.publish_code(&code) {
                Ok(true) => println!("published at the code directory, so it works over the internet too"),
                Ok(false) => println!(
                    "it works on this PC and your local network; over the internet friends need the host code \
                     (or a code directory, {})",
                    servercode::DIRECTORY_ENV
                ),
                Err(e) => println!("the code directory: {e}"),
            }
            println!("\nhost code for your friends:\n\n  {code}\n");
        }
        None => println!(
            "\nthe public address is unknown; give friends this code with your public IP in place of PUBLIC-IP:\n\n  {}\n",
            server.code_for("PUBLIC-IP")
        ),
    }
    println!(
        "on this PC, sign in with:  f1mp mp join {} --name <your name>",
        server.server_code()
    );
    println!("press Ctrl+C to stop the server (the router's port forward runs out within a day)");
    let mut last = (usize::MAX, usize::MAX);
    let mut renewed = std::time::Instant::now();
    loop {
        std::thread::sleep(Duration::from_secs(5));
        let load = server.load();
        if load != last {
            println!("rooms open: {}, live relays: {}", load.0, load.1);
            last = load;
        }
        if forwarded.is_some() && renewed.elapsed() > Duration::from_secs(3600) {
            renewed = std::time::Instant::now();
            if let Err(e) = upnp::forward(server.port()) {
                println!("renewing the router's port forward failed: {e}");
            }
        }
    }
}

/// Where the test friend keeps its own sign-in and its stand-in game folder.
const TEST_FRIEND_DIR: &str = "multiplayer/test-friend";

#[allow(clippy::too_many_arguments)]
fn test_friend(
    store: &Store,
    code: &str,
    name: &str,
    password: Option<&str>,
    room: Option<String>,
    career: Option<String>,
    stay: Duration,
    probe: bool,
    game: Option<std::path::PathBuf>,
) -> Result<()> {
    let dir = store.root.join(TEST_FRIEND_DIR);
    std::fs::create_dir_all(&dir)?;
    let session = mpauth::sign_in_host(&dir, code, name, password).map_err(|e| anyhow!(e))?;
    println!(
        "test friend {} signed in to {}",
        session.name,
        session
            .server
            .as_deref()
            .unwrap_or("?")
            .split('#')
            .next()
            .unwrap_or("?")
    );
    let api = Api::for_session(&session);
    if let Some(career) = career {
        let joined = api
            .join_career(&career)
            .map_err(|e| anyhow!("joining the career {career}: {e:?}"))?;
        println!(
            "joined the shared career \"{}\" ({} member(s), save version {})",
            joined.name,
            joined.members.len(),
            joined.version
        );
    }
    let local = {
        let game_dir = game
            .or_else(|| swofty_core::copies::find_active(&store.config()).map(|c| c.dir))
            .context("F1 25 not found; pass --game")?;
        let located = Game::locate(&store.root, game_dir)?;
        let profile = store.profile(&store.active_profile())?;
        Local::read(store, &profile, &located.identity)?
    };
    let until = Instant::now() + stay;
    let room = match room {
        Some(r) => r.trim().to_ascii_uppercase(),
        None => {
            println!("waiting for an open room on the server (open one in your launcher)...");
            loop {
                let rooms = api.rooms().map_err(|e| anyhow!("listing the rooms: {e:?}"))?;
                if let Some(r) = rooms.into_iter().next() {
                    println!("found {} \"{}\", hosted by {}", r.code, r.name, r.host.name);
                    break r.code;
                }
                if Instant::now() > until {
                    return Err(anyhow!("no room opened in time"));
                }
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    };
    let sim = SimGame::new(&dir.join("game")).context("starting the stand-in game")?;
    let joined = Session::join(
        Setup {
            api,
            game_dir: sim.dir().to_path_buf(),
            game: sim.clone(),
            local,
            relay: Relay::Turn,
            // The stand-in game is already running: nothing to launch, nothing to read afresh.
            room_launch: false,
            refresh: None,
        },
        &room,
    );
    let mut last = (String::new(), usize::MAX, 0u64);
    let mut next_probe = Instant::now() + Duration::from_secs(8);
    let mut next_stats = Instant::now() + Duration::from_secs(5);
    while Instant::now() < until {
        let state = joined.state();
        let phase = state.phase.to_string();
        let traffic = sim.traffic();
        for line in sim.take_log() {
            println!("  game command: {line}");
        }
        if phase != last.0 || state.members.len() != last.1 || traffic.datagrams / 50 != last.2 / 50 {
            let members: Vec<String> = state
                .members
                .iter()
                .map(|m| format!("{}{}", m.name, if m.host { " (host)" } else { "" }))
                .collect();
            println!(
                "[{}] {phase}; members: {}; from your game: {} datagram(s), {} byte(s)",
                clock(),
                members.join(", "),
                traffic.datagrams,
                traffic.bytes
            );
            last = (phase, state.members.len(), traffic.datagrams);
        }
        if state.phase.is_over() {
            break;
        }
        let relays_ready =
            state.members.len() > 1 && state.members.iter().all(|m| m.relay == RelayStatus::Ready);
        // The bridge takes the stand-in's port from its netplay.json at its next housekeeping, so the
        // first probe waits a little, and later ones repeat with the stats.
        if probe && state.phase == Phase::InLobby && relays_ready && Instant::now() >= next_probe {
            next_probe = Instant::now() + Duration::from_secs(30);
            let host = state.members.iter().find(|m| m.host).map(|m| m.number);
            match host.map(|h| sim.probe(h, 5)) {
                Some(Ok(())) => println!(
                    "sent the host's game 5 probe datagrams ({}); the host's bridge counts them as in from                      member {} (state/multiplayer.log, bridge stats)",
                    String::from_utf8_lossy(PROBE),
                    state.self_member.unwrap_or(0)
                ),
                Some(Err(e)) => println!("the probe failed: {e}"),
                None => println!("the probe has no host to go to"),
            }
        }
        if let Some(stats) = &state.bridge
            && Instant::now() >= next_stats
        {
            println!("  bridge: {}", stats_line(stats));
            next_stats = Instant::now() + Duration::from_secs(30);
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    drop(joined);
    let traffic = sim.traffic();
    println!(
        "test friend left: {} datagram(s), {} byte(s) arrived from the host's game",
        traffic.datagrams, traffic.bytes
    );
    Ok(())
}

/// The time of day, `HH:MM:SS` UTC, for the progress lines.
fn clock() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    format!("{:02}:{:02}:{:02}", secs / 3600 % 24, secs / 60 % 60, secs % 60)
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;
    use crate::mp_career::CareerCommand;

    #[derive(Parser)]
    struct Cli {
        #[command(subcommand)]
        mp: MpCommand,
    }

    fn parse(args: &[&str]) -> MpCommand {
        Cli::try_parse_from(std::iter::once("mp").chain(args.iter().copied()))
            .unwrap_or_else(|e| panic!("{args:?}: {e}"))
            .mp
    }

    #[test]
    fn host_runs_the_server_or_releases_a_name() {
        assert!(matches!(
            parse(&["host", "--port", "1234", "--no-upnp"]),
            MpCommand::Host { action: None, port: 1234, no_upnp: true, .. }
        ));
        let MpCommand::Host { action: Some(HostAction::Release { name, dir }), .. } =
            parse(&["host", "release", "Max Power", "--dir", "/tmp/h"])
        else {
            panic!("host release");
        };
        assert_eq!((name.as_str(), dir), ("Max Power", Some(std::path::PathBuf::from("/tmp/h"))));
        assert!(Cli::try_parse_from(["mp", "host", "--port", "1", "release", "Max"]).is_err());
    }

    #[test]
    fn career_leave_remove_and_delete_parse() {
        assert!(matches!(
            parse(&["career", "leave", "--career", "c-1"]),
            MpCommand::Career(CareerCommand::Leave { career: Some(_) })
        ));
        let MpCommand::Career(CareerCommand::Remove { player, career: None }) = parse(&["career", "remove", "Ben"])
        else {
            panic!("career remove");
        };
        assert_eq!(player, "Ben");
        assert!(matches!(
            parse(&["career", "delete", "c-1"]),
            MpCommand::Career(CareerCommand::Delete { .. })
        ));
    }
}