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
    // Career page (league and solo careers)
    ("Alone", "Allein"),
    ("With friends", "Mit Freunden"),
    ("Start career", "Karriere starten"),
    ("Join career", "Karriere beitreten"),
    ("Open Multiplayer", "Mehrspieler öffnen"),
    ("CODE", "CODE"),
    ("Race distance", "Renndistanz"),
    ("Give the career a name.", "Gib der Karriere einen Namen."),
    ("The race number is a number from 1 to 99.", "Die Startnummer ist eine Zahl von 1 bis 99."),
    ("Type the career code your friend sent you.", "Gib den Karriere-Code ein, den dir dein Freund geschickt hat."),
    (
        "That code belongs to a shared career that isn't a league career.",
        "Dieser Code gehört zu einer geteilten Karriere, die keine Liga-Karriere ist.",
    ),
    (
        "Close F1 25 first: it writes its settings when it quits.",
        "Schließe zuerst F1 25: Es speichert seine Einstellungen beim Beenden.",
    ),
    (
        "The game's UDP telemetry is on. Race results arrive by themselves now.",
        "Die UDP-Telemetrie des Spiels ist an. Rennergebnisse kommen jetzt von selbst an.",
    ),
    ("Turn telemetry on", "Telemetrie einschalten"),
    ("Loading your careers…", "Lade deine Karrieren…"),
    (
        "Race results: listening starts with your first league career.",
        "Rennergebnisse: Das Mithören beginnt mit deiner ersten Liga-Karriere.",
    ),
    ("Practice: left out this round.", "Training: in dieser Runde ausgelassen."),
    (
        "You have no seat this season; you get offers when it ends.",
        "Du hast diese Saison kein Cockpit; am Saisonende bekommst du Angebote.",
    ),
    ("You have no seat this season.", "Du hast diese Saison kein Cockpit."),
    ("DRIVER CHANGES THIS ROUND", "FAHRERWECHSEL IN DIESER RUNDE"),
    ("Simulate practice", "Training simulieren"),
    ("Simulate this round", "Diese Runde simulieren"),
    ("Leave it out", "Auslassen"),
    ("Take back the last round", "Letzte Runde zurücknehmen"),
    ("Count it anyway", "Trotzdem werten"),
    ("Count as sprint", "Als Sprint werten"),
    ("Discard", "Verwerfen"),
    ("SESSIONS THE GAME REPORTED, NOT COUNTED YET", "VOM SPIEL GEMELDETE SESSIONS, NOCH NICHT GEWERTET"),
    ("Everyone signed.", "Alle haben unterschrieben."),
    ("Start next season anyway", "Nächste Saison trotzdem starten"),
    ("In the new contract, ask for:", "Im neuen Vertrag verlangen:"),
    ("The career is complete", "Die Karriere ist abgeschlossen"),
    ("SEASONS", "SAISONS"),
    ("YOU", "DU"),
    ("TEAM", "TEAM"),
    ("CONTRACT", "VERTRAG"),
    ("Recognition", "Ansehen"),
    (
        "Valued highly; score more points than your team-mate to become the number one driver.",
        "Hoch geschätzt; hol mehr Punkte als dein Teamkollege, um Nummer-eins-Fahrer zu werden.",
    ),
    ("Fully developed", "Voll entwickelt"),
    ("ON ORDER", "IN ENTWICKLUNG"),
    ("THE CAR", "DAS AUTO"),
    // Multiplayer page and host server
    (
        "Play with friends who use the same addons: host a server on your PC, or join a friend's.",
        "Spiele mit Freunden, die dieselben Addons nutzen: Starte einen Server auf deinem PC oder tritt dem eines Freundes bei.",
    ),
    ("Enter your driver name first.", "Gib zuerst deinen Fahrernamen ein."),
    (
        "Enter your public IP address to get the host code.",
        "Gib deine öffentliche IP-Adresse ein, um den Host-Code zu bekommen.",
    ),
    ("Enter the server code and your driver name.", "Gib den Server-Code und deinen Fahrernamen ein."),
    ("Start server", "Server starten"),
    ("Stop server", "Server stoppen"),
    ("Connect", "Verbinden"),
    ("SERVER CODE", "SERVER-CODE"),
    ("Copy server code", "Server-Code kopieren"),
    ("Copy host code", "Host-Code kopieren"),
    ("Make host code", "Host-Code erstellen"),
    (
        "Your host server isn't running. Start it again to open rooms.",
        "Dein Host-Server läuft nicht. Starte ihn wieder, um Räume zu öffnen.",
    ),
    ("F1 25 isn't running", "F1 25 läuft nicht"),
    ("F1 25 is on the main menu", "F1 25 ist im Hauptmenü"),
    ("F1 25 is in a LAN lobby", "F1 25 ist in einer LAN-Lobby"),
    ("F1 25 is busy", "F1 25 ist beschäftigt"),
    ("Full", "Voll"),
    ("Your addons differ", "Deine Addons weichen ab"),
    ("Your addons match", "Deine Addons passen"),
    (
        "Mark the race over when everyone is back in the lobby.",
        "Beende das Rennen, wenn alle wieder in der Lobby sind.",
    ),
    (
        "Pick tracks and settings in F1 25's lobby, then start the race there.",
        "Wähle Strecken und Einstellungen in der Lobby von F1 25 und starte dort das Rennen.",
    ),
    ("The host picks the tracks and starts the race.", "Der Host wählt die Strecken und startet das Rennen."),
    ("Restart and play", "Neu starten und spielen"),
    (
        "F1 25 was not found. Locate it from the play bar first.",
        "F1 25 wurde nicht gefunden. Gib es zuerst über die Spielleiste an.",
    ),
    ("Create room", "Raum erstellen"),
    (
        "The room list didn't load. It tries again in a few seconds.",
        "Die Raumliste wurde nicht geladen. In ein paar Sekunden wird es erneut versucht.",
    ),
    (
        "Create one, or join a private room with its invite code.",
        "Erstelle einen oder tritt einem privaten Raum mit seinem Einladungscode bei.",
    ),
    ("No open rooms", "Keine offenen Räume"),
    ("OPEN ROOMS", "OFFENE RÄUME"),
    ("Join with code", "Mit Code beitreten"),
    ("View", "Ansehen"),
    ("New room", "Neuer Raum"),
    (
        "F1 25 opens its LAN lobby, where you pick the tracks and settings.",
        "F1 25 öffnet seine LAN-Lobby, dort wählst du Strecken und Einstellungen.",
    ),
    ("Back to rooms", "Zurück zu den Räumen"),
    ("The host's launcher hasn't listed its addons.", "Der Launcher des Hosts hat seine Addons noch nicht gemeldet."),
    ("You have every addon the host has on.", "Du hast jedes Addon, das der Host aktiv hat."),
    ("Retry", "Erneut versuchen"),
    ("Leave", "Verlassen"),
    ("Restart F1 25", "F1 25 neu starten"),
    ("Race over", "Rennen beendet"),
    ("Race started", "Rennen gestartet"),
    ("F1 25 is open. It restarts for this session.", "F1 25 ist offen. Es startet für diese Sitzung neu."),
    ("Listed", "Öffentlich"),
    ("Private", "Privat"),
    ("You're the host", "Du bist der Host"),
    ("Copied", "Kopiert"),
    ("Copy the invite code", "Einladungscode kopieren"),
    ("Nobody is here yet.", "Noch niemand da."),
    (
        "Your own career from 2016 to 2026, alone or with friends: qualifying and races count from the game by themselves, contracts move you between teams each winter.",
        "Deine eigene Karriere von 2016 bis 2026, allein oder mit Freunden: Qualifying und Rennen werden direkt aus dem Spiel gewertet, Verträge bringen dich jeden Winter zu neuen Teams.",
    ),
    ("New career", "Neue Karriere"),
    ("Back to the career", "Zurück zur Karriere"),
    ("Drivers", "Fahrer"),
    ("Constructors", "Konstrukteure"),
    ("Calendar", "Kalender"),
    ("Your driver", "Dein Fahrer"),
    ("R&D", "F&E"),
    ("Your career", "Deine Karriere"),
    (
        "Race with players who use the same addons. The host picks tracks in F1 25's LAN lobby.",
        "Fahre mit Spielern, die dieselben Addons nutzen. Der Host wählt die Strecken in der LAN-Lobby von F1 25.",
    ),
    ("Invite code", "Einladungscode"),
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
