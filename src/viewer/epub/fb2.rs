//! FictionBook 2 (`.fb2`, and `.fb2.zip` with one inside) read as a book,
//! by the same road as an EPUB: its elements are translated into the XHTML
//! ones the [`Writer`] knows (`section`/`title` into headings, `emphasis`
//! into `em`, `poem` into a quote with a verse per line), so the title page,
//! the report, the contents, the notes and the reading kept all come from
//! there.
//!
//! The file is hostile as an EPUB is: one XML, read with no DTD and within
//! the limits of a part; the images embedded (`binary`) are never decoded,
//! only measured; an encoding it declares and this viewer does not know is
//! said, and the text read as UTF-8.

use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Instant;

use super::Ev;
use super::{
    Book, Chapter, Entry, Findings, Front, Limits, MAX_CHAPTERS, MAX_TEXT, Opf, Package, Writer,
    Xml, assemble, attr, collapse,
};
use crate::charset::{self, Charset};

/// The name every link of the book is resolved in: an FB2 is one part.
const PART: &str = "book";
/// The most of the annotation kept for the report.
const MAX_ANNOTATION: usize = 4000;

/// Whether the file is named as a FictionBook.
pub fn is_fb2(path: &std::path::Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    name.ends_with(".fb2") || name.ends_with(".fb2.zip")
}

/// The book in `bytes` (an FB2, or a ZIP with only one), laid out for
/// `width`, or why it is not one; `reading` as in [`super::render_with`].
pub fn render_with(
    bytes: &[u8],
    limits: Limits,
    width: usize,
    reading: &dyn Fn(Option<&str>) -> crate::reading::Summary,
) -> Result<Book, String> {
    let deadline = Instant::now() + limits.time;
    let zipped = bytes.starts_with(b"PK\x03\x04");
    let (raw, cut): (Cow<[u8]>, bool) = if zipped {
        let mut pkg = Package::open(std::io::Cursor::new(bytes), limits)
            .map_err(|_| "Not a ZIP this viewer can read: showing the bytes".to_string())?;
        let files: Vec<&String> = pkg.names.iter().filter(|n| !n.ends_with('/')).collect();
        let [name] = files.as_slice() else {
            return Err("Not one FictionBook alone in the ZIP: showing it as a ZIP".into());
        };
        if !name.to_ascii_lowercase().ends_with(".fb2") {
            return Err("No FictionBook in the ZIP: showing it as a ZIP".into());
        }
        let name = (*name).clone();
        let data = pkg
            .bytes(&name)
            .ok_or("The FictionBook in the ZIP cannot be read: showing it as a ZIP")?;
        (Cow::Owned(data), pkg.cut)
    } else {
        let keep = bytes
            .len()
            .min(usize::try_from(limits.part).unwrap_or(usize::MAX));
        (Cow::Borrowed(&bytes[..keep]), keep < bytes.len())
    };

    let mut found = Findings::default();
    let text = decode(&raw, &mut found).into_owned();
    drop(raw);
    let start = text.trim_start_matches('\u{FEFF}').trim_start();
    let root = start
        .strip_prefix("<?xml")
        .map_or(start, |rest| rest.split_once("?>").map_or("", |(_, r)| r))
        .trim_start();
    if !root.starts_with('<') || !text.contains("FictionBook") {
        return Err("Not a FictionBook: showing the text".into());
    }

    let scan = Scan::read(&text, deadline);
    found.binaries = scan.binaries.len();
    found.binary_bytes = scan.binaries.values().map(|(_, b)| b).sum();
    for (kind, _) in scan.binaries.values() {
        if !found.binary_types.contains(kind) && found.binary_types.len() < 10 {
            found.binary_types.push(kind.clone());
        }
    }
    let mut opf = Opf {
        version: String::new(),
        meta: scan.meta.clone(),
        manifest: HashMap::new(),
        order: Vec::new(),
        spine: Vec::new(),
        ncx: None,
    };
    opf.meta.push((
        "Format".into(),
        if zipped {
            "FictionBook 2, in a ZIP".into()
        } else {
            "FictionBook 2".into()
        },
    ));
    if let Some(a) = &scan.annotation {
        opf.meta.push(("Annotation".into(), a.clone()));
    }
    let book_title = super::meta_of(&opf, &["Title"]).unwrap_or_default();

    let mut t = Translator::new(&scan, book_title);
    let late = t.run(&text, deadline);
    let Translator {
        w, chapters, toc, ..
    } = t;
    found.merge(&w.found);
    let cut = cut || late || scan.late || w.d.cut;
    Ok(assemble(
        Front {
            opf,
            found,
            encrypted: 0,
            toc,
            w,
            chapters,
            cut,
        },
        limits,
        width,
        reading,
    ))
}

/// The text of the file, in the encoding it declares: a byte order mark,
/// else the XML prolog (`encoding="windows-1251"`), else UTF-8.
fn decode<'a>(raw: &'a [u8], found: &mut Findings) -> Cow<'a, str> {
    let head = &raw[..raw.len().min(4096)];
    let sniffed = charset::detect(head, Some(Charset::Utf8)).charset;
    if matches!(sniffed, Charset::Utf16Le | Charset::Utf16Be) {
        return charset::decode(raw, sniffed);
    }
    match prolog_encoding(raw) {
        None => charset::decode(raw, Charset::Utf8),
        Some(label) => match charset::from_label(&label) {
            Some(cs @ (Charset::Utf16Le | Charset::Utf16Be)) if sniffed != cs => {
                // Says UTF-16, is not: read as it is.
                charset::decode(raw, Charset::Utf8)
            }
            Some(cs) => charset::decode(raw, cs),
            None => {
                found.encoding = Some(label);
                charset::decode(raw, Charset::Utf8)
            }
        },
    }
}

/// The encoding a FictionBook declares, if this viewer knows it: for its
/// text (`T`).
pub fn declared_charset(raw: &[u8]) -> Option<Charset> {
    // UTF-16 is told by its bytes, not by what the text says.
    prolog_encoding(raw)
        .and_then(|l| charset::from_label(&l))
        .filter(|c| !matches!(c, Charset::Utf16Le | Charset::Utf16Be))
}

/// The encoding the XML prolog declares: `<?xml version="1.0"
/// encoding="windows-1251"?>`.
fn prolog_encoding(raw: &[u8]) -> Option<String> {
    let raw = raw.strip_prefix(&[0xEF, 0xBB, 0xBF][..]).unwrap_or(raw);
    let head: String = raw.iter().take(256).map(|&b| b as char).collect();
    let prolog = head.trim_start().strip_prefix("<?xml")?;
    let prolog = &prolog[..prolog.find("?>")?];
    let at = prolog.find("encoding")?;
    let rest = prolog[at + "encoding".len()..]
        .trim_start()
        .strip_prefix('=')?;
    let rest = rest.trim_start();
    let quote = rest.chars().next().filter(|c| *c == '"' || *c == '\'')?;
    let value = &rest[1..];
    let value = &value[..value.find(quote)?];
    (!value.is_empty() && value.len() <= 40 && value.is_ascii()).then(|| value.to_string())
}

/// What a first reading of the file finds: what the book says of itself,
/// its embedded files, and how deep its chapters are.
#[derive(Default)]
struct Scan {
    meta: Vec<(String, String)>,
    annotation: Option<String>,
    /// The embedded files by id: their type and their bytes, from the
    /// length of the base64 (never decoded).
    binaries: HashMap<String, (String, u64)>,
    /// The level of the sections that are chapters: 1, or 2 when the book
    /// is all in one section with sections in it.
    chapter_depth: usize,
    late: bool,
}

impl Scan {
    fn read(text: &str, deadline: Instant) -> Scan {
        let mut scan = Scan {
            chapter_depth: 1,
            ..Scan::default()
        };
        let mut xml = Xml::html(text, deadline);
        // Where in the description the reading is.
        let mut path: Vec<String> = Vec::new();
        let mut value = String::new();
        let mut author: Vec<(String, String)> = Vec::new();
        let mut authors: Vec<String> = Vec::new();
        let mut genres: Vec<String> = Vec::new();
        let mut fields: Vec<(&str, String)> = Vec::new();
        let mut annotation = String::new();
        let mut series = None;
        // The binary being measured.
        let mut binary: Option<(String, String, u64)> = None;
        // The first body that is not notes: how deep its sections are open,
        // how many are on the first level, and whether the first one has
        // sections in it.
        let mut bodies = 0usize;
        let mut main = false;
        let mut depth = 0usize;
        let mut top = 0usize;
        let mut nested = false;
        while let Ok(Some(ev)) = xml.next() {
            match ev {
                Ev::Start(n, a) => {
                    if n == "binary" {
                        binary = Some((
                            attr(&a, "id").unwrap_or("").to_string(),
                            attr(&a, "content-type").unwrap_or("?").to_string(),
                            0,
                        ));
                        continue;
                    }
                    if n == "body" {
                        bodies += 1;
                        main = bodies == 1 && !is_notes(&a);
                        depth = 0;
                        continue;
                    }
                    if main && n == "section" {
                        depth += 1;
                        if depth == 1 {
                            top += 1;
                        } else if depth == 2 && top == 1 {
                            nested = true;
                        }
                        continue;
                    }
                    if path.is_empty() && n != "description" {
                        continue;
                    }
                    if path.len() < 16 {
                        path.push(n.clone());
                    }
                    value.clear();
                    let under = |p: &[String], what: &str| p.iter().any(|s| s == what);
                    if n == "author" && under(&path, "title-info") {
                        author.clear();
                    }
                    if n == "date"
                        && under(&path, "title-info")
                        && let Some(v) = attr(&a, "value")
                    {
                        fields.push(("Date", v.to_string()));
                    }
                }
                Ev::Empty(n, a) => {
                    if n == "sequence"
                        && path.iter().any(|s| s == "title-info")
                        && series.is_none()
                        && let Some(name) = attr(&a, "name")
                    {
                        let name = collapse(name).trim().to_string();
                        series = Some(match attr(&a, "number") {
                            Some(k) if !k.trim().is_empty() => format!("{name}, {}", k.trim()),
                            _ => name,
                        });
                    }
                }
                Ev::Text(t) => {
                    if let Some((_, _, bytes)) = &mut binary {
                        let chars = t.bytes().filter(|b| !b.is_ascii_whitespace()).count();
                        *bytes += (chars as u64) * 3 / 4;
                    } else if !path.is_empty() {
                        if path.iter().any(|s| s == "annotation") {
                            if annotation.len() < MAX_ANNOTATION {
                                annotation.push_str(&t);
                            }
                        } else if value.len() < MAX_TEXT {
                            value.push_str(&t);
                        }
                    }
                }
                Ev::End(n) => {
                    if n == "binary" {
                        if let Some((id, kind, bytes)) = binary.take()
                            && scan.binaries.len() < 10_000
                        {
                            scan.binaries.insert(id, (kind, bytes));
                        }
                        continue;
                    }
                    if n == "body" {
                        main = false;
                        continue;
                    }
                    if main && n == "section" {
                        depth = depth.saturating_sub(1);
                        continue;
                    }
                    if path.is_empty() {
                        continue;
                    }
                    let v = collapse(&value).trim().to_string();
                    let parent = path.len().checked_sub(2).map(|i| path[i].as_str());
                    let info = path.get(1).map(String::as_str);
                    match (info, parent, n.as_str()) {
                        (Some("title-info"), _, "book-title") => fields.push(("Title", v)),
                        (Some("title-info"), Some("author"), part) => author.push((part.into(), v)),
                        (Some("title-info"), _, "author") => {
                            let get = |k: &str| {
                                author
                                    .iter()
                                    .find(|(p, _)| p == k)
                                    .map(|(_, v)| v.as_str())
                                    .filter(|v| !v.is_empty())
                            };
                            let name: Vec<&str> = ["first-name", "middle-name", "last-name"]
                                .iter()
                                .filter_map(|k| get(k))
                                .collect();
                            let name = if name.is_empty() {
                                get("nickname").unwrap_or("").to_string()
                            } else {
                                name.join(" ")
                            };
                            if !name.is_empty() && authors.len() < 50 {
                                authors.push(name);
                            }
                        }
                        (Some("title-info"), _, "genre") if genres.len() < 20 => genres.push(v),
                        (Some("title-info"), _, "lang") => fields.push(("Language", v)),
                        (Some("title-info"), _, "date") => fields.push(("Date", v)),
                        (Some("document-info"), _, "program-used") => {
                            fields.push(("Made with", v));
                        }
                        (Some("document-info"), _, "id") => fields.push(("Identifier", v)),
                        (Some("publish-info"), _, "publisher") => fields.push(("Publisher", v)),
                        (Some("publish-info"), _, "year") => fields.push(("Year", v)),
                        (Some("publish-info"), _, "isbn") => fields.push(("ISBN", v)),
                        _ => {}
                    }
                    value.clear();
                    path.pop();
                }
            }
        }
        scan.late = xml.late;
        scan.chapter_depth = if top == 1 && nested { 2 } else { 1 };
        let get = |label: &str| {
            fields
                .iter()
                .find(|(l, v)| *l == label && !v.is_empty())
                .map(|(_, v)| v.clone())
        };
        let mut meta = Vec::new();
        let mut push = |label: &str, v: Option<String>| {
            if let Some(v) = v {
                meta.push((label.to_string(), v));
            }
        };
        push("Title", get("Title"));
        match authors.len() {
            0 => {}
            1 => push("Author", authors.pop()),
            _ => push("Authors", Some(authors.join("; "))),
        }
        push("Series", series);
        push("Publisher", get("Publisher"));
        push("Language", get("Language"));
        // The year it was published, for the title page, else the date of
        // the work.
        push("Date", get("Year").or_else(|| get("Date")));
        match genres.len() {
            0 => {}
            1 => push("Subject", genres.pop()),
            _ => push("Subjects", Some(genres.join("; "))),
        }
        push("ISBN", get("ISBN"));
        push("Made with", get("Made with"));
        push("Identifier", get("Identifier"));
        scan.meta = meta;
        let annotation = collapse(&annotation).trim().to_string();
        scan.annotation = (!annotation.is_empty()).then_some(annotation);
        scan
    }
}

/// A `body` of notes (`name="notes"`, `comments`), not of the text.
fn is_notes(a: &[(String, String)]) -> bool {
    attr(a, "name").is_some_and(|n| {
        let n = n.to_ascii_lowercase();
        n.contains("note") || n.contains("comment")
    })
}

/// Which body is being read.
#[derive(Clone, Copy, PartialEq)]
enum Body {
    None,
    Main,
    Notes,
}

/// An element of the FB2 open: its name, and the XHTML ones it opened in
/// the writer, closed in the reverse order.
struct Open {
    name: String,
    opened: Vec<&'static str>,
}

/// A title being read: the entry of the contents it gives, if any, and its
/// text so far.
struct Title {
    entry: Option<usize>,
    text: String,
    lines: usize,
}

/// The FB2 read into the writer, chapter by chapter.
struct Translator<'s> {
    w: Writer,
    scan: &'s Scan,
    book_title: String,
    open: Vec<Open>,
    /// Elements past the limit, or inside one not read (`description`,
    /// `binary`), skipped to their end.
    skip: usize,
    body: Body,
    depth: usize,
    chapter_open: bool,
    chapters: Vec<Chapter>,
    toc: Vec<Entry>,
    title: Option<Title>,
    /// The id of the section whose title is still to come.
    section_id: Option<String>,
    next_id: usize,
    /// Verses of the stanza being read.
    verses: usize,
    /// The id of the note being read, until its text starts: what is kept
    /// of a note is its text, not its title (the number its call shows).
    note_id: Option<String>,
}

impl<'s> Translator<'s> {
    fn new(scan: &'s Scan, book_title: String) -> Self {
        Translator {
            w: Writer::new(),
            scan,
            book_title,
            open: Vec::new(),
            skip: 0,
            body: Body::None,
            depth: 0,
            chapter_open: false,
            chapters: Vec::new(),
            toc: Vec::new(),
            title: None,
            section_id: None,
            next_id: 0,
            verses: 0,
            note_id: None,
        }
    }

    /// Reads the file; `true` if the time limit stopped it.
    fn run(&mut self, text: &str, deadline: Instant) -> bool {
        let mut xml = Xml::html(text, deadline);
        loop {
            let ev = match xml.next() {
                Ok(Some(ev)) => ev,
                Ok(None) => break,
                Err(why) => {
                    self.end_open();
                    self.w.note(&format!("[the rest cannot be read: {why}]"));
                    break;
                }
            };
            if self.w.d.cut {
                break;
            }
            match ev {
                Ev::Start(n, a) => self.start(n, &a),
                Ev::Empty(n, a) => self.empty(&n, &a),
                Ev::End(n) => self.end(&n),
                Ev::Text(t) => self.text(&t),
            }
        }
        self.end_open();
        if self.chapter_open {
            self.end_chapter();
        }
        xml.late
    }

    fn end_open(&mut self) {
        while let Some(o) = self.open.pop() {
            self.close(o);
        }
    }

    fn in_title(&self) -> bool {
        self.title.is_some()
    }

    /// The FB2 element open around the one being read.
    fn parent(&self) -> Option<&str> {
        self.open.last().map(|o| o.name.as_str())
    }

    /// The text of the main body goes in a chapter: one is opened for what
    /// comes outside the sections that are chapters (a title, an epigraph).
    fn in_chapter(&mut self) {
        if self.body == Body::Main && !self.chapter_open {
            self.begin_chapter();
        }
    }

    fn begin_chapter(&mut self) {
        self.chapter_open = true;
        if self.chapters.len() >= MAX_CHAPTERS {
            return;
        }
        let place: Box<str> = format!("c{}", self.chapters.len() + 1).into();
        self.w.begin_chapter(PART, place.clone());
        self.chapters.push(Chapter {
            place,
            title: String::new(),
            words: 0,
        });
    }

    fn end_chapter(&mut self) {
        self.chapter_open = false;
        let n = self.chapters.len();
        let Some(c) = self.chapters.last_mut() else {
            return;
        };
        if !c.title.is_empty() {
            // Over the limit of chapters: the text went into the last one.
            return;
        }
        let fallback = if n == 1 && !self.book_title.is_empty() {
            self.book_title.clone()
        } else {
            format!("Chapter {n}")
        };
        c.words = self.w.end_chapter(&fallback);
        let own = self.w.title.trim();
        c.title = if own.is_empty() {
            fallback
        } else {
            own.to_string()
        };
    }

    /// Opens XHTML elements in the writer, for the FB2 one `name`.
    fn push(&mut self, name: String, opened: &[(&'static str, Vec<(String, String)>)]) {
        for (n, a) in opened {
            self.w.open_element((*n).to_string(), a);
        }
        self.open.push(Open {
            name,
            opened: opened.iter().map(|(n, _)| *n).collect(),
        });
    }

    fn start(&mut self, name: String, a: &[(String, String)]) {
        if self.skip > 0 || matches!(name.as_str(), "description" | "binary" | "stylesheet") {
            self.skip += 1;
            return;
        }
        if name != "title" && !self.in_title() {
            self.note_text();
        }
        let none = Vec::new;
        match name.as_str() {
            "FictionBook" => self.push(name, &[]),
            "body" => {
                if self.chapter_open {
                    self.end_chapter();
                }
                self.body = if is_notes(a) { Body::Notes } else { Body::Main };
                self.depth = 0;
                self.push(name, &[]);
            }
            _ if self.body == Body::None => {
                // Outside any body: nothing of the text.
                self.skip += 1;
            }
            "section" => {
                self.depth += 1;
                self.w.void_element("div", &[]);
                let id = attr(a, "id").map(str::to_string);
                match self.body {
                    Body::Notes if self.depth == 1 => {
                        let at = vec![("type".to_string(), "footnote".to_string())];
                        self.push(name, &[("aside", at)]);
                        self.note_id = id;
                    }
                    _ => {
                        if self.body == Body::Main && self.depth == self.scan.chapter_depth {
                            if self.chapter_open {
                                self.end_chapter();
                            }
                            self.begin_chapter();
                        }
                        self.section_id = id;
                        self.push(name, &[]);
                    }
                }
            }
            "title" => self.start_title(name),
            "p" if self.in_title() => {
                if let Some(t) = &mut self.title {
                    let before = t.text.trim_end().to_string();
                    if t.lines > 0 && !before.is_empty() {
                        let ends = before.ends_with(['.', '!', '?', ':', '…', ';']);
                        let sep = if ends { " " } else { ". " };
                        t.text.push_str(sep);
                        self.w.text(sep);
                    }
                    t.lines += 1;
                }
                self.push(name, &[("span", none())]);
            }
            "p" => {
                self.in_chapter();
                self.push(name, &[("p", none())]);
            }
            "subtitle" => {
                self.in_chapter();
                self.push(name, &[("p", none()), ("strong", none())]);
            }
            "text-author" | "date" => {
                self.in_chapter();
                self.push(name, &[("p", none()), ("em", none())]);
            }
            "empty-line" => {
                self.in_chapter();
                self.w.void_element("p", &[]);
                self.push(name, &[]);
            }
            "poem" | "epigraph" | "cite" => {
                self.in_chapter();
                self.push(name, &[("blockquote", none())]);
            }
            "stanza" => {
                self.verses = 0;
                self.push(name, &[("p", none())]);
            }
            "v" => {
                if self.verses > 0 {
                    self.w.void_element("br", &[]);
                }
                self.verses += 1;
                self.push(name, &[("span", none())]);
            }
            "emphasis" => self.push(name, &[("em", none())]),
            "strong" => self.push(name, &[("strong", none())]),
            "strikethrough" => self.push(name, &[("s", none())]),
            "sub" => self.push(name, &[("sub", none())]),
            "sup" => self.push(name, &[("sup", none())]),
            "code" => self.push(name, &[("code", none())]),
            "style" => self.push(name, &[("span", none())]),
            "a" => {
                let mut at = Vec::new();
                if let Some(h) = attr(a, "href") {
                    at.push(("href".to_string(), h.to_string()));
                }
                if attr(a, "type") == Some("note") {
                    at.push(("type".into(), "noteref".into()));
                }
                self.push(name, &[("a", at)]);
            }
            "image" => {
                self.in_chapter();
                self.image(a);
                self.push(name, &[]);
            }
            "table" | "tr" | "td" | "th" => {
                self.in_chapter();
                let tag = match name.as_str() {
                    "table" => "table",
                    "tr" => "tr",
                    "td" => "td",
                    _ => "th",
                };
                self.push(name, &[(tag, none())]);
            }
            _ => {
                // Not FictionBook: given to the writer as it is, which says
                // what it carries (a `<script>`, an `<iframe>`).
                self.in_chapter();
                let lower = name.to_ascii_lowercase();
                self.w.open_element(lower, a);
                self.open.push(Open {
                    name,
                    opened: vec!["\u{1}"],
                });
            }
        }
    }

    /// A title: of a section (a heading, in the contents), of the body, of
    /// a poem, or of a note.
    fn start_title(&mut self, name: String) {
        let none = Vec::new;
        if matches!(self.parent(), Some("poem" | "stanza" | "epigraph" | "cite")) {
            self.title = Some(Title {
                entry: None,
                text: String::new(),
                lines: 0,
            });
            self.push(name, &[("p", none()), ("strong", none())]);
            return;
        }
        if self.body == Body::Notes {
            self.title = Some(Title {
                entry: None,
                text: String::new(),
                lines: 0,
            });
            if self.depth == 0 {
                // The title of the notes: shown with them, apart in a book.
                let note = vec![("type".to_string(), "footnote".to_string())];
                self.push(name, &[("aside", note), ("h2", none())]);
            } else {
                self.push(name, &[("p", none()), ("strong", none())]);
            }
            return;
        }
        self.in_chapter();
        let level = self.depth.clamp(1, 6);
        let id = self.section_id.take().unwrap_or_else(|| {
            self.next_id += 1;
            format!("\u{1}{}", self.next_id)
        });
        let entry = (self.depth > 0 && self.toc.len() < MAX_CHAPTERS).then(|| {
            self.toc.push(Entry {
                level: self.depth,
                title: String::new(),
                target: Some((PART.to_string(), Some(id.clone()))),
            });
            self.toc.len() - 1
        });
        self.title = Some(Title {
            entry,
            text: String::new(),
            lines: 0,
        });
        const LEVELS: [&str; 6] = ["h1", "h2", "h3", "h4", "h5", "h6"];
        self.push(name, &[(LEVELS[level - 1], vec![("id".to_string(), id)])]);
    }

    /// The text of a note starts: what follows, to the end of the note, is
    /// kept under its id.
    fn note_text(&mut self) {
        let Some(id) = self.note_id.take() else {
            return;
        };
        self.w.open_element("div".into(), &[("id".to_string(), id)]);
        if let Some(o) = self.open.iter_mut().rev().find(|o| o.name == "section") {
            o.opened.push("div");
        }
    }

    /// An image, as a line: its id, its type and its size.
    fn image(&mut self, a: &[(String, String)]) {
        let href = attr(a, "href").unwrap_or("").trim();
        let src = match href.strip_prefix('#') {
            Some(id) => match self.scan.binaries.get(id) {
                Some((kind, bytes)) => format!("{id}, {kind}, {}", crate::info::human(*bytes)),
                None => format!("{id}, not in the file"),
            },
            None => href.to_string(),
        };
        let src: String = src.chars().take(200).collect();
        self.w.void_element("img", &[("src".to_string(), src)]);
    }

    fn empty(&mut self, name: &str, a: &[(String, String)]) {
        if self.skip > 0 || self.body == Body::None {
            return;
        }
        match name {
            "empty-line" | "p" | "section" | "subtitle" => {
                if self.body == Body::Main && name != "section" {
                    self.in_chapter();
                }
                self.w.void_element("p", &[]);
            }
            "image" => {
                self.in_chapter();
                self.image(a);
            }
            "v" => {
                if self.verses > 0 {
                    self.w.void_element("br", &[]);
                }
            }
            _ => {
                let lower = name.to_ascii_lowercase();
                self.w.void_element(&lower, a);
            }
        }
    }

    fn end(&mut self, name: &str) {
        if self.skip > 0 {
            self.skip -= 1;
            return;
        }
        let Some(i) = self.open.iter().rposition(|o| o.name == name) else {
            return;
        };
        while self.open.len() > i {
            if let Some(o) = self.open.pop() {
                self.close(o);
            }
        }
    }

    /// Closes what `o` opened in the writer, and what it was.
    fn close(&mut self, o: Open) {
        for n in o.opened.iter().rev() {
            if *n == "\u{1}" {
                self.w.close_element(&o.name.to_ascii_lowercase());
            } else {
                self.w.close_element(n);
            }
        }
        match o.name.as_str() {
            "title" => {
                if let Some(t) = self.title.take()
                    && let Some(e) = t.entry.and_then(|i| self.toc.get_mut(i))
                {
                    e.title = collapse(&t.text).trim().chars().take(200).collect();
                }
                self.note_text();
            }
            "section" => {
                self.w.void_element("div", &[]);
                if self.body == Body::Main && self.depth == self.scan.chapter_depth {
                    self.end_chapter();
                }
                self.depth = self.depth.saturating_sub(1);
                self.section_id = None;
                self.note_id = None;
            }
            "body" => {
                if self.chapter_open {
                    self.end_chapter();
                }
                self.body = Body::None;
                self.depth = 0;
            }
            _ => {}
        }
    }

    fn text(&mut self, t: &str) {
        if self.skip > 0 || self.body == Body::None {
            return;
        }
        if !t.trim().is_empty() {
            self.in_chapter();
            if !self.in_title() {
                self.note_text();
            }
        }
        if let Some(title) = &mut self.title
            && title.text.len() < MAX_TEXT
        {
            title.text.push_str(t);
        }
        self.w.text(t);
    }
}

#[cfg(test)]
pub mod test_books {
    //! Small FictionBooks for the tests, made here.

    /// A FictionBook with `body` and the notes `notes` (none if empty).
    pub fn fb2(description: &str, body: &str, notes: &str) -> String {
        let notes = if notes.is_empty() {
            String::new()
        } else {
            format!(r#"<body name="notes">{notes}</body>"#)
        };
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0" xmlns:l="http://www.w3.org/1999/xlink">
<description>{description}</description>
<body>{body}</body>{notes}
<binary id="cover.jpg" content-type="image/jpeg">/9j/4AAQSkZJRgABAQ==</binary>
</FictionBook>"#
        )
    }

    pub const DESCRIPTION: &str = r##"<title-info>
<genre>sf</genre><genre>adventure</genre>
<author><first-name>Ana</first-name><last-name>Lima</last-name></author>
<book-title>The Test Book</book-title>
<annotation><p>A book made for the tests.</p></annotation>
<lang>en</lang>
<sequence name="Tests" number="2"/>
<coverpage><image l:href="#cover.jpg"/></coverpage>
</title-info>
<document-info><program-used>Hand</program-used><id>fb2-test-0001</id></document-info>
<publish-info><publisher>Nobody</publisher><year>2001</year><isbn>978-0</isbn></publish-info>"##;

    /// Two chapters, a poem, a note and an image.
    pub fn book2() -> Vec<u8> {
        fb2(
            DESCRIPTION,
            r##"<title><p>The Test Book</p></title>
<epigraph><p>Before all.</p><text-author>Someone</text-author></epigraph>
<section id="ch1"><title><p>Chapter 1</p><p>The Start</p></title>
<p>It began <emphasis>slowly</emphasis>, with a call<a l:href="#n1" type="note">[1]</a>.</p>
<subtitle>* * *</subtitle>
<p>Then <strong>fast</strong>.</p>
<image l:href="#cover.jpg"/>
</section>
<section id="ch2"><title><p>Chapter 2</p></title>
<poem><title><p>A Song</p></title><stanza><v>First line</v><v>Second line</v></stanza></poem>
<p>See <a l:href="#ch1">the start</a> or <a l:href="https://example.com/x">a site</a>.</p>
<section><title><p>A part of it</p></title><p>Deeper.</p></section>
</section>"##,
            r#"<title><p>Notes</p></title>
<section id="n1"><title><p>1</p></title><p>The note itself.</p></section>"#,
        )
        .into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::test_books::*;
    use super::*;
    use crate::charset::test_util::encode;
    use crate::viewer::markdown::Doc;
    use std::time::Duration;

    fn limits() -> Limits {
        Limits {
            part: 8 << 20,
            total: 8 << 20,
            time: Duration::from_secs(10),
            budget: 64 << 20,
        }
    }

    fn read(bytes: &[u8]) -> Book {
        render_with(bytes, limits(), 80, &|_| crate::reading::Summary::default()).unwrap()
    }

    fn text(doc: &Doc) -> String {
        (0..doc.line_count())
            .map(|i| doc.line(i).to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn names() {
        use std::path::Path;
        assert!(is_fb2(Path::new("a/Book.FB2")));
        assert!(is_fb2(Path::new("book.fb2.zip")));
        assert!(!is_fb2(Path::new("book.zip")));
        assert!(!is_fb2(Path::new("fb2")));
    }

    #[test]
    fn a_fictionbook_reads_as_a_book_with_its_chapters_and_notes() {
        let book = read(&book2());
        let titles: Vec<&str> = book.chapters.iter().map(|c| c.title.as_str()).collect();
        // The body's title and epigraph open a chapter of their own.
        assert_eq!(
            titles,
            ["The Test Book", "Chapter 1. The Start", "Chapter 2"]
        );
        let toc: Vec<(usize, &str)> = book
            .toc
            .iter()
            .map(|e| (e.level, e.title.as_str()))
            .collect();
        assert_eq!(
            toc,
            [
                (1, "Chapter 1. The Start"),
                (1, "Chapter 2"),
                (2, "A part of it")
            ]
        );
        assert!(book.toc.iter().all(|e| e.place.is_some()), "{:?}", book.toc);
        assert_eq!(book.identifier.as_deref(), Some("fb2-test-0001"));
        assert_eq!(book.title.as_deref(), Some("The Test Book"));
        assert_eq!(book.author.as_deref(), Some("Ana Lima"));

        let report = text(&book.report);
        for s in [
            "Title: The Test Book",
            "Authors",
            "Series: Tests, 2",
            "Subjects: sf; adventure",
            "ISBN: 978-0",
            "Made with: Hand",
            "Format: FictionBook 2",
            "A book made for the tests.",
            "Embedded files: 1 (image/jpeg)",
            "[image: cover.jpg, image/jpeg,",
            "Links to outside: 1",
            "The note itself.",
            "First line",
        ] {
            assert!(
                report.contains(s) || (s == "Authors" && report.contains("Author: Ana Lima")),
                "{s:?} not in\n{report}"
            );
        }
        // A verse per line.
        let lines: Vec<&str> = report.lines().collect();
        let first = lines.iter().position(|l| l.contains("First line")).unwrap();
        assert!(lines[first + 1].contains("Second line"), "{report}");

        let reading = text(&book.reading);
        assert!(reading.contains("The Test Book"));
        assert!(reading.contains("Ana Lima"));
        assert!(reading.contains("Nobody · 2001"), "{reading}");
        // The notes are apart in the book; the call is a superscript.
        assert!(!reading.contains("The note itself."), "{reading}");
        assert!(reading.contains("call¹"), "{reading}");
        assert_eq!(book.notes.len(), 1);
        // Its text, without the number its title gives.
        assert!(
            book.notes.values().all(|n| n == "The note itself."),
            "{:?}",
            book.notes
        );
        // The scene break is a rule, not stars.
        assert!(!reading.contains("* * *"), "{reading}");
    }

    #[test]
    fn a_book_in_one_section_takes_its_chapters_one_level_down() {
        let body = r#"<section><title><p>Part One</p></title>
<section><title><p>One</p></title><p>a</p></section>
<section><title><p>Two</p></title><p>b</p></section>
</section>"#;
        let book = read(fb2("", body, "").as_bytes());
        let titles: Vec<&str> = book.chapters.iter().map(|c| c.title.as_str()).collect();
        assert_eq!(titles, ["Part One", "One", "Two"]);
        // With nothing to say of itself, the chapters are still there.
        assert!(text(&book.reading).contains("(no title)"));
    }

    #[test]
    fn cyrillic_encodings_are_read_as_declared() {
        let ru = "Глава первая";
        for (label, table) in [
            ("windows-1251", &crate::charset::CP1251),
            ("KOI8-R", &crate::charset::KOI8_R),
        ] {
            let book =
                fb2("", &format!("<section><p>{ru}</p></section>"), "").replace("utf-8", label);
            // Encoded as declared: ASCII stays, the rest goes through the table.
            let bytes: Vec<u8> = book
                .chars()
                .flat_map(|c| {
                    if c.is_ascii() {
                        vec![c as u8]
                    } else {
                        encode(&c.to_string(), table)
                    }
                })
                .collect();
            let book = read(&bytes);
            assert!(text(&book.reading).contains(ru), "{label}");
        }
        // An encoding not known is said, and the text read as UTF-8.
        let book =
            fb2("", "<section><p>plain</p></section>", "").replace("utf-8", "x-mac-cyrillic");
        let book = read(book.as_bytes());
        let report = text(&book.report);
        assert!(report.contains("x-mac-cyrillic"), "{report}");
        assert!(text(&book.reading).contains("plain"));
    }

    #[test]
    fn a_zip_with_one_fictionbook_is_read_and_another_zip_is_not() {
        let one = crate::viewer::office::test_docs::zip(&[(
            "book.fb2",
            std::str::from_utf8(&book2()).unwrap(),
        )]);
        let book = read(&one);
        assert_eq!(book.chapters.len(), 3);
        assert!(text(&book.report).contains("FictionBook 2, in a ZIP"));
        let two = crate::viewer::office::test_docs::zip(&[
            ("book.fb2", std::str::from_utf8(&book2()).unwrap()),
            ("other.txt", "x"),
        ]);
        let none = |_: Option<&str>| crate::reading::Summary::default();
        assert!(render_with(&two, limits(), 80, &none).is_err());
        let other = crate::viewer::office::test_docs::zip(&[("a.txt", "x")]);
        assert!(render_with(&other, limits(), 80, &none).is_err());
        assert!(render_with(b"not xml at all", limits(), 80, &none).is_err());
    }

    #[test]
    fn what_a_hostile_book_carries_is_said_and_nothing_runs() {
        let body = r##"<section><title><p>One</p></title>
<p>Text<script>alert(1)</script> and <style>bold</style>.</p>
<image l:href="https://tracker.example/pixel.gif"/>
<image l:href="#nowhere"/>
</section>"##;
        let book = read(fb2("", body, "").as_bytes());
        let report = text(&book.report);
        assert!(report.contains("Contains scripts"), "{report}");
        assert!(
            report.contains("https://tracker.example/pixel.gif"),
            "{report}"
        );
        assert!(report.contains("nowhere, not in the file"), "{report}");
        // The FB2 `style` is text, not a style sheet skipped.
        assert!(text(&book.reading).contains("bold"));
        assert!(!text(&book.reading).contains("alert"));
        assert!(book.warning.is_some());
        // A DTD with entities is refused, as everywhere.
        let bomb = r#"<?xml version="1.0"?><!DOCTYPE x [<!ENTITY a "aaaa">]><FictionBook><body><p>&a;</p></body></FictionBook>"#;
        let book = read(bomb.as_bytes());
        assert!(!text(&book.reading).contains("aaaa"));
    }

    #[test]
    fn a_book_over_the_limit_is_read_as_far_as_it_goes() {
        let body: String = (1..=50)
            .map(|i| {
                format!(
                    "<section><title><p>C{i}</p></title><p>{}</p></section>",
                    "word ".repeat(200)
                )
            })
            .collect();
        let bytes = fb2("", &body, "").into_bytes();
        let small = Limits {
            part: 20_000,
            ..limits()
        };
        let book = render_with(&bytes, small, 80, &|_| crate::reading::Summary::default()).unwrap();
        assert!(book.chapters.len() < 50 && !book.chapters.is_empty());
        assert!(text(&book.report).contains("Only the start of the book was read"));
    }

    #[test]
    fn prologs() {
        assert_eq!(
            prolog_encoding(br#"<?xml version="1.0" encoding="windows-1251"?><a/>"#).as_deref(),
            Some("windows-1251")
        );
        assert_eq!(
            prolog_encoding(b"\xEF\xBB\xBF<?xml version='1.0' encoding='KOI8-R' ?>").as_deref(),
            Some("KOI8-R")
        );
        assert_eq!(prolog_encoding(br#"<?xml version="1.0"?>"#), None);
        assert_eq!(prolog_encoding(b"<FictionBook encoding=\"x\">"), None);
    }
}
