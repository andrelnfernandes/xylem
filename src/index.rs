//! The index of the contents for W (`words()`), one per branch, kept by
//! `tantivy` in the user's cache. Only when the user asks for it (Alt+N on
//! the branch), never in the background, and never with Xylem closed.
//!
//! A file is indexed by the words [`words_of`] gives, the same that reading
//! the file gives, so that a condition answers the same with the index or
//! without it. A file changed after it was indexed (another size or date) is
//! read again instead: an old index makes a condition slower, never wrong.
//!
//! The index is a copy of the contents that can be searched: it stays in the
//! cache, in a folder only the user reads, until the user deletes it.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::model::query::{Candidate, Known, Term};

/// The bytes of a file the index reads; a bigger one is not indexed (W reads
/// it when asked).
#[cfg_attr(not(feature = "index"), allow(dead_code))]
pub const MAX_INDEXED: u64 = 64 << 20;

/// Folders and files the index leaves out: they are big and change all the
/// time, or they hold secrets that should not be copied.
const LEFT_OUT_DIRS: &[&str] = &[".git", "target", "node_modules", ".ssh", ".gnupg"];
const LEFT_OUT_FILES: &[&str] = &[".env", "*.key", "*.pem", "id_rsa", "id_ed25519"];

/// What a file was when it was indexed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stamp {
    pub size: u64,
    /// Nanoseconds since 1970; -1 when it has no date.
    pub modified: i64,
}

impl Stamp {
    pub fn of(size: u64, modified: Option<SystemTime>) -> Self {
        let modified = modified
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map_or(-1, |d| i64::try_from(d.as_nanos()).unwrap_or(i64::MAX));
        Stamp { size, modified }
    }
}

/// Whether this Xylem was built with the index.
pub fn available() -> bool {
    cfg!(feature = "index")
}

/// Whether the index leaves the file out, by its path below the branch.
pub fn left_out(relative: &Path) -> bool {
    let mut parts = relative.components().peekable();
    while let Some(part) = parts.next() {
        let name = part.as_os_str().to_string_lossy();
        if parts.peek().is_some() {
            if LEFT_OUT_DIRS.contains(&name.as_ref()) {
                return true;
            }
        } else {
            let lower = name.to_lowercase();
            return LEFT_OUT_FILES.iter().any(|p| match p.strip_prefix('*') {
                Some(ext) => lower.ends_with(ext),
                None => lower == *p,
            });
        }
    }
    false
}

#[cfg(test)]
thread_local! {
    /// A test's own folder of indexes, so that no test writes in the real one.
    pub static TEST_BASE: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

/// Where the indexes are: `~/.cache/xylem/index`.
fn base() -> Option<PathBuf> {
    #[cfg(test)]
    if let Some(base) = TEST_BASE.with_borrow(Clone::clone) {
        return Some(base);
    }
    Some(crate::platform::cache_dir()?.join("index"))
}

/// A short name for the folder of the index of `root`, the same in every run.
fn folder_name(root: &Path) -> String {
    // FNV-1a: no hasher of the standard library promises the same value from
    // one version to the next.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in root.as_os_str().as_encoded_bytes() {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// The folder of the index of `root`.
fn folder(root: &Path) -> Option<PathBuf> {
    Some(base()?.join(folder_name(root)))
}

/// The file in an index's folder that says which branch it is of.
const ROOT_FILE: &str = "branch";

/// The branches with an index.
pub fn roots() -> Vec<PathBuf> {
    let Some(base) = base() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(base) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|e| std::fs::read_to_string(e.path().join(ROOT_FILE)).ok())
        .map(PathBuf::from)
        .collect()
}

/// The branches with an index inside `path` or above it: those a list of
/// `path` may have files of.
pub fn touching(path: &Path) -> Vec<PathBuf> {
    roots()
        .into_iter()
        .filter(|r| path.starts_with(r) || r.starts_with(path))
        .collect()
}

/// Deletes the index of `root`. False if there was none.
pub fn delete(root: &Path) -> std::io::Result<bool> {
    let Some(dir) = folder(root) else {
        return Ok(false);
    };
    if !dir.join(ROOT_FILE).exists() {
        return Ok(false);
    }
    std::fs::remove_dir_all(dir)?;
    Ok(true)
}

/// Makes the folder of an index, readable only by the user.
#[cfg_attr(not(feature = "index"), allow(dead_code))]
fn make_private(dir: &Path) -> std::io::Result<()> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(dir)
}

/// What a condition needs of the indexes: the files each one has, and the
/// files that have each `words` term.
#[derive(Default)]
pub struct Lookup {
    files: HashMap<PathBuf, Stamp>,
    /// The files the index knows are binary.
    binary: HashSet<PathBuf>,
    /// For each term of the condition: the files with all its words (`None`
    /// for `contains`, which the index does not answer).
    hits: Vec<Option<HashSet<PathBuf>>>,
    /// The indexes that could not be read: their branches and why.
    pub broken: Vec<(PathBuf, String)>,
}

impl Lookup {
    /// For a file of the list: what the index knows of each term, if it has
    /// the file as it is now.
    pub fn answer(&self, c: &Candidate) -> Option<Known> {
        let stamp = self.files.get(&c.path)?;
        if *stamp != Stamp::of(c.file.size, c.file.modified) {
            return None;
        }
        if self.binary.contains(&c.path) {
            return Some(Known::Binary);
        }
        Some(Known::Terms(
            self.hits
                .iter()
                .map(|h| h.as_ref().map(|set| set.contains(&c.path)))
                .collect(),
        ))
    }
}

/// Looks up `terms` in the indexes of `roots`. An index that cannot be read
/// is left out and said in [`Lookup::broken`]; it never stops the condition,
/// which then reads those files.
pub fn lookup(roots: &[PathBuf], terms: &[Term]) -> Lookup {
    let mut all = Lookup {
        hits: terms
            .iter()
            .map(|t| t.words.as_ref().map(|_| HashSet::new()))
            .collect(),
        ..Lookup::default()
    };
    for root in roots {
        let read = crate::panic_guard::catch(|| imp::lookup(root, terms))
            .unwrap_or_else(|| Err("the index made its reader fail".into()));
        match read {
            Ok(one) => {
                all.files.extend(one.files);
                all.binary.extend(one.binary);
                for (mine, theirs) in all.hits.iter_mut().zip(one.hits) {
                    if let (Some(mine), Some(theirs)) = (mine, theirs) {
                        mine.extend(theirs);
                    }
                }
            }
            Err(e) => all.broken.push((root.clone(), e)),
        }
    }
    all
}

/// A file of the branch to index.
#[cfg_attr(not(feature = "index"), allow(dead_code))]
pub struct ToIndex {
    pub path: PathBuf,
    pub stamp: Stamp,
}

/// How the indexing of a branch went.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Report {
    /// Files read and indexed now.
    pub indexed: usize,
    /// Files already in the index as they are.
    pub kept: usize,
    /// Files that were in the index and are not in the branch any more.
    pub removed: usize,
    pub binary: usize,
    pub too_big: usize,
    pub unreadable: usize,
    /// The index could not be read and was made again from nothing.
    pub rebuilt: bool,
    /// Esc stopped it: what was indexed until then is kept.
    pub stopped: bool,
}

/// The indexing of a branch, in another thread: the screen stays free.
pub struct IndexJob {
    pub root: PathBuf,
    pub total: usize,
    done: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    rx: std::sync::mpsc::Receiver<Result<Report, String>>,
}

impl IndexJob {
    /// Indexes `files` (the logged files of the branch `root`, those the
    /// index takes) in another thread.
    pub fn start(root: PathBuf, files: Vec<ToIndex>) -> Self {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, AtomicUsize};
        let done = Arc::new(AtomicUsize::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let (tx, rx) = std::sync::mpsc::channel();
        let total = files.len();
        // The folder is found here: the thread does not see a test's own.
        let dir = folder(&root);
        let (at, halt, branch) = (done.clone(), stop.clone(), root.clone());
        std::thread::spawn(move || {
            let result = match dir {
                Some(dir) => {
                    crate::panic_guard::catch(|| imp::index(&dir, &branch, files, &at, &halt))
                        .unwrap_or_else(|| Err("the indexing failed".into()))
                }
                None => Err("no cache folder".into()),
            };
            let _ = tx.send(result);
        });
        IndexJob {
            root,
            total,
            done,
            stop,
            rx,
        }
    }

    pub fn done(&self) -> usize {
        self.done.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Esc: stops after the file being read, keeping what was indexed.
    pub fn stop(&self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// The end, once it comes.
    pub fn poll(&self) -> Option<Result<Report, String>> {
        self.rx.try_recv().ok()
    }

    /// Waits for the end (for the tests and for Esc).
    pub fn wait(&self) -> Result<Report, String> {
        self.rx
            .recv()
            .unwrap_or_else(|_| Err("the indexing failed".into()))
    }
}

#[cfg(feature = "index")]
mod imp {
    use super::*;
    use std::io::Read;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    use tantivy::collector::DocSetCollector;
    use tantivy::query::{BooleanQuery, Query, TermQuery};
    use tantivy::schema::{
        Field, INDEXED, IndexRecordOption, STORED, STRING, Schema, TextFieldIndexing, TextOptions,
        Value,
    };
    use tantivy::{DocAddress, Index, IndexWriter, TantivyDocument, Term as Word, doc};

    use crate::model::query::WordSplitter;

    /// The version of what is kept: another one is made again from nothing.
    const VERSION: &str = "2";
    const VERSION_FILE: &str = "version";
    /// The memory the writer of the index may take.
    const WRITER_MEMORY: usize = 50 << 20;
    /// Files indexed between two commits: Esc or a crash loses at most these.
    const COMMIT_EVERY: usize = 2_000;

    struct Fields {
        path: Field,
        size: Field,
        modified: Field,
        /// 1 for a binary file, which has no words.
        binary: Field,
        body: Field,
    }

    fn schema() -> (Schema, Fields) {
        let mut b = Schema::builder();
        let path = b.add_text_field("path", STRING | STORED);
        let size = b.add_u64_field("size", STORED);
        let modified = b.add_i64_field("modified", STORED | INDEXED);
        let binary = b.add_u64_field("binary", STORED);
        // The words are already split and folded: the index only separates
        // them at the spaces.
        let body = b.add_text_field(
            "body",
            TextOptions::default().set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("whitespace")
                    .set_index_option(IndexRecordOption::Basic),
            ),
        );
        let fields = Fields {
            path,
            size,
            modified,
            binary,
            body,
        };
        (b.build(), fields)
    }

    fn open(dir: &Path) -> Result<(Index, Fields), String> {
        let version = std::fs::read_to_string(dir.join(VERSION_FILE)).unwrap_or_default();
        if version.trim() != VERSION {
            return Err("it is of another version of Xylem".into());
        }
        let index = Index::open_in_dir(dir).map_err(|e| e.to_string())?;
        let (want, fields) = schema();
        if index.schema() != want {
            return Err("it is of another version of Xylem".into());
        }
        Ok((index, fields))
    }

    fn create(dir: &Path, root: &Path) -> Result<(Index, Fields), String> {
        let io = |e: std::io::Error| e.to_string();
        if dir.exists() {
            std::fs::remove_dir_all(dir).map_err(io)?;
        }
        make_private(dir).map_err(io)?;
        std::fs::write(dir.join(VERSION_FILE), VERSION).map_err(io)?;
        let (schema, fields) = schema();
        let index = Index::create_in_dir(dir, schema).map_err(|e| e.to_string())?;
        // Written last: an index without it is not seen as one.
        std::fs::write(dir.join(ROOT_FILE), root.as_os_str().as_encoded_bytes()).map_err(io)?;
        Ok((index, fields))
    }

    /// Every file in the index, with its stamp, and those that are binary.
    fn stamps(
        index: &Index,
        f: &Fields,
    ) -> Result<(HashMap<PathBuf, Stamp>, HashSet<PathBuf>), String> {
        let searcher = index.reader().map_err(|e| e.to_string())?.searcher();
        let mut out = HashMap::new();
        let mut binary = HashSet::new();
        for (ord, segment) in searcher.segment_readers().iter().enumerate() {
            for id in segment.doc_ids_alive() {
                let doc: TantivyDocument = searcher
                    .doc(DocAddress::new(ord as u32, id))
                    .map_err(|e| e.to_string())?;
                let path = doc.get_first(f.path).and_then(|v| v.as_str());
                let size = doc.get_first(f.size).and_then(|v| v.as_u64());
                let modified = doc.get_first(f.modified).and_then(|v| v.as_i64());
                if let (Some(path), Some(size), Some(modified)) = (path, size, modified) {
                    if doc.get_first(f.binary).and_then(|v| v.as_u64()) == Some(1) {
                        binary.insert(PathBuf::from(path));
                    }
                    out.insert(PathBuf::from(path), Stamp { size, modified });
                }
            }
        }
        Ok((out, binary))
    }

    pub fn lookup(root: &Path, terms: &[Term]) -> Result<Lookup, String> {
        let dir = folder(root).ok_or("no cache folder")?;
        let (index, f) = open(&dir)?;
        let (files, binary) = stamps(&index, &f)?;
        let searcher = index.reader().map_err(|e| e.to_string())?.searcher();
        let mut hits = Vec::new();
        for term in terms {
            let Some(words) = &term.words else {
                hits.push(None);
                continue;
            };
            let parts: Vec<Box<dyn Query>> = words
                .iter()
                .map(|w| {
                    let word = Word::from_field_text(f.body, w);
                    Box::new(TermQuery::new(word, IndexRecordOption::Basic)) as Box<dyn Query>
                })
                .collect();
            let found = searcher
                .search(&BooleanQuery::intersection(parts), &DocSetCollector)
                .map_err(|e| e.to_string())?;
            let mut paths = HashSet::new();
            for address in found {
                let doc: TantivyDocument = searcher.doc(address).map_err(|e| e.to_string())?;
                if let Some(path) = doc.get_first(f.path).and_then(|v| v.as_str()) {
                    paths.insert(PathBuf::from(path));
                }
            }
            hits.push(Some(paths));
        }
        Ok(Lookup {
            files,
            binary,
            hits,
            broken: Vec::new(),
        })
    }

    /// The words of a file, joined by spaces; `None` for a binary file.
    fn words(path: &Path) -> std::io::Result<Option<String>> {
        let mut file = std::fs::File::open(path)?.take(MAX_INDEXED);
        let mut block = vec![0u8; 256 << 10];
        let mut splitter = WordSplitter::default();
        let mut body = String::new();
        let mut first = true;
        loop {
            let n = match file.read(&mut block) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };
            if first && block[..n.min(8 << 10)].contains(&0) {
                return Ok(None);
            }
            first = false;
            splitter.feed(&block[..n], |w| {
                body.push_str(&w);
                body.push(' ');
            });
        }
        splitter.finish(|w| {
            body.push_str(&w);
            body.push(' ');
        });
        Ok(Some(body))
    }

    pub fn index(
        dir: &Path,
        root: &Path,
        files: Vec<ToIndex>,
        done: &AtomicUsize,
        stop: &AtomicBool,
    ) -> Result<Report, String> {
        let dir = dir.to_path_buf();
        let mut report = Report::default();
        let opened = if dir.join(ROOT_FILE).exists() {
            crate::panic_guard::catch(|| {
                open(&dir).and_then(|(index, f)| {
                    let (old, _) = stamps(&index, &f)?;
                    Ok((index, f, old))
                })
            })
            .unwrap_or_else(|| Err("its reader failed".into()))
        } else {
            Err(String::new())
        };
        let (index, f, old) = match opened {
            Ok(opened) => opened,
            Err(e) => {
                report.rebuilt = !e.is_empty();
                let (index, f) = create(&dir, root)?;
                (index, f, HashMap::new())
            }
        };
        let mut writer: IndexWriter = index
            .writer_with_num_threads(1, WRITER_MEMORY)
            .map_err(|e| e.to_string())?;
        let wanted: HashSet<&Path> = files.iter().map(|t| t.path.as_path()).collect();
        for path in old.keys() {
            if !wanted.contains(path.as_path()) {
                writer.delete_term(Word::from_field_text(f.path, &path.to_string_lossy()));
                report.removed += 1;
            }
        }
        let mut since_commit = 0;
        for (n, file) in files.iter().enumerate() {
            if stop.load(Ordering::Relaxed) {
                report.stopped = true;
                break;
            }
            done.store(n, Ordering::Relaxed);
            if old.get(&file.path) == Some(&file.stamp) {
                report.kept += 1;
                continue;
            }
            let key = file.path.to_string_lossy();
            writer.delete_term(Word::from_field_text(f.path, &key));
            if file.stamp.size > MAX_INDEXED {
                report.too_big += 1;
                continue;
            }
            match words(&file.path) {
                Ok(Some(body)) => {
                    writer
                        .add_document(doc!(
                            f.path => key.as_ref(),
                            f.size => file.stamp.size,
                            f.modified => file.stamp.modified,
                            f.body => body,
                        ))
                        .map_err(|e| e.to_string())?;
                    report.indexed += 1;
                    since_commit += 1;
                }
                // Noted as binary, so that W does not read it to find it out.
                Ok(None) => {
                    writer
                        .add_document(doc!(
                            f.path => key.as_ref(),
                            f.size => file.stamp.size,
                            f.modified => file.stamp.modified,
                            f.binary => 1u64,
                        ))
                        .map_err(|e| e.to_string())?;
                    report.binary += 1;
                }
                Err(_) => report.unreadable += 1,
            }
            if since_commit >= COMMIT_EVERY {
                writer.commit().map_err(|e| e.to_string())?;
                since_commit = 0;
            }
        }
        writer.commit().map_err(|e| e.to_string())?;
        done.store(files.len(), Ordering::Relaxed);
        Ok(report)
    }
}

#[cfg(not(feature = "index"))]
mod imp {
    use super::*;

    const MISSING: &str = "this Xylem was built without the index";

    pub fn lookup(_root: &Path, _terms: &[Term]) -> Result<Lookup, String> {
        Err(MISSING.into())
    }

    pub fn index(
        _dir: &Path,
        _root: &Path,
        _files: Vec<ToIndex>,
        _done: &std::sync::atomic::AtomicUsize,
        _stop: &std::sync::atomic::AtomicBool,
    ) -> Result<Report, String> {
        Err(MISSING.into())
    }
}

#[cfg(all(test, feature = "index"))]
mod tests {
    use super::*;
    use crate::model::query::FileQuery;
    use crate::model::tree::{FileEntry, FileKind};

    /// A folder of files, and a folder of indexes of its own.
    fn branch(files: &[(&str, &str)]) -> (tempfile::TempDir, PathBuf) {
        let tmp = crate::testing::tempdir();
        TEST_BASE.with_borrow_mut(|b| *b = Some(tmp.path().join("indexes")));
        let root = tmp.path().join("branch");
        for (name, text) in files {
            let path = root.join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        (tmp, root)
    }

    fn entry(path: &Path) -> FileEntry {
        let meta = std::fs::metadata(path).unwrap();
        FileEntry {
            name: path.file_name().unwrap().into(),
            size: meta.len(),
            disk: meta.len(),
            link: None,
            modified: meta.modified().ok(),
            mode: 0o644,
            kind: FileKind::File,
            tagged: false,
        }
    }

    fn candidate(path: PathBuf) -> Candidate {
        Candidate {
            file: entry(&path),
            dir: path.parent().unwrap().to_path_buf(),
            depth: 0,
            path,
        }
    }

    fn index_all(root: &Path, names: &[&str]) -> Report {
        let files = names
            .iter()
            .map(|n| {
                let path = root.join(n);
                let e = entry(&path);
                ToIndex {
                    path,
                    stamp: Stamp::of(e.size, e.modified),
                }
            })
            .collect();
        IndexJob::start(root.to_path_buf(), files).wait().unwrap()
    }

    #[test]
    fn the_index_answers_for_the_files_as_they_were() {
        let (_tmp, root) = branch(&[
            ("a.txt", "Fatura de Março"),
            ("b.txt", "nada"),
            ("c.bin", "x\0y"),
        ]);
        let report = index_all(&root, &["a.txt", "b.txt", "c.bin"]);
        assert_eq!((report.indexed, report.binary), (2, 1));
        assert_eq!(roots(), std::slice::from_ref(&root));
        let query = FileQuery::parse("words('marco fatura') OR contains('x')", false).unwrap();
        let found = lookup(&touching(&root), query.terms());
        assert!(found.broken.is_empty());
        let a = candidate(root.join("a.txt"));
        assert_eq!(found.answer(&a), Some(Known::Terms(vec![Some(true), None])));
        assert_eq!(
            found.answer(&candidate(root.join("b.txt"))),
            Some(Known::Terms(vec![Some(false), None]))
        );
        // A binary file is known as such: it is left out with no reading.
        assert!(matches!(
            found.answer(&candidate(root.join("c.bin"))),
            Some(Known::Binary)
        ));
        // Changed after it was indexed: read again.
        let mut changed = a;
        changed.file.size += 1;
        assert_eq!(found.answer(&changed), None);

        // Again: only what changed is read, and what is gone leaves.
        std::fs::write(root.join("b.txt"), "outra fatura").unwrap();
        let report = index_all(&root, &["b.txt"]);
        assert_eq!((report.indexed, report.kept, report.removed), (1, 0, 2));
        let found = lookup(&touching(&root), query.terms());
        assert_eq!(found.answer(&candidate(root.join("a.txt"))), None);
        assert!(delete(&root).unwrap());
        assert!(roots().is_empty());
        assert!(!delete(&root).unwrap());
    }

    #[test]
    fn a_broken_index_is_said_and_made_again() {
        let (_tmp, root) = branch(&[("a.txt", "palavra")]);
        index_all(&root, &["a.txt"]);
        let dir = folder(&root).unwrap();
        for e in std::fs::read_dir(&dir).unwrap().flatten() {
            let name = e.file_name();
            if name != ROOT_FILE && name != "version" {
                std::fs::write(e.path(), b"garbage").unwrap();
            }
        }
        let query = FileQuery::parse("words('palavra')", false).unwrap();
        let found = lookup(&touching(&root), query.terms());
        assert_eq!(found.broken.len(), 1, "it is said, not a crash");
        assert_eq!(found.answer(&candidate(root.join("a.txt"))), None);
        let report = index_all(&root, &["a.txt"]);
        assert!(report.rebuilt);
        assert_eq!(report.indexed, 1);
    }

    #[test]
    fn secrets_and_build_folders_are_left_out() {
        for path in [
            ".git/config",
            "target/debug/x",
            "a/node_modules/b.js",
            ".env",
            "keys/server.PEM",
            ".ssh/id_rsa",
        ] {
            assert!(left_out(Path::new(path)), "{path}");
        }
        for path in ["src/main.rs", "docs/target.md", "environment.txt"] {
            assert!(!left_out(Path::new(path)), "{path}");
        }
    }
}
