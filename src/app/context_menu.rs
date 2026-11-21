//! The context menu (docs/plans/mouse.md, stage 3): the right button, the
//! Menu key or Shift+F10. Its lines are the commands of the item under the
//! cursor, each with its key, so the menu also teaches the keys; what
//! cannot run here (`refusal`) is left out, as in the footers. Choosing a
//! line runs the command as its key would: the questions and the trash
//! are the same.

use super::*;
use crate::mouse::{self, Owner};

/// What a line of the menu does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Cmd(Command),
    /// A key of the viewer, which has its own keys, not commands.
    Key(KeyEvent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuLine {
    /// A line that runs something: its label and the key shown beside it.
    Run {
        label: String,
        key: String,
        action: MenuAction,
    },
    /// A title of a group ("3 tagged"), not chosen.
    Title(String),
    Separator,
}

impl MenuLine {
    pub fn runs(&self) -> bool {
        matches!(self, MenuLine::Run { .. })
    }
}

/// What the menu is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuTarget {
    /// The file under the cursor.
    File,
    /// The directory under the cursor of the tree.
    Dir,
    /// The file window, with no file under the pointer.
    FileWindow,
    Viewer,
}

pub struct ContextMenu {
    pub lines: Vec<MenuLine>,
    /// The line chosen (always one that runs).
    pub selected: usize,
    /// Where it opens: the menu's top left corner, kept inside the screen
    /// by whoever draws it.
    pub at: (u16, u16),
    /// A letter runs the line whose key it is (the Alt+V selector), instead
    /// of doing nothing.
    pub letters: bool,
}

impl ContextMenu {
    fn step(&mut self, down: bool) {
        let n = self.lines.len();
        let mut i = self.selected;
        for _ in 0..n {
            i = if down { (i + 1) % n } else { (i + n - 1) % n };
            if self.lines[i].runs() {
                self.selected = i;
                return;
            }
        }
    }

    fn edge(&mut self, last: bool) {
        let found = if last {
            self.lines.iter().rposition(MenuLine::runs)
        } else {
            self.lines.iter().position(MenuLine::runs)
        };
        if let Some(i) = found {
            self.selected = i;
        }
    }
}

impl App {
    /// The key of `cmd` in the window with the focus, as shown on screen.
    fn menu_key(&self, cmd: Command) -> String {
        self.keymap.key_in(cmd, self.focus).unwrap_or_default()
    }

    fn menu_cmd(&self, label: &str, cmd: Command) -> Option<MenuLine> {
        self.refusal(cmd).is_none().then(|| MenuLine::Run {
            label: label.to_string(),
            key: self.menu_key(cmd),
            action: MenuAction::Cmd(cmd),
        })
    }

    /// The lines for `target`, with no separator at the ends or twice.
    pub(crate) fn menu_lines(&self, target: MenuTarget) -> Vec<MenuLine> {
        type Entry<'a> = (&'a str, Command);
        let mut groups: Vec<(Option<String>, Vec<Entry>)> = Vec::new();
        match target {
            MenuTarget::File => {
                let tagged_here = self.selected_file().is_some_and(|f| self.file(f).tagged);
                let tag: Entry = if tagged_here {
                    ("Untag", Command::Untag)
                } else {
                    ("Tag", Command::Tag)
                };
                groups.push((
                    None,
                    vec![
                        ("View", Command::View),
                        ("Edit", Command::Edit),
                        ("Open with the system", Command::OpenWith { tagged: false }),
                        ("Info", Command::Info),
                    ],
                ));
                groups.push((None, vec![tag]));
                groups.push((
                    None,
                    vec![
                        (
                            "Copy",
                            Command::Copy {
                                tagged: false,
                                paths: false,
                            },
                        ),
                        (
                            "Move",
                            Command::Move {
                                tagged: false,
                                paths: false,
                            },
                        ),
                        ("Rename", Command::Rename { tagged: false }),
                        (
                            "Delete",
                            Command::Delete {
                                tagged: false,
                                permanent: false,
                            },
                        ),
                        ("Attributes", Command::Attributes { tagged: false }),
                        ("New date", Command::NewDate { tagged: false }),
                        ("Yank the path", Command::Clipboard { tagged: false }),
                    ],
                ));
                let tagged = self.files.iter().filter(|&&f| self.file(f).tagged).count();
                if tagged > 0 {
                    let title = if tagged == 1 {
                        "1 tagged file".to_string()
                    } else {
                        format!("{tagged} tagged files")
                    };
                    groups.push((
                        Some(title),
                        vec![
                            (
                                "Copy tagged",
                                Command::Copy {
                                    tagged: true,
                                    paths: false,
                                },
                            ),
                            (
                                "Move tagged",
                                Command::Move {
                                    tagged: true,
                                    paths: false,
                                },
                            ),
                            ("Rename tagged (mask)", Command::Rename { tagged: true }),
                            (
                                "Delete tagged",
                                Command::Delete {
                                    tagged: true,
                                    permanent: false,
                                },
                            ),
                            ("View tagged", Command::ViewTagged),
                            ("Yank tagged paths", Command::Clipboard { tagged: true }),
                            ("Untag all", Command::UntagAll),
                        ],
                    ));
                }
            }
            MenuTarget::Dir => {
                groups.push((
                    None,
                    vec![
                        ("Files", Command::SwitchWindow),
                        ("Log", Command::Log),
                        ("Relog", Command::Relog),
                        ("Expand all", Command::ExpandAll),
                        ("Collapse", Command::Collapse),
                        ("Release", Command::Release),
                    ],
                ));
                groups.push((
                    None,
                    vec![("Branch", Command::Branch), ("Showall", Command::Showall)],
                ));
                groups.push((
                    None,
                    vec![
                        ("Make a directory", Command::MakeDir),
                        ("Rename", Command::Rename { tagged: false }),
                        (
                            "Delete",
                            Command::Delete {
                                tagged: false,
                                permanent: false,
                            },
                        ),
                        ("Prune", Command::Prune),
                        ("Attributes", Command::Attributes { tagged: false }),
                    ],
                ));
                groups.push((
                    None,
                    vec![("Tag its files", Command::Tag), ("Untag", Command::Untag)],
                ));
                groups.push((
                    None,
                    vec![
                        ("Terminal here", Command::Terminal),
                        ("Usage", Command::Usage),
                        ("Duplicates", Command::Duplicates),
                        ("Bookmark it", Command::BookmarkSet),
                        ("Yank the path", Command::Clipboard { tagged: false }),
                    ],
                ));
            }
            MenuTarget::FileWindow => {
                groups.push((
                    None,
                    vec![
                        ("Filespec", Command::Filespec),
                        ("Sort", Command::Sort),
                        ("File display", Command::FileDisplay),
                    ],
                ));
                groups.push((
                    None,
                    vec![
                        ("Tag all", Command::TagAll),
                        ("Untag all", Command::UntagAll),
                        ("Invert tags", Command::Invert),
                    ],
                ));
            }
            MenuTarget::Viewer => return self.viewer_menu_lines(),
        }

        let mut lines = Vec::new();
        for (title, entries) in groups {
            let runs: Vec<MenuLine> = entries
                .into_iter()
                .filter_map(|(label, cmd)| self.menu_cmd(label, cmd))
                .collect();
            if runs.is_empty() {
                continue;
            }
            if !lines.is_empty() {
                lines.push(MenuLine::Separator);
            }
            if let Some(title) = title {
                lines.push(MenuLine::Title(title));
            }
            lines.extend(runs);
        }
        lines
    }

    /// The viewer's own keys, by mode.
    fn viewer_menu_lines(&self) -> Vec<MenuLine> {
        let Some(v) = &self.viewer else {
            return Vec::new();
        };
        let key = |label: &str, shown: &str, code: KeyCode| MenuLine::Run {
            label: label.to_string(),
            key: shown.to_string(),
            action: MenuAction::Key(KeyEvent::new(code, KeyModifiers::NONE)),
        };
        let mut lines = vec![
            key("Search", "/", KeyCode::Char('/')),
            key("Next match", "N", KeyCode::Char('n')),
            key("Go to line", "G", KeyCode::Char('g')),
        ];
        match v.mode {
            viewer::ViewMode::Table => {
                lines.push(key("SQL filter", "&", KeyCode::Char('&')));
            }
            viewer::ViewMode::Text => {
                lines.push(key("Filter lines", "&", KeyCode::Char('&')));
                lines.push(MenuLine::Separator);
                lines.push(key("Wrap", "W", KeyCode::Char('w')));
                lines.push(key("Line numbers", "#", KeyCode::Char('#')));
                lines.push(key("Hex", "H", KeyCode::Char('h')));
                lines.push(key("Charset", "O", KeyCode::Char('o')));
                lines.push(key("Colors", "C", KeyCode::Char('c')));
            }
            viewer::ViewMode::Hex => {
                lines.push(MenuLine::Separator);
                lines.push(key("Text", "H", KeyCode::Char('h')));
            }
            viewer::ViewMode::Rendered => {
                lines.push(MenuLine::Separator);
                lines.push(key("Source", "T", KeyCode::Char('t')));
                lines.push(key("Colors", "C", KeyCode::Char('c')));
            }
            viewer::ViewMode::Tree => {
                lines.push(MenuLine::Separator);
                lines.push(key("Open or close", "Enter", KeyCode::Enter));
                lines.push(key("Open everything under it", "*", KeyCode::Char('*')));
                lines.push(key("Copy the path of the node", "K", KeyCode::Char('k')));
                lines.push(key("Text at this node", "T", KeyCode::Char('t')));
            }
        }
        lines.push(MenuLine::Separator);
        if v.on_disk {
            lines.push(key("Edit", "E", KeyCode::Char('e')));
        }
        lines.push(key("Yank the path", "Y", KeyCode::Char('y')));
        lines.push(key("Close", "Esc", KeyCode::Esc));
        lines
    }

    /// Opens the menu for `target` with its corner at `at`.
    pub(crate) fn open_menu(&mut self, target: MenuTarget, at: (u16, u16)) {
        let lines = self.menu_lines(target);
        let Some(selected) = lines.iter().position(MenuLine::runs) else {
            return;
        };
        self.context_menu = Some(ContextMenu {
            lines,
            selected,
            at,
            letters: false,
        });
    }

    /// The Menu key or Shift+F10: the menu of what has the cursor, next to
    /// its row.
    pub(crate) fn open_menu_from_keys(&mut self) {
        if self.viewer.is_some() {
            self.open_menu(MenuTarget::Viewer, (4, 2));
            return;
        }
        let (target, list) = match self.focus {
            Focus::Tree => (MenuTarget::Dir, mouse::List::Tree(Owner::Main(self.active))),
            Focus::Files if self.selected_file().is_some() => (
                MenuTarget::File,
                mouse::List::Files(Owner::Main(self.active)),
            ),
            Focus::Files => (
                MenuTarget::FileWindow,
                mouse::List::Files(Owner::Main(self.active)),
            ),
        };
        let at = self
            .hits
            .row_of(list, self.cursor_index())
            .unwrap_or((4, 2));
        self.open_menu(target, at);
    }

    /// The cursor of the window with the focus.
    fn cursor_index(&self) -> usize {
        match self.focus {
            Focus::Tree => self.tree_state.selected().unwrap_or(0),
            Focus::Files => self.file_state.selected().unwrap_or(0),
        }
    }

    /// A key while the menu is open: it takes them all.
    pub(crate) fn context_menu_key(&mut self, key: KeyEvent) {
        let Some(menu) = &mut self.context_menu else {
            return;
        };
        match key.code {
            KeyCode::Esc => self.context_menu = None,
            KeyCode::Up => menu.step(false),
            KeyCode::Down | KeyCode::Tab => menu.step(true),
            KeyCode::Home | KeyCode::PageUp => menu.edge(false),
            KeyCode::End | KeyCode::PageDown => menu.edge(true),
            KeyCode::Enter | KeyCode::Char(' ') => {
                let i = menu.selected;
                self.run_menu_line(i);
            }
            // The menu's key again closes it.
            _ if is_menu_key(key) => self.context_menu = None,
            KeyCode::Char(c) if menu.letters => {
                let line = menu.lines.iter().position(|l| {
                    matches!(l, MenuLine::Run { key, .. }
                        if key.chars().count() == 1 && key.eq_ignore_ascii_case(&c.to_string()))
                });
                if let Some(i) = line {
                    self.run_menu_line(i);
                }
            }
            _ => {}
        }
    }

    /// Runs line `i` of the menu (and closes it).
    pub(crate) fn run_menu_line(&mut self, i: usize) {
        let Some(menu) = self.context_menu.take() else {
            return;
        };
        let Some(MenuLine::Run { action, .. }) = menu.lines.get(i) else {
            // A title or a separator: the menu stays.
            self.context_menu = Some(menu);
            return;
        };
        match *action {
            MenuAction::Cmd(cmd) => {
                self.message = None;
                self.apply(cmd);
                self.sync_autoview();
            }
            MenuAction::Key(key) => self.handle_key(key),
        }
    }
}

/// The Menu key, or Shift+F10 (F22 in the terminals that send it so).
pub(crate) fn is_menu_key(key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Menu | KeyCode::F(22) => true,
        KeyCode::F(10) => key.modifiers.contains(KeyModifiers::SHIFT),
        _ => false,
    }
}
