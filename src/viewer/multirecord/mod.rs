//! Multi-record files, positional (CNAB, SDF) or delimited (SPED), read through
//! a layout (`.xlayout.toml`): one sheet per kind of record, in the order they
//! first appear, each row starting with its line in the file (`#line`).
//!
//! The file is as hostile as any other: lines are read with a limit, a line that
//! matches no record goes to a sheet of its own, and all that is counted (sheets,
//! record codes, checks) has a ceiling. One pass reads the whole file, keeping only
//! the rows of the sheet asked for, so that the counts and the checks of the
//! trailers cover all of it.

pub mod catalog;
pub mod decode;
pub mod xlayout;

use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::ops::ControlFlow;
use std::path::Path;

use crate::charset::{self, Charset};

use super::records::{Budget, Cell, End, MAX_COLS, Pace, RecordSink, Stoppable, Structure};
use super::tabular_loader::thousands;
use xlayout::{At, Check, Format, Kind, Layout, MAX_FIELDS, Role, Unit};

/// A line longer than this is not split into fields.
pub const MAX_LINE_BYTES: usize = 512 * 1024;
/// Sheets of one file: the records it declares, plus the undeclared ones of a
/// delimited file. Lines past that go to the sheet of lines with no record.
const MAX_SHEETS: usize = 256;
/// Record codes counted, for the checks that count by code (SPED's 9900).
const MAX_CODES: usize = 4096;
/// The longest record code a delimited file may give an undeclared record.
const MAX_CODE_CHARS: usize = 32;
/// Checks kept to be done at the end of the file.
const MAX_PENDING: usize = 20_000;
/// Failed checks listed in the structure.
const MAX_FAILURES_SHOWN: usize = 50;
/// The key of the sheet of lines that match no record.
pub const ORPHANS: &str = "!";

/// Which sheet to read: the one of the file's first line, or one by its key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    First,
    Key(String),
}

/// What a reading found besides the rows: the sheets of the file, by key, in
/// the order of the names given to the sink.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scan {
    pub end: End,
    pub keys: Vec<String>,
}

/// A line of the file, split as the layout needs.
struct LineView<'a> {
    raw: &'a [u8],
    text: &'a str,
    /// Byte offset of each character, when the text is not all ASCII.
    starts: Option<Vec<usize>>,
    /// The fields of a delimited line, or `None` when it is not delimited as the
    /// layout says (`outer` without the delimiter first).
    fields: Option<Vec<&'a str>>,
}

impl<'a> LineView<'a> {
    fn new(raw: &'a [u8], text: &'a str, layout: &Layout) -> Self {
        let starts = match layout.format {
            Format::Positional { unit: Unit::Chars } if !text.is_ascii() => {
                Some(text.char_indices().map(|(i, _)| i).collect())
            }
            _ => None,
        };
        let fields = match layout.format {
            Format::Delimited {
                delimiter, outer, ..
            } => split(text, delimiter, outer),
            Format::Positional { .. } => None,
        };
        LineView {
            raw,
            text,
            starts,
            fields,
        }
    }

    /// How long the line is, in the layout's unit.
    fn length(&self, layout: &Layout) -> usize {
        match layout.format {
            Format::Positional { unit: Unit::Bytes } => self.raw.len(),
            _ => self.starts.as_ref().map_or(self.text.len(), |s| s.len()),
        }
    }

    /// The text at a place: what of it the line has, `None` when it starts after the end.
    fn at(&self, at: At, layout: &Layout, charset: Charset) -> Option<Cow<'a, str>> {
        match at {
            At::Col(c) => self.fields.as_ref()?.get(c - 1).map(|s| Cow::Borrowed(*s)),
            At::Pos { start, end } => match layout.format {
                Format::Positional { unit: Unit::Bytes } => {
                    let bytes = self.raw.get(start - 1..end.min(self.raw.len()))?;
                    Some(Cow::Owned(charset::decode(bytes, charset)))
                }
                _ => match &self.starts {
                    None => self
                        .text
                        .get(start - 1..end.min(self.text.len()))
                        .map(Cow::Borrowed),
                    Some(starts) => {
                        let from = *starts.get(start - 1)?;
                        let to = starts.get(end).copied().unwrap_or(self.text.len());
                        Some(Cow::Borrowed(&self.text[from..to]))
                    }
                },
            },
        }
    }
}

/// The fields of a delimited line. `|a|b|` with `outer`: `a`, `b`.
fn split(text: &str, delimiter: char, outer: bool) -> Option<Vec<&str>> {
    let inner = if outer {
        let rest = text.strip_prefix(delimiter)?;
        rest.strip_suffix(delimiter).unwrap_or(rest)
    } else {
        text
    };
    Some(inner.splitn(MAX_FIELDS + 1, delimiter).collect())
}

fn holds(cond: &xlayout::Cond, line: &LineView<'_>, layout: &Layout, charset: Charset) -> bool {
    line.at(cond.at, layout, charset)
        .is_some_and(|text| cond.test.accepts(&text))
}

/// Whether a first line is what a `[detect]` asks: its length in the layout's
/// unit, and its conditions.
fn first_line_fits(
    layout: &Layout,
    raw: &[u8],
    charset: Charset,
    length: Option<usize>,
    conds: &[xlayout::Cond],
) -> bool {
    let text: Cow<'_, str> = match charset {
        Charset::Utf8 => String::from_utf8_lossy(raw),
        other => Cow::Owned(charset::decode(raw, other)),
    };
    let line = LineView::new(raw, &text, layout);
    if layout.is_delimited() && line.fields.is_none() {
        return false;
    }
    length.is_none_or(|n| line.length(layout) == n)
        && conds.iter().all(|c| holds(c, &line, layout, charset))
}

/// What a line is.
enum Kindred {
    Record(usize),
    /// A record of a delimited file that the layout does not declare, by its code.
    Undeclared(String),
    Orphan,
}

/// Tells which record a line is, the most specific first.
struct Classifier<'l> {
    layout: &'l Layout,
    by_code: HashMap<&'l str, Vec<usize>>,
    ordered: Vec<usize>,
}

impl<'l> Classifier<'l> {
    fn new(layout: &'l Layout) -> Self {
        let mut ordered: Vec<usize> = (0..layout.records.len()).collect();
        // Stable: among equally specific records, the order of the file.
        ordered.sort_by_key(|&r| {
            let rec = &layout.records[r];
            std::cmp::Reverse(rec.matches.len() + rec.batch_match.len())
        });
        let mut by_code: HashMap<&str, Vec<usize>> = HashMap::new();
        for &r in &ordered {
            for code in &layout.records[r].codes {
                by_code.entry(code.as_str()).or_default().push(r);
            }
        }
        Classifier {
            layout,
            by_code,
            ordered,
        }
    }

    fn classify(
        &self,
        line: &LineView<'_>,
        batch: Option<&LineView<'_>>,
        charset: Charset,
    ) -> (Kindred, Option<String>) {
        let layout = self.layout;
        if layout.is_delimited() && line.fields.is_none() {
            return (Kindred::Orphan, None);
        }
        let fits = |r: usize| {
            let rec = &layout.records[r];
            rec.matches.iter().all(|c| holds(c, line, layout, charset))
                && (rec.batch_match.is_empty()
                    || batch.is_some_and(|b| {
                        rec.batch_match.iter().all(|c| holds(c, b, layout, charset))
                    }))
        };
        let Some(disc) = layout.discriminator else {
            let found = self.ordered.iter().copied().find(|&r| fits(r));
            return (found.map_or(Kindred::Orphan, Kindred::Record), None);
        };
        let Some(code) = line.at(disc, layout, charset) else {
            return (Kindred::Orphan, None);
        };
        let code = code.into_owned();
        let found = self
            .by_code
            .get(code.as_str())
            .and_then(|candidates| candidates.iter().copied().find(|&r| fits(r)));
        if let Some(r) = found {
            return (Kindred::Record(r), Some(code));
        }
        let undeclared = matches!(
            layout.format,
            Format::Delimited {
                undeclared: true,
                ..
            }
        );
        if undeclared
            && !code.is_empty()
            && code.chars().count() <= MAX_CODE_CHARS
            && code.chars().all(|c| c.is_ascii_alphanumeric())
        {
            return (Kindred::Undeclared(code.clone()), Some(code));
        }
        (Kindred::Orphan, Some(code))
    }
}

/// The sheets found so far, in the order they first appear.
#[derive(Default)]
struct Sheets {
    keys: Vec<String>,
    names: Vec<String>,
    counts: Vec<u64>,
    index: HashMap<String, usize>,
}

impl Sheets {
    /// The sheet of a key, made when it is new and there is room.
    fn sheet(&mut self, key: &str, name: impl FnOnce() -> String) -> Option<usize> {
        if let Some(&i) = self.index.get(key) {
            return Some(i);
        }
        if self.keys.len() >= MAX_SHEETS && key != ORPHANS {
            return None;
        }
        let i = self.keys.len();
        self.keys.push(key.to_string());
        self.names.push(name());
        self.counts.push(0);
        self.index.insert(key.to_string(), i);
        Some(i)
    }
}

fn record_key(id: &str) -> String {
    format!("r:{id}")
}

fn code_key(code: &str) -> String {
    format!("c:{code}")
}

/// A check whose answer needs the whole file.
struct Pending {
    line: u64,
    record: usize,
    field: usize,
    stated: Option<i128>,
    what: Later,
}

enum Later {
    Records(Vec<xlayout::IdSel>),
    Code(String),
    Block(char),
    Sum(Vec<usize>),
}

#[derive(Default)]
struct Outcome {
    passed: u64,
    failed: u64,
    failures: Vec<String>,
    /// Checks that could not be done, and why.
    skipped: u64,
}

/// What the scan counts, for the sheets and the checks.
struct Tally {
    lines: u64,
    by_record: Vec<u64>,
    by_code: HashMap<String, u64>,
    codes_cut: bool,
    by_block: HashMap<char, u64>,
    batch_lines: u64,
    /// The `(record, field)` of every sum, and which of them a record feeds.
    terms: Vec<(usize, usize)>,
    feeds: Vec<Vec<(usize, usize)>>,
    sums: Vec<Option<i128>>,
    batch_sums: Vec<Option<i128>>,
    pending: Vec<Pending>,
    pending_cut: bool,
    outcome: Outcome,
}

impl Tally {
    fn new(layout: &Layout) -> Self {
        let mut terms: Vec<(usize, usize)> = Vec::new();
        for rec in &layout.records {
            for f in &rec.fields {
                if let Some(Check::Sum { terms: t, .. }) = &f.check {
                    for term in t {
                        if !terms.contains(term) {
                            terms.push(*term);
                        }
                    }
                }
            }
        }
        let mut feeds = vec![Vec::new(); layout.records.len()];
        for (i, &(r, f)) in terms.iter().enumerate() {
            feeds[r].push((f, i));
        }
        Tally {
            lines: 0,
            by_record: vec![0; layout.records.len()],
            by_code: HashMap::new(),
            codes_cut: false,
            by_block: HashMap::new(),
            batch_lines: 0,
            sums: vec![Some(0); terms.len()],
            batch_sums: vec![Some(0); terms.len()],
            terms,
            feeds,
            pending: Vec::new(),
            pending_cut: false,
            outcome: Outcome::default(),
        }
    }

    fn count_code(&mut self, code: &str) {
        if let Some(n) = self.by_code.get_mut(code) {
            *n += 1;
        } else if self.by_code.len() < MAX_CODES {
            self.by_code.insert(code.to_string(), 1);
        } else {
            self.codes_cut = true;
        }
        if let Some(c) = code.chars().next() {
            *self.by_block.entry(c).or_default() += 1;
        }
    }

    fn judge(
        &mut self,
        layout: &Layout,
        line: u64,
        record: usize,
        field: usize,
        stated: Option<i128>,
        counted: Option<i128>,
    ) {
        let rec = &layout.records[record];
        let f = &rec.fields[field];
        let ok = stated.is_some() && stated == counted;
        if ok {
            self.outcome.passed += 1;
            return;
        }
        self.outcome.failed += 1;
        if self.outcome.failures.len() < MAX_FAILURES_SHOWN {
            let shown = |v: Option<i128>| {
                v.map_or_else(|| "not a number".to_string(), |v| decode::show_scaled(f, v))
            };
            self.outcome.failures.push(format!(
                "line {}, {} {}: says {}, counted {}",
                thousands(line as usize),
                rec.id,
                f.name,
                shown(stated),
                counted.map_or_else(
                    || "too much to count".to_string(),
                    |v| decode::show_scaled(f, v)
                ),
            ));
        }
    }
}

/// A count at the checks' scale.
fn as_scaled(n: u64) -> Option<i128> {
    i128::from(n).checked_mul(10i128.pow(xlayout::MAX_DECIMALS))
}

/// How the lines of the sheet being read become rows.
enum Shape {
    Record(usize),
    Undeclared,
    Orphans,
}

struct Columns {
    shape: Shape,
    names: Vec<String>,
    /// For a delimited record: the columns of the line past the declared ones.
    extra_from: usize,
    sent: bool,
    cut: bool,
}

impl Columns {
    fn new(layout: &Layout, key: &str) -> Option<Self> {
        let mut names = vec!["#line".to_string()];
        let (shape, extra_from) = if key == ORPHANS {
            names.push("line".into());
            (Shape::Orphans, 0)
        } else if let Some(id) = key.strip_prefix("r:") {
            let r = layout.record_by_id(id)?;
            let rec = &layout.records[r];
            names.extend(rec.visible(layout.hide_filler).map(|(_, f)| f.name.clone()));
            let last = rec
                .fields
                .iter()
                .filter_map(|f| match f.at {
                    At::Col(c) => Some(c),
                    At::Pos { .. } => None,
                })
                .max()
                .unwrap_or(0);
            (Shape::Record(r), last)
        } else {
            key.strip_prefix("c:")?;
            names.push("REG".into());
            (Shape::Undeclared, 1)
        };
        let cut = names.len() > MAX_COLS;
        names.truncate(MAX_COLS);
        Some(Columns {
            shape,
            names,
            extra_from,
            sent: false,
            cut,
        })
    }

    /// Makes room for a delimited line of `fields` fields. True if columns were added.
    fn grow(&mut self, layout: &Layout, fields: usize) -> bool {
        let wanted = match self.shape {
            Shape::Orphans => return false,
            Shape::Record(_) if !layout.is_delimited() => return false,
            Shape::Record(_) | Shape::Undeclared => fields.saturating_sub(self.extra_from),
        };
        let have = self.names.len() - self.base_len(layout);
        if wanted <= have {
            return false;
        }
        for n in have..wanted {
            if self.names.len() >= MAX_COLS {
                self.cut = true;
                break;
            }
            self.names
                .push(format!("CAMPO_{:02}", self.extra_from + n + 1));
        }
        true
    }

    fn base_len(&self, layout: &Layout) -> usize {
        match self.shape {
            Shape::Orphans => 2,
            Shape::Undeclared => 2,
            Shape::Record(r) => {
                (1 + layout.records[r].visible(layout.hide_filler).count()).min(MAX_COLS)
            }
        }
    }
}

/// Reads a line, keeping at most `max` bytes of it. Returns the bytes consumed
/// (0 at the end of the file) and whether the line was longer than `max`.
fn read_line(
    reader: &mut impl BufRead,
    buf: &mut Vec<u8>,
    max: usize,
) -> std::io::Result<(usize, bool)> {
    let mut total = 0;
    let mut long = false;
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return Ok((total, long));
        }
        let (take, done) = match available.iter().position(|&b| b == b'\n') {
            Some(i) => (i + 1, true),
            None => (available.len(), false),
        };
        let room = max.saturating_sub(buf.len());
        buf.extend_from_slice(&available[..take.min(room)]);
        if take > room {
            long = true;
        }
        total += take;
        reader.consume(take);
        if done {
            return Ok((total, long));
        }
    }
}

/// The charset of the file: the layout's, else the viewer's, else what the start says.
fn file_charset(path: &Path, layout: &Layout, viewer: Option<Charset>) -> Result<Charset, String> {
    if let Some(cs) = layout.encoding.or(viewer) {
        return Ok(cs);
    }
    let mut start = Vec::new();
    File::open(path)
        .and_then(|f| f.take(64 * 1024).read_to_end(&mut start))
        .map_err(|e| format!("Failed to open: {e}"))?;
    Ok(charset::detect(&start, None).charset)
}

/// Options of a reading.
pub struct Reading<'a> {
    pub target: &'a Target,
    /// The charset the viewer shows the file in, used when the layout says `auto`.
    pub charset: Option<Charset>,
    /// Go on to the end of the file once the sink has its rows, for the counts
    /// and the checks. A Batch wants only the rows.
    pub count_all: bool,
}

/// Reads the file through the layout, giving the sink the rows of one sheet.
pub fn read(
    path: &Path,
    layout: &Layout,
    reading: Reading<'_>,
    budget: Budget,
    sink: &mut dyn RecordSink,
) -> Result<Scan, String> {
    let charset = file_charset(path, layout, reading.charset)?;
    if matches!(charset, Charset::Utf16Le | Charset::Utf16Be) {
        return Err("UTF-16 files are not read with a layout yet".into());
    }
    let file = File::open(path).map_err(|e| format!("Failed to open: {e}"))?;
    let size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut reader = BufReader::with_capacity(64 * 1024, Stoppable::new(file, budget.clone()));
    let classifier = Classifier::new(layout);
    let mut sheets = Sheets::default();
    let mut tally = Tally::new(layout);
    let mut pace = Pace::new(budget.clone());
    let mut target: Option<String> = match reading.target {
        Target::First => None,
        Target::Key(k) => Some(k.clone()),
    };
    let mut columns = target.as_deref().and_then(|k| Columns::new(layout, k));
    let mut delivering = true;
    let mut refused = false;
    let mut end = End::Complete;
    let mut buf = Vec::new();
    let mut consumed: u64 = 0;
    let mut line_no: u64 = 0;
    let mut batch: Option<String> = None;
    let mut after_trailer: Option<u64> = None;
    // What the sheet being read says of its lines.
    let mut wrong_length = 0u64;
    let mut invalid = 0u64;
    let mut too_long = 0u64;

    loop {
        if pace.over() {
            end = End::OutOfTime;
            break;
        }
        buf.clear();
        let (n, long) = match read_line(&mut reader, &mut buf, MAX_LINE_BYTES) {
            Ok(r) => r,
            Err(_) if budget.over() => {
                end = End::OutOfTime;
                break;
            }
            Err(e) => return Err(format!("Read error: {e}")),
        };
        if n == 0 {
            break;
        }
        consumed += n as u64;
        line_no += 1;
        if !long {
            if buf.last() == Some(&b'\n') {
                buf.pop();
            }
            if buf.last() == Some(&b'\r') {
                buf.pop();
            }
        }
        let mut raw: &[u8] = &buf;
        if line_no == 1 && charset == Charset::Utf8 {
            raw = raw.strip_prefix(&[0xEF, 0xBB, 0xBF][..]).unwrap_or(raw);
        }
        // DOS ended text files with Ctrl+Z (Clipper's SDF among them).
        if raw.is_empty() || raw == [0x1A] {
            continue;
        }
        let decoded: Cow<'_, str> = match charset {
            Charset::Utf8 => String::from_utf8_lossy(raw),
            other => Cow::Owned(charset::decode(raw, other)),
        };
        tally.lines += 1;
        let line = LineView::new(raw, &decoded, layout);
        let batch_view = batch
            .as_deref()
            .map(|b| LineView::new(b.as_bytes(), b, layout));
        let (kind, code) = if long {
            (Kindred::Orphan, None)
        } else {
            classifier.classify(&line, batch_view.as_ref(), charset)
        };
        if long {
            too_long += 1;
        }
        if let Some(code) = &code
            && !matches!(kind, Kindred::Orphan)
        {
            tally.count_code(code);
        }

        let key = match &kind {
            Kindred::Record(r) => record_key(&layout.records[*r].id),
            Kindred::Undeclared(code) => code_key(code),
            Kindred::Orphan => ORPHANS.to_string(),
        };
        let sheet = sheets
            .sheet(&key, || match &kind {
                Kindred::Record(r) => {
                    let rec = &layout.records[*r];
                    format!("{} - {}", rec.id, rec.name)
                }
                Kindred::Undeclared(code) => code.clone(),
                Kindred::Orphan => "! No record type".to_string(),
            })
            .unwrap_or_else(|| {
                sheets
                    .sheet(ORPHANS, || "! No record type".to_string())
                    .expect("the orphans always have a sheet")
            });
        sheets.counts[sheet] += 1;
        let key = sheets.keys[sheet].clone();
        if target.is_none() {
            target = Some(key.clone());
            columns = Columns::new(layout, &key);
        }

        // Counts and checks, over every line.
        if let Kindred::Record(r) = kind {
            let rec = &layout.records[r];
            tally.by_record[r] += 1;
            if rec.role == Role::BatchHeader {
                tally.batch_lines = 0;
                tally.batch_sums.fill(Some(0));
            }
            tally.batch_lines += 1;
            for &(f, term) in &tally.feeds[r].clone() {
                let value = line
                    .at(rec.fields[f].at, layout, charset)
                    .and_then(|t| decode::scaled(&rec.fields[f], &t));
                let add = |s: &mut Option<i128>| {
                    *s = match (*s, value) {
                        (Some(a), Some(v)) => a.checked_add(v),
                        _ => None,
                    }
                };
                add(&mut tally.sums[term]);
                add(&mut tally.batch_sums[term]);
            }
            for (f, field) in rec.fields.iter().enumerate() {
                let Some(check) = &field.check else { continue };
                let stated = line
                    .at(field.at, layout, charset)
                    .and_then(|t| decode::scaled(field, &t));
                let later = match check {
                    Check::TotalLines => {
                        tally.judge(layout, line_no, r, f, stated, as_scaled(tally.lines));
                        None
                    }
                    Check::BatchLines => {
                        tally.judge(layout, line_no, r, f, stated, as_scaled(tally.batch_lines));
                        None
                    }
                    Check::Sum { batch: true, terms } => {
                        let total = terms.iter().try_fold(0i128, |acc, t| {
                            let i = tally.terms.iter().position(|x| x == t)?;
                            acc.checked_add(tally.batch_sums[i]?)
                        });
                        tally.judge(layout, line_no, r, f, stated, total);
                        None
                    }
                    Check::Sum {
                        batch: false,
                        terms,
                    } => Some(Later::Sum(
                        terms
                            .iter()
                            .filter_map(|t| tally.terms.iter().position(|x| x == t))
                            .collect(),
                    )),
                    Check::RecordCount(sels) => Some(Later::Records(sels.clone())),
                    Check::CountByRecord(col) => {
                        let named = line
                            .at(rec.fields[*col].at, layout, charset)
                            .map(|t| t.trim().to_string())
                            .unwrap_or_default();
                        Some(Later::Code(named))
                    }
                    Check::BlockLines => match code.as_deref().and_then(|c| c.chars().next()) {
                        Some(c) => Some(Later::Block(c)),
                        None => {
                            tally.outcome.skipped += 1;
                            None
                        }
                    },
                };
                if let Some(what) = later {
                    if tally.pending.len() < MAX_PENDING {
                        tally.pending.push(Pending {
                            line: line_no,
                            record: r,
                            field: f,
                            stated,
                            what,
                        });
                    } else {
                        tally.pending_cut = true;
                    }
                }
            }
            match rec.role {
                Role::BatchHeader => batch = Some(decoded.clone().into_owned()),
                Role::BatchTrailer | Role::Trailer => batch = None,
                _ => {}
            }
            if rec.role == Role::Trailer && layout.stop_after_trailer {
                after_trailer = Some(size.saturating_sub(consumed));
            }
        }

        // The row, when the line is of the sheet being read.
        if delivering
            && target.as_deref() == Some(key.as_str())
            && let Some(cols) = &mut columns
        {
            let mut cells: Vec<Cell<'_>> =
                vec![Cell::Int(i64::try_from(line_no).unwrap_or(i64::MAX))];
            match (&cols.shape, &kind) {
                (Shape::Record(_), Kindred::Record(r)) => {
                    let rec = &layout.records[*r];
                    if let Some(want) = rec.line_length {
                        let len = line.length(layout);
                        if len != want && rec.min_length.is_none_or(|m| len < m) {
                            wrong_length += 1;
                        }
                    }
                    for (_, field) in rec.visible(layout.hide_filler).take(MAX_COLS - 1) {
                        let (cell, bad) = match line.at(field.at, layout, charset) {
                            Some(t) => decode::cell(field, &t),
                            None => (Cell::Null, false),
                        };
                        invalid += u64::from(bad);
                        cells.push(cell);
                    }
                }
                (Shape::Undeclared, _) => {}
                (Shape::Orphans, _) => {
                    let mut text = decoded.as_ref();
                    if long {
                        text = &text[..text.floor_char_boundary(4096)];
                    }
                    cells.push(Cell::Text(Cow::Owned(text.to_string())));
                }
                _ => {}
            }
            if let Some(fields) = &line.fields
                && layout.is_delimited()
                && !matches!(cols.shape, Shape::Orphans)
            {
                if cols.grow(layout, fields.len()) && cols.sent {
                    sink.columns(&cols.names);
                }
                let from = match cols.shape {
                    Shape::Undeclared => 0,
                    _ => cols.extra_from,
                };
                for f in fields.iter().skip(from) {
                    if cells.len() >= cols.names.len() {
                        break;
                    }
                    cells.push(if f.is_empty() {
                        Cell::Null
                    } else {
                        Cell::Text(Cow::Borrowed(*f))
                    });
                }
            }
            if !cols.sent {
                send_columns(layout, cols, sink);
            }
            if sink.record(&cells) == ControlFlow::Break(()) {
                delivering = false;
                refused = true;
                if !reading.count_all {
                    end = End::Refused;
                    break;
                }
            }
        }
        if after_trailer.is_some() {
            break;
        }
    }

    // Lines of the sheet asked for may never have come.
    if let Some(cols) = &mut columns
        && !cols.sent
    {
        send_columns(layout, cols, sink);
    }
    let complete = end != End::OutOfTime && !(refused && !reading.count_all);
    if complete {
        finish_checks(layout, &mut tally);
    }
    let partial = if complete { "" } else { "+" };
    let names: Vec<String> = sheets
        .names
        .iter()
        .zip(&sheets.counts)
        .map(|(name, n)| format!("{name} ({}{partial})", thousands(*n as usize)))
        .collect();
    let current = target
        .as_deref()
        .and_then(|t| sheets.index.get(t).copied())
        .unwrap_or(0);
    sink.sheets(&names, current);

    let outcome = &tally.outcome;
    if outcome.failed > 0 {
        let word = if outcome.failed == 1 {
            "check"
        } else {
            "checks"
        };
        sink.notice(&format!(
            "{} {word} failed: i shows",
            thousands(outcome.failed as usize)
        ));
    }
    let facts = Facts {
        complete,
        lines: line_no,
        tally: &tally,
        wrong_length,
        invalid,
        too_long,
        after_trailer,
    };
    sink.structure(structure(layout, columns.as_ref(), &facts));
    Ok(Scan {
        end,
        keys: sheets.keys,
    })
}

fn send_columns(layout: &Layout, cols: &mut Columns, sink: &mut dyn RecordSink) {
    if cols.cut {
        sink.columns_cut();
    }
    sink.columns(&cols.names);
    if let Shape::Record(r) = cols.shape {
        let mut domains = vec![Vec::new()];
        domains.extend(
            layout.records[r]
                .visible(layout.hide_filler)
                .take(MAX_COLS - 1)
                .map(|(_, f)| f.values.clone()),
        );
        if domains.iter().any(|d| !d.is_empty()) {
            sink.domains(&domains);
        }
    }
    cols.sent = true;
}

/// The checks that needed the whole file.
fn finish_checks(layout: &Layout, tally: &mut Tally) {
    let pending = std::mem::take(&mut tally.pending);
    for p in pending {
        let counted = match &p.what {
            Later::Records(sels) => {
                let n: u64 = layout
                    .records
                    .iter()
                    .zip(&tally.by_record)
                    .filter(|(r, _)| sels.iter().any(|s| s.matches(&r.id)))
                    .map(|(_, n)| n)
                    .sum();
                as_scaled(n)
            }
            Later::Code(code) => match tally.by_code.get(code.as_str()) {
                Some(&n) => as_scaled(n),
                None if tally.codes_cut => {
                    tally.outcome.skipped += 1;
                    continue;
                }
                None => as_scaled(0),
            },
            Later::Block(c) => as_scaled(tally.by_block.get(c).copied().unwrap_or(0)),
            Later::Sum(terms) => terms.iter().try_fold(0i128, |acc, &i| {
                acc.checked_add(tally.sums.get(i).copied().flatten()?)
            }),
        };
        tally.judge(layout, p.line, p.record, p.field, p.stated, counted);
    }
    if tally.pending_cut {
        tally.outcome.skipped += 1;
    }
}

struct Facts<'a> {
    complete: bool,
    lines: u64,
    tally: &'a Tally,
    wrong_length: u64,
    invalid: u64,
    too_long: u64,
    after_trailer: Option<u64>,
}

fn values_text(values: &[(String, String)]) -> String {
    let mut out = String::new();
    for (v, m) in values {
        if !out.is_empty() {
            out.push_str("; ");
        }
        out.push_str(v);
        out.push_str(" = ");
        out.push_str(m);
        if out.chars().count() > 200 {
            out = out.chars().take(200).collect();
            out.push('…');
            break;
        }
    }
    out
}

/// What `i` shows: the fields of the record being read, then what was found.
fn structure(layout: &Layout, cols: Option<&Columns>, facts: &Facts<'_>) -> Structure {
    let mut about = Vec::new();
    let status = if layout.verified {
        "checked against its manual"
    } else {
        "draft: not yet checked against its manual"
    };
    about.push(format!("Layout: {} ({status})", layout.name));
    if !layout.description.is_empty() {
        about.push(layout.description.clone());
    }
    if !layout.spec_source.is_empty() {
        about.push(
            format!("From: {} {}", layout.spec_source, layout.spec_version)
                .trim_end()
                .to_string(),
        );
        if !layout.spec_url.is_empty() {
            about.push(format!("Manual: {}", layout.spec_url));
        }
    }
    let mut titles: Vec<String> = Vec::new();
    let mut columns = Vec::new();
    match cols.map(|c| &c.shape) {
        Some(Shape::Record(r)) => {
            let rec = &layout.records[*r];
            about.push(format!(
                "Record {}: {} ({})",
                rec.id,
                rec.name,
                rec.role.name()
            ));
            if !rec.desc.is_empty() {
                about.push(rec.desc.clone());
            }
            if let Some(len) = rec.line_length
                && facts.wrong_length > 0
            {
                about.push(format!(
                    "Lines of this record that are not {len} long: {}",
                    thousands(facts.wrong_length as usize)
                ));
            }
            if facts.invalid > 0 {
                about.push(format!(
                    "Values that are not what the layout says (shown as written): {}",
                    thousands(facts.invalid as usize)
                ));
            }
            if layout.hide_filler && rec.fields.iter().any(|f| f.kind == Kind::Filler) {
                about.push("Filler fields are listed here, not in the grid".into());
            }
            titles = if layout.is_delimited() {
                vec!["Field", "Column", "Type", "Description", "Values"]
            } else {
                vec!["Field", "Position", "Size", "Type", "Description", "Values"]
            }
            .into_iter()
            .map(String::from)
            .collect();
            for f in rec.fields.iter().take(MAX_COLS) {
                let mut kind = f.kind.name().to_string();
                if f.kind == Kind::Decimal && f.decimals > 0 {
                    kind.push_str(&format!(" ({})", f.decimals));
                }
                if f.kind == Kind::Date {
                    kind.push_str(&format!(" {}", f.date.name()));
                }
                let mut row = vec![f.name.clone()];
                match f.at {
                    At::Pos { start, end } => {
                        row.push(format!("{start}..{end}"));
                        row.push((end - start + 1).to_string());
                    }
                    At::Col(c) => row.push(c.to_string()),
                }
                row.push(kind);
                row.push(f.desc.clone());
                row.push(values_text(&f.values));
                columns.push(row);
            }
        }
        Some(Shape::Undeclared) => {
            about.push("A record the layout does not declare: its fields are numbered".into());
        }
        Some(Shape::Orphans) => {
            about.push("Lines that match no record of the layout".into());
            if facts.too_long > 0 {
                about.push(format!(
                    "Lines longer than {} KiB, not split into fields: {}",
                    MAX_LINE_BYTES >> 10,
                    thousands(facts.too_long as usize)
                ));
            }
        }
        None => {}
    }
    if facts.complete {
        about.push(format!(
            "Lines in the file: {}",
            thousands(facts.lines as usize)
        ));
    } else {
        about.push(format!(
            "Reading stopped at line {}: counts are partial and the checks were not done",
            thousands(facts.lines as usize)
        ));
    }
    let o = &facts.tally.outcome;
    if facts.complete && (o.passed + o.failed + o.skipped) > 0 {
        let mut line = format!(
            "Checks of totals: {} passed, {} failed",
            thousands(o.passed as usize),
            thousands(o.failed as usize)
        );
        if o.skipped > 0 {
            line.push_str(&format!(", {} not done", thousands(o.skipped as usize)));
        }
        about.push(line);
        for f in &o.failures {
            about.push(format!("  Failed: {f}"));
        }
        if o.failed as usize > o.failures.len() {
            about.push(format!(
                "  ...and {} more",
                thousands(o.failed as usize - o.failures.len())
            ));
        }
    }
    if let Some(rest) = facts.after_trailer
        && rest > 0
    {
        about.push(format!(
            "{} bytes after the closing record were not read (signed files carry their signature there)",
            thousands(rest as usize)
        ));
    }
    Structure {
        titles,
        columns,
        about,
    }
}

#[cfg(test)]
pub(crate) mod tests;
