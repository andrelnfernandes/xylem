//! The Usage view (Alt+W): opening it and its keys.

use super::*;
use crate::usage::Item;

impl App {
    /// Logs the branch of the current directory, without going into other
    /// disks mounted inside it, and opens the view on it.
    pub(crate) fn open_usage(&mut self) {
        let id = self.current();
        let device = self.tree.device(id);
        self.log_for_view(id, LogJob::one_disk(id, device), AfterLog::Usage(id));
    }

    pub(crate) fn usage_key(&mut self, key: KeyEvent) {
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        if key.code == KeyCode::Esc || alt && matches!(key.code, KeyCode::Char('w' | 'W')) {
            self.usage = None;
            return;
        }
        let pane = &mut self.panes[self.active];
        let Some(view) = pane.usage.as_mut() else {
            return;
        };
        let tree = &pane.tree;
        let len = view.rows(tree).len();
        let page = view.page.max(1) as isize;
        let ctrl_or_alt = key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        match key.code {
            KeyCode::Up => view.move_by(-1, len),
            KeyCode::Down => view.move_by(1, len),
            KeyCode::PageUp => view.move_by(-page, len),
            KeyCode::PageDown => view.move_by(page, len),
            KeyCode::Home => view.move_by(-(len as isize), len),
            KeyCode::End => view.move_by(len as isize, len),
            KeyCode::Enter | KeyCode::Right => {
                let Some(row) = view.selected(tree) else {
                    return;
                };
                let Item::Dir(id) = row.item else {
                    self.usage_view_file();
                    return;
                };
                if row.mark == '>' {
                    // Another disk: an analysis of its own.
                    self.usage = None;
                    let device = self.tree.device(id);
                    self.start_job(LogJob::one_disk(id, device), AfterLog::Usage(id));
                } else {
                    view.enter(id);
                }
            }
            KeyCode::Backspace | KeyCode::Left => {
                if !view.back(tree) {
                    self.message = Some("This is where the analysis started".into());
                }
            }
            KeyCode::Char('\\') => {
                let dir = view.dir;
                self.usage = None;
                self.tree.reveal(dir);
                self.refresh_rows(dir);
                self.select_node(dir);
                self.focus = Focus::Tree;
            }
            KeyCode::Char('t' | 'T') => self.usage_tag(true, ctrl_or_alt),
            KeyCode::Char('u' | 'U') => self.usage_tag(false, ctrl_or_alt),
            KeyCode::Char('v' | 'V') if !ctrl_or_alt => self.usage_view_file(),
            KeyCode::Delete => self.usage_delete(ctrl_or_alt, shift),
            KeyCode::Char('d' | 'D') if ctrl_or_alt => self.usage_delete(true, false),
            KeyCode::Char('p' | 'P') if alt => self.usage_prune(),
            KeyCode::Char('d' | 'D') => {
                let dir = view.dir;
                self.usage = None;
                self.duplicates_of(dir);
            }
            KeyCode::Char('a' | 'A') => view.apparent = !view.apparent,
            KeyCode::Char('c' | 'C') => view.counts = !view.counts,
            KeyCode::Char('s' | 'S') => {
                view.order = view.order.next();
                view.list.select(Some(0));
                self.message = Some(format!("Sorted by {}", view.order.label()));
            }
            KeyCode::F(3) | KeyCode::F(2) => {
                let dir = view.dir;
                self.tree.rescan(dir);
                let device = self.tree.device(dir);
                self.start_job(LogJob::one_disk(dir, device), AfterLog::Refresh);
            }
            _ => {
                // What makes sense with the view open: windows, the
                // terminal, settings and quitting.
                if let Some(cmd) = self.keymap.lookup(key, self.focus)
                    && matches!(
                        cmd,
                        Command::Split
                            | Command::SwitchWindow
                            | Command::SwitchPane
                            | Command::Terminal
                            | Command::TerminalFull
                            | Command::Settings
                            | Command::Quit
                            | Command::QuitChangeDir
                    )
                {
                    self.apply(cmd);
                }
            }
        }
    }

    /// Tags or untags the row under the cursor and goes down, or everything
    /// in the folder listed (`all`).
    fn usage_tag(&mut self, tag: bool, all: bool) {
        let pane = &mut self.panes[self.active];
        let Some(view) = pane.usage.as_mut() else {
            return;
        };
        let tree = &pane.tree;
        let refs = if all {
            let dir = view.dir;
            view.files_below(tree, dir)
        } else {
            let Some(row) = view.selected(tree) else {
                return;
            };
            if row.mark == '>' {
                self.message = Some("Another disk: it is not part of this analysis".into());
                return;
            }
            let refs = match row.item {
                Item::Dir(id) => view.files_below(tree, id),
                Item::File(idx) => vec![FileRef { dir: view.dir, idx }],
            };
            let len = view.rows(tree).len();
            view.move_by(1, len);
            refs
        };
        pane.set_tags(&refs, |_| tag);
    }

    fn usage_view_file(&mut self) {
        let pane = &mut self.panes[self.active];
        let Some(view) = pane.usage.as_mut() else {
            return;
        };
        let Some(row) = view.selected(&pane.tree) else {
            return;
        };
        let Item::File(idx) = row.item else {
            self.message = Some("A folder: Enter goes into it".into());
            return;
        };
        let file = FileRef { dir: view.dir, idx };
        let (path, size) = (pane.file_path(file), pane.file(file).size);
        self.tagged_view = None;
        match self.load_viewer(path, size) {
            Ok(v) => self.viewer = Some(v),
            Err(e) => self.message = Some(format!("Cannot view file: {e}")),
        }
    }

    /// Deleting and Prune work on files of this machine only, as in the file
    /// window.
    fn usage_can_write(&mut self) -> bool {
        if self.archive.is_some() {
            self.message = Some("Archives are read-only (use Copy to extract)".into());
            false
        } else if self.remote.is_some() {
            self.message =
                Some("Read-only on a remote machine for now: Copy brings files here".into());
            false
        } else {
            true
        }
    }

    /// Asks about deleting the row under the cursor, or the files tagged in
    /// the folder listed.
    fn usage_delete(&mut self, tagged: bool, permanent: bool) {
        if !self.usage_can_write() {
            return;
        }
        let to_trash = !permanent && self.options.delete_to_trash;
        let pane = &mut self.panes[self.active];
        let Some(view) = pane.usage.as_mut() else {
            return;
        };
        let tree = &pane.tree;
        if tagged {
            let dir = view.dir;
            let sources: Vec<Source> = view
                .files_below(tree, dir)
                .into_iter()
                .filter(|&f| pane.file(f).tagged)
                .map(|f| Source {
                    path: pane.file_path(f),
                    size: pane.file(f).size,
                })
                .collect();
            if sources.is_empty() {
                self.message = Some("Nothing tagged here: T tags a file or a folder".into());
            } else {
                self.ask_delete_files(sources, to_trash);
            }
            return;
        }
        let Some(row) = view.selected(tree) else {
            return;
        };
        match row.item {
            Item::File(idx) => {
                let file = FileRef { dir: view.dir, idx };
                let source = Source {
                    path: pane.file_path(file),
                    size: pane.file(file).size,
                };
                self.ask_delete_files(vec![source], to_trash);
            }
            Item::Dir(_) if row.mark == '>' => {
                self.message = Some("Another disk: it is not deleted from here".into());
            }
            Item::Dir(id) if row.mark == 'e' => self.ask_delete_dir_with(id, to_trash),
            Item::Dir(id) if view.holds_other_disk(tree, id) => {
                self.message =
                    Some("Another disk is mounted inside: Alt+P deletes all but that".into());
            }
            Item::Dir(id) => {
                let dir = tree.path(id);
                self.ask_delete_folders(vec![dir], to_trash);
            }
        }
    }

    /// Prune of the folder under the cursor. Its branch is logged already,
    /// and other disks stay out of it.
    fn usage_prune(&mut self) {
        if !self.usage_can_write() {
            return;
        }
        let pane = &mut self.panes[self.active];
        let Some(view) = pane.usage.as_mut() else {
            return;
        };
        let Some(row) = view.selected(&pane.tree) else {
            return;
        };
        match row.item {
            Item::File(_) => {
                self.message = Some("Prune works on folders; Del deletes a file".into())
            }
            Item::Dir(_) if row.mark == '>' => {
                self.message = Some("Another disk: it is not deleted from here".into());
            }
            Item::Dir(id) => {
                let device = self.tree.device(id);
                self.start_job(LogJob::one_disk(id, device), AfterLog::Prune(id));
            }
        }
    }
}
