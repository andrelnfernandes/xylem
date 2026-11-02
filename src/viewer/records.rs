//! Where tables come from: each format as a stream of records.
//!
//! A record has the file's values as they are, typed when the format has
//! types, and neither cut nor cleaned: what they may do on a screen is the
//! grid's concern (`tabular_loader`), and it is the only one that keeps rows.
//! The stream has no row limit: it ends when the file does, when the sink
//! says so, or when the time is up.

use std::borrow::Cow;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek};
use std::ops::ControlFlow;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use arrow_array::Array;
use arrow_array::cast::AsArray;
use arrow_array::types::{
    Float32Type, Float64Type, Int8Type, Int16Type, Int32Type, Int64Type, UInt8Type, UInt16Type,
    UInt32Type, UInt64Type,
};
use arrow_schema::DataType;
use calamine::{
    Cell as SheetCell, Data, DataRef, Dimensions, Ods, Reader, Xls, Xlsb, Xlsx, XlsxError,
};
use parquet::arrow::ProjectionMask;
use parquet::arrow::arrow_reader::{
    ArrowReaderMetadata, ArrowReaderOptions, ParquetRecordBatchReaderBuilder,
};
use serde_json::Value as JsonValue;

use super::parquet_pages;
use super::table::TableGrid;

/// Columns read per record, in any format: a hostile file could have millions.
pub const MAX_COLS: usize = 256;
/// Rows of a spreadsheet, by its format: a cell past them is not real.
const MAX_SHEET_ROWS: u32 = 1 << 20;
/// What the pages of a Parquet row group may hold in memory at once. Not the viewer's
/// limit: 256 columns with the usual 1 MiB pages are a normal file.
const MAX_PARQUET_PAGES: u64 = 1 << 30;
/// A line of JSON Lines longer than this is not read as a record.
const MAX_LINE_BYTES: u64 = 512 * 1024;
/// How many steps of SQLite's virtual machine between looks at the clock.
const SQLITE_STEPS: i32 = 10_000;

/// A value of a record. Text and bytes are borrowed whenever the reader allows
/// it, so that a huge cell costs nothing to whoever only wants its start.
#[derive(Debug, Clone, PartialEq)]
pub enum Cell<'a> {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(Cow<'a, str>),
    Bytes(&'a [u8]),
}

impl Cell<'_> {
    /// The cell as the grid shows it: safe for a terminal, and short.
    pub fn display(&self) -> String {
        match self {
            Cell::Null => String::new(),
            Cell::Bool(b) => b.to_string(),
            Cell::Int(i) => i.to_string(),
            Cell::Float(f) => f.to_string(),
            Cell::Text(s) => TableGrid::sanitize_cell(s),
            Cell::Bytes(b) => format!("<Blob {} bytes>", b.len()),
        }
    }
}

/// How far a reading may go.
#[derive(Debug, Clone)]
pub struct Budget {
    /// When to stop. A reader looks at it now and then, between records.
    deadline: Option<Instant>,
    /// Set by someone else to stop it, looked at along with the deadline.
    cancel: Option<Arc<AtomicBool>>,
    /// The most a spreadsheet may unpack to, checked before it starts.
    unpacked: u64,
}

/// A file that stops being read when the budget runs out. It sits before any
/// decompressor, so it also stops one spinning with no output.
pub struct Stoppable<R> {
    inner: R,
    budget: Budget,
}

impl<R> Stoppable<R> {
    pub fn new(inner: R, budget: Budget) -> Self {
        Stoppable { inner, budget }
    }
}

impl<R: Read> Read for Stoppable<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.budget.over() {
            return Err(std::io::Error::other("the reading was stopped"));
        }
        self.inner.read(buf)
    }
}

impl<R: Seek> Seek for Stoppable<R> {
    fn seek(&mut self, to: std::io::SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(to)
    }
}

impl Budget {
    pub fn new(time: Duration, unpacked: u64) -> Self {
        Budget {
            deadline: Some(Instant::now() + time),
            cancel: None,
            unpacked,
        }
    }

    /// No deadline: it goes on until `cancel` is set.
    pub fn cancellable(unpacked: u64, cancel: Arc<AtomicBool>) -> Self {
        Budget {
            deadline: None,
            cancel: Some(cancel),
            unpacked,
        }
    }

    /// Someone else can stop it too, before the deadline.
    pub fn stopped_by(mut self, cancel: Arc<AtomicBool>) -> Self {
        self.cancel = Some(cancel);
        self
    }

    /// Whether someone stopped it, rather than the time running out.
    pub fn cancelled(&self) -> bool {
        self.cancel
            .as_ref()
            .is_some_and(|c| c.load(Ordering::Relaxed))
    }

    #[cfg(test)]
    pub fn unlimited() -> Self {
        Budget {
            deadline: None,
            cancel: None,
            unpacked: u64::MAX,
        }
    }

    pub fn over(&self) -> bool {
        self.deadline.is_some_and(|d| Instant::now() >= d)
            || self
                .cancel
                .as_ref()
                .is_some_and(|c| c.load(Ordering::Relaxed))
    }
}

/// Why a stream ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum End {
    /// All of the file was read.
    Complete,
    /// The sink had enough.
    Refused,
    /// The time ran out.
    OutOfTime,
}

/// Who receives a stream. Columns come before the records they apply to and
/// may grow at the end (JSON Lines finds its keys as it goes), so a record may
/// be shorter or longer than the current columns.
pub trait RecordSink {
    /// The file's sheets or tables, and which one is being read.
    fn sheets(&mut self, names: &[String], current: usize);
    /// In a database, from which of the names on they are views.
    fn first_view(&mut self, _index: usize) {}
    fn columns(&mut self, names: &[String]);
    /// The file has more than [`MAX_COLS`] columns.
    fn columns_cut(&mut self);
    fn record(&mut self, cells: &[Cell<'_>]) -> ControlFlow<()>;
}

/// Looks at the clock every so many records.
struct Pace {
    budget: Budget,
    count: u32,
}

impl Pace {
    fn new(budget: Budget) -> Self {
        Pace { budget, count: 0 }
    }

    fn over(&mut self) -> bool {
        self.count = self.count.wrapping_add(1);
        self.count & 0xFF == 0 && self.budget.over()
    }
}

/// Gives the sink a record, unless the time is up or it refuses; `None` to go on.
fn deliver(pace: &mut Pace, sink: &mut dyn RecordSink, cells: &[Cell<'_>]) -> Option<End> {
    if pace.over() {
        return Some(End::OutOfTime);
    }
    match sink.record(cells) {
        ControlFlow::Continue(()) => None,
        ControlFlow::Break(()) => Some(End::Refused),
    }
}

/// Spreadsheet column names: 0 -> "A", 25 -> "Z", 26 -> "AA"...
pub fn column_letter(mut col_idx: usize) -> String {
    let mut s = String::new();
    loop {
        let rem = (col_idx % 26) as u8;
        s.push((b'A' + rem) as char);
        if col_idx < 26 {
            break;
        }
        col_idx = (col_idx / 26) - 1;
    }
    s.chars().rev().collect()
}

/// Helper to pick best CSV delimiter by frequency outside quotes.
fn guess_csv_delimiter(sample: &str) -> u8 {
    let mut comma = 0usize;
    let mut semi = 0usize;
    let mut tab = 0usize;
    let mut pipe = 0usize;
    let mut in_quotes = false;

    for c in sample.chars() {
        if c == '"' {
            in_quotes = !in_quotes;
        } else if !in_quotes {
            match c {
                ',' => comma += 1,
                ';' => semi += 1,
                '\t' => tab += 1,
                '|' => pipe += 1,
                _ => {}
            }
        }
    }

    if tab > 0 && tab >= comma && tab >= semi {
        b'\t'
    } else if semi > comma && semi >= pipe {
        b';'
    } else if pipe > comma && pipe > semi {
        b'|'
    } else {
        b','
    }
}

fn text_cells(record: &csv::StringRecord) -> Vec<Cell<'_>> {
    record
        .iter()
        .take(MAX_COLS)
        .map(|field| Cell::Text(Cow::Borrowed(field)))
        .collect()
}

/// A CSV or TSV file: every value is text, since the format has no types.
pub fn read_csv(
    path: &Path,
    delimiter: Option<u8>,
    budget: Budget,
    sink: &mut dyn RecordSink,
) -> Result<End, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open CSV: {e}"))?;
    let mut reader = BufReader::new(file);

    // Auto-detect delimiter from bounded sample
    let delim = match delimiter {
        Some(d) => d,
        None => {
            let mut sample = String::new();
            let _ = (&mut reader).take(8192).read_to_string(&mut sample);
            let _ = reader.seek(std::io::SeekFrom::Start(0));
            guess_csv_delimiter(&sample)
        }
    };

    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(delim)
        .has_headers(false)
        .flexible(true)
        .from_reader(reader);

    let mut records = csv_reader.records();
    let first = match records.next() {
        Some(r) => r.map_err(|e| format!("CSV read error: {e}"))?,
        None => return Ok(End::Complete),
    };
    if first.is_empty() {
        return Ok(End::Complete);
    }

    // Determine if first row is a header: at least half the fields are non-numeric strings
    let non_numeric_count = first
        .iter()
        .filter(|field| {
            let t = field.trim();
            !t.is_empty() && t.parse::<f64>().is_err()
        })
        .count();
    let has_header = non_numeric_count * 2 >= first.len();

    sink.sheets(&["Data".to_string()], 0);
    if first.len() > MAX_COLS {
        sink.columns_cut();
    }
    let mut pace = Pace::new(budget);
    if has_header {
        let names: Vec<String> = first
            .iter()
            .take(MAX_COLS)
            .map(|s| s.trim().to_string())
            .collect();
        sink.columns(&names);
    } else {
        let names: Vec<String> = (0..first.len().min(MAX_COLS)).map(column_letter).collect();
        sink.columns(&names);
        if let Some(end) = deliver(&mut pace, sink, &text_cells(&first)) {
            return Ok(end);
        }
    }

    for result in records {
        let record = result.map_err(|e| format!("CSV record error: {e}"))?;
        if let Some(end) = deliver(&mut pace, sink, &text_cells(&record)) {
            return Ok(end);
        }
    }
    Ok(End::Complete)
}

fn json_cell(value: Option<&JsonValue>) -> Cell<'_> {
    match value {
        None | Some(JsonValue::Null) => Cell::Null,
        Some(JsonValue::Bool(b)) => Cell::Bool(*b),
        Some(JsonValue::String(s)) => Cell::Text(Cow::Borrowed(s)),
        Some(JsonValue::Number(n)) => match (n.as_i64(), n.as_u64(), n.as_f64()) {
            (Some(i), _, _) => Cell::Int(i),
            // Above i64: as text, since a float would lose digits.
            (None, Some(_), _) => Cell::Text(Cow::Owned(n.to_string())),
            (None, None, Some(f)) => Cell::Float(f),
            _ => Cell::Text(Cow::Owned(n.to_string())),
        },
        Some(other) => Cell::Text(Cow::Owned(other.to_string())),
    }
}

/// A JSON Lines (.jsonl / .ndjson) file. Lines that are not objects are skipped.
pub fn read_jsonl(path: &Path, budget: Budget, sink: &mut dyn RecordSink) -> Result<End, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open JSONL: {e}"))?;
    let mut reader = BufReader::new(file);

    let mut names: Vec<String> = Vec::new();
    let mut line = String::new();
    let mut pace = Pace::new(budget);
    let mut cut = false;

    loop {
        line.clear();
        // Read with bounded capacity per line to prevent OOM
        let bytes_read = (&mut reader)
            .take(MAX_LINE_BYTES)
            .read_line(&mut line)
            .map_err(|e| format!("Read error: {e}"))?;
        if bytes_read == 0 {
            return Ok(End::Complete);
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(JsonValue::Object(map)) = serde_json::from_str(trimmed) else {
            continue;
        };

        let known = names.len();
        for key in map.keys() {
            if names.contains(key) {
                continue;
            }
            if names.len() < MAX_COLS {
                names.push(key.clone());
            } else if !cut {
                cut = true;
                sink.columns_cut();
            }
        }
        if names.len() > known {
            if known == 0 {
                sink.sheets(&["JSONL".to_string()], 0);
            }
            sink.columns(&names);
        }
        if names.is_empty() {
            continue;
        }

        let cells: Vec<Cell<'_>> = names.iter().map(|name| json_cell(map.get(name))).collect();
        if let Some(end) = deliver(&mut pace, sink, &cells) {
            return Ok(end);
        }
    }
}

/// A dBase value. Padding is how the format stores short text, not data.
fn dbase_cell(val: &dbase::FieldValue) -> Cell<'_> {
    use dbase::FieldValue as F;
    match val {
        F::Character(Some(s)) => Cell::Text(Cow::Borrowed(s.trim_end())),
        F::Numeric(Some(n)) => Cell::Float(*n),
        F::Logical(Some(b)) => Cell::Bool(*b),
        F::Date(Some(d)) => Cell::Text(Cow::Owned(format!(
            "{:04}-{:02}-{:02}",
            d.year(),
            d.month(),
            d.day()
        ))),
        // Through text, so that 1.1 stays 1.1 and not 1.100000023841858.
        F::Float(Some(f)) => Cell::Float(f.to_string().parse().unwrap_or(f64::from(*f))),
        F::Integer(i) => Cell::Int(i64::from(*i)),
        // As text: money keeps its two decimals.
        F::Currency(c) => Cell::Text(Cow::Owned(format!("{c:.2}"))),
        F::Double(d) => Cell::Float(*d),
        F::DateTime(dt) => {
            let (d, t) = (dt.date(), dt.time());
            Cell::Text(Cow::Owned(format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                d.year(),
                d.month(),
                d.day(),
                t.hours(),
                t.minutes(),
                t.seconds()
            )))
        }
        F::Memo(m) => Cell::Text(Cow::Borrowed(m)),
        _ => Cell::Null,
    }
}

/// A dBase / Clipper / FoxPro .dbf file (with companion .dbt memo file if present).
pub fn read_dbf(path: &Path, budget: Budget, sink: &mut dyn RecordSink) -> Result<End, String> {
    let mut reader = dbase::Reader::from_path(path)
        .map_err(|e| format!("Failed to open DBF {}: {e}", path.display()))?;

    let total = reader.fields().len();
    let names: Vec<String> = reader
        .fields()
        .iter()
        .take(MAX_COLS)
        .map(|f| f.name().to_string())
        .collect();
    if names.is_empty() {
        return Ok(End::Complete);
    }
    sink.sheets(&["DBF".to_string()], 0);
    if total > MAX_COLS {
        sink.columns_cut();
    }
    sink.columns(&names);

    let mut pace = Pace::new(budget);
    for record in reader.iter_records() {
        let record = record.map_err(|e| format!("Error reading DBF record: {e}"))?;
        let cells: Vec<Cell<'_>> = names
            .iter()
            .map(|name| record.get(name).map_or(Cell::Null, dbase_cell))
            .collect();
        if let Some(end) = deliver(&mut pace, sink, &cells) {
            return Ok(end);
        }
    }
    Ok(End::Complete)
}

/// Refuses a spreadsheet that would unpack to more than the budget allows. A
/// spreadsheet is usually a ZIP of XML, so a small file can be enormous inside.
/// The sizes are the ones the ZIP declares.
fn check_unpacked(path: &Path, limit: u64) -> Result<(), String> {
    let file = File::open(path).map_err(|e| format!("Failed to open spreadsheet: {e}"))?;
    let on_disk = file.metadata().map_err(|e| e.to_string())?.len();
    let size = match zip::ZipArchive::new(file) {
        Ok(mut archive) => {
            let mut sum = 0u64;
            for i in 0..archive.len() {
                let entry = archive
                    .by_index_raw(i)
                    .map_err(|e| format!("Failed to read spreadsheet: {e}"))?;
                sum = sum.saturating_add(entry.size());
            }
            sum
        }
        // An old .xls is not a ZIP; if a damaged one is, calamine will say.
        Err(_) => on_disk,
    };
    if size > limit {
        let mib = |n: u64| n.div_ceil(1 << 20);
        return Err(format!(
            "spreadsheet needs {} MiB to read, over the limit of {} MiB",
            mib(size),
            mib(limit)
        ));
    }
    Ok(())
}

fn sheet_cell(value: &Data) -> Cell<'_> {
    match value {
        Data::Empty => Cell::Null,
        Data::Int(i) => Cell::Int(*i),
        Data::Float(f) => Cell::Float(*f),
        Data::Bool(b) => Cell::Bool(*b),
        Data::String(s) => Cell::Text(Cow::Borrowed(s)),
        other => Cell::Text(Cow::Owned(other.to_string())),
    }
}

/// A value read in a stream: a shared string stays borrowed, so that one long
/// string in many cells costs nothing.
fn streamed_cell<'a>(value: &'a DataRef<'a>) -> Cell<'a> {
    match value {
        DataRef::Empty => Cell::Null,
        DataRef::Int(i) => Cell::Int(*i),
        DataRef::Float(f) => Cell::Float(*f),
        DataRef::Bool(b) => Cell::Bool(*b),
        DataRef::SharedString(s) => Cell::Text(Cow::Borrowed(s)),
        DataRef::String(s) => Cell::Text(Cow::Borrowed(s)),
        other => Cell::Text(Cow::Owned(Data::from(other.clone()).to_string())),
    }
}

/// A sheet read cell by cell, keeping one row at a time. The cells come in the
/// file's order, and whoever wrote the file chose their positions: a cell at the
/// far end of the sheet costs no more than any other.
fn stream_sheet<'a, E: std::fmt::Display>(
    declared: Dimensions,
    mut next: impl FnMut() -> Result<Option<SheetCell<DataRef<'a>>>, E>,
    name: &str,
    budget: Budget,
    sink: &mut dyn RecordSink,
) -> Result<End, String> {
    // A letter is the sheet's own, from A: where a sheet says it starts may be wrong,
    // and a cell left of that would be lost. The width it declares is only a start,
    // and a cell past it widens the table, as in JSON Lines.
    let declared_cols = declared.end.1 as usize + 1;
    let mut letters: Vec<String> = Vec::new();
    let mut row: Vec<DataRef<'a>> = Vec::new();
    let mut at: Option<u32> = None;
    let mut cut = false;
    let mut pace = Pace::new(budget);
    let mut flush =
        |row: &mut Vec<DataRef<'a>>, pace: &mut Pace, sink: &mut dyn RecordSink| -> Option<End> {
            let width = row.len().max(declared_cols.min(MAX_COLS));
            if width > letters.len() {
                letters = (0..width).map(column_letter).collect();
                sink.columns(&letters);
            }
            let cells: Vec<Cell<'_>> = row.iter().map(streamed_cell).collect();
            let end = deliver(pace, sink, &cells);
            drop(cells);
            row.clear();
            end
        };
    loop {
        let cell = next().map_err(|e| {
            let shown = TableGrid::sanitize_cell(name);
            format!("Failed to read sheet '{shown}': {e}")
        })?;
        let Some(cell) = cell else { break };
        // Empty cells give no record, so the clock has to be looked at here too.
        if pace.over() {
            return Ok(End::OutOfTime);
        }
        if cell.get_value() == &DataRef::Empty {
            continue;
        }
        let (r, c) = cell.get_position();
        // Past the format's rows, or going back: each row is written once, so what
        // a file can ask for stays within the format's rows.
        if r >= MAX_SHEET_ROWS || at.is_some_and(|at| r < at) {
            continue;
        }
        if at != Some(r) {
            if let Some(previous) = at {
                if let Some(end) = flush(&mut row, &mut pace, sink) {
                    return Ok(end);
                }
                // Empty rows in the middle stay, and the time limit counts them.
                for _ in previous.saturating_add(1)..r {
                    if let Some(end) = flush(&mut row, &mut pace, sink) {
                        return Ok(end);
                    }
                }
            }
            at = Some(r);
        }
        let col = c as usize;
        if col >= MAX_COLS {
            if !cut {
                cut = true;
                sink.columns_cut();
            }
            continue;
        }
        if col >= row.len() {
            row.resize(col + 1, DataRef::Empty);
        }
        let value = cell.get_value().clone();
        row[col] = value;
    }
    if at.is_some()
        && let Some(end) = flush(&mut row, &mut pace, sink)
    {
        return Ok(end);
    }
    Ok(End::Complete)
}

/// A sheet its reader loads whole, there being no other way (`.ods`, `.xls`).
fn range_sheet<RS: Read + Seek>(
    workbook: &mut impl Reader<RS, Error: std::fmt::Display>,
    name: &str,
    budget: Budget,
    sink: &mut dyn RecordSink,
) -> Result<End, String> {
    let range = workbook.worksheet_range(name).map_err(|e| {
        let shown = TableGrid::sanitize_cell(name);
        format!("Failed to read sheet '{shown}': {e}")
    })?;
    let (height, width) = range.get_size();
    let cols = width.min(MAX_COLS);
    if width > MAX_COLS {
        sink.columns_cut();
    }
    let letters: Vec<String> = (0..cols).map(column_letter).collect();
    sink.columns(&letters);

    let mut pace = Pace::new(budget);
    for r in 0..height {
        let cells: Vec<Cell<'_>> = (0..cols)
            .map(|c| range.get((r, c)).map_or(Cell::Null, sheet_cell))
            .collect();
        if let Some(end) = deliver(&mut pace, sink, &cells) {
            return Ok(end);
        }
    }
    Ok(End::Complete)
}

fn open_book<T, RS>(path: &Path, reader: RS) -> Result<T, String>
where
    RS: Read + Seek,
    T: Reader<RS, Error: std::fmt::Display>,
{
    T::new(reader).map_err(|e| format!("Failed to open spreadsheet {}: {e}", path.display()))
}

/// The sheet's name, once the sink knows the file's sheets.
fn pick_sheet(names: &[String], sheet: usize, sink: &mut dyn RecordSink) -> Result<String, String> {
    let Some(name) = names.get(sheet) else {
        return Err("Spreadsheet has no such sheet".to_string());
    };
    sink.sheets(names, sheet);
    Ok(name.clone())
}

/// A sheet (by position) of an Excel / ODS spreadsheet using `calamine`. Its size
/// is checked first, by what the ZIP declares, and that holds: the `zip` crate
/// refuses an entry that gives more than it declared.
pub fn read_spreadsheet(
    path: &Path,
    sheet: usize,
    budget: Budget,
    sink: &mut dyn RecordSink,
) -> Result<End, String> {
    check_unpacked(path, budget.unpacked)?;
    match read_book(path, sheet, budget.clone(), sink) {
        // Whatever calamine made of it, the file stopped being read.
        Err(_) if budget.over() => Ok(End::OutOfTime),
        result => result,
    }
}

fn read_book(
    path: &Path,
    sheet: usize,
    budget: Budget,
    sink: &mut dyn RecordSink,
) -> Result<End, String> {
    let file = File::open(path)
        .map_err(|e| format!("Failed to open spreadsheet {}: {e}", path.display()))?;
    let reader = BufReader::new(Stoppable::new(file, budget.clone()));
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "xlsx" => {
            let mut book: Xlsx<_> = open_book(path, reader)?;
            let name = pick_sheet(&book.sheet_names(), sheet, sink)?;
            let mut cells = match book.worksheet_cells_reader(&name) {
                Ok(cells) => cells,
                // A chart takes a tab of its own, and has no cells.
                Err(XlsxError::NotAWorksheet(_)) => return Ok(End::Complete),
                Err(e) => {
                    let shown = TableGrid::sanitize_cell(&name);
                    return Err(format!("Failed to read sheet '{shown}': {e}"));
                }
            };
            let declared = cells.dimensions();
            stream_sheet(declared, || cells.next_cell(), &name, budget, sink)
        }
        "xlsb" => {
            let mut book: Xlsb<_> = open_book(path, reader)?;
            let name = pick_sheet(&book.sheet_names(), sheet, sink)?;
            let mut cells = book.worksheet_cells_reader(&name).map_err(|e| {
                let shown = TableGrid::sanitize_cell(&name);
                format!("Failed to read sheet '{shown}': {e}")
            })?;
            let declared = cells.dimensions();
            stream_sheet(declared, || cells.next_cell(), &name, budget, sink)
        }
        "ods" => {
            let mut book: Ods<_> = open_book(path, reader)?;
            let name = pick_sheet(&book.sheet_names(), sheet, sink)?;
            range_sheet(&mut book, &name, budget, sink)
        }
        "xls" => {
            let mut book: Xls<_> = open_book(path, reader)?;
            let name = pick_sheet(&book.sheet_names(), sheet, sink)?;
            range_sheet(&mut book, &name, budget, sink)
        }
        _ => Err(format!(
            "Failed to open spreadsheet {}: not a known format",
            path.display()
        )),
    }
}

fn is_interrupted(error: &rusqlite::Error) -> bool {
    matches!(
        error,
        rusqlite::Error::SqliteFailure(e, _) if e.code == rusqlite::ErrorCode::OperationInterrupted
    )
}

/// Why a statement that changes something does not run.
pub const READS_ONLY: &str =
    "Xylem only reads here: to change data use another program (Esc, then O opens one)";

/// What to read from a database. Only what the user typed is ever a query.
#[derive(Debug, Clone, Copy)]
pub enum SqlitePart<'a> {
    First,
    Table(&'a str),
    Query(&'a str),
}

/// A table or query result from an SQLite database in strictly defensive
/// read-only mode.
pub fn read_sqlite(
    path: &Path,
    part: SqlitePart<'_>,
    budget: Budget,
    sink: &mut dyn RecordSink,
) -> Result<End, String> {
    use rusqlite::fallible_iterator::FallibleIterator;
    use rusqlite::{Connection, OpenFlags, ffi};

    // Open connection strictly READONLY (no URI scheme expansion)
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let path_str = path
        .to_str()
        .ok_or_else(|| "Invalid UTF-8 in SQLite path".to_string())?;

    let conn = Connection::open_with_flags(path_str, flags)
        .map_err(|e| format!("Failed to open SQLite database: {e}"))?;

    // Hardening: Defensive mode + Disable extensions
    // SAFETY: conn.handle() returns a valid sqlite3 connection handle managed by rusqlite.
    unsafe {
        let rc = ffi::sqlite3_db_config(
            conn.handle(),
            ffi::SQLITE_DBCONFIG_DEFENSIVE,
            1,
            std::ptr::null_mut::<i32>(),
        );
        if rc != ffi::SQLITE_OK {
            return Err("Failed to enable SQLITE_DBCONFIG_DEFENSIVE".to_string());
        }

        ffi::sqlite3_db_config(
            conn.handle(),
            ffi::SQLITE_DBCONFIG_ENABLE_LOAD_EXTENSION,
            0,
            std::ptr::null_mut::<i32>(),
        );
    }

    // Opening a view runs what the file wrote in it: nothing it names may be unsafe.
    let _ = conn.execute_batch("PRAGMA query_only = ON; PRAGMA trusted_schema = OFF;");

    // A query can take forever without giving a single row.
    if budget.deadline.is_some() || budget.cancel.is_some() {
        let budget = budget.clone();
        conn.progress_handler(SQLITE_STEPS, Some(move || budget.over()));
    }

    // List user tables, then views (bounded to prevent DoS from DB with millions of tables)
    let mut table_stmt = conn
        .prepare(
            "SELECT name, type = 'view' FROM sqlite_master WHERE type IN ('table', 'view') \
             AND name NOT LIKE 'sqlite_%' ORDER BY type = 'view', name LIMIT 1000",
        )
        .map_err(|e| format!("Failed to query tables: {e}"))?;
    let listed: Vec<(String, bool)> = table_stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| format!("Query error: {e}"))?
        .flatten()
        .collect();
    let first_view = listed.iter().filter(|(_, view)| !view).count();
    let tables: Vec<String> = listed.into_iter().map(|(name, _)| name).collect();

    if tables.is_empty() && !matches!(part, SqlitePart::Query(_)) {
        sink.columns(&["Status".to_string()]);
        let _ = sink.record(&[Cell::Text(Cow::Borrowed("(Database has no tables)"))]);
        return Ok(End::Complete);
    }

    // A table's name comes from the file: it is only ever a name, never SQL.
    let select = |idx: usize| format!("SELECT * FROM \"{}\"", tables[idx].replace('"', "\"\""));
    let (query, active) = match part {
        SqlitePart::Query(q) => {
            // SQLite calls PRAGMA and ATTACH read-only too, so the word comes first.
            let reads = q.split_whitespace().next().is_some_and(|w| {
                w.eq_ignore_ascii_case("select") || w.eq_ignore_ascii_case("with")
            });
            if !reads {
                return Err(READS_ONLY.to_string());
            }
            let mut statements = rusqlite::Batch::new(&conn, q);
            let first = statements
                .next()
                .map_err(|e| format!("SQL statement error: {e}"))?;
            if !first.is_some_and(|s| s.readonly()) {
                return Err(READS_ONLY.to_string());
            }
            if !matches!(statements.next(), Ok(None)) {
                return Err("Only one statement can run at a time".to_string());
            }
            (q.to_string(), 0)
        }
        SqlitePart::Table(tbl) => {
            let idx = tables
                .iter()
                .position(|t| t == tbl)
                .or_else(|| {
                    tables
                        .iter()
                        .position(|t| TableGrid::sanitize_cell(t) == tbl)
                })
                .unwrap_or(0);
            (select(idx), idx)
        }
        SqlitePart::First => (select(0), 0),
    };
    sink.sheets(&tables, active);
    sink.first_view(first_view);

    let mut stmt = conn
        .prepare(&query)
        .map_err(|e| format!("SQL statement error: {e}"))?;

    let total = stmt.column_count();
    let names: Vec<String> = stmt
        .column_names()
        .into_iter()
        .take(MAX_COLS)
        .map(str::to_string)
        .collect();
    let cols = names.len();
    if total > MAX_COLS {
        sink.columns_cut();
    }
    sink.columns(&names);

    let mut rows = stmt
        .query([])
        .map_err(|e| format!("SQL execution error: {e}"))?;

    let mut pace = Pace::new(budget);
    loop {
        let row = match rows.next() {
            Ok(Some(row)) => row,
            Ok(None) => return Ok(End::Complete),
            Err(e) if is_interrupted(&e) => return Ok(End::OutOfTime),
            Err(e) => return Err(format!("Row read error: {e}")),
        };
        let cells: Vec<Cell<'_>> = (0..cols)
            .map(|i| match row.get_ref(i) {
                Ok(rusqlite::types::ValueRef::Null) | Err(_) => Cell::Null,
                Ok(rusqlite::types::ValueRef::Integer(i)) => Cell::Int(i),
                Ok(rusqlite::types::ValueRef::Real(r)) => Cell::Float(r),
                Ok(rusqlite::types::ValueRef::Text(t)) => Cell::Text(String::from_utf8_lossy(t)),
                Ok(rusqlite::types::ValueRef::Blob(b)) => Cell::Bytes(b),
            })
            .collect();
        if let Some(end) = deliver(&mut pace, sink, &cells) {
            return Ok(end);
        }
    }
}

/// A value of a Parquet column. Types with no [`Cell`] of their own (dates,
/// decimals, lists...) come as Arrow writes them.
fn parquet_cell(col: &dyn Array, r: usize) -> Cell<'_> {
    if col.is_null(r) {
        return Cell::Null;
    }
    match col.data_type() {
        DataType::Boolean => Cell::Bool(col.as_boolean().value(r)),
        DataType::Int8 => Cell::Int(i64::from(col.as_primitive::<Int8Type>().value(r))),
        DataType::Int16 => Cell::Int(i64::from(col.as_primitive::<Int16Type>().value(r))),
        DataType::Int32 => Cell::Int(i64::from(col.as_primitive::<Int32Type>().value(r))),
        DataType::Int64 => Cell::Int(col.as_primitive::<Int64Type>().value(r)),
        DataType::UInt8 => Cell::Int(i64::from(col.as_primitive::<UInt8Type>().value(r))),
        DataType::UInt16 => Cell::Int(i64::from(col.as_primitive::<UInt16Type>().value(r))),
        DataType::UInt32 => Cell::Int(i64::from(col.as_primitive::<UInt32Type>().value(r))),
        DataType::UInt64 => {
            let value = col.as_primitive::<UInt64Type>().value(r);
            i64::try_from(value).map_or_else(|_| Cell::Text(value.to_string().into()), Cell::Int)
        }
        // Through text, so that 1.1 stays 1.1 and not 1.100000023841858.
        DataType::Float32 => {
            let value = col.as_primitive::<Float32Type>().value(r);
            Cell::Float(value.to_string().parse().unwrap_or(f64::from(value)))
        }
        DataType::Float64 => Cell::Float(col.as_primitive::<Float64Type>().value(r)),
        DataType::Utf8 => Cell::Text(col.as_string::<i32>().value(r).into()),
        DataType::LargeUtf8 => Cell::Text(col.as_string::<i64>().value(r).into()),
        DataType::Utf8View => Cell::Text(col.as_string_view().value(r).into()),
        DataType::Binary => Cell::Bytes(col.as_binary::<i32>().value(r)),
        DataType::LargeBinary => Cell::Bytes(col.as_binary::<i64>().value(r)),
        DataType::BinaryView => Cell::Bytes(col.as_binary_view().value(r)),
        _ => Cell::Text(
            arrow_cast::display::array_value_to_string(col, r)
                .unwrap_or_else(|_| "<err>".to_string())
                .into(),
        ),
    }
}

/// Whichever format the file's extension says; `sheet` is the position of a
/// spreadsheet's sheet, and `table` the name of a database's table.
pub fn read_table(
    path: &Path,
    sheet: usize,
    part: SqlitePart<'_>,
    budget: Budget,
    sink: &mut dyn RecordSink,
) -> Result<End, String> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "csv" | "tsv" | "tab" => read_csv(path, None, budget, sink),
        "jsonl" | "ndjson" => read_jsonl(path, budget, sink),
        "xlsx" | "ods" | "xls" | "xlsb" => read_spreadsheet(path, sheet, budget, sink),
        "dbf" => read_dbf(path, budget, sink),
        "sqlite" | "sqlite3" | "db" => read_sqlite(path, part, budget, sink),
        "parquet" => read_parquet(path, budget, sink),
        _ => Err("Not supported".to_string()),
    }
}

/// An Apache Parquet file (.parquet). Columns past [`MAX_COLS`] are never
/// decoded from disk.
pub fn read_parquet(path: &Path, budget: Budget, sink: &mut dyn RecordSink) -> Result<End, String> {
    read_parquet_within(path, MAX_PARQUET_PAGES, budget, sink)
}

fn read_parquet_within(
    path: &Path,
    pages: u64,
    budget: Budget,
    sink: &mut dyn RecordSink,
) -> Result<End, String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open Parquet file: {e}"))?;

    // Validate size and magic markers (PAR1 at start and end)
    let file_len = file.metadata().map_err(|e| e.to_string())?.len();
    if file_len < 12 {
        return Err("File too small to be a valid Parquet file (min 12 bytes)".to_string());
    }

    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)
        .map_err(|e| format!("Failed to read Parquet magic header: {e}"))?;
    if &magic != b"PAR1" {
        return Err("Invalid Parquet file: missing PAR1 magic header".to_string());
    }

    // Validate footer PAR1 marker
    file.seek(std::io::SeekFrom::End(-4))
        .map_err(|e| format!("Seek to footer failed: {e}"))?;
    file.read_exact(&mut magic)
        .map_err(|e| format!("Failed to read Parquet magic footer: {e}"))?;
    if &magic != b"PAR1" {
        return Err("Invalid Parquet file: missing PAR1 magic footer".to_string());
    }

    // Rewind back to beginning for Parquet reader
    file.seek(std::io::SeekFrom::Start(0))
        .map_err(|e| format!("Seek failed: {e}"))?;

    let metadata = ArrowReaderMetadata::load(&file, ArrowReaderOptions::new())
        .map_err(|e| format!("Failed to parse Parquet metadata: {e}"))?;

    let schema = metadata.schema().clone();
    let total = schema.fields().len();
    if total == 0 {
        return Ok(End::Complete);
    }
    let cols = total.min(MAX_COLS);
    let names: Vec<String> = schema.fields()[..cols]
        .iter()
        .map(|f| f.name().to_string())
        .collect();
    sink.sheets(&["Parquet".to_string()], 0);
    if total > MAX_COLS {
        sink.columns_cut();
    }
    sink.columns(&names);

    let mask = ProjectionMask::roots(metadata.parquet_schema(), 0..cols);
    let mut pace = Pace::new(budget.clone());
    // One row group at a time: the grid stops early, and pays only for what it reads.
    for group in 0..metadata.metadata().num_row_groups() {
        // An empty group has nothing else that looks at the clock.
        if budget.over() {
            return Ok(End::OutOfTime);
        }
        let handle = file
            .try_clone()
            .map_err(|e| format!("Failed to open Parquet file: {e}"))?;
        let guarded = match parquet_pages::GuardedFile::new(
            handle,
            metadata.metadata().row_group(group),
            |leaf| mask.leaf_included(leaf),
            pages,
            budget.clone(),
        ) {
            Ok(guarded) => guarded,
            Err(_) if budget.over() => return Ok(End::OutOfTime),
            Err(e) => return Err(format!("Refused Parquet file: {e}")),
        };
        let batch_size = parquet_batch(guarded.dictionaries(), pages);
        let reader = ParquetRecordBatchReaderBuilder::new_with_metadata(guarded, metadata.clone())
            .with_projection(mask.clone())
            .with_row_groups(vec![group])
            .with_batch_size(batch_size)
            .build()
            .map_err(|e| format!("Failed to create Parquet batch reader: {e}"))?;

        for batch in reader {
            let batch = match batch {
                Ok(batch) => batch,
                Err(_) if budget.over() => return Ok(End::OutOfTime),
                Err(e) => return Err(format!("Refused Parquet file: {e}")),
            };
            let columns = batch.columns();
            for r in 0..batch.num_rows() {
                let cells: Vec<Cell<'_>> = columns
                    .iter()
                    .take(cols)
                    .map(|col| parquet_cell(col.as_ref(), r))
                    .collect();
                if let Some(end) = deliver(&mut pace, sink, &cells) {
                    return Ok(end);
                }
            }
        }
    }
    Ok(End::Complete)
}

/// Rows per batch. A value of a dictionary is copied into each row that uses it, so
/// a huge dictionary means few rows at a time.
fn parquet_batch(dictionaries: u64, limit: u64) -> usize {
    let rows = limit.checked_div(dictionaries).unwrap_or(1024);
    rows.clamp(1, 1024) as usize
}

/// Hand-made spreadsheets, for what no writer of them would produce.
#[cfg(test)]
pub mod test_book {
    use std::io::Write;
    use std::path::Path;

    const SHEET: &str = "http://schemas.openxmlformats.org/spreadsheetml/2006/main";
    const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
    const PKG: &str = "http://schemas.openxmlformats.org/package/2006";
    const MAIN: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml";

    /// `.xlsx` with one sheet; `body` has its rows, and `<dimension>` if any.
    pub fn write(path: &Path, body: &str) {
        write_parts(path, body, None);
    }

    /// The same, with one shared string.
    pub fn write_with_strings(path: &Path, body: &str, string: &str) {
        write_parts(path, body, Some(string));
    }

    fn write_parts(path: &Path, body: &str, string: Option<&str>) {
        let mut zip = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
        let options = zip::write::SimpleFileOptions::default();
        let mut add = |name: &str, content: String| {
            zip.start_file(name, options).unwrap();
            zip.write_all(content.as_bytes()).unwrap();
        };
        add(
            "[Content_Types].xml",
            format!(
                r#"<Types xmlns="{PKG}/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Override PartName="/xl/workbook.xml" ContentType="{MAIN}.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="{MAIN}.worksheet+xml"/></Types>"#
            ),
        );
        add(
            "_rels/.rels",
            format!(
                r#"<Relationships xmlns="{PKG}/relationships"><Relationship Id="rId1" Type="{REL}/officeDocument" Target="xl/workbook.xml"/></Relationships>"#
            ),
        );
        add(
            "xl/workbook.xml",
            format!(
                r#"<workbook xmlns="{SHEET}" xmlns:r="{REL}"><sheets><sheet name="s" sheetId="1" r:id="rId1"/></sheets></workbook>"#
            ),
        );
        add(
            "xl/_rels/workbook.xml.rels",
            format!(
                r#"<Relationships xmlns="{PKG}/relationships"><Relationship Id="rId1" Type="{REL}/worksheet" Target="worksheets/sheet1.xml"/></Relationships>"#
            ),
        );
        if let Some(string) = string {
            add(
                "xl/sharedStrings.xml",
                format!(r#"<sst xmlns="{SHEET}"><si><t>{string}</t></si></sst>"#),
            );
        }
        let (dimension, rows) = match body.strip_prefix("<dimension") {
            Some(rest) => {
                let end = rest.find("/>").unwrap() + 2;
                (format!("<dimension{}", &rest[..end]), &rest[end..])
            }
            None => (String::new(), body),
        };
        add(
            "xl/worksheets/sheet1.xml",
            format!(
                r#"<worksheet xmlns="{SHEET}">{dimension}<sheetData>{rows}</sheetData></worksheet>"#
            ),
        );
        drop(add);
        zip.finish().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parquet::file::properties::WriterProperties;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// A value with nothing borrowed, to compare with.
    #[derive(Debug, Clone, PartialEq)]
    enum Owned {
        Null,
        Bool(bool),
        Int(i64),
        Float(f64),
        Text(String),
        Bytes(Vec<u8>),
    }

    fn text(s: &str) -> Owned {
        Owned::Text(s.to_string())
    }

    /// Keeps everything it receives, up to `stop` records.
    #[derive(Default)]
    struct Collect {
        columns: Vec<String>,
        rows: Vec<Vec<Owned>>,
        stop: Option<usize>,
        cut: bool,
    }

    impl RecordSink for Collect {
        fn sheets(&mut self, _: &[String], _: usize) {}
        fn columns(&mut self, names: &[String]) {
            self.columns = names.to_vec();
        }
        fn columns_cut(&mut self) {
            self.cut = true;
        }
        fn record(&mut self, cells: &[Cell<'_>]) -> ControlFlow<()> {
            if self.stop == Some(self.rows.len()) {
                return ControlFlow::Break(());
            }
            self.rows.push(
                cells
                    .iter()
                    .map(|c| match c {
                        Cell::Null => Owned::Null,
                        Cell::Bool(b) => Owned::Bool(*b),
                        Cell::Int(i) => Owned::Int(*i),
                        Cell::Float(f) => Owned::Float(*f),
                        Cell::Text(s) => Owned::Text(s.to_string()),
                        Cell::Bytes(b) => Owned::Bytes(b.to_vec()),
                    })
                    .collect(),
            );
            ControlFlow::Continue(())
        }
    }

    fn file_with(text: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(text.as_bytes()).unwrap();
        file
    }

    fn expired() -> Budget {
        Budget {
            deadline: Some(Instant::now()),
            cancel: None,
            unpacked: u64::MAX,
        }
    }

    #[test]
    fn values_come_as_they_are() {
        let long = "x".repeat(10_000);
        let file = file_with(&format!(
            "name,code\n\"a\u{1b}[31m\u{202E}b\",007\n{long},1\n"
        ));
        let mut sink = Collect::default();
        let end = read_csv(file.path(), None, Budget::unlimited(), &mut sink).unwrap();
        assert_eq!(end, End::Complete);
        assert_eq!(sink.columns, ["name", "code"]);
        assert_eq!(sink.rows[0], [text("a\u{1b}[31m\u{202E}b"), text("007")]);
        assert_eq!(sink.rows[1], [text(&long), text("1")]);
    }

    #[test]
    fn the_stream_has_no_row_limit() {
        let mut body = String::from("a,b\n");
        for i in 0..60_000 {
            body.push_str(&format!("{i},x\n"));
        }
        let file = file_with(&body);
        let mut sink = Collect::default();
        let end = read_csv(file.path(), None, Budget::unlimited(), &mut sink).unwrap();
        assert_eq!(end, End::Complete);
        assert_eq!(sink.rows.len(), 60_000);
        assert_eq!(sink.rows[59_999][0], text("59999"));
    }

    #[test]
    fn the_sink_can_stop_the_stream() {
        let file = file_with("a\nb\nc\nd\n");
        let mut sink = Collect {
            stop: Some(1),
            ..Default::default()
        };
        let end = read_csv(file.path(), Some(b','), Budget::unlimited(), &mut sink).unwrap();
        assert_eq!(end, End::Refused);
        assert_eq!(sink.rows.len(), 1);
    }

    #[test]
    fn a_stream_stops_when_the_time_is_up() {
        let mut body = String::from("a\n");
        for i in 0..2_000 {
            body.push_str(&format!("{i}x\n"));
        }
        let file = file_with(&body);
        let mut sink = Collect::default();
        let end = read_csv(file.path(), None, expired(), &mut sink).unwrap();
        assert_eq!(end, End::OutOfTime);
        assert!(sink.rows.len() < 2_000);
    }

    #[test]
    fn json_lines_keep_their_types_and_find_columns_as_they_go() {
        let file = file_with(concat!(
            r#"{"id": 1, "ok": true, "pi": 3.5, "s": "007"}"#,
            "\n\nnot json\n",
            r#"{"id": 18446744073709551615, "list": [1, 2], "s": null}"#,
            "\n"
        ));
        let mut sink = Collect::default();
        read_jsonl(file.path(), Budget::unlimited(), &mut sink).unwrap();
        assert_eq!(sink.columns, ["id", "ok", "pi", "s", "list"]);
        assert_eq!(
            sink.rows[0],
            [
                Owned::Int(1),
                Owned::Bool(true),
                Owned::Float(3.5),
                text("007")
            ]
        );
        assert_eq!(
            sink.rows[1],
            [
                text("18446744073709551615"),
                Owned::Null,
                Owned::Null,
                Owned::Null,
                text("[1,2]")
            ]
        );
    }

    #[test]
    fn dbase_values_have_types() {
        use dbase::{Date, FieldValue as F};
        assert_eq!(
            dbase_cell(&F::Character(Some("Test  ".to_string()))),
            Cell::Text("Test".into())
        );
        assert_eq!(dbase_cell(&F::Numeric(Some(123.45))), Cell::Float(123.45));
        assert_eq!(dbase_cell(&F::Logical(Some(true))), Cell::Bool(true));
        assert_eq!(dbase_cell(&F::Logical(None)), Cell::Null);
        assert_eq!(dbase_cell(&F::Float(Some(1.1))), Cell::Float(1.1));
        assert_eq!(
            dbase_cell(&F::Date(Date::new(30, 10, 2026).ok())),
            Cell::Text("2026-10-30".into())
        );
        assert_eq!(dbase_cell(&F::Currency(42.5)), Cell::Text("42.50".into()));
        let memo = "m".repeat(5_000);
        assert_eq!(
            dbase_cell(&F::Memo(memo.clone())),
            Cell::Text(memo.as_str().into()),
            "not cut"
        );
    }

    fn database(sql: &str) -> NamedTempFile {
        let file = NamedTempFile::new().unwrap();
        rusqlite::Connection::open(file.path())
            .unwrap()
            .execute_batch(sql)
            .unwrap();
        file
    }

    #[test]
    fn sqlite_gives_its_types_and_whole_values() {
        let long = "y".repeat(20_000);
        let file = database(&format!(
            "CREATE TABLE t (a, b, c, d, e);
             INSERT INTO t VALUES (1, 2.5, '{long}', x'00FF', NULL);"
        ));
        let mut sink = Collect::default();
        read_sqlite(
            file.path(),
            SqlitePart::First,
            Budget::unlimited(),
            &mut sink,
        )
        .unwrap();
        assert_eq!(sink.columns, ["a", "b", "c", "d", "e"]);
        assert_eq!(
            sink.rows[0],
            [
                Owned::Int(1),
                Owned::Float(2.5),
                text(&long),
                Owned::Bytes(vec![0, 255]),
                Owned::Null
            ]
        );
    }

    #[test]
    fn a_table_name_is_only_a_name_never_sql() {
        let file = database(
            r#"CREATE TABLE "SELECT 1 AS hacked" (real_column);
               INSERT INTO "SELECT 1 AS hacked" VALUES ('from the table');"#,
        );
        let mut sink = Collect::default();
        read_sqlite(
            file.path(),
            SqlitePart::Table("SELECT 1 AS hacked"),
            Budget::unlimited(),
            &mut sink,
        )
        .unwrap();
        assert_eq!(sink.columns, ["real_column"]);
    }

    #[test]
    fn only_one_query_that_reads_can_run() {
        let file = database("CREATE TABLE t (a); INSERT INTO t VALUES (1);");
        let run = |query: &str| {
            read_sqlite(
                file.path(),
                SqlitePart::Query(query),
                Budget::unlimited(),
                &mut Collect::default(),
            )
        };
        assert!(run("SELECT a FROM t;").is_ok());
        assert!(run("WITH c AS (SELECT a FROM t) SELECT * FROM c").is_ok());
        for query in [
            "DELETE FROM t",
            "WITH c AS (SELECT 1) DELETE FROM t",
            "SELECT 1; DELETE FROM t",
            "SELECT 1; SELECT 2",
            "DROP TABLE t",
            "PRAGMA writable_schema = ON",
            "ATTACH DATABASE 'other.db' AS other",
            "/* hidden */ ATTACH DATABASE 'other.db' AS other",
            "SELECT 1; ATTACH DATABASE 'other.db' AS other",
            "SELECT nothing FROM nowhere",
            "   ",
        ] {
            assert!(run(query).is_err(), "{query}");
        }
        let db = rusqlite::Connection::open(file.path()).unwrap();
        let rows: i64 = db
            .query_row("SELECT count(*) FROM t", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 1);
    }

    #[test]
    fn a_query_that_never_gives_a_row_is_stopped() {
        let file = database("CREATE TABLE t (a);");
        let query = "SELECT x FROM (WITH RECURSIVE c(x) AS \
                     (SELECT 1 UNION ALL SELECT x + 1 FROM c) SELECT x FROM c) WHERE x < 0";
        let budget = Budget::new(Duration::from_millis(100), u64::MAX);
        let mut sink = Collect::default();
        let end = read_sqlite(file.path(), SqlitePart::Query(query), budget, &mut sink).unwrap();
        assert_eq!(end, End::OutOfTime);
        assert!(sink.rows.is_empty());
    }

    #[test]
    fn a_spreadsheet_over_the_limit_is_not_read() {
        let file = NamedTempFile::new().unwrap();
        {
            let mut zip = zip::ZipWriter::new(file.reopen().unwrap());
            zip.start_file("xl/big.xml", zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(&vec![b' '; 3 << 20]).unwrap();
            zip.finish().unwrap();
        }
        let error = check_unpacked(file.path(), 1 << 20).unwrap_err();
        assert_eq!(
            error,
            "spreadsheet needs 3 MiB to read, over the limit of 1 MiB"
        );
        assert!(check_unpacked(file.path(), 4 << 20).is_ok());
    }

    #[test]
    fn parquet_gives_its_types() {
        use arrow_array::{BooleanArray, Float32Array, Int64Array, RecordBatch, StringArray};
        use arrow_schema::{Field, Schema};
        use parquet::arrow::arrow_writer::ArrowWriter;
        use std::sync::Arc;

        let schema = Arc::new(Schema::new(vec![
            Field::new("i", DataType::Int64, false),
            Field::new("f", DataType::Float32, false),
            Field::new("b", DataType::Boolean, false),
            Field::new("s", DataType::Utf8, true),
        ]));
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int64Array::from(vec![7])),
                Arc::new(Float32Array::from(vec![1.1])),
                Arc::new(BooleanArray::from(vec![true])),
                Arc::new(StringArray::from(vec![None::<&str>])),
            ],
        )
        .unwrap();
        let file = NamedTempFile::new().unwrap();
        {
            let mut writer = ArrowWriter::try_new(file.reopen().unwrap(), schema, None).unwrap();
            writer.write(&batch).unwrap();
            writer.close().unwrap();
        }
        let mut sink = Collect::default();
        read_parquet(file.path(), Budget::unlimited(), &mut sink).unwrap();
        assert_eq!(
            sink.rows[0],
            [
                Owned::Int(7),
                Owned::Float(1.1),
                Owned::Bool(true),
                Owned::Null
            ]
        );
    }

    fn collect_sheet(body: &str) -> (Collect, Result<End, String>) {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("book.xlsx");
        test_book::write(&path, body);
        let mut sink = Collect::default();
        let end = read_spreadsheet(&path, 0, Budget::unlimited(), &mut sink);
        (sink, end)
    }

    #[test]
    fn cells_far_apart_cost_no_grid() {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("far.xlsx");
        // With a grid of the sheet's size, this asks for 550 GiB, and that aborts.
        test_book::write(
            &path,
            r#"<row r="1"><c r="A1"><v>1</v></c></row><row r="1048576"><c r="XFD1048576"><v>2</v></c></row>"#,
        );
        assert!(std::fs::metadata(&path).unwrap().len() < 4096);
        let mut sink = Collect {
            stop: Some(5),
            ..Default::default()
        };
        let end = read_spreadsheet(&path, 0, Budget::unlimited(), &mut sink).unwrap();
        assert_eq!(end, End::Refused);
        assert_eq!(sink.rows[0], [Owned::Float(1.0)]);
        assert_eq!(sink.rows[1], [] as [Owned; 0]);
    }

    #[test]
    fn a_sheet_has_its_real_letters_and_empty_rows() {
        let (sink, end) = collect_sheet(
            r#"<dimension ref="C2:D5"/><row r="2"><c r="C2"><v>1</v></c></row><row r="4"><c r="D4" t="inlineStr"><is><t>x</t></is></c></row>"#,
        );
        assert_eq!(end, Ok(End::Complete));
        assert_eq!(sink.columns, ["A", "B", "C", "D"]);
        assert_eq!(sink.rows.len(), 3);
        assert_eq!(sink.rows[0], [Owned::Null, Owned::Null, Owned::Float(1.0)]);
        assert_eq!(sink.rows[1], [] as [Owned; 0]);
        assert_eq!(
            sink.rows[2],
            [Owned::Null, Owned::Null, Owned::Null, text("x")]
        );
    }

    #[test]
    fn a_wrong_dimension_hides_no_data() {
        let (sink, _) = collect_sheet(
            r#"<dimension ref="B2:D5"/><row r="2"><c r="A2"><v>1</v></c><c r="B2"><v>2</v></c></row>"#,
        );
        assert_eq!(sink.rows[0], [Owned::Float(1.0), Owned::Float(2.0)]);
    }

    #[test]
    fn rows_that_go_back_give_no_more_empty_rows() {
        let far = r#"<row r="1"><c r="A1"><v>1</v></c></row><row r="1048576"><c r="A1048576"><v>2</v></c></row>"#;
        let (sink, end) = collect_sheet(&far.repeat(3));
        assert_eq!(end, Ok(End::Complete));
        assert_eq!(sink.rows.len(), 1 << 20, "the format's rows, at most");
    }

    #[test]
    fn a_sheet_with_no_dimension_finds_its_columns_as_it_goes() {
        let (sink, _) = collect_sheet(
            r#"<row r="1"><c r="A1"><v>1</v></c></row><row r="2"><c r="B2"><v>2</v></c></row>"#,
        );
        assert_eq!(sink.columns, ["A", "B"]);
        assert_eq!(sink.rows[1], [Owned::Null, Owned::Float(2.0)]);
    }

    #[test]
    fn a_sheet_that_lies_about_its_cells_gets_nowhere() {
        // Past the format's rows, past the columns kept.
        let (sink, end) = collect_sheet(
            r#"<dimension ref="B1:B1"/><row r="1"><c r="B1"><v>2</v></c><c r="ZZ1"><v>3</v></c></row><row r="2000000"><c r="B2000000"><v>4</v></c></row>"#,
        );
        assert_eq!(end, Ok(End::Complete));
        assert_eq!(sink.rows, [[Owned::Null, Owned::Float(2.0)]]);
        assert!(sink.cut);
    }

    #[test]
    fn a_shared_string_is_borrowed_not_copied() {
        let shared = DataRef::SharedString("long");
        assert!(matches!(
            streamed_cell(&shared),
            Cell::Text(Cow::Borrowed("long"))
        ));
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("s.xlsx");
        test_book::write_with_strings(
            &path,
            r#"<row r="1"><c r="A1" t="s"><v>0</v></c><c r="B1" t="s"><v>0</v></c></row>"#,
            "same",
        );
        let mut sink = Collect::default();
        read_spreadsheet(&path, 0, Budget::unlimited(), &mut sink).unwrap();
        assert_eq!(sink.rows[0], [text("same"), text("same")]);
    }

    #[test]
    fn a_zip_that_lies_about_its_sizes_is_refused() {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("liar.xlsx");
        let body: String = (1..5_000)
            .map(|i| format!(r#"<row r="{i}"><c r="A{i}"><v>{i}</v></c></row>"#))
            .collect();
        test_book::write(&path, &body);
        // The central directory says the sheet is only 10 bytes, which is what
        // the check of the size goes by.
        let mut bytes = std::fs::read(&path).unwrap();
        let name = b"xl/worksheets/sheet1.xml";
        let central = bytes
            .windows(46 + name.len())
            .position(|w| w.starts_with(&[0x50, 0x4b, 1, 2]) && w.ends_with(name))
            .unwrap();
        bytes[central + 24..central + 28].copy_from_slice(&10u32.to_le_bytes());
        std::fs::write(&path, bytes).unwrap();

        assert!(check_unpacked(&path, 1 << 20).is_ok());
        let mut sink = Collect::default();
        let error = read_spreadsheet(&path, 0, Budget::unlimited(), &mut sink).unwrap_err();
        assert!(error.contains("declared"), "{error}");
        assert!(sink.rows.is_empty());
    }

    #[test]
    fn a_spreadsheet_stops_when_the_time_is_up_even_while_opening() {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("book.xlsx");
        test_book::write(&path, r#"<row r="1"><c r="A1"><v>1</v></c></row>"#);
        let mut sink = Collect::default();
        let end = read_spreadsheet(&path, 0, expired(), &mut sink).unwrap();
        assert_eq!(end, End::OutOfTime);
        assert!(sink.rows.is_empty());
    }

    #[test]
    fn a_stopped_spreadsheet_says_so_midway() {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("book.xlsx");
        let body: String = (1..2_000)
            .map(|i| format!(r#"<row r="{i}"><c r="A{i}"><v>{i}</v></c></row>"#))
            .collect();
        test_book::write(&path, &body);

        /// Asks to stop at the first record.
        struct Stopper(Arc<AtomicBool>, usize);
        impl RecordSink for Stopper {
            fn sheets(&mut self, _: &[String], _: usize) {}
            fn columns(&mut self, _: &[String]) {}
            fn columns_cut(&mut self) {}
            fn record(&mut self, _: &[Cell<'_>]) -> ControlFlow<()> {
                self.0.store(true, Ordering::Relaxed);
                self.1 += 1;
                ControlFlow::Continue(())
            }
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let mut sink = Stopper(cancel.clone(), 0);
        let budget = Budget::cancellable(u64::MAX, cancel);
        let end = read_spreadsheet(&path, 0, budget, &mut sink).unwrap();
        assert_eq!(end, End::OutOfTime);
        assert!((1..1_999).contains(&sink.1), "{}", sink.1);
    }

    #[test]
    fn an_empty_sheet_is_no_error() {
        let (sink, end) = collect_sheet("");
        assert_eq!(end, Ok(End::Complete));
        assert!(sink.rows.is_empty());
    }

    #[test]
    fn a_file_with_no_known_extension_is_not_a_spreadsheet() {
        let file = file_with("x");
        let error = read_spreadsheet(file.path(), 0, Budget::unlimited(), &mut Collect::default())
            .unwrap_err();
        assert!(error.contains("not a known format"), "{error}");
    }

    fn parquet_file(properties: WriterProperties, batches: usize) -> NamedTempFile {
        use arrow_array::{Int64Array, RecordBatch, StringArray};
        use arrow_schema::{Field, Schema};
        use parquet::arrow::arrow_writer::ArrowWriter;

        let schema = Arc::new(Schema::new(vec![
            Field::new("i", DataType::Int64, true),
            Field::new("s", DataType::Utf8, true),
        ]));
        let file = NamedTempFile::new().unwrap();
        let mut writer =
            ArrowWriter::try_new(file.reopen().unwrap(), schema.clone(), Some(properties)).unwrap();
        for _ in 0..batches {
            let ints: Vec<Option<i64>> = (0..5_000).map(|i| (i % 7 != 0).then_some(i)).collect();
            let strings: Vec<String> = (0..5_000).map(|i| format!("value {}", i % 50)).collect();
            let batch = RecordBatch::try_new(
                schema.clone(),
                vec![
                    Arc::new(Int64Array::from(ints)),
                    Arc::new(StringArray::from(strings)),
                ],
            )
            .unwrap();
            writer.write(&batch).unwrap();
            writer.flush().unwrap();
        }
        writer.close().unwrap();
        file
    }

    fn dictionaries_of(file: File) -> u64 {
        let metadata = ArrowReaderMetadata::load(&file, ArrowReaderOptions::new()).unwrap();
        let group = metadata.metadata().row_group(0);
        let budget = Budget::unlimited();
        parquet_pages::GuardedFile::new(file, group, |_| true, 1 << 30, budget)
            .unwrap()
            .dictionaries()
    }

    #[test]
    fn every_parquet_codec_and_page_kind_passes_the_check_and_reads() {
        use parquet::basic::{Compression, GzipLevel};
        use parquet::file::properties::WriterVersion;
        for compression in [
            Compression::UNCOMPRESSED,
            Compression::SNAPPY,
            Compression::GZIP(GzipLevel::default()),
            Compression::ZSTD(Default::default()),
        ] {
            for version in [WriterVersion::PARQUET_1_0, WriterVersion::PARQUET_2_0] {
                for dictionary in [true, false] {
                    let properties = WriterProperties::builder()
                        .set_compression(compression)
                        .set_writer_version(version)
                        .set_dictionary_enabled(dictionary)
                        .build();
                    let file = parquet_file(properties, 2);
                    let dictionaries = dictionaries_of(file.reopen().unwrap());
                    assert_eq!(dictionaries > 0, dictionary, "{compression:?} {version:?}");
                    let mut sink = Collect::default();
                    let end = read_parquet(file.path(), Budget::unlimited(), &mut sink).unwrap();
                    assert_eq!(end, End::Complete);
                    assert_eq!(sink.rows.len(), 10_000, "both row groups");
                    assert_eq!(sink.rows[8][0], Owned::Int(8));
                }
            }
        }
    }

    #[test]
    fn parquet_pages_over_the_limit_are_refused() {
        let file = parquet_file(WriterProperties::builder().build(), 1);
        let mut sink = Collect::default();
        let error =
            read_parquet_within(file.path(), 100, Budget::unlimited(), &mut sink).unwrap_err();
        assert!(error.contains("over the limit"), "{error}");
    }

    #[test]
    fn a_gzip_page_that_gives_more_than_it_declares_is_refused() {
        use parquet::basic::{Compression, GzipLevel};
        let properties = WriterProperties::builder()
            .set_compression(Compression::GZIP(GzipLevel::default()))
            .set_dictionary_enabled(false)
            .build();
        let file = parquet_file(properties, 1);
        let handle = file.reopen().unwrap();

        // The first page: its type, then its size, which is cut down to almost
        // nothing while taking up the same bytes.
        let metadata = ArrowReaderMetadata::load(&handle, ArrowReaderOptions::new()).unwrap();
        let start = metadata
            .metadata()
            .row_group(0)
            .column(0)
            .data_page_offset() as usize;
        let mut bytes = std::fs::read(file.path()).unwrap();
        assert_eq!(bytes[start], 0x15, "field 1, an integer");
        assert_eq!(bytes[start + 1], 0, "a data page");
        assert_eq!(bytes[start + 2], 0x15, "field 2, an integer");
        let mut end = start + 3;
        while bytes[end] & 0x80 != 0 {
            bytes[end] = 0x80;
            end += 1;
        }
        bytes[end] = 0;
        bytes[start + 3] |= 2; // zigzag of 1
        std::fs::write(file.path(), bytes).unwrap();
        let mut sink = Collect::default();
        let error = read_parquet(file.path(), Budget::unlimited(), &mut sink).unwrap_err();
        assert!(error.contains("more than it declares"), "{error}");
    }

    #[test]
    fn a_damaged_parquet_page_header_is_refused() {
        let file = parquet_file(WriterProperties::builder().build(), 1);
        let handle = file.reopen().unwrap();
        let metadata = ArrowReaderMetadata::load(&handle, ArrowReaderOptions::new()).unwrap();
        let column = metadata.metadata().row_group(0).column(0);
        let start = column
            .dictionary_page_offset()
            .unwrap_or_else(|| column.data_page_offset()) as usize;
        let mut bytes = std::fs::read(file.path()).unwrap();
        bytes[start] = 0x1D; // a type that does not exist
        std::fs::write(file.path(), bytes).unwrap();
        let mut sink = Collect::default();
        let error = read_parquet(file.path(), Budget::unlimited(), &mut sink).unwrap_err();
        assert!(error.starts_with("Refused Parquet file"), "{error}");
    }

    #[test]
    fn a_page_never_reached_costs_nothing() {
        use parquet::basic::{Compression, GzipLevel};
        let properties = WriterProperties::builder()
            .set_compression(Compression::GZIP(GzipLevel::default()))
            .set_dictionary_enabled(false)
            .set_data_page_size_limit(512)
            .set_write_batch_size(64)
            .build();
        let file = parquet_file(properties, 1);
        let handle = file.reopen().unwrap();
        let metadata = ArrowReaderMetadata::load(&handle, ArrowReaderOptions::new()).unwrap();
        let column = metadata.metadata().row_group(0).column(0);

        // Damages the last page of the column, far past the first rows.
        let mut bytes = std::fs::read(file.path()).unwrap();
        let start = column.data_page_offset() as usize;
        let end = start + column.compressed_size() as usize;
        let (mut at, mut last, mut pages) = (start, (0, 0), 0);
        while at < end {
            let mut input = &bytes[at..end];
            let header = parquet_pages::read_header(&mut input).unwrap();
            let length = header.compressed.unwrap() as usize;
            let data = end - input.len();
            last = (data, length);
            at = data + length;
            pages += 1;
        }
        assert!(pages > 3, "{pages}");
        bytes[last.0..last.0 + last.1].fill(0xAA);
        std::fs::write(file.path(), bytes).unwrap();

        let mut sink = Collect {
            stop: Some(3),
            ..Default::default()
        };
        let end = read_parquet(file.path(), Budget::unlimited(), &mut sink).unwrap();
        assert_eq!(end, End::Refused);
        assert_eq!(sink.rows.len(), 3);
    }

    #[test]
    fn empty_row_groups_look_at_the_clock() {
        use parquet::file::metadata::{
            ColumnChunkMetaData, FileMetaData, ParquetMetaData, ParquetMetaDataWriter,
            RowGroupMetaData,
        };
        use parquet::schema::parser::parse_message_type;
        use parquet::schema::types::SchemaDescriptor;

        // A footer with groups that have nothing in them, which no writer makes:
        // nothing to read in the file means nothing else looks at the clock.
        let schema = Arc::new(parse_message_type("message m { required int32 a; }").unwrap());
        let descriptor = Arc::new(SchemaDescriptor::new(schema));
        let group = RowGroupMetaData::builder(descriptor.clone())
            .set_column_metadata(vec![
                ColumnChunkMetaData::builder(descriptor.column(0))
                    .build()
                    .unwrap(),
            ])
            .set_num_rows(0)
            .build()
            .unwrap();
        let info = FileMetaData::new(1, 0, None, None, descriptor, None);
        let metadata = ParquetMetaData::new(info, vec![group; 3]);
        let mut bytes = b"PAR1".to_vec();
        ParquetMetaDataWriter::new(&mut bytes, &metadata)
            .finish()
            .unwrap();
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&bytes).unwrap();

        assert_eq!(
            read_parquet(file.path(), Budget::unlimited(), &mut Collect::default()),
            Ok(End::Complete)
        );
        assert_eq!(
            read_parquet(file.path(), expired(), &mut Collect::default()),
            Ok(End::OutOfTime)
        );
    }

    #[test]
    fn big_dictionaries_mean_few_rows_at_a_time() {
        assert_eq!(parquet_batch(0, 1 << 30), 1024);
        assert_eq!(parquet_batch(1 << 20, 1 << 30), 1024);
        assert_eq!(parquet_batch(100 << 20, 1 << 30), 10);
        assert_eq!(parquet_batch(1 << 30, 1 << 30), 1);
        assert_eq!(parquet_batch(u64::MAX, 1 << 30), 1);
    }

    proptest::proptest! {
        /// Whatever the bytes, a reader gives a result: a hostile file must
        /// not take the program down.
        #[test]
        fn any_bytes_never_panic(bytes in proptest::collection::vec(proptest::num::u8::ANY, 0..2048)) {
            let mut file = NamedTempFile::new().unwrap();
            file.write_all(&bytes).unwrap();
            let path = file.path();
            let budget = Budget::unlimited;
            let _ = read_csv(path, None, budget(), &mut Collect::default());
            let _ = read_jsonl(path, budget(), &mut Collect::default());
            let _ = read_dbf(path, budget(), &mut Collect::default());
            let _ = read_spreadsheet(path, 0, budget(), &mut Collect::default());
            let copy = path.with_extension("xlsx");
            std::fs::copy(path, &copy).unwrap();
            let _ = read_spreadsheet(&copy, 0, budget(), &mut Collect::default());
            let _ = std::fs::remove_file(&copy);
            let _ = read_parquet(path, budget(), &mut Collect::default());
            let _ = read_sqlite(path, SqlitePart::First, budget(), &mut Collect::default());
        }
    }

    #[test]
    fn columns_past_the_limit_are_cut_and_said() {
        let header: Vec<String> = (0..MAX_COLS + 4).map(|i| format!("c{i}")).collect();
        let file = file_with(&format!("{}\n", header.join(",")));
        let mut sink = Collect::default();
        read_csv(file.path(), None, Budget::unlimited(), &mut sink).unwrap();
        assert_eq!(sink.columns.len(), MAX_COLS);
        assert!(sink.cut);
    }
}
