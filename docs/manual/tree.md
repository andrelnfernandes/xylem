# The tree window

The tree shows the directories that have been logged (read into memory). The
file window below it lists the files of the directory under the cursor.

## Moving around

| Key | What it does |
|---|---|
| ↑ ↓ PgUp PgDn Home End | move the cursor |
| → or `+` | log the directory, if it is not yet, and expand it: its files show, and its subdirectories are listed, not logged |
| ← | collapse it (or go to the parent) |
| `-` | release the branch from memory: what was read in it is dropped, tags included, and the directory is left not logged |
| Backspace | go to the parent directory |
| Space | down one line |
| Shift+letter, 0–9 | next directory starting with it |
| `\` or `/` | [Treespec](treespec.md): go to a directory by typing its path |
| Enter or Tab | go to the file window (Tab switches windows in [split screen](split.md)) |

In the tree, a `+` on the connector (`├─+`) marks a collapsed directory that has
subdirectories, and a `?` (`├─?`) one not logged yet, whose subdirectories are
not known.

## Logging

A directory is read only when you ask for it. Moving the cursor to one that is
not logged shows `Not logged` in the file window, and → or `+` logs it. At
start, and with `L`, the root is logged and expanded; its subdirectories are
not. Going to a directory by its path ([Treespec](treespec.md)) logs it too.

`*` logs and expands the whole branch under the cursor; during a long log,
progress shows in the footer and `Esc` cancels. **F3** (or F2) rereads the
directory from disk. Xylem also watches the current directory of each window
and rereads it on its own when another program changes it (at most once every
half second, after any operation in progress); other directories still need F3.

`L` ([Log](treespec.md)) opens another directory in the window.

## Lists of files

- `B` (Branch): the files of the whole branch, logging it first.
- `S` (Showall): the files of everything that has been logged.
- Ctrl+S: Showall of the tagged files only (ZTreeWin's Tags-Only mode). Files
  untagged in it stay listed until the list is reread (F3); with no tags left,
  all the files show again.
- `F` ([Filespec](filespec.md)): which files show, by name, date and size.

In Branch and Showall, `\` goes to the directory of the selected file.

## Commands in the tree

| Key | What it does |
|---|---|
| `T` / `U` / `I` | tag / untag / invert the files of the directory ([tags](tagging.md)) |
| `M` | make a subdirectory |
| `R` | rename the directory |
| `D` or Del | delete the directory, if empty ([delete](delete.md)) |
| Alt+P | Prune: delete the directory with everything in it |
| `O` or Ctrl+Z | Oops: restore what went to the trash |
| `N` | change the directory's date ([dates](attributes.md)) |
| `C` / Alt+C | [Compare](compare.md) with the other window / the whole branch |
| `A` | Avail: mounted file systems ([information](info.md)) |
| Alt+I | Info: details of the directory |
| Alt+K | sizes, file counts or percentages next to each directory |
| `Y` | copy the directory's path to the clipboard |
| `X` | [eXecute](execute.md) a shell command |
| Alt+X | the [built-in terminal](terminal.md) |
| F8 | [split screen](split.md) |
| F11 / Alt+B | [bookmarks](bookmarks.md); Ctrl+B saves one |

See also: [the file window](files.md), [the keys in effect](keys.md).
