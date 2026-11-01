use crate::manifest::AddonManifest;

pub const TITLE: &str = "This addon has affected your saves";
pub const BODY: &str = "Careers you played with it may not load without it.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavesWarning {
    pub id: String,
    pub name: String,
    pub message: Option<String>,
}

impl SavesWarning {
    pub fn of(m: &AddonManifest) -> Option<Self> {
        m.saves.affects.then(|| Self {
            id: m.id.clone(),
            name: m.name.clone(),
            message: m.saves.message().map(str::to_owned),
        })
    }

    pub fn text(&self) -> String {
        let mut text = format!(
            "Warning: {} ({}) has affected your saves. {BODY}",
            self.name, self.id
        );
        if let Some(m) = &self.message {
            text.push('\n');
            text.push_str(m);
        }
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(saves: &str) -> AddonManifest {
        AddonManifest::parse(&format!(
            r#"{{"schema_version":1,"id":"t.saves","name":"Saves","version":"1.0.0","author":"t",
                "compat":{{"game":"f1_25"}}{saves}}}"#
        ))
        .unwrap()
    }

    #[test]
    fn warns_only_for_addons_that_affect_saves() {
        assert_eq!(SavesWarning::of(&manifest("")), None);
        assert_eq!(
            SavesWarning::of(&manifest(r#","saves":{"affects":false,"message":"m"}"#)),
            None
        );
        let w = SavesWarning::of(&manifest(r#","saves":{"affects":true}"#)).unwrap();
        assert_eq!(w.message, None);
        assert_eq!(
            w.text(),
            "Warning: Saves (t.saves) has affected your saves. Careers you played with it may not load without it."
        );
        let w = SavesWarning::of(&manifest(
            r#","saves":{"affects":true,"message":" Retire drivers instead. "}"#,
        ))
        .unwrap();
        assert_eq!(w.message.as_deref(), Some("Retire drivers instead."));
        assert!(w.text().ends_with("without it.\nRetire drivers instead."));
        let blank = SavesWarning::of(&manifest(r#","saves":{"affects":true,"message":"  "}"#)).unwrap();
        assert_eq!(blank.message, None);
    }
}
