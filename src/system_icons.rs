// SPDX-License-Identifier: GPL-3.0-only

//! Icons libcosmic loads by name for its own widgets (window buttons of the header bar and
//! the nav bar toggle). They come from the Cosmic icon theme, which is usually missing outside
//! COSMIC, leaving those buttons blank. The icon lookup ends by searching for loose
//! `<name>.svg` files in every `icons` folder of `XDG_DATA_DIRS`, so the icons are written to
//! the app cache and that folder is added last: any installed theme still takes precedence.

use std::path::{Path, PathBuf};

const APP_ID: &str = "dev.mariinkys.StarryDex";

macro_rules! icon {
    ($name:literal) => {
        (
            $name,
            include_bytes!(concat!("../resources/icons/system/", $name, ".svg")).as_slice(),
        )
    };
}

const ICONS: &[(&str, &[u8])] = &[
    icon!("navbar-open-symbolic"),
    icon!("navbar-closed-symbolic"),
    icon!("window-minimize-symbolic"),
    icon!("window-maximize-symbolic"),
    icon!("window-restore-symbolic"),
    icon!("window-close-symbolic"),
];

/// Writes the icons and adds them to the icon search path. Must run before any icon is looked
/// up, while the program is still single threaded.
pub fn install() {
    let Some(root) = dirs::cache_dir().map(|cache| cache.join(APP_ID)) else {
        return;
    };
    if let Err(err) = write_icons(&root.join("icons")) {
        eprintln!("failed to install fallback icons: {err}");
        return;
    }

    // Per the XDG spec an unset or empty variable means these two.
    let current = std::env::var("XDG_DATA_DIRS")
        .ok()
        .filter(|dirs| !dirs.is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".to_string());
    let root = root.to_string_lossy();
    if current.split(':').any(|dir| dir == root) {
        return;
    }
    // SAFETY: called at the start of `main`, before any other thread exists.
    unsafe {
        std::env::set_var("XDG_DATA_DIRS", format!("{current}:{root}"));
    }
}

fn write_icons(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    for (name, svg) in ICONS {
        let path: PathBuf = dir.join(format!("{name}.svg"));
        if std::fs::read(&path).ok().as_deref() != Some(*svg) {
            std::fs::write(&path, svg)?;
        }
    }
    Ok(())
}
