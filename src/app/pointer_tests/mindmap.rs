//! The mind map pointer model (MDV-048 P4): what a click, a chevron, and a
//! double click do in the document mind map and in Full Mindmap. The canvas
//! gesture itself is tested in `crate::mindmap`; these tests cover what the
//! app does with the messages the canvas publishes.

use super::group3::{full_mindmap_app, tree_fixture};
use super::*;

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
