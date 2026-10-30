// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    cosmic_config::{self, Config, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry},
    theme,
};
use serde::{Deserialize, Serialize};

const APP_ID: &str = "dev.mariinkys.StarryDex";
const CONFIG_VERSION: u64 = 3;

/// Contains the configurations fields of the application
#[derive(Debug, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 3]
pub struct StarryConfig {
    pub app_theme: AppTheme,
    pub pokemon_per_page: usize,
    pub type_filtering_mode: TypeFilteringMode,
    /// Also list alternate forms (megas, regional forms...) as separate entries.
    pub show_forms: bool,
    /// Volume of the Pokémon cries, 0–100 (%).
    pub cry_volume: u8,
    /// Ids of the Pokémon marked as favorite.
    pub favorites: Vec<u32>,
    /// Ids of the Pokémon in the team, in slot order (at most [`TEAM_SIZE`]).
    pub team: Vec<u32>,
}

pub const TEAM_SIZE: usize = 6;

impl Default for StarryConfig {
    fn default() -> Self {
        Self {
            app_theme: Default::default(),
            type_filtering_mode: Default::default(),
            pokemon_per_page: 60,
            show_forms: false,
            cry_volume: 5,
            favorites: Vec::new(),
            team: Vec::new(),
        }
    }
}

impl StarryConfig {
    pub fn config_handler() -> Option<Config> {
        Config::new(APP_ID, CONFIG_VERSION).ok()
    }

    pub fn config() -> StarryConfig {
        match Self::config_handler() {
            Some(config_handler) => {
                StarryConfig::get_entry(&config_handler).unwrap_or_else(|(error, config)| {
                    eprintln!("Error whilst loading config: {error:#?}");
                    config
                })
            }
            None => StarryConfig::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum AppTheme {
    Dark,
    Light,
    #[default]
    System,
    ModernDex,
}

impl AppTheme {
    pub fn theme(&self) -> theme::Theme {
        match self {
            Self::Dark => theme::Theme::dark(),
            Self::Light => theme::Theme::light(),
            Self::System => theme::system_preference(),
            Self::ModernDex => crate::themes::modern_dex(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum TypeFilteringMode {
    Inclusive,
    #[default]
    Exclusive,
}

/// Represents the different inputs that can happen in the config [`ContextPage`]
#[derive(Debug, Clone)]
pub enum ConfigInput {
    /// Update the application theme
    UpdateTheme(usize),
    /// Update the pokemon per page setting
    UpdatePokemonPerPage(u16),
    /// Update the type filtering mode setting
    UpdateTypeFilterMode(usize),
    /// Toggle listing alternate forms
    UpdateShowForms(bool),
    /// Ask to delete and recreate the app cache
    DeleteCache,
    /// Update the volume of the Pokémon cries (0–100)
    UpdateCryVolume(u8),
}
