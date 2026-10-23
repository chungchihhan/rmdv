//! The main window's saved frame (MDV-048 P8): what prefs keep, how a saved
//! frame loads leniently, and where a restored window goes so that it is
//! fully on a visible screen.
//!
//! Coordinates are logical points with the origin at the top-left corner of
//! the main screen (the one with the menu bar) and y growing down. These are
//! the coordinates Winit reports in `Moved` and takes in a specific launch
//! position, on every platform.

use serde::{Deserialize, Serialize};

/// Frame sizes beyond this are corrupt, not real windows.
const MAX_EXTENT: f32 = 100_000.0;

/// The saved frame of the main window. `x` and `y` are the frame's top-left
/// corner; they are absent when the platform never reported a position.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowFrame {
    pub width: f32,
    pub height: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<f32>,
    /// The window was zoomed (macOS) or maximized. The size and position
    /// stay those of the normal frame under the zoom.
    #[serde(default)]
    pub maximized: bool,
}

impl WindowFrame {
    /// A saved frame from prefs JSON. A malformed, non-finite, or
    /// non-positive size drops the whole frame; a malformed position drops
    /// only the position; a malformed `maximized` reads as false. Nothing
    /// here fails the prefs file.
    pub fn from_json(value: &serde_json::Value) -> Option<WindowFrame> {
        let number = |key: &str| {
            value
                .get(key)
                .and_then(serde_json::Value::as_f64)
                .map(|v| v as f32)
                .filter(|v| v.is_finite() && v.abs() <= MAX_EXTENT)
        };
        let width = number("width").filter(|w| *w > 0.0)?;
        let height = number("height").filter(|h| *h > 0.0)?;
        let (x, y) = match (number("x"), number("y")) {
            (Some(x), Some(y)) => (Some(x), Some(y)),
            _ => (None, None),
        };
        let maximized = value
            .get("maximized")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        Some(WindowFrame {
            width,
            height,
            x,
            y,
            maximized,
        })
    }
}

/// A screen's visible area (without the menu bar and the Dock), in the
/// coordinates of this module.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ScreenRect {
    /// An AppKit rectangle (origin bottom-left of the main screen, y up)
    /// in this module's coordinates. `main_height` is the main screen's
    /// full height, the same flip Winit uses.
    pub fn from_appkit(main_height: f64, x: f64, y: f64, width: f64, height: f64) -> ScreenRect {
        ScreenRect {
            x: x as f32,
            y: (main_height - (y + height)) as f32,
            width: width as f32,
            height: height as f32,
        }
    }

    fn is_usable(&self) -> bool {
        [self.x, self.y, self.width, self.height]
            .iter()
            .all(|v| v.is_finite())
            && self.width > 0.0
            && self.height > 0.0
    }

    fn overlap(&self, x: f32, y: f32, width: f32, height: f32) -> f32 {
        let w = (self.x + self.width).min(x + width) - self.x.max(x);
        let h = (self.y + self.height).min(y + height) - self.y.max(y);
        w.max(0.0) * h.max(0.0)
    }

    /// Squared distance from a point to this rectangle (0 inside it).
    fn distance_sq(&self, px: f32, py: f32) -> f32 {
        let dx = (self.x - px).max(0.0).max(px - (self.x + self.width));
        let dy = (self.y - py).max(0.0).max(py - (self.y + self.height));
        dx * dx + dy * dy
    }
}

/// Where the restored window goes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Placement {
    /// Centered on the main screen: no saved position, or no screen list.
    Centered,
    /// The frame's top-left corner.
    At { x: f32, y: f32 },
}

/// The launch frame of a restored window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Restore {
    pub width: f32,
    pub height: f32,
    pub placement: Placement,
    pub maximized: bool,
}

/// The launch frame for a saved frame. The size is at least `min`. With a
/// screen list, the window goes fully onto the screen it overlaps most (the
/// nearest one when it overlaps none, for example after a display was
/// unplugged), and is no larger than that screen unless `min` is. Without a
/// screen list (`None` or empty) the size is restored and the window is
/// centered.
pub fn restore(saved: &WindowFrame, screens: Option<&[ScreenRect]>, min: (f32, f32)) -> Restore {
    let width = saved.width.max(min.0);
    let height = saved.height.max(min.1);
    let screens: Vec<ScreenRect> = screens
        .unwrap_or(&[])
        .iter()
        .copied()
        .filter(ScreenRect::is_usable)
        .collect();
    let centered = |width: f32, height: f32| Restore {
        width,
        height,
        placement: Placement::Centered,
        maximized: saved.maximized,
    };
    let Some(&main) = screens.first() else {
        return centered(width, height);
    };
    let fit = |screen: &ScreenRect| {
        (
            width.min(screen.width).max(min.0),
            height.min(screen.height).max(min.1),
        )
    };
    let (Some(x), Some(y)) = (saved.x, saved.y) else {
        let (width, height) = fit(&main);
        return centered(width, height);
    };
    let screen = screens
        .iter()
        .copied()
        .map(|s| (s, s.overlap(x, y, width, height)))
        .filter(|(_, area)| *area > 0.0)
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(s, _)| s)
        .unwrap_or_else(|| {
            let (cx, cy) = (x + width / 2.0, y + height / 2.0);
            screens
                .iter()
                .copied()
                .min_by(|a, b| a.distance_sq(cx, cy).total_cmp(&b.distance_sq(cx, cy)))
                .unwrap_or(main)
        });
    let (width, height) = fit(&screen);
    let x = x.min(screen.x + screen.width - width).max(screen.x);
    let y = y.min(screen.y + screen.height - height).max(screen.y);
    Restore {
        width,
        height,
        placement: Placement::At { x, y },
        maximized: saved.maximized,
    }
}

/// The visible area of every screen, the main screen first, read from
/// AppKit. `None` off the main thread. AppKit answers before the
/// application object exists, so this works before the window opens.
#[cfg(target_os = "macos")]
pub fn visible_screens() -> Option<Vec<ScreenRect>> {
    let mtm = objc2::MainThreadMarker::new()?;
    let screens = objc2_app_kit::NSScreen::screens(mtm);
    let main_height = screens.firstObject()?.frame().size.height;
    Some(
        screens
            .iter()
            .map(|screen| {
                let r = screen.visibleFrame();
                ScreenRect::from_appkit(
                    main_height,
                    r.origin.x,
                    r.origin.y,
                    r.size.width,
                    r.size.height,
                )
            })
            .collect(),
    )
}

/// No screen list before the window opens on this platform: a restored
/// window keeps its size and is centered.
#[cfg(not(target_os = "macos"))]
pub fn visible_screens() -> Option<Vec<ScreenRect>> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const MIN: (f32, f32) = (640.0, 420.0);

    fn frame(x: f32, y: f32, width: f32, height: f32) -> WindowFrame {
        WindowFrame {
            width,
            height,
            x: Some(x),
            y: Some(y),
            maximized: false,
        }
    }

    fn screen(x: f32, y: f32, width: f32, height: f32) -> ScreenRect {
        ScreenRect {
            x,
            y,
            width,
            height,
        }
    }

    /// A 1470×956 laptop screen with a 34 pt menu bar.
    fn laptop() -> ScreenRect {
        screen(0.0, 34.0, 1470.0, 922.0)
    }

    fn at(restore: &Restore) -> (f32, f32) {
        match restore.placement {
            Placement::At { x, y } => (x, y),
            Placement::Centered => panic!("expected a position: {restore:?}"),
        }
    }

    #[test]
    fn a_saved_frame_loads_leniently() {
        let load = |value: serde_json::Value| WindowFrame::from_json(&value);
        assert_eq!(
            load(json!({"width": 900, "height": 600, "x": 40, "y": 80})),
            Some(frame(40.0, 80.0, 900.0, 600.0))
        );
        assert_eq!(
            load(json!({"width": 900, "height": 600, "x": 40, "y": 80, "maximized": true}))
                .map(|f| f.maximized),
            Some(true)
        );
        // A bad size drops the frame.
        for bad in [
            json!({"height": 600}),
            json!({"width": "wide", "height": 600}),
            json!({"width": 0, "height": 600}),
            json!({"width": -5, "height": 600}),
            json!({"width": 900, "height": 1e30}),
            json!(42),
            json!(null),
        ] {
            assert_eq!(load(bad.clone()), None, "{bad}");
        }
        // A bad or half position drops only the position.
        for value in [
            json!({"width": 900, "height": 600, "x": 40}),
            json!({"width": 900, "height": 600, "x": "left", "y": 80}),
            json!({"width": 900, "height": 600, "x": 40, "y": 1e30}),
        ] {
            let loaded = load(value.clone()).expect("the size still loads");
            assert_eq!((loaded.x, loaded.y), (None, None), "{value}");
            assert_eq!((loaded.width, loaded.height), (900.0, 600.0), "{value}");
        }
        // A bad maximized flag reads as not maximized.
        let loaded = load(json!({"width": 900, "height": 600, "maximized": "yes"})).unwrap();
        assert!(!loaded.maximized);
    }

    #[test]
    fn a_frame_on_screen_is_restored_as_saved() {
        let saved = frame(100.0, 120.0, 900.0, 600.0);
        let restored = restore(&saved, Some(&[laptop()]), MIN);
        assert_eq!((restored.width, restored.height), (900.0, 600.0));
        assert_eq!(at(&restored), (100.0, 120.0));
        assert!(!restored.maximized);
    }

    #[test]
    fn a_frame_partly_off_screen_moves_fully_onto_it() {
        let s = laptop();
        // Past the right and bottom edges.
        let restored = restore(&frame(1200.0, 800.0, 900.0, 600.0), Some(&[s]), MIN);
        assert_eq!(at(&restored), (1470.0 - 900.0, 34.0 + 922.0 - 600.0));
        // Past the left edge and under the menu bar.
        let restored = restore(&frame(-300.0, 0.0, 900.0, 600.0), Some(&[s]), MIN);
        assert_eq!(at(&restored), (0.0, 34.0));
    }

    #[test]
    fn a_frame_larger_than_its_screen_shrinks_to_fit_but_not_below_the_minimum() {
        let s = laptop();
        let restored = restore(&frame(10.0, 40.0, 3000.0, 2000.0), Some(&[s]), MIN);
        assert_eq!((restored.width, restored.height), (1470.0, 922.0));
        assert_eq!(at(&restored), (0.0, 34.0));
        // A screen smaller than the minimum: the minimum wins, at the
        // screen's top-left corner.
        let tiny = screen(0.0, 0.0, 500.0, 300.0);
        let restored = restore(&frame(50.0, 50.0, 900.0, 600.0), Some(&[tiny]), MIN);
        assert_eq!((restored.width, restored.height), MIN);
        assert_eq!(at(&restored), (0.0, 0.0));
    }

    #[test]
    fn a_frame_smaller_than_the_minimum_grows_to_it() {
        let restored = restore(&frame(100.0, 100.0, 200.0, 100.0), Some(&[laptop()]), MIN);
        assert_eq!((restored.width, restored.height), MIN);
        let restored = restore(&frame(100.0, 100.0, 200.0, 100.0), None, MIN);
        assert_eq!((restored.width, restored.height), MIN);
    }

    #[test]
    fn a_frame_goes_to_the_screen_it_overlaps_most() {
        // An external screen to the right of the laptop, top-aligned.
        let external = screen(1470.0, 0.0, 2560.0, 1415.0);
        let screens = [laptop(), external];
        // Mostly on the external screen, poking back onto the laptop.
        let restored = restore(&frame(1300.0, 200.0, 1200.0, 800.0), Some(&screens), MIN);
        assert_eq!(at(&restored), (1470.0, 200.0));
        assert_eq!((restored.width, restored.height), (1200.0, 800.0));
        // A big window on the external screen keeps its size there.
        let restored = restore(&frame(1600.0, 100.0, 2400.0, 1300.0), Some(&screens), MIN);
        assert_eq!((restored.width, restored.height), (2400.0, 1300.0));
        assert_eq!(at(&restored), (1600.0, 100.0));
    }

    #[test]
    fn a_frame_on_an_unplugged_screen_goes_to_the_nearest_one() {
        let external = screen(1470.0, 0.0, 2560.0, 1415.0);
        // Saved on a screen that sat left of the laptop; now only the
        // laptop and the external screen are there.
        let restored = restore(
            &frame(-3000.0, 100.0, 2400.0, 1300.0),
            Some(&[laptop(), external]),
            MIN,
        );
        assert_eq!((restored.width, restored.height), (1470.0, 922.0));
        assert_eq!(at(&restored), (0.0, 34.0));
        // Far to the right: the external screen is nearer.
        let restored = restore(
            &frame(9000.0, 100.0, 900.0, 600.0),
            Some(&[laptop(), external]),
            MIN,
        );
        assert_eq!(at(&restored), (1470.0 + 2560.0 - 900.0, 100.0));
    }

    #[test]
    fn without_a_screen_list_or_a_position_the_window_is_centered() {
        let saved = frame(100.0, 120.0, 900.0, 600.0);
        for screens in [None, Some(&[][..])] {
            let restored = restore(&saved, screens, MIN);
            assert_eq!(restored.placement, Placement::Centered, "{screens:?}");
            assert_eq!((restored.width, restored.height), (900.0, 600.0));
        }
        // Unusable screen rectangles count as no list.
        let restored = restore(&saved, Some(&[screen(0.0, 0.0, 0.0, f32::NAN)]), MIN);
        assert_eq!(restored.placement, Placement::Centered);
        // No saved position: centered, and no larger than the main screen.
        let sized = WindowFrame {
            x: None,
            y: None,
            ..frame(0.0, 0.0, 3000.0, 600.0)
        };
        let restored = restore(&sized, Some(&[laptop()]), MIN);
        assert_eq!(restored.placement, Placement::Centered);
        assert_eq!((restored.width, restored.height), (1470.0, 600.0));
    }

    #[test]
    fn a_maximized_frame_is_restored_maximized() {
        let saved = WindowFrame {
            maximized: true,
            ..frame(100.0, 120.0, 900.0, 600.0)
        };
        assert!(restore(&saved, Some(&[laptop()]), MIN).maximized);
        assert!(restore(&saved, None, MIN).maximized);
    }

    #[test]
    fn appkit_rectangles_flip_to_top_left_coordinates() {
        // The main screen's visible frame (bottom-left origin) without a
        // 34 pt menu bar.
        let flipped = ScreenRect::from_appkit(956.0, 0.0, 0.0, 1470.0, 922.0);
        assert_eq!(flipped, laptop());
        // A screen above the main screen has a negative top.
        let above = ScreenRect::from_appkit(956.0, 0.0, 956.0, 1920.0, 1080.0);
        assert_eq!(above, screen(0.0, -1080.0, 1920.0, 1080.0));
    }
}
