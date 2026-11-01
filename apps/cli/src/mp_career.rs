//! The `f1mp mp career` (and `f1mp league`) commands: the league career, driven alone (a solo
//! career kept in the launcher's data folder) or with friends on a host server, with races
//! reported by the game's UDP telemetry (see `swofty_core::league`).

use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use clap::Subcommand;
use swofty_core::league::solo::{self, SoloCareer};
use swofty_core::league::{
    self, Action, Focus, League, OfferKind, Part, Phase, PointsRule, RaceReport, RoundState, SessionKind,
    Setup, TeamChoice, Untracked, progress, udp,
};
use swofty_core::mpauth;
use swofty_core::multiplayer::rooms::{Api, LeagueState, SharedCareer};
use swofty_core::store::Store;

#[derive(Subcommand)]
pub enum CareerCommand {
    #[command(about = "List your league careers: solo ones and those on the host server")]
    List,
    #[command(
        about = "Start a league career (2016 to 2026 by default): alone with --solo, else on the host server, where friends join with its code"
    )]
    New {
        #[arg(help = "The career's name")]
        name: String,
        #[arg(long, help = "Drive it alone, without a host server")]
        solo: bool,
        #[arg(long, help = "Your name in a solo career (default: your multiplayer name)")]
        player: Option<String>,
        #[arg(long, default_value_t = league::FIRST_YEAR, help = "The first season")]
        from: u16,
        #[arg(long, default_value_t = league::LAST_YEAR, help = "The last season")]
        to: u16,
        #[arg(long, help = "Your race number")]
        number: Option<u8>,
        #[arg(
            long,
            default_value_t = 50,
            help = "The race distance you agree on, in percent"
        )]
        distance: u8,
        #[arg(long, help = "Don't score sprint races")]
        no_sprints: bool,
        #[arg(
            long,
            help = "Leave out rounds at circuits the game doesn't have, instead of simulating them"
        )]
        skip_missing: bool,
        #[arg(long, help = "Today's points for every year (no fastest-lap point)")]
        current_points: bool,
        #[arg(long, help = "Start in the slower half of the grid")]
        rookie: bool,
        #[arg(
            long,
            help = "Keep each season's starting line-up (no real stand-ins, moves or replaced drivers)"
        )]
        no_lineup_changes: bool,
    },
    #[command(about = "Join a friend's league career with its code (K-XXXXXX)")]
    Join {
        code: String,
        #[arg(long, help = "Your race number")]
        number: Option<u8>,
    },
    #[command(about = "Show the standings, the next round and the latest news")]
    Show {
        #[arg(long, help = "The career id (default: your newest league career)")]
        career: Option<String>,
        #[arg(long, help = "Show the whole drivers' table and every round")]
        all: bool,
    },
    #[command(about = "List this season's teams and drivers (the seats you can take)")]
    Teams {
        #[arg(long)]
        career: Option<String>,
    },
    #[command(about = "Pick your seat for the first season: a team, and optionally whose seat to take")]
    Seat {
        #[arg(help = "The team's number from `career teams`")]
        team: u8,
        #[arg(long, help = "The driver whose seat you take (a key from `career teams`)")]
        replaces: Option<String>,
        #[arg(long)]
        career: Option<String>,
    },
    #[command(about = "Start the first season (the career's owner)")]
    Begin {
        #[arg(long)]
        career: Option<String>,
    },
    #[command(about = "Listen to the game's telemetry and count each race as it ends")]
    Listen {
        #[arg(long)]
        career: Option<String>,
        #[arg(long, help = "Keep the races only; count them later with `career report`")]
        no_report: bool,
    },
    #[command(about = "The races the game reported that the launcher kept")]
    Races,
    #[command(about = "Count a race the game reported (default: the newest one not counted yet)")]
    Report {
        #[arg(help = "The race's session id from `career races`")]
        session: Option<String>,
        #[arg(long, help = "The round to count it for (default: the next round)")]
        round: Option<u8>,
        #[arg(long, help = "It was the sprint", conflicts_with_all = ["race", "qualifying"])]
        sprint: bool,
        #[arg(long, help = "It was the main race", conflicts_with = "qualifying")]
        race: bool,
        #[arg(long, help = "It was the qualifying", conflicts_with = "practice")]
        qualifying: bool,
        #[arg(long, help = "It was a practice session (your practice programmes)")]
        practice: bool,
        #[arg(long, help = "Count it although it was at another track than the round's")]
        force: bool,
        #[arg(long)]
        career: Option<String>,
    },
    #[command(about = "Simulate a round instead of driving it (default: the next round)")]
    Simulate {
        #[arg(long)]
        round: Option<u8>,
        #[arg(long)]
        career: Option<String>,
    },
    #[command(about = "Leave a round out (default: the next round)")]
    Skip {
        #[arg(long)]
        round: Option<u8>,
        #[arg(long)]
        career: Option<String>,
    },
    #[command(about = "Take back the last round that was counted")]
    Undo {
        #[arg(long)]
        career: Option<String>,
    },
    #[command(about = "Simulate your practice of the next round (fewer resource points than driving it)")]
    Practice {
        #[arg(long)]
        round: Option<u8>,
        #[arg(long)]
        career: Option<String>,
    },
    #[command(
        about = "Spend resource points on your car: aero, chassis, powertrain or durability (on the car two rounds later)"
    )]
    Develop {
        part: String,
        #[arg(long)]
        career: Option<String>,
    },
    #[command(
        about = "Your driver: ratings, recognition, contract, earnings, resource points and the car's development"
    )]
    Driver {
        #[arg(long)]
        career: Option<String>,
    },
    #[command(about = "Sign an offer for next season, when the season is over")]
    Sign {
        #[arg(help = "The team's number from the offers `career show` lists")]
        team: u8,
        #[arg(long, help = "The driver whose seat you take")]
        replaces: Option<String>,
        #[arg(
            long,
            help = "Ask for more salary (`salary`, a quarter more) or more say in the development (`development`, a fifth more resource points)"
        )]
        focus: Option<String>,
        #[arg(long)]
        career: Option<String>,
    },
    #[command(about = "Go on to next season although not everyone signed (the career's owner)")]
    NextSeason {
        #[arg(long)]
        career: Option<String>,
    },
    #[command(about = "Show the game's UDP telemetry setting, or turn it on (close the game first)")]
    Telemetry {
        #[arg(long, help = "Turn it on")]
        enable: bool,
    },
    #[command(about = "Delete a solo career (careers on a host server stay)")]
    Delete {
        #[arg(help = "The solo career's id from `career list`")]
        career: String,
    },
}

fn api(store: &Store) -> Result<Api> {
    let session = mpauth::load(&store.root).filter(|s| s.server.is_some()).context(
        "not signed in to a host server; run `f1mp mp join <server code> --name <your name>` first, \
         or drive alone with `career new <name> --solo`",
    )?;
    Ok(Api::for_session(&session))
}

fn races_dir(store: &Store) -> PathBuf {
    store.root.join(udp::RACES_DIR)
}

/// The career a command works on: a solo one on this PC, or one on the host server.
enum Target {
    Solo { root: PathBuf, career: SoloCareer },
    Server { api: Api, career: SharedCareer },
}

impl Target {
    fn name(&self) -> &str {
        match self {
            Target::Solo { career, .. } => &career.name,
            Target::Server { career, .. } => &career.name,
        }
    }

    fn code(&self) -> Option<&str> {
        match self {
            Target::Solo { .. } => None,
            Target::Server { career, .. } => Some(&career.code),
        }
    }

    fn me(&self) -> Result<String> {
        match self {
            Target::Solo { .. } => Ok(solo::ME.to_owned()),
            Target::Server { api, .. } => Ok(api.me().map_err(|e| anyhow!("{}", e.message))?.id),
        }
    }

    fn load(&self) -> Result<Option<League>> {
        match self {
            Target::Solo { career, .. } => Ok(Some(career.league.clone())),
            Target::Server { api, career } => Ok(api
                .league(&career.id)
                .map_err(|e| anyhow!("loading the career: {}", e.message))?
                .league),
        }
    }

    fn act(&mut self, action: Action) -> Result<League> {
        let warnings = match self {
            Target::Solo { root, career } => {
                let (c, warnings) = solo::act(root, &career.id, action).map_err(|e| anyhow!("{e}"))?;
                *career = c;
                warnings
            }
            Target::Server { api, career } => {
                let state: LeagueState = api
                    .league_action(&career.id, &action)
                    .map_err(|e| anyhow!("{}", e.message))?;
                for w in &state.warnings {
                    println!("note: {w}");
                }
                return state.league.context("the server sent no career");
            }
        };
        for w in &warnings {
            println!("note: {w}");
        }
        match self {
            Target::Solo { career, .. } => Ok(career.league.clone()),
            Target::Server { .. } => unreachable!("returned above"),
        }
    }
}

fn signed_in(store: &Store) -> bool {
    mpauth::load(&store.root).is_some_and(|s| s.server.is_some())
}

/// The league career `id` (a solo id, a server career's id or code), or the one changed last.
fn pick(store: &Store, id: Option<String>) -> Result<Target> {
    let root = store.root.clone();
    if let Some(id) = id.as_deref().filter(|id| solo::is_solo(id)) {
        let career = solo::load(&root, id).map_err(|e| anyhow!("{e}"))?;
        return Ok(Target::Solo { root, career });
    }
    let solos = solo::list(&root);
    if id.is_none() && !signed_in(store) {
        let career = solos.into_iter().next().context(
            "no league career yet; start one alone with `career new <name> --solo`, or join a host server first",
        )?;
        return Ok(Target::Solo { root, career });
    }
    let api = api(store)?;
    let careers = match api.careers() {
        Ok(c) => c,
        // The host server is off: a solo career still works.
        Err(e) if id.is_none() && !solos.is_empty() => {
            eprintln!("note: {}; using your solo career", e.message);
            let career = solos.into_iter().next().expect("checked");
            return Ok(Target::Solo { root, career });
        }
        Err(e) => bail!("listing your careers: {}", e.message),
    };
    let leagues: Vec<SharedCareer> = careers.into_iter().filter(|c| c.mode == league::MODE).collect();
    match id {
        Some(id) => {
            let career = leagues
                .into_iter()
                .find(|c| c.id == id || c.code.eq_ignore_ascii_case(&id))
                .with_context(|| format!("you're in no league career {id}"))?;
            Ok(Target::Server { api, career })
        }
        None => {
            let server = leagues.into_iter().max_by_key(|c| c.updated_at);
            let solo = solos.into_iter().next();
            match (server, solo) {
                (Some(s), Some(c)) if c.updated > s.updated_at => Ok(Target::Solo { root, career: c }),
                (Some(career), _) => Ok(Target::Server { api, career }),
                (None, Some(career)) => Ok(Target::Solo { root, career }),
                (None, None) => bail!("you're in no league career; start one with `career new <name>`"),
            }
        }
    }
}

/// Your name for a new solo career.
fn player_name(store: &Store, asked: Option<String>) -> String {
    asked
        .or_else(|| mpauth::load(&store.root).map(|s| s.name))
        .filter(|n| !n.trim().is_empty())
        .or_else(|| std::env::var("USERNAME").ok())
        .unwrap_or_else(|| "Player".to_owned())
}

pub fn command(store: &Store, cmd: CareerCommand) -> Result<()> {
    match cmd {
        CareerCommand::Telemetry { enable } => return telemetry(enable),
        CareerCommand::Races => return races(store),
        CareerCommand::Delete { career } => {
            if !solo::is_solo(&career) {
                bail!("only solo careers are deleted here; {career} isn't one");
            }
            solo::delete(&store.root, &career).map_err(|e| anyhow!("{e}"))?;
            println!("deleted the solo career {career}");
            return Ok(());
        }
        CareerCommand::List => {
            let solos = solo::list(&store.root);
            for c in &solos {
                let l = &c.league;
                println!(
                    "{}  {}  solo, {} ({} to {})",
                    c.id,
                    c.name,
                    l.year(),
                    l.setup.first_year,
                    l.setup.last_year
                );
            }
            let mut any = !solos.is_empty();
            if signed_in(store) {
                match api(store)?.careers() {
                    Ok(careers) => {
                        for c in careers.iter().filter(|c| c.mode == league::MODE) {
                            any = true;
                            let members: Vec<&str> = c.members.iter().map(|m| m.name.as_str()).collect();
                            println!("{}  {}  code {}  ({})", c.id, c.name, c.code, members.join(", "));
                        }
                    }
                    Err(e) => println!("(careers on the host server aren't listed: {})", e.message),
                }
            }
            if !any {
                println!(
                    "no league careers yet; start one with `career new <name>` (add --solo to drive alone)"
                );
            }
            return Ok(());
        }
        CareerCommand::New {
            name,
            solo: true,
            player,
            from,
            to,
            number,
            distance,
            no_sprints,
            skip_missing,
            current_points,
            rookie,
            no_lineup_changes,
        } => {
            let setup = setup_of(
                from,
                to,
                distance,
                no_sprints,
                skip_missing,
                current_points,
                rookie,
                no_lineup_changes,
            );
            let player = player_name(store, player);
            let career =
                solo::create(&store.root, &name, &player, setup, number).map_err(|e| anyhow!("{e}"))?;
            println!(
                "started the solo career \"{}\" ({from} to {to}), id {}",
                career.name, career.id
            );
            println!("pick your seat (`career teams`, `career seat <team>`), then start with `career begin`");
            print_teams(&career.league);
            return Ok(());
        }
        _ => {}
    }
    match cmd {
        CareerCommand::New {
            name,
            solo: _,
            player: _,
            from,
            to,
            number,
            distance,
            no_sprints,
            skip_missing,
            current_points,
            rookie,
            no_lineup_changes,
        } => {
            let api = api(store)?;
            let career = api
                .create_career(&name, league::MODE, serde_json::Value::Null)
                .map_err(|e| anyhow!("{}", e.message))?;
            let setup = setup_of(
                from,
                to,
                distance,
                no_sprints,
                skip_missing,
                current_points,
                rookie,
                no_lineup_changes,
            );
            let code = career.code.clone();
            let mut target = Target::Server { api, career };
            let league = target.act(Action::Start { setup, number })?;
            println!("started \"{}\" ({} to {})", target.name(), from, to);
            println!("your friends join with:  f1mp mp career join {code}");
            println!("then everyone picks a seat (`f1mp mp career teams`, `f1mp mp career seat <team>`)");
            println!("and you start the first season with `f1mp mp career begin`");
            print_teams(&league);
            Ok(())
        }
        CareerCommand::Join { code, number } => {
            let api = api(store)?;
            let career = api.join_career(&code).map_err(|e| anyhow!("{}", e.message))?;
            if career.mode != league::MODE {
                bail!("{} is a shared career, but not a league career", career.code);
            }
            let mut target = Target::Server { api, career };
            let league = target.act(Action::Enter { number })?;
            println!("joined \"{}\"", target.name());
            if league.phase == Phase::Setup {
                println!("pick your seat with `f1mp mp career seat <team>`:");
                print_teams(&league);
            } else {
                println!("the season is running; you get contract offers when it ends");
            }
            Ok(())
        }
        CareerCommand::Show { career, all } => {
            let target = pick(store, career)?;
            let me = target.me()?;
            match target.load()? {
                Some(league) => print_league(&target, &league, &me, all),
                None => println!("\"{}\" hasn't started yet", target.name()),
            }
            Ok(())
        }
        CareerCommand::Teams { career } => {
            let target = pick(store, career)?;
            let league = target.load()?.context("the career hasn't started yet")?;
            print_teams(&league);
            Ok(())
        }
        CareerCommand::Seat {
            team,
            replaces,
            career,
        } => {
            let mut target = pick(store, career)?;
            let league = target.act(Action::Seat { slot: team, replaces })?;
            let me = target.me()?;
            if let Some(e) = league.season().player_entry(&me) {
                let team = league.season().team(e.slot).map_or("?", |t| t.name.as_str());
                println!("you drive for {team} in {} (#{})", league.year(), e.number);
            }
            Ok(())
        }
        CareerCommand::Begin { career } => {
            let mut target = pick(store, career)?;
            let league = target.act(Action::Begin)?;
            let me = target.me()?;
            print_briefing(&league, &me);
            Ok(())
        }
        CareerCommand::Listen { career, no_report } => listen(store, career, no_report),
        CareerCommand::Report {
            session,
            round,
            sprint,
            race,
            qualifying,
            practice,
            force,
            career,
        } => {
            let mut target = pick(store, career)?;
            let league = target.load()?.context("the career hasn't started yet")?;
            let saved = udp::saved(&races_dir(store));
            let report = match session {
                Some(s) => {
                    let id = u64::from_str_radix(s.trim_start_matches("0x"), 16)
                        .with_context(|| format!("{s} is not a session id"))?;
                    saved
                        .into_iter()
                        .find(|r| r.session == id)
                        .context("no kept race has that session id")?
                }
                None => uncounted(&league, saved)
                    .pop()
                    .context("no race the game reported is waiting to be counted")?,
            };
            let kind = if sprint {
                Some(SessionKind::Sprint)
            } else if qualifying {
                Some(SessionKind::Qualifying)
            } else if practice {
                Some(SessionKind::Practice)
            } else if race {
                Some(SessionKind::Race)
            } else {
                None
            };
            let league = target.act(Action::Report {
                round,
                kind,
                force,
                report,
            })?;
            print_news_since(&league, 3);
            Ok(())
        }
        CareerCommand::Simulate { round, career } => {
            let mut target = pick(store, career)?;
            let league = target.act(Action::Simulate { round })?;
            print_news_since(&league, 3);
            Ok(())
        }
        CareerCommand::Skip { round, career } => {
            let mut target = pick(store, career)?;
            let league = target.act(Action::Skip { round })?;
            print_news_since(&league, 2);
            Ok(())
        }
        CareerCommand::Undo { career } => {
            let mut target = pick(store, career)?;
            let league = target.act(Action::Undo)?;
            print_news_since(&league, 1);
            Ok(())
        }
        CareerCommand::Sign {
            team,
            replaces,
            focus,
            career,
        } => {
            let focus = match focus.as_deref() {
                None => None,
                Some("salary") => Some(Focus::Salary),
                Some("development" | "dev") => Some(Focus::Development),
                Some(f) => bail!("ask for `salary` or `development`, not {f}"),
            };
            let mut target = pick(store, career)?;
            let league = target.act(Action::Sign {
                slot: team,
                replaces,
                focus,
            })?;
            print_news_since(&league, 2);
            Ok(())
        }
        CareerCommand::Practice { round, career } => {
            let mut target = pick(store, career)?;
            let league = target.act(Action::Practice { round })?;
            print_news_since(&league, 1);
            Ok(())
        }
        CareerCommand::Develop { part, career } => {
            let part = Part::parse(&part)
                .with_context(|| format!("{part} isn't a part: aero, chassis, powertrain or durability"))?;
            let mut target = pick(store, career)?;
            let league = target.act(Action::Develop { part })?;
            print_news_since(&league, 1);
            let me = target.me()?;
            if let Some(p) = league.player(&me) {
                println!("{} resource points left", p.rp);
            }
            Ok(())
        }
        CareerCommand::Driver { career } => {
            let target = pick(store, career)?;
            let me = target.me()?;
            let league = target.load()?.context("the career hasn't started yet")?;
            print_driver(&league, &me);
            Ok(())
        }
        CareerCommand::NextSeason { career } => {
            let mut target = pick(store, career)?;
            let league = target.act(Action::NextSeason)?;
            print_news_since(&league, 2);
            Ok(())
        }
        CareerCommand::Telemetry { .. }
        | CareerCommand::Races
        | CareerCommand::Delete { .. }
        | CareerCommand::List => unreachable!("handled above"),
    }
}

#[allow(clippy::too_many_arguments)]
fn setup_of(
    from: u16,
    to: u16,
    distance: u8,
    no_sprints: bool,
    skip_missing: bool,
    current_points: bool,
    rookie: bool,
    no_lineup_changes: bool,
) -> Setup {
    Setup {
        first_year: from,
        last_year: to,
        sprints: !no_sprints,
        untracked: if skip_missing {
            Untracked::Skip
        } else {
            Untracked::Simulate
        },
        points: if current_points {
            PointsRule::Current
        } else {
            PointsRule::Historic
        },
        distance,
        team_choice: if rookie {
            TeamChoice::Rookie
        } else {
            TeamChoice::Free
        },
        lineup_changes: !no_lineup_changes,
    }
}

/// Kept races not counted in the league yet, oldest first.
fn uncounted(league: &League, saved: Vec<RaceReport>) -> Vec<RaceReport> {
    let counted: Vec<u64> = league
        .seasons
        .iter()
        .flat_map(|s| &s.rounds)
        .flat_map(|r| {
            r.sessions
                .iter()
                .copied()
                .chain(r.practice.iter().map(|l| l.session))
        })
        .collect();
    saved
        .into_iter()
        .filter(|r| !counted.contains(&r.session))
        .collect()
}

fn telemetry(enable: bool) -> Result<()> {
    let path = udp::settings_path().context("no Documents folder")?;
    let settings = if enable {
        let s = udp::enable(&path).with_context(|| format!("changing {}", path.display()))?;
        println!(
            "turned the game's UDP telemetry on (start the game after this; it rewrites the file when it quits)"
        );
        s
    } else {
        udp::read_settings(&path).with_context(|| format!("reading {}", path.display()))?
    };
    println!(
        "UDP telemetry: {}, to {}:{}, format {}",
        if settings.enabled { "on" } else { "off" },
        settings.ip,
        settings.port,
        settings.format
    );
    if !settings.usable() {
        println!("race results need it on: run `f1mp mp career telemetry --enable` with the game closed");
    }
    Ok(())
}

fn races(store: &Store) -> Result<()> {
    let saved = udp::saved(&races_dir(store));
    if saved.is_empty() {
        println!("no races kept yet; `f1mp mp career listen` keeps every race the game reports");
    }
    for r in saved {
        let winner = r
            .cars
            .iter()
            .find(|c| c.position == 1)
            .map_or("?", |c| c.name.as_str());
        let mine = r
            .cars
            .iter()
            .find(|c| c.mine)
            .map(|c| format!(", you P{}", c.position));
        println!(
            "{:016x}  {}  {}, {} cars, first {winner}{}",
            r.session,
            r.track.and_then(league::track_name).unwrap_or("unknown track"),
            session_text(r.session_type),
            r.cars.len(),
            mine.unwrap_or_default()
        );
    }
    Ok(())
}

fn listen(store: &Store, career: Option<String>, no_report: bool) -> Result<()> {
    let path = udp::settings_path().context("no Documents folder")?;
    let settings = udp::read_settings(&path).with_context(|| format!("reading {}", path.display()))?;
    if !settings.usable() {
        bail!("the game's UDP telemetry is off; close the game and run `f1mp mp career telemetry --enable`");
    }
    let mut career = if no_report {
        None
    } else {
        Some(pick(store, career)?)
    };
    if let Some(t) = &career {
        println!("counting races in \"{}\"", t.name());
    }
    let listener = udp::Listener::start(settings.listen_addr(), races_dir(store)).with_context(|| {
        format!(
            "listening on {} (another telemetry tool may hold it)",
            settings.listen_addr()
        )
    })?;
    println!(
        "listening for the game's telemetry on {}; press Ctrl+C to stop",
        listener.addr()
    );
    let mut races = 0;
    let mut datagrams = 0;
    let mut unread = None;
    loop {
        std::thread::sleep(Duration::from_secs(1));
        let status = listener.status();
        if datagrams == 0 && status.datagrams > 0 {
            println!("the game is sending telemetry");
        }
        datagrams = status.datagrams;
        if status.unread != unread {
            unread = status.unread;
            if let Some((id, size)) = unread {
                println!(
                    "the game sent a packet this launcher can't read (packet {id}, {size} bytes); set the game's \
                     telemetry format to 2025 (Settings > Telemetry) and tell us which format it was"
                );
            }
        }
        if status.races == races {
            continue;
        }
        races = status.races;
        let Some(report) = status.last else { continue };
        println!(
            "{} ended at {} ({} cars)",
            session_text(report.session_type),
            report
                .track
                .and_then(league::track_name)
                .unwrap_or("an unknown track"),
            report.cars.len()
        );
        let Some(target) = &mut career else { continue };
        match target.act(Action::Report {
            round: None,
            kind: None,
            force: false,
            report,
        }) {
            Ok(league) => print_news_since(&league, 2),
            Err(e) => println!("not counted: {e} (count it with `f1mp mp career report`)"),
        }
    }
}

/// "race", "qualifying" or "practice", for a game session type.
fn session_text(session_type: u8) -> &'static str {
    match league::session_kind(session_type) {
        Some(SessionKind::Qualifying) => "qualifying",
        Some(SessionKind::Practice) => "practice",
        _ => "race",
    }
}

fn print_teams(league: &League) {
    let season = league.season();
    let cat = league::catalog_year(season.year);
    println!(
        "\n{} teams (number, name, car's gap to the fastest):",
        season.year
    );
    let mut teams = season.teams.clone();
    teams.sort_by(|a, b| a.pace.total_cmp(&b.pace));
    for t in teams {
        let drivers: Vec<String> = cat
            .and_then(|c| c.teams.iter().find(|x| x.slot == t.slot))
            .map(|c| {
                c.drivers
                    .iter()
                    .map(|d| {
                        let holder = league.players.iter().find(|p| {
                            p.seat
                                .as_ref()
                                .is_some_and(|s| s.slot == t.slot && s.replaces == d.key)
                        });
                        match holder {
                            Some(p) => format!("{} [{}, now {}]", d.name, d.key, p.name),
                            None => format!("{} [{}]", d.name, d.key),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        println!(
            "  {:>2}  {:<16} +{:.2}%  {}",
            t.slot,
            t.name,
            t.pace,
            drivers.join(", ")
        );
    }
}

fn print_briefing(league: &League, me: &str) {
    let Some(b) = league.briefing(me) else { return };
    println!(
        "\nnext: round {}/{} of {}, the {}{}",
        b.round,
        b.rounds,
        b.year,
        b.race,
        if b.sprint && !b.sprint_done {
            " (sprint weekend: sprint first)"
        } else {
            ""
        }
    );
    let place = if b.solo {
        "Grand Prix (F1 World > Grand Prix)"
    } else {
        "LAN lobby"
    };
    println!(
        "  in the game: {place}, the {} season box, {} track, {}% distance{}",
        b.year,
        b.track.and_then(league::track_name).unwrap_or(&b.race),
        b.distance,
        match (&b.team, &b.replaces) {
            (Some(t), Some(r)) => format!(", you drive for {t} as {r}"),
            _ => String::new(),
        }
    );
    println!(
        "  practice {}, qualifying {}, then the race; each counts by itself",
        if b.practice_done {
            "counted"
        } else {
            "first (drive it for resource points, or `career practice` to simulate it)"
        },
        if b.quali_done { "counted" } else { "next" }
    );
    for note in &b.lineup {
        println!("  line-up: {note}");
    }
    let mut aims = Vec::new();
    if let Some(g) = b.goal {
        aims.push(format!("team goal P{g} or better"));
    }
    if let Some(m) = &b.mate {
        aims.push(format!("team-mate {m}"));
    }
    if let Some(r) = &b.rival {
        aims.push(format!("rival {r}"));
    }
    if !aims.is_empty() {
        println!("  {}", aims.join(", "));
    }
}

fn print_record(league: &League, me: &str) {
    let s = league::career_stats(league, me);
    if s.seasons.is_empty() {
        return;
    }
    println!(
        "\nyour career: {} starts, {} wins, {} podiums, {} poles, {} fastest laps, {} points, {} titles{}",
        s.starts,
        s.wins,
        s.podiums,
        s.poles,
        s.fastest_laps,
        s.points,
        s.titles,
        if s.best_finish > 0 {
            format!(", best finish P{}", s.best_finish)
        } else {
            String::new()
        }
    );
    for line in &s.seasons {
        println!(
            "  {}  {:<16} P{:<3} {:>5} pts  {} wins  {} poles{}",
            line.year,
            line.team,
            line.position,
            line.points,
            line.wins,
            line.poles,
            if line.complete { "" } else { "  (running)" }
        );
    }
    let season = league.season();
    if let Some(entry) = season.player_entry(me) {
        if let Some(h) =
            league::team_mate(season, &entry.id).and_then(|m| league::head_to_head(season, &entry.id, &m.id))
        {
            println!(
                "  vs team-mate {}: qualifying {}:{}, races {}:{}, points {}:{}",
                h.other_name, h.quali.0, h.quali.1, h.race.0, h.race.1, h.points.0, h.points.1
            );
        }
        if let Some(h) = league
            .player(me)
            .and_then(|p| p.rival.as_deref())
            .and_then(|r| league::head_to_head(season, &entry.id, r))
        {
            println!(
                "  vs rival {}: qualifying {}:{}, races {}:{}, points {}:{}",
                h.other_name, h.quali.0, h.quali.1, h.race.0, h.race.1, h.points.0, h.points.1
            );
        }
    }
}

fn print_driver(league: &League, me: &str) {
    let Some(p) = league.player(me) else { return };
    let season = league.season();
    let ratings = |a: &league::Attributes| {
        format!(
            "experience {:.1}, racecraft {:.1}, awareness {:.1}, pace {:.1}",
            a.experience, a.racecraft, a.awareness, a.pace
        )
    };
    let a = p
        .attributes
        .unwrap_or_else(|| league::Attributes::from_rating(p.rating));
    println!("{} #{}: overall {:.1}", p.name, p.number, p.rating);
    println!("  {}", ratings(&a));
    let Some(entry) = season.player_entry(me) else {
        println!("  no seat this season");
        return;
    };
    let Some(team) = season.team(entry.slot) else {
        return;
    };
    if let Some(m) = league::team_mate(season, &entry.id)
        && let Some(ma) = league::driver_attributes(season.year, &m.id)
    {
        println!(
            "  team-mate {}: overall {:.1}, {}",
            m.name,
            ma.overall(),
            ratings(&ma)
        );
    }
    println!(
        "  {}: recognition {:.0} of 100{}",
        team.name,
        p.recognition,
        if p.recognition >= progress::NUMBER_ONE {
            ", the team's number one driver"
        } else {
            ""
        }
    );
    println!(
        "  contract until {}: {} a season + {} a point{}; earned {} in the career",
        p.contract_until.map_or("?".to_owned(), |y| y.to_string()),
        progress::money(u64::from(p.salary_k)),
        progress::money(u64::from(p.bonus_k)),
        match p.focus {
            Some(Focus::Salary) => " (you asked for salary)",
            Some(Focus::Development) =>
                " (you asked for a say in the development: a fifth more resource points)",
            None => "",
        },
        progress::money(p.earnings_k)
    );
    println!(
        "\n{}'s development, {} resource points to spend (`career develop <part>`):",
        team.name, p.rp
    );
    for part in Part::ALL {
        let level = season.part_level(entry.slot, part);
        let effect = if part == Part::Durability {
            "fewer breakdowns".to_owned()
        } else {
            format!("{:.2}% a lap", part.gain())
        };
        let next = if level >= progress::UPGRADE_LEVELS {
            "fully developed".to_owned()
        } else {
            format!("next level {} points", progress::upgrade_cost(level))
        };
        println!("  {:<13} level {level:>2}  ({effect} each; {next})", part.name());
    }
    let round = season.current_round();
    for u in season
        .upgrades
        .iter()
        .filter(|u| u.slot == entry.slot && u.ready > round)
    {
        println!(
            "  on order: {} level {}, on the car from round {}",
            u.part.name(),
            u.level,
            u.ready
        );
    }
    println!(
        "  the car's gap to the fastest: {:.2}% now ({:.2}% at the start of {}{}), the {} fastest car",
        season.pace_now(entry.slot),
        team.pace,
        season.year,
        if team.carry.abs() > 1e-4 {
            format!(", {:+.2}% carried over", team.carry)
        } else {
            String::new()
        },
        ordinal(season.pace_rank(entry.slot))
    );
}

fn ordinal(n: usize) -> String {
    let suffix = match (n % 10, n % 100) {
        (1, x) if x != 11 => "st",
        (2, x) if x != 12 => "nd",
        (3, x) if x != 13 => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

fn print_league(target: &Target, league: &League, me: &str, all: bool) {
    let season = league.season();
    let done = season.rounds.iter().filter(|r| r.done()).count();
    println!(
        "{}  ({} to {}, {})  season {}: {done}/{} rounds, {}",
        target.name(),
        league.setup.first_year,
        league.setup.last_year,
        target.code().map_or("solo".to_owned(), |c| format!("code {c}")),
        season.year,
        season.rounds.len(),
        match league.phase {
            Phase::Setup => "picking seats",
            Phase::Racing => "racing",
            Phase::SeasonEnd => "contracts for next season",
            Phase::Finished => "complete",
        }
    );
    for p in &league.players {
        let seat = season
            .player_entry(&p.id)
            .and_then(|e| season.team(e.slot))
            .map_or("no seat".to_owned(), |t| t.name.clone());
        let titles = if p.titles.is_empty() {
            String::new()
        } else {
            format!(
                ", champion {}",
                p.titles.iter().map(u16::to_string).collect::<Vec<_>>().join(", ")
            )
        };
        println!(
            "  {} #{}  {seat}  rating {:.1}, recognition {:.0}, {} resource points, {} a season{titles}",
            p.name,
            p.number,
            p.rating,
            p.recognition,
            p.rp,
            progress::money(u64::from(p.salary_k))
        );
    }
    let drivers = league::driver_standings(season);
    println!("\ndrivers:");
    for (i, d) in drivers
        .iter()
        .enumerate()
        .take(if all { drivers.len() } else { 10 })
    {
        let team = season.team(d.slot).map_or("", |t| t.code.as_str());
        let you = if d.player.as_deref() == Some(me) {
            "  <- you"
        } else {
            ""
        };
        println!(
            "  {:>2}. {:<22} {:<4} {:>5} pts  {} wins{you}",
            i + 1,
            d.name,
            team,
            d.points,
            d.wins
        );
    }
    if !all {
        for (i, d) in drivers
            .iter()
            .enumerate()
            .skip(10)
            .filter(|(_, d)| d.player.is_some())
        {
            println!("  {:>2}. {:<22} {:>9} pts", i + 1, d.name, d.points);
        }
    }
    println!("\nconstructors:");
    for (i, t) in league::team_standings(season)
        .iter()
        .enumerate()
        .take(if all { 11 } else { 5 })
    {
        println!("  {:>2}. {:<18} {:>5} pts", i + 1, t.name, t.points);
    }
    if all {
        println!("\nrounds:");
        for r in &season.rounds {
            let state = match r.state {
                RoundState::Pending => "-".to_owned(),
                RoundState::Raced => format!(
                    "raced{}",
                    r.by.as_ref().map_or(String::new(), |b| format!(" ({b})"))
                ),
                RoundState::Simulated => "simulated".into(),
                RoundState::Skipped => "skipped".into(),
            };
            let winner = r
                .race_result
                .iter()
                .find(|c| c.position == 1)
                .and_then(|c| season.entry(&c.entry))
                .map_or(String::new(), |e| format!(", {} won", e.name));
            println!("  {:>2}. {:<28} {state}{winner}", r.round, r.race);
        }
    }
    match league.phase {
        Phase::Racing => print_briefing(league, me),
        Phase::SeasonEnd => {
            println!("\nyour offers for {}:", season.year + 1);
            for o in league.offers(me) {
                let name = league::catalog_year(season.year + 1)
                    .and_then(|c| c.teams.iter().find(|t| t.slot == o.slot))
                    .map_or("?", |t| t.name.as_str());
                let kind = match o.kind {
                    OfferKind::Contract => " (under contract: stay)".to_owned(),
                    OfferKind::Renew => format!(
                        " (stay, {} year{}, {} a season + {} a point)",
                        o.years,
                        plural(o.years),
                        progress::money(u64::from(o.salary_k)),
                        progress::money(u64::from(o.bonus_k))
                    ),
                    OfferKind::Move => format!(
                        " ({} year{}, {} a season + {} a point)",
                        o.years,
                        plural(o.years),
                        progress::money(u64::from(o.salary_k)),
                        progress::money(u64::from(o.bonus_k))
                    ),
                };
                println!("  {:>2}  {name}{kind}", o.slot);
            }
            println!(
                "sign with `f1mp mp career sign <team>` (add `--focus salary` for a quarter more salary, or `--focus development` for more resource points)"
            );
        }
        _ => {}
    }
    print_record(league, me);
    println!("\nnews:");
    for n in league.news.iter().rev().take(6).rev() {
        println!("  {}  {}", n.year, n.text);
    }
}

fn plural(n: u8) -> &'static str {
    if n == 1 { "" } else { "s" }
}

fn print_news_since(league: &League, last: usize) {
    for n in league.news.iter().rev().take(last).rev() {
        println!("{}  {}", n.year, n.text);
    }
}
