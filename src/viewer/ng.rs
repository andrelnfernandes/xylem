//! Norton Guides and Expert Help guides (`.ng`), shown as a document
//! (docs/plans/viewer-norton-guides.md): the title and credits, the menus,
//! then every entry in the order of the file, each with its links.
//!
//! The format is known thanks to Dave Pearson, who worked it out in the
//! 1990s and has kept Norton Guides readable ever since
//! (<https://www.davep.org/norton-guides/>). This reader is Xylem's own code,
//! written from his description of the format and from the plan; it includes
//! no code from his readers.
//!
//! A guide is a web of offsets that come from outside, and any of them can
//! lie: every offset is checked against the file, every count against its
//! own record, the spaces a line expands to have a limit, and nothing in a
//! guide reaches the terminal as a control: the `^` codes become styles, and
//! the bytes 1 to 31 the CP437 glyphs they stand for.

use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

use super::doc_events::{DocEvents, Look};
use super::markdown::{self, Doc};

/// The header, which is not encrypted.
const HEADER: usize = 378;
const TITLE: usize = 40;
const CREDIT: usize = 66;
/// The fixed part of every record: type, size and 22 more bytes.
const RECORD: usize = 26;
/// Every byte after the header is XORed with this.
const KEY: u8 = 0x1A;
/// Most records read: past them the guide is cut, and says so.
const MAX_RECORDS: usize = 100_000;
/// Most menus.
const MAX_MENUS: usize = 256;
/// Most bytes of one line before its spaces are expanded.
const MAX_LINE_BYTES: usize = 1024;
/// Most characters of one line once expanded.
const MAX_LINE_CHARS: usize = 4096;
/// Most characters of a menu title, a prompt or a see-also.
const MAX_PROMPT: usize = 128;
/// The limit of the Expert Help compiler: past it, the list is corrupt.
const MAX_SEE_ALSO: usize = 20;
const MAX_EVENTS: usize = 4_000_000;
/// The biggest guide read.
pub const MAX_SIZE: u64 = 64 << 20;

/// The glyphs of the bytes 0 to 31 in CP437 (0 is a space here).
const LOW: [char; 32] = [
    ' ', '☺', '☻', '♥', '♦', '♣', '♠', '•', '◘', '○', '◙', '♂', '♀', '♪', '♫', '☼', '►', '◄', '↕',
    '‼', '¶', '§', '▬', '↨', '↑', '↓', '→', '←', '∟', '↔', '▲', '▼',
];

/// A guide by its name (`.ng`) and its signature: two bytes alone match
/// too many files by chance.
pub fn is_guide(name: &Path, data: &[u8]) -> bool {
    name.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("ng"))
        && magic(data).is_some()
}

fn magic(data: &[u8]) -> Option<&'static str> {
    match data.get(..2)? {
        b"NG" => Some("Norton Guide"),
        b"EH" => Some("Expert Help"),
        _ => None,
    }
}

/// One character of a guide, by its code in CP437.
fn glyph(b: u8) -> char {
    match b {
        0..=31 => LOW[b as usize],
        127 => '⌂',
        128..=255 => crate::charset::CP437[(b - 128) as usize],
        _ => b as char,
    }
}

/// A piece of a line, with its look.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Span {
    pub text: String,
    pub bold: bool,
    pub underline: bool,
    pub reverse: bool,
    /// A DOS color attribute (`^A`): background in the high 4 bits.
    pub attr: Option<u8>,
}

pub type Line = Vec<Span>;

fn plain(line: &Line) -> String {
    line.iter().map(|s| s.text.as_str()).collect()
}

/// A line as a title: its runs of spaces (columns, in a guide) as one.
fn title_of(line: &Line) -> String {
    plain(line).split_whitespace().collect::<Vec<_>>().join(" ")
}

fn count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// A link: its text and the offset of the entry it leads to.
#[derive(Debug, Clone)]
pub struct Link {
    pub text: Line,
    pub to: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Menu {
    pub title: String,
    pub prompts: Vec<Link>,
}

#[derive(Debug, Clone)]
pub enum Body {
    /// A list of lines, each leading somewhere.
    Short(Vec<Link>),
    /// Text, and the see-also links.
    Long {
        lines: Vec<Line>,
        see_also: Vec<Link>,
    },
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub offset: u32,
    pub parent: Option<u32>,
    pub previous: Option<u32>,
    pub next: Option<u32>,
    pub body: Body,
}

#[derive(Debug, Default)]
pub struct Guide {
    pub made_with: &'static str,
    pub title: String,
    pub credits: Vec<String>,
    pub menus: Vec<Menu>,
    pub entries: Vec<Entry>,
    /// What was wrong with the guide, said once each.
    pub problems: Vec<String>,
    /// The time limit, the record limit or the end of the file cut it.
    pub cut: bool,
}

impl Guide {
    fn problem(&mut self, what: String) {
        if self.problems.len() < 50 && !self.problems.contains(&what) {
            self.problems.push(what);
        }
    }
}

/// The bytes of one record, decrypted, with reads that never pass its end.
struct Record<'a> {
    data: &'a [u8],
    at: usize,
}

impl Record<'_> {
    fn byte(&mut self) -> Option<u8> {
        let b = *self.data.get(self.at)?;
        self.at += 1;
        Some(b ^ KEY)
    }

    fn word(&mut self) -> Option<u16> {
        Some(u16::from(self.byte()?) | (u16::from(self.byte()?) << 8))
    }

    fn long(&mut self) -> Option<u32> {
        Some(u32::from(self.word()?) | (u32::from(self.word()?) << 16))
    }

    /// An offset: `0xFFFFFFFF` (and 0, the header) mean none.
    fn offset(&mut self) -> Option<Option<u32>> {
        let v = self.long()?;
        Some((v != u32::MAX && v != 0).then_some(v))
    }

    fn skip(&mut self, n: usize) -> Option<()> {
        let at = self.at.checked_add(n)?;
        (at <= self.data.len()).then(|| self.at = at)
    }

    /// The bytes up to a NUL, or `max` bytes, or the end of the record;
    /// decrypted. The NUL is passed.
    fn strz(&mut self, max: usize) -> Vec<u8> {
        let mut out = Vec::new();
        while out.len() < max {
            match self.byte() {
                Some(0) | None => return out,
                Some(b) => out.push(b),
            }
        }
        // Over the limit: the rest of this line is passed, up to its NUL.
        while let Some(b) = self.byte() {
            if b == 0 {
                break;
            }
        }
        out
    }

    fn left(&self) -> usize {
        self.data.len().saturating_sub(self.at)
    }
}

/// `0xFF n` is `n` spaces (`n` without sign; 0 adds nothing); `0xFF 0xFF`
/// is one space, as is a `0xFF` at the end. At most `MAX_LINE_CHARS` bytes
/// come out.
fn expand(raw: &[u8]) -> (Vec<u8>, bool) {
    let mut out = Vec::with_capacity(raw.len());
    let mut i = 0;
    let mut cut = false;
    while i < raw.len() {
        let room = MAX_LINE_CHARS - out.len();
        if room == 0 {
            cut = true;
            break;
        }
        if raw[i] == 0xFF {
            let n = match raw.get(i + 1) {
                None | Some(0xFF) => 1,
                Some(&n) => n as usize,
            };
            if n > room {
                cut = true;
            }
            out.extend(std::iter::repeat_n(b' ', n.min(room)));
            i += 2;
        } else {
            out.push(raw[i]);
            i += 1;
        }
    }
    (out, cut)
}

fn hex2(b: &[u8]) -> Option<u8> {
    let s = std::str::from_utf8(b.get(..2)?).ok()?;
    u8::from_str_radix(s, 16).ok()
}

/// A line of text: its spaces expanded, its `^` codes as looks, its bytes
/// as CP437.
fn line(raw: &[u8]) -> (Line, bool) {
    let (bytes, cut) = expand(raw);
    let mut out: Line = Vec::new();
    let mut cur = Span::default();
    let flush = |cur: &mut Span, out: &mut Line| {
        if !cur.text.is_empty() {
            let next = Span {
                text: String::new(),
                ..cur.clone()
            };
            out.push(std::mem::replace(cur, next));
        }
    };
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b != b'^' {
            cur.text.push(glyph(b));
            i += 1;
            continue;
        }
        let Some(&code) = bytes.get(i + 1) else {
            // A `^` at the end of the line.
            break;
        };
        match code.to_ascii_lowercase() {
            b'^' => {
                cur.text.push('^');
                i += 2;
            }
            b'b' => {
                flush(&mut cur, &mut out);
                cur.bold = !cur.bold;
                i += 2;
            }
            b'u' => {
                flush(&mut cur, &mut out);
                cur.underline = !cur.underline;
                i += 2;
            }
            b'r' => {
                flush(&mut cur, &mut out);
                cur.reverse = !cur.reverse;
                i += 2;
            }
            b'n' => {
                flush(&mut cur, &mut out);
                cur = Span::default();
                i += 2;
            }
            b'a' => match hex2(&bytes[i + 2..]) {
                Some(attr) => {
                    flush(&mut cur, &mut out);
                    // The same attribute again is a return to normal.
                    cur.attr = (cur.attr != Some(attr)).then_some(attr);
                    i += 4;
                }
                None => {
                    cur.text.push('^');
                    cur.text.push(code as char);
                    i += 2;
                }
            },
            b'c' => match hex2(&bytes[i + 2..]) {
                Some(c) => {
                    cur.text.push(glyph(c));
                    i += 4;
                }
                None => {
                    cur.text.push('^');
                    cur.text.push(code as char);
                    i += 2;
                }
            },
            // Any other letter: the `^` goes, the letter stays.
            _ => i += 1,
        }
    }
    flush(&mut cur, &mut out);
    (out, cut)
}

/// A menu title, a prompt or a see-also: `0xFF` is a plain space, and the
/// text has a limit.
fn prompt(raw: &[u8]) -> (Line, bool) {
    let cleaned: Vec<u8> = raw
        .iter()
        .map(|&b| if b == 0xFF { b' ' } else { b })
        .collect();
    let (mut text, _) = line(&cleaned);
    let mut left = MAX_PROMPT;
    let mut cut = false;
    for span in &mut text {
        let n = span.text.chars().count();
        if n > left {
            span.text = span.text.chars().take(left).collect();
            cut = true;
        }
        left = left.saturating_sub(n);
    }
    text.retain(|s| !s.text.is_empty());
    (text, cut)
}

fn fixed(data: &[u8], at: usize, len: usize) -> String {
    let field = &data[at..at + len];
    let end = field.iter().position(|&b| b == 0).unwrap_or(len);
    field[..end]
        .iter()
        .map(|&b| glyph(b))
        .collect::<String>()
        .trim_end()
        .to_string()
}

/// Reads a whole guide from its bytes.
pub fn parse(data: &[u8], time: Duration) -> Result<Guide, String> {
    let made_with = magic(data).ok_or("Not a Norton Guide")?;
    if data.len() < HEADER {
        return Err("Not a Norton Guide: the header is cut".into());
    }
    let deadline = Instant::now() + time;
    let mut g = Guide {
        made_with,
        title: fixed(data, 8, TITLE),
        credits: (0..5)
            .map(|i| fixed(data, 8 + TITLE + i * CREDIT, CREDIT))
            .collect(),
        ..Guide::default()
    };
    while g.credits.last().is_some_and(String::is_empty) {
        g.credits.pop();
    }
    let menu_count = u16::from_le_bytes([data[6], data[7]]) as usize;

    let mut at = HEADER;
    let mut records = 0;
    while at < data.len() {
        if records == MAX_RECORDS || Instant::now() > deadline {
            g.cut = true;
            g.problem(if records == MAX_RECORDS {
                format!("More than {MAX_RECORDS} records: the rest is not shown")
            } else {
                "The time limit was reached: the rest is not shown".into()
            });
            break;
        }
        let mut head = Record { data, at };
        let (Some(kind), Some(size)) = (head.word(), head.word()) else {
            g.cut = true;
            g.problem("The file ends in the middle of a record".into());
            break;
        };
        let end = at + RECORD + size as usize;
        if end > data.len() {
            g.cut = true;
            g.problem(format!("A record at {at} runs past the end of the file"));
            break;
        }
        let rec = Record {
            data: &data[..end],
            at: at + 4,
        };
        match kind {
            0 | 1 => {
                if let Some(entry) = read_entry(rec, kind == 1, at as u32, &mut g) {
                    g.entries.push(entry);
                }
            }
            2 if g.entries.is_empty() => {
                if g.menus.len() >= menu_count.min(MAX_MENUS) {
                    g.problem("More menus than the header says".into());
                }
                if g.menus.len() < MAX_MENUS
                    && let Some(menu) = read_menu(rec, &mut g)
                {
                    g.menus.push(menu);
                }
            }
            2 => g.problem(format!("A menu among the entries, at {at}")),
            _ => {
                g.problem(format!(
                    "A record of unknown type {kind} at {at}: the rest is not read"
                ));
                g.cut = true;
                break;
            }
        }
        records += 1;
        at = end;
    }
    if g.menus.len() < menu_count && !g.cut {
        g.problem(format!(
            "The header says {menu_count} menus, the file has {}",
            g.menus.len()
        ));
    }
    Ok(g)
}

fn read_menu(mut r: Record, g: &mut Guide) -> Option<Menu> {
    let stored = r.word()? as usize;
    // The stored count seems to count the title as an item.
    let n = stored.saturating_sub(1);
    if stored == 0 {
        g.problem("A menu with no items".into());
    }
    r.skip(20)?;
    // Each offset takes 4 bytes and each item 8 more: a count the record
    // cannot hold is a lie.
    if n.saturating_mul(4) + stored.saturating_mul(8) > r.left() {
        g.problem("A menu says it has more items than it holds".into());
        return None;
    }
    let offsets: Vec<Option<u32>> = (0..n).map(|_| r.offset().flatten()).collect();
    r.skip(stored * 8)?;
    let (title, cut) = prompt(&r.strz(MAX_LINE_BYTES));
    let mut long = cut;
    let prompts = offsets
        .into_iter()
        .map(|to| {
            let (text, cut) = prompt(&r.strz(MAX_LINE_BYTES));
            long |= cut;
            Link { text, to }
        })
        .collect();
    if long {
        g.problem("A menu text longer than 128 characters was cut".into());
    }
    Some(Menu {
        title: plain(&title),
        prompts,
    })
}

fn read_entry(mut r: Record, long: bool, offset: u32, g: &mut Guide) -> Option<Entry> {
    let count = r.word()? as usize;
    let see_also = r.word()?;
    let _parent_line = r.word()?;
    let parent = r.offset()?;
    let _menu = r.word()?;
    let _prompt = r.word()?;
    let previous = r.offset()?;
    let next = r.offset()?;
    let mut cut_line = false;
    let body = if long {
        // At least one byte (its NUL) per line.
        if count > r.left() {
            g.problem(format!(
                "An entry at {offset} says it has more lines than it holds"
            ));
        }
        let mut lines = Vec::new();
        for _ in 0..count.min(r.left()) {
            let (l, cut) = line(&r.strz(MAX_LINE_BYTES));
            cut_line |= cut;
            lines.push(l);
        }
        let mut links = Vec::new();
        if see_also != 0 {
            match r.word() {
                Some(n) if (n as usize) <= MAX_SEE_ALSO => {
                    let tos: Vec<Option<u32>> =
                        (0..n).map_while(|_| r.offset()).collect();
                    for to in tos {
                        let (text, _) = prompt(&r.strz(MAX_LINE_BYTES));
                        links.push(Link { text, to });
                    }
                }
                Some(n) => g.problem(format!(
                    "An entry at {offset} has {n} see-also links (at most {MAX_SEE_ALSO}): they are not read"
                )),
                None => {}
            }
        }
        Body::Long {
            lines,
            see_also: links,
        }
    } else {
        // Six bytes per line before the text.
        if count.saturating_mul(6) > r.left() {
            g.problem(format!(
                "An entry at {offset} says it has more lines than it holds"
            ));
            return None;
        }
        let tos: Vec<Option<u32>> = (0..count)
            .map(|_| {
                let _ = r.word();
                r.offset().flatten()
            })
            .collect();
        let mut links = Vec::new();
        for to in tos {
            let (text, cut) = line(&r.strz(MAX_LINE_BYTES));
            cut_line |= cut;
            links.push(Link { text, to });
        }
        Body::Short(links)
    };
    if cut_line {
        g.problem(format!(
            "A line longer than {MAX_LINE_CHARS} characters was cut"
        ));
    }
    let (previous, next) = if long { (previous, next) } else { (None, None) };
    Some(Entry {
        offset,
        parent,
        previous,
        next,
        body,
    })
}

/// The look of a span in the document: bold, underline, or the two together
/// for reverse and DOS colors (the document has no colors of its own).
fn look(s: &Span) -> Look {
    match (s.bold, s.underline || s.reverse || s.attr.is_some()) {
        (true, true) | (false, true) => Look::Underline,
        (true, false) => Look::Strong,
        _ => Look::Plain,
    }
}

fn styled_line(d: &mut DocEvents, l: &Line) {
    for s in l {
        d.styled(&s.text, look(s));
    }
}

/// Opens a link to entry number `n` (see [`place`]).
fn start_jump(d: &mut DocEvents, n: usize) {
    d.start(pulldown_cmark::Tag::Link {
        link_type: pulldown_cmark::LinkType::Inline,
        dest_url: format!("#{}", place(n)).into(),
        title: "".into(),
        id: "".into(),
    });
}

fn end_jump(d: &mut DocEvents) {
    d.end(pulldown_cmark::TagEnd::Link);
}

/// The name of entry number `n` as a place of the document.
pub fn place(n: usize) -> String {
    format!("e{n}")
}

/// The guide as a document: what it is, its menus, then every entry in file
/// order, numbered (`§12`) so that a link says where it goes. Each link is a
/// jump to its entry (see [`markdown::from_events_linked`]): the viewer
/// follows it.
pub fn render(guide: &Guide, name: &str, time: Duration, budget: usize, width: usize) -> Doc {
    let mut d = DocEvents::new(MAX_EVENTS);
    let number: HashMap<u32, usize> = guide
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| (e.offset, i + 1))
        .collect();
    // A title for each entry: the first menu prompt, line or see-also that
    // leads to it.
    let mut titles: HashMap<u32, String> = HashMap::new();
    for m in &guide.menus {
        for p in &m.prompts {
            if let Some(to) = p.to {
                titles
                    .entry(to)
                    .or_insert_with(|| format!("{} › {}", m.title.trim(), title_of(&p.text)));
            }
        }
    }
    for e in &guide.entries {
        let links: Box<dyn Iterator<Item = &Link>> = match &e.body {
            Body::Short(l) => Box::new(l.iter()),
            Body::Long { see_also, .. } => Box::new(see_also.iter()),
        };
        for l in links {
            if let Some(to) = l.to {
                let t = title_of(&l.text);
                if !t.is_empty() {
                    titles.entry(to).or_insert(t);
                }
            }
        }
    }
    let target = |to: Option<u32>| -> String {
        match to.and_then(|o| number.get(&o)) {
            Some(n) => format!("§{n}"),
            None if to.is_some() => "(broken link)".into(),
            None => String::new(),
        }
    };
    // A line that leads somewhere, with where it leads: the whole is the
    // jump, so that even an empty line can be chosen.
    let linked = |d: &mut DocEvents, text: &Line, to: Option<u32>, sep: &str| {
        let n = to.and_then(|o| number.get(&o)).copied();
        if let Some(n) = n {
            start_jump(d, n);
        }
        styled_line(d, text);
        let t = target(to);
        if !t.is_empty() {
            d.styled(&format!("{sep}{t}"), Look::Emphasis);
        }
        if n.is_some() {
            end_jump(d);
        }
    };

    let title = if guide.title.trim().is_empty() {
        name
    } else {
        guide.title.trim()
    };
    d.heading(1, title);
    d.field("Made with", guide.made_with);
    if !guide.credits.is_empty() {
        d.paragraph(&guide.credits.join("\n"));
    }
    d.field(
        "Contents",
        &format!(
            "{}, {}",
            count(guide.menus.len(), "menu", "menus"),
            count(guide.entries.len(), "entry", "entries")
        ),
    );
    if !guide.problems.is_empty() {
        d.heading(2, "Problems in this guide");
        d.start_list(false);
        for p in &guide.problems {
            d.item(p);
        }
        d.end_list(false);
    }

    for m in &guide.menus {
        d.heading(2, &format!("Menu: {}", m.title.trim()));
        d.start_list(false);
        for p in &m.prompts {
            d.start(pulldown_cmark::Tag::Item);
            linked(&mut d, &p.text, p.to, "  → ");
            d.end(pulldown_cmark::TagEnd::Item);
        }
        d.end_list(false);
    }

    for (i, e) in guide.entries.iter().enumerate() {
        let n = i + 1;
        let heading = match titles.get(&e.offset) {
            Some(t) => format!("§{n} {t}"),
            None => format!("§{n}"),
        };
        d.start(pulldown_cmark::Tag::Heading {
            level: pulldown_cmark::HeadingLevel::H2,
            id: Some(place(n).into()),
            classes: Vec::new(),
            attrs: Vec::new(),
        });
        d.text(&heading);
        d.end(pulldown_cmark::TagEnd::Heading(
            pulldown_cmark::HeadingLevel::H2,
        ));
        match &e.body {
            Body::Short(links) => {
                d.start(pulldown_cmark::Tag::Paragraph);
                for (k, l) in links.iter().enumerate() {
                    if k > 0 {
                        d.push(pulldown_cmark::Event::HardBreak);
                    }
                    linked(&mut d, &l.text, l.to, "  → ");
                }
                d.end(pulldown_cmark::TagEnd::Paragraph);
            }
            Body::Long { lines, see_also } => {
                d.start(pulldown_cmark::Tag::Paragraph);
                for (k, l) in lines.iter().enumerate() {
                    if k > 0 {
                        d.push(pulldown_cmark::Event::HardBreak);
                    }
                    if l.iter().all(|s| s.text.is_empty()) {
                        // An empty line keeps its place.
                        d.text(" ");
                    }
                    styled_line(&mut d, l);
                }
                d.end(pulldown_cmark::TagEnd::Paragraph);
                if !see_also.is_empty() {
                    d.start(pulldown_cmark::Tag::Paragraph);
                    d.styled("See also: ", Look::Strong);
                    for (k, l) in see_also.iter().enumerate() {
                        if k > 0 {
                            d.text(" · ");
                        }
                        linked(&mut d, &l.text, l.to, " ");
                    }
                    d.end(pulldown_cmark::TagEnd::Paragraph);
                }
            }
        }
        let up = e.parent.filter(|p| number.contains_key(p));
        let ways = [("previous", e.previous), ("up", up), ("next", e.next)];
        if ways.iter().any(|w| w.1.is_some()) {
            d.start(pulldown_cmark::Tag::Paragraph);
            let mut first = true;
            for (word, to) in ways.iter().filter(|w| w.1.is_some()) {
                if !first {
                    d.styled(" · ", Look::Emphasis);
                }
                first = false;
                let word = vec![Span {
                    text: (*word).into(),
                    ..Span::default()
                }];
                linked(&mut d, &word, *to, " ");
            }
            d.end(pulldown_cmark::TagEnd::Paragraph);
        }
    }
    if guide.cut {
        d.paragraph("Not all of the guide was read: see the problems at the top.");
    }
    let was_cut = d.cut;
    let mut doc = markdown::from_events_linked(d.events, time, budget);
    if was_cut || guide.cut {
        doc.mark_cut();
    }
    doc.layout(width);
    doc
}

#[cfg(test)]
pub(crate) mod test_guides {
    //! Guides built by the tests, byte by byte.

    use super::*;

    fn enc(out: &mut Vec<u8>, bytes: &[u8]) {
        out.extend(bytes.iter().map(|b| b ^ KEY));
    }

    pub fn header(menus: u16, title: &str) -> Vec<u8> {
        let mut h = vec![0u8; HEADER];
        h[..2].copy_from_slice(b"NG");
        h[6..8].copy_from_slice(&menus.to_le_bytes());
        h[8..8 + title.len()].copy_from_slice(title.as_bytes());
        h[48..48 + 6].copy_from_slice(b"Credit");
        h
    }

    /// A record: its type, then `body` (everything after the 4 first bytes),
    /// with the size set so that the record ends where the body does.
    pub fn record(out: &mut Vec<u8>, kind: u16, body: &[u8]) {
        let size = (body.len() + 4 - RECORD) as u16;
        enc(out, &kind.to_le_bytes());
        enc(out, &size.to_le_bytes());
        enc(out, body);
    }

    pub fn menu(title: &str, prompts: &[(&str, u32)]) -> Vec<u8> {
        let stored = prompts.len() as u16 + 1;
        let mut b = Vec::new();
        b.extend(stored.to_le_bytes());
        b.extend([0u8; 20]);
        for (_, to) in prompts {
            b.extend(to.to_le_bytes());
        }
        b.extend(vec![0u8; stored as usize * 8]);
        b.extend(title.as_bytes());
        b.push(0);
        for (p, _) in prompts {
            b.extend(p.as_bytes());
            b.push(0);
        }
        b.push(0);
        b
    }

    fn fixed(count: u16, see_also: u16, parent: u32, previous: u32, next: u32) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend(count.to_le_bytes());
        b.extend(see_also.to_le_bytes());
        b.extend(0xFFFFu16.to_le_bytes());
        b.extend(parent.to_le_bytes());
        b.extend(0xFFFFu16.to_le_bytes());
        b.extend(0xFFFFu16.to_le_bytes());
        b.extend(previous.to_le_bytes());
        b.extend(next.to_le_bytes());
        b
    }

    pub fn short(lines: &[(&[u8], u32)], parent: u32) -> Vec<u8> {
        let mut b = fixed(lines.len() as u16, 0, parent, u32::MAX, u32::MAX);
        for (_, to) in lines {
            b.extend([0u8; 2]);
            b.extend(to.to_le_bytes());
        }
        for (l, _) in lines {
            b.extend(*l);
            b.push(0);
        }
        b
    }

    pub fn long(lines: &[&[u8]], see_also: &[(&str, u32)], parent: u32) -> Vec<u8> {
        let mut b = fixed(
            lines.len() as u16,
            u16::from(!see_also.is_empty()),
            parent,
            u32::MAX,
            u32::MAX,
        );
        for l in lines {
            b.extend(*l);
            b.push(0);
        }
        if !see_also.is_empty() {
            b.extend((see_also.len() as u16).to_le_bytes());
            for (_, to) in see_also {
                b.extend(to.to_le_bytes());
            }
            for (t, _) in see_also {
                b.extend(t.as_bytes());
                b.push(0);
            }
        }
        b
    }

    /// A small guide: one menu with one prompt to a short entry, whose two
    /// lines lead to two long entries; the first long one has a see-also to
    /// the second.
    pub fn small() -> Vec<u8> {
        // Offsets are known once the sizes are: build twice.
        let build = |short_at: u32, long1: u32, long2: u32| {
            let mut g = header(1, "Small guide");
            record(&mut g, 2, &menu("Main", &[("Functions", short_at)]));
            let s = g.len() as u32;
            record(
                &mut g,
                0,
                &short(&[(b"One()", long1), (b"Two()", long2)], 0),
            );
            let l1 = g.len() as u32;
            record(
                &mut g,
                1,
                &long(
                    &[b"^BOne^B does one thing.", b"", b"\xFF\x04indented"],
                    &[("Two()", long2)],
                    s,
                ),
            );
            let l2 = g.len() as u32;
            record(&mut g, 1, &long(&[b"Two \x10 arrow"], &[], s));
            (g, s, l1, l2)
        };
        let (_, s, l1, l2) = build(0, 0, 0);
        build(s, l1, l2).0
    }
}

#[cfg(test)]
mod tests {
    use super::test_guides::*;
    use super::*;

    const TIME: Duration = Duration::from_secs(5);

    fn text_of(data: &[u8]) -> String {
        let g = parse(data, TIME).unwrap();
        let mut doc = render(&g, "x.ng", TIME, 1 << 24, 100);
        doc.layout(100);
        (0..doc.line_count())
            .map(|i| doc.line(i).to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn a_small_guide_reads_whole() {
        let data = small();
        let g = parse(&data, TIME).unwrap();
        assert!(g.problems.is_empty(), "{:?}", g.problems);
        assert_eq!(g.title, "Small guide");
        assert_eq!(g.credits, ["Credit"]);
        assert_eq!(g.menus.len(), 1);
        assert_eq!(g.menus[0].title, "Main");
        assert_eq!(plain(&g.menus[0].prompts[0].text), "Functions");
        assert_eq!(g.menus[0].prompts[0].to, Some(g.entries[0].offset));
        assert_eq!(g.entries.len(), 3);
        let Body::Short(links) = &g.entries[0].body else {
            panic!("short")
        };
        assert_eq!(links[1].to, Some(g.entries[2].offset));
        let Body::Long { lines, see_also } = &g.entries[1].body else {
            panic!("long")
        };
        assert_eq!(
            lines[0][0],
            Span {
                text: "One".into(),
                bold: true,
                ..Span::default()
            }
        );
        assert_eq!(plain(&lines[2]), "    indented");
        assert_eq!(see_also[0].to, Some(g.entries[2].offset));
        assert_eq!(g.entries[1].parent, Some(g.entries[0].offset));
        let t = text_of(&data);
        assert!(t.contains("Small guide"), "{t}");
        assert!(t.contains("§1 Main › Functions"), "{t}");
        assert!(t.contains("§2 One()"), "{t}");
        assert!(t.contains("Contents: 1 menu, 3 entries"), "{t}");
        assert!(t.contains("Two ► arrow"), "{t}");
        assert!(t.contains("See also: Two() §3"), "{t}");
    }

    #[test]
    fn every_link_is_a_jump_to_its_entry() {
        let g = parse(&small(), TIME).unwrap();
        let doc = render(&g, "x.ng", TIME, 1 << 24, 100);
        let jumps: Vec<(String, &str)> = doc
            .jumps()
            .iter()
            .map(|j| {
                let text: String = doc
                    .line(j.line)
                    .chars()
                    .skip(j.start)
                    .take(j.end - j.start)
                    .collect();
                (text, &*j.to)
            })
            .collect();
        // The menu, the two lines of the short entry, the see-also, and the
        // "up" of the two long entries.
        assert_eq!(
            jumps,
            [
                ("Functions  → §1".to_string(), "e1"),
                ("One()  → §2".to_string(), "e2"),
                ("Two()  → §3".to_string(), "e3"),
                ("Two() §3".to_string(), "e3"),
                ("up §1".to_string(), "e1"),
                ("up §1".to_string(), "e1"),
            ]
        );
        for n in 1..=3 {
            let line = doc.place(&place(n)).expect("each entry is a place");
            assert!(doc.line(line).starts_with(&format!("§{n}")), "{n}");
        }
        // A link to nowhere is no jump.
        let mut g = g;
        if let Body::Short(links) = &mut g.entries[0].body {
            links[0].to = Some(12345);
        }
        let doc = render(&g, "x.ng", TIME, 1 << 24, 100);
        assert_eq!(doc.jumps().len(), 5);
        assert!(doc.jumps().iter().all(|j| doc.place(&j.to).is_some()));
    }

    #[test]
    fn spaces_expand_without_sign_and_with_a_limit() {
        assert_eq!(expand(b"a\xFF\x03b").0, b"a   b");
        assert_eq!(expand(b"\xFF\x00x").0, b"x");
        assert_eq!(expand(b"a\xFF\xFFb").0, b"a b");
        assert_eq!(expand(b"a\xFF").0, b"a ");
        assert_eq!(expand(b"\xFF\xC8").0.len(), 200);
        // A bomb: ten bytes do not become a gigabyte.
        let bomb = b"\xFF\xFE".repeat(1000);
        let (out, cut) = expand(&bomb);
        assert_eq!(out.len(), MAX_LINE_CHARS);
        assert!(cut);
    }

    #[test]
    fn codes_become_looks_and_nothing_else() {
        let (l, _) = line(b"^Bbold^B ^Uunder^U ^Rrev^R");
        assert!(l[0].bold && l[0].text == "bold");
        assert!(l[2].underline && l[2].text == "under");
        assert!(l[4].reverse && l[4].text == "rev");
        // Bold and underline add up; ^N turns everything off.
        let (l, _) = line(b"^B^Uboth^Nplain");
        assert!(l[0].bold && l[0].underline);
        assert_eq!(
            l[1],
            Span {
                text: "plain".into(),
                ..Span::default()
            }
        );
        // ^A: a color, and the same one again goes back to normal.
        let (l, _) = line(b"^A1Fc^A1Fn");
        assert_eq!(l[0].attr, Some(0x1F));
        assert_eq!(l[1].attr, None);
        // ^C: a character by its code, through CP437.
        assert_eq!(plain(&line(b"^C41^CB3^C01").0), "A│☺");
        // Without hex they are text; ^^ is a caret; another letter loses
        // the caret; a caret at the end goes.
        assert_eq!(plain(&line(b"^Azz ^Cq").0), "^Azz ^Cq");
        assert_eq!(plain(&line(b"2^^3").0), "2^3");
        assert_eq!(plain(&line(b"^xyz").0), "xyz");
        assert_eq!(plain(&line(b"end^").0), "end");
    }

    #[test]
    fn low_bytes_are_glyphs_never_controls() {
        let raw: Vec<u8> = (1u8..32).chain([127, 0xB3, 0xC4]).collect();
        let text = plain(&line(&raw).0);
        assert!(text.chars().all(|c| !c.is_control()), "{text:?}");
        assert!(text.starts_with("☺☻♥"));
        assert!(text.ends_with("⌂│─"));
        // An ESC byte is a ←, never an escape sequence.
        assert_eq!(plain(&line(b"\x1b[2J").0), "←[2J");
    }

    #[test]
    fn hostile_guides_never_fail_badly() {
        let good = small();
        // Every prefix of the file: nothing panics, and some say why.
        for n in [
            0,
            1,
            2,
            100,
            HEADER,
            HEADER + 3,
            HEADER + 30,
            good.len() - 5,
        ] {
            let _ = parse(&good[..n], TIME);
        }
        let cut = parse(&good[..good.len() - 5], TIME).unwrap();
        assert!(cut.cut);
        assert!(!cut.problems.is_empty());

        // Offsets that point outside: broken links, not crashes.
        let mut g = header(1, "Bad");
        record(
            &mut g,
            2,
            &menu("M", &[("Far", 0x7FFF_FFFF), ("None", u32::MAX)]),
        );
        record(&mut g, 0, &short(&[(b"x", 12345)], 0));
        let t = text_of(&g);
        assert!(t.contains("(broken link)"), "{t}");

        // A menu that says it has more items than it holds.
        let mut g = header(1, "Bad");
        let mut m = menu("M", &[("a", 1)]);
        m[0..2].copy_from_slice(&5000u16.to_le_bytes());
        record(&mut g, 2, &m);
        let p = parse(&g, TIME).unwrap();
        assert!(p.menus.is_empty());
        assert!(
            p.problems.iter().any(|p| p.contains("more items")),
            "{:?}",
            p.problems
        );

        // A menu with a stored count of zero: no items, no huge number.
        let mut g = header(1, "Zero");
        let mut m = menu("M", &[]);
        m[0..2].copy_from_slice(&0u16.to_le_bytes());
        record(&mut g, 2, &m);
        let p = parse(&g, TIME).unwrap();
        assert!(
            p.problems.iter().any(|p| p.contains("no items")),
            "{:?}",
            p.problems
        );

        // A short entry whose line count lies.
        let mut g = header(0, "Lies");
        let mut s = short(&[(b"a", 1)], 0);
        s[0..2].copy_from_slice(&60000u16.to_le_bytes());
        record(&mut g, 0, &s);
        let p = parse(&g, TIME).unwrap();
        assert!(p.entries.is_empty());
        assert!(!p.problems.is_empty());

        // A long entry whose line count lies: what it holds is shown.
        let mut g = header(0, "Lies");
        let mut l = long(&[b"real"], &[], 0);
        l[0..2].copy_from_slice(&60000u16.to_le_bytes());
        record(&mut g, 1, &l);
        let p = parse(&g, TIME).unwrap();
        assert_eq!(p.entries.len(), 1);

        // More see-also links than the compiler allows: not read.
        let mut g = header(0, "SA");
        let many: Vec<(&str, u32)> = (0..25).map(|_| ("x", 1)).collect();
        record(&mut g, 1, &long(&[b"text"], &many, 0));
        let p = parse(&g, TIME).unwrap();
        let Body::Long { lines, see_also } = &p.entries[0].body else {
            panic!()
        };
        assert_eq!(plain(&lines[0]), "text");
        assert!(see_also.is_empty());
        assert!(p.problems.iter().any(|p| p.contains("see-also")));

        // A record of an unknown type stops the reading, and says so.
        let mut g = header(0, "Odd");
        record(&mut g, 9, &[0u8; 22]);
        let p = parse(&g, TIME).unwrap();
        assert!(p.cut);

        // A line with no NUL ends at its record; a line over the limit is
        // cut and the next one still reads.
        let mut g = header(0, "Long line");
        let big = vec![b'x'; 3000];
        record(&mut g, 1, &long(&[&big, b"after"], &[], 0));
        let p = parse(&g, TIME).unwrap();
        let Body::Long { lines, .. } = &p.entries[0].body else {
            panic!()
        };
        assert_eq!(plain(&lines[0]).len(), MAX_LINE_BYTES);
        assert_eq!(plain(&lines[1]), "after");
    }

    #[test]
    fn a_guide_is_known_by_its_name_and_its_signature() {
        let data = small();
        assert!(is_guide(Path::new("a.NG"), &data));
        assert!(!is_guide(Path::new("a.txt"), &data));
        assert!(!is_guide(Path::new("a.ng"), b"PK\x03\x04"));
        let mut eh = data.clone();
        eh[..2].copy_from_slice(b"EH");
        assert_eq!(parse(&eh, TIME).unwrap().made_with, "Expert Help");
    }

    /// One real guide as the viewer lays it out at 80 columns
    /// (`XY_NG_SHOW=/path/to/x.ng cargo test show_guide -- --ignored --nocapture`).
    #[test]
    #[ignore]
    fn show_guide() {
        let Ok(path) = std::env::var("XY_NG_SHOW") else {
            return;
        };
        let data = std::fs::read(&path).unwrap();
        let g = parse(&data, TIME).unwrap();
        let mut doc = render(&g, &path, TIME, 1 << 26, 80);
        doc.layout(80);
        let max: usize = std::env::var("XY_NG_LINES")
            .ok()
            .and_then(|n| n.parse().ok())
            .unwrap_or(120);
        for i in 0..doc.line_count().min(max) {
            println!("{}", doc.line(i));
        }
    }

    /// The real guides of the public collection, when they are there
    /// (`XY_NG_DIR=/path/to/guides cargo test real_guides -- --ignored`).
    #[test]
    #[ignore]
    fn real_guides() {
        let Ok(dir) = std::env::var("XY_NG_DIR") else {
            return;
        };
        let mut seen = 0;
        for f in std::fs::read_dir(dir).unwrap() {
            let path = f.unwrap().path();
            let data = std::fs::read(&path).unwrap();
            if !is_guide(&path, &data) {
                continue;
            }
            seen += 1;
            let g = parse(&data, Duration::from_secs(30)).unwrap();
            let offsets: std::collections::HashSet<u32> =
                g.entries.iter().map(|e| e.offset).collect();
            let mut broken = 0;
            let mut links = 0;
            for m in &g.menus {
                for p in &m.prompts {
                    links += 1;
                    broken += usize::from(p.to.is_some_and(|t| !offsets.contains(&t)));
                }
            }
            for e in &g.entries {
                let ls: Vec<&Link> = match &e.body {
                    Body::Short(l) => l.iter().collect(),
                    Body::Long { see_also, .. } => see_also.iter().collect(),
                };
                for l in ls {
                    links += 1;
                    broken += usize::from(l.to.is_some_and(|t| !offsets.contains(&t)));
                }
            }
            // Every link that leads to an entry is a jump that the viewer
            // can follow, and each jump lands on its entry's heading.
            let doc = render(&g, "x.ng", Duration::from_secs(60), usize::MAX, 100);
            assert!(!doc.cut(), "{path:?} cut");
            let valid = g
                .menus
                .iter()
                .flat_map(|m| m.prompts.iter())
                .chain(g.entries.iter().flat_map(|e| match &e.body {
                    Body::Short(l) => l.iter(),
                    Body::Long { see_also, .. } => see_also.iter(),
                }))
                .filter(|l| l.to.is_some_and(|t| offsets.contains(&t)))
                .count();
            assert!(
                doc.jumps().len() >= valid,
                "{path:?} jumps lost: {} of {valid}",
                doc.jumps().len()
            );
            for j in doc.jumps() {
                let line = doc.place(&j.to).expect("a jump to a place");
                assert!(doc.line(line).starts_with('§'), "{path:?} {}", j.to);
            }
            println!(
                "{:<16} {:>2} menus {:>6} entries {:>6} links {:>4} broken {:?}",
                path.file_name().unwrap().to_string_lossy(),
                g.menus.len(),
                g.entries.len(),
                links,
                broken,
                g.problems
            );
        }
        assert!(seen > 0);
    }
}
