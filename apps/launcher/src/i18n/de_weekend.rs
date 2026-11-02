//! The German texts of the career's race weekend (W3-3): the objectives, the rivalry meter, the
//! rules sheet with the game's option names, and the practice programmes.

pub(super) const DE: &[(&str, &str)] = &[
    // The weekend panel
    (
        "THIS WEEKEND'S OBJECTIVES · ROUND {round}",
        "ZIELE AN DIESEM WOCHENENDE · RUNDE {round}",
    ),
    ("RULES SHEET", "REGELBLATT"),
    ("PRACTICE PROGRAMMES", "TRAININGSPROGRAMME"),
    // Objectives
    (
        "Practice grade {target} or better",
        "Trainingsnote {target} oder besser",
    ),
    (
        "Qualify P{target} or better",
        "Qualifying auf P{target} oder besser",
    ),
    ("Finish P{target} or better", "Ins Ziel auf P{target} oder besser"),
    (
        "Qualify ahead of your team-mate",
        "Im Qualifying vor deinem Teamkollegen",
    ),
    (
        "Finish ahead of your team-mate",
        "Im Rennen vor deinem Teamkollegen ins Ziel",
    ),
    (
        "Finish ahead of your rival {rival}",
        "Vor deinem Rivalen {rival} ins Ziel",
    ),
    ("Finish ahead of your rival", "Vor deinem Rivalen ins Ziel"),
    (
        "Finish the race without a penalty",
        "Das Rennen ohne Strafe beenden",
    ),
    ("+{rp} resource points", "+{rp} Ressourcenpunkte"),
    ("+{n} recognition", "+{n} Anerkennung"),
    // The rivalry meter
    (
        "Level with {rival} in this season's qualifying and races",
        "Gleichauf mit {rival} in den Qualifyings und Rennen dieser Saison",
    ),
    (
        "Ahead of {rival} by {n} in this season's qualifying and races",
        "{n} vor {rival} in den Qualifyings und Rennen dieser Saison",
    ),
    (
        "Behind {rival} by {n} in this season's qualifying and races",
        "{n} hinter {rival} in den Qualifyings und Rennen dieser Saison",
    ),
    // The rules sheet: labels
    ("AI difficulty", "KI-Schwierigkeit"),
    ("Qualifying", "Qualifying"),
    ("Practice", "Training"),
    ("Car damage", "Fahrzeugschaden"),
    ("Safety car", "Safety-Car"),
    ("Parc fermé", "Parc fermé"),
    ("Equal car performance", "Gleiche Fahrzeugleistung"),
    ("Weather", "Wetter"),
    ("Rule set", "Regelwerk"),
    // The rules sheet: the game's option names
    ("Reduced", "Reduziert"),
    ("Standard", "Standard"),
    ("Simulation", "Simulation"),
    ("Increased", "Erhöht"),
    ("Weather: Clear", "Klar"),
    ("Light cloud", "Leicht bewölkt"),
    ("Overcast", "Bedeckt"),
    ("Light rain", "Leichter Regen"),
    ("Heavy rain", "Starker Regen"),
    ("Storm", "Sturm"),
    ("Practice and qualifying", "Training und Qualifying"),
    ("Race", "Rennen"),
    ("Time trial", "Zeitfahren"),
    ("Time attack", "Zeitangriff"),
    ("Checkpoint challenge", "Checkpoint-Herausforderung"),
    ("Autocross", "Autocross"),
    ("Drift", "Drift"),
    ("Average speed zone", "Durchschnittsgeschwindigkeitszone"),
    ("Rival duel", "Rivalenduell"),
    ("Full (Q1, Q2, Q3)", "Voll (Q1, Q2, Q3)"),
    ("Short", "Kurz"),
    ("One-shot", "Ein Versuch"),
    ("Full (P1, P2, P3)", "Voll (P1, P2, P3)"),
    ("One session", "Eine Sitzung"),
    ("None", "Keins"),
    (
        "No rules agreed: everyone drives with the settings they like.",
        "Keine Regeln vereinbart: Jeder fährt mit den Einstellungen, die er mag.",
    ),
    // The rules sheet editor
    ("CAREER OPTIONS", "KARRIERE-OPTIONEN"),
    ("CAREER OPTIONS · RULES SHEET", "KARRIERE-OPTIONEN · REGELBLATT"),
    // The AI driver market (W4-1)
    ("AI driver market", "KI-Fahrermarkt"),
    (
        "Off: each season has its year's real grid. On: from the next season, the AI drivers move between \
         teams, retire and make way for rookies.",
        "Aus: Jede Saison hat das echte Starterfeld ihres Jahres. An: Ab der nächsten Saison wechseln die \
         KI-Fahrer die Teams, treten zurück und machen Platz für Neulinge.",
    ),
    (
        "On: at each season's end the AI drivers move between teams, retire and make way for rookies, the \
         same on every PC. The game shows them in the seats of the year's real drivers, with those drivers' \
         faces, and shows the players' names too.",
        "An: Am Ende jeder Saison wechseln die KI-Fahrer die Teams, treten zurück und machen Platz für \
         Neulinge, auf jedem PC gleich. Das Spiel zeigt sie in den Cockpits der echten Fahrer des Jahres, mit \
         deren Gesichtern, und zeigt auch die Namen der Spieler.",
    ),
    (
        "This season {count} AI drivers race in other drivers' seats.",
        "In dieser Saison fahren {count} KI-Fahrer in den Cockpits anderer Fahrer.",
    ),
    (
        "The rules sheet: what everyone sets in the game for each weekend.",
        "Das Regelblatt: was jeder im Spiel für jedes Wochenende einstellt.",
    ),
    ("Edit the rules sheet", "Regelblatt bearbeiten"),
    ("Not agreed", "Nicht vereinbart"),
    (
        "Off: the league's car pace is in the game.",
        "Aus: Das Fahrzeugtempo der Liga ist im Spiel.",
    ),
    (
        "What everyone sets in the game for each weekend. Each counted session is checked against it from the \
         telemetry; a difference warns and the session counts all the same.",
        "Was jeder im Spiel für jedes Wochenende einstellt. Jede gezählte Sitzung wird anhand der Telemetrie damit \
         verglichen; eine Abweichung gibt eine Warnung, und die Sitzung zählt trotzdem.",
    ),
    // Practice programmes
    ("Track acclimatisation", "Streckenakklimatisierung"),
    ("Qualifying pace", "Qualifying-Tempo"),
    ("Tyre management", "Reifenmanagement"),
    ("ERS management", "ERS-Management"),
    ("Fuel management", "Benzinmanagement"),
    ("Race strategy", "Rennstrategie"),
    (
        "Clean laps to learn the track: eight make full marks.",
        "Saubere Runden, um die Strecke zu lernen: acht geben die volle Punktzahl.",
    ),
    (
        "Your best lap against your team-mate's.",
        "Deine beste Runde gegen die deines Teamkollegen.",
    ),
    (
        "A long run: four or more clean laps close to your best.",
        "Ein Longrun: vier oder mehr saubere Runden nah an deiner besten.",
    ),
    (
        "Three laps at pace deploying the full 4 MJ, with a tenth of the store left at the line.",
        "Drei schnelle Runden, die die vollen 4 MJ einsetzen, mit einem Zehntel des Speichers übrig an der Linie.",
    ),
    (
        "Three laps at pace within the fuel target, about 0.32 kg per km.",
        "Drei schnelle Runden innerhalb des Benzinziels, etwa 0,32 kg pro km.",
    ),
    (
        "Five laps in a row at pace with little tyre wear, about 0.38% per km or less.",
        "Fünf schnelle Runden am Stück mit wenig Reifenverschleiß, etwa 0,38 % pro km oder weniger.",
    ),
    (
        "{clean} clean laps of {laps}",
        "{clean} saubere Runden von {laps}",
    ),
    ("best lap {lap}", "beste Runde {lap}"),
    (
        "{n} clean laps within 107% of the best",
        "{n} saubere Runden innerhalb von 107 % der besten",
    ),
    (
        "{deployed} MJ deployed a lap, {left} MJ left at the line",
        "{deployed} MJ eingesetzt pro Runde, {left} MJ übrig an der Linie",
    ),
    ("{kg} kg of fuel a lap", "{kg} kg Benzin pro Runde"),
    ("{wear}% tyre wear a lap", "{wear} % Reifenverschleiß pro Runde"),
    ("not graded", "nicht bewertet"),
    ("not simulated", "nicht simuliert"),
    ("open", "offen"),
    (
        "A simulated practice grades the first three programmes only; drive it in the game for all six.",
        "Ein simuliertes Training bewertet nur die ersten drei Programme; fahre es im Spiel für alle sechs.",
    ),
    (
        "The ERS, fuel and race-strategy programmes need the game's telemetry with enough laps at pace.",
        "Die Programme für ERS, Benzin und Rennstrategie brauchen die Telemetrie des Spiels mit genug schnellen \
         Runden.",
    ),
    (
        "The figures are of the practice session that waits to be counted.",
        "Die Werte stammen aus der Trainingssitzung, die noch gezählt werden muss.",
    ),
];

#[cfg(test)]
mod tests {
    use swofty_core::league::weekend::{self, QualiFormat};
    use swofty_core::league::{PracticeFormat, Programme, WeekendRules};

    use super::super::SLICES;

    fn german(text: &str) -> bool {
        SLICES.iter().flat_map(|s| s.iter()).any(|(en, _)| *en == text)
    }

    /// Every label and option name the rules sheet can show has German (numbers stay as they are;
    /// "Clear" has its own key, "Weather: Clear", as the Addons page's "Clear" is another word).
    #[test]
    fn every_rules_sheet_word_has_german() {
        let mut words = Vec::new();
        for v in 0..=20u8 {
            let rules = WeekendRules {
                ai_difficulty: Some(v),
                quali: Some(QualiFormat::Full),
                practice: Some(PracticeFormat::Full),
                damage: Some(v),
                safety_car: Some(v),
                parc_ferme: Some(v % 2 == 0),
                equal_performance: Some(v % 2 == 1),
                weather: Some(v),
                rule_set: Some(v),
            };
            words.extend(weekend::describe(&rules));
        }
        for f in [QualiFormat::Full, QualiFormat::Short, QualiFormat::OneShot] {
            words.push((weekend::LABEL_QUALI, f.name().to_owned()));
        }
        for f in [
            PracticeFormat::Full,
            PracticeFormat::OneSession,
            PracticeFormat::Short,
            PracticeFormat::None,
        ] {
            words.push((weekend::LABEL_PRACTICE, f.name().to_owned()));
        }
        for (label, value) in words {
            assert!(german(label), "no German for the label {label}");
            if value.parse::<u8>().is_ok() {
                continue;
            }
            let keyed = format!("{label}: {value}");
            assert!(german(&value) || german(&keyed), "no German for {label} {value}");
        }
    }

    #[test]
    fn every_programme_has_german() {
        for p in [
            Programme::TrackAcclimatisation,
            Programme::QualifyingPace,
            Programme::TyreManagement,
            Programme::ErsManagement,
            Programme::FuelManagement,
            Programme::RaceStrategy,
        ] {
            assert!(german(p.name()), "no German for {}", p.name());
        }
    }
}
