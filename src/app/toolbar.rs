//! Toolbar state: the hover reveal and the view switch.

use super::*;

impl App {
    /// The pointer entered or left the hover toolbar's band. Entering shows
    /// the toolbar at once; leaving hides it after a short delay, which a
    /// return to the band cancels.
    pub(super) fn toolbar_hover(&mut self, entered: bool) -> Task<Message> {
        self.toolbar_hovered = entered;
        self.toolbar_hover_generation = self.toolbar_hover_generation.wrapping_add(1);
        if entered {
            self.toolbar_revealed = true;
            return Task::none();
        }
        if !self.toolbar_revealed {
            return Task::none();
        }
        let generation = self.toolbar_hover_generation;
        Task::perform(
            async { tokio::time::sleep(TOOLBAR_HIDE_DELAY).await },
            move |_| Message::ToolbarHideAfter(generation),
        )
    }

    /// A toolbar segment: show `target`, through the same transitions as its
    /// shortcut (⌘E, ⌘M, ⌘⇧M, Esc). Settings and vault search close first
    /// so the chosen view shows.
    pub(super) fn toolbar_view(&mut self, target: ToolbarView) -> Task<Message> {
        let mut tasks = Vec::new();
        self.settings_open = false;
        if self.vault_open {
            tasks.push(self.update(Message::VaultClose));
        }
        if target == ToolbarView::FullMindmap {
            if self.full_mindmap.is_none() {
                tasks.push(self.update(Message::ToggleFullMindmap));
            }
            return Task::batch(tasks);
        }
        if self.full_mindmap.is_some() {
            tasks.push(self.update(Message::ExitFullMindmap));
            if self.full_mindmap.is_some() {
                // The exit waits for a workspace refresh; the reader shows
                // its own view when it lands.
                return Task::batch(tasks);
            }
        }
        let step = match (target, self.view_mode) {
            (ToolbarView::Read, ViewMode::Raw)
            | (ToolbarView::Edit, ViewMode::Rendered | ViewMode::Mindmap) => {
                Some(Message::ToggleViewMode)
            }
            (ToolbarView::Read, ViewMode::Mindmap)
            | (ToolbarView::Mindmap, ViewMode::Rendered | ViewMode::Raw) => {
                Some(Message::ToggleMindmap)
            }
            _ => None,
        };
        if let Some(step) = step {
            tasks.push(self.update(step));
        }
        Task::batch(tasks)
    }
}
