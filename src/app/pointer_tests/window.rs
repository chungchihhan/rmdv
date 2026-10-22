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
            *x < read.x - 20.0
                && !name.is_some_and(|n| *x >= n.x - 8.0 && *x <= n.x + n.width + 8.0)
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

// --- 2. Close protection --------------------------------------------------------

/// `long_doc_app` in edit mode with one typed character.
fn dirty_app() -> App {
    let mut app = long_doc_app();
    let _ = app.update(Message::ToggleViewMode);
    let _ = app.update(Message::EditorAction(
        iced::widget::text_editor::Action::Edit(iced::widget::text_editor::Edit::Insert('x')),
    ));
    assert!(app.dirty);
    app
}

#[test]
fn window_waits_for_the_app_before_closing() {
    assert!(!window_settings(false).exit_on_close_request);
}

#[test]
fn closing_a_clean_window_closes_at_once() {
    let mut app = long_doc_app();
    let task = app.update(Message::WindowCloseRequested(iced::window::Id::unique()));
    assert!(app.close_confirm.is_none());
    assert!(app.window_closing);
    assert!(reaches_window(task));
}

#[test]
fn closing_with_unsaved_edits_asks_first() {
    let mut app = dirty_app();
    let id = iced::window::Id::unique();
    let _ = app.update(Message::WindowCloseRequested(id));
    assert_eq!(app.close_confirm, Some(CloseConfirm { window: Some(id) }));
    assert!(!app.window_closing);

    // Cancel keeps the window and the edits.
    let _ = app.update(Message::CloseConfirmCancel);
    assert!(app.close_confirm.is_none());
    assert!(!app.window_closing);
    assert!(app.dirty);

    // Don't Save closes without writing.
    let _ = app.update(Message::WindowCloseRequested(id));
    let task = app.update(Message::CloseConfirmDiscard);
    assert!(app.close_confirm.is_none());
    assert!(app.window_closing);
    assert!(reaches_window(task));
}

#[test]
fn quit_with_unsaved_edits_asks_first() {
    let mut app = dirty_app();
    let _ = app.update(Message::QuitRequested);
    assert_eq!(app.close_confirm, Some(CloseConfirm { window: None }));
    assert!(!app.window_closing);

    let mut clean = long_doc_app();
    let _ = clean.update(Message::QuitRequested);
    assert!(clean.close_confirm.is_none());
    assert!(clean.window_closing);
}

#[test]
fn save_in_the_close_confirmation_closes_only_after_the_save_succeeds() {
    let mut app = dirty_app();
    let _ = app.update(Message::WindowCloseRequested(iced::window::Id::unique()));
    let _ = app.update(Message::CloseConfirmSave);
    assert!(app.close_confirm.is_none(), "the confirmation gives way");
    assert!(!app.window_closing, "the write has not landed yet");
    let saved_source = app.source.clone();

    let _ = app.update(Message::FileSaved {
        result: Ok(()),
        saved_source,
    });
    assert!(!app.dirty);
    assert!(app.window_closing, "a successful save closes the window");
}

#[test]
fn a_failed_save_keeps_the_window_open() {
    let mut app = dirty_app();
    let _ = app.update(Message::WindowCloseRequested(iced::window::Id::unique()));
    let _ = app.update(Message::CloseConfirmSave);
    let saved_source = app.source.clone();
    let _ = app.update(Message::FileSaved {
        result: Err("disk full".into()),
        saved_source: saved_source.clone(),
    });
    assert!(!app.window_closing);
    assert!(app.dirty);
    assert!(app.close_after_save.is_none(), "the close is dropped");

    // A later save the user makes on their own does not close the window.
    let _ = app.update(Message::FileSaved {
        result: Ok(()),
        saved_source,
    });
    assert!(!app.window_closing);
}

#[test]
fn close_confirmation_owns_the_keyboard() {
    use iced::keyboard::{key::Named, Key, Modifiers};
    let key = |key: Key, modifiers: Modifiers| close_confirm_key_message(&key, modifiers);
    assert!(matches!(
        key(Key::Named(Named::Escape), Modifiers::empty()),
        Message::CloseConfirmCancel
    ));
    assert!(matches!(
        key(Key::Named(Named::Enter), Modifiers::empty()),
        Message::CloseConfirmSave
    ));
    assert!(matches!(
        key(Key::Character("d".into()), Modifiers::COMMAND),
        Message::CloseConfirmDiscard
    ));
    assert!(matches!(
        key(Key::Character("d".into()), Modifiers::empty()),
        Message::Noop
    ));
    assert!(matches!(
        key(Key::Character("s".into()), Modifiers::COMMAND),
        Message::Noop
    ));

    // Typing does not reach the editor under the confirmation.
    let mut app = dirty_app();
    let _ = app.update(Message::WindowCloseRequested(iced::window::Id::unique()));
    let before = app.editor.as_ref().unwrap().text();
    let _ = app.update(Message::EditorAction(
        iced::widget::text_editor::Action::Edit(iced::widget::text_editor::Edit::Insert('y')),
    ));
    assert_eq!(app.editor.as_ref().unwrap().text(), before);
}

#[test]
fn close_confirmation_buttons_and_backdrop() {
    let mut app = dirty_app();
    let _ = app.update(Message::WindowCloseRequested(iced::window::Id::unique()));
    let mut ui = PointerUi::new();
    assert!(
        ui.text_containing(&app, "pointer-long.md").is_some(),
        "the confirmation names the file"
    );
    for (label, expect) in [
        ("Save", "CloseConfirmSave"),
        ("Don't Save", "CloseConfirmDiscard"),
        ("Cancel", "CloseConfirmCancel"),
    ] {
        let bounds = ui
            .text_bounds(&app, label)
            .unwrap_or_else(|| panic!("{label} button"));
        let messages = ui.click(&app, center(bounds));
        assert!(
            messages.iter().any(|m| format!("{m:?}") == expect),
            "{label}: {messages:?}"
        );
    }
    // The backdrop cancels; the card's blank space does not.
    let messages = ui.click(&app, Point::new(20.0, WINDOW.height - 20.0));
    assert!(
        has(&messages, |m| matches!(m, Message::CloseConfirmCancel)),
        "{messages:?}"
    );
    let title = ui
        .text_containing(&app, "Save changes")
        .and_then(|t| ui.text_bounds(&app, &t))
        .expect("title");
    let messages = ui.click(
        &app,
        Point::new(title.x + title.width + 30.0, title.y + 4.0),
    );
    assert!(messages.is_empty(), "{messages:?}");
}
