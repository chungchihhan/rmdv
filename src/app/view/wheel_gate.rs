//! Takes some wheel events before the widget they would scroll
//! (MDV-048 P5).
//!
//! An Iced `scrollable` scrolls on every wheel event over it, and an
//! `event::listen` subscription cannot stop that. [`WheelGate`] wraps the
//! scrollable (or the editor) and sees each wheel event first:
//!
//! - [`Gate::ReaderZoom`]: ⌘+wheel changes the reader font size and does not
//!   also scroll the reader.

use crate::app::Message;
use iced::advanced::widget::{tree, Operation, Tree, Widget};
use iced::advanced::{layout, mouse, overlay, renderer, Clipboard, Layout, Shell};
use iced::keyboard;
use iced::{Element, Event, Length, Rectangle, Size, Vector};

/// What the gate does with the wheel events it takes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::app) enum Gate {
    /// ⌘+wheel publishes [`Message::ReaderZoomBy`].
    ReaderZoom,
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
    /// The message for a wheel event this gate takes, or `None` to let the
    /// content scroll.
    fn take_wheel(&self, state: &mut State, delta: mouse::ScrollDelta) -> Option<Message> {
        match self.gate {
            Gate::ReaderZoom => {
                if !state.modifiers.command() {
                    return None;
                }
                let steps = crate::app::gestures::wheel_zoom_steps(delta);
                (steps != 0.0).then_some(Message::ReaderZoomBy(steps))
            }
        }
    }
}

#[derive(Default)]
struct State {
    /// The keyboard modifiers now; wheel events do not carry them.
    modifiers: keyboard::Modifiers,
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
        let state = tree.state.downcast_mut::<State>();
        match event {
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.modifiers = *modifiers;
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta })
                if cursor.is_over(layout.bounds()) =>
            {
                if let Some(message) = self.take_wheel(state, *delta) {
                    shell.publish(message);
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
