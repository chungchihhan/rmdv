//! The top toolbar (MDV-048 P2): its setting, hover reveal, controls, and
//! the dirty marks in the toolbar and the window title.

use super::group3::{assert_tips, tips, tree_fixture};
use super::*;
use crate::icon::ic;
use crate::prefs::ToolbarMode;

fn settings_row_index(row: SettingsRow) -> usize {
    SettingsRow::visible()
        .iter()
        .position(|r| *r == row)
        .expect("row is visible on this platform")
}

// --- 1. Toolbar setting ------------------------------------------------------

#[test]
fn toolbar_setting_switches_between_hover_and_always_and_persists() {
    let mut app = App::default();
    let isolated = app.quick_slots_persistence_path.clone().unwrap();
    let saved = |path: &Path| crate::prefs::load_from(path).toolbar;
    assert_eq!(app.prefs.toolbar, ToolbarMode::OnHover);

    let _ = app.update(Message::OpenSettings);
    let _ = app.update(Message::SettingsCursor(settings_row_index(
        SettingsRow::Toolbar,
    )));
    // → picks Always, ← picks On hover, Space flips.
    let _ = app.update(Message::SettingsStep(1));
    assert_eq!(app.prefs.toolbar, ToolbarMode::Always);
    assert_eq!(saved(&isolated), ToolbarMode::Always);
    let _ = app.update(Message::SettingsStep(1));
    assert_eq!(app.prefs.toolbar, ToolbarMode::Always);
    let _ = app.update(Message::SettingsStep(-1));
    assert_eq!(app.prefs.toolbar, ToolbarMode::OnHover);
    assert_eq!(saved(&isolated), ToolbarMode::OnHover);
    let _ = app.update(Message::SettingsActivate);
    assert_eq!(app.prefs.toolbar, ToolbarMode::Always);
    let _ = app.update(Message::SettingsActivate);
    assert_eq!(app.prefs.toolbar, ToolbarMode::OnHover);
    assert_eq!(saved(&isolated), ToolbarMode::OnHover);
}

#[test]
fn toolbar_setting_row_has_a_clickable_segmented_control() {
    let mut app = App::default();
    let _ = app.update(Message::OpenSettings);
    let mut ui = PointerUi::new();
    // Bring the row into view the way the keyboard does.
    let mut at = ui.text_bounds(&app, "Always");
    for _ in 0..20 {
        if at.is_some_and(|b| b.y > 40.0 && b.y + b.height < 760.0) {
            break;
        }
        let _ = ui.wheel(&app, Point::new(640.0, 400.0));
        at = ui.text_bounds(&app, "Always");
    }
    let always = at.expect("the Toolbar row shows an Always segment");
    assert!(ui.text_bounds(&app, "On hover").is_some(), "and On hover");
    let messages = ui.click(&app, center(always));
    assert!(
        has(&messages, |m| matches!(
            m,
            Message::SetToolbarMode(ToolbarMode::Always)
        )),
        "{messages:?}"
    );
    apply(&mut app, messages);
    assert_eq!(app.prefs.toolbar, ToolbarMode::Always);
}

// --- 5. Window title marks unsaved edits -------------------------------------

/// `long_doc_app` in edit mode with one typed character.
fn dirty_app() -> App {
    let mut app = long_doc_app();
    let _ = app.update(Message::ToggleViewMode);
    let _ = app.update(Message::EditorAction(
        iced::widget::text_editor::Action::Edit(iced::widget::text_editor::Edit::Insert('x')),
    ));
    assert!(app.dirty, "typing makes the file dirty");
    app
}

#[test]
fn window_title_shows_a_bullet_while_the_file_is_dirty() {
    let clean = long_doc_app();
    assert_eq!(clean.title(), "rmdv — pointer-long.md");
    assert_eq!(dirty_app().title(), "rmdv — pointer-long.md •");
    assert_eq!(App::default().title(), "rmdv");
}

// --- 2. Toolbar controls (Always mode) ------------------------------------------

fn always(mut app: App) -> App {
    let _ = app.update(Message::SetToolbarMode(ToolbarMode::Always));
    app
}

fn glyph(code: char) -> String {
    code.to_string()
}

fn glyph_at(ui: &mut PointerUi, app: &App, code: char) -> Option<Rectangle> {
    ui.text_bounds(app, &glyph(code))
}

#[test]
fn always_toolbar_shows_file_name_dirty_dot_and_named_controls() {
    let app = always(dirty_app());
    let mut ui = PointerUi::new();
    let name = ui
        .text_bounds(&app, "pointer-long.md")
        .expect("the toolbar names the file");
    assert!(name.y < TOOLBAR_H, "in the top strip: {name:?}");
    let dot = ui
        .id_bounds(&app, "toolbar-dirty-dot")
        .expect("a dirty dot beside the name");
    assert!(dot.x > name.x + name.width - 1.0, "after the name");
    for label in ["Read", "Edit", "Mind Map"] {
        let bounds = ui
            .text_bounds(&app, label)
            .unwrap_or_else(|| panic!("{label} segment"));
        assert!(bounds.y < TOOLBAR_H, "{label} in the top strip");
    }
    for code in [ic::SAVE, ic::SEARCH, ic::COMMAND, ic::SETTINGS, ic::NETWORK] {
        let bounds = glyph_at(&mut ui, &app, code).expect("toolbar icon");
        assert!(bounds.y < TOOLBAR_H, "{code:?} in the top strip");
    }
    assert_tips(
        &tips(&mut ui, &app),
        &[
            "Read  Esc",
            "Edit  ⌘E",
            "Mind Map  ⌘M",
            "Full Mindmap  ⌘⇧M",
            "Save  ⌘S",
            "Find  ⌘F",
            "Command Palette  ⌘⇧P",
            "Settings  ⌘,",
        ],
    );

    // A clean file shows no dot.
    let clean = always(long_doc_app());
    let mut ui = PointerUi::new();
    assert!(ui.text_bounds(&clean, "pointer-long.md").is_some());
    assert!(ui.id_bounds(&clean, "toolbar-dirty-dot").is_none());
}

#[test]
fn always_toolbar_buttons_send_their_actions() {
    // Save works only while the file is dirty.
    let clean = always(long_doc_app());
    let mut ui = PointerUi::new();
    let save = glyph_at(&mut ui, &clean, ic::SAVE).expect("Save");
    let messages = ui.click(&clean, center(save));
    assert!(
        !has(&messages, |m| matches!(m, Message::SaveFile)),
        "Save is disabled on a clean file: {messages:?}"
    );
    let dirty = always(dirty_app());
    let mut ui = PointerUi::new();
    let save = glyph_at(&mut ui, &dirty, ic::SAVE).expect("Save");
    let messages = ui.click(&dirty, center(save));
    assert!(has(&messages, |m| matches!(m, Message::SaveFile)));

    let mut app = always(long_doc_app());
    let mut ui = PointerUi::new();
    let find = glyph_at(&mut ui, &app, ic::SEARCH).expect("Find");
    let messages = ui.click(&app, center(find));
    apply(&mut app, messages);
    assert!(app.search_open, "Find opens the find bar");

    let mut app = always(long_doc_app());
    let mut ui = PointerUi::new();
    let palette = glyph_at(&mut ui, &app, ic::COMMAND).expect("Command Palette");
    let messages = ui.click(&app, center(palette));
    apply(&mut app, messages);
    assert_eq!(app.overlay, Overlay::Command);

    let mut app = always(long_doc_app());
    let mut ui = PointerUi::new();
    let settings = glyph_at(&mut ui, &app, ic::SETTINGS).expect("Settings");
    let messages = ui.click(&app, center(settings));
    apply(&mut app, messages);
    assert!(app.settings_open);

    let mut ui = PointerUi::new();
    for (label, view) in [
        ("Read", ToolbarView::Read),
        ("Edit", ToolbarView::Edit),
        ("Mind Map", ToolbarView::Mindmap),
    ] {
        let bounds = ui.text_bounds(&app, label).expect("segment");
        let messages = ui.click(&app, center(bounds));
        assert!(
            has(
                &messages,
                |m| matches!(m, Message::ToolbarView(v) if *v == view)
            ),
            "{label}: {messages:?}"
        );
    }
}

#[test]
fn toolbar_sidebar_toggle_shows_only_while_the_sidebar_is_hidden() {
    let (app, dir) = tree_fixture("toolbar-sidebar");
    let mut app = always(app);
    let alpha = dir.join("alpha.md");
    let _ = app.update(Message::FileLoaded(Ok((alpha, "# Alpha\n".into()))));
    assert!(app.sidebar_open);
    let mut ui = PointerUi::new();
    assert!(
        glyph_at(&mut ui, &app, ic::PANEL_LEFT_OPEN).is_none(),
        "the sidebar header has its own Hide button"
    );
    let _ = app.update(Message::ToggleSidebar);
    let mut ui = PointerUi::new();
    let show = glyph_at(&mut ui, &app, ic::PANEL_LEFT_OPEN).expect("Show Sidebar");
    if cfg!(target_os = "macos") {
        assert!(show.x > 70.0, "clear of the traffic lights: {show:?}");
    }
    assert!(show.y < TOOLBAR_H);
    assert_tips(&tips(&mut ui, &app), &["Show Sidebar  ⌘B"]);
    let messages = ui.click(&app, center(show));
    apply(&mut app, messages);
    assert!(app.sidebar_open, "the toolbar button opens the sidebar");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn always_toolbar_reserves_its_height_without_moving_the_text() {
    let hover = long_doc_app();
    let mut ui = PointerUi::new();
    let before = ui
        .id_bounds(&hover, "reader-column")
        .expect("reading column");
    let fixed = always(long_doc_app());
    let mut ui = PointerUi::new();
    let after = ui
        .id_bounds(&fixed, "reader-column")
        .expect("reading column");
    assert_eq!(before.y, after.y, "switching modes does not move the text");
    // The reader scrolls below the toolbar, not behind it.
    let toolbar = ui.id_bounds(&fixed, "toolbar").expect("toolbar");
    assert_eq!(toolbar.y, 0.0);
    assert_eq!(toolbar.height, TOOLBAR_H);
    let hover_toolbar = PointerUi::new().id_bounds(&hover, "toolbar");
    assert!(hover_toolbar.is_none(), "On hover starts hidden");
}

// --- 3. Hover reveal -----------------------------------------------------------

fn hover_count(messages: &[Message], entered: bool) -> usize {
    messages
        .iter()
        .filter(|m| matches!(m, Message::ToolbarHover(e) if *e == entered))
        .count()
}

/// Waits for the messages a task yields, timers included.
fn wait_outputs(task: Task<Message>) -> Vec<Message> {
    use iced::futures::StreamExt;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("tokio runtime");
    let Some(stream) = iced_runtime::task::into_stream(task) else {
        return Vec::new();
    };
    runtime.block_on(
        stream
            .filter_map(|action| async move {
                match action {
                    iced_runtime::Action::Output(message) => Some(message),
                    _ => None,
                }
            })
            .collect::<Vec<_>>(),
    )
}

#[test]
fn hover_toolbar_appears_at_the_top_edge_and_hides_after_a_delay() {
    let mut app = long_doc_app();
    let mut ui = PointerUi::new();
    let (messages, _) = ui.move_to(&app, Point::new(640.0, 400.0));
    assert_eq!(hover_count(&messages, true), 0);
    assert!(ui.id_bounds(&app, "toolbar").is_none(), "hidden at first");

    // Entering the top band publishes one message and shows the toolbar.
    let (messages, _) = ui.move_to(&app, Point::new(640.0, 30.0));
    assert_eq!(hover_count(&messages, true), 1, "{messages:?}");
    apply(&mut app, messages);
    let bar = ui.id_bounds(&app, "toolbar").expect("revealed");
    assert_eq!(bar.y, 0.0);
    // Moving inside the band and over the toolbar publishes nothing more.
    for x in [600.0, 500.0, 300.0] {
        let (messages, _) = ui.move_to(&app, Point::new(x, 12.0));
        assert!(
            !has(&messages, |m| matches!(m, Message::ToolbarHover(_))),
            "no message on plain moves: {messages:?}"
        );
    }

    // Leaving publishes one message; the toolbar stays for the delay.
    let (messages, _) = ui.move_to(&app, Point::new(640.0, 400.0));
    assert_eq!(hover_count(&messages, false), 1, "{messages:?}");
    let mut hide = Vec::new();
    for message in messages {
        hide.extend(wait_outputs(app.update(message)));
    }
    assert!(
        ui.id_bounds(&app, "toolbar").is_some(),
        "stays during the delay"
    );
    assert!(
        has(&hide, |m| matches!(m, Message::ToolbarHideAfter(_))),
        "the delay ends in a hide message: {hide:?}"
    );
    apply(&mut app, hide);
    assert!(
        ui.id_bounds(&app, "toolbar").is_none(),
        "hidden after the delay"
    );
}

#[test]
fn returning_to_the_band_cancels_the_pending_hide() {
    let mut app = long_doc_app();
    let _ = app.update(Message::ToolbarHover(true));
    let _ = app.update(Message::ToolbarHover(false));
    let stale = app.toolbar_hover_generation;
    let _ = app.update(Message::ToolbarHover(true));
    let _ = app.update(Message::ToolbarHideAfter(stale));
    assert!(app.toolbar_revealed, "a stale hide does nothing");
    let _ = app.update(Message::ToolbarHover(false));
    let current = app.toolbar_hover_generation;
    let _ = app.update(Message::ToolbarHideAfter(current));
    assert!(!app.toolbar_revealed);
}

#[test]
fn hidden_band_leaves_the_content_its_pointer() {
    let app = long_doc_app();
    let mut ui = PointerUi::new();
    // Scroll text under the band, then probe its height.
    for _ in 0..3 {
        let _ = ui.wheel(&app, Point::new(640.0, 400.0));
    }
    let text_under_band = (1..20).any(|step| {
        let (_, cursor) = ui.move_to(&app, Point::new(640.0, step as f32 * 2.0));
        cursor == mouse::Interaction::Text
    });
    assert!(
        text_under_band,
        "the reader keeps its text cursor under the band"
    );
}

#[test]
fn revealed_toolbar_takes_presses_on_its_blank_space() {
    let mut app = long_doc_app();
    let mut ui = PointerUi::new();
    // Text scrolled under the band shows the text cursor there.
    for _ in 0..3 {
        let _ = ui.wheel(&app, Point::new(640.0, 400.0));
    }
    let probes: Vec<Point> = (1..16)
        .map(|step| Point::new(400.0 + step as f32 * 8.0, 16.0))
        .collect();
    let under: Vec<Point> = probes
        .iter()
        .copied()
        .filter(|at| ui.move_to(&app, *at).1 == mouse::Interaction::Text)
        .collect();
    assert!(!under.is_empty(), "text lies under the band");
    let _ = app.update(Message::ToolbarHover(true));
    let bar = ui.id_bounds(&app, "toolbar").expect("revealed");
    let name = ui.text_bounds(&app, "pointer-long.md").expect("name");
    for at in under {
        if at.x > name.x - 4.0 && at.x < name.x + name.width + 4.0 {
            continue;
        }
        assert!(bar.contains(at));
        let (_, cursor) = ui.move_to(&app, at);
        assert_ne!(
            cursor,
            mouse::Interaction::Text,
            "the reader's text cursor does not show through the toolbar at {at:?}"
        );
    }
}

#[test]
fn always_mode_has_no_hover_band() {
    let app = always(long_doc_app());
    let mut ui = PointerUi::new();
    let _ = ui.move_to(&app, Point::new(640.0, 400.0));
    let (messages, _) = ui.move_to(&app, Point::new(640.0, 10.0));
    assert!(!has(&messages, |m| matches!(m, Message::ToolbarHover(_))));
}
