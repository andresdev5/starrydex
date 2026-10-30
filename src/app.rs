// SPDX-License-Identifier: GPL-3.0

use std::collections::HashMap;

use cosmic::Application;
use cosmic::app::context_drawer;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::core::keyboard::{Key, Modifiers};
use cosmic::iced::keyboard::key::Named;
use cosmic::iced::widget::operation::AbsoluteOffset;
use cosmic::iced::widget::{center, scrollable};
use cosmic::iced::{Event, Length, Subscription};
use cosmic::prelude::*;
use cosmic::widget::menu::Action;
use cosmic::widget::{self, about::About, menu, nav_bar, segmented_button, text};
use starrydex_model::{DamageClass, Generation, PokemonType};

use crate::app::app_menu::MenuAction;
use crate::app::context_page::ContextPage;
use crate::app::core::{ListQuery, StarryCore};
use crate::app::pages::details::{DetailTab, Details};
use crate::app::pages::home::Section;
use crate::app::pages::items::ItemList;
use crate::app::pages::move_list::MoveList;
use crate::app::pages::save::SaveState;
use crate::app::utils::presentation::{Label, data_lang};
use crate::app::utils::{
    Category, Filters, MoveSort, PaginationAction, SortBy, gen4_save, play_audio_file,
    remove_dir_contents,
};
use crate::config::{AppTheme, ConfigInput, StarryConfig, TEAM_SIZE, TypeFilteringMode};
use crate::key_binds::key_binds;
use crate::{fl, icons};

pub mod app_menu;
mod context_page;
mod core;
mod pages;
mod utils;
mod widgets;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// The about page for this app.
    about: About,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    /// Application Keyboard Modifiers
    modifiers: Modifiers,
    /// Application configuration handler
    config_handler: Option<cosmic::cosmic_config::Config>,
    /// Configuration data that persists between application runs.
    config: StarryConfig,
    /// Application Themes
    app_themes: Vec<String>,
    /// Available Type Filter Modes
    type_filter_modes: Vec<String>,
    /// Labels of the sort options
    sort_options: Vec<String>,
    /// Tabs of the Pokémon details page
    detail_tabs: segmented_button::SingleSelectModel,
    /// Sections of the app, shown in the navigation bar
    nav: nav_bar::Model,
    /// Application State
    state: State,
}

pub enum State {
    Loading,
    Error(String),
    Ready(Box<Ready>),
}

/// Everything that exists once the Pokédex is loaded.
pub struct Ready {
    /// Memory-mapped Pokédex
    pub core: StarryCore,
    /// Indices (into the Pokédex) of the Pokémon matching the current search/filters/sort
    pub results: Vec<u32>,
    /// Current page of the list (0-based)
    pub page: usize,
    /// Search input value
    pub search: String,
    /// Currently applied filters
    pub filters: Filters,
    /// Curenntly applied sorting
    pub sort: SortBy,
    /// Is sort descending?
    pub descending: bool,
    /// The open Pokémon, if any
    pub details: Option<Details>,
    /// Scroll position of the Pokémon list, restored when coming back from a details page
    pub list_scroll: AbsoluteOffset,
    /// Visible height of the Pokémon list, to know whether the selected row is in view
    pub list_viewport: f32,
    /// Type whose matchups are shown on the type chart page
    pub selected_type: Option<PokemonType>,
    /// Chart cell under the mouse: (attacking type, defending type)
    pub type_hover: Option<(PokemonType, PokemonType)>,
    /// Types picked in the defensive calculator (at most two)
    pub defense_types: Vec<PokemonType>,
    /// Only list the favorite Pokémon
    pub favorites_only: bool,
    /// Ids of the two Pokémon on the comparator
    pub compare: [Option<u32>; 2],
    /// Search boxes of the two comparator sides
    pub compare_search: [String; 2],
    /// Game save read from the remembered folder
    pub save: SaveState,
    /// Items section
    pub items: ItemList,
    /// Moves section
    pub moves: MoveList,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    /// StarryDex HotKey Callback
    Hotkey(Hotkey),
    /// Opens the given URL in the browser
    LaunchUrl(String),
    /// Opens (or closes if already open) the given [`ContextPage`]
    ToggleContextPage(ContextPage),
    /// Update the application config
    UpdateConfig(StarryConfig),
    /// Callback after clicking something in the app menu
    MenuAction(app_menu::MenuAction),
    /// Executes the appropiate cosmic binding on keyboard shortcut
    Key(Modifiers, Key),
    /// Updates the current state of keyboard modifiers
    Modifiers(Modifiers),
    /// Callback after loading the application core
    CoreLoaded(Result<StarryCore, anywho::Error>),
    /// Follow a link to the Pokémon with the given id (remembered for the back button)
    OpenPokemon(u32),
    /// Show the Pokémon with the given id, picked from the list
    SelectPokemon(u32),
    /// Mark or unmark the Pokémon with the given id as favorite
    ToggleFavorite(u32),
    /// Add the Pokémon with the given id to the team, or remove it
    ToggleTeam(u32),
    /// Open the comparator with the Pokémon with the given id on the left
    CompareWith(u32),
    /// Switch to a section of the navigation bar
    ShowSection(Section),
    /// Input on the comparator
    Compare(CompareInput),
    /// Leave the details page (or go back to the previously viewed Pokémon)
    Back,
    /// A tab of the details page was selected
    DetailTab(segmented_button::Entity),
    /// A type was picked on the type chart page
    SelectType(PokemonType),
    /// Callback after input on the Config [`ContextPage`]
    ConfigInput(ConfigInput),
    /// Input on the details page
    Details(DetailsInput),
    /// Input on the Pokémon list
    List(ListInput),
    /// Input on the Filters [`ContextPage`]
    Filters(FiltersInput),
    /// Input on the items section
    Items(ItemsInput),
    /// Input on the type chart page
    Types(TypesInput),
    /// Input on the moves section
    Moves(MovesInput),
    /// Input on the game save section
    Save(SaveInput),
}

/// Some user interaction that happens on the game save section
#[derive(Debug, Clone)]
pub enum SaveInput {
    /// Ask for the game folder
    PickFolder,
    /// The folder dialog closed (`None` if cancelled)
    Folder(Option<std::path::PathBuf>),
    /// Read the save again from the remembered folder
    Reload,
    /// Show the Pokémon in this party or box slot
    Select(pages::save::Slot),
    /// Replace the team with the party of the save
    UseAsTeam,
}

/// Some user interaction that happens on the moves section
#[derive(Debug, Clone)]
pub enum MovesInput {
    Search(String),
    ToggleType(PokemonType),
    ClearTypes,
    Class(Option<DamageClass>),
    /// Index of the picked entry of the generation dropdown (0 = all)
    Generation(usize),
    /// Sort by this column, or reverse the order if it already is
    Sort(MoveSort),
    Page(PaginationAction),
    /// Show the move with the given id in the context drawer
    Open(u32),
}

/// Some user interaction that happens on the comparator (`side` is 0 or 1)
#[derive(Debug, Clone)]
pub enum CompareInput {
    Search(usize, String),
    Pick(usize, u32),
    Swap,
}

/// Some user interaction that happens on the type chart page
#[derive(Debug, Clone)]
pub enum TypesInput {
    /// The mouse entered a chart cell, or left the chart
    Hover(Option<(PokemonType, PokemonType)>),
    /// Add or remove a type in the defensive calculator
    ToggleDefender(PokemonType),
    ClearDefenders,
}

/// Some user interaction that happens on the items section
#[derive(Debug, Clone)]
pub enum ItemsInput {
    Search(String),
    /// Index of the picked entry of the pocket dropdown
    Pocket(usize),
    Page(PaginationAction),
    /// Show the item with the given id in the context drawer
    Open(u32),
}

/// Some user interaction that happens on the Pokémon list
#[derive(Debug, Clone)]
pub enum ListInput {
    Page(PaginationAction),
    Search(String),
    Sort(usize),
    ToggleOrder,
    ToggleFavoritesOnly,
    ClearAll,
    /// The list was scrolled (remembers where the user was and how much of it is visible)
    Scrolled(AbsoluteOffset, f32),
    /// Scroll the list back to the remembered position
    RestoreScroll(AbsoluteOffset),
}

/// Some user interaction that happens on the details page
#[derive(Debug, Clone)]
pub enum DetailsInput {
    Step(PaginationAction),
    PlayAudio(String),
    ToggleShiny,
    ToggleBack,
    ToggleFemale,
    FlavorEntry(usize),
    MoveVersionGroup(usize),
    ToggleMove(u32),
    EncounterVersion(usize),
}

/// Some user interaction that happens on the Filters [`ContextPage`]
#[derive(Debug, Clone)]
pub enum FiltersInput {
    Type(bool, PokemonType),
    Generation(bool, Generation),
    Category(bool, Category),
    StatsEnabled(bool),
    MinTotal(u16),
    ClearTypes,
    Reset,
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = crate::flags::Flags;
    type Message = Message;
    const APP_ID: &'static str = "dev.mariinkys.StarryDex";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(core: cosmic::Core, flags: Self::Flags) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let about = About::default()
            .name("StarryDex")
            .icon(widget::icon::from_name(Self::APP_ID))
            .version(env!("CARGO_PKG_VERSION"))
            .links([
                (fl!("repository"), REPOSITORY),
                (
                    fl!("support"),
                    "https://github.com/mariinkys/starrydex/issues",
                ),
            ])
            .license(env!("CARGO_PKG_LICENSE"))
            .developers([("mariinkys", "kysdev.owjga@aleeas.com")])
            .comments(format!(
                "{}\n{}\n{}",
                fl!("app-info"),
                fl!("pokeapi-text"),
                fl!("nintendo-text")
            ));

        let mut detail_tabs = segmented_button::SingleSelectModel::default();
        for tab in DetailTab::ALL {
            detail_tabs.insert().text(tab.label()).data(*tab);
        }
        detail_tabs.activate_position(0);

        let mut nav = nav_bar::Model::default();
        for section in Section::ALL {
            nav.insert()
                .text(section.label())
                .icon(widget::icon::icon(icons::get_handle(
                    section.icon_name(),
                    16,
                )))
                .data(*section);
        }
        nav.activate_position(0);

        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            about,
            key_binds: key_binds(),
            modifiers: Modifiers::empty(),
            config_handler: flags.config_handler,
            config: flags.config,
            app_themes: vec![
                fl!("match-desktop"),
                fl!("dark"),
                fl!("light"),
                fl!("modern-dex"),
            ],
            type_filter_modes: vec![fl!("exclusive"), fl!("inclusive")],
            sort_options: SortBy::ALL.iter().map(Label::label).collect(),
            detail_tabs,
            nav,
            state: State::Loading,
        };

        app.refresh_nav_labels();

        let tasks = vec![
            app.update_title(),
            cosmic::command::set_theme(app.config.app_theme.theme()),
            Task::perform(StarryCore::initialize(), |res| {
                cosmic::action::app(Message::CoreLoaded(res))
            }),
        ];

        (app, Task::batch(tasks))
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")).apply(Element::from),
            menu::items(
                &self.key_binds,
                vec![
                    menu::Item::Button(fl!("about"), None, MenuAction::About),
                    menu::Item::Button(fl!("settings"), None, MenuAction::Settings),
                ],
            ),
        )]);

        vec![menu_bar.into()]
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        self.nav.activate(id);
        self.restore_list_scroll()
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }
        self.context_page.display(self)
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let content: Element<_> = match &self.state {
            State::Loading => center(
                widget::column::with_children(vec![
                    widget::progress_bar::indeterminate_circular().into(),
                    text::body(fl!("loading")).into(),
                ])
                .spacing(12)
                .align_x(cosmic::iced::Alignment::Center),
            )
            .into(),
            State::Error(error) => center(text::body(error)).into(),
            State::Ready(ready) => pages::home::view(
                self,
                ready,
                &self.detail_tabs,
                self.active_tab(),
                self.active_section(),
            ),
        };

        widget::container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            cosmic::iced::event::listen_with(|event, status, _| match event {
                Event::Keyboard(cosmic::iced::keyboard::Event::KeyPressed {
                    key,
                    modifiers,
                    ..
                }) => {
                    // Keys typed into a text input (the search box) are not shortcuts.
                    if status == cosmic::iced::event::Status::Captured {
                        return None;
                    }
                    match &key {
                        Key::Named(Named::ArrowLeft) if modifiers.is_empty() => {
                            Some(Message::Hotkey(Hotkey::ArrowLeft))
                        }
                        Key::Named(Named::ArrowRight) if modifiers.is_empty() => {
                            Some(Message::Hotkey(Hotkey::ArrowRight))
                        }
                        Key::Named(Named::ArrowUp) if modifiers.is_empty() => {
                            Some(Message::Hotkey(Hotkey::ArrowUp))
                        }
                        Key::Named(Named::ArrowDown) if modifiers.is_empty() => {
                            Some(Message::Hotkey(Hotkey::ArrowDown))
                        }
                        Key::Character(c)
                            if modifiers == Modifiers::CTRL
                                && matches!(
                                    c.as_str(),
                                    "1" | "2" | "3" | "4" | "5" | "6" | "7"
                                ) =>
                        {
                            let index = c.as_str().parse::<usize>().unwrap_or(1) - 1;
                            Some(Message::Hotkey(Hotkey::Section(index)))
                        }
                        // Shift is allowed: some layouts need it to type "/".
                        Key::Character(c)
                            if c.as_str() == "/"
                                && !modifiers.control()
                                && !modifiers.alt()
                                && !modifiers.logo() =>
                        {
                            Some(Message::Hotkey(Hotkey::Search))
                        }
                        Key::Named(Named::Escape) => Some(Message::Hotkey(Hotkey::Escape)),
                        _ => Some(Message::Key(modifiers, key)),
                    }
                }
                Event::Keyboard(cosmic::iced::keyboard::Event::ModifiersChanged(modifiers)) => {
                    Some(Message::Modifiers(modifiers))
                }
                _ => None,
            }),
            self.core()
                .watch_config::<StarryConfig>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
        ])
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Hotkey(hotkey) => self.on_hotkey(hotkey),
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
                Task::none()
            }
            Message::UpdateConfig(config) => {
                let theme_changed = config.app_theme != self.config.app_theme;
                let favorites_only =
                    matches!(&self.state, State::Ready(ready) if ready.favorites_only);
                // Saving favorites or the team comes back here too: only re-run the query (which
                // also sends the list back to its first page) when the list really changes.
                let list_changed = config.pokemon_per_page != self.config.pokemon_per_page
                    || config.type_filtering_mode != self.config.type_filtering_mode
                    || config.show_forms != self.config.show_forms
                    || (favorites_only && config.favorites != self.config.favorites);
                self.config = config;
                if list_changed {
                    self.refresh_results();
                }
                self.refresh_nav_labels();
                if theme_changed {
                    cosmic::command::set_theme(self.config.app_theme.theme())
                } else {
                    Task::none()
                }
            }
            Message::LaunchUrl(url) => {
                if let Err(err) = open::that_detached(&url) {
                    eprintln!("failed to open {url:?}: {err}");
                }
                Task::none()
            }
            Message::MenuAction(action) => match action {
                MenuAction::About => self.update(Message::ToggleContextPage(ContextPage::About)),
                MenuAction::Settings => {
                    self.update(Message::ToggleContextPage(ContextPage::Settings))
                }
            },
            Message::Key(modifiers, key) => {
                for (key_bind, action) in &self.key_binds {
                    if key_bind.matches(modifiers, &key, None) {
                        return self.update(action.message());
                    }
                }
                Task::none()
            }
            Message::Modifiers(modifiers) => {
                self.modifiers = modifiers;
                Task::none()
            }
            Message::CoreLoaded(Err(e)) => {
                self.state = State::Error(e.to_string());
                Task::none()
            }
            Message::CoreLoaded(Ok(core)) => {
                let items = ItemList::new(&core);
                let moves = MoveList::new(&core);
                self.state = State::Ready(Box::new(Ready {
                    core,
                    results: Vec::new(),
                    page: 0,
                    search: String::new(),
                    filters: Filters::default(),
                    sort: SortBy::default(),
                    descending: false,
                    moves,
                    details: None,
                    list_scroll: AbsoluteOffset::default(),
                    list_viewport: 0.0,
                    selected_type: None,
                    type_hover: None,
                    defense_types: Vec::new(),
                    favorites_only: false,
                    compare: [None, None],
                    compare_search: Default::default(),
                    save: SaveState::default(),
                    items,
                }));
                self.refresh_results();
                self.load_save();
                Task::none()
            }
            Message::OpenPokemon(id) => {
                let State::Ready(ready) = &mut self.state else {
                    return Task::none();
                };
                let Some(index) = ready.core.dex().pokemon_index(id) else {
                    return Task::none();
                };
                match &mut ready.details {
                    Some(details) if details.index != index => {
                        details.history.push(details.index);
                        details.show(index, ready.core.dex());
                    }
                    Some(_) => {}
                    None => ready.details = Some(Details::new(index, ready.core.dex())),
                }
                if matches!(
                    self.context_page,
                    ContextPage::ItemDetails | ContextPage::MoveDetails
                ) {
                    self.core.window.show_context = false;
                }
                self.show_section(Section::Pokemon)
            }
            Message::SelectPokemon(id) => {
                let State::Ready(ready) = &mut self.state else {
                    return Task::none();
                };
                let Some(index) = ready.core.dex().pokemon_index(id) else {
                    return Task::none();
                };
                match &mut ready.details {
                    Some(details) => {
                        details.history.clear();
                        if details.index != index {
                            details.show(index, ready.core.dex());
                        }
                    }
                    None => ready.details = Some(Details::new(index, ready.core.dex())),
                }
                self.show_section(Section::Pokemon)
            }
            Message::ToggleFavorite(id) => {
                let mut favorites = self.config.favorites.clone();
                if let Some(position) = favorites.iter().position(|f| *f == id) {
                    favorites.remove(position);
                } else {
                    favorites.push(id);
                }
                self.save_config(
                    |c, h| c.set_favorites(h, favorites.clone()),
                    |c| c.favorites = favorites.clone(),
                );
                if matches!(&self.state, State::Ready(ready) if ready.favorites_only) {
                    self.refresh_results();
                }
                Task::none()
            }
            Message::ToggleTeam(id) => {
                let mut team = self.config.team.clone();
                if let Some(position) = team.iter().position(|m| *m == id) {
                    team.remove(position);
                } else if team.len() < TEAM_SIZE {
                    team.push(id);
                } else {
                    return Task::none();
                }
                self.save_config(
                    |c, h| c.set_team(h, team.clone()),
                    |c| c.team = team.clone(),
                );
                self.refresh_nav_labels();
                Task::none()
            }
            Message::CompareWith(id) => {
                if let State::Ready(ready) = &mut self.state {
                    let [left, right] = ready.compare;
                    let other = if left == Some(id) { right } else { left };
                    ready.compare = [Some(id), other];
                }
                self.show_section(Section::Compare)
            }
            Message::ShowSection(section) => self.show_section(section),
            Message::Compare(input) => {
                if let State::Ready(ready) = &mut self.state {
                    match input {
                        CompareInput::Search(side, value) => ready.compare_search[side] = value,
                        CompareInput::Pick(side, id) => {
                            ready.compare[side] = Some(id);
                            ready.compare_search[side].clear();
                        }
                        CompareInput::Swap => {
                            ready.compare.swap(0, 1);
                            ready.compare_search.swap(0, 1);
                        }
                    }
                }
                Task::none()
            }
            Message::Back => {
                if let State::Ready(ready) = &mut self.state
                    && let Some(details) = &mut ready.details
                {
                    match details.history.pop() {
                        Some(previous) => details.show(previous, ready.core.dex()),
                        None => {
                            ready.details = None;
                            // Restore on the next update, once the list view exists again. The offset is
                            // captured now because the new scrollable reports 0 through `on_scroll` when it
                            // first draws, which would overwrite `list_scroll`.
                            return Task::done(cosmic::action::app(Message::List(
                                ListInput::RestoreScroll(ready.list_scroll),
                            )));
                        }
                    }
                }
                Task::none()
            }
            Message::DetailTab(entity) => {
                self.detail_tabs.activate(entity);
                Task::none()
            }
            Message::SelectType(t) => {
                if let State::Ready(ready) = &mut self.state {
                    ready.selected_type = Some(t);
                }
                Task::none()
            }
            Message::Types(input) => {
                if let State::Ready(ready) = &mut self.state {
                    match input {
                        TypesInput::Hover(hover) => ready.type_hover = hover,
                        TypesInput::ToggleDefender(t) => {
                            let picked = &mut ready.defense_types;
                            if let Some(position) = picked.iter().position(|p| *p == t) {
                                picked.remove(position);
                            } else {
                                // Only two types: the oldest pick makes room for the new one.
                                if picked.len() == 2 {
                                    picked.remove(0);
                                }
                                picked.push(t);
                            }
                        }
                        TypesInput::ClearDefenders => ready.defense_types.clear(),
                    }
                }
                Task::none()
            }
            Message::ConfigInput(input) => self.on_config_input(input),
            Message::Details(input) => {
                let State::Ready(ready) = &mut self.state else {
                    return Task::none();
                };
                let Some(details) = &mut ready.details else {
                    return Task::none();
                };
                let dex = ready.core.dex();
                match input {
                    DetailsInput::Step(direction) => {
                        let next =
                            step(&ready.results, dex.pokemon.len(), details.index, &direction);
                        details.history.clear();
                        details.show(next, dex);
                        return self.reveal_selected();
                    }
                    DetailsInput::PlayAudio(path) => {
                        play_audio_file(&path, f32::from(self.config.cry_volume) / 100.0);
                    }
                    DetailsInput::ToggleShiny => details.shiny = !details.shiny,
                    DetailsInput::ToggleBack => details.back = !details.back,
                    DetailsInput::ToggleFemale => details.female = !details.female,
                    DetailsInput::FlavorEntry(index) => details.flavor_entry = index,
                    DetailsInput::MoveVersionGroup(index) => {
                        details.move_version_group =
                            details.move_version_groups.get(index).copied();
                        details.expanded_moves.clear();
                    }
                    DetailsInput::ToggleMove(id) => {
                        if !details.expanded_moves.remove(&id) {
                            details.expanded_moves.insert(id);
                        }
                    }
                    DetailsInput::EncounterVersion(index) => {
                        // Index 0 is "all games".
                        details.encounter_version = index
                            .checked_sub(1)
                            .and_then(|i| details.encounter_versions.get(i).copied());
                    }
                }
                Task::none()
            }
            Message::List(input) => {
                let State::Ready(ready) = &mut self.state else {
                    return Task::none();
                };
                match input {
                    ListInput::Page(action) => {
                        let pages = page_count(ready.results.len(), self.config.pokemon_per_page);
                        ready.page = match action {
                            PaginationAction::Next => (ready.page + 1).min(pages - 1),
                            PaginationAction::Back => ready.page.saturating_sub(1),
                        };

                        // go back to the top of the page when changing paage
                        ready.list_scroll = AbsoluteOffset::default();
                        return scrollable::scroll_to(
                            pages::home::LIST_SCROLL_ID.into(),
                            AbsoluteOffset {
                                x: None,
                                y: Some(0.0),
                            },
                        );
                    }
                    ListInput::Search(value) => ready.search = value,
                    ListInput::Sort(index) => {
                        ready.sort = SortBy::ALL.get(index).copied().unwrap_or_default();
                        ready.descending = false;
                    }
                    ListInput::ToggleOrder => ready.descending = !ready.descending,
                    ListInput::ToggleFavoritesOnly => ready.favorites_only = !ready.favorites_only,
                    ListInput::ClearAll => {
                        ready.search.clear();
                        ready.favorites_only = false;
                        ready.filters = Filters::default();
                        ready.sort = SortBy::default();
                        ready.descending = false;
                    }
                    ListInput::Scrolled(offset, height) => {
                        ready.list_scroll = offset;
                        ready.list_viewport = height;
                        return Task::none();
                    }
                    ListInput::RestoreScroll(offset) => {
                        ready.list_scroll = offset;
                        return scrollable::scroll_to(
                            pages::home::LIST_SCROLL_ID.into(),
                            offset.into(),
                        );
                    }
                }
                self.refresh_results();
                Task::none()
            }
            Message::Items(input) => {
                let State::Ready(ready) = &mut self.state else {
                    return Task::none();
                };
                let list = &mut ready.items;
                match input {
                    ItemsInput::Search(value) => {
                        list.search = value;
                        list.refresh(&ready.core);
                    }
                    ItemsInput::Pocket(index) => {
                        list.pocket = list.pocket_ids.get(index).copied().flatten();
                        list.refresh(&ready.core);
                    }
                    ItemsInput::Page(action) => {
                        let pages = page_count(list.results.len(), self.config.pokemon_per_page);
                        list.page = match action {
                            PaginationAction::Next => (list.page + 1).min(pages - 1),
                            PaginationAction::Back => list.page.saturating_sub(1),
                        };
                        return scrollable::scroll_to(
                            pages::items::ITEMS_SCROLL_ID.into(),
                            AbsoluteOffset {
                                x: None,
                                y: Some(0.0),
                            },
                        );
                    }
                    ItemsInput::Open(id) => {
                        list.selected = Some(id);
                        self.context_page = ContextPage::ItemDetails;
                        self.core.window.show_context = true;
                    }
                }
                Task::none()
            }
            Message::Moves(input) => {
                let State::Ready(ready) = &mut self.state else {
                    return Task::none();
                };
                let list = &mut ready.moves;
                match input {
                    MovesInput::Search(value) => list.search = value,
                    MovesInput::ToggleType(t) => {
                        if !list.types.remove(&t) {
                            list.types.insert(t);
                        }
                    }
                    MovesInput::ClearTypes => list.types.clear(),
                    MovesInput::Class(class) => list.class = class,
                    MovesInput::Generation(index) => {
                        list.generation = index
                            .checked_sub(1)
                            .and_then(|i| pages::move_list::generations().nth(i));
                    }
                    MovesInput::Sort(sort) => {
                        if list.sort == sort {
                            list.descending = !list.descending;
                        } else {
                            list.sort = sort;
                            list.descending = false;
                        }
                    }
                    MovesInput::Page(action) => {
                        let pages = page_count(list.results.len(), self.config.pokemon_per_page);
                        list.page = match action {
                            PaginationAction::Next => (list.page + 1).min(pages - 1),
                            PaginationAction::Back => list.page.saturating_sub(1),
                        };
                        return scrollable::scroll_to(
                            pages::move_list::MOVES_SCROLL_ID.into(),
                            AbsoluteOffset {
                                x: None,
                                y: Some(0.0),
                            },
                        );
                    }
                    MovesInput::Open(id) => {
                        list.selected = Some(id);
                        list.learners = ready.core.learners(id, self.config.show_forms);
                        self.context_page = ContextPage::MoveDetails;
                        self.core.window.show_context = true;
                        return Task::none();
                    }
                }
                list.refresh(&ready.core);
                Task::none()
            }
            Message::Save(input) => self.on_save_input(input),
            Message::Filters(input) => {
                let State::Ready(ready) = &mut self.state else {
                    return Task::none();
                };
                let filters = &mut ready.filters;
                match input {
                    FiltersInput::Type(on, t) => toggle(&mut filters.types, t, on),
                    FiltersInput::Generation(on, g) => toggle(&mut filters.generations, g, on),
                    FiltersInput::Category(on, c) => toggle(&mut filters.categories, c, on),
                    FiltersInput::StatsEnabled(on) => filters.stats_enabled = on,
                    FiltersInput::MinTotal(value) => filters.min_total = value,
                    FiltersInput::ClearTypes => filters.types.clear(),
                    FiltersInput::Reset => *filters = Filters::default(),
                }
                self.refresh_results();
                Task::none()
            }
        }
    }
}

impl AppModel {
    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(String::from("StarryDex"), id)
        } else {
            Task::none()
        }
    }

    fn active_tab(&self) -> DetailTab {
        self.detail_tabs
            .active_data::<DetailTab>()
            .copied()
            .unwrap_or_default()
    }

    fn active_section(&self) -> Section {
        self.nav
            .active_data::<Section>()
            .copied()
            .unwrap_or_default()
    }

    fn on_save_input(&mut self, input: SaveInput) -> Task<cosmic::Action<Message>> {
        match input {
            SaveInput::PickFolder => {
                return Task::perform(
                    cosmic::dialog::file_chooser::open::Dialog::new()
                        .title(fl!("save-dialog"))
                        .open_folder(),
                    |result| {
                        let folder = result.ok().and_then(|r| r.url().to_file_path().ok());
                        cosmic::action::app(Message::Save(SaveInput::Folder(folder)))
                    },
                );
            }
            SaveInput::Folder(None) => {}
            SaveInput::Folder(Some(folder)) => {
                let folder = folder.to_string_lossy().into_owned();
                self.save_config(
                    |c, h| c.set_save_folder(h, folder.clone()),
                    |c| c.save_folder = folder.clone(),
                );
                self.load_save();
            }
            SaveInput::Reload => self.load_save(),
            SaveInput::Select(slot) => {
                if let State::Ready(ready) = &mut self.state {
                    ready.save.selected = slot;
                }
            }
            SaveInput::UseAsTeam => {
                if let State::Ready(ready) = &self.state {
                    let dex = ready.core.dex();
                    let team: Vec<u32> = ready
                        .save
                        .data
                        .party
                        .iter()
                        .map(|m| u32::from(m.species))
                        .filter(|id| dex.pokemon(*id).is_some())
                        .take(TEAM_SIZE)
                        .collect();
                    self.save_config(
                        |c, h| c.set_team(h, team.clone()),
                        |c| c.team = team.clone(),
                    );
                    self.refresh_nav_labels();
                }
            }
        }
        Task::none()
    }

    /// Reads the save in the remembered game folder, or the one found in the usual places when
    /// no folder was picked yet.
    fn load_save(&mut self) {
        let State::Ready(ready) = &mut self.state else {
            return;
        };
        let mut save = SaveState {
            selected: ready.save.selected,
            ..SaveState::default()
        };
        let path = if self.config.save_folder.is_empty() {
            // Nothing found: keep the empty state that explains how to pick the folder.
            let Some(path) = gen4_save::detect_save() else {
                return;
            };
            save.detected = true;
            path
        } else {
            let folder = std::path::Path::new(&self.config.save_folder);
            let Some(path) = gen4_save::find_save(folder) else {
                save.error = Some(fl!(
                    "save-not-found",
                    folder = (folder.display().to_string())
                ));
                ready.save = save;
                return;
            };
            path
        };
        let file = path.display().to_string();
        match std::fs::read(&path) {
            Err(e) => {
                save.error = Some(fl!("save-read-error", file = file, error = (e.to_string())));
            }
            Ok(bytes) => match gen4_save::read_save(&bytes) {
                Ok(data) => save.data = data,
                Err(gen4_save::SaveError::NotASave) => {
                    save.error = Some(fl!("save-not-a-save", file = file));
                }
            },
        }
        save.path = Some(path);
        ready.save = save;
    }

    /// Shows how full the team is next to its section name.
    fn refresh_nav_labels(&mut self) {
        let count = self.config.team.len();
        let entity = self
            .nav
            .iter()
            .find(|&id| self.nav.data::<Section>(id) == Some(&Section::Team));
        if let Some(entity) = entity {
            self.nav
                .text_set(entity, fl!("home-team-count", count = count));
        }
    }

    /// Switches to `section` in the navigation bar.
    fn show_section(&mut self, section: Section) -> Task<cosmic::Action<Message>> {
        if self.active_section() == section {
            return Task::none();
        }
        let entity = self
            .nav
            .iter()
            .find(|&id| self.nav.data::<Section>(id) == Some(&section));
        if let Some(entity) = entity {
            self.nav.activate(entity);
        }
        self.restore_list_scroll()
    }

    /// The Pokémon list is rebuilt from scratch when its section is shown again, so bring back
    /// where the user was.
    fn restore_list_scroll(&self) -> Task<cosmic::Action<Message>> {
        match &self.state {
            State::Ready(ready) if self.active_section() == Section::Pokemon => Task::done(
                cosmic::action::app(Message::List(ListInput::RestoreScroll(ready.list_scroll))),
            ),
            _ => Task::none(),
        }
    }

    /// Moves the list to the page of the open Pokémon and scrolls its row into view.
    fn reveal_selected(&mut self) -> Task<cosmic::Action<Message>> {
        let State::Ready(ready) = &mut self.state else {
            return Task::none();
        };
        let Some(details) = &ready.details else {
            return Task::none();
        };
        let Some(position) = ready
            .results
            .iter()
            .position(|&i| i as usize == details.index)
        else {
            return Task::none();
        };
        let per_page = self.config.pokemon_per_page.max(1);
        let page = position / per_page;
        let top =
            (position % per_page) as f32 * (pages::home::ROW_HEIGHT + pages::home::ROW_SPACING);
        let bottom = top + pages::home::ROW_HEIGHT;
        let scroll = ready.list_scroll.y;
        let y = if page != ready.page || top < scroll || ready.list_viewport <= 0.0 {
            top
        } else if bottom > scroll + ready.list_viewport {
            bottom - ready.list_viewport
        } else {
            return Task::none();
        };
        ready.page = page;
        ready.list_scroll.y = y;
        scrollable::scroll_to(
            pages::home::LIST_SCROLL_ID.into(),
            AbsoluteOffset {
                x: None,
                y: Some(y),
            },
        )
    }

    /// Re-runs the list query after search, filters, sorting or settings changed.
    fn refresh_results(&mut self) {
        let State::Ready(ready) = &mut self.state else {
            return;
        };
        ready.results = ready.core.query(&ListQuery {
            search: &ready.search,
            filters: &ready.filters,
            type_mode: self.config.type_filtering_mode,
            show_forms: self.config.show_forms,
            sort: ready.sort,
            descending: ready.descending,
            lang: data_lang(),
        });
        if ready.favorites_only {
            let dex = ready.core.dex();
            let favorites = &self.config.favorites;
            ready
                .results
                .retain(|&i| favorites.contains(&dex.pokemon[i as usize].id.to_native()));
        }
        ready.page = 0;
        ready.list_scroll = AbsoluteOffset::default();
    }

    fn on_hotkey(&mut self, hotkey: Hotkey) -> Task<cosmic::Action<Message>> {
        let State::Ready(ready) = &self.state else {
            return Task::none();
        };
        let section = self.active_section();
        let in_details = section == Section::Pokemon && ready.details.is_some();
        let direction = match hotkey {
            Hotkey::ArrowLeft | Hotkey::ArrowUp => PaginationAction::Back,
            _ => PaginationAction::Next,
        };
        match hotkey {
            Hotkey::Escape if self.core.window.show_context => {
                self.core.window.show_context = false;
                Task::none()
            }
            Hotkey::Escape if in_details => self.update(Message::Back),
            Hotkey::Escape => Task::none(),
            Hotkey::Section(index) => match Section::ALL.get(index) {
                Some(section) => self.show_section(*section),
                None => Task::none(),
            },
            Hotkey::Search => match section {
                Section::Pokemon => {
                    widget::text_input::focus(widget::Id::new(pages::home::SEARCH_ID))
                }
                Section::Items => {
                    widget::text_input::focus(widget::Id::new(pages::items::SEARCH_ID))
                }
                Section::Moves => {
                    widget::text_input::focus(widget::Id::new(pages::move_list::SEARCH_ID))
                }
                Section::Types | Section::Compare | Section::Team | Section::Save => Task::none(),
            },
            Hotkey::ArrowUp | Hotkey::ArrowDown if in_details => {
                self.update(Message::Details(DetailsInput::Step(direction)))
            }
            Hotkey::ArrowUp | Hotkey::ArrowDown if section == Section::Pokemon => {
                let per_page = self.config.pokemon_per_page.max(1);
                let first = ready.results.get(ready.page * per_page).copied();
                match first.and_then(|i| ready.core.dex().pokemon.get(i as usize)) {
                    Some(pokemon) => self.update(Message::SelectPokemon(pokemon.id.to_native())),
                    None => Task::none(),
                }
            }
            Hotkey::ArrowUp | Hotkey::ArrowDown => Task::none(),
            Hotkey::ArrowLeft | Hotkey::ArrowRight => match section {
                Section::Types | Section::Compare | Section::Team | Section::Save => Task::none(),
                Section::Items => self.update(Message::Items(ItemsInput::Page(direction))),
                Section::Moves => self.update(Message::Moves(MovesInput::Page(direction))),
                Section::Pokemon if in_details => {
                    self.update(Message::Details(DetailsInput::Step(direction)))
                }
                Section::Pokemon => self.update(Message::List(ListInput::Page(direction))),
            },
        }
    }

    fn save_config(
        &mut self,
        apply: impl Fn(
            &mut StarryConfig,
            &cosmic::cosmic_config::Config,
        ) -> Result<bool, cosmic::cosmic_config::Error>,
        fallback: impl Fn(&mut StarryConfig),
    ) {
        if let Some(handler) = &self.config_handler {
            if let Err(err) = apply(&mut self.config, handler) {
                eprintln!("Failed to save setting: {err}");
                // Keep the change for this session even if it can't be persisted.
                fallback(&mut self.config);
            }
        } else {
            fallback(&mut self.config);
        }
    }

    fn on_config_input(&mut self, input: ConfigInput) -> Task<cosmic::Action<Message>> {
        match input {
            ConfigInput::UpdateTheme(index) => {
                let app_theme = match index {
                    1 => AppTheme::Dark,
                    2 => AppTheme::Light,
                    3 => AppTheme::ModernDex,
                    _ => AppTheme::System,
                };
                self.save_config(
                    |c, h| c.set_app_theme(h, app_theme),
                    |c| c.app_theme = app_theme,
                );
                return cosmic::command::set_theme(self.config.app_theme.theme());
            }
            ConfigInput::UpdatePokemonPerPage(v) => {
                let value = usize::from(v);
                self.save_config(
                    |c, h| c.set_pokemon_per_page(h, value),
                    |c| c.pokemon_per_page = value,
                );
                self.refresh_results();
            }
            ConfigInput::UpdateTypeFilterMode(index) => {
                let mode = if index == 1 {
                    TypeFilteringMode::Inclusive
                } else {
                    TypeFilteringMode::Exclusive
                };
                self.save_config(
                    |c, h| c.set_type_filtering_mode(h, mode),
                    |c| c.type_filtering_mode = mode,
                );
                self.refresh_results();
            }
            ConfigInput::UpdateShowForms(value) => {
                self.save_config(|c, h| c.set_show_forms(h, value), |c| c.show_forms = value);
                self.refresh_results();
            }
            ConfigInput::DeleteCache => {
                self.core.window.show_context = false;
                self.state = State::Loading;
                let data_dir = dirs::data_dir().map(|d| d.join(Self::APP_ID));
                if let Some(dir) = data_dir
                    && let Err(e) = remove_dir_contents(&dir)
                {
                    eprintln!("Error deleting cache: {e}");
                }
                return Task::perform(StarryCore::initialize(), |core| {
                    cosmic::action::app(Message::CoreLoaded(core))
                });
            }
            ConfigInput::UpdateCryVolume(value) => {
                self.save_config(|c, h| c.set_cry_volume(h, value), |c| c.cry_volume = value);
            }
        }
        Task::none()
    }

    /// The settings context page for this app.
    pub fn settings(&self) -> Element<'_, Message> {
        pages::settings::view(self)
    }
}

fn toggle<T: Ord>(set: &mut std::collections::BTreeSet<T>, value: T, on: bool) {
    if on {
        set.insert(value);
    } else {
        set.remove(&value);
    }
}

/// Number of pages for `len` results (at least one).
pub fn page_count(len: usize, per_page: usize) -> usize {
    len.div_ceil(per_page.max(1)).max(1)
}

/// Index of the previous/next Pokémon: inside the current results when the open Pokémon is
/// part of them, otherwise in plain Pokédex order.
fn step(results: &[u32], total: usize, current: usize, direction: &PaginationAction) -> usize {
    let forward = matches!(direction, PaginationAction::Next);
    if let Some(position) = results.iter().position(|&i| i as usize == current) {
        let len = results.len();
        let next = if forward {
            (position + 1) % len
        } else {
            (position + len - 1) % len
        };
        return results[next] as usize;
    }
    if total == 0 {
        return current;
    }
    if forward {
        (current + 1) % total
    } else {
        (current + total - 1) % total
    }
}

#[derive(Debug, Clone)]
pub enum Hotkey {
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    /// Focus the search box of the current section
    Search,
    /// Show the section at this position of the navigation bar
    Section(usize),
    Escape,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stepping_wraps_around_results() {
        let results = [5, 9, 2];
        assert_eq!(step(&results, 100, 9, &PaginationAction::Next), 2);
        assert_eq!(step(&results, 100, 2, &PaginationAction::Next), 5);
        assert_eq!(step(&results, 100, 5, &PaginationAction::Back), 2);
        // Not in results → dex order
        assert_eq!(step(&results, 100, 50, &PaginationAction::Next), 51);
        assert_eq!(step(&results, 100, 0, &PaginationAction::Back), 99);
    }

    #[test]
    fn page_counts() {
        assert_eq!(page_count(0, 30), 1);
        assert_eq!(page_count(30, 30), 1);
        assert_eq!(page_count(31, 30), 2);
    }
}
