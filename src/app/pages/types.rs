// SPDX-License-Identifier: GPL-3.0-only

//! Type chart: how every attacking type fares against every defending type.

use cosmic::iced::{Alignment, Background, Border, Color, Length};
use cosmic::widget::{self, button, container, text};
use cosmic::{Element, theme};
use starrydex_model::{ArchivedPokedex, PokemonType};

use crate::app::pages::common::{card, type_badge, type_icon, type_icon_tooltip};
use crate::app::utils::presentation::Label;
use crate::app::{Message, Ready};
use crate::fl;

/// Side of a chart cell.
const CELL: f32 = 34.0;
/// Width of the attacking type column.
const LABEL_WIDTH: f32 = 136.0;
/// Widest the page gets; beyond this it stays centered instead of stretching.
const MAX_CONTENT_WIDTH: f32 = 1100.0;

const SUPER_EFFECTIVE: Color = Color::from_rgb(0.30, 0.69, 0.31);
const NOT_VERY_EFFECTIVE: Color = Color::from_rgb(0.85, 0.26, 0.24);
const NO_EFFECT: Color = Color::from_rgb(0.20, 0.20, 0.22);

pub fn view<'a>(ready: &'a Ready) -> Element<'a, Message> {
    let spacing = theme::active().cosmic().spacing;
    let dex = ready.core.dex();

    let matchups_card = match ready.selected_type {
        Some(t) => matchups(dex, t),
        None => card(fl!("type-chart"), text::body(fl!("type-chart-select-hint"))),
    };

    let content = widget::column::with_children(vec![
        matchups_card,
        card(
            fl!("type-chart"),
            widget::column::with_children(vec![
                text::body(fl!("type-chart-hint")).into(),
                legend(),
                widget::scrollable::horizontal(chart(dex, ready.selected_type))
                    .width(Length::Fill)
                    .into(),
            ])
            .spacing(spacing.space_xs),
        ),
    ])
    .spacing(spacing.space_xs)
    .max_width(MAX_CONTENT_WIDTH);

    widget::scrollable(
        container(content)
            .padding([spacing.space_xs, 0])
            .width(Length::Fill)
            .align_x(Alignment::Center),
    )
    .height(Length::Fill)
    .width(Length::Fill)
    .into()
}

/// Full 18×18 grid: rows attack, columns defend.
fn chart<'a>(dex: &'a ArchivedPokedex, selected: Option<PokemonType>) -> Element<'a, Message> {
    let mut header = vec![
        container(text::caption(fl!("type-chart-corner")))
            .width(Length::Fixed(LABEL_WIDTH))
            .into(),
    ];
    header.extend(PokemonType::ALL.iter().map(|defender| {
        container(type_icon_tooltip(*defender, 20))
            .width(Length::Fixed(CELL))
            .height(Length::Fixed(CELL))
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    }));

    let mut rows: Vec<Element<'a, Message>> =
        vec![widget::row::with_children(header).spacing(2).into()];
    for attacker in PokemonType::ALL {
        let label = button::custom(
            widget::row::with_children(vec![
                type_icon(*attacker, 18),
                text::body(attacker.label()).into(),
            ])
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .width(Length::Fixed(LABEL_WIDTH))
        .padding([4, 6])
        .class(if selected == Some(*attacker) {
            theme::Button::Suggested
        } else {
            theme::Button::Text
        })
        .on_press(Message::SelectType(*attacker));

        let mut cells: Vec<Element<'a, Message>> = vec![label.into()];
        cells.extend(
            PokemonType::ALL
                .iter()
                .map(|defender| cell(dex.effectiveness_x2(*attacker, *defender))),
        );
        rows.push(
            widget::row::with_children(cells)
                .spacing(2)
                .align_y(Alignment::Center)
                .into(),
        );
    }

    widget::column::with_children(rows).spacing(2).into()
}

/// One chart cell; `x2` is the damage multiplier times two.
fn cell<'a>(x2: u8) -> Element<'a, Message> {
    let (label, color) = match x2 {
        0 => ("0", Some(NO_EFFECT)),
        1 => ("½", Some(NOT_VERY_EFFECTIVE)),
        4 => ("2", Some(SUPER_EFFECTIVE)),
        _ => ("", None),
    };
    container(text::body(label))
        .width(Length::Fixed(CELL))
        .height(Length::Fixed(CELL))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .class(theme::Container::custom(move |theme| {
            let cosmic = theme.cosmic();
            match color {
                Some(color) => container::Style {
                    background: Some(Background::Color(color)),
                    text_color: Some(Color::WHITE),
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                None => container::Style {
                    border: Border {
                        color: cosmic.bg_divider().into(),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                },
            }
        }))
        .into()
}

fn legend<'a>() -> Element<'a, Message> {
    let item = |x2: u8, label: String| -> Element<'a, Message> {
        widget::row::with_children(vec![cell(x2), text::body(label).into()])
            .spacing(6)
            .align_y(Alignment::Center)
            .into()
    };
    widget::flex_row(vec![
        item(4, fl!("type-chart-legend-super")),
        item(1, fl!("type-chart-legend-not-very")),
        item(0, fl!("type-chart-legend-none")),
        item(2, fl!("type-chart-legend-neutral")),
    ])
    .row_spacing(6)
    .column_spacing(16)
    .into()
}

/// What `t` hits hard or poorly, and what hits `t` hard or poorly.
fn matchups<'a>(dex: &'a ArchivedPokedex, t: PokemonType) -> Element<'a, Message> {
    let attacking = |x2: u8| -> Vec<PokemonType> {
        PokemonType::ALL
            .iter()
            .copied()
            .filter(|defender| dex.effectiveness_x2(t, *defender) == x2)
            .collect()
    };
    let defending = |x2: u8| -> Vec<PokemonType> {
        PokemonType::ALL
            .iter()
            .copied()
            .filter(|attacker| dex.effectiveness_x2(*attacker, t) == x2)
            .collect()
    };

    let offense = section(
        fl!("type-offense"),
        vec![
            (fl!("super-effective-against"), attacking(4)),
            (fl!("not-very-effective-against"), attacking(1)),
            (fl!("no-effect-against"), attacking(0)),
        ],
    );
    let defense = section(
        fl!("type-defense"),
        vec![
            (fl!("weak-2x"), defending(4)),
            (fl!("resist-half"), defending(1)),
            (fl!("immune"), defending(0)),
        ],
    );

    card(
        fl!("type-matchups", name = (t.label())),
        widget::column::with_children(vec![
            type_badge(t),
            widget::flex_row(vec![offense, defense])
                .row_spacing(12)
                .column_spacing(24)
                .into(),
        ])
        .spacing(12),
    )
}

fn section<'a>(title: String, groups: Vec<(String, Vec<PokemonType>)>) -> Element<'a, Message> {
    let mut children: Vec<Element<'a, Message>> = vec![text::title4(title).into()];
    for (label, types) in groups {
        let badges: Element<'a, Message> = if types.is_empty() {
            text::body(fl!("type-chart-none")).into()
        } else {
            widget::flex_row(types.into_iter().map(type_badge).collect())
                .row_spacing(6)
                .column_spacing(6)
                .into()
        };
        children.push(
            widget::column::with_children(vec![text::heading(label).into(), badges])
                .spacing(6)
                .into(),
        );
    }
    widget::column::with_children(children)
        .spacing(10)
        .width(Length::Fixed(440.0))
        .into()
}
