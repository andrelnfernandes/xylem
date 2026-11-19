//! The mouse (docs/plans/mouse.md): what a click or the wheel does, looked
//! up in the map of the last frame drawn (`App::hits`). A click picks a
//! row or stands for a key; the key goes through `handle_key`, as the
//! keyboard's does.

use ratatui::crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use super::*;
use crate::mouse::{Hit, List, Owner, Spot, WHEEL_LINES};

impl App {
    pub fn handle_mouse(&mut self, ev: MouseEvent, _now: Instant) {
        // A terminal left with the mouse on by another program.
        if !self.options.mouse {
            return;
        }
        let hit = self.hits.at(ev.column, ev.row);
        match ev.kind {
            MouseEventKind::Down(MouseButton::Left) => self.click(hit),
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                let up = ev.kind == MouseEventKind::ScrollUp;
                if ev.modifiers.contains(KeyModifiers::SHIFT) {
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

    fn click(&mut self, hit: Option<Hit>) {
        // Like the first key, the first click only closes the start screen.
        if self.splash.take().is_some() {
            return;
        }
        // The framed editor has the whole screen (its mouse: stage 4).
        if self.editor_pty.is_some() {
            return;
        }
        let Some(hit) = hit else { return };
        match hit.spot {
            Spot::Key(key) => self.handle_key(key),
            Spot::Link(id) => {
                if let Some(help) = &mut self.help {
                    help.follow(id);
                }
            }
            Spot::Scroll => {}
            Spot::Window(list) => self.click_list(list, None),
            Spot::Rows { list, .. } | Spot::Grid { list, .. } => self.click_list(list, hit.item),
        }
    }

    /// A click on a window, on its item `item` if there is one there.
    fn click_list(&mut self, list: List, item: Option<usize>) {
        let owner = match list {
            List::Tree(o) | List::Files(o) => o,
        };
        match owner {
            Owner::Picker(i) => {
                let Some(picker) = &mut self.picker else {
                    return;
                };
                if i < picker.panes.len() {
                    picker.active = i;
                }
                if let (List::Tree(_), Some(row)) = (list, item) {
                    picker.pane_mut().select_row(row);
                }
            }
            Owner::Main(i) => {
                if self.windows_blocked() || i >= self.panes.len() {
                    return;
                }
                self.message = None;
                // The tree above the terminal's panel: back to Xylem, as
                // with Alt+X.
                if self.terminal_focus {
                    self.hide_terminal();
                }
                self.active = i;
                let pane = &mut self.panes[i];
                // Usage and Duplicates have rows of their own (stage 2).
                if pane.usage.is_some() || pane.duplicates.is_some() {
                    return;
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
                self.sync_autoview();
            }
        }
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
            Spot::Window(list) | Spot::Rows { list, .. } | Spot::Grid { list, .. } => {
                self.wheel_list(list, cmd)
            }
            Spot::Scroll | Spot::Link(_) => {
                let code = if up { KeyCode::Up } else { KeyCode::Down };
                self.wheel_keys(code);
            }
            Spot::Key(_) => {}
        }
    }

    fn wheel_list(&mut self, list: List, cmd: Command) {
        let owner = match list {
            List::Tree(o) | List::Files(o) => o,
        };
        let (pane, main) = match owner {
            Owner::Picker(i) => match self.picker.as_mut().and_then(|p| p.panes.get_mut(i)) {
                Some(pane) => (pane, false),
                None => return,
            },
            Owner::Main(i) => {
                if self.windows_blocked() || i >= self.panes.len() {
                    return;
                }
                // Usage and Duplicates: their own ↑ and ↓, if in front.
                let pane = &self.panes[i];
                if pane.usage.is_some() || pane.duplicates.is_some() {
                    if i == self.active && !self.terminal_focus {
                        let code = if cmd == Command::Up {
                            KeyCode::Up
                        } else {
                            KeyCode::Down
                        };
                        self.wheel_keys(code);
                    }
                    return;
                }
                (&mut self.panes[i], true)
            }
        };
        for _ in 0..WHEEL_LINES {
            match list {
                List::Tree(_) => pane.apply_tree(cmd),
                List::Files(_) => pane.apply_files(cmd),
            }
        }
        if main {
            self.sync_autoview();
        }
    }

    /// The wheel as keys, to what is in front: never to a question or a
    /// prompt, which would take ↑ as their own (the history) or close.
    fn wheel_keys(&mut self, code: KeyCode) {
        let history_open = self.prompt.as_ref().is_some_and(|p| p.input.list_open());
        if self.confirm.is_some() || (self.prompt.is_some() && !history_open) {
            return;
        }
        let key = KeyEvent::new(code, KeyModifiers::NONE);
        for _ in 0..WHEEL_LINES {
            self.handle_key(key);
        }
    }

    /// Shift+wheel or the wheel's sideways tilt: across, in the viewer only
    /// (in the help, ← is "back").
    fn wheel_sideways(&mut self, hit: Option<Hit>, left: bool) {
        if self.viewer.is_none() || self.help.is_some() {
            return;
        }
        if hit.is_some_and(|h| h.spot == Spot::Scroll) {
            let code = if left { KeyCode::Left } else { KeyCode::Right };
            if self.confirm.is_none() && self.prompt.is_none() {
                self.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
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
}
