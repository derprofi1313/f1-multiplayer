use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::builtin::CORE_ID;
use crate::manifest::{AddonManifest, AddonOption, Binds};

pub const VANILLA: &str = "vanilla";
pub const SCHEMA_VERSION: u32 = 2;

pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 40
        && name != VANILLA
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// Fields a newer launcher wrote are kept in `unknown` and written back unchanged.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    pub schema_version: u32,
    pub name: String,
    #[serde(default, alias = "mods")]
    pub addons: Vec<ProfileAddon>,
    #[serde(default, rename = "maps", skip_serializing)]
    pub legacy_maps: Vec<String>,
    #[serde(default)]
    pub options: BTreeMap<String, BTreeMap<String, serde_json::Value>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub applied: BTreeMap<String, BTreeMap<String, serde_json::Value>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub state: BTreeMap<String, serde_json::Value>,
    /// "Use launcher saves": sessions of this profile play on the profile's own save store; off plays
    /// on Steam's saves.
    #[serde(default = "enabled", skip_serializing_if = "is_true")]
    pub launcher_saves: bool,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub unknown: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProfileAddon {
    pub id: String,
    #[serde(default = "enabled")]
    pub enabled: bool,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub unknown: BTreeMap<String, serde_json::Value>,
}

fn enabled() -> bool {
    true
}

fn is_true(b: &bool) -> bool {
    *b
}

impl Profile {
    pub fn new(name: &str) -> Self {
        let mut p = Self {
            schema_version: SCHEMA_VERSION,
            name: name.into(),
            addons: Vec::new(),
            legacy_maps: Vec::new(),
            options: BTreeMap::new(),
            applied: BTreeMap::new(),
            state: BTreeMap::new(),
            launcher_saves: true,
            unknown: BTreeMap::new(),
        };
        p.normalise();
        p
    }

    pub fn vanilla() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            name: VANILLA.into(),
            addons: Vec::new(),
            legacy_maps: Vec::new(),
            options: BTreeMap::new(),
            applied: BTreeMap::new(),
            state: BTreeMap::new(),
            launcher_saves: true,
            unknown: BTreeMap::new(),
        }
    }

    /// The fields a newer launcher wrote that this one keeps without using, as `name` or
    /// `addons[id].name`.
    pub fn unknown_fields(&self) -> Vec<String> {
        let mut out: Vec<String> = self.unknown.keys().cloned().collect();
        for a in &self.addons {
            out.extend(a.unknown.keys().map(|k| format!("addons[{}].{k}", a.id)));
        }
        out
    }

    pub fn is_vanilla(&self) -> bool {
        self.name == VANILLA
    }

    pub fn save_namespace(&self) -> String {
        format!("swofty-{}", self.name)
    }

    pub fn normalise(&mut self) {
        if self.is_vanilla() {
            *self = Self::vanilla();
            return;
        }
        self.schema_version = SCHEMA_VERSION;
        self.addons.retain(|m| m.id != CORE_ID);
        self.addons.insert(
            0,
            ProfileAddon {
                id: CORE_ID.into(),
                enabled: true,
                unknown: BTreeMap::new(),
            },
        );
    }

    pub fn enabled(&self) -> impl Iterator<Item = &str> {
        self.addons
            .iter()
            .filter(|m| m.enabled || m.id == CORE_ID)
            .map(|m| m.id.as_str())
    }

    /// Moves `id` by `delta` places among the profile's addons, which is its
    /// place in the load order; the core addon stays first.
    pub fn move_addon(&mut self, id: &str, delta: isize) -> crate::Result<()> {
        if self.is_vanilla() {
            return Err(crate::Error::Other(format!(
                "{VANILLA} is built in and cannot be changed"
            )));
        }
        if id == CORE_ID {
            return Err(crate::Error::Other(format!("{CORE_ID} always loads first")));
        }
        let from = self
            .addons
            .iter()
            .position(|m| m.id == id)
            .ok_or_else(|| crate::Error::Other(format!("{id} is not in the {} profile", self.name)))?;
        let first = usize::from(self.addons.first().is_some_and(|m| m.id == CORE_ID));
        let to = from
            .saturating_add_signed(delta)
            .clamp(first, self.addons.len() - 1);
        let addon = self.addons.remove(from);
        self.addons.insert(to, addon);
        Ok(())
    }

    pub fn set_enabled(&mut self, id: &str, on: bool) -> crate::Result<()> {
        if self.is_vanilla() {
            return Err(crate::Error::Other(format!(
                "{VANILLA} is built in and cannot be changed"
            )));
        }
        if id == CORE_ID && !on {
            return Err(crate::Error::Other(format!(
                "{CORE_ID} is built in and cannot be disabled"
            )));
        }
        match self.addons.iter_mut().find(|m| m.id == id) {
            Some(m) => m.enabled = on,
            None => self.addons.push(ProfileAddon {
                id: id.into(),
                enabled: on,
                unknown: BTreeMap::new(),
            }),
        }
        Ok(())
    }

    pub fn option(&self, m: &AddonManifest, o: &AddonOption) -> serde_json::Value {
        stored(&self.options, m, o).unwrap_or_else(|| o.default_value())
    }

    pub fn set_option(
        &mut self,
        m: &AddonManifest,
        id: &str,
        value: serde_json::Value,
    ) -> crate::Result<bool> {
        if self.is_vanilla() {
            return Err(crate::Error::Other(format!(
                "{VANILLA} is built in and cannot be changed"
            )));
        }
        let o = m
            .options
            .iter()
            .find(|o| o.id == id)
            .ok_or_else(|| crate::Error::Other(format!("{} has no option {id}", m.id)))?;
        let v = o
            .normalise(&value)
            .ok_or_else(|| crate::Error::Other(format!("{value} is not a valid value for {}.{id}", m.id)))?;
        let before = self.option(m, o);
        let values = self.options.entry(m.id.clone()).or_default();
        if v == o.default_value() {
            values.remove(id);
        } else {
            values.insert(id.to_owned(), v.clone());
        }
        if values.is_empty() {
            self.options.remove(&m.id);
        }
        Ok(before != v)
    }

    pub fn reset_options(&mut self, m: &AddonManifest) -> crate::Result<bool> {
        if self.is_vanilla() {
            return Err(crate::Error::Other(format!(
                "{VANILLA} is built in and cannot be changed"
            )));
        }
        Ok(self.options.remove(&m.id).is_some())
    }

    pub fn rebuild_pending(&self, installed: &BTreeMap<String, AddonManifest>) -> Vec<String> {
        let mut out = Vec::new();
        for m in self.enabled().filter_map(|id| installed.get(id)) {
            for o in m.options.iter().filter(|o| o.binds == Binds::Db) {
                let baked = stored(&self.applied, m, o).unwrap_or_else(|| o.default_value());
                if self.option(m, o) != baked {
                    out.push(format!("{}.{}", m.id, o.id));
                }
            }
        }
        out
    }

    pub fn mark_built(&mut self, installed: &BTreeMap<String, AddonManifest>) {
        let mut applied = BTreeMap::new();
        for m in self.enabled().filter_map(|id| installed.get(id)) {
            let values: BTreeMap<String, serde_json::Value> = m
                .options
                .iter()
                .filter(|o| o.binds == Binds::Db)
                .map(|o| (o.id.clone(), self.option(m, o)))
                .filter(|(id, v)| m.options.iter().any(|o| &o.id == id && *v != o.default_value()))
                .collect();
            if !values.is_empty() {
                applied.insert(m.id.clone(), values);
            }
        }
        self.applied = applied;
    }

    pub fn set_state(&mut self, addon: &str, value: serde_json::Value) -> crate::Result<()> {
        if self.is_vanilla() {
            return Err(crate::Error::Other(format!(
                "{VANILLA} is built in and cannot be changed"
            )));
        }
        self.state.insert(addon.into(), value);
        Ok(())
    }
}

fn stored(
    map: &BTreeMap<String, BTreeMap<String, serde_json::Value>>,
    m: &AddonManifest,
    o: &AddonOption,
) -> Option<serde_json::Value> {
    map.get(&m.id)?.get(&o.id).and_then(|v| o.normalise(v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_is_always_first_and_enabled() {
        let mut p: Profile = serde_json::from_str(
            r#"{"schema_version":1,"name":"x","mods":[{"id":"a.b"},{"id":"swofty.core","enabled":false}]}"#,
        )
        .unwrap();
        p.normalise();
        assert_eq!(p.enabled().collect::<Vec<_>>(), ["swofty.core", "a.b"]);
        assert!(p.set_enabled(CORE_ID, false).is_err());
        p.set_enabled("a.b", false).unwrap();
        assert_eq!(p.enabled().collect::<Vec<_>>(), ["swofty.core"]);
    }

    #[test]
    fn moves_addons_in_the_load_order() {
        let mut p = Profile::new("x");
        for id in ["a.a", "b.b", "c.c"] {
            p.set_enabled(id, true).unwrap();
        }
        let order = |p: &Profile| p.addons.iter().map(|a| a.id.clone()).collect::<Vec<_>>();
        p.move_addon("c.c", -1).unwrap();
        assert_eq!(order(&p), [CORE_ID, "a.a", "c.c", "b.b"]);
        p.move_addon("b.b", -10).unwrap();
        assert_eq!(order(&p), [CORE_ID, "b.b", "a.a", "c.c"]);
        p.move_addon("b.b", 10).unwrap();
        assert_eq!(order(&p), [CORE_ID, "a.a", "c.c", "b.b"]);
        p.move_addon("a.a", 0).unwrap();
        assert_eq!(order(&p), [CORE_ID, "a.a", "c.c", "b.b"]);
        assert!(p.move_addon(CORE_ID, 1).is_err() && p.move_addon("nope.x", 1).is_err());
        assert!(Profile::vanilla().move_addon("a.a", 1).is_err());
    }

    #[test]
    fn profile_names() {
        assert!(valid_name("grid-24_b"));
        assert!(!valid_name(VANILLA));
        assert!(!valid_name("a b"));
        assert!(!valid_name("../x"));
    }
}
