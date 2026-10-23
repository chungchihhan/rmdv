//! Reader selection with the pointer (MDV-048 P7): a selection starts in
//! the margins and between blocks, a drag past the viewport edge scrolls,
//! double-click selects Chinese and Japanese words, and word, line, and
//! Shift+click selections reach across blocks. The selected text is read
//! back through the reader's right-press hit, which changes nothing.

use super::reader_menu::{block_bounds, drag, fixture, hit, paragraph};
use super::tree_menu::right_press;
use super::*;
use crate::ast::Block;

/// The text the reader reports as selected, read with a right press at
/// `probe` (a spot that does not change the selection).
fn selected(app: &mut App, ui: &mut PointerUi, probe: Point) -> Option<String> {
    let messages = right_press(app, ui, probe);
    let _ = app.update(Message::ContextMenuClose);
    hit(&messages).selection
}

fn heading(ui: &mut PointerUi, app: &App) -> Rectangle {
    block_bounds(ui, app, |b| matches!(b, Block::Heading { .. }))
}

#[test]
fn a_press_in_the_margin_starts_a_selection_at_the_nearest_text() {
    let (mut app, dir) = fixture("margin");
    let mut ui = PointerUi::new();
    let text = paragraph(&mut ui, &app);
    let y = text.y + 8.0;
    // The left margin beside the reading column, level with the paragraph.
    let margin = Point::new(24.0, y);
    assert!(margin.x < text.x - 40.0, "{margin:?} is left of {text:?}");
    drag(&mut app, &mut ui, margin, Point::new(text.x + 80.0, y));

    let got = selected(&mut app, &mut ui, Point::new(text.x + 30.0, y));
    let got = got.expect("the drag from the margin selected text");
    assert!(got.starts_with("Some"), "{got:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_press_between_blocks_starts_a_selection() {
    let (mut app, dir) = fixture("between");
    let mut ui = PointerUi::new();
    let head = heading(&mut ui, &app);
    let text = paragraph(&mut ui, &app);
    assert!(
        text.y > head.y + head.height,
        "a gap separates {head:?} and {text:?}"
    );
    let gap = Point::new(text.x + 40.0, (head.y + head.height + text.y) / 2.0);
    drag(
        &mut app,
        &mut ui,
        gap,
        Point::new(text.x + 80.0, text.y + 8.0),
    );

    let got = selected(&mut app, &mut ui, Point::new(text.x + 30.0, text.y + 8.0));
    let got = got.expect("the drag from between the blocks selected text");
    assert!(got.contains("Some"), "{got:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_margin_and_the_gaps_between_blocks_show_the_text_cursor() {
    let (app, dir) = fixture("ibeam");
    let mut ui = PointerUi::new();
    let head = heading(&mut ui, &app);
    let text = paragraph(&mut ui, &app);
    for at in [
        Point::new(24.0, text.y + 8.0),
        Point::new(text.x + 40.0, (head.y + head.height + text.y) / 2.0),
    ] {
        let (_, interaction) = ui.move_to(&app, at);
        assert_eq!(interaction, mouse::Interaction::Text, "{at:?}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_press_on_an_image_keeps_its_own_action_and_selects_nothing() {
    let (mut app, dir) = fixture("image-press");
    let mut ui = PointerUi::new();
    let image = block_bounds(&mut ui, &app, |b| matches!(b, Block::Image { .. }));
    let text = paragraph(&mut ui, &app);
    let at = center(image);
    let _ = ui.move_to(&app, at);
    let pressed = ui
        .send(
            &app,
            at,
            vec![Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            ))],
        )
        .0;
    assert!(
        has(&pressed, |m| matches!(m, Message::OpenImageZoom(_))),
        "{pressed:?}"
    );
    let mut messages = ui.move_to(&app, Point::new(text.x + 80.0, text.y + 8.0)).0;
    messages.extend(
        ui.send(
            &app,
            Point::new(text.x + 80.0, text.y + 8.0),
            vec![Event::Mouse(mouse::Event::ButtonReleased(
                mouse::Button::Left,
            ))],
        )
        .0,
    );
    // The zoom is not applied: the reader stays in front for the probe.
    apply(&mut app, messages);
    assert_eq!(
        selected(&mut app, &mut ui, Point::new(text.x + 30.0, text.y + 8.0)),
        None
    );
    let _ = std::fs::remove_dir_all(&dir);
}

// --- Drag-select auto-scroll ---------------------------------------------------

/// The bounds of "Paragraph `n`" in `long_doc_app`, when it is mounted.
fn long_paragraph(ui: &mut PointerUi, app: &App, n: usize) -> Option<Rectangle> {
    let prefix = format!("Paragraph {n} ");
    let id = app
        .ast
        .iter()
        .find(|(_, b)| crate::plain_text::block_text(b).is_some_and(|t| t.starts_with(&prefix)))
        .map(|(id, _)| *id)?;
    ui.find(
        app,
        Find {
            text: None,
            contains: None,
            id: Some(crate::render::block_anchor_id(id)),
            found: None,
            found_text: None,
        },
    )
    .map(|(bounds, _)| bounds)
}

fn press(ui: &mut PointerUi, app: &App, at: Point) -> Vec<Message> {
    let mut out = ui.move_to(app, at).0;
    out.extend(
        ui.send(
            app,
            at,
            vec![Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            ))],
        )
        .0,
    );
    out
}

fn release(ui: &mut PointerUi, app: &App, at: Point) -> Vec<Message> {
    ui.send(
        app,
        at,
        vec![Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Left,
        ))],
    )
    .0
}

fn autoscroll(messages: &[Message]) -> Vec<Option<f32>> {
    messages
        .iter()
        .filter_map(|m| match m {
            Message::ReaderAutoScroll(speed) => Some(*speed),
            _ => None,
        })
        .collect()
}

/// The number of the last whole "Paragraph N with" in `text`; the end of
/// a selection can cut a number short.
fn last_paragraph(text: &str) -> usize {
    text.match_indices("Paragraph ")
        .filter_map(|(i, _)| {
            let rest = &text[i + "Paragraph ".len()..];
            let (n, after) = rest.split_once(' ')?;
            after.starts_with("with").then(|| n.parse().ok()).flatten()
        })
        .last()
        .unwrap_or_else(|| panic!("no paragraph in {text:?}"))
}

#[test]
fn a_drag_past_the_viewport_edge_starts_the_scroll_and_coming_back_stops_it() {
    let app = long_doc_app();
    let mut ui = PointerUi::new();
    let text = long_paragraph(&mut ui, &app, 2).expect("paragraph 2 is on screen");
    let start = Point::new(text.x + 1.0, text.y + 8.0);
    let inside = Point::new(text.x + 80.0, 400.0);
    let below = Point::new(text.x + 80.0, WINDOW.height + 60.0);
    let above = Point::new(text.x + 80.0, -60.0);

    let _ = press(&mut ui, &app, start);
    let past = autoscroll(&ui.move_to(&app, below).0);
    assert!(
        matches!(past.as_slice(), [Some(dy)] if *dy > 0.0),
        "below the bottom edge the reader scrolls down: {past:?}"
    );
    // Farther past the edge scrolls faster.
    let farther = autoscroll(&ui.move_to(&app, Point::new(below.x, below.y + 120.0)).0);
    assert!(
        matches!((past[0], farther.as_slice()), (Some(a), [Some(b)]) if *b > a),
        "{past:?} then {farther:?}"
    );
    let back = autoscroll(&ui.move_to(&app, inside).0);
    assert_eq!(back, [None], "back inside, the scroll stops");
    let up = autoscroll(&ui.move_to(&app, above).0);
    assert!(
        matches!(up.as_slice(), [Some(dy)] if *dy < 0.0),
        "above the top edge the reader scrolls up: {up:?}"
    );
    assert_eq!(
        autoscroll(&release(&mut ui, &app, above)),
        [None],
        "releasing the button stops the scroll"
    );
    // A move without the button held starts nothing.
    assert!(autoscroll(&ui.move_to(&app, below).0).is_empty());
}

#[test]
fn the_scroll_tick_runs_only_while_a_drag_is_past_the_edge() {
    fn recipes(app: &App) -> usize {
        iced::advanced::subscription::into_recipes(app.subscription()).len()
    }
    let mut app = long_doc_app();
    let idle = recipes(&app);
    let _ = app.update(Message::ReaderAutoScroll(Some(12.0)));
    assert_eq!(app.reader_autoscroll, Some(12.0));
    assert!(
        recipes(&app) > idle,
        "the tick runs while the drag is past the edge"
    );
    let _ = app.update(Message::ReaderAutoScroll(None));
    assert_eq!(app.reader_autoscroll, None);
    assert_eq!(recipes(&app), idle, "no tick once the drag ends");

    // Only the reader scrolls this way: a mind map panel's text does not
    // move the document behind it.
    app.view_mode = ViewMode::Mindmap;
    let _ = app.update(Message::ReaderAutoScroll(Some(12.0)));
    assert_eq!(app.reader_autoscroll, None);
}

/// Like `long_doc_app`, with enough blocks that the reader mounts only a
/// window of them.
fn virtual_doc_app() -> App {
    let mut source = String::new();
    for i in 0..(crate::virt::VIRT_MIN_BLOCKS * 2) {
        source.push_str(&format!(
            "Paragraph {i} with enough words to fill a reader line here.\n\n"
        ));
    }
    let mut app = App::default();
    let _ = app.update(Message::FileLoaded(Ok((
        std::path::PathBuf::from("/tmp/pointer-virtual.md"),
        source,
    ))));
    app
}

#[test]
fn the_selection_follows_the_auto_scroll_across_window_rebuilds() {
    let mut app = virtual_doc_app();
    let mut ui = PointerUi::new();
    let text = long_paragraph(&mut ui, &app, 2).expect("paragraph 2 is on screen");
    // The last paragraph that starts on screen before the scroll.
    let on_screen = (2..60)
        .take_while(|&n| long_paragraph(&mut ui, &app, n).is_some_and(|b| b.y < WINDOW.height))
        .last()
        .expect("paragraphs fill the screen");
    let start = Point::new(text.x + 1.0, text.y + 8.0);
    let below = Point::new(text.x + 80.0, WINDOW.height + 200.0);

    let mut messages = press(&mut ui, &app, start);
    messages.extend(ui.move_to(&app, below).0);
    group2::apply_ui(&mut app, &mut ui, messages);
    assert!(app.reader_autoscroll.is_some_and(|dy| dy > 0.0));

    for _ in 0..40 {
        group2::apply_ui(&mut app, &mut ui, vec![Message::ReaderAutoScrollTick]);
        let redraw = Event::Window(iced::window::Event::RedrawRequested(
            std::time::Instant::now(),
        ));
        let messages = ui.send(&app, below, vec![redraw]).0;
        group2::apply_ui(&mut app, &mut ui, messages);
    }
    assert!(
        long_paragraph(&mut ui, &app, 2).is_none(),
        "the scroll moved paragraph 2 out of the mounted window"
    );
    let messages = release(&mut ui, &app, below);
    group2::apply_ui(&mut app, &mut ui, messages);
    assert_eq!(app.reader_autoscroll, None);

    let got = selected(&mut app, &mut ui, Point::new(text.x + 30.0, 400.0))
        .expect("the drag selected text");
    assert!(
        got.starts_with("Paragraph 2 "),
        "{:?}",
        &got[..80.min(got.len())]
    );
    let reached = last_paragraph(&got);
    assert!(
        reached > on_screen + 10,
        "the selection end followed the scroll: {reached} vs {on_screen} on screen before"
    );
}
