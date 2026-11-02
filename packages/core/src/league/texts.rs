//! The career's texts in the player's language (W4-2): news, refusals and warnings.
//!
//! Every text a player sees is an English template with `{placeholders}` (a translation key of
//! the launcher, listed in [`TEXTS`]) and the values for them, made with the [`t!`] macro. The
//! league keeps the English rendering too (`News.text`, `LeagueError.message`, the `warnings`
//! of an action), so older launchers and the CLI show what they always showed; a launcher in
//! another language translates the template and fills in the values.
//!
//! A news item of one sentence carries its template in `News.template`/`args`; one made of
//! several sentences carries them in `News.parts`, each translated on its own. Values of the
//! placeholders in [`TRANSLATED_ARGS`] are words the launcher translates too ([`WORDS`]); the
//! other values (names, numbers, races) are shown as they are.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A text to show a player: the English `text`, its English `template` (a translation key) and
/// the values of the template's placeholders.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Text {
    /// The English text. Empty in a news item's `parts`, where `News.text` holds it.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub text: String,
    /// The English template of `text`; empty for a text that has none (shown as it is).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub template: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<(String, String)>,
    /// Placeholders filled with a list of texts joined with ", ", each translated on its own
    /// (their English joined is in `args` too).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lists: Vec<(String, Vec<Text>)>,
}

/// A warning an action returns (`warnings_t` in the host server's answer).
pub type Warning = Text;

impl Text {
    /// `template` with its placeholders filled from `args`.
    pub fn new(template: &str, args: &[(&str, String)]) -> Text {
        let args: Vec<(String, String)> = args.iter().map(|(k, v)| ((*k).to_owned(), v.clone())).collect();
        Text {
            text: render(template, &args),
            template: template.to_owned(),
            args,
            lists: Vec::new(),
        }
    }

    /// Fills the placeholder `key` with `items` joined with ", ".
    #[must_use]
    pub fn list(mut self, key: &str, items: Vec<Text>) -> Text {
        let joined = english(&items).join(", ");
        self.args.retain(|(k, _)| k != key);
        self.args.push((key.to_owned(), joined));
        self.lists.push((key.to_owned(), items));
        self.text = render(&self.template, &self.args);
        self
    }

    /// A text without a template, shown as it is in every language.
    pub fn plain(text: impl Into<String>) -> Text {
        Text {
            text: text.into(),
            template: String::new(),
            args: Vec::new(),
            lists: Vec::new(),
        }
    }

    /// The English text (rendered from the template when `text` was left out).
    pub fn english(&self) -> String {
        if self.text.is_empty() && !self.template.is_empty() {
            render(&self.template, &self.args)
        } else {
            self.text.clone()
        }
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.english())
    }
}

/// A text equals the English text it shows.
impl PartialEq<str> for Text {
    fn eq(&self, other: &str) -> bool {
        self.english() == other
    }
}

impl PartialEq<&str> for Text {
    fn eq(&self, other: &&str) -> bool {
        self.english() == *other
    }
}

impl PartialEq<String> for Text {
    fn eq(&self, other: &String) -> bool {
        self.english() == *other
    }
}

impl From<Text> for String {
    fn from(t: Text) -> String {
        t.english()
    }
}

/// `template` with its `{placeholders}` filled from `args`.
pub fn render(template: &str, args: &[(String, String)]) -> String {
    args.iter().fold(template.to_owned(), |text, (k, v)| {
        text.replace(&format!("{{{k}}}"), v)
    })
}

/// The English texts of `texts`, one after the other.
pub fn english(texts: &[Text]) -> Vec<String> {
    texts.iter().map(Text::english).collect()
}

/// A [`Text`] from an English template literal and its placeholders' values:
/// `t!("{name} left the career.", name)` or `t!("round {n} has no sprint", n = round)`.
macro_rules! t {
    (@value $key:ident = $value:expr) => {
        ($value).to_string()
    };
    (@value $key:ident) => {
        ($key).to_string()
    };
    ($template:literal $(, $key:ident $(= $value:expr)?)* $(,)?) => {
        $crate::league::texts::Text::new(
            $template,
            &[$((stringify!($key), t!(@value $key $(= $value)?))),*],
        )
    };
}
pub(crate) use t;

/// The placeholders whose values are words the launcher translates too ([`words`]); a value may
/// list several, joined with ", ".
pub const TRANSLATED_ARGS: &[&str] = &[
    // A car part (`Part::name`), a session, a country (`driver::NATIONALITIES`), development
    // nodes (`rnd::TREE`)
    "part", "session", "country", "node", "needs",
    // The weekend check's label and values (`weekend::check`)
    "label", "ran", "agreed",
];

/// Every value a placeholder of [`TRANSLATED_ARGS`] takes (besides numbers the weekend check
/// names for values the game's tables don't know).
pub fn words() -> Vec<&'static str> {
    let mut words = WORDS.to_vec();
    words.extend(super::weekend::words());
    words.extend(super::driver::NATIONALITIES.iter().map(|(_, name)| *name));
    words.extend(super::rnd::TREE.iter().flatten().map(|n| n.name));
    words.sort_unstable();
    words.dedup();
    words
}

/// The parts' and sessions' names among [`words`].
const WORDS: &[&str] = &[
    // Car parts (`Part::name`)
    "Aerodynamics",
    "Chassis",
    "Powertrain",
    "Durability",
    // Sessions
    "race",
    "sprint",
    "qualifying",
    "practice",
    "Q1",
    "Q2",
    "Q3",
    "short qualifying",
    "one-shot qualifying",
];

/// Every English template of the league's texts made with [`t!`] (besides those of
/// [`super::inbox::TEMPLATES`], [`super::market::TEMPLATES`] and [`super::aimarket::TEMPLATES`]).
/// The launcher has a German text for each, with the same placeholders.
pub const TEXTS: &[&str] = &[
    "{name} asked to simulate round {round}, the {race} first; agree to it or decline it before asking for something else",
    "{name} asked to leave out round {round}, the {race} first; agree to it or decline it before asking for something else",
    "{name} asked to take back round {round}, the {race} first; agree to it or decline it before asking for something else",
    "{name} asks to {what}; it happens once everyone agrees.",
    "The proposal to {what} lapsed: the career moved on meanwhile.",
    "no season is running",
    "every round is counted",
    "there is no open proposal to vote on",
    "you aren't asked in this vote",
    "only the career's owner goes ahead without the others",
    "the proposal to {what} lapsed: the career moved on meanwhile",
    "{name} withdrew the proposal to {what}.",
    "{name} doesn't agree to {what}; it doesn't happen.",
    "{name} decided to {what} without waiting for {waiting}.",
    "nothing to take back",
    "no round of this season to take back",
    "simulate round {round}, the {race}",
    "leave out round {round}, the {race}",
    "take back round {round}, the {race}",
    "simulate round {round}",
    "leave out round {round}",
    "take back round {round}",
    "Waiting for {names} to agree.",
    "{names} and {last}",
    "join the career first",
    "the career is complete",
    "your driver is set once a session of yours is counted",
    "{n} isn't a nationality to pick",
    "{name} races as an experienced driver from {country}, rated {rating}.",
    "{name} races as a rookie from {country}, rated {rating}.",
    "{name} races as a veteran from {country}, rated {rating}.",
    "{name} races as an experienced driver, rated {rating}.",
    "{name} races as a rookie, rated {rating}.",
    "{name} races as a veteran, rated {rating}.",
    "{name} races from {country}.",
    "{name} races.",
    "there is no mail {news}",
    "only the player it is for answers this mail",
    "this mail is answered already",
    "this mail has no such answer",
    "teams approach drivers during the season",
    "{team} hasn't approached you",
    "{team} isn't waiting to meet you any more",
    "{team} offers {years} from {next}, {salary} a season.",
    "{name} was seen meeting {team}; {own} isn't pleased.",
    "there is no team {slot} in {next} any more",
    "meet {team} first to hear their offer",
    "{team}'s approach is over",
    "{team} walked away from the talks",
    "{team} buys {name} out of the contract with {own}: {name} drives for {team} from {next} to {until}, for {salary} a season.",
    "{team} buys {name} out of the contract with {own}: {name} drives for {team} from {next}, for {salary} a season.",
    "{name} signs early with {team} for {next} to {until}, for {salary} a season.",
    "{name} signs early with {team} for {next}, for {salary} a season.",
    "that approach is over",
    "ask for a salary and a contract of 1 to 3 years",
    "that team made you no offer",
    "contracts are negotiated during and at the end of a season",
    "a contract that runs on isn't negotiated",
    "you agreed terms with {team} already; sign them",
    "{team} won't talk any more; their first offer stands",
    "{team} agrees: {salary} a season for {years}. Sign it to make it count.",
    "{team} agrees: {salary} a season for {years} with a number-one clause. Sign it to make it count.",
    "{team} counters: {salary} a season for {years} with a number-one clause.",
    "{team} counters: {salary} a season for {years}, without a number-one clause.",
    "{team} counters: {salary} a season for {years}.",
    "1 more attempt before they walk away.",
    "{attempts} more attempts before they walk away.",
    "{counter} {left}",
    "{team} ends the talks; their first offer stands.",
    "{team} walks away from the talks and withdraws the offer.",
    "1 year",
    "{years} years",
    "{driver} drives {out}'s {team} (the game shows {out})",
    "the career runs from {first} to {last}; pick years in between",
    "the race distance is a percentage from 5 to 100",
    "{name} started the career, {first_year} to {last_year}.",
    "{name} is back in the career.",
    "this career has no free seat",
    "another player races with number {n}",
    "pick a race number from 1 to 99",
    "{name} joined the career and gets offers when the season ends.",
    "seats are picked before the first season; later, sign an offer",
    "no team {slot} in {year}",
    "as a rookie you start in the slower half of the grid; {team} is too fast",
    "only the career's owner starts the first season",
    "the career has started already",
    "{name} hasn't picked a seat yet",
    "The {year} season begins.",
    "{name}'s rival this season: {rival} ({team}).",
    "{name} drives for {team} (#{number}).",
    "round {n} is counted already",
    "only the next round ({next}) can be counted, simulated, skipped or practised",
    "{year} has no round {n}",
    "this looks like the {session} of round {round}, the {race}, that {by} counted moments ago; if it is another session, count it anyway",
    "this looks like the {session} of round {round}, the {race}, that another player counted moments ago; if it is another session, count it anyway",
    "this race is counted already",
    "this sprint is counted already",
    "this qualifying is counted already",
    "this practice is counted already",
    "this race was at {track}, but round {n} is the {race}",
    "this race was at another track, but round {n} is the {race}",
    "a sprint shootout isn't counted; the sprint and the race are",
    "the game didn't say whether this was the sprint or the Grand Prix of round {n}; count it as one of them",
    "round {n} has no sprint",
    "the sprint of round {n} is counted already",
    "a later part of this qualifying is counted already",
    "none of the cars in this race belong to this season's grid",
    "Qualifying updated ({session}).",
    "{name} takes the first pole position of their career!",
    "{name}'s pole position number {k}.",
    "{name} wins a Grand Prix for the first time!",
    "{name}'s Grand Prix win number {k}!",
    "{name} stands on the podium for the first time.",
    "{name} scores the first points of their career.",
    "{name} meets the team's goal (P{goal} or better).",
    "{name} misses the team's goal of P{goal}.",
    "(simulated: the game doesn't have this circuit)",
    "(simulated)",
    "Round {n}, the {race}, is left out.",
    "Round {n}, the {race}, was taken back.",
    "From the {race}, {driver} stands in for {out} at {team} until {out} is back at the {back}.",
    "From the {race}, {driver} stands in for {out} at {team}.",
    "From the {race}, {driver} drives for {team} in place of {out} for a while.",
    "From the {race}, {driver} drives for {team} in place of {out}.",
    "From the {race}, {driver} replaces {out} at {team}.",
    "The game's season box still shows its own line-up; the career counts each car for the driver of that seat.",
    "{driver} is the {year} world champion with {points} points; {team} win the constructors' title.",
    "{name} earned {season} this season ({career} in the career).",
    "{name} against team-mate {mate}: qualifying {q1}:{q2}, races {r1}:{r2}.",
    "{name} won the rivalry with {rival} {r1}:{r2}.",
    "{name} lost the rivalry with {rival} {r1}:{r2}.",
    "{name} drew the rivalry with {rival} {r1}:{r2}.",
    "The career is complete.",
    "Contracts for {year}:",
    "{team} (under contract)",
    "{team} (1 year, {salary} a season)",
    "{team} ({years} years, {salary} a season)",
    "{name} has offers from {teams}.",
    "{team} doesn't renew the contract.",
    "contracts are signed when a season is over",
    "{player} stays with {team} for {next}, under contract until {until}.",
    "{player} signs with {team} for {next} to {until}, for {salary} a season.",
    "{player} signs with {team} for {next}, for {salary}.",
    "only the career's owner moves on before everyone signed",
    "the season isn't over yet",
    "that player isn't in this career",
    "only the career's owner removes another player",
    "the career's owner can't leave their own career",
    "{name} left the career.",
    "{name} has left the career already",
    "{name} left the career; the game's AI drives their seat for the rest of the season.",
    "only the career's owner changes the career options",
    "equal performance is off while the league's car pace is in the game",
    "the players' names stay in the game while AI drivers move between teams",
    "{name} changed the career options.",
    "this career was saved by a newer F1 Multiplayer; update it first",
    "you left this career; join it again first",
    "this career has started already",
    "{key} doesn't drive for {team}",
    "another player has that seat",
    "both seats at {team} are taken",
    "no season data for {year}",
    "{name}'s seat doesn't exist in {year}",
    "{name}'s seat was driven by the game's AI",
    "matched {shown} to {name} by elimination",
    "the game showed the real drivers in {by_seat} seats the AI market changed; they count for the league's drivers of those seats (prepare the game sync to see them in the game)",
    "P{position} {name} (#{number}) matched no driver of {year} and isn't counted",
    "{name} P{position}",
    "{name} no time",
    "{name} out",
    "Round {n}, {race} qualifying: {winner} on pole",
    "Round {n}, {race} qualifying: no pole time",
    "Round {n}, {race} sprint: {winner} wins",
    "Round {n}, {race} sprint: no winner classified",
    "Round {n}, {race}: {winner} wins",
    "Round {n}, {race}: no winner classified",
    "{result}.",
    "{result}; {players}.",
    "{name} {met} of {count}",
    "Objectives: {counts}.",
    "you have no seat this season",
    "your practice of round {n} is counted already",
    "you drove no lap in this practice session",
    "Round {n}, {race} practice: {name} grade {grade}, {rp} resource points.",
    "Round {n}, {race} practice: {name} grade {grade}, {rp} resource points, best lap {lap}.",
    "Round {n}, {race} practice (simulated): {name} grade {grade}, {rp} resource points.",
    "Round {n}, {race} practice (simulated): {name} grade {grade}, {rp} resource points, best lap {lap}.",
    "{name} is now {team}'s number one driver.",
    "{name} is no longer {team}'s number one driver.",
    "{team}'s new {node} ({part}) fails its tests and gains nothing; {back} of its {cost} resource points go back to {payer}.",
    "{team}'s {node} ({part})",
    "On the car for the {race}: {fitted}.",
    "parts are developed while a season runs",
    "the season is over",
    "{part}'s main branch is developed as far as it goes this season",
    "{part} has no development node {id}",
    "{node} needs {needs} first",
    "{node} is ordered already",
    "{node} ({part}) costs {cost} resource points; you have {have}",
    "{node} would take the team over the cost cap: it has spent {spent} of {cap} resource points this season",
    "{node} takes {rounds} rounds to build; it wouldn't be ready this season",
    "{name} has {team} develop {part} level {level} ({node}); it's on the car from round {ready}, the {race}.",
    "a later part of this sprint shootout is counted already",
    "none of the cars in this sprint shootout belong to this season's grid",
    "sprint shootout counted",
    "Round {n}, {race} sprint shootout: {winner} on sprint pole",
    "Round {n}, {race} sprint shootout updated: {winner} on sprint pole",
    "Round {n}, {race} sprint shootout: no sprint pole time",
    "Round {n}, {race} sprint shootout updated: no sprint pole time",
    "the AI difficulty goes from 0 to {max}",
    "car damage is off (0), reduced (1), standard (2) or simulation (3)",
    "the safety car is off (0), reduced (1), standard (2) or increased (3)",
    "the weather goes from clear (0) to storm (5)",
    "the game has no such rule set",
    "{label}: the game ran {ran}, the agreed rules say {agreed}",
];
