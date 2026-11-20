# Xylem manual

Xylem is a file manager for the terminal, driven by the keyboard: a directory
tree on top, the files of the current directory below, and statistics on the
right. It logs (reads) directories into memory, so one
list can hold files from a whole branch, and one command copies, moves,
deletes, compares or renames all the files tagged in it.

Xylem treats files it did not create as hostile: a file's contents or name
should never make it crash, hang, run code, write outside the destination or
use up the memory. [Security](security.md) lists the measures and what is not
covered.

**F1** (or **Alt+H**, for terminals that keep F1 for themselves) opens the
help for what is on the screen: inside a prompt, the topic of that prompt; in
the tree or the file window, the topic of that window.

| Key in the help | What it does |
|---|---|
| Tab, Shift+Tab | select the next or the previous link |
| Enter | follow the link |
| Backspace | go back |
| `I` | this index |
| `K` | the keys in effect |
| `L` | switch the language |
| Esc | close |

The Spanish, French, German and Chinese versions have not been reviewed by
native speakers. Corrections: `https://github.com/andrelnfernandes/xylem/issues`.

## Start here

- [What Xylem is](overview.md): logging, tags, and what the commands cover
- [Install and first five minutes](start.md)
- [The screen](screen.md): the tree, the file window, the statistics, the footer
- [The menus, Ctrl and Alt](menus.md): a letter alone, with Ctrl, with Alt, and F9
- [The mouse](mouse.md): turning it on in F12, the wheel, clicks and the footer
- [Vocabulary](glossary.md): log, branch, tag, filespec

## Commands

- [The DIR and FILE menus](commands.md): the letters alone
- [The Ctrl menu](commands-ctrl.md): the tagged files
- [The Alt menu](commands-alt.md): the branch, the paths and the tools
- [Git](git.md): the Git menu (Ctrl+G)
- [Function keys and symbols](commands-keys.md)
- [The keys in effect](keys.md): with the changes from the configuration

## Windows and views

- [The tree window](tree.md): moving around, logging, Branch and Showall
- [The file window](files.md): display modes, Autoview, opening files
- [Split screen](split.md): two windows side by side
- [The viewer](viewer.md): text, hex and tables, search, syntax colors
- [Filtering lines](filter.md): `&` in the viewer, with a regular expression guide
- [Searching tagged files](search.md): Ctrl+S and Ctrl+V
- [Information](info.md): Info, Avail, statistics and directory sizes
- [Disk usage](usage.md): where the space goes (Alt+U)
- [Duplicates](duplicates.md): files and folders with the same contents (Alt+D)

## Working with files

- [Tags and group operations](tagging.md)
- [Copy and Move](copy-move.md), [picking a folder](picker.md)
- [Delete, trash and Prune](delete.md)
- [Rename and batch rename](rename.md)
- [Permissions and dates](attributes.md)
- [Filespec: the file filter](filespec.md)
- [Treespec and Log](treespec.md)
- [Compare](compare.md) and [Sort](sort.md)
- [Edit, eXecute and Batch](execute.md)
- [Archives](archives.md): ZIP, 7z and TAR

## More tools

- [The built-in terminal](terminal.md)
- [Remote machines (SSH)](remote.md): another machine or a container as a tree
- [Cloud storage (rclone)](cloud.md): Google Drive, S3 and others
- [Finding files with SQL](sql.md): a condition on the files and their contents (W), and the words in them (P)
- [Tables](tables.md): spreadsheets, CSV, dBase, SQLite and Parquet as a grid, with a SQL filter and each record as a form
- [Bookmarks](bookmarks.md)

## How to

- [Copy files from several folders, keeping their paths](howto-copy-with-paths.md)
- [Rename many files with a counter](howto-rename-mask.md)
- [Pick files with a SQL condition](howto-sql-filter.md)
- [Find duplicates and keep one copy](howto-duplicates.md)
- [Find what fills the disk](howto-disk-space.md)
- [Filter a CSV file or a spreadsheet with SQL](howto-table-sql.md)
- [Copy files from another machine over SSH](howto-ssh-copy.md)
- [Use the tagged files in a shell command](howto-terminal-tagged.md)

## Reference

- [Configuration](configuration.md): the F12 screen, themes, keys
- [The command line](cli.md)
- [Terminals and keys](terminals.md): F1, Ctrl+M, the kitty keyboard protocol
- [Security](security.md)
- [Quitting and changing the shell's directory](quit.md)
- [Windows](windows.md)
- [Comparisons](comparison.md): other file managers, and a thank-you
- [Release notes](releases.md)
