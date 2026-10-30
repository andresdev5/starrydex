// SPDX-License-Identifier: GPL-3.0-only

//! Searchable list of every item; the selected one is shown in the context drawer.

use cosmic::iced::core::text::LineHeight;
use cosmic::iced::{Alignment, Background, Border, Length, Pixels};
use cosmic::widget::{self, button, container, text};
use cosmic::{Element, theme};
use starrydex_model::{ArchivedItem, SpriteKind};

use crate::app::core::StarryCore;
use crate::app::pages::common::{card, hex, info_row, sprite};
use crate::app::utils::PaginationAction;
use crate::app::utils::presentation::data_lang;
use crate::app::{AppModel, ItemsInput, Message, Ready, page_count};
use crate::{fl, icons};

/// Narrower than this, the effect column is left out (it is still in the item drawer).
const EFFECT_MIN_WIDTH: f32 = 760.0;
const ICON_WIDTH: f32 = 36.0;
const POCKET_WIDTH: f32 = 150.0;
const PRICE_WIDTH: f32 = 100.0;
/// Tag colors of the bag pockets, in pocket order.
const POCKET_COLORS: [u32; 8] = [
    0xDEDBCD, 0xC4E6B5, 0xF3C1B6, 0xCFE0F2, 0xEBD9A6, 0xF8D0E8, 0xDCCDEB, 0xC1EDEA,
];
pub const ITEMS_SCROLL_ID: &str = "item-list";
pub const SEARCH_ID: &str = "item-search";

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

    let search = widget::search_input(fl!("items-search-placeholder"), &list.search)
        .id(widget::Id::new(SEARCH_ID))
        .on_input(|v| Message::Items(ItemsInput::Search(v)))
        .on_clear(Message::Items(ItemsInput::Search(String::new())))
        .line_height(LineHeight::Absolute(Pixels(30.0)))
        .width(Length::Fill);

    let active_pocket = list.pocket_ids.iter().position(|id| *id == list.pocket);
    let pockets = widget::flex_row(
        list.pocket_labels
            .iter()
            .enumerate()
            .map(|(i, label)| {
                button::text(label.as_str())
                    .class(if active_pocket == Some(i) {
                        theme::Button::Suggested
                    } else {
                        theme::Button::Text
                    })
                    .on_press(Message::Items(ItemsInput::Pocket(i)))
                    .into()
            })
            .collect(),
    )
    .row_spacing(4)
    .column_spacing(4)
    .width(Length::Fill);

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

    let visible = &list.results[start..end];
    let table = widget::responsive(move |size| {
        let show_effect = size.width >= EFFECT_MIN_WIDTH;
        if visible.is_empty() {
            return widget::column::with_children(vec![
                text::title3(fl!("items-no-results")).into(),
                text::body(fl!("items-no-results-hint")).into(),
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
            .map(|&index| item_row(ready, &dex.items[index as usize], show_effect))
            .collect::<Vec<_>>();
        widget::column::with_children(vec![
            header_row(show_effect),
            widget::scrollable(widget::column::with_children(rows).spacing(2).padding([
                0,
                spacing.space_m,
                0,
                0,
            ]))
            .id(ITEMS_SCROLL_ID)
            .height(Length::Fill)
            .width(Length::Fill)
            .into(),
        ])
        .spacing(spacing.space_xxs)
        .into()
    });

    widget::column::with_children(vec![
        search.into(),
        pockets.into(),
        table.into(),
        footer.into(),
    ])
    .spacing(spacing.space_xs)
    .width(Length::Fill)
    .into()
}

/// Column titles of the item table.
fn header_row<'a>(show_effect: bool) -> Element<'a, Message> {
    let title = |label: String| text::caption_heading(label);
    let mut cells: Vec<Element<'a, Message>> = vec![
        widget::Space::new().width(Length::Fixed(ICON_WIDTH)).into(),
        title(fl!("items-name"))
            .width(Length::FillPortion(3))
            .into(),
        title(fl!("item-pocket"))
            .width(Length::Fixed(POCKET_WIDTH))
            .into(),
    ];
    if show_effect {
        cells.push(
            title(fl!("item-effect"))
                .width(Length::FillPortion(6))
                .into(),
        );
    }
    cells.push(
        title(fl!("item-cost"))
            .width(Length::Fixed(PRICE_WIDTH))
            .align_x(Alignment::End)
            .into(),
    );
    container(
        widget::row::with_children(cells)
            .spacing(12)
            .align_y(Alignment::Center),
    )
    .padding([6, 8 + theme::active().cosmic().spacing.space_m, 6, 8])
    .width(Length::Fill)
    .class(theme::Container::Card)
    .into()
}

fn item_row<'a>(
    ready: &'a Ready,
    item: &'a ArchivedItem,
    show_effect: bool,
) -> Element<'a, Message> {
    let lang = data_lang();
    let dex = ready.core.dex();
    let pocket_id = item.pocket_id.to_native();
    let pocket = dex
        .item_pocket(pocket_id)
        .map(|p| p.display.get(lang))
        .unwrap_or_default();
    let color = dex
        .item_pockets
        .iter()
        .position(|p| p.id.to_native() == pocket_id)
        .map_or(POCKET_COLORS[0], |i| POCKET_COLORS[i % POCKET_COLORS.len()]);
    let price = match item.cost.as_ref().map(|c| c.to_native()) {
        Some(cost) if cost > 0 => fl!("item-cost-value", cost = cost),
        _ => "—".to_string(),
    };

    let mut cells: Vec<Element<'a, Message>> = vec![
        container(sprite(ready.core.item_sprite(item), 30.0))
            .width(Length::Fixed(ICON_WIDTH))
            .align_x(Alignment::Center)
            .into(),
        text::heading(item.display.get(lang))
            .width(Length::FillPortion(3))
            .into(),
        container(pocket_tag(pocket, color))
            .width(Length::Fixed(POCKET_WIDTH))
            .into(),
    ];
    if show_effect {
        // Effects are only in English: prefer the description when it is in the UI language.
        let effect = if item.short_effect.has(lang) || !item.flavor.has(lang) {
            item.short_effect.get(lang)
        } else {
            item.flavor.get(lang)
        };
        let effect = if effect.is_empty() {
            item.flavor.get(lang)
        } else {
            effect
        };
        cells.push(
            text::body(effect.replace('\n', " "))
                .width(Length::FillPortion(6))
                .into(),
        );
    }
    cells.push(
        text::body(price)
            .width(Length::Fixed(PRICE_WIDTH))
            .align_x(Alignment::End)
            .into(),
    );

    button::custom(
        widget::row::with_children(cells)
            .spacing(12)
            .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding([6, 8])
    .selected(ready.items.selected == Some(item.id.to_native()))
    .on_press(Message::Items(ItemsInput::Open(item.id.to_native())))
    .class(theme::Button::IconVertical)
    .into()
}

fn pocket_tag<'a>(label: &'a str, color: u32) -> Element<'a, Message> {
    container(text::caption(label))
        .padding([2, 8])
        .class(theme::Container::custom(move |_| container::Style {
            background: Some(Background::Color(hex(color))),
            text_color: Some(hex(0x141A26)),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }))
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
