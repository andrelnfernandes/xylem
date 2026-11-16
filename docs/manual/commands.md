# The DIR and FILE menus

A letter alone runs the command on the item under the cursor: the directory
in the tree (the DIR menu), the file in the file window (the FILE menu).

Some letters are a different command in each window. `C` compares in the
tree and copies in the file window; `M` makes a directory in the tree and
moves a file in the file window.

## Commands

In the order of the menus. "Tree" and "Files" say where the key works.

| Key | Command | Tree | Files | What it does |
|---|---|---|---|---|
| `A` | Available | yes | | Lists the mounted file systems with total, used and free space. [Information](info.md) |
| `A` | Attributes | | yes | Changes the file's permissions, in octal. [Permissions and dates](attributes.md) |
| `B` | Branch | yes | yes | Lists the files of the directory and of everything below it, logging the branch first. In a Branch or Showall, goes back to the files of the directory alone. |
| `C` | Compare | yes | | With the screen split, compares the directory with the other window's and tags the files that differ in the way you choose. [Compare](compare.md) |
| `C` | Copy | | yes | Copies the file. The prompt asks for the destination. [Copy and Move](copy-move.md) |
| `D` | Delete | yes | yes | Deletes the file, after asking. In the tree, deletes the directory if it is empty. [Delete](delete.md) |
| `E` | Edit | | yes | Opens the file in your editor. [Edit](execute.md) |
| `F` | Filespec | yes | yes | Sets which files the window shows, by name, date and size. [Filespec](filespec.md) |
| `I` | Invert | yes | yes | Tags the untagged files and untags the tagged ones: of the directory in the tree, of the list in the file window. |
| `L` | Log | yes | | Opens another directory, or a remote machine, as the tree of this window. [Log](treespec.md) |
| `M` | Makedir | yes | | Creates a subdirectory in the directory under the cursor. |
| `M` | Move | | yes | Moves the file. [Copy and Move](copy-move.md) |
| `N` | New date | yes | yes | Sets the modification date of the file or directory. [Permissions and dates](attributes.md) |
| `O` | Oops | yes | | Restores what was last sent to the trash. In the file window, Ctrl+Z. [Delete](delete.md) |
| `O` | Open | | yes | Opens the file in the system's default program. [The file window](files.md) |
| `P` | Phrase | yes | yes | Opens the search box: lists the logged files that contain the words typed. [The search box](filespec.md#the-search-box-p) |
| `R` | Rename | yes | yes | Renames the file or the directory. [Rename](rename.md) |
| `S` | Showall | yes | yes | Lists every logged file of the window. |
| `T` | Tag | yes | yes | Files: tags the file and moves down. Tree: tags the files of the directory. [Tags](tagging.md) |
| `U` | Untag | yes | yes | Files: untags the file and moves down. Tree: untags the files of the directory. |
| `V` | View | | yes | Opens the file in the viewer: text, hex, or a table. [The viewer](viewer.md) |
| `W` | Where | yes | yes | Keeps in the list the files that meet a SQL condition. [A SQL condition](filespec.md#a-sql-condition-w) |
| `X` | eXecute | yes | yes | Runs a shell command. `%f` stands for the file and `%t` for the tagged files. [eXecute](execute.md) |
| `Y` | Yank | yes | yes | Copies the path of the file or directory to the clipboard. |
| `Q` | Quit | yes | yes | Quits, after asking. [Quitting](quit.md) |

The keys on the last line of the menu (F1, `*`, `?`, `\`, F3, Enter, F5, F7,
F8, F9) are in [function keys and symbols](commands-keys.md).

## Keys that are not in the menu

These are second keys for a command in the table.

| Key | Same as |
|---|---|
| Del | `D` |
| Shift+Del | `D`, deleting permanently without the trash |
| F10 | `Q` |
| Ctrl+Insert | `Y` |
| Ctrl+Z | `O` (Oops), in both windows |

## Inside an archive or on a remote machine

The title of the menu is ARCHIVE or REMOTE, and the commands that cannot run
there are not listed.

| Where | Not listed |
|---|---|
| Archive | Delete, Edit, Log, Makedir, Move, New date, Oops, Rename, Attributes, eXecute. Copy extracts. |
| Remote machine | Edit, Oops, eXecute. There is no trash on a remote machine. |

See [archives](archives.md) and [remote machines](remote.md).

See also: [the Ctrl menu](commands-ctrl.md), [the Alt menu](commands-alt.md),
[the menus](menus.md).
