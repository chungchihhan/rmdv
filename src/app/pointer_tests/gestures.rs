//! Reader and editor gestures (MDV-048 P5): pinch and ⌘+scroll change the
//! reader font by the ⌘+ and ⌘− steps, and a trackpad scrolls the editor by
//! the finger distance. The step math is tested in `super::super::gestures`;
//! these tests drive `App::update` and the headless UI.

use super::*;
use iced::keyboard;

fn command_wheel(ui: &mut PointerUi, app: &App, at: Point, y: f32) -> Vec<Message> {
    let mut out = ui.move_to(app, at).0;
    out.extend(
        ui.send(
            app,
            at,
            vec![
                Event::Keyboard(keyboard::Event::ModifiersChanged(
                    keyboard::Modifiers::COMMAND,
                )),
                Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Pixels { x: 0.0, y },
                }),
            ],
        )
        .0,
    );
    out
}

/// A point over the reading column, below the toolbar.
fn over_reader_text(ui: &mut PointerUi, app: &App) -> Point {
    let column = ui.id_bounds(app, "reader-column").expect("reader column");
    Point::new(column.center_x(), 300.0)
}

#[test]
fn command_wheel_over_the_reader_zooms_and_does_not_scroll() {
    let app = long_doc_app();
    let mut ui = PointerUi::new();
    let at = over_reader_text(&mut ui, &app);

    // A plain wheel scrolls, as before.
    let plain = ui.wheel(&app, at);
    assert!(
        has(&plain, |m| matches!(m, Message::BodyScrolled(_))),
        "a plain wheel scrolls the reader: {plain:?}"
    );

    let mut ui = PointerUi::new();
    let zoom = command_wheel(&mut ui, &app, at, 120.0);
    assert!(
        !has(&zoom, |m| matches!(m, Message::BodyScrolled(_))),
        "⌘+wheel must not scroll the reader: {zoom:?}"
    );
    assert!(
        has(
            &zoom,
            |m| matches!(m, Message::ReaderZoomBy(s) if *s > 1.9 && *s < 2.1)
        ),
        "⌘+wheel up asks for two larger steps (120 px): {zoom:?}"
    );

    // ⌘ released: the wheel scrolls again.
    let released = ui
        .send(
            &app,
            at,
            vec![
                Event::Keyboard(keyboard::Event::ModifiersChanged(
                    keyboard::Modifiers::empty(),
                )),
                Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -120.0 },
                }),
            ],
        )
        .0;
    assert!(
        !has(&released, |m| matches!(m, Message::ReaderZoomBy(_))),
        "without ⌘ the wheel does not zoom: {released:?}"
    );
}

#[test]
fn reader_zoom_steps_wait_for_the_flush_and_match_the_keys() {
    let mut app = long_doc_app();
    let mut keys = long_doc_app();
    let start = app.font_scale;
    for _ in 0..4 {
        let _ = app.update(Message::ReaderZoomBy(0.7));
    }
    assert_eq!(app.font_scale, start, "no font change before the flush");
    assert!(app.reader_zoom.flush_scheduled());
    let _ = app.update(Message::ReaderZoomFlush);
    let _ = keys.update(Message::FontSizeUp);
    let _ = keys.update(Message::FontSizeUp);
    assert_eq!(
        app.font_scale, keys.font_scale,
        "2.8 steps apply two ⌘+ steps at once"
    );
    assert!(app.typography.body_size > 0.0);
    assert!((app.reader_zoom.pending() - 0.8).abs() < 1e-4);

    // Down by one step, as ⌘−.
    let _ = app.update(Message::ReaderZoomBy(-1.2));
    let _ = app.update(Message::ReaderZoomFlush);
    let _ = keys.update(Message::FontSizeDown);
    assert!((app.font_scale - keys.font_scale).abs() < 1e-6);
}

#[test]
fn reader_zoom_stops_at_the_keyboard_limit() {
    let mut app = long_doc_app();
    for _ in 0..40 {
        let _ = app.update(Message::FontSizeUp);
    }
    let max = app.font_scale;
    let _ = app.update(Message::ReaderZoomBy(1.5));
    let _ = app.update(Message::ReaderZoomFlush);
    assert_eq!(app.font_scale, max, "the same limit as ⌘+");
    assert_eq!(app.reader_zoom.pending(), 0.0, "a limit ends the gesture");
}

#[test]
fn pinch_over_the_reader_collects_font_steps() {
    let mut app = long_doc_app();
    let _ = app.update(Message::MindmapNativePinch(0.25));
    assert!(
        (app.reader_zoom.pending() - 2.5).abs() < 1e-4,
        "a 0.25 pinch is 2.5 font steps: {}",
        app.reader_zoom.pending()
    );
    assert!(app.reader_zoom.flush_scheduled());
    assert_eq!(app.mindmap_native_pinch_log, 0.0);

    // An overlay owns the gesture.
    let mut app = long_doc_app();
    let _ = app.update(Message::OpenCommandPalette);
    let _ = app.update(Message::MindmapNativePinch(0.25));
    let _ = app.update(Message::ReaderZoomBy(3.0));
    assert_eq!(app.reader_zoom.pending(), 0.0);
    let start = app.font_scale;
    let _ = app.update(Message::ReaderZoomFlush);
    assert_eq!(app.font_scale, start);

    // The mind map keeps its own pinch.
    let mut app = long_doc_app();
    let _ = app.update(Message::ToggleMindmap);
    assert_eq!(app.view_mode, ViewMode::Mindmap);
    let _ = app.update(Message::MindmapNativePinch(0.25));
    assert_eq!(app.reader_zoom.pending(), 0.0);
    assert!((app.mindmap_native_pinch_log - 0.25).abs() < 1e-6);
}

fn editor_app() -> App {
    let mut app = long_doc_app();
    let _ = app.update(Message::ToggleViewMode);
    assert_eq!(app.view_mode, ViewMode::Raw);
    assert!(app.editor.is_some());
    app
}

fn scroll_lines(messages: &[Message]) -> Vec<i32> {
    messages
        .iter()
        .filter_map(|m| match m {
            Message::EditorAction(iced::widget::text_editor::Action::Scroll { lines }) => {
                Some(*lines)
            }
            _ => None,
        })
        .collect()
}

fn wheel_at(ui: &mut PointerUi, app: &App, at: Point, delta: mouse::ScrollDelta) -> Vec<Message> {
    let mut out = ui.move_to(app, at).0;
    out.extend(
        ui.send(
            app,
            at,
            vec![Event::Mouse(mouse::Event::WheelScrolled { delta })],
        )
        .0,
    );
    out
}

#[test]
fn trackpad_scrolls_the_editor_by_the_finger_distance() {
    let app = editor_app();
    let line = app.typography.code_size * 1.55;
    let at = Point::new(640.0, 400.0);
    let mut ui = PointerUi::new();
    let messages = wheel_at(
        &mut ui,
        &app,
        at,
        mouse::ScrollDelta::Pixels {
            x: 0.0,
            y: -3.0 * line,
        },
    );
    assert_eq!(
        scroll_lines(&messages),
        vec![3],
        "three lines of finger travel scroll three lines, not ~15: {messages:?}"
    );

    // Less than a line waits for more.
    let messages = wheel_at(
        &mut ui,
        &app,
        at,
        mouse::ScrollDelta::Pixels {
            x: 0.0,
            y: -0.4 * line,
        },
    );
    assert!(scroll_lines(&messages).is_empty(), "{messages:?}");
    let messages = wheel_at(
        &mut ui,
        &app,
        at,
        mouse::ScrollDelta::Pixels {
            x: 0.0,
            y: -0.7 * line,
        },
    );
    assert_eq!(scroll_lines(&messages), vec![1]);

    // A mouse wheel notch keeps the editor's own speed (4 lines).
    let messages = wheel_at(
        &mut ui,
        &app,
        at,
        mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 },
    );
    assert_eq!(scroll_lines(&messages), vec![4], "{messages:?}");
}
