//! Window frame persistence (MDV-048 P8): the main window's size and
//! position are saved once after a resize or move settles and on close (not
//! once per event, and never full screen), and the launch settings restore
//! them fully on a screen.

use super::*;
use crate::app::lifecycle::{
    window_frame_settle_step, window_settings_restored, WINDOW_FRAME_SETTLE, WINDOW_MIN_SIZE,
};
use crate::window_frame::{ScreenRect, WindowFrame};
use iced::window::{Id, Position};
use std::time::{Duration, Instant};

/// A fresh app whose isolated prefs file does not exist yet.
fn fresh_app() -> (App, PathBuf) {
    let app = App::default();
    let path = app
        .quick_slots_persistence_path
        .clone()
        .expect("the test prefs path");
    let _ = std::fs::remove_file(&path);
    (app, path)
}

fn frame(x: f32, y: f32, width: f32, height: f32, maximized: bool) -> WindowFrame {
    WindowFrame {
        width,
        height,
        x: Some(x),
        y: Some(y),
        maximized,
    }
}

fn saved(path: &Path) -> Option<WindowFrame> {
    crate::prefs::load_from(path).window
}

/// Move and resize the window, let the timer fire after the frame settled,
/// and answer its sample.
fn settle(app: &mut App, id: Id, fullscreen: bool, maximized: bool) {
    app.window_frame_changed_at = Instant::now().checked_sub(WINDOW_FRAME_SETTLE * 2);
    let _ = app.update(Message::WindowFrameSettled(id));
    let _ = app.update(Message::WindowFrameSampled {
        fullscreen,
        maximized,
    });
}

#[test]
fn the_settle_timer_waits_for_half_a_second_of_stillness() {
    let ms = Duration::from_millis;
    assert_eq!(WINDOW_FRAME_SETTLE, ms(500));
    assert_eq!(window_frame_settle_step(ms(0)), Some(ms(500)));
    assert_eq!(window_frame_settle_step(ms(120)), Some(ms(380)));
    assert_eq!(window_frame_settle_step(ms(499)), Some(ms(1)));
    assert_eq!(window_frame_settle_step(ms(500)), None);
    assert_eq!(window_frame_settle_step(ms(4000)), None);
}

#[test]
fn a_burst_of_moves_and_resizes_is_saved_once_after_it_settles() {
    let (mut app, path) = fresh_app();
    let id = Id::unique();
    let _ = app.update(Message::WindowOpened(
        id,
        Size::new(1000.0, 700.0),
        Some(Point::new(10.0, 20.0)),
    ));
    assert_eq!(app.window_position, Some(Point::new(10.0, 20.0)));
    assert!(!app.window_frame_settle_armed, "opening is not a change");

    for step in 0..30 {
        let step = step as f32;
        let _ = app.update(Message::WindowResized(id, Size::new(900.0 + step, 600.0)));
        let _ = app.update(Message::WindowMoved(id, Point::new(40.0 + step, 80.0)));
    }
    assert!(app.window_frame_settle_armed, "one timer for the burst");
    assert!(!path.exists(), "no prefs write while the window moves");

    // The timer fires while the window still moves: it waits again.
    app.window_frame_changed_at = Some(Instant::now());
    let again = app.update(Message::WindowFrameSettled(id));
    assert!(app.window_frame_settle_armed);
    assert_eq!(again.units(), 1, "one more timer, no sample");
    assert!(!path.exists());

    // Still for the settle delay: one sample of the window's state.
    app.window_frame_changed_at = Instant::now().checked_sub(Duration::from_millis(600));
    let sample = app.update(Message::WindowFrameSettled(id));
    assert!(!app.window_frame_settle_armed);
    assert!(sample.units() > 0, "the sample asks the window");
    assert!(!path.exists(), "the write waits for the sample");
    let _ = app.update(Message::WindowFrameSampled {
        fullscreen: false,
        maximized: false,
    });
    assert_eq!(saved(&path), Some(frame(69.0, 80.0, 929.0, 600.0, false)));

    // The same frame again writes nothing.
    std::fs::remove_file(&path).expect("the prefs file");
    let _ = app.update(Message::WindowFrameSampled {
        fullscreen: false,
        maximized: false,
    });
    assert!(!path.exists(), "an unchanged frame is not written again");

    // The next change starts a new burst with its own timer.
    let _ = app.update(Message::WindowMoved(id, Point::new(300.0, 90.0)));
    assert!(app.window_frame_settle_armed);
    settle(&mut app, id, false, false);
    assert_eq!(saved(&path), Some(frame(300.0, 90.0, 929.0, 600.0, false)));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn full_screen_keeps_the_saved_frame() {
    let (mut app, path) = fresh_app();
    let id = Id::unique();
    let _ = app.update(Message::WindowMoved(id, Point::new(40.0, 80.0)));
    let _ = app.update(Message::WindowResized(id, Size::new(900.0, 600.0)));
    settle(&mut app, id, false, false);
    let normal = frame(40.0, 80.0, 900.0, 600.0, false);
    assert_eq!(saved(&path), Some(normal));

    // Entering full screen resizes and moves the window.
    let _ = app.update(Message::WindowMoved(id, Point::new(0.0, 0.0)));
    let _ = app.update(Message::WindowResized(id, Size::new(1470.0, 956.0)));
    std::fs::remove_file(&path).expect("the prefs file");
    settle(&mut app, id, true, false);
    assert!(!path.exists(), "full screen writes nothing");
    assert_eq!(app.prefs.window, Some(normal));

    // A close in full screen keeps the normal frame too.
    app.window_fullscreen = true;
    let _ = app.update(Message::WindowCloseRequested(id));
    assert!(!path.exists(), "a close in full screen writes nothing");
}

#[test]
fn a_maximized_window_keeps_its_normal_frame_under_the_zoom() {
    let (mut app, path) = fresh_app();
    let id = Id::unique();
    let _ = app.update(Message::WindowMoved(id, Point::new(40.0, 80.0)));
    let _ = app.update(Message::WindowResized(id, Size::new(900.0, 600.0)));
    settle(&mut app, id, false, false);

    // Zoom fills the screen.
    let _ = app.update(Message::WindowMoved(id, Point::new(0.0, 34.0)));
    let _ = app.update(Message::WindowResized(id, Size::new(1470.0, 922.0)));
    settle(&mut app, id, false, true);
    assert_eq!(saved(&path), Some(frame(40.0, 80.0, 900.0, 600.0, true)));
    assert!(app.window_maximized);

    // Unzoom to a new normal frame.
    let _ = app.update(Message::WindowMoved(id, Point::new(60.0, 90.0)));
    let _ = app.update(Message::WindowResized(id, Size::new(1000.0, 650.0)));
    settle(&mut app, id, false, false);
    assert_eq!(saved(&path), Some(frame(60.0, 90.0, 1000.0, 650.0, false)));
    assert!(!app.window_maximized);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_first_zoomed_frame_is_saved_as_it_is() {
    // No normal frame to keep: the zoomed frame is the saved one.
    let (mut app, path) = fresh_app();
    let id = Id::unique();
    let _ = app.update(Message::WindowMoved(id, Point::new(0.0, 34.0)));
    let _ = app.update(Message::WindowResized(id, Size::new(1470.0, 922.0)));
    settle(&mut app, id, false, true);
    assert_eq!(saved(&path), Some(frame(0.0, 34.0, 1470.0, 922.0, true)));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn closing_saves_a_frame_that_has_not_settled_yet() {
    let (mut app, path) = fresh_app();
    let id = Id::unique();
    let _ = app.update(Message::WindowOpened(
        id,
        Size::new(1000.0, 700.0),
        Some(Point::new(10.0, 20.0)),
    ));
    let _ = app.update(Message::WindowResized(id, Size::new(1100.0, 720.0)));
    let _ = app.update(Message::WindowMoved(id, Point::new(50.0, 60.0)));
    assert!(!path.exists());
    let _ = app.update(Message::WindowCloseRequested(id));
    assert!(app.window_closing, "a clean window closes at once");
    assert_eq!(saved(&path), Some(frame(50.0, 60.0, 1100.0, 720.0, false)));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_platform_without_positions_saves_the_size_only() {
    let (mut app, path) = fresh_app();
    let id = Id::unique();
    let _ = app.update(Message::WindowOpened(id, Size::new(1000.0, 700.0), None));
    let _ = app.update(Message::WindowResized(id, Size::new(1100.0, 720.0)));
    settle(&mut app, id, false, false);
    let frame = saved(&path).expect("the size is saved");
    assert_eq!((frame.width, frame.height), (1100.0, 720.0));
    assert_eq!((frame.x, frame.y), (None, None));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn launch_settings_restore_the_saved_frame_on_a_screen() {
    let laptop = ScreenRect {
        x: 0.0,
        y: 34.0,
        width: 1470.0,
        height: 922.0,
    };
    // No saved frame: today's launch settings.
    let plain = window_settings_restored(false, None, Some(&[laptop]));
    assert_eq!(plain.size, window_settings(false).size);
    assert!(matches!(plain.position, Position::Default));
    assert!(!plain.maximized);

    // On screen: as saved.
    let settings = window_settings_restored(
        false,
        Some(&frame(100.0, 120.0, 900.0, 600.0, false)),
        Some(&[laptop]),
    );
    assert_eq!(settings.size, Size::new(900.0, 600.0));
    assert!(
        matches!(settings.position, Position::Specific(p) if p == Point::new(100.0, 120.0)),
        "{:?}",
        settings.position
    );
    assert_eq!(settings.min_size, Some(WINDOW_MIN_SIZE));
    assert!(!settings.fullscreen, "full screen is never restored");
    assert!(!settings.exit_on_close_request, "close protection stays");

    // Off screen: fully back on it.
    let settings = window_settings_restored(
        true,
        Some(&frame(5000.0, -400.0, 900.0, 600.0, true)),
        Some(&[laptop]),
    );
    assert!(
        matches!(settings.position, Position::Specific(p) if p == Point::new(570.0, 34.0)),
        "{:?}",
        settings.position
    );
    assert!(settings.maximized, "a zoomed window opens zoomed");
    assert!(settings.transparent);

    // No screen list: the size, centered, never below the minimum.
    let settings =
        window_settings_restored(false, Some(&frame(100.0, 120.0, 300.0, 200.0, false)), None);
    assert_eq!(settings.size, WINDOW_MIN_SIZE);
    assert!(matches!(settings.position, Position::Centered));
}

#[test]
fn a_minimized_window_keeps_the_saved_frame() {
    // Windows reports a minimized window as 0×0 at -32000, -32000.
    let (mut app, path) = fresh_app();
    let id = Id::unique();
    let _ = app.update(Message::WindowMoved(id, Point::new(40.0, 80.0)));
    let _ = app.update(Message::WindowResized(id, Size::new(900.0, 600.0)));
    settle(&mut app, id, false, false);
    let normal = frame(40.0, 80.0, 900.0, 600.0, false);
    assert_eq!(saved(&path), Some(normal));
    let _ = app.update(Message::WindowMoved(id, Point::new(-32000.0, -32000.0)));
    let _ = app.update(Message::WindowResized(id, Size::new(0.0, 0.0)));
    settle(&mut app, id, false, false);
    assert_eq!(saved(&path), Some(normal));
    let _ = app.update(Message::WindowCloseRequested(id));
    assert_eq!(saved(&path), Some(normal), "a close while minimized");
    let _ = std::fs::remove_file(&path);
}
