# The mouse

Xylem is made for the keyboard, and the mouse is off until you turn it on:
**F12**, page 2, **R** (or `mouse = true` in the `[options]` of
[the configuration file](configuration.md)), then **F3** to save. It works
at once.

The mouse never has commands of its own. A click picks a row, or stands for
the key written where you clicked, and that key does what it always does:
Delete still asks, and a command that cannot run here is still refused.

## What it does

| Where | Click | Wheel |
|---|---|---|
| The tree | picks the directory; the file window follows it | moves the cursor 3 directories |
| The file window | picks the file and moves to the file window | moves the cursor 3 files (a row in the column modes) |
| The other side of a [split screen](split.md) | makes it the active window, then picks | moves that side's cursor, the active one stays |
| The tree above the [built-in terminal](terminal.md) | goes back to Xylem, as Alt+X | |
| The footer | the key of the entry: `Copy` is C, `Esc cancel` is Esc, `Yes` answers the question | |
| The viewer, the help, F12, the bookmarks | `F3 save`, `Esc`, a link of the help | scrolls, as ↑ and ↓; Shift+wheel scrolls the viewer sideways |

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
