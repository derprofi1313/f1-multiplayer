//! The race weekend views: this weekend's objectives with the rivalry meter, the rules sheet the
//! players agreed on (and its editor for the owner), and the practice programmes with what the
//! game's telemetry showed.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Mutex;

use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Disableable, Icon, Sizable, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use swofty_core::league::weekend::{self, AI_DIFFICULTY_MAX};
use swofty_core::league::{
    Action, League, LineupMode, Objective, ObjectiveKind, Options, Phase, PracticeFormat, PracticeLine,
    Programme, QualiFormat, RaceReport, Round, SessionKind, WeekendRules, progress, udp,
};

use crate::i18n::{tr, tr_owned};
use crate::pages::label;
use crate::shell::Shell;
use crate::theme::Palette;

/// The owner's rules sheet while it is being edited.
#[derive(Default)]
pub(crate) struct RulesEditor {
    /// The rules being edited; `None` while the editor is closed.
    draft: Option<WeekendRules>,
}

/// The practice programmes, in the order the game lists them.
const PROGRAMMES: [Programme; 6] = [
    Programme::TrackAcclimatisation,
    Programme::QualifyingPace,
    Programme::TyreManagement,
    Programme::ErsManagement,
    Programme::FuelManagement,
    Programme::RaceStrategy,
];

/// A lap counts for the programmes within this of the best clean lap (as the core grades them).
const PACE_WINDOW: f32 = 1.07;

/// The AI difficulty a rules sheet starts from, and its step.
const AI_START: u8 = 80;
const AI_STEP: u8 = 5;

// ---------------------------------------------------------------------------------------------
// Pure parts (tested)

/// A setting's value as the launcher shows it. The game's option names go through the German
/// table, first with the setting's name in front ("Weather: Clear"), for words that read
/// differently in another setting.
fn value_text(label: &str, value: &str) -> String {
    let key = format!("{label}: {value}");
    let text = tr_owned(&key);
    if text == key { tr_owned(value) } else { text }
}

/// The agreed rules as translated (label, value) rows; rules not agreed are left out.
pub(super) fn rules_rows(rules: &WeekendRules) -> Vec<(String, String)> {
    weekend::describe(rules)
        .into_iter()
        .map(|(label, value)| (tr_owned(label), value_text(label, &value)))
        .collect()
}

/// One objective of the checklist.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct ObjectiveRow {
    /// `None` open, `Some(true)` met, `Some(false)` missed.
    pub met: Option<bool>,
    pub text: String,
    pub reward: String,
}

/// "1" for 1.0, "0.5" for 0.5.
fn short_number(x: f32) -> String {
    if (x - x.round()).abs() < 0.05 {
        format!("{x:.0}")
    } else {
        format!("{x:.1}")
    }
}

/// The checklist rows of `objectives`, with what each pays.
pub(super) fn objective_rows(objectives: &[&Objective], rival: Option<&str>) -> Vec<ObjectiveRow> {
    objectives
        .iter()
        .map(|o| {
            let target = o.target.to_string();
            let text = match (o.kind, rival) {
                (ObjectiveKind::PracticeGrade, _) => tr_owned("Practice grade {target} or better"),
                (ObjectiveKind::QualiPosition, _) => tr_owned("Qualify P{target} or better"),
                (ObjectiveKind::RacePosition, _) => tr_owned("Finish P{target} or better"),
                (ObjectiveKind::BeatTeamMateQuali, _) => tr_owned("Qualify ahead of your team-mate"),
                (ObjectiveKind::BeatTeamMateRace, _) => tr_owned("Finish ahead of your team-mate"),
                (ObjectiveKind::BeatRival, Some(r)) => {
                    tr_owned("Finish ahead of your rival {rival}").replace("{rival}", r)
                }
                (ObjectiveKind::BeatRival, None) => tr_owned("Finish ahead of your rival"),
                (ObjectiveKind::CleanRace, _) => tr_owned("Finish the race without a penalty"),
            }
            .replace("{target}", &target);
            let mut reward = Vec::new();
            if o.reward_rp > 0 {
                reward.push(tr_owned("+{rp} resource points").replace("{rp}", &o.reward_rp.to_string()));
            }
            if o.reward_recognition > 0.0 {
                reward.push(tr_owned("+{n} recognition").replace("{n}", &short_number(o.reward_recognition)));
            }
            ObjectiveRow {
                met: o.met,
                text,
                reward: reward.join(" · "),
            }
        })
        .collect()
}

/// What a practice session's laps showed, for the programme rows.
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct Figures {
    pub laps: usize,
    pub clean: usize,
    /// Clean laps within [`PACE_WINDOW`] of the best.
    pub at_pace: usize,
    pub best_ms: Option<u32>,
    /// Averages over the laps at pace that sent them.
    pub ers_deployed_mj: Option<f32>,
    pub ers_left_mj: Option<f32>,
    pub fuel_kg: Option<f32>,
    pub wear_pct: Option<f32>,
}

/// The figures of a practice `report`: from its lap telemetry, else from its lap list.
pub(super) fn figures(report: &RaceReport) -> Figures {
    let mut f = Figures::default();
    let times: Vec<(u32, bool)> = if report.lap_telemetry.is_empty() {
        report.laps.iter().map(|l| (l.ms, l.valid)).collect()
    } else {
        report.lap_telemetry.iter().map(|l| (l.ms, l.valid)).collect()
    };
    f.laps = times.len();
    let clean: Vec<u32> = times
        .iter()
        .filter(|(ms, valid)| *valid && *ms > 0)
        .map(|(ms, _)| *ms)
        .collect();
    f.clean = clean.len();
    f.best_ms = clean.iter().copied().min();
    let Some(best) = f.best_ms else {
        return f;
    };
    let limit = best as f32 * PACE_WINDOW;
    f.at_pace = clean.iter().filter(|ms| **ms as f32 <= limit).count();
    let at_pace: Vec<_> = report
        .lap_telemetry
        .iter()
        .filter(|l| l.valid && l.ms > 0 && l.ms as f32 <= limit)
        .collect();
    let mean = |xs: Vec<f32>| (!xs.is_empty()).then(|| xs.iter().sum::<f32>() / xs.len() as f32);
    let ers: Vec<_> = at_pace
        .iter()
        .filter(|l| l.ers_deployed_mj > 0.0 || l.ers_store_end_mj > 0.0)
        .collect();
    f.ers_deployed_mj = mean(ers.iter().map(|l| l.ers_deployed_mj).collect());
    f.ers_left_mj = mean(ers.iter().map(|l| l.ers_store_end_mj).collect());
    f.fuel_kg = mean(
        at_pace
            .iter()
            .map(|l| l.fuel_used_kg)
            .filter(|x| *x > 0.0)
            .collect(),
    );
    f.wear_pct = mean(
        at_pace
            .iter()
            .map(|l| l.tyre_wear_pct)
            .filter(|x| *x > 0.0)
            .collect(),
    );
    f
}

/// What a programme measures.
fn programme_aim(programme: Programme) -> &'static str {
    match programme {
        Programme::TrackAcclimatisation => tr("Clean laps to learn the track: eight make full marks."),
        Programme::QualifyingPace => tr("Your best lap against your team-mate's."),
        Programme::TyreManagement => tr("A long run: four or more clean laps close to your best."),
        Programme::ErsManagement => {
            tr("Three laps at pace deploying the full 4 MJ, with a tenth of the store left at the line.")
        }
        Programme::FuelManagement => tr("Three laps at pace within the fuel target, about 0.32 kg per km."),
        Programme::RaceStrategy => {
            tr("Five laps in a row at pace with little tyre wear, about 0.38% per km or less.")
        }
    }
}

/// What the telemetry showed for a programme; `None` when it showed nothing for it.
pub(super) fn programme_figure(programme: Programme, f: &Figures) -> Option<String> {
    let one = |x: f32| format!("{x:.1}");
    match programme {
        Programme::TrackAcclimatisation => (f.laps > 0).then(|| {
            tr_owned("{clean} clean laps of {laps}")
                .replace("{clean}", &f.clean.to_string())
                .replace("{laps}", &f.laps.to_string())
        }),
        Programme::QualifyingPace => f
            .best_ms
            .map(|ms| tr_owned("best lap {lap}").replace("{lap}", &progress::lap_text(ms))),
        Programme::TyreManagement => f.best_ms.map(|_| {
            tr_owned("{n} clean laps within 107% of the best").replace("{n}", &f.at_pace.to_string())
        }),
        Programme::ErsManagement => f.ers_deployed_mj.map(|d| {
            tr_owned("{deployed} MJ deployed a lap, {left} MJ left at the line")
                .replace("{deployed}", &one(d))
                .replace("{left}", &one(f.ers_left_mj.unwrap_or(0.0)))
        }),
        Programme::FuelManagement => f
            .fuel_kg
            .map(|kg| tr_owned("{kg} kg of fuel a lap").replace("{kg}", &format!("{kg:.2}"))),
        Programme::RaceStrategy => f
            .wear_pct
            .map(|w| tr_owned("{wear}% tyre wear a lap").replace("{wear}", &format!("{w:.2}"))),
    }
}

/// The figures of the practice session `session` this PC kept, read once per session.
fn kept_figures(races_dir: &Path, session: u64) -> Option<Figures> {
    static SEEN: Mutex<BTreeMap<u64, Option<Figures>>> = Mutex::new(BTreeMap::new());
    let mut seen = SEEN.lock().unwrap_or_else(|e| e.into_inner());
    seen.entry(session)
        .or_insert_with(|| {
            let bytes = std::fs::read(races_dir.join(format!("{session:016x}.json"))).ok()?;
            let report: RaceReport = serde_json::from_slice(&bytes).ok()?;
            Some(figures(&report))
        })
        .clone()
}

/// The names of a setting's values (`make` sets one value), as the core describes them: the
/// values the game has, which describe by name rather than by number.
fn value_names(make: fn(u8) -> WeekendRules, last: u8) -> Vec<(u8, String)> {
    (0..=last)
        .filter_map(|v| {
            let (_, name) = weekend::describe(&make(v)).into_iter().next()?;
            name.parse::<u8>().is_err().then_some((v, name))
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Views

/// A small mark: ✓ met, ✗ missed, an open circle otherwise.
fn mark(met: Option<bool>, p: Palette) -> Div {
    let (icon, colour) = match met {
        Some(true) => (IconName::CircleCheck, p.success),
        Some(false) => (IconName::CircleX, p.error),
        None => (IconName::Circle, p.text_placeholder),
    };
    div()
        .w(px(16.))
        .text_size(px(14.))
        .text_color(colour)
        .child(Icon::new(icon).size(px(14.)))
}

/// The checklist of `me`'s objectives in `round`, with ✓, ✗ or open marks and their rewards.
pub(super) fn objectives_list(round: &Round, me: &str, rival: Option<&str>, p: Palette) -> Div {
    let mine = swofty_core::league::objectives::of(round, me);
    v_flex()
        .gap(px(2.))
        .children(objective_rows(&mine, rival).into_iter().map(|row| {
            let missed = row.met == Some(false);
            h_flex()
                .gap(px(8.))
                .items_center()
                .child(mark(row.met, p))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_size(px(13.))
                        .when(missed, |d| d.text_color(p.text_muted))
                        .child(row.text),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(p.text_muted)
                        .child(row.reward),
                )
        }))
}

/// The rivalry meter: this season's head-to-head against the rival, −5 to +5 shown.
pub(super) fn rivalry_meter(rival: &str, score: i32, p: Palette) -> Div {
    let text = match score {
        0 => tr_owned("Level with {rival} in this season's qualifying and races"),
        s if s > 0 => tr_owned("Ahead of {rival} by {n} in this season's qualifying and races"),
        _ => tr_owned("Behind {rival} by {n} in this season's qualifying and races"),
    }
    .replace("{rival}", rival)
    .replace("{n}", &score.unsigned_abs().to_string());
    let cells = (-5..=5).map(|i: i32| {
        let lit = (i > 0 && i <= score) || (i < 0 && i >= score);
        div()
            .w(px(if i == 0 { 2. } else { 12. }))
            .h(px(10.))
            .rounded(px(2.))
            .bg(match (i, lit) {
                (0, _) => p.text_muted,
                (_, true) if score > 0 => p.success,
                (_, true) => p.warning,
                _ => p.border,
            })
    });
    h_flex()
        .gap(px(8.))
        .items_center()
        .child(h_flex().gap(px(2.)).items_center().children(cells))
        .child(div().text_size(px(12.)).text_color(p.text_muted).child(text))
}

/// The agreed rules sheet; a note when nothing is agreed.
pub(super) fn rules_summary(rules: &WeekendRules, p: Palette) -> Div {
    let rows = rules_rows(rules);
    if rows.is_empty() {
        return v_flex().child(div().text_size(px(12.)).text_color(p.text_muted).child(tr(
            "No rules agreed: everyone drives with the settings they like.",
        )));
    }
    h_flex()
        .gap(px(6.))
        .flex_wrap()
        .children(rows.into_iter().map(|(l, v)| {
            h_flex()
                .gap(px(4.))
                .px(px(8.))
                .py(px(2.))
                .rounded(px(4.))
                .border_1()
                .border_color(p.border)
                .child(div().text_size(px(12.)).text_color(p.text_muted).child(l))
                .child(div().text_size(px(12.)).font_weight(FontWeight::MEDIUM).child(v))
        }))
}

impl Shell {
    /// This weekend: the objectives with the rivalry meter, the rules sheet and the practice
    /// programmes. Only while a season runs.
    pub(super) fn career_weekend(&self, league: &League, me: &str, p: Palette) -> Div {
        let (Some(b), Some(round)) = (league.briefing(me), league.season().next_round()) else {
            return v_flex();
        };
        let player = league.player(me);
        let mut panel = v_flex()
            .p(px(14.))
            .gap(px(8.))
            .rounded(px(6.))
            .border_1()
            .border_color(p.border);
        if !b.objectives.is_empty() {
            panel = panel
                .child(label(
                    tr_owned("THIS WEEKEND'S OBJECTIVES · ROUND {round}")
                        .replace("{round}", &b.round.to_string()),
                    p,
                ))
                .child(objectives_list(round, me, b.rival.as_deref(), p));
        }
        if let (Some(rival), Some(pl)) = (&b.rival, player) {
            panel = panel.child(rivalry_meter(rival, pl.rival_score, p));
        }
        panel
            .child(label(tr("RULES SHEET"), p))
            .child(rules_summary(&b.rules, p))
            .when(b.team.is_some(), |d| {
                d.child(label(tr("PRACTICE PROGRAMMES"), p))
                    .child(self.career_programmes(round, me, p))
            })
    }

    /// The six practice programmes: each one's grade this round and what the telemetry showed.
    fn career_programmes(&self, round: &Round, me: &str, p: Palette) -> Div {
        let line: Option<&PracticeLine> = round.practice.iter().find(|l| l.player == me);
        let races_dir = self.store.root.join(udp::RACES_DIR);
        // The counted session's figures, else those of a practice that waits to be counted.
        let (figures, waiting) = match line {
            Some(l) if l.session != 0 => (kept_figures(&races_dir, l.session), false),
            Some(_) => (None, false),
            None => {
                let report = self.career.races.iter().rev().find(|r| {
                    swofty_core::league::session_kind(r.session_type) == Some(SessionKind::Practice)
                });
                (report.map(figures), report.is_some())
            }
        };
        let rows = PROGRAMMES.into_iter().map(|programme| {
            let graded = line.and_then(|l| l.programmes.iter().find(|x| x.programme == programme));
            let grade: Div = match (line, graded) {
                (_, Some(g)) => h_flex()
                    .gap(px(6.))
                    .items_center()
                    .child(
                        div().w(px(60.)).h(px(6.)).rounded(px(3.)).bg(p.border).child(
                            div()
                                .h_full()
                                .rounded(px(3.))
                                .w(px(f32::from(g.grade) * 0.6))
                                .bg(p.accent),
                        ),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .font_weight(FontWeight::MEDIUM)
                            .child(g.grade.to_string()),
                    ),
                (Some(l), None) => div()
                    .text_size(px(12.))
                    .text_color(p.text_muted)
                    .child(if l.driven {
                        tr("not graded")
                    } else {
                        tr("not simulated")
                    }),
                (None, None) => div()
                    .text_size(px(12.))
                    .text_color(p.text_muted)
                    .child(tr("open")),
            };
            let figure = figures.as_ref().and_then(|f| programme_figure(programme, f));
            h_flex()
                .gap(px(10.))
                .py(px(3.))
                .items_start()
                .border_t_1()
                .border_color(p.border)
                .child(
                    div()
                        .w(px(150.))
                        .text_size(px(13.))
                        .child(tr_owned(programme.name())),
                )
                .child(div().w(px(100.)).child(grade))
                .child(
                    v_flex()
                        .flex_1()
                        .min_w_0()
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(p.text_muted)
                                .child(programme_aim(programme)),
                        )
                        .when_some(figure, |d, f| d.child(div().text_size(px(12.)).child(f))),
                )
        });
        let foot = match line {
            Some(l) if !l.driven => Some(tr(
                "A simulated practice grades the first three programmes only; drive it in the game for all six.",
            )),
            Some(_) => Some(tr(
                "The ERS, fuel and race-strategy programmes need the game's telemetry with enough laps at pace.",
            )),
            None if waiting => Some(tr(
                "The figures are of the practice session that waits to be counted.",
            )),
            None => None,
        };
        v_flex().children(rows).when_some(foot, |d, t| {
            d.child(
                div()
                    .pt(px(4.))
                    .text_size(px(12.))
                    .text_color(p.text_muted)
                    .child(t),
            )
        })
    }

    /// The owner's "Career options": the rules sheet everyone sets in the game for each weekend,
    /// edited here and sent as `SetOptions`.
    pub(super) fn career_options(
        &self,
        league: &League,
        owner: bool,
        p: Palette,
        cx: &mut Context<Self>,
    ) -> Div {
        if !owner || league.phase == Phase::Finished {
            return v_flex();
        }
        let busy = self.career.busy.is_some();
        let panel = v_flex()
            .p(px(14.))
            .gap(px(8.))
            .rounded(px(6.))
            .border_1()
            .border_color(p.border);
        let market = lineups_row(league, busy, p, cx);
        let Some(draft) = self.career.rules.draft.clone() else {
            return panel
                .child(
                    h_flex()
                        .gap(px(8.))
                        .items_center()
                        .flex_wrap()
                        .child(label(tr("CAREER OPTIONS"), p))
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(160.))
                                .text_size(px(12.))
                                .text_color(p.text_muted)
                                .child(tr(
                                    "The rules sheet: what everyone sets in the game for each weekend.",
                                )),
                        )
                        .child(
                            Button::new("career-rules-edit")
                                .outline()
                                .small()
                                .icon(IconName::SlidersHorizontal)
                                .label(tr("Edit the rules sheet"))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    let rules = this
                                        .career
                                        .league()
                                        .map(|l| l.options.rules.clone())
                                        .unwrap_or_default();
                                    this.career.rules.draft = Some(rules);
                                    cx.notify();
                                })),
                        ),
                )
                .child(market);
        };
        let pace_in_game = league.options.game_performance;
        let ai = h_flex()
            .gap(px(6.))
            .items_center()
            .flex_wrap()
            .child(rule_label(weekend::LABEL_AI))
            .child(choice(
                "career-rule-ai-none",
                tr("Not agreed"),
                draft.ai_difficulty.is_none(),
                |r| r.ai_difficulty = None,
                cx,
            ))
            .child(
                Button::new("career-rule-ai-down")
                    .outline()
                    .small()
                    .icon(IconName::Minus)
                    .on_click(cx.listener(|this, _, _, cx| {
                        edit(this, |r| {
                            r.ai_difficulty =
                                Some(r.ai_difficulty.map_or(AI_START, |v| v.saturating_sub(AI_STEP)))
                        });
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .w(px(36.))
                    .text_size(px(13.))
                    .font_weight(FontWeight::MEDIUM)
                    .child(
                        draft
                            .ai_difficulty
                            .map_or_else(|| "–".to_owned(), |v| v.to_string()),
                    ),
            )
            .child(
                Button::new("career-rule-ai-up")
                    .outline()
                    .small()
                    .icon(IconName::Plus)
                    .on_click(cx.listener(|this, _, _, cx| {
                        edit(this, |r| {
                            r.ai_difficulty = Some(
                                r.ai_difficulty
                                    .map_or(AI_START, |v| (v + AI_STEP).min(AI_DIFFICULTY_MAX)),
                            )
                        });
                        cx.notify();
                    })),
            );
        let quali = [QualiFormat::Full, QualiFormat::Short, QualiFormat::OneShot]
            .into_iter()
            .map(|f| (f, f.name().to_owned()))
            .collect();
        let practice = [
            PracticeFormat::Full,
            PracticeFormat::OneSession,
            PracticeFormat::Short,
            PracticeFormat::None,
        ]
        .into_iter()
        .map(|f| (f, f.name().to_owned()))
        .collect();
        let on_off = vec![(true, "On".to_owned()), (false, "Off".to_owned())];
        let numbered = |make: fn(u8) -> WeekendRules, last: u8| value_names(make, last);
        let rows = v_flex()
            .gap(px(6.))
            .child(ai)
            .child(choice_row(
                "career-rule-quali",
                weekend::LABEL_QUALI,
                draft.quali,
                quali,
                |r, v| r.quali = v,
                cx,
            ))
            .child(choice_row(
                "career-rule-practice",
                weekend::LABEL_PRACTICE,
                draft.practice,
                practice,
                |r, v| r.practice = v,
                cx,
            ))
            .child(choice_row(
                "career-rule-damage",
                weekend::LABEL_DAMAGE,
                draft.damage,
                numbered(
                    |v| WeekendRules {
                        damage: Some(v),
                        ..WeekendRules::default()
                    },
                    9,
                ),
                |r, v| r.damage = v,
                cx,
            ))
            .child(choice_row(
                "career-rule-safety-car",
                weekend::LABEL_SAFETY_CAR,
                draft.safety_car,
                numbered(
                    |v| WeekendRules {
                        safety_car: Some(v),
                        ..WeekendRules::default()
                    },
                    9,
                ),
                |r, v| r.safety_car = v,
                cx,
            ))
            .child(choice_row(
                "career-rule-parc-ferme",
                weekend::LABEL_PARC_FERME,
                draft.parc_ferme,
                on_off.clone(),
                |r, v| r.parc_ferme = v,
                cx,
            ))
            .child(if pace_in_game {
                h_flex()
                    .gap(px(6.))
                    .items_center()
                    .child(rule_label(weekend::LABEL_EQUAL))
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(p.text_muted)
                            .child(tr("Off: the league's car pace is in the game.")),
                    )
            } else {
                choice_row(
                    "career-rule-equal",
                    weekend::LABEL_EQUAL,
                    draft.equal_performance,
                    on_off,
                    |r, v| r.equal_performance = v,
                    cx,
                )
            })
            .child(choice_row(
                "career-rule-weather",
                weekend::LABEL_WEATHER,
                draft.weather,
                numbered(
                    |v| WeekendRules {
                        weather: Some(v),
                        ..WeekendRules::default()
                    },
                    9,
                ),
                |r, v| r.weather = v,
                cx,
            ))
            .child(choice_row(
                "career-rule-rule-set",
                weekend::LABEL_RULE_SET,
                draft.rule_set,
                numbered(
                    |v| WeekendRules {
                        rule_set: Some(v),
                        ..WeekendRules::default()
                    },
                    20,
                ),
                |r, v| r.rule_set = v,
                cx,
            ));
        panel
            .child(label(tr("CAREER OPTIONS"), p))
            .child(market)
            .child(label(tr("CAREER OPTIONS · RULES SHEET"), p))
            .child(div().text_size(px(12.)).text_color(p.text_muted).child(tr(
                "What everyone sets in the game for each weekend. Each counted session is checked against it from \
                 the telemetry; a difference warns and the session counts all the same.",
            )))
            .child(rows)
            .child(
                h_flex()
                    .gap(px(8.))
                    .child(
                        Button::new("career-rules-save")
                            .primary()
                            .small()
                            .icon(IconName::Check)
                            .label(tr("Save"))
                            .disabled(busy)
                            .on_click(cx.listener(|this, _, _, cx| {
                                let (Some(rules), Some(league)) =
                                    (this.career.rules.draft.take(), this.career.league())
                                else {
                                    return;
                                };
                                let options = Options {
                                    rules,
                                    ..league.options.clone()
                                };
                                this.career_act(Action::SetOptions { options }, cx);
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("career-rules-cancel")
                            .outline()
                            .small()
                            .label(tr("Cancel"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.career.rules.draft = None;
                                cx.notify();
                            })),
                    ),
            )
    }
}

/// The AI driver market switch of the career options (W4-1): off keeps each year's real grid,
/// on lets the AI drivers move from the next season on. It is sent at once as `SetOptions`.
fn lineups_row(league: &League, busy: bool, p: Palette, cx: &mut Context<Shell>) -> Div {
    let on = league.options.lineups == LineupMode::Dynamic;
    let what = if on {
        tr(
            "On: at each season's end the AI drivers move between teams, retire and make way for rookies, the \
            same on every PC. The game shows them in the seats of the year's real drivers, with those drivers' \
            faces, and shows the players' names too.",
        )
    } else {
        tr(
            "Off: each season has its year's real grid. On: from the next season, the AI drivers move between \
            teams, retire and make way for rookies.",
        )
    };
    let renamed = league.season().renames.len();
    h_flex()
        .gap(px(10.))
        .items_start()
        .child(
            Switch::new("career-lineups-dynamic")
                .checked(on)
                .small()
                .disabled(busy)
                .on_click(cx.listener(move |this, on: &bool, _, cx| {
                    let Some(league) = this.career.league() else {
                        return;
                    };
                    let options = Options {
                        lineups: if *on {
                            LineupMode::Dynamic
                        } else {
                            LineupMode::Historic
                        },
                        ..league.options.clone()
                    };
                    this.career_act(Action::SetOptions { options }, cx);
                })),
        )
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .text_size(px(13.))
                        .font_weight(FontWeight::MEDIUM)
                        .child(tr("AI driver market")),
                )
                .child(div().text_size(px(12.)).text_color(p.text_muted).child(what))
                .when(renamed > 0, |d| {
                    d.child(
                        div().text_size(px(12.)).text_color(p.text_muted).child(
                            tr_owned("This season {count} AI drivers race in other drivers' seats.")
                                .replace("{count}", &renamed.to_string()),
                        ),
                    )
                }),
        )
}

/// Change the draft rules sheet.
fn edit(shell: &mut Shell, f: impl FnOnce(&mut WeekendRules)) {
    if let Some(draft) = shell.career.rules.draft.as_mut() {
        f(draft);
    }
}

fn rule_label(label: &'static str) -> Div {
    div().w(px(150.)).text_size(px(13.)).child(tr(label))
}

/// One button of a rules row.
fn choice(
    id: impl Into<ElementId>,
    text: impl Into<SharedString>,
    on: bool,
    set: impl Fn(&mut WeekendRules) + 'static,
    cx: &mut Context<Shell>,
) -> Button {
    let b = Button::new(id)
        .small()
        .label(text)
        .on_click(cx.listener(move |this, _, _, cx| {
            edit(this, &set);
            cx.notify();
        }));
    if on { b.primary() } else { b.outline() }
}

/// A rules row: "Not agreed" and each of the setting's values (`choices`, by English name).
fn choice_row<T: Copy + PartialEq + 'static>(
    id: &'static str,
    label: &'static str,
    current: Option<T>,
    choices: Vec<(T, String)>,
    set: fn(&mut WeekendRules, Option<T>),
    cx: &mut Context<Shell>,
) -> Div {
    h_flex()
        .gap(px(6.))
        .items_center()
        .flex_wrap()
        .child(rule_label(label))
        .child(choice(
            (id, 0usize),
            tr("Not agreed"),
            current.is_none(),
            move |r| set(r, None),
            cx,
        ))
        .children(choices.into_iter().enumerate().map(|(i, (v, name))| {
            choice(
                (id, i + 1),
                value_text(label, &name),
                current == Some(v),
                move |r| set(r, Some(v)),
                cx,
            )
        }))
}

#[cfg(test)]
mod tests {
    use swofty_core::league::weekend::{self, LABEL_AI, LABEL_DAMAGE};
    use swofty_core::league::{LapTelemetry, Objective, ObjectiveKind, Programme, RaceReport, WeekendRules};

    use super::{figures, objective_rows, programme_figure, rules_rows, value_names, value_text};
    use crate::i18n::{tr, tr_owned};

    #[test]
    fn rules_summary_lists_only_agreed_fields() {
        assert!(rules_rows(&WeekendRules::default()).is_empty());
        let rules = WeekendRules {
            ai_difficulty: Some(90),
            damage: Some(2),
            ..WeekendRules::default()
        };
        let rows = rules_rows(&rules);
        assert_eq!(rows.len(), 2, "{rows:?}");
        assert_eq!(rows[0].0, tr(LABEL_AI));
        assert_eq!(rows[0].1, "90");
        assert_eq!(rows[1].0, tr(LABEL_DAMAGE));
        assert_eq!(rows[1].1, value_text(LABEL_DAMAGE, "Standard"));
    }

    fn objective(kind: ObjectiveKind, target: u8, rp: u32, recognition: f32, met: Option<bool>) -> Objective {
        Objective {
            player: "anna".into(),
            kind,
            target,
            reward_rp: rp,
            reward_recognition: recognition,
            met,
            paid_elsewhere: false,
        }
    }

    #[test]
    fn objective_rows_show_rewards() {
        let quali = objective(ObjectiveKind::QualiPosition, 8, 40, 0.5, Some(true));
        let clean = objective(ObjectiveKind::CleanRace, 0, 30, 0.0, None);
        let rival = objective(ObjectiveKind::BeatRival, 0, 60, 1.0, Some(false));
        let rows = objective_rows(&[&quali, &clean, &rival], Some("Sergio Pérez"));
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].met, Some(true));
        assert!(rows[0].text.contains('8'), "{:?}", rows[0]);
        assert_eq!(
            rows[0].reward,
            format!(
                "{} · {}",
                tr_owned("+{rp} resource points").replace("{rp}", "40"),
                tr_owned("+{n} recognition").replace("{n}", "0.5")
            )
        );
        // No recognition, no recognition part.
        assert_eq!(
            rows[1].reward,
            tr_owned("+{rp} resource points").replace("{rp}", "30")
        );
        assert_eq!(rows[1].met, None);
        assert!(rows[2].text.contains("Sergio Pérez"), "{:?}", rows[2]);
        assert!(
            rows[2]
                .reward
                .ends_with(&tr_owned("+{n} recognition").replace("{n}", "1"))
        );
        assert_eq!(rows[2].met, Some(false));
    }

    #[test]
    fn rules_values_are_the_games_names() {
        let damage = value_names(
            |v| WeekendRules {
                damage: Some(v),
                ..WeekendRules::default()
            },
            9,
        );
        let names: Vec<&str> = damage.iter().map(|(_, n)| n.as_str()).collect();
        assert_eq!(names, ["Off", "Reduced", "Standard", "Simulation"]);
        let rule_sets = value_names(
            |v| WeekendRules {
                rule_set: Some(v),
                ..WeekendRules::default()
            },
            20,
        );
        assert!(
            rule_sets.iter().any(|(v, n)| *v == 1 && n == "Race"),
            "{rule_sets:?}"
        );
        assert!(rule_sets.iter().all(|(v, _)| *v != 3), "3 isn't a rule set");
        // Each value passes the core's check.
        for (v, _) in rule_sets {
            let rules = WeekendRules {
                rule_set: Some(v),
                ..WeekendRules::default()
            };
            assert_eq!(weekend::describe(&rules).len(), 1);
        }
    }

    fn lap(lap: u8, ms: u32, fuel: f32, ers: f32, store: f32, wear: f32) -> LapTelemetry {
        LapTelemetry {
            lap,
            ms,
            fuel_used_kg: fuel,
            ers_deployed_mj: ers,
            ers_store_end_mj: store,
            tyre_wear_pct: wear,
            valid: true,
        }
    }

    #[test]
    fn practice_figures_come_from_the_laps_at_pace() {
        let report = RaceReport {
            lap_telemetry: vec![
                lap(1, 95_000, 0.0, 0.0, 0.0, 0.0), // out lap: far off the pace
                lap(2, 80_000, 1.6, 4.0, 1.0, 2.0),
                lap(3, 81_000, 1.8, 3.8, 0.6, 2.2),
                LapTelemetry {
                    valid: false,
                    ..lap(4, 79_000, 1.0, 1.0, 1.0, 1.0)
                },
            ],
            ..RaceReport::default()
        };
        let f = figures(&report);
        assert_eq!((f.laps, f.clean, f.at_pace), (4, 3, 2));
        assert_eq!(f.best_ms, Some(80_000));
        assert!((f.fuel_kg.unwrap() - 1.7).abs() < 1e-4);
        assert!((f.ers_deployed_mj.unwrap() - 3.9).abs() < 1e-4);
        assert!((f.ers_left_mj.unwrap() - 0.8).abs() < 1e-4);
        assert!((f.wear_pct.unwrap() - 2.1).abs() < 1e-4);
        for p in [
            Programme::TrackAcclimatisation,
            Programme::QualifyingPace,
            Programme::TyreManagement,
            Programme::ErsManagement,
            Programme::FuelManagement,
            Programme::RaceStrategy,
        ] {
            assert!(programme_figure(p, &f).is_some(), "{p:?}");
        }
        // An older launcher's report: no lap telemetry, so no ERS, fuel or wear.
        let old = figures(&RaceReport::default());
        assert_eq!(old, super::Figures::default());
        assert!(programme_figure(Programme::FuelManagement, &old).is_none());
    }
}
