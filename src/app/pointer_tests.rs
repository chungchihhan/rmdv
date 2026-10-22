//! Pointer (mouse and trackpad) behavior of overlays, lists, the vault page,
//! and the welcome page (MDV-048 P1). Each test builds `App::view` in a
//! headless tiny-skia `UserInterface`; no window opens.

use super::*;
use iced::advanced::widget::{Id, Operation};
use iced::mouse;
use iced::{Event, Point, Rectangle, Size};
use iced_runtime::user_interface::{self, Cache, UserInterface};

const WINDOW: Size = Size::new(1280.0, 800.0);

/// Feeds events to a fresh `UserInterface` built from the current
/// `App::view`, keeping widget state (hover, last cursor position) between
/// calls the way the event loop does.
struct PointerUi {
    renderer: iced::Renderer,
    cache: Option<Cache>,
}

/// Locates a widget's bounds by the text it shows or by its container id.
struct Find<'a> {
    text: Option<&'a str>,
    contains: Option<&'a str>,
    id: Option<Id>,
    found: Option<Rectangle>,
    found_text: Option<String>,
}

impl Operation for Find<'_> {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        if self.found.is_none() && self.id.is_some() && id == self.id.as_ref() {
            self.found = Some(bounds);
        }
    }

    fn text(&mut self, _id: Option<&Id>, bounds: Rectangle, text: &str) {
        if self.found.is_some() {
            return;
        }
        let hit =
            self.text.is_some_and(|t| t == text) || self.contains.is_some_and(|t| text.contains(t));
        if hit {
            self.found = Some(bounds);
            self.found_text = Some(text.to_string());
        }
    }
}

impl PointerUi {
    fn new() -> Self {
        use iced::advanced::renderer::Headless;
        let renderer = iced::futures::executor::block_on(<iced::Renderer as Headless>::new(
            iced::Font::DEFAULT,
            iced::Pixels(16.0),
            Some("tiny-skia"),
        ))
        .expect("headless tiny-skia renderer");
        Self {
            renderer,
            cache: None,
        }
    }

    /// One batch of events at `at`; returns the published messages and the
    /// resulting mouse interaction (cursor shape).
    fn send(
        &mut self,
        app: &App,
        at: Point,
        events: Vec<Event>,
    ) -> (Vec<Message>, mouse::Interaction) {
        let cache = self.cache.take().unwrap_or_default();
        let mut ui = UserInterface::build(app.view(), WINDOW, cache, &mut self.renderer);
        let mut messages = Vec::new();
        let mut clipboard = iced::advanced::clipboard::Null;
        let (state, _) = ui.update(
            &events,
            mouse::Cursor::Available(at),
            &mut self.renderer,
            &mut clipboard,
            &mut messages,
        );
        let interaction = match state {
            user_interface::State::Updated {
                mouse_interaction, ..
            } => mouse_interaction,
            user_interface::State::Outdated => mouse::Interaction::None,
        };
        self.cache = Some(ui.into_cache());
        (messages, interaction)
    }

    fn move_to(&mut self, app: &App, at: Point) -> (Vec<Message>, mouse::Interaction) {
        self.send(
            app,
            at,
            vec![Event::Mouse(mouse::Event::CursorMoved { position: at })],
        )
    }

    fn click(&mut self, app: &App, at: Point) -> Vec<Message> {
        let mut out = self.move_to(app, at).0;
        out.extend(
            self.send(
                app,
                at,
                vec![Event::Mouse(mouse::Event::ButtonPressed(
                    mouse::Button::Left,
                ))],
            )
            .0,
        );
        out.extend(
            self.send(
                app,
                at,
                vec![Event::Mouse(mouse::Event::ButtonReleased(
                    mouse::Button::Left,
                ))],
            )
            .0,
        );
        out
    }

    fn wheel(&mut self, app: &App, at: Point) -> Vec<Message> {
        let mut out = self.move_to(app, at).0;
        out.extend(
            self.send(
                app,
                at,
                vec![Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -120.0 },
                })],
            )
            .0,
        );
        out
    }

    fn find(&mut self, app: &App, mut find: Find<'_>) -> Option<(Rectangle, Option<String>)> {
        let cache = self.cache.take().unwrap_or_default();
        let mut ui = UserInterface::build(app.view(), WINDOW, cache, &mut self.renderer);
        ui.operate(&self.renderer, &mut find);
        self.cache = Some(ui.into_cache());
        find.found.map(|bounds| (bounds, find.found_text))
    }

    fn text_bounds(&mut self, app: &App, text: &str) -> Option<Rectangle> {
        self.find(
            app,
            Find {
                text: Some(text),
                contains: None,
                id: None,
                found: None,
                found_text: None,
            },
        )
        .map(|(bounds, _)| bounds)
    }

    fn text_containing(&mut self, app: &App, needle: &str) -> Option<String> {
        self.find(
            app,
            Find {
                text: None,
                contains: Some(needle),
                id: None,
                found: None,
                found_text: None,
            },
        )
        .and_then(|(_, text)| text)
    }

    fn id_bounds(&mut self, app: &App, id: &'static str) -> Option<Rectangle> {
        self.find(
            app,
            Find {
                text: None,
                contains: None,
                id: Some(Id::new(id)),
                found: None,
                found_text: None,
            },
        )
        .map(|(bounds, _)| bounds)
    }
}

/// Messages a task yields without waiting (for example `Task::done`, which
/// yields once before its value). Polls a few times with a no-op waker inside
/// a Tokio context, so pending timers are created but never awaited.
fn ready_outputs(task: Task<Message>) -> Vec<Message> {
    use iced::futures::task::{noop_waker, Context, Poll};
    use iced::futures::StreamExt;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("tokio runtime");
    let _guard = runtime.enter();
    let mut out = Vec::new();
    let Some(mut stream) = iced_runtime::task::into_stream(task) else {
        return out;
    };
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);
    for _ in 0..8 {
        match stream.poll_next_unpin(&mut cx) {
            Poll::Ready(Some(iced_runtime::Action::Output(message))) => out.push(message),
            Poll::Ready(Some(_)) | Poll::Pending => {}
            Poll::Ready(None) => break,
        }
    }
    out
}

/// Applies published messages and the messages their tasks yield at once
/// (one level, like `Task::done` follow-ups).
fn apply(app: &mut App, messages: Vec<Message>) {
    for message in messages {
        let task = app.update(message);
        for follow_up in ready_outputs(task) {
            let _ = app.update(follow_up);
        }
    }
}

fn has(messages: &[Message], pred: impl Fn(&Message) -> bool) -> bool {
    messages.iter().any(pred)
}

fn center(bounds: Rectangle) -> Point {
    bounds.center()
}

fn long_doc_app() -> App {
    let mut source = String::new();
    for i in 0..200 {
        source.push_str(&format!(
            "Paragraph {i} with enough words to fill a reader line here.\n\n"
        ));
    }
    let mut app = App::default();
    let _ = app.update(Message::FileLoaded(Ok((
        std::path::PathBuf::from("/tmp/pointer-long.md"),
        source,
    ))));
    app
}

fn command_index(app: &App, label: &str) -> usize {
    app.filtered_commands()
        .iter()
        .position(|(l, _, _)| *l == label)
        .expect("command label")
}

// --- 1. Palette and theme row clicks behave like Enter ----------------------

#[test]
fn palette_row_click_closes_overlay_and_runs_command() {
    let mut app = App::default();
    let _ = app.update(Message::OpenCommandPalette);
    let sidebar_before = app.sidebar_open;
    let mut ui = PointerUi::new();
    let row = ui
        .text_bounds(&app, "Toggle Sidebar  ⌘B")
        .expect("palette row is rendered");

    let messages = ui.click(&app, center(row));
    apply(&mut app, messages);

    assert_eq!(app.overlay, Overlay::None, "a row click closes the palette");
    assert_ne!(app.sidebar_open, sidebar_before, "the command still runs");
}

#[test]
fn theme_row_click_applies_theme_and_closes_overlay() {
    let mut app = App::default();
    let _ = app.update(Message::OpenThemePicker);
    let target = ThemePreset::Nord;
    assert_ne!(app.theme_preset, target);
    let mut ui = PointerUi::new();
    let row = ui
        .text_bounds(&app, target.label())
        .expect("theme row is rendered");

    let messages = ui.click(&app, center(row));
    apply(&mut app, messages);

    assert_eq!(app.overlay, Overlay::None, "a row click closes the picker");
    assert_eq!(app.theme_preset, target, "the clicked theme is applied");
}

// --- 2. Panels are opaque; the scrim blocks hover and wheel -----------------

#[test]
fn clicking_blank_panel_space_keeps_overlays_open() {
    // Shortcut sheet: the whole card is static text.
    let mut app = App::default();
    let _ = app.update(Message::ToggleShortcuts);
    let mut ui = PointerUi::new();
    let title = ui
        .text_bounds(&app, "Navigation")
        .expect("sheet group title");
    let messages = ui.click(&app, center(title));
    assert!(
        !has(&messages, |m| matches!(m, Message::CloseOverlay)),
        "a click on the shortcut sheet must not close it: {messages:?}"
    );

    // Palette: the list padding beside a row is panel space, not scrim.
    let mut app = App::default();
    let _ = app.update(Message::OpenCommandPalette);
    let mut ui = PointerUi::new();
    let row_text = ui
        .text_bounds(&app, "Open Settings  ⌘,")
        .expect("first palette row");
    // Row text starts 12 px inside the row; the list pads rows by 8 px.
    let blank = Point::new(row_text.x - 12.0 - 4.0, row_text.center_y());
    let messages = ui.click(&app, blank);
    assert!(
        !has(&messages, |m| matches!(m, Message::CloseOverlay)),
        "a click on blank palette space must not close it: {messages:?}"
    );

    // The scrim outside the panel still closes it.
    let messages = ui.click(&app, Point::new(20.0, 20.0));
    assert!(
        has(&messages, |m| matches!(m, Message::CloseOverlay)),
        "a click on the scrim closes the palette: {messages:?}"
    );
}

#[test]
fn clicking_quick_slot_confirm_card_keeps_it_open() {
    let mut app = App::default();
    app.quick_slot_replace = Some(QuickSlotReplace {
        index: 0,
        current: "a.md".into(),
        incoming: "b.md".into(),
    });
    let mut ui = PointerUi::new();
    let title = ui
        .text_bounds(&app, "Replace Quick Slot 1?")
        .expect("confirm title");
    let messages = ui.click(&app, center(title));
    assert!(
        !has(&messages, |m| matches!(m, Message::QuickSlotReplaceCancel)),
        "a click on the card must not cancel: {messages:?}"
    );
    let messages = ui.click(&app, Point::new(20.0, 20.0));
    assert!(has(&messages, |m| matches!(
        m,
        Message::QuickSlotReplaceCancel
    )));
}

#[test]
fn overlay_scrim_blocks_hover_and_wheel_to_the_reader() {
    let mut app = long_doc_app();
    let mut ui = PointerUi::new();
    // Reader text above the palette panel (the panel starts at y = 190).
    let over_text = ui
        .text_bounds(
            &app,
            "Paragraph 0 with enough words to fill a reader line here.",
        )
        .map(center)
        .unwrap_or(Point::new(320.0, 70.0));
    assert!(over_text.y < 180.0, "probe point must sit above the panel");
    let (_, reader_cursor) = ui.move_to(&app, over_text);

    let _ = app.update(Message::OpenCommandPalette);
    let mut ui = PointerUi::new();
    let (_, cursor) = ui.move_to(&app, over_text);
    assert_ne!(
        cursor, reader_cursor,
        "the reader's cursor shape must not show through the scrim"
    );
    let messages = ui.wheel(&app, over_text);
    assert!(
        !has(&messages, |m| matches!(m, Message::BodyScrolled(_))),
        "wheel over the scrim must not scroll the reader: {messages:?}"
    );

    // Scrollables inside the panel still scroll.
    let row = ui
        .text_bounds(&app, "Open Settings  ⌘,")
        .expect("first palette row");
    let messages = ui.wheel(&app, center(row));
    assert!(
        has(&messages, |m| matches!(m, Message::OverlayScrolled(_))),
        "the palette list keeps wheel scrolling: {messages:?}"
    );
}

// --- 3. Hover moves the selection only after real pointer motion ------------

#[test]
fn palette_hover_moves_selection_after_pointer_motion() {
    let mut app = App::default();
    let _ = app.update(Message::OpenCommandPalette);
    let label = "Find in Document  ⌘F";
    let target = command_index(&app, label);
    let mut ui = PointerUi::new();
    let row = ui.text_bounds(&app, label).expect("palette row");

    // The first pointer event after the overlay opens only sets a baseline:
    // a pointer resting where the list appears must not steal the selection.
    let messages = ui.move_to(&app, center(row)).0;
    apply(&mut app, messages);
    assert_eq!(app.overlay_selected, 0);

    let messages = ui
        .move_to(&app, Point::new(row.center_x() + 6.0, row.center_y()))
        .0;
    apply(&mut app, messages);
    assert_eq!(app.overlay_selected, target, "hover moved the selection");

    // Arrow keys move on; a still pointer does not take it back.
    let _ = app.update(Message::OverlayMove(1));
    assert_eq!(app.overlay_selected, target + 1);
    let messages = ui
        .move_to(&app, Point::new(row.center_x() + 6.0, row.center_y()))
        .0;
    apply(&mut app, messages);
    assert_eq!(app.overlay_selected, target + 1);
}

// --- 4. Folder picker "Open This Folder" ------------------------------------

#[test]
fn folder_picker_open_this_folder_button_opens_workspace() {
    let dir = std::env::temp_dir().join(format!("rmdv-pointer-picker-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("child")).unwrap();
    std::fs::write(dir.join("child").join("a.md"), "a").unwrap();

    let mut app = App::default();
    app.overlay = Overlay::FolderPicker;
    app.picker = Some(Picker::new(Some(dir.clone()), PickerMode::OpenAny, false));
    let mut ui = PointerUi::new();
    let button = ui
        .text_bounds(&app, "Open This Folder")
        .expect("picker footer has an Open This Folder button");
    let messages = ui.click(&app, center(button));
    assert!(has(&messages, |m| matches!(
        m,
        Message::PickerSelectFolderHere
    )));
    apply(&mut app, messages);

    assert_eq!(app.overlay, Overlay::None);
    assert!(app.picker.is_none());
    assert_eq!(
        app.workspace.as_deref(),
        Some(canonicalize_existing_path(dir.clone()).as_path())
    );
    let _ = std::fs::remove_dir_all(&dir);
}

// --- 5. Vault search close and search buttons --------------------------------

#[test]
fn vault_page_has_close_and_search_buttons() {
    let mut app = App::default();
    app.vault_open = true;
    app.vault_query = "needle".into();
    let mut ui = PointerUi::new();

    let search = ui
        .id_bounds(&app, "vault-search-button")
        .expect("vault header has a search button");
    let messages = ui.click(&app, center(search));
    assert!(
        has(&messages, |m| matches!(m, Message::VaultRunSearch)),
        "{messages:?}"
    );

    let close = ui
        .id_bounds(&app, "vault-close-button")
        .expect("vault header has a close button");
    let messages = ui.click(&app, center(close));
    assert!(
        has(&messages, |m| matches!(m, Message::VaultClose)),
        "{messages:?}"
    );
    apply(&mut app, messages);
    assert!(!app.vault_open);
}

// --- 6. File finder selection stays in the rendered rows ----------------------

fn finder_app(files: usize) -> App {
    let mut app = App::default();
    app.workspace = Some(std::path::PathBuf::from("/w"));
    app.workspace_files = (0..files)
        .map(|i| std::path::PathBuf::from(format!("/w/f{i:03}.md")))
        .collect();
    app.workspace_files_rev += 1;
    let _ = app.update(Message::OpenFileFinder);
    app
}

#[test]
fn file_finder_selection_stays_within_rendered_rows() {
    let mut app = finder_app(150);
    let _ = app.update(Message::OverlayMove(1000));
    assert_eq!(app.overlay_selected, 79, "only 80 rows are rendered");

    let mut ui = PointerUi::new();
    assert_eq!(
        ui.text_containing(&app, "keep typing to narrow").as_deref(),
        Some("70 more — keep typing to narrow")
    );

    // Past the 200-result cap the count is a lower bound.
    let app = finder_app(250);
    let mut ui = PointerUi::new();
    assert_eq!(
        ui.text_containing(&app, "keep typing to narrow").as_deref(),
        Some("120+ more — keep typing to narrow")
    );

    // Under the cap there is no hint.
    let app = finder_app(10);
    let mut ui = PointerUi::new();
    assert!(ui.text_containing(&app, "keep typing to narrow").is_none());
}

// --- 7. Welcome shortcut rows are buttons --------------------------------------

#[test]
fn welcome_shortcut_rows_send_their_shortcut_message() {
    let rows: [(&str, &str, Message); 9] = [
        ("Open Folder", "⌘O", Message::OpenFolderPicker),
        ("Find File in Workspace", "⌘P", Message::OpenFileFinder),
        ("Command Palette", "⌘⇧P", Message::OpenCommandPalette),
        ("Toggle Sidebar", "⌘B", Message::ToggleSidebar),
        ("Find in Document", "⌘F", Message::ToggleSearch),
        ("Cycle Theme", "⌘T", Message::ToggleTheme),
        ("Edit / Select Text", "⌘E", Message::ToggleViewMode),
        ("Fold to Level (then 0–6)", "⌘K", Message::FoldChordStart),
        ("Full Mindmap Mode", "⌘⇧M", Message::ToggleFullMindmap),
    ];
    let app = App::default();
    for (label, key, expected) in rows {
        let mut ui = PointerUi::new();
        assert!(
            ui.text_bounds(&app, key).is_some(),
            "the {key} hint stays visible"
        );
        let bounds = ui.text_bounds(&app, label).expect("welcome row label");
        let messages = ui.click(&app, center(bounds));
        let expected = format!("{expected:?}");
        assert!(
            messages.iter().any(|m| format!("{m:?}") == expected),
            "{label} must send {expected}, got {messages:?}"
        );
    }
}

#[test]
fn hover_selection_ignores_rows_that_scroll_under_a_still_pointer() {
    let mut app = App::default();
    let _ = app.update(Message::OpenCommandPalette);

    // Baseline: the first pointer report is not motion.
    let _ = app.update(Message::ListPointerMoved(Point::new(10.0, 10.0)));
    let _ = app.update(Message::OverlayHover(3));
    assert_eq!(app.overlay_selected, 0);

    let _ = app.update(Message::ListPointerMoved(Point::new(10.0, 14.0)));
    let _ = app.update(Message::OverlayHover(3));
    assert_eq!(app.overlay_selected, 3);

    // Arrow keys scroll the list; the row now under the still pointer
    // reports a hover without pointer motion.
    let _ = app.update(Message::OverlayMove(2));
    assert_eq!(app.overlay_selected, 5);
    let _ = app.update(Message::OverlayHover(4));
    assert_eq!(
        app.overlay_selected, 5,
        "a still pointer keeps the selection"
    );

    // Real motion takes it again.
    let _ = app.update(Message::ListPointerMoved(Point::new(10.0, 20.0)));
    let _ = app.update(Message::OverlayHover(4));
    assert_eq!(app.overlay_selected, 4);
}

#[test]
fn vault_hover_moves_cursor_only_after_pointer_motion() {
    let mut app = App::default();
    app.workspace = Some(std::path::PathBuf::from("/w"));
    let _ = app.update(Message::OpenVaultSearch);
    for line in 1..=3 {
        app.vault_results.push(crate::vault_search::VaultHit {
            path: std::path::PathBuf::from("/w/a.md"),
            line,
            col_start: 0,
            col_end: 1,
            context: Vec::new(),
        });
    }

    let _ = app.update(Message::ListPointerMoved(Point::new(5.0, 5.0)));
    let _ = app.update(Message::VaultHover(2));
    assert_eq!(app.vault_cursor, 0, "baseline report is not motion");

    let _ = app.update(Message::ListPointerMoved(Point::new(5.0, 9.0)));
    let _ = app.update(Message::VaultHover(2));
    assert_eq!(app.vault_cursor, 2);

    let _ = app.update(Message::VaultMove(-1));
    let _ = app.update(Message::VaultHover(2));
    assert_eq!(app.vault_cursor, 1, "a still pointer keeps the cursor");

    // A stale index past the visible hits is ignored.
    let _ = app.update(Message::ListPointerMoved(Point::new(5.0, 30.0)));
    let _ = app.update(Message::VaultHover(9));
    assert_eq!(app.vault_cursor, 1);
}

#[test]
fn file_finder_more_label_counts_hidden_results() {
    assert_eq!(file_finder_more_label(80), None);
    assert_eq!(
        file_finder_more_label(81).as_deref(),
        Some("1 more — keep typing to narrow")
    );
    assert_eq!(
        file_finder_more_label(FILE_FINDER_RESULT_CAP).as_deref(),
        Some("120+ more — keep typing to narrow")
    );
}

// --- Group 2: find bar, toasts, error card, edit focus, file drop -----------

mod group2 {
    use super::*;
    use crate::icon::ic;
    use iced::advanced::widget::operation::Focusable;
    use iced::keyboard;
    use std::time::Duration;

    /// Reports whether the focusable widget with `id` is focused.
    struct FocusProbe {
        id: Id,
        focused: Option<bool>,
    }

    impl Operation for FocusProbe {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }

        fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if id == Some(&self.id) {
                self.focused = Some(state.is_focused());
            }
        }
    }

    impl PointerUi {
        fn operate(&mut self, app: &App, operation: &mut dyn Operation) {
            let cache = self.cache.take().unwrap_or_default();
            let mut ui = UserInterface::build(app.view(), WINDOW, cache, &mut self.renderer);
            ui.operate(&self.renderer, operation);
            self.cache = Some(ui.into_cache());
        }

        fn focused(&mut self, app: &App, id: &'static str) -> Option<bool> {
            let mut probe = FocusProbe {
                id: Id::new(id),
                focused: None,
            };
            self.operate(app, &mut probe);
            probe.focused
        }

        fn type_char(&mut self, app: &App, c: char) -> Vec<Message> {
            let event = Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(c.to_string().into()),
                modified_key: keyboard::Key::Character(c.to_string().into()),
                physical_key: keyboard::key::Physical::Code(keyboard::key::Code::KeyX),
                location: keyboard::Location::Standard,
                modifiers: keyboard::Modifiers::default(),
                text: Some(c.to_string().into()),
                repeat: false,
            });
            self.send(app, Point::new(640.0, 400.0), vec![event]).0
        }
    }

    /// Messages and widget operations a task yields without waiting.
    fn drain(task: Task<Message>) -> (Vec<Message>, Vec<Box<dyn Operation>>) {
        use iced::futures::task::{noop_waker, Context, Poll};
        use iced::futures::StreamExt;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("tokio runtime");
        let _guard = runtime.enter();
        let mut messages = Vec::new();
        let mut operations = Vec::new();
        let Some(mut stream) = iced_runtime::task::into_stream(task) else {
            return (messages, operations);
        };
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);
        for _ in 0..8 {
            match stream.poll_next_unpin(&mut cx) {
                Poll::Ready(Some(iced_runtime::Action::Output(message))) => messages.push(message),
                Poll::Ready(Some(iced_runtime::Action::Widget(operation))) => {
                    operations.push(operation)
                }
                Poll::Ready(Some(_)) | Poll::Pending => {}
                Poll::Ready(None) => break,
            }
        }
        (messages, operations)
    }

    /// Like `apply`, and also runs the widget operations (focus, scroll) the
    /// tasks ask for on `ui`, as the runtime does after an update.
    fn apply_ui(app: &mut App, ui: &mut PointerUi, messages: Vec<Message>) {
        for message in messages {
            let (follow_ups, operations) = drain(app.update(message));
            for mut operation in operations {
                ui.operate(app, operation.as_mut());
            }
            for follow_up in follow_ups {
                let (_, operations) = drain(app.update(follow_up));
                for mut operation in operations {
                    ui.operate(app, operation.as_mut());
                }
            }
        }
    }

    fn glyph_bounds(ui: &mut PointerUi, app: &App, glyph: char) -> Rectangle {
        ui.text_bounds(app, &glyph.to_string())
            .unwrap_or_else(|| panic!("glyph {glyph:?} is rendered"))
    }

    // --- 1. Edit mode focuses the editor ------------------------------------

    #[test]
    fn entering_edit_mode_focuses_the_editor_so_typing_works() {
        let mut app = long_doc_app();
        let mut ui = PointerUi::new();
        let _ = ui.move_to(&app, Point::new(640.0, 400.0));

        apply_ui(&mut app, &mut ui, vec![Message::ToggleViewMode]);
        assert!(app.editor.is_some(), "⌘E enters edit mode");
        assert_eq!(ui.focused(&app, "zen-editor"), Some(true));

        let typed = ui.type_char(&app, 'x');
        assert!(
            has(&typed, |m| matches!(m, Message::EditorAction(_))),
            "a key typed right after ⌘E reaches the editor: {typed:?}"
        );
    }

    #[test]
    fn clicking_the_margin_beside_the_editor_focuses_it() {
        let mut app = long_doc_app();
        let mut ui = PointerUi::new();
        let _ = ui.move_to(&app, Point::new(640.0, 400.0));
        let _ = app.update(Message::ToggleViewMode);
        assert!(app.editor.is_some());
        // The editor column is READING_MAX wide and centered; x = 24 is in
        // the empty left margin.
        assert!(WINDOW.width - READING_MAX > 48.0);

        let messages = ui.click(&app, Point::new(24.0, 400.0));
        apply_ui(&mut app, &mut ui, messages);

        assert_eq!(ui.focused(&app, "zen-editor"), Some(true));
    }

    // --- 2. Find bar keeps the input focused after chevron clicks -----------

    #[test]
    fn find_chevron_clicks_return_focus_to_the_search_input() {
        let mut app = long_doc_app();
        let mut ui = PointerUi::new();
        let _ = ui.move_to(&app, Point::new(640.0, 400.0));
        apply_ui(
            &mut app,
            &mut ui,
            vec![
                Message::ToggleSearch,
                Message::QueryChanged("Paragraph".into()),
            ],
        );
        assert_eq!(ui.focused(&app, "search-input"), Some(true));

        for glyph in [ic::CHEVRON_RIGHT, ic::CHEVRON_LEFT] {
            let bounds = glyph_bounds(&mut ui, &app, glyph);
            let messages = ui.click(&app, center(bounds));
            assert!(
                has(&messages, |m| matches!(m, Message::FindStepClicked(_))),
                "{messages:?}"
            );
            apply_ui(&mut app, &mut ui, messages);
            assert_eq!(
                ui.focused(&app, "search-input"),
                Some(true),
                "after the {glyph:?} chevron, typing and Enter go to the input"
            );
        }
    }

    // --- 3. Toasts: Save action, longer errors, close button ----------------

    #[test]
    fn dirty_guard_toast_has_a_save_button() {
        let mut app = long_doc_app();
        app.dirty = true;
        let _ = app.update(Message::Open(PathBuf::from("/tmp/pointer-other.md")));
        assert!(app.toast.as_ref().is_some_and(|t| t.text.contains("⌘S")));

        let mut ui = PointerUi::new();
        let save = ui
            .text_bounds(&app, "Save")
            .expect("the guard toast has a Save button");
        let messages = ui.click(&app, center(save));
        assert!(
            has(&messages, |m| matches!(m, Message::SaveFile)),
            "{messages:?}"
        );
    }

    #[test]
    fn error_toast_stays_longer_and_has_a_close_button() {
        let mut app = long_doc_app();
        app.pending_clipboard_copy = Some("/tmp/x".into());
        let _ = app.update(Message::ClipboardCopyChecked {
            expected: "/tmp/x".into(),
            actual: None,
        });
        let toast = app.toast.clone().expect("error toast");
        assert_eq!(toast_duration(&toast), Duration::from_secs(8));

        let mut ui = PointerUi::new();
        let close = ui
            .id_bounds(&app, "toast-close")
            .expect("an error toast has a close button");
        let messages = ui.click(&app, center(close));
        apply(&mut app, messages);
        assert!(app.toast.is_none(), "the close button hides the toast");
    }

    #[test]
    fn info_toast_stays_short_without_a_close_button() {
        let mut app = long_doc_app();
        app.pending_clipboard_copy = Some("/tmp/x".into());
        let _ = app.update(Message::ClipboardCopyChecked {
            expected: "/tmp/x".into(),
            actual: Some("/tmp/x".into()),
        });
        let toast = app.toast.clone().expect("info toast");
        assert_eq!(toast_duration(&toast), Duration::from_millis(1500));
        let mut ui = PointerUi::new();
        assert!(ui.id_bounds(&app, "toast-close").is_none());
    }
}
