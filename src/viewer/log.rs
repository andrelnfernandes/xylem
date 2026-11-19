//! Logs (docs/plans/viewer-logs.md): how one is told, the level of each line
//! and its color, and the ANSI sequences a program wrote into it, of which only
//! the text color is kept.
//!
//! Nothing here adds to what reaches the terminal: the ANSI parser only takes
//! sequences out, and the only thing it gives back is a color.

use std::path::Path;
use std::sync::Arc;

use super::lines::LineSource;
use crate::highlight::{self, Highlighter, LineColors};

/// Lines looked at to tell a log by its contents.
const SNIFF_LINES: usize = 200;
/// Columns of a line where its level is looked for.
const LEVEL_COLUMNS: usize = 120;
/// Lines with no level of their own (a stack trace) that take the one before.
const INHERIT: usize = 200;
/// Characters of a line given a color: past them the line has the text color.
const COLORED_CHARS: usize = 10_000;

/// A log by its name: `.log`, `.out`, `.err`, a rotated one (`app.log.1`,
/// `app.log.2026-11-16`, `app.log.1.gz`), or the system's own.
pub fn is_log_name(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    let name = name.to_ascii_lowercase();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    matches!(ext.as_deref(), Some("log" | "out" | "err"))
        || name.contains(".log.")
        || name.ends_with(".journal.txt")
        || matches!(
            name.as_str(),
            "syslog" | "messages" | "kern.log" | "auth.log" | "daemon.log" | "dmesg"
        )
        || name.starts_with("syslog.")
        || name.starts_with("messages.")
}

/// A log by its contents: most of its first lines start with a date and time.
pub fn looks_like_log(lines: &(impl LineSource + ?Sized)) -> bool {
    let n = lines.count().min(SNIFF_LINES);
    let mut seen = 0;
    let mut dated = 0;
    for i in 0..n {
        let line = lines.line(i);
        if line.trim().is_empty() {
            continue;
        }
        seen += 1;
        if starts_with_time(line) {
            dated += 1;
        }
    }
    seen >= 3 && dated * 2 > seen
}

/// `2026-11-16 10:00`, `2026-11-16T10:00`, `[2026-11-16 ...`, or syslog's
/// `Nov 16 10:00:00`.
fn starts_with_time(line: &str) -> bool {
    let b = line.trim_start_matches('[').as_bytes();
    let digit = |i: usize| b.get(i).is_some_and(u8::is_ascii_digit);
    let iso = (0..4).all(digit)
        && b.get(4) == Some(&b'-')
        && digit(5)
        && digit(6)
        && b.get(7) == Some(&b'-')
        && digit(8)
        && digit(9)
        && matches!(b.get(10), Some(b' ' | b'T'))
        && digit(11)
        && digit(12)
        && b.get(13) == Some(&b':');
    const MONTHS: [&[u8]; 12] = [
        b"Jan", b"Feb", b"Mar", b"Apr", b"May", b"Jun", b"Jul", b"Aug", b"Sep", b"Oct", b"Nov",
        b"Dec",
    ];
    let syslog = b.len() >= 15
        && MONTHS.contains(&&b[..3])
        && b[3] == b' '
        && (b[4] == b' ' || b[4].is_ascii_digit())
        && b[5].is_ascii_digit()
        && b[6] == b' '
        && digit(7)
        && digit(8)
        && b[9] == b':';
    iso || syslog
}

/// How serious a line is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Level {
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl Level {
    fn from_byte(b: u8) -> Option<Level> {
        match b {
            1 => Some(Level::Debug),
            2 => Some(Level::Info),
            3 => Some(Level::Warn),
            4 => Some(Level::Error),
            _ => None,
        }
    }
}

/// The level a line says it has: the first level word in its first columns,
/// as a whole word in any case, or syslog's `<n>` at its start.
pub fn level_of(line: &str) -> Option<Level> {
    if let Some(rest) = line.strip_prefix('<')
        && let Some((n, _)) = rest.split_once('>')
        && (1..=3).contains(&n.len())
        && let Ok(n) = n.parse::<u16>()
    {
        return Some(match n % 8 {
            0..=3 => Level::Error,
            4 => Level::Warn,
            5 | 6 => Level::Info,
            _ => Level::Debug,
        });
    }
    let head: String = line.chars().take(LEVEL_COLUMNS).collect();
    head.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| (3..=8).contains(&w.len()))
        .find_map(|w| match w.to_ascii_uppercase().as_str() {
            "FATAL" | "CRIT" | "CRITICAL" | "PANIC" | "ERROR" | "ERR" | "EMERG" | "ALERT"
            | "SEVERE" => Some(Level::Error),
            "WARN" | "WARNING" => Some(Level::Warn),
            "INFO" | "NOTICE" => Some(Level::Info),
            "DEBUG" | "TRACE" => Some(Level::Debug),
            _ => None,
        })
}

/// The level of each line of a text read whole: a line with none of its own
/// takes the one before, for at most [`INHERIT`] lines. One byte per line.
pub fn levels(lines: &(impl LineSource + ?Sized)) -> Vec<u8> {
    let mut out = Vec::with_capacity(lines.count());
    let mut last: Option<(Level, usize)> = None;
    for i in 0..lines.count() {
        let own = level_of(lines.line(i));
        let level = match own {
            Some(l) => {
                last = Some((l, i));
                Some(l)
            }
            None => last.filter(|&(_, at)| i - at <= INHERIT).map(|(l, _)| l),
        };
        out.push(level.map_or(0, |l| l as u8));
    }
    out
}

/// The text color of each piece of a line, from its ANSI colors (`None`: the
/// text's own color).
pub type AnsiLine = Vec<(Option<(u8, u8, u8)>, String)>;

/// A text without its ANSI sequences.
pub struct Stripped {
    pub text: String,
    /// The colors of each line, when any of them had one.
    pub colors: Option<Vec<AnsiLine>>,
    /// Sequences taken out that were not a text color.
    pub removed: usize,
}

/// The text has an escape sequence (7 or 8 bits) to take out.
pub fn has_escapes(text: &str) -> bool {
    text.contains([
        '\u{1b}', '\u{9b}', '\u{9d}', '\u{90}', '\u{98}', '\u{9e}', '\u{9f}',
    ])
}

/// Takes the ANSI sequences out of `text`: the text color of SGR (`ESC [ ...
/// m`) is kept; every other sequence (cursor, OSC 52 and its clipboard, OSC 8
/// links, DCS, APC...) is dropped and counted. Tabs are expanded here, so the
/// colors stay on their characters. Other control characters are left for the
/// viewer to show as `·`.
pub fn strip_ansi(text: &str) -> Stripped {
    // The 8-bit forms (C1) as their 7-bit ones, so the parser sees them.
    let mut seven = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\u{9b}' | '\u{9d}' | '\u{90}' | '\u{98}' | '\u{9e}' | '\u{9f}' | '\u{9c}' => {
                seven.push('\u{1b}');
                seven.push(char::from_u32(c as u32 - 0x40).unwrap_or('['));
            }
            _ => seven.push(c),
        }
    }
    let mut p = Performer {
        text: String::with_capacity(seven.len()),
        line: Vec::new(),
        lines: Vec::new(),
        fg: None,
        col: 0,
        removed: 0,
        colored: false,
    };
    let mut parser = vte::Parser::new();
    parser.advance(&mut p, seven.as_bytes());
    if !p.line.is_empty() || !p.text.ends_with('\n') {
        p.end_line();
    }
    Stripped {
        colors: p.colored.then_some(p.lines),
        text: p.text,
        removed: p.removed,
    }
}

struct Performer {
    text: String,
    line: AnsiLine,
    lines: Vec<AnsiLine>,
    fg: Option<(u8, u8, u8)>,
    col: usize,
    removed: usize,
    colored: bool,
}

impl Performer {
    fn put(&mut self, s: &str) {
        self.text.push_str(s);
        self.col += s.chars().count();
        match self.line.last_mut() {
            Some((c, piece)) if *c == self.fg => piece.push_str(s),
            _ => self.line.push((self.fg, s.to_string())),
        }
    }

    fn end_line(&mut self) {
        self.lines.push(std::mem::take(&mut self.line));
        self.col = 0;
    }
}

impl vte::Perform for Performer {
    fn print(&mut self, c: char) {
        let mut buf = [0u8; 4];
        self.put(c.encode_utf8(&mut buf));
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.text.push('\n');
                self.end_line();
            }
            b'\t' => {
                let n = 8 - self.col % 8;
                self.put(&" ".repeat(n));
            }
            // Shown as `·` by the viewer, as in any text.
            b => {
                let c = char::from(b);
                let mut buf = [0u8; 4];
                self.put(c.encode_utf8(&mut buf));
            }
        }
    }

    fn csi_dispatch(&mut self, params: &vte::Params, inter: &[u8], ignore: bool, action: char) {
        if action == 'm' && inter.is_empty() && !ignore {
            let text: Vec<String> = params
                .iter()
                .map(|sub| sub.iter().map(u16::to_string).collect::<Vec<_>>().join(":"))
                .collect();
            self.fg = highlight::sgr(&text.join(";"), self.fg);
            self.colored |= self.fg.is_some();
        } else {
            self.removed += 1;
        }
    }

    fn osc_dispatch(&mut self, _: &[&[u8]], _: bool) {
        self.removed += 1;
    }

    fn hook(&mut self, _: &vte::Params, _: &[u8], _: bool, _: char) {
        self.removed += 1;
    }

    fn esc_dispatch(&mut self, _: &[u8], _: bool, _: u8) {
        self.removed += 1;
    }
}

/// The colors of a log: by the level of each line, and by the ANSI colors of
/// a text read whole, which win where a line has them.
pub fn colors(
    levels: Option<Arc<Vec<u8>>>,
    ansi: Option<Arc<Vec<AnsiLine>>>,
) -> Option<Highlighter> {
    let palette = [
        highlight::tone((136, 138, 133)), // debug
        crate::config::theme().text,      // info
        highlight::tone((237, 212, 0)),   // warning
        highlight::tone((239, 41, 41)),   // error
    ];
    let text = crate::config::theme().text;
    let colors: LineColors = Box::new(move |i, line: &str| {
        if let Some(ansi) = ansi.as_ref().and_then(|a| a.get(i))
            && ansi.iter().any(|(c, _)| c.is_some())
        {
            return ansi
                .iter()
                .map(|(c, s)| (c.map_or(text, highlight::tone), s.clone()))
                .collect();
        }
        let level = match &levels {
            Some(l) => l.get(i).copied().and_then(Level::from_byte),
            None => level_of(line),
        };
        match level {
            Some(l) => vec![(
                palette[l as usize - 1],
                line.chars().take(COLORED_CHARS).collect(),
            )],
            None => Vec::new(),
        }
    });
    Highlighter::per_line("Log", colors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logs_by_name() {
        for name in [
            "a.log",
            "x/app.LOG",
            "app.log.1",
            "app.log.2026-11-16",
            "app.log.3.gz",
            "syslog",
            "messages.1",
            "run.out",
            "run.err",
            "boot.journal.txt",
        ] {
            assert!(is_log_name(Path::new(name)), "{name}");
        }
        for name in ["log", "a.txt", "catalog.rs", "blog.md"] {
            assert!(!is_log_name(Path::new(name)), "{name}");
        }
    }

    #[test]
    fn logs_by_contents() {
        let log: Vec<String> = [
            "2026-11-16 10:00:01 INFO start",
            "2026-11-16T10:00:02Z WARN slow",
            "  at frame 1",
            "[2026-11-16 10:00:03] ERROR bad",
        ]
        .map(String::from)
        .to_vec();
        assert!(looks_like_log(log.as_slice()));
        let syslog: Vec<String> = [
            "Nov 16 10:00:00 host a",
            "Nov  6 10:00:01 host b",
            "Nov 16 10:00:02 host c",
        ]
        .map(String::from)
        .to_vec();
        assert!(looks_like_log(syslog.as_slice()));
        let code: Vec<String> = ["fn main() {", "    let x = 2026;", "}"]
            .map(String::from)
            .to_vec();
        assert!(!looks_like_log(code.as_slice()));
    }

    #[test]
    fn levels_by_word_syslog_and_json() {
        assert_eq!(level_of("2026-11-16 ERROR: disk"), Some(Level::Error));
        assert_eq!(level_of("[warning] slow"), Some(Level::Warn));
        assert_eq!(level_of("INFO no error here"), Some(Level::Info));
        assert_eq!(
            level_of(r#"{"level":"debug","msg":"x"}"#),
            Some(Level::Debug)
        );
        assert_eq!(level_of("<3>kernel: oops"), Some(Level::Error));
        assert_eq!(level_of("<14>user: hi"), Some(Level::Info));
        assert_eq!(level_of("terrors and informed"), None, "whole words only");
        let far = format!("{}ERROR", "x".repeat(200));
        assert_eq!(level_of(&far), None, "only the first columns");
        let lines: Vec<String> = ["ERROR a", "  at b", "INFO c", "d"]
            .map(String::from)
            .to_vec();
        assert_eq!(levels(lines.as_slice()), [4, 4, 2, 2]);
    }

    #[test]
    fn ansi_keeps_the_color_and_drops_the_rest() {
        let s = strip_ansi(
            "\x1b[31mred\x1b[0m plain\n\x1b]52;c;ZXZpbA==\x07x\x1b[2Jy\n\u{9b}32mgreen\n",
        );
        assert_eq!(s.text, "red plain\nxy\ngreen\n");
        assert_eq!(s.removed, 2, "the OSC 52 and the clear screen");
        let colors = s.colors.unwrap();
        assert_eq!(colors.len(), 3);
        assert!(colors[0][0].0.is_some());
        assert_eq!(colors[0][1], (None, " plain".to_string()));
        assert!(colors[2][0].0.is_some(), "the 8-bit CSI too");
        // A tab is expanded, so the colors stay on their characters.
        let s = strip_ansi("a\tb\n");
        assert_eq!(s.text, "a       b\n");
        assert!(s.colors.is_none());
        // DCS and APC, and a sequence left open at the end.
        let s = strip_ansi("a\x1bPq#0\x1b\\b\x1b_hidden\x1b\\c\x1b[");
        assert_eq!(s.text, "abc");
    }

    proptest::proptest! {
        #[test]
        fn no_escape_survives(text in "[a-z \t\n\u{1b}\u{9b}\u{9d}\u{90}\u{9c}\u{7}\\[\\];0-9m]{0,300}") {
            let s = strip_ansi(&text);
            let clean = !has_escapes(&s.text);
            proptest::prop_assert!(clean);
            let lines: Vec<String> = s.text.lines().map(String::from).collect();
            let _ = levels(lines.as_slice());
        }
    }
}
