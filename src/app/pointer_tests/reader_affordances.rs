//! Reader affordances (MDV-048 P7): copy buttons on code, math, and
//! diagram blocks show on hover and turn into a check after a copy; math
//! copies only from its button; diagrams say a click enlarges them; task
//! items draw read-only checkboxes.

use super::group3::{assert_tips, tips};
use super::reader_menu::block_bounds;
use super::*;
use crate::ast::{Block, DiagramKind};
use crate::render::draw_log;
use std::sync::Arc;

fn doc_app(tag: &str, source: &str) -> App {
    let mut app = App::default();
    let _ = app.update(Message::FileLoaded(Ok((
        PathBuf::from(format!("/tmp/pointer-affordance-{tag}.md")),
        source.to_string(),
    ))));
    app
}

/// Seeds a ready raster for the document's first diagram of `kind`.
fn seed_diagram(app: &mut App, kind: DiagramKind) -> (u64, String) {
    let (hash, source) = app
        .ast
        .iter()
        .find_map(|(_, block)| match block {
            Block::Diagram {
                hash,
                source,
                kind: k,
            } if *k == kind => Some((*hash, source.clone())),
            _ => None,
        })
        .expect("the diagram block");
    let (w, h) = (240u32, 60u32);
    app.diagram_cache.put(
        (hash, app.diagram_theme_id),
        crate::diagram::DiagramState::Ready {
            inline: iced::widget::image::Handle::from_rgba(w, h, vec![90; (w * h * 4) as usize]),
            source_bytes: Arc::new(Vec::new()),
            device_w: w,
        },
    );
    (hash, source)
}

fn glyph(code: char) -> String {
    code.to_string()
}

/// Moves the pointer to `at` and draws the view there.
fn draw_at(ui: &mut PointerUi, app: &App, at: Point) {
    let _ = ui.move_to(app, at);
    let cache = ui.cache.take().unwrap_or_default();
    let mut user_interface = UserInterface::build(app.view(), ui.size, cache, &mut ui.renderer);
    user_interface.draw(
        &mut ui.renderer,
        &iced::Theme::Light,
        &iced::advanced::renderer::Style {
            text_color: iced::Color::BLACK,
        },
        mouse::Cursor::Available(at),
    );
    ui.cache = Some(user_interface.into_cache());
}

/// The redraw the event loop schedules after the pointer moves to `to`.
fn redraw_after_move(ui: &mut PointerUi, app: &App, to: Point) -> iced::window::RedrawRequest {
    let cache = ui.cache.take().unwrap_or_default();
    let mut user_interface = UserInterface::build(app.view(), ui.size, cache, &mut ui.renderer);
    let mut messages = Vec::new();
    let (state, _) = user_interface.update(
        &[Event::Mouse(mouse::Event::CursorMoved { position: to })],
        mouse::Cursor::Available(to),
        &mut ui.renderer,
        &mut iced::advanced::clipboard::Null,
        &mut messages,
    );
    ui.cache = Some(user_interface.into_cache());
    match state {
        user_interface::State::Updated { redraw_request, .. } => redraw_request,
        user_interface::State::Outdated => iced::window::RedrawRequest::NextFrame,
    }
}

/// Bounds of every widget that reports custom state (the reader's rich
/// text reports its selection state this way, not its text).
fn inner_widgets(ui: &mut PointerUi, app: &App) -> Vec<Rectangle> {
    struct Customs(Vec<Rectangle>);
    impl Operation for Customs {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }
        fn custom(&mut self, _id: Option<&Id>, bounds: Rectangle, _state: &mut dyn std::any::Any) {
            self.0.push(bounds);
        }
    }
    let cache = ui.cache.take().unwrap_or_default();
    let mut user_interface = UserInterface::build(app.view(), ui.size, cache, &mut ui.renderer);
    let mut customs = Customs(Vec::new());
    user_interface.operate(&ui.renderer, &mut customs);
    ui.cache = Some(user_interface.into_cache());
    customs.0
}

fn copied(messages: &[Message]) -> Option<String> {
    messages.iter().find_map(|m| match m {
        Message::CopyBlockSource(text) => Some(text.clone()),
        _ => None,
    })
}

const CODE_DOC: &str = "Intro text above the block.\n\n```rust\nfn main() {}\n```\n\nOutro text.\n";

fn code_block(ui: &mut PointerUi, app: &App) -> Rectangle {
    block_bounds(ui, app, |b| matches!(b, Block::CodeBlock { .. }))
}

// --- Copy buttons -------------------------------------------------------------

#[test]
fn the_code_copy_button_shows_only_while_the_pointer_is_over_the_block() {
    let app = doc_app("code-hover", CODE_DOC);
    let mut ui = PointerUi::new();
    let code = code_block(&mut ui, &app);
    let outside = Point::new(code.center_x(), code.y - 30.0);
    let inside = Point::new(code.x + 40.0, code.center_y());

    let _ = draw_log::take_reveals();
    draw_at(&mut ui, &app, outside);
    assert_eq!(draw_log::take_reveals(), vec![false], "hidden away from it");
    draw_at(&mut ui, &app, inside);
    assert_eq!(draw_log::take_reveals(), vec![true], "shown over the block");

    // Entering the block redraws, so the button appears without a view
    // rebuild; moving inside it schedules nothing more.
    let _ = ui.move_to(&app, outside);
    assert_eq!(
        redraw_after_move(&mut ui, &app, inside),
        iced::window::RedrawRequest::NextFrame
    );
    let _ = redraw_after_move(&mut ui, &app, Point::new(inside.x + 5.0, inside.y));
    let still = redraw_after_move(&mut ui, &app, Point::new(inside.x + 10.0, inside.y));
    assert_eq!(still, iced::window::RedrawRequest::Wait);
}

#[test]
fn a_copy_turns_the_icon_into_a_check_until_its_timer_ends() {
    let mut app = doc_app("code-check", CODE_DOC);
    let mut ui = PointerUi::new();
    let copy = ui
        .text_bounds(&app, &glyph(crate::icon::ic::COPY))
        .expect("copy icon");
    let messages = ui.click(&app, center(copy));
    let text = copied(&messages).expect("the button copies the block's source");
    assert!(text.contains("fn main() {}"), "{text:?}");

    let _ = app.update(Message::CopyBlockSource(text));
    assert!(
        ui.text_bounds(&app, &glyph(crate::icon::ic::CHECK))
            .is_some(),
        "a check replaces the copy icon"
    );
    assert!(ui
        .text_bounds(&app, &glyph(crate::icon::ic::COPY))
        .is_none());
    assert!(
        app.toast.is_none(),
        "the check is the feedback, not a toast"
    );

    // An older copy's timer leaves this check alone; its own clears it.
    let seq = app.copy_flash.as_ref().map(|(seq, _)| *seq).expect("flash");
    let _ = app.update(Message::CopyFlashEnd(seq.wrapping_sub(1)));
    assert!(ui
        .text_bounds(&app, &glyph(crate::icon::ic::CHECK))
        .is_some());
    let _ = app.update(Message::CopyFlashEnd(seq));
    assert!(ui
        .text_bounds(&app, &glyph(crate::icon::ic::COPY))
        .is_some());
    assert!(app.copy_flash.is_none());
}

#[test]
fn the_copy_button_leaves_the_first_code_line_clear() {
    let words = "alpha beta gamma delta epsilon zeta eta theta iota kappa ".repeat(6);
    let app = doc_app("code-clear", &format!("```text\n{words}\n```\n"));
    let mut ui = PointerUi::new();
    let copy = ui
        .text_bounds(&app, &glyph(crate::icon::ic::COPY))
        .expect("copy icon");
    let block = code_block(&mut ui, &app);
    let text = inner_widgets(&mut ui, &app)
        .into_iter()
        .filter(|r| r.x >= block.x + 4.0 && block.contains(r.center()))
        .max_by(|a, b| a.width.total_cmp(&b.width))
        .expect("the code text inside the block");
    assert!(
        text.x + text.width <= copy.x - 4.0,
        "the code text {text:?} stays left of the button {copy:?}"
    );
}

#[test]
fn math_copies_only_from_its_button() {
    let mut app = doc_app("math", "Intro\n\n$$\nx^2 + y^2\n$$\n\nOutro\n");
    let (_, source) = seed_diagram(&mut app, DiagramKind::Math);
    let mut ui = PointerUi::new();
    assert_tips(&tips(&mut ui, &app), &["Copy Math"]);

    let math = block_bounds(&mut ui, &app, |b| matches!(b, Block::Diagram { .. }));
    let messages = ui.click(&app, center(math));
    assert!(
        !has(&messages, |m| matches!(
            m,
            Message::CopyCode(_) | Message::CopyBlockSource(_)
        )),
        "a click on the formula copies nothing: {messages:?}"
    );

    let copy = ui
        .text_bounds(&app, &glyph(crate::icon::ic::COPY))
        .expect("math copy button");
    assert!(math.contains(center(copy)), "{copy:?} sits on {math:?}");
    let messages = ui.click(&app, center(copy));
    assert_eq!(copied(&messages).as_deref(), Some(source.as_str()));
}

#[test]
fn diagrams_say_a_click_enlarges_them() {
    let mut app = doc_app("diagram", "Intro\n\n```mermaid\ngraph TD; A-->B\n```\n");
    let (hash, source) = seed_diagram(&mut app, DiagramKind::Mermaid);
    let mut ui = PointerUi::new();
    assert_tips(&tips(&mut ui, &app), &["Click to enlarge", "Copy Source"]);

    let diagram = block_bounds(&mut ui, &app, |b| matches!(b, Block::Diagram { .. }));
    let messages = ui.click(&app, Point::new(diagram.x + 30.0, diagram.center_y()));
    assert!(has(
        &messages,
        |m| matches!(m, Message::DiagramZoom(h) if *h == hash)
    ));

    let copy = ui
        .text_bounds(&app, &glyph(crate::icon::ic::COPY))
        .expect("diagram copy button");
    let messages = ui.click(&app, center(copy));
    assert_eq!(copied(&messages).as_deref(), Some(source.as_str()));
}

// --- Task checkboxes ---------------------------------------------------------

#[test]
fn task_items_draw_read_only_checkboxes() {
    let app = doc_app("tasks", "- [x] finished item\n- [ ] open item\n");
    let mut ui = PointerUi::new();
    assert!(ui.text_bounds(&app, "✓").is_none(), "no text check mark");
    assert!(ui.text_bounds(&app, "○").is_none(), "no text circle");
    assert!(
        ui.text_bounds(&app, &glyph(crate::icon::ic::CHECK))
            .is_some(),
        "the done item's box holds a check"
    );
    assert_tips(&tips(&mut ui, &app), &["Read-only"]);

    // The box sits at the left of the list's 28 px marker column, on the
    // first item's line.
    let list = block_bounds(&mut ui, &app, |b| matches!(b, Block::List { .. }));
    let checkbox = Point::new(list.x + 7.0, list.y + 12.0);
    let (_, cursor) = ui.move_to(&app, checkbox);
    assert_eq!(cursor, mouse::Interaction::NotAllowed);
    let messages = ui.click(&app, checkbox);
    assert!(
        !has(&messages, |m| matches!(m, Message::SaveFile)),
        "{messages:?}"
    );
}
