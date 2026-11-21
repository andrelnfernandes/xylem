//! A Xylem window: tree, file list, filespec and cursors.
//!
//! The normal screen has one; with the window split (F8) there are two, each with
//! its own tree.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use ratatui::widgets::ListState;

use crate::archive::{self, EntryKind};
use crate::input::Command;
use crate::model::filespec::Filespec;
use crate::model::query::{FileQuery, Place, QueryContext};
use crate::model::sort::{self, Sort};
use crate::model::tree::{FileEntry, FileKind, NodeId, ROOT, Tree, VisibleRow};
use crate::ops::Source;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Tree,
    Files,
}

/// What the file window is listing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileView {
    /// Files of the directory under the tree cursor.
    Dir,
    /// Files of the branch starting at this directory (`B` key).
    Branch(NodeId),
    /// Files of everything that was logged (`S` key).
    Showall,
}

/// Files chosen in a view (Alt+V), by their paths, and what to call them in
/// the title.
#[derive(Debug, Clone)]
pub struct Pick {
    pub label: String,
    pub paths: Arc<HashSet<PathBuf>>,
}

/// Reference to a file inside the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileRef {
    pub dir: NodeId,
    pub idx: usize,
}

/// File window modes (Alt+F), in ZTreeWin order.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileDisplay {
    /// Several columns, names only.
    Names,
    /// Several columns: name, size and permissions.
    NamesSize,
    /// One column: name, size, date and permissions.
    #[default]
    Details,
    /// One column: name, size, owner and group, and permissions.
    Owner,
    /// One column: name and what the file is ("PNG image, 800 x 600").
    Type,
    /// One column, names only (full length).
    LongNames,
}

impl FileDisplay {
    pub fn next(self) -> Self {
        match self {
            FileDisplay::Names => FileDisplay::NamesSize,
            FileDisplay::NamesSize => FileDisplay::Details,
            FileDisplay::Details => FileDisplay::Owner,
            FileDisplay::Owner => FileDisplay::Type,
            FileDisplay::Type => FileDisplay::LongNames,
            FileDisplay::LongNames => FileDisplay::Names,
        }
    }

    pub fn multi_column(self) -> bool {
        matches!(self, FileDisplay::Names | FileDisplay::NamesSize)
    }
}

/// What Alt+K shows next to the directories.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Kbyte {
    pub show: KbyteShow,
    /// Tagged files only.
    pub tagged_only: bool,
    /// Sums the whole branch, not just the directory.
    pub branch: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum KbyteShow {
    #[default]
    None,
    Size,
    Count,
    /// Share of the directory above, with a bar against the largest of
    /// its siblings. Always sums whole branches.
    Percent,
}

impl Kbyte {
    /// Ex.: "size of branch, tagged".
    pub fn describe(&self) -> String {
        let tagged = if self.tagged_only { ", tagged" } else { "" };
        let what = match self.show {
            KbyteShow::None => return String::new(),
            KbyteShow::Size => "K",
            KbyteShow::Count => "files",
            KbyteShow::Percent => return format!("% of parent{tagged}"),
        };
        let scope = if self.branch { "branch" } else { "dir" };
        format!("{what} per {scope}{tagged}")
    }
}

pub struct Pane {
    pub tree: Tree,
    /// Cache of the visible rows; recomputed when the tree changes.
    pub rows: Vec<VisibleRow>,
    pub tree_state: ListState,
    pub focus: Focus,
    pub view: FileView,
    /// Tags-Only mode, as in ZTreeWin: the list shows only the tagged files.
    /// Files untagged meanwhile stay until the list is rebuilt.
    pub tags_only: bool,
    /// Files listed in the file window, already filtered.
    pub files: Vec<FileRef>,
    /// Goes up each time `files` is rebuilt: a `FileRef` kept from before
    /// may point elsewhere after it.
    pub files_built: u64,
    pub file_state: ListState,
    pub filespec: Filespec,
    /// W: a condition the files of the list must also meet.
    pub query: Option<FileQuery>,
    /// A choice made in a view (Alt+V): only these files stay in the list.
    /// W with no text takes it away.
    pub pick: Option<Pick>,
    pub sort: Sort,
    /// Alt+K: size or file count next to each directory.
    pub kbyte: Kbyte,
    /// Alt+U: the Usage view, in place of the tree and the files.
    pub usage: Option<crate::usage::UsageView>,
    /// Alt+D: the Duplicates view, in place of the tree and the files.
    pub duplicates: Option<crate::duplicates::DuplicatesView>,
    /// Duplicate group numbers computed by Alt+D (path -> 1-based group).
    pub duplicate_groups: HashMap<PathBuf, usize>,
    /// Alt+F: how the file window displays the list.
    pub display: FileDisplay,
    /// File window columns as of the last draw (multi-column modes),
    /// so the arrows move by row.
    pub file_cols: usize,
    /// Usable height of the lists as of the last draw (for PgUp/PgDn).
    pub tree_page: usize,
    pub file_page: usize,
    /// With a ZIP/TAR open: its path (root of the virtual tree)...
    pub archive: Option<PathBuf>,
    /// ...what it holds, so a copy does not have to decompress it all again...
    pub archive_entries: ArchiveEntries,
    /// ...and the window as it was before opening it, to go back with Esc.
    pub outer: Option<Box<Pane>>,
    /// A window on a remote machine: its files come through SFTP.
    pub remote: Option<std::sync::Arc<crate::remote::Remote>>,
    /// Type display (Alt+F): what each file is, with the size and date it
    /// had when read, so a changed file is read again.
    pub types: RefCell<TypeCache>,
    /// Status of the files in the repositories being displayed.
    pub git_status: Option<std::rc::Rc<RefCell<crate::git::StatusCache>>>,
    pub git_enabled: bool,
    /// The last DISK statistics and what they were computed from: the panel
    /// is drawn again every half second, and the sum walks every logged file.
    stats: RefCell<Option<(StatsKey, DiskStats)>>,
}

/// Cache of file type descriptions for Alt+F and SQL queries.
pub type TypeCache = HashMap<PathBuf, (u64, Option<SystemTime>, String)>;

/// What the DISK statistics depend on: the tree as read and tagged, whether
/// hidden entries count, the filespec, and the disk asked for (its mount
/// point and every mount point).
#[derive(PartialEq, Eq)]
struct StatsKey {
    version: u64,
    show_hidden: bool,
    filespec: String,
    disk: Option<(PathBuf, HashSet<PathBuf>)>,
}

/// The entries of an open archive, by their path in it.
pub type ArchiveEntries = Arc<HashMap<PathBuf, archive::Entry>>;

/// What reading an archive gives, made in the thread that reads it.
pub struct ArchiveContents {
    tree: Tree,
    entries: ArchiveEntries,
}

impl ArchiveContents {
    fn new(path: PathBuf, entries: Vec<archive::Entry>) -> Self {
        let mut map = HashMap::with_capacity(entries.len());
        let listing = entries
            .into_iter()
            .map(|e| {
                let file = (e.kind != EntryKind::Dir).then(|| FileEntry {
                    name: e.path.file_name().unwrap_or_default().to_os_string(),
                    size: e.size,
                    disk: e.size,
                    link: None,
                    modified: e.modified,
                    mode: crate::platform::from_unix_mode(e.mode),
                    kind: match e.kind {
                        EntryKind::File => FileKind::File,
                        EntryKind::Symlink => FileKind::Symlink,
                        _ => FileKind::Other,
                    },
                    tagged: false,
                });
                let item = (e.path.clone(), file);
                map.insert(e.path.clone(), e);
                item
            })
            .collect();
        Self {
            tree: Tree::from_listing(path, listing),
            entries: Arc::new(map),
        }
    }

    /// Starts reading the archive at `path`.
    pub fn read(path: &Path) -> archive::Listing<Self> {
        let root = path.to_path_buf();
        archive::Listing::start(path, move |entries| Self::new(root, entries))
    }
}

impl Pane {
    pub fn new(root: PathBuf) -> Self {
        Self::with_tree(Tree::new(root))
    }

    /// A window that reads in the background, as on a cloud mount.
    #[cfg(test)]
    pub fn slow(root: PathBuf) -> Self {
        Self::with_tree(Tree::slow(root))
    }

    /// Window showing the contents of an archive, already read.
    pub fn from_archive(path: &Path, contents: ArchiveContents) -> Self {
        let mut pane = Self::with_tree(contents.tree);
        pane.archive = Some(path.to_path_buf());
        pane.archive_entries = contents.entries;
        pane
    }

    /// Window on a remote machine, starting at its root.
    pub fn from_remote(remote: std::sync::Arc<crate::remote::Remote>) -> Self {
        let lister: std::sync::Arc<dyn crate::model::tree::Lister> = remote.clone();
        let mut pane = Self::with_tree(Tree::with_lister(remote.root.clone(), lister));
        pane.remote = Some(remote);
        pane
    }

    fn with_tree(tree: Tree) -> Self {
        let rows = tree.visible_rows();
        let mut pane = Pane {
            tree,
            rows,
            tree_state: ListState::default().with_selected(Some(0)),
            focus: Focus::Tree,
            view: FileView::Dir,
            tags_only: false,
            files: Vec::new(),
            files_built: 0,
            file_state: ListState::default().with_selected(Some(0)),
            filespec: Filespec::default(),
            query: None,
            pick: None,
            sort: Sort::default(),
            kbyte: Kbyte::default(),
            usage: None,
            duplicates: None,
            duplicate_groups: HashMap::new(),
            display: FileDisplay::default(),
            file_cols: 1,
            tree_page: 1,
            file_page: 1,
            archive: None,
            archive_entries: ArchiveEntries::default(),
            outer: None,
            remote: None,
            types: RefCell::new(HashMap::new()),
            git_status: None,
            git_enabled: true,
            stats: RefCell::new(None),
        };
        pane.refresh_files();
        pane
    }

    /// Directory under the tree cursor.
    pub fn current(&self) -> NodeId {
        self.rows[self.tree_state.selected().unwrap_or(0)].id
    }

    pub fn current_path(&self) -> PathBuf {
        self.tree.path(self.current())
    }

    pub fn selected_file(&self) -> Option<FileRef> {
        self.files.get(self.file_state.selected()?).copied()
    }

    pub fn file(&self, f: FileRef) -> &FileEntry {
        &self.tree.node(f.dir).files[f.idx]
    }

    pub fn file_path(&self, f: FileRef) -> PathBuf {
        self.tree.path(f.dir).join(&self.file(f).name)
    }

    /// Directory shown in the header: in Showall/Branch, that of the file
    /// under the cursor.
    pub fn header_dir(&self) -> NodeId {
        match (self.view, self.selected_file()) {
            (FileView::Dir, _) | (_, None) => self.current(),
            (_, Some(f)) => f.dir,
        }
    }

    /// Root directory of the current list: the directory, the Branch or the tree.
    pub fn view_root(&self) -> NodeId {
        match self.view {
            FileView::Dir => self.current(),
            FileView::Branch(id) => id,
            FileView::Showall => ROOT,
        }
    }

    /// Files an operation acts on: the tagged ones in the current
    /// list or the file under the cursor.
    pub fn sources(&self, tagged: bool) -> Vec<Source> {
        let refs: Vec<FileRef> = if tagged {
            self.files
                .iter()
                .copied()
                .filter(|&f| self.file(f).tagged)
                .collect()
        } else if self.focus == Focus::Files {
            self.selected_file().into_iter().collect()
        } else {
            Vec::new()
        };
        refs.into_iter()
            .map(|f| Source {
                path: self.file_path(f),
                size: self.file(f).size,
            })
            .collect()
    }

    pub fn set_tags(&mut self, refs: &[FileRef], f: impl Fn(bool) -> bool) {
        for &r in refs {
            let entry = self.tree.file_mut(r.dir, r.idx);
            entry.tagged = f(entry.tagged);
        }
    }

    /// Toggles the tag of `path` if it exists in the tree, returning the new state.
    pub fn toggle_tag_path(&mut self, path: &Path) -> Option<bool> {
        let dir = path.parent()?;
        let name = path.file_name()?;
        for id in self.tree.visible_subtree(ROOT) {
            if self.tree.path(id) == dir {
                for (idx, f) in self.tree.node(id).files.iter().enumerate() {
                    if f.name == name {
                        let entry = self.tree.file_mut(id, idx);
                        entry.tagged = !entry.tagged;
                        return Some(entry.tagged);
                    }
                }
            }
        }
        None
    }

    /// The files the list would show with no condition of W, as a condition
    /// with `contains` reads them; `tagged` keeps only the tagged ones.
    pub fn query_candidates(&self, tagged: bool) -> Vec<crate::model::query::Candidate> {
        let dirs = match self.view {
            FileView::Dir => vec![self.current()],
            FileView::Branch(id) => self.tree.visible_subtree(id),
            FileView::Showall => self.tree.visible_subtree(ROOT),
        };
        let mut out = Vec::new();
        for dir in dirs {
            let path = self.tree.path(dir);
            let mut depth = 0;
            let mut at = dir;
            while let Some(parent) = self.tree.node(at).parent {
                depth += 1;
                at = parent;
            }
            for f in &self.tree.node(dir).files {
                let shown = self.tree.show_hidden || !f.is_hidden();
                let wanted = !(tagged || self.tags_only) || f.tagged;
                if shown && wanted && self.filespec.matches_entry(f) {
                    let full = path.join(&f.name);
                    let git = self.file_git(&full);
                    let dup = self.dup_group(&full).map(|g| g as i64);
                    out.push(crate::model::query::Candidate {
                        path: full,
                        dir: path.clone(),
                        depth,
                        file: f.clone(),
                        git,
                        dup,
                    });
                }
            }
        }
        out
    }

    /// Reads a file's contents for search: from disk, or from inside the open archive/remote.
    pub fn content_opener(&self) -> crate::search::Opener {
        if let Some(remote) = self.remote.clone() {
            return Box::new(move |path: &Path| {
                let file = remote.open(path)?;
                Ok(Box::new(std::io::BufReader::new(file)) as Box<dyn std::io::BufRead>)
            });
        }
        match &self.archive {
            Some(root) => crate::archive::search_opener(root),
            None => Box::new(crate::search::open_file),
        }
    }

    /// What a file is, as Alt+F tells it, read from where this window reads
    /// its files: over the network on a remote machine, by its name only
    /// inside an archive.
    pub fn type_of(&self) -> crate::model::query::TypeOf {
        if let Some(remote) = self.remote.clone() {
            return Box::new(move |path, f| {
                crate::filetype::describe_with(
                    path,
                    f.kind,
                    f.size,
                    || remote.read(path, crate::filetype::HEAD_BYTES).ok(),
                    || remote.read_link(path).ok(),
                )
            });
        }
        let read = self.archive.is_none();
        Box::new(move |path, f| crate::filetype::describe(path, f.kind, f.size, read))
    }

    /// Git status of `path` (`" M"`, `"  "`, etc.), or `None` if outside a repo
    /// or Git is off.
    pub fn file_git(&self, path: &Path) -> Option<String> {
        if !self.git_enabled || self.archive.is_some() || self.remote.is_some() {
            return None;
        }
        let git = self.git_status.as_ref()?;
        git.borrow().file_status(path)
    }

    /// Duplicate group number of `path` (1-based), if Alt+D has calculated it.
    pub fn dup_group(&self, path: &Path) -> Option<usize> {
        self.duplicate_groups
            .get(path)
            .copied()
            .or_else(|| self.duplicates.as_ref().and_then(|v| v.group_of(path)))
    }

    /// Describes what `f` is, remembering results in `types`.
    pub fn describe_file(&self, path: &Path, f: &FileEntry) -> String {
        let (size, modified, kind) = (f.size, f.modified, f.kind);
        if let Ok(types) = self.types.try_borrow()
            && let Some((s, m, what)) = types.get(path)
            && (*s, *m) == (size, modified)
        {
            return what.clone();
        }
        let what = match &self.remote {
            Some(remote) => crate::filetype::describe_with(
                path,
                kind,
                size,
                || remote.read(path, crate::filetype::HEAD_BYTES).ok(),
                || remote.read_link(path).ok(),
            ),
            None => crate::filetype::describe(path, kind, size, self.archive.is_none()),
        };
        if let Ok(mut types) = self.types.try_borrow_mut() {
            if types.len() > 10_000 {
                types.clear();
            }
            types.insert(path.to_path_buf(), (size, modified, what.clone()));
        }
        what
    }

    /// Where the files of `dir` are, if the condition of W asks for it: its
    /// path and how many folders below the root it is.
    fn query_place(&self, dir: NodeId) -> Option<(PathBuf, usize)> {
        self.query.as_ref().filter(|q| q.needs_place())?;
        let mut depth = 0;
        let mut at = dir;
        while let Some(parent) = self.tree.node(at).parent {
            depth += 1;
            at = parent;
        }
        Some((self.tree.path(dir), depth))
    }

    /// Whether the condition of W, if there is one, takes the file.
    fn query_takes(
        &self,
        f: &FileEntry,
        place: &Option<(PathBuf, usize)>,
        now: SystemTime,
    ) -> bool {
        self.query.as_ref().is_none_or(|q| {
            let place_obj = place
                .as_ref()
                .map(|(dir, depth)| Place { dir, depth: *depth });
            let path = place.as_ref().map(|(dir, _)| dir.join(&f.name));
            let git = path.as_ref().and_then(|p| self.file_git(p));
            let dup = path
                .as_ref()
                .and_then(|p| self.dup_group(p))
                .map(|g| g as i64);
            let lazy_type: &dyn Fn() -> Option<String> = &|| {
                let p = path.as_ref()?;
                Some(self.describe_file(p, f))
            };
            let ctx = QueryContext {
                place: place_obj,
                git: git.as_deref(),
                dup,
                lazy_type: Some(lazy_type),
            };
            q.matches(f, &ctx, now)
        })
    }

    /// All logged files that match the filespec.
    pub fn logged_matching_files(&self) -> Vec<FileRef> {
        self.branch_matching_files(ROOT)
    }

    /// The logged files of the branch of `id` that match the filespec.
    pub fn branch_matching_files(&self, id: NodeId) -> Vec<FileRef> {
        let mut refs = Vec::new();
        let now = SystemTime::now();
        for dir in self.tree.visible_subtree(id) {
            let place = self.query_place(dir);
            for (idx, f) in self.tree.node(dir).files.iter().enumerate() {
                let shown = self.tree.show_hidden || !f.is_hidden();
                if shown && self.filespec.matches_entry(f) && self.query_takes(f, &place, now) {
                    refs.push(FileRef { dir, idx });
                }
            }
        }
        refs
    }

    /// Rereads `dir`; if it is not in the tree yet (e.g. it was just
    /// created), rereads the nearest ancestor that is.
    pub fn rescan_path(&mut self, dir: &Path) {
        if let Some(id) = dir.ancestors().find_map(|d| self.tree.find(d)) {
            self.tree.rescan(id);
        }
    }

    /// Puts on screen what was read in the background, keeping the cursor.
    pub fn apply_reads(&mut self) {
        if self.tree.reading() {
            self.keeping_cursor(|pane| {
                pane.tree.apply_reads();
            });
        }
    }

    /// Updates the tree and the file list after changes on disk.
    pub fn after_change(&mut self) {
        self.refresh_rows(self.current());
        self.refresh_files();
    }

    /// Rereads `dirs` and updates the screen, keeping the cursor on the same file
    /// (if it still exists).
    pub fn rescan_paths<P: AsRef<Path>>(&mut self, dirs: &[P]) {
        self.keeping_cursor(|pane| {
            for dir in dirs {
                pane.rescan_path(dir.as_ref());
            }
        });
    }

    /// Changes the sort order, keeping the cursor on the same file.
    pub fn set_sort(&mut self, sort: Sort) {
        self.keeping_cursor(|pane| pane.sort = sort);
    }

    /// Runs `change` and updates the screen with the cursor on the same file as
    /// before (if it still exists).
    fn keeping_cursor(&mut self, change: impl FnOnce(&mut Self)) {
        // Saved before the change: afterwards, the references in the list may
        // no longer be valid.
        let selected = self.selected_file().and_then(|f| {
            let entry = self.tree.node(f.dir).files.get(f.idx)?;
            Some((f.dir, entry.name.clone()))
        });
        change(self);
        self.after_change();
        if let Some((dir, name)) = selected
            && let Some(i) = self
                .files
                .iter()
                .position(|&f| f.dir == dir && self.file(f).name == name)
        {
            self.file_state.select(Some(i));
        }
    }

    /// Moves the tree cursor to `path`, reading and expanding the path.
    /// Stops at the last existing directory; returns whether it reached the target.
    pub fn goto(&mut self, path: &Path) -> bool {
        match self.walk(path, true) {
            Some((id, reached)) => {
                self.show_node(id);
                reached
            }
            None => false,
        }
    }

    /// Finds the directory for `path` in the tree, reading what is missing from disk,
    /// the one it reaches included (Treespec completes from its subdirectories).
    /// Returns the deepest one that exists and whether it got all the way; `None` if
    /// the path is outside the tree. Without `wait`, a directory not read yet on a
    /// slow mount is asked for in the background: the walk stops there, and the
    /// one it reaches has no names until it arrives.
    fn walk(&mut self, path: &Path, wait: bool) -> Option<(NodeId, bool)> {
        let rel = path.strip_prefix(self.tree.path(ROOT)).ok()?;
        let mut cur = ROOT;
        for comp in rel.components() {
            let Component::Normal(name) = comp else {
                continue;
            };
            if !wait && !self.tree.node(cur).scanned && self.tree.reads_slowly(&self.tree.path(cur))
            {
                self.tree.touch(cur);
                return Some((cur, false));
            }
            self.tree.scan(cur);
            let node = self.tree.node(cur);
            match node
                .children
                .iter()
                .find(|&&c| self.tree.node(c).name == name)
            {
                Some(&child) => cur = child,
                None => return Some((cur, false)),
            }
        }
        if wait {
            self.tree.scan(cur);
        } else {
            self.tree.touch(cur);
        }
        Some((cur, true))
    }

    /// Puts the tree cursor on `id`, expanding the path to it.
    pub fn show_node(&mut self, id: NodeId) {
        self.tree.reveal(id);
        // On a slow mount, in the background: the window says it is reading.
        self.tree.touch(id);
        if self.current() != id {
            self.refresh_rows(id);
            self.file_state.select(Some(0));
            self.refresh_files();
        }
    }

    /// Treespec: the directory for a half-typed path, where the last
    /// piece is the start of a name (`/home/an` → `/home/andre`). While typing
    /// (`wait` false), it stops at a slow directory still being read.
    pub fn treespec_target(&mut self, text: &str, wait: bool) -> Option<NodeId> {
        let (dir, partial) = split_partial(text);
        let (node, reached) = self.walk(Path::new(dir), wait)?;
        if !reached || partial.is_empty() {
            return Some(node);
        }
        Some(self.treespec_match(node, partial).unwrap_or(node))
    }

    /// First subdirectory of `parent` whose name starts with `partial`
    /// (case-insensitive). Hidden ones only if the start is a dot.
    fn treespec_match(&self, parent: NodeId, partial: &str) -> Option<NodeId> {
        let partial = partial.to_lowercase();
        self.treespec_choices(parent, &partial)
            .into_iter()
            .find(|&c| {
                self.tree
                    .node(c)
                    .name
                    .to_string_lossy()
                    .to_lowercase()
                    .starts_with(&partial)
            })
    }

    fn treespec_choices(&self, parent: NodeId, partial: &str) -> Vec<NodeId> {
        let show_hidden = self.tree.show_hidden || partial.starts_with('.');
        self.tree
            .node(parent)
            .children
            .iter()
            .copied()
            .filter(|&c| show_hidden || !self.tree.node(c).hidden)
            .collect()
    }

    /// Completes the last piece with the name of the directory it points to.
    pub fn treespec_complete(&mut self, text: &str) -> Option<String> {
        let (dir, partial) = split_partial(text);
        if partial.is_empty() {
            return None;
        }
        let (parent, true) = self.walk(Path::new(dir), true)? else {
            return None;
        };
        let target = self.treespec_match(parent, partial)?;
        Some(format!(
            "{dir}{}",
            self.tree.node(target).name.to_string_lossy()
        ))
    }

    /// Replaces the last piece with the next (or previous) directory at the same
    /// level (Tab / Shift+Tab).
    pub fn treespec_sibling(&mut self, text: &str, forward: bool) -> Option<String> {
        let (dir, partial) = split_partial(text);
        let (parent, true) = self.walk(Path::new(dir), true)? else {
            return None;
        };
        let choices = self.treespec_choices(parent, partial);
        if choices.is_empty() {
            return None;
        }
        let pos = choices
            .iter()
            .position(|&c| self.tree.node(c).name.to_string_lossy() == partial);
        let next = match (pos, forward) {
            (None, true) => 0,
            (None, false) => choices.len() - 1,
            (Some(i), true) => (i + 1) % choices.len(),
            (Some(i), false) => (i + choices.len() - 1) % choices.len(),
        };
        Some(format!(
            "{dir}{}",
            self.tree.node(choices[next]).name.to_string_lossy()
        ))
    }

    /// Shift+letter: next directory (in the tree) or file (in the list)
    /// whose name starts with `c`, wrapping around at the end.
    pub fn spell_jump(&mut self, c: char) {
        let c = c.to_lowercase().next().unwrap_or(c);
        let starts = |name: &std::ffi::OsStr| name.to_string_lossy().to_lowercase().starts_with(c);
        match self.focus {
            Focus::Tree => {
                let n = self.rows.len();
                let cur = self.tree_state.selected().unwrap_or(0);
                if let Some(i) = (1..=n)
                    .map(|k| (cur + k) % n)
                    .find(|&i| starts(&self.tree.node(self.rows[i].id).name))
                {
                    self.select_row(i);
                }
            }
            Focus::Files => {
                let n = self.files.len();
                let cur = self.file_state.selected().unwrap_or(0);
                if let Some(i) = (1..=n)
                    .map(|k| (cur + k) % n)
                    .find(|&i| starts(&self.file(self.files[i]).name))
                {
                    self.file_state.select(Some(i));
                }
            }
        }
    }

    pub fn apply_tree(&mut self, cmd: Command) {
        let cur = self.current();
        let pos = self.tree_state.selected().unwrap_or(0);
        match cmd {
            Command::Expand => {
                self.tree.expand(cur);
                self.refresh_rows(cur);
                // It may have just been logged.
                self.refresh_files();
            }
            Command::Collapse => {
                let node = self.tree.node(cur);
                if node.expanded && self.tree.has_children(cur) {
                    self.tree.collapse(cur);
                    self.refresh_rows(cur);
                } else if let Some(parent) = node.parent {
                    self.select_node(parent);
                }
            }
            _ => self.select_row(move_cursor(cmd, pos, self.rows.len(), self.tree_page)),
        }
    }

    pub fn apply_files(&mut self, cmd: Command) {
        let pos = self.file_state.selected().unwrap_or(0);
        let len = self.files.len();
        let cols = if self.display.multi_column() {
            self.file_cols.max(1)
        } else {
            1
        };
        let target = match cmd {
            // In columns, ← and → move one file and ↑ ↓ one row; in a
            // single column, ← and → page (as in ZTreeWin).
            Command::Left if cols > 1 => pos.saturating_sub(1),
            Command::Right if cols > 1 => (pos + 1).min(len.saturating_sub(1)),
            Command::Left => move_cursor(Command::PageUp, pos, len, self.file_page),
            Command::Right => move_cursor(Command::PageDown, pos, len, self.file_page),
            Command::Up if cols > 1 => pos.checked_sub(cols).unwrap_or(pos),
            Command::Down if cols > 1 && pos + cols < len => pos + cols,
            Command::Down if cols > 1 => pos,
            _ => move_cursor(cmd, pos, len, self.file_page),
        };
        self.file_state.select(Some(target));
    }

    /// Alt+F: next display mode.
    pub fn cycle_display(&mut self) {
        self.display = self.display.next();
        *self.file_state.offset_mut() = 0;
    }

    pub fn set_view(&mut self, view: FileView) {
        self.open_view(view, false);
    }

    /// Opens `view` with only the tagged files (Tags-Only mode).
    pub fn set_view_tagged(&mut self, view: FileView) {
        self.open_view(view, true);
    }

    fn open_view(&mut self, view: FileView, tags_only: bool) {
        self.view = view;
        self.tags_only = tags_only;
        self.focus = Focus::Files;
        self.file_state.select(Some(0));
        self.refresh_files();
    }

    /// B in Branch or Showall: back to the plain list of the directory
    /// (where the Branch started; under the tree cursor for Showall),
    /// staying in the file window.
    pub fn leave_list_view(&mut self) {
        let dir = match self.view {
            FileView::Branch(id) => id,
            _ => self.current(),
        };
        self.view = FileView::Dir;
        self.tags_only = false;
        self.show_node(dir);
        self.file_state.select(Some(0));
        self.refresh_files();
    }

    /// Back to the tree. Leaving Showall/Branch, the tree cursor
    /// goes to the directory of the file that was selected.
    pub fn back_to_tree(&mut self) {
        let target = (self.view != FileView::Dir)
            .then(|| self.selected_file())
            .flatten()
            .map(|f| f.dir);
        self.view = FileView::Dir;
        self.tags_only = false;
        self.focus = Focus::Tree;
        match target {
            Some(dir) => {
                self.tree.reveal(dir);
                self.refresh_rows(dir);
                self.file_state.select(Some(0));
                self.refresh_files();
            }
            None => self.refresh_files(),
        }
    }

    pub fn select_row(&mut self, index: usize) {
        if Some(index) != self.tree_state.selected() {
            self.tree_state.select(Some(index));
            self.file_state.select(Some(0));
            self.refresh_files();
        }
    }

    pub fn select_node(&mut self, id: NodeId) {
        if let Some(i) = self.rows.iter().position(|r| r.id == id) {
            self.select_row(i);
        }
    }

    /// Recomputes the visible rows, keeping the cursor on `keep`.
    pub fn refresh_rows(&mut self, keep: NodeId) {
        self.rows = self.tree.visible_rows();
        let i = self.rows.iter().position(|r| r.id == keep).unwrap_or(0);
        self.tree_state.select(Some(i));
    }

    /// "DISK Statistics", as in ZTreeWin: everything logged, what of it
    /// matches the filespec and what of it is tagged. Matching and Tagged
    /// count what Showall would list: not inside hidden directories (unless
    /// they are displayed), nor hidden files.
    ///
    /// With `disk` (its mount point, and every mount point there is), only
    /// what is on that disk: a folder where another file system is mounted
    /// takes its branch out, as does what is above the disk. So `/proc`
    /// (whose `kcore` "has" 128 TiB) never adds to the disk of `/`. On one
    /// disk, a file with other hard links counts its bytes once in Logged and
    /// Matching, as `du` does, so that they stay within what the disk holds;
    /// Tagged counts each name, as a copy would.
    ///
    /// The result is kept until the tree, the filespec or the disk asked for
    /// changes.
    pub fn disk_stats(&self, disk: Option<(&Path, &HashSet<PathBuf>)>) -> DiskStats {
        let same = |key: &StatsKey| {
            key.version == self.tree.version()
                && key.show_hidden == self.tree.show_hidden
                && key.filespec == self.filespec.text()
                && match (&key.disk, disk) {
                    (None, None) => true,
                    (Some((point, points)), Some((p, ps))) => point == p && points == ps,
                    _ => false,
                }
        };
        if let Some((key, stats)) = &*self.stats.borrow()
            && same(key)
        {
            return *stats;
        }
        let stats = self.compute_disk_stats(disk);
        let key = StatsKey {
            version: self.tree.version(),
            show_hidden: self.tree.show_hidden,
            filespec: self.filespec.text().to_string(),
            disk: disk.map(|(point, points)| (point.to_path_buf(), points.clone())),
        };
        *self.stats.borrow_mut() = Some((key, stats));
        stats
    }

    fn compute_disk_stats(&self, disk: Option<(&Path, &HashSet<PathBuf>)>) -> DiskStats {
        let mut stats = DiskStats::default();
        let add = |t: &mut Count, f: &FileEntry| {
            t.files += 1;
            t.bytes += f.size;
        };
        // Inodes already counted, per total: on one disk an inode is one file.
        let mut seen: [HashSet<u64>; 2] = Default::default();
        let mut add_once = |t: &mut Count, f: &FileEntry, which: usize| {
            t.files += 1;
            match f.link {
                Some(inode) if disk.is_some() && !seen[which].insert(inode) => {}
                _ => t.bytes += f.size,
            }
        };
        let root_path = self.tree.path(ROOT);
        let on_root = match disk {
            None => true,
            Some((point, points)) => {
                let mount = points
                    .iter()
                    .filter(|p| root_path.starts_with(p))
                    .max_by_key(|p| p.as_os_str().len());
                mount.is_some_and(|m| m == point)
            }
        };
        // (folder, its path when the disk is checked, on the disk, visible)
        let mut stack = vec![(ROOT, disk.map(|_| root_path), on_root, true)];
        while let Some((id, path, on_disk, visible)) = stack.pop() {
            let node = self.tree.node(id);
            if node.scanned && on_disk {
                for f in &node.files {
                    add_once(&mut stats.logged, f, 0);
                    if !visible || (!self.tree.show_hidden && f.is_hidden()) {
                        continue;
                    }
                    if self.filespec.matches_entry(f) {
                        add_once(&mut stats.matching, f, 1);
                    }
                    if f.tagged {
                        add(&mut stats.tagged, f);
                    }
                }
            }
            for &c in &node.children {
                let child = self.tree.node(c);
                let shown = visible && (self.tree.show_hidden || !child.hidden);
                let (child_path, child_on) = match (disk, &path) {
                    (Some((point, points)), Some(path)) => {
                        let child_path = path.join(&child.name);
                        let on = if points.contains(&child_path) {
                            child_path == point
                        } else {
                            on_disk
                        };
                        (Some(child_path), on)
                    }
                    _ => (None, on_disk),
                };
                stack.push((c, child_path, child_on, shown));
            }
        }
        stats
    }

    /// The files of `dir` the list would show (hidden files, the Filespec,
    /// `W`), Tags-Only aside: what tagging a folder tags.
    pub fn dir_files(&self, dir: NodeId) -> Vec<FileRef> {
        let place = self.query_place(dir);
        let now = SystemTime::now();
        self.tree
            .node(dir)
            .files
            .iter()
            .enumerate()
            .filter(|(_, f)| {
                (self.tree.show_hidden || !f.is_hidden())
                    && self.filespec.matches_entry(f)
                    && self.query_takes(f, &place, now)
            })
            .map(|(idx, _)| FileRef { dir, idx })
            .collect()
    }

    /// Rebuilds the file window list according to the view and the filespec.
    pub fn refresh_files(&mut self) {
        let dirs = match self.view {
            FileView::Dir => vec![self.current()],
            FileView::Branch(id) => self.tree.visible_subtree(id),
            FileView::Showall => self.tree.visible_subtree(ROOT),
        };
        let mut files = Vec::new();
        let now = SystemTime::now();
        for (rank, dir) in dirs.into_iter().enumerate() {
            let place = self.query_place(dir);
            let picked_dir = self.pick.as_ref().map(|_| self.tree.path(dir));
            for (idx, f) in self.tree.node(dir).files.iter().enumerate() {
                let shown = self.tree.show_hidden || !f.is_hidden();
                let tagged = f.tagged || !self.tags_only;
                let picked = match (&self.pick, &picked_dir) {
                    (Some(pick), Some(dir)) => pick.paths.contains(&dir.join(&f.name)),
                    _ => true,
                };
                if shown
                    && tagged
                    && picked
                    && self.filespec.matches_entry(f)
                    && self.query_takes(f, &place, now)
                {
                    files.push((FileRef { dir, idx }, rank));
                }
            }
        }
        // With no tags left, Tags-Only gives way to all the files (ZTreeWin).
        if self.tags_only && files.is_empty() {
            self.tags_only = false;
            return self.refresh_files();
        }
        let sort = self.sort;
        sort::sort_by(&mut files, &sort, |&(f, rank)| {
            sort.key_for(self.file(f), rank)
        });
        self.files = files.into_iter().map(|(f, _)| f).collect();
        self.files_built += 1;

        let last = self.files.len().saturating_sub(1);
        let sel = self.file_state.selected().unwrap_or(0).min(last);
        self.file_state.select(Some(sel));
    }
}

/// Files and bytes.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Count {
    pub files: u64,
    pub bytes: u64,
}

/// The "DISK Statistics" panel (see `Pane::disk_stats`).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DiskStats {
    pub logged: Count,
    pub matching: Count,
    pub tagged: Count,
}

/// "/home/an" → ("/home/", "an"); "/home/" → ("/home/", ""). On Windows,
/// `\` also separates.
fn split_partial(text: &str) -> (&str, &str) {
    match text.rfind(std::path::is_separator) {
        Some(i) => (&text[..=i], &text[i + 1..]),
        None => ("", text),
    }
}

/// New position of a cursor in a list of `len` items.
pub fn move_cursor(cmd: Command, pos: usize, len: usize, page: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let last = len - 1;
    let page = page.max(1);
    match cmd {
        Command::Up => pos.saturating_sub(1),
        Command::Down => (pos + 1).min(last),
        Command::PageUp => pos.saturating_sub(page),
        Command::PageDown => (pos + page).min(last),
        Command::Home => 0,
        Command::End => last,
        _ => pos,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::create_dir_all;

    #[test]
    fn cursor_stays_in_bounds() {
        assert_eq!(move_cursor(Command::Up, 0, 5, 3), 0);
        assert_eq!(move_cursor(Command::Down, 4, 5, 3), 4);
        assert_eq!(move_cursor(Command::PageDown, 1, 5, 3), 4);
        assert_eq!(move_cursor(Command::PageUp, 4, 5, 3), 1);
        assert_eq!(move_cursor(Command::End, 0, 5, 3), 4);
        assert_eq!(move_cursor(Command::Down, 0, 0, 3), 0);
    }

    #[test]
    fn after_change_keeps_cursor_on_same_file() {
        let tmp = crate::testing::tempdir();
        for f in ["b", "d"] {
            std::fs::write(tmp.path().join(f), "").unwrap();
        }
        let mut pane = Pane::new(tmp.path().to_path_buf());
        pane.file_state.select(Some(1)); // "d"
        std::fs::write(tmp.path().join("a"), "").unwrap();
        std::fs::write(tmp.path().join("c"), "").unwrap();
        pane.rescan_paths(&[tmp.path()]);
        let name = &pane.file(pane.selected_file().unwrap()).name;
        assert_eq!(name, "d");

        // If the file vanished, the cursor stays inside the list.
        std::fs::remove_file(tmp.path().join("d")).unwrap();
        pane.rescan_paths(&[tmp.path()]);
        assert_eq!(pane.file_state.selected(), Some(2));
    }

    #[test]
    fn disk_stats_cover_everything_logged() {
        let tmp = crate::testing::tempdir();
        std::fs::write(tmp.path().join("a.rs"), "0123456789").unwrap();
        std::fs::write(tmp.path().join("b.txt"), "01234").unwrap();
        create_dir_all(tmp.path().join("sub")).unwrap();
        std::fs::write(tmp.path().join("sub/c.rs"), "012").unwrap();
        let mut pane = Pane::new(tmp.path().to_path_buf());
        let count = |files, bytes| Count { files, bytes };
        pane.tree
            .scan(pane.tree.find(&tmp.path().join("sub")).unwrap());

        pane.filespec = Filespec::parse("*.rs").unwrap();
        let b = pane
            .tree
            .node(ROOT)
            .files
            .iter()
            .position(|f| f.name == "b.txt");
        pane.tree.file_mut(ROOT, b.unwrap()).tagged = true;

        let stats = pane.disk_stats(None);
        assert_eq!(stats.logged, count(3, 18));
        assert_eq!(stats.matching, count(2, 13), "a.rs and sub/c.rs");
        assert_eq!(stats.tagged, count(1, 5), "tagged, even if not matching");

        // Another disk mounted at sub: out of the statistics of this one...
        let root = pane.tree.path(ROOT);
        let sub = root.join("sub");
        let points: HashSet<PathBuf> = [root.clone(), sub.clone()].into();
        let stats = pane.disk_stats(Some((&root, &points)));
        assert_eq!(stats.logged, count(2, 15), "a.rs and b.txt");
        assert_eq!(stats.matching, count(1, 10));
        assert_eq!(stats.tagged, count(1, 5));
        // ...and the only thing in the statistics of that disk.
        let stats = pane.disk_stats(Some((&sub, &points)));
        assert_eq!(stats.logged, count(1, 3), "sub/c.rs");
        assert_eq!(stats.tagged, count(0, 0));

        // A hidden directory counts as logged, but not as matching.
        create_dir_all(tmp.path().join(".git")).unwrap();
        std::fs::write(tmp.path().join(".git/x.rs"), "0123").unwrap();
        crate::testing::hide(&tmp.path().join(".git"));
        pane.tree.show_hidden = false;
        pane.tree.rescan(ROOT);
        let git = pane.tree.find(&tmp.path().join(".git")).unwrap();
        pane.tree.scan(git);
        let stats = pane.disk_stats(None);
        assert_eq!(stats.logged, count(4, 22));
        assert_eq!(stats.matching, count(2, 13));
    }

    #[test]
    fn disk_stats_are_computed_again_when_what_they_depend_on_changes() {
        let tmp = crate::testing::tempdir();
        std::fs::write(tmp.path().join("a.rs"), "0123456789").unwrap();
        std::fs::write(tmp.path().join(".b.rs"), "01234").unwrap();
        crate::testing::hide(&tmp.path().join(".b.rs"));
        let mut pane = Pane::new(tmp.path().to_path_buf());
        let count = |files, bytes| Count { files, bytes };
        let first = pane.disk_stats(None);
        assert_eq!(first.matching, count(2, 15));
        assert_eq!(first.tagged, count(0, 0));
        assert_eq!(pane.disk_stats(None), first, "the kept answer");

        // A tag.
        let a = pane
            .tree
            .node(ROOT)
            .files
            .iter()
            .position(|f| f.name == "a.rs");
        pane.tree.file_mut(ROOT, a.unwrap()).tagged = true;
        assert_eq!(pane.disk_stats(None).tagged, count(1, 10));
        // Hidden files out.
        pane.tree.show_hidden = false;
        assert_eq!(pane.disk_stats(None).matching, count(1, 10));
        // Another filespec.
        pane.filespec = Filespec::parse("*.txt").unwrap();
        assert_eq!(pane.disk_stats(None).matching, count(0, 0));
        // A file that appeared on the disk.
        std::fs::write(tmp.path().join("c.txt"), "012").unwrap();
        pane.tree.rescan(ROOT);
        assert_eq!(pane.disk_stats(None).matching, count(1, 3));
    }

    #[test]
    #[cfg(unix)]
    fn on_one_disk_a_hard_link_counts_its_bytes_once() {
        let tmp = crate::testing::tempdir();
        std::fs::write(tmp.path().join("a.rs"), "0123456789").unwrap();
        std::fs::hard_link(tmp.path().join("a.rs"), tmp.path().join("b.rs")).unwrap();
        let mut pane = Pane::new(tmp.path().to_path_buf());
        let count = |files, bytes| Count { files, bytes };
        for f in 0..2 {
            pane.tree.file_mut(ROOT, f).tagged = true;
        }
        let root = pane.tree.path(ROOT);
        let points: HashSet<PathBuf> = [root.clone()].into();
        let stats = pane.disk_stats(Some((&root, &points)));
        assert_eq!(stats.logged, count(2, 10), "two names, one file");
        assert_eq!(stats.matching, count(2, 10));
        assert_eq!(stats.tagged, count(2, 20), "a copy writes both");
        // Without knowing the disk (an archive, a remote machine): each name.
        assert_eq!(pane.disk_stats(None).logged, count(2, 20));
    }

    #[test]
    fn treespec_follows_what_is_typed() {
        let tmp = crate::testing::tempdir();
        for d in ["Docs/rust", "Docs/notes", "downloads", ".config/xt"] {
            create_dir_all(tmp.path().join(d)).unwrap();
        }
        let root = tmp.path().to_string_lossy().into_owned();
        let mut pane = Pane::new(tmp.path().to_path_buf());
        let name = |pane: &Pane, id| pane.tree.node(id).name.to_string_lossy().into_owned();

        let id = pane.treespec_target(&format!("{root}/do"), false).unwrap();
        assert_eq!(name(&pane, id), "Docs", "case-insensitive, first in order");
        let id = pane.treespec_target(&format!("{root}/dow"), false).unwrap();
        assert_eq!(name(&pane, id), "downloads");
        let id = pane
            .treespec_target(&format!("{root}/Docs/n"), false)
            .unwrap();
        assert_eq!(name(&pane, id), "notes");
        let id = pane.treespec_target(&format!("{root}/.c"), false).unwrap();
        assert_eq!(name(&pane, id), ".config");
        // Nothing matches: stays in the parent directory.
        let id = pane.treespec_target(&format!("{root}/zz"), false).unwrap();
        assert_eq!(id, ROOT);
        assert!(pane.treespec_target("/fora/da/arvore", false).is_none());

        assert_eq!(
            pane.treespec_complete(&format!("{root}/do")),
            Some(format!("{root}/Docs"))
        );
        let next = pane
            .treespec_sibling(&format!("{root}/Docs"), true)
            .unwrap();
        assert_eq!(next, format!("{root}/downloads"));
        let wrap = pane.treespec_sibling(&next, true).unwrap();
        assert_eq!(wrap, format!("{root}/.config"), "hidden shown by default");
        let back = pane
            .treespec_sibling(&format!("{root}/Docs"), false)
            .unwrap();
        assert_eq!(back, format!("{root}/.config"));
    }

    #[test]
    fn shift_letter_jumps_with_wraparound() {
        let tmp = crate::testing::tempdir();
        for d in ["alpha", "beta", "bravo"] {
            create_dir_all(tmp.path().join(d)).unwrap();
        }
        let mut pane = Pane::new(tmp.path().to_path_buf());
        let current = |p: &Pane| p.tree.node(p.current()).name.to_string_lossy().into_owned();
        pane.spell_jump('B');
        assert_eq!(current(&pane), "beta");
        pane.spell_jump('B');
        assert_eq!(current(&pane), "bravo");
        pane.spell_jump('B');
        assert_eq!(current(&pane), "beta", "wraps around");
        pane.spell_jump('Z');
        assert_eq!(current(&pane), "beta", "no match: stays");
    }

    #[test]
    fn arrows_move_by_column_in_multi_column_modes() {
        let tmp = crate::testing::tempdir();
        for i in 0..10 {
            std::fs::write(tmp.path().join(format!("f{i}")), "").unwrap();
        }
        let mut pane = Pane::new(tmp.path().to_path_buf());
        pane.display = FileDisplay::Names;
        pane.file_cols = 4;
        pane.file_page = 8;
        let sel = |p: &Pane| p.file_state.selected().unwrap();
        pane.apply_files(Command::Right);
        assert_eq!(sel(&pane), 1);
        pane.apply_files(Command::Down);
        assert_eq!(sel(&pane), 5);
        pane.apply_files(Command::Down);
        assert_eq!(sel(&pane), 9);
        pane.apply_files(Command::Down);
        assert_eq!(sel(&pane), 9, "no row below");
        pane.apply_files(Command::Up);
        assert_eq!(sel(&pane), 5);
        pane.apply_files(Command::Left);
        assert_eq!(sel(&pane), 4);

        // In a single column, ← and → page.
        pane.display = FileDisplay::Details;
        pane.file_page = 3;
        pane.apply_files(Command::Right);
        assert_eq!(sel(&pane), 7);
        pane.cycle_display();
        assert_eq!(pane.display, FileDisplay::Owner);
        pane.cycle_display();
        assert_eq!(pane.display, FileDisplay::Type);
        pane.cycle_display();
        assert_eq!(pane.display, FileDisplay::LongNames);
        pane.cycle_display();
        assert_eq!(pane.display, FileDisplay::Names);
    }

    #[test]
    fn treespec_does_not_wait_for_a_slow_folder_until_enter() {
        let tmp = crate::testing::tempdir();
        create_dir_all(tmp.path().join("Docs/rust")).unwrap();
        let root = tmp.path().to_string_lossy().into_owned();
        let mut pane = Pane::slow(tmp.path().to_path_buf());
        let text = format!("{root}/Docs/ru");
        // Typing: the root is still on its way, so the cursor stays there.
        assert_eq!(pane.treespec_target(&text, false), Some(ROOT));
        assert!(
            !pane.tree.node(ROOT).scanned,
            "nothing read on the screen's thread"
        );
        assert!(pane.tree.is_reading(ROOT));
        // Enter: reads what is missing, one folder per level, to the end.
        let id = pane.treespec_target(&text, true).unwrap();
        assert_eq!(pane.tree.node(id).name, "rust");
    }

    #[test]
    fn goto_opens_the_path() {
        let tmp = crate::testing::tempdir();
        create_dir_all(tmp.path().join("a/b/c")).unwrap();
        let mut pane = Pane::new(tmp.path().to_path_buf());

        assert!(pane.goto(&tmp.path().join("a/b/c")));
        assert_eq!(pane.current_path(), tmp.path().join("a/b/c"));
        assert!(pane.rows.iter().any(|r| r.id == pane.current()));

        // Path that does not exist: stops at the last directory that exists.
        assert!(!pane.goto(&tmp.path().join("a/x/y")));
        assert_eq!(pane.current_path(), tmp.path().join("a"));
        // Outside the tree: no movement.
        assert!(!pane.goto(Path::new("/elsewhere")));
        assert_eq!(pane.current_path(), tmp.path().join("a"));
    }
}
