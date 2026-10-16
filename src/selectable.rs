//! Selectable reader text.
//!
//! The reader builds every paragraph, heading, list item, quote, table cell,
//! and code block from `rich_text` spans. This module owns the widget those
//! spans go into, so the reader can switch between plain and selectable text
//! in one place.

use crate::app::Message;
use iced::{Color, Element};

/// A span of reader text. The link payload is the `Message` to dispatch.
pub type RtSpan<'a> = iced::advanced::text::Span<'a, Message, iced::Font>;

/// Selectable rich text whose link spans dispatch their carried `Message`
/// on click. Drag, double-click (word), and triple-click (line) select;
/// ⌘C copies the selection as plain text. `selection` fills the highlight.
pub fn rich_text_links<'a>(spans: Vec<RtSpan<'a>>, selection: Color) -> Element<'a, Message> {
    iced_selection::rich_text(spans)
        .on_link_click(|m| m)
        .style(move |_| iced_selection::text::Style {
            color: None,
            selection,
        })
        .into()
}

/// [`rich_text_links`] at a relative line height (body or heading).
pub fn rich_text_spaced<'a>(
    spans: Vec<RtSpan<'a>>,
    line_height: f32,
    selection: Color,
) -> Element<'a, Message> {
    iced_selection::rich_text(spans)
        .line_height(iced::widget::text::LineHeight::Relative(line_height))
        .on_link_click(|m| m)
        .style(move |_| iced_selection::text::Style {
            color: None,
            selection,
        })
        .into()
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
}
