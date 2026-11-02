//! The career inbox: the player's mails, department events and media questions
//! (`swofty_core::league::inbox`), shown in the language of the launcher through their templates,
//! with the answers they offer.

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{Disableable, Sizable, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use swofty_core::league::{Action, Effect, League, News, NewsKind, inbox};

use crate::i18n::{tr, tr_owned};
use crate::pages::tag;
use crate::shell::Shell;
use crate::theme::Palette;

/// The most mails the inbox shows.
const SHOWN: usize = 40;

/// `player`'s mails, events and media questions (those for everyone too), the newest first.
pub(super) fn mails_of<'a>(league: &'a League, player: &str) -> Vec<&'a News> {
    let mut mails: Vec<&News> = league
        .news
        .iter()
        .filter(|n| n.kind != NewsKind::News && n.to.as_deref().is_none_or(|to| to == player))
        .collect();
    mails.sort_by_key(|n| std::cmp::Reverse(n.id));
    mails
}

/// The id up to which opening the inbox marks `player`'s mails read, when some are unread.
pub(super) fn read_upto(league: &League, player: &str) -> Option<u32> {
    inbox::unread(league, player).iter().map(|n| n.id).max()
}

/// A news item's text in the launcher's language: its template translated with the placeholders
/// filled (part names, which come joined in English, translated each), or the English text of
/// items from before templates.
pub(super) fn news_text(n: &News) -> String {
    super::texts::news_text(n)
}

/// A rating change like "+0.3" or "−1".
fn signed(x: f32) -> String {
    if (x - x.round()).abs() < 1e-4 {
        format!("{:+}", x.round() as i32)
    } else {
        format!("{x:+.1}")
    }
}

/// What an answer does, like "+1 recognition, +0.3 pace", or "No effect".
pub(super) fn effect_text(e: &Effect) -> String {
    let mut parts = Vec::new();
    if e.rp != 0 {
        parts.push(tr_owned("{value} resource points").replace("{value}", &format!("{:+}", e.rp)));
    }
    for (value, template) in [
        (e.recognition, "{value} recognition"),
        (e.experience, "{value} experience"),
        (e.racecraft, "{value} racecraft"),
        (e.awareness, "{value} awareness"),
        (e.pace, "{value} pace"),
        (e.focus, "{value} focus"),
    ] {
        if value != 0.0 {
            parts.push(tr_owned(template).replace("{value}", &signed(value)));
        }
    }
    if parts.is_empty() {
        tr("No effect").to_owned()
    } else {
        parts.join(", ")
    }
}

fn kind_text(kind: NewsKind) -> &'static str {
    match kind {
        NewsKind::News => tr("News"),
        NewsKind::Mail => tr("Team mail"),
        NewsKind::Event => tr("Team event"),
        NewsKind::Media => tr("Media question"),
    }
}

impl Shell {
    /// The Inbox tab: the player's mails, newest first, with the answers they offer.
    pub(super) fn career_inbox(&self, league: &League, me: &str, p: Palette, cx: &mut Context<Self>) -> Div {
        let mails = mails_of(league, me);
        if mails.is_empty() {
            return v_flex().child(div().text_size(px(13.)).text_color(p.text_muted).child(tr(
                "No mails yet. After each round the team, its departments and the press may write to you.",
            )));
        }
        let read = league.player(me).map_or(0, |pl| pl.mail_read);
        let busy = self.career.busy.is_some();
        let unread = read_upto(league, me);
        let rows = mails.into_iter().take(SHOWN).map(|n| {
            let new = n.id > read;
            let mine = n.to.as_deref().is_none_or(|to| to == me);
            let race = league
                .seasons
                .iter()
                .find(|s| s.year == n.year)
                .and_then(|s| s.round(n.round))
                .map(|r| r.race.clone());
            let when = match race {
                Some(race) if n.round > 0 => tr_owned("{year} · round {round}, {race}")
                    .replace("{year}", &n.year.to_string())
                    .replace("{round}", &n.round.to_string())
                    .replace("{race}", &race),
                _ => n.year.to_string(),
            };
            let answers = match n.answered {
                Some(i) => {
                    let text = n
                        .choices
                        .get(usize::from(i))
                        .map_or_else(String::new, |c| tr_owned(&c.template));
                    v_flex().child(
                        div()
                            .text_size(px(12.))
                            .text_color(p.text_muted)
                            .child(tr_owned("You answered: {answer}").replace("{answer}", &text)),
                    )
                }
                None if n.choices.is_empty() => v_flex(),
                None => v_flex()
                    .gap(px(4.))
                    .children(n.choices.iter().enumerate().map(|(i, c)| {
                        let (news, choice) = (n.id, i as u8);
                        h_flex()
                            .gap(px(10.))
                            .items_center()
                            .child(
                                Button::new(SharedString::from(format!("mail-{news}-{choice}")))
                                    .outline()
                                    .small()
                                    .label(tr_owned(&c.template))
                                    .disabled(busy || !mine)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.career_act(Action::Answer { news, choice }, cx)
                                    })),
                            )
                            .child(
                                div()
                                    .text_size(px(11.))
                                    .text_color(p.text_muted)
                                    .child(effect_text(&c.effect)),
                            )
                    })),
            };
            v_flex()
                .gap(px(6.))
                .py(px(8.))
                .px(px(6.))
                .border_t_1()
                .border_color(p.border)
                .when(new, |d| d.bg(p.selected))
                .child(
                    h_flex()
                        .gap(px(8.))
                        .items_center()
                        .child(tag(kind_text(n.kind), p))
                        .when(new, |d| {
                            d.child(
                                div()
                                    .text_size(px(11.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(p.accent)
                                    .child(tr("Unread")),
                            )
                        })
                        .child(div().text_size(px(12.)).text_color(p.text_muted).child(when)),
                )
                .child(
                    div()
                        .text_size(px(13.))
                        .when(new, |d| d.font_weight(FontWeight::MEDIUM))
                        .child(news_text(n)),
                )
                .child(answers)
        });
        v_flex()
            .gap(px(6.))
            .when(unread.is_some(), |d| {
                d.child(
                    h_flex().child(
                        Button::new("mail-read-all")
                            .ghost()
                            .small()
                            .label(tr("Mark all as read"))
                            .disabled(busy)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if let Some(upto) = unread {
                                    this.career_act(Action::ReadMail { upto }, cx);
                                }
                            })),
                    ),
                )
            })
            .child(v_flex().children(rows))
    }
}

#[cfg(test)]
mod tests {
    use swofty_core::league::{
        Action, Actor, Choice, Effect, League, News, NewsKind, Setup, aimarket, apply, inbox,
    };

    use super::{effect_text, mails_of, news_text, read_upto};

    fn actor(id: &str, owner: bool) -> Actor {
        Actor {
            id: id.into(),
            name: id.into(),
            owner,
        }
    }

    fn mail(id: u32, to: Option<&str>, template: &str, args: &[(&str, &str)]) -> News {
        let args: Vec<(String, String)> = args.iter().map(|(k, v)| ((*k).into(), (*v).into())).collect();
        News {
            id,
            kind: NewsKind::Mail,
            to: to.map(Into::into),
            template: Some(template.into()),
            text: inbox::render(template, &args),
            args,
            ..News::plain(2016, 1, String::new())
        }
    }

    #[test]
    fn mails_render_from_their_templates() {
        let m = mail(
            1,
            Some("u-anna"),
            inbox::T_MEDIA_DNF,
            &[("race", "Australian Grand Prix")],
        );
        // English: the template filled is the English text.
        assert_eq!(news_text(&m), m.text);
        assert!(news_text(&m).contains("Australian Grand Prix"));
        // An item from before templates shows its text.
        let old = News::plain(2016, 1, "Old news".into());
        assert_eq!(news_text(&old), "Old news");
    }

    #[test]
    fn ai_market_news_render_in_the_launchers_language() {
        // A dynamic line-up's season change, as the core writes its news.
        let mut l = League::start(Setup::default(), &actor("u-anna", true), Some(7), 4).unwrap();
        apply(
            &mut l,
            &actor("u-anna", true),
            Action::Seat {
                slot: 2,
                replaces: None,
            },
        )
        .unwrap();
        apply(&mut l, &actor("u-anna", true), Action::Begin).unwrap();
        let mut options = l.options.clone();
        options.lineups = swofty_core::league::LineupMode::Dynamic;
        apply(&mut l, &actor("u-anna", true), Action::SetOptions { options }).unwrap();
        while l.phase == swofty_core::league::Phase::Racing {
            apply(&mut l, &actor("u-anna", true), Action::Simulate { round: None }).unwrap();
        }
        apply(&mut l, &actor("u-anna", true), Action::NextSeason).unwrap();
        let moves: Vec<&News> = l
            .news
            .iter()
            .filter(|n| {
                n.template
                    .as_deref()
                    .is_some_and(|t| aimarket::TEMPLATES.contains(&t))
            })
            .collect();
        assert!(!moves.is_empty());
        let retired = moves
            .iter()
            .find(|n| n.template.as_deref() == Some(aimarket::T_RETIRED))
            .expect("Nico Rosberg retires after 2016");
        crate::i18n::set(None);
        assert_eq!(news_text(retired), retired.text);
        crate::i18n::set(Some("de"));
        for n in &moves {
            let text = news_text(n);
            assert!(!text.contains('{'), "{text}");
            assert_ne!(text, n.text, "German: {text}");
        }
        assert!(news_text(retired).contains("beendet nach der Saison 2016"));
        crate::i18n::set(None);
    }

    #[test]
    fn effects_read_as_changes() {
        assert_eq!(effect_text(&Effect::default()), "No effect");
        let bold = Effect {
            pace: 0.3,
            recognition: -1.0,
            ..Effect::default()
        };
        assert_eq!(effect_text(&bold), "-1 recognition, +0.3 pace");
        let rp = Effect {
            rp: 40,
            ..Effect::default()
        };
        assert_eq!(effect_text(&rp), "+40 resource points");
    }

    #[test]
    fn the_inbox_shows_only_the_players_mails_and_counts_unread() {
        let anna = actor("u-anna", true);
        let mut l = League::start(Setup::default(), &anna, Some(7), 1).unwrap();
        apply(&mut l, &actor("u-ben", false), Action::Enter { number: Some(8) }).unwrap();
        l.news.push(News::plain(2016, 0, "Plain news".into()));
        l.news.push(mail(
            1,
            Some("u-anna"),
            inbox::T_ENGINEER,
            &[("part", "Aerodynamics"), ("race", "Bahrain")],
        ));
        l.news.push(mail(
            2,
            Some("u-ben"),
            "{team} would like to meet you.",
            &[("team", "Ferrari")],
        ));
        l.news.push(News {
            choices: vec![Choice {
                template: inbox::T_DEPT_ACCEPT.into(),
                effect: Effect::default(),
            }],
            kind: NewsKind::Event,
            ..mail(
                3,
                Some("u-anna"),
                inbox::T_DEPT_TRACK,
                &[("race", "Bahrain"), ("rp", "40")],
            )
        });
        l.news_seq = 3;
        let ids: Vec<u32> = mails_of(&l, "u-anna").iter().map(|n| n.id).collect();
        assert_eq!(ids, vec![3, 1], "newest first, no plain news, not Ben's");
        assert_eq!(read_upto(&l, "u-anna"), Some(3));
        assert_eq!(read_upto(&l, "u-ben"), Some(2));
        apply(&mut l, &anna, Action::ReadMail { upto: 3 }).unwrap();
        assert_eq!(read_upto(&l, "u-anna"), None);
        assert_eq!(read_upto(&l, "u-ben"), Some(2));
    }
}
