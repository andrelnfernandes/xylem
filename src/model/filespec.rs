//! Filespec: XTree's file filter (`F` key).
//!
//! Specs separated by space, `;` or `,` (inside quotes, a name
//! may contain spaces). Three kinds, as in ZTreeWin:
//!
//! - **name**: glob (`*.rs`, `a?c*`, `[abc]*`), case-insensitive, or a
//!   regular expression between slashes (`/^report-\d{4}/`, `/\.rs$/i`
//!   ignoring case); a name holds no `/`, so the two never mix up;
//! - **date**: `<`, `=`, `>`, `<=`, `>=` or `<>` and a date (`>01/05/2026`,
//!   `=TODAY`, `>TODAY-7`);
//! - **size**: the same with `s` (`>=s200000`, `<s10K`, `>s1G`).
//!
//! A leading `-` negates the spec. Within each kind, positive specs
//! combine with "or" (except `>a` immediately followed by `<b`, which forms
//! the range between the two) and negative ones with "and not"; across kinds, "and" applies.

use std::ffi::OsStr;
use std::time::SystemTime;

use chrono::{DateTime, Local, NaiveDate};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use regex::bytes::Regex;

use super::regex_build;
use super::tree::FileEntry;

/// Order of day, month and year in typed dates (the one from the configured
/// date format). `yyyy-mm-dd` is always accepted.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DateOrder {
    #[default]
    Ymd,
    Dmy,
    Mdy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Less,
    LessEq,
    Equal,
    GreaterEq,
    Greater,
    NotEqual,
}

impl Op {
    fn holds(self, a: i64, b: i64) -> bool {
        match self {
            Op::Less => a < b,
            Op::LessEq => a <= b,
            Op::Equal => a == b,
            Op::GreaterEq => a >= b,
            Op::Greater => a > b,
            Op::NotEqual => a != b,
        }
    }

    /// `-<` is `>=`, `-=` is `<>`, and so on.
    fn negated(self) -> Op {
        match self {
            Op::Less => Op::GreaterEq,
            Op::LessEq => Op::Greater,
            Op::Equal => Op::NotEqual,
            Op::GreaterEq => Op::Less,
            Op::Greater => Op::LessEq,
            Op::NotEqual => Op::Equal,
        }
    }
}

/// A date condition (days since 1970, in local time) or a size condition.
#[derive(Debug, Clone, Copy)]
struct Cond {
    op: Op,
    value: i64,
}

/// Conditions of one kind (date or size), already grouped.
#[derive(Debug, Default)]
struct Conds {
    /// Positive: any one is enough (each may be a two-part range).
    any: Vec<Vec<Cond>>,
    /// `<>`: none may be equal.
    none: Vec<Cond>,
}

impl Conds {
    fn from_list(list: Vec<Cond>) -> Self {
        let mut conds = Conds::default();
        let mut iter = list.into_iter().peekable();
        while let Some(c) = iter.next() {
            if c.op == Op::NotEqual {
                conds.none.push(c);
                continue;
            }
            // ">a" followed by "<b" (b > a): a range, both must hold together.
            let lower = matches!(c.op, Op::Greater | Op::GreaterEq);
            if lower
                && let Some(next) = iter.peek()
                && matches!(next.op, Op::Less | Op::LessEq)
                && next.value > c.value
            {
                let next = iter.next().unwrap();
                conds.any.push(vec![c, next]);
            } else {
                conds.any.push(vec![c]);
            }
        }
        conds
    }

    fn matches(&self, value: i64) -> bool {
        let positive = self.any.is_empty()
            || self
                .any
                .iter()
                .any(|g| g.iter().all(|c| c.op.holds(value, c.value)));
        positive && self.none.iter().all(|c| c.op.holds(value, c.value))
    }

    fn is_empty(&self) -> bool {
        self.any.is_empty() && self.none.is_empty()
    }
}

pub struct Filespec {
    text: String,
    include: Option<GlobSet>,
    exclude: GlobSet,
    /// Regular expressions (`/.../`): including ones add to the globs ("or"),
    /// excluding ones take away.
    include_re: Vec<Regex>,
    exclude_re: Vec<Regex>,
    dates: Conds,
    sizes: Conds,
}

impl Default for Filespec {
    fn default() -> Self {
        Filespec::parse("*").expect("'*' is a valid filespec")
    }
}

impl Filespec {
    pub fn parse(text: &str) -> Result<Self, String> {
        Self::parse_with(text, DateOrder::default())
    }

    pub fn parse_with(text: &str, order: DateOrder) -> Result<Self, String> {
        let mut include = GlobSetBuilder::new();
        let mut exclude = GlobSetBuilder::new();
        let mut has_include = false;
        let (mut dates, mut sizes) = (Vec::new(), Vec::new());
        let (mut include_re, mut exclude_re) = (Vec::new(), Vec::new());
        let mut parts = Vec::new();

        for Token {
            text: part,
            quoted,
            negated,
            regex,
        } in tokenize(text)
        {
            let minus = if negated { "-" } else { "" };
            if let Some(Slashes { flags, closed }) = regex {
                if !closed {
                    return Err(format!("regex '{minus}/{part}' needs a closing /"));
                }
                if let Some(f) = flags.chars().find(|&c| c != 'i') {
                    return Err(format!(
                        "unknown flag '{f}' in '{minus}/{part}/{flags}' (only i)"
                    ));
                }
                let ignore_case = !flags.is_empty();
                let re = regex_build::build_bytes(&part, ignore_case)?;
                if negated {
                    exclude_re.push(re);
                } else {
                    include_re.push(re);
                }
                let flags = if ignore_case { "i" } else { "" };
                parts.push(format!("{minus}/{part}/{flags}"));
                continue;
            }
            let rest = part.as_str();
            if !quoted && rest.starts_with(['<', '=', '>']) {
                let (op, value) = split_op(rest).ok_or_else(|| format!("invalid spec '{part}'"))?;
                let op = if negated { op.negated() } else { op };
                match value.strip_prefix(['s', 'S']) {
                    Some(size) => sizes.push(Cond {
                        op,
                        value: parse_size(size).ok_or_else(|| format!("invalid size '{size}'"))?,
                    }),
                    None => dates.push(Cond {
                        op,
                        value: parse_date(value, order)
                            .ok_or_else(|| format!("invalid date '{value}'"))?,
                    }),
                }
                parts.push(format!("{}{part}", if negated { "-" } else { "" }));
                continue;
            }
            let set = if negated {
                &mut exclude
            } else {
                has_include = true;
                &mut include
            };
            let glob = GlobBuilder::new(rest)
                .case_insensitive(true)
                .literal_separator(true)
                .build()
                .map_err(|e| format!("invalid pattern '{rest}': {}", e.kind()))?;
            set.add(glob);
            parts.push(if quoted {
                format!("{minus}\"{rest}\"")
            } else {
                format!("{minus}{rest}")
            });
        }

        let build = |b: GlobSetBuilder| b.build().map_err(|e| e.to_string());
        Ok(Filespec {
            text: if parts.is_empty() {
                "*".into()
            } else {
                parts.join(" ")
            },
            include: if has_include {
                Some(build(include)?)
            } else {
                None
            },
            exclude: build(exclude)?,
            include_re,
            exclude_re,
            dates: Conds::from_list(dates),
            sizes: Conds::from_list(sizes),
        })
    }

    /// Normalized text, for display.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Names only.
    pub fn matches(&self, name: &OsStr) -> bool {
        let bytes = name.as_encoded_bytes();
        let included = match &self.include {
            None if self.include_re.is_empty() => true,
            globs => {
                globs.as_ref().is_some_and(|s| s.is_match(name))
                    || self.include_re.iter().any(|r| r.is_match(bytes))
            }
        };
        included
            && !self.exclude.is_match(name)
            && !self.exclude_re.iter().any(|r| r.is_match(bytes))
    }

    /// Name, date and size.
    pub fn matches_entry(&self, f: &FileEntry) -> bool {
        if !self.matches(&f.name) {
            return false;
        }
        if !self.dates.is_empty() {
            let Some(day) = f.modified.map(local_day) else {
                return false;
            };
            if !self.dates.matches(day) {
                return false;
            }
        }
        self.sizes.matches(f.size as i64)
    }
}

/// One spec: the text (without the negating `-`, the quotes or the
/// slashes of a regular expression).
struct Token {
    text: String,
    quoted: bool,
    negated: bool,
    regex: Option<Slashes>,
}

/// What follows a regular expression: its flags, and whether the closing
/// slash was there.
struct Slashes {
    flags: String,
    closed: bool,
}

fn is_separator(c: char) -> bool {
    matches!(c, ' ' | ';' | ',')
}

/// Splits the text into specs. Inside quotes, separators and a leading `-`
/// are part of the name; the `-` only negates outside them (`-"a b*"`). A
/// spec that starts with `/` is a regular expression up to the next `/` not
/// escaped by `\`: separators and quotes inside it are part of it
/// (`/\d{2,4}/`), and the letters after it, up to a separator, its flags.
fn tokenize(text: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let (mut in_quotes, mut quoted, mut negated) = (false, false, false);
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '/' if !in_quotes && !quoted && cur.is_empty() => {
                let mut closed = false;
                while let Some(c) = chars.next() {
                    match c {
                        '\\' => {
                            cur.push(c);
                            if let Some(next) = chars.next() {
                                cur.push(next);
                            }
                        }
                        '/' => {
                            closed = true;
                            break;
                        }
                        c => cur.push(c),
                    }
                }
                let mut flags = String::new();
                while let Some(c) = chars.next_if(|&c| !is_separator(c)) {
                    flags.push(c);
                }
                out.push(Token {
                    text: std::mem::take(&mut cur),
                    quoted: false,
                    negated,
                    regex: Some(Slashes { flags, closed }),
                });
                negated = false;
            }
            '"' => {
                in_quotes = !in_quotes;
                quoted = true;
            }
            c if is_separator(c) && !in_quotes => {
                if !cur.is_empty() {
                    out.push(Token {
                        text: std::mem::take(&mut cur),
                        quoted,
                        negated,
                        regex: None,
                    });
                }
                (quoted, negated) = (false, false);
            }
            '-' if !in_quotes && !quoted && cur.is_empty() && !negated => negated = true,
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(Token {
            text: cur,
            quoted,
            negated,
            regex: None,
        });
    }
    out
}

fn split_op(s: &str) -> Option<(Op, &str)> {
    let ops = [
        ("<>", Op::NotEqual),
        ("<=", Op::LessEq),
        ("=<", Op::LessEq),
        (">=", Op::GreaterEq),
        ("=>", Op::GreaterEq),
        ("<", Op::Less),
        (">", Op::Greater),
        ("=", Op::Equal),
    ];
    ops.iter()
        .find_map(|(t, op)| s.strip_prefix(t).map(|rest| (*op, rest)))
        .filter(|(_, rest)| !rest.is_empty())
}

/// "200000", "10K", "1.5M", "2G" (K = 1024).
fn parse_size(s: &str) -> Option<i64> {
    let s = s.trim();
    let (num, mult) = match s.chars().last()?.to_ascii_uppercase() {
        'K' => (&s[..s.len() - 1], 1024.0),
        'M' => (&s[..s.len() - 1], 1024.0 * 1024.0),
        'G' => (&s[..s.len() - 1], 1024.0 * 1024.0 * 1024.0),
        _ => (s, 1.0),
    };
    let n: f64 = num.parse().ok().filter(|n: &f64| *n >= 0.0)?;
    Some((n * mult) as i64)
}

/// Day (since 1970) of a typed date: `TODAY`, `TODAY-n`, or day,
/// month and year in the configured order, separated by `/`, `.` or `-`.
fn parse_date(s: &str, order: DateOrder) -> Option<i64> {
    let upper = s.to_ascii_uppercase();
    if let Some(rest) = upper.strip_prefix("TODAY") {
        let back: i64 = match rest.strip_prefix('-') {
            Some(n) => n.parse().ok()?,
            None if rest.is_empty() => 0,
            None => return None,
        };
        return Some(day_number(Local::now().date_naive()) - back);
    }
    let nums: Vec<&str> = s.split(['/', '.', '-']).collect();
    let [a, b, c] = nums[..] else {
        return None;
    };
    let (y, m, d) = if a.len() == 4 {
        (a, b, c)
    } else {
        match order {
            DateOrder::Ymd => (a, b, c),
            DateOrder::Dmy => (c, b, a),
            DateOrder::Mdy => (c, a, b),
        }
    };
    let mut year: i32 = y.parse().ok()?;
    match y.len() {
        2 => year += if year < 70 { 2000 } else { 1900 },
        4 => {}
        _ => return None,
    }
    let date = NaiveDate::from_ymd_opt(year, m.parse().ok()?, d.parse().ok()?)?;
    Some(day_number(date))
}

/// A typed date and time (New date): `now`, `today`, or the date in the
/// configured order optionally followed by `HH:MM` or `HH:MM:SS`. Returns the
/// date and, if present, the time.
pub fn parse_date_time(
    text: &str,
    order: DateOrder,
) -> Option<(NaiveDate, Option<chrono::NaiveTime>)> {
    let text = text.trim();
    let now = Local::now().naive_local();
    match text.to_ascii_lowercase().as_str() {
        "now" => return Some((now.date(), Some(now.time()))),
        "today" => return Some((now.date(), None)),
        _ => {}
    }
    let mut parts = text.split_whitespace();
    let date_part = parts.next()?;
    let day = parse_date(date_part, order)?;
    let date = NaiveDate::default().checked_add_signed(chrono::Duration::days(day))?;
    let time = match parts.next() {
        None => None,
        Some(t) => {
            let t = chrono::NaiveTime::parse_from_str(t, "%H:%M:%S")
                .or_else(|_| chrono::NaiveTime::parse_from_str(t, "%H:%M"))
                .ok()?;
            Some(t)
        }
    };
    if parts.next().is_some() {
        return None;
    }
    Some((date, time))
}

fn day_number(d: NaiveDate) -> i64 {
    d.signed_duration_since(NaiveDate::default()).num_days()
}

/// Local day of a modification date.
fn local_day(t: SystemTime) -> i64 {
    day_number(DateTime::<Local>::from(t).date_naive())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// Specs made of the pieces the parser understands: globs, quotes,
    /// negation, operators, dates (some absurd) and sizes.
    fn spec() -> impl Strategy<Value = String> {
        let piece = prop::sample::select(vec![
            "*",
            "?",
            "[a-c]",
            "[!x]",
            "{a,b}",
            "a",
            "B",
            ".",
            "rs",
            " ",
            ",",
            ";",
            "-",
            "\"",
            "<",
            ">",
            "=",
            "<=",
            ">=",
            "<>",
            "s",
            "S",
            "0",
            "1",
            "9",
            "K",
            "M",
            "G",
            "T",
            "TODAY",
            "today",
            "TODAY-7",
            "TODAY+99999999999",
            "2026-10-24",
            "01/05/2026",
            "31/02/2026",
            "9999-99-99",
            "99999999999",
            "/",
            "\\",
            "ç",
            "/^a/",
            "/\\d{2,4}/i",
            "/[a-c] ,;/",
            "/(/",
            "/i",
            "x",
        ]);
        prop::collection::vec(piece, 0..10).prop_map(|p| p.concat())
    }

    proptest! {
        /// Whatever is typed, parsing and matching never panic.
        #[test]
        fn never_panics(
            text in prop_oneof![spec(), any::<String>()],
            name in "\\PC{0,16}",
            size in any::<u64>(),
            secs in 0u64..8_000_000_000,
        ) {
            if let Ok(f) = Filespec::parse(&text) {
                let entry = FileEntry {
                    name: name.into(),
                    size,
                    disk: size,
                    link: None,
                    modified: Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs)),
                    mode: 0o644,
                    kind: crate::model::tree::FileKind::File,
                    tagged: false,
                };
                let _ = f.matches_entry(&entry);
            }
        }

        /// The normalized text means the same: parsing it again gives it back.
        #[test]
        fn normalized_text_is_stable(text in spec()) {
            if let Ok(f) = Filespec::parse(&text) {
                let again = Filespec::parse(f.text()).expect("normalized text parses");
                prop_assert_eq!(again.text(), f.text());
            }
        }

        /// A plain name matches itself, whatever the case.
        #[test]
        fn a_name_matches_itself(name in "[a-zA-Z0-9_][a-zA-Z0-9_.]{0,15}") {
            prop_assert!(m(&name, &name));
            prop_assert!(m(&name.to_uppercase(), &name.to_lowercase()));
        }

        /// `-p` keeps exactly what `p` leaves out; two patterns are "or".
        #[test]
        fn negation_and_union(
            p in "[a-c*?]{1,4}",
            q in "[a-c*?]{1,4}",
            name in "[a-c.]{1,6}",
        ) {
            prop_assert_eq!(m(&format!("-{p}"), &name), !m(&p, &name));
            prop_assert_eq!(m(&format!("{p} {q}"), &name), m(&p, &name) || m(&q, &name));
        }
    }

    fn m(spec: &str, name: &str) -> bool {
        Filespec::parse(spec).unwrap().matches(OsStr::new(name))
    }

    #[test]
    fn default_matches_everything() {
        let f = Filespec::default();
        assert!(f.matches(OsStr::new("anything.txt")));
        assert!(f.matches(OsStr::new(".hidden")));
        assert_eq!(f.text(), "*");
    }

    #[test]
    fn includes_and_excludes() {
        assert!(m("*.rs *.toml", "main.rs"));
        assert!(m("*.rs;*.toml", "Cargo.toml"));
        assert!(!m("*.rs,*.toml", "README.md"));
        assert!(m("-*.o", "main.c"));
        assert!(!m("-*.o", "main.o"));
        assert!(!m("*.c -test*", "test_main.c"));
    }

    #[test]
    fn ignores_case() {
        assert!(m("*.JPG", "photo.jpg"));
        assert!(m("readme*", "README.md"));
    }

    #[test]
    fn normalizes_text() {
        assert_eq!(
            Filespec::parse("  *.rs;;*.md ").unwrap().text(),
            "*.rs *.md"
        );
        assert_eq!(Filespec::parse("").unwrap().text(), "*");
    }

    #[test]
    fn rejects_bad_pattern() {
        assert!(Filespec::parse("[abc").is_err());
    }

    fn file(name: &str, size: u64, date: &str) -> FileEntry {
        let day = NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
        let local = day
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap();
        FileEntry {
            name: name.into(),
            size,
            disk: size,
            link: None,
            modified: Some(local.into()),
            mode: 0o644,
            kind: crate::model::tree::FileKind::File,
            tagged: false,
        }
    }

    fn me(spec: &str, f: &FileEntry) -> bool {
        Filespec::parse_with(spec, DateOrder::Dmy)
            .unwrap()
            .matches_entry(f)
    }

    #[test]
    fn size_specs() {
        let small = file("a", 100, "2026-05-10");
        let big = file("b", 300_000, "2026-05-10");
        assert!(me("=>s200000", &big) && !me("=>s200000", &small));
        assert!(me("<s1K", &small) && !me("<s1K", &big));
        assert!(me(">s0.1M", &big));
        assert!(me("-<s200000", &big), "-< is >=");
        assert!(!me("<>s100", &small));
        // Range: >a followed by <b.
        assert!(me(">s50,<s200", &small) && !me(">s50,<s200", &big));
        assert!(Filespec::parse("<sabc").is_err());
    }

    #[test]
    fn date_specs_follow_configured_order() {
        let may10 = file("x.txt", 1, "2026-05-10");
        let june = file("y.txt", 1, "2026-06-02");
        // May 2026, in dd/mm/yyyy (and yyyy-mm-dd always works).
        assert!(me("=>01/05/2026,=<31/05/2026", &may10));
        assert!(!me("=>01/05/2026,=<31/05/2026", &june));
        assert!(me(">2026-05-31", &june));
        assert!(me("=10.05.26", &may10));
        // Outside May: "<1/5" or ">31/5".
        assert!(me("<01/05/2026,>31/05/2026", &june));
        assert!(!me("<01/05/2026,>31/05/2026", &may10));
        let mdy = Filespec::parse_with("=05/10/2026", DateOrder::Mdy).unwrap();
        assert!(mdy.matches_entry(&may10));
        assert!(Filespec::parse("=31/02/2026").is_err());
    }

    #[test]
    fn today_and_combined_types() {
        let now = FileEntry {
            modified: Some(SystemTime::now()),
            ..file("n.txt", 10, "2026-01-01")
        };
        let old = file("o.txt", 10, "2001-01-01");
        assert!(me("=TODAY", &now) && !me("=TODAY", &old));
        assert!(me(">TODAY-7", &now) && me("<TODAY-7", &old));
        // Name, date and size together: "and".
        assert!(me("*.txt,=TODAY,<s100", &now));
        assert!(!me("*.rs,=TODAY", &now));
        assert!(!me("*.txt,=TODAY,>s100", &now));
    }

    #[test]
    fn parses_date_and_time_for_new_date() {
        let d = |y, m, dd| NaiveDate::from_ymd_opt(y, m, dd).unwrap();
        let t = |h, m| chrono::NaiveTime::from_hms_opt(h, m, 0);
        assert_eq!(
            parse_date_time("12/10/2026 14:30", DateOrder::Dmy),
            Some((d(2026, 10, 12), t(14, 30)))
        );
        assert_eq!(
            parse_date_time("2026-10-12", DateOrder::Dmy),
            Some((d(2026, 10, 12), None))
        );
        assert!(parse_date_time("now", DateOrder::Ymd).unwrap().1.is_some());
        assert!(
            parse_date_time("TODAY", DateOrder::Ymd)
                .unwrap()
                .1
                .is_none()
        );
        assert_eq!(parse_date_time("12/10/2026 25:00", DateOrder::Dmy), None);
        assert_eq!(parse_date_time("ontem", DateOrder::Dmy), None);
    }

    #[test]
    fn quoted_names_keep_spaces() {
        assert!(m("\"meu arquivo*\"", "meu arquivo.txt"));
        assert!(!m("\"meu arquivo*\"", "meu.txt"));
        assert!(m("-\"a b*\"", "c.txt"));
        assert!(!m("-\"a b*\"", "a b.txt"));
        // Inside quotes, the hyphen is part of the name.
        assert!(m("\"-*.txt\"", "-notas.txt"));
        assert!(!m("\"-*.txt\"", "notas.txt"));
        assert_eq!(
            Filespec::parse("-\"a b*\" -*.o").unwrap().text(),
            "-\"a b*\" -*.o"
        );
    }

    #[test]
    fn regex_between_slashes() {
        assert!(m(r"/^report-\d{4}/", "report-2026.pdf"));
        assert!(!m(r"/^report-\d{4}/", "report-26.pdf"));
        // Case counts, unless /i.
        assert!(!m(r"/^Report/", "report.txt"));
        assert!(m(r"/^Report/i", "report.txt"));
        // Separators and quotes inside the slashes belong to the expression.
        assert!(m(r"/^\d{2,4}$/", "123"));
        assert!(m("/a b;c/", "xa b;cx"));
        assert!(m("/\"x\"/", "a\"x\"b"));
        // An escaped slash does not close it.
        assert!(!m(r"/a\/b/", "ab"));
    }

    #[test]
    fn regex_combines_with_globs_and_negation() {
        // "or" with the globs, "and not" with the negated ones.
        assert!(m(r"*.rs /^Cargo\./", "Cargo.toml"));
        assert!(m(r"*.rs /^Cargo\./", "main.rs"));
        assert!(!m(r"*.rs /^Cargo\./", "README.md"));
        assert!(m(r"*.rs -/^test_/", "main.rs"));
        assert!(!m(r"*.rs -/^test_/", "test_main.rs"));
        assert!(m(r"-/\.o$/", "main.c") && !m(r"-/\.o$/", "main.o"));
        // With date and size: "and".
        let f = file("report-2026.txt", 100, "2026-05-10");
        assert!(me(r"/^report-\d+/,<s1K", &f));
        assert!(!me(r"/^report-\d+/,>s1K", &f));
        assert!(!me(r"/^draft/,<s1K", &f));
    }

    #[test]
    fn regex_errors_say_what_and_where() {
        let e = Filespec::parse("*.rs /ab(c/").err().unwrap();
        assert!(e.contains("column 3"), "{e}");
        let e = Filespec::parse("/abc").err().unwrap();
        assert!(e.contains("closing /"), "{e}");
        let e = Filespec::parse("/abc/x").err().unwrap();
        assert!(e.contains("unknown flag 'x'"), "{e}");
        let e = Filespec::parse(r"/\w{1000}{1000}/").err().unwrap();
        assert!(e.contains("too big"), "{e}");
    }

    #[test]
    fn regex_text_is_kept_in_the_normalized_text() {
        let f = Filespec::parse(r"*.rs  -/^a b/  /x{2,3}/ii").unwrap();
        assert_eq!(f.text(), r"*.rs -/^a b/ /x{2,3}/i");
        // A slash after other text is not an expression (and matches nothing).
        assert!(!m("a/b", "a"));
    }

    #[cfg(unix)]
    #[test]
    fn regex_sees_names_that_are_not_utf8() {
        use std::os::unix::ffi::OsStrExt;
        let name = OsStr::from_bytes(b"caf\xe9.txt");
        assert!(Filespec::parse(r"/^caf/").unwrap().matches(name));
        assert!(Filespec::parse(r"/\.txt$/").unwrap().matches(name));
        // `.` is a character: the stray byte is none.
        assert!(!Filespec::parse(r"/^caf.\./").unwrap().matches(name));
    }

    #[cfg(unix)]
    #[test]
    fn works_with_non_utf8_names() {
        use std::os::unix::ffi::OsStrExt;
        let name = OsStr::from_bytes(b"caf\xe9.txt");
        assert!(Filespec::parse("*.txt").unwrap().matches(name));
    }
}
