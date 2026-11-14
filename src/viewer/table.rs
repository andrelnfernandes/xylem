//! In-memory interactive tabular grid for the Xylem viewer.
//!
//! Provides navigation, cell selection, column-width computation,
//! cell sanitization (no escape sequences, bidi-safe), and row filtering.

use ratatui::crossterm::event::KeyCode;
use ratatui::text::Span;
use std::collections::BTreeSet;
use std::time::SystemTime;

use super::records::Structure;
use super::sql_filter::{self, Expr, RowContext, SqlFilter, Value};
use super::table_window::{Progress, Window};
use super::tabular_loader::thousands;
use crate::bidi;

/// Computes visual display width in terminal cells.
fn display_width(s: &str) -> usize {
    Span::raw(s).width()
}

/// A single row with string-formatted cells and typed values.
#[derive(Debug, Clone)]
pub struct TableRow {
    pub cells: Vec<String>,
}

/// The list of a file's sheets or tables (Tab), with what the typed text left of it.
#[derive(Debug, Clone)]
pub struct SheetList {
    pub query: String,
    /// Indices of the sheets kept by `query`.
    pub shown: Vec<usize>,
    /// The cursor, in `shown`.
    pub cursor: usize,
    pub scroll: usize,
}

impl SheetList {
    /// The most that can be typed: no name needs more to be told apart.
    const MAX_QUERY_CHARS: usize = 256;

    pub fn new(names: &[String], current: usize) -> Self {
        Self {
            query: String::new(),
            shown: (0..names.len()).collect(),
            cursor: current.min(names.len().saturating_sub(1)),
            scroll: 0,
        }
    }

    pub fn push(&mut self, c: char, names: &[String]) {
        if self.query.chars().count() < Self::MAX_QUERY_CHARS {
            self.query.push(c);
            self.narrow(names);
        }
    }

    pub fn pop(&mut self, names: &[String]) {
        if self.query.pop().is_some() {
            self.narrow(names);
        }
    }

    fn narrow(&mut self, names: &[String]) {
        let query = self.query.to_lowercase();
        self.shown = names
            .iter()
            .enumerate()
            .filter(|(_, name)| name.to_lowercase().contains(&query))
            .map(|(i, _)| i)
            .collect();
        self.cursor = 0;
        self.scroll = 0;
    }
}

/// The structure of the table on screen, while it is open (`i`).
#[derive(Debug, Clone)]
pub struct StructureView {
    pub structure: Structure,
    pub scroll: usize,
}

/// A structured tabular dataset (from CSV, spreadsheet, SQLite or DBF).
#[derive(Debug)]
pub struct TableGrid {
    #[allow(dead_code)]
    pub title: String,
    pub sheets: Vec<String>,
    pub current_sheet: usize,
    /// From which of `sheets` on they are views of a database.
    pub first_view: usize,
    pub headers: Vec<String>,
    pub rows: Vec<TableRow>,
    /// Filtered row indices (or all rows if no filter active).
    pub filtered_indices: Vec<usize>,

    // Cursor and view coordinates
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_row: usize,
    pub scroll_col: usize,

    /// The query that gave these rows, when a whole query was typed.
    pub query: Option<String>,
    // Active filter query string
    pub active_filter: Option<String>,
    pub filter_expr: Option<Expr>,

    // Cached column widths
    pub col_widths: Vec<usize>,

    /// What of the file is not here, for the header.
    pub cut: Option<String>,

    /// The places of the rows marked for a Batch, in `rows`: the same as in
    /// the file, and whatever the filter hides.
    pub marked: BTreeSet<usize>,
    /// How the file was when the first of them was marked.
    pub marked_stamp: Option<(u64, SystemTime)>,

    /// What the file declares of its columns, made safe when it was read.
    pub structure: Option<Structure>,

    /// A table too big to keep whole: its rows are in the window, read from
    /// the file as the cursor moves, and `rows` and `filtered_indices` are
    /// empty.
    pub window: Option<Box<Window>>,
}

pub(super) struct GridRowContext<'a> {
    pub(super) headers: &'a [String],
    pub(super) row: &'a TableRow,
}

impl<'a> RowContext for GridRowContext<'a> {
    fn get_value(&self, column: &str) -> Option<Value> {
        // 1. Try exact or case-insensitive header match first
        for (idx, header) in self.headers.iter().enumerate() {
            if header.eq_ignore_ascii_case(column)
                && let Some(cell) = self.row.cells.get(idx)
            {
                return Some(Value::infer(cell));
            }
        }

        // 2. Fallback: column letter (A, B, C...)
        if let [b] = column.as_bytes()
            && b.is_ascii_alphabetic()
        {
            let col_idx = (b.to_ascii_uppercase() - b'A') as usize;
            if let Some(cell) = self.row.cells.get(col_idx) {
                return Some(Value::infer(cell));
            }
        }

        None
    }
}

impl TableGrid {
    pub fn new(title: String, headers: Vec<String>, rows: Vec<TableRow>) -> Self {
        let count = rows.len();
        let mut grid = Self {
            title,
            sheets: Vec::new(),
            current_sheet: 0,
            first_view: usize::MAX,
            headers,
            rows,
            filtered_indices: (0..count).collect(),
            cursor_row: 0,
            cursor_col: 0,
            scroll_row: 0,
            scroll_col: 0,
            query: None,
            active_filter: None,
            filter_expr: None,
            col_widths: Vec::new(),
            cut: None,
            marked: BTreeSet::new(),
            marked_stamp: None,
            structure: None,
            window: None,
        };
        grid.recalculate_widths();
        grid
    }

    /// Moves the rows into a window over the whole table (see `table_window`),
    /// once their widths are known. Its threads start with
    /// [`TableGrid::resume_window`].
    pub fn make_window(&mut self, opener: super::table_window::Opener) {
        let head = std::mem::take(&mut self.rows);
        self.filtered_indices = Vec::new();
        self.window = Some(Box::new(Window::new(head, self.headers.clone(), opener)));
    }

    /// Starts the window's threads, if there is a window.
    pub fn resume_window(&mut self) {
        if let Some(w) = &mut self.window {
            w.start();
        }
    }

    /// Lets the window's threads go, keeping the rows on screen.
    pub fn pause_window(&mut self) {
        if let Some(w) = &mut self.window {
            w.pause();
        }
    }

    /// A row by its number in the file (from 0), if it is here: in a window,
    /// one not read yet is not.
    pub fn row(&self, file_row: usize) -> Option<&TableRow> {
        match &self.window {
            None => self.rows.get(file_row),
            Some(w) => w.row(file_row),
        }
    }

    /// The number in the file (from 0) of the `visible`-th row shown.
    pub fn file_row(&self, visible: usize) -> Option<usize> {
        match &self.window {
            None => self.filtered_indices.get(visible).copied(),
            Some(w) => w.file_row(visible),
        }
    }

    /// The rows the table is known to have: all, unless a window is still
    /// counting them.
    pub fn known_rows(&self) -> usize {
        self.window.as_ref().map_or(self.rows.len(), |w| w.rows)
    }

    /// How many rows the table has, as the screen says it: `2,000+` while a
    /// window counts them.
    pub fn total_text(&self) -> String {
        let Some(w) = &self.window else {
            return thousands(self.rows.len());
        };
        let rows = thousands(w.rows);
        match w.counting {
            Progress::Done => rows,
            Progress::Running => format!("{rows}+ (counting…)"),
            Progress::Stopped | Progress::Failed => format!("{rows}+"),
        }
    }

    /// How the filter is going, for the line under the grid.
    pub fn filter_text(&self) -> Option<String> {
        let q = self.active_filter.as_ref()?;
        let found = thousands(self.visible_row_count());
        let Some(scan) = self.window.as_ref().and_then(|w| w.scan.as_ref()) else {
            return Some(format!("Filter: {q} ({found} matches)"));
        };
        let status = if scan.capped {
            format!("first {found} matches")
        } else {
            let scanned = thousands(scan.scanned);
            match scan.state {
                Progress::Done => format!("{found} matches"),
                Progress::Running => format!(
                    "{found} matches so far, row {scanned} of {}",
                    self.total_text()
                ),
                Progress::Stopped | Progress::Failed => {
                    format!("{found} matches in the first {scanned} rows: stopped")
                }
            }
        };
        Some(format!("Filter: {q} ({status})"))
    }

    /// Esc: stops the window's count and filter. False if neither was running.
    pub fn stop_window_work(&mut self) -> bool {
        self.window.as_mut().is_some_and(|w| w.stop_work())
    }

    /// The file is no longer the one its window was reading: R reads it again.
    pub fn file_changed(&self) -> bool {
        self.window.as_ref().is_some_and(|w| w.changed)
    }

    /// The filter is still going over the whole table.
    pub fn filtering(&self) -> bool {
        self.window
            .as_ref()
            .and_then(|w| w.scan.as_ref())
            .is_some_and(|s| s.state == Progress::Running)
    }

    /// The window is reading something: the screen should look again soon.
    pub fn window_working(&self) -> bool {
        self.window.as_ref().is_some_and(|w| w.working())
    }

    /// Takes what the window's threads read and asks for the rows around the
    /// screen (`page` rows tall). True if there is something new to show.
    pub fn poll_window(&mut self, page: usize) -> bool {
        let page = page.max(1);
        let focus = self.file_row(self.cursor_row).unwrap_or(0);
        let first = self.file_row(self.scroll_row).unwrap_or(0);
        let last = self
            .file_row(
                (self.scroll_row + page)
                    .min(self.visible_row_count())
                    .saturating_sub(1),
            )
            .unwrap_or(first);
        let Some(w) = &mut self.window else {
            return false;
        };
        let changed = w.poll(focus, (first, last));
        // Columns the count found further on (JSON Lines).
        if w.headers().len() > self.headers.len() {
            let new = &w.headers()[self.headers.len()..];
            self.col_widths
                .extend(new.iter().map(|h| (display_width(h) + 1).max(4)));
            self.headers.extend_from_slice(new);
        }
        // The table may have turned out shorter than it seemed.
        let visible = w.visible();
        if self.cursor_row >= visible {
            self.cursor_row = visible.saturating_sub(1);
        }
        if self.scroll_row > self.cursor_row {
            self.scroll_row = self.cursor_row;
        }
        // The screen and as much again on each side; a page of rows is
        // read at least, as reading costs most in getting to the first. No
        // more than fits, or what was read would push out what was asked for
        // and be asked for again.
        let w = self.window.as_mut().expect("looked at it above");
        let margin = page
            .max(250)
            .min(w.rows_that_fit().saturating_sub(page) / 2);
        let from = self.scroll_row.saturating_sub(margin);
        let to = (self.scroll_row + page + margin).min(visible);
        let wanted: Vec<usize> = (from..to).filter_map(|v| w.file_row(v)).collect();
        w.want(wanted.into_iter());
        if changed {
            self.widen_to_screen(page);
        }
        changed
    }

    /// Widens a column that a row on screen has wider than its first rows
    /// had (row numbers grow along a table), never narrowing one.
    fn widen_to_screen(&mut self, page: usize) {
        let end = (self.scroll_row + page).min(self.visible_row_count());
        for visible in self.scroll_row..end {
            let Some(row) = self.file_row(visible).and_then(|r| self.row(r)) else {
                continue;
            };
            let widths: Vec<usize> = row
                .cells
                .iter()
                .map(|cell| display_width(cell).min(40) + 1)
                .collect();
            for (have, w) in self.col_widths.iter_mut().zip(widths) {
                *have = (*have).max(w);
            }
        }
    }

    /// Recalculate visual column widths based on headers and sample rows.
    pub fn recalculate_widths(&mut self) {
        let col_count = self.headers.len();
        let mut widths = vec![4; col_count]; // minimum 4 chars

        for (idx, h) in self.headers.iter().enumerate() {
            widths[idx] = widths[idx].max(display_width(h.as_str()) + 1);
        }

        // Sample up to 200 rows for performance
        for row in self.rows.iter().take(200) {
            for (idx, cell) in row.cells.iter().enumerate() {
                if idx < widths.len() {
                    let w = display_width(cell.as_str());
                    widths[idx] = widths[idx].max(w.min(40) + 1); // cap max column auto-width at 40
                }
            }
        }

        self.col_widths = widths;
    }

    /// Apply or clear a SQL filter predicate.
    pub fn apply_filter(&mut self, query: Option<&str>) -> Result<usize, String> {
        match query {
            None => {
                self.active_filter = None;
                self.filter_expr = None;
                if let Some(w) = &mut self.window {
                    w.filter(None);
                } else {
                    self.filtered_indices = (0..self.rows.len()).collect();
                }
                self.cursor_row = 0;
                self.scroll_row = 0;
                Ok(self.filtered_indices.len())
            }
            Some(q) if q.trim().is_empty() => self.apply_filter(None),
            Some(q) => {
                let expr = SqlFilter::parse(q)?;
                if let Some(w) = &mut self.window {
                    w.filter(Some(expr.clone()));
                    self.active_filter = Some(q.to_string());
                    self.filter_expr = Some(expr);
                    self.cursor_row = 0;
                    self.scroll_row = 0;
                    return Ok(self.visible_row_count());
                }
                let mut matched = Vec::new();
                for (idx, row) in self.rows.iter().enumerate() {
                    let ctx = GridRowContext {
                        headers: &self.headers,
                        row,
                    };
                    if let Ok(val) = sql_filter::eval_expr(&expr, &ctx)
                        && val.is_truthy()
                    {
                        matched.push(idx);
                    }
                }
                self.filtered_indices = matched;
                self.active_filter = Some(q.to_string());
                self.filter_expr = Some(expr);
                self.cursor_row = 0;
                self.scroll_row = 0;
                Ok(self.filtered_indices.len())
            }
        }
    }

    /// How a sheet's name is shown: a database's view says so.
    pub fn view_mark(&self, sheet: usize) -> &'static str {
        if sheet >= self.first_view {
            " (view)"
        } else {
            ""
        }
    }

    /// The number the row under the cursor has in the file, from 1. A filter hides
    /// rows but does not renumber them.
    pub fn cursor_file_row(&self) -> usize {
        self.file_row(self.cursor_row).unwrap_or(self.cursor_row) + 1
    }

    /// Go to a row by its number in the file, from 1, and say where the cursor
    /// landed. If the filter hid that row, it goes to the next one kept.
    pub fn goto_row(&mut self, row: usize, page_height: usize) -> String {
        let total = self.known_rows().max(1);
        if let Some(w) = &self.window {
            if row > total && w.counting != Progress::Done {
                return format!(
                    "The count has not reached row {} yet: {} so far",
                    thousands(row),
                    self.total_text()
                );
            }
            if let Some(scan) = &w.scan
                && scan.state == Progress::Running
                && row > scan.scanned
                && scan.matches.last().is_none_or(|&last| last + 1 < row)
            {
                return format!(
                    "The filter has not reached row {} yet: it is on row {}",
                    thousands(row),
                    thousands(scan.scanned)
                );
            }
        }
        let row = row.clamp(1, total);
        let visible = self.visible_row_count();
        if visible == 0 {
            return "No rows to go to".to_string();
        }
        self.cursor_row = match &self.window {
            None => self.filtered_indices.partition_point(|&r| r < row - 1),
            Some(w) => w.shown_from(row - 1),
        }
        .min(visible - 1);
        self.scroll_row = self.cursor_row.saturating_sub(page_height / 2);
        let shown = self.cursor_file_row();
        let total = self.total_text();
        if shown == row {
            format!("Row {} of {total}", thousands(row))
        } else {
            format!(
                "Row {} is filtered out: showing row {}",
                thousands(row),
                thousands(shown)
            )
        }
    }

    /// What the file declares of its columns or, when it declares nothing (CSV,
    /// spreadsheets, a query), what the rows read have in each one.
    pub fn structure(&self) -> Structure {
        if let Some(structure) = &self.structure {
            return structure.clone();
        }
        const KINDS: [&str; 5] = ["integer", "decimal", "logical", "text", "empty"];
        let columns = self
            .headers
            .iter()
            .enumerate()
            .map(|(col, name)| {
                let mut seen = [false; KINDS.len()];
                let mut widest = 0;
                for row in self.sample() {
                    let cell = row.cells.get(col).map_or("", String::as_str);
                    let kind = match Value::infer(cell) {
                        Value::Int(_) => 0,
                        Value::Float(_) => 1,
                        Value::Bool(_) => 2,
                        Value::Text(_) => 3,
                        Value::Null => 4,
                    };
                    seen[kind] = true;
                    widest = widest.max(cell.chars().count());
                }
                let kinds: Vec<&str> = KINDS
                    .iter()
                    .zip(seen)
                    .filter_map(|(kind, seen)| seen.then_some(*kind))
                    .collect();
                vec![name.clone(), kinds.join(", "), widest.to_string()]
            })
            .collect();
        let rows = self.sample().count();
        let count = thousands(rows);
        Structure {
            titles: ["Column", "Values seen", "Widest"]
                .map(String::from)
                .to_vec(),
            columns,
            about: vec![format!(
                "No types declared here: what the {count} row{} read have",
                if rows == 1 { "" } else { "s" }
            )],
        }
    }

    /// The rows read so far: all of them, or those in the window.
    fn sample(&self) -> Box<dyn Iterator<Item = &TableRow> + '_> {
        match &self.window {
            None => Box::new(self.rows.iter()),
            Some(w) => Box::new(w.kept()),
        }
    }

    /// Total count of rows currently visible (matching filter).
    pub fn visible_row_count(&self) -> usize {
        match &self.window {
            None => self.filtered_indices.len(),
            Some(w) => w.visible(),
        }
    }

    /// Get current active cell contents.
    pub fn active_cell(&self) -> Option<&str> {
        let row = self.row(self.file_row(self.cursor_row)?)?;
        row.cells.get(self.cursor_col).map(|s| s.as_str())
    }

    /// Sanitize cell text for safe display (control chars and bidi to '·', truncated at max length).
    pub fn sanitize_cell(cell: &str) -> String {
        const MAX_CELL_CHARS: usize = 2048;
        let mut out = String::new();

        for (count, c) in cell.chars().enumerate() {
            if count >= MAX_CELL_CHARS {
                out.push('…');
                break;
            }
            if c.is_control() || bidi::is_bidi(c) || c == '\u{2028}' || c == '\u{2029}' {
                out.push('·');
            } else {
                out.push(c);
            }
        }
        out
    }

    /// To the first (`last` false) or last of the rows shown, in the same column.
    pub fn go_to_end(&mut self, last: bool, page_height: usize) {
        let total = self.visible_row_count();
        if last && total > 0 {
            self.cursor_row = total - 1;
            self.scroll_row = total.saturating_sub(page_height.max(1));
        } else {
            self.cursor_row = 0;
            self.scroll_row = 0;
        }
    }

    /// Width of the row numbers on the left of the grid.
    pub fn row_number_width(&self) -> usize {
        self.known_rows().max(1).to_string().len().max(3)
    }

    /// Moves the first column shown so that the cursor's column is on a
    /// screen `width` wide: right as far as needed, left back to it. A column
    /// wider than the screen is shown cut, from its start.
    pub fn show_cursor_col(&mut self, width: usize) {
        if self.cursor_col < self.scroll_col {
            self.scroll_col = self.cursor_col;
            return;
        }
        let width_of = |c: usize| self.col_widths.get(c).copied().unwrap_or(8);
        // Each column takes its width, a space and "│ ".
        let start = self.row_number_width() + 3;
        let fits = |first: usize| {
            let before: usize = (first..self.cursor_col).map(|c| width_of(c) + 2).sum();
            start + before + width_of(self.cursor_col) <= width
        };
        while self.scroll_col < self.cursor_col && !fits(self.scroll_col) {
            self.scroll_col += 1;
        }
    }

    /// Handle cursor navigation key events.
    pub fn handle_key(&mut self, code: KeyCode, page_height: usize) {
        let total_rows = self.visible_row_count();
        let total_cols = self.headers.len();

        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.cursor_row > 0 {
                    self.cursor_row -= 1;
                    if self.cursor_row < self.scroll_row {
                        self.scroll_row = self.cursor_row;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.cursor_row + 1 < total_rows {
                    self.cursor_row += 1;
                    if page_height > 0 && self.cursor_row >= self.scroll_row + page_height {
                        self.scroll_row = self.cursor_row - page_height + 1;
                    }
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                    if self.cursor_col < self.scroll_col {
                        self.scroll_col = self.cursor_col;
                    }
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.cursor_col + 1 < total_cols {
                    self.cursor_col += 1;
                }
            }
            KeyCode::PageUp => {
                self.cursor_row = self.cursor_row.saturating_sub(page_height);
                self.scroll_row = self.scroll_row.saturating_sub(page_height);
            }
            KeyCode::PageDown => {
                if total_rows > 0 {
                    self.cursor_row = (self.cursor_row + page_height).min(total_rows - 1);
                    self.scroll_row =
                        (self.scroll_row + page_height).min(total_rows.saturating_sub(1));
                }
            }
            KeyCode::Home => {
                self.cursor_col = 0;
                self.scroll_col = 0;
            }
            KeyCode::End if total_cols > 0 => {
                self.cursor_col = total_cols - 1;
            }
            _ => {}
        }
    }
}
