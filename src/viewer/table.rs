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
#[derive(Debug, Clone)]
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
}

struct GridRowContext<'a> {
    headers: &'a [String],
    row: &'a TableRow,
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
        };
        grid.recalculate_widths();
        grid
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
                self.filtered_indices = (0..self.rows.len()).collect();
                self.cursor_row = 0;
                self.scroll_row = 0;
                Ok(self.filtered_indices.len())
            }
            Some(q) if q.trim().is_empty() => self.apply_filter(None),
            Some(q) => {
                let expr = SqlFilter::parse(q)?;
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
        self.filtered_indices
            .get(self.cursor_row)
            .copied()
            .unwrap_or(self.cursor_row)
            + 1
    }

    /// Go to a row by its number in the file, from 1, and say where the cursor
    /// landed. If the filter hid that row, it goes to the next one kept.
    pub fn goto_row(&mut self, row: usize, page_height: usize) -> String {
        let total = self.rows.len().max(1);
        let row = row.clamp(1, total);
        let Some(&last) = self.filtered_indices.last() else {
            return "No rows to go to".to_string();
        };
        let wanted = (row - 1).min(last);
        self.cursor_row = self.filtered_indices.partition_point(|&r| r < wanted);
        self.scroll_row = self.cursor_row.saturating_sub(page_height / 2);
        let shown = self.cursor_file_row();
        if shown == row {
            format!("Row {row} of {total}")
        } else {
            format!("Row {row} is filtered out: showing row {shown}")
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
                for row in &self.rows {
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
        let rows = self.rows.len();
        let count = super::tabular_loader::thousands(rows);
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

    /// Total count of rows currently visible (matching filter).
    pub fn visible_row_count(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Get current active cell contents.
    pub fn active_cell(&self) -> Option<&str> {
        if let Some(&row_idx) = self.filtered_indices.get(self.cursor_row)
            && let Some(row) = self.rows.get(row_idx)
        {
            return row.cells.get(self.cursor_col).map(|s| s.as_str());
        }
        None
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
