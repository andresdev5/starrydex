// SPDX-License-Identifier: GPL-3.0

use std::borrow::Cow;

use cosmic::iced::core::text::{LineHeight, Wrapping};
use cosmic::iced::{Alignment, Length, Pixels};
use cosmic::widget::{self, button, container, segmented_button, text};
use cosmic::{Element, theme};
use starrydex_model::{ArchivedPokemon, PokemonType, SpriteKind};

use crate::app::context_page::ContextPage;
use crate::app::core::StarryCore;
use crate::app::pages::common::{sprite, toggle_icon, type_chip, type_icon};
use crate::app::pages::details::{self, DetailTab, Marks};
use crate::app::utils::presentation::{Label, data_lang, dex_number};
use crate::app::utils::{PaginationAction, SortBy};
use crate::app::{AppModel, FiltersInput, ListInput, Message, Ready, page_count};
use crate::config::TEAM_SIZE;
use crate::{fl, icons};

pub const LIST_SCROLL_ID: &str = "pokemon-list";
pub const SEARCH_ID: &str = "pokemon-search";
/// Height of a row of the Pokémon list. Fixed so the selected row can be scrolled into view.
pub const ROW_HEIGHT: f32 = 56.0;
pub const ROW_SPACING: f32 = 2.0;
/// Width of the Pokémon list when it is shown next to the details.
const LIST_WIDTH: f32 = 370.0;
/// Narrower than this, the list and the details are shown one at a time.
const SPLIT_MIN_WIDTH: f32 = 860.0;

/// Sections of the app, picked from the navigation bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Section {
    #[default]
    Pokemon,
    Items,
    Moves,
    Types,
    Compare,
    Team,
    Save,
}

impl Section {
    pub const ALL: &'static [Self] = &[
        Self::Pokemon,
        Self::Items,
        Self::Moves,
        Self::Types,
        Self::Compare,
        Self::Team,
        Self::Save,
    ];

    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Pokemon => "view-list-symbolic",
            Self::Items => "package-x-generic-symbolic",
            Self::Moves => "moves-symbolic",
            Self::Types => "view-grid-symbolic",
            Self::Compare => "view-dual-symbolic",
            Self::Team => "system-users-symbolic",
            Self::Save => "pokeball-symbolic",
        }
    }
}

impl Label for Section {
    fn label(&self) -> String {
        match self {
            Self::Pokemon => fl!("home-pokemon"),
            Self::Types => fl!("home-types"),
            Self::Items => fl!("home-items"),
            Self::Moves => fl!("home-moves"),
            Self::Compare => fl!("home-compare"),
            Self::Team => fl!("home-team"),
            Self::Save => fl!("home-save"),
        }
    }
}

pub fn view<'a>(
    app: &'a AppModel,
    ready: &'a Ready,
    tabs: &'a segmented_button::SingleSelectModel,
    tab: DetailTab,
    section: Section,
) -> Element<'a, Message> {
    let spacing = theme::active().cosmic().spacing;
    let content = match section {
        Section::Pokemon => return pokedex(app, ready, tabs, tab),
        Section::Types => crate::app::pages::types::view(ready),
        Section::Items => crate::app::pages::items::view(app, ready),
        Section::Moves => crate::app::pages::move_list::view(app, ready),
        Section::Compare => crate::app::pages::compare::view(ready),
        Section::Team => crate::app::pages::team::view(app, ready),
        Section::Save => crate::app::pages::save::view(app, ready),
    };
    container(content)
        .padding([spacing.space_xxs, spacing.space_s])
        .width(Length::Fill)
        .into()
}

/// The Pokémon list with the open Pokémon next to it, or one of the two on small windows.
fn pokedex<'a>(
    app: &'a AppModel,
    ready: &'a Ready,
    tabs: &'a segmented_button::SingleSelectModel,
    tab: DetailTab,
) -> Element<'a, Message> {
    widget::responsive(move |size| {
        let split = size.width >= SPLIT_MIN_WIDTH;
        let details_width = if split {
            size.width - LIST_WIDTH
        } else {
            size.width
        };
        let details = ready.details.as_ref().map(|d| {
            let show_back = !split || !d.history.is_empty();
            let id = ready.core.dex().pokemon[d.index].id.to_native();
            let marks = Marks {
                favorite: app.config.favorites.contains(&id),
                in_team: app.config.team.contains(&id),
                team_full: app.config.team.len() >= TEAM_SIZE,
            };
            details::view(&ready.core, d, tabs, tab, show_back, details_width, marks)
        });
        if !split {
            return details.unwrap_or_else(|| list_pane(app, ready));
        }
        widget::row::with_children(vec![
            container(list_pane(app, ready))
                .width(Length::Fixed(LIST_WIDTH))
                .height(Length::Fill)
                .into(),
            widget::divider::vertical::default().into(),
            details.unwrap_or_else(placeholder),
        ])
        .height(Length::Fill)
        .into()
    })
    .into()
}

fn placeholder<'a>() -> Element<'a, Message> {
    cosmic::iced::widget::center(
        widget::column::with_children(vec![
            text::title3(fl!("select-pokemon")).into(),
            text::body(fl!("select-pokemon-hint")).into(),
        ])
        .spacing(8)
        .align_x(Alignment::Center),
    )
    .into()
}

fn list_pane<'a>(app: &'a AppModel, ready: &'a Ready) -> Element<'a, Message> {
    let spacing = theme::active().cosmic().spacing;
    let per_page = app.config.pokemon_per_page.max(1);
    let pages = page_count(ready.results.len(), per_page);
    let page = ready.page.min(pages - 1);
    let start = page * per_page;
    let end = (start + per_page).min(ready.results.len());

    // TOOLBAR
    let filter_count = ready.filters.active_count();
    let filter_label = if filter_count > 0 {
        format!("{} ({filter_count})", fl!("filters-page"))
    } else {
        fl!("filters-page")
    };
    let has_query = !ready.search.is_empty()
        || ready.filters.any_applied()
        || ready.sort != Default::default()
        || ready.favorites_only;
    let order_icon = if ready.descending {
        "go-down-symbolic"
    } else {
        "go-up-symbolic"
    };

    let search = widget::row::with_children(vec![
        widget::search_input(fl!("search-placeholder"), &ready.search)
            .id(widget::Id::new(SEARCH_ID))
            .on_input(|v| Message::List(ListInput::Search(v)))
            .on_clear(Message::List(ListInput::Search(String::new())))
            .line_height(LineHeight::Absolute(Pixels(30.0)))
            .width(Length::Fill)
            .into(),
        widget::tooltip(
            toggle_icon("starred-symbolic", ready.favorites_only)
                .on_press(Message::List(ListInput::ToggleFavoritesOnly)),
            text::body(fl!("favorites-only")),
            widget::tooltip::Position::Bottom,
        )
        .into(),
        widget::tooltip(
            toggle_icon("filter-symbolic", filter_count > 0)
                .on_press(Message::ToggleContextPage(ContextPage::FiltersPage)),
            text::body(filter_label),
            widget::tooltip::Position::Bottom,
        )
        .into(),
    ])
    .spacing(spacing.space_xxs)
    .align_y(Alignment::Center);

    let toolbar = widget::row::with_children(vec![
        text::caption(fl!("sort-by")).into(),
        widget::dropdown(
            Cow::Borrowed(app.sort_options.as_slice()),
            SortBy::ALL.iter().position(|s| *s == ready.sort),
            |i| Message::List(ListInput::Sort(i)),
        )
        .into(),
        widget::tooltip(
            button::icon(icons::get_handle(order_icon, 18))
                .on_press(Message::List(ListInput::ToggleOrder)),
            text::body(fl!("reverse-order")),
            widget::tooltip::Position::Bottom,
        )
        .into(),
        widget::Space::new().width(Length::Fill).into(),
        widget::tooltip(
            button::icon(icons::get_handle("edit-clear-all-symbolic", 18))
                .class(theme::Button::Destructive)
                .on_press_maybe(has_query.then_some(Message::List(ListInput::ClearAll))),
            text::body(fl!("clear-all")),
            widget::tooltip::Position::Bottom,
        )
        .into(),
    ])
    .spacing(spacing.space_xxs)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    // LIST
    let core = &ready.core;
    let dex = core.dex();
    let visible = &ready.results[start..end];
    let selected = ready.details.as_ref().map(|d| d.index);
    let content: Element<'a, Message> = if visible.is_empty() {
        widget::column::with_children(vec![
            text::title3(fl!("no-results")).into(),
            text::body(fl!("no-results-hint")).into(),
            button::suggested(fl!("clear-all"))
                .on_press(Message::List(ListInput::ClearAll))
                .into(),
        ])
        .spacing(12)
        .align_x(Alignment::Center)
        .padding(48)
        .width(Length::Fill)
        .into()
    } else {
        widget::column::with_children(
            visible
                .iter()
                .map(|&index| {
                    let pokemon = &dex.pokemon[index as usize];
                    pokemon_row(
                        core,
                        pokemon,
                        ready.sort,
                        selected == Some(index as usize),
                        app.config.favorites.contains(&pokemon.id.to_native()),
                    )
                })
                .collect::<Vec<_>>(),
        )
        .spacing(ROW_SPACING)
        .padding([0, spacing.space_xs, 0, 0])
        .width(Length::Fill)
        .into()
    };

    // FOOTER
    let summary = if ready.results.is_empty() {
        fl!("results-none")
    } else {
        fl!(
            "results-range",
            start = (start + 1),
            end = end,
            total = ready.results.len()
        )
    };
    let footer = widget::row::with_children(vec![
        text::caption(summary).width(Length::Fill).into(),
        button::icon(icons::get_handle("go-previous-symbolic", 18))
            .on_press_maybe(
                (page > 0).then_some(Message::List(ListInput::Page(PaginationAction::Back))),
            )
            .into(),
        text::caption(format!("{}/{pages}", page + 1)).into(),
        button::icon(icons::get_handle("go-next-symbolic", 18))
            .on_press_maybe(
                (page + 1 < pages)
                    .then_some(Message::List(ListInput::Page(PaginationAction::Next))),
            )
            .into(),
    ])
    .spacing(spacing.space_xxs)
    .align_y(Alignment::Center);

    widget::column::with_children(vec![
        search.into(),
        toolbar.into(),
        type_chips(ready),
        widget::scrollable(content)
            .id(LIST_SCROLL_ID)
            .on_scroll(|viewport| {
                Message::List(ListInput::Scrolled(
                    viewport.absolute_offset(),
                    viewport.bounds().height,
                ))
            })
            .height(Length::Fill)
            .width(Length::Fill)
            .into(),
        footer.into(),
    ])
    .spacing(spacing.space_xxs)
    .padding([spacing.space_xxs, spacing.space_s])
    .width(Length::Fill)
    .into()
}

/// Quick type filter: one chip per type plus one that clears them all.
fn type_chips<'a>(ready: &'a Ready) -> Element<'a, Message> {
    let types = &ready.filters.types;
    let mut chips = vec![
        button::text(fl!("all-types"))
            .class(if types.is_empty() {
                theme::Button::Suggested
            } else {
                theme::Button::Text
            })
            .on_press(Message::Filters(FiltersInput::ClearTypes))
            .into(),
    ];
    chips.extend(PokemonType::ALL.iter().map(|&t| {
        let on = types.contains(&t);
        widget::tooltip(
            button::custom(type_chip(t, on, false))
                .padding(0)
                .class(theme::Button::Text)
                .on_press(Message::Filters(FiltersInput::Type(!on, t))),
            text::body(t.label()),
            widget::tooltip::Position::Bottom,
        )
        .into()
    }));
    widget::flex_row(chips)
        .row_spacing(4)
        .column_spacing(4)
        .width(Length::Fill)
        .into()
}

fn pokemon_row<'a>(
    core: &'a StarryCore,
    pokemon: &'a ArchivedPokemon,
    sort: SortBy,
    selected: bool,
    favorite: bool,
) -> Element<'a, Message> {
    let lang = data_lang();
    let types = widget::row::with_children(
        pokemon
            .types
            .iter()
            .map(|t| type_icon(*t, 16))
            .collect::<Vec<_>>(),
    )
    .spacing(4);

    let name = widget::column::with_children(vec![
        text::caption(dex_number(pokemon.species_id.to_native())).into(),
        text::heading(pokemon.display.get(lang))
            .wrapping(Wrapping::None)
            .into(),
    ])
    .width(Length::Fill);

    let star: Element<'a, Message> = if favorite {
        widget::icon(icons::get_handle("starred-symbolic", 16))
            .size(14)
            .class(theme::Svg::custom(|theme| cosmic::widget::svg::Style {
                color: Some(theme.cosmic().accent_color().into()),
            }))
            .into()
    } else {
        widget::Space::new().width(Length::Fixed(14.0)).into()
    };
    let row = widget::row::with_children(vec![
        sprite(core.sprite(pokemon, SpriteKind::Front), 44.0),
        name.into(),
        star,
        types.into(),
        sort_value(pokemon, sort),
    ])
    .spacing(10)
    .align_y(Alignment::Center);

    button::custom(row)
        .width(Length::Fill)
        .height(Length::Fixed(ROW_HEIGHT))
        .padding([4, 8])
        .selected(selected)
        .on_press(Message::SelectPokemon(pokemon.id.to_native()))
        .class(theme::Button::IconVertical)
        .into()
}

/// Value the list is sorted by (the base stat total when sorting by number or name),
/// highlighted when it is what the list is sorted by.
fn sort_value<'a>(pokemon: &ArchivedPokemon, sort: SortBy) -> Element<'a, Message> {
    let stats = &pokemon.stats;
    let value = match sort {
        SortBy::Number | SortBy::Name | SortBy::Total => stats.total().to_string(),
        SortBy::Hp => stats.hp.to_string(),
        SortBy::Attack => stats.attack.to_string(),
        SortBy::Defense => stats.defense.to_string(),
        SortBy::SpAttack => stats.sp_attack.to_string(),
        SortBy::SpDefense => stats.sp_defense.to_string(),
        SortBy::Speed => stats.speed.to_string(),
        SortBy::Height => format!("{:.1} m", f32::from(pokemon.height.to_native()) / 10.0),
        SortBy::Weight => format!("{:.1} kg", f32::from(pokemon.weight.to_native()) / 10.0),
    };
    let label = text::caption(value)
        .width(Length::Fixed(56.0))
        .align_x(Alignment::End);
    if matches!(sort, SortBy::Number | SortBy::Name) {
        label.into()
    } else {
        label.class(theme::Text::Accent).into()
    }
}
