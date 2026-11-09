//! Where layouts come from, and which ones Xylem may use by itself.
//!
//! Trusted, used by Autoview and by V when the first line says what the file is:
//! the catalog built into the program (the `layouts/` folder of the source, so
//! that it cannot differ from the version running) and the user's own folder,
//! `~/.config/xylem/layouts/`. Nothing else is trusted: a layout next to the data
//! (or anywhere a path leads, F2) is read only when the user picks it with K, and
//! then as strictly as the data, since whoever sent the file may have sent it too.

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::SystemTime;

use super::xlayout::{self, At, Field, Format, Kind, Layout, Record, Role, Trim, Unit};
use crate::charset::Charset;

// The catalog: every file of `layouts/`, by its path there (`CATALOG`), listed
// by build.rs. The catalog test checks that every one is valid.
include!(concat!(env!("OUT_DIR"), "/layout_catalog.rs"));

/// Files of the user's folder looked at, and how deep.
const MAX_USER_FILES: usize = 256;
const MAX_USER_DEPTH: usize = 3;
/// Files listed from the folder of the data, in K.
const MAX_HERE: usize = 200;
/// The part of the first line that detection looks at.
const DETECT_BYTES: usize = 64 * 1024;

/// Where the layout in use came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    Catalog(&'static str),
    User(PathBuf),
    /// Picked by the user from anywhere: not trusted.
    File(PathBuf),
    /// Made from the fields of a `.dbf`, for its SDF export.
    Dbf(PathBuf),
}

impl Origin {
    pub fn describe(&self) -> String {
        match self {
            Origin::Catalog(p) => format!("Xylem's catalog: {p}"),
            Origin::User(p) | Origin::File(p) => p.display().to_string(),
            Origin::Dbf(p) => format!("fields of {}", p.display()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Loaded {
    pub layout: Arc<Layout>,
    pub origin: Origin,
}

fn catalog() -> &'static [(&'static str, Arc<Layout>)] {
    static PARSED: OnceLock<Vec<(&'static str, Arc<Layout>)>> = OnceLock::new();
    PARSED.get_or_init(|| {
        CATALOG
            .iter()
            .filter_map(|(path, text)| Some((*path, Arc::new(xlayout::parse(text).ok()?))))
            .collect()
    })
}

/// The user's layouts folder.
pub fn user_dir() -> Option<PathBuf> {
    Some(crate::platform::config_dir()?.join("layouts"))
}

fn is_layout_name(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.to_ascii_lowercase().ends_with(".xlayout.toml"))
}

/// Reads a layout file, with the size limit before anything else.
pub fn load_file(path: &Path) -> Result<Layout, String> {
    let file = File::open(path).map_err(|e| format!("cannot open the layout: {e}"))?;
    let meta = file.metadata().map_err(|e| e.to_string())?;
    if !meta.is_file() {
        return Err("the layout is not a file".into());
    }
    let mut bytes = Vec::new();
    file.take(xlayout::MAX_LAYOUT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > xlayout::MAX_LAYOUT_BYTES {
        return Err(format!(
            "the layout is larger than {} KiB",
            xlayout::MAX_LAYOUT_BYTES >> 10
        ));
    }
    let text = String::from_utf8(bytes).map_err(|_| "the layout is not UTF-8 text".to_string())?;
    xlayout::parse(&text)
}

type Parsed = Result<Arc<Layout>, String>;
/// A user's layout file as last read: its size and time, and what it gave.
type Cached = (u64, Option<SystemTime>, Parsed);

/// The user's layouts, read again only when a file changes.
fn user_layouts() -> Vec<(PathBuf, Parsed)> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, Cached>>> = OnceLock::new();
    let Some(dir) = user_dir() else {
        return Vec::new();
    };
    let mut found = Vec::new();
    walk(&dir, 0, &mut found);
    let mut cache = CACHE
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    cache.retain(|p, _| found.iter().any(|(f, _)| f == p));
    found
        .into_iter()
        .map(|(path, meta)| {
            let stamp = (meta.len(), meta.modified().ok());
            let parsed = match cache.get(&path) {
                Some((len, time, parsed)) if (*len, *time) == stamp => parsed.clone(),
                _ => {
                    let parsed = load_file(&path).map(Arc::new);
                    cache.insert(path.clone(), (stamp.0, stamp.1, parsed.clone()));
                    parsed
                }
            };
            (path, parsed)
        })
        .collect()
}

fn walk(dir: &Path, depth: usize, found: &mut Vec<(PathBuf, std::fs::Metadata)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
        if found.len() >= MAX_USER_FILES {
            return;
        }
        let Ok(meta) = std::fs::metadata(&path) else {
            continue;
        };
        if meta.is_dir() && depth + 1 < MAX_USER_DEPTH {
            walk(&path, depth + 1, found);
        } else if meta.is_file() && is_layout_name(&path) {
            found.push((path, meta));
        }
    }
}

/// The trusted layouts: the user's first, so that one can stand in for the catalog's.
fn trusted() -> Vec<Loaded> {
    let mut all: Vec<Loaded> = user_layouts()
        .into_iter()
        .filter_map(|(path, parsed)| {
            Some(Loaded {
                layout: parsed.ok()?,
                origin: Origin::User(path),
            })
        })
        .collect();
    all.extend(catalog().iter().map(|(path, layout)| Loaded {
        layout: layout.clone(),
        origin: Origin::Catalog(path),
    }));
    all
}

/// The trusted layout whose `[detect]` the start of the file fits, the one with
/// most conditions when several do. `start` is the start of the file as read.
pub fn detect(path: &Path, start: &[u8], charset: Charset) -> Option<Loaded> {
    if matches!(charset, Charset::Utf16Le | Charset::Utf16Be) {
        return None;
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    let start = &start[..start.len().min(DETECT_BYTES)];
    let first = start.split(|&b| b == b'\n').next().unwrap_or_default();
    let first = first.strip_suffix(b"\r").unwrap_or(first);
    let first = if charset == Charset::Utf8 {
        first.strip_prefix(&[0xEF, 0xBB, 0xBF][..]).unwrap_or(first)
    } else {
        first
    };
    if first.is_empty() {
        return None;
    }
    let mut best: Option<(usize, Loaded)> = None;
    for loaded in trusted() {
        let Some(d) = &loaded.layout.detect else {
            continue;
        };
        if !d.extensions.is_empty() && !d.extensions.contains(&ext) {
            continue;
        }
        if !super::first_line_fits(&loaded.layout, first, charset, d.line_length, &d.first_line) {
            continue;
        }
        let score = d.first_line.len() + usize::from(d.line_length.is_some());
        if best.as_ref().is_none_or(|(s, _)| score > *s) {
            best = Some((score, loaded));
        }
    }
    best.map(|(_, l)| l)
}

/// A group of the list of K.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// Turns the layout off.
    Off,
    Here,
    Mine,
    Brazil,
    World,
}

impl Section {
    pub fn label(self) -> &'static str {
        match self {
            Section::Off => "",
            Section::Here => "Here",
            Section::Mine => "Mine",
            Section::Brazil => "Brazil",
            Section::World => "World",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Off,
    Catalog(&'static str),
    User(PathBuf),
    File(PathBuf),
    Dbf(PathBuf),
}

#[derive(Debug, Clone)]
pub struct Choice {
    pub section: Section,
    pub label: String,
    pub detail: String,
    pub source: Source,
}

/// What K offers for the file at `data`: turning the layout off (when one is in
/// use), what is next to the file, the user's layouts and the catalog's.
pub fn choices(data: &Path, active: bool) -> Vec<Choice> {
    let mut out = Vec::new();
    if active {
        out.push(Choice {
            section: Section::Off,
            label: "No layout: plain text".into(),
            detail: String::new(),
            source: Source::Off,
        });
    }
    let stem = data
        .file_stem()
        .and_then(|s| s.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    if let Some(dir) = data.parent()
        && let Ok(entries) = std::fs::read_dir(if dir.as_os_str().is_empty() {
            Path::new(".")
        } else {
            dir
        })
    {
        let mut here: Vec<(bool, String, Source)> = Vec::new();
        for entry in entries.flatten().take(10_000) {
            if here.len() >= MAX_HERE {
                break;
            }
            let path = entry.path();
            let Some(name) = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(str::to_string)
            else {
                continue;
            };
            let lower = name.to_ascii_lowercase();
            let source = if lower.ends_with(".xlayout.toml") {
                Source::File(path.clone())
            } else if lower.ends_with(".dbf") {
                Source::Dbf(path.clone())
            } else {
                continue;
            };
            if !entry.file_type().is_ok_and(|t| t.is_file()) {
                continue;
            }
            let same = !stem.is_empty() && lower.starts_with(&format!("{stem}."));
            here.push((same, name, source));
        }
        here.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        for (same, name, source) in here {
            let detail = match (&source, same) {
                (Source::Dbf(_), _) => "SDF: the fields of this .dbf".to_string(),
                (_, true) => "same name as the file".to_string(),
                _ => String::new(),
            };
            out.push(Choice {
                section: Section::Here,
                label: name,
                detail,
                source,
            });
        }
    }
    let user = user_dir();
    for (path, parsed) in user_layouts() {
        let shown = user
            .as_ref()
            .and_then(|u| path.strip_prefix(u).ok())
            .unwrap_or(&path)
            .display()
            .to_string();
        let (label, detail) = match &parsed {
            Ok(l) => (l.name.clone(), shown),
            Err(e) => (shown, format!("not valid: {e}")),
        };
        out.push(Choice {
            section: Section::Mine,
            label,
            detail,
            source: Source::User(path),
        });
    }
    for (path, layout) in catalog() {
        let section = if layout.country == "BR" {
            Section::Brazil
        } else {
            Section::World
        };
        let mut detail = layout.category.clone();
        if !layout.verified {
            detail.push_str(" · draft");
        }
        out.push(Choice {
            section,
            label: layout.name.clone(),
            detail,
            source: Source::Catalog(path),
        });
    }
    out
}

/// Loads what was picked in K (`None` for turning the layout off).
pub fn load(source: &Source) -> Result<Option<Loaded>, String> {
    Ok(Some(match source {
        Source::Off => return Ok(None),
        Source::Catalog(path) => Loaded {
            layout: catalog()
                .iter()
                .find(|(p, _)| p == path)
                .map(|(_, l)| l.clone())
                .ok_or("not in the catalog")?,
            origin: Origin::Catalog(path),
        },
        Source::User(path) => Loaded {
            layout: Arc::new(load_file(path)?),
            origin: Origin::User(path.clone()),
        },
        Source::File(path) => Loaded {
            layout: Arc::new(load_file(path)?),
            origin: Origin::File(path.clone()),
        },
        Source::Dbf(path) => Loaded {
            layout: Arc::new(sdf_from_dbf(path)?),
            origin: Origin::Dbf(path.clone()),
        },
    }))
}

/// Loads a path typed in K (F2): a `.dbf` gives its SDF layout.
pub fn load_path(path: &Path) -> Result<Loaded, String> {
    let dbf = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("dbf"));
    let source = if dbf {
        Source::Dbf(path.to_path_buf())
    } else {
        Source::File(path.to_path_buf())
    };
    load(&source).map(|l| l.expect("a file always gives a layout"))
}

/// The layout of what `COPY TO ... SDF` writes from a `.dbf`: its fields side by
/// side with their widths, dates as YYYYMMDD and numbers with a point. Memo
/// fields are not exported, and the other binary types have no fixed text form.
pub fn sdf_from_dbf(path: &Path) -> Result<Layout, String> {
    use dbase::FieldType as T;
    let reader =
        dbase::Reader::from_path(path).map_err(|e| format!("cannot read the .dbf: {e}"))?;
    let infos = reader.fields();
    let decimals = super::super::records::dbase_decimals(path, infos);
    let mut fields = Vec::new();
    let mut skipped = Vec::new();
    let mut next = 1usize;
    let mut names = std::collections::HashSet::new();
    for (i, info) in infos.iter().enumerate().take(xlayout::MAX_FIELDS) {
        let width = usize::from(info.length());
        let (kind, decimals) = match info.field_type() {
            T::Character | T::Logical => (Kind::Text, 0),
            T::Date => (Kind::Date, 0),
            T::Numeric | T::Float => match decimals.get(i).copied().flatten().unwrap_or(0) {
                0 => (Kind::Int, 0),
                d => (Kind::Decimal, u32::from(d).min(xlayout::MAX_DECIMALS)),
            },
            other => {
                skipped.push(format!("{} ({other:?})", info.name()));
                continue;
            }
        };
        let width = if kind == Kind::Date { 8 } else { width.max(1) };
        let end = next + width - 1;
        if end > xlayout::MAX_RECORD_BYTES {
            return Err("the .dbf records are too wide".into());
        }
        let mut name: String = info.name().chars().filter(|c| !c.is_control()).collect();
        if name.trim().is_empty() {
            name = format!("FIELD{}", i + 1);
        }
        while !names.insert(name.to_lowercase()) {
            name.push('_');
        }
        fields.push(Field {
            name,
            at: At::Pos { start: next, end },
            kind,
            trim: Trim::Right,
            sign: xlayout::Sign::None,
            decimals,
            decimal_sep: xlayout::DecimalSep::Dot,
            date: xlayout::DateFormat::Yyyymmdd,
            time: xlayout::TimeFormat::Hhmmss,
            desc: format!("{:?}, {} wide", info.field_type(), info.length()),
            values: Vec::new(),
            check: None,
        });
        next = end + 1;
    }
    if fields.is_empty() {
        return Err("the .dbf has no field that SDF exports".into());
    }
    let mut desc = format!("{} characters per line", next - 1);
    if !skipped.is_empty() {
        desc.push_str(&format!(", without {}", skipped.join(", ")));
    }
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    Ok(Layout {
        id: String::new(),
        name: format!("SDF of {name}"),
        description: String::new(),
        country: String::new(),
        category: "sdf".into(),
        spec_source: String::new(),
        spec_version: String::new(),
        spec_url: String::new(),
        verified: false,
        format: Format::Positional { unit: Unit::Chars },
        encoding: None,
        hide_filler: true,
        stop_after_trailer: false,
        detect: None,
        discriminator: None,
        records: vec![Record {
            id: "SDF".into(),
            name: "Records".into(),
            role: Role::Detail,
            codes: Vec::new(),
            matches: Vec::new(),
            batch_match: Vec::new(),
            line_length: Some(next - 1),
            min_length: None,
            desc,
            fields,
        }],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_layouts() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("layouts")
    }

    fn files_under(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                files_under(&path, out);
            } else if is_layout_name(&path) {
                out.push(path);
            }
        }
    }

    /// Every layout of `layouts/` is built in, valid, and says where it comes from.
    #[test]
    fn the_catalog_is_complete_and_valid() {
        let root = repo_layouts();
        let mut files = Vec::new();
        files_under(&root, &mut files);
        assert!(!files.is_empty());
        for file in &files {
            let rel = file
                .strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            assert!(
                CATALOG.iter().any(|(p, _)| *p == rel),
                "{rel} is in layouts/ but not in CATALOG"
            );
        }
        assert_eq!(
            files.len(),
            CATALOG.len(),
            "CATALOG lists a file that is not in layouts/"
        );
        let mut ids = std::collections::HashSet::new();
        for (path, text) in CATALOG {
            let layout = xlayout::parse(text).unwrap_or_else(|e| panic!("{path}: {e}"));
            assert!(!layout.id.is_empty(), "{path}: no id");
            assert!(ids.insert(layout.id.clone()), "{path}: id used twice");
            assert_eq!(
                path.trim_end_matches(".xlayout.toml"),
                layout.id,
                "{path}: the id is the path"
            );
            for (what, value) in [
                ("country", &layout.country),
                ("category", &layout.category),
                ("spec_source", &layout.spec_source),
                ("spec_version", &layout.spec_version),
                ("spec_url", &layout.spec_url),
            ] {
                assert!(!value.is_empty(), "{path}: no {what}");
            }
            assert!(
                layout.detect.is_some(),
                "{path}: catalog layouts are found by [detect]"
            );
            assert!(
                !text.contains("strict_positions = false"),
                "{path}: the catalog's positions are strict"
            );
        }
        assert_eq!(catalog().len(), CATALOG.len());
    }

    #[test]
    fn detection_finds_the_catalog_layouts() {
        let mut header = vec![b' '; 240];
        for (at, text) in [(1, "001"), (4, "0000"), (8, "0"), (143, "1"), (164, "103")] {
            header[at - 1..at - 1 + text.len()].copy_from_slice(text.as_bytes());
        }
        let found = detect(Path::new("x.rem"), &header, Charset::Utf8).unwrap();
        assert_eq!(found.layout.id, "br/bancario/cnab240-febraban");
        header[163] = b'x';
        assert!(detect(Path::new("x.rem"), &header, Charset::Utf8).is_none());
        let sped = b"|0000|019|0|01012026|31012026|EMPRESA|11222333000181||SP|123|3550308|||A|1|\r\n|0001|0|\r\n";
        let found = detect(Path::new("efd.txt"), sped, Charset::Cp1252).unwrap();
        assert_eq!(found.layout.id, "br/fiscal/sped-generico");
        assert!(detect(Path::new("notes.txt"), b"just some text\n", Charset::Utf8).is_none());
        assert!(detect(Path::new("x.txt"), b"|0000\n", Charset::Utf8).is_none());
    }
}
