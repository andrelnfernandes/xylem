use super::*;

impl App {
    /// F2: picks the prompt's folder in a tree, starting at the one typed.
    pub(crate) fn open_picker(&mut self) {
        let Some(prompt) = &self.prompt else {
            return;
        };
        let base = self.outer.as_deref().unwrap_or(self).current_path();
        let typed = ops::resolve_path(&prompt.input.text, &base);
        let start = match prompt.kind.folder() {
            Some(true) => typed.parent().map_or(base, Path::to_path_buf),
            _ => typed,
        };
        // Archives are read-only: their window offers the disk behind them.
        let windows: Vec<(PathBuf, PathBuf, bool)> = self
            .panes
            .iter()
            .map(|p| {
                let p = p.outer.as_deref().unwrap_or(p);
                (p.tree.path(ROOT), p.current_path(), p.tree.show_hidden)
            })
            .collect();
        self.picker = Some(Picker::new(&windows, self.active, &start));
    }

    pub(crate) fn picker_key(&mut self, key: KeyEvent) {
        let Some(picker) = &mut self.picker else {
            return;
        };
        match picker.handle_key(key, &self.keymap) {
            PickerOutcome::Stay => {}
            PickerOutcome::Cancel => self.picker = None,
            PickerOutcome::Chosen(dir) => {
                self.picker = None;
                let Some(prompt) = &mut self.prompt else {
                    return;
                };
                // Ctrl+B and F5 keep the typed file name.
                let path = match prompt.kind.folder() {
                    Some(true) => match Path::new(&prompt.input.text).file_name() {
                        Some(name) => dir.join(name),
                        None => dir,
                    },
                    _ => dir,
                };
                prompt.input.set_text(path.to_string_lossy().into_owned());
            }
        }
    }

    /// Tags, in the active window's list, the files that have the requested
    /// relation to their counterpart in the other window's directory. In Branch
    /// and Showall, the counterpart follows the path relative to the list root
    /// (`src/a.rs` → `<other>/src/a.rs`).
    pub(crate) fn compare(&mut self, mode: CompareMode) {
        let Some(other_base) = self.other().map(Pane::current_path) else {
            return;
        };
        let base = self.tree.path(self.view_root());
        let files = self.files.clone();
        let mut count = 0;
        for &f in &files {
            let path = self.file_path(f);
            let rel = path.strip_prefix(&base).unwrap_or(&path);
            let target = fs::symlink_metadata(other_base.join(rel)).ok();
            let entry = self.file(f);
            let tolerance = compare::Tolerance {
                two_seconds: self.options.compare_ignore_2s,
                one_hour: self.options.compare_ignore_1h,
            };
            let tag = mode.matches(compare::relation(
                entry.size,
                entry.modified,
                target.as_ref(),
                tolerance,
            ));
            count += usize::from(tag);
            self.set_tags(&[f], |_| tag);
        }
        self.message = Some(format!(
            "Tagged {count} of {} files: {} (compared with {})",
            files.len(),
            mode.describe(),
            other_base.display()
        ));
    }

    /// `O`: opens the ZIP/TAR under the cursor as if it were a directory.
    pub(crate) fn open_archive(&mut self) {
        let Some(f) = self.selected_file() else {
            return;
        };
        let path = self.file_path(f);
        if self.remote.is_some() {
            self.message = Some("Archives on a remote machine: copy them here first".into());
        } else if self.archive.is_some() {
            self.message = Some("Archives inside archives are not supported".into());
        } else if archive::Kind::detect(&path).is_none() {
            self.message =
                Some("Not an archive (zip, jar, 7z, tar, tar.gz, tar.xz, tar.bz2, tar.zst)".into());
        } else {
            self.opening = Some(Opening {
                pane: self.active,
                listing: ArchiveContents::read(&path),
                path,
                shown: String::new(),
            });
            self.poll_opening();
        }
    }

    /// Called on every pass of the main loop while an archive is being read:
    /// shows how far it got, and opens the window when it is done.
    pub fn poll_opening(&mut self) {
        let Some(opening) = &mut self.opening else {
            return;
        };
        let Some(result) = opening.listing.poll() else {
            let (entries, bytes) = opening.listing.progress();
            let name = opening
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            let text = format!(
                "Reading {name}: {entries} {}, {} (Esc cancels)",
                if entries == 1 { "entry" } else { "entries" },
                crate::usage::size(bytes)
            );
            // Not over what something else has to say in the meantime.
            if self.message.is_none() || self.message.as_deref() == Some(opening.shown.as_str()) {
                self.message = Some(text.clone());
                opening.shown = text;
            }
            return;
        };
        let opening = self.opening.take().unwrap();
        if self.message.as_deref() == Some(opening.shown.as_str()) {
            self.message = None;
        }
        match result {
            Ok(contents) => {
                let window = &mut self.panes[opening.pane];
                let pane = Pane::from_archive(&opening.path, contents);
                let outer = std::mem::replace(window, pane);
                window.outer = Some(Box::new(outer));
            }
            Err(e) => self.message = Some(format!("Cannot open archive: {e}")),
        }
    }

    /// Esc while an archive is being read: lets its thread go.
    pub(crate) fn cancel_opening(&mut self) {
        if let Some(opening) = self.opening.take() {
            let name = opening
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            self.message = Some(format!("Reading {name} cancelled"));
        }
    }

    /// `\`: opens the Treespec prompt with the current path. In Branch and
    /// Showall, goes straight to the selected file's directory.
    pub(crate) fn start_treespec(&mut self) {
        if self.view != FileView::Dir {
            if let Some(f) = self.selected_file() {
                let name = self.file(f).name.clone();
                self.view = FileView::Dir;
                self.show_node(f.dir);
                self.refresh_files();
                if let Some(i) = self.files.iter().position(|&r| self.file(r).name == name) {
                    self.file_state.select(Some(i));
                }
            }
            return;
        }
        let mut text = self.current_path().to_string_lossy().into_owned();
        if !text.ends_with(std::path::is_separator) {
            text.push(std::path::MAIN_SEPARATOR);
        }
        let origin = self.current();
        self.treespec_back.clear();
        self.open_prompt(PromptKind::Treespec { origin }, "TREESPEC: ".into(), &text);
    }

    /// Ctrl+← in the Treespec: the path one folder up, ending in a separator
    /// so that what is typed next looks inside it. None at the root.
    fn treespec_up(text: &str) -> Option<String> {
        let trimmed = text.trim_end_matches(std::path::is_separator);
        let mut up = Path::new(trimmed).parent()?.to_string_lossy().into_owned();
        if !up.ends_with(std::path::is_separator) {
            up.push(std::path::MAIN_SEPARATOR);
        }
        Some(up)
    }

    /// Keys in the Treespec prompt: `/` or `\` complete and go down one
    /// level, Tab/Shift+Tab complete and then switch siblings, Esc goes back.
    pub(crate) fn treespec_key(&mut self, key: KeyEvent, origin: NodeId) {
        let Some(prompt) = &mut self.prompt else {
            return;
        };
        let text = prompt.input.text.clone();
        let before = text.clone();
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        // Ctrl+← takes a folder off the path, Ctrl+→ brings it back; any
        // other change starts a new path, with nothing to bring back.
        match key.code {
            KeyCode::Left if ctrl => {
                if let Some(up) = Self::treespec_up(&text) {
                    self.treespec_back.push(text);
                    prompt.input.set_text(up.clone());
                    self.treespec_follow(&up);
                }
                return;
            }
            KeyCode::Right if ctrl => {
                if let Some(back) = self.treespec_back.pop() {
                    prompt.input.set_text(back.clone());
                    self.treespec_follow(&back);
                }
                return;
            }
            _ => {}
        }
        let new_text = match key.code {
            KeyCode::Char('/' | '\\') => {
                let mut t = self.treespec_complete(&text).unwrap_or(text.clone());
                if !t.ends_with(std::path::is_separator) {
                    t.push(std::path::MAIN_SEPARATOR);
                }
                Some(t)
            }
            KeyCode::Tab | KeyCode::BackTab => {
                let forward = key.code == KeyCode::Tab;
                match self.treespec_complete(&text) {
                    Some(t) if t != text => Some(t),
                    _ => self.treespec_sibling(&text, forward),
                }
            }
            _ => None,
        };
        let Some(prompt) = &mut self.prompt else {
            return;
        };
        let text = match new_text {
            Some(t) => {
                prompt.input = LineInput::new(&t).with_history(prompt.input.history());
                t
            }
            None => match prompt.input.handle_key(key) {
                LineEvent::Submit => {
                    let prompt = self.prompt.take().unwrap();
                    self.submit(prompt.kind, prompt.input.text);
                    return;
                }
                LineEvent::Cancel => {
                    self.prompt = None;
                    self.show_node(origin);
                    return;
                }
                LineEvent::Editing => prompt.input.text.clone(),
            },
        };
        if text != before {
            self.treespec_back.clear();
        }
        self.treespec_follow(&text);
    }

    /// Puts the tree cursor on what the Treespec says so far, without waiting
    /// for a slow directory: [`App::apply_reads`] calls it again when it arrives.
    pub(crate) fn treespec_follow(&mut self, text: &str) {
        if let Some(id) = self.treespec_target(text, false) {
            self.show_node(id);
        }
    }

    /// Shows a text in the viewer (Info, Avail, statistics).
    pub(crate) fn show_text(&mut self, title: &str, text: String) {
        let data = text.into_bytes();
        self.tagged_view = None;
        self.viewer = Some(Viewer::from_bytes(title.into(), data.len() as u64, data));
    }

    /// Alt+I: the file under the cursor or, in the tree, the directory (with what
    /// is logged in it and in its branch).
    pub(crate) fn info_text(&self) -> String {
        let in_files = self.focus == Focus::Files;
        if let Some(remote) = &self.remote {
            let path = match self.selected_file().filter(|_| in_files) {
                Some(f) => self.file_path(f),
                None => self.current_path(),
            };
            return remote_info(remote, &path);
        }
        match self.selected_file().filter(|_| in_files) {
            Some(f) if self.archive.is_none() => info::path_info(&self.file_path(f)),
            Some(f) => {
                let e = self.file(f);
                format!(
                    "INFO  {} (inside an archive)\n\n  Size          {} bytes\n  Permissions   {:04o}\n",
                    self.file_path(f).display(),
                    info::group(e.size),
                    e.mode
                )
            }
            None => {
                let id = self.current();
                let mut text = if self.archive.is_some() {
                    format!(
                        "INFO  {} (inside an archive)\n",
                        self.current_path().display()
                    )
                } else {
                    info::path_info(&self.current_path())
                };
                let dir = self.tree.dir_totals(false, false)[id];
                let branch = self.tree.dir_totals(true, false)[id];
                let subdirs = self.tree.visible_subtree(id).len() - 1;
                text.push_str(&format!(
                    "\nLOGGED\n  {:<14}{} files, {} bytes\n  {:<14}{} files, {} bytes, {} subdirectories\n",
                    "Directory",
                    info::group(dir.0),
                    info::group(dir.1),
                    "Branch",
                    info::group(branch.0),
                    info::group(branch.1),
                    info::group(subdirs as u64)
                ));
                text
            }
        }
    }

    /// The DISK statistics for `path`: on this machine only what is on its
    /// disk, with that disk's mount point; in an archive or on a remote
    /// machine, everything logged.
    pub(crate) fn disk_stats_at(&self, path: &Path) -> (crate::pane::DiskStats, Option<PathBuf>) {
        let point = match (&self.remote, &self.archive) {
            (None, None) => info::mount_of(path).map(|m| m.point),
            _ => None,
        };
        let Some(point) = point else {
            return (self.disk_stats(None), None);
        };
        let points: std::collections::HashSet<PathBuf> = crate::platform::mounts()
            .into_iter()
            .map(|m| m.point)
            .collect();
        (self.disk_stats(Some((&point, &points))), Some(point))
    }

    /// ?: statistics of the active window.
    pub(crate) fn stats_text(&self) -> String {
        let g = info::group;
        let logged = self.tree.logged_totals();
        let (disk, point) = self.disk_stats_at(&self.current_path());
        let listed: u64 = self.files.iter().map(|&f| self.file(f).size).sum();
        let (tagged, tagged_bytes) = self
            .files
            .iter()
            .map(|&f| self.file(f))
            .filter(|f| f.tagged)
            .fold((0u64, 0u64), |(n, b), f| (n + 1, b + f.size));
        let view = match self.view {
            FileView::Dir => "directory",
            FileView::Branch(_) => "branch",
            FileView::Showall => "showall",
        };
        let mut text = format!("STATISTICS  {}\n\n", self.tree.path(ROOT).display());
        let mut row = |k: &str, v: String| text.push_str(&format!("  {k:<14}{v}\n"));
        let everything = format!(
            "{} dirs, {} files, {} bytes",
            g(logged.dirs),
            g(logged.files),
            g(logged.bytes)
        );
        match point {
            // As the DISK panel counts: only this disk, each hard link once;
            // then everything logged, other file systems mounted inside and
            // every name of a hard link included.
            Some(point)
                if (disk.logged.files, disk.logged.bytes) != (logged.files, logged.bytes) =>
            {
                row(
                    "Logged",
                    format!(
                        "{} files, {} bytes on {}",
                        g(disk.logged.files),
                        g(disk.logged.bytes),
                        point.display()
                    ),
                );
                row("", format!("all logged: {everything}"));
            }
            _ => row("Logged", everything),
        }
        row("Filespec", self.filespec.text().to_string());
        row("Sort", self.sort.label());
        row(
            "Listing",
            format!(
                "{view}: {} files, {} bytes",
                g(self.files.len() as u64),
                g(listed)
            ),
        );
        row(
            "Tagged",
            format!("{} files, {} bytes", g(tagged), g(tagged_bytes)),
        );
        text.push_str(&info::disk(&self.current_path()));
        text.push('\n');
        text.push_str(&info::system());
        text
    }

    /// Writes the Batch file: one line per tagged file. A script (`.sh`, or
    /// `.bat`/`.cmd` on Windows) gets its header and execute permission.
    pub(crate) fn write_batch(&mut self, file: &Path, mask: &str) -> Result<(), String> {
        if mask.trim().is_empty() {
            return Err("Batch cancelled: empty line mask".into());
        }
        let sources = self.sources(true);
        let header = shell::batch_header(file);
        let script = header.is_some();
        let mut data = header.unwrap_or_default().to_vec();
        for s in &sources {
            data.extend(shell::batch_line(mask, &s.path, s.size));
            data.extend_from_slice(shell::NEWLINE);
        }
        let mut out = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(file)
            .map_err(|e| format!("Cannot create {}: {e}", file.display()))?;
        std::io::Write::write_all(&mut out, &data).map_err(|e| e.to_string())?;
        if script {
            let _ = platform::set_file_mode(&out, 0o755);
        }
        if let Some(dir) = file.parent() {
            self.rescan_everywhere(&[dir]);
        }
        self.message = Some(format!(
            "Wrote {} lines to {}",
            sources.len(),
            file.display()
        ));
        Ok(())
    }

    /// Applies the typed date. Without a time, each file keeps its own.
    pub(crate) fn new_date(&mut self, paths: &[PathBuf], text: &str) -> Result<(), String> {
        let (date, time) = filespec::parse_date_time(text, self.date_order())
            .ok_or_else(|| format!("Invalid date: {text}"))?;
        let mut errors = Vec::new();
        let mut changed = 0;
        let remote = self.remote.clone();
        for path in paths {
            let result = (|| -> Result<(), String> {
                let modified = match &remote {
                    // The symlink check is the remote machine's own, below.
                    Some(r) => r.lstat(path).map_err(|e| e.to_string())?.modified,
                    None => {
                        let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
                        if meta.is_symlink() {
                            return Err(
                                "symbolic link skipped (the date would change its target)".into()
                            );
                        }
                        meta.modified().ok()
                    }
                };
                let old = chrono::DateTime::<chrono::Local>::from(
                    modified.ok_or("the file has no date")?,
                );
                let when = date
                    .and_time(time.unwrap_or(old.time()))
                    .and_local_timezone(chrono::Local)
                    .earliest()
                    .ok_or("time does not exist (daylight saving change)")?;
                match &remote {
                    Some(r) => r.set_modified(path, when.into()),
                    None => platform::set_modified(path, when.into()),
                }
                .map_err(|e| e.to_string())
            })();
            match result {
                Ok(()) => changed += 1,
                Err(e) => errors.push(format!("{}: {e}", path.display())),
            }
        }
        let dirs: Vec<&Path> = paths.iter().filter_map(|p| p.parent()).collect();
        self.rescan_everywhere(&dirs);
        let mut msg = format!("New date set on {changed} of {}", paths.len());
        if let Some(e) = errors.first() {
            msg.push_str(&format!(". Error: {e}"));
        }
        self.message = Some(msg);
        Ok(())
    }

    /// `O`: opens the file (or the tagged files) in the system's default program.
    /// Inside an archive or on a remote machine, copies them first to the
    /// temporary folder (deleted when Xylem exits), in steps like any copy
    /// (Esc stops it), and opens them at the end.
    pub(crate) fn open_with(&mut self, tagged: bool) {
        let Some(sources) = self.sources_or_warn(tagged) else {
            return;
        };
        // A name with hidden bidi controls may not be what it looks like.
        let hidden = sources.iter().find_map(|s| {
            let name = s.path.file_name()?.to_string_lossy().into_owned();
            crate::bidi::has_bidi(&name).then_some(name)
        });
        if let Some(name) = hidden {
            let q = format!(
                "Hidden bidirectional characters in {}. Open anyway?",
                crate::bidi::escape(&name)
            );
            // 'y' opens the files asked about, even if the selection
            // changes before the answer.
            self.ask(ConfirmKind::OpenHidden { sources }, q);
        } else {
            self.open_sources(sources);
        }
    }

    /// The selected file's name has bidi controls (the footer warns).
    pub(crate) fn selected_name_has_bidi(&self) -> bool {
        self.selected_file()
            .is_some_and(|f| crate::bidi::has_bidi(&self.file(f).name.to_string_lossy()))
    }

    pub(crate) fn open_sources(&mut self, sources: Vec<Source>) {
        if self.remote.is_none() && self.archive.is_none() {
            let paths = sources.into_iter().map(|s| s.path).collect();
            return self.open_paths(paths, &[]);
        }
        let prepared = (|| {
            let dir = platform::run_dir().map_err(|e| e.to_string())?;
            let copies: Vec<PathBuf> = sources
                .iter()
                .map(|s| dir.join(s.path.file_name().unwrap_or_default()))
                .collect();
            // A fresh copy, without asking to overwrite the one opened before.
            for copy in &copies {
                let _ = fs::remove_file(copy);
            }
            let op = Operation::transfer(OpKind::Copy, sources, &dir, None)?;
            let op = match (&self.remote, &self.archive) {
                (Some(remote), _) => op.reading_remote(remote.clone())?,
                (None, Some(root)) => op.reading_archive(root, self.archive_entries.clone())?,
                (None, None) => op,
            };
            Ok::<_, String>((op, copies))
        })();
        match prepared {
            Ok((op, copies)) => {
                self.open_after = Some(copies);
                self.start_op(op);
            }
            Err(e) => self.message = Some(format!("Cannot open: {e}")),
        }
    }

    /// Opens `paths` in the default program; `errors` come from before (the
    /// copy that brought them here).
    pub(crate) fn open_paths(&mut self, paths: Vec<PathBuf>, errors: &[String]) {
        let mut errors = errors.to_vec();
        let mut opened = 0;
        for path in paths {
            match shell::open_detached(&path, &self.options.opener) {
                Ok(()) => opened += 1,
                Err(e) => errors.push(format!("{}: {e}", path.display())),
            }
        }
        self.message = Some(match errors.first() {
            None => format!("Opened {opened} file(s)"),
            Some(e) => format!("Opened {opened} file(s). Error: {e}"),
        });
    }

    /// Esc in an archive's tree: goes back to where it was.
    pub(crate) fn close_archive(&mut self) {
        if let Some(outer) = self.outer.take() {
            *self.deref_mut() = *outer;
            // The disk may have changed while the archive was open.
            let current = self.current();
            self.tree.rescan(current);
            self.after_change();
        }
    }

    /// Log: opens `path` in the active window. If it is inside the current tree,
    /// just moves the cursor there; otherwise, the window gets `path` as its root.
    pub(crate) fn log(&mut self, text: &str) -> Result<(), String> {
        if let Some(address) = crate::remote::Address::parse(text) {
            return self.connect(address);
        }
        let path = ops::resolve_path(text, &self.current_path());
        let path = dunce::canonicalize(&path).map_err(|e| format!("Cannot log {text}: {e}"))?;
        if !path.is_dir() {
            return Err(format!("{} is not a directory", path.display()));
        }
        if !self.goto(&path) {
            let (sort, display) = (self.sort, self.display);
            *self.deref_mut() = self.new_pane(path);
            self.display = display;
            self.set_sort(sort);
        }
        Ok(())
    }

    /// Jumps directly to a directory path: if it is inside the current tree, moves
    /// the cursor there and expands as needed; otherwise opens it as a new root.
    pub(crate) fn jump_to_path(&mut self, path: &Path) -> Result<(), String> {
        let path = dunce::canonicalize(path)
            .map_err(|e| format!("Cannot jump to {}: {e}", path.display()))?;
        if !path.is_dir() {
            return Err(format!("{} is not a directory", path.display()));
        }
        if !self.goto(&path) {
            let (sort, display) = (self.sort, self.display);
            *self.deref_mut() = self.new_pane(path);
            self.display = display;
            self.set_sort(sort);
        }
        Ok(())
    }

    /// Sets or updates a bookmark for `key` pointing to `path`. Its name
    /// stays if the place is the same, and goes with the old place otherwise.
    pub(crate) fn set_bookmark(&mut self, key: String, path: PathBuf) {
        let label = self
            .bookmarks
            .get(&key)
            .filter(|old| old.path == path)
            .and_then(|old| old.label.clone());
        self.bookmarks.insert(key, Bookmark { path, label });
        self.save_bookmarks();
    }

    fn save_bookmarks(&mut self) {
        if let Some(file) = &self.history_file
            && let Err(e) = state::save_bookmarks(file, &self.bookmarks)
        {
            self.message = Some(format!("Cannot save bookmarks to {}: {e}", file.display()));
        }
    }

    /// Bookmarks the directory under the cursor as `key`: on a remote
    /// machine or in a container, by the address that connects to it.
    pub(crate) fn bookmark_here(&mut self, key: String) {
        let here = self.current_path();
        let place = match &self.remote {
            None => here,
            Some(remote) => match remote.address_of(&here) {
                Some(address) => PathBuf::from(address),
                None => {
                    self.message = Some(format!(
                        "Cannot bookmark {}: an address cannot hold its path",
                        here.display()
                    ));
                    return;
                }
            },
        };
        // A failure to save, if any, takes the place of this message.
        self.message = Some(format!("Bookmark '{key}' set to {}", place.display()));
        self.set_bookmark(key, place);
    }

    /// Gives the bookmark `key` a name, or takes it away (empty).
    pub(crate) fn name_bookmark(&mut self, key: &str, text: &str) -> Result<(), String> {
        let label = state::check_label(text)?;
        let Some(bm) = self.bookmarks.get_mut(key) else {
            return Err(format!("Bookmark '{key}' not found"));
        };
        bm.label = label;
        self.save_bookmarks();
        Ok(())
    }

    /// Goes to a bookmark: a folder here, as Log does, or a remote one, by
    /// connecting to it. A remote machine this window is already on only
    /// moves the cursor, when the folder is in its tree.
    pub(crate) fn jump_to_bookmark(&mut self, bm: &Bookmark) -> Result<(), String> {
        let Some(text) = bm.address() else {
            return self.jump_to_path(&bm.path);
        };
        let Some(address) = crate::remote::Address::parse(text) else {
            return Err(format!(
                "Not an address Xylem can open: {}",
                crate::bidi::escape(text)
            ));
        };
        if let Some(remote) = &self.remote
            && remote.address.same_machine(&address)
            && let Some(path) = remote.virtual_path(&address.path)
            && self.goto(&path)
        {
            return Ok(());
        }
        self.connect(address)
    }

    /// The keys of the local bookmarks whose folder is gone. Remote ones and
    /// those on a slow mount (a cloud, a network share) are not checked:
    /// that would wait for the network.
    pub(crate) fn missing_bookmarks(&self) -> std::collections::BTreeSet<String> {
        let slow = |p: &Path| crate::info::mount_of(p).is_some_and(|m| m.reads_slowly());
        self.bookmarks
            .iter()
            .filter(|(_, bm)| bm.address().is_none() && !slow(&bm.path) && !bm.path.is_dir())
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Removes a bookmark by `key`. Returns true if it existed.
    pub(crate) fn remove_bookmark(&mut self, key: &str) -> bool {
        let removed = self.bookmarks.remove(key).is_some();
        if removed {
            self.save_bookmarks();
            if let Some(view) = &mut self.bookmarks_view {
                view.missing.remove(key);
            }
        }
        removed
    }

    /// Opens the prompt to jump to a bookmark (').
    pub(crate) fn start_bookmark_jump(&mut self) {
        if self.bookmarks.is_empty() {
            self.start_bookmark_set();
            return;
        }
        self.open_prompt(
            PromptKind::BookmarkJump,
            "JUMP (0-9, a-z; ' list; + bookmark here; Esc cancel): ".into(),
            "",
        );
    }

    /// Opens the prompt to bookmark the current directory (Ctrl+B / Ctrl+F11).
    pub(crate) fn start_bookmark_set(&mut self) {
        self.open_prompt(
            PromptKind::BookmarkSet,
            "SET BOOKMARK for current directory (press 0-9 or a-z, Esc to cancel): ".into(),
            "",
        );
    }

    /// Opens the bookmarks manager screen (Alt+B / F11).
    pub(crate) fn open_bookmarks(&mut self) {
        self.bookmarks_view = Some(BookmarksView {
            missing: self.missing_bookmarks(),
            ..BookmarksView::default()
        });
    }

    /// F11 in the Log, Copy or Move prompt: the bookmarks, to choose a place.
    pub(crate) fn pick_bookmark(&mut self) {
        self.bookmarks_view = Some(BookmarksView {
            pick: true,
            missing: self.missing_bookmarks(),
            ..BookmarksView::default()
        });
    }

    /// A bookmark chosen from a prompt: its place, in the prompt, to be
    /// confirmed there.
    fn put_bookmark_in_prompt(&mut self, bm: &Bookmark) {
        self.bookmarks_view = None;
        if let Some(prompt) = &mut self.prompt {
            let place = match bm.address() {
                Some(address) => address.to_string(),
                None => bm.path.to_string_lossy().into_owned(),
            };
            prompt.input.set_text(place);
        }
    }

    /// F2 in the manager: the name of the bookmark under the cursor, typed
    /// over the list.
    fn start_bookmark_name(&mut self) {
        let Some(view) = &self.bookmarks_view else {
            return;
        };
        let Some((key, bm)) = self.bookmarks.iter().nth(view.selected) else {
            return;
        };
        let key = key.clone();
        let current = bm.label.clone().unwrap_or_default();
        let label = format!("NAME of bookmark '{key}' (empty: no name): ");
        self.open_prompt(PromptKind::BookmarkName(key), label, &current);
    }

    /// Handles keyboard events when the bookmarks manager (Alt+B / F11) is open.
    pub(crate) fn bookmarks_key(&mut self, key: KeyEvent) {
        let Some(view) = &self.bookmarks_view else {
            return;
        };
        let (selected, pick) = (view.selected, view.pick);
        if key.modifiers.contains(KeyModifiers::ALT) && matches!(key.code, KeyCode::Char('b' | 'B'))
        {
            self.bookmarks_view = None;
            return;
        }
        let count = self.bookmarks.len();
        let chosen = self.bookmarks.values().nth(selected).cloned();
        let move_to = |app: &mut Self, to: usize| {
            if let Some(view) = &mut app.bookmarks_view {
                view.selected = to;
            }
        };
        match key.code {
            KeyCode::Esc | KeyCode::F(11) => self.bookmarks_view = None,
            KeyCode::Up | KeyCode::Char('k') => move_to(self, selected.saturating_sub(1)),
            KeyCode::Down | KeyCode::Char('j') => {
                move_to(self, (selected + 1).min(count.saturating_sub(1)))
            }
            KeyCode::Home => move_to(self, 0),
            KeyCode::End => move_to(self, count.saturating_sub(1)),
            KeyCode::Enter => {
                if let Some(bm) = chosen {
                    self.choose_bookmark(&bm, pick);
                }
            }
            // From a prompt, the list only chooses: nothing is changed here.
            _ if pick => {
                if let KeyCode::Char(c) = key.code
                    && let Some(bm) = self.bookmarks.get(&c.to_ascii_lowercase().to_string())
                {
                    let bm = bm.clone();
                    self.choose_bookmark(&bm, true);
                }
            }
            KeyCode::F(2) => self.start_bookmark_name(),
            KeyCode::Delete => self.remove_selected_bookmark(),
            KeyCode::Insert | KeyCode::Char('+') => {
                self.bookmarks_view = None;
                self.start_bookmark_set();
            }
            KeyCode::Char(c) if c.is_ascii_alphanumeric() => {
                let key_str = c.to_ascii_lowercase().to_string();
                if let Some(bm) = self.bookmarks.get(&key_str).cloned() {
                    self.choose_bookmark(&bm, false);
                } else if c == 'd' || c == 'D' {
                    self.remove_selected_bookmark();
                } else if c == 'a' || c == 'A' || c == 'm' || c == 'M' {
                    self.bookmarks_view = None;
                    self.start_bookmark_set();
                }
            }
            _ => {}
        }
    }

    /// Enter or a bookmark's key in the list: into the prompt it was opened
    /// from (`pick`), or there.
    fn choose_bookmark(&mut self, bm: &Bookmark, pick: bool) {
        if pick {
            self.put_bookmark_in_prompt(bm);
            return;
        }
        self.bookmarks_view = None;
        if let Err(e) = self.jump_to_bookmark(bm) {
            self.message = Some(e);
        }
    }

    fn remove_selected_bookmark(&mut self) {
        let Some(view) = &self.bookmarks_view else {
            return;
        };
        let Some(k) = self.bookmarks.keys().nth(view.selected).cloned() else {
            return;
        };
        self.remove_bookmark(&k);
        let count = self.bookmarks.len();
        if let Some(view) = &mut self.bookmarks_view {
            view.selected = view.selected.min(count.saturating_sub(1));
        }
        self.message = Some(format!("Bookmark '{k}' removed"));
    }

    /// `sources`, warning when there is nothing to operate on.
    pub(crate) fn sources_or_warn(&mut self, tagged: bool) -> Option<Vec<Source>> {
        let sources = self.sources(tagged);
        if sources.is_empty() {
            if tagged {
                self.message = Some("No tagged files".into());
            }
            return None;
        }
        Some(sources)
    }

    pub(crate) fn rename_tagged(
        &mut self,
        sources: Vec<Source>,
        case_mode: CaseMode,
        pattern: &str,
    ) {
        let pattern = pattern.trim();
        if pattern.is_empty() {
            return;
        }
        let mask = RenameMask::new(pattern, case_mode);
        let mut pairs = Vec::new();
        let mut seen_dsts = std::collections::HashSet::new();

        for (i, s) in sources.iter().enumerate() {
            let index = i + 1;
            let Some(file_name) = s.path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let new_name = mask.apply(file_name, index);
            if let Err(e) = ops::check_name(&new_name) {
                self.message = Some(format!("Invalid name '{new_name}': {e}"));
                return;
            }
            let dst = s.path.with_file_name(&new_name);
            if dst != s.path {
                if !seen_dsts.insert(dst.clone()) {
                    self.message = Some(format!("Duplicate target name generated: {new_name}"));
                    return;
                }
                pairs.push((s.path.clone(), dst, s.size));
            }
        }

        if pairs.is_empty() {
            self.message = Some("No files renamed (names unchanged)".into());
            return;
        }

        match self.on_this_side(Operation::rename_batch(pairs)) {
            Ok(op) => self.start_op(op),
            Err(e) => self.message = Some(e),
        }
    }

    pub(crate) fn rename_file(&mut self, path: &Path, name: &str) -> Result<(), String> {
        ops::check_name(name)?;
        let new = path.with_file_name(name);
        if new == path {
            return Ok(());
        }
        let side = Side::of(self.remote.as_ref());
        if side.exists(&new) {
            return Err(format!("{name} already exists"));
        }
        side.rename(path, &new, false)
            .map_err(|e| format!("Cannot rename: {e}"))?;
        if let Some(dir) = path.parent() {
            self.rescan_everywhere(&[dir]);
        }
        // Keep the cursor on the renamed file.
        if let Some(i) = self.files.iter().position(|&f| self.file_path(f) == new) {
            self.file_state.select(Some(i));
        }
        Ok(())
    }

    pub(crate) fn rename_dir(&mut self, id: NodeId, name: &str) -> Result<(), String> {
        ops::check_name(name)?;
        let path = self.tree.path(id);
        let new = path.with_file_name(name);
        if new == path {
            return Ok(());
        }
        let side = Side::of(self.remote.as_ref());
        if side.exists(&new) {
            return Err(format!("{name} already exists"));
        }
        side.rename(&path, &new, false)
            .map_err(|e| format!("Cannot rename: {e}"))?;
        let parent = self.tree.node(id).parent.unwrap_or(ROOT);
        self.tree.rescan(parent);
        let keep = self.tree.find(&new).unwrap_or(parent);
        self.tree.scan(keep);
        self.refresh_rows(keep);
        self.refresh_files();
        if let Some(dir) = path.parent() {
            self.rescan_everywhere(&[dir]);
        }
        Ok(())
    }

    pub(crate) fn make_dir(&mut self, id: NodeId, name: &str) -> Result<(), String> {
        ops::check_name(name)?;
        let path = self.tree.path(id).join(name);
        Side::of(self.remote.as_ref())
            .create_dir(&path)
            .map_err(|e| format!("Cannot create directory: {e}"))?;
        self.tree.rescan(id);
        self.tree.expand(id);
        self.refresh_rows(id);
        self.rescan_everywhere(&[&path]);
        self.message = Some(format!("Created {}", path.display()));
        Ok(())
    }

    /// Permissions (Linux) or attributes (Windows, where `+R -A` changes
    /// each file's own attributes).
    pub(crate) fn chmod(&mut self, paths: &[PathBuf], text: &str) -> Result<(), String> {
        if let Some(remote) = self.remote.clone() {
            return self.chmod_remote(&remote, paths, text);
        }
        platform::parse_mode(text, 0)?;
        let mut errors = Vec::new();
        let mut changed = 0;
        for path in paths {
            let meta = match fs::symlink_metadata(path) {
                // Symbolic links have no permissions of their own on Linux.
                Ok(m) if m.is_symlink() => continue,
                Ok(m) => m,
                Err(e) => {
                    errors.push(format!("{}: {e}", path.display()));
                    continue;
                }
            };
            let mode = platform::parse_mode(text, platform::mode(&meta))?;
            match platform::set_mode(path, mode) {
                Ok(()) => changed += 1,
                Err(e) => errors.push(format!("{}: {e}", path.display())),
            }
        }
        let mut dirs: Vec<&Path> = paths.iter().filter_map(|p| p.parent()).collect();
        dirs.dedup();
        self.rescan_everywhere(&dirs);
        let what = platform::describe_mode(0).0;
        let mut msg = format!("{what} set to {} on {changed} files", text.trim());
        if let Some(e) = errors.first() {
            msg.push_str(&format!(". {} errors, first: {e}", errors.len()));
        }
        self.message = Some(msg);
        Ok(())
    }

    /// Attributes on a remote machine: octal, as Unix takes them, whatever
    /// this machine is. Symlinks are left out, as here.
    fn chmod_remote(
        &mut self,
        remote: &crate::remote::Remote,
        paths: &[PathBuf],
        text: &str,
    ) -> Result<(), String> {
        let mode = platform::parse_octal_mode(text)?;
        let mut errors = Vec::new();
        let mut changed = 0;
        for path in paths {
            match remote.set_mode(path, mode) {
                Ok(()) => changed += 1,
                Err(e) => errors.push(format!("{}: {e}", path.display())),
            }
        }
        let mut dirs: Vec<&Path> = paths.iter().filter_map(|p| p.parent()).collect();
        dirs.dedup();
        self.rescan_everywhere(&dirs);
        let mut msg = format!("Permissions set to {} on {changed} files", text.trim());
        if let Some(e) = errors.first() {
            msg.push_str(&format!(". {} errors, first: {e}", errors.len()));
        }
        self.message = Some(msg);
        Ok(())
    }

    /// Copies path(s) to the system clipboard (OSC 52 and native clipboard).
    pub(crate) fn clip(&mut self, tagged: bool) {
        if tagged {
            let sources = self.sources(true);
            if sources.is_empty() {
                self.message = Some("No tagged files to copy".into());
                return;
            }
            let text = sources
                .iter()
                .map(|s| s.path.to_string_lossy())
                .collect::<Vec<_>>()
                .join("\n");
            let _ = crate::clipboard::set_clipboard(&text);
            let count = sources.len();
            let plural = if count == 1 { "path" } else { "paths" };
            self.message = Some(format!("Copied {count} {plural} to clipboard"));
        } else {
            let path = if self.focus == Focus::Files
                && let Some(f) = self.selected_file()
            {
                self.file_path(f)
            } else {
                self.current_path()
            };
            let text = path.to_string_lossy().into_owned();
            let _ = crate::clipboard::set_clipboard(&text);
            self.message = Some(format!("Copied to clipboard: {text}"));
        }
    }
}

/// Alt+I on a remote machine: what SFTP tells about `path`.
fn remote_info(remote: &crate::remote::Remote, path: &Path) -> String {
    use crate::model::tree::FileKind;
    let mut text = format!(
        "INFO  {}

",
        path.display()
    );
    match remote.lstat(path) {
        Ok(e) => {
            let kind = match e.kind {
                FileKind::Symlink => "symbolic link",
                FileKind::File => "file",
                FileKind::Other => "directory or special file",
            };
            text.push_str(&format!(
                "  {:<14}{kind}
",
                "Type"
            ));
            if e.kind == FileKind::Symlink
                && let Ok(target) = remote.read_link(path)
            {
                text.push_str(&format!(
                    "  {:<14}{target}
",
                    "Points to"
                ));
            }
            text.push_str(&format!(
                "  {:<14}{} bytes
",
                "Size",
                info::group(e.size)
            ));
            text.push_str(&format!(
                "  {:<14}{:04o}
",
                "Permissions",
                e.mode & 0o7777
            ));
            if let Some(owner) = remote.owner(path) {
                text.push_str(&format!(
                    "  {:<14}{owner} (user:group ids)
",
                    "Owner"
                ));
            }
            if let Some(t) = e.modified {
                let t: chrono::DateTime<chrono::Local> = t.into();
                text.push_str(&format!(
                    "  {:<14}{}
",
                    "Modified",
                    t.format(&crate::config::full_time_pattern())
                ));
            }
        }
        Err(e) => text.push_str(&format!(
            "  {e}
"
        )),
    }
    text.push_str(&format!(
        "
  On {} (SFTP)
",
        remote.address.destination()
    ));
    text
}

/// Last component of a path, for short messages.
pub(crate) fn fit_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}

/// "file.txt" or "3 tagged files".
/// "a.rs", or "3 files".
pub(crate) fn describe_paths(paths: &[PathBuf]) -> String {
    match paths {
        [one] => one
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        many => format!("{} files", many.len()),
    }
}

pub(crate) fn describe(sources: &[Source]) -> String {
    match sources {
        [one] => one
            .path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        many => format!("{} tagged files", many.len()),
    }
}
