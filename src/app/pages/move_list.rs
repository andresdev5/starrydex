// SPDX-License-Identifier: GPL-3.0-only

//! Every move, searchable, filterable and sortable; the selected one is shown in the context
//! drawer together with the Pokémon that learn it.

use std::borrow::Cow;
use std::collections::BTreeSet;

use cosmic::iced::core::text::LineHeight;
use cosmic::iced::{Alignment, Length, Pixels};
use cosmic::widget::{self, button, container, text};
use cosmic::{Element, theme};
use starrydex_model::{ArchivedMove, DamageClass, Generation, PokemonType, SpriteKind};

use crate::app::core::{MoveQuery, StarryCore};
use crate::app::pages::common::{card, info_row, sprite, type_badge, type_chip, type_icon_tooltip};
use crate::app::utils::presentation::{Label, data_lang};
use crate::app::utils::{MoveSort, PaginationAction};
use crate::app::{AppModel, Message, MovesInput, Ready, page_count};
use crate::{fl, icons};

pub const MOVES_SCROLL_ID: &str = "move-list";
pub const SEARCH_ID: &str = "move-search";
/// Narrower than this, the effect column is left out (it is still in the move drawer).
const EFFECT_MIN_WIDTH: f32 = 900.0;
const TYPE_WIDTH: f32 = 48.0;
const CLASS_WIDTH: f32 = 90.0;
const NUMBER_WIDTH: f32 = 64.0;
const GENERATION_WIDTH: f32 = 72.0;

/// State of the moves section.
#[derive(Debug, Default)]
pub struct MoveList {
    pub search: String,
    pub types: BTreeSet<PokemonType>,
    pub class: Option<DamageClass>,
    pub generation: Option<Generation>,
    pub sort: MoveSort,
    pub descending: bool,
    /// Indices (into the Pokédex moves) matching the search and filters
    pub results: Vec<u32>,
    /// Current page of the list (0-based)
    pub page: usize,
    /// Move shown in the context drawer
    pub selected: Option<u32>,
    /// Pokémon (indices) that learn the selected move
    pub learners: Vec<u32>,
    /// Labels of the generation dropdown, "All generations" first
    pub generation_labels: Vec<String>,
}

impl MoveList {
    pub fn new(core: &StarryCore) -> Self {
        let mut list = Self {
            generation_labels: std::iter::once(fl!("moves-all-generations"))
                .chain(generations().map(|g| g.label()))
                .collect(),
            ..Default::default()
        };
        list.refresh(core);
        list
    }

    /// Re-runs the query after the search, a filter or the order changed.
    pub fn refresh(&mut self, core: &StarryCore) {
        self.results = core.move_query(&MoveQuery {
            search: &self.search,
            types: &self.types,
            class: self.class,
            generation: self.generation,
            sort: self.sort,
            descending: self.descending,
            lang: data_lang(),
        });
        self.page = 0;
    }
}

/// Generations a move can be introduced in.
pub fn generations() -> impl Iterator<Item = Generation> {
    Generation::ALL
        .iter()
        .copied()
        .filter(|g| *g != Generation::Unknown)
}

pub fn view<'a>(app: &'a AppModel, ready: &'a Ready) -> Element<'a, Message> {
    let spacing = theme::active().cosmic().spacing;
    let list = &ready.moves;
    let per_page = app.config.pokemon_per_page.max(1);
    let pages = page_count(list.results.len(), per_page);
    let page = list.page.min(pages - 1);
    let start = page * per_page;
    let end = (start + per_page).min(list.results.len());

    let search = widget::search_input(fl!("moves-search-placeholder"), &list.search)
        .id(widget::Id::new(SEARCH_ID))
        .on_input(|v| Message::Moves(MovesInput::Search(v)))
        .on_clear(Message::Moves(MovesInput::Search(String::new())))
        .line_height(LineHeight::Absolute(Pixels(30.0)))
        .width(Length::Fill);

    let class_button = |label: String, class: Option<DamageClass>| -> Element<'a, Message> {
        button::text(label)
            .class(if list.class == class {
                theme::Button::Suggested
            } else {
                theme::Button::Text
            })
            .on_press(Message::Moves(MovesInput::Class(class)))
            .into()
    };
    let mut filters = vec![class_button(fl!("moves-all-classes"), None)];
    filters.extend(
        DamageClass::ALL
            .iter()
            .map(|c| class_button(c.label(), Some(*c))),
    );
    filters.push(widget::Space::new().width(Length::Fill).into());
    filters.push(
        widget::dropdown(
            Cow::Borrowed(list.generation_labels.as_slice()),
            Some(
                list.generation
                    .and_then(|g| generations().position(|x| x == g))
                    .map_or(0, |i| i + 1),
            ),
            |i| Message::Moves(MovesInput::Generation(i)),
        )
        .into(),
    );
    let filters = widget::row::with_children(filters)
        .spacing(4)
        .align_y(Alignment::Center);

    let mut chips = vec![
        button::text(fl!("all-types"))
            .class(if list.types.is_empty() {
                theme::Button::Suggested
            } else {
                theme::Button::Text
            })
            .on_press(Message::Moves(MovesInput::ClearTypes))
            .into(),
    ];
    chips.extend(PokemonType::ALL.iter().map(|&t| {
        let on = list.types.contains(&t);
        widget::tooltip(
            button::custom(type_chip(t, on, false))
                .padding(0)
                .class(theme::Button::Text)
                .on_press(Message::Moves(MovesInput::ToggleType(t))),
            text::body(t.label()),
            widget::tooltip::Position::Bottom,
        )
        .into()
    }));
    let chips = widget::flex_row(chips)
        .row_spacing(4)
        .column_spacing(4)
        .width(Length::Fill);

    let summary = if list.results.is_empty() {
        fl!("moves-none")
    } else {
        fl!(
            "moves-range",
            start = (start + 1),
            end = end,
            total = list.results.len()
        )
    };
    let footer = widget::row::with_children(vec![
        text::caption(summary).width(Length::Fill).into(),
        button::icon(icons::get_handle("go-previous-symbolic", 18))
            .on_press_maybe(
                (page > 0).then_some(Message::Moves(MovesInput::Page(PaginationAction::Back))),
            )
            .into(),
        text::body(fl!("page-of", page = (page + 1), pages = pages)).into(),
        button::icon(icons::get_handle("go-next-symbolic", 18))
            .on_press_maybe(
                (page + 1 < pages)
                    .then_some(Message::Moves(MovesInput::Page(PaginationAction::Next))),
            )
            .into(),
        widget::Space::new().width(Length::Fill).into(),
    ])
    .spacing(spacing.space_s)
    .align_y(Alignment::Center);

    let visible = &list.results[start..end];
    let table = widget::responsive(move |size| {
        let show_effect = size.width >= EFFECT_MIN_WIDTH;
        if visible.is_empty() {
            return widget::column::with_children(vec![
                text::title3(fl!("moves-no-results")).into(),
                text::body(fl!("moves-no-results-hint")).into(),
            ])
            .spacing(12)
            .align_x(Alignment::Center)
            .padding(48)
            .width(Length::Fill)
            .into();
        }
        let dex = ready.core.dex();
        let rows = visible
            .iter()
            .map(|&index| move_row(ready, &dex.moves[index as usize], show_effect))
            .collect::<Vec<_>>();
        widget::column::with_children(vec![
            header_row(list, show_effect),
            widget::scrollable(widget::column::with_children(rows).spacing(2).padding([
                0,
                spacing.space_m,
                0,
                0,
            ]))
            .id(MOVES_SCROLL_ID)
            .height(Length::Fill)
            .width(Length::Fill)
            .into(),
        ])
        .spacing(spacing.space_xxs)
        .into()
    });

    widget::column::with_children(vec![
        search.into(),
        filters.into(),
        chips.into(),
        table.into(),
        footer.into(),
    ])
    .spacing(spacing.space_xs)
    .width(Length::Fill)
    .into()
}

/// Column titles; clicking one sorts by it, clicking it again reverses the order.
fn header_row<'a>(list: &'a MoveList, show_effect: bool) -> Element<'a, Message> {
    let title = |label: String, sort: MoveSort, width: Length, end: bool| -> Element<'a, Message> {
        // Texts read A→Z (old→new) by default, numbers high→low; the arrow shows the order
        // currently on screen.
        let ascending = matches!(
            sort,
            MoveSort::Name | MoveSort::Type | MoveSort::Class | MoveSort::Generation
        ) != list.descending;
        let arrow = match (list.sort == sort, ascending) {
            (false, _) => "",
            (true, true) => " ▲",
            (true, false) => " ▼",
        };
        button::custom(
            container(text::caption_heading(format!("{label}{arrow}")))
                .width(Length::Fill)
                .align_x(if end {
                    Alignment::End
                } else {
                    Alignment::Start
                }),
        )
        .padding([4, 0])
        .width(width)
        .class(theme::Button::Text)
        .on_press(Message::Moves(MovesInput::Sort(sort)))
        .into()
    };
    let mut cells: Vec<Element<'a, Message>> = vec![
        title(
            fl!("moves-type"),
            MoveSort::Type,
            Length::Fixed(TYPE_WIDTH),
            false,
        ),
        title(
            fl!("items-name"),
            MoveSort::Name,
            Length::FillPortion(3),
            false,
        ),
        title(
            fl!("moves-class"),
            MoveSort::Class,
            Length::Fixed(CLASS_WIDTH),
            false,
        ),
        title(
            fl!("power-short"),
            MoveSort::Power,
            Length::Fixed(NUMBER_WIDTH),
            true,
        ),
        title(
            fl!("accuracy-short"),
            MoveSort::Accuracy,
            Length::Fixed(NUMBER_WIDTH),
            true,
        ),
        title(fl!("pp"), MoveSort::Pp, Length::Fixed(NUMBER_WIDTH), true),
        title(
            fl!("moves-generation"),
            MoveSort::Generation,
            Length::Fixed(GENERATION_WIDTH),
            true,
        ),
    ];
    if show_effect {
        cells.push(
            text::caption_heading(fl!("item-effect"))
                .width(Length::FillPortion(5))
                .into(),
        );
    }
    container(
        widget::row::with_children(cells)
            .spacing(12)
            .align_y(Alignment::Center),
    )
    .padding([2, 8 + theme::active().cosmic().spacing.space_m, 2, 8])
    .width(Length::Fill)
    .class(theme::Container::Card)
    .into()
}

fn number(value: Option<&u8>) -> String {
    value.map_or_else(|| "—".to_string(), u8::to_string)
}

/// Effect in the UI language: the in-game description when the effect is only in English.
fn localized_effect(attack: &ArchivedMove) -> &str {
    let lang = data_lang();
    let effect = if attack.short_effect.has(lang) || !attack.flavor.has(lang) {
        attack.short_effect.get(lang)
    } else {
        attack.flavor.get(lang)
    };
    if effect.is_empty() {
        attack.flavor.get(lang)
    } else {
        effect
    }
}

fn move_row<'a>(
    ready: &'a Ready,
    attack: &'a ArchivedMove,
    show_effect: bool,
) -> Element<'a, Message> {
    let lang = data_lang();
    let mut cells: Vec<Element<'a, Message>> = vec![
        container(type_icon_tooltip(attack.move_type, 18))
            .width(Length::Fixed(TYPE_WIDTH))
            .into(),
        text::heading(attack.display.get(lang))
            .width(Length::FillPortion(3))
            .into(),
        text::body(attack.damage_class.label())
            .width(Length::Fixed(CLASS_WIDTH))
            .into(),
        text::body(number(attack.power.as_ref()))
            .width(Length::Fixed(NUMBER_WIDTH))
            .align_x(Alignment::End)
            .into(),
        text::body(
            attack
                .accuracy
                .as_ref()
                .map_or_else(|| "—".to_string(), |a| format!("{a}%")),
        )
        .width(Length::Fixed(NUMBER_WIDTH))
        .align_x(Alignment::End)
        .into(),
        text::body(number(attack.pp.as_ref()))
            .width(Length::Fixed(NUMBER_WIDTH))
            .align_x(Alignment::End)
            .into(),
        text::body(attack.generation.label())
            .width(Length::Fixed(GENERATION_WIDTH))
            .align_x(Alignment::End)
            .into(),
    ];
    if show_effect {
        cells.push(
            text::body(localized_effect(attack).replace('\n', " "))
                .width(Length::FillPortion(5))
                .into(),
        );
    }

    button::custom(
        widget::row::with_children(cells)
            .spacing(12)
            .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding([6, 8])
    .selected(ready.moves.selected == Some(attack.id.to_native()))
    .on_press(Message::Moves(MovesInput::Open(attack.id.to_native())))
    .class(theme::Button::IconVertical)
    .into()
}

/// Contents of the move context drawer.
pub fn details(ready: &Ready) -> Element<'_, Message> {
    let core = &ready.core;
    let dex = core.dex();
    let lang = data_lang();
    let Some(attack) = ready.moves.selected.and_then(|id| dex.move_(id)) else {
        return text::body(fl!("unknown")).into();
    };

    let name = attack.display.get(lang);
    let mut title = vec![text::title3(name).into()];
    if attack.display.en.as_str() != name {
        title.push(text::caption(attack.display.en.as_str()).into());
    }
    title.push(type_badge(attack.move_type));

    let mut info = vec![
        info_row(fl!("moves-class"), text::body(attack.damage_class.label())),
        info_row(
            fl!("power-short"),
            text::body(number(attack.power.as_ref())),
        ),
        info_row(
            fl!("accuracy-short"),
            text::body(
                attack
                    .accuracy
                    .as_ref()
                    .map_or_else(|| "—".to_string(), |a| format!("{a}%")),
            ),
        ),
        info_row(fl!("pp"), text::body(number(attack.pp.as_ref()))),
        info_row(fl!("target"), text::body(attack.target.get(lang))),
        info_row(fl!("introduced"), text::body(attack.generation.label())),
    ];
    if attack.priority != 0 {
        info.push(info_row(
            fl!("priority"),
            text::body(format!("{:+}", attack.priority)),
        ));
    }

    let mut sections: Vec<Element<'_, Message>> = vec![
        widget::column::with_children(title).spacing(6).into(),
        card(
            fl!("item-info"),
            widget::column::with_children(info).spacing(6),
        ),
    ];

    let effect = localized_effect(attack);
    if !effect.is_empty() {
        sections.push(card(
            fl!("item-effect"),
            text::body(effect.replace('\n', " ")),
        ));
    }
    let flavor = attack.flavor.get(lang);
    if !flavor.is_empty() && flavor != effect {
        sections.push(card(
            fl!("item-description"),
            text::body(flavor.replace('\n', " ")),
        ));
    }

    // TM/HM/TR numbers, newest games first.
    let mut machines: Vec<(u16, String)> = attack
        .machines
        .iter()
        .map(|m| {
            let group = dex.version_group(m.version_group_id.to_native());
            (
                group.map_or(0, |g| g.order.to_native()),
                format!(
                    "{} · {}",
                    m.label.as_str(),
                    group.map_or("", |g| g.display.get(lang))
                ),
            )
        })
        .collect();
    machines.sort_by_key(|(order, _)| std::cmp::Reverse(*order));
    if !machines.is_empty() {
        sections.push(card(
            fl!("moves-machines"),
            widget::column::with_children(
                machines
                    .into_iter()
                    .map(|(_, label)| text::body(label).into())
                    .collect::<Vec<_>>(),
            )
            .spacing(2),
        ));
    }

    let learners: Vec<Element<'_, Message>> = ready
        .moves
        .learners
        .iter()
        .filter_map(|&i| dex.pokemon.get(i as usize))
        .map(|p| {
            widget::tooltip(
                button::custom(sprite(core.sprite(p, SpriteKind::Front), 44.0))
                    .padding(2)
                    .class(theme::Button::IconVertical)
                    .on_press(Message::OpenPokemon(p.id.to_native())),
                text::body(p.display.get(lang)),
                widget::tooltip::Position::Top,
            )
            .into()
        })
        .collect();
    sections.push(card(
        fl!("moves-learned-by", count = learners.len()),
        if learners.is_empty() {
            Element::from(text::body(fl!("type-chart-none")))
        } else {
            widget::flex_row(learners)
                .row_spacing(2)
                .column_spacing(2)
                .into()
        },
    ));

    widget::column::with_children(sections).spacing(12).into()
}
