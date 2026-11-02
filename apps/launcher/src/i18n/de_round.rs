//! The German texts of the career's round flow and multiplayer (W3-1).

pub(super) const DE: &[(&str, &str)] = &[
    // Session kinds and lists
    ("an unknown track", "unbekannte Strecke"),
    ("race", "Rennen"),
    ("sprint", "Sprint"),
    ("qualifying", "Qualifying"),
    ("practice", "Training"),
    ("{names} and {last}", "{names} und {last}"),
    ("another player", "ein anderer Spieler"),
    // Counting races
    (
        "This looks like round {round}'s {kind}, counted by {name}.",
        "Das sieht nach Runde {round} aus ({kind}), gewertet von {name}.",
    ),
    (
        "Couldn't listen for race results on {addr}: {error}. Another telemetry tool may use that port.",
        "Konnte auf {addr} nicht auf Rennergebnisse hören: {error}. Vielleicht nutzt ein anderes Telemetrie-Tool diesen Port.",
    ),
    (
        "{what} at {track} ended. Sign in to the host server to count it; it waits here.",
        "{what} ({track}) ist beendet. Melde dich beim Host-Server an, um es zu werten; es wartet hier.",
    ),
    (
        "{what} at {track} fits {careers}: pick the career above and count it below.",
        "{what} ({track}) passt zu {careers}: Wähle oben die Karriere und werte es unten.",
    ),
    (
        "{what} at {track} ended; it waits below until a career has a round for it.",
        "{what} ({track}) ist beendet; es wartet unten, bis eine Karriere eine Runde dafür hat.",
    ),
    (
        "{what} at {track} ended; it waits below until the career has a round for it.",
        "{what} ({track}) ist beendet; es wartet unten, bis die Karriere eine Runde dafür hat.",
    ),
    (
        "Counted in {career}: the {what} is counted.",
        "In {career} gewertet: Das {what} ist gewertet.",
    ),
    ("The {what} is counted.", "Das {what} ist gewertet."),
    (
        "The {what} wasn't counted: {why}",
        "Das {what} wurde nicht gewertet: {why}",
    ),
    (
        "The {what} at {track} wasn't counted: {why}.",
        "Das {what} ({track}) wurde nicht gewertet: {why}.",
    ),
    (
        "Still working on the last step…",
        "Der letzte Schritt läuft noch…",
    ),
    (
        "Sign in to the host server first.",
        "Melde dich zuerst beim Host-Server an.",
    ),
    (
        "Sign in to the host server to count this race; it waits here.",
        "Melde dich beim Host-Server an, um dieses Rennen zu werten; es wartet hier.",
    ),
    ("It was another {kind}", "Es war eine andere Session ({kind})"),
    ("{track}: {question}", "{track}: {question}"),
    ("Sprint", "Sprint"),
    ("Grand Prix", "Grand Prix"),
    ("Count practice", "Training werten"),
    ("Count qualifying", "Qualifying werten"),
    ("Count as race", "Als Rennen werten"),
    (", you P{place}", ", du P{place}"),
    (
        "{what} · {track} · {cars} cars{you}",
        "{what} · {track} · {cars} Autos{you}",
    ),
    (
        "Couldn't turn the telemetry on: {error}",
        "Die Telemetrie konnte nicht eingeschaltet werden: {error}",
    ),
    // Refusals
    (
        "This looks like a session another player counted moments ago.",
        "Das sieht nach einer Session aus, die ein anderer Spieler gerade gewertet hat.",
    ),
    (
        "Sprint or Grand Prix? The game didn't say which race of the weekend this was.",
        "Sprint oder Grand Prix? Das Spiel hat nicht gemeldet, welches Rennen des Wochenendes das war.",
    ),
    (
        "Only the next round can be counted, simulated, left out or practised.",
        "Nur die nächste Runde kann gewertet, simuliert, ausgelassen oder trainiert werden.",
    ),
    (
        "The career changed meanwhile; check and try again.",
        "Die Karriere hat sich inzwischen geändert; prüfe sie und versuche es erneut.",
    ),
    (
        "Another player's proposal is open; agree to it or decline it first.",
        "Ein Vorschlag eines anderen Spielers ist offen; stimme ihm zuerst zu oder lehne ihn ab.",
    ),
    (
        "There is no open proposal any more.",
        "Es gibt keinen offenen Vorschlag mehr.",
    ),
    (
        "You aren't asked in this vote.",
        "Du wirst bei dieser Abstimmung nicht gefragt.",
    ),
    (
        "Only the career's owner can do that.",
        "Das kann nur der Besitzer der Karriere.",
    ),
    (
        "You left this career; join it again with its code first.",
        "Du hast diese Karriere verlassen; tritt ihr zuerst wieder mit ihrem Code bei.",
    ),
    (
        "The host server doesn't know this yet; the host updates F1 Multiplayer.",
        "Der Host-Server kennt das noch nicht; der Host muss F1 Multiplayer aktualisieren.",
    ),
    (
        "Another PC claimed this driver name on the host server. Sign in from the PC you used before, or ask the host to release the name.",
        "Ein anderer PC hat diesen Fahrernamen auf dem Host-Server beansprucht. Melde dich von dem PC an, den du vorher benutzt hast, oder bitte den Host, den Namen freizugeben.",
    ),
    (
        "This driver name is claimed on the host server; update F1 Multiplayer to sign in.",
        "Dieser Fahrername ist auf dem Host-Server beansprucht; aktualisiere F1 Multiplayer, um dich anzumelden.",
    ),
    (
        "You own too many careers on this host server; delete one first.",
        "Dir gehören zu viele Karrieren auf diesem Host-Server; lösche zuerst eine.",
    ),
    (
        "That is too much data for the host server.",
        "Das sind zu viele Daten für den Host-Server.",
    ),
    (
        "The host server refused the race report: it has more cars or laps than a race has.",
        "Der Host-Server hat den Rennbericht abgelehnt: Er hat mehr Autos oder Runden, als ein Rennen hat.",
    ),
    (
        "Too many tries; wait {seconds} seconds and try again.",
        "Zu viele Versuche; warte {seconds} Sekunden und versuche es erneut.",
    ),
    // Taking back a round
    ("the race", "das Rennen"),
    ("the sprint", "der Sprint"),
    ("the practice of {names}", "das Training von {names}"),
    (
        "Round {round}, the {race}, goes back to the start.",
        "Runde {round}, {race}, wird auf den Anfang zurückgesetzt.",
    ),
    (
        "Round {round}, the {race}: {parts} go.",
        "Runde {round}, {race}: {parts} werden zurückgenommen.",
    ),
    (
        "The {n} simulated round(s) after it are counted again.",
        "Die {n} simulierte(n) Runde(n) danach werden neu gewertet.",
    ),
    (
        "Sessions {names} counted go too.",
        "Sessions, die {names} gewertet hat, werden auch zurückgenommen.",
    ),
    (
        "The season opens again: contracts signed for next season are cleared.",
        "Die Saison wird wieder geöffnet: Für die nächste Saison unterschriebene Verträge werden gelöscht.",
    ),
    ("Propose to take it back", "Zurücknehmen vorschlagen"),
    ("Take it back", "Zurücknehmen"),
    ("Keep it", "Behalten"),
    // Proposals and ready marks
    (
        "{name} proposes to simulate round {round}, the {race}.",
        "{name} schlägt vor, Runde {round}, {race}, zu simulieren.",
    ),
    (
        "{name} proposes to leave out round {round}, the {race}.",
        "{name} schlägt vor, Runde {round}, {race}, auszulassen.",
    ),
    (
        "{name} proposes to take back round {round}, the {race}.",
        "{name} schlägt vor, Runde {round}, {race}, zurückzunehmen.",
    ),
    (
        "You propose to simulate round {round}, the {race}.",
        "Du schlägst vor, Runde {round}, {race}, zu simulieren.",
    ),
    (
        "You propose to leave out round {round}, the {race}.",
        "Du schlägst vor, Runde {round}, {race}, auszulassen.",
    ),
    (
        "You propose to take back round {round}, the {race}.",
        "Du schlägst vor, Runde {round}, {race}, zurückzunehmen.",
    ),
    (
        "Waiting for {names} to agree.",
        "Warte auf die Zustimmung von {names}.",
    ),
    ("Agree", "Zustimmen"),
    ("Decline", "Ablehnen"),
    ("Do it now", "Jetzt ausführen"),
    (
        "Propose to simulate this round",
        "Simulieren dieser Runde vorschlagen",
    ),
    ("Propose to leave it out", "Auslassen vorschlagen"),
    (
        "In a shared career, everyone agrees before a round is simulated, left out or taken back.",
        "In einer geteilten Karriere stimmen alle zu, bevor eine Runde simuliert, ausgelassen oder zurückgenommen wird.",
    ),
    ("I'm ready for round {round}", "Ich bin bereit für Runde {round}"),
    ("{name} ready", "{name} bereit"),
    ("{name} not ready", "{name} nicht bereit"),
    // The next round's card
    (
        "qualifying, the sprint, then the race",
        "das Qualifying, den Sprint, dann das Rennen",
    ),
    ("qualifying, then the race", "das Qualifying, dann das Rennen"),
    ("the sprint, then the race", "den Sprint, dann das Rennen"),
    (
        "the race (qualifying is counted)",
        "das Rennen (das Qualifying ist gewertet)",
    ),
    ("a practice session, {what}", "eine Trainingssession, {what}"),
    (
        "Practice: grade {grade} ({grades}), {rp} resource points{best}.",
        "Training: Note {grade} ({grades}), {rp} Ressourcenpunkte{best}.",
    ),
    (
        "Practice (simulated): grade {grade} ({grades}), {rp} resource points{best}.",
        "Training (simuliert): Note {grade} ({grades}), {rp} Ressourcenpunkte{best}.",
    ),
    (", best lap {lap}", ", schnellste Runde {lap}"),
    (
        "Practice: drive a practice session first for resource points. It grades your laps (track acclimatisation), your best lap against your team-mate (qualifying pace) and a long run (tyre management). Or simulate it, for fewer points.",
        "Training: Fahre zuerst eine Trainingssession für Ressourcenpunkte. Bewertet werden deine Runden (Streckenkenntnis), deine schnellste Runde gegen deinen Teamkollegen (Qualifying-Tempo) und ein Longrun (Reifenmanagement). Oder simuliere es, für weniger Punkte.",
    ),
    (
        "{rp} resource points to spend on the car (R&D tab) · recognition {recognition}{status}",
        "{rp} Ressourcenpunkte für das Auto (Reiter F&E) · Ansehen {recognition}{status}",
    ),
    (" · number one driver", " · Nummer-eins-Fahrer"),
    (
        "You drive for {team}: pick {driver} in the game.",
        "Du fährst für {team}: Wähle im Spiel {driver}.",
    ),
    ("Team goal: P{place} or better", "Teamziel: P{place} oder besser"),
    ("team-mate {name}", "Teamkollege {name}"),
    ("rival {name}", "Rivale {name}"),
    (
        "In the game: F1 World > Grand Prix, the F1 {year} box, the {track} track, {distance}% race distance; drive {what}. Each session counts by itself when it ends, also when you let the game simulate it.",
        "Im Spiel: F1 World > Grand Prix, die Box F1 {year}, die Strecke {track}, {distance} % Renndistanz; fahre {what}. Jede Session wird gewertet, sobald sie endet, auch wenn du sie vom Spiel simulieren lässt.",
    ),
    (
        "In the game: open a LAN lobby (Multiplayer page), choose the {year} season box and the {track} track, {distance}% race distance, and drive {what}. Each session counts by itself when it ends.",
        "Im Spiel: Öffne eine LAN-Lobby (Seite Mehrspieler), wähle die Saison-Box {year} und die Strecke {track}, {distance} % Renndistanz, und fahre {what}. Jede Session wird gewertet, sobald sie endet.",
    ),
    (
        "NEXT · ROUND {round} OF {rounds}",
        "NÄCHSTE · RUNDE {round} VON {rounds}",
    ),
    // The banner and members
    ("picking seats", "Cockpitwahl"),
    ("round {round} of {rounds}", "Runde {round} von {rounds}"),
    ("contracts for next season", "Verträge für die nächste Saison"),
    ("complete", "abgeschlossen"),
    ("no seat yet", "noch kein Cockpit"),
    (" · {n}x champion", " · {n}x Meister"),
    (
        " · {salary} a season to {year}",
        " · {salary} pro Saison bis {year}",
    ),
    (" · left the career", " · hat die Karriere verlassen"),
    (
        "{name} #{number} · {team} · rating {rating}{status}{contract}{titles}",
        "{name} #{number} · {team} · Wertung {rating}{status}{contract}{titles}",
    ),
    (
        "{name} joined but has no driver yet",
        "{name} ist beigetreten, hat aber noch keinen Fahrer",
    ),
    (
        "{first} to {last} · {phase}{solo}",
        "{first} bis {last} · {phase}{solo}",
    ),
    (" · solo", " · solo"),
    ("Leave career", "Karriere verlassen"),
    ("Really leave?", "Wirklich verlassen?"),
    (
        "Your seat stays in the career, driven by the game's AI. You can join again with the code.",
        "Dein Cockpit bleibt in der Karriere, gefahren von der KI des Spiels. Mit dem Code kannst du wieder beitreten.",
    ),
    (
        "You left the career. Your seat stays, driven by the game's AI.",
        "Du hast die Karriere verlassen. Dein Cockpit bleibt, gefahren von der KI des Spiels.",
    ),
    ("Really remove {name}?", "{name} wirklich entfernen?"),
    (
        "{name} is out of the career.",
        "{name} ist nicht mehr in der Karriere.",
    ),
    // A career that didn't start
    (
        "The career {name} was made on the host server, but its season didn't start. Try again with the settings below, or delete it.",
        "Die Karriere {name} wurde auf dem Host-Server angelegt, aber ihre Saison ist nicht gestartet. Versuche es mit den Einstellungen unten erneut oder lösche sie.",
    ),
    ("Try again", "Nochmal versuchen"),
    ("Delete", "Löschen"),
    ("Really delete?", "Wirklich löschen?"),
    ("The career is deleted.", "Die Karriere ist gelöscht."),
    // The seat choice
    ("{driver} (now {name})", "{driver} (jetzt {name})"),
    (
        "You haven't picked a seat yet.",
        "Du hast noch kein Cockpit gewählt.",
    ),
    (
        "{name} hasn't picked a seat yet.",
        "{name} hat noch kein Cockpit gewählt.",
    ),
    (
        "Pick your seat for {year}: you take that driver's place for the season",
        "Wähle dein Cockpit für {year}: Du übernimmst für die Saison den Platz dieses Fahrers",
    ),
    (
        "Rookie start: the faster half of the grid opens up through contract offers later.",
        "Rookie-Start: Die schnellere Hälfte des Feldes öffnet sich später über Vertragsangebote.",
    ),
    (
        "Your seat is set. Start the season when you're ready.",
        "Dein Cockpit steht. Starte die Saison, wenn du bereit bist.",
    ),
    (
        "Everyone has a seat. Start the season when your friends are ready.",
        "Alle haben ein Cockpit. Starte die Saison, wenn deine Freunde bereit sind.",
    ),
    (
        "Waiting for everyone to pick a seat.",
        "Warte, bis alle ein Cockpit gewählt haben.",
    ),
    (
        "The career's owner starts the season once everyone has a seat.",
        "Der Besitzer der Karriere startet die Saison, sobald alle ein Cockpit haben.",
    ),
    ("Start the {year} season", "Saison {year} starten"),
    // The host server
    (
        "Server code (123 456) or host code",
        "Server-Code (123 456) oder Host-Code",
    ),
    ("Your driver name", "Dein Fahrername"),
    ("Server password (optional)", "Server-Passwort (optional)"),
    (
        "Your public IP or DNS name",
        "Deine öffentliche IP oder dein DNS-Name",
    ),
    ("Driver name to release", "Freizugebender Fahrername"),
    (
        "Couldn't start the server on port {port}: {error}",
        "Der Server konnte auf Port {port} nicht gestartet werden: {error}",
    ),
    (
        "Your router forwarded the port, but your internet connection sits behind another router or carrier NAT, so friends may not reach you. Ask your provider for a public IPv4 address, or let a friend with one host.",
        "Dein Router hat den Port weitergeleitet, aber deine Internetverbindung liegt hinter einem weiteren Router oder Carrier-NAT, daher erreichen dich Freunde vielleicht nicht. Frag deinen Anbieter nach einer öffentlichen IPv4-Adresse oder lass einen Freund mit einer hosten.",
    ),
    (
        "The router didn't forward the port automatically ({error}). Forward TCP and UDP port {port} to this PC in your router's settings.",
        "Der Router hat den Port nicht automatisch weitergeleitet ({error}). Leite TCP- und UDP-Port {port} in den Einstellungen deines Routers an diesen PC weiter.",
    ),
    ("The code directory: {error}", "Das Code-Verzeichnis: {error}"),
    (
        "Type the driver name to release.",
        "Gib den Fahrernamen ein, der freigegeben werden soll.",
    ),
    (
        "{name} is free again: the next sign-in with that name claims it.",
        "{name} ist wieder frei: Die nächste Anmeldung mit diesem Namen beansprucht ihn.",
    ),
    (
        "Nobody has claimed {name} on your server.",
        "Niemand hat {name} auf deinem Server beansprucht.",
    ),
    (
        "Couldn't release {name}: {error}",
        "{name} konnte nicht freigegeben werden: {error}",
    ),
    ("Host a server", "Server hosten"),
    (
        "Run the multiplayer server on this PC. It gets a six-digit server code; your friends join with it (or with the longer host code over the internet). Your router needs to forward one port (TCP and UDP); the launcher asks it to.",
        "Starte den Mehrspieler-Server auf diesem PC. Er bekommt einen sechsstelligen Server-Code; deine Freunde treten damit bei (oder über das Internet mit dem längeren Host-Code). Dein Router muss einen Port weiterleiten (TCP und UDP); der Launcher bittet ihn darum.",
    ),
    (
        "Without a password, anyone on the internet who finds your server can sign in to it. Set one and send it to your friends.",
        "Ohne Passwort kann sich jeder im Internet, der deinen Server findet, dort anmelden. Lege eins fest und schick es deinen Freunden.",
    ),
    ("Join a friend's server", "Dem Server eines Freundes beitreten"),
    (
        "Type the six-digit server code (or paste the host code) your friend sent you, with your driver name and the server's password if it has one.",
        "Gib den sechsstelligen Server-Code ein (oder füge den Host-Code ein), den dir dein Freund geschickt hat, dazu deinen Fahrernamen und das Passwort des Servers, falls er eins hat.",
    ),
    (
        "YOUR SERVER IS RUNNING · PORT {port}",
        "DEIN SERVER LÄUFT · PORT {port}",
    ),
    (
        "{rooms} room(s), {relays} relay(s)",
        "{rooms} Raum/Räume, {relays} Relay(s)",
    ),
    (
        "UDP port {port} is taken on this PC, so the server code can't be looked up; use the host code.",
        "UDP-Port {port} ist auf diesem PC belegt, daher kann der Server-Code nicht nachgeschlagen werden; nutze den Host-Code.",
    ),
    (
        "Your router forwards the server's port and the server has no password: anyone who finds it can sign in. Stop it and start it again with a password.",
        "Dein Router leitet den Port des Servers weiter und der Server hat kein Passwort: Jeder, der ihn findet, kann sich anmelden. Stoppe ihn und starte ihn mit einem Passwort neu.",
    ),
    ("Release a driver name", "Fahrernamen freigeben"),
    (
        "A driver who lost the PC they signed in from can claim their name again once you release it.",
        "Ein Fahrer, der den PC verloren hat, von dem er sich angemeldet hat, kann seinen Namen wieder beanspruchen, sobald du ihn freigibst.",
    ),
    (
        "Connected to the host server {server}",
        "Verbunden mit dem Host-Server {server}",
    ),
];

#[cfg(test)]
mod tests {
    use super::DE;

    /// Every refusal reason the launcher explains has its text in German.
    #[test]
    fn every_reason_text_has_german() {
        for reason in crate::career::REASONS {
            let text = crate::career::reason_text(reason).expect("a text");
            assert!(
                DE.iter().any(|(en, _)| *en == text),
                "no German for the {reason} text: {text}"
            );
        }
    }

    /// German templates keep every placeholder of their English text.
    #[test]
    fn german_templates_keep_their_placeholders() {
        for (en, de) in DE {
            let mut rest = *en;
            while let Some(start) = rest.find('{') {
                let Some(end) = rest[start..].find('}') else { break };
                let placeholder = &rest[start..start + end + 1];
                assert!(
                    de.contains(placeholder),
                    "{placeholder} is missing in the German of {en}"
                );
                rest = &rest[start + end + 1..];
            }
        }
    }
}
