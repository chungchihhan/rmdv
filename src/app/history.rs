//! Navigation history: Back and Forward between files and `#fragment` jumps,
//! like a browser. Every file open records the stop it leaves, whatever
//! opened it (tree, finder, link, mind map, Full Mindmap, Quick Slot, recent
//! rows, IPC). A stop keeps its scroll position and reading surface, and
//! comes back through the Quick Slot scroll restore, so it lands right
//! under the virtual scroll window.

use super::*;

impl App {
    /// The stop on screen now: the open file, its reader's scroll position,
    /// and its reading surface (Zen edit counts as Read).
    fn history_here(&self) -> Option<HistoryEntry> {
        let path = self.file.clone()?;
        let mode = if self.view_mode == ViewMode::Mindmap {
            ViewMode::Mindmap
        } else {
            ViewMode::Rendered
        };
        Some(HistoryEntry {
            path,
            position: Self::slot_position(self.body_viewport.as_ref()),
            mode,
        })
    }

    /// A new navigation leaves the current stop: it goes on Back, and
    /// Forward no longer applies.
    pub(super) fn record_history(&mut self) {
        if let Some(here) = self.history_here() {
            NavHistory::push(&mut self.history.back, here);
        }
        self.history.forward.clear();
    }

    /// `apply_loaded_file` is about to replace the open file with `path`.
    /// A pending Back or Forward to `path` moves the stacks and returns the
    /// stop to restore; any other open records an ordinary navigation.
    pub(super) fn history_on_file_switch(&mut self, path: &Path) -> Option<HistoryEntry> {
        let here = self.history_here();
        if let Some((dir, target)) = self.history.pending.take() {
            let same = |entry: &HistoryEntry| {
                entry.path == path
                    || canonicalize_existing_path(entry.path.clone())
                        == canonicalize_existing_path(path.to_path_buf())
            };
            let (from, to) = self.history.stacks_mut(dir);
            let lands = from.last().is_some_and(|e| e.path == target && same(e));
            if lands {
                let entry = from.pop();
                if let Some(here) = here {
                    NavHistory::push(to, here);
                }
                return entry;
            }
        }
        self.record_history();
        None
    }

    /// A Back or Forward stop has loaded: show its surface and scroll the
    /// reader back to where it was once the new content reports its size
    /// (`BodyScrolled` applies `quick_slot_body_restore`).
    pub(super) fn restore_history_after_file(&mut self, entry: HistoryEntry) -> Task<Message> {
        self.view_mode = entry.mode;
        if entry.mode == ViewMode::Mindmap {
            self.mindmap_focus_first_child();
            return Task::none();
        }
        if entry.position <= 0.0 {
            return Task::none();
        }
        self.quick_slot_body_restore = Some(entry.position);
        // The body scrollable keeps its offset across files and reports a
        // viewport only when it changes. Coming back past a file too short
        // to scroll, the old viewport repeats exactly and no report would
        // come, so start from the top: that change always reports.
        iced::widget::operation::scroll_to(
            Self::scroll_id(),
            iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
        )
    }

    /// A stop in the open file (a `#fragment` jump): no reload, only the
    /// surface and the scroll position change.
    fn restore_history_here(&mut self, entry: HistoryEntry) -> Task<Message> {
        if self.view_mode == ViewMode::Raw || self.editor.is_some() {
            self.leave_zen_edit_mode(false);
        }
        self.view_mode = entry.mode;
        if entry.mode == ViewMode::Mindmap {
            return Task::none();
        }
        let max = self.body_viewport.as_ref().map_or(0.0, |viewport| {
            (viewport.content_bounds().height - viewport.bounds().height).max(0.0)
        });
        if max > 0.0 {
            self.quick_slot_body_restore = None;
            return self.quick_slot_restore_position(entry.position);
        }
        self.quick_slot_body_restore = Some(entry.position);
        Task::none()
    }

    /// A modal (an overlay, the close or replace confirmation) owns the
    /// pointer and keyboard; Back and Forward wait until it closes.
    pub(super) fn history_blocked(&self) -> bool {
        self.overlay != Overlay::None
            || self.close_confirm.is_some()
            || self.quick_slot_replace.is_some()
    }

    pub(super) fn can_go(&self, dir: HistoryDir) -> bool {
        !self.history.stack(dir).is_empty()
    }

    /// Back or Forward. Unsaved Zen edits block it with the Save toast;
    /// deleted files on the way are skipped with a toast.
    pub(super) fn history_go(&mut self, dir: HistoryDir) -> Task<Message> {
        if self.history_blocked() || !self.can_go(dir) {
            return Task::none();
        }
        if self.dirty {
            return self.show_unsaved_edits_toast(self.unsaved_edits_open_message());
        }
        let current = self.file.clone();
        let mut skipped: Vec<String> = Vec::new();
        let target = loop {
            let (from, _) = self.history.stacks_mut(dir);
            let Some(top) = from.last() else {
                break None;
            };
            if current.as_ref() == Some(&top.path) || top.path.is_file() {
                break Some(top.clone());
            }
            let missing = from.pop().map(|e| e.path).unwrap_or_default();
            skipped.push(
                missing
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| missing.display().to_string()),
            );
        };
        let mut tasks = Vec::new();
        if !skipped.is_empty() {
            let verb = if skipped.len() == 1 { "file" } else { "files" };
            tasks.push(self.show_toast(format!("Skipped missing {verb}: {}", skipped.join(", "))));
        }
        let Some(target) = target else {
            return Task::batch(tasks);
        };
        // Like a toolbar view switch: pages over the reader close first.
        self.settings_open = false;
        if self.vault_open {
            tasks.push(self.update(Message::VaultClose));
        }
        if self.full_mindmap.is_some() {
            tasks.push(self.exit_full_mindmap(false));
            if self.full_mindmap.is_some() {
                // The exit waits for a workspace refresh; the reader shows
                // first, and Back applies from there.
                return Task::batch(tasks);
            }
        }
        if current.as_ref() == Some(&target.path) {
            let here = self.history_here();
            let (from, to) = self.history.stacks_mut(dir);
            let entry = from.pop().unwrap_or(target);
            if let Some(here) = here {
                NavHistory::push(to, here);
            }
            tasks.push(self.restore_history_here(entry));
            return Task::batch(tasks);
        }
        let checkpoint = self.checkpoint_active_quick_slot();
        self.invalidate_pending_quick_slot_restore();
        self.history.pending = Some((dir, target.path.clone()));
        tasks.push(checkpoint);
        tasks.push(self.begin_generic_file_load(target.path));
        Task::batch(tasks)
    }

    /// Opening another workspace starts a new history.
    pub(super) fn clear_history(&mut self) {
        self.history = NavHistory::default();
    }
}
