//! The German texts of the career's contracts and driver (W3-2).

pub(super) const DE: &[(&str, &str)] = &[
    // Contract talks
    ("1 year", "1 Jahr"),
    ("{n} years", "{n} Jahre"),
    (
        "{salary} a season for {years}, with a number-one clause",
        "{salary} pro Saison für {years}, mit Nummer-eins-Klausel",
    ),
    ("{salary} a season for {years}", "{salary} pro Saison für {years}"),
    (
        "{salary} a season + {bonus} a point",
        "{salary} pro Saison + {bonus} pro Punkt",
    ),
    (
        "Agreed: {terms}. Sign it to make it count.",
        "Einig: {terms}. Unterschreib, damit es gilt.",
    ),
    (
        "They walked away from the talks and withdrew the offer.",
        "Sie haben die Verhandlungen abgebrochen und das Angebot zurückgezogen.",
    ),
    (
        "1 more attempt before they walk away.",
        "Noch 1 Versuch, bevor sie abbrechen.",
    ),
    (
        "{n} more attempts before they walk away.",
        "Noch {n} Versuche, bevor sie abbrechen.",
    ),
    (
        "They counter: {terms}. {left} Signing now takes their first offer.",
        "Gegenangebot: {terms}. {left} Unterschreibst du jetzt, gilt ihr erstes Angebot.",
    ),
    (
        "They won't talk any more; their first offer stands.",
        "Sie verhandeln nicht mehr; ihr erstes Angebot steht.",
    ),
    ("Ask for:", "Verlangen:"),
    ("Number-one clause", "Nummer-eins-Klausel"),
    ("Ask for these terms", "Diese Bedingungen verlangen"),
    ("Take their counter", "Gegenangebot annehmen"),
    (
        "You can ask for other terms: a team pays more the better you are against its drivers, the more you're \
         valued and the more offers you have. Ask too much and it counters; after {attempts} tries it walks away, \
         though your own team keeps its first offer. A number-one clause costs {cost}% of what a team would pay and \
         needs a rating at least your team-mate's.",
        "Du kannst andere Bedingungen verlangen: Ein Team zahlt mehr, je besser du gegen seine Fahrer bist, je mehr \
         du geschätzt wirst und je mehr Angebote du hast. Verlangst du zu viel, kommt ein Gegenangebot; nach \
         {attempts} Versuchen bricht es ab, nur dein eigenes Team bleibt bei seinem ersten Angebot. Eine \
         Nummer-eins-Klausel kostet {cost} % dessen, was ein Team zahlen würde, und verlangt eine Wertung \
         mindestens wie die deines Teamkollegen.",
    ),
    // Offers at the season's end
    ("Take {name}'s seat", "Cockpit von {name} übernehmen"),
    (
        "Signed during the season · {years} · {pay}",
        "Während der Saison unterschrieben · {years} · {pay}",
    ),
    (
        "Under contract: {years} more · {pay}",
        "Unter Vertrag: noch {years} · {pay}",
    ),
    (
        "Your team keeps you · {years} · {pay}",
        "Dein Team behält dich · {years} · {pay}",
    ),
    ("Offer · {years} · {pay}", "Angebot · {years} · {pay}"),
    (
        "The {year} season is over. Sign for {next}:",
        "Die Saison {year} ist vorbei. Unterschreib für {next}:",
    ),
    (
        "Offers follow how you did against your car and your team-mate, and how much your team values you: a team \
         that doesn't value you lets your contract run out. Under contract, only faster teams can buy you out; \
         longer deals keep you from the market.",
        "Die Angebote richten sich danach, wie du gegen dein Auto und deinen Teamkollegen abgeschnitten hast und \
         wie sehr dich dein Team schätzt: Ein Team, das dich nicht schätzt, lässt deinen Vertrag auslaufen. Unter \
         Vertrag können dich nur schnellere Teams herauskaufen; längere Verträge halten dich vom Markt fern.",
    ),
    (
        "Sign an offer and the {next} season starts.",
        "Unterschreib ein Angebot, und die Saison {next} beginnt.",
    ),
    (
        "Waiting for {names} to sign. The next season starts when everyone has.",
        "Warten auf die Unterschrift von {names}. Die nächste Saison beginnt, wenn alle unterschrieben haben.",
    ),
    ("Nothing extra", "Nichts extra"),
    ("More salary (+25%)", "Mehr Gehalt (+25 %)"),
    (
        "A say in the development (+20% resource points)",
        "Mitsprache bei der Entwicklung (+20 % Ressourcenpunkte)",
    ),
    // Mid-season approaches
    (
        "{team} would like to meet you.",
        "{team} möchte sich mit dir treffen.",
    ),
    ("Teams want you for {next}", "Teams wollen dich für {next}"),
    (
        "Meet a team in secret to hear its offer. Word of a meeting gets out {chance}% of the time, and your team \
         then thinks less of you.",
        "Triff ein Team heimlich, um sein Angebot zu hören. In {chance} % der Fälle sickert ein Treffen durch, und \
         dein Team schätzt dich dann weniger.",
    ),
    ("Not interested", "Kein Interesse"),
    ("Meet them in secret", "Heimlich treffen"),
    ("You met {team}.", "Du hast dich mit {team} getroffen."),
    (
        "{team} offers {years} from {next}: {pay}.",
        "{team} bietet {years} ab {next}: {pay}.",
    ),
    (
        "Word of the meeting got out, and your team isn't pleased.",
        "Das Treffen ist durchgesickert, und dein Team ist nicht erfreut.",
    ),
    (
        "You're under contract beyond {year}: signing is a buy-out, and your team will think less of you.",
        "Du stehst über {year} hinaus unter Vertrag: Unterschreibst du, kauft dich das Team heraus, und dein Team \
         wird dich weniger schätzen.",
    ),
    ("Sign early:", "Vorzeitig unterschreiben:"),
    (
        "You signed early with {team} for {next}.",
        "Du hast vorzeitig bei {team} für {next} unterschrieben.",
    ),
    ("You turned {team} down.", "Du hast {team} abgesagt."),
    // Your driver
    ("no team", "kein Team"),
    (
        "{team}'s number one driver: a tenth more resource points.",
        "Nummer-eins-Fahrer von {team}: ein Zehntel mehr Ressourcenpunkte.",
    ),
    (
        "{team} isn't happy: below {limit}, a team lets the contract run out.",
        "{team} ist unzufrieden: Unter {limit} lässt ein Team den Vertrag auslaufen.",
    ),
    (
        "Meet the team's goals and beat your team-mate; from {limit} on, and ahead of your team-mate, you're the \
         number one driver.",
        "Erfülle die Ziele des Teams und schlag deinen Teamkollegen; ab {limit} und vor deinem Teamkollegen bist du \
         der Nummer-eins-Fahrer.",
    ),
    (" · you asked for more salary", " · du hast mehr Gehalt verlangt"),
    (
        " · you have a say in the development (+20% resource points)",
        " · du redest bei der Entwicklung mit (+20 % Ressourcenpunkte)",
    ),
    (
        "Number-one clause until {year}.",
        "Nummer-eins-Klausel bis {year}.",
    ),
    (
        "Overall rating, weighed as the game weighs it: pace twice, racecraft once, experience and awareness a \
         quarter each.",
        "Gesamtwertung, gewichtet wie im Spiel: Tempo doppelt, Renngeschick einfach, Erfahrung und Übersicht je \
         ein Viertel.",
    ),
    ("Experience", "Erfahrung"),
    ("Racecraft", "Renngeschick"),
    ("Awareness", "Übersicht"),
    ("Pace", "Tempo"),
    (
        "Every session you drive moves them: experience with each one, pace with qualifying and practice against \
         your team-mate, racecraft with places gained, awareness with clean races (penalties and crashes cost it). \
         Simulated sessions only add experience.",
        "Jede Session, die du fährst, verändert sie: Erfahrung mit jeder, Tempo mit Qualifying und Training gegen \
         deinen Teamkollegen, Renngeschick mit gewonnenen Plätzen, Übersicht mit sauberen Rennen (Strafen und \
         Unfälle kosten sie). Simulierte Sessions bringen nur Erfahrung.",
    ),
    ("Focus", "Fokus"),
    (
        "Focus isn't part of the overall rating. It rises with each clean race (+{clean}) and falls when you retire \
         through your own fault (-{dnf}); in a simulated race, more focus makes a retirement less likely.",
        "Der Fokus zählt nicht zur Gesamtwertung. Er steigt mit jedem sauberen Rennen (+{clean}) und sinkt, wenn du \
         durch eigene Schuld ausfällst (-{dnf}); in einem simulierten Rennen macht mehr Fokus einen Ausfall \
         unwahrscheinlicher.",
    ),
    ("{value} of 100", "{value} von 100"),
    (
        "{salary} a season + {bonus} a point, until {year}",
        "{salary} pro Saison + {bonus} pro Punkt, bis {year}",
    ),
    (
        "Earned in the career: {money}",
        "In der Karriere verdient: {money}",
    ),
    // Creating the driver
    ("Create your driver", "Erstelle deinen Fahrer"),
    (
        "Where you're from and how you start. You can change it until your first session is counted.",
        "Woher du kommst und wie du startest. Du kannst es ändern, bis deine erste Session gewertet ist.",
    ),
    ("Pick a nationality", "Nationalität wählen"),
    ("Nationality", "Nationalität"),
    ("Start as", "Start als"),
    (
        "The archetype sets your starting rating and focus.",
        "Der Fahrertyp legt deine Startwertung und deinen Fokus fest.",
    ),
    ("Rookie · rated {rating}", "Neuling · Wertung {rating}"),
    ("Experienced · rated {rating}", "Erfahren · Wertung {rating}"),
    ("Veteran · rated {rating}", "Veteran · Wertung {rating}"),
    ("From {country}", "Aus {country}"),
    ("Started as a rookie", "Als Neuling gestartet"),
    (
        "Started as an experienced driver",
        "Als erfahrener Fahrer gestartet",
    ),
    ("Started as a veteran", "Als Veteran gestartet"),
    // The start and join forms
    ("Start a career", "Eine Karriere starten"),
    (
        "Just you: kept on this PC, no server needed. Every round is a Grand Prix in the game (F1 World > Grand \
         Prix) with that year's season box.",
        "Nur du: auf diesem PC gespeichert, kein Server nötig. Jede Runde ist ein Grand Prix im Spiel (F1 World > \
         Grand Prix) mit der Saisonbox des Jahres.",
    ),
    (
        "On your host server: your friends join with its code, everyone picks a seat, then you start the first \
         season. Every round is a LAN race.",
        "Auf deinem Host-Server: Deine Freunde treten mit seinem Code bei, alle wählen ein Cockpit, dann startest du \
         die erste Saison. Jede Runde ist ein LAN-Rennen.",
    ),
    ("From {year}", "Ab {year}"),
    ("To {year}", "Bis {year}"),
    ("Each year's points", "Punkte des jeweiligen Jahres"),
    ("Simulate missing circuits", "Fehlende Strecken simulieren"),
    ("Rookie start", "Start als Neuling"),
    ("Real driver changes", "Echte Fahrerwechsel"),
    ("Join a friend's career", "Der Karriere eines Freundes beitreten"),
    (
        "Type the career code your friend sent you (K-XXXXXX). You need to be signed in to their server.",
        "Gib den Karriere-Code ein, den dir dein Freund geschickt hat (K-XXXXXX). Du musst auf seinem Server \
         angemeldet sein.",
    ),
    ("Drive with friends", "Mit Freunden fahren"),
    (
        "A career with friends lives on a host server. Host one or join a friend's on the Multiplayer page, then \
         start or join it here.",
        "Eine Karriere mit Freunden liegt auf einem Host-Server. Starte einen oder tritt dem eines Freundes auf der \
         Mehrspieler-Seite bei, dann starte die Karriere hier oder tritt ihr bei.",
    ),
    // Nationalities (`league::driver::NATIONALITIES`; names German writes as English does are left out)
    ("Argentina", "Argentinien"),
    ("Australia", "Australien"),
    ("Austria", "Österreich"),
    ("Azerbaijan", "Aserbaidschan"),
    ("Belgium", "Belgien"),
    ("Bolivia", "Bolivien"),
    ("Brazil", "Brasilien"),
    ("Bulgaria", "Bulgarien"),
    ("Canada", "Kanada"),
    ("Colombia", "Kolumbien"),
    ("Croatia", "Kroatien"),
    ("Cyprus", "Zypern"),
    ("Czechia", "Tschechien"),
    ("Denmark", "Dänemark"),
    ("Dominican Republic", "Dominikanische Republik"),
    ("Egypt", "Ägypten"),
    ("Estonia", "Estland"),
    ("Finland", "Finnland"),
    ("France", "Frankreich"),
    ("Germany", "Deutschland"),
    ("Greece", "Griechenland"),
    ("Hong Kong", "Hongkong"),
    ("Hungary", "Ungarn"),
    ("Iceland", "Island"),
    ("India", "Indien"),
    ("Indonesia", "Indonesien"),
    ("Ireland", "Irland"),
    ("Italy", "Italien"),
    ("Jamaica", "Jamaika"),
    ("Jordan", "Jordanien"),
    ("Kazakhstan", "Kasachstan"),
    ("Kenya", "Kenia"),
    ("Latvia", "Lettland"),
    ("Lebanon", "Libanon"),
    ("Lithuania", "Litauen"),
    ("Luxembourg", "Luxemburg"),
    ("Mexico", "Mexiko"),
    ("Morocco", "Marokko"),
    ("Netherlands", "Niederlande"),
    ("New Zealand", "Neuseeland"),
    ("Norway", "Norwegen"),
    ("Philippines", "Philippinen"),
    ("Poland", "Polen"),
    ("Qatar", "Katar"),
    ("Romania", "Rumänien"),
    ("Russia", "Russland"),
    ("Saudi Arabia", "Saudi-Arabien"),
    ("Serbia", "Serbien"),
    ("Singapore", "Singapur"),
    ("Slovakia", "Slowakei"),
    ("Slovenia", "Slowenien"),
    ("South Africa", "Südafrika"),
    ("South Korea", "Südkorea"),
    ("Spain", "Spanien"),
    ("Sweden", "Schweden"),
    ("Switzerland", "Schweiz"),
    ("Turkey", "Türkei"),
    ("United Arab Emirates", "Vereinigte Arabische Emirate"),
    ("United Kingdom", "Vereinigtes Königreich"),
    ("United States", "Vereinigte Staaten"),
    ("Zimbabwe", "Simbabwe"),
];

#[cfg(test)]
mod tests {
    use swofty_core::league::{driver, market};

    use super::DE;

    /// Nationalities German writes as English does, so they have no entry.
    const SAME_IN_GERMAN: &[&str] = &[
        "Bahrain",
        "Chile",
        "China",
        "Costa Rica",
        "Ecuador",
        "Ghana",
        "Guatemala",
        "Israel",
        "Japan",
        "Kuwait",
        "Liechtenstein",
        "Malaysia",
        "Malta",
        "Monaco",
        "Nigeria",
        "Oman",
        "Pakistan",
        "Panama",
        "Paraguay",
        "Peru",
        "Portugal",
        "San Marino",
        "Sri Lanka",
        "Taiwan",
        "Thailand",
        "Ukraine",
        "Uruguay",
        "Venezuela",
        "Vietnam",
    ];

    fn german(en: &str) -> Option<&'static str> {
        DE.iter().find(|(k, _)| *k == en).map(|(_, de)| *de)
    }

    #[test]
    fn every_nationality_has_german() {
        for (code, name) in driver::NATIONALITIES {
            if SAME_IN_GERMAN.contains(name) {
                assert!(german(name).is_none(), "{name} needs no entry");
            } else {
                assert!(german(name).is_some(), "no German for {code} {name}");
            }
        }
        for name in SAME_IN_GERMAN {
            assert!(
                driver::NATIONALITIES.iter().any(|(_, n)| n == name),
                "{name} isn't a nationality"
            );
        }
    }

    #[test]
    fn every_market_template_has_german_with_its_placeholders() {
        for template in market::TEMPLATES {
            let de = german(template).unwrap_or_else(|| panic!("no German for {template}"));
            for part in template.split('{').skip(1) {
                let name = part.split('}').next().unwrap_or_default();
                assert!(
                    de.contains(&format!("{{{name}}}")),
                    "{template}: {{{name}}} missing in {de}"
                );
            }
        }
    }

    #[test]
    fn templates_keep_their_placeholders() {
        for (en, de) in DE {
            for part in en.split('{').skip(1) {
                let name = part.split('}').next().unwrap_or_default();
                assert!(
                    de.contains(&format!("{{{name}}}")),
                    "{en}: {{{name}}} missing in {de}"
                );
            }
        }
    }
}
