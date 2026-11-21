# Backlog

What is known to be left to do, kept here so it does not live only on one
machine. The user-facing summary is under "For later" in the
[README](README.md#for-later); detailed plans are in [docs/plans/](docs/plans/)
(in Portuguese). The order of the work and the rhythm of releases are in
[docs/plans/roadmap.md](docs/plans/roadmap.md).

## Bugs

None open.

## Security

Leftovers of the security pass that came from an external audit
([docs/plans/security.md](docs/plans/security.md); PRs #37 to #40 did the
rest).

1. **Copying many files out of a compressed TAR is slow.** Opening one no longer
   holds the screen up ([docs/plans/compressed-tar.md](docs/plans/compressed-tar.md)),
   and Esc stops a copy, but each file copied decompresses the archive from the
   start again. One pass over the archive could serve all the files asked for.
2. **eXecute (`%f`, `%t`) on Windows.** `cmd` expands `%VAR%` even inside
   quotes, so a name with `%PATH%` in it changes. E and O already avoid it
   by passing the file in `XY_FILE`; commands typed with X still put the
   names in the line. Could pass each file in a variable of its own.
3. **ssh password through a file on Windows.** On Unix the askpass answer
   goes through a named pipe (mode 0600); on Windows it is written to a file
   under `%LOCALAPPDATA%` (private to the user) for a moment. A Windows
   named pipe would avoid the disk. Low priority; needs testing on Windows.
4. **What the table readers still cannot limit**
   ([docs/plans/search-and-table-limits.md](docs/plans/search-and-table-limits.md)).
   With V and in a Batch, an `.ods` is built whole by its reader, up to 100 million
   cells (about 3 GiB) that a small file can ask for with `number-rows-repeated`
   (Autoview leaves `.ods` and `.xls` to V, and one such reader runs at a time in the
   viewer). Time and Esc stop the reading of the file but not that last part. A Parquet
   list can repeat one dictionary value many times within a row, which the smaller
   batches do not bound. A Parquet page
   is decompressed whole, within the limit: the screen is free, but Esc waits for it.
   Searching from one tagged file to the next in the viewer still holds the screen.
   Parquet files in LZ4 do not open: the crate's `lz4` feature is off (`lz4_flex` only
   brings the dependency). Turning it on means one more reader of hostile files.
5. **SQLite and Parquet's zstd are C libraries: replace them with Rust as soon
   as a usable option appears.** The author's decision. They run only with V,
   never in Autoview, and that is meant to be temporary. What to watch:
   - zstd: a way for the `parquet` crate to decode through a Rust codec; Xylem
     already has `ruzstd` for `.zst` files. **To investigate, with no date (the
     author asked on 2026-11-14 to be reminded of it):** not waiting for the crate,
     and putting a small crate named `zstd` in its place with `[patch.crates-io]`,
     backed by `ruzstd`. In `parquet` 60.1 the codec is chosen in a fixed
     `create_codec` (`src/compression.rs`), with no hook for another one, and it
     calls only `zstd::bulk::Compressor`, `zstd::bulk::Decompressor`,
     `zstd_safe::get_frame_content_size` and `zstd_safe::compress_bound`; in Xylem
     only `parquet` depends on `zstd`, so `zstd-sys` would leave the tree. Not
     tested. To find out: how much slower `ruzstd` decodes, what to do with
     compression (Xylem only reads Parquet today), the cost of following `parquet`
     at each update, and a fuzz of corrupt Parquet files through it.
   - SQLite: a Rust implementation that reads real SQLite files reliably and is
     mature enough for hostile ones. Limbo is now Turso Database (crate `turso`).
     Checked on 2026-10-30, at 0.9.0-pre: not yet. Replace it when it has reached
     1.0, builds with no C (`default-features = false`, `pure-rust-crypto`, no
     `mimalloc`), and a fuzz of corrupt files no longer makes it panic or hang.
   Once one is replaced, its format can go back to Autoview. Until then, running
   them in a separate, isolated process would contain a bug in them. Also recheck
   the C dependencies whenever these crates are updated: `cargo tree | grep -- -sys`.

Decided against: wiping the password from memory (`zeroize`: ssh and the
pipe hold it too), and checking where remote symlinks point (copying a link
as a link is what `cp -a` and `rsync -l` do).

## Features

- **MOBI ebooks** (asked on 2026-11-21, after EPUB and FB2 in 0.9.0,
  [docs/plans/viewer-ebooks.md](docs/plans/viewer-ebooks.md)): a reader for
  `.mobi` (and `.azw`, `.prc`, the old Mobipocket and Kindle format) by the
  same road as the EPUB, so the book, the report, the notes, `R` and the
  time read come for free. The format is a PalmDB with PalmDOC (LZ77) or
  HUFF/CDIC compression and HTML inside; it is known only from reverse
  engineering, so the reader must treat every offset and length as hostile,
  as the Norton Guides one does. Books with DRM (most bought ones) are only
  said to be, never read. AZW3/KF8 (a newer EPUB-like part inside) to
  decide when planning.
- **Remote machines, phases 3 and 4** (phases 1 and 2 are in 0.4.0, phase 3 in 0.7.0;
  [docs/plans/ssh-write.md](docs/plans/ssh-write.md)):
  3. writing on the remote machine: done (rename, mkdir, copy, move, delete,
     prune, permissions, dates, and Duplicates there). Copying there does not recreate
     symbolic links (OpenSSH's SYMLINK takes its paths in the other order),
     nor copy between two remote machines; downloads still wait for each
     32 KiB block;
  4. finishing: Edit and Open via temporary download and upload, eXecute
     over ssh, Git over ssh, periodic refresh instead of inotify.

  Containers without sshd (`docker://`, through `docker exec` and the
  container's `sftp-server`) are done.
- **Disk usage** ([docs/plans/usage.md](docs/plans/usage.md)): phases 1 to 3 are
  done, the actions of phase 3 in 0.7.0. With no date: grouping by type and
  export.
- **Duplicate files** ([docs/plans/usage.md](docs/plans/usage.md), "Duplicados";
  [the manual](docs/manual/duplicates.md)): done on local folders, whole folders
  included, through rclone by the service's hashes, and over SSH (read through
  SFTP; hard links show as pairs there, since SFTP gives no inode). Still open:
  whole folders in the cloud.
- **Cloud mounts** ([docs/plans/cloud-opening.md](docs/plans/cloud-opening.md);
  [the manual](docs/manual/cloud.md)): the tree reads in the background, and `*`,
  Branch, Usage and Compare ask rclone for the whole branch at once. Automatic Git
  status and inotify stay off there. Still open: the screen waits for the network on
  F3, on going to a typed path and when opening a Prune; the viewer reads a whole file
  on the main thread, and Esc cannot stop it; while an archive on a mount was being
  read in the background, the main thread was also seen waiting for rclone, for a
  reason not found yet.
- **Querying the disk, by name and by content**
  ([docs/plans/query.md](docs/plans/query.md)): stages 1, 2, 4 and 5 are done
  (PRs #118 and #119, 2026-11-15): `W` and `Ctrl+W` keep the files that meet a SQL
  condition, by name, by contents (`contains`, read with no index) or through a
  `tantivy` index per branch, and the search box. Still open: stage 3, the result
  as a table (`SELECT`, `GROUP BY`, `JOIN`), through SQLite.
- **From ZTreeWin, next:** JFC (compare the contents of two files), and saving
  and loading the log (Zlist), so that the tree of a disk or a Drive opens at once.
- Postponed by choice: macros, chown, Graft. macOS: to look at again when there
  is a machine to test on. It builds (#123, checked in CI) and watches the
  directories with kqueue, which reports files created, deleted or renamed in a
  directory but not a change in the contents of a file already there: a file that
  grows keeps its old size on screen until the directory is read again. FSEvents
  would report it, but gives the paths resolved (`/private/tmp` for `/tmp`), which
  `watch.rs` would have to map back.
- Left out by decision: Xylem in its own window, external converters, HTML in the
  viewer, a treemap, and what of ZTreeWin serves parity more than use
  (Wash, Volume, Hide, video commands).
- **The mouse** ([docs/plans/mouse.md](docs/plans/mouse.md)): approved by the
  author on 2026-11-18, after being left out before. Off by default, turned on in
  F12; the wheel, clicks on the tree, the lists and the footer, double click, tagging
  with the middle button, a right-click menu, and the mouse passed to the programs of
  the built-in terminal. Done (PR #124). Not yet: clicks on the viewer's list of
  sheets, its structure and its record (the wheel works there), and a right-click
  menu in Usage and Duplicates.
- **New file viewers** ([docs/plans/viewers.md](docs/plans/viewers.md)):
  six subprojects, security first. Done: subproject 1 (encodings, compressed
  files, the size limit; PRs #44 to #49), and the table mode with SQLite and the
  SQL filter (subprojects 2 and 5; PR #63), documented in
  [the manual](docs/manual/viewer.md). The windowed viewer (3) is started by the
  tables, with no limit of rows in 0.8.0
  ([docs/plans/tables-windowed.md](docs/plans/tables-windowed.md)). Still to do:
  Parquet in windows (0.9). The logs (4) are done with the seven viewers below;
  left of them: End reading a windowed text from its end, the end of a compressed
  log read in the background, and following a remote file. The text
  is in windows too ([docs/plans/text-windowed.md](docs/plans/text-windowed.md)): a
  file on disk over 16 MiB is read as the screen moves, with no size limit (one of
  234 MB takes 15 MB of memory, not 475). Left of it: a Batch over such a text still
  reads it into memory up to the limit; in Ctrl+V, a search that finds nothing in a
  big file does not go on to the next tagged file; End waits for the count instead
  of reading the file from its end (for the logs). Fixed-width and multi-record files (SDF,
  CNAB, SPED) are out, by decision (2026-11-11): they go to a separate product
  ([docs/plans/viewer-positional.md](docs/plans/viewer-positional.md)).
  Seven new viewers are done (PR #122, merged on 2026-11-17, with the fixes of
  its audit; the list is in "Viewers novos" of
  [docs/plans/viewers.md](docs/plans/viewers.md)). What each one leaves for later
  is in the "Implementação" section of its spec:
  logs ([viewer-logs.md](docs/plans/viewer-logs.md), which is subproject 4), JSON,
  YAML and TOML as a tree ([viewer-tree.md](docs/plans/viewer-tree.md)), DOCX,
  ODT and PPTX ([viewer-office.md](docs/plans/viewer-office.md)), executables
  ([viewer-binaries.md](docs/plans/viewer-binaries.md)), certificates and keys
  ([viewer-certs.md](docs/plans/viewer-certs.md)), Jupyter notebooks
  ([viewer-jupyter.md](docs/plans/viewer-jupyter.md)) and patches
  ([viewer-patch.md](docs/plans/viewer-patch.md)).
- **Rendered Markdown in the viewer**
  ([docs/plans/markdown.md](docs/plans/markdown.md)): done, on `.md` and
  `.markdown`. Still open: the help's small parser could move to the same
  parser one day (it also feeds the man page); icons with Nerd Fonts were left
  out on purpose, because they add nothing to the rendering itself.
- **Batch over files and records** ([docs/plans/batch.md](docs/plans/batch.md)):
  complete. Left for when they are missed: Parquet and XLSX as outputs. Records into
  `.bat`/`.cmd` wait for item 2 of Security.

- **Views** (Alt+V, [docs/plans/views.md](docs/plans/views.md)): for 0.9.0.
  The Scan and the Timeline are done (stages 1–3, PR #133). The Table
  (stage 4) waits for the measurement with 2,000,000 nodes the plan asks
  for, on a machine with more memory than the WSL.
- **zoxide** (`Z`): the optional appendix of
  [docs/plans/fuzzy-find.md](docs/plans/fuzzy-find.md), whose stages are done
  (0.9.0).
- **New formats** for the viewer, first `.deb` and `.rpm`
  with their install scripts, then container images, the systemd journal, EVTX,
  Arrow and Avro, pcap and Firebird: proposals for 0.9.0, nothing approved.

## Xylem as a programming cockpit

A draft, with no date yet: [docs/plans/cockpit.md](docs/plans/cockpit.md). First come
searching the project, a list of `file:line` locations, and opening the editor at
the right line.

## Translations behind English

During development only the English manual is updated; the other languages (pt,
es, de, fr, zh) are synced later, in one documentation task with the website
rework. The 0.8.0 release synced them all (#126); since then, in 0.9.0:

- `fuzzy.md` and `finding.md`: new, in English only.
- `viewer.md`: Norton Guides, EPUB and FictionBook.
- `configuration.md`: the settings of the fuzzy find and of the books.
- `git.md`: why Git can be turned off, and the status read in the background.
- `releases.md`: the 0.9.0 notes.
- `views.md`: new, in English only; the indexes have only its link line.
- `commands-alt.md`: the Alt+V line; `configuration.md`: `scan_max_files`.
- `comparison.md`, `commands-keys.md`, `filespec.md`, `glossary.md`,
  `index.md`, `sql.md`, `treespec.md`: the lines about the fuzzy find, Norton
  Guides and the books.

## Tests

- **`term::tests::sync_changes_directory_and_sets_the_tagged_files` failed once
  in about 50 runs** of the whole suite (2026-11-07, home on NFS). `write_script`
  failed and `sync` fell back to the plain ` cd --`. Not reproduced in 60 more
  runs with logging, so the cause is a suspicion: the opener tests remove the
  `run-<pid>` folder that every test in the process shares (`fs::remove_dir(run)`
  in `src/app/tests.rs`), and that may happen between `run_dir()` and the
  `create_dir` of `terminal-<nanos>` in `private_dir`. Two nanosecond names
  alike, or an NFS error of the moment, would give the same symptom. To fix when
  it is caught: log the error of `write_script` in the test.

## To evaluate

- **The memory of a big tree (JSON, YAML, TOML):** the tree stops at 2,000,000
  nodes, which take about 170 MiB; a JSON of 4 MiB such as `[0,0,0,...]` reaches
  that. Nothing breaks, but a lower limit, or nodes made smaller, would weigh
  less (found in the audit of #122).

- **The wheel over less or man in the built-in terminal:** a program on the
  alternate screen that did not ask for the mouse has no scrollback, so the wheel
  does nothing there. xterm's `alternateScroll` sends ↑ and ↓ instead; but some
  programs take ↑ as something else, so it needs a look.

Details from an outside list of suggestions (2026-11-01) that the items above do
not have. Nothing here is decided. The rest of that list was already planned, or
postponed by choice (macros, chown).

- **JFC, side by side:** tag two files, press a key, and see them next to each
  other with the differences highlighted, for `.conf` and `.env` files or versions
  of a script. To decide when JFC gets its spec: this layout, or a plain diff in the
  viewer, as Git's Diff does.
- **Symbolic permissions:** A and Ctrl+A take octal only (`755`). They could also
  take `+x` or `u+w`, which change one bit and keep the rest of each tagged file's
  mode, something octal cannot do across files with different modes.
- **Every drive at a glance:** the statistics panel shows only the disk being
  browsed. A view listing every mounted drive with its size, free space and how
  full it is, like a `df` inside Xylem, would show which one is filling up on a
  machine with many drives, and Enter could log that drive. Suggested in the
  first comment on the r/rust post (2026-11-03), from someone with a home server
  of 8 drives. To decide: a view of its own or a page of `?`, and which mounts to
  leave out (`/proc`, `/sys`, `tmpfs`, snaps).
