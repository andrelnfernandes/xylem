//! Alt+V: the views of the list (docs/plans/views.md). Each one answers a
//! question the tree and the list do not, and only reads: what is chosen in
//! it goes back to the classic list as a filter (`Pane::pick`), and from
//! there Ctrl+T, Ctrl+C or Batch act as always.

use std::collections::HashSet;
use std::sync::Arc;

use super::*;
use crate::pane::Pick;
use crate::scan::{self, Category, Found, Hit, Task};

/// The Scan on screen: the categories with what each found, and the thread
/// still asking the disk.
pub struct ScanView {
    /// What was scanned: "Branch /home/me/downloads".
    pub what: String,
    pub files: usize,
    pub folders: usize,
    pub found: Vec<(Category, Found)>,
    pub selected: usize,
    /// The category whose hits are listed, and the hit under the cursor.
    pub open: Option<usize>,
    pub hit: usize,
    pub job: Option<scan::Job>,
    /// Esc stopped the thread: what it did not look at is not known.
    pub stopped: Option<(usize, usize)>,
    /// Where the list was when it was scanned, to come back to it.
    key: ScanKey,
}

/// The list a Scan was made of: the same list opens the same Scan again.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ScanKey {
    pane: usize,
    view: FileView,
    root: PathBuf,
    filespec: String,
    query: Option<String>,
    tags_only: bool,
}

impl ScanView {
    pub fn running(&self) -> bool {
        self.job.is_some()
    }

    #[cfg(test)]
    pub fn found(&self, cat: Category) -> Option<&Found> {
        self.found.iter().find(|(c, _)| *c == cat).map(|(_, f)| f)
    }

    fn found_mut(&mut self, cat: Category) -> &mut Found {
        let i = self.found.iter().position(|(c, _)| *c == cat).unwrap();
        &mut self.found[i].1
    }

    /// Takes in what the thread found; true once it ended.
    fn poll(&mut self) -> bool {
        let Some(job) = &self.job else {
            return true;
        };
        let finished = job.is_finished();
        for (cat, hit) in job.drain() {
            self.found_mut(cat).push(hit);
        }
        if finished {
            self.job = None;
            for (_, found) in &mut self.found {
                found.hits.sort_by(|a, b| a.path.cmp(&b.path));
            }
        }
        finished
    }

    pub fn open_hits(&self) -> Option<(Category, &Found)> {
        let (cat, found) = self.found.get(self.open?)?;
        Some((*cat, found))
    }
}

impl App {
    /// Alt+V: the selector of the views, a small menu with a letter for each.
    pub(crate) fn open_views(&mut self) {
        let run = |label: &str, key: &str, cmd| MenuLine::Run {
            label: label.into(),
            key: key.into(),
            action: context_menu::MenuAction::Cmd(cmd),
        };
        let lines = vec![
            MenuLine::Title("Views of the list".into()),
            run("Scan: what deserves a look", "S", Command::ViewScan),
            run(
                "Timeline: what changed, and when",
                "T",
                Command::ViewTimeline,
            ),
            run(
                "Table: the list, with a SQL filter",
                "L",
                Command::ViewTable,
            ),
        ];
        let list = match self.focus {
            Focus::Tree => crate::mouse::List::Tree(crate::mouse::Owner::Main(self.active)),
            Focus::Files => crate::mouse::List::Files(crate::mouse::Owner::Main(self.active)),
        };
        let cursor = match self.focus {
            Focus::Tree => self.tree_state.selected().unwrap_or(0),
            Focus::Files => self.file_state.selected().unwrap_or(0),
        };
        let at = self.hits.row_of(list, cursor).unwrap_or((4, 2));
        self.context_menu = Some(ContextMenu {
            lines,
            selected: 1,
            at,
            letters: true,
        });
    }

    fn scan_key_now(&self) -> ScanKey {
        ScanKey {
            pane: self.active,
            view: self.view,
            root: self.tree.path(self.view_root()),
            filespec: self.filespec.text().to_string(),
            query: self.query.as_ref().map(|q| q.text().to_string()),
            tags_only: self.tags_only,
        }
    }

    /// The Scan of the list: the same list as the last one opens that
    /// one again (R scans again).
    pub(crate) fn open_scan(&mut self) {
        let key = self.scan_key_now();
        if let Some(view) = self.scan_back.take()
            && view.key == key
        {
            self.scan = Some(view);
            return;
        }
        self.scan = Some(self.new_scan(key));
    }

    fn new_scan(&mut self, key: ScanKey) -> ScanView {
        // The list as it would be without the choice of a view: a Scan
        // made from a picked list would only find what was picked.
        let pick = self.pick.take();
        if pick.is_some() {
            self.refresh_files();
        }
        let view = self.scan_list(key);
        if pick.is_some() {
            self.pick = pick;
            self.refresh_files();
        }
        view
    }

    /// What can be told at once from memory, and the thread for the rest.
    fn scan_list(&self, key: ScanKey) -> ScanView {
        let pane = &**self;
        let local = pane.remote.is_none() && pane.archive.is_none();
        let root_id = pane.view_root();
        let root = pane.tree.path(root_id);
        let dirs = match pane.view {
            FileView::Dir => vec![pane.current()],
            FileView::Branch(id) => pane.tree.visible_subtree(id),
            FileView::Showall => pane.tree.visible_subtree(ROOT),
        };
        let mut found: Vec<(Category, Found)> = Category::ALL
            .iter()
            .map(|&c| (c, Found::default()))
            .collect();
        let at = |found: &mut Vec<(Category, Found)>, c: Category| -> usize {
            found.iter().position(|(k, _)| *k == c).unwrap()
        };
        let (names, deceiving, perms, links) = (
            at(&mut found, Category::HiddenChars),
            at(&mut found, Category::Deceiving),
            at(&mut found, Category::Permissions),
            at(&mut found, Category::Links),
        );

        // Where the modes are made up by the driver, they say nothing.
        let mounts = if local {
            crate::platform::mounts()
        } else {
            Vec::new()
        };
        let made_up = |path: &Path| -> Option<String> {
            mounts
                .iter()
                .filter(|m| path.starts_with(&m.point))
                .max_by_key(|m| m.point.as_os_str().len())
                .filter(|m| scan::modes_are_made_up(&m.fstype))
                .map(|m| m.fstype.clone())
        };
        let mut fake: std::collections::BTreeSet<String> = Default::default();
        let mut fake_dirs: HashSet<NodeId> = HashSet::new();
        let mut tasks = Vec::new();
        let mut contents = Vec::new();
        // Why the contents are not read here, if they are not.
        let reads: Option<String> = if pane.remote.is_some() {
            Some("Contents not read on a remote machine: each file would come through SSH".into())
        } else if pane.archive.is_none() && pane.tree.reads_slowly(&root) {
            Some("Contents not read on a cloud mount: each file would be downloaded".into())
        } else {
            None
        };
        for &dir in &dirs {
            let path = pane.tree.path(dir);
            match made_up(&path) {
                Some(fs) => {
                    fake.insert(fs);
                    fake_dirs.insert(dir);
                }
                None if local && cfg!(unix) => tasks.push(Task::DirMode(path.clone())),
                None => {}
            }
            if dir != root_id {
                let name = &pane.tree.node(dir).name;
                let mut hit = |i: usize, why: String| {
                    found[i].1.push(Hit {
                        path: path.clone(),
                        dir: true,
                        why,
                    })
                };
                if let Some(why) = scan::hidden_chars(name) {
                    hit(names, why);
                }
                if let Some(why) = scan::deceiving(name) {
                    hit(deceiving, why);
                }
            }
        }
        for &f in &pane.files {
            let entry = pane.file(f);
            let path = || pane.file_path(f);
            if let Some(why) = scan::hidden_chars(&entry.name) {
                found[names].1.push(Hit {
                    path: path(),
                    dir: false,
                    why,
                });
            }
            if let Some(why) = scan::deceiving(&entry.name) {
                found[deceiving].1.push(Hit {
                    path: path(),
                    dir: false,
                    why,
                });
            }
            match entry.kind {
                crate::model::tree::FileKind::File if cfg!(unix) && !fake_dirs.contains(&f.dir) => {
                    if let Some(why) = scan::mode_problem(entry.mode, false) {
                        found[perms].1.push(Hit {
                            path: path(),
                            dir: false,
                            why,
                        });
                    }
                }
                crate::model::tree::FileKind::Symlink if local => tasks.push(Task::Link(path())),
                _ => {}
            }
            if entry.kind == crate::model::tree::FileKind::File && reads.is_none() {
                contents.push(Task::Content(path()));
            }
        }
        // What is in memory first, then the disk's quick answers, then the
        // contents.
        tasks.append(&mut contents);
        for cat in [Category::Executables, Category::Office, Category::Patches] {
            let i = at(&mut found, cat);
            found[i].1.note = reads.clone();
        }

        // What was not looked at, said in the category.
        found[perms].1.note = if !cfg!(unix) {
            Some("Not here: Windows keeps no setuid nor modes of this kind".into())
        } else if !fake.is_empty() {
            let list = fake.into_iter().collect::<Vec<_>>().join(", ");
            let some = if fake_dirs.len() == dirs.len() {
                "Not checked"
            } else {
                "Not checked in some folders"
            };
            Some(format!(
                "{some}: on {list} the modes are made up by the driver (the same for every file), \
                 so they say nothing"
            ))
        } else if pane.archive.is_some() {
            Some("Inside an archive only the files' modes are checked".into())
        } else if pane.remote.is_some() {
            Some("On a remote machine only the files' modes are checked".into())
        } else {
            None
        };
        found[links].1.note = if pane.archive.is_some() {
            Some("Not looked at inside an archive".into())
        } else if pane.remote.is_some() {
            Some("Not looked at on a remote machine yet".into())
        } else {
            None
        };

        let what = match pane.view {
            FileView::Dir => "Files",
            FileView::Branch(_) => "Branch",
            FileView::Showall => "Showall",
        };
        let what = match &pane.archive {
            Some(archive) => format!("{what} {} (in {})", root.display(), archive.display()),
            None => format!("{what} {}", root.display()),
        };
        let source = match &pane.archive {
            Some(archive) => scan::Source::Archive(archive.clone()),
            None => scan::Source::Disk,
        };
        let limits = scan::Limits::new(self.options.scan_max_files as usize);
        let job =
            (!tasks.is_empty()).then(|| scan::Job::start(tasks, root.clone(), source, limits));
        ScanView {
            what,
            files: pane.files.len(),
            folders: dirs.len(),
            found,
            selected: 0,
            open: None,
            hit: 0,
            job,
            stopped: None,
            key,
        }
    }

    /// Takes in what the Scan's thread found.
    pub(crate) fn scan_tick(&mut self, budget: Duration) {
        let Some(view) = &mut self.scan else {
            return;
        };
        if !view.poll() && !self.busy_without_box() {
            // The thread does the work: only a little wait here.
            std::thread::sleep(budget.min(Duration::from_millis(20)));
        }
    }

    /// The keys of the Scan screen.
    pub(crate) fn scan_key(&mut self, key: KeyEvent) {
        let Some(view) = &mut self.scan else {
            return;
        };
        if let Some(open) = view.open {
            let n = view.found[open].1.hits.len();
            match key.code {
                KeyCode::Esc | KeyCode::Left => view.open = None,
                KeyCode::Up | KeyCode::Char('k') => view.hit = view.hit.saturating_sub(1),
                KeyCode::Down | KeyCode::Char('j') => {
                    view.hit = (view.hit + 1).min(n.saturating_sub(1))
                }
                KeyCode::PageUp => view.hit = view.hit.saturating_sub(10),
                KeyCode::PageDown => view.hit = (view.hit + 10).min(n.saturating_sub(1)),
                KeyCode::Home => view.hit = 0,
                KeyCode::End => view.hit = n.saturating_sub(1),
                KeyCode::Enter => self.scan_to_list(),
                KeyCode::Char('v' | 'V') => self.scan_view_hit(),
                _ => {}
            }
            return;
        }
        let n = view.found.len();
        match key.code {
            KeyCode::Esc if view.running() => {
                let job = view.job.take().unwrap();
                job.stop();
                view.stopped = Some((job.done(), job.total));
                for (cat, hit) in job.drain() {
                    view.found_mut(cat).push(hit);
                }
            }
            KeyCode::Esc => self.close_scan(),
            KeyCode::Up | KeyCode::Char('k') => view.selected = view.selected.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => view.selected = (view.selected + 1).min(n - 1),
            KeyCode::Home => view.selected = 0,
            KeyCode::End => view.selected = n - 1,
            KeyCode::Enter | KeyCode::Right => {
                if view.found[view.selected].1.hits.is_empty() {
                    self.message = Some("Nothing found in this category".into());
                } else {
                    view.open = Some(view.selected);
                    view.hit = 0;
                }
            }
            KeyCode::Char('r' | 'R') => {
                let key = view.key.clone();
                self.scan = None;
                let fresh = self.new_scan(key);
                self.scan = Some(fresh);
            }
            _ => {}
        }
    }

    /// Esc: back to the list as it was; the Scan is kept for Alt+V, S.
    pub(crate) fn close_scan(&mut self) {
        let Some(mut view) = self.scan.take() else {
            return;
        };
        view.open = None;
        // A Scan stopped or still running is not kept: the next one starts over.
        if view.job.is_none() && view.stopped.is_none() {
            self.scan_back = Some(view);
        }
    }

    /// Enter on a hit: the list keeps only the files of its category, with
    /// the cursor on it; a folder is shown in the tree.
    fn scan_to_list(&mut self) {
        let Some(view) = &self.scan else {
            return;
        };
        let Some((cat, found)) = view.open_hits() else {
            return;
        };
        let Some(hit) = found.hits.get(view.hit).cloned() else {
            return;
        };
        let paths: HashSet<PathBuf> = found
            .hits
            .iter()
            .filter(|h| !h.dir)
            .map(|h| h.path.clone())
            .collect();
        let files = paths.len();
        self.close_scan();
        if hit.dir {
            if self.goto(&hit.path) {
                self.focus = Focus::Tree;
                self.message = Some(format!("{}: {}", hit.path.display(), hit.why));
            } else {
                self.message = Some(format!("{} is no longer in the tree", hit.path.display()));
            }
            return;
        }
        self.pick = Some(Pick {
            label: format!("Scan: {}", cat.short()),
            paths: Arc::new(paths),
        });
        self.refresh_files();
        if let Some(i) = self
            .files
            .iter()
            .position(|&f| self.file_path(f) == hit.path)
        {
            self.file_state.select(Some(i));
        }
        self.focus = Focus::Files;
        self.message = Some(format!(
            "{files} file{} from the Scan: Ctrl+T tags them, W with no text shows all again",
            if files == 1 { "" } else { "s" }
        ));
    }

    /// V on a hit: the file in the viewer, with its report; Esc comes back
    /// to the Scan.
    fn scan_view_hit(&mut self) {
        let Some(view) = &self.scan else {
            return;
        };
        let Some((_, found)) = view.open_hits() else {
            return;
        };
        let Some(hit) = found.hits.get(view.hit).cloned() else {
            return;
        };
        if hit.dir {
            self.message = Some("A folder: Enter shows it in the tree".into());
            return;
        }
        let size = self
            .files
            .iter()
            .find(|&&f| self.file_path(f) == hit.path)
            .map(|&f| self.file(f).size)
            .unwrap_or(0);
        match self.load_viewer(hit.path.clone(), size) {
            Ok(v) => self.viewer = Some(v),
            Err(e) => self.message = Some(format!("Cannot view file: {e}")),
        }
    }
}

/// The Timeline on screen.
pub struct TimelineView {
    /// What it was made of: "Branch /home/me/photos".
    pub what: String,
    pub line: crate::timeline::Timeline,
    /// The bar under the cursor, and where a range (Shift+← →) started.
    pub cursor: usize,
    pub anchor: Option<usize>,
    /// `B`: bars by the bytes instead of the files.
    pub bytes: bool,
    /// The first bar on screen, kept by whoever draws it.
    pub offset: std::cell::Cell<usize>,
}

impl TimelineView {
    /// The bars chosen: the range, or the one under the cursor.
    pub fn chosen(&self) -> (usize, usize) {
        let a = self.anchor.unwrap_or(self.cursor);
        (a.min(self.cursor), a.max(self.cursor))
    }

    /// Files and bytes of the bars chosen.
    pub fn chosen_sum(&self) -> (u64, u64) {
        let (a, b) = self.chosen();
        self.line.bars[a..=b].iter().fold((0, 0), |(f, s), bar| {
            (f + bar.files, s.saturating_add(bar.bytes))
        })
    }

    /// The name of what is chosen: one bar, or "from ... to ...".
    pub fn chosen_label(&self) -> String {
        let (a, b) = self.chosen();
        if a == b {
            self.line.label(a)
        } else {
            format!("{} to {}", self.line.label(a), self.line.label(b))
        }
    }
}

impl App {
    /// Alt+V, T: the files of the list by the date they were changed.
    pub(crate) fn open_timeline(&mut self) {
        if self.files.is_empty() {
            self.message = Some("No files in the list".into());
            return;
        }
        let pane = &**self;
        let files = pane.files.iter().map(|&f| {
            let e = pane.file(f);
            (e.modified, e.size)
        });
        let line = crate::timeline::Timeline::new(files, std::time::SystemTime::now());
        if line.bars.is_empty() {
            self.message = Some("No file of the list has a date".into());
            return;
        }
        // The most recent bar that is not in the future.
        let cursor = line
            .bars
            .iter()
            .rposition(|b| b.span != crate::timeline::Span::Future)
            .unwrap_or(0);
        let what = match pane.view {
            FileView::Dir => "Files",
            FileView::Branch(_) => "Branch",
            FileView::Showall => "Showall",
        };
        let what = format!("{what} {}", pane.tree.path(pane.view_root()).display());
        self.timeline = Some(TimelineView {
            what,
            line,
            cursor,
            anchor: None,
            bytes: false,
            offset: std::cell::Cell::new(0),
        });
    }

    /// The keys of the Timeline.
    pub(crate) fn timeline_key(&mut self, key: KeyEvent) {
        let Some(view) = &mut self.timeline else {
            return;
        };
        let last = view.line.bars.len().saturating_sub(1);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let moved = |view: &mut TimelineView, to: usize| {
            if shift {
                view.anchor.get_or_insert(view.cursor);
            } else {
                view.anchor = None;
            }
            view.cursor = to.min(last);
        };
        match key.code {
            KeyCode::Esc => self.timeline = None,
            KeyCode::Left | KeyCode::Char('h') => {
                let to = view.cursor.saturating_sub(1);
                moved(view, to)
            }
            KeyCode::Right | KeyCode::Char('l') => {
                let to = view.cursor + 1;
                moved(view, to)
            }
            KeyCode::PageUp => {
                let to = view.cursor.saturating_sub(10);
                moved(view, to)
            }
            KeyCode::PageDown => {
                let to = view.cursor + 10;
                moved(view, to)
            }
            KeyCode::Home => moved(view, 0),
            KeyCode::End => moved(view, last),
            KeyCode::Char('b' | 'B') => view.bytes = !view.bytes,
            KeyCode::Char('+' | '=') | KeyCode::Char('-' | '_') => {
                let finer = matches!(key.code, KeyCode::Char('+' | '='));
                let next = if finer {
                    view.line.scale.finer()
                } else {
                    view.line.scale.coarser()
                };
                let Some(next) = next else {
                    self.message = Some(format!(
                        "No scale {} than {}",
                        if finer { "finer" } else { "coarser" },
                        view.line.scale.name()
                    ));
                    return;
                };
                // The cursor stays on the same moment.
                let here = match view.line.bars[view.cursor].span {
                    crate::timeline::Span::Period { start, .. } => Some(start),
                    _ => None,
                };
                let was_future = view.line.bars[view.cursor].span == crate::timeline::Span::Future;
                if !view.line.set_scale(next) {
                    self.message = Some(format!(
                        "Too many bars by {}: choose a shorter period first (Enter)",
                        next.name()
                    ));
                    return;
                }
                let n = view.line.bars.len();
                view.cursor = match here {
                    Some(t) => view.line.bar_of(t).unwrap_or(0),
                    None if was_future => n - 1,
                    None => 0,
                };
                view.anchor = None;
                self.message = Some(format!("By {}", next.name()));
            }
            KeyCode::Enter => self.timeline_to_list(),
            _ => {}
        }
    }

    /// Enter: the list keeps the files of the bars chosen, as a condition
    /// of W on `modified`, with AND to the one there was.
    fn timeline_to_list(&mut self) {
        let Some(view) = &self.timeline else {
            return;
        };
        let (a, b) = view.chosen();
        let Some(cond) = view.line.condition(a, b) else {
            return;
        };
        let (text, tagged) = match &self.query {
            Some(q) => (format!("({}) AND ({cond})", q.text()), q.tagged()),
            None => (cond, false),
        };
        match self.set_query(&text, tagged) {
            Ok(()) => {
                self.timeline = None;
                self.focus = Focus::Files;
            }
            Err(e) => self.message = Some(e),
        }
    }
}

impl App {
    /// Alt+V, L: the list in the table viewer. The list is copied once,
    /// compactly (see `viewer::list_rows`); W in the table brings its filter
    /// back as the list's `W`.
    pub(crate) fn open_list_table(&mut self) {
        if self.files.is_empty() {
            self.message = Some("No files in the list".into());
            return;
        }
        let pane = &**self;
        let wants_path = (pane.git_enabled && pane.remote.is_none() && pane.archive.is_none())
            || !pane.duplicate_groups.is_empty();
        let mut dirs: Vec<(PathBuf, usize)> = Vec::new();
        let mut dir_of = std::collections::HashMap::new();
        let mut files = Vec::with_capacity(pane.files.len());
        for &f in &pane.files {
            let dir = *dir_of.entry(f.dir).or_insert_with(|| {
                let mut depth = 0;
                let mut at = f.dir;
                while let Some(parent) = pane.tree.node(at).parent {
                    depth += 1;
                    at = parent;
                }
                dirs.push((pane.tree.path(f.dir), depth));
                (dirs.len() - 1) as u32
            });
            let entry = pane.file(f).clone();
            let (git, dup) = if wants_path {
                let path = dirs[dir as usize].0.join(&entry.name);
                (
                    pane.file_git(&path).map(String::into_boxed_str),
                    pane.dup_group(&path).map(|g| g as u32),
                )
            } else {
                (None, None)
            };
            files.push(crate::viewer::list_rows::ListFile {
                entry,
                dir,
                git,
                dup,
            });
        }
        let what = match pane.view {
            FileView::Dir => "Files",
            FileView::Branch(_) => "Branch",
            FileView::Showall => "Showall",
        };
        let title = PathBuf::from(format!(
            "{what} {}",
            pane.tree.path(pane.view_root()).display()
        ));
        let list = Arc::new(crate::viewer::list_rows::ListSnapshot::new(
            dirs,
            files,
            std::time::SystemTime::now(),
        ));
        match Viewer::from_list(title, list.opener()) {
            Ok(mut v) => {
                v.note = Some("\\ filters the table, W takes the filter back to the list".into());
                self.viewer = Some(v);
            }
            Err(e) => self.message = Some(format!("Cannot show the list as a table: {e}")),
        }
    }

    /// W in the table of the list: its SQL filter becomes the list's
    /// condition of W, with AND to the one there was.
    pub(crate) fn list_table_to_list(&mut self) {
        let Some(v) = &mut self.viewer else {
            return;
        };
        let Some(cond) = v.table.as_ref().and_then(|t| t.active_filter.clone()) else {
            v.note = Some("No filter: \\ filters the table, then W takes it to the list".into());
            return;
        };
        let (text, tagged) = match &self.query {
            Some(q) => (format!("({}) AND ({cond})", q.text()), q.tagged()),
            None => (cond, false),
        };
        match self.set_query(&text, tagged) {
            Ok(()) => {
                self.viewer = None;
                self.focus = Focus::Files;
            }
            Err(e) => {
                if let Some(v) = &mut self.viewer {
                    v.note = Some(e);
                }
            }
        }
    }
}
