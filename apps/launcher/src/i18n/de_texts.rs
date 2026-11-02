//! The German texts of the career's news, refusals and warnings (W4-2).

pub(super) const DE: &[(&str, &str)] = &[
    // The AI driver market's news (`swofty_core::league::aimarket::TEMPLATES`, W4-1)
    (
        "{driver} retires from Formula 1 after the {last} season.",
        "{driver} beendet nach der Saison {last} seine Formel-1-Karriere.",
    ),
    (
        "{team} doesn't keep {driver} for {year}.",
        "{team} behält {driver} für {year} nicht.",
    ),
    (
        "{driver} has no seat for {year}.",
        "{driver} hat für {year} kein Cockpit.",
    ),
    (
        "{driver} moves from {from} to {team} for {year}.",
        "{driver} wechselt für {year} von {from} zu {team}.",
    ),
    (
        "{team} gives {driver} a Formula 1 debut in {year}.",
        "{team} gibt {driver} {year} das Formel-1-Debüt.",
    ),
    (
        "{driver} is back in Formula 1 with {team} for {year}.",
        "{driver} kehrt {year} mit {team} in die Formel 1 zurück.",
    ),
    // Parts of qualifying in the league's texts (`swofty_core::league::texts::words`)
    ("short qualifying", "kurzes Qualifying"),
    ("one-shot qualifying", "Qualifying mit einem Versuch"),
    // The league's news, refusals and warnings (`swofty_core::league::texts::TEXTS`)
    // Votes (consent)
    (
        "{name} asked to simulate round {round}, the {race} first; agree to it or decline it before asking for something else",
        "{name} hat zuerst vorgeschlagen, Runde {round} ({race}) zu simulieren; stimm zu oder lehne ab, bevor du etwas anderes vorschlägst",
    ),
    (
        "{name} asked to leave out round {round}, the {race} first; agree to it or decline it before asking for something else",
        "{name} hat zuerst vorgeschlagen, Runde {round} ({race}) auszulassen; stimm zu oder lehne ab, bevor du etwas anderes vorschlägst",
    ),
    (
        "{name} asked to take back round {round}, the {race} first; agree to it or decline it before asking for something else",
        "{name} hat zuerst vorgeschlagen, Runde {round} ({race}) zurückzunehmen; stimm zu oder lehne ab, bevor du etwas anderes vorschlägst",
    ),
    (
        "{name} asks to {what}; it happens once everyone agrees.",
        "{name} schlägt vor: {what}. Es passiert, sobald alle zustimmen.",
    ),
    (
        "The proposal to {what} lapsed: the career moved on meanwhile.",
        "Der Vorschlag „{what}“ ist verfallen: Die Karriere ist inzwischen weiter.",
    ),
    ("no season is running", "es läuft keine Saison"),
    ("every round is counted", "alle Runden sind gewertet"),
    ("there is no open proposal to vote on", "es gibt keinen offenen Vorschlag zum Abstimmen"),
    ("you aren't asked in this vote", "du stimmst bei diesem Vorschlag nicht mit ab"),
    (
        "only the career's owner goes ahead without the others",
        "nur der Besitzer der Karriere macht ohne die anderen weiter",
    ),
    (
        "the proposal to {what} lapsed: the career moved on meanwhile",
        "der Vorschlag „{what}“ ist verfallen: Die Karriere ist inzwischen weiter",
    ),
    ("{name} withdrew the proposal to {what}.", "{name} hat den Vorschlag „{what}“ zurückgezogen."),
    (
        "{name} doesn't agree to {what}; it doesn't happen.",
        "{name} stimmt „{what}“ nicht zu; es passiert nicht.",
    ),
    (
        "{name} decided to {what} without waiting for {waiting}.",
        "{name} hat entschieden: {what}, ohne auf {waiting} zu warten.",
    ),
    ("nothing to take back", "nichts zum Zurücknehmen"),
    ("no round of this season to take back", "keine Runde dieser Saison zum Zurücknehmen"),
    ("simulate round {round}, the {race}", "Runde {round} ({race}) simulieren"),
    ("leave out round {round}, the {race}", "Runde {round} ({race}) auslassen"),
    ("take back round {round}, the {race}", "Runde {round} ({race}) zurücknehmen"),
    ("simulate round {round}", "Runde {round} simulieren"),
    ("leave out round {round}", "Runde {round} auslassen"),
    ("take back round {round}", "Runde {round} zurücknehmen"),
    // Driver profile
    ("join the career first", "tritt zuerst der Karriere bei"),
    ("the career is complete", "die Karriere ist abgeschlossen"),
    (
        "your driver is set once a session of yours is counted",
        "dein Fahrer steht fest, sobald eine deiner Sessions gewertet ist",
    ),
    ("{n} isn't a nationality to pick", "{n} ist keine wählbare Nationalität"),
    (
        "{name} races as an experienced driver from {country}, rated {rating}.",
        "{name} fährt als erfahrener Fahrer aus {country}, Wertung {rating}.",
    ),
    (
        "{name} races as a rookie from {country}, rated {rating}.",
        "{name} fährt als Rookie aus {country}, Wertung {rating}.",
    ),
    (
        "{name} races as a veteran from {country}, rated {rating}.",
        "{name} fährt als Veteran aus {country}, Wertung {rating}.",
    ),
    (
        "{name} races as an experienced driver, rated {rating}.",
        "{name} fährt als erfahrener Fahrer, Wertung {rating}.",
    ),
    ("{name} races as a rookie, rated {rating}.", "{name} fährt als Rookie, Wertung {rating}."),
    ("{name} races as a veteran, rated {rating}.", "{name} fährt als Veteran, Wertung {rating}."),
    ("{name} races from {country}.", "{name} fährt für {country}."),
    ("{name} races.", "{name} fährt."),
    // Inbox
    ("there is no mail {news}", "es gibt keine Nachricht {news}"),
    (
        "only the player it is for answers this mail",
        "nur der Spieler, an den sie geht, beantwortet diese Nachricht",
    ),
    ("this mail is answered already", "diese Nachricht ist schon beantwortet"),
    ("this mail has no such answer", "diese Nachricht hat keine solche Antwort"),
    // Market: approaches and negotiation
    ("teams approach drivers during the season", "Teams sprechen Fahrer während der Saison an"),
    ("{team} hasn't approached you", "{team} hat dich nicht angesprochen"),
    ("{team} isn't waiting to meet you any more", "{team} wartet nicht mehr auf ein Treffen mit dir"),
    (
        "{team} offers {years} from {next}, {salary} a season.",
        "{team} bietet {years} ab {next}, {salary} pro Saison.",
    ),
    (
        "{name} was seen meeting {team}; {own} isn't pleased.",
        "{name} wurde beim Treffen mit {team} gesehen; {own} ist nicht erfreut.",
    ),
    ("there is no team {slot} in {next} any more", "{next} gibt es kein Team {slot} mehr"),
    ("meet {team} first to hear their offer", "triff dich zuerst mit {team}, um das Angebot zu hören"),
    ("{team}'s approach is over", "das Interesse von {team} ist vorbei"),
    ("{team} walked away from the talks", "{team} hat die Verhandlungen abgebrochen"),
    (
        "{team} buys {name} out of the contract with {own}: {name} drives for {team} from {next} to {until}, for {salary} a season.",
        "{team} kauft {name} aus dem Vertrag mit {own} heraus: {name} fährt von {next} bis {until} für {team}, für {salary} pro Saison.",
    ),
    (
        "{team} buys {name} out of the contract with {own}: {name} drives for {team} from {next}, for {salary} a season.",
        "{team} kauft {name} aus dem Vertrag mit {own} heraus: {name} fährt ab {next} für {team}, für {salary} pro Saison.",
    ),
    (
        "{name} signs early with {team} for {next} to {until}, for {salary} a season.",
        "{name} unterschreibt vorzeitig bei {team} für {next} bis {until}, für {salary} pro Saison.",
    ),
    (
        "{name} signs early with {team} for {next}, for {salary} a season.",
        "{name} unterschreibt vorzeitig bei {team} für {next}, für {salary} pro Saison.",
    ),
    ("that approach is over", "dieses Interesse ist vorbei"),
    (
        "ask for a salary and a contract of 1 to 3 years",
        "verlange ein Gehalt und einen Vertrag über 1 bis 3 Jahre",
    ),
    ("that team made you no offer", "dieses Team hat dir kein Angebot gemacht"),
    (
        "contracts are negotiated during and at the end of a season",
        "Verträge werden während und am Ende einer Saison verhandelt",
    ),
    ("a contract that runs on isn't negotiated", "über einen laufenden Vertrag wird nicht verhandelt"),
    (
        "you agreed terms with {team} already; sign them",
        "du hast dich mit {team} schon geeinigt; unterschreib den Vertrag",
    ),
    (
        "{team} won't talk any more; their first offer stands",
        "{team} verhandelt nicht mehr; das erste Angebot steht",
    ),
    (
        "{team} agrees: {salary} a season for {years}. Sign it to make it count.",
        "{team} willigt ein: {salary} pro Saison für {years}. Unterschreib, damit es gilt.",
    ),
    (
        "{team} agrees: {salary} a season for {years} with a number-one clause. Sign it to make it count.",
        "{team} willigt ein: {salary} pro Saison für {years} mit Nummer-eins-Klausel. Unterschreib, damit es gilt.",
    ),
    (
        "{team} counters: {salary} a season for {years} with a number-one clause.",
        "{team} macht ein Gegenangebot: {salary} pro Saison für {years} mit Nummer-eins-Klausel.",
    ),
    (
        "{team} counters: {salary} a season for {years}, without a number-one clause.",
        "{team} macht ein Gegenangebot: {salary} pro Saison für {years}, ohne Nummer-eins-Klausel.",
    ),
    (
        "{team} counters: {salary} a season for {years}.",
        "{team} macht ein Gegenangebot: {salary} pro Saison für {years}.",
    ),
    (
        "{attempts} more attempts before they walk away.",
        "Noch {attempts} Versuche, bevor sie die Verhandlungen abbrechen.",
    ),
    ("{counter} {left}", "{counter} {left}"),
    (
        "{team} ends the talks; their first offer stands.",
        "{team} beendet die Verhandlungen; das erste Angebot steht.",
    ),
    (
        "{team} walks away from the talks and withdraws the offer.",
        "{team} bricht die Verhandlungen ab und zieht das Angebot zurück.",
    ),
    ("{years} years", "{years} Jahre"),
    // Start, seats and the season
    (
        "{driver} drives {out}'s {team} (the game shows {out})",
        "{driver} fährt den {team} von {out} (das Spiel zeigt {out})",
    ),
    (
        "the career runs from {first} to {last}; pick years in between",
        "die Karriere läuft von {first} bis {last}; wähle Jahre dazwischen",
    ),
    ("the race distance is a percentage from 5 to 100", "die Renndistanz ist ein Prozentwert von 5 bis 100"),
    (
        "{name} started the career, {first_year} to {last_year}.",
        "{name} hat die Karriere gestartet, {first_year} bis {last_year}.",
    ),
    ("{name} is back in the career.", "{name} ist zurück in der Karriere."),
    ("this career has no free seat", "diese Karriere hat kein freies Cockpit"),
    ("another player races with number {n}", "ein anderer Spieler fährt mit der Nummer {n}"),
    ("pick a race number from 1 to 99", "wähle eine Startnummer von 1 bis 99"),
    (
        "{name} joined the career and gets offers when the season ends.",
        "{name} ist der Karriere beigetreten und bekommt Angebote, wenn die Saison endet.",
    ),
    (
        "seats are picked before the first season; later, sign an offer",
        "Cockpits werden vor der ersten Saison gewählt; später unterschreibst du ein Angebot",
    ),
    ("no team {slot} in {year}", "{year} gibt es kein Team {slot}"),
    (
        "as a rookie you start in the slower half of the grid; {team} is too fast",
        "als Rookie startest du in der langsameren Hälfte des Felds; {team} ist zu schnell",
    ),
    (
        "only the career's owner starts the first season",
        "nur der Besitzer der Karriere startet die erste Saison",
    ),
    ("the career has started already", "die Karriere hat schon begonnen"),
    ("{name} hasn't picked a seat yet", "{name} hat noch kein Cockpit gewählt"),
    ("The {year} season begins.", "Die Saison {year} beginnt."),
    (
        "{name}'s rival this season: {rival} ({team}).",
        "Rivale von {name} in dieser Saison: {rival} ({team}).",
    ),
    ("{name} drives for {team} (#{number}).", "{name} fährt für {team} (#{number})."),
    // Counting sessions
    ("round {n} is counted already", "Runde {n} ist schon gewertet"),
    (
        "only the next round ({next}) can be counted, simulated, skipped or practised",
        "nur die nächste Runde ({next}) kann gewertet, simuliert, ausgelassen oder trainiert werden",
    ),
    ("{year} has no round {n}", "{year} hat keine Runde {n}"),
    (
        "this looks like the {session} of round {round}, the {race}, that {by} counted moments ago; if it is another session, count it anyway",
        "das sieht nach der Session „{session}“ von Runde {round} ({race}) aus, die {by} gerade gewertet hat; ist es eine andere Session, werte sie trotzdem",
    ),
    (
        "this looks like the {session} of round {round}, the {race}, that another player counted moments ago; if it is another session, count it anyway",
        "das sieht nach der Session „{session}“ von Runde {round} ({race}) aus, die ein anderer Spieler gerade gewertet hat; ist es eine andere Session, werte sie trotzdem",
    ),
    ("this race is counted already", "dieses Rennen ist schon gewertet"),
    ("this sprint is counted already", "dieser Sprint ist schon gewertet"),
    ("this qualifying is counted already", "dieses Qualifying ist schon gewertet"),
    ("this practice is counted already", "dieses Training ist schon gewertet"),
    (
        "this race was at {track}, but round {n} is the {race}",
        "dieses Rennen war in {track}, aber Runde {n} ist {race}",
    ),
    (
        "this race was at another track, but round {n} is the {race}",
        "dieses Rennen war auf einer anderen Strecke, aber Runde {n} ist {race}",
    ),
    (
        "a sprint shootout isn't counted; the sprint and the race are",
        "ein Sprint-Shootout wird nicht gewertet; der Sprint und das Rennen schon",
    ),
    (
        "the game didn't say whether this was the sprint or the Grand Prix of round {n}; count it as one of them",
        "das Spiel hat nicht gesagt, ob das der Sprint oder der Grand Prix von Runde {n} war; werte es als eins von beiden",
    ),
    ("round {n} has no sprint", "Runde {n} hat keinen Sprint"),
    ("the sprint of round {n} is counted already", "der Sprint von Runde {n} ist schon gewertet"),
    (
        "a later part of this qualifying is counted already",
        "ein späterer Teil dieses Qualifyings ist schon gewertet",
    ),
    (
        "none of the cars in this race belong to this season's grid",
        "keines der Autos in diesem Rennen gehört zum Feld dieser Saison",
    ),
    ("Qualifying updated ({session}).", "Qualifying aktualisiert ({session})."),
    // Milestones and results
    (
        "{name} takes the first pole position of their career!",
        "{name} holt die erste Pole-Position der Karriere!",
    ),
    ("{name}'s pole position number {k}.", "Pole-Position Nummer {k} für {name}."),
    ("{name} wins a Grand Prix for the first time!", "{name} gewinnt zum ersten Mal einen Grand Prix!"),
    ("{name}'s Grand Prix win number {k}!", "Grand-Prix-Sieg Nummer {k} für {name}!"),
    ("{name} stands on the podium for the first time.", "{name} steht zum ersten Mal auf dem Podium."),
    ("{name} scores the first points of their career.", "{name} holt die ersten Punkte der Karriere."),
    (
        "{name} meets the team's goal (P{goal} or better).",
        "{name} erfüllt das Ziel des Teams (P{goal} oder besser).",
    ),
    ("{name} misses the team's goal of P{goal}.", "{name} verfehlt das Ziel des Teams von P{goal}."),
    ("(simulated: the game doesn't have this circuit)", "(simuliert: Das Spiel hat diese Strecke nicht)"),
    ("(simulated)", "(simuliert)"),
    ("Round {n}, the {race}, is left out.", "Runde {n} ({race}) wird ausgelassen."),
    ("Round {n}, the {race}, was taken back.", "Runde {n} ({race}) wurde zurückgenommen."),
    (
        "From the {race}, {driver} stands in for {out} at {team} until {out} is back at the {back}.",
        "Ab {race} vertritt {driver} {out} bei {team}, bis {out} bei {back} zurück ist.",
    ),
    (
        "From the {race}, {driver} stands in for {out} at {team}.",
        "Ab {race} vertritt {driver} {out} bei {team}.",
    ),
    (
        "From the {race}, {driver} drives for {team} in place of {out} for a while.",
        "Ab {race} fährt {driver} eine Weile für {team} anstelle von {out}.",
    ),
    (
        "From the {race}, {driver} drives for {team} in place of {out}.",
        "Ab {race} fährt {driver} für {team} anstelle von {out}.",
    ),
    ("From the {race}, {driver} replaces {out} at {team}.", "Ab {race} ersetzt {driver} {out} bei {team}."),
    (
        "The game's season box still shows its own line-up; the career counts each car for the driver of that seat.",
        "Die Saisonbox des Spiels zeigt weiter ihre eigene Aufstellung; die Karriere wertet jedes Auto für den Fahrer dieses Cockpits.",
    ),
    (
        "{driver} is the {year} world champion with {points} points; {team} win the constructors' title.",
        "{driver} ist Weltmeister {year} mit {points} Punkten; {team} gewinnt die Konstrukteurswertung.",
    ),
    (
        "{name} earned {season} this season ({career} in the career).",
        "{name} hat in dieser Saison {season} verdient ({career} in der Karriere).",
    ),
    (
        "{name} against team-mate {mate}: qualifying {q1}:{q2}, races {r1}:{r2}.",
        "{name} gegen Teamkollege {mate}: Qualifying {q1}:{q2}, Rennen {r1}:{r2}.",
    ),
    (
        "{name} won the rivalry with {rival} {r1}:{r2}.",
        "{name} hat das Duell mit {rival} {r1}:{r2} gewonnen.",
    ),
    (
        "{name} lost the rivalry with {rival} {r1}:{r2}.",
        "{name} hat das Duell mit {rival} {r1}:{r2} verloren.",
    ),
    (
        "{name} drew the rivalry with {rival} {r1}:{r2}.",
        "{name} und {rival} trennen sich im Duell {r1}:{r2} unentschieden.",
    ),
    ("The career is complete.", "Die Karriere ist abgeschlossen."),
    // Contracts
    ("Contracts for {year}:", "Verträge für {year}:"),
    ("{team} (under contract)", "{team} (unter Vertrag)"),
    ("{team} (1 year, {salary} a season)", "{team} (1 Jahr, {salary} pro Saison)"),
    ("{team} ({years} years, {salary} a season)", "{team} ({years} Jahre, {salary} pro Saison)"),
    ("{name} has offers from {teams}.", "{name} hat Angebote von {teams}."),
    ("{team} doesn't renew the contract.", "{team} verlängert den Vertrag nicht."),
    ("contracts are signed when a season is over", "Verträge werden am Ende einer Saison unterschrieben"),
    (
        "{player} stays with {team} for {next}, under contract until {until}.",
        "{player} bleibt {next} bei {team}, unter Vertrag bis {until}.",
    ),
    (
        "{player} signs with {team} for {next} to {until}, for {salary} a season.",
        "{player} unterschreibt bei {team} für {next} bis {until}, für {salary} pro Saison.",
    ),
    (
        "{player} signs with {team} for {next}, for {salary}.",
        "{player} unterschreibt bei {team} für {next}, für {salary}.",
    ),
    (
        "only the career's owner moves on before everyone signed",
        "nur der Besitzer der Karriere macht weiter, bevor alle unterschrieben haben",
    ),
    ("the season isn't over yet", "die Saison ist noch nicht vorbei"),
    // Players and options
    ("that player isn't in this career", "dieser Spieler ist nicht in dieser Karriere"),
    (
        "only the career's owner removes another player",
        "nur der Besitzer der Karriere entfernt einen anderen Spieler",
    ),
    (
        "the career's owner can't leave their own career",
        "der Besitzer der Karriere kann seine eigene Karriere nicht verlassen",
    ),
    ("{name} left the career.", "{name} hat die Karriere verlassen."),
    ("{name} has left the career already", "{name} hat die Karriere schon verlassen"),
    (
        "{name} left the career; the game's AI drives their seat for the rest of the season.",
        "{name} hat die Karriere verlassen; die KI des Spiels fährt das Cockpit für den Rest der Saison.",
    ),
    (
        "only the career's owner changes the career options",
        "nur der Besitzer der Karriere ändert die Karriere-Optionen",
    ),
    (
        "equal performance is off while the league's car pace is in the game",
        "gleiche Fahrzeugleistung ist aus, solange das Tempo der Liga-Autos im Spiel ist",
    ),
    (
        "the players' names stay in the game while AI drivers move between teams",
        "die Namen der Spieler bleiben im Spiel, solange KI-Fahrer zwischen Teams wechseln",
    ),
    ("{name} changed the career options.", "{name} hat die Karriere-Optionen geändert."),
    (
        "this career was saved by a newer F1 Multiplayer; update it first",
        "diese Karriere wurde von einem neueren F1 Multiplayer gespeichert; aktualisiere ihn zuerst",
    ),
    (
        "you left this career; join it again first",
        "du hast diese Karriere verlassen; tritt ihr zuerst wieder bei",
    ),
    ("this career has started already", "diese Karriere hat schon begonnen"),
    ("{key} doesn't drive for {team}", "{key} fährt nicht für {team}"),
    ("another player has that seat", "ein anderer Spieler hat dieses Cockpit"),
    ("both seats at {team} are taken", "beide Cockpits bei {team} sind vergeben"),
    ("no season data for {year}", "keine Saisondaten für {year}"),
    ("{name}'s seat doesn't exist in {year}", "das Cockpit von {name} gibt es {year} nicht"),
    // Matching the game's cars
    ("{name}'s seat was driven by the game's AI", "das Cockpit von {name} fuhr die KI des Spiels"),
    ("matched {shown} to {name} by elimination", "{shown} per Ausschluss {name} zugeordnet"),
    (
        "the game showed the real drivers in {by_seat} seats the AI market changed; they count for the league's drivers of those seats (prepare the game sync to see them in the game)",
        "das Spiel zeigte in {by_seat} Cockpits, die der KI-Fahrermarkt geändert hat, die echten Fahrer; sie zählen für die Liga-Fahrer dieser Cockpits (bereite den Spielabgleich vor, um sie im Spiel zu sehen)",
    ),
    (
        "P{position} {name} (#{number}) matched no driver of {year} and isn't counted",
        "P{position} {name} (#{number}) passt zu keinem Fahrer von {year} und wird nicht gewertet",
    ),
    ("{name} P{position}", "{name} P{position}"),
    ("{name} no time", "{name} ohne Zeit"),
    ("{name} out", "{name} ausgefallen"),
    ("Round {n}, {race} qualifying: {winner} on pole", "Runde {n}, {race}, Qualifying: {winner} auf Pole"),
    ("Round {n}, {race} qualifying: no pole time", "Runde {n}, {race}, Qualifying: keine Pole-Zeit"),
    ("Round {n}, {race} sprint: {winner} wins", "Runde {n}, {race}, Sprint: {winner} gewinnt"),
    ("Round {n}, {race} sprint: no winner classified", "Runde {n}, {race}, Sprint: kein Sieger gewertet"),
    ("Round {n}, {race}: {winner} wins", "Runde {n}, {race}: {winner} gewinnt"),
    ("Round {n}, {race}: no winner classified", "Runde {n}, {race}: kein Sieger gewertet"),
    ("{result}.", "{result}."),
    ("{result}; {players}.", "{result}; {players}."),
    // Objectives and practice
    ("{name} {met} of {count}", "{name} {met} von {count}"),
    ("Objectives: {counts}.", "Ziele: {counts}."),
    ("you have no seat this season", "du hast in dieser Saison kein Cockpit"),
    ("your practice of round {n} is counted already", "dein Training von Runde {n} ist schon gewertet"),
    ("you drove no lap in this practice session", "du bist in diesem Training keine Runde gefahren"),
    (
        "Round {n}, {race} practice: {name} grade {grade}, {rp} resource points.",
        "Runde {n}, {race}, Training: {name} Note {grade}, {rp} Ressourcenpunkte.",
    ),
    (
        "Round {n}, {race} practice: {name} grade {grade}, {rp} resource points, best lap {lap}.",
        "Runde {n}, {race}, Training: {name} Note {grade}, {rp} Ressourcenpunkte, beste Runde {lap}.",
    ),
    (
        "Round {n}, {race} practice (simulated): {name} grade {grade}, {rp} resource points.",
        "Runde {n}, {race}, Training (simuliert): {name} Note {grade}, {rp} Ressourcenpunkte.",
    ),
    (
        "Round {n}, {race} practice (simulated): {name} grade {grade}, {rp} resource points, best lap {lap}.",
        "Runde {n}, {race}, Training (simuliert): {name} Note {grade}, {rp} Ressourcenpunkte, beste Runde {lap}.",
    ),
    ("{name} is now {team}'s number one driver.", "{name} ist jetzt Nummer-eins-Fahrer von {team}."),
    (
        "{name} is no longer {team}'s number one driver.",
        "{name} ist nicht mehr Nummer-eins-Fahrer von {team}.",
    ),
    // Development
    (
        "{team}'s new {node} ({part}) fails its tests and gains nothing; {back} of its {cost} resource points go back to {payer}.",
        "{node} ({part}) von {team} fällt bei den Tests durch und bringt nichts; {back} der {cost} Ressourcenpunkte gehen an {payer} zurück.",
    ),
    ("{team}'s {node} ({part})", "{node} ({part}) von {team}"),
    ("On the car for the {race}: {fitted}.", "Am Auto für {race}: {fitted}."),
    ("parts are developed while a season runs", "Teile werden während einer Saison entwickelt"),
    ("the season is over", "die Saison ist vorbei"),
    (
        "{part}'s main branch is developed as far as it goes this season",
        "der Hauptzweig von {part} ist für diese Saison ausgereizt",
    ),
    ("{part} has no development node {id}", "{part} hat keinen Entwicklungsschritt {id}"),
    ("{node} needs {needs} first", "{node} braucht zuerst {needs}"),
    ("{node} is ordered already", "{node} ist schon in Auftrag"),
    (
        "{node} ({part}) costs {cost} resource points; you have {have}",
        "{node} ({part}) kostet {cost} Ressourcenpunkte; du hast {have}",
    ),
    (
        "{node} would take the team over the cost cap: it has spent {spent} of {cap} resource points this season",
        "{node} würde das Team über die Budgetgrenze bringen: Es hat in dieser Saison {spent} von {cap} Ressourcenpunkten ausgegeben",
    ),
    (
        "{node} takes {rounds} rounds to build; it wouldn't be ready this season",
        "{node} braucht {rounds} Runden in der Fertigung; es wäre diese Saison nicht mehr fertig",
    ),
    (
        "{name} has {team} develop {part} level {level} ({node}); it's on the car from round {ready}, the {race}.",
        "{name} lässt {team} {part} Stufe {level} entwickeln ({node}); ab Runde {ready} ({race}) ist es am Auto.",
    ),
    // Weekend
    (
        "a later part of this sprint shootout is counted already",
        "ein späterer Teil dieses Sprint-Shootouts ist schon gewertet",
    ),
    (
        "none of the cars in this sprint shootout belong to this season's grid",
        "keines der Autos in diesem Sprint-Shootout gehört zum Feld dieser Saison",
    ),
    ("sprint shootout counted", "Sprint-Shootout gewertet"),
    (
        "Round {n}, {race} sprint shootout: {winner} on sprint pole",
        "Runde {n}, {race}, Sprint-Shootout: {winner} auf der Sprint-Pole",
    ),
    (
        "Round {n}, {race} sprint shootout updated: {winner} on sprint pole",
        "Runde {n}, {race}, Sprint-Shootout aktualisiert: {winner} auf der Sprint-Pole",
    ),
    (
        "Round {n}, {race} sprint shootout: no sprint pole time",
        "Runde {n}, {race}, Sprint-Shootout: keine Sprint-Pole-Zeit",
    ),
    (
        "Round {n}, {race} sprint shootout updated: no sprint pole time",
        "Runde {n}, {race}, Sprint-Shootout aktualisiert: keine Sprint-Pole-Zeit",
    ),
    ("the AI difficulty goes from 0 to {max}", "die KI-Stärke geht von 0 bis {max}"),
    (
        "car damage is off (0), reduced (1), standard (2) or simulation (3)",
        "Fahrzeugschaden ist aus (0), reduziert (1), Standard (2) oder Simulation (3)",
    ),
    (
        "the safety car is off (0), reduced (1), standard (2) or increased (3)",
        "das Safety-Car ist aus (0), reduziert (1), Standard (2) oder erhöht (3)",
    ),
    ("the weather goes from clear (0) to storm (5)", "das Wetter geht von klar (0) bis Sturm (5)"),
    ("the game has no such rule set", "das Spiel hat kein solches Regelwerk"),
    (
        "{label}: the game ran {ran}, the agreed rules say {agreed}",
        "{label}: Das Spiel lief mit {ran}, die vereinbarten Regeln sagen {agreed}",
    ),
];

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use swofty_core::league::{aimarket, driver, inbox, market, texts};

    use super::super::SLICES;

    fn german(text: &str) -> Option<&'static str> {
        SLICES
            .iter()
            .flat_map(|s| s.iter())
            .find(|(en, _)| *en == text)
            .map(|(_, de)| *de)
    }

    /// The placeholders of a template.
    fn placeholders(template: &str) -> BTreeSet<&str> {
        template
            .split('{')
            .skip(1)
            .filter_map(|part| part.split_once('}').map(|(name, _)| name))
            .collect()
    }

    #[test]
    fn every_ai_market_template_has_german_with_its_placeholders() {
        for template in aimarket::TEMPLATES {
            let de = german(template).unwrap_or_else(|| panic!("no German for {template}"));
            assert_eq!(placeholders(template), placeholders(de), "{template} → {de}");
        }
    }

    /// Every template of the league's texts (news, refusals, warnings, the inbox's and the
    /// markets' news) has German with the same placeholders.
    #[test]
    fn every_news_template_has_german() {
        let lists: [(&str, &[&str]); 4] = [
            ("texts::TEXTS", texts::TEXTS),
            ("inbox::TEMPLATES", inbox::TEMPLATES),
            ("market::TEMPLATES", market::TEMPLATES),
            ("aimarket::TEMPLATES", aimarket::TEMPLATES),
        ];
        let mut missing = Vec::new();
        for (list, templates) in lists {
            for template in templates {
                match german(template) {
                    None => missing.push(format!("{list}: {template}")),
                    Some(de) => {
                        assert_eq!(placeholders(template), placeholders(de), "{template} → {de}");
                        // Only a template of placeholders and codes ("P{position}") reads the same.
                        let words = template.split('{').any(|p| {
                            p.split_once('}')
                                .map_or(p, |(_, rest)| rest)
                                .chars()
                                .any(char::is_lowercase)
                        });
                        assert!(!words || de != *template, "{template} isn't translated");
                    }
                }
            }
        }
        assert!(missing.is_empty(), "no German for:\n{}", missing.join("\n"));
    }

    /// The words the league puts in its texts' translated placeholders (parts, sessions,
    /// countries, development nodes, the weekend check's settings) have German too.
    #[test]
    fn every_translated_word_has_german() {
        // Country names that read the same in German have no entry; codes like "Q3" neither.
        let countries: Vec<&str> = driver::NATIONALITIES.iter().map(|(_, name)| *name).collect();
        let missing: Vec<&str> = texts::words()
            .into_iter()
            .filter(|w| german(w).is_none() && !countries.contains(w) && w.chars().any(char::is_lowercase))
            .collect();
        assert!(missing.is_empty(), "no German for: {missing:?}");
    }
}
