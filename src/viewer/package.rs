//! What the documents that are ZIP packages of XML have in common (Office,
//! EPUB): the package read part by part, each decompressed only up to a
//! limit, and its XML read with no DTD and no entities of its own, with a
//! limit on its nesting and on the time.

use std::io::{Read, Seek};
use std::time::{Duration, Instant};

use quick_xml::events::Event;
use quick_xml::reader::Reader;

pub(super) mod html;

/// Most entries in the package.
pub(super) const MAX_ENTRIES: usize = 10_000;
/// Most XML elements open at once.
pub(super) const MAX_DEPTH: usize = 256;

/// How much of a package is read.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// The most of one part, decompressed.
    pub part: u64,
    /// The most of all parts together.
    pub total: u64,
    pub time: Duration,
    /// Memory of the document's lines (see `markdown::from_events`).
    pub budget: usize,
}

/// An element or a piece of text of an XML part.
pub(super) enum Ev {
    Start(String, Vec<(String, String)>),
    Empty(String, Vec<(String, String)>),
    End(String),
    Text(String),
}

/// An XML part read one event at a time, with no DTD, a limit on its nesting,
/// and the time limit.
pub(super) struct Xml<'a> {
    reader: Reader<&'a [u8]>,
    depth: usize,
    count: usize,
    deadline: Instant,
    /// Read as the XHTML of an ebook: see [`Xml::html`].
    html: bool,
    /// Stopped by the time limit.
    pub late: bool,
}

impl<'a> Xml<'a> {
    pub(super) fn new(text: &'a str, deadline: Instant) -> Self {
        let mut reader = Reader::from_str(text);
        let config = reader.config_mut();
        config.check_end_names = false;
        config.expand_empty_elements = false;
        Xml {
            reader,
            depth: 0,
            count: 0,
            deadline,
            html: false,
            late: false,
        }
    }

    /// The XHTML of an ebook: a `<!DOCTYPE>` with nothing declared in it
    /// (`<!DOCTYPE html>`, the XHTML 1.1 one) is passed over, never read or
    /// fetched, while one that declares anything (`[...]`, where entities
    /// live) is still refused; the named entities of HTML 4 (`&nbsp;`,
    /// `&mdash;`) are known, from a fixed table; and the elements HTML never
    /// closes (`<br>`, `<img>`) do not count as open.
    pub(super) fn html(text: &'a str, deadline: Instant) -> Self {
        let mut xml = Self::new(text, deadline);
        xml.html = true;
        xml
    }

    /// The local part of a name: `w:p` is `p`.
    fn local(name: &str) -> String {
        match name.rsplit_once(':') {
            Some((_, local)) => local.to_string(),
            None => name.to_string(),
        }
    }

    fn attrs(e: &quick_xml::events::BytesStart) -> Vec<(String, String)> {
        e.attributes()
            .flatten()
            .take(64)
            .map(|a| {
                let value = a
                    .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                    .map_or_else(|_| a.value.to_string(), |v| v.into_owned());
                let full: &str = a.key.as_ref();
                // A prefixed name keeps its prefix after a `\u{1}`: `r:id`
                // and a plain `id` can be on the same element.
                let key = match full.rsplit_once(':') {
                    Some((_, local)) => format!("{local}\u{1}"),
                    None => full.to_string(),
                };
                (key, value)
            })
            .collect()
    }

    /// The next event, `Ok(None)` at the end (or at the time limit), or why the
    /// part is refused.
    pub(super) fn next(&mut self) -> Result<Option<Ev>, String> {
        loop {
            self.count += 1;
            if self.count.is_multiple_of(4096) && Instant::now() >= self.deadline {
                self.late = true;
                return Ok(None);
            }
            let event = self
                .reader
                .read_event()
                .map_err(|_| "The XML of this document is broken".to_string())?;
            return Ok(Some(match event {
                Event::Start(e) => {
                    let name = Self::local(e.name().as_ref());
                    if self.html && html::is_void(&name) {
                        Ev::Empty(name, Self::attrs(&e))
                    } else {
                        self.depth += 1;
                        if self.depth > MAX_DEPTH {
                            return Err(format!("The XML nests deeper than {MAX_DEPTH} levels"));
                        }
                        Ev::Start(name, Self::attrs(&e))
                    }
                }
                Event::Empty(e) => Ev::Empty(Self::local(e.name().as_ref()), Self::attrs(&e)),
                Event::End(e) => {
                    let name = Self::local(e.name().as_ref());
                    if self.html && html::is_void(&name) {
                        continue;
                    }
                    self.depth = self.depth.saturating_sub(1);
                    Ev::End(name)
                }
                Event::Text(t) => Ev::Text(t.xml10_content().into_owned()),
                Event::CData(t) => Ev::Text(t.to_string()),
                Event::GeneralRef(r) => {
                    let name = r.to_string();
                    let c = match name.as_str() {
                        "lt" => Some('<'),
                        "gt" => Some('>'),
                        "amp" => Some('&'),
                        "apos" => Some('\''),
                        "quot" => Some('"'),
                        _ if self.html => r
                            .resolve_char_ref()
                            .ok()
                            .flatten()
                            .or_else(|| html::entity(&name)),
                        _ => r.resolve_char_ref().ok().flatten(),
                    };
                    Ev::Text(c.map_or_else(|| format!("&{name};"), String::from))
                }
                Event::DocType(d) if self.html && !d.as_ref().contains('[') => continue,
                Event::DocType(_) => {
                    return Err("A document with a DTD is refused".to_string());
                }
                Event::Eof => return Ok(None),
                _ => continue,
            }));
        }
    }
}

/// An attribute by its local name, with a prefix or not (the one without
/// first).
pub(super) fn attr<'v>(attrs: &'v [(String, String)], name: &str) -> Option<&'v str> {
    attrs
        .iter()
        .find(|(k, _)| k == name)
        .or_else(|| {
            attrs
                .iter()
                .find(|(k, _)| k.strip_suffix('\u{1}') == Some(name))
        })
        .map(|(_, v)| v.as_str())
}

/// An attribute with a prefix only, as `r:id` beside a plain `id`.
pub(super) fn prefixed<'v>(attrs: &'v [(String, String)], name: &str) -> Option<&'v str> {
    attrs
        .iter()
        .find(|(k, _)| k.strip_suffix('\u{1}') == Some(name))
        .map(|(_, v)| v.as_str())
}

/// `target` from the folder `dir` of the package, or `None` if it climbs out.
pub(super) fn resolve(dir: &str, target: &str) -> Option<String> {
    // An absolute target is from the root of the package. No recursion: a
    // hostile target may start with millions of `/`.
    let abs = target.trim_start_matches('/');
    let (dir, target) = if abs.len() < target.len() {
        ("", abs)
    } else {
        (dir, target)
    };
    let mut parts: Vec<&str> = dir.split('/').filter(|p| !p.is_empty()).collect();
    for p in target.split('/') {
        match p {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            p => parts.push(p),
        }
    }
    Some(parts.join("/"))
}

/// A package being read: its parts by name, each decompressed up to a limit.
pub(super) struct Package<R: Read + Seek> {
    zip: zip::ZipArchive<R>,
    pub(super) names: Vec<String>,
    limits: Limits,
    used: u64,
    /// A part was cut at its limit, or the total was reached.
    pub(super) cut: bool,
    pub(super) deadline: Instant,
}

impl<R: Read + Seek> Package<R> {
    pub(super) fn open(reader: R, limits: Limits) -> Result<Self, String> {
        let zip = zip::ZipArchive::new(reader)
            .map_err(|_| "Not a document this viewer can read: showing the bytes".to_string())?;
        if zip.len() > MAX_ENTRIES {
            return Err(format!(
                "The document has over {MAX_ENTRIES} parts: showing the bytes"
            ));
        }
        let names = zip
            .file_names()
            .filter_map(|n| n.ok().map(|n| n.into_owned()))
            .collect();
        Ok(Package {
            zip,
            names,
            limits,
            used: 0,
            cut: false,
            deadline: Instant::now() + limits.time,
        })
    }

    pub(super) fn has(&self, name: &str) -> bool {
        self.names.iter().any(|n| n == name)
    }

    /// The text of a part, decompressed up to the limits; `None` if it is not
    /// there or cannot be read.
    pub(super) fn text(&mut self, name: &str) -> Option<String> {
        let bytes = self.bytes(name)?;
        Some(String::from_utf8_lossy(&bytes).into_owned())
    }

    /// The bytes of a part, decompressed up to the limits, as [`text`] reads
    /// them.
    ///
    /// [`text`]: Self::text
    pub(super) fn bytes(&mut self, name: &str) -> Option<Vec<u8>> {
        if !self.has(name) {
            return None;
        }
        let left = self.limits.total.saturating_sub(self.used);
        let cap = self.limits.part.min(left);
        if cap == 0 {
            self.cut = true;
            return None;
        }
        let file = self.zip.by_name(name).ok()?;
        let mut bytes = Vec::new();
        // One byte over the cap tells a part cut from one that fits.
        file.take(cap + 1).read_to_end(&mut bytes).ok()?;
        if bytes.len() as u64 > cap {
            bytes.truncate(cap as usize);
            self.cut = true;
        }
        self.used += bytes.len() as u64;
        Some(bytes)
    }

    /// The size the package says a part has (only shown, never trusted).
    pub(super) fn declared_size(&mut self, name: &str) -> u64 {
        self.zip.by_name(name).map(|f| f.size()).unwrap_or(0)
    }

    pub(super) fn late(&self) -> bool {
        Instant::now() >= self.deadline
    }
}
