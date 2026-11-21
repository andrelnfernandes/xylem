//! Office documents (docs/plans/viewer-office.md): Word (`.docx`), OpenDocument
//! text (`.odt`), PowerPoint (`.pptx`) and Impress (`.odp`) shown as a
//! document, with what one can hide from whoever opens it at the top: who
//! wrote it, macros, links to other places (a remote template is how
//! Follina, CVE-2022-30190, came in), embedded objects, tracked changes and
//! hidden text.
//!
//! The package is hostile: each part is decompressed only up to a limit, the
//! XML is read with no DTD and no entities of its own, the nesting has a
//! limit, and the text goes into the document as text, never as Markdown.

use std::collections::HashMap;
use std::io::{Read, Seek};
use std::time::Instant;

use super::doc_events::{DocEvents, Look};
use super::markdown::{self, Doc};
pub use super::package::Limits;
use super::package::{Ev, Package, Xml, attr, prefixed, resolve};

/// Most events of the document.
const MAX_EVENTS: usize = 4_000_000;
/// Slides read, at most.
const MAX_SLIDES: usize = 10_000;

/// The kinds of document, by the name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Word,
    Text,
    Slides,
    Impress,
}

pub fn kind(path: &std::path::Path) -> Option<Kind> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    Some(match ext.as_str() {
        "docx" | "docm" | "dotx" | "dotm" => Kind::Word,
        "odt" | "ott" => Kind::Text,
        "pptx" | "pptm" | "potx" | "potm" | "ppsx" => Kind::Slides,
        "odp" | "otp" => Kind::Impress,
        _ => return None,
    })
}

/// `w:b`, `w:i`... are on unless their value says off.
fn on(attrs: &[(String, String)]) -> bool {
    !matches!(attr(attrs, "val"), Some("0" | "false" | "off" | "none"))
}

/// A relationship of a part: its type (the end of the URI), target, and
/// whether it points outside the package.
#[derive(Debug, Clone)]
struct Rel {
    kind: String,
    target: String,
    external: bool,
}

/// The relationships of `part` (from `<dir>/_rels/<name>.rels`), by id. An
/// internal target is resolved from the part's folder; one that climbs out of
/// the package is dropped.
fn rels<R: Read + Seek>(pkg: &mut Package<R>, part: &str) -> HashMap<String, Rel> {
    let (dir, name) = part.rsplit_once('/').unwrap_or(("", part));
    let path = if dir.is_empty() {
        format!("_rels/{name}.rels")
    } else {
        format!("{dir}/_rels/{name}.rels")
    };
    let Some(text) = pkg.text(&path) else {
        return HashMap::new();
    };
    let mut out = HashMap::new();
    let mut xml = Xml::new(&text, pkg.deadline);
    while let Ok(Some(ev)) = xml.next() {
        if let Ev::Start(n, a) | Ev::Empty(n, a) = ev
            && n == "Relationship"
            && let (Some(id), Some(target)) = (attr(&a, "Id"), attr(&a, "Target"))
        {
            let external = attr(&a, "TargetMode") == Some("External");
            let kind = attr(&a, "Type")
                .and_then(|t| t.rsplit('/').next())
                .unwrap_or("")
                .to_string();
            let target = if external {
                Some(target.to_string())
            } else {
                resolve(dir, target)
            };
            if let Some(target) = target {
                out.insert(
                    id.to_string(),
                    Rel {
                        kind,
                        target,
                        external,
                    },
                );
            }
        }
        if out.len() >= 100_000 {
            break;
        }
    }
    out
}

/// The metadata worth knowing: who wrote it, who saved it last, when.
fn metadata<R: Read + Seek>(pkg: &mut Package<R>, odf: bool) -> Vec<(String, String)> {
    let parts: &[&str] = if odf {
        &["meta.xml"]
    } else {
        &["docProps/core.xml", "docProps/app.xml"]
    };
    let wanted: &[(&str, &str)] = &[
        ("title", "Title"),
        ("subject", "Subject"),
        ("creator", "Author"),
        ("initial-creator", "Author"),
        ("lastModifiedBy", "Last saved by"),
        ("created", "Created"),
        ("creation-date", "Created"),
        ("modified", "Modified"),
        ("date", "Modified"),
        ("revision", "Revision"),
        ("editing-cycles", "Revision"),
        ("Application", "Application"),
        ("generator", "Application"),
        ("Company", "Company"),
        ("Manager", "Manager"),
    ];
    let mut out: Vec<(String, String)> = Vec::new();
    for part in parts {
        let Some(text) = pkg.text(part) else { continue };
        let mut xml = Xml::new(&text, pkg.deadline);
        let mut current: Option<&str> = None;
        let mut value = String::new();
        while let Ok(Some(ev)) = xml.next() {
            match ev {
                Ev::Start(n, _) => {
                    current = wanted.iter().find(|(k, _)| *k == n).map(|(_, l)| *l);
                    value.clear();
                }
                Ev::Text(t) if current.is_some() && value.len() < 4096 => value.push_str(&t),
                Ev::End(_) => {
                    if let Some(label) = current.take() {
                        let v = value.trim();
                        if !v.is_empty() && !out.iter().any(|(l, _)| l == label) {
                            out.push((label.to_string(), v.to_string()));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    out
}

/// A paragraph being read.
#[derive(Default)]
struct Para {
    heading: Option<u8>,
    /// An item of a list: numbered or not.
    item: Option<bool>,
    runs: Vec<(String, Look)>,
}

/// A table being read: its rows of cells.
#[derive(Default)]
struct TableAcc {
    rows: Vec<Vec<String>>,
}

/// The body of a document as it is read: paragraphs, lists and tables, and
/// what is counted for the header.
struct Flow {
    d: DocEvents,
    para: Option<Para>,
    list: Option<bool>,
    tables: Vec<TableAcc>,
    notes: Vec<(String, String)>,
    revisions: usize,
    hidden: usize,
    images: usize,
}

impl Flow {
    fn new() -> Self {
        Flow {
            d: DocEvents::new(MAX_EVENTS),
            para: None,
            list: None,
            tables: Vec::new(),
            notes: Vec::new(),
            revisions: 0,
            hidden: 0,
            images: 0,
        }
    }

    fn start_para(&mut self) {
        self.end_para();
        self.para = Some(Para::default());
    }

    fn para(&mut self) -> &mut Para {
        self.para.get_or_insert_with(Para::default)
    }

    fn push(&mut self, text: &str, look: Look) {
        if text.is_empty() {
            return;
        }
        if let Some(t) = self.tables.last_mut() {
            if let Some(cell) = t.rows.last_mut().and_then(|r| r.last_mut())
                && cell.len() < 64 * 1024
            {
                cell.push_str(text);
            }
            return;
        }
        let runs = &mut self.para().runs;
        match runs.last_mut() {
            Some((s, l)) if *l == look => s.push_str(text),
            _ => runs.push((text.to_string(), look)),
        }
    }

    /// A mark of what was hidden or changed, then its text.
    fn marked(&mut self, mark: &str, text: &str, look: Look) {
        self.push(&format!("[{mark}: "), Look::Emphasis);
        self.push(text, look);
        self.push("]", Look::Emphasis);
    }

    fn close_list(&mut self) {
        if let Some(numbered) = self.list.take() {
            self.d.end_list(numbered);
        }
    }

    fn end_para(&mut self) {
        if !self.tables.is_empty() {
            // A paragraph in a cell: a space between it and the next.
            if let Some(cell) = self
                .tables
                .last_mut()
                .and_then(|t| t.rows.last_mut())
                .and_then(|r| r.last_mut())
                && !cell.is_empty()
                && !cell.ends_with(' ')
            {
                cell.push(' ');
            }
            self.para = None;
            return;
        }
        let Some(p) = self.para.take() else { return };
        if p.runs.iter().all(|(s, _)| s.trim().is_empty()) {
            return;
        }
        match (p.heading, p.item) {
            (Some(level), _) => {
                self.close_list();
                self.d.start_heading(level);
                runs(&mut self.d, &p.runs);
                self.d.end_heading(level);
            }
            (None, Some(numbered)) => {
                if self.list != Some(numbered) {
                    self.close_list();
                    self.d.start_list(numbered);
                    self.list = Some(numbered);
                }
                self.d.start(pulldown_cmark::Tag::Item);
                runs(&mut self.d, &p.runs);
                self.d.end(pulldown_cmark::TagEnd::Item);
            }
            (None, None) => {
                self.close_list();
                self.d.start(pulldown_cmark::Tag::Paragraph);
                runs(&mut self.d, &p.runs);
                self.d.end(pulldown_cmark::TagEnd::Paragraph);
            }
        }
    }

    fn start_table(&mut self) {
        if self.tables.is_empty() {
            self.end_para();
            self.close_list();
        }
        self.tables.push(TableAcc::default());
    }

    fn start_row(&mut self) {
        if let Some(t) = self.tables.last_mut() {
            t.rows.push(Vec::new());
        }
    }

    fn start_cell(&mut self) {
        if let Some(row) = self.tables.last_mut().and_then(|t| t.rows.last_mut()) {
            row.push(String::new());
        }
    }

    fn end_table(&mut self) {
        let Some(t) = self.tables.pop() else { return };
        if let Some(outer) = self.tables.last_mut() {
            // A table in a table: its text goes into the cell, flat.
            let text: Vec<String> = t.rows.iter().map(|r| r.join(" | ")).collect();
            if let Some(cell) = outer.rows.last_mut().and_then(|r| r.last_mut()) {
                cell.push_str(&text.join(" / "));
            }
            return;
        }
        let mut rows: Vec<Vec<String>> = t
            .rows
            .into_iter()
            .map(|r| r.into_iter().map(|c| c.trim().to_string()).collect())
            .collect();
        if rows.is_empty() {
            return;
        }
        let header = rows.remove(0);
        self.d.table(&header, &rows);
    }

    fn finish(mut self) -> Self {
        while !self.tables.is_empty() {
            self.end_table();
        }
        self.end_para();
        self.close_list();
        self
    }
}

fn runs(d: &mut DocEvents, runs: &[(String, Look)]) {
    for (s, look) in runs {
        d.styled(s, *look);
    }
}

/// The look of a run from its properties.
#[derive(Default, Clone, Copy)]
struct RunProps {
    bold: bool,
    italic: bool,
    strike: bool,
    hidden: bool,
    /// White text or a font of a point or less: there, but not to be seen.
    invisible: bool,
}

impl RunProps {
    fn look(&self, inserted: bool, deleted: bool) -> Look {
        if deleted || self.strike {
            Look::Strike
        } else if inserted {
            Look::Underline
        } else if self.bold {
            Look::Strong
        } else if self.italic {
            Look::Emphasis
        } else {
            Look::Plain
        }
    }
}

/// The numbered lists of a Word document: for each `numId`, whether its first
/// level counts (decimal, letters...) or is a bullet.
fn word_numbering<R: Read + Seek>(pkg: &mut Package<R>) -> HashMap<String, bool> {
    let Some(text) = pkg.text("word/numbering.xml") else {
        return HashMap::new();
    };
    let mut abstract_fmt: HashMap<String, bool> = HashMap::new();
    let mut num_abstract: Vec<(String, String)> = Vec::new();
    let mut xml = Xml::new(&text, pkg.deadline);
    let (mut current_abstract, mut current_num, mut level) =
        (None::<String>, None::<String>, None::<String>);
    while let Ok(Some(ev)) = xml.next() {
        match ev {
            Ev::Start(n, a) | Ev::Empty(n, a) => match n.as_str() {
                "abstractNum" => current_abstract = attr(&a, "abstractNumId").map(String::from),
                "lvl" => level = attr(&a, "ilvl").map(String::from),
                "numFmt" if level.as_deref() == Some("0") => {
                    if let Some(id) = &current_abstract {
                        abstract_fmt.insert(id.clone(), attr(&a, "val") != Some("bullet"));
                    }
                }
                "num" => current_num = attr(&a, "numId").map(String::from),
                "abstractNumId" => {
                    if let (Some(num), Some(v)) = (&current_num, attr(&a, "val")) {
                        num_abstract.push((num.clone(), v.to_string()));
                    }
                }
                _ => {}
            },
            Ev::End(n) if n == "lvl" => level = None,
            _ => {}
        }
    }
    num_abstract
        .into_iter()
        .map(|(num, abs)| {
            let numbered = abstract_fmt.get(&abs).copied().unwrap_or(false);
            (num, numbered)
        })
        .collect()
}

/// The text of a Word part (the body, a footnote, a comment) into `flow`.
fn word_part(
    xml_text: &str,
    flow: &mut Flow,
    rels: &HashMap<String, Rel>,
    numbering: &HashMap<String, bool>,
    deadline: Instant,
) -> Result<bool, String> {
    let mut xml = Xml::new(xml_text, deadline);
    let mut props = RunProps::default();
    let (mut in_rpr, mut in_ppr) = (false, false);
    let (mut inserted, mut deleted) = (0usize, 0usize);
    let mut in_text = false;
    let mut link: Option<String> = None;
    let mut link_text = String::new();
    while let Some(ev) = xml.next()? {
        if flow.d.cut {
            break;
        }
        match ev {
            Ev::Start(n, a) | Ev::Empty(n, a) => match n.as_str() {
                "p" => flow.start_para(),
                "pPr" => in_ppr = true,
                "pStyle" if in_ppr => {
                    let style = attr(&a, "val").unwrap_or("");
                    let lower = style.to_ascii_lowercase();
                    let level = if lower == "title" {
                        Some(1)
                    } else if lower == "subtitle" {
                        Some(2)
                    } else {
                        lower
                            .strip_prefix("heading")
                            .and_then(|n| n.trim().parse::<u8>().ok())
                            .map(|n| n.clamp(1, 6))
                    };
                    if level.is_some() {
                        flow.para().heading = level;
                    }
                }
                "numId" if in_ppr => {
                    let numbered = attr(&a, "val")
                        .and_then(|id| numbering.get(id))
                        .copied()
                        .unwrap_or(false);
                    if attr(&a, "val") != Some("0") {
                        flow.para().item = Some(numbered);
                    }
                }
                "r" => props = RunProps::default(),
                "rPr" => in_rpr = true,
                "b" if in_rpr => props.bold = on(&a),
                "i" if in_rpr => props.italic = on(&a),
                "strike" | "dstrike" if in_rpr => props.strike = on(&a),
                "vanish" | "specVanish" if in_rpr => props.hidden = on(&a),
                "color" if in_rpr => {
                    if matches!(
                        attr(&a, "val").map(str::to_ascii_uppercase).as_deref(),
                        Some("FFFFFF")
                    ) {
                        props.invisible = true;
                    }
                }
                "sz" if in_rpr => {
                    if attr(&a, "val")
                        .and_then(|v| v.parse::<u32>().ok())
                        .is_some_and(|v| v <= 2)
                    {
                        props.invisible = true;
                    }
                }
                "t" | "delText" | "instrText" => in_text = n != "instrText",
                "tab" if !in_ppr => flow.push(" ", Look::Plain),
                "br" | "cr" => flow.push("\n", Look::Plain),
                "noBreakHyphen" => flow.push("-", Look::Plain),
                "ins" => {
                    inserted += 1;
                    flow.revisions += 1;
                }
                "del" => {
                    deleted += 1;
                    flow.revisions += 1;
                }
                "hyperlink" => {
                    link = prefixed(&a, "id")
                        .and_then(|id| rels.get(id))
                        .map(|r| r.target.clone())
                        .or_else(|| attr(&a, "anchor").map(|x| format!("#{x}")));
                    link_text.clear();
                }
                "blip" => {
                    flow.images += 1;
                    let name = attr(&a, "embed")
                        .and_then(|id| rels.get(id))
                        .map_or("?".to_string(), |r| r.target.clone());
                    flow.push(&format!("[image: {name}]"), Look::Emphasis);
                }
                "imagedata" => {
                    flow.images += 1;
                    flow.push("[image]", Look::Emphasis);
                }
                "footnoteReference" | "endnoteReference" => {
                    let id = attr(&a, "id").unwrap_or("?");
                    flow.push(&format!("[{id}]"), Look::Plain);
                }
                "commentReference" => {
                    let id = attr(&a, "id").unwrap_or("?");
                    flow.push(&format!("[comment {id}]"), Look::Emphasis);
                }
                "tbl" => flow.start_table(),
                "tr" => flow.start_row(),
                "tc" => flow.start_cell(),
                _ => {}
            },
            Ev::End(n) => match n.as_str() {
                "p" => flow.end_para(),
                "pPr" => in_ppr = false,
                "rPr" => in_rpr = false,
                "t" | "delText" | "instrText" => in_text = false,
                "ins" => inserted = inserted.saturating_sub(1),
                "del" => deleted = deleted.saturating_sub(1),
                "hyperlink" => {
                    if let Some(url) = link.take()
                        && !link_text.is_empty()
                    {
                        flow.push(&format!(" <{url}>"), Look::Emphasis);
                    }
                    link_text.clear();
                }
                "tbl" => flow.end_table(),
                _ => {}
            },
            Ev::Text(t) if in_text => {
                let look = props.look(inserted > 0, deleted > 0);
                if link.is_some() {
                    link_text.push_str(&t);
                }
                if props.hidden || props.invisible {
                    flow.hidden += 1;
                    let mark = if props.hidden { "hidden" } else { "invisible" };
                    flow.marked(mark, &t, look);
                } else if deleted > 0 {
                    flow.marked("deleted", &t, look);
                } else if inserted > 0 {
                    flow.marked("inserted", &t, look);
                } else {
                    flow.push(&t, look);
                }
            }
            Ev::Text(_) => {}
        }
    }
    Ok(xml.late)
}

/// Footnotes or comments of a Word document: `(label, text)` of each.
fn word_notes<R: Read + Seek>(
    pkg: &mut Package<R>,
    part: &str,
    element: &str,
) -> Vec<(String, String)> {
    let Some(text) = pkg.text(part) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut xml = Xml::new(&text, pkg.deadline);
    let mut current: Option<(String, String)> = None;
    let mut in_text = false;
    while let Ok(Some(ev)) = xml.next() {
        match ev {
            Ev::Start(n, a) if n == element => {
                let id = attr(&a, "id").unwrap_or("?").to_string();
                let who = attr(&a, "author")
                    .map(|w| format!(" ({w})"))
                    .unwrap_or_default();
                // Word's own separators are notes -1 and 0.
                if !matches!(
                    attr(&a, "type"),
                    Some("separator" | "continuationSeparator")
                ) {
                    current = Some((format!("{id}{who}"), String::new()));
                }
            }
            Ev::Start(n, _) if n == "t" || n == "delText" => in_text = true,
            Ev::End(n) if n == "t" || n == "delText" => in_text = false,
            Ev::End(n) if n == "p" => {
                if let Some((_, t)) = &mut current {
                    t.push(' ');
                }
            }
            Ev::Text(t) if in_text => {
                if let Some((_, s)) = &mut current
                    && s.len() < 64 * 1024
                {
                    s.push_str(&t);
                }
            }
            Ev::End(n) if n == element => {
                if let Some((label, text)) = current.take() {
                    out.push((label, text.trim().to_string()));
                }
            }
            _ => {}
        }
        if out.len() >= 100_000 {
            break;
        }
    }
    out
}

/// The text of an OpenDocument part (`content.xml`): text documents and
/// presentations.
fn odf_part(text: &str, flow: &mut Flow, deadline: Instant) -> Result<bool, String> {
    let mut xml = Xml::new(text, deadline);
    // The automatic styles: which ones are bold, italic, struck or hidden.
    let mut styles: HashMap<String, RunProps> = HashMap::new();
    let mut style_name: Option<String> = None;
    let mut spans: Vec<RunProps> = Vec::new();
    let mut in_body = false;
    let mut slide = 0usize;
    let mut in_note_body = 0usize;
    let mut note_citation = false;
    let mut note_text = String::new();
    let mut note_label = String::new();
    let mut in_annotation = 0usize;
    let mut annotation = String::new();
    let mut annotation_by = String::new();
    let mut in_creator = false;
    let mut in_changes = 0usize;
    let mut in_list = 0usize;
    let mut link: Option<String> = None;
    while let Some(ev) = xml.next()? {
        if flow.d.cut {
            break;
        }
        let props = spans.last().copied().unwrap_or_default();
        match ev {
            Ev::Start(n, a) | Ev::Empty(n, a) => match n.as_str() {
                "style" => style_name = attr(&a, "name").map(String::from),
                "text-properties" => {
                    if let Some(name) = &style_name {
                        let p = styles.entry(name.clone()).or_default();
                        if attr(&a, "font-weight") == Some("bold") {
                            p.bold = true;
                        }
                        if attr(&a, "font-style") == Some("italic") {
                            p.italic = true;
                        }
                        if attr(&a, "text-line-through-style").is_some_and(|v| v != "none") {
                            p.strike = true;
                        }
                        if attr(&a, "display") == Some("none") {
                            p.hidden = true;
                        }
                        if attr(&a, "color").is_some_and(|c| c.eq_ignore_ascii_case("#ffffff")) {
                            p.invisible = true;
                        }
                    }
                }
                "body" => in_body = true,
                _ if !in_body => {}
                "tracked-changes" => in_changes += 1,
                _ if in_changes > 0 => {
                    if matches!(n.as_str(), "insertion" | "deletion" | "format-change") {
                        flow.revisions += 1;
                    }
                }
                "page" => {
                    slide += 1;
                    flow.end_para();
                    let name = attr(&a, "name").unwrap_or("");
                    let title = if name.is_empty() {
                        format!("Slide {slide}")
                    } else {
                        format!("Slide {slide}: {name}")
                    };
                    flow.close_list();
                    flow.d.heading(2, &title);
                }
                "notes" => {
                    flow.end_para();
                    flow.d.start(pulldown_cmark::Tag::Paragraph);
                    flow.d.styled("Notes:", Look::Emphasis);
                    flow.d.end(pulldown_cmark::TagEnd::Paragraph);
                }
                "h" => {
                    flow.start_para();
                    let level = attr(&a, "outline-level")
                        .and_then(|l| l.parse::<u8>().ok())
                        .unwrap_or(1)
                        .clamp(1, 6);
                    flow.para().heading = Some(level);
                }
                "p" if in_note_body == 0 && in_annotation == 0 => {
                    flow.start_para();
                    if in_list > 0 {
                        flow.para().item = Some(false);
                    }
                }
                "list" => in_list += 1,
                "span" => {
                    let mut p = props;
                    if let Some(s) = attr(&a, "style-name").and_then(|s| styles.get(s)) {
                        p.bold |= s.bold;
                        p.italic |= s.italic;
                        p.strike |= s.strike;
                        p.hidden |= s.hidden;
                        p.invisible |= s.invisible;
                    }
                    spans.push(p);
                }
                "s" => {
                    let count = attr(&a, "c")
                        .and_then(|c| c.parse::<usize>().ok())
                        .unwrap_or(1)
                        .min(256);
                    flow.push(&" ".repeat(count), props.look(false, false));
                }
                "tab" => flow.push(" ", Look::Plain),
                "line-break" => flow.push("\n", Look::Plain),
                "a" => link = attr(&a, "href").map(String::from),
                "image" => {
                    flow.images += 1;
                    let name = attr(&a, "href").unwrap_or("?");
                    flow.push(&format!("[image: {name}]"), Look::Emphasis);
                }
                "hidden-text" | "hidden-paragraph" => {
                    flow.hidden += 1;
                    let t = attr(&a, "string-value").unwrap_or("");
                    flow.marked("hidden", t, Look::Plain);
                }
                "note" => {
                    in_note_body = 0;
                    note_label.clear();
                    note_text.clear();
                }
                "note-citation" => note_citation = true,
                "note-body" => in_note_body += 1,
                "annotation" => {
                    in_annotation += 1;
                    annotation.clear();
                    annotation_by.clear();
                }
                "creator" if in_annotation > 0 => in_creator = true,
                "table" => flow.start_table(),
                "table-row" => flow.start_row(),
                "table-cell" => flow.start_cell(),
                "change-start" | "change" => flow.push("[changed]", Look::Emphasis),
                _ => {}
            },
            Ev::End(n) => match n.as_str() {
                "style" => style_name = None,
                "tracked-changes" => in_changes = in_changes.saturating_sub(1),
                "notes" => flow.end_para(),
                "h" | "p" if in_note_body == 0 && in_annotation == 0 => flow.end_para(),
                "list" => {
                    in_list = in_list.saturating_sub(1);
                    if in_list == 0 {
                        flow.end_para();
                        flow.close_list();
                    }
                }
                "span" => {
                    spans.pop();
                }
                "a" => {
                    if let Some(url) = link.take() {
                        flow.push(&format!(" <{url}>"), Look::Emphasis);
                    }
                }
                "note-citation" => {
                    note_citation = false;
                    flow.push(&format!("[{note_label}]"), Look::Plain);
                }
                "note-body" => in_note_body = in_note_body.saturating_sub(1),
                "note" => {
                    let label = if note_label.is_empty() {
                        "?".to_string()
                    } else {
                        note_label.clone()
                    };
                    flow.notes.push((label, note_text.trim().to_string()));
                }
                "creator" => in_creator = false,
                "annotation" => {
                    in_annotation = in_annotation.saturating_sub(1);
                    let label = if annotation_by.is_empty() {
                        "comment".to_string()
                    } else {
                        format!("comment ({})", annotation_by.trim())
                    };
                    flow.notes.push((label, annotation.trim().to_string()));
                    flow.push("[comment]", Look::Emphasis);
                }
                "table" => flow.end_table(),
                _ => {}
            },
            Ev::Text(t) => {
                if !in_body || in_changes > 0 {
                    continue;
                }
                if note_citation {
                    note_label.push_str(&t);
                } else if in_note_body > 0 {
                    if note_text.len() < 64 * 1024 {
                        note_text.push_str(&t);
                        note_text.push(' ');
                    }
                } else if in_annotation > 0 {
                    if in_creator {
                        annotation_by.push_str(&t);
                    } else if annotation.len() < 64 * 1024 {
                        annotation.push_str(&t);
                    }
                } else if props.hidden || props.invisible {
                    flow.hidden += 1;
                    let mark = if props.hidden { "hidden" } else { "invisible" };
                    flow.marked(mark, &t, props.look(false, false));
                } else {
                    flow.push(&t, props.look(false, false));
                }
            }
        }
    }
    Ok(xml.late)
}

/// The text of a PowerPoint slide (or of its notes) into `flow`.
fn slide_part(
    text: &str,
    flow: &mut Flow,
    deadline: Instant,
    notes: bool,
) -> Result<(bool, bool), String> {
    let mut xml = Xml::new(text, deadline);
    let mut in_text = false;
    let mut placeholder: Option<String> = None;
    let mut hidden_slide = false;
    let mut props = RunProps::default();
    while let Some(ev) = xml.next()? {
        if flow.d.cut {
            break;
        }
        match ev {
            Ev::Start(n, a) | Ev::Empty(n, a) => match n.as_str() {
                "sld" if attr(&a, "show") == Some("0") => hidden_slide = true,
                "sp" => placeholder = None,
                "ph" => placeholder = Some(attr(&a, "type").unwrap_or("body").to_string()),
                "p" => {
                    flow.start_para();
                    if matches!(placeholder.as_deref(), Some("title" | "ctrTitle")) && !notes {
                        flow.para().heading = Some(3);
                    }
                }
                "r" => props = RunProps::default(),
                "rPr" => {
                    props.bold = attr(&a, "b") == Some("1");
                    props.italic = attr(&a, "i") == Some("1");
                    props.strike = attr(&a, "strike").is_some_and(|s| s != "noStrike");
                }
                "t" => in_text = true,
                "br" => flow.push("\n", Look::Plain),
                "blip" => {
                    flow.images += 1;
                    flow.push("[image]", Look::Emphasis);
                }
                "tbl" => flow.start_table(),
                "tr" => flow.start_row(),
                "tc" => flow.start_cell(),
                _ => {}
            },
            Ev::End(n) => match n.as_str() {
                "p" => flow.end_para(),
                "t" => in_text = false,
                "tbl" => flow.end_table(),
                _ => {}
            },
            Ev::Text(t) if in_text => {
                // The notes page repeats the slide number and the slide.
                if notes && matches!(placeholder.as_deref(), Some("sldNum" | "sldImg")) {
                    continue;
                }
                flow.push(&t, props.look(false, false));
            }
            Ev::Text(_) => {}
        }
    }
    Ok((xml.late, hidden_slide))
}

/// What the package itself says beyond the text: macros, links out,
/// embedded objects.
struct Findings {
    macros: bool,
    external: Vec<Rel>,
    embedded: Vec<(String, u64)>,
}

fn findings<R: Read + Seek>(pkg: &mut Package<R>, odf: bool) -> Findings {
    let names = pkg.names.clone();
    let macros = names.iter().any(|n| {
        let lower = n.to_ascii_lowercase();
        lower.ends_with("vbaproject.bin")
            || lower.ends_with("vbadata.xml")
            || (odf && (lower.starts_with("basic/") || lower.starts_with("scripts/")))
    });
    let mut external = Vec::new();
    for name in names.iter().filter(|n| n.ends_with(".rels")).take(1000) {
        let Some(text) = pkg.text(name) else { continue };
        let mut xml = Xml::new(&text, pkg.deadline);
        while let Ok(Some(ev)) = xml.next() {
            if let Ev::Start(n, a) | Ev::Empty(n, a) = ev
                && n == "Relationship"
                && attr(&a, "TargetMode") == Some("External")
                && external.len() < 1000
            {
                external.push(Rel {
                    kind: attr(&a, "Type")
                        .and_then(|t| t.rsplit('/').next())
                        .unwrap_or("")
                        .to_string(),
                    target: attr(&a, "Target").unwrap_or("").to_string(),
                    external: true,
                });
            }
        }
    }
    let embedded_names: Vec<String> = names
        .iter()
        .filter(|n| {
            n.contains("/embeddings/")
                || n.starts_with("Object ")
                || n.starts_with("ObjectReplacements/")
        })
        .take(1000)
        .cloned()
        .collect();
    let embedded = embedded_names
        .into_iter()
        .map(|n| {
            let size = pkg.declared_size(&n);
            (n, size)
        })
        .collect();
    Findings {
        macros,
        external,
        embedded,
    }
}

/// The external relationships worth a warning: they make the program fetch
/// or run something when the document opens.
fn risky(kind: &str) -> bool {
    matches!(
        kind,
        "attachedTemplate" | "oleObject" | "frame" | "subDocument" | "externalLinkPath" | "package"
    )
}

/// The package `reader` read: what it says of itself, what it hides, and
/// its text as events.
fn read<R: Read + Seek>(reader: R, kind: Kind, limits: Limits) -> Result<Reading, String> {
    let mut pkg = Package::open(reader, limits)?;
    let odf = matches!(kind, Kind::Text | Kind::Impress);
    let meta = metadata(&mut pkg, odf);
    let found = findings(&mut pkg, odf);
    let mut flow = Flow::new();
    let mut late = false;
    let mut hidden_slides = 0;
    match kind {
        Kind::Word => {
            let main = "word/document.xml";
            let text = pkg
                .text(main)
                .ok_or("No word/document.xml: showing the bytes")?;
            let rels = rels(&mut pkg, main);
            let numbering = word_numbering(&mut pkg);
            late |= word_part(&text, &mut flow, &rels, &numbering, pkg.deadline)?;
            for (part, element, label) in [
                ("word/footnotes.xml", "footnote", ""),
                ("word/endnotes.xml", "endnote", "end note "),
                ("word/comments.xml", "comment", "comment "),
            ] {
                for (l, t) in word_notes(&mut pkg, part, element) {
                    if !t.is_empty() {
                        flow.notes.push((format!("{label}{l}"), t));
                    }
                }
            }
        }
        Kind::Text | Kind::Impress => {
            let text = pkg
                .text("content.xml")
                .ok_or("No content.xml: showing the bytes")?;
            late |= odf_part(&text, &mut flow, pkg.deadline)?;
        }
        Kind::Slides => {
            let main = "ppt/presentation.xml";
            let text = pkg
                .text(main)
                .ok_or("No ppt/presentation.xml: showing the bytes")?;
            let rels = rels(&mut pkg, main);
            let mut order = Vec::new();
            let mut xml = Xml::new(&text, pkg.deadline);
            while let Some(ev) = xml.next()? {
                if let Ev::Start(n, a) | Ev::Empty(n, a) = ev
                    && n == "sldId"
                    && let Some(rel) = prefixed(&a, "id").and_then(|id| rels.get(id))
                    && order.len() < MAX_SLIDES
                {
                    order.push(rel.target.clone());
                }
            }
            for (i, slide) in order.iter().enumerate() {
                if flow.d.cut || pkg.late() {
                    late = true;
                    break;
                }
                let Some(text) = pkg.text(slide) else {
                    continue;
                };
                flow.end_para();
                flow.close_list();
                let at = flow.d.events.len();
                let (l, hidden) = slide_part(&text, &mut flow, pkg.deadline, false)?;
                late |= l;
                let title = if hidden {
                    hidden_slides += 1;
                    format!("Slide {} (hidden)", i + 1)
                } else {
                    format!("Slide {}", i + 1)
                };
                let mut head = DocEvents::new(8);
                head.heading(2, &title);
                let tail = flow.d.events.split_off(at);
                flow.d.events.extend(head.events);
                flow.d.events.extend(tail);
                let slide_rels = rels_of(&mut pkg, slide);
                if let Some(notes) = slide_rels
                    .values()
                    .find(|r| r.kind == "notesSlide" && !r.external)
                    && let Some(text) = pkg.text(&notes.target.clone())
                {
                    flow.end_para();
                    flow.close_list();
                    flow.d.start(pulldown_cmark::Tag::Paragraph);
                    flow.d.styled("Notes:", Look::Emphasis);
                    flow.d.end(pulldown_cmark::TagEnd::Paragraph);
                    late |= slide_part(&text, &mut flow, pkg.deadline, true)?.0;
                }
            }
        }
    }
    let flow = flow.finish();
    late |= pkg.late();
    Ok(Reading {
        meta,
        found,
        flow,
        late,
        hidden_slides,
        cut: pkg.cut,
    })
}

/// What [`read`] gives.
struct Reading {
    meta: Vec<(String, String)>,
    found: Findings,
    flow: Flow,
    late: bool,
    hidden_slides: usize,
    cut: bool,
}

/// What the Scan (Alt+V) says of the document: macros, what it fetches from
/// outside when it opens, embedded objects, hidden text and slides. Empty:
/// nothing of the kind. The same reading and limits as the viewer's.
pub fn warnings<R: Read + Seek>(
    reader: R,
    kind: Kind,
    limits: Limits,
) -> Result<Vec<String>, String> {
    let r = read(reader, kind, limits)?;
    let mut out = Vec::new();
    if r.found.macros {
        out.push("macros".to_string());
    }
    let risky_links: Vec<&Rel> = r.found.external.iter().filter(|l| risky(&l.kind)).collect();
    for l in risky_links.iter().take(2) {
        out.push(format!("fetches from outside: {} {}", l.kind, l.target));
    }
    if risky_links.len() > 2 {
        out.push(format!(
            "{} more links fetched from outside",
            risky_links.len() - 2
        ));
    }
    if !r.found.embedded.is_empty() {
        out.push(format!("{} embedded objects", r.found.embedded.len()));
    }
    if r.flow.hidden > 0 {
        out.push(format!("hidden text ({} pieces)", r.flow.hidden));
    }
    if r.hidden_slides > 0 {
        out.push(format!("{} hidden slides", r.hidden_slides));
    }
    if out.is_empty() && (r.cut || r.late || r.flow.d.cut) {
        return Err("over the limits of the viewer: only the start was read".into());
    }
    Ok(out)
}

/// The document in the package `reader` as a document laid out for `width`,
/// or why it could not be read (the bytes are then shown).
pub fn render<R: Read + Seek>(
    reader: R,
    kind: Kind,
    limits: Limits,
    width: usize,
) -> Result<Doc, String> {
    let Reading {
        meta,
        found,
        mut flow,
        late,
        hidden_slides,
        cut,
    } = read(reader, kind, limits)?;

    // What it says of itself, before the text.
    let mut head = DocEvents::new(100_000);
    head.heading(1, "Document");
    for (label, value) in &meta {
        head.field(label, value);
    }
    if found.macros {
        head.start(pulldown_cmark::Tag::Paragraph);
        head.styled("Contains macros", Look::Strong);
        head.text(" (not read, not run)");
        head.end(pulldown_cmark::TagEnd::Paragraph);
    }
    let risky_links: Vec<&Rel> = found.external.iter().filter(|r| risky(&r.kind)).collect();
    let other_links = found.external.len() - risky_links.len();
    if !risky_links.is_empty() {
        head.start(pulldown_cmark::Tag::Paragraph);
        head.styled("Fetches from outside when it opens:", Look::Strong);
        head.end(pulldown_cmark::TagEnd::Paragraph);
        head.start_list(false);
        for r in &risky_links {
            head.start(pulldown_cmark::Tag::Item);
            head.styled(&format!("{}: ", r.kind), Look::Strong);
            head.code(&r.target);
            head.end(pulldown_cmark::TagEnd::Item);
        }
        head.end_list(false);
    }
    if other_links > 0 {
        head.field("External links", &other_links.to_string());
    }
    if !found.embedded.is_empty() {
        head.field("Embedded objects", &found.embedded.len().to_string());
        head.start_list(false);
        for (name, size) in found.embedded.iter().take(100) {
            head.item(&format!("{name} ({size} bytes)"));
        }
        head.end_list(false);
    }
    if flow.revisions > 0 {
        head.field(
            "Tracked changes",
            &format!("{} (shown marked)", flow.revisions),
        );
    }
    if flow.hidden > 0 {
        head.field(
            "Hidden text",
            &format!("{} pieces (shown marked)", flow.hidden),
        );
    }
    if hidden_slides > 0 {
        head.field("Hidden slides", &hidden_slides.to_string());
    }
    if flow.images > 0 {
        head.field("Images", &format!("{} (not shown)", flow.images));
    }
    if cut || late || flow.d.cut {
        head.paragraph("Only the start of the document was read: it is over the limits.");
    }
    head.rule();

    let mut events = head.events;
    events.append(&mut flow.d.events);
    let mut tail = DocEvents::new(200_000);
    if !flow.notes.is_empty() {
        tail.heading(2, "Notes and comments");
        tail.start_list(false);
        for (label, text) in &flow.notes {
            tail.start(pulldown_cmark::Tag::Item);
            tail.styled(&format!("[{label}] "), Look::Strong);
            tail.text(text);
            tail.end(pulldown_cmark::TagEnd::Item);
        }
        tail.end_list(false);
    }
    events.append(&mut tail.events);
    let mut doc = markdown::from_events("", events, limits.time, limits.budget);
    if cut || late || flow.d.cut {
        doc.mark_cut();
    }
    doc.layout(width);
    Ok(doc)
}

/// The relationships of a slide.
fn rels_of<R: Read + Seek>(pkg: &mut Package<R>, part: &str) -> HashMap<String, Rel> {
    rels(pkg, part)
}

#[cfg(test)]
pub mod test_docs {
    //! Small Office packages for the tests.
    use std::io::Write;

    pub fn zip(parts: &[(&str, &str)]) -> Vec<u8> {
        let mut out = std::io::Cursor::new(Vec::new());
        {
            let mut w = zip::ZipWriter::new(&mut out);
            let opts: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            for (name, text) in parts {
                w.start_file(*name, opts).unwrap();
                w.write_all(text.as_bytes()).unwrap();
            }
            w.finish().unwrap();
        }
        out.into_inner()
    }

    pub const W: &str = r#"xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships""#;

    pub fn docx(body: &str) -> Vec<u8> {
        zip(&[
            ("[Content_Types].xml", "<Types/>"),
            (
                "word/document.xml",
                &format!("<w:document {W}><w:body>{body}</w:body></w:document>"),
            ),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::super::package::MAX_DEPTH;
    use super::test_docs::*;
    use super::*;
    use std::io::Cursor;
    use std::time::Duration;

    const LIMITS: Limits = Limits {
        part: 1 << 20,
        total: 1 << 22,
        time: Duration::from_secs(5),
        budget: 1 << 24,
    };

    fn text_of(bytes: Vec<u8>, kind: Kind) -> String {
        let doc = render(Cursor::new(bytes), kind, LIMITS, 100).unwrap();
        (0..doc.line_count())
            .map(|i| doc.line(i).to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn a_word_document_with_what_it_hides() {
        let body = r#"
<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>The title</w:t></w:r></w:p>
<w:p><w:r><w:rPr><w:b/></w:rPr><w:t>Bold</w:t></w:r><w:r><w:t xml:space="preserve"> and **not markdown** &amp; &#x41;</w:t></w:r></w:p>
<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>an item</w:t></w:r></w:p>
<w:p><w:r><w:rPr><w:vanish/></w:rPr><w:t>secret</w:t></w:r><w:ins w:author="Ann"><w:r><w:t>added</w:t></w:r></w:ins><w:del w:author="Bob"><w:r><w:delText>removed</w:delText></w:r></w:del></w:p>
<w:p><w:r><w:rPr><w:color w:val="FFFFFF"/></w:rPr><w:t>white</w:t></w:r></w:p>
<w:tbl><w:tr><w:tc><w:p><w:r><w:t>k</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>v</w:t></w:r></w:p></w:tc></w:tr>
<w:tr><w:tc><w:p><w:r><w:t>1</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>2</w:t></w:r></w:p></w:tc></w:tr></w:tbl>
<w:p><w:hyperlink r:id="rId9"><w:r><w:t>site</w:t></w:r></w:hyperlink></w:p>
"#;
        let bytes = zip(&[
            (
                "word/document.xml",
                &format!("<w:document {W}><w:body>{body}</w:body></w:document>"),
            ),
            (
                "word/_rels/document.xml.rels",
                r#"<Relationships><Relationship Id="rId9" Type="http://x/hyperlink" Target="https://example.com" TargetMode="External"/></Relationships>"#,
            ),
            (
                "word/_rels/settings.xml.rels",
                r#"<Relationships><Relationship Id="r1" Type="http://x/attachedTemplate" Target="http://evil.example/t.dotm" TargetMode="External"/></Relationships>"#,
            ),
            ("word/vbaProject.bin", "x"),
            ("word/embeddings/oleObject1.bin", "abc"),
            (
                "docProps/core.xml",
                r#"<cp:coreProperties xmlns:cp="c" xmlns:dc="d"><dc:creator>Alice</dc:creator><cp:lastModifiedBy>Mallory</cp:lastModifiedBy></cp:coreProperties>"#,
            ),
        ]);
        let t = text_of(bytes, Kind::Word);
        for want in [
            "Author: Alice",
            "Last saved by: Mallory",
            "Contains macros",
            "attachedTemplate: http://evil.example/t.dotm",
            "Embedded objects: 1",
            "Tracked changes: 2",
            "Hidden text: 2",
            "The title",
            "Bold and **not markdown** & A",
            "an item",
            "[hidden: secret]",
            "[inserted: added]",
            "[deleted: removed]",
            "[invisible: white]",
            "site <https://example.com>",
            "External links: 1",
        ] {
            assert!(t.contains(want), "{want:?} in\n{t}");
        }
        assert!(t.contains('k') && t.contains('2'));
    }

    #[test]
    fn hostile_packages_are_refused_not_read() {
        let r = |b: Vec<u8>| render(Cursor::new(b), Kind::Word, LIMITS, 80).err();
        assert!(r(b"not a zip".to_vec()).is_some());
        assert!(r(zip(&[("x", "y")])).is_some(), "no document.xml");
        assert!(
            r(zip(&[(
                "word/document.xml",
                "<!DOCTYPE x [<!ENTITY a \"aaaa\">]><d>&a;</d>"
            )]))
            .is_some()
        );
        let deep = "<a>".repeat(MAX_DEPTH + 1);
        assert!(r(zip(&[("word/document.xml", &deep)])).is_some());
        // A bomb: the part stops at its limit.
        let big = format!(
            "<w:document {W}><w:body><w:p><w:r><w:t>{}</w:t></w:r></w:p></w:body></w:document>",
            "a".repeat(3 << 20)
        );
        let small = Limits {
            part: 1 << 20,
            ..LIMITS
        };
        let doc = render(
            Cursor::new(zip(&[("word/document.xml", &big)])),
            Kind::Word,
            small,
            80,
        )
        .unwrap();
        assert!(doc.cut());
    }

    #[test]
    fn resolve_stays_in_the_package() {
        assert_eq!(
            resolve("word", "media/a.png").as_deref(),
            Some("word/media/a.png")
        );
        assert_eq!(
            resolve("ppt/slides", "../notesSlides/n1.xml").as_deref(),
            Some("ppt/notesSlides/n1.xml")
        );
        assert_eq!(
            resolve("word", "/customXml/x.xml").as_deref(),
            Some("customXml/x.xml")
        );
        assert_eq!(resolve("word", "../../etc/passwd"), None);
        // Millions of `/` do not take the stack.
        let deep = format!("{}x", "/".repeat(4_000_000));
        assert_eq!(resolve("word", &deep).as_deref(), Some("x"));
    }

    #[test]
    fn an_odt_with_headings_lists_notes_and_hidden_text() {
        let content = r#"<office:document-content xmlns:office="o" xmlns:text="t" xmlns:style="s" xmlns:fo="f" xmlns:table="tb" xmlns:dc="dc">
<office:automatic-styles><style:style style:name="B"><style:text-properties fo:font-weight="bold"/></style:style>
<style:style style:name="H"><style:text-properties text:display="none"/></style:style></office:automatic-styles>
<office:body><office:text>
<text:tracked-changes><text:changed-region><text:deletion><text:p>gone</text:p></text:deletion></text:changed-region></text:tracked-changes>
<text:h text:outline-level="2">Chapter</text:h>
<text:p>Plain <text:span text:style-name="B">bold</text:span><text:s text:c="2"/>end<text:note><text:note-citation>1</text:note-citation><text:note-body><text:p>a note</text:p></text:note-body></text:note></text:p>
<text:list><text:list-item><text:p>one</text:p></text:list-item></text:list>
<text:p><text:span text:style-name="H">secret</text:span></text:p>
<office:annotation><dc:creator>Eve</dc:creator><text:p>a comment</text:p></office:annotation>
<table:table><table:table-row><table:table-cell><text:p>h</text:p></table:table-cell></table:table-row></table:table>
</office:text></office:body></office:document-content>"#;
        let bytes = zip(&[
            ("content.xml", content),
            ("Basic/Standard/Module1.xml", "x"),
        ]);
        let t = text_of(bytes, Kind::Text);
        for want in [
            "Contains macros",
            "Tracked changes: 1",
            "Chapter",
            "Plain bold  end[1]",
            "one",
            "[hidden: secret]",
            "[1] a note",
            "comment (Eve)",
            "a comment",
        ] {
            assert!(t.contains(want), "{want:?} in\n{t}");
        }
        assert!(
            !t.contains("gone"),
            "a deletion is counted, not mixed into the text"
        );
    }

    #[test]
    fn a_pptx_slide_by_slide_with_notes() {
        let p = r#"xmlns:p="p" xmlns:a="a" xmlns:r="r""#;
        let bytes = zip(&[
            (
                "ppt/presentation.xml",
                &format!(
                    r#"<p:presentation {p}><p:sldIdLst><p:sldId id="256" r:id="rId2"/><p:sldId id="257" r:id="rId3"/></p:sldIdLst></p:presentation>"#
                ),
            ),
            (
                "ppt/_rels/presentation.xml.rels",
                r#"<Relationships><Relationship Id="rId2" Type="x/slide" Target="slides/slide1.xml"/><Relationship Id="rId3" Type="x/slide" Target="slides/slide2.xml"/></Relationships>"#,
            ),
            (
                "ppt/slides/slide1.xml",
                &format!(
                    r#"<p:sld {p}><p:cSld><p:spTree><p:sp><p:nvSpPr><p:nvPr><p:ph type="title"/></p:nvPr></p:nvSpPr><p:txBody><a:p><a:r><a:t>First</a:t></a:r></a:p></p:txBody></p:sp><p:sp><p:txBody><a:p><a:r><a:t>body text</a:t></a:r></a:p></p:txBody></p:sp></p:spTree></p:cSld></p:sld>"#
                ),
            ),
            (
                "ppt/slides/_rels/slide1.xml.rels",
                r#"<Relationships><Relationship Id="rId1" Type="x/notesSlide" Target="../notesSlides/notesSlide1.xml"/></Relationships>"#,
            ),
            (
                "ppt/notesSlides/notesSlide1.xml",
                &format!(
                    r#"<p:notes {p}><p:cSld><p:spTree><p:sp><p:nvSpPr><p:nvPr><p:ph type="body"/></p:nvPr></p:nvSpPr><p:txBody><a:p><a:r><a:t>say this</a:t></a:r></a:p></p:txBody></p:sp></p:spTree></p:cSld></p:notes>"#
                ),
            ),
            (
                "ppt/slides/slide2.xml",
                &format!(
                    r#"<p:sld {p} show="0"><p:cSld><p:spTree><p:sp><p:txBody><a:p><a:r><a:t>Second</a:t></a:r></a:p></p:txBody></p:sp></p:spTree></p:cSld></p:sld>"#
                ),
            ),
        ]);
        let t = text_of(bytes, Kind::Slides);
        for want in [
            "Slide 1",
            "First",
            "body text",
            "Notes:",
            "say this",
            "Slide 2 (hidden)",
            "Second",
            "Hidden slides: 1",
        ] {
            assert!(t.contains(want), "{want:?} in\n{t}");
        }
        let first = t.find("Slide 1").unwrap();
        assert!(
            t.find("First").unwrap() > first
                && t.find("Slide 2").unwrap() > t.find("say this").unwrap()
        );
    }

    proptest::proptest! {
        #[test]
        fn hostile_documents_never_panic(
            picks in proptest::collection::vec(0usize..20, 0..120),
            text in "[a-z &;#<>\u{1b}\u{202e}]{0,20}",
        ) {
            const TAGS: [&str; 20] = [
                "<w:p>", "</w:p>", "<w:r>", "</w:r>", "<w:t>", "</w:t>", "<w:tbl>", "</w:tbl>",
                "<w:tr>", "<w:tc>", "</w:tc>", "<w:ins>", "</w:del>", "<w:rPr><w:vanish/></w:rPr>",
                "<w:pPr><w:pStyle w:val=\"Heading9\"/><w:numPr><w:numId w:val=\"7\"/></w:numPr></w:pPr>",
                "<w:hyperlink r:id=\"x\">", "</w:hyperlink>", "<w:br/>", "&amp;", "&#0;",
            ];
            let body: String = picks.iter().map(|&p| format!("{}{text}", TAGS[p])).collect();
            let bytes = zip(&[("word/document.xml", &format!("<w:document {W}><w:body>{body}</w:body></w:document>"))]);
            let _ = render(Cursor::new(bytes.clone()), Kind::Word, LIMITS, 40);
            let odt = zip(&[("content.xml", &format!("<office:document-content><office:body><office:text>{}</office:text></office:body></office:document-content>", body.replace("w:", "text:")))]);
            let _ = render(Cursor::new(odt), Kind::Text, LIMITS, 40);
        }
    }
}
