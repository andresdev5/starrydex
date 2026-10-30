// SPDX-License-Identifier: GPL-3.0-only

//! Two Pokémon side by side, as if they were battling: who hits harder and first, with
//! which moves, and how their base stats compare.

use cosmic::iced::core::text::LineHeight;
use cosmic::iced::{Alignment, Background, Border, Color, Length, Pixels};
use cosmic::widget::{self, button, container, text};
use cosmic::{Element, theme};
use starrydex_model::{
    ArchivedMove, ArchivedPokedex, ArchivedPokemon, DamageClass, PokemonType, SpriteKind, Stat,
};

use crate::app::core::fold;
use crate::app::pages::common::{
    card, multiplier_label, multiplier_pill, sprite, type_badge, type_icon,
};
use crate::app::utils::battle::{self, Pace};
use crate::app::utils::presentation::{Label, data_lang, dex_number, stat_short};
use crate::app::{CompareInput, Message, Ready};
use crate::{fl, icons};

/// Suggestions shown under a comparator search box.
const SUGGESTIONS: usize = 6;
/// Base stat that fills a comparison bar completely.
const STAT_BAR_MAX: u16 = 255;
/// Base stat total that fills the total bar completely.
const TOTAL_BAR_MAX: u16 = 800;
/// Moves listed for each side of the battle.
const BEST_MOVES: usize = 4;
/// Wider than this, the two sides of the battle are shown next to each other.
const BATTLE_COLUMNS_WIDTH: f32 = 900.0;

pub fn view<'a>(ready: &'a Ready) -> Element<'a, Message> {
    widget::responsive(move |size| page(ready, size.width >= BATTLE_COLUMNS_WIDTH)).into()
}

fn page<'a>(ready: &'a Ready, wide: bool) -> Element<'a, Message> {
    let spacing = theme::active().cosmic().spacing;
    let dex = ready.core.dex();
    let picked = ready.compare.map(|id| id.and_then(|id| dex.pokemon(id)));

    let sides = widget::flex_row(vec![
        side(ready, 0, picked[0], picked[1]),
        container(
            widget::column::with_children(vec![
                text::title3(fl!("vs")).class(theme::Text::Accent).into(),
                widget::tooltip(
                    button::icon(icons::get_handle("view-dual-symbolic", 18))
                        .on_press(Message::Compare(CompareInput::Swap)),
                    text::body(fl!("compare-swap")),
                    widget::tooltip::Position::Bottom,
                )
                .into(),
            ])
            .spacing(6)
            .align_x(Alignment::Center),
        )
        .padding([24, 0])
        .into(),
        side(ready, 1, picked[1], picked[0]),
    ])
    .row_spacing(spacing.space_s)
    .column_spacing(spacing.space_s);

    let mut content = vec![text::title3(fl!("compare-title")).into(), sides.into()];
    if let [Some(left), Some(right)] = picked {
        content.push(battle_card(dex, left, right, wide));
        content.push(stats(left, right));
    }

    widget::scrollable(
        widget::column::with_children(content)
            .spacing(spacing.space_s)
            .padding([spacing.space_xs, spacing.space_m, spacing.space_xs, 0])
            .width(Length::Fill),
    )
    .height(Length::Fill)
    .width(Length::Fill)
    .into()
}

/// One side: search box with suggestions, then the picked Pokémon.
fn side<'a>(
    ready: &'a Ready,
    index: usize,
    pokemon: Option<&'a ArchivedPokemon>,
    other: Option<&'a ArchivedPokemon>,
) -> Element<'a, Message> {
    let dex = ready.core.dex();
    let lang = data_lang();
    let search = &ready.compare_search[index];

    let mut content: Vec<Element<'a, Message>> = vec![
        widget::search_input(fl!("compare-search"), search)
            .on_input(move |v| Message::Compare(CompareInput::Search(index, v)))
            .on_clear(Message::Compare(CompareInput::Search(index, String::new())))
            .line_height(LineHeight::Absolute(Pixels(30.0)))
            .width(Length::Fill)
            .into(),
    ];

    if !search.trim().is_empty() {
        let suggestions: Vec<Element<'a, Message>> = matches(dex, search)
            .into_iter()
            .map(|p| {
                button::custom(
                    widget::row::with_children(vec![
                        sprite(ready.core.sprite(p, SpriteKind::Front), 32.0),
                        text::caption(dex_number(p.species_id.to_native())).into(),
                        text::body(p.display.get(lang)).into(),
                    ])
                    .spacing(8)
                    .align_y(Alignment::Center),
                )
                .padding([2, 6])
                .width(Length::Fill)
                .class(theme::Button::Text)
                .on_press(Message::Compare(CompareInput::Pick(
                    index,
                    p.id.to_native(),
                )))
                .into()
            })
            .collect();
        content.push(if suggestions.is_empty() {
            text::caption(fl!("no-results")).into()
        } else {
            widget::column::with_children(suggestions).spacing(2).into()
        });
    }

    content.push(match pokemon {
        Some(pokemon) => summary(ready, pokemon, other),
        None => container(text::body(fl!("compare-pick")))
            .padding(24)
            .center_x(Length::Fill)
            .into(),
    });

    container(widget::column::with_children(content).spacing(10))
        .padding(14)
        .width(Length::Fixed(360.0))
        .class(theme::Container::Card)
        .into()
}

fn summary<'a>(
    ready: &'a Ready,
    pokemon: &'a ArchivedPokemon,
    other: Option<&'a ArchivedPokemon>,
) -> Element<'a, Message> {
    let types = widget::row::with_children(
        pokemon
            .types
            .iter()
            .map(|t| type_badge(*t))
            .collect::<Vec<_>>(),
    )
    .spacing(6);
    let mut info = vec![
        text::caption(dex_number(pokemon.species_id.to_native())).into(),
        text::title4(pokemon.display.get(data_lang())).into(),
        types.into(),
    ];
    if let Some(other) = other {
        info.push(
            text::caption(fl!(
                "compare-best-type",
                name = (other.display.get(data_lang())),
                multiplier = (multiplier_label(best_multiplier(ready.core.dex(), pokemon, other)))
            ))
            .into(),
        );
    }
    button::custom(
        widget::row::with_children(vec![
            sprite(ready.core.sprite(pokemon, SpriteKind::Front), 72.0),
            widget::column::with_children(info)
                .spacing(4)
                .width(Length::Fill)
                .into(),
        ])
        .spacing(12)
        .align_y(Alignment::Center),
    )
    .padding(4)
    .width(Length::Fill)
    .class(theme::Button::IconVertical)
    .on_press(Message::SelectPokemon(pokemon.id.to_native()))
    .into()
}

/// Pokémon whose name or number contains `search`, in Pokédex order.
fn matches<'a>(dex: &'a ArchivedPokedex, search: &str) -> Vec<&'a ArchivedPokemon> {
    let needle = fold(search.trim().trim_start_matches('#'));
    let lang = data_lang();
    dex.pokemon
        .iter()
        .filter(|p| {
            fold(p.display.get(lang)).contains(&needle)
                || fold(&p.display.en).contains(&needle)
                || p.species_id.to_native().to_string() == needle
        })
        .take(SUGGESTIONS)
        .collect()
}

/// Strongest multiplier `attacker` gets against `defender` with a move of its own type.
fn best_multiplier(
    dex: &ArchivedPokedex,
    attacker: &ArchivedPokemon,
    defender: &ArchivedPokemon,
) -> f32 {
    let defending: Vec<PokemonType> = defender.types.iter().copied().collect();
    attacker
        .types
        .iter()
        .map(|t| dex.effectiveness(*t, &defending))
        .fold(0.0, f32::max)
}

/// Base stats of both Pokémon as mirrored bars; the higher value of each row is highlighted.
fn stats<'a>(left: &'a ArchivedPokemon, right: &'a ArchivedPokemon) -> Element<'a, Message> {
    let mut rows: Vec<Element<'a, Message>> = [
        Stat::Hp,
        Stat::Attack,
        Stat::Defense,
        Stat::SpAttack,
        Stat::SpDefense,
        Stat::Speed,
    ]
    .iter()
    .map(|stat| {
        row(
            stat_short(*stat),
            u16::from(left.stats.get(*stat)),
            u16::from(right.stats.get(*stat)),
            STAT_BAR_MAX,
        )
    })
    .collect();
    rows.push(row(
        fl!("total"),
        left.stats.total(),
        right.stats.total(),
        TOTAL_BAR_MAX,
    ));
    card(
        fl!("poke-stats"),
        widget::column::with_children(rows).spacing(8),
    )
}

fn row<'a>(label: String, left: u16, right: u16, max: u16) -> Element<'a, Message> {
    widget::row::with_children(vec![
        text::heading(left.to_string())
            .width(Length::Fixed(44.0))
            .align_x(Alignment::End)
            .into(),
        bar(left, max, left >= right, true),
        text::caption(label)
            .width(Length::Fixed(80.0))
            .align_x(Alignment::Center)
            .into(),
        bar(right, max, right >= left, false),
        text::heading(right.to_string())
            .width(Length::Fixed(44.0))
            .into(),
    ])
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}

/// Horizontal bar filled up to `value`; `from_right` grows it towards the left.
fn bar<'a>(value: u16, max: u16, winner: bool, from_right: bool) -> Element<'a, Message> {
    let filled = value.clamp(1, max);
    let fill = container(widget::Space::new())
        .width(Length::FillPortion(filled))
        .height(Length::Fixed(12.0))
        .class(theme::Container::custom(move |theme| {
            let accent: Color = theme.cosmic().accent_color().into();
            container::Style {
                background: Some(Background::Color(if winner {
                    accent
                } else {
                    Color { a: 0.35, ..accent }
                })),
                border: Border {
                    radius: 2.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }))
        .into();
    let rest = widget::Space::new()
        .width(Length::FillPortion(max - filled + 1))
        .into();
    let children = if from_right {
        vec![rest, fill]
    } else {
        vec![fill, rest]
    };
    container(widget::row::with_children(children))
        .width(Length::Fill)
        .class(theme::Container::custom(|theme| container::Style {
            background: Some(Background::Color(theme.cosmic().bg_divider().into())),
            border: Border {
                radius: 2.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }))
        .into()
}

/// A damaging move and what it does to the opponent.
struct Hit<'a> {
    attack: &'a ArchivedMove,
    multiplier: f32,
    damage: u16,
    pace: Pace,
    /// Hits needed to knock the defender out (`None` if it does no damage)
    hits: Option<u16>,
    /// Turns those hits take, counting charge and recharge turns
    turns: Option<u16>,
}

impl Hit<'_> {
    /// "3 hits", or "2 hits (4 turns)" for moves that charge or recharge.
    fn label(&self) -> String {
        match (self.hits, self.turns) {
            (Some(hits), Some(turns)) if hits != turns => {
                fl!("versus-hits-turns", hits = hits, turns = turns)
            }
            (Some(hits), _) => fl!("versus-hits", hits = hits),
            _ => "—".to_string(),
        }
    }
}

/// Battle stat of `pokemon` at the comparator level.
fn battle_stat(pokemon: &ArchivedPokemon, stat: Stat) -> u16 {
    battle::stat_at_level(pokemon.stats.get(stat), matches!(stat, Stat::Hp))
}

/// Damaging moves `attacker` can learn in any game, the quickest to knock `defender` out
/// first. Moves that knock the user out are left out.
fn hits<'a>(
    dex: &'a ArchivedPokedex,
    attacker: &'a ArchivedPokemon,
    defender: &ArchivedPokemon,
) -> Vec<Hit<'a>> {
    let defending: Vec<PokemonType> = defender.types.iter().copied().collect();
    let hp = battle_stat(defender, Stat::Hp);
    let mut hits: Vec<Hit<'a>> = attacker
        .moves
        .iter()
        .filter_map(|learned| dex.move_(learned.move_id.to_native()))
        .filter_map(|attack| {
            let power = *attack.power.as_ref()?;
            let pace = Pace::of(attack.name.as_str());
            if pace == Pace::SelfKo {
                return None;
            }
            let (offense, defense) = match attack.damage_class {
                DamageClass::Physical => (Stat::Attack, Stat::Defense),
                DamageClass::Special => (Stat::SpAttack, Stat::SpDefense),
                DamageClass::Status => return None,
            };
            let multiplier = dex.effectiveness(attack.move_type, &defending);
            let stab = attacker.types.contains(&attack.move_type);
            let damage = battle::damage(
                power,
                battle_stat(attacker, offense),
                battle_stat(defender, defense),
                stab,
                multiplier,
            );
            let hits = battle::hits_to_ko(hp, damage);
            (power > 0).then_some(Hit {
                attack,
                multiplier,
                pace,
                hits,
                turns: hits.map(|n| pace.turns(n)),
                damage,
            })
        })
        .collect();
    hits.sort_by_key(|hit| (hit.turns.unwrap_or(u16::MAX), std::cmp::Reverse(hit.damage)));
    hits
}

/// Who wins the battle and how, plus who moves first and what each side can do.
fn battle_card<'a>(
    dex: &'a ArchivedPokedex,
    left: &'a ArchivedPokemon,
    right: &'a ArchivedPokemon,
    wide: bool,
) -> Element<'a, Message> {
    let spacing = theme::active().cosmic().spacing;
    let fighters = [left, right];
    let plans = [hits(dex, left, right), hits(dex, right, left)];
    let speed = fighters.map(|p| battle_stat(p, Stat::Speed));
    // Hits the best move of each side needs to knock out the other one.
    // Turns the best move of each side needs to knock out the other one.
    let ko: Vec<Option<(u16, &Hit<'a>)>> = plans
        .iter()
        .map(|plan| {
            let hit = plan.first()?;
            hit.turns.map(|turns| (turns, hit))
        })
        .collect();

    let lang = data_lang();
    let name = |i: usize| fighters[i].display.get(lang).to_string();

    let winner = match (ko[0], ko[1]) {
        (None, None) => None,
        (Some(_), None) => Some(0),
        (None, Some(_)) => Some(1),
        (Some((a, _)), Some((b, _))) if a != b => Some(if a < b { 0 } else { 1 }),
        _ if speed[0] != speed[1] => Some(if speed[0] > speed[1] { 0 } else { 1 }),
        _ => None,
    };
    let mut verdict: Vec<Element<'a, Message>> = Vec::new();
    match winner {
        Some(w) => {
            let loser = 1 - w;
            let (turns, hit) = ko[w].expect("the winner can hit");
            let tie_on_turns = ko[loser].is_some_and(|(n, _)| n == turns);
            verdict.push(
                text::title4(if tie_on_turns {
                    fl!("versus-winner-faster", name = name(w), hits = hit.label())
                } else {
                    fl!(
                        "versus-winner",
                        name = name(w),
                        other = name(loser),
                        hits = hit.label(),
                        attack = (hit.attack.display.get(lang))
                    )
                })
                .into(),
            );
            verdict.push(
                text::body(match ko[loser] {
                    Some((_, hit)) => fl!(
                        "versus-loser",
                        name = name(loser),
                        hits = hit.label(),
                        attack = (hit.attack.display.get(lang))
                    ),
                    None => fl!("versus-loser-none", name = name(loser), other = name(w)),
                })
                .into(),
            );
        }
        None => verdict.push(
            text::title4(match ko[0] {
                Some((_, hit)) => fl!("versus-even", hits = hit.label()),
                None => fl!("versus-nobody"),
            })
            .into(),
        ),
    }
    let speed_line = if speed[0] == speed[1] {
        fl!("versus-speed-tie", speed = speed[0])
    } else {
        let (fast, slow) = if speed[0] > speed[1] { (0, 1) } else { (1, 0) };
        fl!(
            "versus-faster",
            name = name(fast),
            speed = speed[fast],
            other = speed[slow]
        )
    };
    verdict.push(text::body(speed_line).into());

    let banner = container(widget::column::with_children(verdict).spacing(6))
        .padding(12)
        .width(Length::Fill)
        .class(theme::Container::custom(|theme| {
            let accent: Color = theme.cosmic().accent_color().into();
            container::Style {
                background: Some(Background::Color(Color { a: 0.12, ..accent })),
                border: Border {
                    color: Color { a: 0.5, ..accent },
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        }));

    let [left_plan, right_plan] = plans;
    let sides = [
        battle_side(dex, left, right, left_plan),
        battle_side(dex, right, left, right_plan),
    ];
    let sides: Element<'a, Message> = if wide {
        widget::row::with_children(Vec::from(sides))
            .spacing(spacing.space_l)
            .into()
    } else {
        widget::column::with_children(Vec::from(sides))
            .spacing(spacing.space_m)
            .into()
    };

    card(
        fl!("versus-title"),
        widget::column::with_children(vec![
            banner.into(),
            sides,
            text::caption(fl!("versus-assumptions", level = battle::LEVEL)).into(),
        ])
        .spacing(spacing.space_s),
    )
}

/// What `attacker` brings against `defender`: its own types, the better way to attack, its
/// strongest moves and the defender types that threaten it.
fn battle_side<'a>(
    dex: &'a ArchivedPokedex,
    attacker: &'a ArchivedPokemon,
    defender: &'a ArchivedPokemon,
    plan: Vec<Hit<'a>>,
) -> Element<'a, Message> {
    let lang = data_lang();
    let attacking: Vec<PokemonType> = attacker.types.iter().copied().collect();
    let defending: Vec<PokemonType> = defender.types.iter().copied().collect();
    let section = |title: String| text::heading(title).class(theme::Text::Accent);

    let stab = widget::flex_row(
        attacking
            .iter()
            .map(|t| {
                widget::row::with_children(vec![
                    type_badge(*t),
                    multiplier_pill(dex.effectiveness(*t, &defending)),
                ])
                .spacing(6)
                .align_y(Alignment::Center)
                .into()
            })
            .collect(),
    )
    .row_spacing(6)
    .column_spacing(12);

    let physical = (
        battle_stat(attacker, Stat::Attack),
        battle_stat(defender, Stat::Defense),
    );
    let special = (
        battle_stat(attacker, Stat::SpAttack),
        battle_stat(defender, Stat::SpDefense),
    );
    // Compare attack/defense ratios without dividing: a/b >= c/d  <=>  a*d >= c*b
    let lean = if u32::from(physical.0) * u32::from(special.1)
        >= u32::from(special.0) * u32::from(physical.1)
    {
        fl!(
            "versus-lean-physical",
            attack = physical.0,
            defense = physical.1
        )
    } else {
        fl!(
            "versus-lean-special",
            attack = special.0,
            defense = special.1
        )
    };

    let moves: Element<'a, Message> = if plan.is_empty() || plan[0].damage == 0 {
        text::body(fl!(
            "versus-no-moves",
            defender = (defender.display.get(lang))
        ))
        .into()
    } else {
        widget::column::with_children(
            plan.iter()
                .filter(|hit| hit.damage > 0)
                .take(BEST_MOVES)
                .map(|hit| {
                    let mut name = vec![text::body(hit.attack.display.get(lang)).into()];
                    match hit.pace {
                        Pace::Charge => name.push(text::caption(fl!("versus-charge")).into()),
                        Pace::Recharge => {
                            name.push(text::caption(fl!("versus-recharge")).into());
                        }
                        Pace::Normal | Pace::SelfKo => {}
                    }
                    widget::row::with_children(vec![
                        type_icon(hit.attack.move_type, 16),
                        widget::row::with_children(name)
                            .spacing(6)
                            .align_y(Alignment::Center)
                            .width(Length::Fill)
                            .into(),
                        text::caption(hit.attack.damage_class.label())
                            .width(Length::Fixed(64.0))
                            .into(),
                        text::caption(
                            hit.attack
                                .power
                                .as_ref()
                                .map_or_else(String::new, |p| p.to_string()),
                        )
                        .width(Length::Fixed(30.0))
                        .align_x(Alignment::End)
                        .into(),
                        multiplier_pill(hit.multiplier),
                        text::caption(hit.label())
                            .width(Length::Fixed(110.0))
                            .align_x(Alignment::End)
                            .into(),
                    ])
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .into()
                })
                .collect::<Vec<_>>(),
        )
        .spacing(6)
        .into()
    };

    let threats: Vec<Element<'a, Message>> = defending
        .iter()
        .filter_map(|t| {
            let multiplier = dex.effectiveness(*t, &attacking);
            (multiplier >= 2.0).then(|| {
                widget::row::with_children(vec![type_badge(*t), multiplier_pill(multiplier)])
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .into()
            })
        })
        .collect();
    let threats: Element<'a, Message> = if threats.is_empty() {
        text::body(fl!("versus-threats-none")).into()
    } else {
        widget::flex_row(threats)
            .row_spacing(6)
            .column_spacing(12)
            .into()
    };

    widget::column::with_children(vec![
        text::title4(fl!(
            "versus-attacks",
            attacker = (attacker.display.get(lang)),
            defender = (defender.display.get(lang))
        ))
        .into(),
        section(fl!("versus-stab")).into(),
        stab.into(),
        text::body(lean).into(),
        section(fl!("versus-best-moves")).into(),
        moves,
        section(fl!(
            "versus-threats",
            defender = (defender.display.get(lang))
        ))
        .into(),
        threats,
    ])
    .spacing(8)
    .width(Length::Fill)
    .into()
}
