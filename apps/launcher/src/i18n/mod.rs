//! The launcher's language. The source text is English: `tr("Turn off all")` returns the German
//! text when the German language is on and the table has the string, and the English otherwise,
//! so a string without a translation yet still shows.
//!
//! The German table is split per area (one `de_*.rs` slice each), so that work on one area
//! doesn't touch another's texts; `table()` chains them and the keys stay unique across all.

use std::collections::HashMap;
use std::sync::OnceLock;
#[cfg(not(test))]
use std::sync::atomic::{AtomicBool, Ordering};

use gpui_kit::component::WindowExt;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;

use crate::shell::Shell;

mod de_base;
mod de_contracts;
mod de_gamesync;
mod de_round;
mod de_team;
mod de_texts;
mod de_weekend;

/// Every slice of the German table.
const SLICES: &[&[(&str, &str)]] = &[
    de_base::DE,
    de_round::DE,
    de_contracts::DE,
    de_weekend::DE,
    de_team::DE,
    de_texts::DE,
    de_gamesync::DE,
];

#[cfg(not(test))]
static GERMAN: AtomicBool = AtomicBool::new(false);

// Tests run in parallel threads: each has its own language, so a test that switches to German
// doesn't change what another test's `tr()` returns.
#[cfg(test)]
thread_local! {
    static GERMAN: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Sets the language from the config's code. An unknown or missing code is English.
pub fn set(code: Option<&str>) {
    #[cfg(not(test))]
    GERMAN.store(code == Some("de"), Ordering::Relaxed);
    #[cfg(test)]
    GERMAN.with(|g| g.set(code == Some("de")));
}

pub fn is_german() -> bool {
    #[cfg(not(test))]
    return GERMAN.load(Ordering::Relaxed);
    #[cfg(test)]
    GERMAN.with(|g| g.get())
}

/// The text to show for an English source string.
pub fn tr(text: &'static str) -> &'static str {
    if !is_german() {
        return text;
    }
    table().get(text).copied().unwrap_or(text)
}

/// The text to show for an English source string that isn't `'static`, such as a template with
/// `{placeholders}` that came with the data; fill the placeholders afterwards with `.replace()`.
#[allow(dead_code, reason = "the career's inbox and news use it from wave 3 on")]
pub fn tr_owned(text: &str) -> String {
    if !is_german() {
        return text.to_owned();
    }
    table().get(text).copied().unwrap_or(text).to_owned()
}

impl Shell {
    /// Switches the launcher's language and saves it in the config.
    pub(crate) fn set_language(&mut self, code: &str, window: &mut Window, cx: &mut Context<Self>) {
        let mut config = self.store.config();
        config.language = Some(code.to_owned());
        set(Some(code));
        if let Err(e) = self.store.save_config(&config) {
            crate::app_log::action("save language").failed(&e);
            window.push_notification(Notification::error(e.to_string()), cx);
        }
        cx.notify();
    }
}

fn table() -> &'static HashMap<&'static str, &'static str> {
    static TABLE: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    TABLE.get_or_init(|| SLICES.iter().flat_map(|slice| slice.iter().copied()).collect())
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::{SLICES, set, tr, tr_owned};

    /// The language is global: the tests that switch it take turns.
    static LANGUAGE: Mutex<()> = Mutex::new(());

    #[test]
    fn german_keys_are_unique_and_filled() {
        let mut seen = std::collections::HashSet::new();
        for (en, de) in SLICES.iter().flat_map(|slice| slice.iter()) {
            assert!(seen.insert(*en), "duplicate key {en}");
            assert!(!de.trim().is_empty(), "empty German text for {en}");
        }
    }

    #[test]
    fn english_is_the_default_and_german_translates() {
        let _turn = LANGUAGE.lock().unwrap_or_else(|e| e.into_inner());
        set(None);
        assert_eq!(tr("Turn off all"), "Turn off all");
        set(Some("de"));
        assert_eq!(tr("Turn off all"), "Alle ausschalten");
        assert_eq!(tr("Not in the table"), "Not in the table");
        set(None);
    }

    #[test]
    fn tr_owned_translates_templates_and_falls_back() {
        let _turn = LANGUAGE.lock().unwrap_or_else(|e| e.into_inner());
        // A key built at run time, as a template that came with the data would be.
        let key = String::from("Turn off ") + "all";
        set(None);
        assert_eq!(tr_owned(&key), "Turn off all");
        set(Some("de"));
        assert_eq!(tr_owned(&key), "Alle ausschalten");
        assert_eq!(tr_owned(&key), tr("Turn off all"));
        let template = "{name} won in {track}";
        assert_eq!(tr_owned(template), template);
        assert_eq!(
            tr_owned(template)
                .replace("{name}", "Anna")
                .replace("{track}", "Monza"),
            "Anna won in Monza"
        );
        set(None);
    }
}
