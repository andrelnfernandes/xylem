//! Text or byte search: in tagged files (Ctrl+S) and inside the
//! viewer (F, B, /, \, +, -, Space), as in ZTreeWin.
//!
//! Text is searched line by line: a `*` in the middle of the term matches anything
//! on the same line (`erro*disco`), and a term never crosses a
//! line break. When ignoring case, accents count too
//! (`AÇÃO` finds `ação`).

use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Lines longer than this (e.g. binary files) are searched in
/// chunks; a term split between two chunks may not be found.
const MAX_LINE: usize = 1024 * 1024;

/// What to search for and how.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchSpec {
    pub text: String,
    pub case_sensitive: bool,
    /// `text` is hexadecimal pairs: "4d 5a 90".
    pub hex: bool,
}

impl SearchSpec {
    pub fn new(text: &str) -> Self {
        SearchSpec {
            text: text.to_string(),
            case_sensitive: false,
            hex: false,
        }
    }

    /// E.g. "text, ignore case".
    pub fn mode_label(&self) -> &'static str {
        match (self.hex, self.case_sensitive) {
            (true, _) => "hex",
            (false, true) => "text, match case",
            (false, false) => "text, ignore case",
        }
    }
}

/// A term ready to search for.
#[derive(Debug, Clone)]
pub enum Pattern {
    /// Parts separated by the inner `*`, already lowercased if applicable.
    Text {
        parts: Vec<Vec<char>>,
        case_sensitive: bool,
    },
    Bytes(Vec<u8>),
}

impl Pattern {
    pub fn new(spec: &SearchSpec) -> Result<Self, String> {
        if spec.text.is_empty() {
            return Err("Empty search".into());
        }
        if spec.hex {
            let bytes = spec
                .text
                .split_whitespace()
                .map(|h| u8::from_str_radix(h, 16).map_err(|_| format!("invalid hex byte '{h}'")))
                .collect::<Result<Vec<u8>, String>>()?;
            if bytes.is_empty() {
                return Err("Empty search".into());
            }
            return Ok(Pattern::Bytes(bytes));
        }
        let case = spec.case_sensitive;
        let norm = |s: &str| -> Vec<char> { s.chars().map(|c| fold(c, case)).collect() };
        // `*` is a wildcard only between two characters; at the ends it is literal,
        // and then so are all the others (ZTreeWin rule).
        let t = &spec.text;
        let wildcards = !t.starts_with('*') && !t.ends_with('*');
        let parts = if wildcards {
            t.split('*').filter(|p| !p.is_empty()).map(norm).collect()
        } else {
            vec![norm(t)]
        };
        Ok(Pattern::Text {
            parts,
            case_sensitive: case,
        })
    }

    /// First occurrence in a line, starting at character `from`:
    /// returns (start, end) in characters.
    pub fn find_in_line(&self, line: &str, from: usize) -> Option<(usize, usize)> {
        let chars: Vec<char> = line.chars().collect();
        self.find_in_chars(&chars, from)
    }

    /// Last occurrence that starts before character `before`.
    pub fn rfind_in_line(&self, line: &str, before: usize) -> Option<(usize, usize)> {
        let chars: Vec<char> = line.chars().collect();
        let mut best = None;
        let mut from = 0;
        while let Some((s, e)) = self.find_in_chars(&chars, from) {
            if s >= before {
                break;
            }
            best = Some((s, e));
            from = s + 1;
        }
        best
    }

    fn find_in_chars(&self, chars: &[char], from: usize) -> Option<(usize, usize)> {
        match self {
            Pattern::Text {
                parts,
                case_sensitive,
            } => {
                let line: Vec<char> = chars.iter().map(|&c| fold(c, *case_sensitive)).collect();
                // Each part as far left as possible after the previous one:
                // if it doesn't match that way, starting further right won't either.
                let first = find_chars(&line, &parts[0], from)?;
                let mut end = first + parts[0].len();
                for part in &parts[1..] {
                    end = find_chars(&line, part, end)? + part.len();
                }
                Some((first, end))
            }
            Pattern::Bytes(bytes) => {
                // Bytes in a text line: compares against the UTF-8 bytes.
                let text: String = chars.iter().collect();
                let hay = text.as_bytes();
                let pos = find_bytes(hay, bytes, char_to_byte(&text, from))?;
                let s = text[..pos].chars().count();
                let e = text[..pos + bytes.len()].chars().count();
                Some((s, e))
            }
        }
    }

    /// Byte position of the next occurrence in `data` starting at `from`
    /// (for the viewer's hex mode).
    pub fn find_in_bytes(&self, data: &[u8], from: usize) -> Option<(usize, usize)> {
        let needle = self.as_bytes();
        let pos = match self {
            Pattern::Text {
                case_sensitive: false,
                ..
            } => find_bytes_ignore_ascii_case(data, &needle, from)?,
            _ => find_bytes(data, &needle, from)?,
        };
        Some((pos, pos + needle.len()))
    }

    /// Last occurrence in `data` that starts before `before`.
    pub fn rfind_in_bytes(&self, data: &[u8], before: usize) -> Option<(usize, usize)> {
        let mut best = None;
        let mut from = 0;
        while let Some((s, e)) = self.find_in_bytes(data, from) {
            if s >= before {
                break;
            }
            best = Some((s, e));
            from = s + 1;
        }
        best
    }

    /// The term as bytes (the text with literal `*`).
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Pattern::Bytes(b) => b.clone(),
            Pattern::Text { parts, .. } => {
                let joined: Vec<String> = parts.iter().map(|p| p.iter().collect()).collect();
                joined.join("*").into_bytes()
            }
        }
    }

    /// Does the content have the term? Reads until `until`, at least one block or
    /// line: `None` if it has not found out yet. `kept` is what the next call
    /// needs of what was read, and starts empty. A reader that has nothing yet
    /// (`WouldBlock`) also means `None`, with nothing lost.
    fn scan(
        &self,
        reader: &mut dyn BufRead,
        kept: &mut Vec<u8>,
        until: Instant,
    ) -> io::Result<Option<bool>> {
        let found = match self {
            Pattern::Bytes(needle) => scan_bytes(reader, needle, kept, until),
            Pattern::Text { .. } => self.scan_lines(reader, kept, until),
        };
        match found {
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
            other => other,
        }
    }

    /// Text is read line by line; `line` is the one not yet complete.
    fn scan_lines(
        &self,
        reader: &mut dyn BufRead,
        line: &mut Vec<u8>,
        until: Instant,
    ) -> io::Result<Option<bool>> {
        loop {
            let room = (MAX_LINE - line.len()) as u64;
            let n = Read::take(&mut *reader, room).read_until(b'\n', line)?;
            if n == 0 || line.ends_with(b"\n") || line.len() >= MAX_LINE {
                if self
                    .find_in_line(&String::from_utf8_lossy(line), 0)
                    .is_some()
                {
                    return Ok(Some(true));
                }
                line.clear();
            }
            if n == 0 {
                return Ok(Some(false));
            }
            if Instant::now() >= until {
                return Ok(None);
            }
        }
    }

    /// Does the content have the term, however long it takes?
    pub fn matches_reader(&self, mut reader: impl BufRead) -> io::Result<bool> {
        let mut kept = Vec::new();
        loop {
            if let Some(found) = self.scan(&mut reader, &mut kept, Instant::now())? {
                return Ok(found);
            }
        }
    }
}

/// Lowercase, if case-insensitive (`Ç` → `ç`).
fn fold(c: char, case_sensitive: bool) -> char {
    if case_sensitive {
        c
    } else {
        c.to_lowercase().next().unwrap_or(c)
    }
}

fn find_chars(hay: &[char], needle: &[char], from: usize) -> Option<usize> {
    if needle.is_empty() || from > hay.len() || needle.len() > hay.len() - from {
        return None;
    }
    (from..=hay.len() - needle.len()).find(|&i| hay[i..i + needle.len()] == *needle)
}

fn find_bytes(hay: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if needle.is_empty() || from > hay.len() || needle.len() > hay.len() - from {
        return None;
    }
    hay[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + from)
}

fn find_bytes_ignore_ascii_case(hay: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if needle.is_empty() || from > hay.len() || needle.len() > hay.len() - from {
        return None;
    }
    hay[from..]
        .windows(needle.len())
        .position(|w| w.eq_ignore_ascii_case(needle))
        .map(|p| p + from)
}

fn char_to_byte(s: &str, chars: usize) -> usize {
    s.char_indices().nth(chars).map_or(s.len(), |(i, _)| i)
}

/// Searches for bytes in a stream, in overlapping blocks (to find the term
/// even when split between two blocks); `carry` is the end of the last block.
fn scan_bytes(
    reader: &mut dyn BufRead,
    needle: &[u8],
    carry: &mut Vec<u8>,
    until: Instant,
) -> io::Result<Option<bool>> {
    loop {
        let chunk = reader.fill_buf()?;
        if chunk.is_empty() {
            return Ok(Some(false));
        }
        let n = chunk.len();
        carry.extend_from_slice(chunk);
        reader.consume(n);
        if find_bytes(carry, needle, 0).is_some() {
            return Ok(Some(true));
        }
        let keep = needle.len().saturating_sub(1).min(carry.len());
        carry.drain(..carry.len() - keep);
        if Instant::now() >= until {
            return Ok(None);
        }
    }
}

/// Opens a file's content for the search.
pub type Opener = Box<dyn Fn(&Path) -> io::Result<Box<dyn BufRead>>>;

/// The file being searched, from one step to the next.
struct Reading {
    reader: Box<dyn BufRead>,
    kept: Vec<u8>,
}

/// A search over the tagged files in progress (Ctrl+S). Each item
/// carries a caller-supplied identifier (`T`), returned in the result.
pub struct SearchJob<T> {
    pub spec: SearchSpec,
    pattern: Pattern,
    items: Vec<(T, PathBuf)>,
    /// Reads an item's content (from disk, or from inside a ZIP/TAR).
    open: Opener,
    next: usize,
    /// The open reader of item `next`: a huge file takes many steps.
    reading: Option<Reading>,
    pub found: usize,
    pub errors: Vec<String>,
}

impl<T: Copy> SearchJob<T> {
    pub fn new(spec: SearchSpec, items: Vec<(T, PathBuf)>, open: Opener) -> Result<Self, String> {
        Ok(SearchJob {
            pattern: Pattern::new(&spec)?,
            spec,
            items,
            open,
            next: 0,
            reading: None,
            found: 0,
            errors: Vec::new(),
        })
    }

    pub fn total(&self) -> usize {
        self.items.len()
    }

    pub fn done(&self) -> usize {
        self.next
    }

    pub fn is_done(&self) -> bool {
        self.next >= self.items.len()
    }

    pub fn current(&self) -> Option<&Path> {
        self.items.get(self.next).map(|(_, p)| p.as_path())
    }

    /// Searches until `budget` runs out, going on from where the last step
    /// stopped. Returns, for each file finished, whether it contains the term.
    /// Files that could not be read stay tagged (and the error is kept), like
    /// ZTreeWin's "Keep".
    pub fn step(&mut self, budget: Duration) -> Vec<(T, bool)> {
        let until = Instant::now() + budget;
        let mut results = Vec::new();
        while let Some((id, path)) = self.items.get(self.next) {
            let reading = match self.reading.take() {
                Some(reading) => Ok(reading),
                None => (self.open)(path).map(|reader| Reading {
                    reader,
                    kept: Vec::new(),
                }),
            };
            let scanned = reading.and_then(|mut r| {
                let found = self.pattern.scan(&mut r.reader, &mut r.kept, until)?;
                Ok((found, r))
            });
            match scanned {
                Ok((None, reading)) => {
                    self.reading = Some(reading);
                    break;
                }
                Ok((Some(f), _)) => {
                    self.found += usize::from(f);
                    results.push((*id, f));
                }
                Err(e) => self.errors.push(format!("{}: {e}", path.display())),
            }
            self.next += 1;
            if Instant::now() >= until {
                break;
            }
        }
        results
    }

    pub fn summary(&self) -> String {
        let mut s = format!(
            "{} of {} tagged files contain \"{}\"",
            self.found, self.next, self.spec.text
        );
        if let Some(e) = self.errors.first() {
            s.push_str(&format!(
                " ({} unreadable, kept tagged; first: {e})",
                self.errors.len()
            ));
        }
        s
    }
}

/// Opens a file from disk for the search.
pub fn open_file(path: &Path) -> io::Result<Box<dyn BufRead>> {
    Ok(Box::new(BufReader::new(File::open(path)?)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn pat(text: &str) -> Pattern {
        Pattern::new(&SearchSpec::new(text)).unwrap()
    }

    #[test]
    fn finds_text_ignoring_case_including_accents() {
        assert_eq!(pat("ação").find_in_line("Uma AÇÃO aqui", 0), Some((4, 8)));
        assert_eq!(pat("x").find_in_line("axbx", 2), Some((3, 4)));
        assert_eq!(pat("x").rfind_in_line("axbx", 3), Some((1, 2)));
        let case = Pattern::new(&SearchSpec {
            case_sensitive: true,
            ..SearchSpec::new("Ação")
        })
        .unwrap();
        assert_eq!(case.find_in_line("AÇÃO", 0), None);
    }

    #[test]
    fn wildcard_only_inside_the_term() {
        assert_eq!(
            pat("erro*disco").find_in_line("um erro no disco!", 0),
            Some((3, 16))
        );
        assert_eq!(pat("erro*disco").find_in_line("disco e erro", 0), None);
        // At the ends, `*` is literal (and so are all the others).
        assert_eq!(pat("*a*b").find_in_line("x *a*b y", 0), Some((2, 6)));
        assert_eq!(pat("*a*b").find_in_line("x a  b y", 0), None);
    }

    #[test]
    fn hex_patterns() {
        let spec = SearchSpec {
            hex: true,
            ..SearchSpec::new("4d 5A")
        };
        let p = Pattern::new(&spec).unwrap();
        assert_eq!(p.find_in_bytes(b"xxMZyy", 0), Some((2, 4)));
        assert!(p.matches_reader(Cursor::new(b"..MZ..".to_vec())).unwrap());
        assert!(
            Pattern::new(&SearchSpec {
                hex: true,
                ..SearchSpec::new("zz")
            })
            .is_err()
        );
        assert!(Pattern::new(&SearchSpec::new("")).is_err());
    }

    #[test]
    fn text_in_bytes_ignores_ascii_case() {
        assert_eq!(pat("abc").find_in_bytes(b"..ABC..abc", 0), Some((2, 5)));
        assert_eq!(pat("abc").rfind_in_bytes(b"..ABC..abc", 9), Some((7, 10)));
    }

    #[test]
    fn readers_do_not_cross_lines_but_find_split_bytes() {
        let p = pat("foo bar");
        assert!(
            p.matches_reader(Cursor::new(b"a\nfoo bar\nz".to_vec()))
                .unwrap()
        );
        assert!(!p.matches_reader(Cursor::new(b"foo\nbar".to_vec())).unwrap());
        // Bytes split between two reader blocks are still found.
        let needle = Pattern::Bytes(b"XYZ".to_vec());
        let reader = BufReader::with_capacity(4, Cursor::new(b"aaXYZbb".to_vec()));
        assert!(needle.matches_reader(reader).unwrap());
    }

    /// Gives its blocks one by one, with `WouldBlock` before each.
    struct Hesitant {
        blocks: Vec<&'static [u8]>,
        ready: bool,
        current: &'static [u8],
    }

    impl Read for Hesitant {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let block = self.fill_buf()?;
            let n = block.len().min(buf.len());
            buf[..n].copy_from_slice(&block[..n]);
            self.consume(n);
            Ok(n)
        }
    }

    impl BufRead for Hesitant {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            if self.current.is_empty() && !self.blocks.is_empty() {
                if !self.ready {
                    self.ready = true;
                    return Err(io::Error::from(io::ErrorKind::WouldBlock));
                }
                self.ready = false;
                self.current = self.blocks.remove(0);
            }
            Ok(self.current)
        }

        fn consume(&mut self, n: usize) {
            self.current = &self.current[n..];
        }
    }

    fn hesitant_job(spec: SearchSpec, blocks: Vec<&'static [u8]>) -> SearchJob<u8> {
        let open = std::cell::Cell::new(Some(blocks));
        let opener: Opener = Box::new(move |_: &Path| {
            let blocks = open.take().expect("opened only once");
            Ok(Box::new(Hesitant {
                blocks,
                ready: false,
                current: &[],
            }) as Box<dyn BufRead>)
        });
        SearchJob::new(spec, vec![(1, PathBuf::from("x"))], opener).unwrap()
    }

    fn run_steps(job: &mut SearchJob<u8>) -> (usize, Vec<(u8, bool)>) {
        let (mut steps, mut results) = (0, Vec::new());
        while !job.is_done() {
            results.extend(job.step(Duration::ZERO));
            steps += 1;
        }
        (steps, results)
    }

    #[test]
    fn a_line_split_by_a_reader_with_nothing_yet_is_still_one_line() {
        let spec = SearchSpec::new("foo bar");
        let mut job = hesitant_job(spec, vec![b"a\nfoo ", b"bar\nz"]);
        let (steps, results) = run_steps(&mut job);
        assert_eq!(results, [(1, true)]);
        assert!(steps > 1, "it gave the screen back: {steps}");
        assert!(job.errors.is_empty());

        // Across a line break it is not a match.
        let mut job = hesitant_job(SearchSpec::new("foo bar"), vec![b"foo\n", b"bar\n"]);
        assert_eq!(run_steps(&mut job).1, [(1, false)]);
    }

    #[test]
    fn bytes_split_by_a_reader_with_nothing_yet_are_still_found() {
        let spec = SearchSpec {
            hex: true,
            ..SearchSpec::new("58 59 5a")
        };
        let mut job = hesitant_job(spec, vec![b"aaX", b"YZbb"]);
        assert_eq!(run_steps(&mut job).1, [(1, true)]);
    }

    #[test]
    fn a_huge_file_takes_many_steps_with_its_reader_open() {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("big");
        let mut body = "uma linha sem nada\n".repeat(50_000);
        body.push_str("a CHAVE no fim\n");
        std::fs::write(&path, &body).unwrap();
        let opened = std::rc::Rc::new(std::cell::Cell::new(0));
        let count = opened.clone();
        let opener: Opener = Box::new(move |p: &Path| {
            count.set(count.get() + 1);
            open_file(p)
        });
        let mut job = SearchJob::new(SearchSpec::new("chave"), vec![(1, path)], opener).unwrap();
        let (steps, results) = run_steps(&mut job);
        assert_eq!(results, [(1, true)]);
        assert!(steps > 1000, "one line per step at zero budget: {steps}");
        assert_eq!(opened.get(), 1, "the reader stays open between steps");
    }

    #[test]
    fn a_line_over_the_limit_is_searched_in_chunks() {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("long");
        let mut body = vec![b'x'; MAX_LINE * 2 + 10];
        body.extend_from_slice(b"chave");
        std::fs::write(&path, &body).unwrap();
        let items = vec![(1, path)];
        let mut job = SearchJob::new(SearchSpec::new("CHAVE"), items, Box::new(open_file)).unwrap();
        assert_eq!(run_steps(&mut job).1, [(1, true)]);
    }

    #[test]
    fn job_reports_matches_and_errors() {
        let tmp = crate::testing::tempdir();
        let (a, b) = (tmp.path().join("a"), tmp.path().join("b"));
        std::fs::write(&a, "tem a palavra chave\n").unwrap();
        std::fs::write(&b, "não tem\n").unwrap();
        let missing = tmp.path().join("missing");
        let items = vec![(1, a), (2, b), (3, missing)];
        let mut job = SearchJob::new(SearchSpec::new("CHAVE"), items, Box::new(open_file)).unwrap();
        let mut results = Vec::new();
        while !job.is_done() {
            results.extend(job.step(Duration::ZERO));
        }
        assert_eq!(results, [(1, true), (2, false)]);
        assert_eq!(job.errors.len(), 1);
        assert!(
            job.summary()
                .starts_with("1 of 3 tagged files contain \"CHAVE\" (1 unreadable")
        );
    }
}
