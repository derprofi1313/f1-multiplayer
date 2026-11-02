//! The player's driver and career record, the driver's creation (nationality and archetype) and
//! the AI drivers' form.

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::menu::{DropdownMenu, PopupMenuItem};
use gpui_kit::component::{Disableable, Sizable, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use swofty_core::league::{self, Action, Archetype, Focus, League, Phase, driver, progress};

use crate::i18n::{tr, tr_owned};
use crate::pages::label;
use crate::shell::Shell;
use crate::theme::Palette;

impl Shell {
    /// The player's career: totals, every season, and this season's duels.
    pub(super) fn career_record(&self, league: &League, me: &str, p: Palette) -> Div {
        let s = league::career_stats(league, me);
        let tile = |label_text: &'static str, value: String| {
            v_flex()
                .min_w(px(90.))
                .p(px(10.))
                .gap(px(2.))
                .rounded(px(6.))
                .border_1()
                .border_color(p.border)
                .child(
                    div()
                        .text_size(px(18.))
                        .font_weight(FontWeight::BOLD)
                        .child(value),
                )
                .child(label(label_text, p))
        };
        let totals = h_flex()
            .gap(px(8.))
            .flex_wrap()
            .child(tile(tr("STARTS"), s.starts.to_string()))
            .child(tile(tr("WINS"), s.wins.to_string()))
            .child(tile(tr("PODIUMS"), s.podiums.to_string()))
            .child(tile(tr("POLES"), s.poles.to_string()))
            .child(tile(tr("FASTEST LAPS"), s.fastest_laps.to_string()))
            .child(tile(tr("POINTS"), format!("{}", s.points)))
            .child(tile(tr("TITLES"), s.titles.to_string()))
            .child(tile(
                tr("BEST FINISH"),
                if s.best_finish > 0 {
                    format!("P{}", s.best_finish)
                } else {
                    "-".to_owned()
                },
            ));
        let seasons = v_flex().children(s.seasons.iter().map(|l| {
            h_flex()
                .gap(px(10.))
                .py(px(4.))
                .px(px(6.))
                .border_t_1()
                .border_color(p.border)
                .child(div().w(px(50.)).text_size(px(13.)).child(l.year.to_string()))
                .child(div().w(px(170.)).text_size(px(13.)).child(l.team.clone()))
                .child(
                    div()
                        .w(px(60.))
                        .text_size(px(13.))
                        .child(format!("P{}", l.position)),
                )
                .child(
                    div()
                        .w(px(70.))
                        .text_size(px(13.))
                        .child(tr_owned("{points} pts").replace("{points}", &l.points.to_string())),
                )
                .child(div().text_size(px(12.)).text_color(p.text_muted).child({
                    let mut line = tr_owned("{wins} wins · {podiums} podiums · {poles} poles · {starts} starts")
                        .replace("{wins}", &l.wins.to_string())
                        .replace("{podiums}", &l.podiums.to_string())
                        .replace("{poles}", &l.poles.to_string())
                        .replace("{starts}", &l.starts.to_string());
                    if !l.complete {
                        line.push_str(" · ");
                        line.push_str(tr("running"));
                    }
                    line
                }))
        }));
        let season = league.season();
        let duel = |h: league::HeadToHead, template: &str| {
            div().text_size(px(13.)).child(
                tr_owned(template)
                    .replace("{name}", &h.other_name)
                    .replace("{q1}", &h.quali.0.to_string())
                    .replace("{q2}", &h.quali.1.to_string())
                    .replace("{r1}", &h.race.0.to_string())
                    .replace("{r2}", &h.race.1.to_string())
                    .replace("{p1}", &h.points.0.to_string())
                    .replace("{p2}", &h.points.1.to_string()),
            )
        };
        let entry = season.player_entry(me);
        let mate = entry
            .and_then(|e| league::team_mate(season, &e.id).map(|m| (e, m)))
            .and_then(|(e, m)| league::head_to_head(season, &e.id, &m.id));
        let rival = entry
            .zip(league.player(me).and_then(|pl| pl.rival.clone()))
            .and_then(|(e, r)| league::head_to_head(season, &e.id, &r));
        v_flex()
            .gap(px(12.))
            .child(totals)
            .when(mate.is_some() || rival.is_some(), |d| {
                d.child(label(tr_owned("{year} DUELS").replace("{year}", &season.year.to_string()), p))
                    .when_some(mate, |d, h| {
                        d.child(duel(
                            h,
                            "Team-mate {name}: qualifying {q1}:{q2}, races {r1}:{r2}, points {p1}:{p2}",
                        ))
                    })
                    .when_some(rival, |d, h| {
                        d.child(duel(h, "Rival {name}: qualifying {q1}:{q2}, races {r1}:{r2}, points {p1}:{p2}"))
                    })
            })
            .child(label(tr("SEASONS"), p))
            .child(seasons)
    }

    /// The player's driver: the four ratings next to the team-mate's and the rival's, recognition,
    /// the contract and the earnings.
    pub(super) fn career_driver(&self, league: &League, me: &str, p: Palette) -> Div {
        let Some(pl) = league.player(me) else {
            return v_flex();
        };
        let season = league.season();
        let mine = pl
            .attributes
            .unwrap_or_else(|| league::Attributes::from_rating(pl.rating));
        let entry = season.player_entry(me);
        let others: Vec<(String, league::Attributes)> = entry
            .and_then(|e| league::team_mate(season, &e.id))
            .into_iter()
            .chain(pl.rival.as_deref().and_then(|r| season.entry(r)))
            .filter_map(|e| {
                let a = match &e.player {
                    Some(id) => league.player(id).and_then(|x| x.attributes)?,
                    None => league::driver_attributes(season.year, &e.id)?,
                };
                Some((e.name.clone(), a))
            })
            .collect();
        let bar = |value: f32| {
            let w = ((value - 40.0) / 59.0).clamp(0.0, 1.0) * 160.0;
            div()
                .w(px(160.))
                .h(px(8.))
                .rounded(px(4.))
                .bg(p.border)
                .child(div().w(px(w)).h(px(8.)).rounded(px(4.)).bg(p.accent))
        };
        let row = |name: &'static str, get: fn(&league::Attributes) -> f32| {
            h_flex()
                .gap(px(10.))
                .py(px(4.))
                .items_center()
                .border_t_1()
                .border_color(p.border)
                .child(div().w(px(110.)).text_size(px(13.)).child(name))
                .child(bar(get(&mine)))
                .child(
                    div()
                        .w(px(50.))
                        .text_size(px(13.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(format!("{:.1}", get(&mine))),
                )
                .children(others.iter().map(|(_, a)| {
                    div()
                        .w(px(150.))
                        .text_size(px(12.))
                        .text_color(p.text_muted)
                        .child(format!("{:.1}", get(a)))
                }))
        };
        let header = h_flex()
            .gap(px(10.))
            .child(div().w(px(110.)))
            .child(div().w(px(160.)))
            .child(div().w(px(50.)).child(label(tr("YOU"), p)))
            .children(others.iter().map(|(n, _)| {
                div()
                    .w(px(150.))
                    .text_size(px(12.))
                    .text_color(p.text_muted)
                    .child(n.clone())
            }));
        let team = entry
            .and_then(|e| season.team(e.slot))
            .map_or_else(|| tr("no team").to_owned(), |t| t.name.clone());
        let status = if pl.number_one {
            tr_owned("{team}'s number one driver: a tenth more resource points.").replace("{team}", &team)
        } else if pl.recognition >= progress::NUMBER_ONE {
            tr("Valued highly; score more points than your team-mate to become the number one driver.")
                .to_owned()
        } else if pl.recognition < progress::RECOGNITION_RENEW {
            tr_owned("{team} isn't happy: below {limit}, a team lets the contract run out.")
                .replace("{team}", &team)
                .replace("{limit}", &format!("{:.0}", progress::RECOGNITION_RENEW))
        } else {
            tr_owned(
                "Meet the team's goals and beat your team-mate; from {limit} on, and ahead of your team-mate, \
                 you're the number one driver.",
            )
            .replace("{limit}", &format!("{:.0}", progress::NUMBER_ONE))
        };
        let focus = match pl.focus {
            Some(Focus::Salary) => tr(" · you asked for more salary"),
            Some(Focus::Development) => tr(" · you have a say in the development (+20% resource points)"),
            None => "",
        };
        let clause = pl
            .number_one_until
            .filter(|y| *y >= season.year)
            .map(|y| tr_owned("Number-one clause until {year}.").replace("{year}", &y.to_string()));
        let concentration = driver::focus_of(pl);
        v_flex()
            .gap(px(12.))
            .child(
                h_flex()
                    .gap(px(16.))
                    .items_center()
                    .child(
                        div()
                            .text_size(px(28.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(p.accent)
                            .child(format!("{:.1}", pl.rating)),
                    )
                    .child(
                        v_flex()
                            .child(div().text_size(px(15.)).font_weight(FontWeight::SEMIBOLD).child(format!(
                                "{} #{} · {team}",
                                pl.name, pl.number
                            )))
                            .when_some(profile_line(pl.nationality.as_deref(), pl.archetype), |d, t| {
                                d.child(div().text_size(px(12.)).child(t))
                            })
                            .child(div().text_size(px(12.)).text_color(p.text_muted).child(tr(
                                "Overall rating, weighed as the game weighs it: pace twice, racecraft once, \
                                 experience and awareness a quarter each.",
                            ))),
                    ),
            )
            .child(
                v_flex()
                    .child(header)
                    .child(row(tr("Experience"), |a| a.experience))
                    .child(row(tr("Racecraft"), |a| a.racecraft))
                    .child(row(tr("Awareness"), |a| a.awareness))
                    .child(row(tr("Pace"), |a| a.pace)),
            )
            .child(div().text_size(px(12.)).text_color(p.text_muted).child(tr(
                "Every session you drive moves them: experience with each one, pace with qualifying and practice \
                 against your team-mate, racecraft with places gained, awareness with clean races (penalties and \
                 crashes cost it). Simulated sessions only add experience.",
            )))
            .child(
                h_flex()
                    .gap(px(10.))
                    .py(px(4.))
                    .items_center()
                    .border_t_1()
                    .border_color(p.border)
                    .child(div().w(px(110.)).text_size(px(13.)).child(tr("Focus")))
                    .child(bar(concentration))
                    .child(
                        div()
                            .w(px(50.))
                            .text_size(px(13.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format!("{concentration:.1}")),
                    ),
            )
            .child(div().text_size(px(12.)).text_color(p.text_muted).child(focus_text()))
            .child(label(tr("TEAM"), p))
            .child(
                h_flex()
                    .gap(px(10.))
                    .items_center()
                    .child(div().w(px(110.)).text_size(px(13.)).child(tr("Recognition")))
                    .child(bar(40.0 + pl.recognition / 100.0 * 59.0))
                    .child(
                        div()
                            .text_size(px(13.))
                            .child(tr_owned("{value} of 100").replace("{value}", &format!("{:.0}", pl.recognition))),
                    ),
            )
            .child(div().text_size(px(12.)).text_color(p.text_muted).child(status))
            .child(label(tr("CONTRACT"), p))
            .child(
                div().text_size(px(13.)).child(
                    tr_owned("{salary} a season + {bonus} a point, until {year}")
                        .replace("{salary}", &progress::money(u64::from(pl.salary_k)))
                        .replace("{bonus}", &progress::money(u64::from(pl.bonus_k)))
                        .replace("{year}", &pl.contract_until.map_or("?".to_owned(), |y| y.to_string()))
                        + focus,
                ),
            )
            .when_some(clause, |d, t| d.child(div().text_size(px(13.)).child(t)))
            .child(
                div()
                    .text_size(px(13.))
                    .child(tr_owned("Earned in the career: {money}").replace("{money}", &progress::money(pl.earnings_k))),
            )
    }

    /// Pick the nationality or archetype: in the start form (kept until the career is made), or for
    /// the player of the career on show (sent at once).
    fn career_pick(&mut self, target: Target, pick: Pick, cx: &mut Context<Self>) {
        match target {
            Target::Form => {
                match pick {
                    Pick::Nationality(n) => self.career.profile.nationality = Some(n),
                    Pick::Archetype(a) => self.career.profile.archetype = Some(a),
                }
                cx.notify();
            }
            Target::Career => self.career_act(pick.action(), cx),
        }
    }

    /// The nationality dropdown and the archetype buttons.
    pub(super) fn career_profile_picker(
        &self,
        target: Target,
        id: &'static str,
        current: ProfileForm,
        p: Palette,
        cx: &mut Context<Self>,
    ) -> Div {
        let busy = target == Target::Career && self.career.busy.is_some();
        let shell = cx.entity().downgrade();
        let nationality = Button::new(SharedString::from(format!("{id}-nationality")))
            .outline()
            .small()
            .w(px(230.))
            .label(
                current
                    .nationality
                    .and_then(driver::nationality_name)
                    .map_or(tr("Pick a nationality"), tr),
            )
            .dropdown_caret(true)
            .disabled(busy)
            .dropdown_menu_with_anchor(Anchor::TopLeft, move |mut menu, _, _| {
                menu = menu.scrollable(true).max_h(px(340.));
                for (code, name) in nationality_choices() {
                    let shell = shell.clone();
                    menu = menu.item(
                        PopupMenuItem::new(name)
                            .checked(current.nationality == Some(code))
                            .on_click(move |_, _, cx| {
                                shell
                                    .update(cx, |s, cx| s.career_pick(target, Pick::Nationality(code), cx))
                                    .ok();
                            }),
                    );
                }
                menu
            });
        let archetypes = h_flex()
            .gap(px(6.))
            .flex_wrap()
            .children(ARCHETYPES.iter().map(|&a| {
                let b = Button::new(SharedString::from(format!("{id}-archetype-{}", a.name())))
                    .small()
                    .label(archetype_text(a))
                    .disabled(busy)
                    .on_click(
                        cx.listener(move |this, _, _, cx| this.career_pick(target, Pick::Archetype(a), cx)),
                    );
                if current.archetype == Some(a) {
                    b.primary()
                } else {
                    b.outline()
                }
            }));
        let line = |text: &'static str| div().w(px(110.)).text_size(px(13.)).child(text);
        v_flex()
            .gap(px(6.))
            .child(
                h_flex()
                    .gap(px(6.))
                    .items_center()
                    .child(line(tr("Nationality")))
                    .child(nationality),
            )
            .child(
                h_flex()
                    .gap(px(6.))
                    .items_center()
                    .flex_wrap()
                    .child(line(tr("Start as")))
                    .child(archetypes),
            )
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(p.text_muted)
                    .child(tr("The archetype sets your starting rating and focus.")),
            )
    }

    /// The driver card while the player can still change it: in Setup, and in the first season
    /// until a session of theirs is counted.
    pub(super) fn career_profile_card(
        &self,
        league: &League,
        me: &str,
        p: Palette,
        cx: &mut Context<Self>,
    ) -> Option<Div> {
        if !matches!(league.phase, Phase::Setup | Phase::Racing) {
            return None;
        }
        let pl = league.player(me).filter(|pl| !pl.left)?;
        if driver::has_driven(league, me) {
            return None;
        }
        let current = ProfileForm {
            nationality: pl.nationality.as_deref().and_then(static_code),
            archetype: pl.archetype,
        };
        Some(
            v_flex()
                .p(px(14.))
                .gap(px(8.))
                .rounded(px(6.))
                .border_1()
                .border_color(p.border)
                .child(
                    div()
                        .text_size(px(14.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(tr("Create your driver")),
                )
                .child(div().text_size(px(12.)).text_color(p.text_muted).child(tr(
                    "Where you're from and how you start. You can change it until your first session is counted.",
                )))
                .child(self.career_profile_picker(Target::Career, "career-profile", current, p, cx)),
        )
    }
}

/// Where a pick of the driver goes.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Target {
    /// The start form: kept, then sent with the new career.
    Form,
    /// The career on show: sent at once.
    Career,
}

#[derive(Clone, Copy)]
enum Pick {
    Nationality(&'static str),
    Archetype(Archetype),
}

impl Pick {
    fn action(self) -> Action {
        match self {
            Pick::Nationality(n) => Action::Profile {
                nationality: Some(n.to_owned()),
                archetype: None,
            },
            Pick::Archetype(a) => Action::Profile {
                nationality: None,
                archetype: Some(a),
            },
        }
    }
}

/// The driver picked in the start form.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ProfileForm {
    pub(super) nationality: Option<&'static str>,
    pub(super) archetype: Option<Archetype>,
}

impl ProfileForm {
    /// The `Profile` action for a new career, if anything was picked.
    pub(super) fn action(self) -> Option<Action> {
        (self.nationality.is_some() || self.archetype.is_some()).then(|| Action::Profile {
            nationality: self.nationality.map(str::to_owned),
            archetype: self.archetype,
        })
    }
}

const ARCHETYPES: [Archetype; 3] = [Archetype::Rookie, Archetype::Experienced, Archetype::Veteran];

/// "Rookie · rated 70".
fn archetype_text(a: Archetype) -> String {
    let text = match a {
        Archetype::Rookie => tr("Rookie · rated {rating}"),
        Archetype::Experienced => tr("Experienced · rated {rating}"),
        Archetype::Veteran => tr("Veteran · rated {rating}"),
    };
    text.replace("{rating}", &format!("{:.0}", driver::start_rating(Some(a))))
}

/// "From Germany · Rookie" under the driver's name, when there is a profile.
fn profile_line(nationality: Option<&str>, archetype: Option<Archetype>) -> Option<String> {
    let from = nationality
        .and_then(driver::nationality_name)
        .map(|n| tr_owned("From {country}").replace("{country}", tr(n)));
    let kind = archetype.map(|a| {
        match a {
            Archetype::Rookie => tr("Started as a rookie"),
            Archetype::Experienced => tr("Started as an experienced driver"),
            Archetype::Veteran => tr("Started as a veteran"),
        }
        .to_owned()
    });
    let parts: Vec<String> = from.into_iter().chain(kind).collect();
    (!parts.is_empty()).then(|| parts.join(" · "))
}

fn focus_text() -> String {
    tr_owned(
        "Focus isn't part of the overall rating. It rises with each clean race (+{clean}) and falls when you \
         retire through your own fault (-{dnf}); in a simulated race, more focus makes a retirement less likely.",
    )
    .replace("{clean}", &format!("{:.1}", driver::FOCUS_CLEAN))
    .replace("{dnf}", &format!("{:.1}", driver::FOCUS_DNF))
}

/// The code of [`driver::NATIONALITIES`] for `code`, as a `'static` string.
fn static_code(code: &str) -> Option<&'static str> {
    driver::NATIONALITIES
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(c, _)| *c)
}

/// Every nationality to pick: the code and the name in the launcher's language, sorted by that
/// name (umlauts sort as their vowel).
pub(super) fn nationality_choices() -> Vec<(&'static str, &'static str)> {
    nationality_choices_by(tr)
}

/// [`nationality_choices`] with the names translated by `translate`.
fn nationality_choices_by(translate: fn(&'static str) -> &'static str) -> Vec<(&'static str, &'static str)> {
    let mut list: Vec<(&'static str, &'static str)> = driver::NATIONALITIES
        .iter()
        .map(|(code, name)| (*code, translate(name)))
        .collect();
    list.sort_by_cached_key(|(_, name)| sort_key(name));
    list
}

fn sort_key(name: &str) -> String {
    name.chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'ä' => "a".to_owned(),
            'ö' => "o".to_owned(),
            'ü' => "u".to_owned(),
            'ß' => "ss".to_owned(),
            c => c.to_string(),
        })
        .collect()
}

/// An AI driver's form this season as an arrow: up (better than the season data) or down, with
/// the rating change; `None` for no change worth showing.
pub(super) fn form_arrow(total: f32) -> Option<(bool, String)> {
    if total.abs() < 0.05 {
        return None;
    }
    let up = total > 0.0;
    Some((up, format!("{} {:.1}", if up { "▲" } else { "▼" }, total.abs())))
}

/// The form column of the drivers table: the AI driver's rating change this season.
pub(super) fn form_cell(season: &league::Season, entry: &str, p: Palette) -> Div {
    let arrow = season
        .ai_form
        .iter()
        .find(|f| f.entry == entry)
        .and_then(|f| form_arrow(f.total()));
    div()
        .w(px(50.))
        .text_size(px(12.))
        .when_some(arrow, |d, (up, text)| {
            d.text_color(if up { p.success } else { p.error }).child(text)
        })
}

#[cfg(test)]
mod tests {
    use swofty_core::league::{Action, Archetype, driver};

    use super::{ProfileForm, form_arrow, nationality_choices_by, sort_key};

    /// A German translation of a few names, without switching the launcher's language (the
    /// language is global, and other tests switch it).
    fn some_german(name: &'static str) -> &'static str {
        match name {
            "Austria" => "Österreich",
            "Egypt" => "Ägypten",
            "Germany" => "Deutschland",
            "United Kingdom" => "Vereinigtes Königreich",
            other => other,
        }
    }

    fn english(name: &'static str) -> &'static str {
        name
    }

    #[test]
    fn nationality_list_is_sorted_and_translated() {
        for translate in [some_german as fn(&'static str) -> &'static str, english] {
            let list = nationality_choices_by(translate);
            assert_eq!(list.len(), driver::NATIONALITIES.len());
            let keys: Vec<String> = list.iter().map(|(_, n)| sort_key(n)).collect();
            assert!(keys.windows(2).all(|w| w[0] <= w[1]), "not sorted: {keys:?}");
            for (code, name) in driver::NATIONALITIES {
                let shown = list.iter().find(|(c, _)| c == code).expect("every code").1;
                assert_eq!(shown, translate(name));
            }
        }
        let list = nationality_choices_by(some_german);
        let at = |code: &str| list.iter().position(|(c, _)| *c == code).expect("listed");
        // Deutschland sorts under D; Österreich as "osterreich", after Oman and before Pakistan;
        // Ägypten as "agypten", first.
        assert!(at("DNK") < at("DEU") && at("DEU") < at("DOM"));
        assert!(at("OMN") < at("AUT") && at("AUT") < at("PAK"));
        assert_eq!(list[0].0, "EGY");
    }

    #[test]
    fn the_start_form_sends_what_was_picked() {
        assert_eq!(ProfileForm::default().action(), None);
        let form = ProfileForm {
            nationality: Some("DEU"),
            archetype: Some(Archetype::Veteran),
        };
        assert_eq!(
            form.action(),
            Some(Action::Profile {
                nationality: Some("DEU".to_owned()),
                archetype: Some(Archetype::Veteran),
            })
        );
        let only_kind = ProfileForm {
            nationality: None,
            archetype: Some(Archetype::Rookie),
        };
        assert_eq!(
            only_kind.action(),
            Some(Action::Profile {
                nationality: None,
                archetype: Some(Archetype::Rookie),
            })
        );
    }

    #[test]
    fn form_arrows_point_the_way_the_rating_moved() {
        assert_eq!(form_arrow(0.0), None);
        assert_eq!(form_arrow(0.04), None);
        assert_eq!(form_arrow(1.26), Some((true, "▲ 1.3".to_owned())));
        assert_eq!(form_arrow(-0.8), Some((false, "▼ 0.8".to_owned())));
    }
}
