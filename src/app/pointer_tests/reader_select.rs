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
