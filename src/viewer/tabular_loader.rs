//! The grid on screen, built from a stream of records (see `records`).
//!
//! Enforces security bounds: this is the one place that keeps rows in memory,
//! so it stops the stream at its limit, and what it keeps is safe for a terminal.

use std::ops::ControlFlow;
use std::path::Path;

use super::records::RowSource;
use super::records::{self, Budget, Cell, End, MAX_COLS, RecordSink, SqlitePart, Structure};
use super::table::{TableGrid, TableRow};
use super::table_window::{Opener, WINDOW_ROWS};
use std::sync::Arc;

pub use super::records::column_letter;

/// Rows the grid keeps in memory (columns are limited by the stream).
pub const MAX_TABLE_ROWS: usize = 50_000;

/// Detects if a file looks like a CSV, TSV, JSONL, DBF, Spreadsheet or SQLite DB.
pub fn is_tabular_candidate(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();

    matches!(
        ext.as_str(),
        "csv"
            | "tsv"
            | "tab"
            | "jsonl"
            | "ndjson"
            | "xlsx"
            | "ods"
            | "xls"
            | "xlsb"
            | "dbf"
            | "sqlite"
            | "sqlite3"
            | "db"
            | "parquet"
    )
}

/// The formats read by libraries in C: SQLite and Parquet's zstd codec.
/// Everything else here is parsed in Rust.
pub fn is_sqlite(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    matches!(ext.as_str(), "sqlite" | "sqlite3" | "db")
}

/// Whether what was typed in the SQL prompt is a whole query and not a condition.
/// A column may be called `select` or `with`: `select = 1` is still a condition.
pub fn is_whole_query(text: &str) -> bool {
    starts_a_statement(text, &["select", "with"])
}

/// Whether what was typed in the SQL prompt asks to change something, which Xylem
/// does not do. A column may have one of these names too.
pub fn is_write_statement(text: &str) -> bool {
    starts_a_statement(
        text,
        &[
            "update", "delete", "insert", "replace", "drop", "alter", "create", "truncate",
            "attach", "detach", "pragma", "vacuum", "reindex",
        ],
    )
}

fn starts_a_statement(text: &str, statements: &[&str]) -> bool {
    let mut words = text.split_whitespace();
    let Some(first) = words.next() else {
        return false;
    };
    if !statements.iter().any(|s| first.eq_ignore_ascii_case(s)) {
        return false;
    }
    let Some(next) = words.next() else {
        return false;
    };
    !next.starts_with(['=', '<', '>', '!'])
        && !["is", "in", "like", "ilike", "not"]
            .iter()
            .any(|word| next.eq_ignore_ascii_case(word))
}

pub fn uses_c_parser(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    matches!(ext.as_str(), "sqlite" | "sqlite3" | "db" | "parquet")
}

/// A spreadsheet: XLSX, XLSB, XLS or ODS.
fn is_spreadsheet(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    matches!(ext.as_str(), "xlsx" | "xlsb" | "xls" | "ods")
}

/// The sheets a reader builds whole before giving a row: no one can stop that part,
/// and a small file can make it take gigabytes.
pub fn is_built_whole(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    matches!(ext.as_str(), "ods" | "xls")
}

/// Builds a grid from a stream.
#[derive(Default)]
struct GridSink {
    sheets: Vec<String>,
    current_sheet: usize,
    first_view: Option<usize>,
    headers: Vec<String>,
    rows: Vec<TableRow>,
    /// The most rows kept, when not [`MAX_TABLE_ROWS`].
    max_rows: Option<usize>,
    rows_cut: bool,
    columns_cut: bool,
    structure: Option<Structure>,
}

impl RecordSink for GridSink {
    fn first_view(&mut self, index: usize) {
        self.first_view = Some(index);
    }

    fn sheets(&mut self, names: &[String], current: usize) {
        self.sheets = names.iter().map(|s| TableGrid::sanitize_cell(s)).collect();
        self.current_sheet = current;
    }

    fn columns(&mut self, names: &[String]) {
        self.headers = names.iter().map(|s| TableGrid::sanitize_cell(s)).collect();
    }

    fn columns_cut(&mut self) {
        self.columns_cut = true;
    }

    fn structure(&mut self, structure: Structure) {
        let clean = |texts: &[String]| texts.iter().map(|s| TableGrid::sanitize_cell(s)).collect();
        self.structure = Some(Structure {
            titles: clean(&structure.titles),
            columns: structure
                .columns
                .iter()
                .take(MAX_COLS)
                .map(|column| clean(column))
                .collect(),
            about: clean(&structure.about),
        });
    }

    fn record(&mut self, cells: &[Cell<'_>]) -> ControlFlow<()> {
        // One more than it keeps, to tell a table cut from one of that size.
        if self.rows.len() >= self.max_rows.unwrap_or(MAX_TABLE_ROWS) {
            self.rows_cut = true;
            return ControlFlow::Break(());
        }
        let cells = cells.iter().take(MAX_COLS).map(Cell::display).collect();
        self.rows.push(TableRow { cells });
        ControlFlow::Continue(())
    }
}

/// `50000` -> `50,000`.
pub(super) fn thousands(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

impl GridSink {
    fn finish(mut self, path: &Path, end: End, by_user: bool) -> TableGrid {
        // Columns found after a row was read leave it short.
        for row in &mut self.rows {
            row.cells
                .resize(self.headers.len().max(row.cells.len()), String::new());
        }
        let mut cut = Vec::new();
        if end == End::OutOfTime {
            let by = if by_user { "Esc" } else { "the time limit" };
            cut.push(format!(
                "stopped by {by}: showing {} rows",
                thousands(self.rows.len())
            ));
        } else if self.rows_cut && is_spreadsheet(path) {
            // A sheet cannot be read in windows, so the rest never shows here:
            // say where to see it.
            cut.push(format!(
                "the sheet has more than {} rows, the most Xylem shows: open it in Excel \
                 or another spreadsheet program to see them all (O)",
                thousands(MAX_TABLE_ROWS)
            ));
        } else if self.rows_cut {
            cut.push(format!("showing first {} rows", thousands(MAX_TABLE_ROWS)));
        }
        if self.columns_cut {
            cut.push(format!("showing first {MAX_COLS} columns"));
        }
        let mut grid = TableGrid::new(path.display().to_string(), self.headers, self.rows);
        grid.sheets = self.sheets;
        grid.current_sheet = self.current_sheet;
        grid.first_view = self.first_view.unwrap_or(usize::MAX);
        grid.cut = (!cut.is_empty()).then(|| cut.join(", "));
        grid.structure = self.structure;
        grid
    }
}

fn load(
    path: &Path,
    budget: Budget,
    read: impl FnOnce(Budget, &mut GridSink) -> Result<End, String>,
) -> Result<TableGrid, String> {
    let mut sink = GridSink::default();
    let end = read(budget.clone(), &mut sink)?;
    Ok(sink.finish(path, end, budget.cancelled()))
}

/// Load a CSV or TSV file into a TableGrid.
pub fn load_csv(path: &Path, delimiter: Option<u8>, budget: Budget) -> Result<TableGrid, String> {
    load(path, budget, |budget, sink| {
        records::read_csv(path, delimiter, budget, sink)
    })
}

/// Load a JSON Lines (.jsonl / .ndjson) file into a TableGrid.
pub fn load_jsonl(path: &Path, budget: Budget) -> Result<TableGrid, String> {
    load(path, budget, |budget, sink| {
        records::read_jsonl(path, budget, sink)
    })
}

/// Load a dBase / Clipper / FoxPro .dbf file.
pub fn load_dbf(path: &Path, budget: Budget) -> Result<TableGrid, String> {
    load(path, budget, |budget, sink| {
        records::read_dbf(path, budget, sink)
    })
}

/// Load a sheet (by position) of an Excel / ODS spreadsheet.
pub fn load_spreadsheet(path: &Path, sheet: usize, budget: Budget) -> Result<TableGrid, String> {
    load(path, budget, |budget, sink| {
        records::read_spreadsheet(path, sheet, budget, sink)
    })
}

/// Load a table or query result from an SQLite database.
pub fn load_sqlite(path: &Path, part: SqlitePart<'_>, budget: Budget) -> Result<TableGrid, String> {
    load(path, budget, |budget, sink| {
        records::read_sqlite(path, part, budget, sink)
    })
}

/// Opens a table again from where it is, in the thread that reads it.
fn sqlite_opener(path: &Path, part: SqlitePart<'_>) -> Opener {
    let path = path.to_path_buf();
    let (table, query) = match part {
        SqlitePart::First => (None, None),
        SqlitePart::Table(t) => (Some(t.to_string()), None),
        SqlitePart::Query(q) => (None, Some(q.to_string())),
    };
    Arc::new(move || {
        let part = match (&table, &query) {
            (_, Some(q)) => SqlitePart::Query(q),
            (Some(t), None) => SqlitePart::Table(t),
            (None, None) => SqlitePart::First,
        };
        records::SqliteRows::open(&path, part).map(|rows| Box::new(rows) as Box<dyn RowSource>)
    })
}

/// Like [`load_sqlite`], but a table with more than [`WINDOW_ROWS`] rows is
/// not cut: the grid keeps a window over it (see `table_window`), whose
/// threads start when the grid is on screen.
pub fn load_sqlite_windowed(
    path: &Path,
    part: SqlitePart<'_>,
    budget: Budget,
) -> Result<TableGrid, String> {
    let mut sink = GridSink {
        max_rows: Some(WINDOW_ROWS),
        ..GridSink::default()
    };
    let end = records::read_sqlite(path, part, budget.clone(), &mut sink)?;
    let more = std::mem::take(&mut sink.rows_cut);
    let mut grid = sink.finish(path, end, budget.cancelled());
    if more {
        grid.make_window(sqlite_opener(path, part));
    }
    Ok(grid)
}

/// Load an Apache Parquet file (.parquet).
pub fn load_parquet(path: &Path, budget: Budget) -> Result<TableGrid, String> {
    load(path, budget, |budget, sink| {
        records::read_parquet(path, budget, sink)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_column_letter() {
        assert_eq!(column_letter(0), "A");
        assert_eq!(column_letter(25), "Z");
        assert_eq!(column_letter(26), "AA");
        assert_eq!(column_letter(27), "AB");
    }

    #[test]
    fn test_load_csv_with_headers() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name,Age,City").unwrap();
        writeln!(file, "Alice,30,Brasilia").unwrap();
        writeln!(file, "Bob,25,Curitiba").unwrap();

        let grid = load_csv(file.path(), Some(b','), Budget::unlimited()).unwrap();
        assert_eq!(grid.headers, vec!["Name", "Age", "City"]);
        assert_eq!(grid.rows.len(), 2);
        assert_eq!(grid.rows[0].cells, vec!["Alice", "30", "Brasilia"]);
    }

    #[test]
    fn test_load_jsonl() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"id": 1, "task": "Test"}}"#).unwrap();
        writeln!(file, r#"{{"id": 2, "task": "Verify", "done": true}}"#).unwrap();

        let grid = load_jsonl(file.path(), Budget::unlimited()).unwrap();
        assert!(grid.headers.contains(&"id".to_string()));
        assert!(grid.headers.contains(&"task".to_string()));
        assert_eq!(grid.rows.len(), 2);
    }

    #[test]
    fn test_load_sqlite_defensive_and_query() {
        use rusqlite::Connection;
        let file = NamedTempFile::new().unwrap();
        {
            let conn = Connection::open(file.path()).unwrap();
            conn.execute(
                "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, balance REAL)",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO users (id, name, balance) VALUES (1, 'Alice', 100.50)",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO users (id, name, balance) VALUES (2, 'Bob', 250.00)",
                [],
            )
            .unwrap();
        }

        // Test loading default table
        let grid = load_sqlite(file.path(), SqlitePart::First, Budget::unlimited()).unwrap();
        assert_eq!(grid.sheets, vec!["users".to_string()]);
        assert_eq!(grid.headers, vec!["id", "name", "balance"]);
        assert_eq!(grid.rows.len(), 2);
        assert_eq!(grid.rows[0].cells, vec!["1", "Alice", "100.5"]);

        // Test custom SELECT query
        let grid_query = load_sqlite(
            file.path(),
            SqlitePart::Query("SELECT name, balance FROM users WHERE balance > 150"),
            Budget::unlimited(),
        )
        .unwrap();
        assert_eq!(grid_query.headers, vec!["name", "balance"]);
        assert_eq!(grid_query.rows.len(), 1);
        assert_eq!(grid_query.rows[0].cells, vec!["Bob", "250"]);
    }

    #[test]
    fn test_load_parquet_basic_and_sanitization() {
        use arrow_array::{Int32Array, RecordBatch, StringArray};
        use arrow_schema::{DataType, Field, Schema};
        use parquet::arrow::arrow_writer::ArrowWriter;
        use std::sync::Arc;

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("label", DataType::Utf8, true),
        ]));

        let ids = Arc::new(Int32Array::from(vec![10, 20, 30]));
        // Test normal string, null, and ANSI escape + bidi Trojan Source
        let labels = Arc::new(StringArray::from(vec![
            Some("Valid"),
            None,
            Some("Evil\x1b[31mColor\x1b[0m\u{202E}Bidi"),
        ]));

        let batch = RecordBatch::try_new(schema.clone(), vec![ids, labels]).unwrap();

        let file = NamedTempFile::new().unwrap();
        {
            let mut writer = ArrowWriter::try_new(file.reopen().unwrap(), schema, None).unwrap();
            writer.write(&batch).unwrap();
            writer.close().unwrap();
        }

        let grid = load_parquet(file.path(), Budget::unlimited()).unwrap();
        assert_eq!(grid.headers, vec!["id", "label"]);
        assert_eq!(grid.sheets, vec!["Parquet"]);
        assert_eq!(grid.rows.len(), 3);
        assert_eq!(grid.rows[0].cells, vec!["10", "Valid"]);
        assert_eq!(grid.rows[1].cells, vec!["20", ""]);
        // Control chars (ESC) and bidi controls replaced with '·' to neutralize terminal injections
        assert_eq!(grid.rows[2].cells, vec!["30", "Evil·[31mColor·[0m·Bidi"]);
    }

    #[test]
    fn test_load_parquet_invalid_header_or_size() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"NOT_A_PARQUET_FILE_TOO_LONG").unwrap();
        let res = load_parquet(file.path(), Budget::unlimited());
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("missing PAR1 magic header"));

        let mut tiny_file = NamedTempFile::new().unwrap();
        tiny_file.write_all(b"PAR").unwrap();
        let tiny_res = load_parquet(tiny_file.path(), Budget::unlimited());
        assert!(tiny_res.is_err());
        assert!(tiny_res.unwrap_err().contains("too small"));

        let mut no_footer = NamedTempFile::new().unwrap();
        no_footer.write_all(b"PAR112345678NOT1").unwrap();
        let no_footer_res = load_parquet(no_footer.path(), Budget::unlimited());
        assert!(no_footer_res.is_err());
        assert!(
            no_footer_res
                .unwrap_err()
                .contains("missing PAR1 magic footer")
        );
    }

    #[test]
    fn test_load_parquet_sql_filtering() {
        use arrow_array::{Int32Array, RecordBatch, StringArray};
        use arrow_schema::{DataType, Field, Schema};
        use parquet::arrow::arrow_writer::ArrowWriter;
        use std::sync::Arc;

        let schema = Arc::new(Schema::new(vec![
            Field::new("name", DataType::Utf8, false),
            Field::new("score", DataType::Int32, false),
        ]));

        let names = Arc::new(StringArray::from(vec!["Alice", "Bob", "Charlie", "David"]));
        let scores = Arc::new(Int32Array::from(vec![95, 45, 80, 60]));

        let batch = RecordBatch::try_new(schema.clone(), vec![names, scores]).unwrap();

        let file = NamedTempFile::new().unwrap();
        {
            let mut writer = ArrowWriter::try_new(file.reopen().unwrap(), schema, None).unwrap();
            writer.write(&batch).unwrap();
            writer.close().unwrap();
        }

        let mut grid = load_parquet(file.path(), Budget::unlimited()).unwrap();
        assert_eq!(grid.rows.len(), 4);

        // Filter score >= 80
        let matches = grid.apply_filter(Some("score >= 80")).unwrap();
        assert_eq!(matches, 2);
        assert_eq!(grid.filtered_indices, vec![0, 2]);

        // Filter name LIKE '%a%' (case-sensitive: Charlie, David)
        let matches = grid.apply_filter(Some("name LIKE '%a%'")).unwrap();
        assert_eq!(matches, 2);

        // Filter name ILIKE '%a%' (case-insensitive: Alice, Charlie, David)
        let matches_ilike = grid.apply_filter(Some("name ILIKE '%a%'")).unwrap();
        assert_eq!(matches_ilike, 3);
    }

    fn numbers_file(rows: usize) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "n,w").unwrap();
        for i in 0..rows {
            writeln!(file, "{i},x").unwrap();
        }
        file
    }

    #[test]
    fn the_grid_stops_at_its_limit_and_says_so() {
        let file = numbers_file(MAX_TABLE_ROWS + 1);
        let grid = load_csv(file.path(), None, Budget::unlimited()).unwrap();
        assert_eq!(grid.rows.len(), MAX_TABLE_ROWS);
        assert_eq!(grid.cut.as_deref(), Some("showing first 50,000 rows"));
    }

    #[test]
    fn a_sheet_over_the_limit_says_where_to_see_it_all() {
        let full = |path: &str| {
            let mut sink = GridSink::default();
            sink.columns(&["n".to_string()]);
            for i in 0..=MAX_TABLE_ROWS {
                if sink.record(&[Cell::Int(i as i64)]).is_break() {
                    break;
                }
            }
            sink.finish(Path::new(path), End::Complete, false).cut
        };
        for sheet in ["big.xlsx", "big.XLS", "big.xlsb", "big.ods"] {
            let cut = full(sheet).unwrap();
            assert!(
                cut.starts_with("the sheet has more than 50,000 rows"),
                "{cut}"
            );
            assert!(
                cut.contains("Excel or another spreadsheet program"),
                "{cut}"
            );
        }
        // Other tables keep their message.
        assert_eq!(
            full("big.csv").as_deref(),
            Some("showing first 50,000 rows")
        );
    }

    #[test]
    fn a_table_exactly_at_the_limit_is_not_cut() {
        let file = numbers_file(MAX_TABLE_ROWS);
        let grid = load_csv(file.path(), None, Budget::unlimited()).unwrap();
        assert_eq!(grid.rows.len(), MAX_TABLE_ROWS);
        assert_eq!(grid.cut, None);
    }

    #[test]
    fn the_grid_cleans_and_shortens_what_the_stream_gives() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "h\u{1b}\u{202E},b").unwrap();
        writeln!(file, "a\u{1b}[31m\u{202E}z,{}", "y".repeat(3_000)).unwrap();
        let grid = load_csv(file.path(), None, Budget::unlimited()).unwrap();
        assert_eq!(grid.headers[0], "h··");
        assert_eq!(grid.rows[0].cells[0], "a·[31m·z");
        assert_eq!(grid.rows[0].cells[1].chars().count(), 2049);
        assert!(grid.rows[0].cells[1].ends_with('…'));
    }

    #[test]
    fn rows_are_as_wide_as_the_columns_found_later() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"a": 1}}"#).unwrap();
        writeln!(file, r#"{{"a": 2, "b": 3}}"#).unwrap();
        let grid = load_jsonl(file.path(), Budget::unlimited()).unwrap();
        assert_eq!(grid.headers, ["a", "b"]);
        assert_eq!(grid.rows[0].cells, ["1", ""]);
        assert_eq!(grid.rows[1].cells, ["2", "3"]);
    }

    #[test]
    fn a_table_out_of_time_says_how_much_it_shows() {
        let file = numbers_file(2_000);
        let budget = Budget::new(std::time::Duration::ZERO, u64::MAX);
        let grid = load_csv(file.path(), None, budget).unwrap();
        assert!(grid.rows.len() < 2_000);
        assert_eq!(
            grid.cut,
            Some(format!(
                "stopped by the time limit: showing {} rows",
                grid.rows.len()
            ))
        );
    }

    #[test]
    fn a_table_with_too_many_columns_says_so() {
        let mut file = NamedTempFile::new().unwrap();
        let header: Vec<String> = (0..MAX_COLS + 1).map(|i| format!("c{i}")).collect();
        writeln!(file, "{}", header.join(",")).unwrap();
        let grid = load_csv(file.path(), None, Budget::unlimited()).unwrap();
        assert_eq!(grid.headers.len(), MAX_COLS);
        assert_eq!(grid.cut.as_deref(), Some("showing first 256 columns"));
    }

    #[test]
    fn a_blob_shows_as_its_size() {
        let file = NamedTempFile::new().unwrap();
        rusqlite::Connection::open(file.path())
            .unwrap()
            .execute_batch("CREATE TABLE t (b); INSERT INTO t VALUES (x'0001');")
            .unwrap();
        let grid = load_sqlite(file.path(), SqlitePart::First, Budget::unlimited()).unwrap();
        assert_eq!(grid.rows[0].cells, ["<Blob 2 bytes>"]);
    }

    #[test]
    fn a_dbf_shows_its_fields_as_display_structure_does() {
        use dbase::{FieldName, FieldValue, Record, TableWriterBuilder};
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("people.dbf");
        let name = |s: &str| FieldName::try_from(s).unwrap();
        let mut writer = TableWriterBuilder::new()
            .add_character_field(name("NAME"), 20)
            .add_numeric_field(name("SALARY"), 10, 2)
            .add_date_field(name("BORN"))
            .add_logical_field(name("ACTIVE"))
            .build_with_file_dest(&path)
            .unwrap();
        for who in ["Ana", "Rui"] {
            let mut record = Record::default();
            record.insert("NAME".into(), FieldValue::Character(Some(who.into())));
            record.insert("SALARY".into(), FieldValue::Numeric(Some(1234.5)));
            record.insert("BORN".into(), FieldValue::Date(None));
            record.insert("ACTIVE".into(), FieldValue::Logical(Some(true)));
            writer.write_record(&record).unwrap();
        }
        drop(writer);

        let grid = load_dbf(&path, Budget::unlimited()).unwrap();
        let s = grid.structure.clone().expect("a dbf declares its fields");
        assert_eq!(s, grid.structure(), "the declared one wins");
        assert_eq!(s.titles, ["Field", "Type", "Width", "Dec"]);
        assert_eq!(
            s.columns,
            [
                ["NAME", "C Character", "20", "0"],
                ["SALARY", "N Numeric", "10", "2"],
                ["BORN", "D Date", "8", "0"],
                ["ACTIVE", "L Logical", "1", "0"],
            ]
        );
        assert_eq!(s.about[0], "Number of data records: 2");
        assert!(s.about[1].starts_with("Date of last update: "), "{s:?}");
        // The deletion flag, then the fields.
        assert_eq!(s.about[2], "Record length: 40 bytes");
    }

    #[test]
    fn decimals_that_disagree_with_the_crate_are_unknown() {
        use dbase::{FieldName, TableWriterBuilder};
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("odd.dbf");
        let writer = TableWriterBuilder::new()
            .add_numeric_field(FieldName::try_from("N").unwrap(), 10, 3)
            .build_with_file_dest(&path)
            .unwrap();
        drop(writer);
        let fields = dbase::Reader::from_path(&path).unwrap().fields().to_vec();
        assert_eq!(records_decimals(&path, &fields), [Some(3)]);
        // Another size in the descriptor than the crate read: not to be trusted.
        let mut bytes = std::fs::read(&path).unwrap();
        bytes[32 + 16] = 99;
        std::fs::write(&path, &bytes).unwrap();
        assert_eq!(records_decimals(&path, &fields), [None]);
        // Gone: nothing to read them from.
        std::fs::remove_file(&path).unwrap();
        assert_eq!(records_decimals(&path, &fields), [None]);
    }

    fn records_decimals(path: &Path, fields: &[dbase::FieldInfo]) -> Vec<Option<u8>> {
        records::dbase_decimals(path, fields)
    }

    #[test]
    fn a_database_declares_types_and_keys_and_its_names_come_clean() {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("data.db");
        rusqlite::Connection::open(&path)
            .unwrap()
            .execute_batch(
                "CREATE TABLE \"t\x1b[31m\u{202E}\" (id INTEGER PRIMARY KEY, \
                 name TEXT NOT NULL DEFAULT 'x\x1b]0;evil\x07', \"bad\u{202E}\" BLOB);
                 CREATE VIEW v AS SELECT id FROM \"t\x1b[31m\u{202E}\";",
            )
            .unwrap();
        let grid = load_sqlite(&path, SqlitePart::First, Budget::unlimited()).unwrap();
        let s = grid.structure.expect("SQLite declares its columns");
        assert_eq!(
            s.titles,
            ["Column", "Declared type", "Not null", "Default", "Key"]
        );
        assert_eq!(
            s.columns,
            [
                ["id", "INTEGER", "", "", "1"],
                ["name", "TEXT", "yes", "'x·]0;evil·'", ""],
                ["bad·", "BLOB", "", "", ""],
            ]
        );
        assert_eq!(s.about, ["Table: t·[31m·"]);

        let grid = load_sqlite(&path, SqlitePart::Table("v"), Budget::unlimited()).unwrap();
        let s = grid.structure.unwrap();
        assert_eq!(s.columns, [["id", "INTEGER", "", "", ""]]);
        assert_eq!(s.about, ["View: v"]);

        // A query: the columns of its result, typed where they come from a table.
        let grid = load_sqlite(
            &path,
            SqlitePart::Query("SELECT name, id + 1 AS next FROM \"t\x1b[31m\u{202E}\""),
            Budget::unlimited(),
        )
        .unwrap();
        let s = grid.structure.unwrap();
        assert_eq!(s.titles, ["Column", "Declared type"]);
        assert_eq!(s.columns, [["name", "TEXT"], ["next", "(expression)"]]);
        assert_eq!(s.about, ["Columns of the query's result"]);
    }

    #[test]
    fn a_database_table_lists_its_indexes() {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("data.db");
        let mut sql = String::from(
            "CREATE TABLE people (id INTEGER PRIMARY KEY, email TEXT UNIQUE, city TEXT, born DATE);
             CREATE INDEX by_place ON people (city, born);
             CREATE INDEX \"lower\x1b[2J\" ON people (lower(email));
             CREATE TABLE many (x);",
        );
        for i in 0..records::MAX_INDEXES + 1 {
            sql.push_str(&format!("CREATE INDEX many_{i:02} ON many (x);"));
        }
        rusqlite::Connection::open(&path)
            .unwrap()
            .execute_batch(&sql)
            .unwrap();
        let s = load_sqlite(&path, SqlitePart::Table("people"), Budget::unlimited())
            .unwrap()
            .structure
            .unwrap();
        assert_eq!(
            s.about,
            [
                "Table: people",
                "Index by_place: city, born",
                "Index lower·[2J: (expression)",
                "Index sqlite_autoindex_people_1 unique: email",
            ]
        );
        assert_eq!(s.columns[0], ["id", "INTEGER", "", "", "1"]);

        let s = load_sqlite(&path, SqlitePart::Table("many"), Budget::unlimited())
            .unwrap()
            .structure
            .unwrap();
        assert_eq!(s.about.len(), 1 + records::MAX_INDEXES + 1);
        assert_eq!(s.about[1], "Index many_00: x");
        assert_eq!(
            s.about.last().unwrap(),
            &format!("(only the first {} indexes)", records::MAX_INDEXES)
        );
    }

    #[test]
    fn a_parquet_file_declares_arrow_types_and_its_rows() {
        use arrow_array::{Int64Array, RecordBatch, StringArray};
        use arrow_schema::{DataType, Field, Schema};
        use parquet::arrow::arrow_writer::ArrowWriter;
        use std::sync::Arc;

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("label\u{202E}", DataType::Utf8, true),
        ]));
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int64Array::from(vec![1, 2, 3])),
                Arc::new(StringArray::from(vec![Some("a"), None, Some("c")])),
            ],
        )
        .unwrap();
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("data.parquet");
        let file = std::fs::File::create(&path).unwrap();
        let mut writer = ArrowWriter::try_new(file, schema, None).unwrap();
        writer.write(&batch).unwrap();
        writer.close().unwrap();

        let grid = load_parquet(&path, Budget::unlimited()).unwrap();
        let s = grid.structure.expect("Parquet declares its columns");
        assert_eq!(s.titles, ["Column", "Type", "Nulls"]);
        assert_eq!(
            s.columns,
            [["id", "Int64", "no"], ["label·", "Utf8", "yes"]]
        );
        assert_eq!(s.about[..2], ["Rows: 3", "Row groups: 1"]);
        assert!(s.about[2].starts_with("Written by: parquet-rs"), "{s:?}");
    }

    #[test]
    fn a_csv_shows_what_its_rows_have() {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("data.csv");
        std::fs::write(&path, "code,price,note,ok\n7,1.5,hi,true\n8,,there,false\n").unwrap();
        let grid = load_csv(&path, None, Budget::unlimited()).unwrap();
        assert!(grid.structure.is_none());
        let s = grid.structure();
        assert_eq!(s.titles, ["Column", "Values seen", "Widest"]);
        assert_eq!(
            s.columns,
            [
                ["code", "integer", "1"],
                ["price", "decimal, empty", "3"],
                ["note", "text", "5"],
                ["ok", "logical", "5"],
            ]
        );
        assert_eq!(
            s.about,
            ["No types declared here: what the 2 rows read have"]
        );
    }
}
