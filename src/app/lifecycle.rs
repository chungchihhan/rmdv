//! The window's lifecycle: its launch settings and closing it.

use super::*;

/// The smallest window size. A narrower or shorter window crowds the
/// toolbar's controls and the reading column.
pub(super) const WINDOW_MIN_SIZE: iced::Size = iced::Size::new(640.0, 420.0);

/// The main window's settings. `transparent` lets whole-window glass show
/// through (macOS only).
pub fn window_settings(transparent: bool) -> iced::window::Settings {
    #[cfg(target_os = "macos")]
    let platform_specific = iced::window::settings::PlatformSpecific {
        title_hidden: true,
        titlebar_transparent: true,
        fullsize_content_view: true,
    };
    #[cfg(not(target_os = "macos"))]
    let platform_specific = iced::window::settings::PlatformSpecific::default();
    iced::window::Settings {
        platform_specific,
        transparent,
        min_size: Some(WINDOW_MIN_SIZE),
        ..Default::default()
    }
}
