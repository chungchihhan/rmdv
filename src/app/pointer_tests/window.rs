//! Window and lifecycle pointer paths (MDV-048 P2, group 2): moving the
//! window from the toolbar, close protection, the minimum size, the drop
//! highlight, and the welcome page's recent items.

use super::group3::tree_fixture;
use super::*;
use crate::icon::ic;
use crate::prefs::ToolbarMode;

fn always(mut app: App) -> App {
    let _ = app.update(Message::SetToolbarMode(ToolbarMode::Always));
    app
}

/// A point on the toolbar's blank space: left of the view switch and clear
/// of the file name and the Show Sidebar button.
fn blank_toolbar_point(ui: &mut PointerUi, app: &App) -> Point {
    let bar = ui.id_bounds(app, "toolbar").expect("toolbar shown");
    let read = ui.text_bounds(app, "Read").expect("Read segment");
    let name = ui.text_bounds(app, "pointer-long.md");
    let y = bar.y + bar.height / 2.0;
    let x = (0..40)
        .map(|step| bar.x + 160.0 + step as f32 * 10.0)
        .find(|x| {
            *x < read.x - 20.0 && name.is_none_or(|n| *x < n.x - 8.0 || *x > n.x + n.width + 8.0)
        })
        .expect("blank toolbar space");
    Point::new(x, y)
}

fn drags(messages: &[Message]) -> bool {
    has(messages, |m| matches!(m, Message::WindowDrag))
}

fn zooms(messages: &[Message]) -> bool {
    has(messages, |m| {
        matches!(m, Message::WindowTitlebarDoubleClick)
    })
}

// --- 1. Window drag -------------------------------------------------------------

#[test]
fn pressing_blank_toolbar_space_moves_the_window() {
    let app = always(long_doc_app());
    let mut ui = PointerUi::new();
    let at = blank_toolbar_point(&mut ui, &app);
    let messages = ui.click(&app, at);
    assert!(drags(&messages), "{messages:?}");
    assert!(!zooms(&messages), "one click does not zoom");
}

#[test]
fn toolbar_buttons_do_not_move_the_window() {
    let app = always(long_doc_app());
    let mut ui = PointerUi::new();
    for label in ["Read", "Edit"] {
        let at = center(ui.text_bounds(&app, label).expect("segment"));
        let messages = ui.click(&app, at);
        assert!(!drags(&messages), "{label}: {messages:?}");
    }
    let palette = ui
        .text_bounds(&app, &ic::COMMAND.to_string())
        .expect("palette button");
    let messages = ui.click(&app, center(palette));
    assert!(
        has(&messages, |m| matches!(m, Message::OpenCommandPalette)),
        "{messages:?}"
    );
    assert!(!drags(&messages), "{messages:?}");
}

#[test]
fn double_clicking_blank_toolbar_space_zooms_the_window() {
    let app = always(long_doc_app());
    let mut ui = PointerUi::new();
    let at = blank_toolbar_point(&mut ui, &app);
    let mut messages = ui.click(&app, at);
    messages.extend(ui.click(&app, at));
    assert!(zooms(&messages), "{messages:?}");
}

#[test]
fn revealed_hover_toolbar_moves_the_window() {
    let mut app = long_doc_app();
    let _ = app.update(Message::ToolbarHover(true));
    let mut ui = PointerUi::new();
    let at = blank_toolbar_point(&mut ui, &app);
    let messages = ui.click(&app, at);
    assert!(drags(&messages), "{messages:?}");
}

#[test]
fn sidebar_header_moves_the_window() {
    let (app, dir) = tree_fixture("window-drag-sidebar");
    let mut ui = PointerUi::new();
    let title = ui
        .text_containing(&app, "RMDV-TREE-WINDOW-DRAG-SIDEBAR")
        .and_then(|label| ui.text_bounds(&app, &label))
        .expect("workspace title");
    // The workspace title and the strip above it.
    for at in [
        center(title),
        Point::new(title.x + 4.0, (title.y - 6.0).max(1.0)),
    ] {
        let messages = ui.click(&app, at);
        assert!(drags(&messages), "{at:?}: {messages:?}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// A task that would reach the window (its stream exists) or none at all.
fn reaches_window(task: Task<Message>) -> bool {
    iced_runtime::task::into_stream(task).is_some()
}

#[test]
fn window_drag_and_zoom_act_only_outside_fullscreen() {
    let mut app = long_doc_app();
    assert!(reaches_window(app.update(Message::WindowDrag)));
    assert!(reaches_window(
        app.update(Message::WindowTitlebarDoubleClick)
    ));
    app.window_fullscreen = true;
    assert!(!reaches_window(app.update(Message::WindowDrag)));
    assert!(!reaches_window(
        app.update(Message::WindowTitlebarDoubleClick)
    ));
}

// --- 3. Window minimum size -----------------------------------------------------

#[test]
fn window_keeps_a_minimum_size() {
    for transparent in [false, true] {
        let settings = window_settings(transparent);
        assert_eq!(settings.min_size, Some(Size::new(640.0, 420.0)));
        assert!(
            settings.size.width >= 640.0 && settings.size.height >= 420.0,
            "the default size fits the minimum: {:?}",
            settings.size
        );
    }
}
