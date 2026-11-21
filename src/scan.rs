//! The Scan (Alt+V, S; docs/plans/views.md): what in the list deserves a
//! look. The checks that need only the name and the mode run at once, from
//! what is in memory; the ones that ask the disk (a folder's mode, where a
//! link points) and the ones that read contents run in a thread, with Esc to
//! stop.
//!
//! The contents go through the same readers as the viewer's reports (`V`),
//! with their limits, one file at a time, each under `panic_guard`: a file
//! a reader cannot take is listed as not analysed, with why, and never left
//! out in silence. No reader is new here.
//!
//! Nothing here changes a file, and nothing follows a link to read it. The
//! Scan is not an antivirus: it points at what deserves a look.

use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};

/// Most hits kept per category: past it they are only counted.
pub const MAX_HITS: usize = 100_000;

/// What the Scan looks for, in the order of the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    /// Control or text-direction characters, or a name that is not UTF-8.
    HiddenChars,
    /// `fatura.pdf.exe`, spaces before the extension, a trailing dot.
    Deceiving,
    /// setuid, setgid, writable by anyone.
    Permissions,
    /// Links that leave the folder scanned, or point to nothing.
    Links,
    /// Executables packed, without PIE or NX, with writable and executable
    /// sections.
    Executables,
    /// Office documents with macros, a remote template, embedded objects or
    /// hidden text.
    Office,
    /// Patches that touch dangerous paths or hide what they change.
    Patches,
    /// Files whose contents were not read: over a limit, or a reader failed.
    NotAnalysed,
}

impl Category {
    pub const ALL: [Category; 8] = [
        Category::HiddenChars,
        Category::Deceiving,
        Category::Permissions,
        Category::Links,
        Category::Executables,
        Category::Office,
        Category::Patches,
        Category::NotAnalysed,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Category::HiddenChars => "Names with control or text-direction characters",
            Category::Deceiving => "Names that deceive (fatura.pdf.exe, spaces, a trailing dot)",
            Category::Permissions => "setuid, setgid, writable by anyone",
            Category::Links => "Links that leave the folder, or point to nothing",
            Category::Executables => {
                "Executables: packed, without PIE or NX, writable and executable"
            }
            Category::Office => "Office: macros, remote template, embedded objects, hidden text",
            Category::Patches => "Patches: dangerous paths, hidden characters, hidden changes",
            Category::NotAnalysed => "Not analysed: over a limit, or the reader could not take it",
        }
    }

    /// The name in the file window's title, once the list is filtered by it.
    pub fn short(self) -> &'static str {
        match self {
            Category::HiddenChars => "hidden characters",
            Category::Deceiving => "deceiving names",
            Category::Permissions => "permissions",
            Category::Links => "links",
            Category::Executables => "executables",
            Category::Office => "Office documents",
            Category::Patches => "patches",
            Category::NotAnalysed => "not analysed",
        }
    }
}

/// A file or a folder the Scan points at, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub path: PathBuf,
    pub dir: bool,
    pub why: String,
}

/// What was found for one category.
#[derive(Debug, Clone, Default)]
pub struct Found {
    pub hits: Vec<Hit>,
    /// Hits past [`MAX_HITS`], counted only.
    pub more: usize,
    /// What was not looked at, and why ("inside an archive").
    pub note: Option<String>,
}

impl Found {
    pub fn count(&self) -> usize {
        self.hits.len() + self.more
    }

    pub fn push(&mut self, hit: Hit) {
        if self.hits.len() < MAX_HITS {
            self.hits.push(hit);
        } else {
            self.more += 1;
        }
    }
}

/// A name with characters that hide what it is: control characters, the
/// ones that change the direction of the text (`exe.pdf` drawn from
/// `fdp.exe`), or bytes that are not UTF-8.
pub fn hidden_chars(name: &OsStr) -> Option<String> {
    let Some(text) = name.to_str() else {
        return Some("not valid UTF-8".into());
    };
    let found: Vec<char> = text
        .chars()
        .filter(|&c| crate::bidi::is_bidi(c) || c.is_control())
        .collect();
    if found.is_empty() {
        return None;
    }
    let bidi = found.iter().any(|&c| crate::bidi::is_bidi(c));
    let mut codes: Vec<String> = found
        .iter()
        .take(4)
        .map(|&c| format!("U+{:04X}", c as u32))
        .collect();
    if found.len() > 4 {
        codes.push(format!("{} more", found.len() - 4));
    }
    let what = if bidi { "text direction" } else { "control" };
    Some(format!("{what}: {}", codes.join(", ")))
}

/// Extensions that run when opened, on Windows above all.
const RUNS: &[&str] = &[
    "exe", "scr", "bat", "cmd", "com", "pif", "js", "jse", "vbs", "vbe", "wsf", "jar", "msi",
    "ps1", "lnk", "hta", "cpl",
];

/// Extensions of what people open without a second thought.
const DOCUMENTS: &[&str] = &[
    "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods", "odp", "rtf", "txt", "csv",
    "jpg", "jpeg", "png", "gif", "bmp", "mp3", "mp4", "avi", "mov", "mkv", "wav", "zip", "rar",
    "7z", "html", "htm",
];

fn is_one_of(ext: &str, list: &[&str]) -> bool {
    list.iter().any(|e| e.eq_ignore_ascii_case(ext))
}

/// A name made to look like what it is not: a document's extension before
/// one that runs (`fatura.pdf.exe`), spaces before the extension (that push
/// it out of a narrow column), or a name ending in a space or a dot (that
/// Windows drops, so the name seen is not the name there).
pub fn deceiving(name: &OsStr) -> Option<String> {
    let name = name.to_str()?;
    if name == "." || name == ".." {
        return None;
    }
    if name.ends_with(' ') {
        return Some("ends in a space".into());
    }
    if name.ends_with('.') {
        return Some("ends in a dot".into());
    }
    let (stem, ext) = name.rsplit_once('.')?;
    if ext.is_empty() || stem.is_empty() {
        return None;
    }
    if stem.ends_with(char::is_whitespace) {
        return Some(format!("spaces before the extension .{ext}"));
    }
    if is_one_of(ext, RUNS)
        && let Some((_, inner)) = stem.rsplit_once('.')
        && is_one_of(inner, DOCUMENTS)
    {
        return Some(format!("looks like .{inner}, runs as .{ext}"));
    }
    None
}

const SETUID: u32 = 0o4000;
const SETGID: u32 = 0o2000;
const STICKY: u32 = 0o1000;
const OTHERS_WRITE: u32 = 0o002;

/// The mode of a regular file or a folder that deserves a look: setuid or
/// setgid (a file), or writable by anyone (a folder only without the sticky
/// bit, as `/tmp` has).
pub fn mode_problem(mode: u32, dir: bool) -> Option<String> {
    if dir {
        return (mode & OTHERS_WRITE != 0 && mode & STICKY == 0)
            .then(|| "anyone can write in it (no sticky bit)".into());
    }
    let mut why = Vec::new();
    if mode & SETUID != 0 {
        why.push("setuid");
    }
    if mode & SETGID != 0 {
        why.push("setgid");
    }
    if mode & OTHERS_WRITE != 0 {
        why.push("anyone can write it");
    }
    (!why.is_empty()).then(|| why.join(", "))
}

/// File systems whose modes are made up by the driver (the same for every
/// file), so a mode there says nothing: the Windows drives under WSL, FAT,
/// exFAT, NTFS, SMB shares and rclone mounts.
pub fn modes_are_made_up(fstype: &str) -> bool {
    matches!(
        fstype,
        "9p" | "drvfs"
            | "vfat"
            | "msdos"
            | "fat"
            | "exfat"
            | "ntfs"
            | "ntfs3"
            | "fuseblk"
            | "cifs"
            | "smb3"
            | "smbfs"
    ) || fstype.starts_with("fuse.rclone")
}

/// `path` with `.` and `..` resolved by the text alone, following nothing.
pub fn lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                // Never above the root.
                if !out.pop() && !out.has_root() {
                    out.push(part);
                }
            }
            other => out.push(other),
        }
    }
    out
}

/// A link that points out of `root`, by its text and never by following it,
/// or that points to nothing.
pub fn link_problem(link: &Path, root: &Path) -> Option<String> {
    let target = std::fs::read_link(link).ok()?;
    let shown = crate::bidi::escape(&target.to_string_lossy());
    let whole = match link.parent() {
        Some(dir) if target.is_relative() => dir.join(&target),
        _ => target.clone(),
    };
    let outside = !lexical(&whole).starts_with(lexical(root));
    // `metadata` asks where the link ends without opening it.
    let nothing = std::fs::metadata(link).is_err();
    match (outside, nothing) {
        (true, true) => Some(format!("points out of the folder, to nothing: {shown}")),
        (true, false) => Some(format!("points out of the folder: {shown}")),
        (false, true) => Some(format!("points to nothing: {shown}")),
        (false, false) => None,
    }
}

/// What the thread is asked to look at.
#[derive(Debug, Clone)]
pub enum Task {
    /// A folder: its mode (from `symlink_metadata`).
    DirMode(PathBuf),
    /// A link: where it points.
    Link(PathBuf),
    /// A regular file: its contents, if it is an executable, an Office
    /// document or a patch.
    Content(PathBuf),
}

/// Where the contents are read from.
#[derive(Debug, Clone)]
pub enum Source {
    /// The disk: files are read where they are.
    Disk,
    /// An archive (Alt+F5): its members, extracted one by one with a limit.
    /// The tree's paths start at the archive's own path.
    Archive(PathBuf),
}

/// How much the thread reads.
#[derive(Debug, Clone)]
pub struct Limits {
    /// The most files whose contents are read (`scan_max_files`).
    pub files: usize,
    /// The most of one binary read (the viewer's limit).
    pub binary: u64,
    /// The most of one archive member extracted.
    pub member: u64,
    /// The most of a patch read.
    pub patch: u64,
    /// Office documents: the viewer's limits.
    pub office: crate::viewer::office::Limits,
}

impl Limits {
    pub fn new(files: usize) -> Self {
        Limits {
            files,
            binary: 256 << 20,
            member: 64 << 20,
            patch: 32 << 20,
            office: crate::viewer::office::Limits {
                part: 64 << 20,
                total: 256 << 20,
                time: std::time::Duration::from_secs(3),
                budget: 16 << 20,
            },
        }
    }
}

/// The checks that ask the disk, in a thread.
pub struct Job {
    rx: mpsc::Receiver<(Category, Hit)>,
    stop: Arc<AtomicBool>,
    done: Arc<AtomicUsize>,
    finished: Arc<AtomicBool>,
    pub total: usize,
}

impl Job {
    /// Starts looking at `tasks`; links are judged against `root`.
    pub fn start(tasks: Vec<Task>, root: PathBuf, source: Source, limits: Limits) -> Self {
        let (tx, rx) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let done = Arc::new(AtomicUsize::new(0));
        let finished = Arc::new(AtomicBool::new(false));
        let total = tasks.len();
        let (stop2, done2, finished2) = (stop.clone(), done.clone(), finished.clone());
        std::thread::spawn(move || {
            let mut reader = Reading::new(source, limits);
            for task in tasks {
                if stop2.load(Ordering::Relaxed) {
                    break;
                }
                if let Some(found) = run(&task, &root, &mut reader) {
                    let _ = tx.send(found);
                }
                done2.fetch_add(1, Ordering::Relaxed);
            }
            finished2.store(true, Ordering::Release);
        });
        Job {
            rx,
            stop,
            done,
            finished,
            total,
        }
    }

    /// What was found since the last call.
    pub fn drain(&self) -> Vec<(Category, Hit)> {
        self.rx.try_iter().collect()
    }

    pub fn done(&self) -> usize {
        self.done.load(Ordering::Relaxed)
    }

    /// The thread ended, and everything it found was sent.
    pub fn is_finished(&self) -> bool {
        self.finished.load(Ordering::Acquire)
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run(task: &Task, root: &Path, reading: &mut Reading) -> Option<(Category, Hit)> {
    match task {
        Task::Content(path) => {
            let found = crate::panic_guard::catch(|| reading.look(path)).unwrap_or_else(|| {
                Some((
                    Category::NotAnalysed,
                    "the reader failed on it (please report it)".into(),
                ))
            });
            let (cat, why) = found?;
            Some((
                cat,
                Hit {
                    path: path.clone(),
                    dir: false,
                    why,
                },
            ))
        }
        Task::DirMode(path) => {
            let mode = dir_mode(path)?;
            let why = mode_problem(mode, true)?;
            Some((
                Category::Permissions,
                Hit {
                    path: path.clone(),
                    dir: true,
                    why,
                },
            ))
        }
        Task::Link(path) => {
            let why = link_problem(path, root)?;
            Some((
                Category::Links,
                Hit {
                    path: path.clone(),
                    dir: false,
                    why,
                },
            ))
        }
    }
}

/// What a file is, for the readers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Binary,
    Office(crate::viewer::office::Kind),
    Patch,
}

/// The first bytes that tell a binary.
const SNIFF: u64 = 64;

/// The readers' side of the thread: the archive, opened once, and how many
/// files were read so far.
struct Reading {
    source: Source,
    limits: Limits,
    archive: Option<std::io::Result<crate::archive::Reader>>,
    read: usize,
}

impl Reading {
    fn new(source: Source, limits: Limits) -> Self {
        Reading {
            source,
            limits,
            archive: None,
            read: 0,
        }
    }

    /// At most `limit` bytes of `path`: from the disk, or extracted from the
    /// archive.
    fn bytes(&mut self, path: &Path, limit: u64) -> std::io::Result<Vec<u8>> {
        use std::io::Read;
        match &self.source {
            Source::Disk => {
                let mut data = Vec::new();
                crate::viewer::open_regular(path)?
                    .take(limit)
                    .read_to_end(&mut data)?;
                Ok(data)
            }
            Source::Archive(root) => {
                let root = root.clone();
                let reader = self
                    .archive
                    .get_or_insert_with(|| crate::archive::Reader::open(&root));
                let reader = reader
                    .as_mut()
                    .map_err(|e| std::io::Error::new(e.kind(), e.to_string()))?;
                let entry = path.strip_prefix(&root).unwrap_or(path);
                let mut data = Vec::new();
                reader.extract(entry, &mut data, limit)?;
                Ok(data)
            }
        }
    }

    /// The readers' kind of file: by the name for Office and patches, by
    /// the first bytes for binaries.
    fn kind(&mut self, path: &Path) -> Option<Kind> {
        if let Some(kind) = crate::viewer::office::kind(path) {
            return Some(Kind::Office(kind));
        }
        if crate::viewer::patch::is_patch_name(path) {
            return Some(Kind::Patch);
        }
        let head = self.bytes(path, SNIFF).ok()?;
        crate::viewer::binaries::is_binary(&head).then_some(Kind::Binary)
    }

    /// What the readers say of `path`: the category and why, or nothing.
    fn look(&mut self, path: &Path) -> Option<(Category, String)> {
        let kind = self.kind(path)?;
        if self.read >= self.limits.files {
            return Some((
                Category::NotAnalysed,
                format!("past the {} files read (scan_max_files)", self.limits.files),
            ));
        }
        self.read += 1;
        let archive = matches!(self.source, Source::Archive(_));
        let (cat, result) = match kind {
            Kind::Binary => {
                let result = if archive {
                    let limit = self.limits.member;
                    self.bytes(path, limit)
                        .map_err(|e| format!("cannot be read: {e}"))
                        .and_then(|data| crate::viewer::binaries::warnings(&data))
                } else {
                    crate::viewer::binaries::warnings_file(path, self.limits.binary)
                };
                (Category::Executables, result)
            }
            Kind::Office(kind) => {
                let limits = self.limits.office;
                let result = if archive {
                    let limit = self.limits.member;
                    self.bytes(path, limit)
                        .map_err(|e| format!("cannot be read: {e}"))
                        .and_then(|data| {
                            crate::viewer::office::warnings(
                                std::io::Cursor::new(data),
                                kind,
                                limits,
                            )
                        })
                } else {
                    crate::viewer::open_regular(path)
                        .map_err(|e| format!("cannot be read: {e}"))
                        .and_then(|file| {
                            crate::viewer::office::warnings(
                                std::io::BufReader::new(file),
                                kind,
                                limits,
                            )
                        })
                };
                (Category::Office, result)
            }
            Kind::Patch => {
                let limit = if archive {
                    self.limits.patch.min(self.limits.member)
                } else {
                    self.limits.patch
                };
                let result = self
                    .bytes(path, limit)
                    .map_err(|e| format!("cannot be read: {e}"))
                    .map(|data| {
                        let lines: Vec<String> = String::from_utf8_lossy(&data)
                            .lines()
                            .map(str::to_string)
                            .collect();
                        crate::viewer::patch::warnings(&lines[..])
                    });
                (Category::Patches, result)
            }
        };
        match result {
            Ok(warn) if warn.is_empty() => None,
            Ok(warn) => Some((cat, warn.join("; "))),
            Err(e) => Some((Category::NotAnalysed, e)),
        }
    }
}

#[cfg(unix)]
fn dir_mode(path: &Path) -> Option<u32> {
    use std::os::unix::fs::MetadataExt;
    Some(std::fs::symlink_metadata(path).ok()?.mode())
}

#[cfg(not(unix))]
fn dir_mode(_path: &Path) -> Option<u32> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os(s: &str) -> &OsStr {
        OsStr::new(s)
    }

    #[test]
    fn hidden_characters_are_named_by_their_code() {
        assert_eq!(hidden_chars(os("plain name.txt")), None);
        let why = hidden_chars(os("fdp\u{202E}exe.pdf")).unwrap();
        assert!(
            why.contains("text direction") && why.contains("U+202E"),
            "{why}"
        );
        let why = hidden_chars(os("bell\u{7}.txt")).unwrap();
        assert!(why.starts_with("control"), "{why}");
        // Accents and other scripts are not hidden characters.
        assert_eq!(hidden_chars(os("relatório 日本.txt")), None);
    }

    #[cfg(unix)]
    #[test]
    fn a_name_that_is_not_utf8_is_found() {
        use std::os::unix::ffi::OsStrExt;
        let name = OsStr::from_bytes(b"bad\xff.txt");
        assert_eq!(hidden_chars(name).as_deref(), Some("not valid UTF-8"));
    }

    #[test]
    fn deceiving_names() {
        let why = |n| deceiving(os(n));
        assert_eq!(
            why("fatura.pdf.exe").unwrap(),
            "looks like .pdf, runs as .exe"
        );
        assert_eq!(
            why("FOTO.JPG.SCR").unwrap(),
            "looks like .JPG, runs as .SCR"
        );
        assert!(why("fatura.pdf        .exe").unwrap().contains("spaces"));
        assert_eq!(why("readme ").unwrap(), "ends in a space");
        assert_eq!(why("report.").unwrap(), "ends in a dot");
        for fine in [
            "fatura.pdf",
            "setup.exe",
            "archive.tar.gz",
            "lib.min.js",
            ".bashrc",
            "..",
            "a b.txt",
        ] {
            assert_eq!(why(fine), None, "{fine}");
        }
    }

    #[test]
    fn modes_that_deserve_a_look() {
        assert_eq!(mode_problem(0o100644, false), None);
        assert_eq!(mode_problem(0o104755, false).unwrap(), "setuid");
        assert_eq!(mode_problem(0o102755, false).unwrap(), "setgid");
        assert_eq!(
            mode_problem(0o100666, false).unwrap(),
            "anyone can write it"
        );
        assert!(mode_problem(0o40777, true).is_some());
        // /tmp: anyone writes, but only removes what is theirs.
        assert_eq!(mode_problem(0o41777, true), None);
        assert_eq!(mode_problem(0o40755, true), None);
        // setgid on a folder only passes its group on.
        assert_eq!(mode_problem(0o42775, true), None);
    }

    #[test]
    fn made_up_modes() {
        for fs in [
            "9p",
            "drvfs",
            "vfat",
            "exfat",
            "ntfs3",
            "fuseblk",
            "fuse.rclone",
        ] {
            assert!(modes_are_made_up(fs), "{fs}");
        }
        for fs in ["ext4", "btrfs", "xfs", "tmpfs", "overlay", "nfs4"] {
            assert!(!modes_are_made_up(fs), "{fs}");
        }
    }

    #[test]
    fn lexical_follows_nothing() {
        assert_eq!(lexical(Path::new("/a/b/../c/./d")), PathBuf::from("/a/c/d"));
        assert_eq!(lexical(Path::new("/a/../../etc")), PathBuf::from("/etc"));
    }

    #[cfg(unix)]
    #[test]
    fn links_out_of_the_folder_or_to_nothing() {
        use std::os::unix::fs::symlink;
        let tmp = crate::testing::tempdir();
        let root = tmp.path().join("root");
        std::fs::create_dir_all(root.join("sub")).unwrap();
        std::fs::write(root.join("sub/real"), "x").unwrap();
        symlink("real", root.join("sub/inside")).unwrap();
        symlink("../sub/real", root.join("sub/up_and_back")).unwrap();
        symlink("/etc/passwd", root.join("passwd")).unwrap();
        symlink("../../outside", root.join("sub/escape")).unwrap();
        symlink("gone", root.join("broken")).unwrap();
        let why = |name: &str| link_problem(&root.join(name), &root);
        assert_eq!(why("sub/inside"), None);
        assert_eq!(why("sub/up_and_back"), None);
        assert!(
            why("passwd")
                .unwrap()
                .starts_with("points out of the folder: /etc/passwd")
        );
        assert!(why("sub/escape").unwrap().contains("to nothing"));
        assert_eq!(why("broken").unwrap(), "points to nothing: gone");
    }

    #[cfg(unix)]
    #[test]
    fn the_thread_finds_folders_and_links() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let tmp = crate::testing::tempdir();
        let root = tmp.path().to_path_buf();
        let open = root.join("open");
        std::fs::create_dir(&open).unwrap();
        std::fs::set_permissions(&open, std::fs::Permissions::from_mode(0o777)).unwrap();
        symlink("/", root.join("slash")).unwrap();
        let job = Job::start(
            vec![
                Task::DirMode(root.clone()),
                Task::DirMode(open.clone()),
                Task::Link(root.join("slash")),
            ],
            root.clone(),
            Source::Disk,
            Limits::new(10),
        );
        while !job.is_finished() {
            std::thread::yield_now();
        }
        let mut found = job.drain();
        found.sort_by_key(|(c, _)| *c as u8);
        assert_eq!(found.len(), 2, "{found:?}");
        assert_eq!(found[0].0, Category::Permissions);
        assert_eq!(found[0].1.path, open);
        assert_eq!(found[1].0, Category::Links);
        assert_eq!(job.done(), 3);
    }
}
