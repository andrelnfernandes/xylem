# Release notes

The newest version first. Each section lists what is new, what changed for
someone who used the version before, and the keys that moved. The full notes
of each version are on
`https://github.com/andrelnfernandes/xylem/releases`.

## 0.8.0

**Changed: some Alt keys.** Alt no longer repeats Ctrl. It is the other
version of a command, as in ZTreeWin.

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
  viewer. Spreadsheets keep the limit of 50,000 rows. [The viewer](viewer.md)
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
