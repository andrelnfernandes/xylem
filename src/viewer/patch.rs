//! Patches and diffs (docs/plans/viewer-patch.md): their colors, line by
//! line, and an index of the files and hunks, for `]` `[` `}` `{` and Tab.
//!
//! A patch is hostile like any file: the counts in a hunk's `@@` line are only
//! compared with the lines that follow, never trusted to read or to reserve
//! memory, and the names in it go through the viewer's cleaning as any text.

use std::path::Path;
use std::sync::Arc;

use ratatui::style::Color;

use super::lines::LineSource;
use crate::highlight::{self, Highlighter, LineColors};

/// Lines looked at to tell a patch by its contents.
const SNIFF_LINES: usize = 200;
/// Most files and hunks kept in the index; past these it is cut, and says so.
const MAX_FILES: usize = 1_000_000;
const MAX_HUNKS: usize = 10_000_000;
/// Most lines marked as changed only in what cannot be seen.
const MAX_UNCHANGED: usize = 1_000_000;
/// Characters of a line given a color: past them the line has the text color.
const COLORED_CHARS: usize = 10_000;

/// A `.patch` or `.diff`, by the name.
pub fn is_patch_name(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("patch") || e.eq_ignore_ascii_case("diff"))
}

/// A patch by its contents: a `diff --git`, or `---`, `+++` and `@@` in a
/// row, in its first lines.
pub fn looks_like_patch(lines: &(impl LineSource + ?Sized)) -> bool {
    let n = lines.count().min(SNIFF_LINES);
    (0..n).any(|i| {
        let line = lines.line(i);
        line.starts_with("diff --git ")
            || (line.starts_with("--- ")
                && i + 2 < lines.count()
                && lines.line(i + 1).starts_with("+++ ")
                && lines.line(i + 2).starts_with("@@ "))
    })
}

/// What a file of the patch does besides changing lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Status {
    #[default]
    Modified,
    New,
    Deleted,
    Renamed,
    Copied,
    Binary,
    /// A message of `git format-patch`, with its subject as the name.
    Mail,
}

/// One file (or one message) of the patch.
#[derive(Debug, Clone, Default)]
pub struct PatchFile {
    /// The line where it starts.
    pub line: usize,
    pub name: String,
    pub status: Status,
    pub added: usize,
    pub removed: usize,
    /// It becomes executable, a symbolic link or a submodule, or its path is
    /// absolute, climbs out (`..`), or goes into `.git`.
    pub danger: bool,
}

/// Where the files and hunks of a patch start, and what it changes.
#[derive(Debug, Clone, Default)]
pub struct PatchIndex {
    pub files: Vec<PatchFile>,
    /// The `@@` line of each hunk.
    pub hunks: Vec<usize>,
    pub added: usize,
    pub removed: usize,
    /// Hunks whose `@@` counts do not match their lines.
    pub lying_hunks: usize,
    /// Added lines with a space at the end, or a space before a tab in the
    /// indentation (as `git diff --check`).
    pub whitespace: usize,
    /// Lines (sorted) of removed and added pairs that differ only in what
    /// cannot be seen: spaces, `\r`, look-alike letters, invisible characters.
    pub unchanged: Vec<usize>,
    /// The index stopped at one of its limits.
    pub cut: bool,
}

impl PatchIndex {
    /// The summary for the header: `12 files, +340 −98`.
    pub fn summary(&self) -> String {
        let files = self
            .files
            .iter()
            .filter(|f| f.status != Status::Mail)
            .count();
        let mut s = format!(
            "{} file{}, +{} −{}",
            files,
            if files == 1 { "" } else { "s" },
            self.added,
            self.removed
        );
        let dangers = self.files.iter().filter(|f| f.danger).count();
        if dangers > 0 {
            s.push_str(&format!(", {dangers} to check"));
        }
        if !self.unchanged.is_empty() {
            s.push_str(&format!(", {} look unchanged", self.unchanged.len() / 2));
        }
        if self.lying_hunks > 0 {
            s.push_str(&format!(", {} hunks with wrong counts", self.lying_hunks));
        }
        if self.whitespace > 0 {
            s.push_str(&format!(", {} whitespace", self.whitespace));
        }
        if self.cut {
            s.push_str(", index cut");
        }
        s
    }

    /// The file a line belongs to (its index in `files`).
    pub fn file_at(&self, line: usize) -> Option<usize> {
        self.files
            .partition_point(|f| f.line <= line)
            .checked_sub(1)
    }

    /// The hunk a line belongs to, within its file: (number from 1, count).
    pub fn hunk_at(&self, line: usize) -> Option<(usize, usize)> {
        let file = self.file_at(line)?;
        let start = self.files[file].line;
        let end = self.files.get(file + 1).map_or(usize::MAX, |f| f.line);
        let first = self.hunks.partition_point(|&h| h < start);
        let last = self.hunks.partition_point(|&h| h < end);
        let here = self.hunks.partition_point(|&h| h <= line);
        (here > first).then_some((here - first, last - first))
    }

    /// The first start in `starts` after `line` (forward), or the last one
    /// before it.
    pub fn next(
        starts: impl DoubleEndedIterator<Item = usize>,
        line: usize,
        forward: bool,
    ) -> Option<usize> {
        if forward {
            starts.into_iter().find(|&s| s > line)
        } else {
            starts.into_iter().rev().find(|&s| s < line)
        }
    }

    /// The names for the list (Tab): `+3 −1  src/a.rs`, with what it does.
    pub fn names(&self) -> Vec<String> {
        self.files
            .iter()
            .map(|f| {
                let what = match f.status {
                    Status::Modified => "",
                    Status::New => " (new)",
                    Status::Deleted => " (deleted)",
                    Status::Renamed => " (renamed)",
                    Status::Copied => " (copied)",
                    Status::Binary => " (binary)",
                    Status::Mail => "",
                };
                let danger = if f.danger { "  ⚠ check" } else { "" };
                let name = shown(&f.name);
                if f.status == Status::Mail {
                    format!("✉ {name}")
                } else {
                    format!("+{} −{}  {name}{what}{danger}", f.added, f.removed)
                }
            })
            .collect()
    }
}

/// A name as it is shown: control characters as `·`, like the text.
pub fn shown(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_control() { '·' } else { c })
        .collect()
}

/// The name in a header line (`a/src/x.rs`, `"a/\303\251"`, `/dev/null`),
/// without its `a/` or `b/`, and without what follows a tab (a date, in
/// `diff -u`).
fn header_name(raw: &str) -> String {
    let raw = raw.split('\t').next().unwrap_or("");
    let name = if raw.starts_with('"') {
        unquote(raw)
    } else {
        raw.trim_end().to_string()
    };
    match name.strip_prefix("a/").or_else(|| name.strip_prefix("b/")) {
        Some(rest) => rest.to_string(),
        None => name,
    }
}

/// A name as git quotes it: `"a/\303\251 x.txt"` (C escapes, octal bytes).
/// Invalid escapes are kept as they are; the bytes are read as UTF-8, lossily.
fn unquote(raw: &str) -> String {
    let inner = raw.strip_prefix('"').unwrap_or(raw);
    let inner = inner.strip_suffix('"').unwrap_or(inner);
    let bytes = inner.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'\\' || i + 1 == bytes.len() {
            out.push(bytes[i]);
            i += 1;
            continue;
        }
        let c = bytes[i + 1];
        let simple = match c {
            b'n' => Some(b'\n'),
            b't' => Some(b'\t'),
            b'r' => Some(b'\r'),
            b'a' => Some(7),
            b'b' => Some(8),
            b'f' => Some(12),
            b'v' => Some(11),
            b'\\' => Some(b'\\'),
            b'"' => Some(b'"'),
            _ => None,
        };
        if let Some(b) = simple {
            out.push(b);
            i += 2;
            continue;
        }
        let octal: Vec<u8> = bytes[i + 1..]
            .iter()
            .take(3)
            .take_while(|b| (b'0'..=b'7').contains(b))
            .copied()
            .collect();
        if octal.len() == 3 {
            let v = octal.iter().fold(0u32, |v, d| v * 8 + u32::from(d - b'0'));
            out.push((v & 0xff) as u8);
            i += 4;
        } else {
            out.push(b'\\');
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The two names of `diff --git a/x b/y`. With spaces in the names and no
/// quotes, the split is where the two halves are the same name, as git does.
fn git_names(rest: &str) -> (String, String) {
    if let Some(inner) = rest.strip_prefix('"') {
        // `"a/x y" "b/x y"` or `"a/x" b/x`.
        if let Some(c) = inner.find("\" ").map(|i| i + 1) {
            return (header_name(&rest[..=c]), header_name(rest[c + 2..].trim()));
        }
    }
    for (i, _) in rest.match_indices(" b/") {
        let (a, b) = (&rest[..i], &rest[i + 1..]);
        if a.strip_prefix("a/") == b.strip_prefix("b/") {
            return (header_name(a), header_name(b));
        }
    }
    match rest.rsplit_once(' ') {
        Some((a, b)) => (header_name(a), header_name(b)),
        None => (header_name(rest), header_name(rest)),
    }
}

/// A path that a patch should not touch without being looked at: absolute,
/// climbing out with `..`, into `.git`, or with control characters.
pub fn dangerous_path(name: &str) -> bool {
    if name == "/dev/null" || name.is_empty() {
        return false;
    }
    let windows_drive = name.len() > 1 && name.as_bytes()[1] == b':';
    name.starts_with('/')
        || name.starts_with('\\')
        || windows_drive
        || name.chars().any(char::is_control)
        || name
            .split(['/', '\\'])
            .any(|part| part == ".." || part.eq_ignore_ascii_case(".git"))
}

/// A file mode worth seeing: executable, a symbolic link, a submodule.
fn dangerous_mode(line: &str) -> bool {
    let mode = line.rsplit(' ').next().unwrap_or("");
    let mode = mode.trim();
    mode == "120000" || mode == "160000" || (mode.len() == 6 && mode.ends_with("755"))
}

/// The counts of `@@ -a,b +c,d @@`: (b, d), each 1 when left out. `None` when
/// the line is not a hunk header.
fn hunk_counts(line: &str) -> Option<(usize, usize)> {
    let rest = line.strip_prefix("@@ -")?;
    let (old, rest) = rest.split_once(" +")?;
    let new = rest.split(' ').next()?;
    let count = |r: &str| -> Option<usize> {
        match r.split_once(',') {
            Some((start, n)) => {
                start.parse::<u64>().ok()?;
                // A count too big for usize is still a count: it will not
                // match, and the hunk is marked.
                Some(
                    n.parse::<u64>()
                        .ok()
                        .map_or(usize::MAX, |n| usize::try_from(n).unwrap_or(usize::MAX)),
                )
            }
            None => {
                r.parse::<u64>().ok()?;
                Some(1)
            }
        }
    };
    Some((count(old)?, count(new)?))
}

/// The start of a message of `git format-patch`: `From <40 hex> ...`.
fn mail_start(line: &str) -> bool {
    line.strip_prefix("From ")
        .and_then(|r| r.split(' ').next())
        .is_some_and(|h| h.len() == 40 && h.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// What is left of a line once what cannot be seen is gone: spaces, `\r`,
/// invisible characters, and letters that look like ASCII ones.
fn seen(line: &str) -> String {
    line.chars()
        .filter(|c| !c.is_whitespace() && !invisible(*c))
        .map(look_alike)
        .collect()
}

fn invisible(c: char) -> bool {
    matches!(
        c,
        '\u{200B}'..='\u{200F}' | '\u{2060}'..='\u{2064}' | '\u{FEFF}' | '\u{00AD}' | '\u{034F}'
            | '\u{180E}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'
    )
}

/// The ASCII letter a character passes for, as in a homograph.
fn look_alike(c: char) -> char {
    match c {
        // Cyrillic.
        'а' => 'a',
        'е' => 'e',
        'о' => 'o',
        'р' => 'p',
        'с' => 'c',
        'у' => 'y',
        'х' => 'x',
        'і' => 'i',
        'ј' => 'j',
        'ѕ' => 's',
        'ԁ' => 'd',
        'ԛ' => 'q',
        'ԝ' => 'w',
        'А' => 'A',
        'В' => 'B',
        'Е' => 'E',
        'К' => 'K',
        'М' => 'M',
        'Н' => 'H',
        'О' => 'O',
        'Р' => 'P',
        'С' => 'C',
        'Т' => 'T',
        'Х' => 'X',
        'І' => 'I',
        'Ј' => 'J',
        'Ѕ' => 'S',
        // Greek.
        'ο' => 'o',
        'ν' => 'v',
        'Α' => 'A',
        'Β' => 'B',
        'Ε' => 'E',
        'Ζ' => 'Z',
        'Η' => 'H',
        'Ι' => 'I',
        'Κ' => 'K',
        'Μ' => 'M',
        'Ν' => 'N',
        'Ο' => 'O',
        'Ρ' => 'P',
        'Τ' => 'T',
        'Υ' => 'Y',
        'Χ' => 'X',
        // Dashes and quotes.
        '\u{2010}'..='\u{2015}' | '\u{2212}' => '-',
        '\u{2018}' | '\u{2019}' | '\u{2032}' => '\'',
        '\u{201C}' | '\u{201D}' | '\u{2033}' => '"',
        // Full-width ASCII.
        '\u{FF01}'..='\u{FF5E}' => char::from_u32(c as u32 - 0xFF01 + 0x21).unwrap_or(c),
        _ => c,
    }
}

/// An added line with a space at its end or a space before a tab in its
/// indentation.
fn bad_whitespace(added: &str) -> bool {
    let body = added.strip_suffix('\r').unwrap_or(added);
    let indent: String = body
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect();
    body.ends_with(' ') || body.ends_with('\t') || indent.contains(" \t")
}

/// A hunk being read.
struct Hunk {
    old_left: usize,
    new_left: usize,
    lying: bool,
    /// The run of removed lines, then of added ones, for the look-alike check.
    removed: Vec<usize>,
    added: Vec<usize>,
}

/// Builds the index of a patch, in one pass over its lines.
pub fn index(lines: &(impl LineSource + ?Sized)) -> PatchIndex {
    let mut ix = PatchIndex::default();
    let mut hunk: Option<Hunk> = None;
    // A `diff --git` whose `---`/`+++` are still to come.
    let mut git_header = false;
    let n = lines.count();
    let mut i = 0;
    while i < n {
        let line = lines.line(i);
        if let Some(h) = &mut hunk {
            let exhausted = h.old_left == 0 && h.new_left == 0;
            let first = line.as_bytes().first().copied();
            // `-- ` ends the message of `git format-patch`, before git's version.
            let next_file = exhausted && (starts_file(lines, i) || line == "-- ");
            match first {
                Some(b'\\') => {
                    i += 1;
                    continue;
                }
                Some(b'+' | b'-' | b' ') | None
                    if !next_file && !(exhausted && first.is_none()) =>
                {
                    let line_kind = first.unwrap_or(b' ');
                    if exhausted {
                        h.lying = true;
                    }
                    match line_kind {
                        b'-' => {
                            if h.old_left == 0 {
                                h.lying = true;
                            }
                            h.old_left = h.old_left.saturating_sub(1);
                            ix.removed += 1;
                            if let Some(f) = ix.files.last_mut() {
                                f.removed += 1;
                            }
                            if !h.added.is_empty() {
                                pair_up(lines, h, &mut ix.unchanged);
                            }
                            h.removed.push(i);
                        }
                        b'+' => {
                            if h.new_left == 0 {
                                h.lying = true;
                            }
                            h.new_left = h.new_left.saturating_sub(1);
                            ix.added += 1;
                            if let Some(f) = ix.files.last_mut() {
                                f.added += 1;
                            }
                            if bad_whitespace(&line[1..]) {
                                ix.whitespace += 1;
                            }
                            if !h.removed.is_empty() {
                                h.added.push(i);
                            }
                        }
                        _ => {
                            if h.old_left == 0 || h.new_left == 0 {
                                h.lying = true;
                            }
                            h.old_left = h.old_left.saturating_sub(1);
                            h.new_left = h.new_left.saturating_sub(1);
                            pair_up(lines, h, &mut ix.unchanged);
                        }
                    }
                    i += 1;
                    continue;
                }
                _ => {
                    if !exhausted {
                        h.lying = true;
                    }
                    end_hunk(lines, &mut hunk, &mut ix);
                }
            }
        }
        if ix.cut {
            break;
        }
        if mail_start(line) {
            let subject = (i + 1..n.min(i + 40))
                .map(|j| lines.line(j))
                .take_while(|l| !l.is_empty())
                .find_map(|l| l.strip_prefix("Subject: "))
                .unwrap_or("(no subject)")
                .to_string();
            push_file(
                &mut ix,
                PatchFile {
                    line: i,
                    name: subject,
                    status: Status::Mail,
                    ..PatchFile::default()
                },
            );
            git_header = false;
        } else if let Some(rest) = line.strip_prefix("diff --git ") {
            let (a, b) = git_names(rest);
            let danger = dangerous_path(&a) || dangerous_path(&b);
            push_file(
                &mut ix,
                PatchFile {
                    line: i,
                    name: b,
                    danger,
                    ..PatchFile::default()
                },
            );
            git_header = true;
        } else if line.starts_with("--- ") && i + 1 < n && lines.line(i + 1).starts_with("+++ ") {
            let old = header_name(&line[4..]);
            let new = header_name(&lines.line(i + 1)[4..]);
            let danger = dangerous_path(&old) || dangerous_path(&new);
            let name = if new == "/dev/null" {
                old.clone()
            } else {
                new.clone()
            };
            if git_header {
                if let Some(f) = ix.files.last_mut() {
                    f.danger |= danger;
                }
            } else {
                push_file(
                    &mut ix,
                    PatchFile {
                        line: i,
                        name,
                        status: if old == "/dev/null" {
                            Status::New
                        } else if new == "/dev/null" {
                            Status::Deleted
                        } else {
                            Status::Modified
                        },
                        danger,
                        ..PatchFile::default()
                    },
                );
            }
            git_header = false;
            i += 2;
            continue;
        } else if let Some((old, new)) = hunk_counts(line) {
            git_header = false;
            if ix.hunks.len() >= MAX_HUNKS {
                ix.cut = true;
                break;
            }
            ix.hunks.push(i);
            hunk = Some(Hunk {
                old_left: old,
                new_left: new,
                lying: false,
                removed: Vec::new(),
                added: Vec::new(),
            });
        } else if let Some(f) = ix.files.last_mut().filter(|_| git_header) {
            if line.starts_with("new file mode") {
                f.status = Status::New;
            } else if line.starts_with("deleted file mode") {
                f.status = Status::Deleted;
            } else if line.starts_with("rename from") || line.starts_with("rename to") {
                f.status = Status::Renamed;
            } else if line.starts_with("copy from") || line.starts_with("copy to") {
                f.status = Status::Copied;
            } else if line.starts_with("GIT binary patch") || line.starts_with("Binary files ") {
                f.status = Status::Binary;
            }
            if mode_line(line) && dangerous_mode(line) {
                f.danger = true;
            }
            if let Some(name) = ["rename from ", "rename to ", "copy from ", "copy to "]
                .iter()
                .find_map(|p| line.strip_prefix(p))
            {
                f.danger |= dangerous_path(&header_name(name));
            }
        } else if line.starts_with("Binary files ")
            && let Some(f) = ix.files.last_mut()
        {
            f.status = Status::Binary;
        }
        i += 1;
    }
    end_hunk(lines, &mut hunk, &mut ix);
    ix.unchanged.sort_unstable();
    ix.unchanged.dedup();
    ix
}

fn mode_line(line: &str) -> bool {
    [
        "new mode ",
        "old mode ",
        "new file mode ",
        "deleted file mode ",
    ]
    .iter()
    .any(|p| line.starts_with(p))
}

/// A new file starts at line `i`: a `diff --git`, a `---`/`+++` pair or a
/// message.
fn starts_file(lines: &(impl LineSource + ?Sized), i: usize) -> bool {
    let line = lines.line(i);
    line.starts_with("diff --git ")
        || mail_start(line)
        || (line.starts_with("--- ")
            && i + 1 < lines.count()
            && lines.line(i + 1).starts_with("+++ "))
}

fn push_file(ix: &mut PatchIndex, file: PatchFile) {
    if ix.files.len() >= MAX_FILES {
        ix.cut = true;
    } else {
        ix.files.push(file);
    }
}

fn end_hunk(lines: &(impl LineSource + ?Sized), hunk: &mut Option<Hunk>, ix: &mut PatchIndex) {
    if let Some(mut h) = hunk.take() {
        pair_up(lines, &mut h, &mut ix.unchanged);
        if h.lying || h.old_left > 0 || h.new_left > 0 {
            ix.lying_hunks += 1;
        }
    }
}

/// A run of removed lines followed by as many added ones: the pairs that
/// differ only in what cannot be seen are marked.
fn pair_up(lines: &(impl LineSource + ?Sized), h: &mut Hunk, out: &mut Vec<usize>) {
    if h.removed.len() == h.added.len() && out.len() < MAX_UNCHANGED {
        for (&r, &a) in h.removed.iter().zip(&h.added) {
            let (old, new) = (&lines.line(r)[1..], &lines.line(a)[1..]);
            if old != new && seen(old) == seen(new) {
                out.push(r);
                out.push(a);
            }
        }
    }
    h.removed.clear();
    h.added.clear();
}

/// The colors of a patch's lines. With the `index` (a patch read whole), the
/// pairs that only look unchanged are marked too.
pub fn colors(index: Option<Arc<PatchIndex>>) -> Option<Highlighter> {
    let green = highlight::tone((78, 154, 6));
    let red = highlight::tone((204, 0, 0));
    let cyan = highlight::tone((6, 152, 154));
    let header = highlight::tone((114, 159, 207));
    let warn = highlight::tone((237, 212, 0));
    let danger = highlight::tone((239, 41, 41));
    let colors: LineColors = Box::new(move |i, line: &str| {
        let unchanged = index
            .as_ref()
            .is_some_and(|ix| ix.unchanged.binary_search(&i).is_ok());
        let color = line_color(line, unchanged, [green, red, cyan, header, warn, danger]);
        color
            .map(|c| vec![(c, line.chars().take(COLORED_CHARS).collect())])
            .unwrap_or_default()
    });
    Highlighter::per_line("Patch", colors)
}

/// The color of one line of a patch, from the line alone: `palette` is green,
/// red, cyan, header, warning and danger.
fn line_color(line: &str, unchanged: bool, palette: [Color; 6]) -> Option<Color> {
    let [green, red, cyan, header, warn, danger] = palette;
    if unchanged {
        return Some(warn);
    }
    if mode_line(line) {
        return Some(if dangerous_mode(line) { danger } else { header });
    }
    let path_of = |rest: &str| dangerous_path(&header_name(rest));
    if let Some(rest) = line.strip_prefix("diff --git ") {
        let (a, b) = git_names(rest);
        return Some(if dangerous_path(&a) || dangerous_path(&b) {
            danger
        } else {
            header
        });
    }
    for p in ["--- a/", "+++ b/", "--- /", "+++ /"] {
        if line.starts_with(p) {
            return Some(if path_of(&line[4..]) { danger } else { header });
        }
    }
    for p in ["rename from ", "rename to ", "copy from ", "copy to "] {
        if let Some(rest) = line.strip_prefix(p) {
            return Some(if path_of(rest) { danger } else { header });
        }
    }
    if line.starts_with("Subproject commit ")
        || line.starts_with("-Subproject commit ")
        || line.starts_with("+Subproject commit ")
    {
        return Some(danger);
    }
    if line.starts_with("GIT binary patch") || line.starts_with("Binary files ") {
        return Some(warn);
    }
    if line.starts_with("@@") {
        return Some(cyan);
    }
    if mail_start(line) || line.starts_with("Subject: ") {
        return Some(header);
    }
    if ["index ", "similarity index", "dissimilarity index"]
        .iter()
        .any(|p| line.starts_with(p))
    {
        return Some(header);
    }
    match line.as_bytes().first() {
        Some(b'+') => Some(green),
        Some(b'-') => Some(red),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(text: &str) -> Vec<String> {
        text.lines().map(str::to_string).collect()
    }

    const GIT: &str = "\
diff --git a/src/a.rs b/src/a.rs
index 1111111..2222222 100644
--- a/src/a.rs
+++ b/src/a.rs
@@ -1,3 +1,3 @@
 fn a() {
-    old();
+    new();
 }
@@ -10,2 +10,3 @@
 x
+y
 z
diff --git a/run.sh b/run.sh
old mode 100644
new mode 100755
diff --git a/new.txt b/new.txt
new file mode 100644
index 0000000..3333333
--- /dev/null
+++ b/new.txt
@@ -0,0 +1 @@
+hello
";

    #[test]
    fn indexes_files_hunks_and_counts() {
        let l = lines(GIT);
        assert!(looks_like_patch(l.as_slice()));
        let ix = index(l.as_slice());
        let names: Vec<_> = ix.files.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, ["src/a.rs", "run.sh", "new.txt"]);
        assert_eq!(ix.files[0].line, 0);
        assert_eq!((ix.files[0].added, ix.files[0].removed), (2, 1));
        assert_eq!(ix.hunks, [4, 9, 21]);
        assert_eq!((ix.added, ix.removed), (3, 1));
        assert!(ix.files[1].danger, "becomes executable");
        assert_eq!(ix.files[2].status, Status::New);
        assert_eq!(ix.lying_hunks, 0);
        assert_eq!(ix.summary(), "3 files, +3 −1, 1 to check");
        assert_eq!(ix.file_at(10), Some(0));
        assert_eq!(ix.hunk_at(10), Some((2, 2)));
        assert_eq!(ix.hunk_at(1), None);
        assert_eq!(PatchIndex::next(ix.hunks.iter().copied(), 4, true), Some(9));
        assert_eq!(PatchIndex::next(ix.hunks.iter().copied(), 4, false), None);
    }

    #[test]
    fn a_plain_unified_diff_has_its_files_too() {
        let l = lines(
            "--- old/a.txt\t2026-11-16 10:00\n+++ new/a.txt\t2026-11-16 11:00\n@@ -1 +1 @@\n-a\n+b\n",
        );
        assert!(looks_like_patch(l.as_slice()));
        let ix = index(l.as_slice());
        assert_eq!(ix.files.len(), 1);
        assert_eq!(ix.files[0].name, "new/a.txt");
        assert_eq!(ix.hunks, [2]);
    }

    #[test]
    fn a_hunk_that_lies_about_its_counts_is_marked() {
        let l = lines("--- a/x\n+++ b/x\n@@ -1,1 +1,1 @@\n-a\n+b\n+c\n+d\n");
        let ix = index(l.as_slice());
        assert_eq!(ix.lying_hunks, 1);
        assert_eq!(ix.added, 3, "the lines that are there count");
        let l = lines("--- a/x\n+++ b/x\n@@ -1,300 +1,300 @@\n-a\n+b\n");
        assert_eq!(index(l.as_slice()).lying_hunks, 1);
        let l = lines("--- a/x\n+++ b/x\n@@ -1,99999999999999999999999 +1 @@\n-a\n+b\n");
        assert_eq!(index(l.as_slice()).lying_hunks, 1);
    }

    #[test]
    fn dangerous_paths_and_modes() {
        for bad in [
            "/etc/passwd",
            "../x",
            "a/../../x",
            ".git/hooks/pre-commit",
            "x/.GIT/config",
            "C:\\x",
            "a\u{1b}b",
        ] {
            assert!(dangerous_path(bad), "{bad}");
        }
        for good in ["src/a.rs", "/dev/null", "a..b/c", "x.git/y"] {
            assert!(!dangerous_path(good), "{good}");
        }
        assert!(dangerous_mode("new file mode 120000"));
        assert!(dangerous_mode("new mode 100755"));
        assert!(dangerous_mode("new file mode 160000"));
        assert!(!dangerous_mode("new mode 100644"));
        let l = lines(
            "diff --git a/x b/x\nnew file mode 120000\n--- /dev/null\n+++ b/x\n@@ -0,0 +1 @@\n+../../etc\n",
        );
        assert!(index(l.as_slice()).files[0].danger);
        let l = lines("diff --git a/ok b/../../evil\n");
        assert!(index(l.as_slice()).files[0].danger);
    }

    #[test]
    fn names_quoted_by_git() {
        assert_eq!(header_name("\"a/\\303\\251 x.txt\""), "é x.txt");
        assert_eq!(header_name("\"b/tab\\there\""), "tab\there");
        assert_eq!(header_name("\"a/bad\\9\""), "bad\\9");
        let (a, b) = git_names("a/with space b/with space");
        assert_eq!((a.as_str(), b.as_str()), ("with space", "with space"));
        let (a, b) = git_names("\"a/\\303\\251\" \"b/\\303\\251\"");
        assert_eq!((a.as_str(), b.as_str()), ("é", "é"));
    }

    #[test]
    fn changes_that_cannot_be_seen_are_marked() {
        // A Cyrillic "а", a trailing space and a zero-width space.
        let l = lines(
            "--- a/x\n+++ b/x\n@@ -1,4 +1,4 @@\n-let admin = 1;\n-ok\n-x = 1\n-same\n+let аdmin = 1;\n+ok \n+x =\u{200B} 1\n+changed\n",
        );
        let ix = index(l.as_slice());
        // Lines 3..=5 removed pair with 7..=9 added; "same"/"changed" differ.
        assert_eq!(ix.unchanged, [3, 4, 5, 7, 8, 9]);
        assert_eq!(ix.whitespace, 1);
        assert!(ix.summary().contains("3 look unchanged"));
    }

    #[test]
    fn format_patch_messages_are_in_the_list() {
        let l = lines(
            "From 0123456789abcdef0123456789abcdef01234567 Mon Sep 17 00:00:00 2001\nFrom: A <a@b>\nSubject: [PATCH] fix it\n\n---\n a | 1 +\n\ndiff --git a/a b/a\n--- a/a\n+++ b/a\n@@ -1 +1 @@\n-x\n+y\n",
        );
        let ix = index(l.as_slice());
        assert_eq!(ix.files[0].status, Status::Mail);
        assert_eq!(ix.files[0].name, "[PATCH] fix it");
        assert_eq!(ix.files[1].name, "a");
        assert_eq!(ix.names()[0], "✉ [PATCH] fix it");
        assert_eq!(ix.summary(), "1 file, +1 −1");
    }

    #[test]
    fn line_colors_by_kind() {
        let p = [
            Color::Green,
            Color::Red,
            Color::Cyan,
            Color::Blue,
            Color::Yellow,
            Color::LightRed,
        ];
        assert_eq!(line_color("+x", false, p), Some(Color::Green));
        assert_eq!(line_color("-x", false, p), Some(Color::Red));
        assert_eq!(line_color("@@ -1 +1 @@", false, p), Some(Color::Cyan));
        assert_eq!(line_color(" x", false, p), None);
        assert_eq!(line_color("--- a/x", false, p), Some(Color::Blue));
        assert_eq!(
            line_color("+++ b/.git/hooks/x", false, p),
            Some(Color::LightRed)
        );
        assert_eq!(
            line_color("new mode 100755", false, p),
            Some(Color::LightRed)
        );
        assert_eq!(line_color("+x", true, p), Some(Color::Yellow));
    }

    proptest::proptest! {
        #[test]
        fn hostile_patches_never_panic(
            picks in proptest::collection::vec(0usize..14, 0..200),
            words in proptest::collection::vec("[ -~\\\\\"é\u{202e}]{0,12}", 0..200),
        ) {
            const START: [&str; 14] = [
                "diff --git ", "--- ", "+++ ", "@@ -", "+", "-", " ", "\\ ", "",
                "new mode ", "rename from ", "From 0123456789abcdef0123456789abcdef01234567 ",
                "Subject: ", "@@ -1,18446744073709551615 +",
            ];
            let l: Vec<String> = picks
                .iter()
                .zip(words.iter().chain(std::iter::repeat(&String::new())))
                .map(|(&p, w)| format!("{}{w}", START[p]))
                .collect();
            let ix = index(l.as_slice());
            for f in &ix.files {
                proptest::prop_assert!(f.line < l.len().max(1));
            }
            for &h in &ix.hunks {
                proptest::prop_assert!(h < l.len());
            }
            for &u in &ix.unchanged {
                proptest::prop_assert!(u < l.len());
            }
            let _ = ix.summary();
            let _ = ix.names();
            for line in &l {
                let _ = line_color(line, false, [Color::Reset; 6]);
            }
        }
    }
}
