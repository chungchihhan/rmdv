//! Takes some wheel events before the widget they would scroll
//! (MDV-048 P5).
//!
//! An Iced `scrollable` scrolls on every wheel event over it, and an
//! `event::listen` subscription cannot stop that. [`WheelGate`] wraps the
//! scrollable (or the editor) and sees each wheel event first:
//!
//! - [`Gate::Reader`]: ⌘+wheel changes the reader font size and does not
//!   also scroll the reader. On macOS the gate also tells the swipe bridge
//!   (`crate::native_swipe`) whether the pointer is over the reader and
//!   over nothing that scrolls horizontally ([`HScrollProbe`]).
//! - [`Gate::EditorScroll`]: a trackpad (pixel) scroll moves the editor by
//!   the finger distance. Iced's `text_editor` turns `y` pixels into `y / 4`
//!   lines, about five times the finger distance at a 20 px line. A mouse
//!   wheel (lines) keeps the editor's own speed.

use crate::app::Message;
use iced::advanced::widget::{tree, Operation, Tree, Widget};
use iced::advanced::{layout, mouse, overlay, renderer, Clipboard, Layout, Shell};
use iced::keyboard;
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector};

/// What the gate does with the wheel events it takes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::app) enum Gate {
    /// ⌘+wheel publishes [`Message::ReaderZoomBy`]; the pointer position
    /// arms a two-finger swipe.
    Reader,
    /// Pixel scrolls publish the editor's scroll action; `line_height` is
    /// the editor's line height in logical pixels.
    EditorScroll { line_height: f32 },
}

pub(in crate::app) struct WheelGate<'a> {
    content: Element<'a, Message>,
    gate: Gate,
}

impl<'a> WheelGate<'a> {
    pub(in crate::app) fn new(content: impl Into<Element<'a, Message>>, gate: Gate) -> Self {
        Self {
            content: content.into(),
            gate,
        }
    }
}

impl WheelGate<'_> {
    /// `None` lets the content scroll. `Some` takes the wheel event, with
    /// the message to publish if there is one.
    fn take_wheel(&self, state: &mut State, delta: mouse::ScrollDelta) -> Option<Option<Message>> {
        match self.gate {
            Gate::Reader => {
                if !state.modifiers.command() {
                    return None;
                }
                let steps = crate::app::gestures::wheel_zoom_steps(delta);
                (steps != 0.0).then_some(Some(Message::ReaderZoomBy(steps)))
            }
            Gate::EditorScroll { line_height } => match delta {
                // A mouse wheel keeps the editor's own speed.
                mouse::ScrollDelta::Lines { .. } => None,
                mouse::ScrollDelta::Pixels { y, .. } => {
                    let lines = crate::app::gestures::editor_scroll_lines(
                        &mut state.partial_lines,
                        y,
                        line_height,
                    );
                    // The gate takes every pixel scroll, also one that
                    // adds up to less than a line, so the editor never
                    // scrolls by its own pixel speed.
                    Some((lines != 0).then_some(Message::EditorAction(
                        iced::widget::text_editor::Action::Scroll { lines },
                    )))
                }
            },
        }
    }
}

impl WheelGate<'_> {
    /// Tell the swipe bridge whether a swipe may start at the pointer: over
    /// the reader and over no horizontal scroller. Only while a swipe is
    /// armed, so other platforms and other states pay nothing.
    fn sync_swipe_pointer(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
    ) {
        let moved = matches!(
            event,
            Event::Mouse(
                mouse::Event::CursorMoved { .. }
                    | mouse::Event::WheelScrolled { .. }
                    | mouse::Event::CursorLeft
            )
        );
        if !moved || crate::native_swipe::armed() == 0 {
            return;
        }
        let ok = match cursor.position_over(layout.bounds()) {
            Some(at) => {
                let mut probe = HScrollProbe::new(at);
                self.content.as_widget_mut().operate(
                    &mut tree.children[0],
                    child_layout(layout),
                    renderer,
                    &mut probe,
                );
                !probe.found()
            }
            None => false,
        };
        crate::native_swipe::set_pointer_ok(ok);
    }
}

/// Finds whether a point (in window coordinates) lies over a scrollable
/// whose content is wider than it, such as a wide table, so a two-finger
/// swipe there scrolls the table instead of going Back.
///
/// A scrollable reports its bounds in its parent's content coordinates and
/// then lays out its children shifted by its scroll offset (`translation`).
/// The probe keeps the sum of the offsets above the widget it visits, so
/// a table inside the scrolled reader is found where it is on screen.
pub(in crate::app) struct HScrollProbe {
    at: Point,
    found: bool,
    /// The scroll offset of every `traverse` level now open.
    offsets: Vec<Vector>,
    /// The offset the next `traverse` adds: a scrollable's translation, set
    /// right before it traverses its content; zero for any other widget.
    next: Vector,
}

impl HScrollProbe {
    pub(in crate::app) fn new(at: Point) -> Self {
        Self {
            at,
            found: false,
            offsets: Vec::new(),
            next: Vector::ZERO,
        }
    }

    pub(in crate::app) fn found(&self) -> bool {
        self.found
    }

    fn offset(&self) -> Vector {
        self.offsets.last().copied().unwrap_or(Vector::ZERO)
    }
}

impl Operation for HScrollProbe {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        if self.found {
            return;
        }
        let level = self.offset() + std::mem::replace(&mut self.next, Vector::ZERO);
        self.offsets.push(level);
        operate(self);
        self.offsets.pop();
    }

    fn container(&mut self, _id: Option<&iced::advanced::widget::Id>, _bounds: Rectangle) {
        self.next = Vector::ZERO;
    }

    fn scrollable(
        &mut self,
        _id: Option<&iced::advanced::widget::Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        _state: &mut dyn iced::advanced::widget::operation::Scrollable,
    ) {
        let at = self.at + self.offset();
        if content_bounds.width > bounds.width + 0.5 && bounds.contains(at) {
            self.found = true;
        }
        self.next = translation;
    }

    fn focusable(
        &mut self,
        _id: Option<&iced::advanced::widget::Id>,
        _bounds: Rectangle,
        _state: &mut dyn iced::advanced::widget::operation::Focusable,
    ) {
        self.next = Vector::ZERO;
    }

    fn text(&mut self, _id: Option<&iced::advanced::widget::Id>, _bounds: Rectangle, _text: &str) {
        self.next = Vector::ZERO;
    }
}

#[derive(Default)]
struct State {
    /// The keyboard modifiers now; wheel events do not carry them.
    modifiers: keyboard::Modifiers,
    /// The editor lines scrolled but not sent yet (less than one line).
    partial_lines: f32,
}

impl<'a> Widget<Message, iced::Theme, iced::Renderer> for WheelGate<'a> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let child = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        layout::Node::with_children(child.size(), vec![child])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            child_layout(layout),
            renderer,
            operation,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if self.gate == Gate::Reader {
            self.sync_swipe_pointer(tree, event, layout, cursor, renderer);
        }
        let state = tree.state.downcast_mut::<State>();
        match event {
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.modifiers = *modifiers;
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta })
                if cursor.is_over(layout.bounds()) =>
            {
                if let Some(message) = self.take_wheel(state, *delta) {
                    if let Some(message) = message {
                        shell.publish(message);
                    }
                    shell.capture_event();
                    return;
                }
            }
            _ => {}
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            child_layout(layout),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            child_layout(layout),
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            child_layout(layout),
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            child_layout(layout),
            renderer,
            viewport,
            translation,
        )
    }
}

fn child_layout(layout: Layout<'_>) -> Layout<'_> {
    layout.children().next().expect("WheelGate has one child")
}

impl<'a> From<WheelGate<'a>> for Element<'a, Message> {
    fn from(gate: WheelGate<'a>) -> Self {
        Element::new(gate)
    }
}
