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

A double click never opens a file with a program of the system: a file you
did not create may be hostile, and a click by mistake must not run it. Open
(`O`) is still there.

Dragging with the left button moves the cursor along a list; past its top
or bottom edge, the list scrolls a row at each move.

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
