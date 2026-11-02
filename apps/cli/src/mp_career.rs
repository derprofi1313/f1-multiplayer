//! The `f1mp mp career` (and `f1mp league`) commands: the league career, driven alone (a solo
//! career kept in the launcher's data folder) or with friends on a host server, with races
//! reported by the game's UDP telemetry (see `swofty_core::league`).

use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use clap::{Args, Subcommand, ValueEnum};
use swofty_core::careersync;
use swofty_core::league::solo::{self, SoloCareer};
use swofty_core::league::{
    self, Action, ApproachStatus, Archetype, Effect, Focus, League, NewsKind, Objective, ObjectiveKind, Offer,
    LineupMode, OfferKind, OfferStatus, Options, Part, Phase, PointsRule, PracticeFormat, Proposal, QualiFormat, RaceReport,
    RoundState, SessionKind, Setup, TeamChoice, Terms, UndoPreview, Untracked, WeekendRules, driver, gamesync,
    inbox, market, progress, rnd, udp, weekend,
};
use swofty_core::mpauth;
use swofty_core::multiplayer::rooms::{Api, ApiError, LeagueState, SharedCareer};
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
    #[command(
        about = "Take back the last round that was counted (with friends, once everyone agrees)"
    )]
    Undo {
        #[arg(long, help = "Only show what it would take back")]
        preview: bool,
        #[arg(long)]
        career: Option<String>,
    },
    #[command(about = "Say you're ready for the next round (or not, with --off); your friends see it")]
    Ready {
        #[arg(long, help = "Not ready after all")]
        off: bool,
        #[arg(long)]
        career: Option<String>,
    },
    #[command(
        about = "Show the open proposal (simulate, leave out or take back a round), or answer it: yes, no, or force (the owner decides for everyone)"
    )]
    Vote {
        #[arg(value_enum)]
        choice: Option<VoteChoice>,
        #[arg(long)]
        career: Option<String>,
    },
    #[command(
        about = "Your driver's profile: nationality and archetype (rookie 70, experienced 78, veteran 84), until your first session counts"
    )]
    Profile {
        #[arg(
            long,
            help = "Three-letter code, e.g. DEU (`--nationalities` lists them; \"none\" clears it)"
        )]
        nationality: Option<String>,
        #[arg(long, value_enum)]
        archetype: Option<ArchetypeArg>,
        #[arg(long, help = "List the nationalities to pick from")]
        nationalities: bool,
        #[arg(long)]
        career: Option<String>,
    },
    #[command(about = "The teams that approached you this season, and their offers once you met them")]
    Approaches {
        #[arg(long)]
        career: Option<String>,
    },
    #[command(about = "Meet a team that approached you, to hear its offer for next season")]
    Meet {
        #[arg(help = "The team's number from `career approaches`")]
        team: u8,
        #[arg(long)]
        career: Option<String>,
    },
    #[command(about = "Accept a team's offer from a meeting: sign for next season now")]
    Accept {
        #[arg(help = "The team's number from `career approaches`")]
        team: u8,
        #[arg(long, help = "The driver whose seat you take")]
        replaces: Option<String>,
        #[arg(long)]
        career: Option<String>,
    },
    #[command(about = "Turn down a team that approached you")]
    Decline {
        #[arg(help = "The team's number from `career approaches`")]
        team: u8,
        #[arg(long)]
        career: Option<String>,
    },
    #[command(
        about = "Ask a team for other terms: after a meeting, or for an offer at the season's end (three tries)"
    )]
    Negotiate {
        #[arg(help = "The team's number from `career approaches` or the offers `career show` lists")]
        team: u8,
        #[arg(long, help = "The salary a season, in thousands of dollars (2500 = $2.5M)")]
        salary: u32,
        #[arg(long, help = "The contract's length, 1 to 3 seasons")]
        years: u8,
        #[arg(long, help = "Ask to be the team's number one driver")]
        number_one: bool,
        #[arg(long)]
        career: Option<String>,
    },
    #[command(about = "Your mails, department requests and media questions")]
    Inbox {
        #[arg(long, help = "Every mail of the career, not only the latest")]
        all: bool,
        #[arg(long)]
        career: Option<String>,
    },
    #[command(about = "Answer a mail or media question with one of its choices")]
    Answer {
        #[arg(help = "The mail's number from `career inbox`")]
        id: u32,
        #[arg(help = "The choice's number from `career inbox` (1, 2, ...)")]
        choice: u8,
        #[arg(long)]
        career: Option<String>,
    },
    #[command(about = "Mark your mails read (default: all of them)")]
    Read {
        #[arg(help = "Up to this mail's number")]
        upto: Option<u32>,
        #[arg(long)]
        career: Option<String>,
    },
    #[command(about = "The development tree of your car: what is ordered, what you can order, the cost cap")]
    Tree {
        #[arg(help = "Only this part: aero, chassis, powertrain or durability")]
        part: Option<String>,
        #[arg(long)]
        career: Option<String>,
    },
    #[command(
        about = "Show the career options and the weekend rules everyone agrees to drive by; the owner changes them with the flags"
    )]
    Options(OptionsArgs),
    #[command(
        about = "The game sync of the career on this PC: what the game will show, and prepare or restore it"
    )]
    Sync {
        #[arg(value_enum, default_value = "show")]
        action: SyncAction,
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
        about = "Spend resource points on your car: aero, chassis, powertrain or durability (the next step of its main branch, or a node of `career tree` with --node)"
    )]
    Develop {
        part: String,
        #[arg(long, help = "The node's number from `career tree`")]
        node: Option<u8>,
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
    #[command(
        about = "Delete a career: a solo one, or one you own on the host server (the server keeps it aside)"
    )]
    Delete {
        #[arg(help = "The career's id from `career list` (or a host server career's code)")]
        career: String,
    },
    #[command(
        about = "Leave a career on the host server; in a running season your seat stays, driven by the game's AI"
    )]
    Leave {
        #[arg(long)]
        career: Option<String>,
    },
    #[command(
        about = "Remove a player from your career on the host server (the owner); their seat stays, driven by the game's AI"
    )]
    Remove {
        #[arg(help = "The player's name or id")]
        player: String,
        #[arg(long)]
        career: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum VoteChoice {
    /// Agree.
    Yes,
    /// Decline (the proposer withdraws it).
    No,
    /// Go ahead without waiting for the others (the owner).
    Force,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ArchetypeArg {
    Rookie,
    Experienced,
    Veteran,
}

impl From<ArchetypeArg> for Archetype {
    fn from(a: ArchetypeArg) -> Self {
        match a {
            ArchetypeArg::Rookie => Archetype::Rookie,
            ArchetypeArg::Experienced => Archetype::Experienced,
            ArchetypeArg::Veteran => Archetype::Veteran,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum SyncAction {
    /// What the game will show and whether this PC has it prepared.
    Show,
    /// Write the career's addon for the next round and turn it on in the profile (the next launch builds it).
    Prepare,
    /// Take the addon out of the profile and delete it.
    Restore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum OnOff {
    On,
    Off,
}

impl OnOff {
    fn on(self) -> bool {
        self == OnOff::On
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum QualiArg {
    Full,
    Short,
    OneShot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum PracticeArg {
    Full,
    OneSession,
    Short,
    None,
}

/// The game's `carDamage` values, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum DamageArg {
    Off,
    Reduced,
    Standard,
    Simulation,
}

/// The game's `safetyCarSetting` values, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum SafetyCarArg {
    Off,
    Reduced,
    Standard,
    Increased,
}

/// The game's `weather` values, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum WeatherArg {
    Clear,
    LightCloud,
    Overcast,
    LightRain,
    HeavyRain,
    Storm,
}

/// The game's `ruleSet` values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum RuleSetArg {
    PracticeAndQualifying,
    Race,
    TimeTrial,
    TimeAttack,
    CheckpointChallenge,
    Autocross,
    Drift,
    AverageSpeedZone,
    RivalDuel,
}

impl RuleSetArg {
    fn value(self) -> u8 {
        match self {
            RuleSetArg::PracticeAndQualifying => 0,
            RuleSetArg::Race => 1,
            RuleSetArg::TimeTrial => 2,
            RuleSetArg::TimeAttack => 4,
            RuleSetArg::CheckpointChallenge => 6,
            RuleSetArg::Autocross => 8,
            RuleSetArg::Drift => 9,
            RuleSetArg::AverageSpeedZone => 10,
            RuleSetArg::RivalDuel => 11,
        }
    }
}

/// A weekend rule, to stop agreeing on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum RuleName {
    Ai,
    Quali,
    Practice,
    Damage,
    SafetyCar,
    ParcFerme,
    EqualPerformance,
    Weather,
    RuleSet,
    /// Every rule.
    All,
}

#[derive(Args, Debug, Default)]
pub struct OptionsArgs {
    #[arg(long, value_name = "0-110", help = "The AI difficulty everyone races")]
    ai: Option<u8>,
    #[arg(long, value_enum, help = "The qualifying format")]
    quali: Option<QualiArg>,
    #[arg(long, value_enum, help = "The practice format")]
    practice: Option<PracticeArg>,
    #[arg(long, value_enum, help = "Car damage")]
    damage: Option<DamageArg>,
    #[arg(long, value_enum)]
    safety_car: Option<SafetyCarArg>,
    #[arg(long, value_enum)]
    parc_ferme: Option<OnOff>,
    #[arg(long, value_enum)]
    equal_performance: Option<OnOff>,
    #[arg(long, value_enum)]
    weather: Option<WeatherArg>,
    #[arg(long, value_enum)]
    rule_set: Option<RuleSetArg>,
    #[arg(
        long,
        value_enum,
        value_delimiter = ',',
        help = "Stop agreeing on these rules (comma-separated, or `all`)"
    )]
    unset: Vec<RuleName>,
    #[arg(long, value_enum, help = "Show the players' names and numbers in the game")]
    game_identity: Option<OnOff>,
    #[arg(long, value_enum, help = "Give the cars in the game the league's pace")]
    game_performance: Option<OnOff>,
    #[arg(long, value_enum, help = "Experimental: let each developed part change how the car feels")]
    game_parts: Option<OnOff>,
    #[arg(
        long,
        value_enum,
        help = "The AI drivers from next season on: `historic` (each year's real grid, the default) or `dynamic` (they move between teams, retire and make way for rookies; the game shows them in renamed seats)"
    )]
    lineups: Option<LineupsArg>,
    #[arg(long)]
    career: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum LineupsArg {
    Historic,
    Dynamic,
}

impl OptionsArgs {
    /// Whether any flag asks for a change.
    fn changes(&self) -> bool {
        let OptionsArgs {
            ai,
            quali,
            practice,
            damage,
            safety_car,
            parc_ferme,
            equal_performance,
            weather,
            rule_set,
            unset,
            game_identity,
            game_performance,
            game_parts,
            lineups,
            career: _,
        } = self;
        ai.is_some()
            || quali.is_some()
            || practice.is_some()
            || damage.is_some()
            || safety_car.is_some()
            || parc_ferme.is_some()
            || equal_performance.is_some()
            || weather.is_some()
            || rule_set.is_some()
            || !unset.is_empty()
            || game_identity.is_some()
            || game_performance.is_some()
            || game_parts.is_some()
            || lineups.is_some()
    }

    /// `options` with these flags applied: first the rules to forget, then the values set.
    fn apply(&self, mut options: Options) -> Options {
        let r = &mut options.rules;
        for name in &self.unset {
            match name {
                RuleName::Ai => r.ai_difficulty = None,
                RuleName::Quali => r.quali = None,
                RuleName::Practice => r.practice = None,
                RuleName::Damage => r.damage = None,
                RuleName::SafetyCar => r.safety_car = None,
                RuleName::ParcFerme => r.parc_ferme = None,
                RuleName::EqualPerformance => r.equal_performance = None,
                RuleName::Weather => r.weather = None,
                RuleName::RuleSet => r.rule_set = None,
                RuleName::All => *r = WeekendRules::default(),
            }
        }
        if let Some(v) = self.ai {
            r.ai_difficulty = Some(v);
        }
        if let Some(v) = self.quali {
            r.quali = Some(match v {
                QualiArg::Full => QualiFormat::Full,
                QualiArg::Short => QualiFormat::Short,
                QualiArg::OneShot => QualiFormat::OneShot,
            });
        }
        if let Some(v) = self.practice {
            r.practice = Some(match v {
                PracticeArg::Full => PracticeFormat::Full,
                PracticeArg::OneSession => PracticeFormat::OneSession,
                PracticeArg::Short => PracticeFormat::Short,
                PracticeArg::None => PracticeFormat::None,
            });
        }
        if let Some(v) = self.damage {
            r.damage = Some(v as u8);
        }
        if let Some(v) = self.safety_car {
            r.safety_car = Some(v as u8);
        }
        if let Some(v) = self.parc_ferme {
            r.parc_ferme = Some(v.on());
        }
        if let Some(v) = self.equal_performance {
            r.equal_performance = Some(v.on());
        }
        if let Some(v) = self.weather {
            r.weather = Some(v as u8);
        }
        if let Some(v) = self.rule_set {
            r.rule_set = Some(v.value());
        }
        if let Some(v) = self.game_identity {
            options.game_identity = v.on();
        }
        if let Some(v) = self.game_performance {
            options.game_performance = v.on();
        }
        if let Some(v) = self.game_parts {
            options.game_parts = v.on();
        }
        if let Some(v) = self.lineups {
            options.lineups = match v {
                LineupsArg::Historic => LineupMode::Historic,
                LineupsArg::Dynamic => LineupMode::Dynamic,
            };
        }
        options
    }
}

/// What to do after a refusal, by its reason (the host server's or the league's); `None` when
/// the refusal's own message says enough.
fn refusal_hint(reason: &str) -> Option<&'static str> {
    Some(match reason {
        "vote_open" => {
            "answer the open proposal first: `f1mp mp career vote` shows it, `vote yes` agrees, `vote no` declines"
        }
        "not_voter" => "only the career's active players vote; you left it or aren't one of its players",
        "no_vote" => "nobody proposed anything; `f1mp mp career show` shows where the career is",
        "left" => "you left this career; the game's AI drives your seat until the season ends",
        "not_yet" => "the host server runs an older F1 Multiplayer; ask the host to update it",
        "not_next" => "only the next round can be counted, simulated, left out or practised",
        "duplicate" => "this session is counted already; `f1mp mp career show --all` lists the rounds",
        "ask_kind" => {
            "the game didn't say which it was: count it with `f1mp mp career report --sprint` or `--race`"
        }
        "cost_cap" => "`f1mp mp career tree` shows what the team spent this season",
        "node_locked" | "max_level" => "`f1mp mp career tree <part>` shows the nodes you can order",
        "no_points" => "drive practice and races for more resource points",
        "too_late" => "order a node that builds faster (`f1mp mp career tree`), or wait for next season",
        "no_approach" => "`f1mp mp career approaches` lists the teams that approached you",
        "walked_away" => "the team's last offer stands; sign it at the season's end, or look at your other offers",
        "bad_offer" => "ask for a salary (in thousands) and 1 to 3 years: `negotiate <team> --salary 2500 --years 2`",
        "no_offer" => "`f1mp mp career show` lists your offers",
        "answered" | "no_mail" | "bad_choice" | "not_yours" => "`f1mp mp career inbox` shows your open mails",
        "bad_profile" => "`f1mp mp career profile --nationalities` lists the codes",
        "bad_rules" => "`f1mp mp career options --help` lists the values",
        "name_claimed" => "sign in from the PC you used before, or ask the host to run `f1mp mp host release <name>`",
        "secret_required" => "update F1 Multiplayer to sign in",
        "too_many" => "delete a career you own first: `f1mp mp career delete <career>`",
        _ => return None,
    })
}

/// A refusal as the CLI says it: the message, then what to do about it.
fn refusal(reason: &str, message: &str) -> anyhow::Error {
    match refusal_hint(reason) {
        Some(hint) => anyhow!("{message}\n  ({hint})"),
        None => anyhow!("{message}"),
    }
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
    Server {
        api: Api,
        career: SharedCareer,
        /// The league's version when this command last read it.
        version: Option<u64>,
    },
}

impl Target {
    fn name(&self) -> &str {
        match self {
            Target::Solo { career, .. } => &career.name,
            Target::Server { career, .. } => &career.name,
        }
    }

    /// The career's id: a solo id, or the host server's.
    fn id(&self) -> &str {
        match self {
            Target::Solo { career, .. } => &career.id,
            Target::Server { career, .. } => &career.id,
        }
    }

    fn server(&self) -> Result<(&Api, &SharedCareer)> {
        match self {
            Target::Server { api, career, .. } => Ok((api, career)),
            Target::Solo { .. } => bail!("\"{}\" is a solo career, not one on the host server", self.name()),
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

    fn load(&mut self) -> Result<Option<League>> {
        match self {
            Target::Solo { career, .. } => Ok(Some(career.league.clone())),
            Target::Server { api, career, version } => {
                let state = api
                    .league(&career.id)
                    .map_err(|e| anyhow!("loading the career: {}", e.message))?;
                *version = Some(state.version);
                Ok(state.league)
            }
        }
    }

    /// Apply ction. Undo, Simulate and Skip on the host server name the version they were
    /// decided on, so they can't take back or pass over what a friend counted meanwhile.
    fn act(&mut self, action: Action) -> Result<League> {
        let guarded = matches!(action, Action::Undo | Action::Simulate { .. } | Action::Skip { .. });
        if guarded && matches!(self, Target::Server { version: None, .. }) {
            self.load()?;
        }
        let warnings = match self {
            Target::Solo { root, career } => {
                let (c, warnings) = solo::act(root, &career.id, action).map_err(|e| refusal(e.reason(), &e.to_string()))?;
                *career = c;
                warnings
            }
            Target::Server { api, career, version } => {
                let base = if guarded { *version } else { None };
                let state: LeagueState = api
                    .league_action_at(&career.id, &action, base)
                    .map_err(|e| server_error(&e))?;
                *version = Some(state.version);
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

/// A host server's refusal as the CLI says it.
fn server_error(e: &ApiError) -> anyhow::Error {
    match (e.reason.as_str(), e.retry_after) {
        ("stale", _) => anyhow!("the career changed meanwhile; check `f1mp mp career show` and try again"),
        ("rate_limited", Some(secs)) => anyhow!("{}
  (try again in {secs} seconds)", e.message),
        (reason, _) => refusal(reason, &e.message),
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
            Ok(Target::Server {
                api,
                career,
                version: None,
            })
        }
        None => {
            let server = leagues.into_iter().max_by_key(|c| c.updated_at);
            let solo = solos.into_iter().next();
            match (server, solo) {
                (Some(s), Some(c)) if c.updated > s.updated_at => Ok(Target::Solo { root, career: c }),
                (Some(career), _) => Ok(Target::Server {
                api,
                career,
                version: None,
            }),
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

/// Runs a career command; `profile` is the profile the game sync works on.
pub fn command(store: &Store, profile: &str, cmd: CareerCommand) -> Result<()> {
    match cmd {
        CareerCommand::Telemetry { enable } => return telemetry(enable),
        CareerCommand::Races => return races(store),
        CareerCommand::Delete { career } if solo::is_solo(&career) => {
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
            let mut target = Target::Server {
                api,
                career,
                version: None,
            };
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
            let mut target = Target::Server {
                api,
                career,
                version: None,
            };
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
            let mut target = pick(store, career)?;
            let me = target.me()?;
            match target.load()? {
                Some(league) => print_league(&target, &league, &me, all),
                None => println!("\"{}\" hasn't started yet", target.name()),
            }
            Ok(())
        }
        CareerCommand::Teams { career } => {
            let mut target = pick(store, career)?;
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
            print_vote(&league, &target.me()?);
            Ok(())
        }
        CareerCommand::Skip { round, career } => {
            let mut target = pick(store, career)?;
            let league = target.act(Action::Skip { round })?;
            print_news_since(&league, 2);
            print_vote(&league, &target.me()?);
            Ok(())
        }
        CareerCommand::Undo { preview, career } => {
            let mut target = pick(store, career)?;
            let before = target.load()?.context("the career hasn't started yet")?;
            let Some(what) = league::undo_preview(&before) else {
                bail!("nothing to take back: no round of this season is counted");
            };
            let me = target.me()?;
            print_undo_preview(&what, &me, &before);
            if preview {
                return Ok(());
            }
            let league = target.act(Action::Undo)?;
            print_news_since(&league, 1);
            print_vote(&league, &me);
            Ok(())
        }
        CareerCommand::Ready { off, career } => {
            let mut target = pick(store, career)?;
            let league = target.act(Action::Ready { ready: !off })?;
            println!(
                "{}",
                if off {
                    "you're not ready for the next round"
                } else {
                    "you're ready for the next round"
                }
            );
            print_ready(&league, &target.me()?);
            Ok(())
        }
        CareerCommand::Vote { choice, career } => {
            let mut target = pick(store, career)?;
            let me = target.me()?;
            let Some(choice) = choice else {
                let league = target.load()?.context("the career hasn't started yet")?;
                if league.vote.is_none() {
                    println!("there is no open proposal");
                }
                print_vote(&league, &me);
                return Ok(());
            };
            let league = target.act(Action::Vote {
                yes: choice != VoteChoice::No,
                force: choice == VoteChoice::Force,
            })?;
            print_news_since(&league, 3);
            print_vote(&league, &me);
            Ok(())
        }
        CareerCommand::Profile {
            nationality,
            archetype,
            nationalities,
            career,
        } => {
            if nationalities {
                for (code, name) in driver::NATIONALITIES {
                    println!("  {code}  {name}");
                }
                return Ok(());
            }
            let mut target = pick(store, career)?;
            let me = target.me()?;
            let league = if nationality.is_none() && archetype.is_none() {
                target.load()?.context("the career hasn't started yet")?
            } else {
                let league = target.act(Action::Profile {
                    nationality: nationality.as_deref().map(nationality_arg),
                    archetype: archetype.map(Archetype::from),
                })?;
                print_news_since(&league, 1);
                league
            };
            print_profile(&league, &me);
            Ok(())
        }
        CareerCommand::Approaches { career } => {
            let mut target = pick(store, career)?;
            let me = target.me()?;
            let league = target.load()?.context("the career hasn't started yet")?;
            print_approaches(&league, &me, true);
            Ok(())
        }
        CareerCommand::Meet { team, career } => {
            let mut target = pick(store, career)?;
            let me = target.me()?;
            let league = target.act(Action::Meet { slot: team })?;
            print_news_since(&league, 1);
            print_approaches(&league, &me, false);
            Ok(())
        }
        CareerCommand::Accept {
            team,
            replaces,
            career,
        } => {
            let mut target = pick(store, career)?;
            let league = target.act(Action::AcceptApproach { slot: team, replaces })?;
            print_news_since(&league, 2);
            Ok(())
        }
        CareerCommand::Decline { team, career } => {
            let mut target = pick(store, career)?;
            let league = target.act(Action::DeclineApproach { slot: team })?;
            print_news_since(&league, 1);
            println!("you turned down {}", next_team_name(&league, team));
            Ok(())
        }
        CareerCommand::Negotiate {
            team,
            salary,
            years,
            number_one,
            career,
        } => {
            let mut target = pick(store, career)?;
            let me = target.me()?;
            let league = target.act(Action::Negotiate {
                slot: team,
                terms: Terms {
                    salary_k: salary,
                    years,
                    number_one,
                },
            })?;
            print_news_since(&league, 1);
            match negotiated_offer(&league, &me, team) {
                Some(o) => println!("{}: {}", next_team_name(&league, team), offer_text(o)),
                None => println!("{} has no offer for you", next_team_name(&league, team)),
            }
            Ok(())
        }
        CareerCommand::Inbox { all, career } => {
            let mut target = pick(store, career)?;
            let me = target.me()?;
            let league = target.load()?.context("the career hasn't started yet")?;
            print_inbox(&league, &me, all);
            Ok(())
        }
        CareerCommand::Answer { id, choice, career } => {
            let index = choice_index(choice)?;
            let mut target = pick(store, career)?;
            let league = target.act(Action::Answer {
                news: id,
                choice: index,
            })?;
            if let Some(c) = league
                .news
                .iter()
                .find(|n| n.id == id)
                .and_then(|n| n.choices.get(usize::from(index)))
            {
                println!("you answered: {}{}", c.template, effect_text(&c.effect));
            }
            print_news_since(&league, 1);
            Ok(())
        }
        CareerCommand::Read { upto, career } => {
            let mut target = pick(store, career)?;
            let before = target.load()?.context("the career hasn't started yet")?;
            let upto = upto.unwrap_or(before.news_seq);
            let me = target.me()?;
            let league = target.act(Action::ReadMail { upto })?;
            match inbox::unread(&league, &me).len() {
                0 => println!("your inbox is read"),
                n => println!("{n} unread mail{} left", if n == 1 { "" } else { "s" }),
            }
            Ok(())
        }
        CareerCommand::Tree { part, career } => {
            let part = part.as_deref().map(parse_part).transpose()?;
            let mut target = pick(store, career)?;
            let me = target.me()?;
            let league = target.load()?.context("the career hasn't started yet")?;
            print_tree(&league, &me, part);
            Ok(())
        }
        CareerCommand::Options(args) => {
            let mut target = pick(store, args.career.clone())?;
            let league = target.load()?.context("the career hasn't started yet")?;
            let league = if args.changes() {
                let options = args.apply(league.options.clone());
                let league = target.act(Action::SetOptions { options })?;
                print_news_since(&league, 1);
                league
            } else {
                league
            };
            print_options(&league.options);
            Ok(())
        }
        CareerCommand::Sync { action, career } => {
            let target = pick(store, career)?;
            sync(store, profile, target, action)
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
        CareerCommand::Develop { part, node, career } => {
            let part = parse_part(&part)?;
            let mut target = pick(store, career)?;
            let league = target.act(match node {
                Some(node) => Action::DevelopNode { part, node },
                None => Action::Develop { part },
            })?;
            print_news_since(&league, 1);
            let me = target.me()?;
            if let Some(p) = league.player(&me) {
                let season = league.season();
                let spent = season
                    .player_entry(&me)
                    .map_or(0, |e| rnd::spent(season, e.slot));
                println!(
                    "{} resource points left; the team spent {spent} of {} this season",
                    p.rp,
                    rnd::COST_CAP
                );
            }
            Ok(())
        }
        CareerCommand::Driver { career } => {
            let mut target = pick(store, career)?;
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
        CareerCommand::Delete { career } => {
            let target = pick(store, Some(career))?;
            let (api, shared) = target.server()?;
            api.delete_career(&shared.id).map_err(|e| server_error(&e))?;
            println!("deleted \"{}\" on the host server", shared.name);
            Ok(())
        }
        CareerCommand::Leave { career } => {
            let target = pick(store, career)?;
            let (api, shared) = target.server()?;
            api.leave_career(&shared.id).map_err(|e| server_error(&e))?;
            println!("you left \"{}\"", shared.name);
            Ok(())
        }
        CareerCommand::Remove { player, career } => {
            let target = pick(store, career)?;
            let (api, shared) = target.server()?;
            let member = shared
                .members
                .iter()
                .find(|m| m.id == player)
                .or_else(|| shared.members.iter().find(|m| m.name.eq_ignore_ascii_case(player.trim())))
                .with_context(|| format!("\"{}\" has no member {player}", shared.name))?;
            let state = api
                .remove_member(&shared.id, &member.id)
                .map_err(|e| server_error(&e))?;
            println!("removed {} from \"{}\"", member.name, shared.name);
            if let Some(league) = &state.league {
                print_news_since(league, 1);
            }
            Ok(())
        }
        CareerCommand::Telemetry { .. } | CareerCommand::Races | CareerCommand::List => {
            unreachable!("handled above")
        }
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
    for o in &b.objectives {
        let mut reward = Vec::new();
        if o.reward_rp > 0 {
            reward.push(format!("+{} resource points", o.reward_rp));
        }
        if o.reward_recognition > 0.0 {
            reward.push(format!("+{} recognition", o.reward_recognition));
        }
        println!(
            "  objective: {}{}{}",
            objective_text(o),
            if reward.is_empty() {
                String::new()
            } else {
                format!(" ({})", reward.join(", "))
            },
            match o.met {
                None => "",
                Some(true) => "  - met",
                Some(false) => "  - missed",
            }
        );
    }
    let rules = weekend::describe(&b.rules);
    if !rules.is_empty() {
        let rules: Vec<String> = rules.iter().map(|(l, v)| format!("{l} {v}")).collect();
        println!("  agreed rules: {}", rules.join(", "));
    }
    if let Some(g) = &b.game {
        println!("  the game shows you as {}", identity_text(g));
    }
    if !b.solo {
        if b.ready.is_empty() {
            println!("  nobody is ready yet (`f1mp mp career ready`)");
        } else {
            println!("  ready: {}", names(&b.ready));
        }
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
    println!(
        "  {}{}",
        ratings(&a),
        a.focus.map_or_else(String::new, |f| format!(", focus {f:.1}"))
    );
    if p.nationality.is_some() || p.archetype.is_some() {
        println!(
            "  {}{}",
            p.archetype.map_or("", Archetype::name),
            p.nationality
                .as_deref()
                .map_or_else(String::new, |c| format!(
                    " from {}",
                    driver::nationality_name(c).unwrap_or(c)
                ))
        );
    }
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
    if let Some(until) = p.number_one_until {
        println!("  number-one clause until {until}");
    }
    println!(
        "\n{}'s development, {} resource points to spend; the team spent {} of {} this season (`career tree` shows every node):",
        team.name,
        p.rp,
        rnd::spent(season, entry.slot),
        rnd::COST_CAP
    );
    let profile = rnd::car_profile(season, entry.slot, season.current_round());
    for part in Part::ALL {
        let gained = match part {
            Part::Aero => profile.aero,
            Part::Chassis => profile.chassis,
            Part::Powertrain => profile.powertrain,
            Part::Durability => profile.durability,
        } + 0.0; // an empty sum is -0.0
        let effect = if part == Part::Durability {
            format!("{:.0}% fewer breakdowns", gained * 100.0)
        } else {
            format!("{gained:.2}% a lap")
        };
        let next = match rnd::next_main(season, entry.slot, part) {
            Some(n) => format!("next: {} for {} points", n.name, n.cost),
            None => "main branch complete".to_owned(),
        };
        println!("  {:<13} {effect:<22} {next}", part.name());
    }
    let round = season.current_round();
    for u in season
        .upgrades
        .iter()
        .filter(|u| u.slot == entry.slot && u.ready > round && !u.failed)
    {
        println!(
            "  on order: {} {}, on the car from round {}",
            u.part.name(),
            u.node
                .and_then(|id| rnd::node(u.part, id))
                .map_or_else(|| format!("level {}", u.level), |n| n.name.to_owned()),
            u.ready
        );
    }
    for u in season
        .upgrades
        .iter()
        .filter(|u| u.slot == entry.slot && u.failed)
    {
        println!(
            "  failed: {} {}",
            u.part.name(),
            u.node
                .and_then(|id| rnd::node(u.part, id))
                .map_or("", |n| n.name)
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
            "  {} #{}  {seat}  rating {:.1}, recognition {:.0}, {} resource points, {} a season{titles}{}",
            p.name,
            p.number,
            p.rating,
            p.recognition,
            p.rp,
            progress::money(u64::from(p.salary_k)),
            if p.left {
                " (left the career; the game's AI drives the seat)"
            } else {
                ""
            }
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
                println!("  {:>2}  {}: {}", o.slot, next_team_name(league, o.slot), offer_text(o));
            }
            println!(
                "sign with `f1mp mp career sign <team>` (add `--focus salary` for a quarter more salary, or `--focus development` for more resource points)"
            );
            println!("or ask for other terms: `f1mp mp career negotiate <team> --salary 2500 --years 2 [--number-one]`");
        }
        _ => {}
    }
    if league.phase == Phase::Racing {
        print_approaches(league, me, false);
    }
    print_vote(league, me);
    print_record(league, me);
    println!("\nnews:");
    for n in latest_news(league, 6) {
        println!("  {}  {}", n.year, n.text);
    }
    let unread = inbox::unread(league, me).len();
    if unread > 0 {
        println!(
            "\n{unread} unread mail{} (`f1mp mp career inbox`)",
            if unread == 1 { "" } else { "s" }
        );
    }
}

/// The latest `n` news everyone reads, oldest first (mails are in the inbox).
fn latest_news(league: &League, n: usize) -> Vec<&league::News> {
    let mut news: Vec<&league::News> = league
        .news
        .iter()
        .rev()
        .filter(|x| x.kind == NewsKind::News)
        .take(n)
        .collect();
    news.reverse();
    news
}

fn parse_part(s: &str) -> Result<Part> {
    Part::parse(s).with_context(|| format!("{s} isn't a part: aero, chassis, powertrain or durability"))
}

/// A `--nationality` value as `Action::Profile` takes it: the code in capitals, empty for "none".
fn nationality_arg(s: &str) -> String {
    let s = s.trim();
    if s.eq_ignore_ascii_case("none") {
        String::new()
    } else {
        s.to_ascii_uppercase()
    }
}

/// A choice as the CLI numbers it (from 1) as `Action::Answer` takes it (from 0).
fn choice_index(choice: u8) -> Result<u8> {
    choice
        .checked_sub(1)
        .context("the choices are numbered from 1, as `f1mp mp career inbox` shows them")
}

/// The name of next season's team `slot`.
fn next_team_name(league: &League, slot: u8) -> String {
    league::catalog_year(league.year() + 1)
        .and_then(|c| c.teams.iter().find(|t| t.slot == slot))
        .map_or_else(|| format!("team {slot}"), |t| t.name.clone())
}

/// `me`'s offer from team `slot`: an approach's during the season, else the season's end's.
fn negotiated_offer<'a>(league: &'a League, me: &str, slot: u8) -> Option<&'a Offer> {
    match league.phase {
        Phase::Racing => market::approaches_of(league, me)
            .into_iter()
            .find(|a| a.slot == slot)
            .and_then(|a| a.offer.as_ref()),
        _ => league.offers(me).into_iter().find(|o| o.slot == slot),
    }
}

fn years_text(years: u8) -> String {
    format!("{years} year{}", plural(years))
}

/// "$2.5M a season for 2 years, number one driver".
fn terms_text(t: &Terms) -> String {
    format!(
        "{} a season for {}{}",
        progress::money(u64::from(t.salary_k)),
        years_text(t.years),
        if t.number_one { ", number one driver" } else { "" }
    )
}

/// An offer's terms and where the talks are.
fn offer_text(o: &Offer) -> String {
    let base = match o.kind {
        OfferKind::Contract => "under contract: stay".to_owned(),
        OfferKind::Renew | OfferKind::Move => format!(
            "{}{}, {} a season + {} a point",
            if o.kind == OfferKind::Renew { "stay, " } else { "" },
            years_text(o.years),
            progress::money(u64::from(o.salary_k)),
            progress::money(u64::from(o.bonus_k))
        ),
    };
    let tries = if o.kind == OfferKind::Contract {
        String::new()
    } else {
        match o.attempts {
            0 => "; they won't talk any more".to_owned(),
            1 => "; 1 try left to negotiate".to_owned(),
            n => format!("; {n} tries left to negotiate"),
        }
    };
    let talks = match o.status {
        OfferStatus::Open => tries,
        OfferStatus::Countered => match &o.counter {
            Some(t) => format!("; they counter with {}{tries}", terms_text(t)),
            None => format!("; they countered{tries}"),
        },
        OfferStatus::Agreed => match &o.agreed {
            Some(t) => format!("; agreed: {}", terms_text(t)),
            None => "; terms agreed".to_owned(),
        },
        OfferStatus::Withdrawn => "; they walked away from the talks".to_owned(),
    };
    format!("{base}{talks}{}", if o.early { " (mid-season offer)" } else { "" })
}

/// " (+40 resource points, +1 recognition)" for what an answer does; empty when it does nothing.
fn effect_text(e: &Effect) -> String {
    let mut parts = Vec::new();
    if e.rp != 0 {
        parts.push(format!("{:+} resource points", e.rp));
    }
    for (value, name) in [
        (e.recognition, "recognition"),
        (e.experience, "experience"),
        (e.racecraft, "racecraft"),
        (e.awareness, "awareness"),
        (e.pace, "pace"),
        (e.focus, "focus"),
    ] {
        if value != 0.0 {
            parts.push(format!("{value:+} {name}"));
        }
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" ({})", parts.join(", "))
    }
}

/// "Ben", "Ben and Cleo", "Ben, Cleo and Dan".
fn names(list: &[String]) -> String {
    match list {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

fn player_name_of(league: &League, id: &str) -> String {
    league.player(id).map_or_else(|| id.to_owned(), |p| p.name.clone())
}

/// The open proposal, who agreed and who it waits for; nothing without one.
fn print_vote(league: &League, me: &str) {
    let Some(v) = &league.vote else { return };
    let verb = match v.proposal {
        Proposal::Simulate => "simulate",
        Proposal::Skip => "leave out",
        Proposal::Undo => "take back",
    };
    let race = league
        .season()
        .round(v.round)
        .map_or_else(String::new, |r| format!(", the {}", r.race));
    println!("\nopen proposal: {} asks to {verb} round {}{race}", v.by_name, v.round);
    let agreed: Vec<String> = v.yes.iter().map(|id| player_name_of(league, id)).collect();
    let waiting: Vec<String> = league.waiting_for().iter().map(|p| p.name.clone()).collect();
    println!("  agreed: {}", names(&agreed));
    if !waiting.is_empty() {
        println!("  waiting for: {}", names(&waiting));
    }
    if league.waiting_for().iter().any(|p| p.id == me) {
        println!("  agree with `f1mp mp career vote yes`, decline with `f1mp mp career vote no`");
    } else if v.by == me {
        println!(
            "  it happens once everyone agrees; withdraw it with `f1mp mp career vote no`, or (the owner) go ahead with `vote force`"
        );
    }
}

/// Who is ready for the next round.
fn print_ready(league: &League, me: &str) {
    if let Some(b) = league.briefing(me)
        && !b.ready.is_empty()
    {
        println!("ready for round {}: {}", b.round, names(&b.ready));
    }
}

fn print_undo_preview(p: &UndoPreview, me: &str, league: &League) {
    let mut what = Vec::new();
    if p.race_counted {
        what.push("the race".to_owned());
    }
    if p.sprint {
        what.push("the sprint".to_owned());
    }
    if p.quali {
        what.push("the qualifying".to_owned());
    }
    if !p.practice_of.is_empty() {
        what.push(format!("the practice of {}", names(&p.practice_of)));
    }
    if what.is_empty() {
        what.push("the round".to_owned());
    }
    println!("undo takes back round {}, the {}: {}", p.round, p.race, names(&what));
    let mine = league.player(me).map(|p| p.name.as_str());
    let others: Vec<String> = p
        .by_others
        .iter()
        .filter(|n| Some(n.as_str()) != mine)
        .cloned()
        .collect();
    if !others.is_empty() {
        println!("  sessions {} counted go too", names(&others));
    }
    if p.auto_after > 0 {
        println!(
            "  {} automatic round{} after it {} counted again",
            p.auto_after,
            plural(p.auto_after),
            if p.auto_after == 1 { "is" } else { "are" }
        );
    }
    if p.season_end {
        println!("  it reopens the finished season: the contracts for next season are cleared");
    }
    if league.needs_consent() {
        println!("  with friends in the career, everyone agrees first (`f1mp mp career vote`)");
    }
}

fn print_profile(league: &League, me: &str) {
    let Some(p) = league.player(me) else {
        println!("you're not a player of this career");
        return;
    };
    let nationality = p.nationality.as_deref().map_or_else(
        || "not set".to_owned(),
        |c| format!("{} ({c})", driver::nationality_name(c).unwrap_or(c)),
    );
    let archetype = p.archetype.map_or("not set", Archetype::name);
    println!(
        "{} #{}: nationality {nationality}, {archetype}, rated {:.1}",
        p.name, p.number, p.rating
    );
    if driver::has_driven(league, me) {
        println!("  your driver is set: a session of yours is counted");
    } else {
        println!(
            "  until your first session counts: `f1mp mp career profile --nationality DEU --archetype rookie|experienced|veteran`"
        );
    }
}

/// `me`'s approaches this season; `empty` says so when there are none.
fn print_approaches(league: &League, me: &str, empty: bool) {
    let list = market::approaches_of(league, me);
    if list.is_empty() {
        if empty {
            println!(
                "no team approached you this season; teams faster than yours may, 40%, 60% and 80% into the season"
            );
        }
        return;
    }
    println!("teams interested in you for {}:", league.year() + 1);
    for a in list {
        let state = match a.status {
            ApproachStatus::Interest => format!("wants to meet you (`f1mp mp career meet {}`)", a.slot),
            ApproachStatus::Met => match &a.offer {
                Some(o) if o.status == OfferStatus::Withdrawn => offer_text(o),
                Some(o) => format!(
                    "offers {} (`accept {s}`, `negotiate {s} --salary .. --years ..` or `decline {s}`)",
                    offer_text(o),
                    s = a.slot
                ),
                None => "you met them".to_owned(),
            },
            ApproachStatus::Accepted => "you signed with them for next season".to_owned(),
            ApproachStatus::Declined => "you turned them down".to_owned(),
            ApproachStatus::Expired => "their interest expired".to_owned(),
        };
        println!(
            "  {:>2}  {}: {state}{}",
            a.slot,
            next_team_name(league, a.slot),
            if a.leaked {
                " (word of the meeting got out)"
            } else {
                ""
            }
        );
    }
}

/// `me`'s mails, the latest ten (every one with `all`); unread ones marked with `*`.
fn print_inbox(league: &League, me: &str, all: bool) {
    let mails: Vec<&league::News> = league
        .news
        .iter()
        .filter(|n| n.kind != NewsKind::News && n.to.as_deref().is_none_or(|to| to == me))
        .collect();
    if mails.is_empty() {
        println!("no mails yet; departments, the media and your team write after the rounds");
        return;
    }
    let read = league.player(me).map_or(0, |p| p.mail_read);
    let skip = if all { 0 } else { mails.len().saturating_sub(10) };
    let mut open = 0;
    for n in &mails[skip..] {
        let kind = match n.kind {
            NewsKind::Mail => "mail",
            NewsKind::Event => "team",
            NewsKind::Media => "media",
            NewsKind::News => "news",
        };
        println!(
            "{}{:>4}  {} round {}  [{kind}]  {}",
            if n.id > read { "*" } else { " " },
            n.id,
            n.year,
            n.round,
            n.text
        );
        for (i, c) in n.choices.iter().enumerate() {
            let picked = n.answered == Some(i as u8);
            println!(
                "          {}{}. {}{}",
                if picked { "> " } else { "  " },
                i + 1,
                c.template,
                effect_text(&c.effect)
            );
        }
        if !n.choices.is_empty() && n.answered.is_none() {
            open += 1;
        }
    }
    if open > 0 {
        println!("answer with `f1mp mp career answer <mail> <choice>`");
    }
    if mails.iter().any(|n| n.id > read) {
        println!("* unread; mark them read with `f1mp mp career read`");
    }
}

/// The development trees of `me`'s team (one part with `only`).
fn print_tree(league: &League, me: &str, only: Option<Part>) {
    let season = league.season();
    let Some(entry) = season.player_entry(me) else {
        println!("you have no seat this season");
        return;
    };
    let slot = entry.slot;
    let team = season.team(slot).map_or("your team", |t| t.name.as_str());
    let rp = league.player(me).map_or(0, |p| p.rp);
    println!(
        "{team}'s development in {}: {rp} resource points to spend; the team spent {} of {} this season (cost cap)",
        season.year,
        rnd::spent(season, slot),
        rnd::COST_CAP
    );
    let round = season.current_round();
    for part in Part::ALL.into_iter().filter(|p| only.is_none_or(|o| o == *p)) {
        let available = rnd::available(season, slot, part);
        let next = rnd::next_main(season, slot, part);
        let ordered: Vec<&progress::Upgrade> = season
            .upgrades
            .iter()
            .filter(|u| u.slot == slot && u.part == part)
            .collect();
        let legacy = ordered.iter().filter(|u| u.node.is_none() && !u.failed).count();
        println!(
            "\n{} ({}):",
            part.name(),
            if part == Part::Durability {
                "fewer breakdowns"
            } else {
                "lap time"
            }
        );
        for branch in [0, 1] {
            println!(
                "  {}",
                if branch == 0 {
                    "main branch (steady):"
                } else {
                    "alternative branch (bigger gains, riskier):"
                }
            );
            for node in rnd::nodes(part).iter().filter(|n| n.branch == branch) {
                let gain = if part == Part::Durability {
                    format!("{:.0}% fewer breakdowns", node.gain * 100.0)
                } else {
                    format!("{:.2}% a lap", node.gain)
                };
                let mine: Vec<&&progress::Upgrade> = ordered.iter().filter(|u| u.node == Some(node.id)).collect();
                let state = if let Some(u) = mine.iter().find(|u| !u.failed) {
                    if u.ready <= round {
                        "on the car".to_owned()
                    } else {
                        format!("ordered, on the car from round {}", u.ready)
                    }
                } else if node.branch == 0 && usize::from(node.id) <= legacy {
                    "developed (before the tree)".to_owned()
                } else if available.iter().any(|a| a.id == node.id) {
                    format!(
                        "{}{} points, {} rounds to build, {:.0}% chance it fails{}",
                        if mine.is_empty() {
                            ""
                        } else {
                            "failed once; again: "
                        },
                        node.cost,
                        node.rounds,
                        node.fail * 100.0,
                        if next.is_some_and(|n| n.id == node.id) {
                            "  <- next of `develop`"
                        } else {
                            ""
                        }
                    )
                } else {
                    match node.requires.and_then(|r| rnd::node(part, r)) {
                        Some(r) => format!("needs {} first", r.name),
                        None => "locked".to_owned(),
                    }
                };
                println!("    {:>2}  {:<28} {gain:<22} {state}", node.id, node.name);
            }
        }
    }
    println!(
        "\norder with `f1mp mp career develop <part>` (the next main node) or `develop <part> --node <number>`"
    );
}

fn on_off(on: bool) -> &'static str {
    if on { "on" } else { "off" }
}

fn print_options(o: &Options) {
    let rules = weekend::describe(&o.rules);
    if rules.is_empty() {
        println!("weekend rules: none agreed (the game's settings aren't checked)");
    } else {
        println!("weekend rules everyone drives by (the game's settings are checked against them):");
        for (label, value) in rules {
            println!("  {label:<22} {value}");
        }
    }
    println!("in the game:");
    println!("  your driver's name and number   {}", on_off(o.game_identity));
    println!("  the league's car pace           {}", on_off(o.game_performance));
    println!("  per-part feel (experimental)    {}", on_off(o.game_parts));
    println!(
        "AI drivers: {}",
        match o.lineups {
            LineupMode::Historic => "each year's real grid (`--lineups dynamic` lets them move between teams)",
            LineupMode::Dynamic => {
                "they move between teams, retire and make way for rookies at each season's end; the game shows them in renamed seats with the face of the seat's real driver"
            }
        }
    );
    println!("the owner changes them, e.g. `f1mp mp career options --ai 90 --quali one-shot --game-identity on`");
}

/// What an objective asks for.
fn objective_text(o: &Objective) -> String {
    match o.kind {
        ObjectiveKind::PracticeGrade => format!("a practice grade of {} or better", o.target),
        ObjectiveKind::QualiPosition => format!("qualify P{} or better", o.target),
        ObjectiveKind::RacePosition => format!("finish the race P{} or better", o.target),
        ObjectiveKind::BeatTeamMateQuali => "out-qualify your team-mate".to_owned(),
        ObjectiveKind::BeatTeamMateRace => "finish ahead of your team-mate".to_owned(),
        ObjectiveKind::BeatRival => "finish ahead of your rival".to_owned(),
        ObjectiveKind::CleanRace => "finish the race without a penalty".to_owned(),
    }
}

/// The game sync of `target` on this PC for `profile`.
fn sync(store: &Store, profile: &str, mut target: Target, action: SyncAction) -> Result<()> {
    if action == SyncAction::Restore {
        careersync::restore(store, profile).map_err(|e| anyhow!(e))?;
        println!(
            "took the league career's game sync out of the {profile} profile; the next launch builds without it"
        );
        return Ok(());
    }
    let league = target.load()?.context("the career hasn't started yet")?;
    let (id, name) = (target.id().to_owned(), target.name().to_owned());
    let me = target.me()?;
    let o = &league.options;
    println!("game sync of \"{name}\" in the {profile} profile:");
    println!(
        "  your driver's name and number {}, the league's car pace {}{}",
        on_off(o.game_identity),
        on_off(o.game_performance),
        if o.game_parts {
            " (with the per-part feel)"
        } else {
            ""
        }
    );
    let ctx = careersync::context(store, league.year());
    let overlay = match &ctx {
        Ok(c) => gamesync::overlay(&league, &id, &name, &c.get()),
        Err(_) => None,
    };
    let wanted = match (&ctx, &overlay) {
        (Ok(c), Some(ov)) => gamesync::files(ov, &c.get())
            .iter()
            .find(|(path, _)| path == gamesync::LEAGUE_FILE)
            .and_then(|(_, bytes)| careersync::Prepared::parse(bytes))
            .map(|p| p.hash),
        _ => None,
    };
    if action == SyncAction::Prepare {
        if let Some(why) = no_overlay_reason(&league) {
            bail!("nothing to prepare: {why}");
        }
        if let Err(e) = &ctx {
            bail!("can't prepare the game: {e}");
        }
        let Some(overlay) = &overlay else {
            bail!("nothing to prepare for the next round");
        };
        let state = careersync::prepare(store, profile, overlay).map_err(|e| anyhow!(e))?;
        print_sync_state(&state, &id, wanted.as_deref());
        if !(state.game_running && state.prepared.as_ref().map(|p| &p.hash) != wanted.as_ref()) {
            println!(
                "  the next launch builds it (Play in the launcher, or `f1mp run`); with friends, everyone prepares the same round and gets the same hash"
            );
        }
    } else {
        print_sync_state(&careersync::state(store, profile), &id, wanted.as_deref());
        match (no_overlay_reason(&league), &ctx) {
            (Some(why), _) => println!("  nothing to prepare: {why}"),
            (None, Err(e)) => println!("  can't prepare: {e}"),
            (None, Ok(_)) => {}
        }
    }
    match &overlay {
        Some(ov) => print_overlay(ov),
        None => {
            if let Some(g) = gamesync::identity_of(&league, &me) {
                println!("the game shows you as {}", identity_text(&g));
            }
        }
    }
    Ok(())
}

/// Why the career has no overlay for its next round, whatever this PC has installed; `None` when
/// it has one (given Classic Seasons).
fn no_overlay_reason(league: &League) -> Option<&'static str> {
    let o = &league.options;
    if league.phase != Phase::Racing {
        Some("no season is running")
    } else if !o.game_identity && !o.game_performance && o.lineups != LineupMode::Dynamic {
        Some(
            "every game option is off; the owner turns one on with `f1mp mp career options --game-identity on` or `--game-performance on`",
        )
    } else if league.season().next_round().is_none() {
        Some("every round of the season is counted")
    } else {
        None
    }
}

/// What this PC has prepared, against the overlay the career needs now (`wanted`, its hash).
fn print_sync_state(state: &careersync::SyncState, career: &str, wanted: Option<&str>) {
    match &state.prepared {
        None => println!("  this PC has nothing prepared (`f1mp mp career sync prepare`)"),
        Some(p) if p.career != career => println!(
            "  this PC has another career prepared ({}); `f1mp mp career sync prepare` replaces it",
            p.career
        ),
        Some(p) => println!(
            "  prepared for round {} of {} (hash {}){}",
            p.round,
            p.year,
            p.hash,
            match wanted {
                Some(w) if w == p.hash => ", up to date",
                Some(_) => ", out of date: run `f1mp mp career sync prepare`",
                None => "",
            }
        ),
    }
    if state.prepared.is_some() {
        println!(
            "  {} in the profile",
            if state.enabled { "on" } else { "off" }
        );
    }
    if state.game_running {
        println!("  the game is running: a new round's sync waits until it closes; prepare again then");
    }
}

/// "Anna Berg (BER) #7, nationality German, in Max Verstappen's seat".
fn identity_text(g: &league::GameIdentity) -> String {
    format!(
        "{} ({}) #{}{}{}, in {}'s seat",
        g.full,
        g.code,
        g.number,
        if g.number_clash {
            " (your own number is taken that year)"
        } else {
            ""
        },
        g.nationality
            .as_deref()
            .map_or_else(String::new, |n| format!(", {n}")),
        g.replaces
    )
}

fn print_overlay(ov: &gamesync::Overlay) {
    println!("what the game shows for round {} of {}:", ov.round, ov.year);
    for g in &ov.identities {
        println!("  {}", identity_text(g));
    }
    for c in &ov.cars {
        println!(
            "  {:<24} {}",
            c.team_key,
            if c.stock {
                "stock car".to_owned()
            } else {
                format!("{:.1} kg", c.mass_kg)
            }
        );
    }
    for r in &ov.renames {
        let seat = league::aimarket::seat_driver_name(ov.year, r.slot, &r.seat_driver).unwrap_or(&r.seat_driver);
        println!(
            "  {} ({}) #{} in {seat}'s seat (with {seat}'s face)",
            gamesync::market::full_name(r),
            r.code,
            r.number
        );
    }
}

fn plural(n: u8) -> &'static str {
    if n == 1 { "" } else { "s" }
}

fn print_news_since(league: &League, last: usize) {
    for n in latest_news(league, last) {
        println!("{}  {}", n.year, n.text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stale_answer_says_to_look_again() {
        let stale = ApiError {
            status: 409,
            reason: "stale".into(),
            message: "the career changed meanwhile; look at it again and try again".into(),
            lobby: None,
            retry_after: None,
            text: None,
            version: Some(7),
        };
        assert!(server_error(&stale).to_string().contains("career show"));
        let other = ApiError {
            reason: "owner".into(),
            message: "only the career's owner deletes it".into(),
            ..stale
        };
        assert_eq!(server_error(&other).to_string(), "only the career's owner deletes it");
    }

    #[derive(clap::Parser)]
    struct Cli {
        #[command(subcommand)]
        career: CareerCommand,
    }

    fn parse(args: &[&str]) -> CareerCommand {
        <Cli as clap::Parser>::try_parse_from(std::iter::once("career").chain(args.iter().copied()))
            .unwrap_or_else(|e| panic!("{args:?}: {e}"))
            .career
    }

    fn fails(args: &[&str]) -> bool {
        <Cli as clap::Parser>::try_parse_from(std::iter::once("career").chain(args.iter().copied())).is_err()
    }

    #[test]
    fn lineups_parse() {
        let CareerCommand::Options(args) = parse(&["options", "--lineups", "dynamic"]) else {
            panic!("options --lineups");
        };
        assert!(args.changes());
        let o = args.apply(Options::default());
        assert_eq!(o.lineups, LineupMode::Dynamic);
        assert_eq!(o.rules, WeekendRules::default(), "nothing else changes");
        let CareerCommand::Options(args) = parse(&["options", "--lineups", "historic"]) else {
            panic!("options --lineups historic");
        };
        assert_eq!(args.apply(o).lineups, LineupMode::Historic);
        assert!(fails(&["options", "--lineups", "random"]));
    }

    #[test]
    fn the_command_tree_is_valid() {
        <Cli as clap::CommandFactory>::command().debug_assert();
    }

    #[test]
    fn ready_and_vote_parse() {
        assert!(matches!(parse(&["ready"]), CareerCommand::Ready { off: false, career: None }));
        assert!(matches!(
            parse(&["ready", "--off", "--career", "c-1"]),
            CareerCommand::Ready { off: true, career: Some(_) }
        ));
        assert!(matches!(parse(&["vote"]), CareerCommand::Vote { choice: None, .. }));
        for (word, choice) in [
            ("yes", VoteChoice::Yes),
            ("no", VoteChoice::No),
            ("force", VoteChoice::Force),
        ] {
            let CareerCommand::Vote { choice: Some(c), .. } = parse(&["vote", word]) else {
                panic!("vote {word}");
            };
            assert_eq!(c, choice);
        }
        assert!(fails(&["vote", "maybe"]));
    }

    #[test]
    fn undo_preview_parses() {
        assert!(matches!(parse(&["undo"]), CareerCommand::Undo { preview: false, .. }));
        assert!(matches!(parse(&["undo", "--preview"]), CareerCommand::Undo { preview: true, .. }));
    }

    #[test]
    fn profile_parses() {
        let CareerCommand::Profile {
            nationality,
            archetype,
            nationalities,
            ..
        } = parse(&["profile", "--nationality", "deu", "--archetype", "rookie"])
        else {
            panic!("profile");
        };
        assert_eq!(nationality.as_deref().map(nationality_arg).as_deref(), Some("DEU"));
        assert_eq!(archetype.map(Archetype::from), Some(Archetype::Rookie));
        assert!(!nationalities);
        assert!(matches!(
            parse(&["profile", "--archetype", "veteran"]),
            CareerCommand::Profile { archetype: Some(ArchetypeArg::Veteran), nationality: None, .. }
        ));
        assert!(matches!(
            parse(&["profile", "--nationalities"]),
            CareerCommand::Profile { nationalities: true, .. }
        ));
        assert!(fails(&["profile", "--archetype", "legend"]));
        assert_eq!(nationality_arg(" none "), "");
    }

    #[test]
    fn market_commands_parse() {
        assert!(matches!(parse(&["approaches"]), CareerCommand::Approaches { career: None }));
        assert!(matches!(parse(&["meet", "3"]), CareerCommand::Meet { team: 3, .. }));
        let CareerCommand::Accept { team, replaces, .. } = parse(&["accept", "4", "--replaces", "ricciardo"]) else {
            panic!("accept");
        };
        assert_eq!((team, replaces.as_deref()), (4, Some("ricciardo")));
        assert!(matches!(parse(&["decline", "5"]), CareerCommand::Decline { team: 5, .. }));
        let CareerCommand::Negotiate {
            team,
            salary,
            years,
            number_one,
            ..
        } = parse(&["negotiate", "2", "--salary", "2500", "--years", "2", "--number-one"])
        else {
            panic!("negotiate");
        };
        assert_eq!((team, salary, years, number_one), (2, 2500, 2, true));
        assert!(matches!(
            parse(&["negotiate", "2", "--salary", "900", "--years", "1"]),
            CareerCommand::Negotiate { number_one: false, .. }
        ));
        assert!(fails(&["negotiate", "2", "--years", "2"]), "the salary is required");
        assert!(fails(&["meet"]), "the team is required");
    }

    #[test]
    fn inbox_commands_parse() {
        assert!(matches!(parse(&["inbox"]), CareerCommand::Inbox { all: false, .. }));
        assert!(matches!(parse(&["inbox", "--all"]), CareerCommand::Inbox { all: true, .. }));
        assert!(matches!(parse(&["answer", "12", "2"]), CareerCommand::Answer { id: 12, choice: 2, .. }));
        assert!(matches!(parse(&["read"]), CareerCommand::Read { upto: None, .. }));
        assert!(matches!(parse(&["read", "7"]), CareerCommand::Read { upto: Some(7), .. }));
        assert_eq!(choice_index(1).unwrap(), 0);
        assert!(choice_index(0).is_err());
    }

    #[test]
    fn develop_and_tree_parse() {
        let CareerCommand::Develop { part, node, .. } = parse(&["develop", "aero", "--node", "6"]) else {
            panic!("develop");
        };
        assert_eq!((part.as_str(), node), ("aero", Some(6)));
        assert!(matches!(parse(&["develop", "chassis"]), CareerCommand::Develop { node: None, .. }));
        assert!(matches!(parse(&["tree"]), CareerCommand::Tree { part: None, .. }));
        assert!(matches!(parse(&["tree", "powertrain"]), CareerCommand::Tree { part: Some(_), .. }));
        assert_eq!(parse_part("durability").unwrap(), Part::Durability);
        assert!(parse_part("wings").is_err());
    }

    #[test]
    fn options_set_and_unset_the_rules() {
        let CareerCommand::Options(args) = parse(&[
            "options",
            "--ai",
            "90",
            "--quali",
            "one-shot",
            "--practice",
            "one-session",
            "--damage",
            "simulation",
            "--safety-car",
            "reduced",
            "--parc-ferme",
            "on",
            "--weather",
            "light-rain",
            "--rule-set",
            "race",
            "--game-identity",
            "on",
        ]) else {
            panic!("options");
        };
        assert!(args.changes());
        let o = args.apply(Options::default());
        let r = &o.rules;
        assert_eq!(r.ai_difficulty, Some(90));
        assert_eq!(r.quali, Some(QualiFormat::OneShot));
        assert_eq!(r.practice, Some(PracticeFormat::OneSession));
        assert_eq!((r.damage, r.safety_car, r.weather, r.rule_set), (Some(3), Some(1), Some(3), Some(1)));
        assert_eq!((r.parc_ferme, r.equal_performance), (Some(true), None));
        assert!(o.game_identity && !o.game_performance && !o.game_parts);

        let CareerCommand::Options(args) = parse(&["options", "--unset", "ai,weather", "--game-identity", "off"]) else {
            panic!("options --unset");
        };
        let o = args.apply(o);
        assert_eq!((o.rules.ai_difficulty, o.rules.weather), (None, None));
        assert_eq!(o.rules.quali, Some(QualiFormat::OneShot), "kept");
        assert!(!o.game_identity);
        // Unset first, then set: `--unset all --ai 80` agrees on the AI difficulty only.
        let CareerCommand::Options(args) = parse(&["options", "--unset", "all", "--ai", "80"]) else {
            panic!("options --unset all");
        };
        let o = args.apply(o);
        assert_eq!(
            o.rules,
            WeekendRules {
                ai_difficulty: Some(80),
                ..WeekendRules::default()
            }
        );
        let CareerCommand::Options(args) = parse(&["options"]) else {
            panic!("options");
        };
        assert!(!args.changes());
        assert!(fails(&["options", "--quali", "two-shot"]));
        // The names follow the game's values.
        assert_eq!(RuleSetArg::RivalDuel.value(), 11);
        assert_eq!(WeatherArg::Storm as u8, 5);
        assert_eq!(SafetyCarArg::Increased as u8, 3);
    }

    #[test]
    fn sync_parses() {
        assert!(matches!(parse(&["sync"]), CareerCommand::Sync { action: SyncAction::Show, .. }));
        assert!(matches!(parse(&["sync", "prepare"]), CareerCommand::Sync { action: SyncAction::Prepare, .. }));
        assert!(matches!(parse(&["sync", "restore"]), CareerCommand::Sync { action: SyncAction::Restore, .. }));
        assert!(fails(&["sync", "build"]));
    }

    #[test]
    fn new_refusals_say_what_to_do() {
        for reason in [
            "vote_open",
            "not_voter",
            "no_vote",
            "left",
            "not_yet",
            "not_next",
            "duplicate",
            "ask_kind",
            "cost_cap",
            "node_locked",
            "no_approach",
            "walked_away",
            "answered",
            "bad_profile",
            "bad_rules",
        ] {
            let e = ApiError {
                status: 409,
                reason: reason.into(),
                message: "the server's words".into(),
                lobby: None,
                retry_after: None,
                text: None,
                version: None,
            };
            let text = server_error(&e).to_string();
            assert!(text.starts_with("the server's words\n  ("), "{reason}: {text}");
            assert_eq!(refusal(reason, "solo words").to_string().lines().count(), 2, "{reason}");
        }
        assert!(refusal("vote_open", "x").to_string().contains("vote yes"));
        assert!(refusal("bad_profile", "x").to_string().contains("--nationalities"));
        assert_eq!(refusal("wrong_track", "another track").to_string(), "another track");
        let limited = ApiError {
            status: 429,
            reason: "rate_limited".into(),
            message: "too many tries".into(),
            lobby: None,
            retry_after: Some(30),
            text: None,
            version: None,
        };
        assert!(server_error(&limited).to_string().contains("30 seconds"));
    }

    #[test]
    fn offers_and_answers_read_well() {
        let mut o = Offer {
            player: "a".into(),
            slot: 3,
            kind: OfferKind::Move,
            years: 2,
            salary_k: 2000,
            bonus_k: 10,
            ..Offer::fresh()
        };
        assert!(offer_text(&o).contains("3 tries left"), "{}", offer_text(&o));
        o.status = OfferStatus::Countered;
        o.attempts = 1;
        o.counter = Some(Terms {
            salary_k: 2500,
            years: 1,
            number_one: true,
        });
        let text = offer_text(&o);
        assert!(text.contains("they counter with $2.5M a season for 1 year, number one driver"), "{text}");
        assert!(text.contains("1 try left"), "{text}");
        o.status = OfferStatus::Withdrawn;
        assert!(offer_text(&o).contains("walked away"));
        let e = Effect {
            rp: 40,
            recognition: -1.0,
            ..Effect::default()
        };
        assert_eq!(effect_text(&e), " (+40 resource points, -1 recognition)");
        assert_eq!(effect_text(&Effect::default()), "");
        assert_eq!(names(&["Anna".into(), "Ben".into(), "Cleo".into()]), "Anna, Ben and Cleo");
    }

    /// A solo career on a scratch data folder, driven through the commands as a player would.
    #[test]
    fn a_solo_career_runs_the_new_commands() {
        let root = std::env::temp_dir().join(format!("swofty-cli-career-{}", std::process::id()));
        std::fs::remove_dir_all(&root).ok();
        let store = Store::new(root.join("data"));
        let run = |args: &[&str]| command(&store, "default", parse(args));
        run(&["new", "Test", "--solo", "--player", "Anna", "--from", "2016", "--to", "2017"]).unwrap();
        let id = solo::list(&store.root)[0].id.clone();
        let slot = solo::load(&store.root, &id).unwrap().league.season().teams[0].slot;
        run(&["seat", &slot.to_string()]).unwrap();
        run(&["profile", "--nationality", "DEU", "--archetype", "experienced"]).unwrap();
        let err = run(&["profile", "--nationality", "XXX"]).unwrap_err().to_string();
        assert!(err.contains("--nationalities"), "{err}");
        run(&["begin"]).unwrap();
        let league = solo::load(&store.root, &id).unwrap().league;
        let p = league.player(solo::ME).unwrap();
        assert_eq!((p.nationality.as_deref(), p.archetype), (Some("DEU"), Some(Archetype::Experienced)));

        run(&["options", "--ai", "95", "--quali", "short"]).unwrap();
        let err = run(&["options", "--ai", "200"]).unwrap_err().to_string();
        assert!(err.contains("options --help"), "{err}");
        let league = solo::load(&store.root, &id).unwrap().league;
        assert_eq!(league.options.rules.ai_difficulty, Some(95));
        assert_eq!(league.options.rules.quali, Some(QualiFormat::Short));

        run(&["tree"]).unwrap();
        run(&["tree", "aero"]).unwrap();
        assert!(run(&["undo", "--preview"]).is_err(), "nothing to take back yet");
        run(&["ready"]).unwrap();
        run(&["vote"]).unwrap();
        assert!(run(&["vote", "yes"]).unwrap_err().to_string().contains("nobody proposed"));
        run(&["simulate"]).unwrap();
        run(&["undo", "--preview"]).unwrap();
        assert!(solo::load(&store.root, &id).unwrap().league.season().rounds[0].done(), "only a preview");
        run(&["approaches"]).unwrap();
        run(&["inbox", "--all"]).unwrap();
        run(&["read"]).unwrap();
        let league = solo::load(&store.root, &id).unwrap().league;
        assert!(inbox::unread(&league, solo::ME).is_empty());
        assert!(run(&["answer", "999", "1"]).unwrap_err().to_string().contains("career inbox"));
        let err = run(&["develop", "aero", "--node", "2"]).unwrap_err().to_string();
        assert!(err.contains("tree"), "{err}");
        run(&["driver"]).unwrap();
        run(&["show", "--all"]).unwrap();
        // No Classic Seasons on this data folder and every game option off: nothing to prepare.
        run(&["sync"]).unwrap();
        let err = run(&["sync", "prepare"]).unwrap_err().to_string();
        assert!(err.contains("--game-identity on"), "{err}");
        run(&["sync", "restore"]).unwrap();
        run(&["undo"]).unwrap();
        assert!(!solo::load(&store.root, &id).unwrap().league.season().rounds[0].done());
        std::fs::remove_dir_all(&root).ok();
    }
}