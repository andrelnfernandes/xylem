//! What is kept of the books read (docs/plans/viewer-ebooks.md, 1c), in
//! `~/.local/state/xylem/reading.toml`: for each book the user asked to be
//! remembered (`R` in the viewer), where the reading stopped and, with
//! `reading_stats` on, the time spent reading it.
//!
//! What someone reads is personal: a book is known only by a hash of what it
//! says it is (or of its bytes), never by its name or path; its title and
//! author are kept only with `reading_titles` on.

use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Books kept, at most: past them, the one remembered longest ago goes.
pub const MAX_BOOKS: usize = 1_000;
/// Sessions kept of each book, the last ones.
pub const MAX_SESSIONS: usize = 100;
/// A pause longer than this starts another session.
pub const SESSION_GAP: Duration = Duration::from_secs(30 * 60);
/// A page turned sooner than this is skimmed, not read.
pub const SKIMMED: Duration = Duration::from_secs(5);
/// Reading measured before the speed measured is trusted for the time left.
pub const MEASURED: f64 = 10.0 * 60.0;

/// A session of reading: when it started (seconds since 1970), how long it
/// lasted and how many words were read in it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub start: u64,
    pub seconds: u64,
    pub words: u64,
}

/// What is kept of one book.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Record {
    /// The hash of the book (see [`identity`]).
    pub id: String,
    /// Where the reading stopped: the chapter (from 0) and the line of the
    /// text within it (not the row of the screen: it does not change with
    /// the width).
    pub chapter: usize,
    pub line: usize,
    /// When it was last remembered (seconds since 1970).
    pub at: u64,
    /// The time read, in seconds; the words read at a measured pace and the
    /// seconds they took (the speed comes from them).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub seconds: u64,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub words: u64,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub word_seconds: u64,
    /// The farthest the reading went, from 0 to 1: going back does not take
    /// it back.
    #[serde(default, skip_serializing_if = "is_zero_f")]
    pub farthest: f64,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub first: u64,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub last: u64,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub finished: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sessions: Vec<Session>,
    /// Only with `reading_titles` on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

fn is_zero(n: &u64) -> bool {
    *n == 0
}

fn is_zero_f(n: &f64) -> bool {
    *n == 0.0
}

impl Record {
    /// Whether the time of the book was measured.
    pub fn has_stats(&self) -> bool {
        self.seconds > 0 || !self.sessions.is_empty()
    }

    /// Forgets the time read, keeping where the reading stopped.
    pub fn clear_stats(&mut self) {
        let keep = Record {
            id: std::mem::take(&mut self.id),
            chapter: self.chapter,
            line: self.line,
            at: self.at,
            ..Record::default()
        };
        *self = keep;
    }

    /// Words per minute at a measured pace, once enough was measured.
    pub fn speed(&self) -> Option<u16> {
        (self.word_seconds as f64 >= MEASURED)
            .then(|| (self.words as f64 * 60.0 / self.word_seconds as f64).round() as u16)
            .filter(|&w| w > 0)
    }
}

/// The books kept, as `reading.toml` has them.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Store {
    #[serde(default, rename = "book")]
    pub books: Vec<Record>,
}

impl Store {
    pub fn get(&self, id: &str) -> Option<&Record> {
        self.books.iter().find(|b| b.id == id)
    }

    /// Keeps `record` in place of the one of the same book; past
    /// [`MAX_BOOKS`], the one remembered longest ago goes.
    pub fn put(&mut self, record: Record) {
        self.books.retain(|b| b.id != record.id);
        self.books.push(record);
        while self.books.len() > MAX_BOOKS {
            if let Some(oldest) = self
                .books
                .iter()
                .enumerate()
                .min_by_key(|(_, b)| b.at)
                .map(|(i, _)| i)
            {
                self.books.remove(oldest);
            }
        }
    }

    pub fn forget(&mut self, id: &str) {
        self.books.retain(|b| b.id != id);
    }

    /// The books with time read.
    pub fn with_stats(&self) -> usize {
        self.books.iter().filter(|b| b.has_stats()).count()
    }

    /// Forgets the time read of every book, keeping where each stopped.
    pub fn clear_stats(&mut self) {
        for b in &mut self.books {
            b.clear_stats();
        }
    }

    /// Forgets the titles and authors kept.
    pub fn clear_titles(&mut self) {
        for b in &mut self.books {
            b.title = None;
            b.author = None;
        }
    }
}

#[cfg(test)]
thread_local! {
    /// Where the tests keep their books: nowhere unless a test says so, so
    /// that no test writes the user's file.
    pub static TEST_PATH: std::cell::RefCell<Option<PathBuf>> =
        const { std::cell::RefCell::new(None) };
}

/// The file of the books kept.
pub fn path() -> Option<PathBuf> {
    #[cfg(test)]
    {
        TEST_PATH.with(|p| p.borrow().clone())
    }
    #[cfg(not(test))]
    {
        Some(crate::platform::state_dir()?.join("reading.toml"))
    }
}

/// The books kept; none if the file is not there or cannot be read.
pub fn load() -> Store {
    path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|text| toml::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save(store: &Store) -> std::io::Result<()> {
    let Some(path) = path() else { return Ok(()) };
    let text = toml::to_string(store).map_err(std::io::Error::other)?;
    crate::state::write_private(&path, &text)
}

/// What a book is known by: the hash of what it says it is and of its size,
/// or, when it says nothing, of its bytes and size. Moved or renamed, it is
/// the same book; another file with its name is not.
pub fn identity(identifier: Option<&str>, size: u64, bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    match identifier.map(str::trim).filter(|i| !i.is_empty()) {
        Some(id) => {
            h.update(b"id\0");
            h.update(id.as_bytes());
        }
        None => {
            h.update(b"bytes\0");
            h.update(bytes);
        }
    }
    h.update(b"\0");
    h.update(size.to_le_bytes());
    let digest = h.finalize();
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Seconds since 1970, now.
pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// The clock of a book being read: the time between two keys counts when
/// it is no longer than `idle` (a longer pause stops it at the first key);
/// the words of the pages turned forward at a reading pace count for the
/// speed.
#[derive(Debug, Clone)]
pub struct Clock {
    idle: Duration,
    /// The last key, and the reading position then (words from the start).
    last: Option<(Instant, f64)>,
    /// The session being read: when it started (seconds since 1970), how
    /// long it lasted and its words.
    pub session: Option<Session>,
    session_seconds: f64,
    pub seconds: f64,
    pub words: f64,
    pub word_seconds: f64,
}

/// What a key did to the reading, for the clock.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Move {
    /// A page or a row forward (Space, PgDn, ↓).
    Forward,
    /// Anything that is not reading on: a jump, a search, going back.
    Other,
}

impl Clock {
    pub fn new(idle: Duration) -> Self {
        Clock {
            idle,
            last: None,
            session: None,
            session_seconds: 0.0,
            seconds: 0.0,
            words: 0.0,
            word_seconds: 0.0,
        }
    }

    /// The book opened at `now`, `at` words into it.
    pub fn start(&mut self, now: Instant, at: f64, wall: u64) {
        self.last = Some((now, at));
        self.session = Some(Session {
            start: wall,
            seconds: 0,
            words: 0,
        });
        self.session_seconds = 0.0;
    }

    /// A key at `now`, after which the reading is `at` words into the book.
    /// Returns a session that ended (a pause longer than [`SESSION_GAP`]).
    pub fn key(&mut self, now: Instant, at: f64, how: Move, wall: u64) -> Option<Session> {
        let mut ended = None;
        if let Some((then, before)) = self.last {
            let gap = now.saturating_duration_since(then);
            if gap > SESSION_GAP {
                ended = self.end();
                self.session = Some(Session {
                    start: wall,
                    seconds: 0,
                    words: 0,
                });
            } else if gap <= self.idle {
                let secs = gap.as_secs_f64();
                self.seconds += secs;
                self.session_seconds += secs;
                if how == Move::Forward && at > before && gap >= SKIMMED {
                    self.words += at - before;
                    self.word_seconds += secs;
                    if let Some(s) = &mut self.session {
                        s.words += (at - before).round() as u64;
                    }
                }
            }
        }
        if let Some(s) = &mut self.session {
            s.seconds = self.session_seconds.round() as u64;
        }
        self.last = Some((now, at));
        ended
    }

    /// The session being read, ended.
    pub fn end(&mut self) -> Option<Session> {
        let s = self.session.take()?;
        let s = Session {
            seconds: self.session_seconds.round() as u64,
            ..s
        };
        self.session_seconds = 0.0;
        (s.seconds > 0).then_some(s)
    }

    /// The seconds of the session being read.
    pub fn session_seconds(&self) -> u64 {
        self.session_seconds.round() as u64
    }
}

/// `3 h 20 min`, `12 min`, `40 s`.
pub fn duration(seconds: u64) -> String {
    let minutes = seconds / 60;
    match minutes {
        0 => format!("{seconds} s"),
        m if m < 60 => format!("{m} min"),
        m if m % 60 == 0 => format!("{} h", m / 60),
        m => format!("{} h {} min", m / 60, m % 60),
    }
}

/// A day, as `2026-11-20`, from seconds since 1970 (UTC).
pub fn day(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    // Days to a civil date (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

/// What is shown of a book's reading, on its title page and in its report.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Summary {
    /// On the title page: `Your reading: 3 h 20 min in 6 sessions · 46% …`.
    pub line: Option<String>,
    /// In the report: the numbers, and the sessions as rows of a table.
    pub fields: Vec<(String, String)>,
    pub sessions: Vec<Vec<String>>,
    /// With the titles kept: every book, as rows of a table.
    pub books: Vec<Vec<String>>,
    /// Where the reading stopped, to go back there.
    pub at: Option<(usize, usize)>,
}

/// The summary of the book `id` in `store`: `stats` shows its time,
/// `titles` the list of every book.
pub fn summary(store: &Store, id: &str, stats: bool, titles: bool) -> Summary {
    let mut out = Summary::default();
    if titles && stats {
        let mut books: Vec<&Record> = store.books.iter().filter(|b| b.title.is_some()).collect();
        books.sort_by_key(|b| std::cmp::Reverse(b.last.max(b.at)));
        out.books = books
            .iter()
            .map(|b| {
                vec![
                    b.title.clone().unwrap_or_default(),
                    b.author.clone().unwrap_or_default(),
                    duration(b.seconds),
                    format!("{}%", (b.farthest * 100.0).floor()),
                    if b.last > 0 {
                        day(b.last)
                    } else {
                        String::new()
                    },
                ]
            })
            .collect();
    }
    let Some(r) = store.get(id) else { return out };
    out.at = Some((r.chapter, r.line));
    if !stats || !r.has_stats() {
        return out;
    }
    let sessions = r.sessions.len();
    let mut parts = vec![format!(
        "{} in {sessions} {}",
        duration(r.seconds),
        if sessions == 1 { "session" } else { "sessions" }
    )];
    parts.push(format!("{}%", (r.farthest * 100.0).floor()));
    if let Some(w) = r.speed() {
        parts.push(format!("{w} words/min"));
    }
    if r.first > 0 {
        parts.push(format!("since {}", day(r.first)));
    }
    out.line = Some(format!("Your reading: {}", parts.join(" · ")));
    out.fields.push(("Time read".into(), duration(r.seconds)));
    out.fields.push(("Sessions".into(), sessions.to_string()));
    out.fields.push((
        "Farthest".into(),
        format!("{}% of the book", (r.farthest * 100.0).floor()),
    ));
    out.fields.push((
        "Speed".into(),
        match r.speed() {
            Some(w) => format!("{w} words per minute"),
            None => "not measured yet (10 minutes of reading on)".into(),
        },
    ));
    if r.first > 0 {
        out.fields.push(("First read".into(), day(r.first)));
    }
    if r.last > 0 {
        out.fields.push(("Last read".into(), day(r.last)));
    }
    if r.finished > 0 {
        out.fields.push(("Finished".into(), day(r.finished)));
    }
    out.sessions = r
        .sessions
        .iter()
        .rev()
        .map(|s| vec![day(s.start), duration(s.seconds), s.words.to_string()])
        .collect();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(id: &str, at: u64) -> Record {
        Record {
            id: id.into(),
            at,
            ..Record::default()
        }
    }

    #[test]
    fn a_book_is_known_by_what_it_says_it_is_and_its_size() {
        let a = identity(Some("urn:uuid:1"), 100, b"one");
        assert_eq!(a, identity(Some(" urn:uuid:1 "), 100, b"other bytes"));
        assert_ne!(a, identity(Some("urn:uuid:1"), 101, b"one"));
        assert_ne!(a, identity(Some("urn:uuid:2"), 100, b"one"));
        let b = identity(None, 3, b"one");
        assert_eq!(b, identity(Some(""), 3, b"one"));
        assert_ne!(b, identity(None, 3, b"two"));
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn the_store_keeps_a_thousand_books_and_forgets_the_oldest() {
        let mut s = Store::default();
        for i in 0..MAX_BOOKS as u64 + 5 {
            s.put(record(&format!("b{i}"), 1_000 + i));
        }
        assert_eq!(s.books.len(), MAX_BOOKS);
        assert!(s.get("b0").is_none() && s.get("b4").is_none());
        assert!(s.get("b5").is_some());
        // Kept again, a book replaces itself.
        s.put(Record {
            chapter: 3,
            ..record("b5", 5)
        });
        assert_eq!(s.books.len(), MAX_BOOKS);
        assert_eq!(s.get("b5").unwrap().chapter, 3);
        s.forget("b5");
        assert!(s.get("b5").is_none());
    }

    #[test]
    fn the_file_has_hashes_and_numbers_only() {
        let tmp = crate::testing::tempdir();
        let file = tmp.path().join("state/reading.toml");
        TEST_PATH.with(|p| *p.borrow_mut() = Some(file.clone()));
        let mut s = Store::default();
        s.put(Record {
            chapter: 2,
            line: 7,
            seconds: 90,
            ..record(&identity(Some("urn:isbn:42"), 10, b""), 1)
        });
        save(&s).unwrap();
        let text = std::fs::read_to_string(&file).unwrap();
        assert!(!text.contains("isbn") && !text.contains("title"), "{text}");
        assert!(text.contains("chapter = 2"), "{text}");
        assert_eq!(load(), s);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&file).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        TEST_PATH.with(|p| *p.borrow_mut() = None);
        // A file that cannot be read is no books.
        assert_eq!(load(), Store::default());
    }

    #[test]
    fn the_clock_counts_reading_and_not_pauses() {
        let t0 = Instant::now();
        let at = |s: u64| t0 + Duration::from_secs(s);
        let mut c = Clock::new(Duration::from_secs(180));
        c.start(t0, 0.0, 1_000);
        // A page every 60 s: read, at 300 words a page.
        c.key(at(60), 300.0, Move::Forward, 0);
        c.key(at(120), 600.0, Move::Forward, 0);
        assert_eq!(c.seconds, 120.0);
        assert_eq!((c.words, c.word_seconds), (600.0, 120.0));
        // Skimmed (2 s), and a jump: time, but no words.
        c.key(at(122), 900.0, Move::Forward, 0);
        c.key(at(140), 5_000.0, Move::Other, 0);
        assert_eq!(c.seconds, 140.0);
        assert_eq!(c.words, 600.0);
        // Back a page: not words read.
        c.key(at(200), 4_700.0, Move::Forward, 0);
        assert_eq!(c.words, 600.0);
        // Away for 10 minutes: the pause does not count.
        c.key(at(800), 5_000.0, Move::Forward, 0);
        assert_eq!(c.seconds, 200.0);
        assert_eq!(c.session_seconds(), 200);
        // Away for an hour: another session.
        let ended = c.key(at(4_400), 5_000.0, Move::Other, 9_000).unwrap();
        assert_eq!((ended.start, ended.seconds, ended.words), (1_000, 200, 600));
        assert_eq!(c.session.unwrap().start, 9_000);
        assert_eq!(c.session_seconds(), 0);
        c.key(at(4_460), 5_300.0, Move::Forward, 0);
        let last = c.end().unwrap();
        assert_eq!((last.seconds, last.words), (60, 300));
        assert!(c.end().is_none());
    }

    #[test]
    fn a_summary_says_the_time_the_speed_and_the_sessions() {
        let mut s = Store::default();
        s.put(Record {
            seconds: 12_000,
            words: 238 * 30,
            word_seconds: 30 * 60,
            farthest: 0.468,
            first: 1_762_041_600,
            last: 1_763_596_800,
            sessions: vec![
                Session {
                    start: 1_762_041_600,
                    seconds: 3_600,
                    words: 9_000,
                };
                6
            ],
            title: Some("A Book".into()),
            ..record("x", 1)
        });
        let sum = summary(&s, "x", true, false);
        assert_eq!(
            sum.line.as_deref(),
            Some("Your reading: 3 h 20 min in 6 sessions · 46% · 238 words/min · since 2025-11-02")
        );
        assert_eq!(sum.sessions.len(), 6);
        assert!(sum.books.is_empty(), "titles off");
        assert_eq!(summary(&s, "x", true, true).books[0][0], "A Book");
        // With the statistics off, only where it stopped.
        let off = summary(&s, "x", false, true);
        assert!(off.line.is_none() && off.fields.is_empty() && off.books.is_empty());
        assert_eq!(off.at, Some((0, 0)));
        assert_eq!(summary(&s, "y", true, false), Summary::default());
        // Cleared, the time goes and the place stays.
        let mut cleared = s.clone();
        cleared.clear_stats();
        assert_eq!(cleared.with_stats(), 0);
        assert_eq!(cleared.get("x").unwrap().title, None);
    }

    #[test]
    fn days_and_durations() {
        assert_eq!(day(0), "1970-01-01");
        assert_eq!(day(1_763_596_800), "2025-11-20");
        assert_eq!(day(951_782_400), "2000-02-29");
        assert_eq!(duration(40), "40 s");
        assert_eq!(duration(12 * 60), "12 min");
        assert_eq!(duration(3 * 3600), "3 h");
        assert_eq!(duration(12_000), "3 h 20 min");
    }
}
