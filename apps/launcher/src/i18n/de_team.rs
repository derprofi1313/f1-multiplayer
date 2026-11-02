//! The German texts of the career's team development and inbox (W3-4).

pub(super) const DE: &[(&str, &str)] = &[
    // The tabs.
    ("Inbox", "Posteingang"),
    // R&D: the parts and their development trees.
    ("Aerodynamics", "Aerodynamik"),
    ("Chassis", "Chassis"),
    ("Powertrain", "Antrieb"),
    ("Durability", "Haltbarkeit"),
    ("Front wing endplates", "Frontflügel-Endplatten"),
    ("Floor edge", "Unterbodenkante"),
    ("Rear wing flap", "Heckflügelklappe"),
    ("Diffuser strakes", "Diffusor-Leitbleche"),
    ("Bargeboards", "Bargeboards"),
    ("High-downforce front wing", "Frontflügel mit viel Abtrieb"),
    ("Radical floor", "Radikaler Unterboden"),
    ("Low-drag rear wing", "Heckflügel mit wenig Luftwiderstand"),
    ("Double diffuser", "Doppeldiffusor"),
    ("New aero concept", "Neues Aerokonzept"),
    ("Suspension geometry", "Aufhängungsgeometrie"),
    ("Brake cooling", "Bremsenkühlung"),
    ("Weight saving", "Gewichtsersparnis"),
    ("Stiffer monocoque", "Steiferes Monocoque"),
    ("Ride height control", "Bodenfreiheitskontrolle"),
    ("Pull-rod rear suspension", "Pull-Rod-Hinterradaufhängung"),
    ("Carbon brake package", "Karbon-Bremspaket"),
    ("Lightweight chassis", "Leichtbau-Chassis"),
    ("Hydraulic suspension", "Hydraulische Aufhängung"),
    ("Chassis concept B", "Chassiskonzept B"),
    ("Engine mapping", "Motor-Mapping"),
    ("Turbo tune", "Turbo-Abstimmung"),
    ("ERS harvesting", "ERS-Rückgewinnung"),
    ("Combustion upgrade", "Verbrennungs-Upgrade"),
    ("Gearbox ratios", "Getriebeübersetzung"),
    ("New fuel blend", "Neue Kraftstoffmischung"),
    ("Split turbo", "Geteilter Turbo"),
    ("High-power ERS", "Hochleistungs-ERS"),
    ("Combustion chamber redesign", "Neu konstruierte Brennkammer"),
    ("Power unit spec 2", "Antriebseinheit Spezifikation 2"),
    ("Cooling louvres", "Kühlschlitze"),
    ("Gearbox seals", "Getriebedichtungen"),
    ("Hydraulics reroute", "Neu verlegte Hydraulik"),
    ("Electronics shielding", "Elektronik-Abschirmung"),
    ("Power unit cooling", "Kühlung der Antriebseinheit"),
    ("Lifed components", "Langlebige Bauteile"),
    ("Reinforced gearbox", "Verstärktes Getriebe"),
    ("Redundant hydraulics", "Redundante Hydraulik"),
    ("Sealed electronics", "Versiegelte Elektronik"),
    ("Bulletproof spec", "Kugelsichere Spezifikation"),
    // R&D: the view.
    ("{value}% fewer breakdowns", "{value} % weniger Ausfälle"),
    ("{value}% a lap", "{value} % pro Runde"),
    (
        "{cost} RP · {rounds} rounds · {risk}% risk",
        "{cost} RP · {rounds} Runden · {risk} % Risiko",
    ),
    ("The season is over", "Die Saison ist vorbei"),
    (
        "Too late: it wouldn't be ready this season",
        "Zu spät: Das Teil wäre diese Saison nicht mehr fertig",
    ),
    ("Over the cost cap", "Über der Budgetobergrenze"),
    ("You have {rp} resource points", "Du hast {rp} Ressourcenpunkte"),
    ("Needs {node}", "Braucht zuerst: {node}"),
    ("Can be developed", "Kann entwickelt werden"),
    ("On the car from round {round}", "Am Auto ab Runde {round}"),
    ("On the car", "Am Auto"),
    (
        "Failed its tests; half the points came back",
        "Bei den Tests durchgefallen; die Hälfte der Punkte kam zurück",
    ),
    ("Develop", "Entwickeln"),
    ("Develop again", "Erneut entwickeln"),
    ("on the car now: {gain}", "jetzt am Auto: {gain}"),
    (
        "Main branch: steady steps, {risk}% failure risk",
        "Hauptzweig: stetige Schritte, {risk} % Ausfallrisiko",
    ),
    (
        "Alternative branch: bigger steps, {risk}% failure risk",
        "Alternativzweig: größere Schritte, {risk} % Ausfallrisiko",
    ),
    ("level {level}", "Stufe {level}"),
    (
        "{part}: {node}, on the car from round {round} ({race})",
        "{part}: {node}, am Auto ab Runde {round} ({race})",
    ),
    ("COST CAP", "BUDGETOBERGRENZE"),
    (
        "{team} has spent {spent} of {cap} resource points this season",
        "{team} hat diese Saison {spent} von {cap} Ressourcenpunkten ausgegeben",
    ),
    (
        "With game sync on, the game's car gets the team's pace too.",
        "Mit eingeschaltetem Spielabgleich bekommt auch das Auto im Spiel das Tempo des Teams.",
    ),
    (
        "The game's own car stays as the season box has it unless the game sync for team pace is on.",
        "Das Auto im Spiel bleibt, wie die Saison-Box es vorgibt, außer der Spielabgleich für das Team-Tempo ist an.",
    ),
    ("resource points for {team}", "Ressourcenpunkte für {team}"),
    (
        "Earned in practice programmes, with qualifying, results, team goals and beating your team-mate. A part \
         takes two to four rounds to build and can fail its tests when it arrives: then it gains nothing and half \
         of its points come back.",
        "Verdient mit Trainingsprogrammen, im Qualifying, mit Ergebnissen, Teamzielen und Siegen gegen deinen \
         Teamkollegen. Ein Teil braucht zwei bis vier Runden in der Fertigung und kann bei der Ankunft durch die \
         Tests fallen: Dann bringt es nichts, und die Hälfte seiner Punkte kommt zurück.",
    ),
    (
        "Too late in the season: a new part wouldn't be ready. Your points carry over to next season.",
        "Zu spät in der Saison: Ein neues Teil wäre nicht mehr fertig. Deine Punkte gehen in die nächste Saison mit.",
    ),
    (
        "Gap to the fastest car: {now}% now, {start}% at the start of {year}. The {rank} fastest car of {count}.",
        "Abstand zum schnellsten Auto: jetzt {now} %, {start} % zu Beginn von {year}. Beim Tempo auf Platz \
         {place} von {count} Autos.",
    ),
    (
        "Every team's car also moves over the season as it did for real. Half of what a team develops carries into \
         next season (a sixth when the rules change, 2017, 2022 and 2026){carry}. The upgrades make the car faster \
         in simulated rounds and raise the team's goals.",
        "Jedes Auto entwickelt sich außerdem über die Saison so, wie es das in echt tat. Die Hälfte dessen, was \
         ein Team entwickelt, geht in die nächste Saison mit (ein Sechstel bei neuen Regeln: 2017, 2022 und \
         2026){carry}. Die Upgrades machen das Auto in simulierten Runden schneller und heben die Ziele des Teams.",
    ),
    (
        "; {team} carries {carry}% from earlier seasons",
        "; {team} bringt {carry} % aus früheren Saisons mit",
    ),
    // The inbox.
    ("Team mail", "Teammail"),
    ("Team event", "Team-Ereignis"),
    ("Media question", "Medienfrage"),
    ("Unread", "Ungelesen"),
    ("Mark all as read", "Alle als gelesen markieren"),
    (
        "No mails yet. After each round the team, its departments and the press may write to you.",
        "Noch keine Mails. Nach jeder Runde können dir das Team, seine Abteilungen und die Presse schreiben.",
    ),
    ("{year} · round {round}, {race}", "{year} · Runde {round}, {race}"),
    ("You answered: {answer}", "Deine Antwort: {answer}"),
    ("No effect", "Keine Auswirkung"),
    ("{value} resource points", "{value} Ressourcenpunkte"),
    ("{value} recognition", "{value} Anerkennung"),
    ("{value} experience", "{value} Erfahrung"),
    ("{value} racecraft", "{value} Zweikampf"),
    ("{value} awareness", "{value} Übersicht"),
    ("{value} pace", "{value} Tempo"),
    ("{value} focus", "{value} Fokus"),
    // The inbox's templates (`swofty_core::league::inbox::TEMPLATES`).
    (
        "Engineering report: the new {part} parts are on the car from the {race}. The first simulator runs match \
         what the factory promised.",
        "Bericht der Ingenieure: Die neuen Teile ({part}) sind ab dem {race} am Auto. Die ersten Simulatorläufe \
         bestätigen, was die Fabrik versprochen hat.",
    ),
    (
        "The simulator team asks you to put Track acclimatisation first at the {race}. If you agree, they add {rp} \
         resource points of their own work.",
        "Das Simulator-Team bittet dich, beim {race} die Streckeneingewöhnung vorzuziehen. Wenn du zustimmst, \
         steuert es {rp} Ressourcenpunkte aus eigener Arbeit bei.",
    ),
    (
        "The aerodynamics department asks you to put Qualifying pace first at the {race}. If you agree, they add \
         {rp} resource points of their own work.",
        "Die Aerodynamik-Abteilung bittet dich, beim {race} das Qualifying-Tempo vorzuziehen. Wenn du zustimmst, \
         steuert sie {rp} Ressourcenpunkte aus eigener Arbeit bei.",
    ),
    (
        "The vehicle dynamics group asks you to put Tyre management first at the {race}. If you agree, they add \
         {rp} resource points of their own work.",
        "Die Fahrdynamik-Gruppe bittet dich, beim {race} das Reifenmanagement vorzuziehen. Wenn du zustimmst, \
         steuert sie {rp} Ressourcenpunkte aus eigener Arbeit bei.",
    ),
    ("Agree to the programme", "Dem Programm zustimmen"),
    (
        "Decline: you have your own plan",
        "Ablehnen: Du hast deinen eigenen Plan",
    ),
    (
        "Media after the {race}: you finished P{position}. A reporter asks whether {team} can fight at the front \
         all season.",
        "Medien nach dem {race}: Du bist auf P{position} ins Ziel gekommen. Ein Reporter fragt, ob {team} die \
         ganze Saison vorne mitkämpfen kann.",
    ),
    (
        "Stay humble: the team gave you a great car",
        "Bescheiden bleiben: Das Team hat dir ein tolles Auto gegeben",
    ),
    (
        "Be bold: nobody out there was faster than you",
        "Selbstbewusst sein: Da draußen war niemand schneller als du",
    ),
    (
        "Media after the {race}: your race ended early. A reporter asks what went wrong.",
        "Medien nach dem {race}: Dein Rennen war früh vorbei. Ein Reporter fragt, was schiefgelaufen ist.",
    ),
    (
        "Take it on the chin: the team will find the cause together",
        "Es sportlich nehmen: Das Team findet die Ursache gemeinsam",
    ),
    (
        "Be bold: you would have been on the podium",
        "Selbstbewusst sein: Du wärst auf dem Podium gelandet",
    ),
    (
        "Deflect: you only think about the next race",
        "Ausweichen: Du denkst nur an das nächste Rennen",
    ),
    (
        "{mate} has finished ahead of you in the last {count} races, and the press talks of a feud inside {team}.",
        "{mate} ist in den letzten {count} Rennen vor dir ins Ziel gekommen, und die Presse spricht von einem \
         Streit bei {team}.",
    ),
    (
        "Demand equal treatment from the team",
        "Vom Team Gleichbehandlung verlangen",
    ),
    (
        "Study your team-mate's data with the engineers",
        "Mit den Ingenieuren die Daten deines Teamkollegen studieren",
    ),
    ("Keep it inside the team", "Es im Team klären"),
    // The market's mails (`swofty_core::league::market::TEMPLATES`).
];

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashMap};

    use swofty_core::league::{Part, inbox, market, rnd};

    use super::super::SLICES;
    use super::DE;

    fn german() -> HashMap<&'static str, &'static str> {
        SLICES.iter().flat_map(|slice| slice.iter().copied()).collect()
    }

    /// The `{placeholders}` of a text.
    fn placeholders(text: &str) -> BTreeSet<&str> {
        text.split('{')
            .skip(1)
            .filter_map(|s| s.split_once('}').map(|(name, _)| name))
            .collect()
    }

    #[test]
    fn every_inbox_template_has_german() {
        let de = german();
        for template in inbox::TEMPLATES.iter().chain(market::TEMPLATES) {
            let Some(text) = de.get(template) else {
                panic!("no German for the template {template:?}");
            };
            assert_eq!(
                placeholders(template),
                placeholders(text),
                "the German of {template:?} has other placeholders"
            );
        }
    }

    #[test]
    fn every_development_node_and_part_has_german() {
        let de = german();
        for part in Part::ALL {
            assert!(
                de.contains_key(part.name()),
                "no German for the part {}",
                part.name()
            );
            for node in rnd::nodes(part) {
                assert!(de.contains_key(node.name), "no German for the node {}", node.name);
            }
        }
    }

    #[test]
    fn team_texts_keep_their_placeholders() {
        // The English ordinal ("3rd") becomes a number in German: {rank} -> {place}.
        for (en, de) in DE {
            let (mut en, de) = (placeholders(en), placeholders(de));
            if en.remove("rank") {
                en.insert("place");
            }
            assert_eq!(en, de, "placeholders differ for {en:?}");
        }
    }
}
