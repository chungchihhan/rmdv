//! The native menu bar's model (MDV-048 P8): which commands it lists,
//! their titles and shortcuts, and what a click runs.
//!
//! Titles and shortcuts come from the command palette (`command_items`)
//! where the palette lists the command; a test holds them equal. Every
//! item with a shortcut that the key handler maps runs, on a click, the
//! message the shortcut would run now ([`key_message`] on the current
//! [`KeyContext`]), so a menu click and a key press do the same thing on
//! every surface, and the item is disabled where the shortcut does nothing.
//! The menu never takes those keys itself (see `crate::native_menu`).

use super::*;
use crate::native_menu::{self, Action, Entry, Key, Menu, Mods, Native, Role, Shortcut};

/// The rmdv website, for Help.
pub(super) const WEBSITE: &str = "https://rmdv.mclee.dev/";

/// A menu bar command; [`MenuCommand::tag`] is its item's tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MenuCommand {
    Settings,
    OpenFolder,
    FindFile,
    Save,
    CloseSlot,
    CloseWindow,
    Refresh,
    RevealInFinder,
    CopyPath,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
    Find,
    FindInVault,
    View(ToolbarView),
    ToggleSidebar,
    AlwaysShowToolbar,
    ToggleFooter,
    ShowHidden,
    FontUp,
    FontDown,
    FontReset,
    CycleTheme,
    PickTheme,
    CommandPalette,
    Back,
    Forward,
    QuickSlot(u8),
    NextSlot,
    PreviousSlot,
    KeyboardShortcuts,
    Website,
}

/// Where an item's title comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Title {
    /// The palette's name for the same command (before its shortcut).
    Palette(&'static str),
    /// A menu-only title: a macOS standard name, a view name as the
    /// toolbar shows it, or a command the palette does not list.
    Own(&'static str),
}

/// An item: its title, its shortcut, and, for an item whose click runs
/// its shortcut, which messages count as that command.
#[derive(Clone, Copy)]
pub(super) struct CommandSpec {
    pub title: Title,
    pub shortcut: Option<Shortcut>,
    pub keyed: Option<fn(&Message) -> bool>,
}

const SLOT_TITLES: [&str; crate::quick_slots::SLOT_COUNT] = [
    "Slot 1", "Slot 2", "Slot 3", "Slot 4", "Slot 5", "Slot 6", "Slot 7", "Slot 8", "Slot 9",
];

const NO_MODS: Mods = Mods {
    command: false,
    shift: false,
    option: false,
    control: false,
};

const CMD: Mods = Mods {
    command: true,
    ..NO_MODS
};

impl MenuCommand {
    pub(super) const ALL: [MenuCommand; 46] = [
        MenuCommand::Settings,
        MenuCommand::OpenFolder,
        MenuCommand::FindFile,
        MenuCommand::Save,
        MenuCommand::CloseSlot,
        MenuCommand::CloseWindow,
        MenuCommand::Refresh,
        MenuCommand::RevealInFinder,
        MenuCommand::CopyPath,
        MenuCommand::Undo,
        MenuCommand::Redo,
        MenuCommand::Cut,
        MenuCommand::Copy,
        MenuCommand::Paste,
        MenuCommand::SelectAll,
        MenuCommand::Find,
        MenuCommand::FindInVault,
        MenuCommand::View(ToolbarView::Read),
        MenuCommand::View(ToolbarView::Edit),
        MenuCommand::View(ToolbarView::Mindmap),
        MenuCommand::View(ToolbarView::FullMindmap),
        MenuCommand::ToggleSidebar,
        MenuCommand::AlwaysShowToolbar,
        MenuCommand::ToggleFooter,
        MenuCommand::ShowHidden,
        MenuCommand::FontUp,
        MenuCommand::FontDown,
        MenuCommand::FontReset,
        MenuCommand::CycleTheme,
        MenuCommand::PickTheme,
        MenuCommand::CommandPalette,
        MenuCommand::Back,
        MenuCommand::Forward,
        MenuCommand::QuickSlot(0),
        MenuCommand::QuickSlot(1),
        MenuCommand::QuickSlot(2),
        MenuCommand::QuickSlot(3),
        MenuCommand::QuickSlot(4),
        MenuCommand::QuickSlot(5),
        MenuCommand::QuickSlot(6),
        MenuCommand::QuickSlot(7),
        MenuCommand::QuickSlot(8),
        MenuCommand::NextSlot,
        MenuCommand::PreviousSlot,
        MenuCommand::KeyboardShortcuts,
        MenuCommand::Website,
    ];

    /// The item's tag: its place in [`MenuCommand::ALL`], from 1.
    pub(super) fn tag(self) -> u32 {
        Self::ALL
            .iter()
            .position(|command| *command == self)
            .map_or(0, |index| index as u32 + 1)
    }

    pub(super) fn from_tag(tag: u32) -> Option<Self> {
        let index = usize::try_from(tag).ok()?.checked_sub(1)?;
        Self::ALL.get(index).copied()
    }

    pub(super) fn spec(self) -> CommandSpec {
        use MenuCommand as C;
        fn keyed(title: Title, shortcut: Shortcut, accept: fn(&Message) -> bool) -> CommandSpec {
            CommandSpec {
                title,
                shortcut: Some(shortcut),
                keyed: Some(accept),
            }
        }
        fn pointer(title: Title, shortcut: Option<Shortcut>) -> CommandSpec {
            CommandSpec {
                title,
                shortcut,
                keyed: None,
            }
        }
        let p = Title::Palette;
        let own = Title::Own;
        match self {
            C::Settings => keyed(own("Settings…"), Shortcut::cmd(','), |m| {
                matches!(m, Message::ToggleSettings)
            }),
            C::OpenFolder => keyed(p("Open Folder…"), Shortcut::cmd('o'), |m| {
                matches!(m, Message::OpenFolderPicker)
            }),
            C::FindFile => keyed(p("Find File in Workspace…"), Shortcut::cmd('p'), |m| {
                matches!(m, Message::OpenFileFinder)
            }),
            C::Save => keyed(own("Save"), Shortcut::cmd('s'), |m| {
                matches!(m, Message::SaveFile)
            }),
            C::CloseSlot => keyed(own("Close Slot"), Shortcut::cmd('w'), |m| {
                matches!(m, Message::QuickSlotClose)
            }),
            C::CloseWindow => keyed(own("Close Window"), Shortcut::cmd_shift('w'), |m| {
                matches!(m, Message::QuickSlotCloseWindow)
            }),
            C::Refresh => keyed(p("Refresh File / Folder"), Shortcut::cmd('r'), |m| {
                matches!(m, Message::Refresh)
            }),
            C::RevealInFinder => {
                keyed(p("Reveal File in Finder"), Shortcut::cmd_option('r'), |m| {
                    matches!(m, Message::RevealFileInFinder)
                })
            }
            C::CopyPath => keyed(
                p("Copy Focused File Path"),
                Shortcut::cmd_option('c'),
                |m| matches!(m, Message::CopyFilePath),
            ),
            C::Undo => keyed(own("Undo"), Shortcut::cmd('z'), |m| {
                matches!(m, Message::EditorUndo)
            }),
            C::Redo => keyed(own("Redo"), Shortcut::cmd_shift('z'), |m| {
                matches!(m, Message::EditorRedo)
            }),
            C::Cut => pointer(own("Cut"), Some(Shortcut::cmd('x'))),
            C::Copy => pointer(own("Copy"), Some(Shortcut::cmd('c'))),
            C::Paste => pointer(own("Paste"), Some(Shortcut::cmd('v'))),
            C::SelectAll => pointer(own("Select All"), Some(Shortcut::cmd('a'))),
            C::Find => keyed(p("Find in Document"), Shortcut::cmd('f'), |m| {
                matches!(m, Message::ToggleSearch)
            }),
            C::FindInVault => keyed(p("Search All Files…"), Shortcut::cmd_shift('f'), |m| {
                matches!(m, Message::OpenVaultSearch)
            }),
            // The toolbar's view names; the shortcuts are its tooltips'.
            C::View(ToolbarView::Read) => pointer(own("Read"), None),
            C::View(ToolbarView::Edit) => pointer(own("Edit"), Some(Shortcut::cmd('e'))),
            C::View(ToolbarView::Mindmap) => pointer(own("Mind Map"), Some(Shortcut::cmd('m'))),
            C::View(ToolbarView::FullMindmap) => {
                pointer(own("Full Mindmap"), Some(Shortcut::cmd_shift('m')))
            }
            C::ToggleSidebar => keyed(p("Toggle Sidebar"), Shortcut::cmd('b'), |m| {
                matches!(m, Message::ToggleSidebar)
            }),
            C::AlwaysShowToolbar => pointer(own("Always Show Toolbar"), None),
            C::ToggleFooter => pointer(p("Toggle Status Footer"), None),
            C::ShowHidden => keyed(p("Toggle Hidden Files"), Shortcut::cmd_shift('.'), |m| {
                matches!(m, Message::ToggleHidden)
            }),
            C::FontUp => keyed(p("Increase Font Size"), Shortcut::cmd('+'), |m| {
                matches!(m, Message::FontSizeUp)
            }),
            C::FontDown => keyed(p("Decrease Font Size"), Shortcut::cmd('-'), |m| {
                matches!(m, Message::FontSizeDown)
            }),
            C::FontReset => keyed(p("Reset Font Size"), Shortcut::cmd('0'), |m| {
                matches!(m, Message::FontSizeReset)
            }),
            C::CycleTheme => keyed(p("Cycle Theme"), Shortcut::cmd('t'), |m| {
                matches!(m, Message::ToggleTheme)
            }),
            C::PickTheme => pointer(p("Pick Theme…"), None),
            C::CommandPalette => keyed(own("Command Palette…"), Shortcut::cmd_shift('p'), |m| {
                matches!(m, Message::OpenCommandPalette)
            }),
            C::Back => keyed(p("Go Back"), Shortcut::cmd('['), |m| {
                matches!(m, Message::HistoryGo(HistoryDir::Back))
            }),
            C::Forward => keyed(p("Go Forward"), Shortcut::cmd(']'), |m| {
                matches!(m, Message::HistoryGo(HistoryDir::Forward))
            }),
            C::QuickSlot(index) => {
                let index = usize::from(index).min(crate::quick_slots::SLOT_COUNT - 1);
                let digit = char::from_digit(index as u32 + 1, 10).unwrap_or('1');
                CommandSpec {
                    title: own(SLOT_TITLES[index]),
                    shortcut: Some(Shortcut::cmd(digit)),
                    keyed: Some(match index {
                        0 => |m| matches!(m, Message::QuickSlotActivate(0)),
                        1 => |m| matches!(m, Message::QuickSlotActivate(1)),
                        2 => |m| matches!(m, Message::QuickSlotActivate(2)),
                        3 => |m| matches!(m, Message::QuickSlotActivate(3)),
                        4 => |m| matches!(m, Message::QuickSlotActivate(4)),
                        5 => |m| matches!(m, Message::QuickSlotActivate(5)),
                        6 => |m| matches!(m, Message::QuickSlotActivate(6)),
                        7 => |m| matches!(m, Message::QuickSlotActivate(7)),
                        _ => |m| matches!(m, Message::QuickSlotActivate(8)),
                    }),
                }
            }
            C::NextSlot => keyed(own("Next Slot"), Shortcut::with(Key::Down, CMD), |m| {
                matches!(m, Message::QuickSlotCycle(1))
            }),
            C::PreviousSlot => keyed(own("Previous Slot"), Shortcut::with(Key::Up, CMD), |m| {
                matches!(m, Message::QuickSlotCycle(-1))
            }),
            C::KeyboardShortcuts => keyed(p("Show Keyboard Shortcuts"), Shortcut::cmd('/'), |m| {
                matches!(m, Message::ToggleShortcuts)
            }),
            C::Website => pointer(own("rmdv Website"), None),
        }
    }
}

/// The key press a shortcut makes, as the key subscription receives it.
pub(super) fn shortcut_event(shortcut: Shortcut) -> iced::Event {
    use iced::keyboard::{self, key::Named, key::Physical, Modifiers};
    let (key, code) = match shortcut.key {
        Key::Char(c) => (
            iced::keyboard::Key::Character(c.to_string().into()),
            char_code(c),
        ),
        Key::Up => (
            iced::keyboard::Key::Named(Named::ArrowUp),
            Some(keyboard::key::Code::ArrowUp),
        ),
        Key::Down => (
            iced::keyboard::Key::Named(Named::ArrowDown),
            Some(keyboard::key::Code::ArrowDown),
        ),
    };
    let mut modifiers = Modifiers::empty();
    if shortcut.mods.command {
        modifiers |= Modifiers::COMMAND;
    }
    if shortcut.mods.shift {
        modifiers |= Modifiers::SHIFT;
    }
    if shortcut.mods.option {
        modifiers |= Modifiers::ALT;
    }
    if shortcut.mods.control {
        modifiers |= Modifiers::CTRL;
    }
    iced::Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: code.map_or(
            Physical::Unidentified(keyboard::key::NativeCode::Unidentified),
            Physical::Code,
        ),
        location: keyboard::Location::Standard,
        modifiers,
        text: None,
        repeat: false,
    })
}

/// The US-layout physical key of a shortcut character.
fn char_code(c: char) -> Option<iced::keyboard::key::Code> {
    use iced::keyboard::key::Code;
    Some(match c.to_ascii_lowercase() {
        'a' => Code::KeyA,
        'b' => Code::KeyB,
        'c' => Code::KeyC,
        'd' => Code::KeyD,
        'e' => Code::KeyE,
        'f' => Code::KeyF,
        'g' => Code::KeyG,
        'h' => Code::KeyH,
        'i' => Code::KeyI,
        'j' => Code::KeyJ,
        'k' => Code::KeyK,
        'l' => Code::KeyL,
        'm' => Code::KeyM,
        'n' => Code::KeyN,
        'o' => Code::KeyO,
        'p' => Code::KeyP,
        'q' => Code::KeyQ,
        'r' => Code::KeyR,
        's' => Code::KeyS,
        't' => Code::KeyT,
        'u' => Code::KeyU,
        'v' => Code::KeyV,
        'w' => Code::KeyW,
        'x' => Code::KeyX,
        'y' => Code::KeyY,
        'z' => Code::KeyZ,
        '0' => Code::Digit0,
        '1' => Code::Digit1,
        '2' => Code::Digit2,
        '3' => Code::Digit3,
        '4' => Code::Digit4,
        '5' => Code::Digit5,
        '6' => Code::Digit6,
        '7' => Code::Digit7,
        '8' => Code::Digit8,
        '9' => Code::Digit9,
        ',' => Code::Comma,
        '.' => Code::Period,
        '/' => Code::Slash,
        '[' => Code::BracketLeft,
        ']' => Code::BracketRight,
        '-' => Code::Minus,
        '=' | '+' => Code::Equal,
        _ => return None,
    })
}

fn native(title: &str, shortcut: Option<Shortcut>, action: Native) -> Entry {
    Entry::Item {
        title: title.to_string(),
        shortcut,
        action: Action::Native(action),
    }
}

impl App {
    /// The menu bar: the app menu, File, Edit, View, Go, Window, Help.
    pub(super) fn menu_bar(&self) -> Vec<Menu> {
        use MenuCommand as C;
        let item = |command| self.menu_item(command);
        let menu = |title: &str, role, entries| Menu {
            title: title.to_string(),
            role,
            entries,
        };
        let app = menu(
            "rmdv",
            Role::Plain,
            vec![
                native("About rmdv", None, Native::About),
                Entry::Separator,
                item(C::Settings),
                Entry::Separator,
                Entry::Submenu(menu("Services", Role::Services, Vec::new())),
                Entry::Separator,
                native("Hide rmdv", Some(Shortcut::cmd('h')), Native::Hide),
                native(
                    "Hide Others",
                    Some(Shortcut::cmd_option('h')),
                    Native::HideOthers,
                ),
                native("Show All", None, Native::ShowAll),
                Entry::Separator,
                // `terminate:` reaches `macos_quit`, which asks about
                // unsaved edits first.
                native("Quit rmdv", Some(Shortcut::cmd('q')), Native::Quit),
            ],
        );
        let file = menu(
            "File",
            Role::Plain,
            vec![
                item(C::OpenFolder),
                Entry::Submenu(menu("Open Recent", Role::Recent, Vec::new())),
                item(C::FindFile),
                Entry::Separator,
                item(C::Save),
                Entry::Separator,
                item(C::CloseSlot),
                item(C::CloseWindow),
                Entry::Separator,
                item(C::Refresh),
                item(C::RevealInFinder),
                item(C::CopyPath),
            ],
        );
        let edit = menu(
            "Edit",
            Role::Plain,
            vec![
                item(C::Undo),
                item(C::Redo),
                Entry::Separator,
                item(C::Cut),
                item(C::Copy),
                item(C::Paste),
                item(C::SelectAll),
                Entry::Separator,
                item(C::Find),
                item(C::FindInVault),
            ],
        );
        let view = menu(
            "View",
            Role::Plain,
            vec![
                item(C::View(ToolbarView::Read)),
                item(C::View(ToolbarView::Edit)),
                item(C::View(ToolbarView::Mindmap)),
                item(C::View(ToolbarView::FullMindmap)),
                Entry::Separator,
                item(C::ToggleSidebar),
                item(C::AlwaysShowToolbar),
                item(C::ToggleFooter),
                item(C::ShowHidden),
                Entry::Separator,
                item(C::FontUp),
                item(C::FontDown),
                item(C::FontReset),
                Entry::Separator,
                item(C::CycleTheme),
                item(C::PickTheme),
                item(C::CommandPalette),
                Entry::Separator,
                native(
                    "Enter Full Screen",
                    Some(Shortcut::cmd_control('f')),
                    Native::ToggleFullScreen,
                ),
            ],
        );
        let slots = (0..crate::quick_slots::SLOT_COUNT as u8)
            .map(|index| item(C::QuickSlot(index)))
            .collect();
        let go = menu(
            "Go",
            Role::Plain,
            vec![
                item(C::Back),
                item(C::Forward),
                Entry::Separator,
                Entry::Submenu(menu("Quick Slots", Role::Plain, slots)),
                item(C::NextSlot),
                item(C::PreviousSlot),
            ],
        );
        // Minimize has no ⌘M: ⌘M is Mind Map.
        let window = menu(
            "Window",
            Role::Windows,
            vec![
                native("Minimize", None, Native::Minimize),
                native("Zoom", None, Native::Zoom),
                Entry::Separator,
                native("Bring All to Front", None, Native::BringAllToFront),
            ],
        );
        let help = menu(
            "Help",
            Role::Help,
            vec![item(C::KeyboardShortcuts), item(C::Website)],
        );
        vec![app, file, edit, view, go, window, help]
    }

    /// An item's title: the palette's name when the palette lists it.
    fn menu_title(&self, title: Title) -> String {
        match title {
            Title::Own(own) => own.to_string(),
            Title::Palette(name) => self
                .command_items()
                .into_iter()
                .map(|(label, _)| split_command_label(label).0)
                .find(|palette| *palette == name)
                .unwrap_or(name)
                .to_string(),
        }
    }

    fn menu_item(&self, command: MenuCommand) -> Entry {
        let spec = command.spec();
        Entry::Item {
            title: self.menu_title(spec.title),
            shortcut: spec.shortcut,
            action: Action::Command(command.tag()),
        }
    }

    /// Install the menu bar once, after NSApp and the window exist.
    pub(super) fn install_native_menu(&self) {
        if cfg!(target_os = "macos") && !native_menu::installed() {
            native_menu::install(&self.menu_bar());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items(menus: &[Menu]) -> Vec<(String, Option<Shortcut>, Action)> {
        fn walk(entries: &[Entry], out: &mut Vec<(String, Option<Shortcut>, Action)>) {
            for entry in entries {
                match entry {
                    Entry::Item {
                        title,
                        shortcut,
                        action,
                    } => out.push((title.clone(), *shortcut, *action)),
                    Entry::Separator => {}
                    Entry::Submenu(menu) => walk(&menu.entries, out),
                }
            }
        }
        let mut out = Vec::new();
        for menu in menus {
            walk(&menu.entries, &mut out);
        }
        out
    }

    fn find_menu<'a>(menus: &'a [Menu], title: &str) -> &'a Menu {
        fn walk<'a>(menu: &'a Menu, title: &str) -> Option<&'a Menu> {
            if menu.title == title {
                return Some(menu);
            }
            menu.entries.iter().find_map(|entry| match entry {
                Entry::Submenu(sub) => walk(sub, title),
                _ => None,
            })
        }
        menus
            .iter()
            .find_map(|menu| walk(menu, title))
            .unwrap_or_else(|| panic!("menu {title}"))
    }

    fn item<'a>(menu: &'a Menu, title: &str) -> (&'a Option<Shortcut>, &'a Action) {
        menu.entries
            .iter()
            .find_map(|entry| match entry {
                Entry::Item {
                    title: t,
                    shortcut,
                    action,
                } if t == title => Some((shortcut, action)),
                _ => None,
            })
            .unwrap_or_else(|| panic!("item {title} in {}", menu.title))
    }

    /// A reader on a file inside a workspace: the state where every
    /// palette command and most shortcuts act.
    pub(in crate::app) fn reader_app() -> App {
        let mut app = App::default();
        app.workspace = Some(PathBuf::from("/tmp/menu-bar-ws"));
        let _ = app.update(Message::FileLoaded(Ok((
            PathBuf::from("/tmp/menu-bar-ws/a.md"),
            "# A\n\nText.\n".into(),
        ))));
        app
    }

    /// The same file in the Zen editor.
    pub(in crate::app) fn editing_app() -> App {
        let mut app = reader_app();
        let _ = app.update(Message::ToggleViewMode);
        assert_eq!(app.view_mode, ViewMode::Raw);
        assert!(app.editor.is_some());
        app
    }

    #[test]
    fn every_menu_command_is_listed_once_and_its_tag_round_trips() {
        let app = reader_app();
        let menus = app.menu_bar();
        let tags: Vec<u32> = items(&menus)
            .into_iter()
            .filter_map(|(_, _, action)| match action {
                Action::Command(tag) => Some(tag),
                Action::Native(_) => None,
            })
            .collect();
        for command in MenuCommand::ALL {
            let tag = command.tag();
            assert!((1..=native_menu::MAX_COMMAND_TAG).contains(&tag));
            assert_eq!(MenuCommand::from_tag(tag), Some(command));
            assert_eq!(
                tags.iter().filter(|t| **t == tag).count(),
                1,
                "{command:?} must be in the menu bar once"
            );
        }
        assert_eq!(tags.len(), MenuCommand::ALL.len());
        assert_eq!(MenuCommand::from_tag(0), None);
        assert_eq!(
            MenuCommand::from_tag(MenuCommand::ALL.len() as u32 + 1),
            None
        );
    }

    #[test]
    fn palette_titles_and_shortcuts_match_command_items() {
        let app = reader_app();
        let palette: Vec<(&str, Option<&str>, Message)> = app
            .command_items()
            .into_iter()
            .map(|(label, message)| {
                let (name, keys) = split_command_label(label);
                (name, keys, message)
            })
            .collect();
        let menu_titles: Vec<String> = items(&app.menu_bar())
            .into_iter()
            .map(|(title, _, _)| title)
            .collect();
        let mut checked = 0;
        for command in MenuCommand::ALL {
            let spec = command.spec();
            let Title::Palette(name) = spec.title else {
                continue;
            };
            checked += 1;
            let (_, keys, message) = palette
                .iter()
                .find(|(n, _, _)| *n == name)
                .unwrap_or_else(|| panic!("the palette has no {name:?}"));
            assert!(menu_titles.iter().any(|t| t == name), "{name} title");
            match (keys, spec.shortcut) {
                (None, None) => {}
                (Some(keys), Some(shortcut)) => {
                    let mut mods = std::collections::BTreeSet::new();
                    let mut key = String::new();
                    for c in keys.chars() {
                        if "⌘⌥⇧⌃".contains(c) {
                            mods.insert(c);
                        } else {
                            key.push(c.to_ascii_uppercase());
                        }
                    }
                    assert_eq!(shortcut.glyphs(), (mods, key), "{name} shortcut");
                }
                other => panic!("{name}: palette and menu shortcuts differ: {other:?}"),
            }
            // The click runs the palette's command.
            if let Some(accept) = spec.keyed {
                assert!(accept(message), "{name}: palette runs {message:?}");
            }
        }
        assert!(checked >= 15, "most items name a palette command");
    }

    #[test]
    fn every_shortcut_is_the_key_its_command_has() {
        let reader = reader_app();
        let editing = editing_app();
        for command in MenuCommand::ALL {
            let spec = command.spec();
            let (Some(accept), Some(shortcut)) = (spec.keyed, spec.shortcut) else {
                continue;
            };
            let event = shortcut_event(shortcut);
            let in_reader = key_message(reader.key_context(), event.clone(), false);
            let in_editor = key_message(editing.key_context(), event, false);
            assert!(
                accept(&in_reader) || accept(&in_editor),
                "{command:?}: the key gives {in_reader:?} / {in_editor:?}"
            );
        }
    }

    #[test]
    fn menu_bar_has_the_standard_macos_items() {
        let menus = reader_app().menu_bar();
        let titles: Vec<&str> = menus.iter().map(|menu| menu.title.as_str()).collect();
        assert_eq!(
            titles,
            ["rmdv", "File", "Edit", "View", "Go", "Window", "Help"]
        );
        let app_menu = &menus[0];
        assert_eq!(
            item(app_menu, "About rmdv").1,
            &Action::Native(Native::About)
        );
        assert_eq!(item(app_menu, "Settings…").0, &Some(Shortcut::cmd(',')));
        assert_eq!(
            item(app_menu, "Hide rmdv"),
            (&Some(Shortcut::cmd('h')), &Action::Native(Native::Hide))
        );
        assert_eq!(
            item(app_menu, "Hide Others"),
            (
                &Some(Shortcut::cmd_option('h')),
                &Action::Native(Native::HideOthers)
            )
        );
        assert_eq!(
            item(app_menu, "Show All").1,
            &Action::Native(Native::ShowAll)
        );
        // Quit goes through `terminate:`, which asks about unsaved edits.
        assert_eq!(
            item(app_menu, "Quit rmdv"),
            (&Some(Shortcut::cmd('q')), &Action::Native(Native::Quit))
        );
        assert_eq!(find_menu(&menus, "Services").role, Role::Services);
        assert_eq!(find_menu(&menus, "Open Recent").role, Role::Recent);
        // ⌘M stays Mind Map and ⌘W stays Close Slot.
        let window = find_menu(&menus, "Window");
        assert_eq!(window.role, Role::Windows);
        assert_eq!(
            item(window, "Minimize"),
            (&None, &Action::Native(Native::Minimize))
        );
        assert_eq!(item(window, "Zoom").1, &Action::Native(Native::Zoom));
        assert_eq!(
            item(window, "Bring All to Front").1,
            &Action::Native(Native::BringAllToFront)
        );
        let file = find_menu(&menus, "File");
        assert_eq!(item(file, "Close Slot").0, &Some(Shortcut::cmd('w')));
        let view = find_menu(&menus, "View");
        assert_eq!(item(view, "Mind Map").0, &Some(Shortcut::cmd('m')));
        assert_eq!(
            item(view, "Enter Full Screen"),
            (
                &Some(Shortcut::cmd_control('f')),
                &Action::Native(Native::ToggleFullScreen)
            )
        );
        assert_eq!(find_menu(&menus, "Help").role, Role::Help);
        assert_eq!(find_menu(&menus, "Quick Slots").entries.len(), 9);
    }

    #[test]
    fn no_two_menu_items_show_the_same_shortcut() {
        let all = items(&reader_app().menu_bar());
        let mut seen: Vec<(Shortcut, String)> = Vec::new();
        for (title, shortcut, _) in all {
            let Some(shortcut) = shortcut else {
                continue;
            };
            if let Some((_, other)) = seen.iter().find(|(s, _)| *s == shortcut) {
                panic!("{title} and {other} both show {:?}", shortcut.glyphs());
            }
            seen.push((shortcut, title));
        }
    }
}
