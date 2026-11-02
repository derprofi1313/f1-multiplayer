//! The league's texts in the launcher's language (W4-2): news, refusals and warnings come with
//! their English template (a `tr` key) and the values of its placeholders
//! (`swofty_core::league::texts`); this translates the template and fills it in.

use swofty_core::league::texts::TRANSLATED_ARGS;
use swofty_core::league::{News, Text};

use crate::i18n::tr_owned;

/// A value of a placeholder: translated when it is a word ([`TRANSLATED_ARGS`]), each of a list
/// joined with ", " on its own; names and numbers stay as they are.
fn value(key: &str, value: &str) -> String {
    if TRANSLATED_ARGS.contains(&key) {
        value.split(", ").map(tr_owned).collect::<Vec<_>>().join(", ")
    } else {
        value.to_owned()
    }
}

/// `template` in the launcher's language with its placeholders filled from `args` and `lists`.
fn render(template: &str, args: &[(String, String)], lists: &[(String, Vec<Text>)]) -> String {
    let mut out = tr_owned(template);
    for (key, items) in lists {
        let joined = items.iter().map(text).collect::<Vec<_>>().join(", ");
        out = out.replace(&format!("{{{key}}}"), &joined);
    }
    for (key, v) in args {
        out = out.replace(&format!("{{{key}}}"), &value(key, v));
    }
    out
}

/// A league text in the launcher's language; one without a template as it is.
pub(crate) fn text(t: &Text) -> String {
    if t.template.is_empty() {
        return t.english();
    }
    render(&t.template, &t.args, &t.lists)
}

/// Texts in the launcher's language, joined with `sep`.
pub(crate) fn joined(ts: &[Text], sep: &str) -> String {
    ts.iter().map(text).collect::<Vec<_>>().join(sep)
}

/// A news item's text in the launcher's language: its template (or the templates of its
/// sentences) translated with the placeholders filled, or the English text of items from before
/// templates.
pub(crate) fn news_text(n: &News) -> String {
    if let Some(template) = n.template.as_deref() {
        return render(template, &n.args, &[]);
    }
    if !n.parts.is_empty() {
        return joined(&n.parts, " ");
    }
    n.text.clone()
}

/// A refusal's text from its template, when it came with one.
pub(crate) fn refusal(template: &str, args: &[(String, String)]) -> Option<String> {
    (!template.is_empty()).then(|| render(template, args, &[]))
}

#[cfg(test)]
mod tests {
    use swofty_core::league::{self, Action, Actor, League, News, Phase, Setup, Text};

    use super::{news_text, refusal, text};
    use crate::i18n::set;

    fn anna() -> Actor {
        Actor {
            id: "u-anna".into(),
            name: "Anna".into(),
            owner: true,
        }
    }

    fn arg<'a>(k: &'a str, v: &str) -> (&'a str, String) {
        (k, v.to_owned())
    }

    #[test]
    fn rendering_fills_placeholders() {
        let left = Text::new(
            "Round {n}, the {race}, is left out.",
            &[arg("n", "3"), arg("race", "Bahrain Grand Prix")],
        );
        let counts = Text::new("Objectives: {counts}.", &[]).list(
            "counts",
            vec![Text::new(
                "{name} {met} of {count}",
                &[arg("name", "Anna"), arg("met", "2"), arg("count", "3")],
            )],
        );
        let quali = Text::new("Qualifying updated ({session}).", &[arg("session", "Q3")]);
        let part = Text::new(
            "{part}'s main branch is developed as far as it goes this season",
            &[arg("part", "Chassis")],
        );
        set(None);
        assert_eq!(text(&left), left.text);
        assert_eq!(text(&counts), "Objectives: Anna 2 of 3.");
        set(Some("de"));
        assert_eq!(text(&left), "Runde 3 (Bahrain Grand Prix) wird ausgelassen.");
        assert_eq!(text(&counts), "Ziele: Anna 2 von 3.");
        assert_eq!(text(&quali), "Qualifying aktualisiert (Q3).");
        // A translated placeholder's value is translated too.
        assert!(text(&part).starts_with("der Hauptzweig von "), "{}", text(&part));
        // A news item of several sentences translates each.
        let news = News::of(2016, 3, vec![left.clone(), counts]);
        assert_eq!(
            news_text(&news),
            "Runde 3 (Bahrain Grand Prix) wird ausgelassen. Ziele: Anna 2 von 3."
        );
        // A refusal from its template; none without one.
        assert_eq!(
            refusal("round {n} has no sprint", &[("n".into(), "4".into())]).as_deref(),
            Some("Runde 4 hat keinen Sprint")
        );
        assert_eq!(refusal("", &[]), None);
        set(None);
    }

    #[test]
    fn old_news_without_template_still_show() {
        let old = News::plain(2016, 3, "Round 3, Bahrain Grand Prix: Anna wins.".into());
        set(Some("de"));
        assert_eq!(news_text(&old), "Round 3, Bahrain Grand Prix: Anna wins.");
        // A text without a template shows as it is.
        assert_eq!(text(&Text::plain("Ben")), "Ben");
        set(None);
        assert_eq!(news_text(&old), old.text);
    }

    /// A season played out reads in German: every news item, warning and refusal.
    #[test]
    fn a_played_season_reads_in_german() {
        let setup = Setup {
            first_year: 2016,
            last_year: 2017,
            ..Setup::default()
        };
        let mut l = League::start(setup, &anna(), Some(7), 11).expect("start");
        league::apply(&mut l, &anna(), Action::Seat { slot: 2, replaces: None }).expect("seat");
        league::apply(&mut l, &anna(), Action::Begin).expect("begin");
        let mut warnings = Vec::new();
        while l.phase == Phase::Racing {
            let _ = league::apply_t(&mut l, &anna(), Action::Develop { part: league::Part::Aero });
            warnings.extend(league::apply_t(&mut l, &anna(), Action::Practice { round: None }).unwrap_or_default());
            warnings.extend(league::apply_t(&mut l, &anna(), Action::Simulate { round: None }).expect("simulate"));
        }
        let refused = league::apply_t(&mut l, &anna(), Action::Simulate { round: None }).expect_err("season over");
        set(Some("de"));
        for n in &l.news {
            let shown = news_text(n);
            assert!(!shown.contains('{'), "{shown}");
            assert_ne!(shown, n.text, "not translated: {shown}");
        }
        for w in &warnings {
            assert_ne!(text(w), w.text, "not translated: {}", w.text);
        }
        let shown = refusal(&refused.template, &refused.args).expect("a template");
        assert_ne!(shown, refused.message);
        set(None);
    }
}
