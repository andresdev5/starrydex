// SPDX-License-Identifier: GPL-3.0-only

//! The user's team: up to six Pokémon and how well they cover each other defensively.

use cosmic::iced::{Alignment, Background, Border, Color, Length};
use cosmic::widget::{self, button, container, text};
use cosmic::{Element, theme};
use starrydex_model::{ArchivedPokedex, ArchivedPokemon, PokemonType, SpriteKind};

use crate::app::pages::common::{card, hex, sprite, type_badge, type_icon};
use crate::app::pages::home::Section;
use crate::app::utils::presentation::{Label, data_lang, dex_number};
use crate::app::{AppModel, Message, Ready};
use crate::config::TEAM_SIZE;
use crate::{fl, icons};

const SLOT_WIDTH: f32 = 250.0;
const SLOT_HEIGHT: f32 = 96.0;
/// Wider than this, the coverage list is split in two columns.
const TWO_COLUMNS_WIDTH: f32 = 720.0;
const WEAK: u32 = 0xFF9EA8;
const RESIST: u32 = 0x7DD3A8;

pub fn view<'a>(app: &'a AppModel, ready: &'a Ready) -> Element<'a, Message> {
    let spacing = theme::active().cosmic().spacing;
    let dex = ready.core.dex();
    let team: Vec<&ArchivedPokemon> = app
        .config
        .team
        .iter()
        .filter_map(|id| dex.pokemon(*id))
        .collect();

    let mut title = vec![
        text::title3(fl!("team-title")).into(),
        text::caption(fl!("team-count", count = team.len())).into(),
        widget::Space::new().width(Length::Fill).into(),
    ];
    title.push(crate::app::pages::save::link());
    let title = widget::row::with_children(title)
        .spacing(spacing.space_s)
        .align_y(Alignment::Center);

    let slots: Vec<Element<'a, Message>> = (0..TEAM_SIZE)
        .map(|i| match team.get(i) {
            Some(pokemon) => member(ready, pokemon),
            None => empty_slot(),
        })
        .collect();

    let coverage = widget::responsive(move |size| coverage(dex, &team, size.width));

    widget::scrollable(
        widget::column::with_children(vec![
            title.into(),
            widget::flex_row(slots)
                .row_spacing(spacing.space_xs)
                .column_spacing(spacing.space_xs)
                .into(),
            coverage.into(),
        ])
        .spacing(spacing.space_s)
        .padding([spacing.space_xs, spacing.space_m, spacing.space_xs, 0])
        .width(Length::Fill),
    )
    .height(Length::Fill)
    .width(Length::Fill)
    .into()
}

fn member<'a>(ready: &'a Ready, pokemon: &'a ArchivedPokemon) -> Element<'a, Message> {
    let id = pokemon.id.to_native();
    let types = widget::row::with_children(
        pokemon
            .types
            .iter()
            .map(|t| type_icon(*t, 16))
            .collect::<Vec<_>>(),
    )
    .spacing(4);
    let info = widget::column::with_children(vec![
        text::caption(format!(
            "{} · {}",
            dex_number(pokemon.species_id.to_native()),
            pokemon.stats.total()
        ))
        .into(),
        text::heading(pokemon.display.get(data_lang())).into(),
        types.into(),
    ])
    .spacing(4)
    .width(Length::Fill);

    let open = button::custom(
        widget::row::with_children(vec![
            sprite(ready.core.sprite(pokemon, SpriteKind::Front), 56.0),
            info.into(),
        ])
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .padding(6)
    .width(Length::Fill)
    .class(theme::Button::IconVertical)
    .on_press(Message::SelectPokemon(id));

    container(
        widget::row::with_children(vec![
            open.into(),
            widget::tooltip(
                button::icon(icons::get_handle("window-close-symbolic", 18))
                    .on_press(Message::ToggleTeam(id)),
                text::body(fl!("team-remove")),
                widget::tooltip::Position::Bottom,
            )
            .into(),
        ])
        .align_y(Alignment::Start),
    )
    .padding(6)
    .width(Length::Fixed(SLOT_WIDTH))
    .height(Length::Fixed(SLOT_HEIGHT))
    .class(theme::Container::Card)
    .into()
}

fn empty_slot<'a>() -> Element<'a, Message> {
    button::custom(
        container(text::body(fl!("team-add-slot")))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .class(theme::Container::custom(|theme| container::Style {
                border: Border {
                    color: theme.cosmic().bg_divider().into(),
                    width: 2.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            })),
    )
    .padding(0)
    .width(Length::Fixed(SLOT_WIDTH))
    .height(Length::Fixed(SLOT_HEIGHT))
    .class(theme::Button::Text)
    .on_press(Message::ShowSection(Section::Pokemon))
    .into()
}

/// For every attacking type, how many members are weak to it and how many resist it.
fn coverage<'a>(
    dex: &'a ArchivedPokedex,
    team: &[&'a ArchivedPokemon],
    width: f32,
) -> Element<'a, Message> {
    let mut holes = Vec::new();
    let rows: Vec<Element<'a, Message>> = PokemonType::ALL
        .iter()
        .map(|&attacker| {
            let (mut weak, mut resist) = (0, 0);
            for member in team {
                let types: Vec<PokemonType> = member.types.iter().copied().collect();
                let multiplier = dex.effectiveness(attacker, &types);
                if multiplier > 1.0 {
                    weak += 1;
                } else if multiplier < 1.0 {
                    resist += 1;
                }
            }
            if weak >= 2 && resist == 0 {
                holes.push(attacker.label());
            }
            widget::row::with_children(vec![
                container(type_badge(attacker))
                    .width(Length::Fixed(130.0))
                    .into(),
                container(marks(weak, WEAK))
                    .width(Length::Fill)
                    .align_x(Alignment::End)
                    .into(),
                container(marks(resist, RESIST)).width(Length::Fill).into(),
            ])
            .spacing(8)
            .align_y(Alignment::Center)
            .into()
        })
        .collect();

    let note = if team.is_empty() {
        fl!("team-coverage-empty")
    } else if holes.is_empty() {
        fl!("team-coverage-ok")
    } else {
        fl!("team-coverage-holes", types = (holes.join(", ")))
    };

    let list: Element<'a, Message> = if width >= TWO_COLUMNS_WIDTH {
        let mut left = rows;
        let right = left.split_off(left.len().div_ceil(2));
        widget::row::with_children(vec![
            widget::column::with_children(left)
                .spacing(4)
                .width(Length::Fill)
                .into(),
            widget::column::with_children(right)
                .spacing(4)
                .width(Length::Fill)
                .into(),
        ])
        .spacing(24)
        .into()
    } else {
        widget::column::with_children(rows).spacing(4).into()
    };

    let legend = widget::row::with_children(vec![
        marks(1, WEAK),
        text::caption(fl!("team-coverage-weak")).into(),
        marks(1, RESIST),
        text::caption(fl!("team-coverage-resist")).into(),
    ])
    .spacing(6)
    .align_y(Alignment::Center);

    card(
        fl!("team-coverage"),
        widget::column::with_children(vec![legend.into(), list, text::body(note).into()])
            .spacing(10),
    )
}

/// `count` small colored squares.
fn marks<'a>(count: usize, color: u32) -> Element<'a, Message> {
    widget::row::with_children(
        (0..count)
            .map(|_| {
                container(widget::Space::new())
                    .width(Length::Fixed(10.0))
                    .height(Length::Fixed(14.0))
                    .class(theme::Container::custom(move |_| container::Style {
                        background: Some(Background::Color(hex(color))),
                        border: Border {
                            color: Color::TRANSPARENT,
                            width: 0.0,
                            radius: 2.0.into(),
                        },
                        ..Default::default()
                    }))
                    .into()
            })
            .collect::<Vec<_>>(),
    )
    .spacing(2)
    .into()
}
