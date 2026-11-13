//! The Duplicates view (Alt+D): opening it and its keys.

use super::*;
use crate::duplicates::{DuplicatesView, Fail, Finder, Scan};
use crate::usage;
use std::collections::HashMap;

impl App {
    pub(crate) fn open_duplicates(&mut self) {
        let id = self.current();
        self.duplicates_of(id);
    }

    /// Looks for duplicates in the branch of `id`: logs it, on one disk.
    pub(crate) fn duplicates_of(&mut self, id: NodeId) {
        if self.archive.is_some() {
            self.message = Some("Duplicates do not look inside archives".into());
            return;
        }
        // In the cloud, the service knows the hashes: nothing to log or read.
        let path = self.tree.path(id);
        if let Some(mount) = crate::info::mount_of(&path)
            && let Some(remote) = crate::rclone::of(&mount)
            && let Some(spec) = crate::rclone::spec_of(&remote.spec, &mount.point, &path)
        {
            self.duplicates_root = Some(path.clone());
            self.duplicates_id = Some(id);
            self.duplicates_job = Some(Finder::start_cloud(
                remote,
                spec,
                path,
                self.options.duplicates_min_size,
                self.options.duplicates_ignore.clone(),
            ));
            return;
        }
        self.duplicates_log(id);
    }

    fn duplicates_log(&mut self, id: NodeId) {
        let device = self.tree.device(id);
        self.start_job(LogJob::one_disk(id, device), AfterLog::Duplicates(id));
    }

    /// The branch is in memory. On a network, reading it costs: asks first.
    pub(crate) fn duplicates_logged(&mut self, id: NodeId) {
        let path = self.tree.path(id);
        let network = self.remote.is_some()
            || crate::info::mount_of(&path).is_some_and(|m| is_network(&m.fstype));
        if network {
            let total = match self.duplicates_scan(id) {
                Ok(scan) => scan.to_read(),
                Err(e) => {
                    self.message = Some(e);
                    return;
                }
            };
            if total > 0 {
                let why = self
                    .duplicates_why
                    .take()
                    .map_or(String::new(), |w| format!("{w}. "));
                let place = match &self.remote {
                    Some(_) => "on a remote machine",
                    None => "on a network",
                };
                let q = format!(
                    "{why}This folder is {place}: comparing reads up to {}. Continue?",
                    usage::size(total)
                );
                self.ask(ConfirmKind::ReadThroughNetwork(id), q);
                return;
            }
        }
        self.duplicates_compare(id);
    }

    fn duplicates_scan(&self, id: NodeId) -> Result<Scan, String> {
        let (min, ignore) = (
            self.options.duplicates_min_size,
            &self.options.duplicates_ignore,
        );
        let Some(remote) = &self.remote else {
            return Ok(Scan::of(&self.tree, id, min, ignore));
        };
        // The tree has no devices there: each folder's filesystem says which
        // are other disks, as for Prune.
        let disks = self.remote_disks(remote, id)?;
        let root = disks.get(&id).copied();
        let other_disk = |c| disks.get(&c).copied() != root;
        let scan = Scan::of_disks(&self.tree, id, min, ignore, &other_disk);
        Ok(scan.on_remote(remote.clone()))
    }

    /// The filesystem of each folder of the branch of `id`, on a remote
    /// machine.
    fn remote_disks(
        &self,
        remote: &crate::remote::Remote,
        id: NodeId,
    ) -> Result<HashMap<NodeId, crate::remote::sftp::Filesystem>, String> {
        let mut disks = HashMap::new();
        let mut stack = vec![id];
        while let Some(n) = stack.pop() {
            let path = self.tree.path(n);
            match remote.filesystem(&path) {
                Ok(fs) => {
                    disks.insert(n, fs);
                }
                Err(e) if n == id => return Err(format!("Cannot compare: {e}")),
                // Unreadable: left out, as another disk would be.
                Err(_) => continue,
            }
            stack.extend(self.tree.node(n).children.iter().copied());
        }
        Ok(disks)
    }

    pub(crate) fn duplicates_compare(&mut self, id: NodeId) {
        self.duplicates_why = None;
        let scan = match self.duplicates_scan(id) {
            Ok(scan) => scan,
            Err(e) => {
                self.message = Some(e);
                return;
            }
        };
        self.duplicates_root = Some(self.tree.path(id));
        self.duplicates_job = Some(Finder::start(scan));
    }

    pub(crate) fn duplicates_tick(&mut self, budget: Duration) {
        let Some(job) = &self.duplicates_job else {
            return;
        };
        // Nothing to do here but wait: the thread does the reading.
        let Some(result) = job.poll() else {
            std::thread::sleep(budget.min(Duration::from_millis(20)));
            return;
        };
        self.duplicates_job = None;
        let root = self.duplicates_root.take();
        match result {
            Ok(found) => {
                let Some(root) = root else { return };
                if found.files.is_empty() && found.folders.is_empty() {
                    let mut text = "No duplicates found".to_string();
                    let not_read = found.notes.unreadable + found.notes.changed;
                    if not_read > 0 {
                        text.push_str(&format!(" ({not_read} files could not be compared)"));
                    }
                    self.message = Some(text);
                    return;
                }
                let mut view = DuplicatesView::new(root, found);
                if view.files.is_empty() {
                    view.toggle_folders();
                }
                self.duplicates = Some(view);
            }
            Err(Fail::Cancelled) => self.message = Some("Cancelled".into()),
            Err(Fail::Stopped(why)) => self.message = Some(format!("Cannot compare: {why}")),
            // No hashes: reading the files is what is left, and it is asked.
            Err(Fail::NoHashes(why)) => match self.duplicates_id.take() {
                Some(id) => {
                    self.duplicates_why = Some(why);
                    self.duplicates_log(id);
                }
                None => self.message = Some(format!("Cannot compare: {why}")),
            },
        }
    }

    pub(crate) fn duplicates_key(&mut self, key: KeyEvent) {
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        // Esc, or the key that opened it (Alt+D), closes it.
        if key.code == KeyCode::Esc
            || self.keymap.lookup(key, self.focus) == Some(Command::Duplicates)
        {
            self.duplicates = None;
            return;
        }
        let Some(view) = self.duplicates.as_mut() else {
            return;
        };
        let page = view.page.max(1) as isize;
        let len = view.rows().len() as isize;
        let plain = !alt && !ctrl;
        match key.code {
            KeyCode::Up => view.move_by(-1),
            KeyCode::Down => view.move_by(1),
            KeyCode::PageUp => view.move_by(-page),
            KeyCode::PageDown => view.move_by(page),
            KeyCode::Home => view.move_by(-len),
            KeyCode::End => view.move_by(len),
            KeyCode::Delete => self.duplicates_delete(),
            KeyCode::Char('d' | 'D') => self.duplicates_delete(),
            KeyCode::Char('m' | 'M') if !alt => self.duplicates_move(),
            KeyCode::Enter => self.duplicates_view_file(),
            KeyCode::Char('v' | 'V') if plain => self.duplicates_view_file(),
            KeyCode::Char(' ') if plain => self.duplicates_toggle(),
            KeyCode::Char('t' | 'T') if plain => self.duplicates_toggle(),
            KeyCode::Char('k' | 'K') if plain => view.keep_selected(),
            KeyCode::Char('a' | 'A') if plain => {
                let keep = view.tag_all_but_one();
                self.message = Some(format!(
                    "Tagged all but {} of each group (A again: another)",
                    keep.label()
                ));
            }
            KeyCode::Char('f' | 'F') if plain => {
                let count = view.keep_in_folder();
                self.message = Some(format!("Kept the copy in this folder in {count} group(s)"));
            }
            KeyCode::Char('u' | 'U') if plain => view.untag_all(),
            KeyCode::Char('g' | 'G') if plain => {
                if view.folders.is_empty() && !view.show_folders {
                    self.message = Some("No whole folders are the same".into());
                } else if view.files.is_empty() && view.show_folders {
                    self.message = Some("Only whole folders are the same".into());
                } else {
                    view.toggle_folders();
                }
            }
            KeyCode::Char('e' | 'E') if plain => {
                let default = view.root.join("duplicates.txt");
                let default = default.to_string_lossy().into_owned();
                self.open_prompt(
                    PromptKind::ExportDuplicates,
                    "WRITE the list to: ".into(),
                    &default,
                );
            }
            KeyCode::Char('\\') => {
                let dir = view.selected().and_then(|i| {
                    let is_dir = i.snapshot.is_some();
                    if is_dir {
                        Some(i.path.clone())
                    } else {
                        i.path.parent().map(Path::to_path_buf)
                    }
                });
                self.duplicates = None;
                if let Some(dir) = dir {
                    self.goto(&dir);
                }
                self.focus = Focus::Tree;
            }
            KeyCode::F(3) | KeyCode::F(2) => {
                let root = view.root.clone();
                self.duplicates = None;
                if self.goto(&root) {
                    let id = self.current();
                    // Whatever changed since the tree read it is read again.
                    for dir in self.tree.subtree(id) {
                        self.tree.rescan(dir);
                    }
                    self.duplicates_of(id);
                }
            }
            _ => {
                // What makes sense with the view open: windows, the
                // terminal, settings and quitting.
                if let Some(cmd) = self.keymap.lookup(key, self.focus)
                    && matches!(
                        cmd,
                        Command::Split
                            | Command::SwitchWindow
                            | Command::SwitchPane
                            | Command::Terminal
                            | Command::TerminalFull
                            | Command::Settings
                            | Command::Quit
                            | Command::QuitChangeDir
                    )
                {
                    self.apply(cmd);
                }
            }
        }
    }

    fn duplicates_toggle(&mut self) {
        let Some(view) = self.duplicates.as_mut() else {
            return;
        };
        if view.toggle() {
            view.move_by(1);
        } else {
            self.message = Some("One copy of each group always stays".into());
        }
    }

    fn duplicates_view_file(&mut self) {
        let Some(item) = self.duplicates.as_ref().and_then(|v| v.selected().cloned()) else {
            return;
        };
        if item.snapshot.is_some() {
            self.message = Some("A folder: \\ goes to it in the tree".into());
            return;
        }
        match self.load_viewer(item.path, item.size) {
            Ok(v) => self.viewer = Some(v),
            Err(e) => self.message = Some(format!("Cannot view file: {e}")),
        }
    }

    /// The tagged items that may still go, saying what was refused.
    fn duplicates_checked(&mut self) -> Vec<crate::duplicates::Item> {
        let Some(view) = self.duplicates.as_mut() else {
            return Vec::new();
        };
        let had = view.tagged().0;
        let (items, refused) = view.check_tagged();
        if refused > 0 {
            self.message = Some(format!(
                "{refused} group(s) changed since the analysis and were left alone: F3 analyzes again"
            ));
        } else if had == 0 {
            self.message = Some("Nothing tagged: T tags a copy, A all but one of each".into());
        }
        items
    }

    fn duplicates_delete(&mut self) {
        let show_folders = self.duplicates.as_ref().is_some_and(|v| v.show_folders);
        let cloud = self.duplicates.as_ref().is_some_and(|v| v.cloud.is_some());
        let items = self.duplicates_checked();
        if items.is_empty() {
            return;
        }
        // In the cloud the service has its own trash; a local one means nothing.
        let to_trash = self.options.delete_to_trash && !cloud;
        if show_folders {
            let dirs = items.into_iter().map(|i| i.path).collect();
            self.ask_delete_folders(dirs, to_trash);
        } else {
            let sources = items
                .into_iter()
                .map(|i| Source {
                    path: i.path,
                    size: i.size,
                })
                .collect();
            self.ask_delete_files(sources, to_trash);
        }
    }

    fn duplicates_move(&mut self) {
        if self.duplicates.as_ref().is_some_and(|v| v.show_folders) {
            self.message = Some("Moving works on files; D deletes folders".into());
            return;
        }
        let items = self.duplicates_checked();
        if items.is_empty() {
            return;
        }
        let sources = items
            .into_iter()
            .map(|i| Source {
                path: i.path,
                size: i.size,
            })
            .collect();
        self.prompt_transfer_of(OpKind::Move, sources, true, false);
    }

    pub(crate) fn ask_delete_folders(&mut self, dirs: Vec<PathBuf>, to_trash: bool) {
        let what = match dirs.as_slice() {
            [dir] => format!("folder {} and all it holds", fit_name(dir)),
            _ => format!("{} folders and all they hold", dirs.len()),
        };
        let to_trash = to_trash && self.remote.is_none();
        let q = match &self.remote {
            Some(r) => format!(
                "DELETE {what} on {}? There is no trash there: this cannot be undone.",
                r.address.destination()
            ),
            None if to_trash => format!("MOVE {what} to trash?"),
            None => format!("DELETE {what}? This cannot be undone."),
        };
        let options: &'static [&'static str] = match (&self.remote, to_trash) {
            (Some(_), _) => &["Yes", "No"],
            (None, true) => &["Yes", "No", "Permanent"],
            (None, false) => &["Yes", "No", "Trash"],
        };
        self.ask_options(ConfirmKind::DeleteFolders { dirs, to_trash }, q, options);
    }

    pub(crate) fn delete_folders(&mut self, dirs: Vec<PathBuf>, to_trash: bool) {
        if let Some(remote) = self.remote.clone() {
            return self.delete_remote_folders(&remote, dirs);
        }
        if to_trash {
            let sources = dirs
                .into_iter()
                .map(|path| Source { path, size: 0 })
                .collect();
            self.start_op(Operation::delete(sources, true));
            return;
        }
        // The same steps as a Prune: the files, then the empty folders.
        let mut files = Vec::new();
        let mut folders = Vec::new();
        let mut stack = Vec::new();
        for dir in dirs {
            match fs::symlink_metadata(&dir) {
                Ok(m) if m.is_dir() => {
                    let device = platform::device(&dir, &m);
                    stack.push((dir, device));
                }
                Ok(_) => {
                    self.message = Some(format!("Not a folder: {}", dir.display()));
                    return;
                }
                Err(e) => {
                    self.message = Some(format!("Cannot read {}: {e}", dir.display()));
                    return;
                }
            }
        }
        while let Some((dir, device)) = stack.pop() {
            let entries = match fs::read_dir(&dir) {
                Ok(entries) => entries,
                Err(e) => {
                    self.message = Some(format!("Cannot read {}: {e}", dir.display()));
                    return;
                }
            };
            for entry in entries.flatten() {
                let path = entry.path();
                match fs::symlink_metadata(&path) {
                    // Another disk mounted since: it stays, and so do the
                    // folders above it, which the summary reports.
                    Ok(m) if m.is_dir() => {
                        if platform::device(&path, &m) == device {
                            stack.push((path, device));
                        }
                    }
                    Ok(m) => files.push(Source {
                        path,
                        size: m.len(),
                    }),
                    Err(_) => {}
                }
            }
            folders.push(dir);
        }
        self.start_op(Operation::prune(files, folders));
    }

    /// [`App::delete_folders`] on a remote machine: no trash, and another
    /// disk mounted inside (told by its filesystem) stays.
    fn delete_remote_folders(&mut self, remote: &crate::remote::Remote, dirs: Vec<PathBuf>) {
        use crate::model::tree::{FileKind, Lister};
        let (mut files, mut folders) = (Vec::new(), Vec::new());
        let mut stack = Vec::new();
        for dir in dirs {
            let disk = remote.attrs(&dir).and_then(|a| {
                if a.is_dir() {
                    remote.filesystem(&dir)
                } else {
                    Err(std::io::Error::other("not a folder"))
                }
            });
            match disk {
                Ok(disk) => stack.push((dir, disk)),
                Err(e) => {
                    self.message = Some(format!("Cannot delete {}: {e}", dir.display()));
                    return;
                }
            }
        }
        while let Some((dir, disk)) = stack.pop() {
            let (subdirs, entries) = match remote.list(&dir) {
                Ok(listed) => listed,
                Err(e) => {
                    self.message = Some(format!("Cannot read {}: {e}", dir.display()));
                    return;
                }
            };
            for (name, _) in subdirs {
                let path = dir.join(name);
                if remote.filesystem(&path).is_ok_and(|d| d == disk) {
                    stack.push((path, disk));
                }
            }
            for f in entries {
                let size = if f.kind == FileKind::File { f.size } else { 0 };
                files.push(Source {
                    path: dir.join(&f.name),
                    size,
                });
            }
            folders.push(dir);
        }
        match self.on_this_side(Operation::prune(files, folders)) {
            Ok(op) => self.start_op(op),
            Err(e) => self.message = Some(e),
        }
    }

    /// Writes the list shown, never over a file that exists.
    pub(crate) fn export_duplicates(&mut self, text: &str) -> Result<(), String> {
        use std::io::Write;
        let Some(view) = self.duplicates.as_ref() else {
            return Ok(());
        };
        let path = ops::resolve_path(text, &view.root);
        let list = view.export();
        let write = || -> std::io::Result<()> {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)?;
            file.write_all(list.as_bytes())
        };
        write().map_err(|e| format!("Cannot write {}: {e}", path.display()))?;
        self.message = Some(format!("List written to {}", path.display()));
        Ok(())
    }
}

/// A file system whose reads go through a network, and may cost money or a
/// long time.
fn is_network(fstype: &str) -> bool {
    const PREFIXES: [&str; 9] = [
        "nfs",
        "cifs",
        "smb",
        "9p",
        "afs",
        "ceph",
        "davfs",
        "fuse.rclone",
        "fuse.sshfs",
    ];
    PREFIXES.iter().any(|p| fstype.starts_with(p))
}
