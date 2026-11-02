//! The German texts of the career's game sync (W3-3).

pub(super) const DE: &[(&str, &str)] = &[
    // The panel's head
    ("THE CAREER IN THE GAME", "DIE KARRIERE IM SPIEL"),
    ("Ready for round {round}", "Bereit für Runde {round}"),
    (
        "Out of date: prepare for round {round}",
        "Veraltet: für Runde {round} vorbereiten",
    ),
    (
        "Prepared for round {round} after the game closes",
        "Wird für Runde {round} vorbereitet, sobald das Spiel geschlossen ist",
    ),
    (
        "Off: the game is prepared for another career",
        "Aus: Das Spiel ist für eine andere Karriere vorbereitet",
    ),
    (
        "Off: the addon is turned off in the profile",
        "Aus: Das Addon ist im Profil ausgeschaltet",
    ),
    (
        "On, but this career wants nothing in the game now: restore the normal game",
        "An, aber diese Karriere braucht gerade nichts im Spiel: Stelle das normale Spiel wieder her",
    ),
    ("Preparing the game…", "Spiel wird vorbereitet …"),
    (
        "Restoring the normal game…",
        "Normales Spiel wird wiederhergestellt …",
    ),
    ("Looking at the game…", "Spiel wird geprüft …"),
    (
        "Prepare the game for round {round}",
        "Spiel für Runde {round} vorbereiten",
    ),
    ("Restore the normal game", "Normales Spiel wiederherstellen"),
    ("Details", "Details"),
    ("Less", "Weniger"),
    (
        "There is nothing to prepare: the game sync needs a running season and one of its options on.",
        "Es gibt nichts vorzubereiten: Der Spielabgleich braucht eine laufende Saison und eine seiner Optionen.",
    ),
    (
        "The profile on the play bar plays without addons: pick another one to prepare the game.",
        "Das Profil in der Spielleiste spielt ohne Addons: Wähle ein anderes, um das Spiel vorzubereiten.",
    ),
    (
        "Couldn't change the game: {error}",
        "Das Spiel konnte nicht geändert werden: {error}",
    ),
    // What the game shows
    (
        "The game sync works while a season runs: it prepares the game for each round.",
        "Der Spielabgleich arbeitet, solange eine Saison läuft: Er bereitet das Spiel für jede Runde vor.",
    ),
    (
        "The game shows you as {name} #{number} in {driver}'s {team}.",
        "Das Spiel zeigt dich als {name} #{number} im {team} von {driver}.",
    ),
    (
        "Your number #{own} belongs to another driver in {year}, so the game shows #{number}.",
        "Deine Nummer #{own} gehört {year} einem anderen Fahrer, darum zeigt das Spiel #{number}.",
    ),
    (
        "Your name isn't in the game: it shows {driver} in your seat.",
        "Dein Name ist nicht im Spiel: Es zeigt {driver} in deinem Cockpit.",
    ),
    ("{name} #{number} in the {team}", "{name} #{number} im {team}"),
    ("Also in the game: {players}.", "Außerdem im Spiel: {players}."),
    // Details
    (
        "Preparing writes a small generated addon (League career sync) into your addons and turns it on in \
         the profile on the play bar; the next Play builds the game with it. It follows the career round \
         by round by itself. Restoring takes it out again, and the game is as it was.",
        "Das Vorbereiten schreibt ein kleines erzeugtes Addon (League career sync) in deine Addons und schaltet \
         es im Profil der Spielleiste ein; das nächste Spielen baut das Spiel damit. Es folgt der Karriere Runde \
         für Runde von selbst. Das Wiederherstellen nimmt es wieder heraus, und das Spiel ist wie vorher.",
    ),
    (
        "Both players must prepare the same round; the lobby lists a mismatch as an addon difference.",
        "Beide Spieler müssen dieselbe Runde vorbereiten; die Lobby zeigt eine Abweichung als Addon-Unterschied.",
    ),
    (
        "Set Car performance to Realistic in the lobby (or the Grand Prix settings): equal performance \
         cancels the league's car pace.",
        "Stelle die Fahrzeugleistung in der Lobby (oder in den Grand-Prix-Einstellungen) auf Realistisch: \
         Gleiche Leistung hebt das Fahrzeugtempo der Liga auf.",
    ),
    (
        "CAR PACE IN THE GAME · ROUND {round}",
        "FAHRZEUGTEMPO IM SPIEL · RUNDE {round}",
    ),
    (
        "The game gets each team's league pace through its car's weight, the lever Classic Seasons uses \
         and the AI follows: each kilogram costs about {percent}% of a lap.",
        "Das Spiel bekommt das Ligatempo jedes Teams über das Gewicht seines Autos, den Hebel, den Classic \
         Seasons nutzt und dem die KI folgt: Jedes Kilogramm kostet etwa {percent} % einer Runde.",
    ),
    ("{mass} kg", "{mass} kg"),
    ("{gap} a lap", "{gap} pro Runde"),
    (
        "No team moved by a kilogram since the last round: the game reuses its last build of the cars.",
        "Kein Team hat sich seit der letzten Runde um ein Kilogramm bewegt: Das Spiel nutzt seinen letzten \
         Build der Autos wieder.",
    ),
    // The options
    ("WHAT GOES INTO THE GAME", "WAS INS SPIEL KOMMT"),
    ("Your names in the game", "Eure Namen im Spiel"),
    (
        "Each player's name, number and nationality replace the driver of their seat, in the menus and \
         on the timing screens. In a LAN race the game may still show a human car's network name.",
        "Name, Nummer und Nationalität jedes Spielers ersetzen den Fahrer seines Cockpits, in den Menüs und \
         auf den Zeitenbildschirmen. In einem LAN-Rennen zeigt das Spiel bei einem menschlichen Auto vielleicht \
         trotzdem den Netzwerknamen.",
    ),
    (
        "The league's car pace in the game",
        "Das Fahrzeugtempo der Liga im Spiel",
    ),
    (
        "Each team's car weighs what its league pace says, development included, so a team that develops \
         is faster in the game, its AI driver too. It needs equal car performance off.",
        "Das Auto jedes Teams wiegt, was sein Ligatempo sagt, Entwicklung eingeschlossen; ein Team, das \
         entwickelt, ist also im Spiel schneller, auch sein KI-Fahrer. Dafür muss gleiche Fahrzeugleistung aus sein.",
    ),
    ("Per-part feel (experimental)", "Gefühl pro Teil (experimentell)"),
    (
        "Aero parts add downforce and powertrain parts add power, with the car's pace kept the same. The \
         strengths are a guess until they are measured in the game; the AI team-mate follows only the pace.",
        "Aero-Teile geben mehr Abtrieb und Antriebsteile mehr Leistung, bei gleichem Tempo des Autos. Die \
         Stärken sind geschätzt, bis sie im Spiel gemessen sind; der KI-Teamkollege folgt nur dem Tempo.",
    ),
    (
        "Only the career's owner changes these.",
        "Nur der Besitzer der Karriere ändert diese.",
    ),
    // The AI driver market (W4-1)
    (
        "Each player's name, number and nationality replace the driver of their seat, in the menus \
         and on the timing screens. It stays on with the AI driver market: the seat's real driver may \
         race for another team.",
        "Name, Nummer und Nationalität jedes Spielers ersetzen den Fahrer seines Cockpits, in den Menüs und \
         auf den Zeitenbildschirmen. Mit dem KI-Fahrermarkt bleibt das an: Der echte Fahrer des Cockpits fährt \
         vielleicht für ein anderes Team.",
    ),
    ("AI DRIVER MARKET IN THE GAME", "KI-FAHRERMARKT IM SPIEL"),
    (
        "This season every AI driver races in their own seat; the market moves them at the season's end.",
        "In dieser Saison fährt jeder KI-Fahrer in seinem eigenen Cockpit; der Markt bewegt sie am \
         Saisonende.",
    ),
    (
        "The game shows each of these drivers in the seat of the year's real driver, with that driver's face.",
        "Das Spiel zeigt jeden dieser Fahrer im Cockpit des echten Fahrers des Jahres, mit dessen Gesicht.",
    ),
    (
        "{driver} #{number} in {seat}'s seat at {team}",
        "{driver} #{number} im Cockpit von {seat} bei {team}",
    ),
];
