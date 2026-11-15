//! A condition over the files of the list, written as the `WHERE` of a SQL
//! query: `ext = 'jpg' AND size > 10000000`. Each file is a row of a table,
//! `files`, and the list keeps those the condition takes.
//!
//! The condition is read by the viewer's small SQL (`sql_filter`), kept to
//! what SQLite also takes, so that the same text can later be the `WHERE` of
//! a whole query (see `docs/plans/query.md`).
//!
//! `contains('text')` looks inside the files, with no index: each file is
//! read when the rest of the condition cannot decide without it, in steps
//! (see [`ContentJob`]), so the screen stays free and Esc stops it.

use std::collections::HashSet;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use crate::model::tree::{FileEntry, FileKind};
use crate::search::{Opener, Pattern, SearchSpec};
use crate::viewer::sql_filter::{
    BinaryOp, Expr, RowContext, SqlFilter, Value, columns_in, eval_expr,
};

/// The columns of `files`, in the order the help shows them.
pub const COLUMNS: &[&str] = &[
    "name", "ext", "path", "dir", "depth", "size", "disk", "modified", "age_days", "mode", "kind",
    "tagged",
];

/// Conditions for the prompt's history the first time it opens: who does not
/// write SQL copies one and changes it.
pub const EXAMPLES: &[&str] = &[
    "ext IN ('md', 'txt') AND contains('invoice')",
    "ext IN ('jpg', 'png') AND NOT tagged",
    "modified >= '2026-01-01'",
    "age_days < 7",
    "size > 100000000",
];

/// A file bigger than this is not read for `contains`.
pub const MAX_READ: u64 = 256 << 20;
/// What is looked at to tell a binary file, which `contains` leaves out.
const BINARY_PROBE: usize = 8 << 10;

/// A condition ready to run.
#[derive(Debug, Clone)]
pub struct FileQuery {
    text: String,
    expr: Expr,
    /// Only the tagged files are looked at (Ctrl).
    tagged: bool,
    /// It asks for `path`, `dir` or `depth`: the folder of each file is needed.
    places: bool,
    /// The texts of its `contains` and `words`, each once, in order.
    terms: Vec<Term>,
    /// With `contains`: the files that met it when the contents were read.
    hits: Option<Arc<HashSet<PathBuf>>>,
}

/// What a function of the condition looks for in a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Term {
    /// The text in quotes.
    pub text: String,
    /// `words`: the words of the text, as [`word_pats`] splits them; `None`
    /// for `contains`, which looks for the text as it is.
    pub words: Option<Vec<WordPat>>,
}

/// One word `words()` looks for: the whole word, or any word that starts with
/// it (`fatur*`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordPat {
    pub word: String,
    pub prefix: bool,
}

impl WordPat {
    pub fn matches(&self, word: &str) -> bool {
        if self.prefix {
            word.starts_with(&self.word)
        } else {
            word == self.word
        }
    }
}

/// Splits a text into the words `words()` and the index look for: runs of
/// letters and digits, in lower case and without accents (`Ação` is `acao`).
/// The index and the reading of a file split them the same way.
pub fn words_of(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut word = String::new();
    for c in text.chars() {
        if c.is_alphanumeric() {
            for low in c.to_lowercase() {
                word.push(fold(low));
            }
        } else if !word.is_empty() {
            out.push(std::mem::take(&mut word));
        }
    }
    if !word.is_empty() {
        out.push(word);
    }
    // A word that long is not a word: the index leaves it out too.
    out.retain(|w| w.len() <= MAX_WORD);
    out
}

/// Splits the argument of `words()` into the words it looks for, with `prefix`
/// on those followed by `*` (`fatur*`).
pub fn word_pats(text: &str) -> Vec<WordPat> {
    let mut out = Vec::new();
    let mut word = String::new();
    for c in text.chars() {
        if c.is_alphanumeric() {
            for low in c.to_lowercase() {
                word.push(fold(low));
            }
        } else if !word.is_empty() {
            out.push(WordPat {
                word: std::mem::take(&mut word),
                prefix: c == '*',
            });
        }
    }
    if !word.is_empty() {
        out.push(WordPat {
            word,
            prefix: false,
        });
    }
    out.retain(|w| w.word.len() <= MAX_WORD);
    out
}

/// The longest word kept, in bytes.
pub const MAX_WORD: usize = 64;

/// The letter without its accent, for the Latin letters that have one.
fn fold(c: char) -> char {
    match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
        'ç' => 'c',
        'è' | 'é' | 'ê' | 'ë' => 'e',
        'ì' | 'í' | 'î' | 'ï' => 'i',
        'ñ' => 'n',
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
        'ù' | 'ú' | 'û' | 'ü' => 'u',
        'ý' | 'ÿ' => 'y',
        c => c,
    }
}

/// Where a file is, for the columns that say it.
#[derive(Debug, Clone, Copy)]
pub struct Place<'a> {
    /// The folder of the file.
    pub dir: &'a Path,
    /// How many folders below the root of the tree that folder is.
    pub depth: usize,
}

/// Checks that each call is a `contains` of one text, standing as a condition
/// of its own (under AND, OR and NOT), and notes its text in `terms`.
fn check_calls(expr: &Expr, standing: bool, terms: &mut Vec<Term>) -> Result<(), String> {
    match expr {
        Expr::Call { name, args } => {
            let name = name.to_ascii_lowercase();
            if name != "contains" && name != "words" {
                return Err(format!(
                    "No function {name}(): there are contains('text') and words('text')"
                ));
            }
            let [Expr::Literal(Value::Text(text))] = args.as_slice() else {
                return Err(format!(
                    "{name}() takes one text in quotes: {name}('invoice')"
                ));
            };
            let words = (name == "words").then(|| word_pats(text));
            if text.is_empty() || words.as_ref().is_some_and(Vec::is_empty) {
                return Err(format!("{name}('{text}') has nothing to look for"));
            }
            if !standing {
                return Err(format!(
                    "{name}() is a condition of its own: use it with AND, OR, NOT"
                ));
            }
            let term = Term {
                text: text.clone(),
                words,
            };
            if !terms.contains(&term) {
                terms.push(term);
            }
            Ok(())
        }
        Expr::Binary {
            op: BinaryOp::And | BinaryOp::Or,
            left,
            right,
        } => {
            check_calls(left, true, terms)?;
            check_calls(right, true, terms)
        }
        Expr::Not(inner) => check_calls(inner, standing, terms),
        Expr::Binary { left, right, .. } => {
            check_calls(left, false, terms)?;
            check_calls(right, false, terms)
        }
        Expr::IsNull(inner) | Expr::IsNotNull(inner) => check_calls(inner, false, terms),
        Expr::InList { expr, list, .. } => {
            check_calls(expr, false, terms)?;
            list.iter().try_for_each(|e| check_calls(e, false, terms))
        }
        Expr::Literal(_) | Expr::Column(_) => Ok(()),
    }
}

impl FileQuery {
    /// Reads the condition; `tagged` keeps it to the tagged files. An error
    /// says what is wrong, or which columns there are.
    pub fn parse(text: &str, tagged: bool) -> Result<Self, String> {
        let text = text.trim();
        let expr = SqlFilter::parse_with_calls(text)?;
        let mut used = Vec::new();
        columns_in(&expr, &mut used);
        if let Some(unknown) = used
            .iter()
            .find(|c| !COLUMNS.contains(&c.to_ascii_lowercase().as_str()))
        {
            return Err(format!(
                "No column \"{unknown}\": there are {}",
                COLUMNS.join(", ")
            ));
        }
        let mut terms = Vec::new();
        check_calls(&expr, true, &mut terms)?;
        let places = used
            .iter()
            .any(|c| matches!(c.to_ascii_lowercase().as_str(), "path" | "dir" | "depth"));
        Ok(FileQuery {
            text: text.to_string(),
            expr,
            tagged,
            places,
            terms,
            hits: None,
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// Only the tagged files are looked at.
    pub fn tagged(&self) -> bool {
        self.tagged
    }

    /// The condition needs to know where each file is (see [`Place`]).
    pub fn needs_place(&self) -> bool {
        self.places || self.reads_contents()
    }

    /// The terms of its `contains` and `words`.
    pub fn terms(&self) -> &[Term] {
        &self.terms
    }

    /// It has `contains` or `words`: the files must be read, or looked up in
    /// an index (see [`ContentJob`]).
    pub fn reads_contents(&self) -> bool {
        !self.terms.is_empty()
    }

    /// Whether the list keeps `file`. `place` may be left out when
    /// `needs_place` says so. A comparison that makes no sense (a text with
    /// a number) takes no file. With `contains`, the files found when they
    /// were read; none before that.
    pub fn matches(&self, file: &FileEntry, place: Option<Place<'_>>, now: SystemTime) -> bool {
        if self.tagged && !file.tagged {
            return false;
        }
        if self.reads_contents() {
            return match (&self.hits, place) {
                (Some(hits), Some(place)) => hits.contains(&place.dir.join(&file.name)),
                _ => false,
            };
        }
        let row = Row {
            file,
            place,
            now,
            found: None,
        };
        eval_expr(&self.expr, &row).is_ok_and(|v| v.is_truthy())
    }

    /// Whether the file meets the condition, knowing for each term whether
    /// the file has it (`found`, in the order of the terms); without that,
    /// `None` when only its contents can tell.
    fn decide(
        &self,
        file: &FileEntry,
        place: Option<Place<'_>>,
        now: SystemTime,
        found: &[Option<bool>],
    ) -> Option<bool> {
        if self.tagged && !file.tagged {
            return Some(false);
        }
        let row = Row {
            file,
            place,
            now,
            found: Some((self.terms.as_slice(), found)),
        };
        decide(&self.expr, &row)
    }
}

/// Three answers: yes, no, or `None` when a `contains` not read yet decides.
/// `check_calls` keeps the calls where only AND, OR and NOT are above them.
fn decide(expr: &Expr, row: &Row<'_>) -> Option<bool> {
    match expr {
        Expr::Binary {
            op: BinaryOp::And,
            left,
            right,
        } => match (decide(left, row), decide(right, row)) {
            (Some(false), _) | (_, Some(false)) => Some(false),
            (Some(true), Some(true)) => Some(true),
            _ => None,
        },
        Expr::Binary {
            op: BinaryOp::Or,
            left,
            right,
        } => match (decide(left, row), decide(right, row)) {
            (Some(true), _) | (_, Some(true)) => Some(true),
            (Some(false), Some(false)) => Some(false),
            _ => None,
        },
        Expr::Not(inner) => decide(inner, row).map(|b| !b),
        Expr::Call { name, args } => {
            let (terms, found) = row.found?;
            let Some(Expr::Literal(Value::Text(text))) = args.first() else {
                return Some(false);
            };
            let words = name.eq_ignore_ascii_case("words");
            let i = terms
                .iter()
                .position(|t| t.text == *text && t.words.is_some() == words)?;
            found.get(i).copied().flatten()
        }
        _ => Some(eval_expr(expr, row).is_ok_and(|v| v.is_truthy())),
    }
}

/// One file as a row of `files`.
struct Row<'a> {
    file: &'a FileEntry,
    place: Option<Place<'a>>,
    now: SystemTime,
    /// The terms of `contains`, and whether the file has each one.
    found: Option<(&'a [Term], &'a [Option<bool>])>,
}

impl RowContext for Row<'_> {
    fn get_value(&self, column: &str) -> Option<Value> {
        let f = self.file;
        let text = |s: std::borrow::Cow<'_, str>| Some(Value::Text(s.into_owned()));
        match column.to_ascii_lowercase().as_str() {
            "name" => text(f.name.to_string_lossy()),
            // In lower case and without the dot; empty with no extension.
            "ext" => {
                let ext = Path::new(&f.name).extension().unwrap_or_default();
                Some(Value::Text(ext.to_string_lossy().to_lowercase()))
            }
            "path" => text(self.place?.dir.join(&f.name).to_string_lossy()),
            "dir" => text(self.place?.dir.to_string_lossy()),
            "depth" => Some(Value::Int(self.place?.depth as i64)),
            "size" => Some(Value::Int(f.size as i64)),
            "disk" => Some(Value::Int(f.disk as i64)),
            // Local time, written so that it compares as text:
            // `modified >= '2026-01-01'`.
            "modified" => {
                let local: chrono::DateTime<chrono::Local> = f.modified?.into();
                Some(Value::Text(local.format("%Y-%m-%d %H:%M:%S").to_string()))
            }
            // Days since it was changed; negative for a date in the future.
            "age_days" => {
                let modified = f.modified?;
                let secs = match self.now.duration_since(modified) {
                    Ok(d) => d.as_secs_f64(),
                    Err(e) => -e.duration().as_secs_f64(),
                };
                Some(Value::Float(secs / 86_400.0))
            }
            // The permissions as they are written: 644, 755.
            "mode" => {
                let octal = format!("{:o}", f.mode & 0o7777);
                Some(Value::Int(octal.parse().unwrap_or(0)))
            }
            "kind" => Some(Value::Text(
                match f.kind {
                    FileKind::File => "file",
                    FileKind::Symlink => "link",
                    FileKind::Other => "other",
                }
                .to_string(),
            )),
            "tagged" => Some(Value::Bool(f.tagged)),
            _ => None,
        }
    }
}

/// The file of the list being looked at, with what the condition needs of it.
pub struct Candidate {
    pub path: PathBuf,
    pub dir: PathBuf,
    pub depth: usize,
    pub file: FileEntry,
}

/// Splits a text read in pieces into [`words_of`]'s words, the same as if it
/// were read whole: a piece is cut only after a byte that cannot be inside a
/// word (an ASCII byte that is not a letter or a digit).
#[derive(Default)]
pub struct WordSplitter {
    carry: Vec<u8>,
}

/// A piece of text with no place to cut it this long is cut anyway: its words
/// are longer than [`MAX_WORD`], and left out.
const MAX_CARRY: usize = 4 << 10;

impl WordSplitter {
    /// Gives the words of `bytes`, keeping a word that may go on.
    pub fn feed(&mut self, bytes: &[u8], mut each: impl FnMut(String)) {
        self.carry.extend_from_slice(bytes);
        let cut = self
            .carry
            .iter()
            .rposition(|&b| b.is_ascii() && !b.is_ascii_alphanumeric())
            .map(|i| i + 1)
            .or((self.carry.len() > MAX_CARRY).then_some(self.carry.len()));
        if let Some(cut) = cut {
            let rest = self.carry.split_off(cut);
            words_of(&String::from_utf8_lossy(&self.carry))
                .into_iter()
                .for_each(&mut each);
            self.carry = rest;
        }
    }

    /// The words left at the end of the text.
    pub fn finish(&mut self, each: impl FnMut(String)) {
        let rest = std::mem::take(&mut self.carry);
        words_of(&String::from_utf8_lossy(&rest))
            .into_iter()
            .for_each(each);
    }
}

/// How a file is being looked at for a term.
enum Scan {
    /// `contains`: what the next step needs of what was read.
    Text(Vec<u8>),
    /// `words`: the words not found yet.
    Words(WordSplitter, Vec<WordPat>),
}

/// The file being read, from one step to the next.
struct Reading {
    /// Which term is being looked for.
    term: usize,
    reader: Box<dyn BufRead>,
    scan: Scan,
}

/// Reads `reader` for the words `left` until all are found, the text ends or
/// `until` comes: `None` if not finished.
fn scan_words(
    reader: &mut dyn BufRead,
    splitter: &mut WordSplitter,
    left: &mut Vec<WordPat>,
    until: Instant,
) -> std::io::Result<Option<bool>> {
    loop {
        if left.is_empty() {
            return Ok(Some(true));
        }
        let chunk = match reader.fill_buf() {
            Ok(chunk) => chunk,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(None),
            Err(e) => return Err(e),
        };
        if chunk.is_empty() {
            splitter.finish(|w| {
                left.retain(|p| !p.matches(&w));
            });
            return Ok(Some(left.is_empty()));
        }
        let n = chunk.len();
        splitter.feed(chunk, |w| {
            left.retain(|p| !p.matches(&w));
        });
        reader.consume(n);
        if Instant::now() >= until {
            return Ok(None);
        }
    }
}

/// What the reading of the contents left out, and why.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Skipped {
    pub binary: usize,
    pub links: usize,
    pub too_big: usize,
    pub unreadable: usize,
}

impl Skipped {
    fn total(&self) -> usize {
        self.binary + self.links + self.too_big + self.unreadable
    }
}

/// A file to read, and what is known of each term in it so far.
struct Item {
    c: Candidate,
    found: Vec<Option<bool>>,
}

/// What an index up to date knows of a file.
#[derive(Debug, PartialEq, Eq)]
pub enum Known {
    /// For each term, whether the file has it (`None` where the index cannot
    /// tell: `contains`).
    Terms(Vec<Option<bool>>),
    /// It is binary: left out, as when it is read.
    Binary,
}

/// The index's answer for a file, when it has the file as it is now.
pub type IndexAnswer<'a> = &'a dyn Fn(&Candidate) -> Option<Known>;

/// A condition with `contains` or `words` going over the files of the list:
/// the files the rest of the condition decides are not read, nor those an
/// index answers for.
pub struct ContentJob {
    query: FileQuery,
    open: Opener,
    /// The files to read, and how many of them are done.
    items: Vec<Item>,
    next: usize,
    reading: Option<Reading>,
    hits: HashSet<PathBuf>,
    /// The files an index answered for, with no reading.
    pub from_index: usize,
    pub bytes_total: u64,
    pub bytes_done: u64,
    pub skipped: Skipped,
    pub first_error: Option<String>,
}

impl ContentJob {
    /// Decides what it can of `candidates` at once, with the help of `index`;
    /// the rest is read by `step`.
    pub fn new(
        query: FileQuery,
        candidates: Vec<Candidate>,
        open: Opener,
        now: SystemTime,
        index: IndexAnswer<'_>,
    ) -> Self {
        let terms = query.terms.len();
        let mut hits = HashSet::new();
        let mut items = Vec::new();
        let mut skipped = Skipped::default();
        let mut from_index = 0;
        for c in candidates {
            let place = Place {
                dir: &c.dir,
                depth: c.depth,
            };
            let mut found = vec![None; terms];
            let mut decided = query.decide(&c.file, Some(place), now, &found);
            if decided.is_none() {
                match index(&c) {
                    Some(Known::Terms(known)) => {
                        found = known;
                        decided = query.decide(&c.file, Some(place), now, &found);
                        from_index += usize::from(decided.is_some());
                    }
                    Some(Known::Binary) => {
                        skipped.binary += 1;
                        from_index += 1;
                        continue;
                    }
                    None => {}
                }
            }
            match decided {
                Some(true) => {
                    hits.insert(c.path);
                }
                Some(false) => {}
                // Read only what is a file, of a size that can be read.
                None if c.file.kind == FileKind::Symlink => skipped.links += 1,
                None if c.file.kind != FileKind::File => skipped.unreadable += 1,
                None if c.file.size > MAX_READ => skipped.too_big += 1,
                None => items.push(Item { c, found }),
            }
        }
        let bytes_total = items.iter().map(|i| i.c.file.size).sum();
        ContentJob {
            query,
            open,
            items,
            next: 0,
            reading: None,
            hits,
            from_index,
            bytes_total,
            bytes_done: 0,
            skipped,
            first_error: None,
        }
    }

    /// The files to be read.
    pub fn total(&self) -> usize {
        self.items.len()
    }

    /// The files read so far.
    pub fn done(&self) -> usize {
        self.next
    }

    pub fn is_done(&self) -> bool {
        self.next >= self.items.len()
    }

    pub fn current(&self) -> Option<&Path> {
        self.items.get(self.next).map(|i| i.c.path.as_path())
    }

    pub fn found(&self) -> usize {
        self.hits.len()
    }

    fn skip(&mut self, why: fn(&mut Skipped) -> &mut usize, error: Option<String>) {
        *why(&mut self.skipped) += 1;
        if self.first_error.is_none() {
            self.first_error = error;
        }
        self.reading = None;
        self.bytes_done += self.items[self.next].c.file.size;
        self.next += 1;
    }

    /// Opens the file being read for its next term not known yet, or decides
    /// it: false when it is done with (decided, or left out).
    fn open_next(&mut self) -> bool {
        let item = &self.items[self.next];
        let first = item.found.iter().all(Option::is_none);
        let place = Place {
            dir: &item.c.dir,
            depth: item.c.depth,
        };
        let decided = self
            .query
            .decide(&item.c.file, Some(place), SystemTime::now(), &item.found);
        let term = item.found.iter().position(Option::is_none);
        let (Some(term), None) = (term, decided) else {
            if decided == Some(true) {
                self.hits.insert(item.c.path.clone());
            }
            self.bytes_done += item.c.file.size;
            self.next += 1;
            return false;
        };
        let opened = (self.open)(&item.c.path).and_then(|mut reader| {
            // The first reading looks whether it is text.
            let binary = first && {
                let start = reader.fill_buf()?;
                start[..start.len().min(BINARY_PROBE)].contains(&0)
            };
            Ok((binary, reader))
        });
        let scan = match &self.query.terms[term].words {
            None => Scan::Text(Vec::new()),
            Some(words) => Scan::Words(WordSplitter::default(), words.clone()),
        };
        match opened {
            Ok((true, _)) => self.skip(|s| &mut s.binary, None),
            Ok((false, reader)) => {
                self.reading = Some(Reading { term, reader, scan });
                return true;
            }
            Err(e) => {
                let error = format!("{}: {e}", item.c.path.display());
                self.skip(|s| &mut s.unreadable, Some(error));
            }
        }
        false
    }

    /// Reads until `budget` runs out, going on from where the last step
    /// stopped. A file is read again for each term it still needs, and not
    /// for one the others already decide.
    pub fn step(&mut self, budget: Duration) {
        let until = Instant::now() + budget;
        while self.next < self.items.len() {
            if self.reading.is_none() && !self.open_next() {
                continue;
            }
            let Some(mut reading) = self.reading.take() else {
                continue;
            };
            let scanned = match &mut reading.scan {
                Scan::Text(kept) => {
                    match Pattern::new(&SearchSpec::new(&self.query.terms[reading.term].text)) {
                        Ok(pattern) => pattern.scan(&mut reading.reader, kept, until),
                        Err(e) => Err(std::io::Error::other(e)),
                    }
                }
                Scan::Words(splitter, left) => {
                    scan_words(&mut reading.reader, splitter, left, until)
                }
            };
            match scanned {
                // Not finished: on in the next step.
                Ok(None) => {
                    self.reading = Some(reading);
                    break;
                }
                Ok(Some(has)) => self.items[self.next].found[reading.term] = Some(has),
                Err(e) => {
                    let error = format!("{}: {e}", self.items[self.next].c.path.display());
                    self.skip(|s| &mut s.unreadable, Some(error));
                }
            }
            if Instant::now() >= until {
                break;
            }
        }
    }

    /// The condition, with the files found so far: what the list then shows.
    pub fn finish(self) -> FileQuery {
        FileQuery {
            hits: Some(Arc::new(self.hits)),
            ..self.query
        }
    }

    /// What was read and what was left out, for the message at the end.
    pub fn summary(&self, stopped: bool) -> String {
        let n = self.found();
        let files = if n == 1 { "file meets" } else { "files meet" };
        let mut read = if stopped {
            format!(
                "stopped after reading {} of {} files",
                self.next,
                self.items.len()
            )
        } else {
            format!("{} files read, with no index", self.items.len())
        };
        if self.from_index > 0 {
            read.push_str(&format!("; {} answered by the index", self.from_index));
        }
        let mut s = format!("{n} {files} the condition ({read}");
        let skipped = self.skipped;
        if skipped.total() > 0 {
            let mut parts = Vec::new();
            for (count, what) in [
                (skipped.binary, "binary"),
                (skipped.links, "links"),
                (skipped.too_big, "over 256 MiB"),
                (skipped.unreadable, "unreadable"),
            ] {
                if count > 0 {
                    parts.push(format!("{count} {what}"));
                }
            }
            s.push_str(&format!("; left out: {}", parts.join(", ")));
        }
        s.push(')');
        if let Some(e) = &self.first_error {
            s.push_str(&format!(". First error: {e}"));
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn file(name: &str, size: u64) -> FileEntry {
        FileEntry {
            name: name.into(),
            size,
            disk: size,
            link: None,
            modified: None,
            mode: 0o644,
            kind: FileKind::File,
            tagged: false,
        }
    }

    fn takes(query: &str, f: &FileEntry) -> bool {
        let place = Place {
            dir: Path::new("/data/photos"),
            depth: 1,
        };
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000 * 86_400);
        FileQuery::parse(query, false)
            .unwrap()
            .matches(f, Some(place), now)
    }

    #[test]
    fn a_condition_takes_files_by_name_size_and_place() {
        let f = file("Beach.JPG", 20_000_000);
        assert!(takes("ext = 'jpg'", &f));
        assert!(takes("EXT = 'jpg' AND size > 10000000", &f));
        assert!(!takes("ext = 'jpg' AND size > 100000000", &f));
        assert!(takes("name LIKE 'Beach%'", &f));
        assert!(takes("ext IN ('png', 'jpg')", &f));
        assert!(takes("dir = '/data/photos' AND depth = 1", &f));
        // The path is joined as the system joins it (`\` on Windows).
        let path = Path::new("/data/photos").join("Beach.JPG");
        assert!(takes(&format!("path = '{}'", path.display()), &f));
        assert!(takes("path LIKE '%Beach.JPG'", &f));
        assert!(takes("mode = 644 AND kind = 'file'", &f));
        assert!(takes("NOT tagged", &f));
        // No extension is the empty text.
        assert!(takes("ext = ''", &file("Makefile", 1)));
    }

    #[test]
    fn dates_compare_as_text_and_as_an_age() {
        let mut f = file("a.txt", 1);
        // 990 days after 1970: ten days before `now` in `takes`.
        f.modified = Some(SystemTime::UNIX_EPOCH + Duration::from_secs(990 * 86_400));
        assert!(takes("age_days < 11 AND age_days > 9", &f));
        assert!(takes(
            "modified >= '1972-01-01' AND modified < '1973-01-01'",
            &f
        ));
        assert!(!takes("modified >= '1973-01-01'", &f));
        // A file with no date has neither.
        let undated = file("b.txt", 1);
        assert!(!takes("age_days < 11", &undated));
        assert!(takes("modified IS NULL", &undated));
    }

    #[test]
    fn with_ctrl_only_the_tagged_files_are_looked_at() {
        let mut f = file("a.txt", 1);
        let query = FileQuery::parse("size > 0", true).unwrap();
        assert!(!query.matches(&f, None, SystemTime::now()));
        f.tagged = true;
        assert!(query.matches(&f, None, SystemTime::now()));
        assert!(!query.needs_place());
        assert!(FileQuery::parse("depth > 2", false).unwrap().needs_place());
    }

    #[test]
    fn a_wrong_condition_says_what_is_wrong() {
        let err = FileQuery::parse("colour = 'red'", false).unwrap_err();
        assert!(
            err.starts_with("No column \"colour\": there are name, ext"),
            "{err}"
        );
        assert!(FileQuery::parse("size >", false).is_err());
        // A text compared with a number takes nothing, and does not fail.
        assert!(!takes("name > 5", &file("a", 1)));
    }

    #[test]
    fn contains_is_a_condition_of_its_own() {
        let q = FileQuery::parse("ext = 'md' AND contains('invoice')", false).unwrap();
        assert!(q.reads_contents() && q.needs_place());
        let q = FileQuery::parse("NOT contains('a') OR contains('b') OR contains('a')", false);
        let texts: Vec<String> = q.unwrap().terms().iter().map(|t| t.text.clone()).collect();
        assert_eq!(texts, ["a", "b"]);
        for (bad, says) in [
            ("contains('a') = 1", "a condition of its own"),
            ("contains(name)", "one text in quotes"),
            ("contains('')", "nothing to look for"),
            ("sum('a')", "No function sum()"),
            ("words('...')", "nothing to look for"),
        ] {
            let err = FileQuery::parse(bad, false).unwrap_err();
            assert!(err.contains(says), "{bad}: {err}");
        }
        // The table filter of the viewer takes no functions.
        assert!(SqlFilter::parse("contains('a')").is_err());
    }

    /// Files of `texts` in a folder, as candidates of the list.
    fn candidates(texts: &[(&str, &[u8])]) -> (tempfile::TempDir, Vec<Candidate>) {
        let tmp = crate::testing::tempdir();
        let dir = tmp.path().to_path_buf();
        let list = texts
            .iter()
            .map(|(name, text)| {
                std::fs::write(dir.join(name), text).unwrap();
                Candidate {
                    path: dir.join(name),
                    dir: dir.clone(),
                    depth: 0,
                    file: file(name, text.len() as u64),
                }
            })
            .collect();
        (tmp, list)
    }

    fn run(query: &str, list: Vec<Candidate>) -> ContentJob {
        let q = FileQuery::parse(query, false).unwrap();
        let mut job = ContentJob::new(
            q,
            list,
            Box::new(crate::search::open_file),
            SystemTime::now(),
            &|_| None,
        );
        while !job.is_done() {
            job.step(Duration::from_millis(50));
        }
        job
    }

    fn names(job: ContentJob) -> Vec<String> {
        let q = job.finish();
        let mut found: Vec<String> = q
            .hits
            .unwrap()
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        found.sort();
        found
    }

    #[test]
    fn only_the_files_the_rest_cannot_decide_are_read() {
        let (_tmp, list) = candidates(&[
            ("a.md", b"the Invoice of May\n"),
            ("b.md", b"nothing here\n"),
            ("c.txt", b"an invoice, but not markdown\n"),
        ]);
        let job = run("ext = 'md' AND contains('invoice')", list);
        assert_eq!(job.total(), 2, "c.txt is decided without reading it");
        assert!(
            job.summary(false)
                .starts_with("1 file meets the condition (2 files read, with no index")
        );
        assert_eq!(names(job), ["a.md"]);
    }

    #[test]
    fn several_terms_or_and_not() {
        let texts: &[(&str, &[u8])] = &[
            ("both", b"apple\nbanana\n"),
            ("apple", b"apple pie\n"),
            ("none", b"cherry\n"),
        ];
        let (_tmp, list) = candidates(texts);
        assert_eq!(
            names(run("contains('apple') AND contains('banana')", list)),
            ["both"]
        );
        let (_tmp, list) = candidates(texts);
        assert_eq!(names(run("NOT contains('apple')", list)), ["none"]);
        let (_tmp, list) = candidates(texts);
        // `name = 'none'` decides that one without reading it.
        let job = run("name = 'none' OR contains('banana')", list);
        assert_eq!(job.total(), 2);
        assert_eq!(names(job), ["both", "none"]);
    }

    #[test]
    fn binary_files_and_links_are_left_out_and_said() {
        let (tmp, mut list) = candidates(&[("text", b"key here\n"), ("bin", b"key\0here\n")]);
        let mut link = file("link", 4);
        link.kind = FileKind::Symlink;
        list.push(Candidate {
            path: tmp.path().join("link"),
            dir: tmp.path().to_path_buf(),
            depth: 0,
            file: link,
        });
        let job = run("contains('key')", list);
        assert_eq!(job.skipped.binary, 1);
        assert_eq!(job.skipped.links, 1);
        let said = job.summary(false);
        assert!(said.contains("left out: 1 binary, 1 links"), "{said}");
        assert_eq!(names(job), ["text"]);
    }

    #[test]
    fn a_file_gone_is_unreadable_and_the_rest_goes_on() {
        let (tmp, list) = candidates(&[("a", b"x\n"), ("b", b"x\n")]);
        std::fs::remove_file(tmp.path().join("a")).unwrap();
        let job = run("contains('x')", list);
        assert_eq!(job.skipped.unreadable, 1);
        assert!(job.summary(false).contains("First error:"));
        assert_eq!(names(job), ["b"]);
    }

    #[test]
    fn words_are_split_the_same_whole_or_in_pieces() {
        assert_eq!(
            words_of("Ação, FATURA-2024 e ünico_x"),
            ["acao", "fatura", "2024", "e", "unico", "x"]
        );
        let text = "uma Fatura de março, paga em 2024; ação concluída.\n".repeat(50);
        let mut whole = Vec::new();
        let mut splitter = WordSplitter::default();
        for piece in text.as_bytes().chunks(7) {
            splitter.feed(piece, |w| whole.push(w));
        }
        splitter.finish(|w| whole.push(w));
        assert_eq!(whole, words_of(&text));
    }

    #[test]
    fn words_finds_whole_words_and_contains_pieces() {
        let texts: &[(&str, &[u8])] = &[
            ("a", "Fatura de Março\n".as_bytes()),
            ("b", b"faturamento anual\n"),
        ];
        let (_tmp, list) = candidates(texts);
        assert_eq!(names(run("words('fatura marco')", list)), ["a"]);
        let (_tmp, list) = candidates(texts);
        assert_eq!(names(run("words('fatur*')", list)), ["a", "b"]);
        let (_tmp, list) = candidates(texts);
        assert_eq!(names(run("words('fatur* anual')", list)), ["b"]);
        let (_tmp, list) = candidates(texts);
        assert_eq!(names(run("contains('fatura')", list)), ["a", "b"]);
    }

    #[test]
    fn what_an_index_answers_is_not_read() {
        let (_tmp, list) = candidates(&[("a", b"apple\n"), ("b", b"apple\n")]);
        let q = FileQuery::parse("words('apple')", false).unwrap();
        // The index says "a" has no apple: it is believed, and not read.
        let index = |c: &Candidate| (c.file.name == "a").then(|| Known::Terms(vec![Some(false)]));
        let mut job = ContentJob::new(
            q,
            list,
            Box::new(crate::search::open_file),
            SystemTime::now(),
            &index,
        );
        assert_eq!((job.total(), job.from_index), (1, 1));
        while !job.is_done() {
            job.step(Duration::from_millis(50));
        }
        assert!(job.summary(false).contains("1 answered by the index"));
        assert_eq!(names(job), ["b"]);
    }
}
