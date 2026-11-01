//! The launcher's language. The source text is English: `tr("Turn off all")` returns the German
//! text when the German language is on and the table has the string, and the English otherwise,
//! so a string without a translation yet still shows.

use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use gpui_kit::component::WindowExt;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;

use crate::shell::Shell;

static GERMAN: AtomicBool = AtomicBool::new(false);

/// Sets the language from the config's code. An unknown or missing code is English.
pub fn set(code: Option<&str>) {
    GERMAN.store(code == Some("de"), Ordering::Relaxed);
}

pub fn is_german() -> bool {
    GERMAN.load(Ordering::Relaxed)
}

/// The text to show for an English source string.
pub fn tr(text: &'static str) -> &'static str {
    if !is_german() {
        return text;
    }
    table().get(text).copied().unwrap_or(text)
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
    TABLE.get_or_init(|| DE.iter().copied().collect())
}

const DE: &[(&str, &str)] = &[
    // Pages and the sidebar
    ("News", "Neuigkeiten"),
    ("Addons", "Addons"),
    ("Addon Browser", "Addon-Browser"),
    ("Simplified Importer", "Vereinfachter Importer"),
    ("Importer", "Importer"),
    ("Tools", "Werkzeuge"),
    ("Profiles", "Profile"),
    ("DB Editor", "DB-Editor"),
    ("Multiplayer", "Mehrspieler"),
    ("Career", "Karriere"),
    ("Saves", "Spielstände"),
    // Title bar and play bar
    ("Log out", "Abmelden"),
    ("Play", "Spielen"),
    ("Play in VR", "In VR spielen"),
    ("Play with console", "Mit Konsole spielen"),
    ("Developer mode", "Entwicklermodus"),
    ("Play vanilla (no addons)", "Vanilla spielen (ohne Addons)"),
    ("Use launcher saves", "Launcher-Spielstände nutzen"),
    ("Play once with your Steam saves", "Einmal mit deinen Steam-Spielständen spielen"),
    // Addons page
    ("Search addons", "Addons suchen"),
    ("Turn off all", "Alle ausschalten"),
    ("Open Tracks", "Tracks öffnen"),
    ("Clear", "Leeren"),
    ("Clear filters", "Filter leeren"),
    ("Filters", "Filter"),
    ("New folder", "Neuer Ordner"),
    ("Folder name", "Ordnername"),
    ("Make folder", "Ordner erstellen"),
    ("Rename", "Umbenennen"),
    ("Rename folder", "Ordner umbenennen"),
    ("No addons match", "Keine Addons gefunden"),
    ("Drop here to move to the end", "Hier ablegen, um ans Ende zu verschieben"),
    ("CAN'T BE USED", "KANN NICHT VERWENDET WERDEN"),
    ("INSTALLED", "INSTALLIERT"),
    ("Empty · drag addons here", "Leer · Addons hierher ziehen"),
    ("Remove", "Entfernen"),
    ("Uses experimental features", "Nutzt experimentelle Funktionen"),
    ("Update available", "Update verfügbar"),
    ("Track left out", "Track ausgelassen"),
    ("Works with any game version", "Funktioniert mit jeder Spielversion"),
    ("Works with this game version", "Funktioniert mit dieser Spielversion"),
    ("On", "An"),
    ("Off", "Aus"),
    ("Has conflicts", "Hat Konflikte"),
    ("Uncategorised", "Ohne Kategorie"),
    ("CATEGORY", "KATEGORIE"),
    ("TYPE", "TYP"),
    ("SOURCE", "QUELLE"),
    ("STATE", "STATUS"),
    ("Always on", "Immer an"),
    ("Choose a folder…", "Ordner wählen…"),
    ("Reset to defaults", "Auf Standard zurücksetzen"),
    ("SETTINGS", "EINSTELLUNGEN"),
    ("Undo", "Rückgängig"),
    ("Restore", "Wiederherstellen"),
    ("Dismiss", "Ausblenden"),
    ("Checking…", "Prüfe…"),
    ("Check conflicts", "Konflikte prüfen"),
    ("Move up", "Nach oben"),
    ("Move down", "Nach unten"),
    ("Close", "Schließen"),
    ("Status", "Status"),
    ("Load order", "Ladereihenfolge"),
    ("Folder", "Ordner"),
    ("Category", "Kategorie"),
    ("Type", "Typ"),
    ("In game", "Im Spiel"),
    ("Source", "Quelle"),
    ("Game version", "Spielversion"),
    ("License", "Lizenz"),
    ("Homepage", "Webseite"),
    ("Requires", "Benötigt"),
    ("Advanced", "Erweitert"),
    ("Addon ID", "Addon-ID"),
    ("WHAT IT CHANGES", "WAS SIE ÄNDERT"),
    ("Edit in DB Editor", "Im DB-Editor bearbeiten"),
    // Saves page (Swofty 1.7: career vault and launcher saves)
    ("Your careers have their own saves now", "Deine Karrieren haben jetzt eigene Spielstände"),
    (
        "Each profile now has its own save, starting from a copy of your Steam save. Playing from Steam still uses your Steam save, and Vanilla plays on it too.",
        "Jedes Profil hat jetzt einen eigenen Spielstand, angefangen mit einer Kopie deines Steam-Spielstands. Wenn du über Steam spielst, wird weiter dein Steam-Spielstand genutzt, und Vanilla spielt auch darauf.",
    ),
    (
        "This profile's launcher save is empty. Import from Steam brings your careers in.",
        "Der Launcher-Spielstand dieses Profils ist leer. „Von Steam importieren“ holt deine Karrieren herein.",
    ),
    (
        "Launcher save is empty · import from Steam on the Saves page",
        "Launcher-Spielstand ist leer · auf der Seite Spielstände von Steam importieren",
    ),
    ("After Play", "Nach dem Spielen"),
    ("Before Play", "Vor dem Spielen"),
    ("Before restore", "Vor dem Wiederherstellen"),
    ("Backup", "Sicherung"),
    ("Old backup", "Alte Sicherung"),
    ("Saved", "Gespeichert"),
    ("Saved game", "Spielstand"),
    ("Restored", "Wiederhergestellt"),
    ("Launcher start", "Launcher-Start"),
    ("Changed outside the launcher", "Außerhalb des Launchers geändert"),
    ("F1 25 started outside the launcher", "F1 25 außerhalb des Launchers gestartet"),
    ("F1 25 closed outside the launcher", "F1 25 außerhalb des Launchers beendet"),
    ("Imported from Steam", "Von Steam importiert"),
    ("Imported Steam save", "Importierter Steam-Spielstand"),
    ("Copied from another profile", "Aus einem anderen Profil kopiert"),
    ("CAREERS", "KARRIEREN"),
    ("VERSIONS", "VERSIONEN"),
    ("Cancel", "Abbrechen"),
    ("Change", "Ändern"),
    ("Choose", "Auswählen"),
    ("Choose the folder…", "Ordner auswählen…"),
    ("Copy", "Kopieren"),
    ("Copy careers to…", "Karrieren kopieren nach…"),
    ("Export", "Exportieren"),
    ("Export to Steam", "Zu Steam exportieren"),
    ("Import", "Importieren"),
    ("Import from Steam", "Von Steam importieren"),
    ("Imported your Steam save.", "Dein Steam-Spielstand wurde importiert."),
    ("Open folder", "Ordner öffnen"),
    ("Open Career saves", "Karriere-Spielstände öffnen"),
    ("Pin", "Anheften"),
    ("Unpin", "Lösen"),
    ("Play with your Steam saves", "Mit deinen Steam-Spielständen spielen"),
    ("Plays on your Steam save", "Spielt auf deinem Steam-Spielstand"),
    ("This profile's own save", "Eigener Spielstand dieses Profils"),
    ("Vanilla saves", "Vanilla-Spielstände"),
    ("Vanilla uses the Steam save", "Vanilla nutzt den Steam-Spielstand"),
    ("Your Steam save", "Dein Steam-Spielstand"),
    ("Restored your save.", "Dein Spielstand wurde wiederhergestellt."),
    ("Review the restore", "Wiederherstellung prüfen"),
    ("Save", "Speichern"),
    ("Show me", "Zeigen"),
    ("Close F1 25 first", "Schließe zuerst F1 25"),
    ("Close Steam first", "Beende zuerst Steam"),
    ("Loading saves…", "Lade Spielstände…"),
    ("No F1 25 save found yet.", "Noch kein F1-25-Spielstand gefunden."),
    ("No Steam save found", "Kein Steam-Spielstand gefunden"),
    ("No career list to compare.", "Keine Karriereliste zum Vergleichen."),
    (
        "No careers yet. They appear after you play this profile.",
        "Noch keine Karrieren. Sie erscheinen, nachdem du mit diesem Profil gespielt hast.",
    ),
    ("No careers.", "Keine Karrieren."),
    ("No other profiles", "Keine anderen Profile"),
    ("No versions yet.", "Noch keine Versionen."),
    ("Nothing to copy yet", "Noch nichts zum Kopieren"),
    ("Nothing to export yet", "Noch nichts zum Exportieren"),
    ("Career saves storage", "Speicherplatz für Karriere-Spielstände"),
    (
        "The most space, in MB, the saved versions may take. The oldest unpinned versions go first.",
        "Der meiste Platz in MB, den die gespeicherten Versionen belegen dürfen. Die ältesten nicht angehefteten Versionen werden zuerst gelöscht.",
    ),
    ("Enter a number of MB.", "Gib eine Zahl in MB ein."),
    ("The limit must be at least 1 MB.", "Das Limit muss mindestens 1 MB sein."),
    (
        "Steam uploads the restored save when it starts.",
        "Steam lädt den wiederhergestellten Spielstand beim nächsten Start hoch.",
    ),
    ("You get back", "Du bekommst zurück"),
    ("You lose", "Du verlierst"),
    (
        "Couldn't find your Steam save, so this profile's launcher saves are empty.",
        "Dein Steam-Spielstand wurde nicht gefunden, deshalb sind die Launcher-Spielstände dieses Profils leer.",
    ),
    ("Switch profiles here to see their saves", "Hier das Profil wechseln, um seine Spielstände zu sehen"),
    ("ADDON PANELS", "ADDON-PANELS"),
    ("ACTIVE PROFILE", "AKTIVES PROFIL"),
    ("Send Logs to Developers", "Logs an die Entwickler senden"),
    ("Save Logs", "Logs speichern"),
];

#[cfg(test)]
mod tests {
    use super::{DE, set, tr};

    #[test]
    fn german_keys_are_unique_and_filled() {
        let mut seen = std::collections::HashSet::new();
        for (en, de) in DE {
            assert!(seen.insert(*en), "duplicate key {en}");
            assert!(!de.trim().is_empty(), "empty German text for {en}");
        }
    }

    #[test]
    fn english_is_the_default_and_german_translates() {
        set(None);
        assert_eq!(tr("Turn off all"), "Turn off all");
        set(Some("de"));
        assert_eq!(tr("Turn off all"), "Alle ausschalten");
        assert_eq!(tr("Not in the table"), "Not in the table");
        set(None);
    }
}
