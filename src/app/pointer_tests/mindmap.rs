//! The mind map pointer model (MDV-048 P4): what a click, a chevron, and a
//! double click do in the document mind map and in Full Mindmap. The canvas
//! gesture itself is tested in `crate::mindmap`; these tests cover what the
//! app does with the messages the canvas publishes.

use super::group3::{assert_tips, full_mindmap_app, tips, tree_fixture};
use super::*;
use crate::icon::ic;
use crate::mindmap::ZoomCommand;

#[test]
fn a_full_mindmap_click_selects_and_opens_the_panel() {
    let (mut app, dir) = full_mindmap_app("p4-click-panel");
    assert!(!app.full_mindmap.as_ref().unwrap().panel_open);

    // Keyboard selection leaves the panel closed, as before.
    let _ = app.update(Message::FullMindmapNavigate(MindmapDir::Right));
    assert!(!app.full_mindmap.as_ref().unwrap().panel_open);

    let folder = WorkspaceNodeId::Folder(dir.join("sub"));
    let expanded_before = app.full_mindmap.as_ref().unwrap().expanded.clone();
    let _ = app.update(Message::FullMindmapPointerSelect(folder.clone()));
    let full = app.full_mindmap.as_ref().unwrap();
    assert!(full.panel_open, "a click opens the panel");
    assert_eq!(full.selected.as_ref(), Some(&folder));
    assert_eq!(
        full.expanded, expanded_before,
        "a click on a folder does not fold it"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_document_mindmap_branch_click_opens_the_panel_without_folding() {
    let (mut app, dir) = tree_fixture("p4-doc-branch");
    let _ = app.update(Message::FileLoaded(Ok((
        dir.join("alpha.md"),
        "# Top\n\n## Child\n\ntext\n".into(),
    ))));
    let _ = app.update(Message::ToggleMindmap);
    let (nodes, _, _) = app.mindmap_layout();
    let branch = nodes
        .iter()
        .find(|node| !node.children.is_empty() && node.id.is_some())
        .and_then(|node| node.id)
        .expect("a branch heading");
    let _ = app.update(Message::MindmapSelectLeaf(branch));
    assert!(app.mindmap_panel_open);
    assert_eq!(app.mindmap_selected, Some(branch));
    assert_eq!(app.mindmap_panel_shown, Some(branch));
    assert!(!app.mindmap_collapsed.contains(&branch));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_full_mindmap_double_click_does_what_enter_does() {
    // A file opens, as Enter does, even when the first click's selection
    // has not landed.
    let (mut app, dir) = full_mindmap_app("p4-activate-file");
    let file = dir.join("alpha.md");
    let _ = app.update(Message::FullMindmapActivateNode(WorkspaceNodeId::File(
        file.clone(),
    )));
    let full = app.full_mindmap.as_ref().unwrap();
    assert_eq!(full.selected, Some(WorkspaceNodeId::File(file.clone())));
    assert_eq!(
        full.pending_open.as_ref().map(|open| open.path.as_path()),
        Some(file.as_path()),
        "the file opens"
    );
    let _ = std::fs::remove_dir_all(&dir);

    // A folder becomes the root.
    let (mut app, dir) = full_mindmap_app("p4-activate-folder");
    let folder = dir.join("sub");
    let _ = app.update(Message::FullMindmapPointerSelect(WorkspaceNodeId::Folder(
        folder.clone(),
    )));
    let _ = app.update(Message::FullMindmapActivateNode(WorkspaceNodeId::Folder(
        folder.clone(),
    )));
    assert_eq!(
        app.full_mindmap
            .as_ref()
            .and_then(|full| full.pending_workspace_load.as_ref())
            .map(|load| load.path.as_path()),
        Some(folder.as_path())
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn one_click_on_more_shows_the_next_page() {
    let dir = std::env::temp_dir().join(format!("rmdv-p4-more-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let dir = canonicalize_existing_path(dir);
    let folder = |name: &str| crate::tree::Node {
        path: dir.join(name),
        name: name.into(),
        is_dir: true,
        children: Vec::new(),
        recursive_supported_file_count: Some(crate::tree::RecursiveFileCount::Exact(1)),
    };
    let mut app = App::default();
    app.set_workspace(dir.clone(), false);
    app.full_mindmap = Some(App::new_full_mindmap_state());
    {
        let full = app.full_mindmap.as_mut().unwrap();
        full.expanded.insert(dir.clone());
        full.materialized_folders.insert(
            dir.clone(),
            crate::workspace_mindmap::MaterializedFolder::Loaded {
                folders: std::sync::Arc::new(vec![folder("a"), folder("b")]),
                files: std::sync::Arc::new(Vec::new()),
                recursive_supported_file_count: crate::tree::RecursiveFileCount::Exact(2),
                truncated: false,
                shown: 1,
            },
        );
    }
    let more = WorkspaceNodeId::Status(
        dir.clone(),
        crate::workspace_mindmap::WorkspaceStatus::MoreItems,
    );
    assert!(app.full_mindmap_graph().unwrap().node(&more).is_some());

    let _ = app.update(Message::FullMindmapPointerSelect(more.clone()));
    let graph = app.full_mindmap_graph().unwrap();
    assert!(graph.node(&more).is_none(), "the page is shown");
    assert!(graph
        .node(&WorkspaceNodeId::Folder(dir.join("b")))
        .is_some());
    let _ = std::fs::remove_dir_all(&dir);
}

/// The tree fixture with a two-heading document open in the mind map.
fn document_mindmap_app(tag: &str) -> (App, PathBuf) {
    let (mut app, dir) = tree_fixture(tag);
    let _ = app.update(Message::FileLoaded(Ok((
        dir.join("alpha.md"),
        "# Top\n\n## Child\n\ntext\n".into(),
    ))));
    let _ = app.update(Message::ToggleMindmap);
    assert_eq!(app.view_mode, ViewMode::Mindmap);
    (app, dir)
}

#[test]
fn zoom_buttons_send_their_command_and_leave_the_canvas_its_events() {
    let (mut app, dir) = document_mindmap_app("p4-zoom-buttons");
    assert!(
        app.mindmap_selected.is_some(),
        "the first heading is selected"
    );
    let mut ui = PointerUi::new();
    assert_tips(
        &tips(&mut ui, &app),
        &["Zoom to Fit", "Zoom Out  −", "Actual Size  0", "Zoom In  ="],
    );

    let plus = ui
        .text_bounds(&app, &ic::PLUS.to_string())
        .expect("zoom in button");
    let messages = ui.click(&app, center(plus));
    assert!(
        has(&messages, |m| matches!(
            m,
            Message::MindmapZoom(ZoomCommand::In)
        )),
        "{messages:?}"
    );
    assert!(
        !has(&messages, |m| matches!(m, Message::MindmapDeselect)),
        "the canvas under the button sees no click"
    );
    apply(&mut app, messages);
    assert_eq!(app.mindmap_zoom_request.generation, 1);
    assert_eq!(app.mindmap_zoom_request.command, Some(ZoomCommand::In));
    let fit = ui
        .text_bounds(&app, &ic::SCAN.to_string())
        .expect("zoom to fit button");
    let messages = ui.click(&app, center(fit));
    apply(&mut app, messages);
    assert_eq!(app.mindmap_zoom_request.command, Some(ZoomCommand::Fit));
    assert_eq!(app.mindmap_zoom_request.generation, 2);
    let reset = ui.text_bounds(&app, "100%").expect("100% button");
    let messages = ui.click(&app, center(reset));
    apply(&mut app, messages);
    assert_eq!(app.mindmap_zoom_request.command, Some(ZoomCommand::Reset));

    // Just above the buttons the canvas still owns the pointer: it shows
    // its grab cursor and an empty-canvas click clears the selection.
    let probe = Point::new(plus.center_x(), plus.y - 40.0);
    let (_, cursor) = ui.move_to(&app, probe);
    assert_eq!(cursor, mouse::Interaction::Grab);
    let messages = ui.click(&app, probe);
    assert!(
        has(&messages, |m| matches!(m, Message::MindmapDeselect)),
        "{messages:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn full_mindmap_zoom_buttons_drive_their_own_canvas() {
    let (mut app, dir) = full_mindmap_app("p4-fmm-zoom");
    let mut ui = PointerUi::new();
    let minus = ui
        .text_bounds(&app, &ic::MINUS.to_string())
        .expect("zoom out button");
    let messages = ui.click(&app, center(minus));
    apply(&mut app, messages);
    assert_eq!(
        app.full_mindmap_zoom_request.command,
        Some(ZoomCommand::Out)
    );
    assert_eq!(
        app.mindmap_zoom_request,
        crate::mindmap::ZoomRequest::default(),
        "the document map's zoom is left alone"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
