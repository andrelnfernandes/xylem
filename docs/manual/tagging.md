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

As in ZTreeWin:

- **A letter alone** acts on the item under the cursor.
- **Ctrl+letter** acts on the tagged files: Ctrl+C copies them, Ctrl+D
  deletes them, Ctrl+T tags everything.
- **Alt+letter** is the other version of a command: in the tree, the whole
  branch (Alt+P Prune, Alt+R Release, Alt+U Usage, Alt+D Duplicates, Alt+C
  Compare); in the file window, the tagged files with their paths (Alt+C,
  Alt+M); and the tools (Alt+S Sort, Alt+F File display, Alt+B Bookmarks,
  Alt+H Help, Alt+I Info, Alt+X the terminal).

**Changed in version 0.8.** Alt no longer repeats Ctrl, and some keys moved:

| Key | Up to 0.7 | From 0.8 |
|---|---|---|
| Alt+A | Attributes of the tagged files | nothing (Ctrl+A) |
| Alt+D | delete the tagged files | Duplicates (deleting them: Ctrl+D) |
| Alt+R | rename the tagged files | Release the branch, as `-` (renaming them: Ctrl+R) |
| Alt+T | tag everything | nothing (Ctrl+T) |
| Alt+U | untag everything | Usage (untagging: Ctrl+U) |
| Alt+W | Usage | nothing (Alt+U) |
| Alt+Y | Duplicates | nothing (Alt+D) |

The old keys can be brought back in `[keys.any]` of the
[configuration](configuration.md), one or all of them. All of them, as up to 0.7:

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

They apply in both windows, also Alt+R in the tree; Release stays on `-`.
**K** in the help (F1) shows the keys in effect.

## Ctrl and Alt menus (F9)

The footer has three lines on an ordinary screen: the letter commands in
alphabetical order on the first two, the function keys and symbols on the
third. On a screen wide enough for every menu to fit, it has two: the letters
on one, the keys on the other. Every command is in one
of the menus (DIR or FILE, CTRL, ALT); inside an archive or on a remote
machine, they leave out what does not work there.

**F9** switches to the
Ctrl menu, then the Alt menu, and back. With one of these menus open, the next
letter counts as Ctrl+letter or Alt+letter (`F9` `M` = Ctrl+M); `Esc` cancels.
In terminals with the kitty keyboard protocol, just **holding Ctrl or Alt**
shows the corresponding menu, and letting go brings the usual one back, as
in ZTree.

The terminal has to report Ctrl and Alt pressed and released on their own,
which only the kitty keyboard protocol does: kitty, Ghostty, WezTerm, foot
(Wayland) and recent Alacritty support it, and on Ubuntu or Debian most of
them are a package away (`sudo apt install kitty`, or `ghostty`,
`alacritty`, `foot`). Terminals built on VTE (GNOME Terminal, Ptyxis,
Terminator, Tilix) do not support it yet, so there the menus stay on F9.

**Ctrl+M:** in most terminals, Ctrl+M arrives the same as Enter. Xylem enables
the kitty keyboard protocol when the terminal supports it (kitty, WezTerm,
foot, Ghostty, recent Alacritty, iTerm2), and then Ctrl+M moves the tagged
files as in ZTree. In VTE-based terminals (GNOME Terminal, Ptyxis, Terminator,
Tilix) use **F9 M** (or Alt+M and F6 at the prompt).
