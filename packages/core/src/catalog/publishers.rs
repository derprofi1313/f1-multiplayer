//! The registry's publishers: the Discord accounts that publish addons. The
//! catalog groups by them, not by the free-text manifest author, which anyone
//! can set to anything.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize};

use super::{Addon, CACHE_DIR, CATALOG_LIMIT, Publisher, TIMEOUT, Trouble, base_url, get};

pub const CREATORS_FILE: &str = "publishers.json";

fn text<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Ok(Option::<String>::deserialize(d)?.unwrap_or_default())
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Link {
    #[serde(default, deserialize_with = "text")]
    pub label: String,
    #[serde(default, deserialize_with = "text")]
    pub url: String,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct Totals {
    #[serde(default)]
    pub addons: u64,
    #[serde(default)]
    pub downloads: u64,
    #[serde(default)]
    pub downloads_30d: u64,
    #[serde(default)]
    pub stars: u64,
}

/// A publisher as the registry lists them, with their totals.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Creator {
    pub id: String,
    #[serde(default, deserialize_with = "text")]
    pub name: String,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default, deserialize_with = "text")]
    pub bio: String,
    #[serde(default, deserialize_with = "text")]
    pub joined: String,
    #[serde(default)]
    pub addons: u64,
    #[serde(default)]
    pub downloads: u64,
    #[serde(default)]
    pub downloads_30d: u64,
    #[serde(default)]
    pub stars: u64,
}

impl Creator {
    pub fn totals(&self) -> Totals {
        Totals {
            addons: self.addons,
            downloads: self.downloads,
            downloads_30d: self.downloads_30d,
            stars: self.stars,
        }
    }
}

/// A publisher's profile and the listed addons they publish.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    pub id: String,
    #[serde(default, deserialize_with = "text")]
    pub name: String,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default, deserialize_with = "text")]
    pub bio: String,
    #[serde(default)]
    pub links: Vec<Link>,
    #[serde(default, deserialize_with = "text")]
    pub joined: String,
    #[serde(default)]
    pub totals: Totals,
    #[serde(default)]
    pub addons: Vec<Addon>,
}

impl Addon {
    /// Who owns it: its first publisher.
    pub fn owner(&self) -> Option<&Publisher> {
        self.publisher.as_ref().or(self.publishers.first())
    }

    pub fn published_by(&self, id: &str) -> bool {
        self.owner().is_some_and(|o| o.id == id) || self.publishers.iter().any(|p| p.id == id)
    }

    /// The manifest's author when it names someone other than the owner.
    pub fn credit(&self) -> Option<&str> {
        let author = self.author.trim();
        let owner = self.owner().map_or("", |o| o.name.trim());
        (!author.is_empty() && !owner.is_empty() && !author.eq_ignore_ascii_case(owner)).then_some(author)
    }
}

/// The owners of `addons` with how many each owns, most first.
pub fn by_publisher(addons: &[Addon]) -> Vec<(Publisher, usize)> {
    let mut out: Vec<(Publisher, usize)> = Vec::new();
    for owner in addons.iter().filter_map(Addon::owner) {
        match out.iter_mut().find(|(p, _)| p.id == owner.id) {
            Some((_, n)) => *n += 1,
            None => out.push((owner.clone(), 1)),
        }
    }
    out.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| a.0.name.to_lowercase().cmp(&b.0.name.to_lowercase()))
    });
    out
}

/// Up to `limit` of the other listed addons `a`'s owner publishes, most starred first.
pub fn more_from<'a>(addons: &'a [Addon], a: &Addon, limit: usize) -> Vec<&'a Addon> {
    let Some(owner) = a.owner() else {
        return Vec::new();
    };
    let mut out: Vec<&Addon> = addons
        .iter()
        .filter(|x| x.id != a.id && x.published_by(&owner.id))
        .collect();
    out.sort_by(|x, y| {
        y.stars
            .cmp(&x.stars)
            .then_with(|| y.downloads.cmp(&x.downloads))
            .then_with(|| x.name.to_lowercase().cmp(&y.name.to_lowercase()))
    });
    out.truncate(limit);
    out
}

/// The publishers of `addons` with totals summed from the catalog, most
/// downloaded first, for when the registry's list isn't at hand.
pub fn from_catalog(addons: &[Addon]) -> Vec<Creator> {
    let mut out: Vec<Creator> = Vec::new();
    for a in addons {
        let mut seen: Vec<&str> = Vec::new();
        for p in a.owner().into_iter().chain(&a.publishers) {
            if seen.contains(&p.id.as_str()) {
                continue;
            }
            seen.push(&p.id);
            let i = match out.iter().position(|c| c.id == p.id) {
                Some(i) => i,
                None => {
                    out.push(Creator {
                        id: p.id.clone(),
                        name: p.name.clone(),
                        avatar: p.avatar.clone(),
                        ..Creator::default()
                    });
                    out.len() - 1
                }
            };
            let c = &mut out[i];
            c.addons += 1;
            c.downloads += a.downloads;
            c.downloads_30d += a.downloads_30d;
            c.stars += a.stars;
        }
    }
    sort(&mut out);
    out
}

fn sort(creators: &mut [Creator]) {
    creators.sort_by(|a, b| {
        b.downloads
            .cmp(&a.downloads)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
}

#[derive(Deserialize, Serialize)]
struct Creators {
    publishers: Vec<Creator>,
}

pub fn parse(text: &str) -> Option<Vec<Creator>> {
    serde_json::from_str::<Creators>(text).ok().map(|c| c.publishers)
}

pub fn cache_path(root: &Path) -> PathBuf {
    root.join(CACHE_DIR).join(CREATORS_FILE)
}

pub fn cached(root: &Path) -> Option<Vec<Creator>> {
    std::fs::read_to_string(cache_path(root))
        .ok()
        .and_then(|t| parse(&t))
}

fn read(url: &str) -> Result<String, Trouble> {
    get(url, TIMEOUT)?
        .with_config()
        .limit(CATALOG_LIMIT)
        .read_to_string()
        .map_err(|e| Trouble::Failed(format!("{url}: {e}")))
}

/// The publishers from the registry, most downloaded first, saved for the next start.
pub fn fetch(root: &Path) -> Result<Vec<Creator>, Trouble> {
    let url = format!("{}/v1/publishers", base_url());
    let text = read(&url)?;
    let mut creators = parse(&text).ok_or_else(|| Trouble::Failed(format!("{url}: not a publisher list")))?;
    sort(&mut creators);
    let path = cache_path(root);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).ok();
    }
    if let Err(e) = std::fs::write(&path, text) {
        eprintln!("swofty-core: could not save the publishers: {e}");
    }
    Ok(creators)
}

pub fn fetch_profile(id: &str) -> Result<Profile, Trouble> {
    let url = format!("{}/v1/publishers/{id}", base_url());
    let text = read(&url)?;
    serde_json::from_str(&text).map_err(|e| Trouble::Failed(format!("{url}: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn publisher(id: &str, name: &str) -> Publisher {
        Publisher {
            id: id.into(),
            name: name.into(),
            avatar: None,
        }
    }

    fn addon(id: &str, author: &str, owner: Option<Publisher>, stars: u64) -> Addon {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "name": id,
            "author": author,
            "publisher": owner,
            "publishers": owner.iter().collect::<Vec<_>>(),
            "stars": stars,
            "downloads": stars * 10,
        }))
        .unwrap()
    }

    fn catalog() -> Vec<Addon> {
        let grid = publisher("1", "Retro Grid");
        let jane = publisher("2", "Jane");
        let mut shared = addon("shared", "Retro Grid", Some(jane.clone()), 3);
        shared.publishers.push(grid.clone());
        vec![
            addon("liveries", "Retro Grid", Some(grid.clone()), 9),
            addon("season", "Jane D.", Some(grid.clone()), 4),
            addon("helmets", "Retro Grid", Some(grid.clone()), 7),
            addon("grids", "Retro Grid", Some(grid), 1),
            addon("tracks", "Jane", Some(jane), 2),
            shared,
            addon("legacy", "Somebody", None, 50),
        ]
    }

    #[test]
    fn groups_by_publisher_not_manifest_author() {
        let groups = by_publisher(&catalog());
        let named: Vec<(&str, &str, usize)> = groups
            .iter()
            .map(|(p, n)| (p.id.as_str(), p.name.as_str(), *n))
            .collect();
        assert_eq!(named, [("1", "Retro Grid", 4), ("2", "Jane", 2)]);
    }

    #[test]
    fn credits_a_manifest_author_who_is_not_the_owner() {
        let c = catalog();
        assert_eq!(c[0].credit(), None);
        assert_eq!(c[1].credit(), Some("Jane D."));
        assert_eq!(c[5].credit(), Some("Retro Grid"));
        assert_eq!(c[6].credit(), None);
        let mut lower = c[0].clone();
        lower.author = "retro grid".into();
        assert_eq!(lower.credit(), None);
        let mut old = c[4].clone();
        old.publisher = None;
        assert_eq!(old.owner().map(|o| o.id.as_str()), Some("2"));
    }

    #[test]
    fn picks_more_from_the_same_publisher() {
        let c = catalog();
        let ids = |a: &Addon, n| {
            more_from(&c, a, n)
                .iter()
                .map(|x| x.id.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(ids(&c[0], 4), ["helmets", "season", "shared", "grids"]);
        assert_eq!(ids(&c[0], 2), ["helmets", "season"]);
        assert_eq!(ids(&c[4], 4), ["shared"]);
        assert_eq!(ids(&c[5], 4), ["tracks"]);
        assert!(ids(&c[6], 4).is_empty());
        let alone = vec![c[4].clone()];
        assert!(more_from(&alone, &c[4], 4).is_empty());
    }

    #[test]
    fn sums_creators_from_the_catalog() {
        let creators = from_catalog(&catalog());
        let totals: Vec<(&str, u64, u64)> = creators
            .iter()
            .map(|c| (c.name.as_str(), c.addons, c.stars))
            .collect();
        assert_eq!(totals, [("Retro Grid", 5, 24), ("Jane", 2, 5)]);
    }

    #[test]
    fn reads_publishers_and_profiles() {
        let list = r#"{"publishers":[{"id":"9","name":"Lumen","avatar":null,"bio":null,"addons":2,
            "downloads":581,"downloads_30d":237,"stars":16,"joined":"2026-04-07T13:13:04Z"},
            {"id":"8","name":"Aero","addons":1,"downloads":900}]}"#;
        let creators = parse(list).unwrap();
        assert_eq!(creators[0].bio, "");
        assert_eq!(creators[0].totals().downloads_30d, 237);
        assert_eq!(creators[1].joined, "");
        let profile: Profile = serde_json::from_str(
            r#"{"id":"9","name":"Lumen","avatar":"http://x/v1/publishers/9/avatar","bio":"a\nb",
            "links":[{"label":"YouTube","url":"https://youtube.com/@l"}],"joined":"2026-04-07T13:13:04Z",
            "totals":{"addons":2,"downloads":581,"downloads_30d":237,"stars":16},
            "addons":[{"id":"x","name":"X","publisher":{"id":"9","name":"Lumen","avatar":null}}]}"#,
        )
        .unwrap();
        assert_eq!(profile.links[0].label, "YouTube");
        assert_eq!(profile.totals.stars, 16);
        assert!(profile.addons[0].published_by("9"));
        assert!(parse("{}").is_none());
    }
}
