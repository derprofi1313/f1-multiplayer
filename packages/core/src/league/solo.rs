//! Solo careers: the league career driven alone, without a host server.
//!
//! A solo career is the same [`League`] document a host server keeps, stored in the launcher's
//! data folder (`<data>/league/solo/<id>.json`) and changed with the same [`Action`]s through
//! [`apply`](super::apply). The player is always the career's owner. Each change keeps the
//! version before it (the last [`VERSIONS_KEPT`]) next to the career, like a host server does.

use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{Action, Actor, League, LeagueError, Setup};

/// Where solo careers live, under the launcher's data folder.
pub const DIR: &str = "league/solo";
/// The player's id in a solo career.
pub const ME: &str = "solo";
/// Earlier versions kept of each solo career.
const VERSIONS_KEPT: usize = 10;
const ID_PREFIX: &str = "solo-";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoloCareer {
    pub id: String,
    pub name: String,
    /// Unix seconds.
    pub created: u64,
    /// Unix seconds of the last change.
    #[serde(default)]
    pub updated: u64,
    /// Counts the changes, like a host server's save version.
    #[serde(default)]
    pub version: u64,
    pub league: League,
}

#[derive(Debug)]
pub enum SoloError {
    League(LeagueError),
    Io(io::Error),
    NotFound(String),
}

impl std::fmt::Display for SoloError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SoloError::League(e) => write!(f, "{e}"),
            SoloError::Io(e) => write!(f, "saving the career: {e}"),
            SoloError::NotFound(id) => write!(f, "there is no solo career {id}"),
        }
    }
}

impl std::error::Error for SoloError {}

impl From<LeagueError> for SoloError {
    fn from(e: LeagueError) -> Self {
        SoloError::League(e)
    }
}

impl From<io::Error> for SoloError {
    fn from(e: io::Error) -> Self {
        SoloError::Io(e)
    }
}

impl SoloError {
    /// The league's reason code (`wrong_track`, `phase`, ...), or `io`/`not_found`.
    pub fn reason(&self) -> &'static str {
        match self {
            SoloError::League(e) => e.reason,
            SoloError::Io(_) => "io",
            SoloError::NotFound(_) => "not_found",
        }
    }
}

/// Whether `id` names a solo career (and not one on a host server).
pub fn is_solo(id: &str) -> bool {
    id.starts_with(ID_PREFIX)
}

/// The player of a solo career, named `name`.
pub fn actor(name: &str) -> Actor {
    Actor {
        id: ME.to_owned(),
        name: name.to_owned(),
        owner: true,
    }
}

pub fn dir(root: &Path) -> PathBuf {
    root.join(DIR)
}

fn path(root: &Path, id: &str) -> PathBuf {
    dir(root).join(format!("{id}.json"))
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn fresh_seed() -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as u64);
    // splitmix64, so ids made in the same second differ in every digit.
    let mut z = nanos ^ (u64::from(std::process::id()) << 32);
    z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// Every solo career, the most recently changed first. Files that don't read are left out.
pub fn list(root: &Path) -> Vec<SoloCareer> {
    let Ok(entries) = std::fs::read_dir(dir(root)) else {
        return Vec::new();
    };
    let mut all: Vec<SoloCareer> = entries
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .filter_map(|e| serde_json::from_slice(&std::fs::read(e.path()).ok()?).ok())
        .collect();
    all.sort_by(|a, b| b.updated.cmp(&a.updated).then(b.created.cmp(&a.created)));
    all
}

pub fn load(root: &Path, id: &str) -> Result<SoloCareer, SoloError> {
    let bytes = match std::fs::read(path(root, id)) {
        Ok(b) => b,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Err(SoloError::NotFound(id.to_owned())),
        Err(e) => return Err(e.into()),
    };
    serde_json::from_slice(&bytes).map_err(|e| SoloError::Io(io::Error::new(io::ErrorKind::InvalidData, e)))
}

fn write(root: &Path, career: &SoloCareer) -> io::Result<()> {
    let file = path(root, &career.id);
    std::fs::create_dir_all(dir(root))?;
    // The version this one replaces goes to the history first.
    if file.is_file() {
        let history = dir(root).join("history").join(&career.id);
        std::fs::create_dir_all(&history)?;
        std::fs::copy(
            &file,
            history.join(format!("{:06}.json", career.version.saturating_sub(1))),
        )?;
        let mut old: Vec<PathBuf> = std::fs::read_dir(&history)?.flatten().map(|e| e.path()).collect();
        old.sort();
        while old.len() > VERSIONS_KEPT {
            let _ = std::fs::remove_file(old.remove(0));
        }
    }
    let tmp = file.with_extension("tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(career).unwrap_or_default())?;
    std::fs::rename(&tmp, &file)
}

/// Start a solo career named `name`, driven by `player`.
pub fn create(
    root: &Path,
    name: &str,
    player: &str,
    setup: Setup,
    number: Option<u8>,
) -> Result<SoloCareer, SoloError> {
    let seed = fresh_seed();
    let league = League::start(setup, &actor(player), number, seed)?;
    let now = now_secs();
    let career = SoloCareer {
        id: format!("{ID_PREFIX}{:06x}", seed & 0xff_ffff),
        name: name.to_owned(),
        created: now,
        updated: now,
        version: 1,
        league,
    };
    write(root, &career)?;
    Ok(career)
}

/// Apply `action` to the solo career `id` and save it. Returns the career and the warnings.
pub fn act(root: &Path, id: &str, action: Action) -> Result<(SoloCareer, Vec<String>), SoloError> {
    let mut career = load(root, id)?;
    let name = career
        .league
        .player(ME)
        .map_or_else(|| "You".to_owned(), |p| p.name.clone());
    let before = career.league.clone();
    let warnings = super::apply(&mut career.league, &actor(&name), action)?;
    if career.league != before {
        career.version += 1;
        career.updated = now_secs();
        write(root, &career)?;
    }
    Ok((career, warnings))
}

/// Rename a solo career.
pub fn rename(root: &Path, id: &str, name: &str) -> Result<SoloCareer, SoloError> {
    let mut career = load(root, id)?;
    career.name = name.to_owned();
    career.version += 1;
    career.updated = now_secs();
    write(root, &career)?;
    Ok(career)
}

/// Delete a solo career and its history.
pub fn delete(root: &Path, id: &str) -> Result<(), SoloError> {
    match std::fs::remove_file(path(root, id)) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Err(SoloError::NotFound(id.to_owned())),
        Err(e) => return Err(e.into()),
    }
    let _ = std::fs::remove_dir_all(dir(root).join("history").join(id));
    Ok(())
}
