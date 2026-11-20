//! EPUB ebooks (docs/plans/viewer-ebooks.md) shown as a document: what the
//! book says of itself and what it carries besides text (scripts, things
//! fetched from outside, DRM) at the top, then its contents as links, then
//! its chapters in reading order, each starting at a heading the contents
//! and `]`/`[` lead to.
//!
//! The book is hostile, as an Office package is (the same [`Package`] and
//! [`Xml`]): each part decompressed up to a limit, no DTD ever read, paths
//! taken only as names of parts of the package, and the text going into the
//! document as text, never as Markdown. Nothing in it is run or fetched.

use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek};
use std::time::Instant;

use pulldown_cmark::{CowStr, Event, LinkType, Tag, TagEnd};

use super::doc_events::{DocEvents, Look};
use super::markdown::{self, Doc};
pub use super::package::Limits;
use super::package::{Ev, MAX_DEPTH, Package, Xml, attr, resolve};

/// Most events of the document.
const MAX_EVENTS: usize = 4_000_000;
/// Chapters read, at most, and entries of the contents.
const MAX_CHAPTERS: usize = 10_000;
/// Addresses and names listed in the panel, at most, of each kind.
const MAX_LISTED: usize = 1_000;
/// Elements kept open at once while a chapter is read: past them an element
/// is read as if it had no tag (a chapter that never closes anything).
const MAX_FRAMES: usize = 2 * MAX_DEPTH;
/// Most bytes of one piece of text kept (a title, a cell, a code block).
const MAX_TEXT: usize = 64 * 1024;
/// Most bytes kept of the text of a note, of the notes in all, and how many
/// elements with an id are kept.
const MAX_NOTE: usize = 4096;
const MAX_CAPTURED: usize = 8 << 20;
const MAX_TEXTS: usize = 50_000;

/// Events of a document, with where each stands in the source.
type Events = Vec<(Event<'static>, std::ops::Range<usize>)>;

/// Where a link leads: a part of the package and a fragment in it.
type Target = (String, Option<String>);

/// Whether the file is named as an EPUB.
pub fn is_epub(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("epub"))
}

/// A book read, in its two views: the report on the file and the book to
/// read; and what the reader needs besides them.
pub struct Book {
    /// What the file is and carries, the contents as links, then the text.
    pub report: Doc,
    /// The title page, then the text, with the footnotes apart and their
    /// calls as superscripts.
    pub reading: Doc,
    pub chapters: Vec<Chapter>,
    /// The contents, each entry with the place it leads to.
    pub toc: Vec<TocEntry>,
    /// The text of each note, by the place its call leads to (`n12`).
    pub notes: HashMap<Box<str>, String>,
    /// What the report warns about, in a line, when there is anything.
    pub warning: Option<String>,
}

/// A chapter: the place it starts at (see [`Doc::place`]), in both views;
/// its title; and how many words it has.
#[derive(Debug, Clone, PartialEq)]
pub struct Chapter {
    pub place: Box<str>,
    pub title: String,
    pub words: usize,
}

/// An entry of the contents: its level (1 at the top), title, and the place
/// it leads to.
#[derive(Debug, Clone, PartialEq)]
pub struct TocEntry {
    pub level: usize,
    pub title: String,
    pub place: Option<Box<str>>,
}

/// The book in the package `reader`, laid out for `width`, or why it is not
/// one (the bytes are then shown).
pub fn render<R: Read + Seek>(reader: R, limits: Limits, width: usize) -> Result<Book, String> {
    let mut pkg = Package::open(reader, limits)
        .map_err(|_| "Not an EPUB this viewer can read: showing the bytes".to_string())?;
    let container = pkg
        .text("META-INF/container.xml")
        .ok_or("No META-INF/container.xml: not an EPUB, showing the bytes")?;
    let opf_path = rootfile(&container, pkg.deadline)
        .ok_or("The EPUB names no package file (OPF): showing the bytes")?;
    let opf_text = pkg
        .text(&opf_path)
        .ok_or_else(|| format!("No {opf_path} in the EPUB: showing the bytes"))?;
    let opf = Opf::read(&opf_text, &opf_path, pkg.deadline);

    let mut found = Findings::default();
    manifest_findings(&mut pkg, &opf, &opf_path, &mut found);
    let encrypted = encryption(&mut pkg, &mut found);

    let toc = contents(&mut pkg, &opf);

    // The chapters, each once, in the order of the spine.
    let mut seen = HashSet::new();
    let mut chapters: Vec<(String, String)> = Vec::new();
    for idref in &opf.spine {
        if chapters.len() >= MAX_CHAPTERS {
            break;
        }
        let Some(item) = opf.manifest.get(idref) else {
            push_listed(&mut found.missing, format!("spine item {idref}"));
            continue;
        };
        let Some(path) = &item.path else { continue };
        if seen.insert(path.clone()) {
            chapters.push((path.clone(), item.media.clone()));
        }
    }

    let mut w = Writer::new();
    let mut late = false;
    let mut book_chapters = Vec::new();
    for (i, (path, media)) in chapters.iter().enumerate() {
        if w.d.cut || pkg.late() {
            late = true;
            break;
        }
        let n = i + 1;
        let place: Box<str> = format!("c{n}").into();
        let title = toc
            .iter()
            .find(|e| e.target.as_ref().is_some_and(|(p, _)| p == path))
            .map(|e| e.title.clone())
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| file_stem(path));
        w.begin_chapter(path, place.clone());
        if encrypted.contains(path) {
            w.note(&format!("[chapter {n}: encrypted, cannot be read]"));
        } else if !media.contains("html") && !media.contains("xml") {
            w.note(&format!("[chapter {n}: {media}, not shown]"));
        } else {
            match pkg.text(path) {
                Some(text) => match w.chapter(&text, pkg.deadline) {
                    Ok(l) => late |= l,
                    Err(why) => w.note(&format!("[chapter {n}: cannot be read: {why}]")),
                },
                None if pkg.has(path) => {
                    w.note(&format!("[chapter {n}: over the limits, not read]"));
                }
                None => {
                    push_listed(&mut found.missing, path.clone());
                    w.note(&format!("[chapter {n}: {path} is not in the file]"));
                }
            }
        }
        let words = w.end_chapter(&title);
        book_chapters.push(Chapter {
            place,
            title,
            words,
        });
    }
    late |= pkg.late();
    found.merge(&w.found);
    css_findings(&mut pkg, &opf, &mut found);

    // What it says of itself, then the contents, then the text.
    let mut head = DocEvents::new(200_000);
    panel(&mut head, &opf, &found, encrypted.len());
    if pkg.cut || late || w.d.cut {
        head.paragraph("Only the start of the book was read: it is over the limits.");
    }
    head.rule();
    head.heading(2, "Contents");
    if toc.is_empty() {
        head.start_list(false);
        for c in &book_chapters {
            head.start(Tag::Item);
            start_link(&mut head, &c.place);
            head.text(&c.title);
            head.end(TagEnd::Link);
            head.end(TagEnd::Item);
        }
        head.end_list(false);
    } else {
        toc_list(&mut head, &toc, &mut w);
    }
    head.rule();

    let cut = pkg.cut || late || w.d.cut;
    let chapter_events = std::mem::take(&mut w.d.events);

    // The book to read: the title page, then the text without the
    // footnotes, which are shown on demand.
    let mut reading_events = title_page(&opf, &found, &book_chapters, cut).events;
    let mut skip = w.notes_at.iter().peekable();
    for (i, ev) in chapter_events.iter().enumerate() {
        while skip.peek().is_some_and(|r| r.end <= i) {
            skip.next();
        }
        if skip.peek().is_some_and(|r| r.contains(&i)) {
            continue;
        }
        reading_events.push(ev.clone());
    }
    let (reading_events, notes) = w.resolve_links(reading_events, true);

    let mut events = head.events;
    events.extend(chapter_events);
    let (events, _) = w.resolve_links(events, false);

    let mut report = markdown::from_events_linked(events, limits.time, limits.budget);
    let mut reading = markdown::from_events_linked(reading_events, limits.time, limits.budget);
    if cut {
        report.mark_cut();
        reading.mark_cut();
    }
    report.layout(width);
    reading.layout(width);
    let toc = toc
        .iter()
        .map(|e| TocEntry {
            level: e.level,
            title: e.title.clone(),
            place: e
                .target
                .as_ref()
                .and_then(|(p, f)| w.place_for(p, f.as_deref())),
        })
        .collect();
    Ok(Book {
        report,
        reading,
        chapters: book_chapters,
        toc,
        notes,
        warning: warning(&found),
    })
}

/// The first page of the book to read: its title, who wrote it, who
/// published it and when; what the report warns about, if anything; and
/// how long the book is.
fn title_page(opf: &Opf, found: &Findings, chapters: &[Chapter], cut: bool) -> DocEvents {
    let mut d = DocEvents::new(1_000);
    let meta = |label: &str| {
        opf.meta
            .iter()
            .find(|(l, _)| l == label)
            .map(|(_, v)| v.as_str())
    };
    d.heading(1, meta("Title").unwrap_or("(no title)"));
    if let Some(who) = meta("Author").or(meta("Authors")) {
        d.heading(3, who);
    }
    let year = meta("Date").map(|d| d.chars().take(4).collect::<String>());
    let imprint: Vec<&str> = [meta("Publisher"), year.as_deref()]
        .into_iter()
        .flatten()
        .collect();
    if !imprint.is_empty() {
        d.heading(4, &imprint.join(" · "));
    }
    if let Some(w) = warning(found) {
        d.start(Tag::Paragraph);
        d.styled(&format!("⚠ {w} (I shows the report)"), Look::Strong);
        d.end(TagEnd::Paragraph);
    }
    let words: usize = chapters.iter().map(|c| c.words).sum();
    let n = chapters.len();
    d.paragraph(&format!(
        "{n} {}, {} words.{}",
        if n == 1 { "chapter" } else { "chapters" },
        group(words),
        if cut {
            " Only the start was read: it is over the limits."
        } else {
            ""
        }
    ));
    d.rule();
    d
}

/// What the report warns about, in a line: `3 scripts, 2 addresses outside`.
fn warning(found: &Findings) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(system) = &found.drm {
        parts.push(format!("DRM ({system})"));
    }
    if found.scripts_found() {
        parts.push("scripts".to_string());
    }
    match found.fetches.len() {
        0 if !found.remote.is_empty() => parts.push("things fetched from outside".into()),
        0 => {}
        1 => parts.push("1 address fetched from outside".into()),
        n => parts.push(format!("{n} addresses fetched from outside")),
    }
    if found.forms + found.frames + found.objects > 0 {
        parts.push("forms or frames".into());
    }
    if !found.outside.is_empty() {
        parts.push("paths outside the book".into());
    }
    (!parts.is_empty()).then(|| format!("This book has {}", parts.join(", ")))
}

/// `1234567` as `1,234,567`.
fn group(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// The name of a part without its folder and extension: a chapter's title
/// when nothing gives one.
fn file_stem(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    name.rsplit_once('.')
        .map_or(name, |(stem, _)| stem)
        .to_string()
}

/// Runs of white space as one space, as HTML shows them.
fn collapse(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut space = false;
    for c in s.chars() {
        if c.is_ascii_whitespace() {
            space = true;
        } else {
            if space {
                out.push(' ');
                space = false;
            }
            out.push(c);
        }
    }
    if space {
        out.push(' ');
    }
    out
}

/// `%20` and the like decoded, as an address in a package is written; a `%`
/// that is not followed by two hex digits stays as it is.
fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && let Some(hex) = b.get(i + 1..i + 3)
            && let Ok(hex) = std::str::from_utf8(hex)
            && let Ok(byte) = u8::from_str_radix(hex, 16)
        {
            out.push(byte);
            i += 3;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// An address to something outside the book: a reader that loads it tells
/// whoever is there that the book was opened.
fn external(href: &str) -> bool {
    let h = href.trim_start().to_ascii_lowercase();
    h.starts_with("http:")
        || h.starts_with("https:")
        || h.starts_with("//")
        || h.starts_with("ftp:")
}

/// An address inside the book, from the folder `dir`: the part and the
/// fragment (`chap2.xhtml#s3`), or `None` if it climbs out of the package.
fn inside(dir: &str, href: &str) -> Option<Target> {
    let (path, frag) = match href.split_once('#') {
        Some((p, f)) => (p, Some(percent_decode(f))),
        None => (href, None),
    };
    let path = path.split('?').next().unwrap_or("");
    // `dir` may carry the part's own name after a `\u{1}` (see [`base`]),
    // so that `#id` alone means the part itself.
    let (dir, own) = dir.split_once('\u{1}').unwrap_or((dir, ""));
    let part = if path.is_empty() {
        own.to_string()
    } else {
        resolve(dir, &percent_decode(path))?
    };
    Some((part, frag.filter(|f| !f.is_empty())))
}

fn folder(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(d, _)| d)
}

/// The folder of `part` with the part itself, for [`inside`].
fn base(part: &str) -> String {
    format!("{}\u{1}{part}", folder(part))
}

fn push_listed(list: &mut Vec<String>, s: String) {
    if list.len() < MAX_LISTED && !list.contains(&s) {
        list.push(s);
    }
}

/// The package file (OPF) `container.xml` names.
fn rootfile(container: &str, deadline: Instant) -> Option<String> {
    let mut xml = Xml::html(container, deadline);
    let mut first = None;
    while let Ok(Some(ev)) = xml.next() {
        if let Ev::Start(n, a) | Ev::Empty(n, a) = ev
            && n == "rootfile"
            && let Some(path) = attr(&a, "full-path")
        {
            let path = resolve("", &percent_decode(path))?;
            if attr(&a, "media-type") == Some("application/oebps-package+xml") {
                return Some(path);
            }
            first.get_or_insert(path);
        }
    }
    first
}

/// An item of the manifest: its part (`None` if its address leaves the
/// package), its type and its properties.
struct Item {
    href: String,
    path: Option<String>,
    media: String,
    properties: String,
}

/// What the package file says: the metadata, the manifest and the spine.
struct Opf {
    version: String,
    meta: Vec<(String, String)>,
    manifest: HashMap<String, Item>,
    /// The order of the manifest, for listing it.
    order: Vec<String>,
    spine: Vec<String>,
    /// The id of the EPUB 2 contents (`toc.ncx`).
    ncx: Option<String>,
}

impl Opf {
    fn read(text: &str, path: &str, deadline: Instant) -> Opf {
        let dir = folder(path);
        let mut opf = Opf {
            version: String::new(),
            meta: Vec::new(),
            manifest: HashMap::new(),
            order: Vec::new(),
            spine: Vec::new(),
            ncx: None,
        };
        let labels: &[(&str, &str)] = &[
            ("title", "Title"),
            ("creator", "Author"),
            ("publisher", "Publisher"),
            ("language", "Language"),
            ("date", "Date"),
            ("identifier", "Identifier"),
            ("subject", "Subject"),
        ];
        let mut unique = None;
        let mut current: Option<(&str, Option<String>)> = None;
        let mut value = String::new();
        let mut in_metadata = false;
        let mut values: Vec<(&str, String)> = Vec::new();
        let mut modified = None;
        let mut identifiers: Vec<(Option<String>, String)> = Vec::new();
        let mut xml = Xml::html(text, deadline);
        while let Ok(Some(ev)) = xml.next() {
            match ev {
                Ev::Start(n, a) | Ev::Empty(n, a) if n == "package" => {
                    opf.version = attr(&a, "version").unwrap_or("").to_string();
                    unique = attr(&a, "unique-identifier").map(str::to_string);
                }
                Ev::Start(n, _) if n == "metadata" => in_metadata = true,
                Ev::End(n) if n == "metadata" => in_metadata = false,
                Ev::Empty(n, a) | Ev::Start(n, a)
                    if in_metadata && n == "meta" && attr(&a, "name") == Some("generator") =>
                {
                    if let Some(c) = attr(&a, "content") {
                        values.push(("Made with", collapse(c).trim().to_string()));
                    }
                }
                Ev::Start(n, a) if in_metadata && n == "meta" => {
                    current = (attr(&a, "property") == Some("dcterms:modified"))
                        .then_some(("Modified", None));
                    value.clear();
                }
                Ev::Start(n, a) if in_metadata => {
                    current = labels
                        .iter()
                        .find(|(k, _)| *k == n)
                        .map(|(_, l)| (*l, attr(&a, "id").map(str::to_string)));
                    value.clear();
                }
                Ev::Text(t) if current.is_some() && value.len() < MAX_TEXT => value.push_str(&t),
                Ev::End(_) if current.is_some() => {
                    let (label, id) = current.take().unwrap_or(("", None));
                    let v = collapse(&value).trim().to_string();
                    if v.is_empty() {
                        continue;
                    }
                    match label {
                        "Modified" => modified = Some(v),
                        "Identifier" => identifiers.push((id, v)),
                        _ => values.push((label, v)),
                    }
                }
                Ev::Start(n, a) | Ev::Empty(n, a) if n == "item" => {
                    let (Some(id), Some(href)) = (attr(&a, "id"), attr(&a, "href")) else {
                        continue;
                    };
                    if opf.manifest.len() >= MAX_CHAPTERS * 4 || opf.manifest.contains_key(id) {
                        continue;
                    }
                    let path = if external(href) {
                        None
                    } else {
                        let p = href.split('#').next().unwrap_or("");
                        resolve(dir, &percent_decode(p))
                    };
                    opf.order.push(id.to_string());
                    opf.manifest.insert(
                        id.to_string(),
                        Item {
                            href: href.to_string(),
                            path,
                            media: attr(&a, "media-type").unwrap_or("").to_string(),
                            properties: attr(&a, "properties").unwrap_or("").to_string(),
                        },
                    );
                }
                Ev::Start(n, a) | Ev::Empty(n, a) if n == "spine" => {
                    opf.ncx = attr(&a, "toc").map(str::to_string);
                }
                Ev::Start(n, a) | Ev::Empty(n, a) if n == "itemref" => {
                    if let Some(id) = attr(&a, "idref")
                        && opf.spine.len() < MAX_CHAPTERS * 4
                    {
                        opf.spine.push(id.to_string());
                    }
                }
                _ => {}
            }
        }
        // Several authors or subjects on one line each.
        for (label, plural) in [("Author", "Authors"), ("Subject", "Subjects")] {
            let all: Vec<String> = values
                .iter()
                .filter(|(l, _)| *l == label)
                .map(|(_, v)| v.clone())
                .collect();
            if all.len() > 1 {
                values.retain(|(l, _)| *l != label);
                values.push((plural, all.join("; ")));
            }
        }
        let order = [
            "Title",
            "Author",
            "Authors",
            "Publisher",
            "Language",
            "Date",
            "Subject",
            "Subjects",
            "Made with",
        ];
        for label in order {
            if let Some((_, v)) = values.iter().find(|(l, _)| *l == label) {
                opf.meta.push((label.to_string(), v.clone()));
            }
        }
        let id = identifiers
            .iter()
            .find(|(i, _)| i.is_some() && *i == unique)
            .or(identifiers.first())
            .map(|(_, v)| v.clone());
        if let Some(id) = id {
            opf.meta.push(("Identifier".into(), id));
        }
        if let Some(m) = modified {
            opf.meta.push(("Modified".into(), m));
        }
        if !opf.version.is_empty() {
            opf.meta
                .push(("Format".into(), format!("EPUB {}", opf.version)));
        }
        opf
    }

    fn has_property(&self, property: &str) -> impl Iterator<Item = &Item> {
        self.order
            .iter()
            .filter_map(|id| self.manifest.get(id))
            .filter(move |i| i.properties.split_whitespace().any(|p| p == property))
    }
}

/// What a book carries besides its text, and what is wrong with its package.
#[derive(Default)]
struct Findings {
    /// `<script>` elements, by chapter.
    scripts: Vec<(String, usize)>,
    /// `onload=` and the like.
    handlers: usize,
    /// `javascript:` links.
    js_links: usize,
    /// Parts the manifest says are scripted.
    scripted: Vec<String>,
    /// Parts the manifest says load things from outside.
    remote: Vec<String>,
    /// Addresses outside that a reader would fetch: images, styles, fonts,
    /// frames.
    fetches: Vec<String>,
    /// Links to outside, which a reader opens only when clicked.
    links_out: usize,
    forms: usize,
    frames: usize,
    objects: usize,
    images: usize,
    /// Addresses that climb out of the package.
    outside: Vec<String>,
    /// In the manifest but not in the file.
    missing: Vec<String>,
    /// In the file but not in the manifest.
    unlisted: Vec<String>,
    unlisted_count: usize,
    /// Fonts obfuscated, as books may do (not DRM).
    fonts: usize,
    /// The DRM system when chapters are encrypted.
    drm: Option<String>,
}

impl Findings {
    fn merge(&mut self, other: &Findings) {
        self.scripts.extend(other.scripts.iter().cloned());
        self.handlers += other.handlers;
        self.js_links += other.js_links;
        for f in &other.fetches {
            push_listed(&mut self.fetches, f.clone());
        }
        for f in &other.outside {
            push_listed(&mut self.outside, f.clone());
        }
        self.links_out += other.links_out;
        self.forms += other.forms;
        self.frames += other.frames;
        self.objects += other.objects;
        self.images += other.images;
    }

    fn scripts_found(&self) -> bool {
        !self.scripts.is_empty()
            || self.handlers > 0
            || self.js_links > 0
            || !self.scripted.is_empty()
    }
}

fn manifest_findings<R: Read + Seek>(
    pkg: &mut Package<R>,
    opf: &Opf,
    opf_path: &str,
    found: &mut Findings,
) {
    let mut listed = HashSet::new();
    for id in &opf.order {
        let item = &opf.manifest[id];
        match &item.path {
            Some(p) => {
                listed.insert(p.clone());
                if !pkg.has(p) {
                    push_listed(&mut found.missing, p.clone());
                }
            }
            None if external(&item.href) => push_listed(&mut found.fetches, item.href.clone()),
            None => push_listed(&mut found.outside, item.href.clone()),
        }
    }
    found.scripted = opf
        .has_property("scripted")
        .map(|i| i.href.clone())
        .take(MAX_LISTED)
        .collect();
    found.remote = opf
        .has_property("remote-resources")
        .map(|i| i.href.clone())
        .take(MAX_LISTED)
        .collect();
    for name in &pkg.names {
        if name.ends_with('/')
            || name == "mimetype"
            || name.starts_with("META-INF/")
            || name == opf_path
            || listed.contains(name)
        {
            continue;
        }
        found.unlisted_count += 1;
        if found.unlisted.len() < 20 {
            found.unlisted.push(name.clone());
        }
    }
}

/// The parts `META-INF/encryption.xml` says are encrypted, other than fonts
/// obfuscated (which books do, and which is not DRM).
fn encryption<R: Read + Seek>(pkg: &mut Package<R>, found: &mut Findings) -> HashSet<String> {
    const FONTS: &[&str] = &[
        "http://www.idpf.org/2008/embedding",
        "http://ns.adobe.com/pdf/enc#RC",
    ];
    let mut out = HashSet::new();
    let Some(text) = pkg.text("META-INF/encryption.xml") else {
        return out;
    };
    let mut xml = Xml::html(&text, pkg.deadline);
    let mut algorithm = String::new();
    while let Ok(Some(ev)) = xml.next() {
        match ev {
            Ev::Start(n, _) if n == "EncryptedData" => algorithm.clear(),
            Ev::Start(n, a) | Ev::Empty(n, a) if n == "EncryptionMethod" => {
                algorithm = attr(&a, "Algorithm").unwrap_or("").to_string();
            }
            Ev::Start(n, a) | Ev::Empty(n, a) if n == "CipherReference" => {
                let Some(uri) = attr(&a, "URI") else { continue };
                if FONTS.contains(&algorithm.as_str()) {
                    found.fonts += 1;
                } else if let Some(p) = resolve("", &percent_decode(uri))
                    && out.len() < MAX_CHAPTERS * 4
                {
                    out.insert(p);
                }
            }
            _ => {}
        }
    }
    if !out.is_empty() {
        let system = if pkg.has("META-INF/rights.xml") {
            "Adobe ADEPT"
        } else if pkg.has("META-INF/license.lcpl") {
            "Readium LCP"
        } else if pkg.has("META-INF/sinf.xml") {
            "Apple FairPlay"
        } else {
            "unknown"
        };
        found.drm = Some(system.to_string());
    }
    out
}

/// Style sheets that load from outside (`url(https://...)`, `@import`).
fn css_findings<R: Read + Seek>(pkg: &mut Package<R>, opf: &Opf, found: &mut Findings) {
    let sheets: Vec<String> = opf
        .order
        .iter()
        .filter_map(|id| opf.manifest.get(id))
        .filter(|i| i.media == "text/css")
        .filter_map(|i| i.path.clone())
        .take(200)
        .collect();
    for path in sheets {
        if pkg.late() {
            break;
        }
        if let Some(text) = pkg.text(&path) {
            for url in css_urls(&text) {
                push_listed(&mut found.fetches, url);
            }
        }
    }
}

/// The addresses outside a style sheet loads.
fn css_urls(css: &str) -> Vec<String> {
    let mut out = Vec::new();
    let lower = css.to_ascii_lowercase();
    for key in ["url(", "@import"] {
        let mut from = 0;
        while let Some(at) = lower[from..].find(key) {
            let start = from + at + key.len();
            let rest = css[start..].trim_start_matches([' ', '\t', '\n', '\r', '"', '\'', '(']);
            let end = rest
                .find(['"', '\'', ')', ';', ' ', '\n'])
                .unwrap_or(rest.len());
            let url = &rest[..end.min(2048)];
            if external(url) && out.len() < MAX_LISTED {
                out.push(url.to_string());
            }
            from = start;
        }
    }
    out
}

/// An entry of the contents: its level (1 at the top), title and where it
/// leads.
struct Entry {
    level: usize,
    title: String,
    target: Option<Target>,
}

/// The contents: the EPUB 3 `nav` if the book has one with entries, else the
/// EPUB 2 `toc.ncx`.
fn contents<R: Read + Seek>(pkg: &mut Package<R>, opf: &Opf) -> Vec<Entry> {
    if let Some(path) = opf.has_property("nav").find_map(|i| i.path.clone())
        && let Some(text) = pkg.text(&path)
    {
        let toc = nav(&text, &path, pkg.deadline);
        if !toc.is_empty() {
            return toc;
        }
    }
    let ncx = opf
        .ncx
        .as_ref()
        .and_then(|id| opf.manifest.get(id))
        .or_else(|| {
            opf.order
                .iter()
                .filter_map(|id| opf.manifest.get(id))
                .find(|i| i.media == "application/x-dtbncx+xml")
        })
        .and_then(|i| i.path.clone());
    match ncx.and_then(|p| pkg.text(&p).map(|t| (p, t))) {
        Some((path, text)) => ncx_entries(&text, &path, pkg.deadline),
        None => Vec::new(),
    }
}

/// The entries of the `nav` of type `toc` of an EPUB 3 book.
fn nav(text: &str, path: &str, deadline: Instant) -> Vec<Entry> {
    let dir = base(path);
    let mut out = Vec::new();
    let mut xml = Xml::html(text, deadline);
    // Inside the toc nav: how deep the nav is open, and the lists in it.
    let mut in_toc = 0usize;
    let mut lists = 0usize;
    // The link being read: the element that opened it, where it leads, its
    // title so far.
    let mut link: Option<(String, Option<Target>, String)> = None;
    while let Ok(Some(ev)) = xml.next() {
        match ev {
            Ev::Start(n, a) if n == "nav" => {
                if in_toc > 0 {
                    in_toc += 1;
                } else if attr(&a, "type").is_some_and(|t| t.split_whitespace().any(|t| t == "toc"))
                {
                    in_toc = 1;
                }
            }
            Ev::End(n) if n == "nav" && in_toc > 0 => {
                in_toc -= 1;
                if in_toc == 0 && !out.is_empty() {
                    break;
                }
            }
            Ev::Start(n, _) if in_toc > 0 && n == "ol" => lists += 1,
            Ev::End(n) if in_toc > 0 && n == "ol" => lists = lists.saturating_sub(1),
            Ev::Start(n, a) if in_toc > 0 && link.is_none() && (n == "a" || n == "span") => {
                let target = attr(&a, "href").and_then(|h| inside(&dir, h));
                link = Some((n, target, String::new()));
            }
            Ev::Text(t) => {
                if let Some((_, _, title)) = &mut link
                    && title.len() < MAX_TEXT
                {
                    title.push_str(&t);
                }
            }
            Ev::End(n) if link.as_ref().is_some_and(|(open, ..)| *open == n) => {
                if let Some((_, target, title)) = link.take()
                    && out.len() < MAX_CHAPTERS
                {
                    out.push(Entry {
                        level: lists.max(1),
                        title: collapse(&title).trim().to_string(),
                        target,
                    });
                }
            }
            _ => {}
        }
    }
    out
}

/// The entries of an EPUB 2 `toc.ncx`.
fn ncx_entries(text: &str, path: &str, deadline: Instant) -> Vec<Entry> {
    let dir = base(path);
    let mut out = Vec::new();
    let mut xml = Xml::html(text, deadline);
    let mut depth = 0usize;
    let mut in_label = false;
    let mut title = String::new();
    // The entry of each open navPoint, until its `content` is read.
    let mut open: Vec<usize> = Vec::new();
    while let Ok(Some(ev)) = xml.next() {
        match ev {
            Ev::Start(n, _) if n == "navPoint" => {
                depth += 1;
                if out.len() < MAX_CHAPTERS {
                    out.push(Entry {
                        level: depth,
                        title: String::new(),
                        target: None,
                    });
                    open.push(out.len() - 1);
                } else {
                    open.push(usize::MAX);
                }
            }
            Ev::End(n) if n == "navPoint" => {
                depth = depth.saturating_sub(1);
                open.pop();
            }
            Ev::Start(n, _) if n == "navLabel" => {
                in_label = true;
                title.clear();
            }
            Ev::End(n) if n == "navLabel" => {
                in_label = false;
                if let Some(e) = open.last().and_then(|&i| out.get_mut(i))
                    && e.title.is_empty()
                {
                    e.title = collapse(&title).trim().to_string();
                }
            }
            Ev::Text(t) if in_label && title.len() < MAX_TEXT => title.push_str(&t),
            Ev::Start(n, a) | Ev::Empty(n, a) if n == "content" => {
                if let Some(e) = open.last().and_then(|&i| out.get_mut(i))
                    && e.target.is_none()
                {
                    e.target = attr(&a, "src").and_then(|s| inside(&dir, s));
                }
            }
            _ => {}
        }
    }
    out
}

/// A link to the place `place` of the document (see
/// [`markdown::from_events_linked`]).
fn start_link(d: &mut DocEvents, place: &str) {
    d.start(Tag::Link {
        link_type: LinkType::Inline,
        dest_url: format!("#{place}").into(),
        title: "".into(),
        id: "".into(),
    });
}

/// The contents as nested lists of links.
fn toc_list(d: &mut DocEvents, toc: &[Entry], w: &mut Writer) {
    let mut depth = 0usize;
    for e in toc {
        let level = e.level.clamp(1, depth + 1).min(8);
        while depth > level {
            d.end(TagEnd::Item);
            d.end_list(false);
            depth -= 1;
        }
        if depth == level {
            d.end(TagEnd::Item);
        }
        while depth < level {
            d.start_list(false);
            depth += 1;
        }
        d.start(Tag::Item);
        let title = if e.title.is_empty() {
            "(no title)"
        } else {
            &e.title
        };
        match &e.target {
            Some((part, frag)) => {
                let key = w.key(part, frag.as_deref());
                start_link(d, &key);
                d.text(title);
                d.end(TagEnd::Link);
            }
            None => d.text(title),
        }
    }
    while depth > 0 {
        d.end(TagEnd::Item);
        d.end_list(false);
        depth -= 1;
    }
}

/// The panel at the top: what the book says of itself and what it carries.
fn panel(d: &mut DocEvents, opf: &Opf, found: &Findings, encrypted: usize) {
    d.heading(1, "Ebook");
    for (label, value) in &opf.meta {
        d.field(label, value);
    }
    let strong = |d: &mut DocEvents, s: &str| {
        d.start(Tag::Paragraph);
        d.styled(s, Look::Strong);
        d.end(TagEnd::Paragraph);
    };
    let list = |d: &mut DocEvents, items: &[String], total: usize| {
        d.start_list(false);
        for s in items.iter().take(100) {
            d.start(Tag::Item);
            d.code(s);
            d.end(TagEnd::Item);
        }
        let shown = items.len().min(100);
        if total > shown {
            d.item(&format!("and {} more", total - shown));
        }
        d.end_list(false);
    };
    let mut quiet = true;
    if let Some(system) = &found.drm {
        quiet = false;
        strong(
            d,
            &format!(
                "Protected by DRM ({system}): {encrypted} parts are encrypted and cannot be read"
            ),
        );
    }
    if found.scripts_found() {
        quiet = false;
        strong(
            d,
            "Contains scripts (not run, but a reader that runs scripts would):",
        );
        d.start_list(false);
        if !found.scripts.is_empty() {
            let total: usize = found.scripts.iter().map(|(_, n)| n).sum();
            let names: Vec<&str> = found
                .scripts
                .iter()
                .take(10)
                .map(|(p, _)| p.as_str())
                .collect();
            let more = if found.scripts.len() > 10 {
                ", ..."
            } else {
                ""
            };
            d.item(&format!(
                "{total} script elements in {} chapters: {}{more}",
                found.scripts.len(),
                names.join(", ")
            ));
        }
        if found.handlers > 0 {
            d.item(&format!(
                "{} event attributes (onload, onclick...)",
                found.handlers
            ));
        }
        if found.js_links > 0 {
            d.item(&format!("{} javascript: links", found.js_links));
        }
        if !found.scripted.is_empty() {
            d.item(&format!(
                "the manifest says {} parts are scripted",
                found.scripted.len()
            ));
        }
        d.end_list(false);
    }
    if !found.fetches.is_empty() || !found.remote.is_empty() {
        quiet = false;
        strong(d, "Fetches from outside when a reader opens it:");
        if found.fetches.is_empty() {
            d.paragraph(&format!(
                "the manifest says {} parts load things from outside",
                found.remote.len()
            ));
        } else {
            list(d, &found.fetches, found.fetches.len());
        }
    }
    if found.links_out > 0 {
        d.field("Links to outside", &found.links_out.to_string());
    }
    let embedded: Vec<String> = [
        (found.forms, "form", "forms"),
        (found.frames, "frame", "frames"),
        (found.objects, "embedded object", "embedded objects"),
    ]
    .iter()
    .filter(|(n, ..)| *n > 0)
    .map(|(n, one, many)| format!("{n} {}", if *n == 1 { one } else { many }))
    .collect();
    if !embedded.is_empty() {
        quiet = false;
        d.field("Forms and frames", &embedded.join(", "));
    }
    if !found.outside.is_empty() {
        quiet = false;
        strong(d, "Points outside the package (not read):");
        list(d, &found.outside, found.outside.len());
    }
    if !found.missing.is_empty() {
        d.field("In the manifest but not in the file", "");
        list(d, &found.missing, found.missing.len());
    }
    if found.unlisted_count > 0 {
        d.field(
            "Parts not in the manifest",
            &found.unlisted_count.to_string(),
        );
        list(d, &found.unlisted, found.unlisted_count);
    }
    if found.fonts > 0 {
        d.field("Obfuscated fonts", &format!("{} (not DRM)", found.fonts));
    }
    if found.images > 0 {
        d.field("Images", &format!("{} (not shown)", found.images));
    }
    if quiet {
        d.paragraph("No scripts, nothing fetched from outside.");
    }
}

/// What an open element of a chapter stands for.
#[derive(Debug, Clone, PartialEq)]
enum Frame {
    /// A block with no events of its own (`p`, `div`, `section`): it ends
    /// the paragraph before it and the one in it.
    Block,
    Heading(u8, Option<Box<str>>),
    /// `ul`/`ol`; `true` if numbered.
    List(bool),
    /// `li`; `true` if a list had to be opened for it.
    Item(bool),
    /// `blockquote`; `true` for a footnote (`aside`), which a reader shows
    /// apart.
    Quote(bool),
    /// A look that starts and ends with the text: emphasis, bold, a link.
    Inline(Tag<'static>),
    /// A link outside: its address goes after its text.
    External(String),
    /// Read for nothing but its text (`span`).
    Plain,
    /// Not shown (`head`, `script`, `style`).
    Skip,
    Pre,
    Code,
    Table,
    Row,
    Cell,
}

/// An element open in a chapter, and whether its events were written (a
/// list inside a table is not).
struct Open {
    name: String,
    frame: Frame,
    wrote: bool,
}

/// The block being written: a paragraph, a heading, or the text of an item
/// with no paragraph of its own.
#[derive(Clone, Copy, PartialEq)]
enum Block {
    Paragraph,
    Heading(pulldown_cmark::HeadingLevel),
    Bare,
}

/// A table being read: its rows of cells.
#[derive(Default)]
struct TableAcc {
    rows: Vec<Vec<String>>,
}

/// The chapters as events, and what they carry.
struct Writer {
    d: DocEvents,
    open: Vec<Open>,
    /// Elements past [`MAX_FRAMES`], read as text only.
    deeper: usize,
    block: Option<Block>,
    /// The block has no text yet.
    fresh: bool,
    /// A space is owed before the next text of the block.
    space: bool,
    skip: usize,
    pre: Option<String>,
    code: Option<String>,
    tables: Vec<TableAcc>,
    found: Findings,
    // The chapter being read.
    part: String,
    place: Box<str>,
    placed: bool,
    seen_text: bool,
    start: usize,
    scripts: usize,
    /// Links by key, to the place they lead to.
    keys: Vec<Target>,
    key_of: HashMap<(String, Option<String>), usize>,
    /// The places of the headings with an id, by part and id, and of each
    /// chapter, by part.
    places: HashMap<(String, String), Box<str>>,
    chapter_places: HashMap<String, Box<str>>,
    next_place: usize,
    /// Words of the chapter being read.
    words: usize,
    /// Where the paragraph being written starts, and the first of its text:
    /// enough to tell a break between scenes (`* * *`).
    para_at: usize,
    para_text: String,
    /// The events of the footnotes (`aside`), which a reader shows apart;
    /// and where those still open start.
    notes_at: Vec<std::ops::Range<usize>>,
    open_notes: Vec<usize>,
    /// The text of the elements with an id, by part and id, for the notes;
    /// those being read, with the open element they end with.
    texts: HashMap<(String, String), String>,
    capturing: Vec<((String, String), usize, String)>,
    captured: usize,
    /// The links that call a note, by key; the text of the links open.
    note_keys: HashSet<usize>,
    link_texts: Vec<String>,
}

impl Writer {
    fn new() -> Self {
        Writer {
            d: DocEvents::new(MAX_EVENTS),
            open: Vec::new(),
            deeper: 0,
            block: None,
            fresh: false,
            space: false,
            skip: 0,
            pre: None,
            code: None,
            tables: Vec::new(),
            found: Findings::default(),
            part: String::new(),
            place: "".into(),
            placed: false,
            seen_text: false,
            start: 0,
            scripts: 0,
            keys: Vec::new(),
            key_of: HashMap::new(),
            places: HashMap::new(),
            chapter_places: HashMap::new(),
            next_place: 0,
            words: 0,
            para_at: 0,
            para_text: String::new(),
            notes_at: Vec::new(),
            open_notes: Vec::new(),
            texts: HashMap::new(),
            capturing: Vec::new(),
            captured: 0,
            note_keys: HashSet::new(),
            link_texts: Vec::new(),
        }
    }

    /// Starts keeping the text of an element with an id (a note, maybe):
    /// until the element ends, or, for an anchor in a paragraph
    /// (`<p><a id="n1">1</a> The note</p>`), the paragraph.
    fn capture(&mut self, name: &str, id: &str) {
        if self.texts.len() + self.capturing.len() >= MAX_TEXTS || self.captured >= MAX_CAPTURED {
            return;
        }
        let block = matches!(
            name,
            "p" | "div" | "aside" | "li" | "dd" | "dt" | "section" | "blockquote" | "td" | "footer"
        );
        let until = if block {
            self.open.len()
        } else {
            self.open
                .iter()
                .rposition(|o| {
                    matches!(
                        o.frame,
                        Frame::Block | Frame::Item(_) | Frame::Quote(_) | Frame::Heading(..)
                    )
                })
                .unwrap_or(self.open.len())
        };
        let key = (self.part.clone(), id.to_string());
        self.capturing.push((key, until, String::new()));
    }

    /// The texts kept of the elements that end with the open one at `depth`.
    fn end_captures(&mut self, depth: usize) {
        let mut i = 0;
        while i < self.capturing.len() {
            if self.capturing[i].1 >= depth {
                let (key, _, text) = self.capturing.remove(i);
                let text = collapse(&text).trim().to_string();
                if !text.is_empty() {
                    self.captured += text.len();
                    self.texts.entry(key).or_insert(text);
                }
            } else {
                i += 1;
            }
        }
    }

    /// The key of a link to `part#frag`, resolved to a place at the end.
    fn key(&mut self, part: &str, frag: Option<&str>) -> String {
        let k = (part.to_string(), frag.map(str::to_string));
        let n = match self.key_of.get(&k) {
            Some(&n) => n,
            None => {
                self.keys.push(k.clone());
                self.key_of.insert(k, self.keys.len() - 1);
                self.keys.len() - 1
            }
        };
        format!("k{n}")
    }

    fn begin_chapter(&mut self, part: &str, place: Box<str>) {
        self.part = part.to_string();
        self.chapter_places.insert(part.to_string(), place.clone());
        self.place = place;
        self.placed = false;
        self.seen_text = false;
        self.start = self.d.events.len();
        self.scripts = 0;
        self.words = 0;
    }

    /// A line in place of what could not be shown.
    fn note(&mut self, s: &str) {
        self.close_block();
        self.d.start(Tag::Paragraph);
        self.d.styled(s, Look::Emphasis);
        self.d.end(TagEnd::Paragraph);
        self.seen_text = true;
    }

    /// Closes what the chapter left open, and gives it its heading if it
    /// did not start with one.
    fn end_chapter(&mut self, title: &str) -> usize {
        while !self.open.is_empty() {
            self.pop();
        }
        self.end_captures(0);
        self.link_texts.clear();
        while !self.tables.is_empty() {
            self.end_table();
        }
        self.close_block();
        self.deeper = 0;
        self.skip = 0;
        self.pre = None;
        self.code = None;
        if self.scripts > 0 {
            self.found.scripts.push((self.part.clone(), self.scripts));
        }
        if !self.placed {
            let mut head = DocEvents::new(8);
            head.start(Tag::Heading {
                level: pulldown_cmark::HeadingLevel::H1,
                id: Some(CowStr::from(self.place.to_string())),
                classes: Vec::new(),
                attrs: Vec::new(),
            });
            head.text(title);
            head.end_heading(1);
            let start = self.start.min(self.d.events.len());
            let added = head.events.len();
            let tail = self.d.events.split_off(start);
            self.d.events.extend(head.events);
            self.d.events.extend(tail);
            for r in self.notes_at.iter_mut().filter(|r| r.start >= start) {
                *r = r.start + added..r.end + added;
            }
        }
        self.open_notes.clear();
        self.words
    }

    /// Reads one chapter; `Ok(true)` if the time limit stopped it.
    fn chapter(&mut self, text: &str, deadline: Instant) -> Result<bool, String> {
        let mut xml = Xml::html(text, deadline);
        loop {
            let ev = match xml.next() {
                Ok(Some(ev)) => ev,
                Ok(None) => break,
                Err(why) => {
                    // What was read stays; the rest is said to be lost.
                    while !self.open.is_empty() {
                        self.pop();
                    }
                    return Err(why);
                }
            };
            if self.d.cut {
                break;
            }
            match ev {
                Ev::Start(n, a) => {
                    let name = n.to_ascii_lowercase();
                    self.attributes(&name, &a);
                    if self.open.len() >= MAX_FRAMES {
                        self.deeper += 1;
                        continue;
                    }
                    self.start_element(name, &a);
                }
                Ev::Empty(n, a) => {
                    let name = n.to_ascii_lowercase();
                    self.attributes(&name, &a);
                    self.empty_element(&name, &a);
                }
                Ev::End(n) => {
                    let name = n.to_ascii_lowercase();
                    if self.deeper > 0 {
                        self.deeper -= 1;
                        continue;
                    }
                    if let Some(i) = self.open.iter().rposition(|o| o.name == name) {
                        while self.open.len() > i {
                            self.pop();
                        }
                    }
                }
                Ev::Text(t) => self.text(&t),
            }
        }
        Ok(xml.late)
    }

    /// What the attributes of any element carry: scripts, things fetched
    /// from outside, links out.
    fn attributes(&mut self, name: &str, a: &[(String, String)]) {
        for (k, v) in a {
            let key = k.trim_end_matches('\u{1}').to_ascii_lowercase();
            if key.starts_with("on") && key.len() > 2 {
                self.found.handlers += 1;
            }
            let address = matches!(key.as_str(), "src" | "href" | "data" | "poster" | "srcset");
            if !address {
                continue;
            }
            if v.trim_start()
                .to_ascii_lowercase()
                .starts_with("javascript:")
            {
                self.found.js_links += 1;
            } else if external(v) {
                if name == "a" && key == "href" {
                    self.found.links_out += 1;
                } else {
                    push_listed(&mut self.found.fetches, v.trim().to_string());
                }
            } else if key == "href"
                && name == "a"
                && !v.starts_with('#')
                && !v.contains(':')
                && inside(&base(&self.part), v).is_none()
            {
                push_listed(&mut self.found.outside, v.clone());
            }
        }
        match name {
            "form" => self.found.forms += 1,
            "iframe" | "frame" => self.found.frames += 1,
            "object" | "embed" | "applet" => self.found.objects += 1,
            "script" => self.scripts += 1,
            _ => {}
        }
    }

    fn in_table(&self) -> bool {
        !self.tables.is_empty()
    }

    fn start_element(&mut self, name: String, a: &[(String, String)]) {
        if self.skip > 0 {
            self.push_open(name, Frame::Skip, false);
            self.skip += 1;
            return;
        }
        if let Some(id) = attr(a, "id") {
            self.capture(&name, id);
        }
        let frame = match name.as_str() {
            "head" | "script" | "style" | "title" | "template" | "svg" | "math" => {
                if matches!(name.as_str(), "svg" | "math") {
                    let what = if name == "svg" { "image" } else { "formula" };
                    if name == "svg" {
                        self.found.images += 1;
                    }
                    self.inline_text(&format!("[{what}]"));
                }
                self.skip += 1;
                Frame::Skip
            }
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = name.as_bytes()[1] - b'0';
                self.close_block();
                let id = attr(a, "id").map(|i| {
                    let place = self.heading_place(i);
                    self.places
                        .insert((self.part.clone(), i.to_string()), place.clone());
                    place
                });
                Frame::Heading(level, id)
            }
            "ul" | "ol" | "dl" => {
                self.close_block();
                let numbered = name == "ol";
                if !self.in_table() {
                    self.d.start_list(numbered);
                }
                Frame::List(numbered)
            }
            "li" | "dt" | "dd" => {
                self.close_block();
                let listed = self.open.iter().rev().find_map(|o| match o.frame {
                    Frame::List(_) => Some(true),
                    Frame::Item(_) | Frame::Quote(_) | Frame::Table => Some(false),
                    _ => None,
                });
                let implicit = listed != Some(true);
                if !self.in_table() {
                    if implicit {
                        self.d.start_list(false);
                    }
                    self.d.start(Tag::Item);
                }
                Frame::Item(implicit)
            }
            "blockquote" => self.quote(false),
            "aside"
                if attr(a, "type").is_some_and(|t| {
                    t.split_whitespace()
                        .any(|t| matches!(t, "footnote" | "endnote" | "note" | "rearnote"))
                }) =>
            {
                self.quote(true)
            }
            "pre" => {
                self.close_block();
                if !self.in_table() && self.pre.is_none() {
                    self.pre = Some(String::new());
                    Frame::Pre
                } else {
                    Frame::Plain
                }
            }
            "code" | "kbd" | "samp" | "tt" if self.pre.is_none() && self.code.is_none() => {
                self.code = Some(String::new());
                Frame::Code
            }
            "table" => {
                if !self.in_table() {
                    self.close_block();
                }
                self.tables.push(TableAcc::default());
                Frame::Table
            }
            "tr" => {
                if let Some(t) = self.tables.last_mut()
                    && t.rows.len() < 10_000
                {
                    t.rows.push(Vec::new());
                }
                Frame::Row
            }
            "td" | "th" => {
                if let Some(row) = self.tables.last_mut().and_then(|t| t.rows.last_mut())
                    && row.len() < 256
                {
                    row.push(String::new());
                }
                Frame::Cell
            }
            "em" | "i" | "cite" | "dfn" | "var" => Frame::Inline(Tag::Emphasis),
            "strong" | "b" => Frame::Inline(Tag::Strong),
            "s" | "strike" | "del" => Frame::Inline(Tag::Strikethrough),
            "a" => match attr(a, "href") {
                Some(h) if external(h) => Frame::External(h.trim().to_string()),
                Some(h) if !h.contains(':') || h.starts_with('#') => {
                    match inside(&base(&self.part), h) {
                        Some((part, frag)) => {
                            let key = self.key(&part, frag.as_deref());
                            let call = attr(a, "type")
                                .is_some_and(|t| t.split_whitespace().any(|t| t == "noteref"))
                                || self.open.iter().any(|o| o.name == "sup")
                                || attr(a, "class").is_some_and(|c| {
                                    let c = c.to_ascii_lowercase();
                                    c.contains("noteref") || c.contains("footnote")
                                });
                            if call && let Some(n) = key_number(&key) {
                                self.note_keys.insert(n);
                            }
                            self.link_texts.push(String::new());
                            Frame::Inline(Tag::Link {
                                link_type: LinkType::Inline,
                                dest_url: format!("#{key}").into(),
                                title: "".into(),
                                id: "".into(),
                            })
                        }
                        None => Frame::Plain,
                    }
                }
                _ => Frame::Plain,
            },
            "p" | "div" | "section" | "article" | "header" | "footer" | "main" | "aside"
            | "figure" | "figcaption" | "center" | "address" | "hgroup" | "details" | "summary"
            | "nav" | "body" | "html" | "caption" => {
                self.close_block();
                Frame::Block
            }
            "video" | "audio" => {
                self.inline_text(&format!("[{name}]"));
                self.skip += 1;
                Frame::Skip
            }
            _ => Frame::Plain,
        };
        // A look opened inside a block written now starts now.
        let wrote = match &frame {
            Frame::Inline(tag) if self.block.is_some() && !self.in_table() => {
                self.d.start(tag.clone());
                true
            }
            Frame::Inline(_) => false,
            Frame::List(_) | Frame::Item(_) | Frame::Quote(_) => !self.in_table(),
            _ => true,
        };
        self.push_open(name, frame, wrote);
    }

    fn quote(&mut self, note: bool) -> Frame {
        self.close_block();
        if !self.in_table() {
            if note {
                self.open_notes.push(self.d.events.len());
            }
            self.d.start(Tag::BlockQuote(None));
        }
        Frame::Quote(note && !self.in_table())
    }

    fn push_open(&mut self, name: String, frame: Frame, wrote: bool) {
        self.open.push(Open { name, frame, wrote });
    }

    /// The place of a heading with an id: the chapter's, if it comes first.
    fn heading_place(&mut self, _id: &str) -> Box<str> {
        if !self.placed && !self.seen_text {
            return self.place.clone();
        }
        self.next_place += 1;
        format!("h{}", self.next_place).into()
    }

    fn empty_element(&mut self, name: &str, a: &[(String, String)]) {
        if self.skip > 0 {
            return;
        }
        match name {
            "br" => {
                if let Some(pre) = &mut self.pre {
                    pre.push('\n');
                } else if self.in_table() {
                    self.cell_text(" ");
                } else if self.block.is_some() {
                    self.d.push(Event::HardBreak);
                    self.space = false;
                    self.fresh = true;
                }
            }
            "hr" => {
                self.close_block();
                if !self.in_table() {
                    self.d.rule();
                }
            }
            "img" | "image" => {
                self.found.images += 1;
                let src = attr(a, "src")
                    .or_else(|| attr(a, "href"))
                    .unwrap_or("")
                    .chars()
                    .take(200)
                    .collect::<String>();
                self.inline_text(&format!("[image: {src}]"));
            }
            "p" | "div" => self.close_block(),
            _ => {}
        }
    }

    /// Closes the open element at the top.
    fn pop(&mut self) {
        let Some(o) = self.open.pop() else { return };
        self.end_captures(self.open.len());
        match o.frame {
            Frame::Skip => self.skip = self.skip.saturating_sub(1),
            Frame::Block => self.close_block(),
            Frame::Heading(..) => self.close_block(),
            Frame::List(numbered) => {
                self.close_block();
                if o.wrote {
                    self.d.end_list(numbered);
                }
            }
            Frame::Item(implicit) => {
                self.close_block();
                if o.wrote {
                    self.d.end(TagEnd::Item);
                    if implicit {
                        self.d.end_list(false);
                    }
                }
            }
            Frame::Quote(note) => {
                self.close_block();
                if o.wrote {
                    self.d.end(TagEnd::BlockQuote(None));
                }
                if note && let Some(at) = self.open_notes.pop() {
                    self.notes_at.push(at..self.d.events.len());
                }
            }
            Frame::Inline(tag) => {
                if o.wrote && self.block.is_some() && !self.in_table() {
                    self.d.end(tag.to_end());
                }
                if let Tag::Link { dest_url, .. } = &tag {
                    let text = self.link_texts.pop().unwrap_or_default();
                    if looks_like_a_call(&text)
                        && let Some(n) = key_number(dest_url)
                    {
                        self.note_keys.insert(n);
                    }
                }
            }
            Frame::External(url) => {
                if self.in_table() {
                    self.cell_text(&format!(" ({url})"));
                } else if self.block.is_some() {
                    self.flush_space();
                    self.d.text(" ");
                    self.d.code(&url);
                }
            }
            Frame::Plain | Frame::Row | Frame::Cell => {
                if matches!(o.frame, Frame::Cell) {
                    self.cell_text(" ");
                }
            }
            Frame::Pre => {
                if let Some(text) = self.pre.take() {
                    let text = text.trim_matches('\n');
                    if !text.trim().is_empty() {
                        self.close_block();
                        self.d.code_block("", text);
                        self.seen_text = true;
                    }
                }
            }
            Frame::Code => {
                if let Some(text) = self.code.take() {
                    let text = collapse(&text);
                    let text = text.trim();
                    if !text.is_empty() {
                        if self.in_table() {
                            self.cell_text(text);
                        } else {
                            self.ensure_block();
                            self.flush_space();
                            self.d.code(text);
                            self.fresh = false;
                            self.seen_text = true;
                        }
                    }
                }
            }
            Frame::Table => self.end_table(),
        }
    }

    fn end_table(&mut self) {
        let Some(t) = self.tables.pop() else { return };
        if let Some(outer) = self.tables.last_mut() {
            // A table in a table: its text goes into the cell, flat.
            let text: Vec<String> = t.rows.iter().map(|r| r.join(" | ")).collect();
            if let Some(cell) = outer.rows.last_mut().and_then(|r| r.last_mut())
                && cell.len() < MAX_TEXT
            {
                cell.push_str(&text.join(" / "));
            }
            return;
        }
        let mut rows: Vec<Vec<String>> = t
            .rows
            .into_iter()
            .map(|r| {
                r.into_iter()
                    .map(|c| collapse(&c).trim().to_string())
                    .collect()
            })
            .filter(|r: &Vec<String>| !r.is_empty())
            .collect();
        if rows.is_empty() {
            return;
        }
        let header = rows.remove(0);
        self.d.table(&header, &rows);
        self.seen_text = true;
    }

    fn cell_text(&mut self, s: &str) {
        if let Some(cell) = self
            .tables
            .last_mut()
            .and_then(|t| t.rows.last_mut())
            .and_then(|r| r.last_mut())
            && cell.len() < MAX_TEXT
        {
            cell.push_str(s);
        }
    }

    /// Text that is not from the book (`[image: ...]`), where the text is.
    fn inline_text(&mut self, s: &str) {
        if self.skip > 0 {
            return;
        }
        if self.in_table() {
            self.cell_text(s);
            return;
        }
        if let Some(code) = &mut self.code {
            code.push_str(s);
            return;
        }
        self.ensure_block();
        if !self.fresh {
            self.space = true;
        }
        self.flush_space();
        self.d.styled(s, Look::Emphasis);
        self.space = true;
        self.fresh = false;
        self.seen_text = true;
    }

    fn text(&mut self, t: &str) {
        if self.skip > 0 {
            return;
        }
        if let Some(pre) = &mut self.pre {
            if pre.len() < MAX_TEXT * 16 {
                pre.push_str(t);
            }
            return;
        }
        if let Some(code) = &mut self.code {
            if code.len() < MAX_TEXT {
                code.push_str(t);
            }
            return;
        }
        self.words += t.split_whitespace().count();
        for (_, _, kept) in &mut self.capturing {
            if kept.len() < MAX_NOTE {
                kept.push_str(t);
            }
        }
        if let Some(link) = self.link_texts.last_mut()
            && link.len() < 16
        {
            link.push_str(t);
        }
        if self.in_table() {
            self.cell_text(t);
            return;
        }
        let text = collapse(t);
        if text.trim().is_empty() {
            if self.block.is_some() && !self.fresh {
                self.space = true;
            }
            return;
        }
        self.ensure_block();
        let mut body = text.as_str();
        if let Some(rest) = body.strip_prefix(' ') {
            if !self.fresh {
                self.space = true;
            }
            body = rest;
        }
        let trailing = body.ends_with(' ');
        let body = body.trim_end_matches(' ');
        self.flush_space();
        self.d.text(body);
        if self.para_text.len() < 16 {
            self.para_text.push_str(body);
        }
        self.space = trailing;
        self.fresh = false;
        self.seen_text = true;
    }

    fn flush_space(&mut self) {
        if self.space && !self.fresh {
            self.d.text(" ");
        }
        self.space = false;
    }

    /// Opens the block the text goes in, if none is open: a heading inside
    /// a heading, the bare text of an item, else a paragraph; with the looks
    /// open around it.
    fn ensure_block(&mut self) {
        if self.block.is_some() {
            return;
        }
        let mut kind = Block::Paragraph;
        let mut id = None;
        for o in self.open.iter().rev() {
            match &o.frame {
                Frame::Heading(level, i) => {
                    kind = Block::Heading(heading_level(*level));
                    id = i.clone();
                    break;
                }
                Frame::Item(_) => {
                    kind = Block::Bare;
                    break;
                }
                Frame::Block | Frame::Quote(_) | Frame::List(_) => break,
                _ => {}
            }
        }
        if let Block::Heading(_) = kind
            && !self.placed
            && !self.seen_text
        {
            if id.is_none() {
                id = Some(self.place.clone());
            }
            if id.as_ref() == Some(&self.place) {
                self.placed = true;
            }
        }
        if kind == Block::Paragraph {
            self.para_at = self.d.events.len();
            self.para_text.clear();
        }
        match kind {
            Block::Paragraph => self.d.start(Tag::Paragraph),
            Block::Heading(level) => self.d.start(Tag::Heading {
                level,
                id: id.map(|i| CowStr::from(i.to_string())),
                classes: Vec::new(),
                attrs: Vec::new(),
            }),
            Block::Bare => {}
        }
        self.block = Some(kind);
        self.fresh = true;
        self.space = false;
        let looks: Vec<Tag<'static>> = self
            .open
            .iter_mut()
            .filter_map(|o| match &o.frame {
                Frame::Inline(tag) => {
                    o.wrote = true;
                    Some(tag.clone())
                }
                _ => None,
            })
            .collect();
        for tag in looks {
            self.d.start(tag);
        }
    }

    /// Ends the block being written, and the looks open in it.
    fn close_block(&mut self) {
        let Some(kind) = self.block.take() else {
            return;
        };
        let looks: Vec<TagEnd> = self
            .open
            .iter()
            .filter_map(|o| match &o.frame {
                Frame::Inline(tag) if o.wrote => Some(tag.to_end()),
                _ => None,
            })
            .collect();
        for end in looks.into_iter().rev() {
            self.d.end(end);
        }
        match kind {
            Block::Paragraph => {
                self.d.end(TagEnd::Paragraph);
                // A paragraph of `* * *` alone is a break between scenes.
                if is_scene(&self.para_text) && self.para_at <= self.d.events.len() {
                    self.d.events.truncate(self.para_at);
                    self.d.rule();
                }
                self.para_text.clear();
            }
            Block::Heading(level) => self.d.end(TagEnd::Heading(level)),
            Block::Bare => {}
        }
        self.space = false;
        self.fresh = false;
    }

    /// The place a link to `part#frag` leads to: the heading with that id,
    /// else the start of its chapter; `None` for a part that is not one.
    fn place_for(&self, part: &str, frag: Option<&str>) -> Option<Box<str>> {
        frag.and_then(|f| self.places.get(&(part.to_string(), f.to_string())))
            .or_else(|| self.chapter_places.get(part))
            .cloned()
    }

    /// The links of `events` led to their places (see [`place_for`]); one to
    /// a part that is not a chapter becomes its text. For the book to read
    /// (`reading`), a link that calls a note whose text is known leads to
    /// the note instead (`#n12`), its text set as a superscript; the texts of
    /// those notes come back by that place.
    ///
    /// [`place_for`]: Self::place_for
    fn resolve_links(&self, events: Events, reading: bool) -> (Events, HashMap<Box<str>, String>) {
        let mut notes = HashMap::new();
        let mut out = Vec::with_capacity(events.len());
        // For each link open: whether it was kept, and whether it calls a note.
        let mut open: Vec<(bool, bool)> = Vec::new();
        for (ev, range) in events {
            match ev {
                Event::Start(Tag::Link {
                    link_type,
                    dest_url,
                    title,
                    id,
                }) => {
                    let to = dest_url.strip_prefix('#').unwrap_or("");
                    let (place, note) = match key_number(to) {
                        Some(n) => {
                            let (part, frag) = self.keys.get(n).cloned().unwrap_or_default();
                            let text = frag
                                .as_ref()
                                .filter(|_| reading && self.note_keys.contains(&n))
                                .and_then(|f| self.texts.get(&(part.clone(), f.clone())));
                            match text {
                                Some(text) => {
                                    let place: Box<str> = format!("n{n}").into();
                                    notes.insert(place.clone(), text.clone());
                                    (Some(place), true)
                                }
                                None => (self.place_for(&part, frag.as_deref()), false),
                            }
                        }
                        None => (Some(to.into()), false),
                    };
                    match place {
                        Some(p) => {
                            open.push((true, note));
                            out.push((
                                Event::Start(Tag::Link {
                                    link_type,
                                    dest_url: format!("#{p}").into(),
                                    title,
                                    id,
                                }),
                                range,
                            ));
                        }
                        None => open.push((false, false)),
                    }
                }
                Event::End(TagEnd::Link) => {
                    if open.pop().is_none_or(|(kept, _)| kept) {
                        out.push((Event::End(TagEnd::Link), range));
                    }
                }
                Event::Text(t) if open.last().is_some_and(|&(_, note)| note) => {
                    out.push((Event::Text(superscript(&t).into()), range));
                }
                ev => out.push((ev, range)),
            }
        }
        (out, notes)
    }
}

/// The number of a link key (`k12`, `#k12`).
fn key_number(key: &str) -> Option<usize> {
    key.trim_start_matches('#').strip_prefix('k')?.parse().ok()
}

/// The text of a link that calls a note: `1`, `[12]`, `*`, `†`, `a`.
fn looks_like_a_call(text: &str) -> bool {
    let t = text
        .trim()
        .trim_start_matches(['[', '('])
        .trim_end_matches([']', ')']);
    let n = t.chars().count();
    (1..=3).contains(&n)
        && (t.chars().all(|c| c.is_ascii_digit())
            || t.chars().all(|c| matches!(c, '*' | '†' | '‡' | '§'))
            || (n == 1 && t.chars().all(|c| c.is_ascii_lowercase())))
}

/// A paragraph that is only a break between scenes: `* * *`, `***`, `#`,
/// `⁂`, `~`.
fn is_scene(text: &str) -> bool {
    let t = text.trim();
    !t.is_empty()
        && t.chars().count() <= 15
        && t.chars().all(|c| {
            c.is_whitespace()
                || matches!(
                    c,
                    '*' | '#' | '~' | '⁂' | '•' | '·' | '◇' | '◆' | '❖' | '✻' | '✱' | '❦'
                )
        })
}

/// A note's call as it is set in a book: `1` as `¹`, `[12]` as `¹²`.
fn superscript(text: &str) -> String {
    const DIGITS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    let t = text
        .trim()
        .trim_start_matches(['[', '('])
        .trim_end_matches([']', ')']);
    if t.chars().all(|c| c.is_ascii_digit()) {
        t.chars()
            .map(|c| DIGITS[(c as u8 - b'0') as usize])
            .collect()
    } else {
        t.to_string()
    }
}

fn heading_level(level: u8) -> pulldown_cmark::HeadingLevel {
    use pulldown_cmark::HeadingLevel::*;
    match level {
        0 | 1 => H1,
        2 => H2,
        3 => H3,
        4 => H4,
        5 => H5,
        _ => H6,
    }
}

#[cfg(test)]
pub mod test_books {
    //! Small EPUBs for the tests, made here: nothing is downloaded.
    pub use crate::viewer::office::test_docs::zip;

    pub const CONTAINER: &str = r#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles>
</container>"#;

    pub const XHTML11: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.1//EN" "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd">"#;

    /// A package file with `manifest` and `spine` (the inside of each).
    pub fn opf(version: &str, manifest: &str, spine: &str) -> String {
        format!(
            r#"<?xml version="1.0"?>
<package xmlns="http://www.idpf.org/2007/opf" version="{version}" unique-identifier="uid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>The Test Book</dc:title>
    <dc:creator>Ann Author</dc:creator>
    <dc:creator>Bo Writer</dc:creator>
    <dc:language>en</dc:language>
    <dc:identifier id="uid">urn:uuid:1234</dc:identifier>
    <meta name="generator" content="Tool 1.0"/>
  </metadata>
  <manifest>{manifest}</manifest>
  <spine toc="ncx">{spine}</spine>
</package>"#
        )
    }

    pub fn page(body: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head><title>t</title><style>p {{ margin: 0 }}</style></head>
<body>{body}</body></html>"#
        )
    }

    /// An EPUB 3 book: a nav, two chapters and a style sheet.
    pub fn book3() -> Vec<u8> {
        let manifest = r#"
    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
    <item id="c1" href="Text/one.xhtml" media-type="application/xhtml+xml"/>
    <item id="c2" href="Text/two%20b.xhtml" media-type="application/xhtml+xml"/>
    <item id="css" href="style.css" media-type="text/css"/>"#;
        let spine = r#"<itemref idref="c1"/><itemref idref="c2"/><itemref idref="c1"/>"#;
        let nav = page(
            r#"<nav epub:type="toc"><ol>
  <li><a href="Text/one.xhtml">One</a></li>
  <li><a href="Text/two%20b.xhtml"><span>Two</span></a>
    <ol><li><a href="Text/two%20b.xhtml#later">Later on</a></li></ol></li>
</ol></nav>"#,
        );
        let one = page(
            r#"<h1 id="top">Chapter One</h1>
<p>It was <em>dark</em> and&nbsp;<strong>stormy</strong>&mdash;truly.</p>
<p>Plain   text
   over lines, then <a href="two%20b.xhtml#later">a jump</a> and
   <a href="https://example.com/x">a site</a>.</p>
<ul><li>first</li><li>second</li></ul>
<table><tr><th>Name</th><th>Age</th></tr><tr><td>Ann</td><td>30</td></tr></table>
<p><img src="../Images/pic.jpg" alt="pic"/>After the image<br/>a new line.</p>
<pre>  code
  kept</pre>"#,
        );
        let two = page(
            r#"<p>Opening words with no heading.</p>
<h2 id="later">Later</h2><p>The end.</p>"#,
        );
        zip(&[
            ("mimetype", "application/epub+zip"),
            ("META-INF/container.xml", CONTAINER),
            ("OEBPS/content.opf", &opf("3.0", manifest, spine)),
            ("OEBPS/nav.xhtml", &nav),
            ("OEBPS/Text/one.xhtml", &one),
            ("OEBPS/Text/two b.xhtml", &two),
            ("OEBPS/style.css", "p { color: black }"),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::test_books::*;
    use super::*;
    use std::io::Cursor;
    use std::time::Duration;

    const LIMITS: Limits = Limits {
        part: 1 << 20,
        total: 1 << 22,
        time: Duration::from_secs(5),
        budget: 1 << 24,
    };

    fn read(bytes: Vec<u8>) -> Book {
        render(Cursor::new(bytes), LIMITS, 100).unwrap()
    }

    fn lines(doc: &Doc) -> Vec<String> {
        (0..doc.line_count())
            .map(|i| doc.line(i).to_string())
            .collect()
    }

    fn text(bytes: Vec<u8>) -> String {
        lines(&read(bytes).report).join("\n")
    }

    #[test]
    fn an_epub_3_reads_as_a_document_with_its_contents() {
        let book = read(book3());
        let all = lines(&book.report);
        let text = all.join("\n");
        for want in [
            "Title: The Test Book",
            "Authors: Ann Author; Bo Writer",
            "Identifier: urn:uuid:1234",
            "Made with: Tool 1.0",
            "Format: EPUB 3.0",
            "No scripts, nothing fetched from outside.",
            "Chapter One",
            "stormy—truly.",
            "Plain text over lines, then a jump and a site https://example.com/x.",
            "first",
            "second",
            "Ann",
            "After the image",
            "a new line.",
            "  code",
            "Opening words with no heading.",
            "Later",
            "The end.",
            "Links to outside: 1",
            "Images: 1 (not shown)",
        ] {
            assert!(all.iter().any(|l| l.contains(want)), "{want:?} in\n{text}");
        }
        assert!(text.contains("[image: ../Images/pic.jpg]"), "{text}");
        assert!(!text.contains("margin"), "the style is not text");
        // Each chapter once, in the spine's order; the second starts at a
        // heading made from the contents, since it has none of its own first.
        let names: Vec<(&str, &str)> = book
            .chapters
            .iter()
            .map(|c| (&*c.place, c.title.as_str()))
            .collect();
        assert_eq!(names, [("c1", "One"), ("c2", "Two")]);
        assert!(book.chapters[0].words > book.chapters[1].words);
        let doc = &book.report;
        let c1 = doc.place("c1").unwrap();
        let c2 = doc.place("c2").unwrap();
        assert_eq!(doc.line(c1), "Chapter One");
        assert_eq!(doc.line(c2), "Two");
        assert!(c1 < c2);
        // The contents are links to the chapters and the heading inside.
        let to: Vec<&str> = doc.jumps().iter().map(|j| &*j.to).collect();
        let later = doc
            .jumps()
            .iter()
            .find(|j| doc.line(j.line).contains("Later on"))
            .unwrap();
        let later_line = doc.place(&later.to).unwrap();
        assert_eq!(doc.line(later_line), "Later");
        assert!(to.starts_with(&["c1", "c2"]), "{to:?}");
        // The jump in the text leads to the same heading.
        let jump = doc
            .jumps()
            .iter()
            .find(|j| doc.line(j.line).contains("a jump"))
            .unwrap();
        assert_eq!(jump.to, later.to);
    }

    #[test]
    fn the_book_to_read_has_a_title_page_and_its_notes_apart() {
        let manifest = r#"<item id="c1" href="one.xhtml" media-type="application/xhtml+xml"/>
    <item id="c2" href="notes.xhtml" media-type="application/xhtml+xml"/>"#;
        let spine = r#"<itemref idref="c1"/><itemref idref="c2"/>"#;
        let one = page(
            r##"<h1>Start</h1><p>A call<a epub:type="noteref" href="#fn1">1</a> and one
to the end<sup><a href="notes.xhtml#e2">[2]</a></sup>, and <a href="notes.xhtml">a link</a>.</p>
<p>* * *</p><p>After the break.</p>
<aside epub:type="footnote" id="fn1"><p>The aside note.</p></aside>"##,
        );
        let notes =
            page(r#"<h1>Notes</h1><p id="e2"><a href="one.xhtml">2</a>. The end note.</p>"#);
        let book = read(zip(&[
            ("mimetype", "application/epub+zip"),
            ("META-INF/container.xml", CONTAINER),
            ("OEBPS/content.opf", &opf("3.0", manifest, spine)),
            ("OEBPS/one.xhtml", &one),
            ("OEBPS/notes.xhtml", &notes),
        ]));
        let reading = lines(&book.reading);
        let text = reading.join("\n");
        // The title page: title, authors, then how long the book is.
        assert_eq!(reading[0], "The Test Book");
        assert_eq!(reading[1], "Ann Author; Bo Writer");
        assert!(text.contains("2 chapters,"), "{text}");
        assert!(book.warning.is_none());
        // The calls as superscripts; the aside is not in the text.
        assert!(
            text.contains("A call¹ and one to the end², and a link."),
            "{text}"
        );
        assert!(!text.contains("The aside note."), "{text}");
        assert!(text.contains("After the break."));
        assert!(!text.contains("* * *"), "the break is a rule");
        // The notes by the place their calls lead to.
        let doc = &book.reading;
        let calls: Vec<String> = doc
            .jumps()
            .iter()
            .filter_map(|j| book.notes.get(&j.to).cloned())
            .collect();
        assert_eq!(calls, ["The aside note.", "2. The end note."]);
        // A link that is not a note still leads to its chapter, in both.
        let link = doc
            .jumps()
            .iter()
            .find(|j| !book.notes.contains_key(&j.to))
            .unwrap();
        assert_eq!(&*link.to, "c2");
        // The report keeps the aside and the calls as they were.
        let report = lines(&book.report).join("\n");
        assert!(report.contains("The aside note."));
        assert!(report.contains("A call1 and one to the end[2]"), "{report}");
        // The contents lead to the chapters.
        assert!(book.toc.is_empty(), "no nav, no ncx");
        assert_eq!(doc.line(doc.place("c2").unwrap()), "Notes");
    }

    #[test]
    fn a_book_with_scripts_says_so_on_its_title_page() {
        let manifest = r#"<item id="c1" href="one.xhtml" media-type="application/xhtml+xml"/>"#;
        let one = page(r#"<p onload="x()">Text <img src="https://t.example/p.gif"/></p>"#);
        let book = read(zip(&[
            ("mimetype", "application/epub+zip"),
            ("META-INF/container.xml", CONTAINER),
            (
                "OEBPS/content.opf",
                &opf("3.0", manifest, r#"<itemref idref="c1"/>"#),
            ),
            ("OEBPS/one.xhtml", &one),
        ]));
        assert_eq!(
            book.warning.as_deref(),
            Some("This book has scripts, 1 address fetched from outside")
        );
        let text = lines(&book.reading).join("\n");
        assert!(
            text.contains(
                "⚠ This book has scripts, 1 address fetched from outside (I shows the report)"
            ),
            "{text}"
        );
    }

    #[test]
    fn calls_breaks_and_superscripts() {
        for call in ["1", "12", "[3]", "(4)", "*", "†", "a"] {
            assert!(looks_like_a_call(call), "{call}");
        }
        for not in ["", "1234", "see", "Chapter 2", "ab"] {
            assert!(!looks_like_a_call(not), "{not}");
        }
        for scene in ["* * *", "***", "#", "⁂", " ~ ~ "] {
            assert!(is_scene(scene), "{scene}");
        }
        for not in ["", "a * b", "- - -", "1."] {
            assert!(!is_scene(not), "{not}");
        }
        assert_eq!(superscript("[12]"), "¹²");
        assert_eq!(superscript("*"), "*");
        assert_eq!(group(1_234_567), "1,234,567");
        assert_eq!(group(999), "999");
    }

    #[test]
    fn an_epub_2_takes_its_contents_from_the_ncx() {
        let manifest = r#"
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
    <item id="c1" href="one.xhtml" media-type="application/xhtml+xml"/>"#;
        let ncx = r#"<?xml version="1.0"?>
<!DOCTYPE ncx PUBLIC "-//NISO//DTD ncx 2005-1//EN" "http://www.daisy.org/z3986/2005/ncx-2005-1.dtd">
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1"><navMap>
  <navPoint id="p1"><navLabel><text>The First</text></navLabel><content src="one.xhtml"/>
    <navPoint id="p2"><navLabel><text>Inside</text></navLabel><content src="one.xhtml#in"/></navPoint>
  </navPoint>
</navMap></ncx>"#;
        let one = format!(
            "{XHTML11}<html xmlns=\"http://www.w3.org/1999/xhtml\"><body><p>Before.</p><h2 id=\"in\">In &eacute;t&eacute;</h2></body></html>"
        );
        let book = read(zip(&[
            ("mimetype", "application/epub+zip"),
            ("META-INF/container.xml", CONTAINER),
            (
                "OEBPS/content.opf",
                &opf("2.0", manifest, r#"<itemref idref="c1"/>"#),
            ),
            ("OEBPS/toc.ncx", ncx),
            ("OEBPS/one.xhtml", &one),
        ]));
        let doc = &book.report;
        let all = lines(doc);
        assert!(all.iter().any(|l| l == "In été"), "{all:?}");
        assert_eq!(doc.line(doc.place("c1").unwrap()), "The First");
        let inside = doc
            .jumps()
            .iter()
            .find(|j| doc.line(j.line).contains("Inside"))
            .unwrap();
        assert_eq!(doc.line(doc.place(&inside.to).unwrap()), "In été");
    }

    #[test]
    fn what_a_hostile_book_carries_is_said_and_nothing_is_read_outside() {
        let manifest = r#"
    <item id="c1" href="one.xhtml" media-type="application/xhtml+xml" properties="scripted"/>
    <item id="c2" href="bomb.xhtml" media-type="application/xhtml+xml"/>
    <item id="out" href="../../etc/passwd" media-type="application/xhtml+xml"/>
    <item id="css" href="s.css" media-type="text/css"/>
    <item id="gone" href="gone.xhtml" media-type="application/xhtml+xml"/>"#;
        let spine = r#"<itemref idref="c1"/><itemref idref="c2"/><itemref idref="out"/><itemref idref="nope"/>"#;
        let one = page(
            r#"<h1 onclick="x()">Hi</h1><script>steal()</script>
<p><img src="https://tracker.example/p.gif"/><a href="javascript:alert(1)">click</a></p>
<form><input/></form><iframe src="http://frame.example/"></iframe>"#,
        );
        let bomb = r#"<?xml version="1.0"?>
<!DOCTYPE html [<!ENTITY a "aaaaaaaaaa"><!ENTITY b "&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;">]>
<html><body><p>&b;</p></body></html>"#;
        let text = text(zip(&[
            ("mimetype", "application/epub+zip"),
            ("META-INF/container.xml", CONTAINER),
            ("OEBPS/content.opf", &opf("3.0", manifest, spine)),
            ("OEBPS/one.xhtml", &one),
            ("OEBPS/bomb.xhtml", bomb),
            (
                "OEBPS/s.css",
                "@import url('https://fonts.example/f.css'); p { background: url(//cdn.example/b.png) }",
            ),
            ("OEBPS/extra.bin", "x"),
        ]));
        for want in [
            "Contains scripts",
            "1 script elements in 1 chapters: OEBPS/one.xhtml",
            "1 event attributes",
            "1 javascript: links",
            "the manifest says 1 parts are scripted",
            "Fetches from outside",
            "https://tracker.example/p.gif",
            "http://frame.example/",
            "https://fonts.example/f.css",
            "//cdn.example/b.png",
            "Forms and frames: 1 form, 1 frame",
            "Points outside the package",
            "../../etc/passwd",
            "OEBPS/gone.xhtml",
            "spine item nope",
            "Parts not in the manifest: 1",
            "OEBPS/extra.bin",
            "[chapter 2: cannot be read: A document with a DTD is refused]",
            "Hi",
        ] {
            assert!(text.contains(want), "{want:?} in\n{text}");
        }
        assert!(!text.contains("steal"), "a script is not text");
        assert!(!text.contains("aaaa"), "no entity of the book is expanded");
        assert!(!text.contains("No scripts"));
    }

    #[test]
    fn drm_is_named_and_obfuscated_fonts_are_not_drm() {
        let manifest = r#"
    <item id="c1" href="one.xhtml" media-type="application/xhtml+xml"/>
    <item id="c2" href="two.xhtml" media-type="application/xhtml+xml"/>"#;
        let spine = r#"<itemref idref="c1"/><itemref idref="c2"/>"#;
        let encryption = r#"<encryption xmlns="urn:oasis:names:tc:opendocument:xmlns:container" xmlns:enc="http://www.w3.org/2001/04/xmlenc#">
  <enc:EncryptedData><enc:EncryptionMethod Algorithm="http://www.idpf.org/2008/embedding"/>
    <enc:CipherData><enc:CipherReference URI="OEBPS/font.otf"/></enc:CipherData></enc:EncryptedData>
  <enc:EncryptedData><enc:EncryptionMethod Algorithm="http://www.w3.org/2001/04/xmlenc#aes128-cbc"/>
    <enc:CipherData><enc:CipherReference URI="OEBPS/two.xhtml"/></enc:CipherData></enc:EncryptedData>
</encryption>"#;
        let text = text(zip(&[
            ("mimetype", "application/epub+zip"),
            ("META-INF/container.xml", CONTAINER),
            ("META-INF/encryption.xml", encryption),
            ("META-INF/rights.xml", "<rights/>"),
            ("OEBPS/content.opf", &opf("3.0", manifest, spine)),
            ("OEBPS/one.xhtml", &page("<p>Free text.</p>")),
            ("OEBPS/two.xhtml", "\u{1}\u{2}garbage"),
        ]));
        assert!(
            text.contains("Protected by DRM (Adobe ADEPT): 1 parts are encrypted"),
            "{text}"
        );
        assert!(text.contains("Obfuscated fonts: 1 (not DRM)"), "{text}");
        assert!(text.contains("Free text."));
        assert!(
            text.contains("[chapter 2: encrypted, cannot be read]"),
            "{text}"
        );
        assert!(!text.contains("garbage"));
    }

    #[test]
    fn broken_xhtml_is_read_as_far_as_it_goes() {
        let manifest = r#"<item id="c1" href="one.xhtml" media-type="application/xhtml+xml"/>
    <item id="c2" href="two.xhtml" media-type="application/xhtml+xml"/>"#;
        let spine = r#"<itemref idref="c1"/><itemref idref="c2"/>"#;
        // Tags never closed, closed in the wrong order, a <br> with no slash,
        // a closing tag of nothing open, and deep nesting.
        let deep = "<div>".repeat(200);
        let one = format!(
            "<html><body><p>Open <b>bold <i>both</p></b><p>line<br>next</i></span>{deep}deep<p>last"
        );
        let two = "<html><body><p>Fine & dandy</p></body></html>";
        let text = text(zip(&[
            ("mimetype", "application/epub+zip"),
            ("META-INF/container.xml", CONTAINER),
            ("OEBPS/content.opf", &opf("3.0", manifest, spine)),
            ("OEBPS/one.xhtml", &one),
            ("OEBPS/two.xhtml", two),
        ]));
        for want in ["Open bold both", "line", "next", "deep", "last"] {
            assert!(text.contains(want), "{want:?} in\n{text}");
        }
        assert!(text.contains("[chapter 2: cannot be read:"), "{text}");
    }

    #[test]
    fn a_book_over_the_limits_says_it_was_cut() {
        let manifest = r#"<item id="c1" href="one.xhtml" media-type="application/xhtml+xml"/>"#;
        let big = page(&"<p>word word word word word word word word</p>\n".repeat(40_000));
        let bytes = zip(&[
            ("mimetype", "application/epub+zip"),
            ("META-INF/container.xml", CONTAINER),
            (
                "OEBPS/content.opf",
                &opf("3.0", manifest, r#"<itemref idref="c1"/>"#),
            ),
            ("OEBPS/one.xhtml", &big),
        ]);
        let limits = Limits {
            part: 64 << 10,
            ..LIMITS
        };
        let book = render(Cursor::new(bytes), limits, 100).unwrap();
        assert!(book.report.cut() && book.reading.cut());
        let text = lines(&book.report).join("\n");
        assert!(
            text.contains("Only the start of the book was read"),
            "{text}"
        );
        assert!(text.contains("word word"));
    }

    #[test]
    fn what_is_not_an_epub_is_refused() {
        assert!(render(Cursor::new(b"not a zip".to_vec()), LIMITS, 80).is_err());
        let no_container = zip(&[("mimetype", "application/epub+zip")]);
        let err = render(Cursor::new(no_container), LIMITS, 80).err().unwrap();
        assert!(err.contains("container.xml"), "{err}");
        assert!(is_epub(std::path::Path::new("a/Book.EPUB")));
        assert!(!is_epub(std::path::Path::new("book.zip")));
    }

    #[test]
    fn paths_and_addresses() {
        assert_eq!(percent_decode("two%20b.xhtml"), "two b.xhtml");
        assert_eq!(percent_decode("100%"), "100%");
        assert_eq!(percent_decode("%zz"), "%zz");
        assert_eq!(
            inside(&base("OEBPS/Text/a.xhtml"), "#n1"),
            Some(("OEBPS/Text/a.xhtml".into(), Some("n1".into())))
        );
        assert_eq!(
            inside(&base("OEBPS/Text/a.xhtml"), "../b.xhtml#x"),
            Some(("OEBPS/b.xhtml".into(), Some("x".into())))
        );
        assert_eq!(inside(&base("OEBPS/a.xhtml"), "../../../x"), None);
        assert!(external("HTTPS://a") && external("//cdn") && !external("a.xhtml"));
        assert_eq!(collapse("  a \n\t b  "), " a b ");
        assert_eq!(
            css_urls("a{b:url( \"http://x/y.png\" )} @import 'https://z/f.css'; c{url(local.png)}"),
            ["http://x/y.png", "https://z/f.css"]
        );
    }
}
