//! The `f1mp setup` commands and the launch gate for the one-time setup.
//!
//! See `swofty_core::setup`: the game folder is put back, the record of
//! original game files starts over, and the player verifies the game files
//! before the first modded session.

use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::Subcommand;
use swofty_core::install::Source;
use swofty_core::setup;
use swofty_core::store::Store;

#[derive(Subcommand)]
pub enum SetupCommand {
    #[command(about = "Show whether the one-time setup before modded play is done")]
    Status,
    #[command(
        about = "Put the game folder back and start the record of original game files over; verify the game files next"
    )]
    Reset {
        #[arg(long, help = "The game folder holds mods F1 Multiplayer didn't add")]
        other_mods: bool,
        #[arg(long, help = crate::GAME_HELP)]
        game: Option<PathBuf>,
    },
    #[command(about = "Finish the setup once the game files are verified")]
    Done,
}

pub fn command(store: &Store, cmd: SetupCommand) -> Result<()> {
    match cmd {
        SetupCommand::Status => {
            match setup::read(&store.root) {
                Some(s) if setup::done(&store.root) => {
                    println!("setup: done at unix {}", s.done_at.unwrap_or_default())
                }
                Some(s) if s.version >= setup::VERSION => println!(
                    "setup: the game folder was put back at unix {}; verify the game files, then run `f1mp setup done`",
                    s.reset_at
                ),
                _ => println!("setup: not done; run `f1mp setup reset`"),
            }
            Ok(())
        }
        SetupCommand::Reset { other_mods, game } => {
            let g = crate::build::game(store, game)?;
            let reset = setup::reset(&store.root, &g, other_mods)?;
            if let Some(why) = &reset.unrestored {
                println!("not put back from the old record: {why}");
                println!("the old record and any unfinished changes were set aside");
            }
            let done = reset.restored;
            for n in &done.notes {
                println!("restored: {n}");
            }
            for p in &done.set_aside {
                println!("set aside: {}", p.display());
            }
            for d in &done.left {
                println!("not restored: {d}");
            }
            println!("the record of original game files starts over");
            if !other_mods && reset.unrestored.is_none() && setup::store_checks(&g.dir) {
                println!(
                    "the launch checks every game file against Steam's own list, so there's nothing to verify"
                );
            } else {
                println!("{}", verify_step(Source::detect(&g.dir)));
            }
            println!("then run `f1mp setup done`");
            Ok(())
        }
        SetupCommand::Done => {
            setup::finish(&store.root)?;
            println!("setup done: modded play can start");
            Ok(())
        }
    }
}

fn verify_step(source: Source) -> &'static str {
    match source {
        Source::Steam => {
            "verify the game files: Steam > right-click F1 25 > Properties > Installed Files > Verify integrity of game files"
        }
        Source::EaApp => "repair the game: EA app > Library > F1 25 > Manage > Repair",
        Source::Unknown => {
            "make sure the game folder holds no mods: this copy isn't from a store that can verify it"
        }
    }
}

/// Modded sessions wait for the setup.
pub fn gate(store: &Store) -> Result<()> {
    if !setup::done(&store.root) {
        bail!("{}", setup::NEEDED);
    }
    Ok(())
}
