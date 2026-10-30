// SPDX-License-Identifier: GPL-3.0-only

//! Searchable list of every item; the selected one is shown in the context drawer.

use std::borrow::Cow;

use cosmic::iced::core::text::LineHeight;
use cosmic::iced::{Alignment, Length, Pixels};
use cosmic::widget::{self, button, container, text};
use cosmic::{Element, theme};
use starrydex_model::{ArchivedItem, SpriteKind};

use crate::app::core::StarryCore;
use crate::app::pages::common::{card, info_row, sprite};
use crate::app::utils::PaginationAction;
use crate::app::utils::presentation::data_lang;
use crate::app::{AppModel, ItemsInput, Message, Ready, page_count};
use crate::{fl, icons};

const CARD_WIDTH: f32 = 136.0;
const CARD_HEIGHT: f32 = 120.0;
pub const ITEMS_SCROLL_ID: &str = "item-list";

/// State of the items section.
#[derive(Debug, Default)]
pub struct ItemList {
    pub search: String,
    /// Bag pocket filter (`None` = every pocket)
    pub pocket: Option<u32>,
    /// Indices (into the Pokédex items) matching the search and pocket
    pub results: Vec<u32>,
    /// Current page of the list (0-based)
    pub page: usize,
    /// Item shown in the context drawer
    pub selected: Option<u32>,
    /// Labels of the pocket dropdown, "All pockets" first
    pub pocket_labels: Vec<String>,
    /// Pocket id of each dropdown entry
    pub pocket_ids: Vec<Option<u32>>,
}

impl ItemList {
    pub fn new(core: &StarryCore) -> Self {
        let lang = data_lang();
        let mut list = Self {
            pocket_labels: vec![fl!("items-all-pockets")],
            pocket_ids: vec![None],
            ..Default::default()
        };
        for pocket in core.dex().item_pockets.iter() {
            list.pocket_labels
                .push(pocket.display.get(lang).to_string());
            list.pocket_ids.push(Some(pocket.id.to_native()));
        }
        list.refresh(core);
        list
    }

    /// Re-runs the search after the query or the pocket changed.
    pub fn refresh(&mut self, core: &StarryCore) {
        self.results = core.item_query(&self.search, self.pocket, data_lang());
        self.page = 0;
    }
}

pub fn view<'a>(app: &'a AppModel, ready: &'a Ready) -> Element<'a, Message> {
    let spacing = theme::active().cosmic().spacing;
    let list = &ready.items;
    let per_page = app.config.pokemon_per_page.max(1);
    let pages = page_count(list.results.len(), per_page);
    let page = list.page.min(pages - 1);
    let start = page * per_page;
    let end = (start + per_page).min(list.results.len());

    let toolbar = widget::row::with_children(vec![
        widget::search_input(fl!("items-search-placeholder"), &list.search)
            .on_input(|v| Message::Items(ItemsInput::Search(v)))
            .on_clear(Message::Items(ItemsInput::Search(String::new())))
            .line_height(LineHeight::Absolute(Pixels(30.0)))
            .width(Length::Fill)
            .into(),
        widget::dropdown(
            Cow::Borrowed(list.pocket_labels.as_slice()),
            list.pocket_ids.iter().position(|id| *id == list.pocket),
            |i| Message::Items(ItemsInput::Pocket(i)),
        )
        .into(),
    ])
    .spacing(spacing.space_xxs)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let core = &ready.core;
    let dex = core.dex();
    let visible = &list.results[start..end];
    let content: Element<'a, Message> = if visible.is_empty() {
        widget::column::with_children(vec![
            text::title3(fl!("items-no-results")).into(),
            text::body(fl!("items-no-results-hint")).into(),
        ])
        .spacing(12)
        .align_x(Alignment::Center)
        .padding(48)
        .width(Length::Fill)
        .into()
    } else {
        widget::flex_row(
            visible
                .iter()
                .map(|&index| item_card(ready, &dex.items[index as usize]))
                .collect(),
        )
        .row_spacing(spacing.space_xs)
        .column_spacing(spacing.space_xs)
        .justify_content(widget::JustifyContent::Center)
        .width(Length::Fill)
        .into()
    };

    let summary = if list.results.is_empty() {
        fl!("items-none")
    } else {
        fl!(
            "items-range",
            start = (start + 1),
            end = end,
            total = list.results.len()
        )
    };
    let footer = widget::row::with_children(vec![
        text::caption(summary).width(Length::Fill).into(),
        button::icon(icons::get_handle("go-previous-symbolic", 18))
            .on_press_maybe(
                (page > 0).then_some(Message::Items(ItemsInput::Page(PaginationAction::Back))),
            )
            .into(),
        text::body(fl!("page-of", page = (page + 1), pages = pages)).into(),
        button::icon(icons::get_handle("go-next-symbolic", 18))
            .on_press_maybe(
                (page + 1 < pages)
                    .then_some(Message::Items(ItemsInput::Page(PaginationAction::Next))),
            )
            .into(),
        widget::Space::new().width(Length::Fill).into(),
    ])
    .spacing(spacing.space_s)
    .align_y(Alignment::Center);

    widget::column::with_children(vec![
        toolbar.into(),
        widget::scrollable(container(content).padding([spacing.space_xs, 0]))
            .id(ITEMS_SCROLL_ID)
            .height(Length::Fill)
            .width(Length::Fill)
            .into(),
        footer.into(),
    ])
    .spacing(spacing.space_xxs)
    .width(Length::Fill)
    .into()
}

fn item_card<'a>(ready: &'a Ready, item: &'a ArchivedItem) -> Element<'a, Message> {
    let lang = data_lang();
    let pocket = ready
        .core
        .dex()
        .item_pocket(item.pocket_id.to_native())
        .map(|p| p.display.get(lang))
        .unwrap_or_default();
    let column = widget::column::with_children(vec![
        sprite(ready.core.item_sprite(item), 48.0),
        text::heading(item.display.get(lang))
            .align_x(Alignment::Center)
            .wrapping(cosmic::iced::core::text::Wrapping::WordOrGlyph)
            .into(),
        text::caption(pocket).into(),
    ])
    .spacing(4)
    .width(Length::Fill)
    .align_x(Alignment::Center);

    button::custom(column)
        .width(Length::Fixed(CARD_WIDTH))
        .height(Length::Fixed(CARD_HEIGHT))
        .padding(6)
        .selected(ready.items.selected == Some(item.id.to_native()))
        .on_press(Message::Items(ItemsInput::Open(item.id.to_native())))
        .class(theme::Button::IconVertical)
        .into()
}

/// Contents of the item context drawer.
pub fn details(ready: &Ready) -> Element<'_, Message> {
    let core = &ready.core;
    let dex = core.dex();
    let lang = data_lang();
    let Some(item) = ready.items.selected.and_then(|id| dex.item(id)) else {
        return text::body(fl!("unknown")).into();
    };

    let name = item.display.get(lang);
    let mut title = vec![text::title3(name).into()];
    if item.display.en.as_str() != name {
        title.push(text::caption(item.display.en.as_str()).into());
    }
    let header = widget::row::with_children(vec![
        sprite(core.item_sprite(item), 64.0),
        widget::column::with_children(title)
            .spacing(2)
            .width(Length::Fill)
            .into(),
    ])
    .spacing(12)
    .align_y(Alignment::Center);

    let pocket = dex
        .item_pocket(item.pocket_id.to_native())
        .map(|p| p.display.get(lang).to_string())
        .unwrap_or_else(|| fl!("unknown"));
    let cost = match item.cost.as_ref() {
        Some(cost) => fl!("item-cost-value", cost = (cost.to_native())),
        None => fl!("item-cost-none"),
    };
    let mut sections: Vec<Element<'_, Message>> = vec![
        header.into(),
        card(
            fl!("item-info"),
            widget::column::with_children(vec![
                info_row(fl!("item-pocket"), text::body(pocket)),
                info_row(fl!("item-category"), text::body(item.category.as_str())),
                info_row(fl!("item-cost"), text::body(cost)),
            ])
            .spacing(6),
        ),
    ];

    let flavor = item.flavor.get(lang);
    if !flavor.is_empty() {
        sections.push(card(
            fl!("item-description"),
            text::body(flavor.replace('\n', " ")),
        ));
    }
    let effect = item.short_effect.get(lang);
    if !effect.is_empty() {
        sections.push(card(fl!("item-effect"), text::body(effect)));
    }

    let holders: Vec<Element<'_, Message>> = dex
        .pokemon
        .iter()
        .filter(|p| {
            p.held_items
                .iter()
                .any(|h| h.item_id.to_native() == item.id.to_native())
        })
        .map(|p| {
            button::custom(
                widget::row::with_children(vec![
                    sprite(core.sprite(p, SpriteKind::Front), 40.0),
                    text::body(p.display.get(lang)).into(),
                ])
                .spacing(8)
                .align_y(Alignment::Center),
            )
            .padding([2, 6])
            .width(Length::Fill)
            .class(theme::Button::Text)
            .on_press(Message::OpenPokemon(p.id.to_native()))
            .into()
        })
        .collect();
    if !holders.is_empty() {
        sections.push(card(
            fl!("item-held-by"),
            widget::column::with_children(holders).spacing(2),
        ));
    }

    widget::column::with_children(sections).spacing(12).into()
}
