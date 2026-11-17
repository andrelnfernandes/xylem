//! A big text seen through a window, as `less` does: the viewer keeps the
//! lines near the screen and asks the file for the others as it moves.
//!
//! - an index of lines is made in another thread: the byte where one line in
//!   every [`MARK_EVERY`] starts, so reaching line N reads from the mark before;
//! - the lines near the screen are read in another thread too, and one not
//!   there yet shows as [`MISSING`] for a moment;
//! - the search and the line filter go over the file, not over memory, each
//!   in its thread, and Esc stops them.
//!
//! Nothing here grows with the file: the window, the index and the matches of
//! the filter have their limits. The lines are made by the viewer's `put_line`,
//! as those of a small file are, so the screen has no new way to hostile bytes.

use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};
use std::ops::{Index, Range};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use super::line_filter::LineMatcher;
use super::lines::{LineSource, Lines};
use super::records::{Stamp, file_stamp};
use super::{StopOnDrop, put_line};
use crate::charset::{self, Charset};
use crate::search::Pattern;

/// A file on disk bigger than this is seen through a window.
const WINDOW_OVER: u64 = 16 << 20;
/// How much of the file is read at once to open it: its first lines, and what
/// the encoding is guessed from.
const HEAD: u64 = 1 << 20;
#[cfg(test)]
thread_local! {
    /// A test's own sizes for [`WINDOW_OVER`] and [`HEAD`].
    pub static TEST_SIZES: std::cell::Cell<Option<(u64, u64)>> = const { std::cell::Cell::new(None) };
}

/// [`WINDOW_OVER`], unless a test says otherwise.
pub fn window_over() -> u64 {
    #[cfg(test)]
    if let Some((over, _)) = TEST_SIZES.get() {
        return over;
    }
    WINDOW_OVER
}

/// [`HEAD`], unless a test says otherwise.
pub fn head() -> u64 {
    #[cfg(test)]
    if let Some((_, head)) = TEST_SIZES.get() {
        return head;
    }
    HEAD
}

/// One line in this many has its place in the index.
const MARK_EVERY: usize = 256;
/// The most marks the index keeps: 32 MiB, about a billion lines. Lines after
/// that are still reached, from the last mark.
const MAX_MARKS: usize = 1 << 22;
/// What a thread reads of the file at a time.
const BLOCK: usize = 1 << 20;
/// The bytes of a line that are shown; the rest of it only in hex.
pub const MAX_LINE: usize = 1 << 20;
/// The lines kept before and after the top of the screen.
pub const AROUND: usize = 1_000;
/// The window is asked for again once the screen moved this far.
const AGAIN: usize = AROUND / 4;
/// The bytes of the lines kept that are not on screen.
const WINDOW_BYTES: usize = 32 << 20;
/// The most lines a filter keeps, as in the tables.
pub const MAX_MATCHES: usize = super::table_window::MAX_MATCHES;
/// The bytes of the hex mode kept around the screen.
const HEX_BLOCK: u64 = 256 << 10;
/// What a search in hex reads at a time, and how much of it is read again so
/// that a match across two readings is found.
const SEARCH_BLOCK: u64 = 4 << 20;
const SEARCH_OVERLAP: u64 = 64 << 10;
/// How long the screen waits for lines it asked for before showing [`MISSING`].
/// A test wants them when the call returns.
const FETCH_WAIT: Duration = if cfg!(test) {
    Duration::from_secs(60)
} else {
    Duration::from_millis(30)
};
/// How often the filter hands over what it found.
const FLUSH: Duration = Duration::from_millis(50);
/// What a line not read yet shows.
pub const MISSING: &str = "…";

/// What ends a line in the file: one byte, or two in UTF-16.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Unit {
    Byte,
    Utf16Le,
    Utf16Be,
}

impl Unit {
    fn of(charset: Charset) -> Self {
        match charset {
            Charset::Utf16Le => Unit::Utf16Le,
            Charset::Utf16Be => Unit::Utf16Be,
            _ => Unit::Byte,
        }
    }

    fn newline(self) -> [u8; 2] {
        match self {
            Unit::Utf16Be => [0, b'\n'],
            _ => [b'\n', 0],
        }
    }
}

/// What the threads of a window share: the file and its index.
struct Shared {
    path: PathBuf,
    /// The size and date the file had when it was opened.
    stamp: Stamp,
    size: u64,
    unit: Unit,
    /// The bidi controls are counted by their bytes: only in UTF-8.
    utf8: bool,
    index: Mutex<LineIndex>,
}

#[derive(Default)]
struct LineIndex {
    /// The byte where line `k * MARK_EVERY` starts, for each `k` so far.
    marks: Vec<u64>,
    /// The lines counted so far.
    lines: usize,
    /// How far the count went.
    scanned: u64,
    /// The count reached the end of the file.
    done: bool,
    /// The bidi controls the count found.
    bidi: usize,
    /// The file is no longer the one that was opened.
    changed: bool,
    /// What was scanned ends in a line end.
    ends_in_newline: bool,
    /// The last two bytes scanned: a bidi control may start in them.
    tail: Vec<u8>,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, LineIndex> {
        self.index.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The last mark at or before `line`: its byte and its line.
    fn mark_before(&self, line: usize) -> (u64, usize) {
        let index = self.lock();
        let k = (line / MARK_EVERY).min(index.marks.len().saturating_sub(1));
        (index.marks.get(k).copied().unwrap_or(0), k * MARK_EVERY)
    }

    /// The mark of the block of `line`, if the count got there.
    fn mark_of(&self, line: usize) -> Option<u64> {
        self.lock().marks.get(line / MARK_EVERY).copied()
    }

    fn still(&self) -> bool {
        file_stamp(&self.path) == self.stamp
    }
}

/// How much of `head`, the start of a file, is whole lines of `charset`.
pub fn whole_lines(head: &[u8], charset: Charset) -> usize {
    let mut whole = 0;
    line_ends(Unit::of(charset), head, |end| whole = end);
    whole
}

/// Reads until `buf` is full or the file ends.
fn read_full(file: &mut impl Read, buf: &mut [u8]) -> io::Result<usize> {
    let mut n = 0;
    while n < buf.len() {
        match file.read(&mut buf[n..]) {
            Ok(0) => break,
            Ok(k) => n += k,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(n)
}

/// Whether a bidi control starts at `i` of `data`: U+061C, U+200E, U+200F,
/// U+202A to U+202E and U+2066 to U+2069, in UTF-8.
fn bidi_at(data: &[u8], i: usize) -> bool {
    match data.get(i..) {
        Some([0xD8, 0x9C, ..]) => true,
        Some([0xE2, 0x80, c, ..]) => matches!(c, 0x8E | 0x8F | 0xAA..=0xAE),
        Some([0xE2, 0x81, c, ..]) => matches!(c, 0xA6..=0xA9),
        _ => false,
    }
}

/// The bidi controls that start in `data`, whole.
fn count_bidi(data: &[u8]) -> usize {
    let mut n = 0;
    for lead in [0xD8, 0xE2] {
        n += memchr::memchr_iter(lead, data)
            .filter(|&i| bidi_at(data, i))
            .count();
    }
    n
}

/// Calls `at` with where each line of `data` ends (the byte after its end).
fn line_ends(unit: Unit, data: &[u8], mut at: impl FnMut(usize)) {
    match unit {
        Unit::Byte => memchr::memchr_iter(b'\n', data).for_each(|i| at(i + 1)),
        _ => {
            let newline = unit.newline();
            for (k, pair) in data.as_chunks::<2>().0.iter().enumerate() {
                if *pair == newline {
                    at(k * 2 + 2);
                }
            }
        }
    }
}

/// Counts the lines of the file from where the count stopped, noting the marks
/// and the bidi controls, until the end or `stop`.
fn count(shared: &Shared, stop: &AtomicBool) {
    let (mut pos, mut lines, mut tail) = {
        let index = shared.lock();
        if index.done {
            return;
        }
        (index.scanned, index.lines, index.tail.clone())
    };
    let opened = File::open(&shared.path).and_then(|mut f| {
        f.seek(SeekFrom::Start(pos))?;
        Ok(f)
    });
    let Ok(mut file) = opened else {
        shared.lock().changed = true;
        return;
    };
    let mut block = vec![0u8; BLOCK];
    loop {
        if stop.load(Ordering::Relaxed) {
            return;
        }
        let n = match read_full(&mut file, &mut block) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => {
                shared.lock().changed = true;
                return;
            }
        };
        let data = &block[..n];
        let mut marks = Vec::new();
        let mut last_end = 0;
        line_ends(shared.unit, data, |end| {
            lines += 1;
            last_end = end;
            if lines.is_multiple_of(MARK_EVERY) {
                marks.push((lines / MARK_EVERY, pos + end as u64));
            }
        });
        let mut bidi = 0;
        if shared.utf8 {
            bidi = count_bidi(data);
            // One that started in the block before.
            let mut joint = tail.clone();
            joint.extend_from_slice(&data[..n.min(2)]);
            bidi += (0..tail.len()).filter(|&i| bidi_at(&joint, i)).count();
            // Counted whole in this block: not again with the next.
            let keep = (n.saturating_sub(2)..n).filter(|&i| !bidi_at(data, i));
            tail = keep.map(|i| data[i]).collect();
        }
        pos += n as u64;
        let mut index = shared.lock();
        index.lines = lines;
        index.scanned = pos;
        index.bidi += bidi;
        index.ends_in_newline = last_end == n;
        index.tail.clone_from(&tail);
        for (k, at) in marks {
            // Only the next one: a full index stops taking them.
            if k == index.marks.len() && k < MAX_MARKS {
                index.marks.push(at);
            }
        }
    }
    let still = shared.still();
    let mut index = shared.lock();
    // The last line may have no end.
    if pos > 0 && !index.ends_in_newline {
        index.lines += 1;
    }
    index.done = true;
    index.changed |= !still;
}

/// Reads the lines of the file one after the other, from a mark.
struct LineReader<'a> {
    shared: &'a Shared,
    file: BufReader<File>,
    /// The number of the line read next.
    next: usize,
}

impl<'a> LineReader<'a> {
    /// A reader about to read `line`.
    fn at(shared: &'a Shared, line: usize) -> io::Result<Option<Self>> {
        let (byte, first) = shared.mark_before(line);
        let mut file = File::open(&shared.path)?;
        file.seek(SeekFrom::Start(byte))?;
        let mut reader = LineReader {
            shared,
            file: BufReader::with_capacity(256 << 10, file),
            next: first,
        };
        Ok(reader.go_to(line)?.then_some(reader))
    }

    /// Moves on to `line`, at or after the next one. False if the file ends
    /// before it.
    fn go_to(&mut self, line: usize) -> io::Result<bool> {
        if line / MARK_EVERY > self.next / MARK_EVERY
            && let Some(byte) = self.shared.mark_of(line)
        {
            self.file.seek(SeekFrom::Start(byte))?;
            self.next = line / MARK_EVERY * MARK_EVERY;
        }
        let mut skipped = Vec::new();
        while self.next < line {
            if self.read(&mut skipped, 0)?.is_none() {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Reads the next line into `buf`, without its end and at most `cap` bytes
    /// of it: true if it was longer. `None` at the end of the file.
    fn read(&mut self, buf: &mut Vec<u8>, cap: usize) -> io::Result<Option<bool>> {
        buf.clear();
        let mut cut = false;
        let mut any = false;
        if self.shared.unit == Unit::Byte {
            loop {
                let avail = self.file.fill_buf()?;
                if avail.is_empty() {
                    break;
                }
                any = true;
                let (take, ended) = match memchr::memchr(b'\n', avail) {
                    Some(i) => (i, true),
                    None => (avail.len(), false),
                };
                let room = cap - buf.len();
                cut |= take > room;
                buf.extend_from_slice(&avail[..take.min(room)]);
                self.file.consume(take + usize::from(ended));
                if ended {
                    break;
                }
            }
        } else {
            let newline = self.shared.unit.newline();
            let mut pair = [0u8; 2];
            loop {
                // An odd byte at the end of the file is left out.
                if read_full(&mut self.file, &mut pair)? < 2 {
                    break;
                }
                any = true;
                if pair == newline {
                    break;
                }
                if buf.len() + 2 <= cap {
                    buf.extend_from_slice(&pair);
                } else {
                    cut = true;
                }
            }
        }
        if !any {
            return Ok(None);
        }
        self.next += 1;
        Ok(Some(cut))
    }
}

/// Writes the line read from the file (`raw`, `cut` if not all of it) as the
/// next line of `lines`, as the screen shows it.
fn put_raw(
    lines: &mut Lines,
    bidi: &mut Vec<(usize, usize)>,
    num: usize,
    raw: &[u8],
    cut: bool,
    charset: Charset,
) {
    let text = charset::decode(raw, charset);
    let text = text.strip_suffix('\r').unwrap_or(&text);
    let rest = put_line(lines, bidi, num, text, MAX_LINE);
    if cut || !rest.is_empty() {
        lines.push('…');
    }
    lines.end_line();
}

/// One line as the screen shows it, for a search or a filter to look at.
#[derive(Default)]
struct Shown {
    lines: Lines,
    bidi: Vec<(usize, usize)>,
}

impl Shown {
    fn of(&mut self, raw: &[u8], cut: bool, charset: Charset) -> &str {
        self.lines.clear();
        self.bidi.clear();
        put_raw(&mut self.lines, &mut self.bidi, 0, raw, cut, charset);
        &self.lines[0]
    }
}

/// Lines read for the window.
struct Fetched {
    /// The number of each line, in order.
    nums: Vec<usize>,
    lines: Lines,
    /// Where a bidi control was swapped: (line, column), in order.
    bidi: Vec<(usize, usize)>,
    changed: bool,
}

/// Reads the lines `wanted` (in order). Those at `screen` (positions in
/// `wanted`) always come; the others, while they fit in [`WINDOW_BYTES`].
fn fetch(
    shared: &Shared,
    charset: Charset,
    wanted: &[usize],
    screen: Range<usize>,
    stop: &AtomicBool,
) -> Fetched {
    let mut got = Fetched {
        nums: Vec::with_capacity(wanted.len()),
        lines: Lines::default(),
        bidi: Vec::new(),
        changed: !shared.still(),
    };
    let Some(&first) = wanted.first() else {
        return got;
    };
    let Ok(Some(mut reader)) = LineReader::at(shared, first) else {
        return got;
    };
    let mut raw = Vec::new();
    for (k, &num) in wanted.iter().enumerate() {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        if got.lines.bytes() > WINDOW_BYTES && !screen.contains(&k) {
            if k >= screen.end {
                break;
            }
            continue;
        }
        match reader.go_to(num).and_then(|there| {
            if there {
                reader.read(&mut raw, MAX_LINE)
            } else {
                Ok(None)
            }
        }) {
            Ok(Some(cut)) => {
                put_raw(&mut got.lines, &mut got.bidi, num, &raw, cut, charset);
                got.nums.push(num);
            }
            _ => break,
        }
    }
    got.changed |= !shared.still();
    got
}

fn fetch_hex(shared: &Shared, offset: u64) -> io::Result<Vec<u8>> {
    let mut file = File::open(&shared.path)?;
    file.seek(SeekFrom::Start(offset))?;
    let mut block = vec![0u8; HEX_BLOCK as usize];
    let n = read_full(&mut file, &mut block)?;
    block.truncate(n);
    Ok(block)
}

/// Where a search over the file starts and what it looks at.
pub struct TextSearch {
    pub pattern: Pattern,
    pub forward: bool,
    /// The line it starts in, and the column in it (as `Pattern` takes them).
    pub line: usize,
    pub col: usize,
    /// With a line filter: the lines it kept, the only ones looked at.
    pub kept: Option<Vec<usize>>,
}

/// What a search found: the line and the columns, or the bytes in hex.
pub type Found = Option<(usize, usize, usize)>;

fn search_text(
    shared: &Shared,
    charset: Charset,
    search: &TextSearch,
    progress: &AtomicUsize,
    stop: &AtomicBool,
) -> io::Result<Found> {
    let kept = |num: usize| {
        search
            .kept
            .as_ref()
            .is_none_or(|rows| rows.binary_search(&num).is_ok())
    };
    let mut shown = Shown::default();
    let mut raw = Vec::new();
    if search.forward {
        let Some(mut reader) = LineReader::at(shared, search.line)? else {
            return Ok(None);
        };
        loop {
            if stop.load(Ordering::Relaxed) {
                return Ok(None);
            }
            let num = reader.next;
            let Some(cut) = reader.read(&mut raw, MAX_LINE)? else {
                return Ok(None);
            };
            if num.is_multiple_of(MARK_EVERY) {
                progress.store(num, Ordering::Relaxed);
            }
            if !kept(num) {
                continue;
            }
            let from = if num == search.line { search.col } else { 0 };
            let text = shown.of(&raw, cut, charset);
            if let Some((s, e)) = search.pattern.find_in_line(text, from) {
                return Ok(Some((num, s, e)));
            }
        }
    }
    // Backwards: one block of the index at a time, from its mark.
    let mut first = search.line / MARK_EVERY * MARK_EVERY;
    let mut last = search.line;
    loop {
        if stop.load(Ordering::Relaxed) {
            return Ok(None);
        }
        progress.store(first, Ordering::Relaxed);
        // The last match of the block is the one wanted: only it is kept.
        let mut best = None;
        if let Some(mut reader) = LineReader::at(shared, first)? {
            while reader.next <= last {
                let num = reader.next;
                let Some(cut) = reader.read(&mut raw, MAX_LINE)? else {
                    break;
                };
                if !kept(num) {
                    continue;
                }
                let before = if num == search.line {
                    search.col
                } else {
                    usize::MAX
                };
                let text = shown.of(&raw, cut, charset);
                if let Some((s, e)) = search.pattern.rfind_in_line(text, before) {
                    best = Some((num, s, e));
                }
            }
        }
        if best.is_some() {
            return Ok(best);
        }
        if first == 0 {
            return Ok(None);
        }
        last = first - 1;
        first -= MARK_EVERY;
    }
}

/// Searches the bytes of the file: forward, the first match starting at
/// `from` or after; backwards, the last one starting before it.
fn search_bytes(
    shared: &Shared,
    pattern: &Pattern,
    forward: bool,
    from: u64,
    progress: &AtomicUsize,
    stop: &AtomicBool,
) -> io::Result<Found> {
    let mut file = File::open(&shared.path)?;
    let mut block = vec![0u8; SEARCH_BLOCK as usize];
    let mut read = |start: u64, block: &mut Vec<u8>| -> io::Result<usize> {
        file.seek(SeekFrom::Start(start))?;
        read_full(&mut file, block)
    };
    if forward {
        let mut start = from;
        loop {
            if stop.load(Ordering::Relaxed) {
                return Ok(None);
            }
            progress.store(start as usize, Ordering::Relaxed);
            let n = read(start, &mut block)?;
            if let Some((s, e)) = pattern.find_in_bytes(&block[..n], 0) {
                return Ok(Some((0, start as usize + s, start as usize + e)));
            }
            if n < block.len() {
                return Ok(None);
            }
            start += SEARCH_BLOCK - SEARCH_OVERLAP;
        }
    }
    let mut before = from.min(shared.size);
    loop {
        if stop.load(Ordering::Relaxed) || before == 0 {
            return Ok(None);
        }
        progress.store(before as usize, Ordering::Relaxed);
        // A match starts before `before` and may end after it.
        let end = (before + SEARCH_OVERLAP).min(shared.size);
        let start = end.saturating_sub(SEARCH_BLOCK);
        let n = read(start, &mut block)?;
        let n = n.min((end - start) as usize);
        if let Some((s, e)) = pattern.rfind_in_bytes(&block[..n], (before - start) as usize) {
            return Ok(Some((0, start as usize + s, start as usize + e)));
        }
        before = start;
    }
}

/// What the filter's thread hands over.
#[derive(Default)]
struct Matches {
    /// The lines found since the screen last took them.
    lines: Vec<usize>,
    done: bool,
    /// It stopped at [`MAX_MATCHES`].
    capped: bool,
}

fn filter_lines(
    shared: &Shared,
    charset: Charset,
    matcher: &LineMatcher,
    out: &Mutex<Matches>,
    progress: &AtomicUsize,
    stop: &AtomicBool,
) {
    let lock = || out.lock().unwrap_or_else(PoisonError::into_inner);
    let mut found = Vec::new();
    let mut total = 0;
    let mut capped = false;
    let mut shown = Shown::default();
    let mut raw = Vec::new();
    let mut flushed = Instant::now();
    if let Ok(Some(mut reader)) = LineReader::at(shared, 0) {
        while !stop.load(Ordering::Relaxed) {
            let num = reader.next;
            let Ok(Some(cut)) = reader.read(&mut raw, MAX_LINE) else {
                break;
            };
            if matcher.matches(shown.of(&raw, cut, charset)) {
                found.push(num);
                total += 1;
                if total >= MAX_MATCHES {
                    capped = true;
                    break;
                }
            }
            if num.is_multiple_of(MARK_EVERY) {
                progress.store(num, Ordering::Relaxed);
                if !found.is_empty() && flushed.elapsed() >= FLUSH {
                    lock().lines.append(&mut found);
                    flushed = Instant::now();
                }
            }
        }
    }
    let mut out = lock();
    out.lines.append(&mut found);
    out.capped = capped;
    out.done = true;
}

fn spawn(work: impl FnOnce() + Send + 'static) {
    std::thread::spawn(move || {
        crate::panic_guard::catch(work);
    });
}

fn stop_flag() -> (Arc<AtomicBool>, StopOnDrop) {
    let flag = Arc::new(AtomicBool::new(false));
    (flag.clone(), StopOnDrop(flag))
}

/// Something asked of a thread: its answer, and the flag that stops it when
/// the answer is no longer wanted.
struct Asked<T> {
    rx: Receiver<T>,
    _stop: StopOnDrop,
}

impl<T: Send + 'static> Asked<T> {
    fn start(work: impl FnOnce(&AtomicBool) -> T + Send + 'static) -> Self {
        let (stop, guard) = stop_flag();
        let (tx, rx) = mpsc::channel();
        spawn(move || {
            let _ = tx.send(work(&stop));
        });
        Asked { rx, _stop: guard }
    }
}

/// A search going on, and how far it went (a line, or a byte in hex).
struct Searching {
    asked: Asked<Found>,
    progress: Arc<AtomicUsize>,
}

struct Filtering {
    out: Arc<Mutex<Matches>>,
    progress: Arc<AtomicUsize>,
    _stop: StopOnDrop,
}

/// How a filter over the file is going.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterState {
    /// At this line.
    Running(usize),
    /// Over; `capped` if it stopped at [`MAX_MATCHES`].
    Done { capped: bool },
}

/// The window over one file.
pub struct TextWindow {
    shared: Arc<Shared>,
    charset: Charset,
    /// The lines kept: their numbers in order, and each as the screen shows it.
    nums: Vec<usize>,
    lines: Lines,
    bidi: Vec<(usize, usize)>,
    /// The lines the head of the file had: known before the count says so.
    floor: usize,
    /// The lines known and whether the count is over, as of the last `poll`.
    known: usize,
    done: bool,
    bidi_total: usize,
    changed: bool,
    counting: Option<StopOnDrop>,
    /// The count was started once: Esc stops it for good, until asked again.
    count_asked: bool,
    /// The lines asked for and not here yet, and what was asked last: the row
    /// at the top, how many lines, and the last of them.
    fetching: Option<Asked<Fetched>>,
    asked: Option<(usize, usize, usize)>,
    /// The bytes of the hex mode kept, and where they start.
    hex: (u64, Vec<u8>),
    hex_fetching: Option<Asked<(u64, Vec<u8>)>>,
    hex_asked: Option<u64>,
    searching: Option<Searching>,
    filtering: Option<Filtering>,
}

impl std::fmt::Debug for TextWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextWindow")
            .field("path", &self.shared.path)
            .field("known", &self.known)
            .field("kept", &self.nums.len())
            .finish_non_exhaustive()
    }
}

impl TextWindow {
    /// A window over `path`, which has `size` bytes and starts with `head`
    /// (its first whole lines, already read, with their bidi marks).
    pub fn new(
        path: &Path,
        size: u64,
        stamp: Stamp,
        charset: Charset,
        head: Lines,
        bidi: Vec<(usize, usize)>,
    ) -> Self {
        TextWindow {
            shared: Self::shared(path, size, stamp, charset),
            charset,
            nums: (0..head.len()).collect(),
            floor: head.len(),
            known: head.len(),
            lines: head,
            bidi,
            done: false,
            bidi_total: 0,
            changed: false,
            counting: None,
            count_asked: false,
            fetching: None,
            asked: None,
            hex: (0, Vec::new()),
            hex_fetching: None,
            hex_asked: None,
            searching: None,
            filtering: None,
        }
    }

    fn shared(path: &Path, size: u64, stamp: Stamp, charset: Charset) -> Arc<Shared> {
        Arc::new(Shared {
            path: path.to_path_buf(),
            stamp,
            size,
            unit: Unit::of(charset),
            utf8: charset == Charset::Utf8,
            index: Mutex::new(LineIndex {
                marks: vec![0],
                ..LineIndex::default()
            }),
        })
    }

    /// Starts the count the first time the text is looked at: a file seen
    /// only as a table or in hex is never counted.
    pub fn count_once(&mut self) {
        if !self.count_asked {
            self.count();
        }
    }

    /// Starts the count, or takes it up from where it was stopped.
    pub fn count(&mut self) {
        self.count_asked = true;
        if self.done || self.counting.is_some() {
            return;
        }
        let (stop, guard) = stop_flag();
        let shared = self.shared.clone();
        spawn(move || count(&shared, &stop));
        self.counting = Some(guard);
    }

    /// The lines known so far.
    pub fn len(&self) -> usize {
        self.known
    }

    /// The count has not reached the end of the file.
    pub fn counting(&self) -> bool {
        !self.done
    }

    pub fn size(&self) -> u64 {
        self.shared.size
    }

    /// The file is no longer the one that was opened.
    pub fn changed(&self) -> bool {
        self.changed
    }

    /// Where a bidi control was swapped in the lines kept.
    pub fn bidi(&self) -> &[(usize, usize)] {
        &self.bidi
    }

    /// The bidi controls of the whole file found so far (in UTF-8; in another
    /// encoding, those of the lines kept).
    pub fn bidi_total(&self) -> usize {
        self.bidi_total.max(self.bidi.len())
    }

    fn position(&self, line: usize) -> Option<usize> {
        self.nums.binary_search(&line).ok()
    }

    pub fn has(&self, line: usize) -> bool {
        self.position(line).is_some()
    }

    /// The line as the screen shows it, or [`MISSING`] while it is not read.
    pub fn line(&self, line: usize) -> &str {
        self.position(line).map_or(MISSING, |i| &self.lines[i])
    }

    /// Looks at what the threads did. True if there is something new to show.
    pub fn poll(&mut self) -> bool {
        let mut new = false;
        {
            let index = self.shared.lock();
            let known = index.lines.max(self.floor);
            new |= known != self.known || index.done != self.done;
            new |= index.bidi != self.bidi_total || index.changed != self.changed;
            self.known = known;
            self.done = index.done;
            self.bidi_total = index.bidi;
            self.changed |= index.changed;
        }
        if self.done {
            self.counting = None;
        }
        new |= self.take_fetched(Duration::ZERO);
        new |= self.take_hex(Duration::ZERO);
        new
    }

    fn take_fetched(&mut self, wait: Duration) -> bool {
        let Some(asked) = &self.fetching else {
            return false;
        };
        let got = if wait.is_zero() {
            asked.rx.try_recv().map_err(|e| e == TryRecvError::Empty)
        } else {
            asked
                .rx
                .recv_timeout(wait)
                .map_err(|e| e == mpsc::RecvTimeoutError::Timeout)
        };
        match got {
            Ok(got) => {
                self.fetching = None;
                self.changed |= got.changed;
                self.nums = got.nums;
                self.lines = got.lines;
                self.bidi = got.bidi;
                true
            }
            // Still reading.
            Err(true) => false,
            Err(false) => {
                self.fetching = None;
                false
            }
        }
    }

    /// Makes the window be around `wanted` (the lines of the rows around the
    /// top of the screen, in order; those at `screen` are on it), which is
    /// around the row `center`. Waits a moment for lines of the screen that
    /// are not here. True if the lines kept changed.
    pub fn want(&mut self, wanted: Vec<usize>, screen: Range<usize>, center: usize) -> bool {
        let (Some(&first), Some(&last)) = (wanted.first(), wanted.last()) else {
            return false;
        };
        let missing = wanted[screen.clone()].iter().any(|&n| !self.has(n));
        let key = (center, wanted.len(), last);
        let same = self.asked == Some(key);
        if missing {
            // Asked already and it did not come: the file has no such line.
            if same && self.fetching.is_none() {
                return false;
            }
        } else {
            let edge = !self.has(first) || !self.has(last);
            let far = self.asked.is_none_or(|(c, ..)| c.abs_diff(center) >= AGAIN);
            if !edge || !far {
                return false;
            }
        }
        if !same || self.fetching.is_none() {
            let shared = self.shared.clone();
            let charset = self.charset;
            self.asked = Some(key);
            self.fetching = Some(Asked::start(move |stop| {
                fetch(&shared, charset, &wanted, screen, stop)
            }));
        }
        missing && self.take_fetched(FETCH_WAIT)
    }

    /// Forgets the lines kept: the rows are others now (a filter, an encoding).
    pub fn forget(&mut self) {
        self.nums.clear();
        self.lines = Lines::default();
        self.bidi.clear();
        self.fetching = None;
        self.asked = None;
    }

    /// Reads the file as `charset` from now on. A line of UTF-16 ends
    /// elsewhere, so its count starts again.
    pub fn set_charset(&mut self, charset: Charset) {
        let again =
            Unit::of(charset) != self.shared.unit || (charset == Charset::Utf8) != self.shared.utf8;
        self.charset = charset;
        self.forget();
        self.searching = None;
        self.filtering = None;
        if again {
            let old = &self.shared;
            self.shared = Self::shared(&old.path, old.size, old.stamp, charset);
            self.counting = None;
            self.count_asked = false;
            self.floor = 0;
            self.known = 0;
            self.done = false;
            self.bidi_total = 0;
        }
    }

    /// The bytes of the file from `offset`, for one row of hex: empty while
    /// they are not read.
    pub fn hex(&self, offset: u64) -> &[u8] {
        let (start, bytes) = &self.hex;
        offset
            .checked_sub(*start)
            .and_then(|i| bytes.get(i as usize..))
            .unwrap_or_default()
    }

    fn take_hex(&mut self, wait: Duration) -> bool {
        let Some(asked) = &self.hex_fetching else {
            return false;
        };
        let got = if wait.is_zero() {
            asked.rx.try_recv().ok()
        } else {
            asked.rx.recv_timeout(wait).ok()
        };
        match got {
            Some(block) => {
                self.hex_fetching = None;
                self.hex = block;
                true
            }
            None => false,
        }
    }

    /// Makes the bytes `bytes` of the file be here for the hex mode, waiting a
    /// moment for them. True if the bytes kept changed.
    pub fn want_hex(&mut self, bytes: Range<u64>) -> bool {
        let end = bytes.end.min(self.shared.size);
        let (start, kept) = (self.hex.0, self.hex.1.len() as u64);
        if bytes.start >= end || (bytes.start >= start && end <= start + kept) {
            return false;
        }
        // The screen in the middle of what is read.
        let from = bytes.start.saturating_sub(HEX_BLOCK / 2) / 16 * 16;
        if self.hex_asked == Some(from) && self.hex_fetching.is_none() {
            return false;
        }
        if self.hex_asked != Some(from) || self.hex_fetching.is_none() {
            let shared = self.shared.clone();
            self.hex_asked = Some(from);
            self.hex_fetching = Some(Asked::start(move |_| {
                (from, fetch_hex(&shared, from).unwrap_or_default())
            }));
        }
        self.take_hex(FETCH_WAIT)
    }

    /// Starts a search over the lines of the file, in place of any other.
    pub fn search_text(&mut self, search: TextSearch) {
        let shared = self.shared.clone();
        let charset = self.charset;
        let progress = Arc::new(AtomicUsize::new(search.line));
        let at = progress.clone();
        self.searching = Some(Searching {
            asked: Asked::start(move |stop| {
                search_text(&shared, charset, &search, &at, stop).unwrap_or(None)
            }),
            progress,
        });
    }

    /// Starts a search over the bytes of the file, in place of any other.
    pub fn search_bytes(&mut self, pattern: Pattern, forward: bool, from: u64) {
        let shared = self.shared.clone();
        let progress = Arc::new(AtomicUsize::new(from as usize));
        let at = progress.clone();
        self.searching = Some(Searching {
            asked: Asked::start(move |stop| {
                search_bytes(&shared, &pattern, forward, from, &at, stop).unwrap_or(None)
            }),
            progress,
        });
    }

    /// How far the search going on is: a line, or a byte in hex.
    pub fn searching(&self) -> Option<usize> {
        self.searching
            .as_ref()
            .map(|s| s.progress.load(Ordering::Relaxed))
    }

    /// The end of the search, once: what it found, or nothing.
    pub fn take_found(&mut self) -> Option<Found> {
        let searching = self.searching.as_ref()?;
        let found = match searching.asked.rx.try_recv() {
            Ok(found) => found,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => None,
        };
        self.searching = None;
        Some(found)
    }

    /// Starts keeping only the lines `matcher` takes, over the whole file, or
    /// stops with `None`.
    pub fn filter(&mut self, matcher: Option<LineMatcher>) {
        self.filtering = None;
        self.forget();
        let Some(matcher) = matcher else { return };
        let shared = self.shared.clone();
        let charset = self.charset;
        let out = Arc::new(Mutex::new(Matches::default()));
        let progress = Arc::new(AtomicUsize::new(0));
        let (stop, guard) = stop_flag();
        let (to, at) = (out.clone(), progress.clone());
        spawn(move || filter_lines(&shared, charset, &matcher, &to, &at, &stop));
        self.filtering = Some(Filtering {
            out,
            progress,
            _stop: guard,
        });
    }

    /// Moves the lines the filter found since the last call into `rows`, and
    /// says how it is going. `None` with no filter going on (or once it said
    /// it was over).
    pub fn take_matches(&mut self, rows: &mut Vec<usize>) -> Option<FilterState> {
        let filtering = self.filtering.as_ref()?;
        let (done, capped) = {
            let mut out = filtering.out.lock().unwrap_or_else(PoisonError::into_inner);
            rows.append(&mut out.lines);
            (out.done, out.capped)
        };
        if done {
            self.filtering = None;
            return Some(FilterState::Done { capped });
        }
        Some(FilterState::Running(
            filtering.progress.load(Ordering::Relaxed),
        ))
    }

    /// Esc: stops the search, else the filter, else the count. False if
    /// nothing was going on.
    pub fn stop_work(&mut self) -> bool {
        if self.searching.take().is_some() || self.filtering.take().is_some() {
            return true;
        }
        self.counting.take().is_some() && !self.done
    }

    /// A thread is at work: the screen should look again soon.
    pub fn working(&self) -> bool {
        (self.counting.is_some() && !self.done)
            || self.fetching.is_some()
            || self.hex_fetching.is_some()
            || self.searching.is_some()
            || self.filtering.is_some()
    }
}

/// The text of the viewer: all of it in memory, or a window over the file.
#[derive(Debug)]
pub enum Text {
    Whole(Lines),
    Window(Box<TextWindow>),
}

impl Default for Text {
    fn default() -> Self {
        Text::Whole(Lines::default())
    }
}

impl From<Lines> for Text {
    fn from(lines: Lines) -> Self {
        Text::Whole(lines)
    }
}

impl Text {
    /// The lines there are: in a window, those known so far.
    pub fn len(&self) -> usize {
        match self {
            Text::Whole(lines) => lines.len(),
            Text::Window(w) => w.len(),
        }
    }

    pub fn window(&self) -> Option<&TextWindow> {
        match self {
            Text::Whole(_) => None,
            Text::Window(w) => Some(w),
        }
    }

    pub fn window_mut(&mut self) -> Option<&mut TextWindow> {
        match self {
            Text::Whole(_) => None,
            Text::Window(w) => Some(w),
        }
    }
}

impl Index<usize> for Text {
    type Output = str;

    fn index(&self, i: usize) -> &str {
        match self {
            Text::Whole(lines) => &lines[i],
            Text::Window(w) => w.line(i),
        }
    }
}

impl LineSource for Text {
    fn count(&self) -> usize {
        self.len()
    }

    fn line(&self, i: usize) -> &str {
        &self[i]
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Action, LoadOptions, SearchFrom, ViewMode, Viewer};
    use super::*;
    use crate::search::SearchSpec;
    use crate::viewer::line_filter::LineFilterSpec;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    /// A file of `text` seen through a window: any file over 64 bytes is, and
    /// only its first 40 bytes are read to open it.
    fn open(name: &str, text: &[u8]) -> (tempfile::TempDir, Viewer) {
        TEST_SIZES.set(Some((64, 40)));
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join(name);
        std::fs::write(&path, text).unwrap();
        let load = LoadOptions {
            in_background: true,
            ..LoadOptions::view(1 << 20)
        };
        let mut v = Viewer::open(path, load).unwrap();
        v.page = 5;
        (tmp, v)
    }

    /// `n` lines: "line 0", "line 1"...
    fn numbered(n: usize) -> Vec<u8> {
        (0..n)
            .flat_map(|i| format!("line {i}\n").into_bytes())
            .collect()
    }

    /// Waits for the threads of the window to have nothing more to do.
    fn settle(v: &mut Viewer) {
        let end = Instant::now() + Duration::from_secs(20);
        loop {
            v.poll_table();
            if !v.table_busy() {
                v.poll_table();
                return;
            }
            assert!(Instant::now() < end, "the window never settled");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn press(v: &mut Viewer, code: KeyCode) -> Action {
        v.handle_key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn pattern(text: &str) -> Pattern {
        Pattern::new(&SearchSpec::new(text)).unwrap()
    }

    #[test]
    fn a_big_text_opens_with_its_first_lines_and_counts_the_rest() {
        let (_tmp, mut v) = open("a.txt", &numbered(5_000));
        assert!(v.windowed());
        assert_eq!(v.mode, ViewMode::Text);
        // Only the whole lines of the first 40 bytes at first.
        assert_eq!(v.visible_lines(3), ["line 0", "line 1", "line 2"]);
        assert!(!v.truncated());
        settle(&mut v);
        assert!(!v.counting_lines());
        assert_eq!(v.total_lines(), 5_000);
        // The window keeps the lines around the screen, not the file.
        let w = v.lines.window().unwrap();
        assert!(w.nums.len() <= 2 * AROUND, "{} lines kept", w.nums.len());
    }

    #[test]
    fn any_line_is_reached_and_the_end_too() {
        let (_tmp, mut v) = open("a.txt", &numbered(5_000));
        settle(&mut v);
        v.goto_line(4_321);
        v.poll_table();
        assert_eq!(v.visible_lines(2), ["line 4320", "line 4321"]);
        press(&mut v, KeyCode::End);
        v.poll_table();
        assert_eq!(v.visible_lines(5).last().unwrap(), "line 4999");
        press(&mut v, KeyCode::Home);
        v.poll_table();
        assert_eq!(v.visible_lines(1), ["line 0"]);
        for _ in 0..3_000 {
            v.scroll_down(1);
        }
        v.poll_table();
        assert_eq!(v.visible_lines(1), ["line 3000"]);
        // With wrap, going up reads the line above to know its rows.
        v.wrap = true;
        v.width = 80;
        for _ in 0..2_500 {
            v.scroll_up(1);
        }
        v.poll_table();
        assert_eq!(v.visible_lines(1), ["line 500"]);
    }

    #[test]
    fn a_last_line_with_no_end_counts() {
        let mut text = numbered(300);
        text.extend_from_slice(b"the last");
        let (_tmp, mut v) = open("a.txt", &text);
        settle(&mut v);
        assert_eq!(v.total_lines(), 301);
        press(&mut v, KeyCode::End);
        v.poll_table();
        assert_eq!(v.visible_lines(5).last().unwrap(), "the last");
    }

    #[test]
    fn the_index_goes_across_the_blocks_read() {
        // Lines of 100 bytes: three blocks of 1 MiB and a bit.
        let line = |i: usize| format!("{i:<99}\n");
        let n = 3 * BLOCK / 100 + 500;
        let text: String = (0..n).map(line).collect();
        let (_tmp, mut v) = open("a.txt", text.as_bytes());
        settle(&mut v);
        assert_eq!(v.total_lines(), n);
        for target in [BLOCK / 100, 2 * BLOCK / 100 + 1, n - 1] {
            v.goto_line(target + 1);
            v.poll_table();
            assert_eq!(v.visible_lines(1)[0].trim_end(), target.to_string());
        }
    }

    #[test]
    fn a_search_goes_over_the_file_both_ways() {
        let (_tmp, mut v) = open("a.txt", &numbered(5_000));
        settle(&mut v);
        assert!(v.search(&pattern("line 4777"), true, SearchFrom::Edge));
        settle(&mut v);
        let hit = v.hit.unwrap();
        assert_eq!((hit.line, hit.start, hit.end), (4_777, 0, 9));
        assert!(v.visible_lines(5).contains(&"line 4777".to_string()));
        // The next one, backwards from the match.
        assert!(v.search_next(&pattern("line 12"), false));
        settle(&mut v);
        assert_eq!(v.hit.unwrap().line, 1_299);
        // And forward again, from it.
        assert!(v.search_next(&pattern("line 20"), true));
        settle(&mut v);
        assert_eq!(v.hit.unwrap().line, 2_000);
        assert!(v.search(&pattern("nowhere"), true, SearchFrom::Edge));
        settle(&mut v);
        assert_eq!(v.note.as_deref(), Some("Not found"));
    }

    #[test]
    fn a_filter_goes_over_the_file_and_the_search_sees_what_it_kept() {
        let (_tmp, mut v) = open("a.txt", &numbered(5_000));
        settle(&mut v);
        let spec = LineFilterSpec {
            text: "77".into(),
            ..LineFilterSpec::default()
        };
        let note = v.apply_line_filter(&spec);
        assert!(note.starts_with("Filtering"), "{note}");
        settle(&mut v);
        // 77, 177.., 770..779, 1770.., x77x: counted by the same rule.
        let expected = (0..5_000).filter(|i| i.to_string().contains("77")).count();
        assert_eq!(v.line_count(), expected);
        assert_eq!(v.total_lines(), 5_000);
        assert_eq!(v.visible_lines(3), ["line 77", "line 177", "line 277"]);
        assert_eq!(
            v.note.as_deref(),
            Some(format!("Filter: {expected} of 5,000 lines").as_str())
        );
        // "line 4" is in many lines, but only those kept are looked at.
        assert!(v.search(&pattern("line 4"), true, SearchFrom::Edge));
        settle(&mut v);
        assert_eq!(v.hit.unwrap().line, 477);
        // Off again: the line at the top stays.
        v.goto_line(1_771);
        v.poll_table();
        let off = LineFilterSpec::default();
        assert_eq!(v.apply_line_filter(&off), "Filter off: 5000 lines");
        v.poll_table();
        assert_eq!(v.visible_lines(1), ["line 1770"]);
        // A filter that keeps nothing goes away by itself.
        let none = LineFilterSpec {
            text: "nowhere".into(),
            ..LineFilterSpec::default()
        };
        v.apply_line_filter(&none);
        settle(&mut v);
        assert!(v.line_filter().is_none());
        assert_eq!(v.line_count(), 5_000);
    }

    #[test]
    fn hex_reads_the_bytes_where_the_screen_is() {
        let text = numbered(100_000);
        let (_tmp, mut v) = open("a.txt", &text);
        press(&mut v, KeyCode::Char('h'));
        assert_eq!(v.mode, ViewMode::Hex);
        assert_eq!(v.line_count(), text.len().div_ceil(16));
        v.poll_table();
        assert!(v.visible_lines(1)[0].starts_with("00000000  6C 69 6E 65 20 30 0A"));
        // Far from the start: past what one reading of the hex brings.
        let offset = text.len() - 64;
        v.goto_line(offset);
        v.poll_table();
        let row = &text[offset / 16 * 16..][..16];
        let hex: Vec<String> = row.iter().map(|b| format!("{b:02X}")).collect();
        let shown = v.visible_lines(1)[0].replace("  ", " ");
        assert!(shown.contains(&hex.join(" ")), "{shown}");
        // The search goes over the bytes of the file.
        assert!(v.search(&pattern("line 99999"), true, SearchFrom::Edge));
        settle(&mut v);
        let hit = v.hit.unwrap();
        assert_eq!(hit.mode, ViewMode::Hex);
        assert_eq!(&text[hit.start..hit.end], b"line 99999");
        assert!(v.search(&pattern("line 12345"), false, SearchFrom::Edge));
        settle(&mut v);
        let hit = v.hit.unwrap();
        assert_eq!(&text[hit.start..hit.end], b"line 12345");
    }

    #[test]
    fn another_encoding_reads_the_lines_again() {
        // "ação" in CP850, in every line.
        let line = [b'a', 0x87, 0xC6, b'o', b'\n'];
        let text: Vec<u8> = (0..2_000).flat_map(|_| line).collect();
        let (_tmp, mut v) = open("a.txt", &text);
        settle(&mut v);
        assert_eq!(v.total_lines(), 2_000);
        while v.charset() != Charset::Cp850 {
            v.cycle_charset();
        }
        v.goto_line(1_500);
        v.poll_table();
        assert_eq!(v.visible_lines(1), ["ação"]);
    }

    #[test]
    fn utf16_lines_end_in_two_bytes() {
        let mut text = vec![0xFF, 0xFE];
        for i in 0..1_000 {
            for unit in format!("linha {i} é\n").encode_utf16() {
                text.extend_from_slice(&unit.to_le_bytes());
            }
        }
        let (_tmp, mut v) = open("a.txt", &text);
        assert_eq!(v.charset(), Charset::Utf16Le);
        settle(&mut v);
        assert_eq!(v.total_lines(), 1_000);
        assert_eq!(v.visible_lines(1), ["linha 0 é"]);
        v.goto_line(900);
        v.poll_table();
        assert_eq!(v.visible_lines(1), ["linha 899 é"]);
    }

    #[test]
    fn a_very_long_line_is_cut_on_screen() {
        let mut text = b"first\n".to_vec();
        text.extend(std::iter::repeat_n(b'x', MAX_LINE + 10));
        text.extend_from_slice(b"\nlast\n");
        let (_tmp, mut v) = open("a.txt", &text);
        settle(&mut v);
        assert_eq!(v.total_lines(), 3);
        v.poll_table();
        let long = &v.lines[1];
        assert!(long.ends_with('…'));
        assert!(long.len() <= MAX_LINE + 8, "{}", long.len());
        assert_eq!(&v.lines[2], "last");
    }

    #[test]
    fn the_bidi_controls_of_the_whole_file_are_counted() {
        let mut text = numbered(2_000);
        text.extend_from_slice("a\u{202E}b\u{2066}c\u{061C}\n".as_bytes());
        text.extend(numbered(10));
        let (_tmp, mut v) = open("a.txt", &text);
        settle(&mut v);
        assert_eq!(v.bidi_count(), 3);
        v.goto_line(2_001);
        v.poll_table();
        assert_eq!(v.visible_lines(1), ["a·b·c·"]);
    }

    #[test]
    fn a_file_that_changed_says_so_and_r_reads_it_again() {
        let (tmp, mut v) = open("a.txt", &numbered(3_000));
        settle(&mut v);
        v.goto_line(2_000);
        v.poll_table();
        assert!(!v.text_changed());
        std::fs::write(tmp.path().join("a.txt"), numbered(4_000)).unwrap();
        v.goto_line(2_900);
        settle(&mut v);
        assert!(v.text_changed());
        let top = v.top_line();
        press(&mut v, KeyCode::Char('r'));
        settle(&mut v);
        assert!(!v.text_changed());
        assert_eq!(v.total_lines(), 4_000);
        // Back where it was.
        assert_eq!(v.top_line(), top);
    }

    #[test]
    fn a_small_file_and_autoview_stay_in_memory() {
        let (tmp, v) = open("small.txt", b"one\ntwo\n");
        assert!(!v.windowed());
        let path = tmp.path().join("big.txt");
        std::fs::write(&path, numbered(100)).unwrap();
        let v = Viewer::open(path, LoadOptions::view(1 << 20)).unwrap();
        assert!(!v.windowed());
        assert_eq!(v.total_lines(), 100);
    }

    #[test]
    fn a_batch_over_a_window_reads_the_file() {
        let (_tmp, mut v) = open("a.txt", &numbered(3_000));
        settle(&mut v);
        let (text, _, cut) = v.batch_text().unwrap();
        assert_eq!(text.lines().count(), 3_000);
        assert!(!cut);
    }

    #[test]
    fn a_big_patch_keeps_its_colors_but_has_no_index() {
        let text = "diff --git a/x b/x\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n-a\n+b\n".repeat(4);
        let (_tmp, mut v) = open("big.patch", text.as_bytes());
        assert!(v.windowed());
        assert_eq!(v.language(), Some("Patch"));
        assert_eq!(v.patch_summary(), None);
        v.handle_key(KeyEvent::new(KeyCode::Char(']'), KeyModifiers::NONE));
        assert_eq!(v.note.as_deref(), Some("No file list in a patch this big"));
    }
}
