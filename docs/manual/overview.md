# What Xylem is

Xylem is a file manager that runs in a terminal. Its screen and keys follow
XTreeGold and ZTreeWin: a directory tree at the top, the files of the
directory under the cursor below it, statistics on the right, and a menu of
one-letter commands in the footer. The command is `xy`.

## Logging

Xylem reads a directory into memory when you ask for it, and keeps it there.
This is called logging. `→` logs one directory, `*` logs the whole branch
under the cursor, and `-` releases a branch from memory.

Two commands list logged files from more than one directory at once: `B`
(Branch) lists the files of the directory and everything below it, and `S`
(Showall) lists every logged file. In these lists you tag files with `T`, and
the Ctrl commands then act on the tagged files: Ctrl+C copies them, Ctrl+D
deletes them, Ctrl+R renames them with a mask.

Files in a directory that is not logged do not appear in any list and are
not affected by any command.

## What the commands cover

| To do this | Use | Topic |
|---|---|---|
| Copy or move, with or without the source paths | `C`, `M`, Ctrl+C, Alt+C | [Copy and Move](copy-move.md) |
| Delete, and restore what went to the trash | `D`, Ctrl+Z | [Delete](delete.md) |
| Rename one file, or many with a mask | `R`, Ctrl+R | [Rename](rename.md) |
| Show only some files, by name, date or size | `F` | [Filespec](filespec.md) |
| Compare two directories | `C` in the tree | [Compare](compare.md) |
| View a file as text, hex or a table | `V` | [The viewer](viewer.md) |
| Open ZIP, 7z and TAR as directories | Alt+F5 | [Archives](archives.md) |

[The menus](menus.md) explains how the keys are organized, and
[the commands](commands.md) lists all of them.

[Comparison with ZTreeWin](ztreewin.md) lists the ZTreeWin commands Xylem
does not have, and the keys that differ.

## Files you do not trust

Xylem reads every file as if it were written to attack it. Viewing a file
runs nothing from it, extracting an archive does not write outside the
destination, and each reader stops at a size limit.
[Security](security.md) lists the limits and what is not covered.
