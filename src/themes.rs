// SPDX-License-Identifier: GPL-3.0-only

//! The "Modern Dex" theme: a dark, blue-tinted palette with a light blue accent.

use std::sync::Arc;

use cosmic::cosmic_theme::ThemeBuilder;
use cosmic::cosmic_theme::palette::{Srgb, Srgba};

const BACKGROUND: u32 = 0x0F1520;
const PRIMARY_CONTAINER: u32 = 0x161D2B;
const SECONDARY_CONTAINER: u32 = 0x1E2738;
const NEUTRAL_TINT: u32 = 0x1A2C42;
const TEXT_TINT: u32 = 0xE6EDF7;
const ACCENT: u32 = 0x7CC4FA;
const SUCCESS: u32 = 0x7DD3A8;
const WARNING: u32 = 0xFBE491;
const DESTRUCTIVE: u32 = 0xE8546A;

fn rgb(hex: u32) -> Srgb {
    Srgb::new(
        ((hex >> 16) & 0xFF) as u8,
        ((hex >> 8) & 0xFF) as u8,
        (hex & 0xFF) as u8,
    )
    .into_format()
}

fn rgba(hex: u32) -> Srgba {
    rgb(hex).into()
}

pub fn modern_dex() -> cosmic::Theme {
    let mut builder = ThemeBuilder::dark()
        .bg_color(rgba(BACKGROUND))
        .primary_container_bg(rgba(PRIMARY_CONTAINER))
        .neutral_tint(rgb(NEUTRAL_TINT))
        .text_tint(rgb(TEXT_TINT))
        .accent(rgb(ACCENT))
        .success(rgb(SUCCESS))
        .warning(rgb(WARNING))
        .destructive(rgb(DESTRUCTIVE));
    builder.secondary_container_bg = Some(rgba(SECONDARY_CONTAINER));
    cosmic::Theme::custom(Arc::new(builder.build()))
}
