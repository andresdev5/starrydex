// SPDX-License-Identifier: GPL-3.0-only

//! Type chart: how every attacking type fares against every defending type, plus a
//! calculator for the weaknesses of any one or two type combination.

use cosmic::iced::{Alignment, Background, Border, Color, Length};
use cosmic::widget::{self, button, container, text};
use cosmic::{Element, theme};
use starrydex_model::{ArchivedPokedex, PokemonType};

use crate::app::pages::common::{
    card, defense_rows, hex, multiplier_label, type_badge, type_chip, type_icon, type_icon_tooltip,
};
use crate::app::utils::presentation::Label;
use crate::app::{Message, Ready, TypesInput};
use crate::fl;

/// Side of a chart cell.
const CELL: f32 = 34.0;
/// Width of the attacking type column.
const LABEL_WIDTH: f32 = 136.0;
/// Width of the chart card: labels, 18 cells with their spacing, and the card padding.
const CHART_CARD_WIDTH: f32 = LABEL_WIDTH + 18.0 * (CELL + 2.0) + 28.0;
/// Narrowest the calculator gets when it sits beside the chart.
const CALCULATOR_MIN_WIDTH: f32 = 360.0;

const SUPER_EFFECTIVE: u32 = 0xA8E6C1;
const NOT_VERY_EFFECTIVE: u32 = 0xFFC4C4;
const NO_EFFECT: u32 = 0x05070B;
const DARK_TEXT: u32 = 0x141A26;
const LIGHT_TEXT: u32 = 0xE6EDF7;

/// How a chart cell relates to the hovered one.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Highlight {
    None,
    /// Same row or column as the hovered cell
    Line,
    /// The hovered cell itself
    Hit,
}

pub fn view<'a>(ready: &'a Ready) -> Element<'a, Message> {
    let page = widget::responsive(move |size| {
        let spacing = theme::active().cosmic().spacing;
        let dex = ready.core.dex();
        let side_by_side = size.width
            >= CHART_CARD_WIDTH
                + CALCULATOR_MIN_WIDTH
                + f32::from(spacing.space_s + spacing.space_m);

        let matchups_card = match ready.selected_type {
            Some(t) => matchups(dex, t),
            None => card(
                fl!("type-matchups-title"),
                text::body(fl!("type-chart-select-hint")),
            ),
        };
        let side = widget::column::with_children(vec![calculator(ready), matchups_card])
            .spacing(spacing.space_s)
            .width(Length::Fill);

        let chart_card = card(
            fl!("type-chart"),
            widget::column::with_children(vec![
                text::body(fl!("type-chart-hint")).into(),
                legend(),
                // Outside the scrollable: inside it, the cursor is unknown once it leaves the
                // chart, so the exit would never be noticed.
                widget::mouse_area(
                    widget::scrollable::horizontal(chart(
                        dex,
                        ready.selected_type,
                        ready.type_hover,
                    ))
                    .width(Length::Fill),
                )
                .on_exit(Message::Types(TypesInput::Hover(None)))
                .into(),
                text::heading(hover_text(dex, ready.type_hover)).into(),
            ])
            .spacing(spacing.space_xs),
        );

        let content: Element<'a, Message> = if side_by_side {
            widget::row::with_children(vec![
                container(chart_card)
                    .width(Length::Fixed(CHART_CARD_WIDTH))
                    .into(),
                side.into(),
            ])
            .spacing(spacing.space_s)
            .align_y(Alignment::Start)
            .into()
        } else {
            widget::column::with_children(vec![chart_card, side.into()])
                .spacing(spacing.space_s)
                .into()
        };

        widget::scrollable(
            container(content)
                .padding([spacing.space_xs, spacing.space_m, spacing.space_xs, 0])
                .width(Length::Fill),
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
    });
    // The page scrolls too, so the chart can't notice the cursor leaving to the nav bar.
    widget::mouse_area(page)
        .on_exit(Message::Types(TypesInput::Hover(None)))
        .into()
}

fn hover_text(dex: &ArchivedPokedex, hover: Option<(PokemonType, PokemonType)>) -> String {
    match hover {
        Some((attacker, defender)) => fl!(
            "type-chart-hover",
            attacker = (attacker.label()),
            defender = (defender.label()),
            multiplier = (multiplier_label(dex.effectiveness(attacker, &[defender])))
        ),
        None => fl!("type-chart-hover-hint"),
    }
}

/// Weaknesses and resistances of the one or two types picked by the user.
fn calculator<'a>(ready: &'a Ready) -> Element<'a, Message> {
    let picked = &ready.defense_types;
    let chips: Vec<Element<'a, Message>> = PokemonType::ALL
        .iter()
        .map(|&t| {
            button::custom(type_chip(t, picked.contains(&t), true))
                .padding(0)
                .class(theme::Button::Text)
                .on_press(Message::Types(TypesInput::ToggleDefender(t)))
                .into()
        })
        .collect();

    let mut content = vec![
        text::body(fl!("type-calculator-hint")).into(),
        widget::flex_row(chips)
            .row_spacing(4)
            .column_spacing(4)
            .into(),
    ];
    if !picked.is_empty() {
        content.push(widget::divider::horizontal::default().into());
        content.push(defense_rows(ready.core.dex(), picked, true));
        content.push(
            button::text(fl!("type-calculator-clear"))
                .on_press(Message::Types(TypesInput::ClearDefenders))
                .into(),
        );
    }
    card(
        fl!("type-calculator"),
        widget::column::with_children(content).spacing(10),
    )
}

/// Full 18×18 grid: rows attack, columns defend.
fn chart<'a>(
    dex: &'a ArchivedPokedex,
    selected: Option<PokemonType>,
    hover: Option<(PokemonType, PokemonType)>,
) -> Element<'a, Message> {
    let hovered_row = hover.map(|(attacker, _)| attacker);
    let hovered_column = hover.map(|(_, defender)| defender);

    let mut header = vec![
        container(text::caption(fl!("type-chart-corner")))
            .width(Length::Fixed(LABEL_WIDTH))
            .into(),
    ];
    header.extend(PokemonType::ALL.iter().map(|defender| {
        ring(
            container(type_icon_tooltip(*defender, 20))
                .width(Length::Fixed(CELL))
                .height(Length::Fixed(CELL))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
            hovered_column == Some(*defender),
        )
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

        let mut cells: Vec<Element<'a, Message>> =
            vec![ring(container(label), hovered_row == Some(*attacker))];
        cells.extend(PokemonType::ALL.iter().map(|defender| {
            let highlight = match hover {
                Some(h) if h == (*attacker, *defender) => Highlight::Hit,
                Some((a, d)) if a == *attacker || d == *defender => Highlight::Line,
                _ => Highlight::None,
            };
            widget::mouse_area(cell(dex.effectiveness_x2(*attacker, *defender), highlight))
                .on_enter(Message::Types(TypesInput::Hover(Some((
                    *attacker, *defender,
                )))))
                .into()
        }));
        rows.push(
            widget::row::with_children(cells)
                .spacing(2)
                .align_y(Alignment::Center)
                .into(),
        );
    }

    widget::column::with_children(rows).spacing(2).into()
}

/// Outlines `content` with the accent color when `on`.
fn ring<'a>(
    content: widget::Container<'a, Message, cosmic::Theme, cosmic::Renderer>,
    on: bool,
) -> Element<'a, Message> {
    content
        .class(theme::Container::custom(move |theme| container::Style {
            border: Border {
                color: if on {
                    theme.cosmic().accent_color().into()
                } else {
                    Color::TRANSPARENT
                },
                width: 2.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }))
        .into()
}

/// One chart cell; `x2` is the damage multiplier times two.
fn cell<'a>(x2: u8, highlight: Highlight) -> Element<'a, Message> {
    let (label, colors) = match x2 {
        0 => ("0", Some((NO_EFFECT, LIGHT_TEXT))),
        1 => ("½", Some((NOT_VERY_EFFECTIVE, DARK_TEXT))),
        4 => ("2", Some((SUPER_EFFECTIVE, DARK_TEXT))),
        _ => ("", None),
    };
    container(text::heading(label))
        .width(Length::Fixed(CELL))
        .height(Length::Fixed(CELL))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .class(theme::Container::custom(move |theme| {
            let cosmic = theme.cosmic();
            let accent: Color = cosmic.accent_color().into();
            let background = match (colors, highlight) {
                (Some((background, _)), _) => Some(hex(background)),
                (None, Highlight::Hit) => Some(Color { a: 0.45, ..accent }),
                (None, Highlight::Line) => Some(Color { a: 0.15, ..accent }),
                (None, Highlight::None) => None,
            };
            container::Style {
                background: background.map(Background::Color),
                text_color: colors.map(|(_, text)| hex(text)),
                border: Border {
                    color: match highlight {
                        Highlight::Hit => accent,
                        _ if colors.is_none() => cosmic.bg_divider().into(),
                        _ => Color::TRANSPARENT,
                    },
                    width: if highlight == Highlight::Hit {
                        2.0
                    } else {
                        1.0
                    },
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        }))
        .into()
}

fn legend<'a>() -> Element<'a, Message> {
    let item = |x2: u8, label: String| -> Element<'a, Message> {
        widget::row::with_children(vec![cell(x2, Highlight::None), text::body(label).into()])
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
