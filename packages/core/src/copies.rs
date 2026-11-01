//! Every copy of F1 25 on this computer (Steam's, the EA app's, folders the
//! player added) and the one Play uses.

use std::path::{Path, PathBuf};

use crate::GAME_EXE;
use crate::install::Source;
use crate::session::Config;
use crate::steam::GameInstall;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Found {
    Steam,
    EaApp,
    Added,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameCopy {
    pub dir: PathBuf,
    pub source: Source,
    pub found: Found,
    pub steam: Option<GameInstall>,
}

impl GameCopy {
    pub fn version(&self) -> Option<String> {
        crate::exe_version::read(&self.dir.join(GAME_EXE))
    }

    pub fn build_id(&self) -> Option<u64> {
        self.steam.as_ref().and_then(|g| g.build_id)
    }
}

pub fn store_name(source: Source) -> &'static str {
    match source {
        Source::Steam => "Steam",
        Source::EaApp => "EA app",
        Source::Unknown => "Other",
    }
}

/// Where the player moves a copy out of Program Files, by the store it came
/// from.
pub fn move_hint(source: Source) -> &'static str {
    match source {
        Source::EaApp => " with EA app > F1 25 > Manage > Move, not File Explorer",
        Source::Steam => " in Steam (Properties > Installed Files > Move install folder)",
        Source::Unknown => "",
    }
}

pub fn same_dir(a: &Path, b: &Path) -> bool {
    a == b
        || match (a.canonicalize(), b.canonicalize()) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
}

/// The copies in the order the switcher lists them: Steam's, the EA app's,
/// then added folders, each folder once and only with the game in it.
pub fn gather(steam: Option<GameInstall>, ea: Vec<PathBuf>, added: &[PathBuf]) -> Vec<GameCopy> {
    let mut out: Vec<GameCopy> = Vec::new();
    let mut push = |dir: PathBuf, found: Found, steam: Option<GameInstall>| {
        if dir.join(GAME_EXE).is_file() && !out.iter().any(|c| same_dir(&c.dir, &dir)) {
            out.push(GameCopy {
                source: match found {
                    Found::Steam => Source::Steam,
                    _ => Source::detect(&dir),
                },
                dir,
                found,
                steam,
            });
        }
    };
    if let Some(g) = steam {
        push(g.dir.clone(), Found::Steam, Some(g));
    }
    for dir in ea {
        push(dir, Found::EaApp, None);
    }
    for dir in added {
        push(dir.clone(), Found::Added, None);
    }
    out
}

pub fn discover(config: &Config) -> Vec<GameCopy> {
    gather(
        crate::steam::find_game(),
        crate::ea_app::installs(),
        &config.games.added,
    )
}

/// The copy Play uses: the one the player picked while it is still there,
/// else the first.
pub fn active<'a>(copies: &'a [GameCopy], config: &Config) -> Option<&'a GameCopy> {
    config
        .games
        .active
        .as_deref()
        .and_then(|a| copies.iter().find(|c| same_dir(&c.dir, a)))
        .or_else(|| copies.first())
}

/// Added folders without the game in them any more: uninstalled, moved, or
/// on a drive that is not connected.
pub fn missing(copies: &[GameCopy], config: &Config) -> Vec<PathBuf> {
    config
        .games
        .added
        .iter()
        .filter(|d| !copies.iter().any(|c| same_dir(&c.dir, d)))
        .cloned()
        .collect()
}

pub fn find_active(config: &Config) -> Option<GameCopy> {
    active(&discover(config), config).cloned()
}

/// What `f1mp game use` names: a store or a folder.
#[derive(Debug, Clone, PartialEq)]
pub enum Choice {
    Store(Source),
    Dir(PathBuf),
}

impl Choice {
    pub fn parse(text: &str) -> Self {
        match text.to_ascii_lowercase().as_str() {
            "steam" => Self::Store(Source::Steam),
            "ea" | "ea-app" | "eaapp" | "ea app" => Self::Store(Source::EaApp),
            _ => Self::Dir(PathBuf::from(text)),
        }
    }

    pub fn pick<'a>(&self, copies: &'a [GameCopy]) -> Option<&'a GameCopy> {
        match self {
            Self::Store(s) => copies.iter().find(|c| c.source == *s),
            Self::Dir(d) => copies.iter().find(|c| same_dir(&c.dir, d)),
        }
    }
}

pub fn set_active(config: &mut Config, dir: &Path) {
    config.games.active = Some(dir.to_path_buf());
}

/// Add a folder with the game in it and make it the active copy.
pub fn add(config: &mut Config, dir: &Path) -> Result<(), String> {
    if !dir.join(GAME_EXE).is_file() {
        return Err(format!("{GAME_EXE} is not in {}", dir.display()));
    }
    config.games.add(dir.to_path_buf());
    set_active(config, dir);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Root(PathBuf);

    impl Root {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("swofty-copies-{}-{name}", std::process::id()));
            std::fs::remove_dir_all(&dir).ok();
            Self(dir)
        }

        fn game(&self, rel: &str, ea: bool) -> PathBuf {
            let d = self.0.join(rel);
            std::fs::create_dir_all(d.join("__Installer")).unwrap();
            std::fs::write(d.join(GAME_EXE), b"MZ").unwrap();
            if ea {
                std::fs::write(d.join(crate::ea_app::MARKER), "<contentID>1</contentID>").unwrap();
            }
            d
        }
    }

    impl Drop for Root {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).ok();
        }
    }

    fn steam(dir: &Path) -> GameInstall {
        GameInstall {
            steam_root: dir.into(),
            library: dir.into(),
            dir: dir.into(),
            build_id: Some(7),
            compatdata: None,
        }
    }

    #[test]
    fn lists_each_copy_once_in_store_order() {
        let root = Root::new("gather");
        let s = root.game("steam/F1 25", false);
        let e = root.game("ea/F1 25", true);
        let a = root.game("added/F1 25", false);
        let gone = root.0.join("gone");
        let copies = gather(
            Some(steam(&s)),
            vec![e.clone()],
            &[e.clone(), a.clone(), gone, s.clone()],
        );
        let dirs: Vec<&Path> = copies.iter().map(|c| c.dir.as_path()).collect();
        assert_eq!(dirs, [s.as_path(), e.as_path(), a.as_path()]);
        assert_eq!(copies[0].found, Found::Steam);
        assert_eq!(copies[0].build_id(), Some(7));
        assert_eq!(copies[1].source, Source::EaApp);
        assert_eq!(copies[1].found, Found::EaApp);
        assert_eq!(copies[2].found, Found::Added);
    }

    #[test]
    fn an_added_ea_folder_is_labelled_ea() {
        let root = Root::new("label");
        let e = root.game("anywhere", true);
        let copies = gather(None, Vec::new(), &[e]);
        assert_eq!(copies[0].source, Source::EaApp);
        assert_eq!(store_name(copies[0].source), "EA app");
    }

    #[test]
    fn plays_the_picked_copy_while_it_is_there() {
        let root = Root::new("active");
        let s = root.game("steam", false);
        let e = root.game("ea", true);
        let copies = gather(Some(steam(&s)), vec![e.clone()], &[]);
        let mut config = Config::default();
        assert_eq!(active(&copies, &config).unwrap().dir, s);
        set_active(&mut config, &e);
        assert_eq!(active(&copies, &config).unwrap().dir, e);
        set_active(&mut config, &root.0.join("uninstalled"));
        assert_eq!(active(&copies, &config).unwrap().dir, s);
        config.games.added = vec![e.clone(), root.0.join("uninstalled")];
        assert_eq!(missing(&copies, &config), [root.0.join("uninstalled")]);
        assert!(active(&[], &config).is_none());
    }

    #[test]
    fn switching_copies_switches_the_store_and_the_build() {
        let root = Root::new("swap");
        let s = root.game("steam", false);
        let e = root.game("ea", true);
        std::fs::write(e.join(GAME_EXE), b"MZ another build").unwrap();
        let copies = gather(Some(steam(&s)), vec![e.clone()], &[]);
        let data = root.0.join("data");
        let mut config = Config::default();
        let seen: Vec<_> = [&s, &e, &s]
            .into_iter()
            .map(|dir| {
                set_active(&mut config, dir);
                let c = active(&copies, &config).unwrap();
                let id = crate::baseline::Identity::detect_cached(&data, &c.dir, c.build_id()).unwrap();
                (c.dir.clone(), c.source, c.build_id(), id.key())
            })
            .collect();
        assert_eq!((seen[0].1, seen[0].2), (Source::Steam, Some(7)));
        assert_eq!((seen[1].1, seen[1].2), (Source::EaApp, None));
        assert_eq!(seen[1].0, e);
        assert!(seen[1].3.starts_with("exe-"));
        assert_ne!(seen[0].3, seen[1].3);
        assert_eq!(seen[0], seen[2]);
    }

    #[test]
    fn a_located_folder_stays_the_active_copy_after_migration() {
        let root = Root::new("migrate");
        let s = root.game("steam", false);
        let mine = root.game("mine", false);
        let mut config: Config = serde_json::from_value(serde_json::json!({ "game_dir": mine })).unwrap();
        config.migrate_games();
        let copies = gather(Some(steam(&s)), Vec::new(), &config.games.added);
        assert_eq!(copies.len(), 2);
        assert_eq!(active(&copies, &config).unwrap().dir, mine);
    }

    #[test]
    fn picks_by_store_or_folder() {
        let root = Root::new("pick");
        let s = root.game("steam", false);
        let e = root.game("ea", true);
        let copies = gather(Some(steam(&s)), vec![e.clone()], &[]);
        assert_eq!(Choice::parse("EA").pick(&copies).unwrap().dir, e);
        assert_eq!(Choice::parse("steam").pick(&copies).unwrap().dir, s);
        assert_eq!(Choice::parse(e.to_str().unwrap()).pick(&copies).unwrap().dir, e);
        assert!(Choice::parse("/nowhere").pick(&copies).is_none());
    }

    #[test]
    fn adding_needs_the_game_and_switches_to_it() {
        let root = Root::new("add");
        let a = root.game("added", false);
        let mut config = Config::default();
        assert!(add(&mut config, &root.0).is_err());
        add(&mut config, &a).unwrap();
        add(&mut config, &a).unwrap();
        assert_eq!(config.games.added, std::slice::from_ref(&a));
        assert_eq!(config.games.active, Some(a));
    }
}
