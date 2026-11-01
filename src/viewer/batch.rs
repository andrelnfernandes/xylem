//! Batch (Ctrl+B) over what the viewer shows: a line of output per record of a
//! table or per line of text, from a mask with fields such as `%{name:url}`.
//!
//! The values come from a file that may be hostile and go into something that
//! may run, such as a script. So each field has an escape, chosen by the output
//! file's extension unless the mask names another, and raw output must be asked
//! for by writing it.

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use chrono::{DateTime, Datelike, Local, Utc};
use regex::Regex;

use super::records::{self, Budget, Cell, End, RecordSink};
use super::sql_filter::{self, Expr, RowContext, SqlFilter, Value};
use super::table::TableGrid;
use crate::bidi;
use crate::shell;

/// How many records the preview looks at, also to find late columns.
const SCAN_RECORDS: u64 = 1_000;
/// How many lines of output the preview shows.
const PREVIEW_LINES: usize = 20;
/// How long the preview may take.
const SCAN_TIME: Duration = Duration::from_secs(3);
/// How many lines of a template's header the preview shows, so that a long one
/// does not hide the records.
const PREVIEW_HEADER_LINES: usize = 3;
/// A template is typed by hand; anything larger is not one.
const MAX_TEMPLATE_BYTES: u64 = 1024 * 1024;
pub const TEMPLATE_EXTENSION: &str = "tpl";

/// How a value goes into the output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Escape {
    /// One argument of a POSIX shell command, quoted.
    Shell,
    /// Percent-encoded, for a part of a URL.
    Url,
    /// A JSON value: numbers as numbers, text with its quotes.
    Json,
    /// Text or attribute value of XML or HTML.
    Xml,
    /// A SQL literal, with its quotes.
    Sql,
    /// A SQL literal for MySQL and MariaDB, where a backslash also escapes.
    Mysql,
    /// A CSV field, quoted when needed.
    Csv,
    /// A CSV field that a spreadsheet will not take for a formula.
    CsvSafe,
    /// The value itself, with control characters as spaces, so that it never
    /// starts a new line of output.
    Line,
    /// The value itself.
    Raw,
}

impl Escape {
    fn named(name: &str) -> Option<Self> {
        Some(match name {
            "sh" => Escape::Shell,
            "url" => Escape::Url,
            "json" => Escape::Json,
            "xml" | "html" => Escape::Xml,
            "sql" => Escape::Sql,
            "mysql" => Escape::Mysql,
            "csv" => Escape::Csv,
            "csvsafe" => Escape::CsvSafe,
            "line" => Escape::Line,
            "raw" => Escape::Raw,
            _ => return None,
        })
    }

    /// The escape of a field that does not name one.
    pub fn for_file(file: &Path) -> Self {
        let ext = extension(file);
        match ext.as_str() {
            "sh" | "bash" | "zsh" => Escape::Shell,
            "json" | "jsonl" | "ndjson" => Escape::Json,
            "xml" | "html" | "htm" | "xhtml" | "svg" => Escape::Xml,
            "sql" => Escape::Sql,
            "csv" | "tsv" => Escape::Csv,
            _ => Escape::Line,
        }
    }
}

fn extension(file: &Path) -> String {
    file.extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default()
}

/// `cmd` expands `%VAR%` even inside quotes, and a value from a file could use
/// that. So there is no safe way to put one in these scripts yet.
pub fn refuses_records(file: &Path) -> bool {
    matches!(extension(file).as_str(), "bat" | "cmd")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The value as text, with no escape.
fn plain<'a>(cell: &'a Cell<'_>) -> Cow<'a, str> {
    match cell {
        Cell::Null => Cow::Borrowed(""),
        Cell::Bool(b) => Cow::Owned(b.to_string()),
        Cell::Int(i) => Cow::Owned(i.to_string()),
        Cell::Float(f) => Cow::Owned(f.to_string()),
        Cell::Text(s) => Cow::Borrowed(s),
        Cell::Bytes(b) => Cow::Owned(hex(b)),
    }
}

fn shell_quote(text: &str, out: &mut Vec<u8>) {
    out.push(b'\'');
    // A NUL cannot be in an argument, and sh stops reading a line at it.
    for piece in text.replace('\0', "").split_inclusive('\'') {
        match piece.strip_suffix('\'') {
            Some(head) => {
                out.extend_from_slice(head.as_bytes());
                out.extend_from_slice(b"'\\''");
            }
            None => out.extend_from_slice(piece.as_bytes()),
        }
    }
    out.push(b'\'');
}

fn csv_field(text: &str, out: &mut Vec<u8>) {
    if text.contains([',', ';', '\t', '"', '\n', '\r']) {
        out.push(b'"');
        out.extend_from_slice(text.replace('"', "\"\"").as_bytes());
        out.push(b'"');
    } else {
        out.extend_from_slice(text.as_bytes());
    }
}

fn starts_like_formula(text: &str) -> bool {
    text.starts_with(['=', '+', '-', '@', '\t', '\r'])
}

/// Writes the value of `cell` to `out`.
fn escape_into(cell: &Cell<'_>, escape: Escape, out: &mut Vec<u8>) {
    match escape {
        Escape::Raw => match cell {
            Cell::Bytes(b) => out.extend_from_slice(b),
            other => out.extend_from_slice(plain(other).as_bytes()),
        },
        Escape::Line => {
            let text = plain(cell);
            let clean: String = text
                .chars()
                .map(|c| if c.is_control() { ' ' } else { c })
                .collect();
            out.extend_from_slice(clean.as_bytes());
        }
        Escape::Shell => shell_quote(&plain(cell), out),
        Escape::Url => {
            for b in plain(cell).bytes() {
                if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
                    out.push(b);
                } else {
                    out.extend_from_slice(format!("%{b:02X}").as_bytes());
                }
            }
        }
        Escape::Json => {
            let value = match cell {
                Cell::Null => serde_json::Value::Null,
                Cell::Bool(b) => serde_json::Value::Bool(*b),
                Cell::Int(i) => serde_json::Value::from(*i),
                // Not a number in JSON (NaN, infinity) comes out as null.
                Cell::Float(f) => serde_json::Value::from(*f),
                other => serde_json::Value::String(plain(other).into_owned()),
            };
            out.extend_from_slice(value.to_string().as_bytes());
        }
        Escape::Xml => {
            for c in plain(cell).chars() {
                match c {
                    '&' => out.extend_from_slice(b"&amp;"),
                    '<' => out.extend_from_slice(b"&lt;"),
                    '>' => out.extend_from_slice(b"&gt;"),
                    '"' => out.extend_from_slice(b"&quot;"),
                    '\'' => out.extend_from_slice(b"&#39;"),
                    // XML 1.0 has no way to write these.
                    c if c.is_control() && !matches!(c, '\t' | '\n' | '\r') => {
                        out.extend_from_slice("\u{FFFD}".as_bytes());
                    }
                    c => {
                        let mut buf = [0; 4];
                        out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
                    }
                }
            }
        }
        Escape::Sql | Escape::Mysql => match cell {
            Cell::Null => out.extend_from_slice(b"NULL"),
            Cell::Bool(b) => out.extend_from_slice(if *b { b"TRUE" } else { b"FALSE" }),
            Cell::Int(i) => out.extend_from_slice(i.to_string().as_bytes()),
            Cell::Float(f) if f.is_finite() => out.extend_from_slice(f.to_string().as_bytes()),
            Cell::Float(_) => out.extend_from_slice(b"NULL"),
            Cell::Bytes(b) => out.extend_from_slice(format!("X'{}'", hex(b)).as_bytes()),
            Cell::Text(s) if escape == Escape::Mysql => {
                out.push(b'\'');
                for c in s.chars() {
                    match c {
                        '\\' => out.extend_from_slice(b"\\\\"),
                        '\'' => out.extend_from_slice(b"''"),
                        '\0' => out.extend_from_slice(b"\\0"),
                        // Ends the input of mysql on Windows.
                        '\u{1A}' => out.extend_from_slice(b"\\Z"),
                        c => {
                            let mut buf = [0; 4];
                            out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
                        }
                    }
                }
                out.push(b'\'');
            }
            Cell::Text(s) => {
                out.push(b'\'');
                out.extend_from_slice(s.replace('\0', "").replace('\'', "''").as_bytes());
                out.push(b'\'');
            }
        },
        Escape::Csv => csv_field(&plain(cell), out),
        Escape::CsvSafe => {
            let text = plain(cell);
            if matches!(cell, Cell::Text(_)) && starts_like_formula(&text) {
                csv_field(&format!("'{text}"), out);
            } else {
                csv_field(&text, out);
            }
        }
    }
}

/// What a mask can say about where it is used, fixed when it is read: all the
/// records of a file get the same date.
struct Context<'a> {
    open: &'a Path,
    size: u64,
    out: &'a Path,
    template: Option<&'a str>,
    now: DateTime<Local>,
}

fn text_cell(text: impl Into<String>) -> Cell<'static> {
    Cell::Text(Cow::Owned(text.into()))
}

impl Context<'_> {
    /// A field that every record shares: unlike `%N`, it goes through an escape,
    /// and a file's name can be as hostile as its contents.
    fn field(&self, name: &str) -> Result<Option<Cell<'static>>, String> {
        let now = &self.now;
        let date = |format: &str| Some(text_cell(now.format(format).to_string()));
        let from_file = |var: &str| {
            let value = shell::batch_line(var, self.open, self.size);
            Some(text_cell(String::from_utf8_lossy(&value)))
        };
        let unknown = || format!("Mask: cannot tell %{{{name}}} here");
        Ok(match name {
            "#path" => from_file("%F"),
            "#dir" => from_file("%D"),
            "#name" => from_file("%N"),
            "#base" => from_file("%B"),
            "#ext" => from_file("%E"),
            "#size" => Some(Cell::Int(self.size as i64)),
            "#date" => date("%Y-%m-%d"),
            "#time" => date("%H:%M:%S"),
            "#datetime" => date("%Y-%m-%dT%H:%M:%S%:z"),
            "#utc" => Some(text_cell(
                now.with_timezone(&Utc)
                    .format("%Y-%m-%dT%H:%M:%SZ")
                    .to_string(),
            )),
            "#year" => date("%Y"),
            "#month" => date("%m"),
            "#day" => date("%d"),
            // ISO: Monday is 1.
            "#weekday" => Some(Cell::Int(i64::from(now.weekday().number_from_monday()))),
            // In English, so that a script does not change with the machine's language.
            "#weekday_name" => Some(text_cell(now.format("%A").to_string().to_lowercase())),
            "#epoch" => Some(Cell::Int(now.timestamp())),
            "#user" => {
                let user = std::env::var("USER").or_else(|_| std::env::var("USERNAME"));
                Some(text_cell(user.map_err(|_| unknown())?))
            }
            "#host" => Some(text_cell(
                crate::platform::system().host.ok_or_else(unknown)?,
            )),
            "#template" => Some(text_cell(self.template.ok_or_else(unknown)?)),
            "#output" => Some(text_cell(self.out.to_string_lossy())),
            _ => None,
        })
    }
}

/// A variable of the process, which a mask has to name. It is there as it was
/// when the file was generated; `$NAME` in a script is read when that runs.
fn environment_field(name: &str) -> Result<Cell<'static>, String> {
    if name.is_empty() {
        return Err("Mask: %{$} has no variable name".into());
    }
    let value =
        std::env::var_os(name).ok_or_else(|| format!("Mask: no environment variable {name}"))?;
    Ok(text_cell(value.to_string_lossy()))
}

/// Where a mask's text is in SQL, to know what is inside a string. A line starts
/// outside of one: a multi-line string is rare, and an apostrophe in text that
/// is not SQL should not count for the rest of the file.
#[derive(Default)]
struct SqlText {
    in_string: bool,
    in_comment: bool,
}

impl SqlText {
    fn read(&mut self, text: &str) {
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '\n' => *self = SqlText::default(),
                _ if self.in_comment => {}
                '\'' => self.in_string = !self.in_string,
                '-' if !self.in_string && chars.peek() == Some(&'-') => self.in_comment = true,
                _ => {}
            }
        }
    }
}

/// Writes `bytes`, with each line ending in `newline`.
fn literal_into(bytes: &[u8], newline: &[u8], out: &mut Vec<u8>) {
    for (i, piece) in bytes.split(|&b| b == b'\n').enumerate() {
        if i > 0 {
            out.extend_from_slice(newline);
        }
        out.extend_from_slice(piece);
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Part {
    Literal(Vec<u8>),
    Field {
        name: String,
        escape: Escape,
    },
    /// In a footer: a total of a column.
    Total {
        kind: TotalKind,
        column: String,
        format: NumberFormat,
        /// The one asked for; with none, a plain number needs no escape.
        escape: Option<Escape>,
        default: Escape,
    },
}

/// What a footer can ask about a column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TotalKind {
    Sum,
    Min,
    Max,
    Avg,
}

/// How numbers are written, in the file read and in the total. Only the user can
/// say: `1.234` is one thing in a Brazilian CSV and another in an American one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NumberFormat {
    /// `1234.56`, with no thousands separators.
    Plain,
    /// `1.234,56`
    Br,
    /// `1,234.56`
    Us,
}

impl NumberFormat {
    /// The thousands and decimal separators.
    fn separators(self) -> (Option<char>, char) {
        match self {
            NumberFormat::Plain => (None, '.'),
            NumberFormat::Br => (Some('.'), ','),
            NumberFormat::Us => (Some(','), '.'),
        }
    }
}

/// Digits after the point that a total keeps. Everything is an integer of this
/// scale: money is never added in floating point.
const TOTAL_SCALE: u32 = 18;
const TOTAL_UNIT: i128 = 10_i128.pow(TOTAL_SCALE);

/// A number as `format` writes it, with how many decimals it has.
fn parse_decimal(text: &str, format: NumberFormat) -> Option<(i128, u32)> {
    let text = text.trim();
    let (negative, digits) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    let (thousands, point) = format.separators();
    let (whole, fraction) = digits.split_once(point).unwrap_or((digits, ""));
    if whole.is_empty() && fraction.is_empty() {
        return None;
    }
    let mut number: i128 = 0;
    let mut add_digits = |group: &str| -> Option<()> {
        for b in group.bytes() {
            if !b.is_ascii_digit() {
                return None;
            }
            number = number.checked_mul(10)?.checked_add(i128::from(b - b'0'))?;
        }
        Some(())
    };
    match thousands.filter(|t| whole.contains(*t)) {
        // With separators they have to be right, or this is another format.
        Some(thousands) => {
            let mut groups = whole.split(thousands);
            let first = groups.next()?;
            if first.is_empty() || first.len() > 3 {
                return None;
            }
            add_digits(first)?;
            for group in groups {
                if group.len() != 3 {
                    return None;
                }
                add_digits(group)?;
            }
        }
        None => add_digits(whole)?,
    }
    let scale = u32::try_from(fraction.len()).ok()?;
    if scale > TOTAL_SCALE || !fraction.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let mut scaled = number.checked_mul(TOTAL_UNIT)?;
    let mut unit = TOTAL_UNIT;
    for b in fraction.bytes() {
        unit /= 10;
        scaled = scaled.checked_add(i128::from(b - b'0') * unit)?;
    }
    Some((if negative { -scaled } else { scaled }, scale))
}

/// A number of `TOTAL_SCALE` written with `scale` decimals, rounded half away
/// from zero.
fn write_decimal(value: i128, scale: u32, format: NumberFormat) -> String {
    let unit = 10_u128.pow(TOTAL_SCALE - scale);
    let rounded = (value.unsigned_abs() + unit / 2) / unit;
    let power = 10_u128.pow(scale);
    let whole = (rounded / power).to_string();
    let (thousands, point) = format.separators();
    let mut text = String::new();
    if value < 0 && rounded > 0 {
        text.push('-');
    }
    for (i, c) in whole.chars().enumerate() {
        if i > 0
            && (whole.len() - i).is_multiple_of(3)
            && let Some(thousands) = thousands
        {
            text.push(thousands);
        }
        text.push(c);
    }
    if scale > 0 {
        text.push(point);
        text.push_str(&format!(
            "{:0width$}",
            rounded % power,
            width = scale as usize
        ));
    }
    text
}

/// What was added of a column.
#[derive(Debug, Clone, Default)]
struct Tally {
    sum: i128,
    min: Option<i128>,
    max: Option<i128>,
    count: i128,
    /// The most decimals seen: the total shows as many.
    scale: u32,
}

impl Tally {
    /// Adds a value. An empty one is no value, as in SQL; `false` says this one
    /// could not be added.
    fn add(&mut self, cell: &Cell<'_>, format: NumberFormat) -> bool {
        let found = match cell {
            Cell::Null => return true,
            Cell::Int(i) => Some((i128::from(*i) * TOTAL_UNIT, 0)),
            // As the shortest text that reads back as it, so that 0.1 is 0.1.
            Cell::Float(f) if f.is_finite() => parse_decimal(&f.to_string(), NumberFormat::Plain),
            Cell::Text(text) if text.trim().is_empty() => return true,
            Cell::Text(text) => parse_decimal(text, format),
            _ => None,
        };
        let Some((value, scale)) = found else {
            return false;
        };
        let Some(sum) = self.sum.checked_add(value) else {
            return false;
        };
        self.sum = sum;
        self.min = Some(self.min.map_or(value, |m| m.min(value)));
        self.max = Some(self.max.map_or(value, |m| m.max(value)));
        self.count += 1;
        self.scale = self.scale.max(scale);
        true
    }

    /// The answer, if there is one: a minimum of no values is nothing.
    fn answer(&self, kind: TotalKind, format: NumberFormat) -> Option<String> {
        let (value, scale) = match kind {
            TotalKind::Sum => (self.sum, self.scale),
            TotalKind::Min => (self.min?, self.scale),
            TotalKind::Max => (self.max?, self.scale),
            // Two digits more than the values have, or it would look exact.
            TotalKind::Avg if self.count == 0 => return None,
            TotalKind::Avg => (self.sum / self.count, (self.scale + 2).min(TOTAL_SCALE)),
        };
        Some(write_decimal(value, scale, format))
    }
}

/// The totals a footer asks for, being added up.
#[derive(Debug, Default)]
struct Totals(Vec<(String, NumberFormat, Tally)>);

impl Totals {
    fn for_output(output: &Output) -> Self {
        let mut totals: Vec<(String, NumberFormat, Tally)> = Vec::new();
        for (column, format) in output.total_columns() {
            if !totals.iter().any(|(c, f, _)| c == column && *f == format) {
                totals.push((column.to_string(), format, Tally::default()));
            }
        }
        Totals(totals)
    }

    fn add(&mut self, row: &Row<'_>, stats: &mut Stats) {
        for (column, format, tally) in &mut self.0 {
            if !tally.add(&row.value(column), *format) {
                stats.not_numbers += 1;
            }
        }
    }

    fn get(&self, column: &str, format: NumberFormat) -> Option<&Tally> {
        self.0
            .iter()
            .find(|(c, f, _)| c == column && *f == format)
            .map(|(_, _, tally)| tally)
    }
}

/// `sum:column`, `sum:column:br` or `sum:column:br:sql`, as a footer asks for a
/// total. The name of a column can hold a colon, so its ends are read first.
fn parse_total(field: &str, default: Escape) -> Result<Option<Part>, String> {
    let Some((kind, rest)) = total_kind(field) else {
        return Ok(None);
    };
    let mut column = rest;
    let mut format = NumberFormat::Plain;
    let mut escape = None;
    if let Some((before, last)) = column.rsplit_once(':')
        && let Some(wanted) = Escape::named(last)
    {
        escape = Some(wanted);
        column = before;
    }
    if let Some((before, last)) = column.rsplit_once(':') {
        let named = match last {
            "br" => Some(NumberFormat::Br),
            "us" => Some(NumberFormat::Us),
            _ => None,
        };
        if let Some(named) = named {
            format = named;
            column = before;
        }
    }
    if column.is_empty() {
        return Err(format!("Mask: %{{{field}}} has no column"));
    }
    Ok(Some(Part::Total {
        kind,
        column: column.to_string(),
        format,
        escape,
        default,
    }))
}

fn total_kind(field: &str) -> Option<(TotalKind, &str)> {
    let (word, rest) = field.split_once(':')?;
    let kind = match word {
        "sum" => TotalKind::Sum,
        "min" => TotalKind::Min,
        "max" => TotalKind::Max,
        "avg" => TotalKind::Avg,
        _ => return None,
    };
    Some((kind, rest))
}

/// A parsed line mask.
#[derive(Debug, Clone, PartialEq)]
pub struct Mask {
    parts: Vec<Part>,
    /// The variables of the process it reads.
    environment: Vec<String>,
}

impl Mask {
    /// `%{name}` and `%{name:escape}` are fields of the record; `%f %d %n %b
    /// %e %s` are the open file's, as in the Batch of tagged files; `%%` is a `%`.
    /// `%{$NAME}` is a variable of the process, and `%{#...}` what `Context` knows.
    fn parse_in(text: &str, default: Escape, context: &Context<'_>) -> Result<Self, String> {
        Self::parse_section(text, default, context, false)
    }

    /// `totals`: this is a footer, where `%{sum:column}` and its like are totals.
    fn parse_section(
        text: &str,
        default: Escape,
        context: &Context<'_>,
        totals: bool,
    ) -> Result<Self, String> {
        if text.trim().is_empty() {
            return Err("Mask: empty. Type what to write for each record, or :template".into());
        }
        let mut parts = Vec::new();
        let mut environment = Vec::new();
        let mut literal = Vec::new();
        let mut rest = text;
        let mut sql = SqlText::default();
        // The first variable of the open file inside a SQL string.
        let mut in_string = None;
        while let Some(at) = rest.find('%') {
            literal.extend_from_slice(&rest.as_bytes()[..at]);
            sql.read(&rest[..at]);
            let after = &rest[at + 1..];
            if let Some(inner) = after.strip_prefix('{') {
                let Some(close) = inner.find('}') else {
                    return Err("Mask: %{ without its }".into());
                };
                if totals && let Some(total) = parse_total(&inner[..close], default)? {
                    if !literal.is_empty() {
                        parts.push(Part::Literal(std::mem::take(&mut literal)));
                    }
                    parts.push(total);
                    rest = &inner[close + 1..];
                    continue;
                }
                let (name, escape) = match inner[..close].rsplit_once(':') {
                    Some((name, wanted)) => {
                        let escape = Escape::named(wanted).ok_or_else(|| {
                            if total_kind(&inner[..close]).is_some() {
                                format!(
                                    "Mask: %{{{}}} is only known in the footer",
                                    &inner[..close]
                                )
                            } else {
                                format!("Mask: unknown escape :{wanted}")
                            }
                        })?;
                        (name, escape)
                    }
                    None => (&inner[..close], default),
                };
                if name.is_empty() {
                    return Err("Mask: %{} has no field name".into());
                }
                let shared = match name.strip_prefix('$') {
                    Some(variable) => {
                        let value = environment_field(variable)?;
                        environment.push(variable.to_string());
                        Some(value)
                    }
                    None => context.field(name)?,
                };
                if let Some(value) = shared {
                    escape_into(&value, escape, &mut literal);
                    rest = &inner[close + 1..];
                    continue;
                }
                if !literal.is_empty() {
                    parts.push(Part::Literal(std::mem::take(&mut literal)));
                }
                parts.push(Part::Field {
                    name: name.to_string(),
                    escape,
                });
                rest = &inner[close + 1..];
            } else if let Some(c) = after.chars().next()
                && matches!(
                    c.to_ascii_lowercase(),
                    'f' | 'd' | 'n' | 'b' | 'e' | 's' | '%'
                )
            {
                let var = format!("%{c}");
                if sql.in_string && c != '%' {
                    in_string.get_or_insert_with(|| var.clone());
                }
                literal.extend(shell::batch_line(&var, context.open, context.size));
                rest = &after[c.len_utf8()..];
            } else {
                literal.push(b'%');
                rest = after;
            }
        }
        literal.extend_from_slice(rest.as_bytes());
        if !literal.is_empty() {
            parts.push(Part::Literal(literal));
        }
        let writes_sql = matches!(default, Escape::Sql | Escape::Mysql)
            || parts.iter().any(|part| {
                matches!(
                    part,
                    Part::Field {
                        escape: Escape::Sql | Escape::Mysql,
                        ..
                    }
                )
            });
        // LIKE '%foo%' would hold the file's path, with no escape for SQL.
        if let (true, Some(var)) = (writes_sql, in_string) {
            return Err(format!(
                "Mask: {var} in a SQL string is the open file's: \
                 write %{var} for a %, or a field like %{{#name}} for its value"
            ));
        }
        Ok(Mask { parts, environment })
    }

    /// The first field that `names` has no answer for.
    fn unknown(&self, row: &Row<'_>) -> Option<&str> {
        self.parts.iter().find_map(|part| match part {
            Part::Field { name, .. } if !row.knows(name) => Some(name.as_str()),
            _ => None,
        })
    }

    /// The first field that has no answer outside a record.
    fn record_field(&self, count_too: bool) -> Option<&str> {
        self.parts.iter().find_map(|part| match part {
            Part::Field { name, .. } if count_too || name != TOTAL_COUNT => Some(name.as_str()),
            _ => None,
        })
    }

    /// A header or footer: its text, how many records there were and their totals.
    fn edge(&self, count: u64, totals: &Totals, newline: &[u8], out: &mut Vec<u8>) {
        for part in &self.parts {
            match part {
                Part::Literal(bytes) => literal_into(bytes, newline, out),
                Part::Field { escape, .. } => {
                    escape_into(&Cell::Int(count as i64), *escape, out);
                }
                Part::Total {
                    kind,
                    column,
                    format,
                    escape,
                    default,
                } => {
                    let answer = totals
                        .get(column, *format)
                        .and_then(|tally| tally.answer(*kind, *format));
                    match (answer, escape) {
                        // Only digits, a sign and a point: right in any file as it is.
                        (Some(number), None) if *format == NumberFormat::Plain => {
                            out.extend_from_slice(number.as_bytes());
                        }
                        (answer, escape) => {
                            let cell = answer.map_or(Cell::Null, text_cell);
                            escape_into(&cell, escape.unwrap_or(*default), out);
                        }
                    }
                }
            }
        }
    }

    fn render(&self, row: &Row<'_>, newline: &[u8], stats: &mut Stats, out: &mut Vec<u8>) {
        for part in &self.parts {
            match part {
                Part::Literal(bytes) => literal_into(bytes, newline, out),
                Part::Field { name, escape } => {
                    let cell = row.value(name);
                    stats.look_at(&cell, *escape);
                    escape_into(&cell, *escape, out);
                }
                // Only a footer has them.
                Part::Total { .. } => {}
            }
        }
    }
}

/// Values worth a warning.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    /// With control or text direction characters.
    pub suspicious: u64,
    /// Going into a CSV starting like a spreadsheet formula.
    pub formulas: u64,
    /// Going into a `:sql` literal with a backslash, which in MySQL escapes
    /// the quote that ends it.
    pub backslashes: u64,
    /// Whole records only: columns of JSON Lines found after the preview,
    /// and so not written.
    pub late_columns: u64,
    /// Values a total could not add, and left out of it.
    pub not_numbers: u64,
}

impl Stats {
    fn look_at(&mut self, cell: &Cell<'_>, escape: Escape) {
        let Cell::Text(text) = cell else { return };
        if text
            .chars()
            .any(|c| (c.is_control() && c != '\t') || bidi::is_bidi(c))
        {
            self.suspicious += 1;
        }
        if escape == Escape::Csv && starts_like_formula(text) {
            self.formulas += 1;
        }
        if escape == Escape::Sql && text.contains('\\') {
            self.backslashes += 1;
        }
    }

    /// The warnings, in one line.
    pub fn warning(&self) -> Option<String> {
        let mut parts = Vec::new();
        if self.suspicious > 0 {
            parts.push(format!(
                "{} values with control or text direction characters",
                self.suspicious
            ));
        }
        if self.formulas > 0 {
            parts.push(format!(
                "{} values a spreadsheet may take for formulas (:csvsafe avoids it)",
                self.formulas
            ));
        }
        if self.backslashes > 0 {
            parts.push(format!(
                "{} values with a backslash (for MySQL or MariaDB, use :mysql)",
                self.backslashes
            ));
        }
        if self.late_columns > 0 {
            parts.push(format!(
                "{} columns appeared after the first {SCAN_RECORDS} records and were left out",
                self.late_columns
            ));
        }
        if self.not_numbers > 0 {
            parts.push(format!(
                "{} values could not be added to the totals and were left out \
                 (for 1.234,56 use :br, for 1,234.56 use :us)",
                self.not_numbers
            ));
        }
        (!parts.is_empty()).then(|| parts.join(", "))
    }
}

/// The formats that hold whole records, with no mask.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// One array of objects.
    Json,
    /// An object per line.
    JsonLines,
    Csv,
    Tsv,
    Xml,
    Html,
    /// A database with a table `records`. It is not text, so it has its own
    /// writer, and what a mask could do to it is refused.
    Sqlite,
}

/// `file`'s format can hold whole records, with no mask.
pub fn holds_records(file: &Path) -> bool {
    Format::for_file(file).is_some()
}

impl Format {
    fn for_file(file: &Path) -> Option<Self> {
        Some(match extension(file).as_str() {
            "json" => Format::Json,
            "jsonl" | "ndjson" => Format::JsonLines,
            "csv" => Format::Csv,
            "tsv" => Format::Tsv,
            "xml" => Format::Xml,
            "html" | "htm" => Format::Html,
            "db" | "sqlite" | "sqlite3" => Format::Sqlite,
            _ => return None,
        })
    }

    fn escape(self) -> Escape {
        match self {
            Format::Json | Format::JsonLines => Escape::Json,
            Format::Csv | Format::Tsv => Escape::Csv,
            Format::Xml | Format::Html => Escape::Xml,
            // Only the preview, which shows values as they are: the file binds them.
            Format::Sqlite => Escape::Raw,
        }
    }
}

/// The name in a mask that asks for a template, `:name`. It can only be a name,
/// so a template is always in its folder, and a mask with anything else in it,
/// like `: %{a}` in a script, stays a mask.
fn template_name(text: &str) -> Option<&str> {
    let name = text.trim().strip_prefix(':')?;
    let valid = !name.is_empty()
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'));
    valid.then_some(name)
}

pub fn template_dir() -> Option<PathBuf> {
    Some(crate::platform::config_dir()?.join("batch"))
}

/// The most templates the prompt offers.
const MAX_LISTED_TEMPLATES: usize = 100;

/// Longest line of a template on the screen: longer ones continue below, so
/// that scrolling reaches all of them.
const TEMPLATE_VIEW_CHARS: usize = 200;

/// A template the prompt offers.
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateInfo {
    pub name: String,
    /// What its first comment says. The file wrote this: it informs, and only
    /// reading the template shows what it really does.
    pub about: String,
}

/// The templates there are, for the prompt.
pub fn template_list() -> Vec<TemplateInfo> {
    let Some(dir) = template_dir() else {
        return Vec::new();
    };
    names_in(&dir)
        .into_iter()
        .take(MAX_LISTED_TEMPLATES)
        .map(|name| {
            let about =
                read_template(&name, Some(&dir)).map_or(String::new(), |t| template_about(&t));
            TemplateInfo { name, about }
        })
        .collect()
}

/// The first sentence of the comment a template starts with.
fn template_about(text: &str) -> String {
    let text = text.strip_prefix('\u{FEFF}').unwrap_or(text);
    let comment: Vec<&str> = text
        .lines()
        .map_while(|line| line.strip_prefix("%%#"))
        .map(str::trim)
        .collect();
    let comment = comment.join(" ");
    let first = comment
        .find(". ")
        .map_or(comment.as_str(), |at| &comment[..=at]);
    screen_safe(first.trim())
}

/// What a file wrote, made safe for the screen.
fn screen_safe(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_control() || crate::bidi::is_bidi(c) || c == '\u{2028}' || c == '\u{2029}' {
                '·'
            } else {
                c
            }
        })
        .collect()
}

/// The lines of a template, to be read on the screen. Everything in the file
/// is shown, comments too.
pub fn template_lines(name: &str) -> Result<Vec<String>, String> {
    let text = read_template(name, template_dir().as_deref())?;
    Ok(view_lines(&text))
}

fn view_lines(text: &str) -> Vec<String> {
    let text = text.strip_prefix('\u{FEFF}').unwrap_or(text);
    let mut lines = Vec::new();
    for line in text.lines() {
        let line: Vec<char> = screen_safe(&line.replace('\t', "    ")).chars().collect();
        if line.is_empty() {
            lines.push(String::new());
        }
        lines.extend(
            line.chunks(TEMPLATE_VIEW_CHARS)
                .map(|c| c.iter().collect::<String>()),
        );
    }
    lines
}

fn names_in(dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension()? != TEMPLATE_EXTENSION {
                return None;
            }
            let name = path.file_stem()?.to_str()?.to_string();
            template_name(&format!(":{name}")).is_some().then_some(name)
        })
        .collect();
    names.sort();
    names
}

fn read_template(name: &str, dir: Option<&Path>) -> Result<String, String> {
    let dir = dir.ok_or_else(|| format!("No template {name}: no configuration folder"))?;
    let path = dir.join(format!("{name}.{TEMPLATE_EXTENSION}"));
    let shown = path.display();
    let meta = fs::metadata(&path).map_err(|_| format!("No template {name}: no {shown}"))?;
    if !meta.is_file() {
        return Err(format!("Template {name}: {shown} is not a file"));
    }
    if meta.len() > MAX_TEMPLATE_BYTES {
        return Err(format!("Template {name}: {shown} is too large"));
    }
    fs::read_to_string(&path).map_err(|e| format!("Template {name}: {e}"))
}

/// Cuts a template into its header, record and footer. With no `%%` lines all
/// of it is the record, and a line starting with `%%#` is a comment. Lines are
/// kept with `\n`, and each record ends one, so that records never run together.
fn template_sections(text: &str) -> Result<[Option<String>; 3], String> {
    const RECORD: usize = 1;
    let text = text.strip_prefix('\u{FEFF}').unwrap_or(text);
    let mut sections: [Option<String>; 3] = [None, None, None];
    let mut before = String::new();
    let mut current = None;
    for line in text.lines() {
        if line.starts_with("%%#") {
            continue;
        }
        let word = line
            .trim_end()
            .strip_prefix("%%")
            // These stay in the text: the record's are read with it.
            .filter(|w| !matches!(*w, "else" | "end"));
        if let Some(word) = word.filter(|w| !w.is_empty() && w.chars().all(char::is_alphabetic)) {
            let i = match word {
                "header" => 0,
                "record" => RECORD,
                "footer" => 2,
                "if" => return Err("%%if needs a condition".into()),
                "stop" => return Err("%%stop needs a condition: %%stop if ...".into()),
                _ => return Err(format!("unknown section %%{word}")),
            };
            if sections[i].is_some() {
                return Err(format!("two sections %%{word}"));
            }
            sections[i] = Some(String::new());
            current = Some(i);
            continue;
        }
        let section = match current {
            Some(i) => sections[i].get_or_insert_default(),
            None => &mut before,
        };
        section.push_str(line);
        section.push('\n');
    }
    if current.is_none() {
        sections[RECORD] = Some(before);
    } else if !before.trim().is_empty() {
        return Err("text before the first section".into());
    }
    if sections[RECORD]
        .as_deref()
        .is_none_or(|r| r.trim().is_empty())
    {
        return Err("it has no record".into());
    }
    Ok(sections)
}

/// A line of a record that says what to write, and not what is written.
enum Directive<'a> {
    If(&'a str),
    Else,
    End,
    StopIf(&'a str),
}

fn directive(line: &str) -> Option<Directive<'_>> {
    let rest = line.trim_end().strip_prefix("%%")?;
    match rest {
        "else" => Some(Directive::Else),
        "end" => Some(Directive::End),
        _ => rest
            .strip_prefix("if ")
            .map(Directive::If)
            .or_else(|| rest.strip_prefix("stop if ").map(Directive::StopIf)),
    }
}

/// How many `%%if` may be open at once: the file is someone else's.
const MAX_IF_DEPTH: usize = 16;

/// A piece of what a template writes for a record.
#[derive(Debug, Clone, PartialEq)]
enum Node {
    Text(Mask),
    If {
        condition: Expr,
        /// Where each branch is in `RecordBody::branches`.
        then_branch: usize,
        else_branch: Option<usize>,
        then: Vec<Node>,
        otherwise: Vec<Node>,
    },
    /// The file ends here: what the record wrote so far stays.
    Stop(Expr),
}

impl Node {
    fn render(
        nodes: &[Node],
        row: &Row<'_>,
        newline: &[u8],
        stats: &mut Stats,
        seen: &mut [bool],
        out: &mut Vec<u8>,
    ) -> ControlFlow<()> {
        for node in nodes {
            match node {
                Node::Text(mask) => mask.render(row, newline, stats, out),
                Node::If {
                    condition,
                    then_branch,
                    else_branch,
                    then,
                    otherwise,
                } => {
                    let (branch, nodes) = if row.satisfies(condition) {
                        (Some(*then_branch), then)
                    } else {
                        (*else_branch, otherwise)
                    };
                    if let Some(place) = branch.and_then(|b| seen.get_mut(b)) {
                        *place = true;
                    }
                    Node::render(nodes, row, newline, stats, seen, out)?;
                }
                Node::Stop(condition) => {
                    if row.satisfies(condition) {
                        return ControlFlow::Break(());
                    }
                }
            }
        }
        ControlFlow::Continue(())
    }

    fn unknown(nodes: &[Node], row: &Row<'_>) -> Option<String> {
        let in_condition = |condition: &Expr| {
            let mut names = Vec::new();
            sql_filter::columns_in(condition, &mut names);
            names
                .into_iter()
                .find(|name| !row.knows(name))
                .map(str::to_string)
        };
        nodes.iter().find_map(|node| match node {
            Node::Text(mask) => mask.unknown(row).map(str::to_string),
            Node::If {
                condition,
                then,
                otherwise,
                ..
            } => in_condition(condition)
                .or_else(|| Node::unknown(then, row))
                .or_else(|| Node::unknown(otherwise, row)),
            Node::Stop(condition) => in_condition(condition),
        })
    }

    fn environment(nodes: &[Node], names: &mut Vec<String>) {
        for node in nodes {
            match node {
                Node::Text(mask) => names.extend(mask.environment.iter().cloned()),
                Node::If {
                    then, otherwise, ..
                } => {
                    Node::environment(then, names);
                    Node::environment(otherwise, names);
                }
                Node::Stop(_) => {}
            }
        }
    }
}

/// What a template writes for each record.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordBody {
    nodes: Vec<Node>,
    /// How each branch is called, for saying which ones no record took.
    branches: Vec<String>,
}

/// An `%%if` whose `%%end` has not come yet.
struct OpenIf {
    condition: Expr,
    label: String,
    then_branch: usize,
    else_branch: Option<usize>,
    /// What the condition writes, once `%%else` was read.
    then: Option<Vec<Node>>,
    /// What came before this `%%if`.
    outer: Vec<Node>,
}

/// Reads the record of a template, with its `%%if`, `%%else`, `%%end` and
/// `%%stop if`. A condition is in the language of the SQL filter.
fn parse_record(text: &str, default: Escape, context: &Context<'_>) -> Result<RecordBody, String> {
    fn condition(what: &str, text: &str) -> Result<Expr, String> {
        SqlFilter::parse_with_marks(text).map_err(|e| format!("{what} {text}: {e}"))
    }
    let flush = |pending: &mut String, nodes: &mut Vec<Node>| -> Result<(), String> {
        let text = std::mem::take(pending);
        if text.is_empty() {
            return Ok(());
        }
        // Blank lines are text too, and a mask refuses to be only that.
        let mask = if text.trim().is_empty() {
            Mask {
                parts: vec![Part::Literal(text.into_bytes())],
                environment: Vec::new(),
            }
        } else {
            Mask::parse_in(&text, default, context)?
        };
        nodes.push(Node::Text(mask));
        Ok(())
    };
    let mut nodes = Vec::new();
    let mut pending = String::new();
    let mut open: Vec<OpenIf> = Vec::new();
    let mut branches = Vec::new();
    for line in text.lines() {
        let Some(found) = directive(line) else {
            pending.push_str(line);
            pending.push('\n');
            continue;
        };
        flush(&mut pending, &mut nodes)?;
        match found {
            Directive::If(text) => {
                if open.len() >= MAX_IF_DEPTH {
                    return Err(format!("more than {MAX_IF_DEPTH} %%if inside each other"));
                }
                let label = screen_safe(&format!("%%if {text}"));
                branches.push(label.clone());
                open.push(OpenIf {
                    condition: condition("%%if", text)?,
                    label,
                    then_branch: branches.len() - 1,
                    else_branch: None,
                    then: None,
                    outer: std::mem::take(&mut nodes),
                });
            }
            Directive::Else => {
                let current = open.last_mut().ok_or("%%else without %%if")?;
                if current.then.is_some() {
                    return Err("two %%else in the same %%if".into());
                }
                current.then = Some(std::mem::take(&mut nodes));
                branches.push(format!("%%else of {}", current.label));
                current.else_branch = Some(branches.len() - 1);
            }
            Directive::End => {
                let closed = open.pop().ok_or("%%end without %%if")?;
                let last = std::mem::replace(&mut nodes, closed.outer);
                let (then, otherwise) = match closed.then {
                    Some(then) => (then, last),
                    None => (last, Vec::new()),
                };
                nodes.push(Node::If {
                    condition: closed.condition,
                    then_branch: closed.then_branch,
                    else_branch: closed.else_branch,
                    then,
                    otherwise,
                });
            }
            Directive::StopIf(text) => nodes.push(Node::Stop(condition("%%stop if", text)?)),
        }
    }
    flush(&mut pending, &mut nodes)?;
    if !open.is_empty() {
        return Err("%%if without %%end".into());
    }
    Ok(RecordBody { nodes, branches })
}

/// In a footer, `%{count}` is how many records were written.
const TOTAL_COUNT: &str = "count";

/// Header or footer: no fields of a record. The header is written before any
/// record is read, so only the footer knows how many there are.
fn template_edge(
    section: Option<String>,
    default: Escape,
    context: &Context<'_>,
    is_header: bool,
) -> Result<Option<Mask>, String> {
    let Some(text) = section.filter(|s| !s.trim().is_empty()) else {
        return Ok(None);
    };
    if text.lines().any(|line| directive(line).is_some()) {
        return Err("%%if, %%else, %%end and %%stop work only in the record".into());
    }
    let mask = Mask::parse_section(&text, default, context, !is_header)?;
    match mask.record_field(is_header) {
        Some(TOTAL_COUNT) => Err(format!("%{{{TOTAL_COUNT}}} is only known in the footer")),
        Some(field) => Err(format!(
            "%{{{field}}} is a field of a record, outside the record"
        )),
        None => Ok(Some(mask)),
    }
}

fn parse_template(text: &str, default: Escape, context: &Context<'_>) -> Result<Output, String> {
    let [header, record, footer] = template_sections(text)?;
    Ok(Output::Template {
        header: template_edge(header, default, context, true)?,
        record: parse_record(&record.unwrap_or_default(), default, context)?,
        footer: template_edge(footer, default, context, false)?,
    })
}

const SQLITE_NO_MASK: &str =
    "A SQLite file takes whole records: leave the mask empty, or @a,b for those fields";

/// The table a Batch creates in a SQLite file.
const SQLITE_TABLE: &str = "records";

/// The most columns SQLite takes in a table, by default.
const SQLITE_MAX_COLUMNS: usize = 2000;

/// The names of a table's columns, from names in a hostile file: nothing that
/// cannot be typed, none empty, and none twice, which in SQLite ignores case.
fn sqlite_columns(fields: &[String]) -> Vec<String> {
    let mut columns: Vec<String> = Vec::with_capacity(fields.len());
    for (i, field) in fields.iter().enumerate() {
        let clean: String = key_of(field)
            .chars()
            .filter(|c| !c.is_control() && !bidi::is_bidi(*c))
            .collect();
        let base = if clean.trim().is_empty() {
            format!("column_{}", i + 1)
        } else {
            clean
        };
        let taken =
            |name: &str, columns: &[String]| columns.iter().any(|c| c.eq_ignore_ascii_case(name));
        let mut name = base.clone();
        let mut n = 2;
        while taken(&name, &columns) {
            name = format!("{base}_{n}");
            n += 1;
        }
        columns.push(name);
    }
    columns
}

/// `CREATE TABLE`, with no types: a value stays what it is, and a `007` from a
/// CSV is not made 7 by a column that wants numbers.
fn sqlite_create(fields: &[String]) -> String {
    let columns: Vec<String> = sqlite_columns(fields)
        .iter()
        .map(|c| format!("\"{}\"", c.replace('"', "\"\"")))
        .collect();
    format!("CREATE TABLE {SQLITE_TABLE} ({})", columns.join(", "))
}

/// How a field is named in the output.
fn key_of(name: &str) -> &str {
    if name == "#line" { "line_number" } else { name }
}

/// What a Batch writes for each record.
#[derive(Debug, Clone, PartialEq)]
pub enum Output {
    /// A line from a mask.
    Lines(Mask),
    /// Whole records in a format that has its own rules for them.
    Records {
        format: Format,
        /// The fields to write, if the user named them.
        wanted: Option<Vec<String>>,
        /// The fields being written, known once the source was looked at.
        fields: Vec<String>,
        /// The title of an HTML page: the name of what is read.
        title: String,
    },
    /// Text of the user's, from a file, around the masks of the records.
    Template {
        header: Option<Mask>,
        record: RecordBody,
        footer: Option<Mask>,
    },
}

impl From<Mask> for Output {
    fn from(mask: Mask) -> Self {
        Output::Lines(mask)
    }
}

impl Output {
    /// An empty mask writes all fields, and `@a,b` those fields, in the
    /// formats that hold whole records; anything else is a mask.
    /// `:name` is the template with that name.
    pub fn parse(text: &str, out: &Path, open: &Path, size: u64) -> Result<Self, String> {
        Self::parse_in(text, out, open, size, template_dir().as_deref())
    }

    fn parse_in(
        text: &str,
        out: &Path,
        open: &Path,
        size: u64,
        templates: Option<&Path>,
    ) -> Result<Self, String> {
        let mut context = Context {
            open,
            size,
            out,
            template: None,
            now: Local::now(),
        };
        if let Some(name) = template_name(text) {
            if Format::for_file(out) == Some(Format::Sqlite) {
                return Err(SQLITE_NO_MASK.into());
            }
            let body = read_template(name, templates)?;
            context.template = Some(name);
            return parse_template(&body, Escape::for_file(out), &context)
                .map_err(|e| format!("Template {name}: {e}"));
        }
        let format = Format::for_file(out);
        let trimmed = text.trim();
        if format == Some(Format::Sqlite) && !trimmed.is_empty() && !trimmed.starts_with('@') {
            return Err(SQLITE_NO_MASK.into());
        }
        if let Some(format) = format
            && (trimmed.is_empty() || trimmed.starts_with('@'))
        {
            let wanted = match trimmed.strip_prefix('@') {
                None => None,
                Some(list) => {
                    let fields: Vec<String> = list
                        .split(',')
                        .map(|f| f.trim().to_string())
                        .filter(|f| !f.is_empty())
                        .collect();
                    if fields.is_empty() {
                        return Err("Mask: @ needs the names of the fields".into());
                    }
                    Some(fields)
                }
            };
            let title = open
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            return Ok(Output::Records {
                format,
                wanted,
                fields: Vec::new(),
                title,
            });
        }
        Mask::parse_in(text, Escape::for_file(out), &context).map(Output::Lines)
    }

    /// The variables of the process it reads, without repeating them.
    fn environment(&self) -> Vec<String> {
        let mut names = Vec::new();
        match self {
            Output::Lines(mask) => names.extend(mask.environment.iter().cloned()),
            Output::Template {
                header,
                record,
                footer,
            } => {
                for mask in [header, footer].into_iter().flatten() {
                    names.extend(mask.environment.iter().cloned());
                }
                Node::environment(&record.nodes, &mut names);
            }
            Output::Records { .. } => {}
        }
        names.sort();
        names.dedup();
        names
    }

    /// The columns that the footer totals, each with how its numbers are written.
    fn total_columns(&self) -> Vec<(&str, NumberFormat)> {
        let Output::Template {
            footer: Some(footer),
            ..
        } = self
        else {
            return Vec::new();
        };
        footer
            .parts
            .iter()
            .filter_map(|part| match part {
                Part::Total { column, format, .. } => Some((column.as_str(), *format)),
                _ => None,
            })
            .collect()
    }

    /// The first field the records do not have.
    fn unknown(&self, row: &Row<'_>) -> Option<String> {
        match self {
            Output::Lines(mask) => mask.unknown(row).map(str::to_string),
            Output::Template { record, .. } => Node::unknown(&record.nodes, row).or_else(|| {
                self.total_columns()
                    .into_iter()
                    .find(|(column, _)| !row.knows(column))
                    .map(|(column, _)| column.to_string())
            }),
            Output::Records { wanted, .. } => wanted
                .iter()
                .flatten()
                .find(|name| !row.knows(name))
                .cloned(),
        }
    }

    /// Fixes the fields, from the last record looked at.
    fn settle(&mut self, row_fields: Vec<String>) {
        if let Output::Records { wanted, fields, .. } = self {
            *fields = wanted.clone().unwrap_or(row_fields);
        }
    }

    fn begin(&self, newline: &[u8], out: &mut Vec<u8>) {
        if let Output::Template {
            header: Some(header),
            ..
        } = self
        {
            header.edge(0, &Totals::default(), newline, out);
        }
        let Output::Records {
            format,
            fields,
            title,
            ..
        } = self
        else {
            return;
        };
        let separator = if *format == Format::Tsv { "\t" } else { "," };
        match format {
            Format::Json => out.extend_from_slice(b"[\n"),
            Format::JsonLines => {}
            Format::Csv | Format::Tsv => {
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        out.extend_from_slice(separator.as_bytes());
                    }
                    csv_field(key_of(field), out);
                }
                out.push(b'\n');
            }
            Format::Xml => {
                out.extend_from_slice(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<records>\n");
            }
            Format::Html => {
                out.extend_from_slice(
                    b"<!doctype html>\n<html>\n<head>\n<meta charset=\"utf-8\">\n<title>",
                );
                escape_into(&Cell::Text(title.as_str().into()), Escape::Xml, out);
                out.extend_from_slice(b"</title>\n</head>\n<body>\n<table>\n<thead>\n<tr>");
                for field in fields {
                    out.extend_from_slice(b"<th>");
                    escape_into(&Cell::Text(key_of(field).into()), Escape::Xml, out);
                    out.extend_from_slice(b"</th>");
                }
                out.extend_from_slice(b"</tr>\n</thead>\n<tbody>\n");
            }
            Format::Sqlite => {
                out.extend_from_slice(sqlite_create(fields).as_bytes());
                out.push(b'\n');
            }
        }
    }

    /// What ends the file, after `count` records.
    fn end(&self, count: u64, totals: &Totals, newline: &[u8], out: &mut Vec<u8>) {
        if let Output::Template {
            footer: Some(footer),
            ..
        } = self
        {
            footer.edge(count, totals, newline, out);
        }
        let Output::Records { format, .. } = self else {
            return;
        };
        match format {
            Format::Json => out.extend_from_slice(b"\n]\n"),
            Format::Xml => out.extend_from_slice(b"</records>\n"),
            Format::Html => out.extend_from_slice(b"</tbody>\n</table>\n</body>\n</html>\n"),
            _ => {}
        }
    }

    /// The branches of a template, by what `seen` calls them.
    fn branches(&self) -> &[String] {
        match self {
            Output::Template { record, .. } => &record.branches,
            _ => &[],
        }
    }

    /// A record, with what ends its line. `first` is whether it is the first
    /// one written, since a JSON array has commas only between its objects.
    /// `Break` says that a template asked to stop here, and `seen` marks the
    /// branches taken (an empty one marks nothing).
    fn render(
        &self,
        row: &Row<'_>,
        first: bool,
        newline: &[u8],
        stats: &mut Stats,
        seen: &mut [bool],
        out: &mut Vec<u8>,
    ) -> ControlFlow<()> {
        let Output::Records {
            format,
            wanted,
            fields,
            ..
        } = self
        else {
            match self {
                Output::Lines(mask) => {
                    mask.render(row, newline, stats, out);
                    out.extend_from_slice(newline);
                }
                // Its lines already end in what the file says.
                Output::Template { record, .. } => {
                    return Node::render(&record.nodes, row, newline, stats, seen, out);
                }
                Output::Records { .. } => {}
            }
            return ControlFlow::Continue(());
        };
        if wanted.is_none() && row.positional && row.names.len() > fields.len() {
            stats.late_columns = stats
                .late_columns
                .max((row.names.len() - fields.len()) as u64);
        }
        let escape = format.escape();
        let mut value = |name: &str, out: &mut Vec<u8>| {
            let cell = row.value(name);
            stats.look_at(&cell, escape);
            escape_into(&cell, escape, out);
        };
        match format {
            Format::Json | Format::JsonLines => {
                if *format == Format::Json {
                    out.extend_from_slice(if first { b"  " } else { b",\n  " });
                }
                out.push(b'{');
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        out.extend_from_slice(b", ");
                    }
                    let key = serde_json::Value::from(key_of(field)).to_string();
                    out.extend_from_slice(key.as_bytes());
                    out.extend_from_slice(b": ");
                    value(field, out);
                }
                out.push(b'}');
                if *format == Format::JsonLines {
                    out.push(b'\n');
                }
            }
            Format::Sqlite => {
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        out.extend_from_slice(b" | ");
                    }
                    value(field, out);
                }
                out.push(b'\n');
            }
            Format::Csv | Format::Tsv => {
                let separator = if *format == Format::Tsv { b'\t' } else { b',' };
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        out.push(separator);
                    }
                    value(field, out);
                }
                out.push(b'\n');
            }
            Format::Xml => {
                out.extend_from_slice(b"  <record>\n");
                for field in fields {
                    out.extend_from_slice(b"    <field name=\"");
                    escape_into(&Cell::Text(key_of(field).into()), Escape::Xml, out);
                    out.extend_from_slice(b"\">");
                    value(field, out);
                    out.extend_from_slice(b"</field>\n");
                }
                out.extend_from_slice(b"  </record>\n");
            }
            Format::Html => {
                out.extend_from_slice(b"<tr>");
                for field in fields {
                    out.extend_from_slice(b"<td>");
                    value(field, out);
                    out.extend_from_slice(b"</td>");
                }
                out.extend_from_slice(b"</tr>\n");
            }
        }
        ControlFlow::Continue(())
    }
}

/// A record, as a mask sees it.
struct Row<'a> {
    names: &'a [String],
    cells: &'a [Cell<'a>],
    /// 1, 2, 3... among the records that passed the filter.
    sequence: u64,
    /// The line's number in the file (text only).
    line: Option<u64>,
    /// Columns can be reached by letter and position (tables only).
    positional: bool,
}

impl Row<'_> {
    /// All the fields of a record, for a format that writes it whole. A group
    /// with a name goes by the name; 0 repeats what the line already has.
    fn all_fields(&self) -> Vec<String> {
        if self.line.is_none() {
            return self.names.to_vec();
        }
        let mut fields = vec!["#line".to_string(), "line".to_string()];
        for name in self.names.iter().skip(1) {
            if name == "0" {
                continue;
            }
            // The number comes right before the name of the same group.
            let named = name.parse::<usize>().is_ok()
                && self
                    .names
                    .iter()
                    .position(|n| n == name)
                    .and_then(|i| self.names.get(i + 1))
                    .is_some_and(|next| next.parse::<usize>().is_err());
            if !named {
                fields.push(name.clone());
            }
        }
        fields
    }

    fn index(&self, name: &str) -> Option<usize> {
        if let Some(i) = self.names.iter().position(|n| n.eq_ignore_ascii_case(name)) {
            return Some(i);
        }
        if !self.positional {
            return None;
        }
        if let Ok(n) = name.parse::<usize>() {
            return (n >= 1 && n <= self.names.len()).then(|| n - 1);
        }
        (0..self.names.len()).find(|&i| records::column_letter(i).eq_ignore_ascii_case(name))
    }

    /// Whether a condition holds, with the rules of the filter on the screen.
    fn satisfies(&self, condition: &Expr) -> bool {
        sql_filter::eval_expr(condition, self).is_ok_and(|v| v.is_truthy())
    }

    fn knows(&self, name: &str) -> bool {
        name == "#" || (name == "#line" && self.line.is_some()) || self.index(name).is_some()
    }

    fn value(&self, name: &str) -> Cell<'_> {
        match name {
            "#" => Cell::Int(self.sequence as i64),
            "#line" if self.line.is_some() => Cell::Int(self.line.unwrap_or(0) as i64),
            _ => self
                .index(name)
                .and_then(|i| self.cells.get(i))
                .cloned()
                .unwrap_or(Cell::Null),
        }
    }
}

/// What a Batch reads: everything is read again, or kept, away from the screen.
pub enum Source {
    Table {
        path: PathBuf,
        /// The sheet of a spreadsheet.
        sheet: usize,
        /// The table of a database.
        table: Option<String>,
        /// The query typed over a database, which comes before the table.
        query: Option<String>,
        filter: Option<Expr>,
        /// The most a spreadsheet may unpack to.
        unpacked: u64,
        /// When not empty, only the records at these places, as long as the
        /// filter keeps them.
        marked: BTreeSet<usize>,
    },
    Text {
        /// The file decoded, with its tabs and control characters.
        text: String,
        /// How many lines the viewer has of it.
        total: usize,
        /// The lines the filter kept.
        kept: Option<Vec<usize>>,
        /// The filter's regular expression, for its groups.
        regex: Option<Regex>,
        /// The viewer did not read all of the file.
        cut: bool,
    },
}

/// Applies a table's filter to a stream, with the rules of the screen: a value
/// is judged by what the grid would show, so both keep the same rows.
struct TableSink<'a, F> {
    names: Vec<String>,
    shown_names: Vec<String>,
    filter: Option<&'a Expr>,
    marked: &'a BTreeSet<usize>,
    /// Where the next record is in the file.
    place: usize,
    sequence: u64,
    deliver: F,
}

struct CellContext<'a> {
    names: &'a [String],
    cells: &'a [Cell<'a>],
}

impl RowContext for Row<'_> {
    fn get_value(&self, column: &str) -> Option<Value> {
        self.knows(column)
            .then(|| Value::infer(&self.value(column).display()))
    }
}

impl RowContext for CellContext<'_> {
    fn get_value(&self, column: &str) -> Option<Value> {
        let by_name = self
            .names
            .iter()
            .position(|n| n.eq_ignore_ascii_case(column));
        let index = by_name.or_else(|| match column.as_bytes() {
            [b] if b.is_ascii_alphabetic() => Some((b.to_ascii_uppercase() - b'A') as usize),
            _ => None,
        })?;
        self.cells.get(index).map(|c| Value::infer(&c.display()))
    }
}

impl<F: FnMut(&Row<'_>) -> ControlFlow<()>> RecordSink for TableSink<'_, F> {
    fn sheets(&mut self, _: &[String], _: usize) {}

    fn columns(&mut self, names: &[String]) {
        self.names = names.to_vec();
        self.shown_names = names.iter().map(|n| TableGrid::sanitize_cell(n)).collect();
    }

    fn columns_cut(&mut self) {}

    fn record(&mut self, cells: &[Cell<'_>]) -> ControlFlow<()> {
        let place = self.place;
        self.place += 1;
        if !self.marked.is_empty() && !self.marked.contains(&place) {
            return ControlFlow::Continue(());
        }
        if let Some(filter) = self.filter {
            let ctx = CellContext {
                names: &self.shown_names,
                cells,
            };
            let keep = sql_filter::eval_expr(filter, &ctx).is_ok_and(|v| v.is_truthy());
            if !keep {
                return ControlFlow::Continue(());
            }
        }
        self.sequence += 1;
        (self.deliver)(&Row {
            names: &self.names,
            cells,
            sequence: self.sequence,
            line: None,
            positional: true,
        })
    }
}

impl Source {
    /// The names a text record has: the whole line and the filter's groups.
    fn text_names(regex: Option<&Regex>) -> Vec<String> {
        let mut names = vec!["line".to_string()];
        if let Some(regex) = regex {
            for (i, name) in regex.capture_names().enumerate() {
                names.push(i.to_string());
                if let Some(name) = name {
                    names.push(name.to_string());
                }
            }
        }
        names
    }

    /// What can be asked of this source, for the prompt.
    pub fn field_hint(&self, columns: &[String]) -> String {
        match self {
            Source::Table { .. } => {
                let shown: Vec<&str> = columns.iter().take(6).map(String::as_str).collect();
                let more = if columns.len() > shown.len() {
                    ", ..."
                } else {
                    ""
                };
                format!("{}{more}, #", shown.join(", "))
            }
            Source::Text { regex, .. } => {
                let names = Self::text_names(regex.as_ref());
                format!("{}, #, #line", names.join(", "))
            }
        }
    }

    /// A mask that works on this source, for the prompt to show.
    pub fn example(&self, columns: &[String]) -> String {
        match self {
            Source::Table { .. } => {
                let simple: Vec<String> = columns
                    .iter()
                    .filter(|c| c.chars().all(|c| c.is_alphanumeric() || c == '_'))
                    .filter(|c| !c.is_empty())
                    .take(2)
                    .map(|c| format!("%{{{c}}}"))
                    .collect();
                if simple.is_empty() {
                    "%{a}".to_string()
                } else {
                    simple.join(" ")
                }
            }
            Source::Text { regex, .. } => {
                let group = regex.as_ref().and_then(|regex| {
                    let named = regex.capture_names().flatten().next();
                    match named {
                        Some(name) => Some(name.to_string()),
                        None => (regex.captures_len() > 1).then(|| "1".to_string()),
                    }
                });
                format!("%{{#line}}: %{{{}}}", group.as_deref().unwrap_or("line"))
            }
        }
    }

    /// Reads the source, stopping when `deliver` says so or `budget` runs out.
    fn run(
        &self,
        budget: Budget,
        deliver: &mut dyn FnMut(&Row<'_>) -> ControlFlow<()>,
    ) -> Result<End, String> {
        match self {
            Source::Table {
                path,
                sheet,
                table,
                query,
                filter,
                marked,
                ..
            } => {
                let mut sink = TableSink {
                    names: Vec::new(),
                    shown_names: Vec::new(),
                    filter: filter.as_ref(),
                    marked,
                    place: 0,
                    sequence: 0,
                    deliver,
                };
                let part = match (query, table) {
                    (Some(query), _) => records::SqlitePart::Query(query),
                    (None, Some(table)) => records::SqlitePart::Table(table),
                    (None, None) => records::SqlitePart::First,
                };
                records::read_table(path, *sheet, part, budget, &mut sink)
            }
            Source::Text {
                text,
                total,
                kept,
                regex,
                ..
            } => {
                let names = Self::text_names(regex.as_ref());
                // Split as the viewer splits, so that a line's number is the same.
                let body = text.strip_suffix('\n').unwrap_or(text);
                let mut wanted = kept.as_ref().map(|k| k.iter().copied().peekable());
                let mut sequence = 0;
                for (i, line) in body.split('\n').take(*total).enumerate() {
                    if let Some(w) = &mut wanted {
                        if w.peek() != Some(&i) {
                            continue;
                        }
                        w.next();
                    }
                    if sequence & 0xFF == 0 && budget.over() {
                        return Ok(End::OutOfTime);
                    }
                    let line = line.strip_suffix('\r').unwrap_or(line);
                    let mut cells = vec![Cell::Text(Cow::Borrowed(line))];
                    if let Some(regex) = regex {
                        let captures = regex.captures(line);
                        for (g, name) in regex.capture_names().enumerate() {
                            let value = captures
                                .as_ref()
                                .and_then(|c| c.get(g))
                                .map_or(Cell::Null, |m| Cell::Text(Cow::Borrowed(m.as_str())));
                            if name.is_some() {
                                cells.push(value.clone());
                            }
                            cells.push(value);
                        }
                    }
                    sequence += 1;
                    let row = Row {
                        names: &names,
                        cells: &cells,
                        sequence,
                        line: Some(i as u64 + 1),
                        positional: false,
                    };
                    if deliver(&row).is_break() {
                        return Ok(End::Refused);
                    }
                }
                Ok(End::Complete)
            }
        }
    }

    /// Part of the file will not be written.
    fn cut(&self) -> bool {
        matches!(self, Source::Text { cut: true, .. })
    }

    fn unpacked(&self) -> u64 {
        match self {
            Source::Table { unpacked, .. } => *unpacked,
            Source::Text { .. } => 0,
        }
    }
}

/// What the preview found.
#[derive(Debug, Clone, PartialEq)]
pub struct Preview {
    /// The first lines of output.
    pub lines: Vec<String>,
    /// How many records it looked at.
    pub scanned: u64,
    /// Those are all of them.
    pub complete: bool,
    /// The viewer read only part of the file, so the rest will not be written.
    pub cut: bool,
    pub stats: Stats,
    /// The variables of the process it reads: a template from someone else
    /// could send a secret away when its script runs.
    pub environment: Vec<String>,
    /// The record at which a template's `%%stop if` ended it.
    pub stopped: Option<u64>,
    /// The branches of a template that no record looked at took.
    pub unseen: Vec<String>,
    /// What to write, with its fields known.
    pub output: Output,
}

/// A line of output, safe for the screen: it may hold anything.
fn shown(line: &[u8]) -> String {
    TableGrid::sanitize_cell(&String::from_utf8_lossy(line))
}

/// Leaves only the last `count` lines of `text`, all ending in a newline.
fn keep_last_lines(text: &mut Vec<u8>, count: usize) {
    let ends = text.iter().enumerate().filter(|&(_, &b)| b == b'\n');
    let ends: Vec<usize> = ends.map(|(i, _)| i).collect();
    if ends.len() > count {
        text.drain(..=ends[ends.len() - count - 1]);
    }
}

/// Reads the start of `source` through `output`. A field that no record among
/// those read has is an error here, so that it never comes out empty.
pub fn preview(source: &Source, output: impl Into<Output>) -> Result<Preview, String> {
    let mut output = output.into();
    // A first look finds the fields, since JSON Lines finds them as it goes.
    let mut scanned = 0;
    let mut unknown = None;
    let mut fields = Vec::new();
    let end = source.run(Budget::new(SCAN_TIME, source.unpacked()), &mut |row| {
        if scanned >= SCAN_RECORDS {
            return ControlFlow::Break(());
        }
        scanned += 1;
        // The last answer counts, for the same reason.
        unknown = output.unknown(row);
        fields = row.all_fields();
        ControlFlow::Continue(())
    })?;
    if let Some(name) = unknown {
        return Err(format!("Mask: no field named {name}"));
    }
    output.settle(fields);
    if let Output::Records {
        format: Format::Sqlite,
        fields,
        ..
    } = &output
        && fields.len() > SQLITE_MAX_COLUMNS
    {
        return Err(format!(
            "A SQLite table takes {SQLITE_MAX_COLUMNS} columns: use @a,b for those to write"
        ));
    }

    // Then what those records become.
    let mut stats = Stats::default();
    let mut text = Vec::new();
    let mut count = 0;
    let mut stopped = None;
    let mut seen = vec![false; output.branches().len()];
    let mut totals = Totals::for_output(&output);
    output.begin(b"\n", &mut text);
    if matches!(output, Output::Template { .. }) {
        keep_last_lines(&mut text, PREVIEW_HEADER_LINES);
    }
    source.run(Budget::new(SCAN_TIME, source.unpacked()), &mut |row| {
        if count >= scanned {
            return ControlFlow::Break(());
        }
        let first = count == 0;
        count += 1;
        // Only what the screen can show is kept; the rest is only looked at.
        let shown_enough = text.iter().filter(|&&b| b == b'\n').count() > PREVIEW_LINES;
        let mut line = Vec::new();
        let flow = output.render(row, first, b"\n", &mut stats, &mut seen, &mut line);
        if !line.is_empty() {
            totals.add(row, &mut stats);
        }
        if !shown_enough {
            text.extend_from_slice(&line);
        }
        if flow.is_break() {
            stopped = Some(count);
        }
        flow
    })?;
    let mut lines: Vec<String> = text
        .split(|&b| b == b'\n')
        .map(shown)
        .take(PREVIEW_LINES)
        .collect();
    if lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    let unseen = seen
        .iter()
        .zip(output.branches())
        .filter(|(seen, _)| !**seen)
        .map(|(_, label)| label.clone())
        .collect();
    Ok(Preview {
        lines,
        scanned,
        complete: end == End::Complete || stopped.is_some(),
        cut: source.cut(),
        stats,
        environment: output.environment(),
        stopped,
        unseen,
        output,
    })
}

/// How a Batch that ran ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Done {
    pub records: u64,
    pub stats: Stats,
    pub cancelled: bool,
    /// The viewer read only part of the file, so the rest was not written.
    pub cut: bool,
    /// A template's `%%stop if` ended it.
    pub stopped: bool,
}

/// A Batch being written, in a thread of its own: a read can take long, and Esc
/// has to work meanwhile.
pub struct Job {
    pub file: PathBuf,
    cancel: Arc<AtomicBool>,
    written: Arc<AtomicU64>,
    handle: JoinHandle<Result<Done, String>>,
}

impl Job {
    pub fn start(source: Source, output: impl Into<Output>, file: PathBuf) -> Self {
        let output = output.into();
        let cancel = Arc::new(AtomicBool::new(false));
        let written = Arc::new(AtomicU64::new(0));
        let handle = {
            let (cancel, written, file) = (cancel.clone(), written.clone(), file.clone());
            thread::spawn(move || {
                let result = crate::panic_guard::catch(|| {
                    write_file(&source, &output, &file, cancel, &written)
                })
                .unwrap_or_else(|| Err("the file could not be read".into()));
                // Nothing incomplete stays behind, unless it was never ours.
                let keep = matches!(
                    result,
                    Ok(Done {
                        cancelled: false,
                        ..
                    })
                );
                if !keep && !matches!(&result, Err(e) if e.starts_with(NOT_CREATED)) {
                    let _ = fs::remove_file(&file);
                }
                result
            })
        };
        Job {
            file,
            cancel,
            written,
            handle,
        }
    }

    /// Asks it to stop; it ends at the next record.
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelling(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    pub fn written(&self) -> u64 {
        self.written.load(Ordering::Relaxed)
    }

    pub fn is_done(&self) -> bool {
        self.handle.is_finished()
    }

    pub fn finish(self) -> Result<Done, String> {
        self.handle
            .join()
            .unwrap_or_else(|_| Err("the file could not be read".into()))
    }
}

const NOT_CREATED: &str = "Cannot create ";

/// Writes the records into a new SQLite file, always through bound values: what
/// a record holds never becomes SQL. It is all one transaction in a new file, so
/// there is no journal to leave next to it: a Batch that fails removes the file.
fn write_sqlite(
    source: &Source,
    fields: &[String],
    all_fields: bool,
    file: &Path,
    cancel: Arc<AtomicBool>,
    written: &AtomicU64,
) -> Result<Done, String> {
    use rusqlite::types::{ToSqlOutput, ValueRef};
    // Never an existing file, nor one a link points to: SQLite alone would open it.
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(file)
        .map_err(|e| format!("{NOT_CREATED}{}: {e}", file.display()))?;
    let failed = |e: rusqlite::Error| format!("Cannot write {}: {e}", file.display());
    let connection = rusqlite::Connection::open(file).map_err(failed)?;
    connection
        .execute_batch("PRAGMA journal_mode = MEMORY; PRAGMA synchronous = OFF; BEGIN;")
        .map_err(failed)?;
    connection
        .execute_batch(&sqlite_create(fields))
        .map_err(failed)?;
    let places = vec!["?"; fields.len()].join(", ");
    let mut insert = connection
        .prepare(&format!("INSERT INTO {SQLITE_TABLE} VALUES ({places})"))
        .map_err(failed)?;

    let budget = Budget::cancellable(source.unpacked(), cancel);
    let mut stats = Stats::default();
    let mut records = 0;
    let mut failure = None;
    let end = source.run(budget, &mut |row| {
        if all_fields && row.positional && row.names.len() > fields.len() {
            stats.late_columns = stats
                .late_columns
                .max((row.names.len() - fields.len()) as u64);
        }
        let cells: Vec<Cell<'_>> = fields.iter().map(|field| row.value(field)).collect();
        for cell in &cells {
            stats.look_at(cell, Escape::Raw);
        }
        let values = cells.iter().map(|cell| {
            ToSqlOutput::Borrowed(match cell {
                Cell::Null => ValueRef::Null,
                Cell::Bool(b) => ValueRef::Integer(i64::from(*b)),
                Cell::Int(i) => ValueRef::Integer(*i),
                Cell::Float(f) => ValueRef::Real(*f),
                Cell::Text(text) => ValueRef::Text(text.as_bytes()),
                Cell::Bytes(bytes) => ValueRef::Blob(bytes),
            })
        });
        if let Err(e) = insert.execute(rusqlite::params_from_iter(values)) {
            failure = Some(e);
            return ControlFlow::Break(());
        }
        records += 1;
        written.store(records, Ordering::Relaxed);
        ControlFlow::Continue(())
    })?;
    if let Some(e) = failure {
        return Err(failed(e));
    }
    drop(insert);
    if end != End::OutOfTime {
        connection.execute_batch("COMMIT").map_err(failed)?;
    }
    connection.close().map_err(|(_, e)| failed(e))?;
    Ok(Done {
        records,
        stats,
        cancelled: end == End::OutOfTime,
        cut: source.cut(),
        stopped: false,
    })
}

fn write_file(
    source: &Source,
    output: &Output,
    file: &Path,
    cancel: Arc<AtomicBool>,
    written: &AtomicU64,
) -> Result<Done, String> {
    if let Output::Records {
        format: Format::Sqlite,
        fields,
        wanted,
        ..
    } = output
    {
        return write_sqlite(source, fields, wanted.is_none(), file, cancel, written);
    }
    let created = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(file)
        .map_err(|e| format!("{NOT_CREATED}{}: {e}", file.display()))?;
    let mut out = BufWriter::new(created);
    let newline: &[u8] = if extension(file) == "sh" {
        b"\n"
    } else {
        shell::NEWLINE
    };
    if extension(file) == "sh" {
        out.write_all(b"#!/bin/sh\n").map_err(|e| e.to_string())?;
    }
    let mut line = Vec::new();
    output.begin(newline, &mut line);
    out.write_all(&line)
        .map_err(|e| format!("Cannot write {}: {e}", file.display()))?;

    let budget = Budget::cancellable(source.unpacked(), cancel);
    let mut stats = Stats::default();
    let mut records = 0;
    let mut failure = None;
    let mut stopped = false;
    let mut totals = Totals::for_output(output);
    let end = source.run(budget, &mut |row| {
        line.clear();
        let flow = output.render(row, records == 0, newline, &mut stats, &mut [], &mut line);
        if let Err(e) = out.write_all(&line) {
            failure = Some(e);
            return ControlFlow::Break(());
        }
        // A record that an `%%if` left with nothing to say was not written.
        if !line.is_empty() {
            records += 1;
            written.store(records, Ordering::Relaxed);
            totals.add(row, &mut stats);
        }
        stopped = flow.is_break();
        flow
    })?;
    if let Some(e) = failure {
        return Err(format!("Cannot write {}: {e}", file.display()));
    }
    line.clear();
    output.end(records, &totals, newline, &mut line);
    out.write_all(&line)
        .map_err(|e| format!("Cannot write {}: {e}", file.display()))?;
    let flushed: Result<File, _> = out.into_inner();
    flushed.map_err(|e| format!("Cannot write {}: {e}", file.display()))?;
    Ok(Done {
        records,
        stats,
        cancelled: end == End::OutOfTime,
        cut: source.cut(),
        stopped,
    })
}

#[cfg(test)]
#[path = "batch_demo.rs"]
mod demo;

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    impl Mask {
        fn parse(text: &str, default: Escape, file: &Path, size: u64) -> Result<Self, String> {
            Self::parse_in(text, default, &test_context(file, size))
        }
    }

    fn test_context(open: &Path, size: u64) -> Context<'_> {
        Context {
            open,
            size,
            out: Path::new("/o/out.txt"),
            template: None,
            now: Local::now(),
        }
    }

    fn mask(text: &str, default: Escape) -> Mask {
        Mask::parse(text, default, Path::new("/d/a.csv"), 10).unwrap()
    }

    fn escaped(cell: Cell<'_>, escape: Escape) -> String {
        let mut out = Vec::new();
        escape_into(&cell, escape, &mut out);
        String::from_utf8(out).unwrap()
    }

    fn t(s: &str) -> Cell<'_> {
        Cell::Text(s.into())
    }

    /// Runs a command line through the real shell and returns what it prints.
    #[cfg(unix)]
    fn through_sh(line: &[u8]) -> Vec<u8> {
        use std::os::unix::ffi::OsStrExt;
        let out = std::process::Command::new("sh")
            .arg("-c")
            .arg(std::ffi::OsStr::from_bytes(line))
            .env_remove("HOSTILE")
            .output()
            .unwrap();
        out.stdout
    }

    #[cfg(unix)]
    #[test]
    fn a_hostile_value_is_one_argument_in_a_real_shell() {
        let hostile = [
            "$(touch /nonexistent/pwned)",
            "`id`",
            "a'b",
            "'; echo injected; '",
            "line\nbreak",
            "* ? ~",
            "-n",
            "",
        ];
        for value in hostile {
            let mut line = b"printf '%s' ".to_vec();
            shell_quote(value, &mut line);
            assert_eq!(through_sh(&line), value.as_bytes(), "{value:?}");
        }
    }

    #[test]
    fn each_escape_writes_its_own_syntax() {
        assert_eq!(escaped(t("a b/é&"), Escape::Url), "a%20b%2F%C3%A9%26");
        assert_eq!(escaped(t("a-b_c.d~"), Escape::Url), "a-b_c.d~");
        assert_eq!(escaped(t("a\"b\n"), Escape::Json), "\"a\\\"b\\n\"");
        assert_eq!(escaped(t("007"), Escape::Json), "\"007\"");
        assert_eq!(escaped(Cell::Int(7), Escape::Json), "7");
        assert_eq!(escaped(Cell::Float(f64::NAN), Escape::Json), "null");
        assert_eq!(escaped(Cell::Null, Escape::Json), "null");
        assert_eq!(escaped(Cell::Bool(true), Escape::Json), "true");
        assert_eq!(
            escaped(t("<a href=\"x\">&'\u{1}"), Escape::Xml),
            "&lt;a href=&quot;x&quot;&gt;&amp;&#39;\u{FFFD}"
        );
        assert_eq!(escaped(t("O'Neil"), Escape::Sql), "'O''Neil'");
        assert_eq!(escaped(Cell::Null, Escape::Sql), "NULL");
        assert_eq!(escaped(Cell::Bytes(&[0, 255]), Escape::Sql), "X'00ff'");
        assert_eq!(escaped(t("a,\"b"), Escape::Csv), "\"a,\"\"b\"");
        assert_eq!(escaped(t("=1+1"), Escape::Csv), "=1+1");
        assert_eq!(escaped(t("=1+1"), Escape::CsvSafe), "'=1+1");
        assert_eq!(escaped(Cell::Int(-5), Escape::CsvSafe), "-5");
        assert_eq!(escaped(t("a\nb\tc\u{1b}"), Escape::Line), "a b c ");
        assert_eq!(escaped(t("a\nb"), Escape::Raw), "a\nb");
        assert_eq!(escaped(Cell::Bytes(&[0xAB]), Escape::Line), "ab");
    }

    /// What a MySQL literal at the start of `text` holds and what comes after
    /// it, read with the rules of MySQL's default mode.
    fn read_mysql_literal(text: &str) -> Option<(String, &str)> {
        let mut chars = text.strip_prefix('\'')?.char_indices();
        let body = &text[1..];
        let mut value = String::new();
        while let Some((i, c)) = chars.next() {
            match c {
                '\\' => match chars.next()?.1 {
                    '0' => value.push('\0'),
                    'Z' => value.push('\u{1A}'),
                    other => value.push(other),
                },
                '\'' if body[i + 1..].starts_with('\'') => {
                    chars.next();
                    value.push('\'');
                }
                '\'' => return Some((value, &body[i + 1..])),
                c => value.push(c),
            }
        }
        None
    }

    #[test]
    fn a_mysql_literal_cannot_be_closed_by_a_backslash() {
        let hostile = r"\'; DROP TABLE t; -- ";
        let mysql = escaped(t(hostile), Escape::Mysql);
        assert_eq!(mysql, r"'\\''; DROP TABLE t; -- '");
        assert_eq!(read_mysql_literal(&mysql), Some((hostile.to_string(), "")));
        // The problem this avoids: read by MySQL, :sql ends the literal early.
        let sql = escaped(t(hostile), Escape::Sql);
        let (_, after) = read_mysql_literal(&sql).unwrap();
        assert!(after.contains("DROP TABLE"), "{after}");

        assert_eq!(escaped(t("a\0\u{1A}"), Escape::Mysql), r"'a\0\Z'");
        assert_eq!(escaped(Cell::Null, Escape::Mysql), "NULL");
        assert_eq!(escaped(Cell::Int(7), Escape::Mysql), "7");
        assert_eq!(escaped(Cell::Bytes(&[0, 255]), Escape::Mysql), "X'00ff'");
        assert_eq!(
            mask("%{a:mysql}", Escape::Line).parts,
            [Part::Field {
                name: "a".into(),
                escape: Escape::Mysql
            }]
        );
    }

    #[test]
    fn a_backslash_in_a_sql_literal_gets_a_warning() {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("in.csv");
        fs::write(&path, "v\na\\b\nc\n").unwrap();
        let found = |m: &str| {
            preview(&table_source(&path, None), mask(m, Escape::Line))
                .unwrap()
                .stats
        };
        let stats = found("%{v:sql}");
        assert_eq!(stats.backslashes, 1);
        assert!(stats.warning().unwrap().contains(":mysql"));
        assert_eq!(found("%{v:mysql}").backslashes, 0);
        assert_eq!(found("%{v:line}").backslashes, 0);
    }

    #[test]
    fn the_default_escape_follows_the_extension() {
        let of = |name: &str| Escape::for_file(Path::new(name));
        assert_eq!(of("x.sh"), Escape::Shell);
        assert_eq!(of("x.JSON"), Escape::Json);
        assert_eq!(of("x.html"), Escape::Xml);
        assert_eq!(of("x.sql"), Escape::Sql);
        assert_eq!(of("x.csv"), Escape::Csv);
        assert_eq!(of("x.txt"), Escape::Line);
        assert_eq!(of("x"), Escape::Line);
        assert!(refuses_records(Path::new("x.BAT")));
        assert!(!refuses_records(Path::new("x.sh")));
    }

    #[test]
    fn a_mask_is_parsed_or_says_what_is_wrong() {
        let m = mask("a%{x}b%{y:url}%%", Escape::Shell);
        assert_eq!(
            m.parts,
            [
                Part::Literal(b"a".to_vec()),
                Part::Field {
                    name: "x".into(),
                    escape: Escape::Shell
                },
                Part::Literal(b"b".to_vec()),
                Part::Field {
                    name: "y".into(),
                    escape: Escape::Url
                },
                Part::Literal(b"%".to_vec()),
            ]
        );
        let error = |text: &str| Mask::parse(text, Escape::Line, Path::new("a"), 0).unwrap_err();
        assert_eq!(error("%{x"), "Mask: %{ without its }");
        assert_eq!(error("%{x:nope}"), "Mask: unknown escape :nope");
        assert_eq!(error("%{}"), "Mask: %{} has no field name");
        assert!(error("  ").contains("empty"));
    }

    #[test]
    fn a_mask_keeps_the_variables_of_the_open_file() {
        let m = mask("%N %S %q", Escape::Line);
        assert_eq!(m.parts, [Part::Literal(b"a.csv 10 %q".to_vec())]);
    }

    #[test]
    fn a_variable_of_the_file_is_refused_inside_a_sql_string() {
        let sql = |text: &str| Mask::parse(text, Escape::Sql, Path::new("/d/a.csv"), 10);
        let error = sql("SELECT 1 WHERE x LIKE '%foo%'").unwrap_err();
        assert!(error.starts_with("Mask: %f in a SQL string"), "{error}");
        assert!(error.contains("write %%f for a %"), "{error}");
        assert!(
            sql("WHERE x = 'a' AND y = '%N'").is_err(),
            "the second string"
        );
        assert!(sql("'it''s %d'").is_err(), "a doubled quote stays inside");

        // What a % is meant to be, and what is not in a string.
        let literal = |text: &str| match sql(text).unwrap().parts.as_slice() {
            [Part::Literal(bytes)] => String::from_utf8(bytes.clone()).unwrap(),
            other => panic!("{other:?}"),
        };
        assert_eq!(literal("LIKE '%%foo%%'"), "LIKE '%foo%'");
        assert_eq!(literal("LIKE '%a%'"), "LIKE '%a%'");
        assert_eq!(literal("-- %N"), "-- a.csv");
        assert_eq!(literal("-- don't\n-- %N\n"), "-- don't\n-- a.csv\n");
        assert_eq!(literal("-- it's %N"), "-- it's a.csv");
        assert_eq!(literal("'a'\n%N"), "'a'\na.csv");
        assert!(sql("'%{#name}' %{x}").is_ok(), "fields have an escape");

        // Only where SQL is written: a field asking for it counts too.
        let line = |text: &str| Mask::parse(text, Escape::Line, Path::new("/d/a.csv"), 10);
        assert!(line("'%N'").is_ok());
        assert!(line("'%N' %{a:sql}").is_err());
        assert!(line("'%N' %{a:mysql}").is_err());
    }

    #[test]
    fn a_field_name_may_hold_a_colon_before_its_escape() {
        let m = mask("%{a:b:url}", Escape::Line);
        assert_eq!(
            m.parts,
            [Part::Field {
                name: "a:b".into(),
                escape: Escape::Url
            }]
        );
    }

    fn table_source(path: &Path, filter: Option<&str>) -> Source {
        Source::Table {
            path: path.to_path_buf(),
            sheet: 0,
            table: None,
            query: None,
            filter: filter.map(|f| sql_filter::SqlFilter::parse(f).unwrap()),
            unpacked: u64::MAX,
            marked: Default::default(),
        }
    }

    fn csv_rows(n: usize) -> tempfile::TempDir {
        let tmp = crate::testing::tempdir();
        let mut body = String::from("id,name\n");
        for i in 1..=n {
            body.push_str(&format!("{i},n{i}\n"));
        }
        fs::write(tmp.path().join("in.csv"), body).unwrap();
        tmp
    }

    fn run_to_file(source: Source, output: impl Into<Output>, out: &Path) -> Done {
        let job = Job::start(source, output, out.to_path_buf());
        while !job.is_done() {
            thread::sleep(Duration::from_millis(2));
        }
        job.finish().unwrap()
    }

    #[test]
    fn a_batch_writes_every_row_past_the_limit_of_the_screen() {
        let tmp = csv_rows(60_000);
        let out = tmp.path().join("out.txt");
        let m = mask("%{#} %{name}", Escape::Line);
        let done = run_to_file(table_source(&tmp.path().join("in.csv"), None), m, &out);
        assert_eq!(done.records, 60_000);
        assert!(!done.cancelled);
        let text = fs::read_to_string(&out).unwrap();
        assert_eq!(text.lines().count(), 60_000);
        assert_eq!(text.lines().last(), Some("60000 n60000"));
    }

    #[test]
    fn the_filter_applies_to_every_row() {
        let tmp = csv_rows(60_000);
        let out = tmp.path().join("out.txt");
        let source = table_source(&tmp.path().join("in.csv"), Some("id > 59998"));
        let done = run_to_file(source, mask("%{#}:%{id}", Escape::Line), &out);
        assert_eq!(done.records, 2);
        assert_eq!(
            fs::read_to_string(&out).unwrap(),
            "1:59999\n2:60000\n".replace('\n', std::str::from_utf8(shell::NEWLINE).unwrap())
        );
    }

    #[test]
    fn a_value_keeps_what_the_screen_would_change() {
        let tmp = crate::testing::tempdir();
        let long = "z".repeat(5_000);
        fs::write(
            tmp.path().join("in.csv"),
            format!("code,long\n007,{long}\n"),
        )
        .unwrap();
        let out = tmp.path().join("out.txt");
        let source = table_source(&tmp.path().join("in.csv"), Some("code = 7"));
        run_to_file(source, mask("%{code} %{long}", Escape::Line), &out);
        let text = fs::read_to_string(&out).unwrap();
        assert_eq!(text.trim_end(), format!("007 {long}"));
    }

    #[test]
    fn columns_answer_to_name_letter_and_position() {
        let tmp = csv_rows(1);
        let source = table_source(&tmp.path().join("in.csv"), None);
        let p = preview(&source, mask("%{NAME} %{b} %{2}", Escape::Line)).unwrap();
        assert_eq!(p.lines, ["n1 n1 n1"]);
        assert_eq!(p.scanned, 1);
        assert!(p.complete);
    }

    #[test]
    fn a_field_that_does_not_exist_is_an_error_not_an_empty_value() {
        let tmp = csv_rows(1);
        let source = table_source(&tmp.path().join("in.csv"), None);
        let error = preview(&source, mask("%{nope}", Escape::Line)).unwrap_err();
        assert_eq!(error, "Mask: no field named nope");
    }

    #[test]
    fn a_late_column_of_json_lines_is_found_by_the_preview() {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("in.jsonl");
        fs::write(&path, "{\"a\": 1}\n{\"a\": 2, \"b\": 3}\n").unwrap();
        let p = preview(&table_source(&path, None), mask("%{a}-%{b}", Escape::Line)).unwrap();
        assert_eq!(p.lines, ["1-", "2-3"]);
    }

    #[test]
    fn the_preview_stops_at_its_limits_and_shows_safe_lines() {
        let tmp = csv_rows(5_000);
        let source = table_source(&tmp.path().join("in.csv"), None);
        let p = preview(&source, mask("%{id}", Escape::Line)).unwrap();
        assert_eq!(p.lines.len(), PREVIEW_LINES);
        assert_eq!(p.scanned, SCAN_RECORDS);
        assert!(!p.complete);

        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("in.csv");
        fs::write(&path, "v,w\n\"a\u{1b}[31m\u{202E}\",=1\n").unwrap();
        let p = preview(
            &table_source(&path, None),
            mask("%{v:raw},%{w:csv}", Escape::Line),
        )
        .unwrap();
        assert_eq!(p.lines, ["a·[31m·,=1"]);
        assert_eq!(
            p.stats,
            Stats {
                suspicious: 1,
                formulas: 1,
                backslashes: 0,
                late_columns: 0,
                not_numbers: 0,
            }
        );
        assert!(p.stats.warning().unwrap().contains(":csvsafe"));
    }

    fn text_source(text: &str, kept: Option<Vec<usize>>, regex: Option<&str>) -> Source {
        let body = text.strip_suffix('\n').unwrap_or(text);
        Source::Text {
            text: text.to_string(),
            total: body.split('\n').count(),
            kept,
            regex: regex.map(|r| Regex::new(r).unwrap()),
            cut: false,
        }
    }

    #[test]
    fn text_gives_the_original_line_its_number_and_the_sequence() {
        let source = text_source("a\tb\r\nskip\nc\u{1b}\n", Some(vec![0, 2]), None);
        let p = preview(&source, mask("%{#}/%{#line}:%{line:raw}", Escape::Line)).unwrap();
        assert_eq!(p.lines, ["1/1:a·b", "2/3:c·"]);
    }

    #[test]
    fn text_gives_the_groups_of_the_filter() {
        let source = text_source(
            "10.0.0.1 GET 200\nbad\n10.0.0.2 PUT 500\n",
            Some(vec![0, 2]),
            Some(r"^(?P<ip>\S+) (\S+) (?P<status>\d+)$"),
        );
        let m = mask("%{ip} %{2} %{status} %{0}", Escape::Line);
        let p = preview(&source, m.clone()).unwrap();
        assert_eq!(
            p.lines,
            [
                "10.0.0.1 GET 200 10.0.0.1 GET 200",
                "10.0.0.2 PUT 500 10.0.0.2 PUT 500"
            ]
        );
    }

    #[test]
    fn a_line_with_no_match_has_empty_groups() {
        let source = text_source("x\n", None, Some(r"(?P<n>\d+)"));
        let p = preview(&source, mask("[%{n}]", Escape::Line)).unwrap();
        assert_eq!(p.lines, ["[]"]);
    }

    #[test]
    fn text_has_no_columns_by_position() {
        let source = text_source("x\n", None, None);
        assert!(preview(&source, mask("%{1}", Escape::Line)).is_err());
        assert!(preview(&source, mask("%{A}", Escape::Line)).is_err());
    }

    #[test]
    fn a_shell_script_starts_with_its_interpreter_and_is_not_executable() {
        let tmp = csv_rows(1);
        let out = tmp.path().join("run.sh");
        let source = table_source(&tmp.path().join("in.csv"), None);
        run_to_file(source, mask("echo %{name}", Escape::Shell), &out);
        assert_eq!(fs::read_to_string(&out).unwrap(), "#!/bin/sh\necho 'n1'\n");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&out).unwrap().permissions().mode();
            assert_eq!(mode & 0o111, 0, "a script made from data is reviewed first");
        }
    }

    #[test]
    fn an_existing_file_is_not_touched() {
        let tmp = csv_rows(1);
        let out = tmp.path().join("out.txt");
        fs::write(&out, "mine").unwrap();
        let source = table_source(&tmp.path().join("in.csv"), None);
        let job = Job::start(source, mask("%{id}", Escape::Line), out.clone());
        while !job.is_done() {
            thread::sleep(Duration::from_millis(2));
        }
        assert!(job.finish().unwrap_err().starts_with("Cannot create"));
        assert_eq!(fs::read_to_string(&out).unwrap(), "mine");
    }

    #[test]
    fn a_cancelled_batch_leaves_no_file() {
        let tmp = csv_rows(200_000);
        let out = tmp.path().join("out.txt");
        let source = table_source(&tmp.path().join("in.csv"), None);
        let job = Job::start(source, mask("%{id}", Escape::Line), out.clone());
        job.cancel();
        assert!(job.is_cancelling());
        while !job.is_done() {
            thread::sleep(Duration::from_millis(2));
        }
        let done = job.finish().unwrap();
        assert!(done.cancelled);
        assert!(done.records < 200_000);
        assert!(!out.exists());
    }

    #[test]
    fn a_bad_source_leaves_no_file() {
        let tmp = crate::testing::tempdir();
        let bad = tmp.path().join("in.parquet");
        fs::write(&bad, "not parquet").unwrap();
        let out = tmp.path().join("out.txt");
        let job = Job::start(
            table_source(&bad, None),
            mask("%{id}", Escape::Line),
            out.clone(),
        );
        while !job.is_done() {
            thread::sleep(Duration::from_millis(2));
        }
        assert!(job.finish().is_err());
        assert!(!out.exists());
    }

    fn output(text: &str, out: &str) -> Output {
        Output::parse(text, Path::new(out), Path::new("/d/in.csv"), 1).unwrap()
    }

    fn write_records(body: &str, out_name: &str, text: &str) -> String {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("in.csv");
        fs::write(&path, body).unwrap();
        let out = tmp.path().join(out_name);
        let source = table_source(&path, None);
        let found = preview(&source, output(text, out_name)).unwrap();
        run_to_file(source, found.output, &out);
        fs::read_to_string(&out).unwrap()
    }

    const PEOPLE: &str = "id,name\n1,\"a<b, \"\"c\"\"\"\n007,\n";

    #[test]
    fn an_empty_mask_means_whole_records_only_where_a_format_holds_them() {
        assert!(matches!(output("", "x.json"), Output::Records { .. }));
        assert!(matches!(output("  ", "x.HTML"), Output::Records { .. }));
        assert!(matches!(output("%{a}", "x.json"), Output::Lines(_)));
        let error = Output::parse("", Path::new("x.txt"), Path::new("a"), 0).unwrap_err();
        assert!(error.contains("empty"), "{error}");
        assert!(matches!(output("@a", "x.txt"), Output::Lines(_)));
        assert!(Output::parse("@ , ", Path::new("x.csv"), Path::new("a"), 0).is_err());
    }

    #[test]
    fn json_is_one_valid_array_with_nothing_lost() {
        let text = write_records(PEOPLE, "out.json", "");
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(
            value,
            serde_json::json!([
                {"id": "1", "name": "a<b, \"c\""},
                {"id": "007", "name": ""},
            ])
        );
    }

    #[test]
    fn an_empty_json_array_is_still_valid() {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("in.csv");
        fs::write(&path, "id,name\n1,a\n").unwrap();
        let out = tmp.path().join("out.json");
        let source = table_source(&path, Some("id > 5"));
        let wanted = Output::Records {
            format: Format::Json,
            wanted: None,
            fields: vec!["id".into()],
            title: String::new(),
        };
        run_to_file(source, wanted, &out);
        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&out).unwrap()).unwrap();
        assert_eq!(value, serde_json::json!([]));
    }

    #[test]
    fn json_lines_give_an_object_per_line() {
        let text = write_records(PEOPLE, "out.jsonl", "@name");
        let lines: Vec<serde_json::Value> = text
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(lines[1], serde_json::json!({"name": ""}));
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn csv_reads_back_the_same_table() {
        let text = write_records(PEOPLE, "out.csv", "");
        let mut reader = csv::Reader::from_reader(text.as_bytes());
        assert_eq!(reader.headers().unwrap(), vec!["id", "name"]);
        let rows: Vec<_> = reader.records().map(Result::unwrap).collect();
        assert_eq!(&rows[0], vec!["1", "a<b, \"c\""]);
        assert_eq!(&rows[1], vec!["007", ""]);
    }

    #[test]
    fn tsv_separates_with_tabs() {
        let text = write_records("a,b\n1,2\n", "out.tsv", "");
        assert_eq!(text, "a\tb\n1\t2\n");
    }

    #[test]
    fn xml_and_html_escape_values_names_and_title() {
        let body = "<evil>,b\n</td><script>,&\n";
        let xml = write_records(body, "out.xml", "");
        assert!(xml.starts_with("<?xml"));
        assert!(xml.contains("<field name=\"&lt;evil&gt;\">&lt;/td&gt;&lt;script&gt;</field>"));
        assert!(xml.contains("<field name=\"b\">&amp;</field>"));
        assert!(xml.ends_with("</records>\n"));

        let html = write_records(body, "out.html", "");
        assert!(html.contains("<th>&lt;evil&gt;</th>"));
        assert!(html.contains("<td>&lt;/td&gt;&lt;script&gt;</td>"));
        assert!(!html.contains("<script>"));
        assert!(html.ends_with("</html>\n"));
    }

    #[test]
    fn columns_can_be_chosen_by_name_letter_or_number() {
        let text = write_records("a,b\n1,2\n", "out.csv", "@b, A, #");
        assert_eq!(text, "b,A,#\n2,1,1\n");
    }

    #[test]
    fn a_wrong_column_is_an_error() {
        let tmp = csv_rows(1);
        let source = table_source(&tmp.path().join("in.csv"), None);
        let error = preview(&source, output("@nope", "x.csv")).unwrap_err();
        assert_eq!(error, "Mask: no field named nope");
    }

    #[test]
    fn whole_records_of_text_have_the_number_the_line_and_the_groups() {
        let source = text_source(
            "ip 1.2 get\nskip\n",
            Some(vec![0]),
            Some(r"ip (?P<addr>\S+) (\w+)"),
        );
        let found = preview(&source, output("", "x.jsonl")).unwrap();
        let value: serde_json::Value = serde_json::from_str(&found.lines[0]).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "line_number": 1, "line": "ip 1.2 get", "addr": "1.2", "2": "get"
            })
        );
    }

    #[test]
    fn a_column_found_after_the_preview_is_told_not_silently_lost() {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("in.jsonl");
        let mut body = String::new();
        for i in 0..SCAN_RECORDS + 5 {
            body.push_str(&format!("{{\"a\": {i}}}\n"));
        }
        body.push_str("{\"a\": 1, \"late\": 2}\n");
        fs::write(&path, body).unwrap();
        let out = tmp.path().join("out.jsonl");
        let source = table_source(&path, None);
        let found = preview(&source, output("", "out.jsonl")).unwrap();
        let done = run_to_file(source, found.output, &out);
        assert_eq!(done.stats.late_columns, 1);
        assert!(done.stats.warning().unwrap().contains("left out"));
    }

    #[test]
    fn the_preview_and_the_end_say_when_text_was_cut() {
        let source = Source::Text {
            text: "a\nb\n".into(),
            total: 1,
            kept: None,
            regex: None,
            cut: true,
        };
        let found = preview(&source, mask("%{line}", Escape::Line)).unwrap();
        assert!(found.cut);
        assert_eq!(found.lines, ["a"]);
        let tmp = crate::testing::tempdir();
        let done = run_to_file(source, found.output, &tmp.path().join("o.txt"));
        assert!(done.cut);
        let tmp = csv_rows(1);
        let source = table_source(&tmp.path().join("in.csv"), None);
        assert!(!preview(&source, mask("%{id}", Escape::Line)).unwrap().cut);
    }

    /// Writes `in.csv` and the template, and the file `out_name` from them.
    fn through_template(template: &str, body: &str, out_name: &str, mask_text: &str) -> String {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("in.csv");
        fs::write(&path, body).unwrap();
        let dir = tmp.path().join("batch");
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join("t.tpl"), template).unwrap();
        let out = tmp.path().join(out_name);
        let parsed = Output::parse_in(mask_text, &out, &path, 1, Some(&dir)).unwrap();
        let source = table_source(&path, None);
        let found = preview(&source, parsed).unwrap();
        run_to_file(source, found.output, &out);
        fs::read_to_string(&out).unwrap()
    }

    fn template_error(template: &str) -> String {
        let tmp = crate::testing::tempdir();
        fs::write(tmp.path().join("t.tpl"), template).unwrap();
        Output::parse_in(
            ":t",
            Path::new("x.txt"),
            Path::new("a"),
            0,
            Some(tmp.path()),
        )
        .unwrap_err()
    }

    const NEWLINE: &str = if cfg!(windows) { "\r\n" } else { "\n" };

    #[test]
    fn a_template_writes_its_header_records_and_footer() {
        let template =
            "%%# a comment\n%%header\nfirst\n%%record\n- %{#}: %{name}\n  %{id}\n%%footer\nlast\n";
        let text = through_template(template, "id,name\n1,a\n2,b\n", "out.txt", ":t");
        let expected = "first\n- 1: a\n  1\n- 2: b\n  2\nlast\n";
        assert_eq!(text, expected.replace('\n', NEWLINE));
    }

    #[test]
    fn a_template_with_no_sections_is_all_record_and_ends_its_lines() {
        let text = through_template("%{id}", "id\n1\n2\n", "out.txt", " :t ");
        assert_eq!(text, "1\n2\n".replace('\n', NEWLINE));
    }

    #[test]
    fn a_template_follows_the_escape_of_the_output_and_reads_crlf() {
        let template = "%%header\r\n<ul>\r\n%%record\r\n<li>%{v}</li>\r\n<!-- %{v:raw} -->\r\n%%footer\r\n</ul>\r\n";
        let text = through_template(template, "v\n<x>\n", "out.html", ":t");
        let expected = "<ul>\n<li>&lt;x&gt;</li>\n<!-- <x> -->\n</ul>\n";
        assert_eq!(text, expected.replace('\n', NEWLINE));
    }

    #[test]
    fn a_template_quotes_a_hostile_value_in_a_script() {
        let text = through_template("echo %{v}\n", "v\n$(id)\n", "run.sh", ":t");
        assert_eq!(text, "#!/bin/sh\necho '$(id)'\n");
    }

    #[test]
    fn the_open_file_goes_through_the_escape_everywhere() {
        let m = Mask::parse(
            "%{#name} %{#base:url} %{#ext} %{#size} %{#dir:raw}",
            Escape::Xml,
            Path::new("/d/<a b>.csv"),
            10,
        )
        .unwrap();
        assert_eq!(
            m.parts,
            [Part::Literal(
                b"&lt;a b&gt;.csv %3Ca%20b%3E csv 10 /d".to_vec()
            )]
        );
        let text = through_template(
            "%%header\n<title>%{#name}</title>\n%%record\n%{v}\n",
            "v\n1\n",
            "out.html",
            ":t",
        );
        assert!(text.starts_with("<title>in.csv</title>"), "{text}");
    }

    #[test]
    fn a_footer_knows_how_many_records_were_written() {
        let template = "%%record\n%{id}\n%%footer\n-- %{count} records\n";
        let text = through_template(template, "id\n1\n2\n3\n", "out.txt", ":t");
        assert_eq!(text, "1\n2\n3\n-- 3 records\n".replace('\n', NEWLINE));
        // A number, whatever the file: no quotes in SQL.
        let text = through_template(template, "id\n1\n", "out.sql", ":t");
        assert!(text.ends_with(&format!("-- 1 records{NEWLINE}")), "{text}");
    }

    #[test]
    fn a_column_called_count_is_still_a_column_in_a_record() {
        let template = "%%record\n%{count}\n%%footer\n%{count}\n";
        let text = through_template(template, "count\n7\n8\n", "out.txt", ":t");
        assert_eq!(text, "7\n8\n2\n".replace('\n', NEWLINE));
    }

    #[test]
    fn the_date_is_the_same_in_every_field_and_comes_in_each_form() {
        use chrono::TimeZone;
        let moment = Utc.with_ymd_and_hms(2026, 10, 30, 17, 5, 9).unwrap();
        let context = Context {
            now: moment.with_timezone(&Local),
            ..test_context(Path::new("/d/a.csv"), 1)
        };
        let value = |field: &str| {
            let m = Mask::parse_in(&format!("%{{{field}:raw}}"), Escape::Line, &context).unwrap();
            match m.parts.as_slice() {
                [Part::Literal(bytes)] => String::from_utf8(bytes.clone()).unwrap(),
                other => panic!("{other:?}"),
            }
        };
        let now = &context.now;
        assert_eq!(value("#utc"), "2026-10-30T17:05:09Z");
        assert_eq!(value("#epoch"), "1793379909");
        assert_eq!(value("#date"), now.format("%Y-%m-%d").to_string());
        assert_eq!(value("#time"), now.format("%H:%M:%S").to_string());
        let datetime = now.format("%Y-%m-%dT%H:%M:%S%:z").to_string();
        assert_eq!(value("#datetime"), datetime);
        assert_eq!(value("#year"), now.format("%Y").to_string());
        assert_eq!(value("#month"), now.format("%m").to_string());
        assert_eq!(value("#day"), now.format("%d").to_string());
        // Fridays in UTC; the local date may be another day, around midnight.
        let weekday = now.weekday();
        assert_eq!(value("#weekday"), weekday.number_from_monday().to_string());
        assert_eq!(
            value("#weekday_name"),
            now.format("%A").to_string().to_lowercase()
        );
        assert_eq!(value("#output"), "/o/out.txt");
    }

    #[test]
    fn a_shared_field_goes_through_the_escape() {
        let in_sql = Mask::parse("%{#date}", Escape::Sql, Path::new("a"), 0).unwrap();
        let [Part::Literal(bytes)] = in_sql.parts.as_slice() else {
            panic!("{:?}", in_sql.parts);
        };
        assert!(bytes.starts_with(b"'") && bytes.ends_with(b"'"));

        let path = std::env::var("PATH").unwrap();
        let in_shell = Mask::parse("%{$PATH}", Escape::Shell, Path::new("a"), 0).unwrap();
        assert_eq!(
            in_shell.parts,
            [Part::Literal(format!("'{path}'").into_bytes())]
        );
    }

    #[test]
    fn the_environment_is_read_by_name_and_told() {
        let m = Mask::parse("%{$PATH:raw}", Escape::Line, Path::new("a"), 0).unwrap();
        assert_eq!(m.environment, ["PATH"]);

        let error = |text: &str| Mask::parse(text, Escape::Line, Path::new("a"), 0).unwrap_err();
        assert_eq!(
            error("%{$XY_NO_SUCH_VARIABLE}"),
            "Mask: no environment variable XY_NO_SUCH_VARIABLE"
        );
        assert_eq!(error("%{$}"), "Mask: %{$} has no variable name");

        let template =
            "%%header\n%{$PATH:raw}\n%%record\n%{id}%{$HOME:raw}\n%%footer\n%{$PATH:raw}\n";
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("in.csv");
        fs::write(&path, "id\n1\n").unwrap();
        fs::write(tmp.path().join("t.tpl"), template).unwrap();
        if std::env::var_os("HOME").is_some() {
            let parsed =
                Output::parse_in(":t", Path::new("x.txt"), &path, 0, Some(tmp.path())).unwrap();
            let found = preview(&table_source(&path, None), parsed).unwrap();
            assert_eq!(found.environment, ["HOME", "PATH"]);
        }
        let csv = csv_rows(1);
        let source = table_source(&csv.path().join("in.csv"), None);
        let found = preview(&source, mask("%{id}", Escape::Line)).unwrap();
        assert!(found.environment.is_empty());
    }

    #[test]
    fn the_template_name_is_known_only_in_a_template() {
        let error = Mask::parse("%{#template}", Escape::Line, Path::new("a"), 0).unwrap_err();
        assert_eq!(error, "Mask: cannot tell %{#template} here");
        let text = through_template("%{#template}-%{id}\n", "id\n1\n", "out.txt", ":t");
        assert_eq!(text.trim_end(), "t-1");
    }

    #[test]
    fn a_wrong_template_says_what_is_wrong() {
        let error = template_error;
        assert_eq!(error(""), "Template t: it has no record");
        assert_eq!(error("%%header\nx\n"), "Template t: it has no record");
        assert_eq!(error("%%record\n \n"), "Template t: it has no record");
        assert_eq!(error("%%heder\nx\n"), "Template t: unknown section %%heder");
        assert_eq!(
            error("%%record\na\n%%record\nb\n"),
            "Template t: two sections %%record"
        );
        assert_eq!(
            error("loose\n%%record\na\n"),
            "Template t: text before the first section"
        );
        assert_eq!(
            error("%%footer\n%{id}\n%%record\na\n"),
            "Template t: %{id} is a field of a record, outside the record"
        );
        assert_eq!(
            error("%%header\n%{count}\n%%record\na\n"),
            "Template t: %{count} is only known in the footer"
        );
        assert_eq!(
            error("%{a:nope}\n"),
            "Template t: Mask: unknown escape :nope"
        );
    }

    #[test]
    fn a_template_must_be_a_small_file_in_its_folder() {
        let tmp = crate::testing::tempdir();
        let dir = tmp.path();
        let parse =
            |text: &str| Output::parse_in(text, Path::new("x.txt"), Path::new("a"), 0, Some(dir));
        assert!(parse(":none").unwrap_err().starts_with("No template none"));
        fs::create_dir(dir.join("d.tpl")).unwrap();
        assert!(parse(":d").unwrap_err().contains("not a file"));
        let big = "x".repeat(MAX_TEMPLATE_BYTES as usize + 1);
        fs::write(dir.join("big.tpl"), big).unwrap();
        assert!(parse(":big").unwrap_err().contains("too large"));
        fs::write(dir.join("bin.tpl"), [0xFF, 0xFE]).unwrap();
        assert!(parse(":bin").is_err());
        assert!(
            Output::parse_in(":t", Path::new("x"), Path::new("a"), 0, None)
                .unwrap_err()
                .contains("no configuration folder")
        );
    }

    /// What a template writes for a CSV of `n` rows with columns `id` and `v`.
    fn template_output(template: &str, rows: &str) -> (Preview, String, Done) {
        let tmp = crate::testing::tempdir();
        fs::write(tmp.path().join("t.tpl"), template).unwrap();
        let data = tmp.path().join("in.csv");
        fs::write(&data, format!("id,v\n{rows}")).unwrap();
        let out = tmp.path().join("out.txt");
        let output = Output::parse_in(":t", &out, &data, 1, Some(tmp.path())).unwrap();
        let source = table_source(&data, None);
        let found = preview(&source, output).unwrap();
        let done = run_to_file(source, found.output.clone(), &out);
        let written = fs::read_to_string(&out).unwrap();
        (found, written, done)
    }

    #[test]
    fn a_template_chooses_what_to_write_for_each_record() {
        let template = "%%record\n%%if v > 10\nbig %{id}\n%%else\nsmall %{id}\n%%end\n";
        let (found, written, done) = template_output(template, "1,5\n2,50\n");
        assert_eq!(written.replace("\r\n", "\n"), "small 1\nbig 2\n");
        assert_eq!(done.records, 2);
        assert_eq!(found.lines, ["small 1", "big 2"]);
        assert!(found.unseen.is_empty());
    }

    #[test]
    fn a_record_with_nothing_to_write_does_not_count() {
        let template = "%%record\n%%if v = 1\n%{#} %{id}\n%%end\n%%footer\n%{count}\n";
        let (_, written, done) = template_output(template, "a,1\nb,2\nc,1\n");
        // `#` is the record's place in what was read, and `count` what was written.
        assert_eq!(written.replace("\r\n", "\n"), "1 a\n3 c\n2\n");
        assert_eq!(done.records, 2);
    }

    #[test]
    fn stop_ends_the_file_and_keeps_its_footer() {
        let template =
            "%%record\nbefore %{id}\n%%stop if v = 2\nafter %{id}\n%%footer\nend %{count}\n";
        let (found, written, done) = template_output(template, "a,1\nb,2\nc,3\n");
        assert_eq!(
            written.replace("\r\n", "\n"),
            "before a\nafter a\nbefore b\nend 2\n"
        );
        assert!(done.stopped);
        assert!(!done.cancelled);
        assert_eq!(found.stopped, Some(2));
        assert!(found.complete);
    }

    #[test]
    fn a_condition_can_count_the_records() {
        let (_, written, done) =
            template_output("%%stop if # > 2\n%{id}\n", "a,1\nb,2\nc,3\nd,4\n");
        assert_eq!(written.replace("\r\n", "\n"), "a\nb\n");
        assert_eq!(done.records, 2);
    }

    #[test]
    fn the_preview_says_which_branches_no_record_took() {
        let template = "%%if v = 9\nx\n%%else\ny\n%%if v = 1\nz\n%%end\n%%end\n";
        let (found, _, _) = template_output(template, "a,2\n");
        assert_eq!(found.unseen, ["%%if v = 9", "%%if v = 1"]);
        let (found, _, _) = template_output(template, "a,9\n");
        assert_eq!(found.unseen, ["%%else of %%if v = 9", "%%if v = 1"]);
    }

    #[test]
    fn a_condition_on_a_field_that_does_not_exist_is_an_error() {
        let tmp = crate::testing::tempdir();
        fs::write(tmp.path().join("t.tpl"), "%%if nope = 1\nx\n%%end\n").unwrap();
        let data = tmp.path().join("in.csv");
        fs::write(&data, "id\n1\n").unwrap();
        let out = tmp.path().join("out.txt");
        let output = Output::parse_in(":t", &out, &data, 1, Some(tmp.path())).unwrap();
        let error = preview(&table_source(&data, None), output).unwrap_err();
        assert_eq!(error, "Mask: no field named nope");
    }

    #[test]
    fn a_template_with_badly_built_conditions_says_what_is_wrong() {
        let parse = |text: &str| {
            let context = test_context(Path::new("f"), 1);
            parse_template(text, Escape::Raw, &context).unwrap_err()
        };
        assert_eq!(parse("%%end\n"), "%%end without %%if");
        assert_eq!(parse("%%else\n"), "%%else without %%if");
        assert_eq!(parse("%%if a\nx\n"), "%%if without %%end");
        assert_eq!(parse("%%if\nx\n%%end\n"), "%%if needs a condition");
        assert!(parse("%%stop\n").contains("%%stop if"));
        assert_eq!(
            parse("%%if a\n%%else\n%%else\n%%end\n"),
            "two %%else in the same %%if"
        );
        assert!(parse("%%if a = \nx\n%%end\n").starts_with("%%if a ="));
        assert!(parse("%%header\n%%if a\n%%end\n%%record\nx\n").contains("only in the record"));
        let deep = "%%if a\n".repeat(MAX_IF_DEPTH + 1);
        assert!(parse(&deep).contains("inside each other"));
    }

    #[test]
    fn a_blank_line_between_conditions_stays() {
        let (_, written, _) =
            template_output("%%if v = 1\nx\n%%end\n\n%%if v = 1\ny\n%%end\n", "a,1\n");
        assert_eq!(written.replace("\r\n", "\n"), "x\n\ny\n");
    }

    #[test]
    fn a_decimal_is_read_only_as_its_format_writes_it() {
        let read =
            |text, format| parse_decimal(text, format).map(|(v, s)| (v / (TOTAL_UNIT / 100), s));
        use NumberFormat::{Br, Plain, Us};
        assert_eq!(read("1234.56", Plain), Some((123_456, 2)));
        assert_eq!(read("-1234.5", Plain), Some((-123_450, 1)));
        assert_eq!(read("+7", Plain), Some((700, 0)));
        assert_eq!(read(".5", Plain), Some((50, 1)));
        assert_eq!(read("1.234,56", Br), Some((123_456, 2)));
        assert_eq!(read("1234,56", Br), Some((123_456, 2)));
        assert_eq!(read("1,234.56", Us), Some((123_456, 2)));
        // Each of these is another format, or nothing.
        for (text, format) in [
            ("1,234.56", Plain),
            ("1,5", Plain),
            ("1.5", Br),
            ("1.23.456", Br),
            ("1234,567.8", Us),
            ("", Plain),
            ("-", Plain),
            (".", Plain),
            ("1e5", Plain),
            ("abc", Plain),
            ("٣", Plain),
            ("1.2.3", Plain),
        ] {
            assert_eq!(parse_decimal(text, format), None, "{text}");
        }
        // Past what it can hold exactly: refused, never rounded.
        assert_eq!(parse_decimal(&"9".repeat(60), Plain), None);
        assert_eq!(parse_decimal(&format!("0.{}", "1".repeat(19)), Plain), None);
    }

    #[test]
    fn a_decimal_is_written_in_its_format() {
        use NumberFormat::{Br, Plain, Us};
        let value = |text| parse_decimal(text, Plain).unwrap().0;
        assert_eq!(write_decimal(value("1234567.5"), 2, Br), "1.234.567,50");
        assert_eq!(write_decimal(value("1234567.5"), 2, Us), "1,234,567.50");
        assert_eq!(write_decimal(value("1234567.5"), 1, Plain), "1234567.5");
        assert_eq!(write_decimal(value("-999"), 0, Br), "-999");
        assert_eq!(write_decimal(value("-1000"), 0, Br), "-1.000");
        assert_eq!(write_decimal(value("0.125"), 2, Plain), "0.13");
        assert_eq!(write_decimal(value("-0.001"), 2, Plain), "0.00");
    }

    fn footer_of(footer: &str, rows: &str) -> (String, Done) {
        let template = format!("%%record\n%{{id}}\n%%footer\n{footer}\n");
        let (_, written, done) = template_output(&template, rows);
        let written = written.replace("\r\n", "\n");
        let last = written.lines().last().unwrap_or("").to_string();
        (last, done)
    }

    #[test]
    fn a_total_is_exact() {
        let (total, _) = footer_of("%{sum:v}", "a,0.1\nb,0.2\n");
        assert_eq!(total, "0.3");
        let (total, _) = footer_of(
            "%{sum:v} %{min:v} %{max:v} %{avg:v}",
            "a,1.50\nb,-3\nc,10\n",
        );
        assert_eq!(total, "8.50 -3.00 10.00 2.8333");
    }

    #[test]
    fn a_total_reads_and_writes_the_format_asked_for() {
        let (total, _) = footer_of("%{sum:v:br}", "a,\"1.000,50\"\nb,\"2.000,25\"\n");
        assert_eq!(total, "3.000,75");
    }

    #[test]
    fn a_value_that_is_not_a_number_is_counted_not_hidden() {
        let (total, done) = footer_of("%{sum:v}", "a,1\nb,oops\nc,\nd,\"1,5\"\ne,2\n");
        assert_eq!(total, "3");
        // An empty one is no value, as in SQL.
        assert_eq!(done.stats.not_numbers, 2);
        assert!(done.stats.warning().unwrap().contains("left out"));
    }

    #[test]
    fn a_total_with_no_values_is_zero_or_nothing() {
        let (total, _) = footer_of("[%{sum:v}][%{min:v}][%{avg:v}]", "a,\n");
        assert_eq!(total, "[0][][]");
    }

    #[test]
    fn a_total_sees_only_the_records_written() {
        let template = "%%record\n%%if v > 1\n%{id}\n%%end\n%%stop if v = 3\n%%footer\n%{sum:v}\n";
        let (_, written, _) = template_output(template, "a,1\nb,2\nc,3\nd,4\n");
        assert_eq!(written.replace("\r\n", "\n"), "b\nc\n5\n");
    }

    #[test]
    fn a_total_goes_through_the_escape_of_the_file() {
        let total = |out: &str, footer: &str| {
            let tmp = crate::testing::tempdir();
            let body = format!("%%record\n%{{id}}\n%%footer\n{footer}\n");
            fs::write(tmp.path().join("t.tpl"), body).unwrap();
            let data = tmp.path().join("in.csv");
            fs::write(&data, "id,v\na,\"1.234,50\"\n").unwrap();
            let out = tmp.path().join(out);
            let output = Output::parse_in(":t", &out, &data, 1, Some(tmp.path())).unwrap();
            run_to_file(table_source(&data, None), output, &out);
            let text = fs::read_to_string(&out).unwrap().replace("\r\n", "\n");
            text.lines().last().unwrap().to_string()
        };
        // In a CSV, a comma in the number has to be inside quotes.
        assert_eq!(total("o.csv", "%{sum:v:br}"), "\"1.234,50\"");
        assert_eq!(total("o.sql", "%{sum:v:br}"), "'1.234,50'");
        assert_eq!(total("o.sql", "%{sum:v:br:raw}"), "1.234,50");
    }

    #[test]
    fn a_plain_total_is_a_number_in_any_file() {
        let tmp = crate::testing::tempdir();
        fs::write(
            tmp.path().join("t.tpl"),
            "%%record\n%{id}\n%%footer\nSELECT %{sum:v}\n",
        )
        .unwrap();
        let data = tmp.path().join("in.csv");
        fs::write(&data, "id,v\na,1.5\n").unwrap();
        let out = tmp.path().join("o.sql");
        let output = Output::parse_in(":t", &out, &data, 1, Some(tmp.path())).unwrap();
        run_to_file(table_source(&data, None), output, &out);
        let text = fs::read_to_string(&out).unwrap().replace("\r\n", "\n");
        assert_eq!(text.lines().last().unwrap(), "SELECT 1.5");
    }

    #[test]
    fn a_total_of_a_column_that_does_not_exist_is_an_error() {
        let tmp = crate::testing::tempdir();
        fs::write(
            tmp.path().join("t.tpl"),
            "%%record\n%{id}\n%%footer\n%{sum:nope}\n",
        )
        .unwrap();
        let data = tmp.path().join("in.csv");
        fs::write(&data, "id\n1\n").unwrap();
        let out = tmp.path().join("o.txt");
        let output = Output::parse_in(":t", &out, &data, 1, Some(tmp.path())).unwrap();
        let error = preview(&table_source(&data, None), output).unwrap_err();
        assert_eq!(error, "Mask: no field named nope");
    }

    #[test]
    fn a_total_is_known_only_in_the_footer() {
        let parse = |text: &str| {
            let context = test_context(Path::new("f"), 1);
            parse_template(text, Escape::Raw, &context).unwrap_err()
        };
        assert!(parse("%%record\n%{sum:v}\n").contains("only known in the footer"));
        assert!(parse("%%header\n%{sum:v}\n%%record\nx\n").contains("only known in the footer"));
        assert!(parse("%%record\nx\n%%footer\n%{sum:}\n").contains("no column"));
    }

    #[test]
    fn a_column_called_sum_is_still_a_field_of_the_record() {
        let m = mask("%{sum:sql}", Escape::Raw);
        assert!(matches!(&m.parts[0], Part::Field { name, .. } if name == "sum"));
    }

    /// A CSV through a Batch into a new SQLite file.
    fn into_sqlite(csv: &str, mask: &str) -> (tempfile::TempDir, PathBuf, Done) {
        let tmp = crate::testing::tempdir();
        let data = tmp.path().join("in.csv");
        fs::write(&data, csv).unwrap();
        let out = tmp.path().join("out.db");
        let output = Output::parse_in(mask, &out, &data, 1, None).unwrap();
        let source = table_source(&data, None);
        let found = preview(&source, output).unwrap();
        let done = run_to_file(source, found.output, &out);
        (tmp, out, done)
    }

    fn rows_of(db: &Path, query: &str) -> Vec<String> {
        let db = rusqlite::Connection::open(db).unwrap();
        let mut query = db.prepare(query).unwrap();
        query
            .query_map([], |row| row.get::<_, rusqlite::types::Value>(0))
            .unwrap()
            .map(|v| format!("{:?}", v.unwrap()))
            .collect()
    }

    #[test]
    fn a_sqlite_file_keeps_each_value_as_it_was() {
        let (_tmp, out, done) = into_sqlite("id,zip\n1,007\n2,010\n", "");
        assert_eq!(done.records, 2);
        assert_eq!(
            rows_of(&out, "SELECT zip FROM records ORDER BY rowid"),
            ["Text(\"007\")", "Text(\"010\")"]
        );
        let names = out.with_file_name("out.db-journal");
        assert!(!names.exists(), "no journal stays next to it");
    }

    #[test]
    fn a_sqlite_file_keeps_the_types_of_the_source() {
        let tmp = crate::testing::tempdir();
        let source_db = tmp.path().join("in.db");
        let db = rusqlite::Connection::open(&source_db).unwrap();
        db.execute_batch(
            "CREATE TABLE t (i INTEGER, r REAL, s TEXT, b BLOB, n TEXT);
             INSERT INTO t VALUES (7, 1.5, 'x', x'00ff', NULL);",
        )
        .unwrap();
        drop(db);
        let source = Source::Table {
            path: source_db.clone(),
            sheet: 0,
            table: Some("t".to_string()),
            query: None,
            filter: None,
            unpacked: u64::MAX,
            marked: Default::default(),
        };
        let out = tmp.path().join("out.sqlite3");
        let output = Output::parse_in("", &out, &source_db, 1, None).unwrap();
        let found = preview(&source, output).unwrap();
        run_to_file(source, found.output, &out);
        let row = |column: &str| rows_of(&out, &format!("SELECT {column} FROM records"));
        assert_eq!(row("i"), ["Integer(7)"]);
        assert_eq!(row("r"), ["Real(1.5)"]);
        assert_eq!(row("s"), ["Text(\"x\")"]);
        assert_eq!(row("b"), ["Blob([0, 255])"]);
        assert_eq!(row("n"), ["Null"]);
    }

    #[test]
    fn a_batch_writes_all_that_a_whole_query_gives() {
        let tmp = crate::testing::tempdir();
        let source_db = tmp.path().join("in.db");
        rusqlite::Connection::open(&source_db)
            .unwrap()
            .execute_batch(
                "CREATE TABLE a (id, name); INSERT INTO a VALUES (1, 'ana'), (2, 'bob');
                 CREATE TABLE b (who, item); INSERT INTO b VALUES (1, 'pen'), (2, 'cup');",
            )
            .unwrap();
        let source = Source::Table {
            path: source_db.clone(),
            sheet: 0,
            table: Some("a".to_string()),
            query: Some("SELECT name, item FROM a JOIN b ON who = id ORDER BY id".to_string()),
            filter: None,
            unpacked: u64::MAX,
            marked: Default::default(),
        };
        let out = tmp.path().join("out.txt");
        let mask = Mask::parse("%{name}=%{item}", Escape::Raw, &source_db, 1).unwrap();
        let found = preview(&source, mask).unwrap();
        run_to_file(source, found.output, &out);
        assert_eq!(std::fs::read_to_string(&out).unwrap(), "ana=pen\nbob=cup\n");
    }

    #[test]
    fn a_hostile_value_or_column_name_is_only_data_in_a_sqlite_file() {
        let attack = "x'); DROP TABLE records; --";
        let csv = format!("\"a\"\"); DROP TABLE records; --\",ID,id,\n\"{attack}\",1,2,3\n");
        let (_tmp, out, done) = into_sqlite(&csv, "");
        assert_eq!(done.records, 1);
        let db = rusqlite::Connection::open(&out).unwrap();
        let mut names = db
            .prepare("SELECT name FROM pragma_table_info('records')")
            .unwrap();
        let names: Vec<String> = names
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert_eq!(
            names,
            ["a\"); DROP TABLE records; --", "ID", "id_2", "column_4"]
        );
        let value: String = db
            .query_row(
                "SELECT \"a\"\"); DROP TABLE records; --\" FROM records",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(value, attack);
    }

    #[test]
    fn a_sqlite_file_can_take_only_the_fields_named_and_the_filter() {
        let tmp = crate::testing::tempdir();
        let data = tmp.path().join("in.csv");
        fs::write(&data, "id,name,other\n1,ana,x\n2,bob,y\n").unwrap();
        let out = tmp.path().join("out.db");
        let output = Output::parse_in("@name", &out, &data, 1, None).unwrap();
        let source = table_source(&data, Some("id = 2"));
        let found = preview(&source, output).unwrap();
        assert_eq!(found.lines, ["CREATE TABLE records (\"name\")", "bob"]);
        let done = run_to_file(source, found.output, &out);
        assert_eq!(done.records, 1);
        assert_eq!(rows_of(&out, "SELECT name FROM records"), ["Text(\"bob\")"]);
    }

    #[test]
    fn a_text_file_goes_into_sqlite_with_its_line_numbers() {
        let tmp = crate::testing::tempdir();
        let out = tmp.path().join("out.db");
        let source = text_source("first\nsecond\n", Some(vec![1]), None);
        let output = Output::parse_in("", &out, Path::new("a.txt"), 1, None).unwrap();
        let found = preview(&source, output).unwrap();
        run_to_file(source, found.output, &out);
        assert_eq!(
            rows_of(&out, "SELECT line_number FROM records"),
            ["Integer(2)"]
        );
        assert_eq!(
            rows_of(&out, "SELECT line FROM records"),
            ["Text(\"second\")"]
        );
    }

    #[test]
    fn a_sqlite_file_takes_no_mask_or_template() {
        let out = Path::new("out.sqlite");
        for mask in ["%{id}", "INSERT", ":report"] {
            let error = Output::parse_in(mask, out, Path::new("a"), 1, None).unwrap_err();
            assert!(error.contains("whole records"), "{mask}: {error}");
        }
    }

    #[test]
    fn a_sqlite_file_does_not_touch_what_exists() {
        let tmp = crate::testing::tempdir();
        let data = tmp.path().join("in.csv");
        fs::write(&data, "id\n1\n").unwrap();
        let out = tmp.path().join("out.db");
        fs::write(&out, "mine").unwrap();
        let output = Output::parse_in("", &out, &data, 1, None).unwrap();
        let source = table_source(&data, None);
        let found = preview(&source, output).unwrap();
        let job = Job::start(source, found.output, out.clone());
        while !job.is_done() {
            thread::sleep(Duration::from_millis(2));
        }
        assert!(job.finish().unwrap_err().starts_with(NOT_CREATED));
        assert_eq!(fs::read_to_string(&out).unwrap(), "mine");
    }

    #[test]
    fn a_cancelled_sqlite_file_is_not_kept() {
        let tmp = csv_rows(50_000);
        let data = tmp.path().join("in.csv");
        let out = tmp.path().join("out.db");
        let output = Output::parse_in("", &out, &data, 1, None).unwrap();
        let source = table_source(&data, None);
        let found = preview(&source, output).unwrap();
        let job = Job::start(source, found.output, out.clone());
        job.cancel();
        while !job.is_done() {
            thread::sleep(Duration::from_millis(2));
        }
        // It may have ended before seeing the cancel: then the file is whole.
        match job.finish().unwrap() {
            done if done.cancelled => assert!(!out.exists()),
            _ => assert!(out.exists()),
        }
    }

    #[test]
    fn marked_records_are_the_only_ones_written_past_the_limit_of_the_screen() {
        let tmp = csv_rows(60_000);
        let data = tmp.path().join("in.csv");
        let Source::Table {
            path,
            sheet,
            table,
            query,
            filter,
            unpacked,
            ..
        } = table_source(&data, Some("id != 3"))
        else {
            panic!("a table");
        };
        let source = Source::Table {
            path,
            sheet,
            table,
            query,
            filter,
            unpacked,
            // Places, from 0; the third record is hidden by the filter.
            marked: BTreeSet::from([0, 2, 59_999]),
        };
        let out = tmp.path().join("out.txt");
        let mask = Mask::parse("%{#} %{id}", Escape::Raw, &data, 1).unwrap();
        let found = preview(&source, mask).unwrap();
        assert_eq!(found.lines, ["1 1", "2 60000"]);
        let done = run_to_file(source, found.output, &out);
        assert_eq!(done.records, 2);
        let text = fs::read_to_string(&out).unwrap().replace("\r\n", "\n");
        assert_eq!(text, "1 1\n2 60000\n");
    }

    #[test]
    fn only_a_plain_name_asks_for_a_template() {
        assert_eq!(template_name(":a-b_c.1"), Some("a-b_c.1"));
        for mask in [
            ":", ":../up", ":a/b", ":a\\b", ":.hidden", ": %{a}", ":%{a}", "a",
        ] {
            assert_eq!(template_name(mask), None, "{mask}");
        }
        assert!(matches!(output(": %{id}", "x.sh"), Output::Lines(_)));
    }

    #[test]
    fn a_template_says_what_it_does_in_its_first_comment() {
        let about =
            template_about("%%# One UPDATE per record. Write it\n%%# to a .sql.\n%%record\n");
        assert_eq!(about, "One UPDATE per record.");
        let about = template_about("%%# Reads well\n%%# on two lines\n%%record\n%%# not this\n");
        assert_eq!(about, "Reads well on two lines");
        assert_eq!(template_about("%%record\n%%# too late\n"), "");
        // The file wrote it: nothing in it may act on the screen.
        let about = template_about("%%# safe\u{1b}[2J\u{202e}text\n");
        assert_eq!(about, "safe·[2J·text");
    }

    #[test]
    fn a_template_on_the_screen_hides_nothing() {
        let long = "x".repeat(TEMPLATE_VIEW_CHARS + 1);
        let lines = view_lines(&format!("%%# a\n\n\tb\u{1b}\n{long}\n"));
        assert_eq!(lines[..3], ["%%# a", "", "    b·"]);
        assert_eq!(lines[3].chars().count(), TEMPLATE_VIEW_CHARS);
        assert_eq!(lines[4], "x", "a long line continues below");
    }

    #[test]
    fn the_templates_there_are_come_sorted() {
        let tmp = crate::testing::tempdir();
        assert!(names_in(&tmp.path().join("none")).is_empty());
        for name in ["b.tpl", "a.tpl", "c.txt", ".h.tpl"] {
            fs::write(tmp.path().join(name), "x").unwrap();
        }
        assert_eq!(names_in(tmp.path()), ["a", "b"]);
    }

    #[test]
    fn the_preview_of_a_long_header_still_shows_records() {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("in.csv");
        fs::write(&path, "v\nrow\n").unwrap();
        let header: String = (0..40).map(|i| format!("h{i}\n")).collect();
        fs::write(
            tmp.path().join("t.tpl"),
            format!("%%header\n{header}%%record\n%{{v}}\n"),
        )
        .unwrap();
        let parsed =
            Output::parse_in(":t", Path::new("x.txt"), &path, 0, Some(tmp.path())).unwrap();
        let found = preview(&table_source(&path, None), parsed).unwrap();
        assert_eq!(found.lines, ["h37", "h38", "h39", "row"]);
    }

    proptest! {
        /// Whatever the value, it stays one literal and reads back the same.
        #[test]
        fn a_mysql_literal_reads_back_the_value(text in "[\\\\'a0Z\\x00\\x1a\\n ;-]{0,30}") {
            let mysql = escaped(t(&text), Escape::Mysql);
            prop_assert_eq!(read_mysql_literal(&mysql), Some((text.clone(), "")));
        }

        #[test]
        fn a_template_never_panics(text in "(%%|%|\\{|\\}|header|record|footer|\\n|.){0,40}") {
            let open = Path::new("/a/b");
            let _ = parse_template(&text, Escape::Shell, &test_context(open, 1));
        }

        /// Whatever the text, each escape gives its format valid output.
        #[test]
        fn json_and_csv_read_back_the_value(text in "\\PC*") {
            let json = escaped(t(&text), Escape::Json);
            prop_assert_eq!(
                serde_json::from_str::<serde_json::Value>(&json).unwrap(),
                serde_json::Value::String(text.clone())
            );
            let csv = escaped(t(&text), Escape::Csv);
            let mut reader = csv::ReaderBuilder::new()
                .has_headers(false)
                .flexible(true)
                .from_reader(csv.as_bytes());
            if !text.is_empty() {
                let record = reader.records().next().unwrap().unwrap();
                prop_assert_eq!(record.get(0).unwrap(), text.as_str());
            }
        }

        #[test]
        fn a_line_never_breaks_and_a_url_is_plain_ascii(text in ".*\\n?.*") {
            prop_assert!(!escaped(t(&text), Escape::Line).contains('\n'));
            prop_assert!(escaped(t(&text), Escape::Url).bytes().all(|b| b.is_ascii_graphic()));
        }

        #[test]
        fn a_mask_never_panics(text in ".{0,60}") {
            let _ = Mask::parse(&text, Escape::Shell, Path::new("/a/b"), 1);
        }
    }
}
