# Release notes

The newest version first. Each section lists what is new, what changed for
someone who used the version before, and the keys that moved. The full notes
of each version are on
`https://github.com/andrelnfernandes/xylem/releases`.

## 0.10.0 (in development)

**New.**

- The Filespec takes regular expressions between slashes, mixed with globs,
  dates and sizes: `/^report-\d{4}/`, `/\.jpe?g$/i`. [Filespec](filespec.md)
- The batch Rename (Ctrl+R) takes regular expressions with the `r` flag:
  `s/(\d+)-(\w+)/$2-$1/r`, with `${name}` groups and `\U`, `\L`, `\u`, `\l`
  for case. Without `r`, `s/from/to/` stays literal. Two files that would end
  with the same name are named in the message. [Rename](rename.md)
- Ctrl+S looks for each term in UTF-8, UTF-16 and Windows-1252 at once (F4:
  text+Unicode, text, hex), and takes `AND`, `OR`, `NOT`, parentheses, quotes
  and `x'4D5A'`, as W's `contains()`: `invoice AND NOT draft`.
  [Search](search.md)

**Fixed.**

- Showall (and Branch) of a tree with millions of files froze the screen for
  seconds while it sorted the list: about 5 s with 2,000,000 files. It now
  takes well under a second (0.66 s on the same machine), in every sort order,
  and the order of the files is the same as before.

## 0.9.0

**New.**

- Norton Guides (`.ng`), the help files of DOS and Clipper, open in the
  viewer as a document: the menus, then each entry numbered (`§12`), with
  its links and its see-also pointing to those numbers. Tab chooses a link,
  Enter or a click follows it, Backspace goes back. [The viewer](viewer.md)
- Fuzzy find (`|`): type pieces of a name (`cfgxy` finds `config/xylem`)
  and the cursor goes to the best folder, in the tree, or file of the list,
  in the file window. The results show as you type, the best first, even in
  a tree of millions of folders. Ctrl+T tags the chosen result and Alt+T the
  list. [Fuzzy find](fuzzy.md)
- [Finding files and folders](finding.md): a page that says which search
  answers which question.
- Ebooks: EPUB and FictionBook (`.fb2`, `.fb2.zip`) open as a book to
  read, in a column with the paragraphs of a printed book: a title page,
  Space for the next page, `]` `[` for chapters, Tab for the contents, Enter
  for the notes called on the page, `g 40%` for a part of the book, and how
  long is left of the chapter and of the book. `I` shows the report on the
  file: what the book says of itself, and whether it carries scripts, things
  fetched from outside or DRM. [The viewer](viewer.md#ebooks)
- `R` in a book remembers where you stopped, and the book opens there next
  time. If you turn it on (page 5 of F12), Xylem also keeps the time you
  spent reading each book remembered, your sessions and your reading speed,
  only on your computer. [Reading time](configuration.md#reading-time)
- F1 opens the help at the section of what you are doing: in the Batch of
  the viewer (Ctrl+B), at "Batch in the viewer", not at the top of the page.
  Links to a section of a page of the help now lead to it.
- Xylem can tell that a new version is out. The first time it starts, it
  asks whether it may look once a day; with a yes, it fetches only the
  number of the latest version from the website, and the top line says when
  a newer one is out. Nothing is downloaded or installed. [New
  versions](configuration.md#new-versions)
- Views (Alt+V): other ways to look at the list. The Scan (`S`) points
  at what in a folder deserves a look before you open it: names
  with hidden characters or a double extension (`fatura.pdf.exe`), setuid
  files and what anyone can write, links that leave the folder or point to
  nothing, and, read with the viewer's own readers and limits, executables
  without PIE or NX or packed, Office documents with macros or a remote
  template, patches that climb out with `..`. Enter on a category brings
  its files back to the list, for Ctrl+T and the rest. The Timeline (`T`)
  shows the files of the list in a histogram by date, by hours, days, weeks,
  months or years; Enter on a bar, or a range, keeps only its files, as a
  `W` condition. The Table (`L`) opens the list in the table viewer, a file
  per row with the columns of `W`, even a list of millions of files; `s`
  sorts it by a column, and `W` there takes its SQL filter back to the list. [Views](views.md)

**Changed.**

- `O` in the viewer goes through two more encodings, Windows-1251 and
  KOI8-R (Cyrillic), before UTF-16.
- The Type display (Alt+F) says `EPUB ebook (ZIP)`, `FB2 ebook` and
  `FB2 ebook (ZIP)`.

**Fixed.**

- With Git on, a big repository on a slow file system (`/mnt/c` from WSL,
  a network share) froze the screen while `git status` ran, for minutes.
  The status is now read in the background, and a slow repository is read
  again less often. [Git](git.md)
- The library Xylem uses for system calls (rustix 1.1.6) is updated for
  an advisory about extended attributes
  ([RUSTSEC-2026-0486](https://rustsec.org/advisories/RUSTSEC-2026-0486)).

## 0.8.0

**Changed: some Alt keys.** Alt no longer repeats Ctrl. It is the other
version of a command.

| Key | Up to 0.7 | From 0.8 |
|---|---|---|
| Alt+A | Attributes of the tagged files | nothing (Ctrl+A) |
| Alt+D | delete the tagged files | Duplicates (Ctrl+D deletes them) |
| Alt+R | rename the tagged files | Release the branch, as `-` (Ctrl+R renames them) |
| Alt+T | tag everything | nothing (Ctrl+T) |
| Alt+U | untag everything | Usage (Ctrl+U untags) |
| Alt+W | Usage | nothing (Alt+U) |
| Alt+Y | Duplicates | nothing (Alt+D) |

To keep the old keys, add them to `config.toml`, one or all of them:

```toml
[keys.any]
"alt+a" = "attributes_tagged"
"alt+d" = "delete_tagged"
"alt+r" = "rename_tagged"
"alt+t" = "tag_all"
"alt+u" = "untag_all"
"alt+w" = "usage"
"alt+y" = "duplicates"
```

**Changed: the footer.** Every command is in a menu, and the footer has
three lines, or two on a screen wide enough to fit each menu on one line. On
a narrow screen the words are shortened before any entry is dropped. Inside
an archive and on a remote machine the menus list only what runs there.
[The menus](menus.md)

**Changed: prompts.** ↑ opens the history of a prompt as a list; ↓ no longer
steps through the entries. `B` in a Branch or Showall goes back to the
directory's own list. The small file window takes a quarter of the screen's
height.

**New.**

- `W` keeps in the list the files that meet a SQL condition, Ctrl+W among
  the tagged files. `contains('text')` reads the files, and `words('text')`
  uses an index built with Alt+N. [A SQL condition](sql.md#a-sql-condition-w)
- `P` opens a search box that lists files by the words typed.
  [The search box](sql.md#the-search-box-p)
- CSV, TSV, JSON Lines, DBF and SQLite tables have no limit of rows in the
  viewer. Spreadsheets keep the limit of 50,000 rows. [Tables](tables.md)
- In a table, Enter shows the record as a form, and `c` copies it; `c` also
  copies the structure shown by `i`. [Tables](tables.md)
- A text over 16 MiB is read through a window, with no size limit.
  [Big files](viewer.md#big-files)
- New viewers: JSON, YAML and TOML as a tree that folds; logs with the colors
  of their levels, and follow; patches with an index of files and hunks and
  what to check; Jupyter notebooks; Office documents, with their macros,
  remote templates and hidden text; executables (ELF, PE, Mach-O) and
  certificates and keys as reports. [The viewer](viewer.md)
- It builds on macOS (not tested on a Mac yet).
- Alt+L makes the directory under the cursor the root of the tree.
- In Treespec, Ctrl+← takes the last folder off the path and Ctrl+→ brings
  it back.
- `clock_seconds` shows seconds in the header's clock.
- The mouse, off by default (F12, page 2, R): the wheel, clicks and double
  clicks on the tree, the lists and the footer's keys, tagging with the middle
  button, a menu on the right button (also the Menu key and Shift+F10), and
  the mouse passed to the programs of the built-in terminal and the framed
  editor.
  [The mouse](mouse.md)
- This manual: the pages on the screen, the menus, the commands of each
  menu, the vocabulary and the how-to pages.

## 0.7.0

**Changed: logging.** A directory is read only when you ask. Moving the
cursor over one that is not logged shows `Not logged`; `+` or → logs it.
Before, passing over a directory read it. [The tree window](tree.md)

**Changed: prompts.** Log, and Copy and Move without a split screen, open
empty. ↑ lists the places used before.

**New.**

- Remote machines are no longer read-only: Copy to them, Move, Rename,
  Makedir, Delete, Prune, Attributes, New date and Duplicates work there.
  [Remote machines](remote.md)
- `docker://container/path` opens a container through `docker exec`.
- Bookmarks can have names, and a bookmark on a remote machine keeps the
  address that opens it. F11 in the Log, Copy and Move prompts brings a
  bookmark into the prompt. [Bookmarks](bookmarks.md)
- Folders mounted by rclone and network shares are read in the background;
  the tree shows at once. [Cloud storage](cloud.md)
- The Usage view tags, deletes, prunes and views from inside it.
- The start screen, the name and version in the header, file names colored
  by type and rounded borders. [Configuration](configuration.md)

## 0.6.1

Updates the spreadsheet reader (calamine 0.38) for the advisory
RUSTSEC-2026-0436.

## 0.6.0

**New.**

- The viewer shows spreadsheets, CSV, JSON Lines, DBF, SQLite and Parquet as
  tables, with a SQL filter. A SQLite database also takes a whole read-only
  query. [The viewer](viewer.md)
- Ctrl+B in the viewer writes a file from the records of a table or the
  lines of a text: JSON, CSV, XML, HTML, a template, or a SQLite table.
  [Batch](execute.md)
- Duplicates finds equal files and folders, on local disks and through the
  hashes rclone reports. The key was Alt+Y; it is Alt+D from 0.8.
  [Duplicates](duplicates.md)
- Markdown files open rendered in the viewer. `T` shows the source.
- The viewer filters lines with a regular expression (`&`).
  [Filtering lines](filter.md)
- `*` on a folder mounted by rclone asks for the whole branch in one request.
- Looking at a Git repository runs nothing the repository names, unless its
  folder is in `git_trusted`. [Git](git.md)

## 0.5.0

**Changed.** `-` in the tree releases the branch from memory; ← still only
closes it. `E` opens the editor inside Xylem's frame.

**New.**

- The security measures for hostile files: limits on decompression, checks
  on extracted paths, control characters kept from the terminal.
  [Security](security.md)
- The viewer wraps lines (`W`), numbers them (`#`), jumps to a line (`g`),
  shows `.gz`, `.xz`, `.zst` and `.bz2` files decompressed, and recognizes
  CP437, CP850, Windows-1252 and UTF-16.
- The help in six languages; `L` in the help switches.
- Ctrl+S in the tree lists only the tagged files of everything logged.
- Graph in the Git menu.
- Icons with Nerd Fonts, off by default.
- `.deb` and `.rpm` packages.

## 0.4.0

**New.**

- The Usage view: the folders and files of a branch by the space they take.
  The key was Alt+W; it is Alt+U from 0.8. [Disk usage](usage.md)
- Alt+K Percent shows each directory's share of the one above it.
- Remote machines over SSH, read-only in this version.
  [Remote machines](remote.md)
- Google Drive, S3 and other storage through rclone. [Cloud storage](cloud.md)
- The themes slate, graphite, daylight and vivid.
- `[editors]` in `config.toml` chooses the editor by file name, and `E`
  refuses binary files that have none.
- `xy --man` shows the manual page.

## 0.3.0 and before

- 0.3.0: the help on F1 and the man page; 7z, tar.xz, tar.bz2 and tar.zst
  archives; the owner and type display modes (Alt+F); both files shown in a
  copy conflict.
- 0.2.0: the program was renamed from xt to Xylem, and the command to `xy`.
  Delete to the trash and Oops, batch rename with a mask, and the clipboard.
- 0.1.0: the first version, as xt.
