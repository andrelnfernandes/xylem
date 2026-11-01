# Backlog

What is known to be left to do, kept here so it does not live only on one
machine. The user-facing summary is under "For later" in the
[README](README.md#for-later); detailed plans are in [docs/plans/](docs/plans/)
(in Portuguese). The order of the work and the rhythm of releases are in
[docs/plans/roadmap.md](docs/plans/roadmap.md).

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
4. **Searching inside an archive reads each entry in one step.** Memory is
   bounded now (streamed), but a huge entry holds the screen up while it is
   searched, as a huge file on disk does. The search could keep the open
   reader between steps.

5. **What the table readers still cannot limit.** The time limit stops them
   between records, and SQLite midway, but a spreadsheet's reader loads the whole
   sheet first: its size is checked before that, by what the ZIP declares, so a
   ZIP that lies about its sizes gets through. A Parquet page is decompressed
   whole too. Esc cannot interrupt a table being read, only the time limit can.
6. **SQLite and Parquet's zstd are C libraries: replace them with Rust as soon
   as a usable option appears.** The author's decision. They run only with V,
   never in Autoview, and that is meant to be temporary. What to watch:
   - zstd: a way for the `parquet` crate to decode through a Rust codec; Xylem
     already has `ruzstd` for `.zst` files.
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

- **Remote machines, phases 3 and 4** (phases 1 and 2 are in 0.4.0):
  3. writing on the remote machine: upload, move, delete, rename, mkdir,
     chmod, dates; needs a filesystem layer under `ops` and the tree scan;
  4. finishing: Edit and Open via temporary download and upload, eXecute
     over ssh, Git over ssh, periodic refresh instead of inotify.
- **Disk usage, phases 3 and 4** ([docs/plans/usage.md](docs/plans/usage.md);
  phases 1 and 2 are in 0.4.0): actions inside the Usage view (tag, delete,
  prune, viewer); with no date, grouping by type and export.
- **Duplicate files** ([docs/plans/usage.md](docs/plans/usage.md), "Duplicados";
  [the manual](docs/manual/duplicates.md)): done on local folders, whole folders
  included, and through rclone by the service's hashes. Still open: over SSH
  (needs writing on the remote machine, phase 3 above, to be of use); whole
  folders in the cloud.
- **Cloud mounts:** `*` asks rclone for the whole branch at once
  ([the manual](docs/manual/cloud.md)). Still open: opening the root of a large
  Drive reads every folder in it before the first draw (over 29 s on a blank
  screen, with a cold cache); Branch, Usage and Compare still read folder by
  folder, and a single slow read holds the screen up.
- **From ZTreeWin, next:** JFC (compare the contents of two files), and saving
  and loading the log (Zlist), so that the tree of a disk or a Drive opens at once.
- Postponed by choice: macros, chown, Graft. macOS: to look at again when there
  is a machine to test on.
- Left out by decision: Xylem in its own window, external converters, HTML in the
  viewer, a treemap, the mouse, and what of ZTreeWin serves parity more than use
  (Wash, Volume, Hide, video commands).
- **New file viewers** ([docs/plans/viewers.md](docs/plans/viewers.md)):
  six subprojects, security first. Done: subproject 1 (encodings, compressed
  files, the size limit; PRs #44 to #49), and the table mode with SQLite and the
  SQL filter (subprojects 2 and 5; PR #63), documented in
  [the manual](docs/manual/viewer.md). Still to do: the windowed viewer (3); logs
  (4), of which the line filter is done; SDF in the table mode, as an extra.
- **Rendered Markdown in the viewer**
  ([docs/plans/markdown.md](docs/plans/markdown.md)): done, on `.md` and
  `.markdown`. Still open: the help's small parser could move to the same
  parser one day (it also feeds the man page); icons with Nerd Fonts were left
  out on purpose, because they add nothing to the rendering itself.
- **Batch over files and records** ([docs/plans/batch.md](docs/plans/batch.md)):
  complete. Left for when they are missed: Parquet and XLSX as outputs. Records into
  `.bat`/`.cmd` wait for item 2 of Security.

## Xylem as a programming cockpit

A draft, with no date yet: [docs/plans/cockpit.md](docs/plans/cockpit.md). First come
searching the project, a list of `file:line` locations, and opening the editor at
the right line.

## To evaluate

Details from an outside list of suggestions (2026-11-01) that the items above do
not have. Nothing here is decided. The rest of that list was already planned, or
postponed by choice (macros, chown).

- **Follow with the line filter:** when logs get follow (subproject 4 of the
  viewers), new lines could go through the filter in use (`&`), showing only the
  ones that match. The list suggested F as the key, but F is Find in the viewer.
- **JFC, side by side:** tag two files, press a key, and see them next to each
  other with the differences highlighted, for `.conf` and `.env` files or versions
  of a script. To decide when JFC gets its spec: this layout, or a plain diff in the
  viewer, as Git's Diff does.
- **Symbolic permissions:** A and Ctrl+A take octal only (`755`). They could also
  take `+x` or `u+w`, which change one bit and keep the rest of each tagged file's
  mode, something octal cannot do across files with different modes.
