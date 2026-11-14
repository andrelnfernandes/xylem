//! Commands and the prompts' text field. The key map is in
//! `keymap.rs`.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Expand,
    Collapse,
    /// `-`: releases the branch from memory (unlogs it).
    Release,
    /// Logs and expands the whole branch.
    ExpandAll,
    /// Switches between the tree and the file window.
    SwitchWindow,
    /// Tab: switches window in split screen.
    SwitchPane,
    /// F8: splits the screen into two windows (or back to one).
    Split,
    /// Opens another directory in the window.
    Log,
    /// Compares with the other window's directory and tags files.
    Compare,
    /// Opens a ZIP/TAR as if it were a directory (Alt+F5).
    OpenArchive,
    /// Opens the file (or the tagged ones) in the system's default program.
    OpenWith {
        tagged: bool,
    },
    /// Alt+C in the tree: Compare of the whole branch.
    CompareBranch,
    /// Backspace: parent directory.
    Parent,
    /// Opens the file in the editor.
    Edit,
    /// Runs a shell command.
    Execute,
    /// Alt+X: shows the built-in terminal (or goes back to Xylem).
    Terminal,
    /// Alt+Z: the terminal in full screen (inside it: full screen / panel).
    TerminalFull,
    /// Alt+O: types the tagged files in the terminal.
    TerminalTagged,
    /// Ctrl+G: Git menu (when `git` is on in the configuration).
    Git,
    /// F1: key list.
    Help,
    /// Alt+S: chooses the file sort order.
    Sort,
    /// Alt+P: deletes the current directory with everything inside.
    Prune,
    /// F12 / Alt+F10: settings screen.
    Settings,
    /// Ctrl+S: searches for text in the tagged files and untags those without it.
    SearchTagged,
    /// Ctrl+V: views the tagged files, one after the other.
    ViewTagged,
    /// `\`: navigates by typing the path.
    Treespec,
    /// Alt+K: size or number of files per directory in the tree.
    Kbyte,
    /// Alt+U: where the space of the branch goes.
    Usage,
    /// Alt+D: files and folders with the same contents.
    Duplicates,
    /// Alt+Q: quits, leaving the shell in the current directory.
    QuitChangeDir,
    /// Alt+F: display mode of the file window.
    FileDisplay,
    /// Ctrl+F7 / Ctrl+F8 / Ctrl+F9 in the tree: tags, untags or inverts
    /// the branch's logged files.
    TagBranch,
    UntagBranch,
    InvertBranch,
    /// F7: the file list with the selected file's content beside it.
    Autoview,
    /// N: changes the modification date (of the file, the tagged ones or the
    /// directory, in the tree).
    NewDate {
        tagged: bool,
    },
    /// Ctrl+B: creates a file with one line per tagged file.
    Batch,
    /// Alt+I: details of the file or directory.
    Info,
    /// A in the tree: mounted file systems.
    Avail,
    /// ?: window statistics.
    Stats,
    /// F5 / Ctrl+F5: creates a ZIP/TAR with the file or the tagged ones.
    CreateArchive {
        tagged: bool,
    },
    /// ← and → in the file window: previous/next file in columns,
    /// page up/down in a single column.
    Left,
    Right,
    Back,
    Filespec,
    Showall,
    /// Ctrl+S in the tree: Showall of the tagged files only.
    ShowallTagged,
    Branch,
    View,
    /// Rereads the current directory from disk.
    Relog,
    Tag,
    Untag,
    TagAll,
    UntagAll,
    Invert,
    // `tagged`: acts on the tagged files instead of the current file.
    // `paths`: recreates the subdirectories at the destination (Alt+C / Alt+M).
    Copy {
        tagged: bool,
        paths: bool,
    },
    Move {
        tagged: bool,
        paths: bool,
    },
    Delete {
        tagged: bool,
        permanent: bool,
    },
    /// Restores the most recently trashed file(s) from system trash (Oops!).
    Oops,
    Attributes {
        tagged: bool,
    },
    Rename {
        tagged: bool,
    },
    MakeDir,
    /// Copies path(s) to the system clipboard (OSC 52 / native tools).
    Clipboard {
        tagged: bool,
    },
    /// F11: opens the directory bookmarks list / manager.
    Bookmarks,
    /// ': quick jump to a bookmark key.
    BookmarkJump,
    /// Ctrl+F11: saves a bookmark for the current directory.
    BookmarkSet,
    Quit,
}

/// Result of a key in a text field.
#[derive(Debug, PartialEq, Eq)]
pub enum LineEvent {
    Submit,
    Cancel,
    Editing,
}

/// Single-line text field, used in prompts (Filespec etc.).
pub struct LineInput {
    pub text: String,
    /// Cursor position, in characters.
    pub cursor: usize,
    /// Previous entries of this kind of prompt, oldest to newest.
    history: Vec<String>,
    /// The history open as a list (↑), over the prompt.
    list: Option<HistoryList>,
    /// Whether the text was typed or edited here, not only put in ready: only
    /// then does it filter the list.
    edited: bool,
    /// Entries taken out of the history with Del, for the caller to forget.
    forgotten: Vec<String>,
}

/// The history shown as a list: ↑↓ choose, typing filters, Enter puts the
/// entry in the line, Esc gives the line back as it was.
pub struct HistoryList {
    /// What the list shows: indices into the history, oldest first (the
    /// newest is at the bottom, next to the line).
    pub shown: Vec<usize>,
    /// Position in `shown`.
    pub cursor: usize,
    /// First row on screen, kept by whoever draws it.
    pub scroll: std::cell::Cell<usize>,
    /// The screen row of the prompt's line in the last frame drawn: the list
    /// goes just above it.
    pub anchor: std::cell::Cell<Option<u16>>,
    /// The line as it was when the list opened, for Esc.
    saved: (String, usize, bool),
}

/// Rows PgUp and PgDn move in the list.
const LIST_PAGE: usize = 10;

impl LineInput {
    pub fn new(text: &str) -> Self {
        LineInput {
            text: text.to_string(),
            cursor: text.chars().count(),
            history: Vec::new(),
            list: None,
            edited: false,
            forgotten: Vec::new(),
        }
    }

    pub fn with_history(mut self, history: Vec<String>) -> Self {
        self.history = history;
        self
    }

    pub fn history(&self) -> Vec<String> {
        self.history.clone()
    }

    pub fn has_history(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn history_entry(&self, i: usize) -> &str {
        self.history.get(i).map_or("", String::as_str)
    }

    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// The history list, while it is open.
    pub fn list(&self) -> Option<&HistoryList> {
        self.list.as_ref()
    }

    pub fn list_open(&self) -> bool {
        self.list.is_some()
    }

    /// Entries taken out with Del since the last call.
    pub fn take_forgotten(&mut self) -> Vec<String> {
        std::mem::take(&mut self.forgotten)
    }

    /// Puts a ready text in the line (not typed: it does not filter the list).
    pub fn set_text(&mut self, text: String) {
        self.cursor = text.chars().count();
        self.text = text;
        self.edited = false;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> LineEvent {
        if self.list.is_some() {
            self.list_key(key);
            return LineEvent::Editing;
        }
        let len = self.text.chars().count();
        match key.code {
            KeyCode::Enter => return LineEvent::Submit,
            KeyCode::Esc => return LineEvent::Cancel,
            KeyCode::Up => self.open_list(),
            KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Right => self.cursor = (self.cursor + 1).min(len),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = len,
            _ => {
                self.edit_key(key);
            }
        }
        LineEvent::Editing
    }

    /// The keys that change the text: whether it changed.
    fn edit_key(&mut self, key: KeyEvent) -> bool {
        let len = self.text.chars().count();
        match key.code {
            KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Right => self.cursor = (self.cursor + 1).min(len),
            KeyCode::Backspace if self.cursor > 0 => {
                self.cursor -= 1;
                self.text.remove(self.byte_pos());
            }
            KeyCode::Delete if self.cursor < len => {
                self.text.remove(self.byte_pos());
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.text.clear();
                self.cursor = 0;
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.text.insert(self.byte_pos(), c);
                self.cursor += 1;
            }
            _ => return false,
        }
        if !matches!(key.code, KeyCode::Left | KeyCode::Right) {
            self.edited = true;
        }
        true
    }

    /// ↑: the history as a list, filtered by what was typed. Nothing to show,
    /// nothing opens.
    fn open_list(&mut self) {
        if self.history.is_empty() {
            return;
        }
        self.list = Some(HistoryList {
            shown: Vec::new(),
            cursor: 0,
            scroll: std::cell::Cell::new(0),
            anchor: std::cell::Cell::new(None),
            saved: (self.text.clone(), self.cursor, self.edited),
        });
        self.refilter();
    }

    /// The entries holding the typed text, the cursor on the newest.
    fn refilter(&mut self) {
        let filter = if self.edited {
            self.text.trim().to_lowercase()
        } else {
            String::new()
        };
        let Some(list) = &mut self.list else {
            return;
        };
        list.shown = self
            .history
            .iter()
            .enumerate()
            .filter(|(_, h)| filter.is_empty() || h.to_lowercase().contains(&filter))
            .map(|(i, _)| i)
            .collect();
        list.cursor = list.shown.len().saturating_sub(1);
    }

    fn list_key(&mut self, key: KeyEvent) {
        let Some(list) = &mut self.list else {
            return;
        };
        let last = list.shown.len().saturating_sub(1);
        match key.code {
            KeyCode::Up => list.cursor = list.cursor.saturating_sub(1),
            KeyCode::Down => list.cursor = (list.cursor + 1).min(last),
            KeyCode::PageUp => list.cursor = list.cursor.saturating_sub(LIST_PAGE),
            KeyCode::PageDown => list.cursor = (list.cursor + LIST_PAGE).min(last),
            KeyCode::Home => list.cursor = 0,
            KeyCode::End => list.cursor = last,
            KeyCode::Esc => {
                let (text, cursor, edited) = std::mem::take(&mut list.saved);
                self.list = None;
                self.text = text;
                self.cursor = cursor;
                self.edited = edited;
            }
            KeyCode::Enter => {
                let chosen = list
                    .shown
                    .get(list.cursor)
                    .map(|&i| self.history[i].clone());
                self.list = None;
                if let Some(text) = chosen {
                    self.set_text(text);
                }
            }
            KeyCode::Delete => {
                let Some(&i) = list.shown.get(list.cursor) else {
                    return;
                };
                let cursor = list.cursor;
                let gone = self.history.remove(i);
                self.history.retain(|h| *h != gone);
                self.forgotten.push(gone);
                if self.history.is_empty() {
                    self.list = None;
                    return;
                }
                self.refilter();
                if let Some(list) = &mut self.list {
                    // Stays where it was, now on the next one down.
                    list.cursor = cursor.min(list.shown.len().saturating_sub(1));
                }
            }
            _ => {
                if self.edit_key(key) {
                    self.refilter();
                }
            }
        }
    }

    fn byte_pos(&self) -> usize {
        self.text
            .char_indices()
            .nth(self.cursor)
            .map_or(self.text.len(), |(i, _)| i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn type_keys(input: &mut LineInput, codes: &[KeyCode]) {
        for &code in codes {
            input.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
        }
    }

    #[test]
    fn edits_text() {
        let mut input = LineInput::new("*.rs");
        assert_eq!(input.cursor, 4);
        type_keys(&mut input, &[KeyCode::Char(' '), KeyCode::Char('*')]);
        assert_eq!(input.text, "*.rs *");
        type_keys(
            &mut input,
            &[KeyCode::Home, KeyCode::Delete, KeyCode::Char('x')],
        );
        assert_eq!(input.text, "x.rs *");
        type_keys(
            &mut input,
            &[KeyCode::End, KeyCode::Backspace, KeyCode::Backspace],
        );
        assert_eq!(input.text, "x.rs");
    }

    fn shown(input: &LineInput) -> Vec<&str> {
        let list = input.list().expect("the list is open");
        list.shown.iter().map(|&i| input.history_entry(i)).collect()
    }

    fn chosen(input: &LineInput) -> &str {
        let list = input.list().expect("the list is open");
        input.history_entry(list.shown[list.cursor])
    }

    fn history3(text: &str) -> LineInput {
        LineInput::new(text).with_history(vec![
            "/srv/one".into(),
            "/home/two".into(),
            "/srv/three".into(),
        ])
    }

    #[test]
    fn up_opens_the_history_as_a_list_on_the_newest() {
        let mut input = history3("");
        type_keys(&mut input, &[KeyCode::Up]);
        assert_eq!(shown(&input), ["/srv/one", "/home/two", "/srv/three"]);
        assert_eq!(chosen(&input), "/srv/three");
        assert_eq!(input.text, "", "the line does not change while choosing");
        type_keys(&mut input, &[KeyCode::Up, KeyCode::Up, KeyCode::Up]);
        assert_eq!(chosen(&input), "/srv/one");
        type_keys(&mut input, &[KeyCode::Down]);
        assert_eq!(chosen(&input), "/home/two");
        type_keys(&mut input, &[KeyCode::End]);
        assert_eq!(chosen(&input), "/srv/three");
        type_keys(&mut input, &[KeyCode::Home]);
        assert_eq!(chosen(&input), "/srv/one");
    }

    #[test]
    fn enter_puts_the_entry_in_the_line_without_submitting() {
        let mut input = history3("");
        type_keys(&mut input, &[KeyCode::Up, KeyCode::Up]);
        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(input.handle_key(enter), LineEvent::Editing);
        assert!(!input.list_open());
        assert_eq!((input.text.as_str(), input.cursor), ("/home/two", 9));
        assert_eq!(input.handle_key(enter), LineEvent::Submit);
    }

    #[test]
    fn esc_gives_the_line_back_as_it_was() {
        let mut input = history3("draft");
        type_keys(
            &mut input,
            &[KeyCode::Left, KeyCode::Up, KeyCode::Char('x')],
        );
        let esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(
            input.handle_key(esc),
            LineEvent::Editing,
            "Esc closes the list only"
        );
        assert!(!input.list_open());
        assert_eq!((input.text.as_str(), input.cursor), ("draft", 4));
        assert_eq!(input.handle_key(esc), LineEvent::Cancel);
    }

    #[test]
    fn typing_filters_the_list() {
        let mut input = history3("");
        type_keys(
            &mut input,
            &[KeyCode::Up, KeyCode::Char('S'), KeyCode::Char('r')],
        );
        assert_eq!(shown(&input), ["/srv/one", "/srv/three"]);
        assert_eq!(chosen(&input), "/srv/three");
        type_keys(&mut input, &[KeyCode::Char('z')]);
        assert!(shown(&input).is_empty());
        type_keys(&mut input, &[KeyCode::Enter]);
        assert_eq!(
            input.text, "Srz",
            "nothing chosen: the line keeps what was typed"
        );

        // Typed before ↑: it filters too.
        let mut typed = history3("");
        type_keys(
            &mut typed,
            &[KeyCode::Char('h'), KeyCode::Char('o'), KeyCode::Up],
        );
        assert_eq!(shown(&typed), ["/home/two"]);
    }

    #[test]
    fn a_text_put_in_ready_does_not_filter() {
        let mut input = history3("/elsewhere");
        type_keys(&mut input, &[KeyCode::Up]);
        assert_eq!(shown(&input).len(), 3);
        type_keys(&mut input, &[KeyCode::Enter]);
        input.set_text("/srv".into());
        type_keys(&mut input, &[KeyCode::Up]);
        assert_eq!(shown(&input).len(), 3);
        type_keys(&mut input, &[KeyCode::Char('/'), KeyCode::Char('o')]);
        assert_eq!(shown(&input), ["/srv/one"]);
    }

    #[test]
    fn delete_forgets_the_entry() {
        let mut input = history3("");
        type_keys(&mut input, &[KeyCode::Up, KeyCode::Up, KeyCode::Delete]);
        assert_eq!(shown(&input), ["/srv/one", "/srv/three"]);
        assert_eq!(chosen(&input), "/srv/three");
        assert_eq!(input.take_forgotten(), ["/home/two"]);
        assert!(input.take_forgotten().is_empty());
        assert_eq!(input.history(), ["/srv/one", "/srv/three"]);
        type_keys(&mut input, &[KeyCode::Delete, KeyCode::Delete]);
        assert!(!input.list_open(), "nothing left: the list closes");
        assert_eq!(input.take_forgotten(), ["/srv/three", "/srv/one"]);
        type_keys(&mut input, &[KeyCode::Up]);
        assert!(!input.list_open());
    }

    #[test]
    fn down_and_an_empty_history_do_nothing() {
        let mut input = history3("draft");
        type_keys(&mut input, &[KeyCode::Down]);
        assert_eq!(input.text, "draft");
        assert!(!input.list_open());

        let mut empty = LineInput::new("x");
        type_keys(&mut empty, &[KeyCode::Up]);
        assert!(!empty.list_open());
        assert_eq!(empty.text, "x");
    }

    #[test]
    fn page_keys_move_by_pages() {
        let history: Vec<String> = (0..30).map(|i| format!("e{i}")).collect();
        let mut input = LineInput::new("").with_history(history);
        type_keys(&mut input, &[KeyCode::Up, KeyCode::PageUp]);
        assert_eq!(chosen(&input), "e19");
        type_keys(
            &mut input,
            &[KeyCode::PageUp, KeyCode::PageUp, KeyCode::PageUp],
        );
        assert_eq!(chosen(&input), "e0");
        type_keys(&mut input, &[KeyCode::PageDown]);
        assert_eq!(chosen(&input), "e10");
    }

    #[test]
    fn handles_multibyte_chars() {
        let mut input = LineInput::new("ação");
        type_keys(&mut input, &[KeyCode::Left, KeyCode::Backspace]);
        assert_eq!(input.text, "aço");
    }

    #[test]
    fn submit_and_cancel() {
        let mut input = LineInput::new("");
        let key = |c| KeyEvent::new(c, KeyModifiers::NONE);
        assert_eq!(input.handle_key(key(KeyCode::Enter)), LineEvent::Submit);
        assert_eq!(input.handle_key(key(KeyCode::Esc)), LineEvent::Cancel);
    }
}
