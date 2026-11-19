//! The events of a document Xylem makes itself (a notebook, an Office
//! document, a report on a binary or a certificate), for
//! [`markdown::from_events`](super::markdown::from_events).
//!
//! Text goes in as `Text` events, never as Markdown: a `**` or a `[a](b)` in
//! a hostile file stays as it is. The viewer cleans it like any text (control
//! characters as `·`, bidi controls marked) when the document is built.

use std::ops::Range;

use pulldown_cmark::{Alignment, CodeBlockKind, CowStr, Event, HeadingLevel, Tag, TagEnd};

/// Events, with the place in the source each one stands for.
pub struct DocEvents {
    pub events: Vec<(Event<'static>, Range<usize>)>,
    /// The offset in the source the next events stand for.
    at: usize,
    /// Most events kept: past them the document is cut, and says so.
    max: usize,
    pub cut: bool,
}

impl DocEvents {
    pub fn new(max: usize) -> Self {
        DocEvents {
            events: Vec::new(),
            at: 0,
            max,
            cut: false,
        }
    }

    pub fn push(&mut self, event: Event<'static>) {
        if self.events.len() >= self.max {
            self.cut = true;
            return;
        }
        self.events.push((event, self.at..self.at));
    }

    /// Events of another piece (Markdown of a notebook cell): their ranges
    /// all stand for the current offset.
    pub fn extend<'a>(&mut self, events: impl IntoIterator<Item = (Event<'a>, Range<usize>)>) {
        for (e, _) in events {
            if self.cut {
                return;
            }
            self.push(e.into_static());
        }
    }

    pub fn start(&mut self, tag: Tag<'static>) {
        self.push(Event::Start(tag));
    }

    pub fn end(&mut self, tag: TagEnd) {
        self.push(Event::End(tag));
    }

    /// Text of a paragraph: its line ends are line breaks.
    pub fn text(&mut self, s: &str) {
        for (i, line) in s.split('\n').enumerate() {
            if i > 0 {
                self.push(Event::HardBreak);
            }
            if !line.is_empty() {
                self.push(Event::Text(CowStr::from(line.to_string())));
            }
        }
    }

    /// Text with a look of its own.
    pub fn styled(&mut self, s: &str, look: Look) {
        let tags: &[Tag<'static>] = match look {
            Look::Plain => &[],
            Look::Strong => &[Tag::Strong],
            Look::Emphasis => &[Tag::Emphasis],
            Look::Strike => &[Tag::Strikethrough],
            Look::Underline => &[Tag::Strong, Tag::Emphasis],
        };
        for t in tags {
            self.start(t.clone());
        }
        self.text(s);
        for t in tags.iter().rev() {
            self.end(t.to_end());
        }
    }

    /// A piece of code, inline.
    pub fn code(&mut self, s: &str) {
        self.push(Event::Code(CowStr::from(s.replace('\n', " "))));
    }

    pub fn heading(&mut self, level: u8, s: &str) {
        self.start_heading(level);
        self.text(s);
        self.end_heading(level);
    }

    pub fn start_heading(&mut self, level: u8) {
        self.start(Tag::Heading {
            level: heading_level(level),
            id: None,
            classes: Vec::new(),
            attrs: Vec::new(),
        });
    }

    pub fn end_heading(&mut self, level: u8) {
        self.end(TagEnd::Heading(heading_level(level)));
    }

    pub fn paragraph(&mut self, s: &str) {
        self.start(Tag::Paragraph);
        self.text(s);
        self.end(TagEnd::Paragraph);
    }

    /// A paragraph of a label in bold and its value: `Kernel: Python 3`.
    pub fn field(&mut self, label: &str, value: &str) {
        self.start(Tag::Paragraph);
        self.styled(&format!("{label}: "), Look::Strong);
        self.text(value);
        self.end(TagEnd::Paragraph);
    }

    /// A block of code, colored as `language` (empty: no colors).
    pub fn code_block(&mut self, language: &str, s: &str) {
        let kind = if language.is_empty() {
            CodeBlockKind::Indented
        } else {
            CodeBlockKind::Fenced(CowStr::from(language.to_string()))
        };
        self.start(Tag::CodeBlock(kind));
        let mut text = s.to_string();
        if !text.ends_with('\n') {
            text.push('\n');
        }
        self.push(Event::Text(CowStr::from(text)));
        self.end(TagEnd::CodeBlock);
    }

    pub fn rule(&mut self) {
        self.push(Event::Rule);
    }

    pub fn start_list(&mut self, numbered: bool) {
        self.start(Tag::List(numbered.then_some(1)));
    }

    pub fn end_list(&mut self, numbered: bool) {
        self.end(TagEnd::List(numbered));
    }

    /// A list item of plain text.
    pub fn item(&mut self, s: &str) {
        self.start(Tag::Item);
        self.text(s);
        self.end(TagEnd::Item);
    }

    /// A table: its header and rows, of plain text.
    pub fn table(&mut self, header: &[String], rows: &[Vec<String>]) {
        let columns = header
            .len()
            .max(rows.iter().map(Vec::len).max().unwrap_or(0))
            .max(1);
        self.start(Tag::Table(vec![Alignment::None; columns]));
        self.start(Tag::TableHead);
        for c in 0..columns {
            self.cell(header.get(c).map_or("", String::as_str));
        }
        self.end(TagEnd::TableHead);
        for row in rows {
            self.start(Tag::TableRow);
            for c in 0..columns {
                self.cell(row.get(c).map_or("", String::as_str));
            }
            self.end(TagEnd::TableRow);
        }
        self.end(TagEnd::Table);
    }

    fn cell(&mut self, s: &str) {
        self.start(Tag::TableCell);
        if !s.is_empty() {
            self.push(Event::Text(CowStr::from(s.replace('\n', " "))));
        }
        self.end(TagEnd::TableCell);
    }
}

fn heading_level(level: u8) -> HeadingLevel {
    match level {
        0 | 1 => HeadingLevel::H1,
        2 => HeadingLevel::H2,
        3 => HeadingLevel::H3,
        4 => HeadingLevel::H4,
        5 => HeadingLevel::H5,
        _ => HeadingLevel::H6,
    }
}

/// How a piece of text looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Look {
    Plain,
    Strong,
    Emphasis,
    Strike,
    /// Underline is not in Markdown: bold and italic stand for it.
    Underline,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn rendered(d: DocEvents) -> Vec<String> {
        let mut doc =
            super::super::markdown::from_events("", d.events, Duration::from_secs(3), 1 << 20);
        doc.layout(60);
        (0..doc.line_count())
            .map(|i| doc.line(i).to_string())
            .collect()
    }

    #[test]
    fn text_is_never_read_as_markdown() {
        let mut d = DocEvents::new(100);
        d.heading(1, "Title **not bold**");
        d.paragraph("[a](http://x) and `code`\nnext line");
        d.table(&["k".into(), "v".into()], &[vec!["*1*".into(), "2".into()]]);
        d.code_block("rust", "fn x() {}");
        let lines = rendered(d).join("\n");
        assert!(lines.contains("Title **not bold**"), "{lines}");
        assert!(lines.contains("[a](http://x) and `code`"), "{lines}");
        assert!(lines.contains("next line"));
        assert!(lines.contains("*1*"));
        assert!(lines.contains("fn x() {}"));
    }

    #[test]
    fn the_events_have_a_limit() {
        let mut d = DocEvents::new(5);
        for _ in 0..10 {
            d.paragraph("x");
        }
        assert!(d.cut);
        assert_eq!(d.events.len(), 5);
    }
}
