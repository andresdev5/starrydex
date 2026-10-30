// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use cosmic::iced::core::image::FilterMethod;
use cosmic::iced::{Alignment, Background, Border, ContentFit, Length};
use cosmic::widget::{self, container, text};
use cosmic::{Element, theme};
use starrydex_model::{ArchivedPokedex, PokemonType};

use crate::app::Message;
use crate::app::utils::presentation::{Label, type_color, type_icon_name, type_svg_class};
use crate::{fl, icons, images};

/// A sprite scaled with nearest-neighbour filtering so pixel art stays crisp.
pub fn sprite<'a>(path: Option<PathBuf>, size: f32) -> Element<'a, Message> {
    let handle = match path {
        Some(path) => widget::image::Handle::from_path(path),
        None => images::get("fallback"),
    };
    widget::image(handle)
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .content_fit(ContentFit::Contain)
        .filter_method(FilterMethod::Nearest)
        .into()
}

/// Colored type icon.
pub fn type_icon<'a>(t: PokemonType, size: u16) -> Element<'a, Message> {
    widget::icon(icons::get_handle_owned(type_icon_name(t), 18))
        .size(size)
        .class(type_svg_class(t))
        .into()
}

/// Type icon with a tooltip.
pub fn type_icon_tooltip<'a>(t: PokemonType, size: u16) -> Element<'a, Message> {
    widget::tooltip(
        type_icon(t, size),
        text::body(t.label()),
        widget::tooltip::Position::Bottom,
    )
    .into()
}

/// Pill with the type icon and name, tinted with the type color.
pub fn type_badge<'a>(t: PokemonType) -> Element<'a, Message> {
    let color = type_color(t);
    container(
        widget::row::with_children(vec![type_icon(t, 16), text::body(t.label()).into()])
            .spacing(6)
            .align_y(Alignment::Center),
    )
    .padding([3, 10])
    .class(theme::Container::custom(move |_| container::Style {
        background: Some(Background::Color(cosmic::iced::Color { a: 0.18, ..color })),
        border: Border {
            color,
            width: 1.0,
            radius: 14.0.into(),
        },
        ..Default::default()
    }))
    .into()
}

/// Small rounded label ("Hidden", "Legendary"...).
pub fn tag<'a>(label: impl Into<String>) -> Element<'a, Message> {
    container(text::caption_heading(label.into()))
        .padding([2, 8])
        .class(theme::Container::custom(|theme| {
            let cosmic = theme.cosmic();
            container::Style {
                background: Some(Background::Color(cosmic.accent.base.into())),
                text_color: Some(cosmic.accent.on.into()),
                border: Border {
                    radius: 10.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }))
        .into()
}

/// A titled card.
pub fn card<'a>(
    title: impl Into<String>,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(
        widget::column::with_children(vec![text::title4(title.into()).into(), content.into()])
            .spacing(8)
            .width(Length::Fill),
    )
    .padding(12)
    .width(Length::Fill)
    .class(theme::Container::Card)
    .into()
}

/// Label/value row used in info cards.
pub fn info_row<'a>(
    label: impl Into<String>,
    value: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    widget::row::with_children(vec![
        text::body(label.into())
            .class(theme::Text::Accent)
            .width(Length::FillPortion(2))
            .into(),
        container(value.into()).width(Length::FillPortion(3)).into(),
    ])
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

/// Centered placeholder for empty sections.
pub fn empty<'a>(message: impl Into<String>) -> Element<'a, Message> {
    container(text::body(message.into()))
        .width(Length::Fill)
        .padding(24)
        .align_x(Alignment::Center)
        .into()
}

/// Number of segments of a [`stat_bar`].
const STAT_SEGMENTS: usize = 20;
/// Base stat that fills a [`stat_bar`] completely.
const STAT_BAR_MAX: f32 = 180.0;

/// Base stat drawn as a row of segments; values of 100 or more are highlighted.
pub fn stat_bar<'a>(value: u8) -> Element<'a, Message> {
    let filled =
        ((f32::from(value).min(STAT_BAR_MAX) / STAT_BAR_MAX * STAT_SEGMENTS as f32).round()
            as usize)
            .max(1);
    let strong = value >= 100;
    widget::row::with_children(
        (0..STAT_SEGMENTS)
            .map(|i| {
                let on = i < filled;
                container(widget::Space::new())
                    .width(Length::Fill)
                    .height(Length::Fixed(14.0))
                    .class(theme::Container::custom(move |theme| {
                        let cosmic = theme.cosmic();
                        let accent: cosmic::iced::Color = cosmic.accent_color().into();
                        let color = match (on, strong) {
                            (true, true) => accent,
                            (true, false) => cosmic::iced::Color { a: 0.5, ..accent },
                            (false, _) => cosmic.bg_divider().into(),
                        };
                        container::Style {
                            background: Some(Background::Color(color)),
                            border: Border {
                                radius: 2.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    }))
                    .into()
            })
            .collect::<Vec<_>>(),
    )
    .spacing(2)
    .width(Length::Fill)
    .into()
}

/// Damage multiplier as text: "×2", "×½"...
pub fn multiplier_label(multiplier: f32) -> &'static str {
    if multiplier >= 4.0 {
        "×4"
    } else if multiplier >= 2.0 {
        "×2"
    } else if multiplier >= 1.0 {
        "×1"
    } else if multiplier >= 0.5 {
        "×½"
    } else if multiplier > 0.0 {
        "×¼"
    } else {
        "×0"
    }
}

/// Damage multiplier on a pill colored by how good it is for the defender.
pub fn multiplier_pill<'a>(multiplier: f32) -> Element<'a, Message> {
    let (background, foreground) = if multiplier >= 4.0 {
        (0xFF9EA8, 0x141A26)
    } else if multiplier >= 2.0 {
        (0xFFD3D3, 0x141A26)
    } else if multiplier >= 1.0 {
        (0xDEDBCD, 0x141A26)
    } else if multiplier >= 0.5 {
        (0xC6EFD6, 0x141A26)
    } else if multiplier > 0.0 {
        (0x8FDDB3, 0x141A26)
    } else {
        (0x05070B, 0xE6EDF7)
    };
    container(text::heading(multiplier_label(multiplier)))
        .padding([2, 6])
        .width(Length::Fixed(40.0))
        .align_x(Alignment::Center)
        .class(theme::Container::custom(move |_| container::Style {
            background: Some(Background::Color(hex(background))),
            text_color: Some(hex(foreground)),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }))
        .into()
}

/// Attacking types grouped by the damage they deal to `defender` (one or two types), from
/// "very weak" to "immune". Groups without types are left out.
pub fn defense_rows<'a>(
    dex: &'a ArchivedPokedex,
    defender: &[PokemonType],
    stacked: bool,
) -> Element<'a, Message> {
    let groups: [(f32, String); 5] = [
        (4.0, fl!("very-weak")),
        (2.0, fl!("weak")),
        (0.5, fl!("resists")),
        (0.25, fl!("very-resistant")),
        (0.0, fl!("immune-label")),
    ];
    let mut rows: Vec<Element<'a, Message>> = Vec::new();
    for (multiplier, label) in groups {
        let matching: Vec<Element<'a, Message>> = PokemonType::ALL
            .iter()
            .filter(|attacker| (dex.effectiveness(**attacker, defender) - multiplier).abs() < 0.01)
            .map(|t| type_badge(*t))
            .collect();
        if matching.is_empty() {
            continue;
        }
        let label =
            widget::row::with_children(vec![multiplier_pill(multiplier), text::body(label).into()])
                .spacing(8)
                .align_y(Alignment::Center);
        let badges = widget::flex_row(matching).row_spacing(6).column_spacing(6);
        rows.push(if stacked {
            widget::column::with_children(vec![label.into(), badges.into()])
                .spacing(6)
                .into()
        } else {
            widget::row::with_children(vec![
                container(label).width(Length::Fixed(180.0)).into(),
                badges.into(),
            ])
            .spacing(8)
            .align_y(Alignment::Center)
            .into()
        });
    }
    widget::column::with_children(rows).spacing(10).into()
}

/// Toggleable type chip: tinted with the type color when `on`, outlined otherwise.
pub fn type_chip<'a>(t: PokemonType, on: bool, with_label: bool) -> Element<'a, Message> {
    let color = type_color(t);
    let content: Element<'a, Message> = if with_label {
        widget::row::with_children(vec![type_icon(t, 16), text::body(t.label()).into()])
            .spacing(6)
            .align_y(Alignment::Center)
            .into()
    } else {
        type_icon(t, 16)
    };
    container(content)
        .padding(if with_label { [3, 10] } else { [5, 5] })
        .class(theme::Container::custom(move |_| container::Style {
            background: on.then_some(Background::Color(cosmic::iced::Color { a: 0.35, ..color })),
            border: Border {
                color: if on {
                    color
                } else {
                    cosmic::iced::Color { a: 0.35, ..color }
                },
                width: 1.0,
                radius: if with_label { 14.0 } else { 8.0 }.into(),
            },
            ..Default::default()
        }))
        .into()
}

/// Small tile with a caption and a value ("Height · 1.7 m").
pub fn fact<'a>(label: impl Into<String>, value: impl Into<String>) -> Element<'a, Message> {
    container(
        widget::column::with_children(vec![
            text::caption(label.into()).into(),
            text::heading(value.into()).into(),
        ])
        .spacing(4),
    )
    .padding([10, 14])
    .width(Length::Fill)
    .class(theme::Container::Card)
    .into()
}

pub fn hex(value: u32) -> cosmic::iced::Color {
    cosmic::iced::Color::from_rgb8((value >> 16) as u8, (value >> 8) as u8, value as u8)
}

/// Icon button that shows its icon in the accent color while `on`.
pub fn toggle_icon<'a>(name: &'static str, on: bool) -> widget::Button<'a, Message> {
    widget::button::custom(
        widget::icon(icons::get_handle(name, 18))
            .size(18)
            .class(if on {
                theme::Svg::custom(|theme| widget::svg::Style {
                    color: Some(theme.cosmic().accent_color().into()),
                })
            } else {
                theme::Svg::default()
            }),
    )
    .padding(8)
    .class(theme::Button::Icon)
}
