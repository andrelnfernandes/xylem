//! JSON, YAML and TOML as a tree that folds (docs/plans/viewer-tree.md).
//!
//! The file is hostile: nothing here recurses (the descent is a stack in a
//! `Vec`, with a limit on its depth), a YAML alias is never expanded (so a
//! "billion laughs" costs what its text costs), a number is shown as it is in
//! the file, never through `f64`, and a key that appears twice in one object
//! is shown twice, marked, since parsers disagree on which one wins.

use std::collections::HashSet;
use std::time::{Duration, Instant};

/// The formats, by the name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    /// JSON with comments and trailing commas (`.jsonc`, `.json5`).
    JsonC,
    Toml,
    Yaml,
}

pub fn format_of(path: &std::path::Path) -> Option<Format> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    Some(match ext.as_str() {
        "json" | "geojson" | "har" | "webmanifest" | "topojson" => Format::Json,
        "jsonc" | "json5" => Format::JsonC,
        "toml" => Format::Toml,
        "yaml" | "yml" => Format::Yaml,
        "lock" if path.file_name().is_some_and(|n| n == "Cargo.lock") => Format::Toml,
        _ => return None,
    })
}

impl Format {
    pub fn name(self) -> &'static str {
        match self {
            Format::Json => "JSON",
            Format::JsonC => "JSON with comments",
            Format::Toml => "TOML",
            Format::Yaml => "YAML",
        }
    }
}

/// Most levels of nesting.
pub const MAX_DEPTH: usize = 512;
/// Most nodes of a tree: past them it is cut, and says so.
pub const MAX_NODES: usize = 2_000_000;
/// Characters of a string kept for the screen: the rest is `…`.
const MAX_VALUE: usize = 4096;
/// A tree opens all open up to this many nodes; past it, two levels.
const OPEN_ALL: usize = 2000;
const NONE: u32 = u32::MAX;

/// What a node holds.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Object(u32),
    Array(u32),
    Str(Box<str>),
    /// A number, a date or a plain scalar, as the file writes it.
    Raw(Box<str>),
    Bool(bool),
    Null,
    /// A YAML alias, never expanded: `*name`.
    Alias(Box<str>),
}

#[derive(Debug, Clone)]
pub struct Node {
    pub key: Option<Box<str>>,
    pub value: Value,
    parent: u32,
    first: u32,
    next: u32,
    last: u32,
    pub depth: u16,
    /// The line of the source it starts on (from 0).
    pub line: u32,
    /// Its key is already in the same object.
    pub dup: bool,
    /// A YAML anchor (`&name`) or tag (`!Ref`) before the value.
    pub mark: Option<Box<str>>,
}

/// A tree of nodes in one `Vec`, and which of them are open.
#[derive(Debug, Clone)]
pub struct Tree {
    pub format: Format,
    nodes: Vec<Node>,
    open: Vec<bool>,
    /// The nodes on screen, in order: those under an open node.
    rows: Vec<u32>,
    pub dups: usize,
    /// It stopped at a limit (nodes or time): only the start is in it.
    pub cut: bool,
    /// The first node only holds the others (YAML's one document): it is not
    /// shown, nor in the paths.
    skip_root: bool,
}

/// Builds a tree: nodes are added under the one on top of the stack.
struct Builder {
    nodes: Vec<Node>,
    stack: Vec<(u32, HashSet<Box<str>>)>,
    dups: usize,
    start: Instant,
    time: Duration,
    cut: bool,
}

impl Builder {
    fn new(time: Duration) -> Self {
        Builder {
            nodes: Vec::new(),
            stack: Vec::new(),
            dups: 0,
            start: Instant::now(),
            time,
            cut: false,
        }
    }

    /// Room for one more node, and time: else the tree is cut here.
    fn room(&mut self) -> bool {
        if self.nodes.len() >= MAX_NODES
            || (self.nodes.len().is_multiple_of(4096) && self.start.elapsed() >= self.time)
        {
            self.cut = true;
        }
        !self.cut
    }

    fn add(
        &mut self,
        key: Option<Box<str>>,
        value: Value,
        line: usize,
        mark: Option<Box<str>>,
    ) -> Result<u32, String> {
        let id = self.nodes.len() as u32;
        let (parent, depth) = match self.stack.last() {
            Some(&(p, _)) => (p, self.nodes[p as usize].depth + 1),
            None => (NONE, 0),
        };
        let mut dup = false;
        if let (Some(k), Some((_, seen))) = (&key, self.stack.last_mut()) {
            dup = !seen.insert(k.clone());
            if dup {
                self.dups += 1;
            }
        }
        if parent != NONE {
            let p = &mut self.nodes[parent as usize];
            if p.first == NONE {
                p.first = id;
            } else {
                let last = p.last;
                self.nodes[last as usize].next = id;
            }
            let p = &mut self.nodes[parent as usize];
            p.last = id;
            match &mut p.value {
                Value::Object(n) | Value::Array(n) => *n += 1,
                _ => {}
            }
        }
        self.nodes.push(Node {
            key,
            value,
            parent,
            first: NONE,
            next: NONE,
            last: NONE,
            depth,
            line: line.min(u32::MAX as usize) as u32,
            dup,
            mark,
        });
        Ok(id)
    }

    /// A container: added, and the next nodes go under it.
    fn open(
        &mut self,
        key: Option<Box<str>>,
        value: Value,
        line: usize,
        mark: Option<Box<str>>,
    ) -> Result<(), String> {
        if self.stack.len() >= MAX_DEPTH {
            return Err(format!("Nested deeper than {MAX_DEPTH} levels"));
        }
        let id = self.add(key, value, line, mark)?;
        self.stack.push((id, HashSet::new()));
        Ok(())
    }

    fn close(&mut self) {
        self.stack.pop();
    }

    fn finish(self, format: Format) -> Tree {
        let n = self.nodes.len();
        let open_all = n <= OPEN_ALL;
        let open = self
            .nodes
            .iter()
            .map(|node| open_all || node.depth < 1)
            .collect();
        let mut t = Tree {
            format,
            nodes: self.nodes,
            open,
            rows: Vec::new(),
            dups: self.dups,
            cut: self.cut,
            skip_root: false,
        };
        t.lay_out();
        t
    }
}

/// A string kept for the screen: at most [`MAX_VALUE`] characters.
fn kept(s: &str) -> Box<str> {
    if s.len() <= MAX_VALUE {
        return s.into();
    }
    let mut out: String = s.chars().take(MAX_VALUE).collect();
    out.push('…');
    out.into()
}

// ---------------------------------------------------------------- JSON

/// A JSON reader with no recursion: the position, the line, and where the
/// tree is.
struct Json<'a> {
    s: &'a [u8],
    text: &'a str,
    i: usize,
    line: usize,
    comments: bool,
}

impl Json<'_> {
    fn err(&self, what: &str) -> String {
        format!("Not valid JSON ({what}, line {})", self.line + 1)
    }

    fn skip(&mut self) -> Result<(), String> {
        loop {
            match self.s.get(self.i) {
                Some(b'\n') => {
                    self.line += 1;
                    self.i += 1;
                }
                Some(b' ' | b'\t' | b'\r') => self.i += 1,
                Some(b'/') if self.comments => match self.s.get(self.i + 1) {
                    Some(b'/') => {
                        while self.s.get(self.i).is_some_and(|&b| b != b'\n') {
                            self.i += 1;
                        }
                    }
                    Some(b'*') => {
                        self.i += 2;
                        loop {
                            match self.s.get(self.i) {
                                None => return Err(self.err("comment not closed")),
                                Some(b'*') if self.s.get(self.i + 1) == Some(&b'/') => {
                                    self.i += 2;
                                    break;
                                }
                                Some(b'\n') => {
                                    self.line += 1;
                                    self.i += 1;
                                }
                                _ => self.i += 1,
                            }
                        }
                    }
                    _ => return Ok(()),
                },
                // A byte order mark at the start.
                Some(0xEF) if self.i == 0 && self.s.starts_with(&[0xEF, 0xBB, 0xBF]) => self.i += 3,
                _ => return Ok(()),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        // At the opening quote.
        self.i += 1;
        let mut out = String::new();
        loop {
            let start = self.i;
            while self
                .s
                .get(self.i)
                .is_some_and(|&b| b != b'"' && b != b'\\' && b != b'\n')
            {
                self.i += 1;
            }
            if out.len() < MAX_VALUE * 4 {
                out.push_str(&self.text[start..self.i]);
            }
            match self.s.get(self.i) {
                Some(b'"') => {
                    self.i += 1;
                    return Ok(out);
                }
                Some(b'\\') => {
                    let c = self.s.get(self.i + 1).copied();
                    self.i += 2;
                    let ch = match c {
                        Some(b'"') => '"',
                        Some(b'\\') => '\\',
                        Some(b'/') => '/',
                        Some(b'b') => '\u{8}',
                        Some(b'f') => '\u{c}',
                        Some(b'n') => '\n',
                        Some(b'r') => '\r',
                        Some(b't') => '\t',
                        Some(b'u') => match self.hex4() {
                            Some(hi @ 0xD800..=0xDBFF) => {
                                // A pair, or a half alone, shown as it is written.
                                if self.s.get(self.i) == Some(&b'\\')
                                    && self.s.get(self.i + 1) == Some(&b'u')
                                {
                                    let at = self.i;
                                    self.i += 2;
                                    match self.hex4() {
                                        Some(lo @ 0xDC00..=0xDFFF) => char::from_u32(
                                            0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00),
                                        )
                                        .unwrap_or('\u{FFFD}'),
                                        _ => {
                                            self.i = at;
                                            out.push_str(&format!("\\u{hi:04x}"));
                                            continue;
                                        }
                                    }
                                } else {
                                    out.push_str(&format!("\\u{hi:04x}"));
                                    continue;
                                }
                            }
                            Some(lo @ 0xDC00..=0xDFFF) => {
                                out.push_str(&format!("\\u{lo:04x}"));
                                continue;
                            }
                            Some(v) => char::from_u32(v).unwrap_or('\u{FFFD}'),
                            None => return Err(self.err("bad \\u escape")),
                        },
                        Some(b'\n') if self.comments => {
                            self.line += 1;
                            continue;
                        }
                        _ => return Err(self.err("bad escape")),
                    };
                    if out.len() < MAX_VALUE * 4 {
                        out.push(ch);
                    }
                }
                _ => return Err(self.err("string not closed")),
            }
        }
    }

    fn hex4(&mut self) -> Option<u32> {
        let h = self.s.get(self.i..self.i + 4)?;
        let v = u32::from_str_radix(std::str::from_utf8(h).ok()?, 16).ok()?;
        self.i += 4;
        Some(v)
    }

    /// A number, `true`, `false` or `null`: the word as it is written.
    fn word(&mut self) -> &str {
        let start = self.i;
        while self
            .s
            .get(self.i)
            .is_some_and(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'+' | b'.'))
        {
            self.i += 1;
        }
        &self.text[start..self.i]
    }
}

/// What the JSON reader expects next.
#[derive(Clone, Copy, PartialEq)]
enum Want {
    Value,
    /// A key or the end of the object (just after `{` or a `,`).
    Key {
        first: bool,
    },
    /// A `,` or the end of the container.
    After,
}

pub fn parse_json(text: &str, comments: bool, time: Duration) -> Result<Tree, String> {
    let mut j = Json {
        s: text.as_bytes(),
        text,
        i: 0,
        line: 0,
        comments,
    };
    let mut b = Builder::new(time);
    // For each open container: true for an object.
    let mut kinds: Vec<bool> = Vec::new();
    let mut want = Want::Value;
    let mut key: Option<Box<str>> = None;
    let mut done = false;
    loop {
        j.skip()?;
        if !b.room() {
            break;
        }
        let Some(&c) = j.s.get(j.i) else {
            if !done || !kinds.is_empty() {
                return Err(j.err("it ends too soon"));
            }
            break;
        };
        if done {
            return Err(j.err("more after the end"));
        }
        match want {
            Want::Key { first } => match c {
                b'"' => {
                    key = Some(kept(&j.string()?));
                    j.skip()?;
                    if j.s.get(j.i) != Some(&b':') {
                        return Err(j.err("a `:` was expected"));
                    }
                    j.i += 1;
                    want = Want::Value;
                }
                b'}' if first || comments => {
                    j.i += 1;
                    kinds.pop();
                    b.close();
                    want = Want::After;
                    done = kinds.is_empty();
                }
                _ => return Err(j.err("a key was expected")),
            },
            Want::After => match (c, kinds.last()) {
                (b',', Some(&object)) => {
                    j.i += 1;
                    want = if object {
                        Want::Key { first: false }
                    } else {
                        Want::Value
                    };
                    if !object && comments {
                        // A trailing comma: the end may come next.
                        j.skip()?;
                        if j.s.get(j.i) == Some(&b']') {
                            want = Want::After;
                        }
                    }
                }
                (b'}', Some(true)) | (b']', Some(false)) => {
                    j.i += 1;
                    kinds.pop();
                    b.close();
                    done = kinds.is_empty();
                }
                _ => return Err(j.err("a `,` or the end was expected")),
            },
            Want::Value => {
                let line = j.line;
                let k = key.take();
                match c {
                    b'{' => {
                        j.i += 1;
                        b.open(k, Value::Object(0), line, None)?;
                        kinds.push(true);
                        want = Want::Key { first: true };
                        continue;
                    }
                    b'[' => {
                        j.i += 1;
                        b.open(k, Value::Array(0), line, None)?;
                        kinds.push(false);
                        j.skip()?;
                        if j.s.get(j.i) == Some(&b']') {
                            j.i += 1;
                            kinds.pop();
                            b.close();
                            want = Want::After;
                            done = kinds.is_empty();
                        }
                        continue;
                    }
                    b'"' => {
                        let s = j.string()?;
                        b.add(k, Value::Str(kept(&s)), line, None)?;
                    }
                    b']' if comments => {
                        // After a trailing comma.
                        return Err(j.err("a value was expected"));
                    }
                    _ => {
                        let w = j.word();
                        let v = match w {
                            "true" => Value::Bool(true),
                            "false" => Value::Bool(false),
                            "null" => Value::Null,
                            w if is_number(w) => Value::Raw(kept(w)),
                            _ => return Err(j.err("a value was expected")),
                        };
                        b.add(k, v, line, None)?;
                    }
                }
                want = Want::After;
                done = kinds.is_empty();
            }
        }
    }
    Ok(b.finish(if comments {
        Format::JsonC
    } else {
        Format::Json
    }))
}

fn is_number(w: &str) -> bool {
    let b = w.as_bytes();
    let mut i = usize::from(b.first() == Some(&b'-'));
    let digits = |i: &mut usize| {
        let s = *i;
        while b.get(*i).is_some_and(u8::is_ascii_digit) {
            *i += 1;
        }
        *i > s
    };
    if !digits(&mut i) {
        return false;
    }
    if b.get(i) == Some(&b'.') {
        i += 1;
        if !digits(&mut i) {
            return false;
        }
    }
    if matches!(b.get(i), Some(b'e' | b'E')) {
        i += 1;
        if matches!(b.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        if !digits(&mut i) {
            return false;
        }
    }
    i == b.len()
}

// ---------------------------------------------------------------- TOML

/// How deep `text` nests its `[` and `{`, outside strings and comments: the
/// TOML parser recurses, so a file nested too deep never gets to it.
fn bracket_depth(text: &str) -> usize {
    let (mut depth, mut most) = (0usize, 0usize);
    let mut chars = text.chars().peekable();
    let mut quote: Option<char> = None;
    let mut line_start = true;
    while let Some(c) = chars.next() {
        match quote {
            Some(q) => {
                if c == '\\' && q == '"' {
                    chars.next();
                } else if c == q {
                    quote = None;
                }
            }
            None => match c {
                '"' | '\'' => quote = Some(c),
                '#' => {
                    while chars.peek().is_some_and(|&c| c != '\n') {
                        chars.next();
                    }
                }
                // A table header (`[a]`, `[[a]]`) is not nesting.
                '[' if line_start => {
                    while chars.peek().is_some_and(|&c| c != '\n') {
                        chars.next();
                    }
                }
                '[' | '{' => {
                    depth += 1;
                    most = most.max(depth);
                }
                ']' | '}' => depth = depth.saturating_sub(1),
                _ => {}
            },
        }
        if c == '\n' {
            line_start = true;
        } else if !c.is_whitespace() {
            line_start = false;
        }
    }
    most
}

pub fn parse_toml(text: &str, time: Duration) -> Result<Tree, String> {
    if bracket_depth(text) > MAX_DEPTH {
        return Err(format!("Nested deeper than {MAX_DEPTH} levels"));
    }
    let doc = toml_edit::Document::parse(text).map_err(|e| {
        let line = e
            .span()
            .map(|s| text[..s.start.min(text.len())].matches('\n').count() + 1)
            .unwrap_or(0);
        format!("Not valid TOML (line {line})")
    })?;
    let line_of = |span: Option<std::ops::Range<usize>>| {
        span.map(|s| text[..s.start.min(text.len())].matches('\n').count())
    };
    let mut b = Builder::new(time);
    // The items to walk: (key, item, line of the parent), with a marker to
    // close a container once its children are done.
    enum Step<'t> {
        Item(Option<Box<str>>, &'t toml_edit::Item, usize),
        Value(Option<Box<str>>, &'t toml_edit::Value, usize),
        /// A table of an array of tables starts: its items follow.
        Table(usize),
        Close,
    }
    let root = doc.as_item();
    let mut todo = vec![Step::Item(None, root, 0)];
    while let Some(step) = todo.pop() {
        if !b.room() {
            break;
        }
        match step {
            Step::Close => b.close(),
            Step::Table(line) => b.open(None, Value::Object(0), line, None)?,
            Step::Item(key, item, up) => match item {
                toml_edit::Item::None => {}
                toml_edit::Item::Value(v) => todo.push(Step::Value(key, v, up)),
                toml_edit::Item::Table(t) => {
                    let line = line_of(t.span()).unwrap_or(up);
                    b.open(key, Value::Object(0), line, None)?;
                    todo.push(Step::Close);
                    let items: Vec<_> = t.iter().collect();
                    for (k, i) in items.into_iter().rev() {
                        todo.push(Step::Item(Some(kept(k)), i, line));
                    }
                }
                toml_edit::Item::ArrayOfTables(a) => {
                    let line = line_of(a.span()).unwrap_or(up);
                    b.open(key, Value::Array(0), line, None)?;
                    todo.push(Step::Close);
                    let tables: Vec<_> = a.iter().collect();
                    for t in tables.into_iter().rev() {
                        let line = line_of(t.span()).unwrap_or(line);
                        todo.push(Step::Close);
                        let items: Vec<_> = t.iter().collect();
                        for (k, i) in items.into_iter().rev() {
                            todo.push(Step::Item(Some(kept(k)), i, line));
                        }
                        // Opened before its items, which come after it.
                        todo.push(Step::Table(line));
                    }
                }
            },
            Step::Value(key, v, up) => {
                let line = line_of(v.span()).unwrap_or(up);
                match v {
                    toml_edit::Value::String(s) => {
                        b.add(key, Value::Str(kept(s.value())), line, None)?;
                    }
                    toml_edit::Value::Integer(n) => {
                        b.add(
                            key,
                            Value::Raw(kept(&as_written(text, v.span(), n.value()))),
                            line,
                            None,
                        )?;
                    }
                    toml_edit::Value::Float(n) => {
                        b.add(
                            key,
                            Value::Raw(kept(&as_written(text, v.span(), n.value()))),
                            line,
                            None,
                        )?;
                    }
                    toml_edit::Value::Boolean(x) => {
                        b.add(key, Value::Bool(*x.value()), line, None)?;
                    }
                    toml_edit::Value::Datetime(d) => {
                        b.add(key, Value::Raw(kept(&d.value().to_string())), line, None)?;
                    }
                    toml_edit::Value::Array(a) => {
                        b.open(key, Value::Array(0), line, None)?;
                        todo.push(Step::Close);
                        let values: Vec<_> = a.iter().collect();
                        for v in values.into_iter().rev() {
                            todo.push(Step::Value(None, v, line));
                        }
                    }
                    toml_edit::Value::InlineTable(t) => {
                        b.open(key, Value::Object(0), line, None)?;
                        todo.push(Step::Close);
                        let items: Vec<_> = t.iter().collect();
                        for (k, v) in items.into_iter().rev() {
                            todo.push(Step::Value(Some(kept(k)), v, line));
                        }
                    }
                }
            }
        }
    }
    Ok(b.finish(Format::Toml))
}

/// A TOML value as the file writes it (`1_000`, `0xff`), not as the parser
/// reads it.
fn as_written(text: &str, span: Option<std::ops::Range<usize>>, value: impl ToString) -> String {
    span.and_then(|s| text.get(s))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| value.to_string())
}

// ---------------------------------------------------------------- YAML

/// A mapping or a sequence being read, and the key waiting for its value.
struct YamlOpen {
    mapping: bool,
    key: Option<Option<Box<str>>>,
}

/// The key of the next value: the one a mapping is waiting with, or the name
/// of the document at the top.
fn yaml_key(open: &mut [YamlOpen], document: &mut Option<Box<str>>) -> Option<Box<str>> {
    match open.last_mut() {
        Some(o) if o.mapping => o.key.take().flatten(),
        Some(_) => None,
        None => document.take(),
    }
}

/// YAML, from the parser's events: an alias is shown as `*a1`, never
/// expanded, and an anchor as `&a1` (the parser keeps the names to itself).
/// The documents go under one root, which is not shown when there is only
/// one.
pub fn parse_yaml(text: &str, time: Duration) -> Result<Tree, String> {
    use saphyr_parser::{Event, Parser, ScalarStyle};
    let mut p = Parser::new_from_str(text);
    let mut b = Builder::new(time);
    b.open(None, Value::Array(0), 0, None)?;
    let mut open: Vec<YamlOpen> = Vec::new();
    let mut docs = 0usize;
    // A mapping or a sequence used as a key: skipped, and named as such.
    let mut skipping = 0usize;
    let mut document: Option<Box<str>> = None;
    while let Some(next) = p.next_event() {
        if !b.room() {
            break;
        }
        let (event, span) =
            next.map_err(|e| format!("Not valid YAML ({}, line {})", e.info(), e.marker().line()))?;
        let line = span.start.line().saturating_sub(1);
        if skipping > 0 {
            match event {
                Event::MappingStart(..) | Event::SequenceStart(..) => skipping += 1,
                Event::MappingEnd | Event::SequenceEnd => skipping -= 1,
                _ => {}
            }
            if skipping == 0
                && let Some(o) = open.last_mut()
            {
                o.key = Some(Some("(complex key)".into()));
            }
            continue;
        }
        let waiting_key = open.last().is_some_and(|o| o.mapping && o.key.is_none());
        let mark = |anchor: usize, tag: &Option<std::borrow::Cow<'_, saphyr_parser::Tag>>| {
            let mut m = String::new();
            if anchor > 0 {
                m.push_str(&format!("&a{anchor}"));
            }
            if let Some(t) = tag {
                if !m.is_empty() {
                    m.push(' ');
                }
                m.push_str(&format!("{}{}", t.handle, t.suffix));
            }
            (!m.is_empty()).then(|| kept(&m))
        };
        match event {
            Event::DocumentStart(_) => {
                docs += 1;
                document = Some(format!("document {docs}").into());
            }
            Event::Alias(id) => {
                let alias: Box<str> = format!("*a{id}").into();
                if waiting_key {
                    if let Some(o) = open.last_mut() {
                        o.key = Some(Some(alias));
                    }
                    continue;
                }
                let key = yaml_key(&mut open, &mut document);
                b.add(key, Value::Alias(alias), line, None)?;
            }
            Event::Scalar(value, style, anchor, tag) => {
                if waiting_key {
                    if let Some(o) = open.last_mut() {
                        o.key = Some(Some(kept(&value)));
                    }
                    continue;
                }
                let key = yaml_key(&mut open, &mut document);
                let v = if style != ScalarStyle::Plain {
                    Value::Str(kept(&value))
                } else {
                    match &*value {
                        "true" | "True" | "TRUE" => Value::Bool(true),
                        "false" | "False" | "FALSE" => Value::Bool(false),
                        "null" | "Null" | "NULL" | "~" | "" => Value::Null,
                        _ => Value::Raw(kept(&value)),
                    }
                };
                b.add(key, v, line, mark(anchor, &tag))?;
            }
            Event::MappingStart(..) | Event::SequenceStart(..) if waiting_key => skipping = 1,
            Event::MappingStart(anchor, tag) => {
                let key = yaml_key(&mut open, &mut document);
                b.open(key, Value::Object(0), line, mark(anchor, &tag))?;
                open.push(YamlOpen {
                    mapping: true,
                    key: None,
                });
            }
            Event::SequenceStart(anchor, tag) => {
                let key = yaml_key(&mut open, &mut document);
                b.open(key, Value::Array(0), line, mark(anchor, &tag))?;
                open.push(YamlOpen {
                    mapping: false,
                    key: None,
                });
            }
            Event::MappingEnd | Event::SequenceEnd => {
                open.pop();
                b.close();
            }
            _ => {}
        }
    }
    let mut t = b.finish(Format::Yaml);
    // One document: its root is the tree's.
    if t.nodes
        .first()
        .is_some_and(|r| matches!(r.value, Value::Array(1)))
    {
        t.skip_root = true;
        let only = t.nodes[0].first;
        t.nodes[only as usize].key = None;
        t.open[0] = true;
        t.lay_out();
    }
    Ok(t)
}

// ---------------------------------------------------------------- the view

impl Tree {
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn node(&self, id: u32) -> &Node {
        &self.nodes[id as usize]
    }

    /// The rows on screen, by node.
    pub fn rows(&self) -> &[u32] {
        &self.rows
    }

    pub fn is_open(&self, id: u32) -> bool {
        self.open[id as usize]
    }

    pub fn has_children(&self, id: u32) -> bool {
        self.nodes[id as usize].first != NONE
    }

    /// The nodes under open nodes, in order.
    fn lay_out(&mut self) {
        self.rows.clear();
        let mut stack: Vec<u32> = Vec::new();
        // The roots: the first node and its siblings.
        if self.skip_root {
            if let Some(r) = self.nodes.first().map(|r| r.first).filter(|&f| f != NONE) {
                stack.push(r);
            }
        } else if !self.nodes.is_empty() {
            stack.push(0);
        }
        while let Some(id) = stack.pop() {
            self.rows.push(id);
            let node = &self.nodes[id as usize];
            if node.next != NONE {
                stack.push(node.next);
            }
            if self.open[id as usize] && node.first != NONE {
                stack.push(node.first);
            }
        }
    }

    /// Opens or closes `id`: true if it changed.
    pub fn set_open(&mut self, id: u32, open: bool) -> bool {
        if !self.has_children(id) || self.open[id as usize] == open {
            return false;
        }
        self.open[id as usize] = open;
        self.lay_out();
        true
    }

    /// Opens everything under `id`, up to [`OPEN_ALL`] nodes.
    pub fn open_all(&mut self, id: u32) {
        let mut stack = vec![id];
        let mut n = 0;
        while let Some(i) = stack.pop() {
            if n >= OPEN_ALL {
                break;
            }
            n += 1;
            self.open[i as usize] = true;
            let mut c = self.nodes[i as usize].first;
            while c != NONE {
                stack.push(c);
                c = self.nodes[c as usize].next;
            }
        }
        self.lay_out();
    }

    /// Opens the nodes above `id`, so it is on screen.
    pub fn reveal(&mut self, id: u32) {
        let mut p = self.nodes[id as usize].parent;
        let mut changed = false;
        while p != NONE {
            changed |= !self.open[p as usize];
            self.open[p as usize] = true;
            p = self.nodes[p as usize].parent;
        }
        if changed {
            self.lay_out();
        }
    }

    pub fn parent(&self, id: u32) -> Option<u32> {
        let p = self.nodes[id as usize].parent;
        (p != NONE && !(self.skip_root && p == 0)).then_some(p)
    }

    /// How far in a node is drawn.
    pub fn depth(&self, id: u32) -> usize {
        let d = self.nodes[id as usize].depth as usize;
        if self.skip_root {
            d.saturating_sub(1)
        } else {
            d
        }
    }

    /// The row of `id`, if it is on screen.
    pub fn row_of(&self, id: u32) -> Option<usize> {
        self.rows.iter().position(|&r| r == id)
    }

    /// The path of a node: `.servers[2].name` in JSON, `servers.2.name` in
    /// YAML and TOML.
    pub fn path(&self, id: u32) -> String {
        let mut parts = Vec::new();
        let mut i = id;
        while i != NONE {
            let node = &self.nodes[i as usize];
            let parent = node.parent;
            if parent != NONE && !(self.skip_root && parent == 0) {
                let p = &self.nodes[parent as usize];
                match (&p.value, &node.key) {
                    (Value::Array(_), _) => {
                        let mut index = 0;
                        let mut c = p.first;
                        while c != i && c != NONE {
                            index += 1;
                            c = self.nodes[c as usize].next;
                        }
                        parts.push(Part::Index(index));
                    }
                    (_, Some(k)) => parts.push(Part::Key(k.to_string())),
                    _ => {}
                }
            }
            i = parent;
        }
        parts.reverse();
        let json = matches!(self.format, Format::Json | Format::JsonC);
        let mut out = String::new();
        for p in parts {
            match p {
                Part::Index(n) if json => out.push_str(&format!("[{n}]")),
                Part::Index(n) => {
                    if !out.is_empty() {
                        out.push('.');
                    }
                    out.push_str(&n.to_string());
                }
                Part::Key(k) => {
                    let plain = !k.is_empty()
                        && k.chars()
                            .all(|c| c.is_alphanumeric() || c == '_' || c == '-');
                    if json && !plain {
                        out.push_str(&format!("[{}]", quoted(&k)));
                    } else if json {
                        out.push('.');
                        out.push_str(&k);
                    } else {
                        if !out.is_empty() {
                            out.push('.');
                        }
                        if plain {
                            out.push_str(&k);
                        } else {
                            out.push_str(&quoted(&k));
                        }
                    }
                }
            }
        }
        if out.is_empty() {
            out.push(if json { '.' } else { '/' });
        }
        out
    }

    /// The text of a row: its key and value, as the screen shows it (without
    /// the indentation and the fold mark).
    #[cfg(test)]
    pub fn label(&self, id: u32) -> String {
        let node = &self.nodes[id as usize];
        let mut s = String::new();
        if let Some(k) = &node.key {
            s.push_str(&shown(k));
            s.push_str(": ");
        }
        if let Some(m) = &node.mark {
            s.push_str(&shown(m));
            s.push(' ');
        }
        s.push_str(&self.value_text(id));
        if node.dup {
            s.push_str("  (dup key)");
        }
        s
    }

    pub fn value_text(&self, id: u32) -> String {
        let node = &self.nodes[id as usize];
        let open = self.open[id as usize];
        match &node.value {
            Value::Object(n) if open && *n > 0 => "{".into(),
            Value::Array(n) if open && *n > 0 => "[".into(),
            Value::Object(n) => format!("{{{n} key{}}}", if *n == 1 { "" } else { "s" }),
            Value::Array(n) => format!("[{n} item{}]", if *n == 1 { "" } else { "s" }),
            Value::Str(s) => format!("\"{}\"", shown(s)),
            Value::Raw(s) => shown(s),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".into(),
            Value::Alias(a) => shown(a),
        }
    }

    /// The first node after (or before) `from` in the whole tree, open or not,
    /// whose label `matches`.
    pub fn find(
        &self,
        from: Option<u32>,
        forward: bool,
        matches: impl Fn(&str) -> bool,
    ) -> Option<u32> {
        let n = self.nodes.len() as u32;
        if n == 0 {
            return None;
        }
        // Nodes are in the order of the file: their ids are the order.
        let ids: Box<dyn Iterator<Item = u32>> = match (from, forward) {
            (Some(f), true) => Box::new(f + 1..n),
            (None, true) => Box::new(0..n),
            (Some(f), false) => Box::new((0..f).rev()),
            (None, false) => Box::new((0..n).rev()),
        };
        for id in ids {
            let node = &self.nodes[id as usize];
            let key = node.key.as_deref().unwrap_or("");
            let value = match &node.value {
                Value::Str(s) | Value::Raw(s) | Value::Alias(s) => s.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => "null".into(),
                _ => String::new(),
            };
            if matches(key) || matches(&value) {
                return Some(id);
            }
        }
        None
    }

    /// The node starting at or after the source line `line` (from 0).
    pub fn node_at_line(&self, line: usize) -> Option<u32> {
        let i = self.nodes.partition_point(|n| (n.line as usize) < line);
        (i < self.nodes.len())
            .then_some(i as u32)
            .or_else(|| self.nodes.len().checked_sub(1).map(|i| i as u32))
    }
}

enum Part {
    Key(String),
    Index(usize),
}

fn quoted(k: &str) -> String {
    format!("\"{}\"", k.replace('\\', "\\\\").replace('"', "\\\""))
}

/// A key or a value on one row: a line end as `↵`, a tab as a space. Other
/// control characters are shown as `·` when the screen is drawn.
pub fn shown(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\n' => '↵',
            '\t' => ' ',
            c => c,
        })
        .collect()
}

pub fn parse(format: Format, text: &str, time: Duration) -> Result<Tree, String> {
    match format {
        Format::Json => parse_json(text, false, time),
        Format::JsonC => parse_json(text, true, time),
        Format::Toml => parse_toml(text, time),
        Format::Yaml => parse_yaml(text, time),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: Duration = Duration::from_secs(5);

    fn labels(t: &Tree) -> Vec<String> {
        t.rows()
            .iter()
            .map(|&r| format!("{}{}", "  ".repeat(t.depth(r)), t.label(r)))
            .collect()
    }

    #[test]
    fn json_as_a_tree() {
        let t = parse_json(
            r#"{"name": "x\ny", "n": 12345678901234567890, "list": [1, true, null, {"a": []}], "name": 2}"#,
            false,
            T,
        )
        .unwrap();
        assert_eq!(
            labels(&t),
            [
                "{",
                "  name: \"x↵y\"",
                "  n: 12345678901234567890",
                "  list: [",
                "    1",
                "    true",
                "    null",
                "    {",
                "      a: [0 items]",
                "  name: 2  (dup key)",
            ]
        );
        assert_eq!(t.dups, 1);
        let a = t.find(None, true, |s| s == "a").unwrap();
        assert_eq!(t.path(a), ".list[3].a");
        let list = t.find(None, true, |s| s == "list").unwrap();
        assert!(
            t.set_open_clone(list, false)
                .iter()
                .any(|l| l == "  list: [4 items]")
        );
    }

    impl Tree {
        fn set_open_clone(&self, id: u32, open: bool) -> Vec<String> {
            let mut t = self.clone();
            t.set_open(id, open);
            labels(&t)
        }
    }

    #[test]
    fn json_errors_and_limits() {
        for bad in [
            "",
            "{",
            "[1,]",
            "{\"a\" 1}",
            "[1] 2",
            "\"open",
            "[01x]",
            "{,}",
            "nul",
        ] {
            assert!(parse_json(bad, false, T).is_err(), "{bad:?}");
        }
        let deep = "[".repeat(MAX_DEPTH + 1) + &"]".repeat(MAX_DEPTH + 1);
        assert!(parse_json(&deep, false, T).unwrap_err().contains("deeper"));
        let ok = "[".repeat(MAX_DEPTH) + &"]".repeat(MAX_DEPTH);
        assert!(parse_json(&ok, false, T).is_ok());
        // Escapes: a pair, a half alone, and a control character.
        let t = parse_json(
            r#"["\ud83d\ude00", "\ud800x", "\u001b[2J\u202e"]"#,
            false,
            T,
        )
        .unwrap();
        assert_eq!(t.label(1), "\"😀\"");
        assert_eq!(t.label(2), "\"\\ud800x\"");
        assert_eq!(
            t.label(3),
            "\"\u{1b}[2J\u{202e}\"",
            "cleaned when drawn, like any text"
        );
    }

    #[test]
    fn jsonc_takes_comments_and_trailing_commas() {
        let t = parse_json("// c\n{\"a\": [1, 2,], /* x */ \"b\": 1,}\n", true, T).unwrap();
        assert_eq!(labels(&t), ["{", "  a: [", "    1", "    2", "  b: 1"]);
        assert_eq!(t.node(t.find(None, true, |s| s == "b").unwrap()).line, 1);
        assert!(parse_json("{\"a\": 1,}", false, T).is_err());
    }

    #[test]
    fn toml_as_a_tree() {
        let t = parse_toml(
            "title = \"x\"\nn = 1_000\n[server]\nport = 8080\nat = 1979-05-27T07:32:00Z\n[[bin]]\nname = \"a\"\n[[bin]]\nname = \"b\"\n",
            T,
        )
        .unwrap();
        let l = labels(&t);
        assert!(l.contains(&"  title: \"x\"".to_string()), "{l:?}");
        assert!(l.contains(&"  n: 1_000".to_string()), "as written: {l:?}");
        assert!(l.contains(&"    port: 8080".to_string()), "{l:?}");
        assert!(
            l.contains(&"    at: 1979-05-27T07:32:00Z".to_string()),
            "{l:?}"
        );
        let b = t.find(None, true, |s| s == "b").unwrap();
        assert_eq!(t.path(b), "bin.1.name");
        assert_eq!(t.node(b).line, 8);
        assert!(parse_toml("a = ", T).is_err());
        let deep = format!(
            "a = {}{}",
            "[".repeat(MAX_DEPTH + 1),
            "]".repeat(MAX_DEPTH + 1)
        );
        assert!(parse_toml(&deep, T).unwrap_err().contains("deeper"));
    }

    #[test]
    fn yaml_as_a_tree_without_expanding_aliases() {
        let t = parse_yaml(
            "base: &b\n  x: 1\nuse: *b\nlist:\n  - 'q'\n  - ~\n  - yes\n",
            T,
        )
        .unwrap();
        let l = labels(&t);
        assert_eq!(l[0], "{", "one document: its mapping is the root");
        assert!(l.contains(&"  base: &a1 {".to_string()), "{l:?}");
        assert!(l.contains(&"  use: *a1".to_string()), "{l:?}");
        let x = t.find(None, true, |s| s == "x").unwrap();
        assert_eq!(t.path(x), "base.x");
        assert!(l.contains(&"    \"q\"".to_string()), "{l:?}");
        assert!(l.contains(&"    null".to_string()), "{l:?}");
        // A billion laughs costs what its text costs.
        let mut lol = String::from("a: &a [x, x, x, x, x, x, x, x, x, x]\n");
        for i in 1..12 {
            let prev = (b'a' + i - 1) as char;
            let cur = (b'a' + i) as char;
            lol.push_str(&format!("{cur}: &{cur} [*{prev}, *{prev}, *{prev}, *{prev}, *{prev}, *{prev}, *{prev}, *{prev}, *{prev}, *{prev}]\n"));
        }
        let t = parse_yaml(&lol, T).unwrap();
        assert!(t.len() < 200, "{} nodes", t.len());
        assert!(parse_yaml("a: [1, 2", T).is_err());
    }

    #[test]
    fn yaml_with_several_documents() {
        let t = parse_yaml("a: 1\n---\nb: 2\n", T).unwrap();
        let l = labels(&t);
        assert!(l.iter().any(|x| x.contains("document 1")), "{l:?}");
        assert!(l.iter().any(|x| x.contains("document 2")), "{l:?}");
        let t = parse_yaml("x\n---\ny\n", T).unwrap();
        assert_eq!(labels(&t), ["[", "  document 1: x", "  document 2: y"]);
    }

    #[test]
    fn a_big_tree_opens_two_levels() {
        let text = format!(
            "[{}]",
            (0..3000)
                .map(|i| format!("[{i}]"))
                .collect::<Vec<_>>()
                .join(",")
        );
        let t = parse_json(&text, false, T).unwrap();
        assert_eq!(t.rows().len(), 3001, "the root and its children, closed");
        let mut t = t;
        t.open_all(0);
        assert!(t.rows().len() > 3001);
    }

    proptest::proptest! {
        #[test]
        fn hostile_json_never_panics(text in "[\\[\\]{}\":,0-9a-z\\\\u \n/*.-]{0,200}") {
            for comments in [false, true] {
                if let Ok(t) = parse_json(&text, comments, T) {
                    for &r in t.rows() {
                        let _ = t.label(r);
                        let _ = t.path(r);
                    }
                }
            }
        }

        #[test]
        fn hostile_yaml_and_toml_never_panic(text in "[a-z:&*!\\-\\[\\]{}'\"# \n=.,0-9]{0,200}") {
            if let Ok(t) = parse_yaml(&text, T) {
                for &r in t.rows() {
                    let _ = t.label(r);
                    let _ = t.path(r);
                }
            }
            let _ = parse_toml(&text, T);
            let _ = bracket_depth(&text);
        }
    }
}
