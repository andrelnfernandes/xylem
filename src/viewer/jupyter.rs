//! Jupyter notebooks (docs/plans/viewer-jupyter.md): a `.ipynb` shown as a
//! document, cell by cell. Markdown cells are rendered, code cells colored,
//! and the outputs shown as text. Nothing is ever run: not the code, not the
//! JavaScript or HTML of an output, not a widget. An image or an HTML output
//! is a line that says what it is and how big.

use std::collections::BTreeMap;
use std::fmt;
use std::time::Duration;

use serde::Deserialize;
use serde::de::{self, Deserializer, IgnoredAny, SeqAccess, Visitor};

use super::doc_events::{DocEvents, Look};
use super::markdown::{self, Doc};

/// Most cells shown, and most text of one output.
const MAX_CELLS: usize = 100_000;
const MAX_OUTPUT: usize = 1 << 20;
/// Most events of the whole document.
const MAX_EVENTS: usize = 4_000_000;

pub fn is_notebook(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("ipynb"))
}

/// A string, or a list of strings to be joined (as `source` and `text` are).
#[derive(Default)]
struct MultiLine(String);

impl<'de> Deserialize<'de> for MultiLine {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = MultiLine;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a string or a list of strings")
            }
            fn visit_str<E: de::Error>(self, s: &str) -> Result<MultiLine, E> {
                Ok(MultiLine(s.to_string()))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<MultiLine, A::Error> {
                let mut s = String::new();
                while let Some(part) = seq.next_element::<String>()? {
                    s.push_str(&part);
                }
                Ok(MultiLine(s))
            }
            fn visit_unit<E: de::Error>(self) -> Result<MultiLine, E> {
                Ok(MultiLine::default())
            }
        }
        d.deserialize_any(V)
    }
}

/// The size of a value nobody reads (an image in base64, HTML): only its
/// length is kept, never the text.
#[derive(Default, Clone, Copy)]
struct Size(usize);

impl<'de> Deserialize<'de> for Size {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Size;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("any value")
            }
            fn visit_str<E: de::Error>(self, s: &str) -> Result<Size, E> {
                Ok(Size(s.len()))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Size, A::Error> {
                let mut n = 0usize;
                while let Some(part) = seq.next_element::<Size>()? {
                    n = n.saturating_add(part.0);
                }
                Ok(Size(n))
            }
            fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Size, A::Error> {
                let mut n = 0usize;
                while let Some((_, v)) = map.next_entry::<IgnoredAny, Size>()? {
                    n = n.saturating_add(v.0);
                }
                Ok(Size(n))
            }
            fn visit_bool<E: de::Error>(self, _: bool) -> Result<Size, E> {
                Ok(Size(0))
            }
            fn visit_i64<E: de::Error>(self, _: i64) -> Result<Size, E> {
                Ok(Size(0))
            }
            fn visit_u64<E: de::Error>(self, _: u64) -> Result<Size, E> {
                Ok(Size(0))
            }
            fn visit_f64<E: de::Error>(self, _: f64) -> Result<Size, E> {
                Ok(Size(0))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Size, E> {
                Ok(Size(0))
            }
        }
        d.deserialize_any(V)
    }
}

/// The data of an output, by its type: text is kept, the rest is a size.
#[derive(Default)]
struct Data {
    plain: Option<String>,
    markdown: Option<String>,
    others: Vec<(String, usize)>,
}

impl<'de> Deserialize<'de> for Data {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Data;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("the data of an output")
            }
            fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Data, A::Error> {
                let mut data = Data::default();
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "text/plain" | "text" => {
                            data.plain = Some(map.next_value::<MultiLine>()?.0)
                        }
                        "text/markdown" => data.markdown = Some(map.next_value::<MultiLine>()?.0),
                        _ => {
                            let size = map.next_value::<Size>()?;
                            if data.others.len() < 64 {
                                data.others.push((key, size.0));
                            }
                        }
                    }
                }
                Ok(data)
            }
        }
        d.deserialize_map(V)
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Output {
    output_type: String,
    name: Option<String>,
    text: Option<MultiLine>,
    data: Option<Data>,
    execution_count: Option<i64>,
    ename: Option<String>,
    evalue: Option<String>,
    traceback: Option<Vec<String>>,
    // nbformat 3: the data are beside the type.
    prompt_number: Option<i64>,
    png: Option<Size>,
    jpeg: Option<Size>,
    html: Option<Size>,
    svg: Option<Size>,
    latex: Option<Size>,
    javascript: Option<Size>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Cell {
    cell_type: String,
    source: MultiLine,
    /// nbformat 3.
    input: Option<MultiLine>,
    execution_count: Option<i64>,
    prompt_number: Option<i64>,
    outputs: Vec<Output>,
    attachments: Option<BTreeMap<String, Size>>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct LanguageInfo {
    name: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct KernelSpec {
    display_name: Option<String>,
    language: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Metadata {
    kernelspec: Option<KernelSpec>,
    language_info: Option<LanguageInfo>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Worksheet {
    cells: Vec<Cell>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Notebook {
    nbformat: Option<u64>,
    nbformat_minor: Option<u64>,
    metadata: Metadata,
    cells: Vec<Cell>,
    worksheets: Vec<Worksheet>,
}

fn kib(n: usize) -> String {
    if n < 1024 {
        format!("{n} bytes")
    } else {
        format!("{} KiB", n.div_ceil(1024))
    }
}

/// At most [`MAX_OUTPUT`] of a text, cut at a character.
fn capped(s: &str) -> (&str, bool) {
    if s.len() <= MAX_OUTPUT {
        return (s, false);
    }
    let mut end = MAX_OUTPUT;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    (&s[..end], true)
}

/// The notebook in `text` as a document laid out for `width`, or why it
/// could not be read (it is then shown as its source).
pub fn render(text: &str, time: Duration, budget: usize, width: usize) -> Result<Doc, String> {
    let nb: Notebook = serde_json::from_str(text).map_err(|e| {
        format!(
            "Not a notebook this viewer can read ({}): showing the source",
            short(&e)
        )
    })?;
    match nb.nbformat {
        Some(3 | 4) => {}
        Some(n) => {
            return Err(format!(
                "Notebook format {n} is not known: showing the source"
            ));
        }
        None if nb.cells.is_empty() && nb.worksheets.is_empty() => {
            return Err("Not a notebook: showing the source".into());
        }
        None => {}
    }
    let language = nb
        .metadata
        .language_info
        .as_ref()
        .and_then(|l| l.name.clone())
        .or_else(|| {
            nb.metadata
                .kernelspec
                .as_ref()
                .and_then(|k| k.language.clone())
        })
        .unwrap_or_default();
    let kernel = nb
        .metadata
        .kernelspec
        .as_ref()
        .and_then(|k| k.display_name.clone());
    let mut cells = nb.cells;
    for w in nb.worksheets {
        cells.extend(w.cells);
    }
    let mut d = DocEvents::new(MAX_EVENTS);
    let count = |t: &str| cells.iter().filter(|c| c.cell_type == t).count();
    let (md, code, raw) = (count("markdown"), count("code"), count("raw"));
    let mut about = format!(
        "{} cells: {md} Markdown, {code} code, {raw} raw",
        cells.len()
    );
    if let Some(k) = &kernel {
        about = format!("{k} · {about}");
    }
    if let Some(v) = nb.nbformat {
        about.push_str(&format!(
            " · nbformat {v}.{}",
            nb.nbformat_minor.unwrap_or(0)
        ));
    }
    d.start(pulldown_cmark::Tag::Paragraph);
    d.styled("Notebook: ", Look::Strong);
    d.text(&about);
    d.end(pulldown_cmark::TagEnd::Paragraph);
    d.rule();
    let shown = cells.len().min(MAX_CELLS);
    for cell in cells.iter().take(shown) {
        if d.cut {
            break;
        }
        cell_events(&mut d, cell, &language);
    }
    if cells.len() > shown {
        d.paragraph(&format!("… {} more cells not shown", cells.len() - shown));
    }
    let cut = d.cut;
    let mut doc = markdown::from_events(text, d.events, time, budget);
    if cut {
        doc.mark_cut();
    }
    doc.layout(width);
    Ok(doc)
}

/// The start of a JSON error, without what it quotes of the file.
fn short(e: &serde_json::Error) -> String {
    format!("line {}, column {}", e.line(), e.column())
}

fn cell_events(d: &mut DocEvents, cell: &Cell, language: &str) {
    let source = match &cell.input {
        Some(input) if cell.source.0.is_empty() => &input.0,
        _ => &cell.source.0,
    };
    match cell.cell_type.as_str() {
        "markdown" | "heading" => {
            d.extend(markdown::events(source));
            if let Some(a) = &cell.attachments {
                for (name, size) in a {
                    d.paragraph(&format!("[attachment: {name}, {}]", kib(size.0)));
                }
            }
        }
        "code" => {
            let n = cell.execution_count.or(cell.prompt_number);
            let label = n.map_or("In [ ]:".to_string(), |n| format!("In [{n}]:"));
            d.start(pulldown_cmark::Tag::Paragraph);
            d.styled(&label, Look::Strong);
            d.end(pulldown_cmark::TagEnd::Paragraph);
            d.code_block(language, source);
            for out in &cell.outputs {
                output_events(d, out, n);
            }
        }
        other => {
            let label = if other == "raw" {
                "Raw:".to_string()
            } else {
                format!("Cell of type {other}:")
            };
            d.start(pulldown_cmark::Tag::Paragraph);
            d.styled(&label, Look::Emphasis);
            d.end(pulldown_cmark::TagEnd::Paragraph);
            d.code_block("", source);
        }
    }
}

fn output_events(d: &mut DocEvents, out: &Output, cell_n: Option<i64>) {
    let n = out.execution_count.or(out.prompt_number).or(cell_n);
    let label = n.map_or("Out:".to_string(), |n| format!("Out [{n}]:"));
    match out.output_type.as_str() {
        "stream" => {
            let text = out.text.as_ref().map_or("", |t| t.0.as_str());
            let (text, cut) = capped(text);
            if out.name.as_deref() == Some("stderr") {
                d.start(pulldown_cmark::Tag::Paragraph);
                d.styled("stderr:", Look::Emphasis);
                d.end(pulldown_cmark::TagEnd::Paragraph);
            }
            d.code_block("", &plain(text, cut));
        }
        "error" | "pyerr" => {
            d.start(pulldown_cmark::Tag::Paragraph);
            d.styled(
                &format!(
                    "{}: {}",
                    out.ename.as_deref().unwrap_or("Error"),
                    out.evalue.as_deref().unwrap_or("")
                ),
                Look::Strong,
            );
            d.end(pulldown_cmark::TagEnd::Paragraph);
            if let Some(tb) = &out.traceback {
                let joined = tb.join("\n");
                let (text, cut) = capped(&joined);
                d.code_block("", &plain(text, cut));
            }
        }
        _ => {
            d.start(pulldown_cmark::Tag::Paragraph);
            d.styled(&label, Look::Strong);
            d.end(pulldown_cmark::TagEnd::Paragraph);
            let data = out.data.as_ref();
            let markdown_text = data.and_then(|x| x.markdown.as_deref());
            let plain_text = data
                .and_then(|x| x.plain.as_deref())
                .or(out.text.as_ref().map(|t| t.0.as_str()));
            if let Some(md) = markdown_text {
                let (md, _) = capped(md);
                d.extend(markdown::events(md));
            } else if let Some(text) = plain_text {
                let (text, cut) = capped(text);
                d.code_block("", &plain(text, cut));
            }
            let mut others: Vec<(String, usize)> =
                data.map(|x| x.others.clone()).unwrap_or_default();
            for (name, size) in [
                ("image/png", out.png),
                ("image/jpeg", out.jpeg),
                ("text/html", out.html),
                ("image/svg+xml", out.svg),
                ("text/latex", out.latex),
                ("application/javascript", out.javascript),
            ] {
                if let Some(s) = size {
                    others.push((name.to_string(), s.0));
                }
            }
            for (mime, size) in others {
                d.paragraph(&format!("[{mime}, {}: not shown]", kib(size)));
            }
        }
    }
}

/// An output's text without its ANSI sequences (tracebacks are colored).
fn plain(text: &str, cut: bool) -> String {
    let mut s = super::log::strip_ansi(text).text;
    if cut {
        s.push_str("\n… (cut at 1 MiB)");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    const NB: &str = r##"{
 "nbformat": 4, "nbformat_minor": 5,
 "metadata": {"kernelspec": {"display_name": "Python 3", "language": "python"},
              "language_info": {"name": "python"}},
 "cells": [
  {"cell_type": "markdown", "source": ["# Title\n", "Some **bold** text"]},
  {"cell_type": "code", "execution_count": 3, "source": "print(1)",
   "outputs": [
     {"output_type": "stream", "name": "stdout", "text": ["1\n"]},
     {"output_type": "execute_result", "execution_count": 3,
      "data": {"text/plain": ["42"], "image/png": "iVBORw0KGgo=", "text/html": ["<script>alert(1)</script>"]}},
     {"output_type": "error", "ename": "ValueError", "evalue": "bad",
      "traceback": ["\u001b[31mValueError\u001b[0m: bad"]}
   ]},
  {"cell_type": "raw", "source": "raw **text**"}
 ]
}"##;

    fn lines(doc: &Doc) -> String {
        (0..doc.line_count())
            .map(|i| doc.line(i).to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn a_notebook_cell_by_cell() {
        let doc = render(NB, Duration::from_secs(3), 1 << 24, 80).unwrap();
        let text = lines(&doc);
        assert!(
            text.contains("Python 3 · 3 cells: 1 Markdown, 1 code, 1 raw · nbformat 4.5"),
            "{text}"
        );
        assert!(text.contains("Title"));
        assert!(!text.contains("**bold**"), "Markdown cells are rendered");
        assert!(text.contains("In [3]:"));
        assert!(text.contains("print(1)"));
        assert!(text.contains("Out [3]:"));
        assert!(text.contains("42"));
        assert!(text.contains("[image/png, 12 bytes: not shown]"), "{text}");
        assert!(text.contains("[text/html, 25 bytes: not shown]"));
        assert!(!text.contains("<script>"), "HTML is never shown");
        assert!(text.contains("ValueError: bad"));
        assert!(
            !text.contains('\u{1b}'),
            "the traceback's colors are taken out"
        );
        assert!(text.contains("raw **text**"), "a raw cell is literal");
    }

    #[test]
    fn what_is_not_a_notebook_is_its_source() {
        let t = Duration::from_secs(3);
        assert!(render("{", t, 1 << 20, 80).is_err());
        assert!(render("[1, 2]", t, 1 << 20, 80).is_err());
        assert!(render("{}", t, 1 << 20, 80).is_err());
        assert!(render(r#"{"nbformat": 9, "cells": []}"#, t, 1 << 20, 80).is_err());
        assert!(
            render(
                r#"{"nbformat": 4, "cells": [{"cell_type": 3}]}"#,
                t,
                1 << 20,
                80
            )
            .is_err()
        );
        // nbformat 3, with worksheets.
        let v3 = r#"{"nbformat": 3, "worksheets": [{"cells": [{"cell_type": "code", "input": "x = 1", "prompt_number": 1, "outputs": [{"output_type": "pyout", "text": "1", "png": "AAAA"}]}]}]}"#;
        let doc = render(v3, t, 1 << 20, 80).unwrap();
        let text = lines(&doc);
        assert!(
            text.contains("x = 1")
                && text.contains("In [1]:")
                && text.contains("[image/png, 4 bytes: not shown]"),
            "{text}"
        );
    }

    proptest::proptest! {
        #[test]
        fn hostile_notebooks_never_panic(
            kinds in proptest::collection::vec(0usize..4, 0..20),
            texts in proptest::collection::vec("[a-z#*`\\[\\]()<>\u{1b}\u{202e} \n]{0,40}", 0..20),
        ) {
            let kind = ["markdown", "code", "raw", "other"];
            let cells: Vec<String> = kinds
                .iter()
                .zip(texts.iter().chain(std::iter::repeat(&String::new())))
                .map(|(&k, t)| {
                    let t = serde_json::to_string(t).unwrap();
                    format!(r#"{{"cell_type": "{}", "source": {t}, "outputs": [{{"output_type": "stream", "text": {t}}}, {{"output_type": "error", "traceback": [{t}]}}]}}"#, kind[k])
                })
                .collect();
            let nb = format!(r#"{{"nbformat": 4, "cells": [{}]}}"#, cells.join(","));
            let doc = render(&nb, Duration::from_secs(3), 1 << 22, 40);
            proptest::prop_assert!(doc.is_ok());
        }
    }
}
