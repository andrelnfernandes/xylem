# The Alt menu

Alt with a letter is the other version of a command. In the tree it covers
the whole branch under the cursor. In the file window, Alt+C and Alt+M act on
the tagged files and keep their paths. The other entries open tools that do
not act on one file.

To show this menu, hold Alt or press F4 twice. See [the menus](menus.md).

## Commands

In the order of the menu. "Tree" and "Files" say where the key works.

| Key | Command | Tree | Files | What it does |
|---|---|---|---|---|
| Alt+B | Bookmarks | yes | yes | Opens the bookmarks manager: jump to a saved directory, name it, remove it. [Bookmarks](bookmarks.md) |
| Alt+C | Compare | yes | | Compares the whole branch with the other window's, by relative path, and opens a Branch with the files tagged. [Compare](compare.md) |
| Alt+C | Copy | | yes | Copies the tagged files and recreates each file's path under the destination. F7 at the prompt sets where the path starts. [Copy and Move](copy-move.md) |
| Alt+D | Duplicates | yes | yes | Finds files and folders with the same contents in the branch. Deletes nothing without asking. [Duplicates](duplicates.md) |
| Alt+F | File display | yes | yes | Changes what each line of the file window has: names in columns, sizes, details, owner, type, long names. [The file window](files.md) |
| Alt+H | Help | yes | yes | Opens the help, as F1 does. |
| Alt+I | Info | yes | yes | Shows the type, size, permissions, owner and dates of the file or directory. [Information](info.md) |
| Alt+K | Kbyte | yes | | Shows the size, the number of files or the percentage next to each directory of the tree. [Information](info.md) |
| Alt+L | Log | yes | yes | Makes the directory under the cursor the root of the tree. What was logged and the tags are dropped. [Log](treespec.md) |
| Alt+M | Move | | yes | Moves the tagged files and recreates each file's path under the destination. [Copy and Move](copy-move.md) |
| Alt+P | Prune | yes | | Deletes the directory with everything in it, permanently, after you type a confirmation. [Delete](delete.md) |
| Alt+Q | Quit here | yes | yes | Quits and leaves the shell in the current directory. Needs the shell function from [Quitting](quit.md). |
| Alt+R | Release | yes | | Drops the branch from memory, as `-` does. Its tags are lost. |
| Alt+S | Sort | yes | yes | Sorts the file window by name, extension, date or size. [Sort](sort.md) |
| Alt+U | Usage | yes | yes | Lists the folders and files of the branch by the space they take, largest first. [Disk usage](usage.md) |
| Alt+V | Views | yes | yes | Opens the views of the list: the Scan, which points at what deserves a look, the Timeline, by date, and the Table, the list in the table viewer. [Views](views.md) |
| Alt+N | iNdex | yes | yes | Indexes the words of the branch's logged files, so that `words()` in a SQL condition answers at once. [The index](sql.md#whole-words-and-an-index-words) |

## Other keys

| Key | Where | What it does |
|---|---|---|
| Alt+X | both | Opens the built-in terminal, and goes back from it. [The built-in terminal](terminal.md) |
| Alt+Z | both | Opens the terminal in full screen. |
| Alt+O | both | Types the tagged files into the terminal's command line, quoted. |
| Alt+F5 | Files | Opens the archive under the cursor as a tree. [Archives](archives.md) |
| Alt+F10 | both | Opens the configuration, as F12 does. On GNOME, Alt+F10 maximizes the window and does not reach Xylem. [Configuration](configuration.md) |
| Alt+Shift+Del | both | Deletes the tagged files permanently, as Ctrl+Shift+Del does. |

## Inside an archive or on a remote machine

| Where | Not listed |
|---|---|
| Archive | Log, Move, Prune, the terminal keys. Alt+C extracts the tagged files with their paths inside the archive. |
| Remote machine | Log (Alt+L), Alt+O. Alt+X opens a shell on that machine. |

## Changed in version 0.8

Up to 0.7, Alt+A, Alt+D, Alt+R, Alt+T and Alt+U did the same as Ctrl with
that letter, Alt+W was Usage and Alt+Y was Duplicates.
[Release notes](releases.md) has the table and the lines for `config.toml`
that restore the old keys.

See also: [the DIR and FILE menus](commands.md),
[the Ctrl menu](commands-ctrl.md).
