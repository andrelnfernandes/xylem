//! A book read in the viewer (docs/plans/viewer-ebooks.md, 1b): the two
//! views of an [`epub::Book`] (the book to read and the report on the file),
//! the settings of the reading layout, the glyphs with and without Nerd
//! Fonts, and how far the reading went.

use std::collections::HashMap;

use super::epub::{Book, Chapter, TocEntry};
use super::markdown::{BookStyle, Doc};
use super::table::SheetList;

/// The reader's settings, from F12 (page "Reading").
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReaderOptions {
    /// The width of the text column, in cells; 0 for the whole window.
    pub width: u16,
    /// Paragraphs parted by a blank row, not indented.
    pub blocks: bool,
    pub justify: bool,
    /// Words per minute, for the time left.
    pub wpm: u16,
    /// The terminal font has the Nerd Fonts glyphs.
    pub nerd: bool,
}

impl Default for ReaderOptions {
    fn default() -> Self {
        ReaderOptions {
            width: 70,
            blocks: false,
            justify: false,
            wpm: 250,
            nerd: false,
        }
    }
}

impl ReaderOptions {
    pub fn style(&self) -> BookStyle {
        let g = glyphs(self.nerd);
        BookStyle {
            indent: 3,
            blocks: self.blocks,
            justify: self.justify,
            ornament: g.chapter.into(),
            scene: g.scene.into(),
        }
    }

    /// The width the text takes in a window `room` cells wide.
    pub fn column(&self, room: usize) -> usize {
        match usize::from(self.width) {
            0 => room,
            w => w.min(room),
        }
    }
}

/// The glyphs of the reader: with Nerd Fonts, or those every font has. Each
/// is one cell wide (a test checks it), so nothing moves between the two.
#[derive(Debug, Clone, Copy)]
pub struct Glyphs {
    /// Under the heading of a chapter.
    pub chapter: &'static str,
    /// A break between scenes.
    pub scene: &'static str,
    /// The progress bar: done and to do; with Nerd Fonts, its two ends too.
    pub bar_done: &'static str,
    pub bar_left: &'static str,
    pub bar_ends: Option<[(&'static str, &'static str); 2]>,
    /// Before the chapter and before the time, in the footer.
    pub book: &'static str,
    pub clock: &'static str,
    /// In the contents: the chapter being read, and those before it.
    pub current: &'static str,
    pub read: &'static str,
}

/// Each glyph is named as in Nerd Fonts' `glyphnames.json` (v3).
pub fn glyphs(nerd: bool) -> Glyphs {
    if nerd {
        Glyphs {
            chapter: "────  \u{f06c}  ────",           // fa-leaf
            scene: "\u{f06c}    \u{f06c}    \u{f06c}", // fa-leaf
            // The progress bar of Fira Code: middle (empty, full), and the
            // left and right ends (empty, full).
            bar_done: "\u{ee04}",
            bar_left: "\u{ee01}",
            bar_ends: Some([("\u{ee00}", "\u{ee03}"), ("\u{ee02}", "\u{ee05}")]),
            book: "\u{f02d} ",   // fa-book
            clock: "\u{f017} ",  // fa-clock_o
            current: "\u{f02e}", // fa-bookmark
            read: "\u{f00c}",    // fa-check
        }
    } else {
        Glyphs {
            chapter: "────  *  ────",
            scene: "*    *    *",
            bar_done: "━",
            bar_left: "─",
            bar_ends: None,
            book: "",
            clock: "",
            current: "▸",
            read: "✓",
        }
    }
}

/// A bar of `cells` cells, `done` of the way: `━━━━────`.
pub fn bar(done: f64, cells: usize, g: &Glyphs) -> String {
    let full = ((done.clamp(0.0, 1.0) * cells as f64).round() as usize).min(cells);
    let mut out = String::new();
    for i in 0..cells {
        let filled = i < full;
        let glyph = match g.bar_ends {
            Some([left, _]) if i == 0 => {
                if filled {
                    left.1
                } else {
                    left.0
                }
            }
            Some([_, right]) if i + 1 == cells => {
                if filled {
                    right.1
                } else {
                    right.0
                }
            }
            _ if filled => g.bar_done,
            _ => g.bar_left,
        };
        out.push_str(glyph);
    }
    out
}

/// How long `words` take at `wpm` words a minute: `12 min`, `4 h 10 min`.
pub fn time_left(words: usize, wpm: u16) -> String {
    let minutes = (words as f64 / f64::from(wpm.max(1))).ceil() as usize;
    match minutes {
        0 => "nothing".into(),
        1 => "1 min".into(),
        m if m < 60 => format!("{m} min"),
        m if m % 60 == 0 => format!("{} h", m / 60),
        m => format!("{} h {} min", m / 60, m % 60),
    }
}

/// A book in the viewer: the view not on the screen, what the reader needs
/// besides the documents, and what is open over the text.
pub struct Reader {
    /// The book to read is on the screen (else the report on the file).
    pub reading: bool,
    /// The view that is not on the screen.
    pub other: Option<Doc>,
    pub chapters: Vec<Chapter>,
    pub toc: Vec<TocEntry>,
    /// The text of each note, by the place its call leads to.
    pub notes: HashMap<Box<str>, String>,
    /// Tab: the contents over the text.
    pub contents: Option<SheetList>,
    /// Enter: the notes called on the screen, and how far they are scrolled.
    pub notes_view: Option<(Vec<(String, String)>, usize)>,
    /// Z: only the text, and a line of progress.
    pub focus: bool,
}

impl Reader {
    /// The book, its book to read on the screen: what is kept here, and the
    /// document that goes on the screen.
    pub fn new(book: Book) -> (Reader, Doc) {
        let reader = Reader {
            reading: true,
            other: Some(book.report),
            chapters: book.chapters,
            toc: book.toc,
            notes: book.notes,
            contents: None,
            notes_view: None,
            focus: false,
        };
        (reader, book.reading)
    }

    /// The names of the contents, indented by level, for the list.
    pub fn toc_names(&self) -> Vec<String> {
        let entries: Vec<(usize, &str)> = if self.toc.is_empty() {
            self.chapters
                .iter()
                .map(|c| (1, c.title.as_str()))
                .collect()
        } else {
            self.toc
                .iter()
                .map(|e| (e.level, e.title.as_str()))
                .collect()
        };
        entries
            .into_iter()
            .map(|(level, title)| {
                let title = if title.is_empty() {
                    "(no title)"
                } else {
                    title
                };
                format!("{}{title}", "  ".repeat(level.saturating_sub(1).min(8)))
            })
            .collect()
    }

    /// The place each entry of the contents leads to.
    pub fn toc_places(&self) -> Vec<Option<Box<str>>> {
        if self.toc.is_empty() {
            self.chapters
                .iter()
                .map(|c| Some(c.place.clone()))
                .collect()
        } else {
            self.toc.iter().map(|e| e.place.clone()).collect()
        }
    }

    pub fn total_words(&self) -> usize {
        self.chapters.iter().map(|c| c.words).sum()
    }
}

/// How far the reading went, at the top of the screen.
#[derive(Debug, Clone, PartialEq)]
pub struct Progress {
    /// The chapter (from 0), and how many there are.
    pub chapter: usize,
    pub chapters: usize,
    pub title: String,
    /// Of the whole book, from 0 to 1.
    pub done: f64,
    pub words_left_in_chapter: usize,
    pub words_left: usize,
}

impl Progress {
    /// From the rows where the chapters start, their words, and the rows
    /// on the screen (`top` and how many rows the document has).
    pub fn at(chapters: &[Chapter], starts: &[usize], top: usize, rows: usize) -> Option<Self> {
        let i = starts.iter().rposition(|&r| r <= top)?;
        let start = starts[i];
        let end = starts.get(i + 1).copied().unwrap_or(rows).max(start + 1);
        let within = ((top - start) as f64 / (end - start) as f64).clamp(0.0, 1.0);
        let words = chapters.get(i).map_or(0, |c| c.words);
        let before: usize = chapters[..i.min(chapters.len())]
            .iter()
            .map(|c| c.words)
            .sum();
        let total: usize = chapters.iter().map(|c| c.words).sum();
        let read = before as f64 + within * words as f64;
        let left_in_chapter = ((1.0 - within) * words as f64).round() as usize;
        let after: usize = chapters.iter().skip(i + 1).map(|c| c.words).sum();
        Some(Progress {
            chapter: i,
            chapters: chapters.len(),
            title: chapters.get(i).map_or(String::new(), |c| c.title.clone()),
            done: if total == 0 { 0.0 } else { read / total as f64 },
            words_left_in_chapter: left_in_chapter,
            words_left: left_in_chapter + after,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::text::Span;

    fn chapter(place: &str, words: usize) -> Chapter {
        Chapter {
            place: place.into(),
            title: place.to_string(),
            words,
        }
    }

    #[test]
    fn the_glyphs_take_the_same_cells_with_or_without_nerd_fonts() {
        let (plain, nerd) = (glyphs(false), glyphs(true));
        let w = |s: &str| Span::raw(s).width();
        assert_eq!(w(plain.chapter), w(nerd.chapter));
        assert_eq!(w(plain.scene), w(nerd.scene));
        for g in [plain, nerd] {
            assert_eq!(w(g.bar_done), 1);
            assert_eq!(w(g.bar_left), 1);
            assert_eq!(w(g.current), 1);
            assert_eq!(w(g.read), 1);
            assert_eq!(w(&bar(0.5, 10, &g)), 10);
        }
        assert_eq!(w(nerd.book), 2);
        assert_eq!(w(nerd.clock), 2);
    }

    #[test]
    fn a_bar_fills_as_far_as_the_reading_went() {
        let g = glyphs(false);
        assert_eq!(bar(0.0, 4, &g), "────");
        assert_eq!(bar(0.5, 4, &g), "━━──");
        assert_eq!(bar(1.0, 4, &g), "━━━━");
        let n = glyphs(true);
        assert_eq!(bar(0.5, 4, &n), "\u{ee03}\u{ee04}\u{ee01}\u{ee02}");
    }

    #[test]
    fn the_time_left_is_said_in_minutes_and_hours() {
        assert_eq!(time_left(0, 250), "nothing");
        assert_eq!(time_left(100, 250), "1 min");
        assert_eq!(time_left(3_000, 250), "12 min");
        assert_eq!(time_left(15_000, 250), "1 h");
        assert_eq!(time_left(62_500, 250), "4 h 10 min");
    }

    #[test]
    fn progress_is_counted_in_words() {
        let chapters = [chapter("c1", 100), chapter("c2", 300)];
        // Chapter 2 runs from row 10 to row 30; the top at row 20 is half.
        let p = Progress::at(&chapters, &[0, 10], 20, 30).unwrap();
        assert_eq!(p.chapter, 1);
        assert_eq!(p.title, "c2");
        assert_eq!(p.words_left_in_chapter, 150);
        assert_eq!(p.words_left, 150);
        assert!((p.done - 250.0 / 400.0).abs() < 1e-9);
        assert_eq!(
            Progress::at(&chapters, &[5, 10], 2, 30),
            None,
            "before the first"
        );
    }

    #[test]
    fn the_column_is_at_most_the_window() {
        let o = ReaderOptions::default();
        assert_eq!(o.column(100), 70);
        assert_eq!(o.column(50), 50);
        let whole = ReaderOptions { width: 0, ..o };
        assert_eq!(whole.column(100), 100);
    }
}
