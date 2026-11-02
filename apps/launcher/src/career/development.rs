//! The team's development with the player's resource points: each part's tree of a main and an
//! alternative branch (`swofty_core::league::rnd`), the nodes' states, the cost cap, and the car.

use gpui_kit::assets::IconName;
use gpui_kit::component::button::Button;
use gpui_kit::component::{Disableable, Sizable, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use swofty_core::league::rnd::{self, Node};
use swofty_core::league::{Action, League, Part, Phase, Season, Upgrade};

use crate::i18n::{tr, tr_owned};
use crate::pages::label;
use crate::shell::Shell;
use crate::theme::Palette;

use super::ordinal;

/// Where a node of a team's development tree stands this season.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NodeState {
    /// It needs the node before it first.
    Locked,
    /// It can be ordered.
    Available,
    /// Ordered, on the car from this round.
    Building(u8),
    /// On the car.
    Arrived,
    /// It failed its tests when it arrived (half of its points went back); it can be ordered
    /// again when the node before it is still in place.
    Failed,
}

/// The upgrades of team `slot` that are node `id` of `part`: the node ordered, or for a legacy
/// level (from before the tree) the main node it counts as.
fn upgrades_of(season: &Season, slot: u8, part: Part, id: u8) -> Vec<&Upgrade> {
    let mut legacy = 0u8;
    season
        .upgrades
        .iter()
        .filter(|u| u.slot == slot && u.part == part)
        .filter(|u| match u.node {
            Some(n) => n == id,
            None => {
                legacy += 1;
                legacy == id && id <= 5
            }
        })
        .collect()
}

/// Where node `node` of `part` stands for team `slot` in `season`.
pub(super) fn node_state(season: &Season, slot: u8, part: Part, node: &Node) -> NodeState {
    let now = season.current_round();
    let ups = upgrades_of(season, slot, part, node.id);
    if let Some(u) = ups.iter().rev().find(|u| !u.failed) {
        return if u.ready > now {
            NodeState::Building(u.ready)
        } else {
            NodeState::Arrived
        };
    }
    if ups.iter().any(|u| u.failed) {
        return NodeState::Failed;
    }
    if rnd::available(season, slot, part).iter().any(|n| n.id == node.id) {
        NodeState::Available
    } else {
        NodeState::Locked
    }
}

/// "+0.07% a lap", or for durability "12% fewer breakdowns".
fn gain_text(part: Part, gain: f32) -> String {
    if part == Part::Durability {
        // `+ 0.0` turns a rounded -0 into 0, so nothing shows "-0%".
        tr_owned("{value}% fewer breakdowns").replace("{value}", &format!("{:.0}", (gain * 100.0).round() + 0.0))
    } else {
        tr_owned("{value}% a lap").replace("{value}", &format!("{:.2}", (gain * 100.0).round() / 100.0 + 0.0))
    }
}

#[cfg(test)]
mod gain_tests {
    use super::{Part, gain_text};

    #[test]
    fn a_zero_gain_reads_without_a_minus() {
        assert_eq!(gain_text(Part::Durability, -0.0), "0% fewer breakdowns");
        assert_eq!(gain_text(Part::Durability, -0.004), "0% fewer breakdowns");
        assert_eq!(gain_text(Part::Aero, -0.0), "0.00% a lap");
        assert_eq!(gain_text(Part::Aero, 0.07), "0.07% a lap");
    }
}

/// What a node costs and risks: "800 RP · 2 rounds · 5% risk".
fn terms_text(node: &Node) -> String {
    tr_owned("{cost} RP · {rounds} rounds · {risk}% risk")
        .replace("{cost}", &node.cost.to_string())
        .replace("{rounds}", &node.rounds.to_string())
        .replace("{risk}", &format!("{:.0}", node.fail * 100.0))
}

/// Why a node that can be ordered can't be ordered right now, if it can't.
fn blocked(node: &Node, have: u32, spent: u32, next: Option<u8>, last: u8) -> Option<String> {
    let Some(next) = next else {
        return Some(tr("The season is over").to_owned());
    };
    if next + node.rounds > last {
        return Some(tr("Too late: it wouldn't be ready this season").to_owned());
    }
    if spent + node.cost > rnd::COST_CAP {
        return Some(tr("Over the cost cap").to_owned());
    }
    if have < node.cost {
        return Some(tr_owned("You have {rp} resource points").replace("{rp}", &have.to_string()));
    }
    None
}

impl Shell {
    /// The team's development: the player's resource points, the cost cap, each part's tree with
    /// the nodes' states and **Develop** buttons, the parts on order and the car's pace.
    pub(super) fn career_development(
        &self,
        league: &League,
        me: &str,
        p: Palette,
        cx: &mut Context<Self>,
    ) -> Div {
        let season = league.season();
        let (Some(pl), Some(entry)) = (league.player(me), season.player_entry(me)) else {
            return v_flex().child(
                div()
                    .text_size(px(13.))
                    .text_color(p.text_muted)
                    .child(tr("You have no seat this season.")),
            );
        };
        let slot = entry.slot;
        let Some(team) = season.team(slot) else {
            return v_flex();
        };
        let busy = self.career.busy.is_some();
        let racing = league.phase == Phase::Racing;
        let round = season.current_round();
        let last = season.rounds.last().map_or(0, |r| r.round);
        let next = season.next_round().filter(|_| racing).map(|r| r.round);
        let spent = rnd::spent(season, slot);
        let shortest = rnd::TREE.iter().flatten().map(|n| n.rounds).min().unwrap_or(2);
        let late = next.is_none_or(|n| n + shortest > last);
        let profile = rnd::car_profile(season, slot, round);

        let tile = |part: Part, node: &'static Node, cx: &mut Context<Self>| {
            let state = node_state(season, slot, part, node);
            let (status, color) = match state {
                NodeState::Locked => (
                    node.requires
                        .and_then(|r| rnd::node(part, r))
                        .map_or_else(String::new, |r| {
                            tr_owned("Needs {node}").replace("{node}", tr(r.name))
                        }),
                    p.text_muted,
                ),
                NodeState::Available => (tr("Can be developed").to_owned(), p.text_muted),
                NodeState::Building(ready) => (
                    tr_owned("On the car from round {round}").replace("{round}", &ready.to_string()),
                    p.accent,
                ),
                NodeState::Arrived => (tr("On the car").to_owned(), p.success),
                NodeState::Failed => (
                    tr("Failed its tests; half the points came back").to_owned(),
                    p.error,
                ),
            };
            let orderable = rnd::available(season, slot, part).iter().any(|n| n.id == node.id);
            let why = if orderable {
                blocked(node, pl.rp, spent, next, last)
            } else {
                None
            };
            let border = match state {
                NodeState::Arrived => p.success,
                NodeState::Building(_) => p.accent,
                NodeState::Failed => p.error,
                _ => p.border,
            };
            v_flex()
                .w(px(168.))
                .gap(px(3.))
                .p(px(8.))
                .rounded(px(6.))
                .border_1()
                .border_color(border)
                .when(state == NodeState::Locked, |d| d.opacity(0.55))
                .child(
                    div()
                        .text_size(px(12.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(tr(node.name)),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(p.text_muted)
                        .child(gain_text(part, node.gain)),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(p.text_muted)
                        .child(terms_text(node)),
                )
                .child(div().text_size(px(11.)).text_color(color).child(status))
                .when(orderable, |d| {
                    let (part, id) = (part, node.id);
                    d.child(
                        Button::new(SharedString::from(format!("develop-{part:?}-{id}")))
                            .outline()
                            .small()
                            .icon(IconName::Wrench)
                            .label(if state == NodeState::Failed {
                                tr("Develop again")
                            } else {
                                tr("Develop")
                            })
                            .disabled(busy || !racing || why.is_some())
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.career_act(Action::DevelopNode { part, node: id }, cx)
                            })),
                    )
                    .when_some(why, |d, why| {
                        d.child(div().text_size(px(11.)).text_color(p.warning).child(why))
                    })
                })
        };

        let parts: Vec<Div> = Part::ALL
            .into_iter()
            .map(|part| {
                let nodes = rnd::nodes(part);
                let now = match part {
                    Part::Aero => profile.aero,
                    Part::Chassis => profile.chassis,
                    Part::Powertrain => profile.powertrain,
                    Part::Durability => profile.durability,
                };
                let branch =
                    |b: u8, title: &'static str, risk: f32, cx: &mut Context<Self>| {
                        v_flex()
                            .gap(px(4.))
                            .child(
                                div().text_size(px(11.)).text_color(p.text_muted).child(
                                    tr_owned(title).replace("{risk}", &format!("{:.0}", risk * 100.0)),
                                ),
                            )
                            .child(
                                h_flex().gap(px(6.)).flex_wrap().items_start().children(
                                    nodes.iter().filter(|n| n.branch == b).map(|n| tile(part, n, cx)),
                                ),
                            )
                    };
                v_flex()
                    .gap(px(6.))
                    .py(px(8.))
                    .border_t_1()
                    .border_color(p.border)
                    .child(
                        h_flex()
                            .gap(px(10.))
                            .items_center()
                            .child(
                                div()
                                    .text_size(px(14.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(tr(part.name())),
                            )
                            .child(div().text_size(px(12.)).text_color(p.text_muted).child(
                                tr_owned("on the car now: {gain}").replace("{gain}", &gain_text(part, now)),
                            )),
                    )
                    .child(branch(
                        0,
                        "Main branch: steady steps, {risk}% failure risk",
                        rnd::MAIN_FAIL,
                        cx,
                    ))
                    .child(branch(
                        1,
                        "Alternative branch: bigger steps, {risk}% failure risk",
                        rnd::ALTERNATIVE_FAIL,
                        cx,
                    ))
            })
            .collect();

        let on_order: Vec<String> = season
            .upgrades
            .iter()
            .filter(|u| u.slot == slot && u.ready > round && !u.failed)
            .map(|u| {
                let race = season.round(u.ready).map_or("", |r| r.race.as_str());
                let name = u.node.and_then(|id| rnd::node(u.part, id)).map_or_else(
                    || tr_owned("level {level}").replace("{level}", &u.level.to_string()),
                    |n| tr(n.name).to_owned(),
                );
                tr_owned("{part}: {node}, on the car from round {round} ({race})")
                    .replace("{part}", tr(u.part.name()))
                    .replace("{node}", &name)
                    .replace("{round}", &u.ready.to_string())
                    .replace("{race}", race)
            })
            .collect();

        let cap = rnd::COST_CAP.max(1);
        let share = (spent.min(cap) as f32 / cap as f32).clamp(0.0, 1.0);
        let cap_bar = v_flex()
            .gap(px(4.))
            .child(
                h_flex().justify_between().child(label(tr("COST CAP"), p)).child(
                    div().text_size(px(12.)).text_color(p.text_muted).child(
                        tr_owned("{team} has spent {spent} of {cap} resource points this season")
                            .replace("{team}", &team.name)
                            .replace("{spent}", &spent.to_string())
                            .replace("{cap}", &rnd::COST_CAP.to_string()),
                    ),
                ),
            )
            .child(
                div().w_full().h(px(8.)).rounded(px(4.)).bg(p.border).child(
                    div()
                        .h_full()
                        .rounded(px(4.))
                        .w(relative(share))
                        .bg(if share >= 0.9 { p.warning } else { p.accent }),
                ),
            );

        let start = team.pace + team.carry;
        let now = season.pace_now(slot);
        let game = if league.options.game_performance {
            tr("With game sync on, the game's car gets the team's pace too.")
        } else {
            tr("The game's own car stays as the season box has it unless the game sync for team pace is on.")
        };
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
                            .child(pl.rp.to_string()),
                    )
                    .child(
                        v_flex()
                            .child(
                                div()
                                    .text_size(px(15.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(tr_owned("resource points for {team}").replace("{team}", &team.name)),
                            )
                            .child(div().text_size(px(12.)).text_color(p.text_muted).child(tr(
                                "Earned in practice programmes, with qualifying, results, team goals and beating \
                                 your team-mate. A part takes two to four rounds to build and can fail its tests \
                                 when it arrives: then it gains nothing and half of its points come back.",
                            ))),
                    ),
            )
            .child(cap_bar)
            .when(late && racing, |d| {
                d.child(div().text_size(px(12.)).text_color(p.warning).child(tr(
                    "Too late in the season: a new part wouldn't be ready. Your points carry over to next season.",
                )))
            })
            .child(v_flex().children(parts))
            .when(!on_order.is_empty(), |d| {
                d.child(label(tr("ON ORDER"), p)).children(
                    on_order
                        .into_iter()
                        .map(|t| div().text_size(px(13.)).child(t)),
                )
            })
            .child(label(tr("THE CAR"), p))
            .child(
                div().text_size(px(13.)).child(
                    tr_owned(
                        "Gap to the fastest car: {now}% now, {start}% at the start of {year}. The {rank} fastest car of {count}.",
                    )
                    .replace("{now}", &format!("{now:.2}"))
                    .replace("{start}", &format!("{start:.2}"))
                    .replace("{year}", &season.year.to_string())
                    .replace("{rank}", &ordinal(season.pace_rank(slot)))
                    .replace("{place}", &season.pace_rank(slot).to_string())
                    .replace("{count}", &season.teams.len().to_string()),
                ),
            )
            .child(
                div().text_size(px(12.)).text_color(p.text_muted).child(
                    tr_owned(
                        "Every team's car also moves over the season as it did for real. Half of what a team develops \
                         carries into next season (a sixth when the rules change, 2017, 2022 and 2026){carry}. The \
                         upgrades make the car faster in simulated rounds and raise the team's goals.",
                    )
                    .replace(
                        "{carry}",
                        &if team.carry.abs() > 1e-4 {
                            tr_owned("; {team} carries {carry}% from earlier seasons")
                                .replace("{team}", &team.name)
                                .replace("{carry}", &format!("{:+.2}", team.carry))
                        } else {
                            String::new()
                        },
                    )
                        + " "
                        + game,
                ),
            )
    }
}

#[cfg(test)]
mod tests {
    use swofty_core::league::{self, Action, Actor, League, Part, Setup, Upgrade, rnd};

    use super::{NodeState, node_state};

    fn anna() -> Actor {
        Actor {
            id: "u-anna".into(),
            name: "Anna".into(),
            owner: true,
        }
    }

    fn upgrade(part: Part, node: Option<u8>, ready: u8, failed: bool) -> Upgrade {
        Upgrade {
            slot: 2,
            part,
            level: 1,
            by: "u-anna".into(),
            cost: 800,
            round: 1,
            started: false,
            ready,
            node,
            failed,
        }
    }

    /// Anna at Ferrari (slot 2) in 2016, the season begun.
    fn league() -> League {
        let mut l = League::start(Setup::default(), &anna(), Some(7), 1).unwrap();
        league::apply(
            &mut l,
            &anna(),
            Action::Seat {
                slot: 2,
                replaces: None,
            },
        )
        .unwrap();
        league::apply(&mut l, &anna(), Action::Begin).unwrap();
        l
    }

    fn state(season: &league::Season, part: Part, id: u8) -> NodeState {
        node_state(season, 2, part, rnd::node(part, id).unwrap())
    }

    #[test]
    fn node_state_of_each_upgrade() {
        let l = league();
        let mut season = l.season().clone();
        let now = season.current_round();
        assert_eq!(now, 1);
        // Nothing ordered: the first node of each branch is available, the rest locked.
        assert_eq!(state(&season, Part::Aero, 1), NodeState::Available);
        assert_eq!(state(&season, Part::Aero, 6), NodeState::Available);
        assert_eq!(state(&season, Part::Aero, 2), NodeState::Locked);
        assert_eq!(state(&season, Part::Aero, 7), NodeState::Locked);

        // Aero 1 arrived, aero 2 building, aero 6 failed, chassis 1 from before the tree.
        season.upgrades = vec![
            upgrade(Part::Aero, Some(1), now, false),
            upgrade(Part::Aero, Some(2), now + 2, false),
            upgrade(Part::Aero, Some(6), now, true),
            upgrade(Part::Chassis, None, now, false),
            Upgrade {
                slot: 3,
                ..upgrade(Part::Powertrain, Some(1), now, false)
            },
        ];
        assert_eq!(state(&season, Part::Aero, 1), NodeState::Arrived);
        assert_eq!(state(&season, Part::Aero, 2), NodeState::Building(now + 2));
        // Building counts as taken: the next node unlocks.
        assert_eq!(state(&season, Part::Aero, 3), NodeState::Available);
        assert_eq!(state(&season, Part::Aero, 6), NodeState::Failed);
        assert_eq!(state(&season, Part::Aero, 7), NodeState::Locked);
        // A legacy level counts as the main branch's first node.
        assert_eq!(state(&season, Part::Chassis, 1), NodeState::Arrived);
        assert_eq!(state(&season, Part::Chassis, 2), NodeState::Available);
        assert_eq!(state(&season, Part::Chassis, 6), NodeState::Available);
        // Another team's parts don't count.
        assert_eq!(state(&season, Part::Powertrain, 1), NodeState::Available);

        // A failed node ordered again shows the new order.
        season.upgrades.push(upgrade(Part::Aero, Some(6), now + 3, false));
        assert_eq!(state(&season, Part::Aero, 6), NodeState::Building(now + 3));
    }

    #[test]
    fn develop_node_shows_as_building() {
        let mut l = league();
        l.players[0].rp = 5_000;
        league::apply(
            &mut l,
            &anna(),
            Action::DevelopNode {
                part: Part::Aero,
                node: 1,
            },
        )
        .unwrap();
        let season = l.season();
        assert!(matches!(state(season, Part::Aero, 1), NodeState::Building(_)));
        assert_eq!(state(season, Part::Aero, 2), NodeState::Available);
    }
}
