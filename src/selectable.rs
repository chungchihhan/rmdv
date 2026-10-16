//! Selectable reader text.
//!
//! The reader builds every paragraph, heading, list item, quote, table cell,
//! and code block from `rich_text` spans. This module owns the widget those
//! spans go into, so the reader can switch between plain and selectable text
//! in one place.

use crate::app::Message;
use iced::widget::text::LineHeight;
use iced::{Color, Element};

/// A span of reader text. The link payload is the `Message` to dispatch.
pub type RtSpan<'a> = iced::advanced::text::Span<'a, Message, iced::Font>;

/// Selectable rich text whose link spans dispatch their carried `Message`
/// on click. Drag, double-click (word), and triple-click (line) select;
/// ⌘C copies the selection as plain text. `selection` fills the highlight.
pub fn rich_text_links<'a>(spans: Vec<RtSpan<'a>>, selection: Color) -> Element<'a, Message> {
    selectable(spans, None, selection)
}

/// [`rich_text_links`] at a relative line height (body or heading).
pub fn rich_text_spaced<'a>(
    spans: Vec<RtSpan<'a>>,
    line_height: f32,
    selection: Color,
) -> Element<'a, Message> {
    selectable(spans, Some(LineHeight::Relative(line_height)), selection)
}

fn selectable<'a>(
    spans: Vec<RtSpan<'a>>,
    line_height: Option<LineHeight>,
    selection: Color,
) -> Element<'a, Message> {
    // The highlight rows follow the widget's size and line height, not the
    // spans'. Reader blocks use one size per block, so take it from the spans.
    let size = spans.iter().find_map(|s| s.size);
    let line_height = line_height.or_else(|| spans.iter().find_map(|s| s.line_height));
    let mut text = iced_selection::rich_text(spans)
        .on_link_click(|m| m)
        .style(move |_| iced_selection::text::Style {
            color: None,
            selection,
        });
    if let Some(size) = size {
        text = text.size(size);
    }
    if let Some(line_height) = line_height {
        text = text.line_height(line_height);
    }
    text.into()
}

/// True when a reader text widget consumed this ⌘C to copy its selection.
pub fn copied_by_reader(event: &iced::Event, status: iced::event::Status) -> bool {
    use iced::keyboard::{Event, Key};
    status == iced::event::Status::Captured
        && matches!(
            event,
            iced::Event::Keyboard(Event::KeyPressed { key: Key::Character(c), modifiers, .. })
                if c.as_str() == "c" && modifiers.command()
        )
}

#[cfg(test)]
mod tests {
    use super::{copied_by_reader, rich_text_links};
    use crate::app::Message;
    use iced::advanced::clipboard::{Clipboard, Kind};
    use iced::advanced::renderer::Headless;
    use iced::keyboard::{self, Key, Modifiers};
    use iced::mouse;
    use iced::widget::span;
    use iced::{Event, Font, Pixels, Point, Size};
    use iced_runtime::user_interface::{Cache, UserInterface};

    #[derive(Default)]
    struct RecordingClipboard(Option<String>);

    impl Clipboard for RecordingClipboard {
        fn read(&self, _kind: Kind) -> Option<String> {
            self.0.clone()
        }

        fn write(&mut self, _kind: Kind, contents: String) {
            self.0 = Some(contents);
        }
    }

    fn headless_renderer() -> iced::Renderer {
        iced::futures::executor::block_on(<iced::Renderer as Headless>::new(
            Font::DEFAULT,
            Pixels(16.0),
            Some("tiny-skia"),
        ))
        .expect("headless tiny-skia renderer")
    }

    fn key_press(c: &str, modifiers: Modifiers) -> Event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: Key::Character(c.into()),
            modified_key: Key::Character(c.into()),
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers,
            text: None,
            repeat: false,
        })
    }

    /// Feeds `events` one batch at a time, the way the event loop does, with
    /// the cursor at each batch's position.
    fn run(
        element: iced::Element<'_, Message>,
        batches: Vec<(Point, Vec<Event>)>,
    ) -> (RecordingClipboard, Vec<Message>) {
        let mut renderer = headless_renderer();
        let mut clipboard = RecordingClipboard::default();
        let mut messages = Vec::new();
        let mut ui = UserInterface::build(
            element,
            Size::new(800.0, 200.0),
            Cache::default(),
            &mut renderer,
        );
        for (at, events) in batches {
            let _ = ui.update(
                &events,
                mouse::Cursor::Available(at),
                &mut renderer,
                &mut clipboard,
                &mut messages,
            );
        }
        (clipboard, messages)
    }

    fn moved(p: Point) -> Event {
        Event::Mouse(mouse::Event::CursorMoved { position: p })
    }

    #[test]
    fn drag_then_cmd_c_copies_the_selected_plain_text() {
        let el = rich_text_links(
            vec![span("Hello "), span("world").underline(true)],
            iced::Color::BLACK,
        );
        let start = Point::new(1.0, 8.0);
        let end = Point::new(700.0, 8.0);
        let (clipboard, _) = run(
            el,
            vec![
                (start, vec![moved(start)]),
                (
                    start,
                    vec![Event::Mouse(mouse::Event::ButtonPressed(
                        mouse::Button::Left,
                    ))],
                ),
                (end, vec![moved(end)]),
                (
                    end,
                    vec![Event::Mouse(mouse::Event::ButtonReleased(
                        mouse::Button::Left,
                    ))],
                ),
                (
                    end,
                    vec![
                        Event::Keyboard(keyboard::Event::ModifiersChanged(Modifiers::COMMAND)),
                        key_press("c", Modifiers::COMMAND),
                    ],
                ),
            ],
        );
        assert_eq!(clipboard.0.as_deref(), Some("Hello world"));
    }

    #[test]
    fn clicking_a_link_without_dragging_still_dispatches_it() {
        let el = rich_text_links(vec![span("go").link(Message::Noop)], iced::Color::BLACK);
        let at = Point::new(4.0, 8.0);
        let (clipboard, messages) = run(
            el,
            vec![
                (at, vec![moved(at)]),
                (
                    at,
                    vec![Event::Mouse(mouse::Event::ButtonPressed(
                        mouse::Button::Left,
                    ))],
                ),
                (
                    at,
                    vec![Event::Mouse(mouse::Event::ButtonReleased(
                        mouse::Button::Left,
                    ))],
                ),
            ],
        );
        assert!(matches!(messages.as_slice(), [Message::Noop]));
        assert_eq!(clipboard.0, None);
    }

    #[test]
    fn only_a_captured_cmd_c_counts_as_a_reader_copy() {
        use iced::event::Status;
        let cmd_c = key_press("c", Modifiers::COMMAND);
        assert!(copied_by_reader(&cmd_c, Status::Captured));
        assert!(!copied_by_reader(&cmd_c, Status::Ignored));
        assert!(!copied_by_reader(
            &key_press("c", Modifiers::empty()),
            Status::Captured
        ));
        assert!(!copied_by_reader(
            &key_press("e", Modifiers::COMMAND),
            Status::Captured
        ));
    }

    fn press_release(at: Point) -> Vec<(Point, Vec<Event>)> {
        vec![
            (at, vec![moved(at)]),
            (
                at,
                vec![Event::Mouse(mouse::Event::ButtonPressed(
                    mouse::Button::Left,
                ))],
            ),
            (
                at,
                vec![Event::Mouse(mouse::Event::ButtonReleased(
                    mouse::Button::Left,
                ))],
            ),
        ]
    }

    fn heading(block: &crate::ast::Block) -> iced::Element<'_, Message> {
        let pal = crate::theme::palette_for(crate::theme::ThemePreset::OneDark);
        crate::render::render_heading_with_chevron(
            crate::ast::BlockId(7),
            block,
            &pal,
            &crate::theme::Typography::DEFAULT,
            "",
            None,
            false,
            true,
            0.0,
        )
    }

    fn heading_block() -> crate::ast::Block {
        crate::ast::Block::Heading {
            level: 2,
            id: "title".into(),
            inlines: vec![crate::ast::Inline::Text("Title".into())],
        }
    }

    fn folds(messages: &[Message]) -> bool {
        messages
            .iter()
            .any(|m| matches!(m, Message::ToggleFold(id) if id.0 == 7))
    }

    #[test]
    fn clicking_heading_text_selects_instead_of_folding() {
        let block = heading_block();
        let (_, messages) = run(heading(&block), press_release(Point::new(6.0, 14.0)));
        assert!(!folds(&messages), "text click must not fold: {messages:?}");
    }

    #[test]
    fn clicking_right_of_heading_text_still_folds() {
        let block = heading_block();
        let (_, messages) = run(heading(&block), press_release(Point::new(500.0, 14.0)));
        assert!(
            folds(&messages),
            "empty-space click must fold: {messages:?}"
        );
    }

    #[test]
    fn dragging_across_a_heading_copies_its_text() {
        let block = heading_block();
        let (start, end) = (Point::new(1.0, 14.0), Point::new(400.0, 14.0));
        let (clipboard, _) = run(
            heading(&block),
            vec![
                (start, vec![moved(start)]),
                (
                    start,
                    vec![Event::Mouse(mouse::Event::ButtonPressed(
                        mouse::Button::Left,
                    ))],
                ),
                (end, vec![moved(end)]),
                (
                    end,
                    vec![Event::Mouse(mouse::Event::ButtonReleased(
                        mouse::Button::Left,
                    ))],
                ),
                (
                    end,
                    vec![
                        Event::Keyboard(keyboard::Event::ModifiersChanged(Modifiers::COMMAND)),
                        key_press("c", Modifiers::COMMAND),
                    ],
                ),
            ],
        );
        assert_eq!(clipboard.0.as_deref(), Some("Title"));
    }

    #[test]
    fn dragging_across_the_data_view_copies_its_text() {
        let pal = crate::theme::palette_for(crate::theme::ThemePreset::OneDark);
        let code = "{\"a\": 1}";
        let (start, end) = (Point::new(33.0, 40.0), Point::new(600.0, 40.0));
        let (clipboard, _) = run(
            crate::render::data_view(code, &[], &pal, &crate::theme::Typography::DEFAULT),
            vec![
                (start, vec![moved(start)]),
                (
                    start,
                    vec![Event::Mouse(mouse::Event::ButtonPressed(
                        mouse::Button::Left,
                    ))],
                ),
                (end, vec![moved(end)]),
                (
                    end,
                    vec![Event::Mouse(mouse::Event::ButtonReleased(
                        mouse::Button::Left,
                    ))],
                ),
                (
                    end,
                    vec![
                        Event::Keyboard(keyboard::Event::ModifiersChanged(Modifiers::COMMAND)),
                        key_press("c", Modifiers::COMMAND),
                    ],
                ),
            ],
        );
        assert_eq!(clipboard.0.as_deref(), Some(code));
    }

    #[test]
    fn selection_highlight_covers_the_whole_line_of_large_text() {
        // Heading spans are larger than the widget's default size. The
        // highlight must follow the spans' line height, not the default.
        use super::rich_text_spaced;
        let red = iced::Color::from_rgb(1.0, 0.0, 0.0);
        let el = rich_text_spaced(vec![span("Title Title Title").size(40.0)], 1.3, red);
        let size = Size::new(400.0, 120.0);
        let mut renderer = headless_renderer();
        let mut clipboard = RecordingClipboard::default();
        let mut messages = Vec::new();
        let mut ui = UserInterface::build(el, size, Cache::default(), &mut renderer);
        let (start, end) = (Point::new(1.0, 10.0), Point::new(390.0, 10.0));
        for (at, events) in [
            (start, vec![moved(start)]),
            (
                start,
                vec![Event::Mouse(mouse::Event::ButtonPressed(
                    mouse::Button::Left,
                ))],
            ),
            (end, vec![moved(end)]),
            (
                end,
                vec![Event::Mouse(mouse::Event::ButtonReleased(
                    mouse::Button::Left,
                ))],
            ),
        ] {
            let _ = ui.update(
                &events,
                mouse::Cursor::Available(at),
                &mut renderer,
                &mut clipboard,
                &mut messages,
            );
        }
        ui.draw(
            &mut renderer,
            &iced::Theme::Light,
            &iced::advanced::renderer::Style {
                text_color: iced::Color::BLACK,
            },
            mouse::Cursor::Unavailable,
        );
        let rgba = renderer.screenshot(iced::Size::new(400, 120), 1.0, iced::Color::WHITE);
        let red_at = |x: usize, y: usize| {
            let i = (y * 400 + x) * 4;
            rgba[i] > 200 && rgba[i + 1] < 80 && rgba[i + 2] < 80
        };
        // Line height is 40 * 1.3 = 52 px. Count highlight pixels in a row
        // near the top and a row near the bottom of the line.
        let red_in_row = |y: usize| (0..380).filter(|&x| red_at(x, y)).count();
        assert!(red_in_row(4) > 20, "top of the line is highlighted");
        assert!(red_in_row(48) > 20, "bottom of the line is highlighted");
    }
}
