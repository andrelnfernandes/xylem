# Comparisons

## A thank-you

I have used some flavor of XTree since the 1990s, and I have had a paid
ZTreeWin license since 2013. To me, ZTreeWin is still the best text-mode file
manager there is: fast, predictable, entirely driven by the keyboard, and built
around a way of working (log the tree, tag across whole branches, act on the
tags) that no other file manager has matched. Kim Henkel has kept the XTree way
alive on Windows for decades, and Xylem would not exist
without that work.

Xylem is not meant to replace ZTreeWin. It exists because I wanted the same
way of working on Linux, where ZTreeWin does not run natively. Xylem also
builds for Windows, because Rust makes that easy, but that is not a goal of
mine: on Windows, ZTreeWin is well worth its license:
[ztree.com](https://www.ztree.com/), and its users'
[forum](https://groups.io/g/ZTreeWin).

The viewer reads Norton Guides, the help files of DOS and Clipper, thanks to
Dave Pearson, who worked out their format in the 1990s and has kept them
readable ever since:
[davep.org/norton-guides](https://www.davep.org/norton-guides/).

## Other file managers for the terminal

There are many good file managers for the terminal, and most people who try
Xylem already use one. This is how they differ, as fairly as I can put it.
Checked against each project's own documentation in November 2026; these
programs change fast, so if something here is out of date, please open an
issue.

| Program | Layout | Known for |
|---|---|---|
| ranger | three columns (parent, current, preview) | vi keys, previews through scripts, Python to extend it |
| lf | three columns, like ranger | one small binary, fast, shell commands as its extension |
| yazi | three columns | speed (everything asynchronous), image previews, Lua plugins with a package manager, SFTP built in |
| superfile | several panels side by side | a modern look, image previews, themes |
| nnn | one list | tiny and fast, a selection across directories, a disk usage mode, many plugins |
| Midnight Commander (mc) | two panels | the classic: its own viewer and editor, SFTP and archives as folders, the results of Find file in a panel |
| vifm | two panels | vim keys and commands, the results of `:find` as a list to work on, a tree view |
| broot | one tree that fits the screen | fuzzy search over the whole tree, sizes, a staging area to act on files from many folders |
| far2l | two panels | Far Manager for Linux, with its plugins: SFTP, SMB, NFS, WebDAV and S3 (NetRocks), a hex editor, Python scripts |

**What sets Xylem apart.** The tree is read into memory, and one key lists
the files of a whole branch, or of everything read, in one list (Branch,
Showall), narrowed by name, date and size (Filespec) or a SQL condition
(`W`), with the bytes that match and those tagged counted as you go. Tag what
you want there and copy, move, rename or delete it all at once. Several of
the programs above can put files from many folders in one list too (mc's
panelize, vifm's custom views, yazi's search results, broot's staging area),
or keep a selection across folders (nnn, yazi); in Xylem that list is how you
work all the time, not the result of a search. Besides that, nothing else
here has in the same program:

- tables with a SQL filter for spreadsheets, CSV, SQLite (with `JOIN`) and
  Parquet, and their records turned into a script, a page or a database
  (Batch). A yazi plugin built on DuckDB shows a preview and a summary of
  CSV and Parquet files, with no filter, query or export;
- reports on files you don't trust: the macros, remote templates and hidden
  text of Office documents, the protections and signs of packing of
  executables, the names to check in a patch, with nothing in them run;
- duplicate files and folders, even on Google Drive without downloading them;
- a built-in terminal whose tree follows the shell's `cd`;
- files from outside treated as hostile everywhere, with a limit on
  everything read ([Security](security.md)).

**Where the others are ahead.** Image previews (yazi, superfile, ranger). A
plugin system and the plugins already written for it (yazi, nnn, ranger, lf,
far2l): Xylem has none, only eXecute and Batch. vi keys from the start
(ranger, lf, yazi, vifm), where Xylem has its own one-letter commands. Years of
use by many people, and macOS and Windows builds that are tested, where Xylem
is new, untested on a Mac, and experimental on Windows.

## ZTreeWin

This part compares the two command by command, based on the official ZTreeWin
v2.2 manual ([ztwhlp.pdf](https://www.zedtek.com/download/ztwhlp.pdf), from
2011, in particular the "Quick Command Table" and section 3.7 on
configuration). Newer versions of ZTreeWin may have changed details; if
something here is out of date, please open an issue.

### The same way of working

Tree with on-demand log (`+`, `*`), file window, Filespec, Branch, Showall (also
of the tagged files only, Ctrl+S), tags (T, U, I, Ctrl+T/U),
Copy/Move/Delete/Rename with the Ctrl (tagged) and Alt (tagged with paths)
versions, Make dir, Delete dir, Prune, Attributes, View, Edit, eXecute, Compare
(directory and branch), Sort, Log, split window (F8 and Tab), relog (F3), Open
(`O`), automatic refresh, browsing and extraction of archives, history in
prompts, bookmarks, help (F1) and a configuration screen. The keys are
ZTreeWin's whenever possible; the exceptions are under "Different keys", at the
end.

### What Xylem does not have yet

- **Tree:** Global (G, and Ctrl+G for the tagged files), Graft (Alt+G), Mirror (Alt+M),
  partial release (Ctrl+Backspace), Hide (Alt+H), Wash (Alt+W), Volume
  (Ctrl+V), F5/F6 (hide subdirectory levels),
  tag by attribute (Alt+T/U),
  Treespec in edit mode (F5, wildcards) and special folders (F6), type-ahead
  search with prompt (`|`) and active mode, drive switching (`<` and `>`).
- **Files:** Open in a new terminal window (Alt+O), Batch of the current file (Alt+B),
  JFC (J, compare the contents of two files), sHortcut (H), Print (P),
  owner/group (chown), Megaview (Ctrl+F7 in the file window), merge tagged files between
  windows (Ctrl+F6), Partial Untag (Ctrl+F8), Tags-Only outside Showall
  (Ctrl+Enter and Ctrl+F4, Ctrl+B in the tree).
- **General:** macros (F12), applications menu (F9), Quit and shell (Ctrl+Q), saving and
  loading the log (Zlist, Ctrl/Alt+Z), colors by name and attribute
  (ZCOLORS.INI), ZTreeWin command-line options, video
  commands (Alt+F6–F9).

Some of these have a stand-in in another program; see "What Xylem leaves to
other programs", below.

### Where Xylem is simpler

| Feature | In ZTreeWin | In Xylem |
|---|---|---|
| Filespec | name with its own name and extension rules, date, size, attribute, inverted filespec (Ctrl+I), labels | glob for names, date and size as in ZTreeWin (with K/M/G); no attributes, inversion or labels |
| Viewer | Alpha, Dump, Hex, Junk and Wrap modes; character sets; hex editing; bookmarks; auto-scroll | text, hex, tables with a SQL filter and rendered Markdown, with ZTreeWin's search (F, B, /, \\, Space, + and -), soft wrap (W), line numbers (#), goto line (g / :), line filter with regex (&); a file on disk of any size, read as the screen moves; no hex editing, by choice (see "What Xylem leaves to other programs") |
| Ctrl+S | text, hex or Unicode; searches only the ZIP index | text or hex |
| Attributes | Windows attributes (R, A, S, H) | Unix permissions in octal (`644`); no owner/group. On Windows, R, H, S and A, as in ZTreeWin |
| Sort | Name, Ext, Date, Time, Length, Alpha, nuMber, Size, Value, Unsorted; Charset; same/diff in split screen | Name, Ext, Date, Size, ascending/descending, Path first; each window has its own order |
| Compare | options Identical, Unique, Newer, Older, Size, Binary, Case; display filter | tags unique, newer, older, identical, different or unique+newer; no binary comparison |
| Prune | deletes only the subdirectories or the current one too; only empty directories; recycle bin | always deletes the whole directory, permanently; configurable confirmation |
| Log | several logged drives, Global, drive switching | one root per window; `L` opens another directory |
| eXecute | batch parameters `%1`–`%8`, new window (NTW) | `sh` with `%f`, `%t` and `%d`; always in Xylem's terminal |
| History | history window with bookmarks | ↑ at the prompt opens a list (filter by typing, Del forgets), one history per type, saved between runs |
| Configuration | 7 pages, per-element color editor (F4), Alt-Export | 4 pages with the options that make sense on Linux; F4 chooses among 12 ready-made themes; individual colors in `config.toml` |
| Split screen | optional statistics panels per side | no statistics panel |
| Menus | Ctrl/Alt menu when holding the key | same in terminals with the kitty protocol; F9 in any terminal |

ZTreeWin configuration options that were not brought over because they are
Windows-specific: RSH attributes, junctions, OEM/ANSI, EBCDIC, window
size, shortcuts (.lnk).

### Where Xylem goes further

Things the ZTreeWin v2.2 manual has no counterpart for:

- **A built-in terminal** (Alt+X): the shell keeps running while you use the
  tree; a `cd` in the shell takes the tree along, and the shell follows the
  tree when you come back to it. Alt+O types the tagged files at the prompt.
  See [the built-in terminal](terminal.md).
- **Remote machines over SSH** (`xy user@host:`): the tree, Branch, the viewer
  and Copy work on another machine, which has its own terminal too. See
  [remote machines](remote.md).
- **Cloud storage:** Google Drive, S3 and the other services rclone supports, mounted and shown as a tree. See [cloud storage](cloud.md).
- **SQL on the file list** (`W`): `ext = 'jpg' AND size > 10000000` keeps the files that meet it, and `contains()` and `words()` look inside them; the search box (`P`) finds files by the words in them. See [finding files](sql.md).
- **Git** (optional): status letters in the file list, a Git menu (Ctrl+G) and
  diffs in the viewer. See [Git](git.md).
- **Disk usage** (Alt+U): a branch's folders and files together, largest
  first, down to what fills the disk; Alt+K shows each folder's share in the
  tree. See [disk usage](usage.md).
- **Duplicates** (Alt+D): files and whole folders with the same contents,
  compared by SHA-256, or by the service's own hashes in a folder mounted by
  rclone, with nothing downloaded. One copy of each group always stays. See
  [duplicates](duplicates.md).
- **A viewer that reads more:** syntax colors, the text encoding found by
  itself (UTF-8, UTF-16, CP850, Windows-1252, CP437), `.gz`, `.xz`, `.zst` and
  `.bz2` files shown decompressed, and hidden text direction characters (Trojan
  Source) shown instead of obeyed. See [the viewer](viewer.md).
- **Tables and Markdown:** spreadsheets, CSV, JSON Lines, DBF, SQLite and
  Parquet open in a grid with a SQL filter, and Markdown files open rendered.
  See [tables](tables.md).
- **Archives without an external archiver:** ZIP, 7z and TAR (plain, gzip, xz,
  bzip2, zstd) open as folders, and ZIP and TAR are created, all by Xylem
  itself. See [archives](archives.md).
- **Batch rename** with search and replace (`s/foo/bar/`), counters and case
  changes. See [rename](rename.md).
- **Batch over records:** Ctrl+B in the viewer turns the records of a table or
  the lines of a text into a script, JSON, CSV, XML, HTML, SQL or a SQLite
  database, with template files if you want. See [Batch](execute.md).
- **Oops:** O (or Ctrl+Z) puts back where it was what was last sent to the
  system trash. See [delete](delete.md).
- **The clipboard everywhere:** Y copies the path even over SSH or inside tmux
  (OSC 52).
- **What a file is:** a display mode (Alt+F) that tells, in the spirit of the
  `file` command, what each file contains, and another with owners and
  permissions.
- **Looks:** 12 ready-made themes and optional Nerd Fonts icons. Every key can
  be rebound under `[keys]` in `config.toml`. See
  [configuration](configuration.md).
- **Hostile files:** archives, names and contents are treated as potentially
  hostile, with limits on everything read and decompressed (see the security
  statement in the README).
- **Help in six languages** (F1, then L), the same text as this manual and the
  man page.

### What Xylem leaves to other programs

Some things Xylem does not do on its own on purpose: a dedicated program does
them better, and Xylem brings it in.

- **Hex editing:** ZTreeWin edits bytes in its viewer. Xylem has no hex editor
  of its own: writing and maintaining one would be a lot of work for little
  use, and editing raw bytes by hand is an easy way to corrupt a file. Instead,
  E on a binary file opens `hexedit`, `heh`, `bvi` or `dhex` (whichever is
  installed, or the one you choose) inside Xylem's frame and colors, or a
  graphical one such as ImHex or GHex. See [Edit](execute.md).
- **Editing text:** your editor (`$VISUAL`, `$EDITOR`, or the one set in F12),
  and a different editor per file type if you want (`[editors]`).
- **SSH:** the system's `ssh`, so your keys, agent, `~/.ssh/config` and jump
  hosts work as they do everywhere else.
- **Cloud storage:** Google Drive, S3 and others through `rclone mount`; Xylem
  recognizes the mount and adapts. See [cloud storage](cloud.md).
- **Git:** the `git` program itself.
- **Opening files** (O): the desktop's program for each type (`xdg-open`).
- **Comparing two files' contents** (JFC in ZTreeWin): until it is built in,
  tag the two files and run `diff %t` with eXecute (X), or use the terminal.
- **Printing** (P in ZTreeWin): `lp %t` with eXecute.
- **Other archive formats** (ZTreeWin's ZAAP): your archiver, with eXecute or
  in the terminal.

### Different keys

The keys follow ZTreeWin's. The exceptions:

| Function | ZTreeWin | Xylem |
|---|---|---|
| Help | F1 | F1 or Alt+H (on GNOME Terminal, F1 opens terminal help) |
| Configuration | Alt+F10 | F12 or Alt+F10 (on GNOME, Alt+F10 maximizes the window and never reaches Xylem) |
| Bookmarks | F11 | F11, Alt+B, or single quote `'` (on GNOME Terminal, F11 toggles fullscreen) |
| Move tagged | Ctrl+M | Ctrl+M in terminals with the kitty protocol; in others, `F9` `M` (or Alt+M and F6) |
| F2 | (nothing) | rereads the directory, same as F3 |
| F9 | applications menu (not implemented) | Ctrl/Alt menus |
| F12 | macros (not implemented) | configuration |
| Alt+B | Batch of the current file (not implemented) | bookmarks |
| Alt+H | Hide (not implemented) | help |
| Alt+O | Open in a new terminal window (not implemented) | types the tagged files in the built-in terminal |
| Alt+D | (nothing) | Duplicates |
| Alt+T | tag by attribute (not implemented) | (nothing) |
| Alt+U | untag by attribute (not implemented) | Usage, where the space goes |
| Ctrl+G | Global for the tagged files (not implemented) | the Git menu (optional) |
| Alt+Z | Zlist (not implemented) | the built-in terminal in full screen |
| `/` | same as `?` (statistics) | Treespec, like `\` (`?` still shows the statistics) |
| Ctrl+C, Ctrl+M in the tree | (nothing) | Copy/Move of the current directory's tagged files |
| Copy path to clipboard | Ctrl+Insert | Y or Ctrl+Insert (Ctrl+Y for tagged; works via OSC 52 and system clipboard) |

All keys can be rebound under `[keys]` in `config.toml`.

## Names

Xylem is an independent project, written from scratch. It is not affiliated
with, endorsed by or connected to Zedtek, Inc. (ZTreeWin), to the owners of
XTree, or to the authors of the other programs named on this page, and it
contains no code from any of them. Their names are mentioned only to describe
what inspired Xylem and how it compares. XTree, ZTree and ZTreeWin are
trademarks of their respective owners.
