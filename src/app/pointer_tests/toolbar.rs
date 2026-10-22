//! The top toolbar (MDV-048 P2): its setting, hover reveal, controls, and
//! the dirty marks in the toolbar and the window title.

use super::*;
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
