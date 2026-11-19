//! The mouse (docs/plans/mouse.md): what a click, a double click, a drag or
//! the wheel does, looked up in the map of the last frame drawn
//! (`App::hits`). A click picks a row or stands for a key; the key goes
//! through `handle_key`, as the keyboard's does.

use ratatui::crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use super::context_menu::MenuTarget;
use super::*;
use crate::mouse::{DOUBLE_CLICK, Hit, List, Own, Owner, Spot, WHEEL_LINES};

/// What the left button is dragging, from the click that began it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Drag {
    /// The cursor of a list follows the pointer; `area` is where its rows
    /// are, so that past its edges the list scrolls.
    Rows { list: List, area: Rect },
    /// A button held down on the program of a terminal (`area` is its
    /// screen): the moves and the release go to it, even off the panel.
    Pty { area: Rect },
    /// The thumb of a scroll bar (`area` is its track).
    Thumb {
        list: List,
        area: Rect,
        len: usize,
        cols: usize,
    },
}

/// The last click, for a double click: when, where and with which button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LastClick {
    pub at: Instant,
    pub x: u16,
    pub y: u16,
    pub button: MouseButton,
}

impl App {
    pub fn handle_mouse(&mut self, ev: MouseEvent, now: Instant) {
        // A terminal left with the mouse on by another program.
        if !self.options.mouse {
            return;
        }
        let hit = self.hits.at(ev.column, ev.row);
        let mods = ev.modifiers;
        if self.context_menu.is_some() {
            return self.menu_mouse(ev.kind, hit);
        }
        if let Some(Drag::Pty { area }) = self.drag
            && matches!(ev.kind, MouseEventKind::Drag(_) | MouseEventKind::Up(_))
        {
            if matches!(ev.kind, MouseEventKind::Up(_)) {
                self.drag = None;
            }
            return self.pty_mouse(ev, area);
        }
        if let Some(h) = hit
            && h.spot == Spot::Pty
            && self.splash.is_none()
        {
            self.last_click = None;
            self.drag =
                matches!(ev.kind, MouseEventKind::Down(_)).then_some(Drag::Pty { area: h.area });
            return self.pty_mouse(ev, h.area);
        }
        match ev.kind {
            MouseEventKind::Down(button) => {
                let double = self.last_click.is_some_and(|c| {
                    c.button == button
                        && (c.x, c.y) == (ev.column, ev.row)
                        && now.saturating_duration_since(c.at) <= DOUBLE_CLICK
                });
                // A third click starts again: it is not a second double.
                self.last_click = (!double).then_some(LastClick {
                    at: now,
                    x: ev.column,
                    y: ev.row,
                    button,
                });
                self.drag = None;
                match button {
                    MouseButton::Left if mods.contains(KeyModifiers::CONTROL) => {
                        self.tag_click(hit)
                    }
                    MouseButton::Left if mods.contains(KeyModifiers::SHIFT) => {
                        self.range_click(hit)
                    }
                    MouseButton::Left => self.click(hit, double),
                    MouseButton::Middle => self.tag_click(hit),
                    MouseButton::Right => self.right_click(hit),
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => self.drag_to(ev.column, ev.row),
            MouseEventKind::Up(_) => self.drag = None,
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                let up = ev.kind == MouseEventKind::ScrollUp;
                if mods.contains(KeyModifiers::SHIFT) {
                    self.wheel_sideways(hit, up);
                } else {
                    self.wheel(hit, up);
                }
            }
            MouseEventKind::ScrollLeft => self.wheel_sideways(hit, true),
            MouseEventKind::ScrollRight => self.wheel_sideways(hit, false),
            _ => {}
        }
    }

    /// The mouse over the screen `area` of the program in the framed editor
    /// or the built-in terminal, which has the keys.
    fn pty_mouse(&mut self, ev: MouseEvent, area: Rect) {
        let col = ev.column.saturating_sub(area.x);
        let row = ev.row.saturating_sub(area.y);
        let term = match (&mut self.editor_pty, &mut self.terminal) {
            (Some(ed), _) => &mut ed.term,
            (None, Some(t)) if self.terminal_focus => t,
            _ => return,
        };
        term.send_mouse(ev, col, row, WHEEL_LINES);
    }

    /// Something in front of the windows that the keys go to (a question, a
    /// prompt, work in progress): the windows behind it take no clicks.
    fn windows_blocked(&self) -> bool {
        self.prompt.is_some()
            || self.confirm.is_some()
            || self.op.is_some()
            || self.job.is_some()
            || self.batch_preview.is_some()
            || self.batch_job.is_some()
            || self.duplicates_job.is_some()
            || self.cloud_log.is_some()
            || self.archive_job.is_some()
            || self.index_job.is_some()
            || self.search_job.is_some()
            || self.search_box.is_some()
            || self.content_job.as_ref().is_some_and(|(_, _, go)| *go)
    }

    fn press(&mut self, code: KeyCode) {
        self.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
    }

    fn click(&mut self, hit: Option<Hit>, double: bool) {
        // Like the first key, the first click only closes the start screen.
        if self.splash.take().is_some() {
            return;
        }
        // The framed editor has the whole screen: only its panel takes the
        // mouse (`pty_mouse`).
        if self.editor_pty.is_some() {
            return;
        }
        let Some(hit) = hit else { return };
        match hit.spot {
            // Each click on a key is a press of it, as fast as they come.
            Spot::Key(key) => self.handle_key(key),
            // Only with the context menu open, which `menu_mouse` handles.
            // The program of a terminal: `pty_mouse`, before it gets here.
            Spot::Outside | Spot::Pty => {}
            Spot::Link(id) => {
                if let Some(help) = &mut self.help {
                    help.follow(id);
                }
            }
            Spot::Scroll => {}
            Spot::Window(list) => {
                self.click_list(list, None, double);
            }
            Spot::Rows { list, .. } | Spot::Grid { list, .. } => {
                if self.click_list(list, hit.item, double) {
                    self.drag = Some(Drag::Rows {
                        list,
                        area: hit.area,
                    });
                }
            }
            Spot::Marker { list, row } => self.click_marker(list, row),
            Spot::Scrollbar {
                list,
                len,
                cols,
                thumb,
                ..
            } => self.click_scrollbar(list, hit, len, cols, thumb),
            Spot::Item { list, .. } => {
                if let Some(i) = hit.item {
                    self.click_item(list, i, double);
                }
            }
            Spot::Cell { row, col } => self.click_cell(row, col, double),
            Spot::Prompt { start } => self.click_prompt(usize::from(hit.x.saturating_sub(start))),
        }
    }

    /// The pane and whether the window may take the click: a window of the
    /// main screen, not behind a question; `Some` with its index.
    fn pane_for(&mut self, owner: Owner) -> Option<&mut Pane> {
        match owner {
            Owner::Picker(i) => {
                let picker = self.picker.as_mut()?;
                if i < picker.panes.len() {
                    picker.active = i;
                }
                Some(picker.pane_mut())
            }
            Owner::Main(i) => {
                if self.windows_blocked() || i >= self.panes.len() {
                    return None;
                }
                self.message = None;
                // The tree above the terminal's panel: back to Xylem, as
                // with Alt+X.
                if self.terminal_focus {
                    self.hide_terminal();
                }
                self.active = i;
                Some(&mut self.panes[i])
            }
        }
    }

    /// A click on a window, on its item `item` if there is one there.
    /// Returns whether a drag may go on from it.
    fn click_list(&mut self, list: List, item: Option<usize>, double: bool) -> bool {
        let owner = match list {
            List::Tree(o) | List::Files(o) => o,
        };
        let Some(pane) = self.pane_for(owner) else {
            return false;
        };
        // Usage and Duplicates have rows of their own (`Own`).
        if pane.usage.is_some() || pane.duplicates.is_some() {
            return false;
        }
        match list {
            List::Tree(_) => {
                if let Some(row) = item {
                    pane.select_row(row);
                }
            }
            List::Files(_) => {
                pane.focus = Focus::Files;
                if let Some(file) = item {
                    pane.file_state.select(Some(file));
                }
            }
        }
        if let Owner::Main(_) = owner {
            self.sync_autoview();
        }
        if double && item.is_some() {
            match (list, owner) {
                // The picker: Enter chooses the folder.
                (_, Owner::Picker(_)) => self.press(KeyCode::Enter),
                // A folder: Enter, which goes to its files.
                (List::Tree(_), _) => self.press(KeyCode::Enter),
                // A file: the viewer, never a program of the system (a file
                // may be hostile).
                (List::Files(_), _) => self.apply(Command::View),
            }
            return false;
        }
        true
    }

    /// The mark before a folder: `+` expands it, `?` logs it, the `─` of an
    /// open one with subfolders closes it.
    fn click_marker(&mut self, list: List, row: usize) {
        let owner = match list {
            List::Tree(o) | List::Files(o) => o,
        };
        let Some(pane) = self.pane_for(owner) else {
            return;
        };
        let Some(r) = pane.rows.get(row) else { return };
        let mark = r.prefix.chars().last();
        let id = r.id;
        let open = pane.tree.node(id).expanded && pane.tree.has_children(id);
        pane.select_row(row);
        let cmd = match mark {
            Some('+' | '?') => Command::Expand,
            _ if open => Command::Collapse,
            _ => return,
        };
        match owner {
            Owner::Main(_) => self.apply(cmd),
            Owner::Picker(_) => {
                let key = if cmd == Command::Expand {
                    KeyCode::Right
                } else {
                    KeyCode::Left
                };
                self.press(key);
            }
        }
    }

    /// A click on a scroll bar: above or below the thumb, a page; on the
    /// thumb, it can be dragged.
    fn click_scrollbar(
        &mut self,
        list: List,
        hit: Hit,
        len: usize,
        cols: usize,
        thumb: (u16, u16),
    ) {
        let area = hit.area;
        let y = hit.y.saturating_sub(area.y);
        let cmd = if y < thumb.0 {
            Command::PageUp
        } else if y >= thumb.0 + thumb.1 {
            Command::PageDown
        } else {
            self.drag = Some(Drag::Thumb {
                list,
                area,
                len,
                cols,
            });
            return;
        };
        self.move_list(list, |pane, list| match list {
            List::Tree(_) => pane.apply_tree(cmd),
            List::Files(_) => pane.apply_files(cmd),
        });
    }

    /// Runs `f` on the pane of `list`, without changing the focus, and
    /// keeps Autoview in step.
    fn move_list(&mut self, list: List, f: impl Fn(&mut Pane, List)) {
        let owner = match list {
            List::Tree(o) | List::Files(o) => o,
        };
        let pane = match owner {
            Owner::Picker(i) => match self.picker.as_mut().and_then(|p| p.panes.get_mut(i)) {
                Some(pane) => pane,
                None => return,
            },
            Owner::Main(i) => {
                if self.windows_blocked() || i >= self.panes.len() {
                    return;
                }
                &mut self.panes[i]
            }
        };
        if pane.usage.is_some() || pane.duplicates.is_some() {
            return;
        }
        f(pane, list);
        if let Owner::Main(_) = owner {
            self.sync_autoview();
        }
    }

    /// The left button moving while held: the cursor follows it, and past
    /// the edges of the list it scrolls one row a move.
    fn drag_to(&mut self, x: u16, y: u16) {
        let Some(drag) = self.drag else { return };
        match drag {
            Drag::Rows { list, area } => {
                let hit = self.hits.at(x, y);
                let over = hit.filter(|h| match h.spot {
                    Spot::Rows { list: l, .. } | Spot::Grid { list: l, .. } => l == list,
                    _ => false,
                });
                if let Some(Hit { item: Some(i), .. }) = over {
                    self.move_list(list, |pane, list| match list {
                        List::Tree(_) => pane.select_row(i),
                        List::Files(_) => pane.file_state.select(Some(i)),
                    });
                } else if y < area.y || y >= area.bottom() {
                    let cmd = if y < area.y {
                        Command::Up
                    } else {
                        Command::Down
                    };
                    self.move_list(list, |pane, list| match list {
                        List::Tree(_) => pane.apply_tree(cmd),
                        List::Files(_) => pane.apply_files(cmd),
                    });
                }
            }
            Drag::Thumb {
                list,
                area,
                len,
                cols,
            } => {
                let row = y.saturating_sub(area.y);
                let item = crate::mouse::bar_item(row, area.height, len, cols);
                self.move_list(list, |pane, list| match list {
                    List::Tree(_) => pane.select_row(item.min(pane.rows.len().saturating_sub(1))),
                    List::Files(_) => {
                        let last = pane.files.len().saturating_sub(1);
                        pane.file_state.select(Some(item.min(last)));
                    }
                });
            }
            // Handled before it gets here (`handle_mouse`).
            Drag::Pty { .. } => {}
        }
    }

    /// The mouse while the context menu is open: it takes every click. A
    /// line runs; a click off the menu closes it and does nothing else.
    fn menu_mouse(&mut self, kind: MouseEventKind, hit: Option<Hit>) {
        let on_line = match hit {
            Some(Hit {
                spot: Spot::Item {
                    list: Own::Menu, ..
                },
                item,
                ..
            }) => Some(item),
            _ => None,
        };
        match kind {
            MouseEventKind::Down(_) => match on_line {
                Some(Some(i)) => self.run_menu_line(i),
                // The border of the menu.
                Some(None) => {}
                None if hit.is_some_and(|h| h.spot == Spot::Scroll) => {}
                None => self.context_menu = None,
            },
            MouseEventKind::ScrollUp => self.press(KeyCode::Up),
            MouseEventKind::ScrollDown => self.press(KeyCode::Down),
            _ => {}
        }
    }

    /// The right button: the cursor goes to the item under the pointer (as a
    /// click), and the menu of that item opens there.
    fn right_click(&mut self, hit: Option<Hit>) {
        if self.splash.take().is_some() || self.editor_pty.is_some() {
            return;
        }
        let Some(hit) = hit else { return };
        let at = (hit.x, hit.y);
        let target = match hit.spot {
            Spot::Rows {
                list: list @ List::Tree(Owner::Main(_)),
                ..
            } if hit.item.is_some() => {
                self.click_list(list, hit.item, false);
                MenuTarget::Dir
            }
            Spot::Rows {
                list: list @ List::Files(Owner::Main(_)),
                ..
            }
            | Spot::Grid {
                list: list @ List::Files(Owner::Main(_)),
                ..
            }
            | Spot::Window(list @ List::Files(Owner::Main(_))) => {
                if !self.click_list(list, hit.item, false) && hit.item.is_some() {
                    return;
                }
                let on_file = hit.item.is_some() && self.focus == Focus::Files;
                if self.windows_blocked() || self.usage.is_some() || self.duplicates.is_some() {
                    return;
                }
                if on_file {
                    MenuTarget::File
                } else {
                    MenuTarget::FileWindow
                }
            }
            Spot::Scroll | Spot::Cell { .. }
                if self.viewer.is_some()
                    && self.help.is_none()
                    && self.prompt.is_none()
                    && self.confirm.is_none() =>
            {
                if let Spot::Cell { row, col } = hit.spot {
                    self.click_cell(row, col, false);
                }
                MenuTarget::Viewer
            }
            _ => return,
        };
        if self.windows_blocked() {
            return;
        }
        self.open_menu(target, at);
    }

    /// The middle button, or Ctrl with the left one: tags or untags the file
    /// under the pointer without moving the cursor; on a folder of the tree,
    /// picks it and tags its files (or untags them, when all were tagged).
    fn tag_click(&mut self, hit: Option<Hit>) {
        if self.splash.take().is_some() {
            return;
        }
        let Some(Hit {
            spot: Spot::Rows { list, .. } | Spot::Grid { list, .. },
            item: Some(i),
            ..
        }) = hit
        else {
            return;
        };
        let owner = match list {
            List::Tree(o) | List::Files(o) => o,
        };
        if let Owner::Picker(_) = owner {
            return;
        }
        let Some(pane) = self.pane_for(owner) else {
            return;
        };
        match list {
            List::Files(_) => {
                if let Some(&f) = pane.files.get(i) {
                    pane.set_tags(&[f], |t| !t);
                }
            }
            List::Tree(_) => {
                pane.select_row(i);
                let all = !pane.files.is_empty() && pane.files.iter().all(|&f| pane.file(f).tagged);
                let cmd = if all { Command::Untag } else { Command::Tag };
                self.apply(cmd);
            }
        }
    }

    /// Shift with the left button on a file: tags the files from the
    /// cursor to it, and the cursor goes there.
    fn range_click(&mut self, hit: Option<Hit>) {
        if self.splash.take().is_some() {
            return;
        }
        let Some(Hit {
            spot: Spot::Rows { list, .. } | Spot::Grid { list, .. },
            item: Some(i),
            ..
        }) = hit
        else {
            return self.click(hit, false);
        };
        let List::Files(owner @ Owner::Main(_)) = list else {
            return self.click(hit, false);
        };
        let Some(pane) = self.pane_for(owner) else {
            return;
        };
        let from = if pane.focus == Focus::Files {
            pane.file_state.selected().unwrap_or(0)
        } else {
            i
        };
        let (a, b) = (from.min(i), from.max(i));
        let refs: Vec<FileRef> = pane.files.get(a..=b).map(<[_]>::to_vec).unwrap_or_default();
        pane.set_tags(&refs, |_| true);
        pane.focus = Focus::Files;
        pane.file_state.select(Some(i));
        self.sync_autoview();
    }

    /// A row of a list of a screen of its own.
    fn click_item(&mut self, list: Own, i: usize, double: bool) {
        match list {
            Own::Menu => self.run_menu_line(i),
            Own::Settings => {
                // As its letter: picks and changes the option (nothing is
                // saved before F3).
                let Some(s) = &self.settings else { return };
                if double {
                    return;
                }
                if let Some(&(letter, ..)) = s.items().get(i) {
                    self.handle_key(KeyEvent::new(
                        KeyCode::Char(letter.to_ascii_lowercase()),
                        KeyModifiers::NONE,
                    ));
                }
            }
            Own::Theme => {
                let Some(picker) = self.settings.as_ref().and_then(|s| s.picker.as_ref()) else {
                    return;
                };
                // Step by step, so that each theme is previewed as with ↑↓.
                let at = picker.index;
                let code = if i < at { KeyCode::Up } else { KeyCode::Down };
                for _ in 0..at.abs_diff(i) {
                    self.press(code);
                }
                if double {
                    self.press(KeyCode::Enter);
                }
            }
            Own::Bookmarks => {
                if self.prompt.is_some() {
                    return;
                }
                if let Some(view) = &mut self.bookmarks_view {
                    view.selected = i;
                }
                if double {
                    self.press(KeyCode::Enter);
                }
            }
            Own::History => {
                let Some(list) = self.prompt.as_mut().and_then(|p| p.input.list_mut()) else {
                    return;
                };
                list.cursor = i;
                self.press(KeyCode::Enter);
            }
            Own::SearchResults => {
                if let Some(sb) = &mut self.search_box {
                    sb.selected = i;
                }
                if double {
                    self.press(KeyCode::Enter);
                }
            }
            Own::Usage(p) | Own::Duplicates(p) => {
                if self.windows_blocked() || p >= self.panes.len() {
                    return;
                }
                if self.terminal_focus {
                    self.hide_terminal();
                }
                self.active = p;
                let pane = &mut self.panes[p];
                if let Some(view) = &mut pane.usage {
                    view.list.select(Some(i));
                } else if let Some(view) = &mut pane.duplicates {
                    view.at = i;
                }
                if double {
                    self.press(KeyCode::Enter);
                }
            }
        }
    }

    /// A cell of the viewer's table; twice, its Enter (the record).
    fn click_cell(&mut self, row: usize, col: usize, double: bool) {
        if self.prompt.is_some() || self.confirm.is_some() {
            return;
        }
        let Some(grid) = self.viewer.as_mut().and_then(|v| v.table.as_mut()) else {
            return;
        };
        if row < grid.visible_row_count() && col < grid.headers.len() {
            grid.cursor_row = row;
            grid.cursor_col = col;
        }
        if double {
            self.press(KeyCode::Enter);
        }
    }

    /// A click on the text of a prompt (or of a text option of F12): the
    /// cursor goes to that character.
    fn click_prompt(&mut self, at: usize) {
        let input = match (&mut self.prompt, &mut self.settings) {
            (Some(p), _) => &mut p.input,
            (None, Some(s)) => match &mut s.editing {
                Some((_, input)) => input,
                None => return,
            },
            _ => return,
        };
        input.cursor = at.min(input.text.chars().count());
    }

    /// The wheel moves the cursor of the list under the pointer, without
    /// giving it the focus; over text (viewer, help) or a list of a screen
    /// of its own, it stands for ↑ and ↓.
    fn wheel(&mut self, hit: Option<Hit>, up: bool) {
        if self.splash.is_some() || self.editor_pty.is_some() {
            return;
        }
        let Some(hit) = hit else { return };
        let cmd = if up { Command::Up } else { Command::Down };
        match hit.spot {
            Spot::Window(list)
            | Spot::Rows { list, .. }
            | Spot::Grid { list, .. }
            | Spot::Marker { list, .. }
            | Spot::Scrollbar { list, .. } => self.wheel_list(list, cmd),
            Spot::Scroll | Spot::Link(_) | Spot::Item { .. } | Spot::Cell { .. } => {
                let code = if up { KeyCode::Up } else { KeyCode::Down };
                self.wheel_keys(code);
            }
            Spot::Key(_) | Spot::Prompt { .. } | Spot::Outside | Spot::Pty => {}
        }
    }

    fn wheel_list(&mut self, list: List, cmd: Command) {
        // Usage and Duplicates: their own ↑ and ↓, if in front.
        if let List::Files(Owner::Main(i)) = list
            && let Some(pane) = self.panes.get(i)
            && (pane.usage.is_some() || pane.duplicates.is_some())
        {
            if i == self.active && !self.terminal_focus && !self.windows_blocked() {
                let code = if cmd == Command::Up {
                    KeyCode::Up
                } else {
                    KeyCode::Down
                };
                self.wheel_keys(code);
            }
            return;
        }
        self.move_list(list, |pane, list| {
            for _ in 0..WHEEL_LINES {
                match list {
                    List::Tree(_) => pane.apply_tree(cmd),
                    List::Files(_) => pane.apply_files(cmd),
                }
            }
        });
    }

    /// The wheel as keys, to what is in front: never to a question or a
    /// prompt, which would take ↑ as their own (the history) or close.
    fn wheel_keys(&mut self, code: KeyCode) {
        let history_open = self.prompt.as_ref().is_some_and(|p| p.input.list_open());
        if self.confirm.is_some() || (self.prompt.is_some() && !history_open) {
            return;
        }
        for _ in 0..WHEEL_LINES {
            self.press(code);
        }
    }

    /// Shift+wheel or the wheel's sideways tilt: across, in the viewer only
    /// (in the help, ← is "back").
    fn wheel_sideways(&mut self, hit: Option<Hit>, left: bool) {
        if self.viewer.is_none() || self.help.is_some() {
            return;
        }
        if hit.is_some_and(|h| matches!(h.spot, Spot::Scroll | Spot::Cell { .. })) {
            let code = if left { KeyCode::Left } else { KeyCode::Right };
            if self.confirm.is_none() && self.prompt.is_none() {
                self.press(code);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
    use ratatui::{Terminal, backend::TestBackend};
    use std::fs::{create_dir_all, write};
    use std::time::Duration;

    fn press_key(app: &mut App, code: KeyCode) {
        app.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
    }

    const W: u16 = 120;
    const H: u16 = 30;

    /// root/{a.txt, b.txt, c.txt}, and the folders d1 to d6.
    fn sample() -> (tempfile::TempDir, App) {
        let tmp = crate::testing::tempdir();
        let p = tmp.path();
        for d in 1..=6 {
            create_dir_all(p.join(format!("d{d}"))).unwrap();
        }
        for f in ["a.txt", "b.txt", "c.txt"] {
            write(p.join(f), f).unwrap();
        }
        let mut app = App::new(p.to_path_buf());
        app.options.mouse = true;
        (tmp, app)
    }

    /// Draws the app as the screen would, and returns the text of each row.
    fn draw(app: &mut App) -> Vec<String> {
        let mut term = Terminal::new(TestBackend::new(W, H)).unwrap();
        term.draw(|f| crate::ui::draw(f, app)).unwrap();
        let buf = term.backend().buffer();
        (0..H)
            .map(|y| (0..W).map(|x| buf[(x, y)].symbol()).collect())
            .collect()
    }

    /// Where `text` first is on screen, at or below row `from`.
    fn find(screen: &[String], text: &str, from: usize) -> (u16, u16) {
        for (y, row) in screen.iter().enumerate().skip(from) {
            if let Some(i) = row.find(text) {
                let x = row[..i].chars().count();
                return (x as u16, y as u16);
            }
        }
        panic!("{text:?} not on screen:\n{}", screen.join("\n"));
    }

    fn mouse(app: &mut App, kind: MouseEventKind, (x, y): (u16, u16)) {
        let ev = MouseEvent {
            kind,
            column: x,
            row: y,
            modifiers: KeyModifiers::NONE,
        };
        app.handle_mouse(ev, Instant::now());
    }

    fn click(app: &mut App, at: (u16, u16)) {
        mouse(app, MouseEventKind::Down(MouseButton::Left), at);
    }

    /// A press of `button` with `mods`, at `now`.
    fn press_at(
        app: &mut App,
        button: MouseButton,
        mods: KeyModifiers,
        (x, y): (u16, u16),
        now: Instant,
    ) {
        let ev = MouseEvent {
            kind: MouseEventKind::Down(button),
            column: x,
            row: y,
            modifiers: mods,
        };
        app.handle_mouse(ev, now);
    }

    fn tagged(app: &App) -> Vec<String> {
        app.files
            .iter()
            .filter(|&&f| app.file(f).tagged)
            .map(|&f| app.file(f).name.to_string_lossy().into_owned())
            .collect()
    }

    fn current_name(app: &App) -> String {
        let path = app.current_path();
        path.file_name().unwrap().to_string_lossy().into_owned()
    }

    #[test]
    fn a_click_on_the_tree_picks_the_folder() {
        let (_tmp, mut app) = sample();
        let screen = draw(&mut app);
        click(&mut app, find(&screen, "d3", 1));
        assert_eq!(current_name(&app), "d3");
        assert_eq!(app.focus, Focus::Tree);
    }

    #[test]
    fn a_click_on_a_file_goes_to_the_file_window() {
        let (_tmp, mut app) = sample();
        let screen = draw(&mut app);
        click(&mut app, find(&screen, "b.txt", 1));
        assert_eq!(app.focus, Focus::Files);
        let f = app.selected_file().unwrap();
        assert_eq!(app.file(f).name, "b.txt");
    }

    #[test]
    fn the_wheel_moves_the_list_under_the_pointer() {
        let (_tmp, mut app) = sample();
        let screen = draw(&mut app);
        // Over the tree: three folders down, the focus stays.
        mouse(&mut app, MouseEventKind::ScrollDown, find(&screen, "d1", 1));
        assert_eq!(app.tree_state.selected(), Some(3));
        // Over the small file window: its cursor moves, the tree's does not.
        let (_tmp2, mut app2) = sample();
        let screen = draw(&mut app2);
        mouse(
            &mut app2,
            MouseEventKind::ScrollDown,
            find(&screen, "a.txt", 1),
        );
        assert_eq!(app2.focus, Focus::Tree);
        assert_eq!(app2.tree_state.selected(), Some(0));
        assert_eq!(app2.file_state.selected(), Some(2));
        mouse(
            &mut app2,
            MouseEventKind::ScrollUp,
            find(&screen, "a.txt", 1),
        );
        assert_eq!(app2.file_state.selected(), Some(0));
    }

    #[test]
    fn a_word_of_the_footer_is_its_key() {
        let (_tmp, mut app) = sample();
        app.options.confirm_quit = true;
        let screen = draw(&mut app);
        // "Quit" in the footer asks, and "Yes" answers.
        click(&mut app, find(&screen, "Quit", H as usize - 3));
        assert!(app.confirm.is_some());
        let screen = draw(&mut app);
        click(&mut app, find(&screen, "Yes", H as usize - 3));
        assert!(app.should_quit);
    }

    #[test]
    fn a_question_holds_the_windows_and_the_wheel() {
        let (_tmp, mut app) = sample();
        app.options.confirm_quit = true;
        app.apply(Command::Quit);
        let screen = draw(&mut app);
        click(&mut app, find(&screen, "d3", 1));
        mouse(&mut app, MouseEventKind::ScrollDown, find(&screen, "d3", 1));
        assert_eq!(app.tree_state.selected(), Some(0));
        assert!(app.confirm.is_some(), "the question is still there");
    }

    #[test]
    fn with_the_option_off_the_mouse_does_nothing() {
        let (_tmp, mut app) = sample();
        app.options.mouse = false;
        let screen = draw(&mut app);
        click(&mut app, find(&screen, "d3", 1));
        assert_eq!(app.tree_state.selected(), Some(0));
    }

    #[test]
    fn a_click_off_the_screen_does_nothing() {
        let (_tmp, mut app) = sample();
        draw(&mut app);
        click(&mut app, (W + 50, H + 50));
        mouse(&mut app, MouseEventKind::ScrollDown, (W + 50, 2));
        assert_eq!(app.tree_state.selected(), Some(0));
    }

    #[test]
    fn the_first_click_only_closes_the_splash() {
        let (_tmp, mut app) = sample();
        draw(&mut app);
        app.splash = Some(Instant::now() + SPLASH);
        click(&mut app, (5, 5));
        assert!(app.splash.is_none());
        assert_eq!(app.tree_state.selected(), Some(0));
    }

    #[test]
    fn a_click_on_the_other_side_of_the_split_makes_it_active() {
        let (_tmp, mut app) = sample();
        app.apply(Command::Split);
        app.active = 1;
        let screen = draw(&mut app);
        // The left window's d2.
        click(&mut app, find(&screen, "d2", 1));
        assert_eq!(app.active, 0);
        assert_eq!(current_name(&app), "d2");
    }

    #[test]
    fn the_help_scrolls_and_follows_a_link() {
        let (_tmp, mut app) = sample();
        app.open_help();
        let topic = app.help.as_ref().unwrap().topic.clone();
        let screen = draw(&mut app);
        let body = find(&screen, " ", 2);
        mouse(&mut app, MouseEventKind::ScrollDown, body);
        assert_eq!(app.help.as_ref().unwrap().scroll, WHEEL_LINES);
        mouse(&mut app, MouseEventKind::ScrollUp, body);
        assert_eq!(app.help.as_ref().unwrap().scroll, 0);
        // The first link on screen.
        let spot = (0..H)
            .flat_map(|y| (0..W).map(move |x| (x, y)))
            .find(|&(x, y)| {
                matches!(
                    app.hits.at(x, y),
                    Some(Hit {
                        spot: Spot::Link(_),
                        ..
                    })
                )
            })
            .expect("a link on screen");
        click(&mut app, spot);
        assert_ne!(app.help.as_ref().unwrap().topic, topic);
    }

    #[test]
    fn the_wheel_scrolls_the_viewer() {
        let (tmp, mut app) = sample();
        let text: String = (0..200).map(|i| format!("line {i}\n")).collect();
        write(tmp.path().join("a.txt"), text).unwrap();
        app.apply(Command::Relog);
        app.focus = Focus::Files;
        app.file_state.select(Some(0));
        app.apply(Command::View);
        assert!(app.viewer.is_some());
        let screen = draw(&mut app);
        mouse(
            &mut app,
            MouseEventKind::ScrollDown,
            find(&screen, "line 1", 1),
        );
        assert_eq!(app.viewer.as_ref().unwrap().top, WHEEL_LINES);
        // A click on Esc in its footer closes it.
        let screen = draw(&mut app);
        click(&mut app, find(&screen, "Esc", H as usize - 2));
        assert!(app.viewer.is_none());
    }

    #[test]
    fn a_double_click_on_a_file_views_it_and_a_slow_one_does_not() {
        let (_tmp, mut app) = sample();
        let screen = draw(&mut app);
        let at = find(&screen, "b.txt", 1);
        let t0 = Instant::now();
        let left = MouseButton::Left;
        let none = KeyModifiers::NONE;
        press_at(&mut app, left, none, at, t0);
        press_at(&mut app, left, none, at, t0 + Duration::from_millis(500));
        assert!(app.viewer.is_none(), "500 ms apart: two clicks");
        let screen = draw(&mut app);
        let at = find(&screen, "b.txt", 1);
        let t1 = t0 + Duration::from_secs(2);
        press_at(&mut app, left, none, at, t1);
        press_at(&mut app, left, none, at, t1 + Duration::from_millis(300));
        assert!(app.viewer.is_some(), "300 ms apart: a double click");
        assert!(app.viewer.as_ref().unwrap().path.ends_with("b.txt"));
    }

    #[test]
    fn a_double_click_on_a_folder_goes_to_its_files() {
        let (_tmp, mut app) = sample();
        let screen = draw(&mut app);
        let at = find(&screen, "d2", 1);
        let t0 = Instant::now();
        let (left, none) = (MouseButton::Left, KeyModifiers::NONE);
        press_at(&mut app, left, none, at, t0);
        press_at(&mut app, left, none, at, t0 + Duration::from_millis(100));
        assert_eq!(current_name(&app), "d2");
        assert_eq!(app.focus, Focus::Files);
    }

    #[test]
    fn the_middle_button_and_ctrl_tag_without_moving_the_cursor() {
        let (_tmp, mut app) = sample();
        app.focus = Focus::Files;
        let screen = draw(&mut app);
        let now = Instant::now();
        let b = find(&screen, "b.txt", 1);
        press_at(&mut app, MouseButton::Middle, KeyModifiers::NONE, b, now);
        assert_eq!(tagged(&app), ["b.txt"]);
        assert_eq!(app.file_state.selected(), Some(0));
        let c = find(&screen, "c.txt", 1);
        let later = now + Duration::from_secs(1);
        press_at(&mut app, MouseButton::Left, KeyModifiers::CONTROL, c, later);
        assert_eq!(tagged(&app), ["b.txt", "c.txt"]);
        // Again: untagged.
        let later = now + Duration::from_secs(2);
        press_at(&mut app, MouseButton::Middle, KeyModifiers::NONE, b, later);
        assert_eq!(tagged(&app), ["c.txt"]);
        assert_eq!(app.file_state.selected(), Some(0));
    }

    #[test]
    fn shift_tags_from_the_cursor_to_the_click() {
        let (_tmp, mut app) = sample();
        app.focus = Focus::Files;
        let screen = draw(&mut app);
        let c = find(&screen, "c.txt", 1);
        let shift = KeyModifiers::SHIFT;
        press_at(&mut app, MouseButton::Left, shift, c, Instant::now());
        assert_eq!(tagged(&app), ["a.txt", "b.txt", "c.txt"]);
        assert_eq!(app.file_state.selected(), Some(2));
    }

    #[test]
    fn the_mark_of_a_folder_opens_and_closes_it() {
        let (tmp, mut app) = sample();
        create_dir_all(tmp.path().join("d1/inner")).unwrap();
        app.apply(Command::Relog);
        let mark = |screen: &[String]| {
            let (x, y) = find(screen, "d1", 1);
            let c = screen[y as usize].chars().nth(x as usize - 1);
            ((x - 1, y), c)
        };
        let shows_inner = |screen: &[String]| screen.iter().any(|r| r.contains("inner"));
        let mut now = Instant::now();
        let mut click_at = |app: &mut App, at| {
            now += Duration::from_secs(1);
            press_at(app, MouseButton::Left, KeyModifiers::NONE, at, now);
        };
        // Not logged: `?` logs it and opens it.
        let screen = draw(&mut app);
        let (at, c) = mark(&screen);
        assert_eq!(c, Some('?'));
        click_at(&mut app, at);
        let screen = draw(&mut app);
        assert!(shows_inner(&screen), "logged and open");
        // Open: `─` closes it, and then it is `+`.
        click_at(&mut app, mark(&screen).0);
        let screen = draw(&mut app);
        assert!(!shows_inner(&screen), "closed");
        let (at, c) = mark(&screen);
        assert_eq!(c, Some('+'));
        click_at(&mut app, at);
        assert!(shows_inner(&draw(&mut app)), "open again");
    }

    #[test]
    fn a_drag_moves_the_cursor_and_scrolls_past_the_edge() {
        let (_tmp, mut app) = sample();
        let screen = draw(&mut app);
        click(&mut app, find(&screen, "d1", 1));
        let drag = |app: &mut App, at| mouse(app, MouseEventKind::Drag(MouseButton::Left), at);
        drag(&mut app, find(&screen, "d4", 1));
        assert_eq!(current_name(&app), "d4");
        // Above the tree: one up a move.
        drag(&mut app, (5, 0));
        assert_eq!(current_name(&app), "d3");
        mouse(&mut app, MouseEventKind::Up(MouseButton::Left), (5, 0));
        drag(&mut app, find(&screen, "d6", 1));
        assert_eq!(current_name(&app), "d3", "released: no drag");
    }

    #[test]
    fn a_click_on_an_option_of_f12_changes_it_and_saves_nothing() {
        let (_tmp, mut app) = sample();
        app.apply(Command::Settings);
        press_key(&mut app, KeyCode::PageDown);
        let screen = draw(&mut app);
        let before = app.settings.as_ref().unwrap().config.options.clock_seconds;
        click(&mut app, find(&screen, "Clock with seconds", 1));
        let s = app.settings.as_ref().unwrap();
        assert_ne!(s.config.options.clock_seconds, before);
        assert_eq!(app.options.clock_seconds, before, "nothing saved");
    }

    #[test]
    fn a_click_on_a_history_entry_puts_it_in_the_prompt() {
        let (_tmp, mut app) = sample();
        app.remember(&PromptKind::Filespec, "*.rs");
        app.remember(&PromptKind::Filespec, "*.txt");
        app.apply(Command::Filespec);
        press_key(&mut app, KeyCode::Up);
        assert!(app.prompt.as_ref().unwrap().input.list_open());
        let screen = draw(&mut app);
        click(&mut app, find(&screen, "*.rs", 1));
        let prompt = app.prompt.as_ref().unwrap();
        assert!(!prompt.input.list_open());
        assert_eq!(prompt.input.text, "*.rs");
    }

    #[test]
    fn a_click_on_the_prompt_moves_its_cursor() {
        let (_tmp, mut app) = sample();
        app.apply(Command::Filespec);
        let prompt = app.prompt.as_mut().unwrap();
        prompt.input = LineInput::new("abcdef");
        let screen = draw(&mut app);
        let (x, y) = find(&screen, "abcdef", H as usize - 3);
        click(&mut app, (x + 2, y));
        assert_eq!(app.prompt.as_ref().unwrap().input.cursor, 2);
        // Past the end: the end.
        let later = Instant::now() + Duration::from_secs(1);
        let none = KeyModifiers::NONE;
        press_at(&mut app, MouseButton::Left, none, (x + 40, y), later);
        assert_eq!(app.prompt.as_ref().unwrap().input.cursor, 6);
    }

    #[test]
    fn the_scroll_bar_pages_and_its_thumb_drags() {
        let tmp = crate::testing::tempdir();
        for i in 0..120 {
            write(tmp.path().join(format!("f{i:03}")), "").unwrap();
        }
        let mut app = App::new(tmp.path().to_path_buf());
        app.options.mouse = true;
        app.focus = Focus::Files;
        app.panes[0].display = crate::pane::FileDisplay::LongNames;
        draw(&mut app);
        // The bar is the right border of the file window.
        let bar = (0..H)
            .flat_map(|y| (0..W).map(move |x| (x, y)))
            .find_map(|(x, y)| match app.hits.at(x, y) {
                Some(Hit {
                    spot: Spot::Scrollbar { thumb, .. },
                    area,
                    ..
                }) => Some((x, area, thumb)),
                _ => None,
            })
            .expect("a scroll bar");
        let (x, area, thumb) = bar;
        // Below the thumb: a page down.
        let below = area.y + thumb.0 + thumb.1 + 1;
        click(&mut app, (x, below));
        let page = app.file_page;
        assert_eq!(app.file_state.selected(), Some(page));
        // The thumb dragged to the bottom: the last file.
        draw(&mut app);
        let Some(Hit {
            spot: Spot::Scrollbar { thumb, .. },
            ..
        }) = app.hits.at(x, area.y)
        else {
            panic!("no bar");
        };
        let later = Instant::now() + Duration::from_secs(1);
        let on_thumb = (x, area.y + thumb.0);
        press_at(
            &mut app,
            MouseButton::Left,
            KeyModifiers::NONE,
            on_thumb,
            later,
        );
        mouse(
            &mut app,
            MouseEventKind::Drag(MouseButton::Left),
            (x, area.bottom() - 1),
        );
        assert_eq!(app.file_state.selected(), Some(119));
    }

    #[test]
    fn a_click_picks_a_row_of_the_usage_view() {
        let (_tmp, mut app) = sample();
        app.apply(Command::Usage);
        assert!(app.usage.is_some());
        let screen = draw(&mut app);
        click(&mut app, find(&screen, "/d4", 1));
        draw(&mut app);
        let shown = app.usage.as_ref().unwrap().shown.clone().unwrap();
        assert_eq!(shown.name, "d4");
    }

    #[test]
    fn a_click_picks_a_cell_of_the_table() {
        let (tmp, mut app) = sample();
        write(
            tmp.path().join("t.csv"),
            "name,size\nalpha,1\nbeta,2\ngamma,3\n",
        )
        .unwrap();
        app.apply(Command::Relog);
        app.focus = Focus::Files;
        let at = app
            .files
            .iter()
            .position(|&f| app.file(f).name == "t.csv")
            .unwrap();
        app.file_state.select(Some(at));
        app.apply(Command::View);
        for _ in 0..200 {
            app.table_tick();
            if app.viewer.as_ref().is_some_and(|v| v.table.is_some()) {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let screen = draw(&mut app);
        click(&mut app, find(&screen, "beta", 2));
        let grid = app.viewer.as_ref().unwrap().table.as_ref().unwrap();
        assert_eq!((grid.cursor_row, grid.cursor_col), (1, 0));
        // The `size` column, on the row of gamma.
        let (x, _) = find(&screen, "size", 1);
        let (_, y) = find(&screen, "gamma", 2);
        click(&mut app, (x, y));
        let grid = app.viewer.as_ref().unwrap().table.as_ref().unwrap();
        assert_eq!((grid.cursor_row, grid.cursor_col), (2, 1));
    }

    fn right_click(app: &mut App, at: (u16, u16)) {
        mouse(app, MouseEventKind::Down(MouseButton::Right), at);
    }

    /// The labels of the menu's lines that run something.
    fn menu_labels(app: &App) -> Vec<String> {
        let menu = app.context_menu.as_ref().expect("the menu is open");
        menu.lines
            .iter()
            .filter_map(|l| match l {
                MenuLine::Run { label, .. } => Some(label.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_right_button_on_a_file_picks_it_and_opens_its_menu() {
        let (_tmp, mut app) = sample();
        let screen = draw(&mut app);
        right_click(&mut app, find(&screen, "b.txt", 1));
        assert_eq!(app.focus, Focus::Files);
        assert_eq!(app.file(app.selected_file().unwrap()).name, "b.txt");
        let labels = menu_labels(&app);
        for l in ["View", "Copy", "Delete", "Tag"] {
            assert!(labels.contains(&l.to_string()), "{l} in {labels:?}");
        }
        assert!(
            !labels.iter().any(|l| l.contains("tagged")),
            "nothing tagged"
        );
        // Each line shows its key.
        let menu = app.context_menu.as_ref().unwrap();
        let copy = menu.lines.iter().find_map(|l| match l {
            MenuLine::Run { label, key, .. } if label == "Copy" => Some(key.clone()),
            _ => None,
        });
        assert_eq!(copy.as_deref(), Some("C"));
        // Drawn, with the key beside the label.
        let screen = draw(&mut app);
        assert!(
            screen
                .iter()
                .any(|r| r.contains("Copy") && r.contains(" C "))
        );
    }

    #[test]
    fn a_line_of_the_menu_runs_its_command_on_the_item() {
        let (_tmp, mut app) = sample();
        let screen = draw(&mut app);
        right_click(&mut app, find(&screen, "c.txt", 1));
        let screen = draw(&mut app);
        // The menu's own "View" (the menu is drawn over the list).
        let (x, y) = find(&screen, " View", 1);
        click(&mut app, (x + 1, y));
        assert!(app.context_menu.is_none());
        let v = app.viewer.as_ref().expect("viewing");
        assert!(v.path.ends_with("c.txt"));
    }

    #[test]
    fn a_click_off_the_menu_closes_it_and_does_nothing_else() {
        let (_tmp, mut app) = sample();
        let screen = draw(&mut app);
        right_click(&mut app, find(&screen, "a.txt", 1));
        let before = (
            app.focus,
            app.tree_state.selected(),
            app.file_state.selected(),
        );
        draw(&mut app);
        click(&mut app, (1, 1));
        assert!(app.context_menu.is_none());
        let after = (
            app.focus,
            app.tree_state.selected(),
            app.file_state.selected(),
        );
        assert_eq!(before, after);
    }

    #[test]
    fn the_menu_from_the_keys_and_esc() {
        let (_tmp, mut app) = sample();
        draw(&mut app);
        // Shift+F10 is the menu, not F10 (Quit).
        app.handle_key(KeyEvent::new(KeyCode::F(10), KeyModifiers::SHIFT));
        assert!(!app.should_quit && app.confirm.is_none());
        let labels = menu_labels(&app);
        assert!(
            labels.contains(&"Make a directory".to_string()),
            "{labels:?}"
        );
        // ↓ skips separators and titles; Esc closes.
        let first = app.context_menu.as_ref().unwrap().selected;
        press_key(&mut app, KeyCode::Down);
        let menu = app.context_menu.as_ref().unwrap();
        assert_ne!(menu.selected, first);
        assert!(menu.lines[menu.selected].runs());
        press_key(&mut app, KeyCode::Esc);
        assert!(app.context_menu.is_none());
        // The Menu key: the file window's menu.
        app.focus = Focus::Files;
        press_key(&mut app, KeyCode::Menu);
        assert!(menu_labels(&app).contains(&"View".to_string()));
        // Enter runs the chosen line: the first, View.
        press_key(&mut app, KeyCode::Enter);
        assert!(app.viewer.is_some());
    }

    #[test]
    fn with_tags_the_menu_has_a_group_for_them() {
        let (_tmp, mut app) = sample();
        app.focus = Focus::Files;
        app.apply(Command::TagAll);
        let screen = draw(&mut app);
        right_click(&mut app, find(&screen, "a.txt", 1));
        let menu = app.context_menu.as_ref().unwrap();
        assert!(
            menu.lines
                .contains(&MenuLine::Title("3 tagged files".into()))
        );
        let labels = menu_labels(&app);
        assert!(labels.contains(&"Copy tagged".to_string()));
        assert!(labels.contains(&"Untag".to_string()), "this one is tagged");
    }

    #[test]
    fn inside_an_archive_the_menu_leaves_out_what_writes() {
        let (tmp, mut app) = sample();
        crate::archive::tests::make_zip(&tmp.path().join("z.zip"));
        app.apply(Command::Relog);
        app.focus = Focus::Files;
        let at = app
            .files
            .iter()
            .position(|&f| app.file(f).name == "z.zip")
            .unwrap();
        app.file_state.select(Some(at));
        app.apply(Command::OpenArchive);
        for _ in 0..200 {
            app.poll_opening();
            if app.archive.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(app.archive.is_some(), "inside the zip");
        app.focus = Focus::Files;
        app.file_state.select(Some(0));
        press_key(&mut app, KeyCode::Menu);
        let labels = menu_labels(&app);
        assert!(labels.contains(&"Copy".to_string()), "{labels:?}");
        for l in ["Delete", "Move", "Rename", "Edit"] {
            assert!(!labels.contains(&l.to_string()), "{l} in {labels:?}");
        }
    }

    #[test]
    fn the_right_button_in_the_viewer_opens_its_keys() {
        let (_tmp, mut app) = sample();
        app.focus = Focus::Files;
        app.apply(Command::View);
        let screen = draw(&mut app);
        right_click(&mut app, find(&screen, "a.txt", 2));
        let labels = menu_labels(&app);
        assert!(labels.contains(&"Wrap".to_string()), "{labels:?}");
        // Close: the viewer's Esc.
        let i = app
            .context_menu
            .as_ref()
            .unwrap()
            .lines
            .iter()
            .position(|l| matches!(l, MenuLine::Run { label, .. } if label == "Close"))
            .unwrap();
        app.run_menu_line(i);
        assert!(app.viewer.is_none());
    }

    /// The framed editor's program asked for the mouse: a click and a drag
    /// go to it, where they were on its panel, even a drag that left it.
    #[cfg(unix)]
    #[test]
    fn the_program_of_the_framed_editor_gets_its_mouse() {
        let (tmp, mut app) = sample();
        let file = tmp.path().join("a.txt");
        let line = "printf '\\033[?1002h\\033[?1006h'; cat > %f";
        let ext = crate::shell::editor_command(line, &file, tmp.path());
        app.start_framed_editor(ext).unwrap();
        let screen = |app: &App| {
            let ed = app.editor_pty.as_ref().unwrap();
            ed.term.parser().screen().contents()
        };
        let wait = |app: &App, ok: &dyn Fn(&App) -> bool| {
            let deadline = Instant::now() + Duration::from_secs(10);
            while !ok(app) {
                assert!(Instant::now() < deadline, "screen:\n{}", screen(app));
                std::thread::sleep(Duration::from_millis(20));
            }
        };
        wait(&app, &|app| {
            let ed = app.editor_pty.as_ref().unwrap();
            ed.term.parser().screen().mouse_protocol_mode() != vt100::MouseProtocolMode::None
        });
        draw(&mut app);
        // The panel's border is around it: its first cell is (1, 2).
        let at = |kind, column, row| MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        };
        let now = Instant::now();
        app.handle_mouse(at(MouseEventKind::Down(MouseButton::Left), 6, 5), now);
        wait(&app, &|app| screen(app).contains("^[[<0;6;4M"));
        // Off the panel, held at its edge (the last row of a screen of 30
        // rows, less the header, the footer and the border).
        app.handle_mouse(at(MouseEventKind::Drag(MouseButton::Left), 0, 29), now);
        wait(&app, &|app| screen(app).contains("^[[<32;1;25M"));
        app.handle_mouse(at(MouseEventKind::Up(MouseButton::Left), 0, 29), now);
        wait(&app, &|app| screen(app).contains("^[[<0;1;25m"));
        assert!(app.drag.is_none());
        app.editor_pty.as_mut().unwrap().term.write(b"\x04");
    }
}
