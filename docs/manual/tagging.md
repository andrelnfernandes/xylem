# Tags and group operations

Tagged files (marked with `■`) are what the group operations act on: the
tagged files **of the current list** (the directory, or the open Branch or
Showall).

## Tagging

| Key | Tree | File window |
|---|---|---|
| `T` / `U` | tag / untag the files of the directory | tag / untag the file and move down |
| Ctrl+T / Ctrl+U | tag / untag everything logged (that matches the Filespec) | tag / untag the whole list |
| Ctrl+S | Showall of the tagged files only | |
| `I` | invert the directory's tags | invert the list's tags |
| Ctrl+F7 / F8 / F9 | tag / untag / invert the logged files of the branch | |

## Operations on the tagged files

| Key | What it does |
|---|---|
| Ctrl+C / Ctrl+M | [copy / move](copy-move.md) them into the destination |
| Alt+C / Alt+M | copy / move them, **recreating their paths** |
| Ctrl+D | [delete](delete.md) them |
| Ctrl+A | change their [permissions](attributes.md) |
| Ctrl+N | change their [date](attributes.md) |
| Ctrl+R | [rename](rename.md) them with a mask |
| Ctrl+B | [Batch](execute.md): a script or list with one line per file |
| Ctrl+F5 | put them in an [archive](archives.md) |
| Ctrl+S / Ctrl+V | [search them for text / view them](search.md) |
| Ctrl+O | open them in the default program |
| Ctrl+Y | copy their paths to the clipboard |
| Alt+O | type them into the [built-in terminal](terminal.md) |

In eXecute, `%t` stands for the tagged files ([eXecute](execute.md)).

## What Ctrl and Alt mean

- A letter alone acts on the item under the cursor.
- Ctrl+letter acts on the tagged files: Ctrl+C copies them, Ctrl+D deletes
  them.
- Alt+letter is the other version of a command: the whole branch in the
  tree, the tagged files with their paths in the file window.

[The menus](menus.md) explains the rule and how to show the Ctrl and Alt
menus with F9. [The Ctrl menu](commands-ctrl.md) and
[the Alt menu](commands-alt.md) list every command.

**Changed in version 0.8.** Alt no longer repeats Ctrl: Alt+A, Alt+D, Alt+R,
Alt+T, Alt+U, Alt+W and Alt+Y changed. [Release notes](releases.md) has the
old and the new meaning of each key, and the lines for `config.toml` that
restore the old ones.
