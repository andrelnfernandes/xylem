//! The `.xlayout.toml` format, schema 1: what each kind of line of a positional
//! or delimited multi-record file holds (spec in `docs/plans/viewer-positional.md`).
//!
//! A layout that the user picks from the folder being browsed is as hostile as
//! the data it describes, so it is read strictly: unknown keys are refused, every
//! position and size is checked, and everything that grows with the file
//! (records, fields, values, conditions, regular expressions) has a limit.

use std::collections::{BTreeMap, HashMap, HashSet};

use regex::{Regex, RegexBuilder};
use serde::Deserialize;

use crate::charset::Charset;

/// The largest layout file read.
pub const MAX_LAYOUT_BYTES: u64 = 1 << 20;
/// Kinds of record a layout may declare.
pub const MAX_RECORD_TYPES: usize = 256;
/// The widest record a positional layout may declare.
pub const MAX_RECORD_BYTES: usize = 64 * 1024;
/// Fields of one record.
pub const MAX_FIELDS: usize = 1024;
/// Conditions of a `match`, a `batch_match` or a `[detect]`.
const MAX_CONDITIONS: usize = 16;
/// Values of a field's domain (`values`), and of an `in` list.
const MAX_VALUES: usize = 1024;
/// Characters of any text of the layout: names, descriptions, values.
const MAX_TEXT_CHARS: usize = 2048;
/// What a regular expression may compile to.
const REGEX_SIZE: usize = 1 << 16;
/// Decimal places of a `decimal` field: sums are kept at this scale.
pub const MAX_DECIMALS: u32 = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Chars,
    Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Format {
    Positional {
        unit: Unit,
    },
    Delimited {
        delimiter: char,
        /// `|a|b|`: the line starts and ends with the delimiter.
        outer: bool,
        /// Records the layout does not declare get a sheet of their own.
        undeclared: bool,
    },
}

/// A place in a line, 1-based: characters or bytes `start..=end`, or a field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum At {
    Pos { start: usize, end: usize },
    Col(usize),
}

#[derive(Debug, Clone)]
pub enum Test {
    Value(String),
    In(Vec<String>),
    Regex(Regex),
}

impl Test {
    pub fn accepts(&self, text: &str) -> bool {
        match self {
            Test::Value(v) => v == text,
            Test::In(list) => list.iter().any(|v| v == text),
            Test::Regex(re) => re.is_match(text),
        }
    }
}

/// What a place of a line must hold.
#[derive(Debug, Clone)]
pub struct Cond {
    pub at: At,
    pub test: Test,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Header,
    BatchHeader,
    Detail,
    BatchTrailer,
    Trailer,
}

impl Role {
    pub fn name(self) -> &'static str {
        match self {
            Role::Header => "header",
            Role::BatchHeader => "batch header",
            Role::Detail => "detail",
            Role::BatchTrailer => "batch trailer",
            Role::Trailer => "trailer",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Text,
    Code,
    Int,
    Decimal,
    Date,
    Time,
    Filler,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Text => "text",
            Kind::Code => "code",
            Kind::Int => "int",
            Kind::Decimal => "decimal",
            Kind::Date => "date",
            Kind::Time => "time",
            Kind::Filler => "filler",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trim {
    Right,
    Both,
    Left,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    None,
    Leading,
    Trailing,
    Overpunch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecimalSep {
    Implied,
    Dot,
    Comma,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateFormat {
    Yyyymmdd,
    Ddmmyyyy,
    Yymmdd,
    Ddmmyy,
    /// Year and day of the year (Julian date).
    Yyyyddd,
}

impl DateFormat {
    fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "YYYYMMDD" => DateFormat::Yyyymmdd,
            "DDMMYYYY" => DateFormat::Ddmmyyyy,
            "YYMMDD" => DateFormat::Yymmdd,
            "DDMMYY" => DateFormat::Ddmmyy,
            "YYYYDDD" => DateFormat::Yyyyddd,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            DateFormat::Yyyymmdd => "YYYYMMDD",
            DateFormat::Ddmmyyyy => "DDMMYYYY",
            DateFormat::Yymmdd => "YYMMDD",
            DateFormat::Ddmmyy => "DDMMYY",
            DateFormat::Yyyyddd => "YYYYDDD",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeFormat {
    Hhmmss,
    Hhmm,
}

/// Which records a check counts or sums: one id, or every id with a prefix (`1*`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdSel {
    Exact(String),
    Prefix(String),
}

impl IdSel {
    fn parse(text: &str) -> Self {
        match text.strip_suffix('*') {
            Some(prefix) => IdSel::Prefix(prefix.to_string()),
            None => IdSel::Exact(text.to_string()),
        }
    }

    pub fn matches(&self, id: &str) -> bool {
        match self {
            IdSel::Exact(e) => e == id,
            IdSel::Prefix(p) => id.starts_with(p.as_str()),
        }
    }
}

/// What the value of a field must agree with (`check`). Never stops the reading:
/// the result goes to the structure (`i`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Check {
    /// The lines of the file up to and including this one.
    TotalLines,
    /// The lines of these records in the whole file.
    RecordCount(Vec<IdSel>),
    /// The lines whose record code is the one in field `usize` of this line (SPED's 9900).
    CountByRecord(usize),
    /// The lines whose record code starts like this line's (SPED's x990 and 9990).
    BlockLines,
    /// The lines from the last batch header up to and including this one.
    BatchLines,
    /// The sum of fields `(record, field)`, in the whole file or in the batch.
    Sum {
        batch: bool,
        terms: Vec<(usize, usize)>,
    },
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub at: At,
    pub kind: Kind,
    pub trim: Trim,
    pub sign: Sign,
    pub decimals: u32,
    pub decimal_sep: DecimalSep,
    pub date: DateFormat,
    pub time: TimeFormat,
    pub desc: String,
    pub values: Vec<(String, String)>,
    pub check: Option<Check>,
}

#[derive(Debug, Clone)]
pub struct Record {
    pub id: String,
    pub name: String,
    pub role: Role,
    /// The record codes (`id` and `ids`) the discriminator gives for it.
    pub codes: Vec<String>,
    pub matches: Vec<Cond>,
    /// Conditions on the line of the batch header this record is in.
    pub batch_match: Vec<Cond>,
    /// The length this record's lines should have, without the line break.
    pub line_length: Option<usize>,
    pub min_length: Option<usize>,
    pub desc: String,
    pub fields: Vec<Field>,
}

impl Record {
    /// The fields shown in the grid.
    pub fn visible(&self, hide_filler: bool) -> impl Iterator<Item = (usize, &Field)> {
        self.fields
            .iter()
            .enumerate()
            .filter(move |(_, f)| !(hide_filler && f.kind == Kind::Filler))
    }
}

#[derive(Debug, Clone)]
pub struct Detect {
    pub extensions: Vec<String>,
    pub line_length: Option<usize>,
    pub first_line: Vec<Cond>,
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub id: String,
    pub name: String,
    pub description: String,
    pub country: String,
    pub category: String,
    pub spec_source: String,
    pub spec_version: String,
    pub spec_url: String,
    /// Checked against the official manual (`"verified"`) or not yet (`"draft"`).
    pub verified: bool,
    pub format: Format,
    /// `None`: the charset the viewer found for the file.
    pub encoding: Option<Charset>,
    pub hide_filler: bool,
    pub stop_after_trailer: bool,
    pub detect: Option<Detect>,
    pub discriminator: Option<At>,
    pub records: Vec<Record>,
}

impl Layout {
    pub fn is_delimited(&self) -> bool {
        matches!(self.format, Format::Delimited { .. })
    }

    pub fn record_by_id(&self, id: &str) -> Option<usize> {
        self.records.iter().position(|r| r.id == id)
    }
}

// What the file says, before it is checked.

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLayout {
    schema: i64,
    id: Option<String>,
    name: String,
    description: Option<String>,
    country: Option<String>,
    category: Option<String>,
    spec_source: Option<String>,
    spec_version: Option<String>,
    spec_url: Option<String>,
    status: Option<String>,
    format: Option<String>,
    encoding: Option<String>,
    offset_unit: Option<String>,
    line_length: Option<usize>,
    strict_positions: Option<bool>,
    hide_filler: Option<bool>,
    delimiter: Option<String>,
    outer_delimiters: Option<bool>,
    quote: Option<String>,
    decimal_sep: Option<String>,
    date_format: Option<String>,
    allow_undeclared_records: Option<bool>,
    stop_after_trailer: Option<bool>,
    detect: Option<RawDetect>,
    discriminator: Option<RawAt>,
    #[serde(default)]
    records: Vec<RawRecord>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAt {
    pos: Option<[usize; 2]>,
    start: Option<usize>,
    len: Option<usize>,
    col: Option<usize>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCond {
    pos: Option<[usize; 2]>,
    start: Option<usize>,
    len: Option<usize>,
    col: Option<usize>,
    value: Option<String>,
    #[serde(rename = "in")]
    any_of: Option<Vec<String>>,
    regex: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDetect {
    extensions: Option<Vec<String>>,
    line_length: Option<usize>,
    first_line: Option<Vec<RawCond>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRecord {
    id: Option<String>,
    ids: Option<Vec<String>>,
    name: String,
    role: Option<String>,
    desc: Option<String>,
    line_length: Option<usize>,
    min_length: Option<usize>,
    #[serde(rename = "match")]
    matches: Option<Vec<RawCond>>,
    batch_match: Option<Vec<RawCond>>,
    copybook: Option<String>,
    copybook_record: Option<String>,
    #[serde(default)]
    fields: Vec<RawField>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawField {
    name: Option<String>,
    pos: Option<[usize; 2]>,
    start: Option<usize>,
    len: Option<usize>,
    col: Option<usize>,
    #[serde(rename = "type")]
    kind: String,
    decimals: Option<u32>,
    decimal_sep: Option<String>,
    sign: Option<String>,
    format: Option<String>,
    trim: Option<String>,
    desc: Option<String>,
    values: Option<BTreeMap<String, String>>,
    check: Option<String>,
}

/// Reads a layout from the text of a `.xlayout.toml`.
pub fn parse(text: &str) -> Result<Layout, String> {
    if text.len() as u64 > MAX_LAYOUT_BYTES {
        return Err(format!(
            "the layout is larger than {} KiB",
            MAX_LAYOUT_BYTES >> 10
        ));
    }
    let raw: RawLayout = toml::from_str(text).map_err(|e| first_line(&e.to_string()))?;
    build(raw)
}

/// The first line of a parser's message, which may quote the file.
fn first_line(message: &str) -> String {
    message
        .lines()
        .next()
        .unwrap_or("")
        .chars()
        .take(200)
        .collect()
}

fn text(value: Option<String>, what: &str) -> Result<String, String> {
    let value = value.unwrap_or_default();
    check_text(&value, what)?;
    Ok(value)
}

fn check_text(value: &str, what: &str) -> Result<(), String> {
    if value.chars().count() > MAX_TEXT_CHARS {
        return Err(format!("{what} is longer than {MAX_TEXT_CHARS} characters"));
    }
    if value
        .chars()
        .any(|c| c.is_control() || crate::bidi::is_bidi(c))
    {
        return Err(format!("{what} has control characters"));
    }
    Ok(())
}

fn name_text(value: &str, what: &str) -> Result<(), String> {
    check_text(value, what)?;
    if value.trim().is_empty() {
        return Err(format!("{what} is empty"));
    }
    if value.chars().count() > 128 {
        return Err(format!("{what} is longer than 128 characters"));
    }
    Ok(())
}

fn build(raw: RawLayout) -> Result<Layout, String> {
    if raw.schema != 1 {
        return Err(format!(
            "schema {} is not known: this Xylem reads schema 1",
            raw.schema
        ));
    }
    name_text(&raw.name, "name")?;
    let verified = match raw.status.as_deref() {
        None | Some("draft") => false,
        Some("verified") => true,
        Some(other) => {
            return Err(format!(
                "status \"{}\" is not verified or draft",
                clip(other)
            ));
        }
    };
    let delimited = match raw.format.as_deref() {
        None | Some("positional") => false,
        Some("delimited") => true,
        Some(other) => {
            return Err(format!(
                "format \"{}\" is not positional or delimited",
                clip(other)
            ));
        }
    };
    let encoding = match raw.encoding.as_deref() {
        None | Some("auto") => None,
        Some("utf-8") => Some(Charset::Utf8),
        Some("cp1252") => Some(Charset::Cp1252),
        Some("cp850") => Some(Charset::Cp850),
        Some("cp437") => Some(Charset::Cp437),
        Some("utf-16le" | "utf-16be") => {
            return Err("UTF-16 files are not read with a layout yet".into());
        }
        Some(other) => return Err(format!("encoding \"{}\" is not known", clip(other))),
    };
    let unit = match raw.offset_unit.as_deref() {
        None | Some("chars") => Unit::Chars,
        Some("bytes") => Unit::Bytes,
        Some(other) => {
            return Err(format!(
                "offset_unit \"{}\" is not chars or bytes",
                clip(other)
            ));
        }
    };
    let decimal_sep = match raw.decimal_sep.as_deref() {
        None if delimited => DecimalSep::Comma,
        None => DecimalSep::Implied,
        Some(s) => parse_decimal_sep(s)?,
    };
    let date = match raw.date_format.as_deref() {
        None => DateFormat::Ddmmyyyy,
        Some(s) => DateFormat::parse(s)
            .ok_or_else(|| format!("date_format \"{}\" is not known", clip(s)))?,
    };
    let format = if delimited {
        if raw.offset_unit.is_some() || raw.line_length.is_some() || raw.strict_positions.is_some()
        {
            return Err(
                "offset_unit, line_length and strict_positions are for positional files".into(),
            );
        }
        let delimiter = match raw.delimiter.as_deref() {
            Some(d) if d.len() == 1 && d.is_ascii() && !d.contains(['\n', '\r']) => {
                d.chars().next().expect("one character")
            }
            Some(_) => return Err("delimiter must be one ASCII character".into()),
            None => return Err("a delimited layout needs a delimiter".into()),
        };
        match raw.quote.as_deref() {
            None | Some("") => {}
            Some(_) => return Err("quoted fields are not read yet: quote must be \"\"".into()),
        }
        Format::Delimited {
            delimiter,
            outer: raw.outer_delimiters.unwrap_or(false),
            undeclared: raw.allow_undeclared_records.unwrap_or(false),
        }
    } else {
        if raw.delimiter.is_some()
            || raw.outer_delimiters.is_some()
            || raw.quote.is_some()
            || raw.allow_undeclared_records.is_some()
        {
            return Err("delimiter, outer_delimiters, quote and allow_undeclared_records are for delimited files".into());
        }
        Format::Positional { unit }
    };
    let strict = !delimited && raw.strict_positions.unwrap_or(true);
    if let Some(len) = raw.line_length {
        check_width(len, "line_length")?;
    }

    let discriminator = raw
        .discriminator
        .map(|d| place(d.pos, d.start, d.len, d.col, delimited, "discriminator"))
        .transpose()?;

    if raw.records.is_empty() {
        return Err("the layout declares no record".into());
    }
    if raw.records.len() > MAX_RECORD_TYPES {
        return Err(format!("more than {MAX_RECORD_TYPES} records"));
    }
    let multi = raw.records.len() > 1;
    let defaults = Defaults {
        delimited,
        decimal_sep,
        date,
    };
    let mut records = Vec::with_capacity(raw.records.len());
    let mut ids = HashSet::new();
    let mut codes: HashMap<String, usize> = HashMap::new();
    let mut pending_checks = Vec::new();
    for (index, r) in raw.records.into_iter().enumerate() {
        let what = format!("record {}", index + 1);
        name_text(&r.name, &format!("{what}: name"))?;
        if r.copybook.is_some() || r.copybook_record.is_some() {
            return Err(format!(
                "{what}: COBOL copybooks are not read yet (copybook = ...)"
            ));
        }
        let id = match r.id {
            Some(id) => id,
            None if multi => return Err(format!("{what} has no id")),
            None => "1".to_string(),
        };
        name_text(&id, &format!("{what}: id"))?;
        if id.chars().count() > 32 {
            return Err(format!("{what}: id is longer than 32 characters"));
        }
        if !ids.insert(id.clone()) {
            return Err(format!("record id \"{}\" is used twice", clip(&id)));
        }
        let what = format!("record \"{}\"", clip(&id));
        let role = match r.role.as_deref() {
            None | Some("detail") => Role::Detail,
            Some("header") => Role::Header,
            Some("batch_header") => Role::BatchHeader,
            Some("batch_trailer") => Role::BatchTrailer,
            Some("trailer") => Role::Trailer,
            Some(other) => return Err(format!("{what}: role \"{}\" is not known", clip(other))),
        };
        let mut record_codes = Vec::new();
        if discriminator.is_some() {
            record_codes.push(id.clone());
            for code in r.ids.unwrap_or_default() {
                name_text(&code, &format!("{what}: ids"))?;
                if !record_codes.contains(&code) {
                    record_codes.push(code);
                }
            }
            if record_codes.len() > MAX_VALUES {
                return Err(format!("{what}: more than {MAX_VALUES} ids"));
            }
        } else if r.ids.is_some() {
            return Err(format!("{what}: ids needs a [discriminator]"));
        }
        let matches = conditions(r.matches, delimited, &format!("{what}: match"))?;
        let batch_match = conditions(r.batch_match, delimited, &format!("{what}: batch_match"))?;
        if role == Role::BatchHeader && !batch_match.is_empty() {
            return Err(format!("{what}: a batch header has no batch_match"));
        }
        if multi && discriminator.is_none() && matches.is_empty() {
            return Err(format!(
                "{what}: with more than one record, each needs a match or a [discriminator]"
            ));
        }
        // Two records the discriminator cannot tell apart need a match to do it.
        for code in &record_codes {
            if let Some(&other) = codes.get(code)
                && matches.is_empty()
                && batch_match.is_empty()
                && records
                    .get(other)
                    .is_some_and(|o: &Record| o.matches.is_empty() && o.batch_match.is_empty())
            {
                return Err(format!(
                    "{what}: code \"{}\" is also another record's, and no match tells them apart",
                    clip(code)
                ));
            }
            codes.entry(code.clone()).or_insert(index);
        }
        let line_length = r.line_length.or(raw.line_length);
        if delimited && (r.line_length.is_some() || r.min_length.is_some()) {
            return Err(format!(
                "{what}: line_length and min_length are for positional files"
            ));
        }
        if let Some(len) = line_length {
            check_width(len, &format!("{what}: line_length"))?;
        }
        if let Some(len) = r.min_length {
            check_width(len, &format!("{what}: min_length"))?;
        }
        let desc = text(r.desc, &format!("{what}: desc"))?;
        if r.fields.is_empty() {
            return Err(format!("{what} has no fields"));
        }
        if r.fields.len() > MAX_FIELDS {
            return Err(format!("{what}: more than {MAX_FIELDS} fields"));
        }
        let mut fields = Vec::with_capacity(r.fields.len());
        let mut names = HashSet::new();
        for (n, f) in r.fields.into_iter().enumerate() {
            let (field, check) = build_field(f, &defaults, &format!("{what}, field {}", n + 1))?;
            if !names.insert(field.name.to_lowercase()) {
                return Err(format!(
                    "{what}: field \"{}\" is there twice",
                    clip(&field.name)
                ));
            }
            if let Some(check) = check {
                pending_checks.push((index, fields.len(), check));
            }
            fields.push(field);
        }
        if strict {
            check_coverage(&fields, line_length, &what)?;
        } else if !delimited {
            check_inside(&fields, &what)?;
        } else {
            let mut cols = HashSet::new();
            for f in &fields {
                if let At::Col(c) = f.at
                    && !cols.insert(c)
                {
                    return Err(format!("{what}: column {c} is used by two fields"));
                }
            }
        }
        records.push(Record {
            id,
            name: r.name,
            role,
            codes: record_codes,
            matches,
            batch_match,
            line_length,
            min_length: r.min_length,
            desc,
            fields,
        });
    }

    for (record, field, check) in pending_checks {
        let resolved = resolve_check(&check, &records, record, field)?;
        records[record].fields[field].check = Some(resolved);
    }

    let detect = raw.detect.map(|d| build_detect(d, delimited)).transpose()?;
    Ok(Layout {
        id: text(raw.id, "id")?,
        name: raw.name,
        description: text(raw.description, "description")?,
        country: text(raw.country, "country")?,
        category: text(raw.category, "category")?,
        spec_source: text(raw.spec_source, "spec_source")?,
        spec_version: text(raw.spec_version, "spec_version")?,
        spec_url: text(raw.spec_url, "spec_url")?,
        verified,
        format,
        encoding,
        hide_filler: raw.hide_filler.unwrap_or(true),
        stop_after_trailer: raw.stop_after_trailer.unwrap_or(false),
        detect,
        discriminator,
        records,
    })
}

/// A short piece of what the file said, for a message.
fn clip(text: &str) -> String {
    let mut out: String = text
        .chars()
        .take(40)
        .map(|c| if c.is_control() { '·' } else { c })
        .collect();
    if text.chars().count() > 40 {
        out.push('…');
    }
    out
}

fn check_width(len: usize, what: &str) -> Result<(), String> {
    if len == 0 || len > MAX_RECORD_BYTES {
        return Err(format!("{what} must be from 1 to {MAX_RECORD_BYTES}"));
    }
    Ok(())
}

fn parse_decimal_sep(text: &str) -> Result<DecimalSep, String> {
    Ok(match text {
        "implied" => DecimalSep::Implied,
        "." => DecimalSep::Dot,
        "," => DecimalSep::Comma,
        other => {
            return Err(format!(
                "decimal_sep \"{}\" is not implied, . or ,",
                clip(other)
            ));
        }
    })
}

/// Where a field or a condition is: `pos`, `start` and `len`, or `col`.
fn place(
    pos: Option<[usize; 2]>,
    start: Option<usize>,
    len: Option<usize>,
    col: Option<usize>,
    delimited: bool,
    what: &str,
) -> Result<At, String> {
    if delimited {
        if pos.is_some() || start.is_some() || len.is_some() {
            return Err(format!("{what}: a delimited file uses col, not pos"));
        }
        return match col {
            Some(c) if (1..=MAX_FIELDS).contains(&c) => Ok(At::Col(c)),
            Some(_) => Err(format!("{what}: col must be from 1 to {MAX_FIELDS}")),
            None => Err(format!("{what} needs col")),
        };
    }
    if col.is_some() {
        return Err(format!("{what}: a positional file uses pos, not col"));
    }
    let (s, e) = match (pos, start, len) {
        (Some([s, e]), None, l) => {
            if let Some(l) = l
                && e.checked_sub(s).and_then(|d| d.checked_add(1)) != Some(l)
            {
                return Err(format!("{what}: pos [{s}, {e}] is not {l} long"));
            }
            (s, e)
        }
        (None, Some(s), Some(l)) if l > 0 => {
            let e = s
                .checked_add(l - 1)
                .ok_or_else(|| format!("{what}: start + len is too large"))?;
            (s, e)
        }
        (None, Some(_), _) => return Err(format!("{what}: start needs a len above 0")),
        (Some(_), Some(_), _) => return Err(format!("{what}: pos or start, not both")),
        (None, None, _) => return Err(format!("{what} needs pos or start and len")),
    };
    if s == 0 || e < s {
        return Err(format!(
            "{what}: pos [{s}, {e}] must have 1 <= start <= end"
        ));
    }
    if e > MAX_RECORD_BYTES {
        return Err(format!("{what}: ends after {MAX_RECORD_BYTES}"));
    }
    Ok(At::Pos { start: s, end: e })
}

fn conditions(raw: Option<Vec<RawCond>>, delimited: bool, what: &str) -> Result<Vec<Cond>, String> {
    let raw = raw.unwrap_or_default();
    if raw.len() > MAX_CONDITIONS {
        return Err(format!("{what}: more than {MAX_CONDITIONS} conditions"));
    }
    raw.into_iter()
        .map(|c| {
            let at = place(c.pos, c.start, c.len, c.col, delimited, what)?;
            let test = match (c.value, c.any_of, c.regex) {
                (Some(v), None, None) => {
                    check_text(&v, what)?;
                    Test::Value(v)
                }
                (None, Some(list), None) => {
                    if list.is_empty() || list.len() > MAX_VALUES {
                        return Err(format!("{what}: in must have 1 to {MAX_VALUES} values"));
                    }
                    for v in &list {
                        check_text(v, what)?;
                    }
                    Test::In(list)
                }
                (None, None, Some(re)) => {
                    check_text(&re, what)?;
                    let re = RegexBuilder::new(&re)
                        .size_limit(REGEX_SIZE)
                        .dfa_size_limit(REGEX_SIZE)
                        .nest_limit(32)
                        .build()
                        .map_err(|_| format!("{what}: the regex is not valid or too large"))?;
                    Test::Regex(re)
                }
                _ => {
                    return Err(format!(
                        "{what}: each condition has one of value, in and regex"
                    ));
                }
            };
            Ok(Cond { at, test })
        })
        .collect()
}

fn build_detect(d: RawDetect, delimited: bool) -> Result<Detect, String> {
    let extensions: Vec<String> = d
        .extensions
        .unwrap_or_default()
        .into_iter()
        .map(|e| e.trim_start_matches('.').to_ascii_lowercase())
        .collect();
    if extensions.len() > 64 {
        return Err("detect: more than 64 extensions".into());
    }
    for e in &extensions {
        name_text(e, "detect: extensions")?;
    }
    if let Some(len) = d.line_length {
        check_width(len, "detect: line_length")?;
    }
    let first_line = conditions(d.first_line, delimited, "detect: first_line")?;
    if first_line.is_empty() && d.line_length.is_none() {
        return Err("detect needs first_line or line_length: extensions alone tell nothing".into());
    }
    Ok(Detect {
        extensions,
        line_length: d.line_length,
        first_line,
    })
}

struct Defaults {
    delimited: bool,
    decimal_sep: DecimalSep,
    date: DateFormat,
}

fn build_field(
    f: RawField,
    defaults: &Defaults,
    what: &str,
) -> Result<(Field, Option<String>), String> {
    let kind = match f.kind.as_str() {
        "text" => Kind::Text,
        "code" => Kind::Code,
        "int" => Kind::Int,
        "decimal" => Kind::Decimal,
        "date" => Kind::Date,
        "time" => Kind::Time,
        "filler" => Kind::Filler,
        other => return Err(format!("{what}: type \"{}\" is not known", clip(other))),
    };
    let name = match f.name {
        Some(n) => n,
        None if kind == Kind::Filler => "filler".to_string(),
        None => return Err(format!("{what} has no name")),
    };
    name_text(&name, &format!("{what}: name"))?;
    let what = format!("{what} (\"{}\")", clip(&name));
    let at = place(f.pos, f.start, f.len, f.col, defaults.delimited, &what)?;
    let numeric = matches!(kind, Kind::Int | Kind::Decimal);
    if f.decimals.is_some() && kind != Kind::Decimal {
        return Err(format!("{what}: decimals is for decimal fields"));
    }
    let decimals = f.decimals.unwrap_or(0);
    if decimals > MAX_DECIMALS {
        return Err(format!("{what}: more than {MAX_DECIMALS} decimals"));
    }
    if f.decimal_sep.is_some() && kind != Kind::Decimal {
        return Err(format!("{what}: decimal_sep is for decimal fields"));
    }
    let decimal_sep = match f.decimal_sep.as_deref() {
        Some(s) => parse_decimal_sep(s)?,
        None => defaults.decimal_sep,
    };
    if kind == Kind::Decimal
        && decimal_sep == DecimalSep::Implied
        && decimals == 0
        && f.decimals.is_none()
    {
        return Err(format!(
            "{what}: an implied decimal point needs decimals (0 if there are none)"
        ));
    }
    let sign = match f.sign.as_deref() {
        None | Some("none") => Sign::None,
        Some(_) if !numeric => return Err(format!("{what}: sign is for int and decimal fields")),
        Some("leading") => Sign::Leading,
        Some("trailing") => Sign::Trailing,
        Some("overpunch") => Sign::Overpunch,
        Some(other) => return Err(format!("{what}: sign \"{}\" is not known", clip(other))),
    };
    let (date, time) = match (kind, f.format.as_deref()) {
        (Kind::Date, Some(s)) => (
            DateFormat::parse(s)
                .ok_or_else(|| format!("{what}: date format \"{}\" is not known", clip(s)))?,
            TimeFormat::Hhmmss,
        ),
        (Kind::Time, Some("HHMMSS") | None) => (defaults.date, TimeFormat::Hhmmss),
        (Kind::Time, Some("HHMM")) => (defaults.date, TimeFormat::Hhmm),
        (Kind::Time, Some(s)) => {
            return Err(format!("{what}: time format \"{}\" is not known", clip(s)));
        }
        (_, Some(_)) => return Err(format!("{what}: format is for date and time fields")),
        (_, None) => (defaults.date, TimeFormat::Hhmmss),
    };
    let trim = match f.trim.as_deref() {
        None => Trim::Right,
        Some(_) if !matches!(kind, Kind::Text | Kind::Code) => {
            return Err(format!("{what}: trim is for text and code fields"));
        }
        Some("right") => Trim::Right,
        Some("both") => Trim::Both,
        Some("left") => Trim::Left,
        Some("none") => Trim::None,
        Some(other) => return Err(format!("{what}: trim \"{}\" is not known", clip(other))),
    };
    let desc = text(f.desc, &format!("{what}: desc"))?;
    let values = f.values.unwrap_or_default();
    if values.len() > MAX_VALUES {
        return Err(format!("{what}: more than {MAX_VALUES} values"));
    }
    for (k, v) in &values {
        check_text(k, &format!("{what}: values"))?;
        check_text(v, &format!("{what}: values"))?;
    }
    if f.check.is_some() && !numeric {
        return Err(format!("{what}: check is for int and decimal fields"));
    }
    if let Some(c) = &f.check {
        check_text(c, &format!("{what}: check"))?;
    }
    Ok((
        Field {
            name,
            at,
            kind,
            trim,
            sign,
            decimals,
            decimal_sep,
            date,
            time,
            desc,
            values: values.into_iter().collect(),
            check: None,
        },
        f.check,
    ))
}

/// A strict positional record covers `1..=line_length` exactly: no gap and no
/// overlap, so that a typo in a position cannot go unseen.
fn check_coverage(fields: &[Field], line_length: Option<usize>, what: &str) -> Result<(), String> {
    let mut spans: Vec<(usize, usize, &str)> = fields
        .iter()
        .filter_map(|f| match f.at {
            At::Pos { start, end } => Some((start, end, f.name.as_str())),
            At::Col(_) => None,
        })
        .collect();
    spans.sort_unstable();
    let mut next = 1;
    for (start, end, name) in spans {
        if start < next {
            return Err(format!(
                "{what}: field \"{}\" at {start} overlaps the one before it",
                clip(name)
            ));
        }
        if start > next {
            return Err(format!(
                "{what}: nothing covers positions {next} to {} (strict_positions)",
                start - 1
            ));
        }
        next = end + 1;
    }
    if let Some(len) = line_length
        && next != len + 1
    {
        return Err(format!(
            "{what}: the fields cover 1 to {}, not the line_length {len}",
            next - 1
        ));
    }
    Ok(())
}

/// Without strict positions, fields may overlap or leave gaps, but stay inside the record.
fn check_inside(fields: &[Field], what: &str) -> Result<(), String> {
    for f in fields {
        if let At::Pos { end, .. } = f.at
            && end > MAX_RECORD_BYTES
        {
            return Err(format!("{what}: field \"{}\" ends too far", clip(&f.name)));
        }
    }
    Ok(())
}

fn resolve_check(
    text: &str,
    records: &[Record],
    record: usize,
    field: usize,
) -> Result<Check, String> {
    let what = format!(
        "record \"{}\", field \"{}\": check",
        clip(&records[record].id),
        clip(&records[record].fields[field].name)
    );
    let (kind, arg) = match text.split_once(':') {
        Some((k, a)) => (k, Some(a)),
        None => (text, None),
    };
    Ok(match (kind, arg) {
        ("total_lines", None) => Check::TotalLines,
        ("block_lines", None) => Check::BlockLines,
        ("batch_lines", None) => Check::BatchLines,
        ("record_count", Some(ids)) => {
            let sels: Vec<IdSel> = ids.split('+').map(IdSel::parse).collect();
            for sel in &sels {
                if !records.iter().any(|r| sel.matches(&r.id)) {
                    return Err(format!("{what}: no record has the id {}", clip(ids)));
                }
            }
            Check::RecordCount(sels)
        }
        ("count_by_record", Some(col)) => {
            let f = records[record]
                .fields
                .iter()
                .position(|f| f.name.eq_ignore_ascii_case(col))
                .ok_or_else(|| format!("{what}: this record has no field {}", clip(col)))?;
            Check::CountByRecord(f)
        }
        ("sum" | "batch_sum", Some(terms)) => {
            let mut resolved = Vec::new();
            for term in terms.split('+') {
                let (ids, name) = term
                    .rsplit_once('.')
                    .ok_or_else(|| format!("{what}: {} is not <record>.<field>", clip(term)))?;
                let sel = IdSel::parse(ids);
                let before = resolved.len();
                for (r, rec) in records.iter().enumerate() {
                    if !sel.matches(&rec.id) {
                        continue;
                    }
                    if let Some(f) = rec
                        .fields
                        .iter()
                        .position(|f| f.name.eq_ignore_ascii_case(name))
                    {
                        if !matches!(rec.fields[f].kind, Kind::Int | Kind::Decimal) {
                            return Err(format!("{what}: {} is not a number", clip(term)));
                        }
                        resolved.push((r, f));
                    }
                }
                if resolved.len() == before {
                    return Err(format!("{what}: no record has {}", clip(term)));
                }
            }
            Check::Sum {
                batch: kind == "batch_sum",
                terms: resolved,
            }
        }
        _ => return Err(format!("{what}: \"{}\" is not a known check", clip(text))),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const MINIMAL: &str = r#"
schema = 1
name = "Two records"
line_length = 10
[discriminator]
pos = [1, 1]
[[records]]
id = "0"
name = "Header"
role = "header"
fields = [
  { name = "kind", pos = [1, 1], type = "code" },
  { name = "date", pos = [2, 9], type = "date", format = "YYYYMMDD" },
  { pos = [10, 10], type = "filler" },
]
[[records]]
id = "1"
name = "Detail"
fields = [
  { name = "kind", pos = [1, 1], type = "code" },
  { name = "amount", pos = [2, 10], type = "decimal", decimals = 2 },
]
"#;

    #[test]
    fn a_minimal_layout_is_read() {
        let layout = parse(MINIMAL).unwrap();
        assert_eq!(layout.records.len(), 2);
        assert_eq!(layout.discriminator, Some(At::Pos { start: 1, end: 1 }));
        assert_eq!(layout.records[1].fields[1].decimals, 2);
        assert!(!layout.verified);
    }

    fn refused(text: &str) -> String {
        parse(text).expect_err("should be refused")
    }

    #[test]
    fn a_gap_or_an_overlap_is_refused() {
        let gap = MINIMAL.replace("pos = [2, 9]", "pos = [3, 9]");
        assert!(
            refused(&gap).contains("nothing covers positions 2 to 2"),
            "{}",
            refused(&gap)
        );
        let overlap = MINIMAL.replace(
            "pos = [2, 10], type = \"decimal\"",
            "pos = [1, 10], type = \"decimal\"",
        );
        assert!(refused(&overlap).contains("overlaps"));
        let short = MINIMAL.replace("line_length = 10", "line_length = 11");
        assert!(refused(&short).contains("not the line_length 11"));
    }

    #[test]
    fn a_len_that_disagrees_with_pos_is_refused() {
        let text = MINIMAL.replace("pos = [2, 9], type", "pos = [2, 9], len = 7, type");
        assert!(refused(&text).contains("is not 7 long"));
    }

    #[test]
    fn unknown_keys_and_schemas_are_refused() {
        assert!(refused(&MINIMAL.replace("schema = 1", "schema = 2")).contains("schema 2"));
        assert!(
            !refused(&MINIMAL.replace("name = \"Two records\"", "name = \"x\"\nexec = \"rm\""))
                .is_empty()
        );
        assert!(
            refused(&MINIMAL.replace("type = \"code\" }", "type = \"cmd\" }"))
                .contains("not known")
        );
    }

    #[test]
    fn control_and_bidi_characters_are_refused_in_names() {
        let text = MINIMAL.replace("name = \"Header\"", "name = \"Head\\u202Eer\"");
        assert!(refused(&text).contains("control characters"));
        let text = MINIMAL.replace("name = \"Header\"", "name = \"Head\\u001b[2Jer\"");
        assert!(refused(&text).contains("control characters"));
    }

    #[test]
    fn copybooks_wait_for_their_stage() {
        let text = MINIMAL.replace(
            "name = \"Detail\"",
            "name = \"Detail\"\ncopybook = \"../../etc/passwd\"",
        );
        assert!(refused(&text).contains("COBOL copybooks are not read yet"));
    }

    #[test]
    fn two_records_with_the_same_code_need_a_match() {
        let text = MINIMAL.replace(
            "id = \"1\"\nname = \"Detail\"",
            "id = \"1\"\nids = [\"0\"]\nname = \"Detail\"",
        );
        assert!(refused(&text).contains("no match tells them apart"));
    }

    #[test]
    fn checks_are_resolved_against_the_records() {
        let text = MINIMAL.replace(
            "{ name = \"amount\", pos = [2, 10], type = \"decimal\", decimals = 2 },",
            "{ name = \"amount\", pos = [2, 10], type = \"decimal\", decimals = 2, check = \"sum:1.amount\" },",
        );
        let layout = parse(&text).unwrap();
        assert_eq!(
            layout.records[1].fields[1].check,
            Some(Check::Sum {
                batch: false,
                terms: vec![(1, 1)]
            })
        );
        let wrong = text.replace("sum:1.amount", "sum:7.amount");
        assert!(refused(&wrong).contains("no record has 7.amount"));
        let text_field = text.replace("sum:1.amount", "sum:0.kind");
        assert!(refused(&text_field).contains("is not a number"));
    }

    #[test]
    fn a_huge_regex_is_refused() {
        let text = MINIMAL.replace(
            "[discriminator]",
            "[detect]\nfirst_line = [{ pos = [1, 1], regex = \"(a{1000}){1000}\" }]\n[discriminator]",
        );
        assert!(refused(&text).contains("not valid or too large"));
    }

    #[test]
    fn deep_nesting_does_not_overflow_the_stack() {
        let deep = format!(
            "schema = 1\nname = \"x\"\nrecords = {}{}",
            "[".repeat(50_000),
            "]".repeat(50_000)
        );
        assert!(parse(&deep).is_err());
        let tables = format!(
            "schema = 1\nname = \"x\"\nx = {}{}",
            "{a=".repeat(30_000),
            "}".repeat(30_000)
        );
        assert!(parse(&tables).is_err());
    }

    #[test]
    fn too_large_a_file_is_refused_before_parsing() {
        let big = format!("{MINIMAL}\n#{}", "x".repeat(MAX_LAYOUT_BYTES as usize));
        assert!(refused(&big).contains("larger than"));
    }

    proptest! {
        #[test]
        fn any_text_is_read_or_refused_without_panic(text in ".{0,400}") {
            let _ = parse(&text);
        }

        #[test]
        fn any_positions_are_read_or_refused_without_panic(
            a in 0usize..100_000, b in 0usize..100_000, len in 0usize..100_000, line in 0usize..100_000
        ) {
            let text = format!(
                "schema = 1\nname = \"p\"\nline_length = {line}\n[[records]]\nname = \"r\"\nfields = [{{ name = \"f\", pos = [{a}, {b}], len = {len}, type = \"int\" }}]\n"
            );
            let _ = parse(&text);
        }
    }
}
