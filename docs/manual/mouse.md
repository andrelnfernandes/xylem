# The mouse

Xylem is made for the keyboard, and the mouse is off until you turn it on:
**F12**, page 2, **R** (or `mouse = true` in the `[options]` of
[the configuration file](configuration.md)), then **F3** to save. It works
at once.

The mouse never has commands of its own. A click picks a row, or stands for
the key written where you clicked, and that key does what it always does:
Delete still asks, and a command that cannot run here is still refused.

## What it does

| Where | Click | Double click | Wheel |
|---|---|---|---|
| The tree | picks the directory; the file window follows it | Enter: to its files | moves the cursor 3 directories |
| The mark before a folder | `+` opens it, `?` logs and opens it, the `─` of an open one closes it | | |
| The file window | picks the file and moves to the file window | View, in Xylem's own viewer | moves the cursor 3 files (a row in the column modes) |
| The scroll bar on the right border | above or below the thumb: a page; drag the thumb to go anywhere | | |
| The other side of a [split screen](split.md) | makes it the active window, then picks | | moves that side's cursor, the active one stays |
| The tree above the [built-in terminal](terminal.md) | goes back to Xylem, as Alt+X | | |
| The footer | the key of the entry: `Copy` is C, `Esc cancel` is Esc, `Yes` answers the question | | |
| F12 | an option changes, as its letter (nothing is saved before F3); a theme of the list (F4) is previewed | the theme is chosen | moves the selection |
| Bookmarks, the search box, Usage, Duplicates | picks the row | its Enter | 3 rows |
| The history of a prompt (↑) | puts that entry in the prompt | | 3 rows |
| The text of a prompt | the cursor goes there | | |
| A table in the viewer | picks the cell | its Enter: the record | 3 rows; Shift+wheel, a column |
| The viewer, the help | a link of the help is followed | | scrolls, as ↑ and ↓; Shift+wheel scrolls the viewer sideways |
| Anywhere a menu makes sense | the right button opens it (below) | | |

A double click never opens a file with a program of the system: a file you
did not create may be hostile, and a click by mistake must not run it. Open
(`O`) is still there.

Dragging with the left button moves the cursor along a list; past its top
or bottom edge, the list scrolls a row at each move.

## The menu of the right button

The right button moves the cursor to the item under the pointer, as a
click, and opens a menu of what can be done with it, next to the pointer.
The **Menu** key of the keyboard, or **Shift+F10**, opens the same menu for
the item under the cursor, with no mouse.

| Under the pointer | The menu |
|---|---|
| A file | View, Edit, Open with the system, Info; Tag or Untag; Copy, Move, Rename, Delete, Attributes, New date, Yank the path |
| A file, with files tagged | the same, and a group for the tagged ones: Copy, Move, Rename (mask), Delete, View, Yank, Untag all |
| A directory of the tree | Files, Log, Relog, Expand all, Collapse, Release; Branch, Showall; Make a directory, Rename, Delete, Prune, Attributes; Tag its files, Untag; Terminal here, Usage, Duplicates, Bookmark it, Yank the path |
| The file window, past its files | Filespec, Sort, File display; Tag all, Untag all, Invert tags |
| The viewer | Search, Next match, Go to line, and the keys of its mode (Wrap, Line numbers, Hex, SQL filter...), Edit, Yank the path, Close |

Each line shows its key on the right, as your configuration has it, so
the menu is also a way to learn the keys. What cannot run here is left out:
inside an archive there is no Delete, Move or Rename. A line runs as its key
would: Delete still asks first.

↑↓ choose, Enter runs, Esc closes; a click off the menu only closes it.

## Tagging

| Gesture | What it does |
|---|---|
| Middle button, or Ctrl+click, on a file | tags or untags it; the cursor stays where it is |
| The same on a folder of the tree | picks it and tags its files, or untags them when all were tagged |
| Shift+click on a file | tags every file from the cursor to that one, and the cursor goes there |

Many terminals keep Shift+click for their own selection; there the
middle button or Ctrl+click tags each file.

## The focus, the menus and the questions

The wheel moves the list under the pointer, not the one with the focus: the
small file window under the tree scrolls without leaving the tree.

In the Ctrl and Alt menus (held down, or opened with F9), a word stands for
its letter with Ctrl or Alt: a click on `Copy` in the Ctrl menu copies the
tagged files.

While a question, a prompt or some work in progress is on the screen, the
windows behind it take no clicks: only its own entries in the footer do.

## Selecting text in the terminal

With the mouse on, the terminal sends the clicks to Xylem, so dragging no
longer selects text on the screen. Hold **Shift** while dragging (in xterm,
GNOME Terminal, Konsole, kitty, WezTerm, Windows Terminal), or **Option**
in iTerm2. To copy the path of a file, Yank (`Y`) needs no mouse.

## Terminals

Xylem asks for clicks and for moves with a button held, in the SGR format
that every current terminal knows; never for the moves of the pointer alone,
which would send a stream of events for nothing, a weight over SSH. A
program run by eXecute or Edit may turn the mouse off as it ends; Xylem
turns it on again when it gets the screen back.
