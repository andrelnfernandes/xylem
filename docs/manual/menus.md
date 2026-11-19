# The menus, Ctrl and Alt

The footer lists the commands that work in the window that has the focus.
There are four menus. The title on the left says which one is showing.

| Title | Shows when | Its commands act on |
|---|---|---|
| DIR | the tree has the focus | the directory under the cursor |
| FILE | the file window has the focus | the file under the cursor |
| CTRL | Ctrl is held, or after F9 | the tagged files |
| ALT | Alt is held, or after F9 twice | see [the Alt menu](#alt-the-other-version-of-a-command) |

Inside an archive the title is ARCHIVE, and on a remote machine REMOTE. Both
leave out the commands that do not work there.

## Reading a menu

Each word is a command, and its capital letter is the key: `Copy` is `C`,
`eXecute` is `X`, `iNdex` is `N`. In the CTRL and ALT menus the key is that
letter with Ctrl or Alt.

The words are in alphabetical order by their key, with Quit last. Below them
is a line with the function keys and symbols (`F3 relog`, `? stats`).

The footer has three lines. On a screen wide enough to fit all the words on
one line, it has two. On a narrow screen the words are shortened, all by the
same number of letters, and a `…` marks entries that did not fit.

## Ctrl: the tagged files

A letter alone acts on one item. The same letter with Ctrl acts on the tagged
files of the list:

| Key | Alone | With Ctrl |
|---|---|---|
| `C` | copy the file | copy the tagged files |
| `D` | delete the file | delete the tagged files |
| `R` | rename the file | rename the tagged files with a mask |
| `T` | tag the file | tag every file in the list |

In the tree, "the list" is the directory under the cursor. Three commands are
wider there: Ctrl+T and Ctrl+U tag and untag every logged file that matches
the Filespec, and Ctrl+S lists the tagged files of everything logged.

[The Ctrl menu](commands-ctrl.md) has every command.

## Alt: the other version of a command

Alt does not repeat Ctrl. What it does depends on the window:

- In the tree, the command covers the whole branch. Alt+P (Prune) deletes the
  directory and everything in it, Alt+R releases the branch from memory,
  Alt+C compares the branch with the other window.
- In the file window, Alt+C and Alt+M copy and move the tagged files and
  recreate each file's path under the destination.
- In both, Alt opens the tools that are not about one file: Alt+S (Sort),
  Alt+F (File display), Alt+B (Bookmarks), Alt+I (Info), Alt+X (the terminal).

[The Alt menu](commands-alt.md) has every command.

## Showing the Ctrl and Alt menus

**F9** shows the CTRL menu, F9 again the ALT menu, and a third time goes
back. While one of them is showing, the next letter is taken with that
modifier: `F9` `C` is Ctrl+C. Esc goes back without running anything.

In kitty, Ghostty, WezTerm, foot and recent Alacritty, holding Ctrl or Alt
shows the menu, and releasing the key brings back DIR or FILE. These
terminals report a modifier pressed on its own (the kitty keyboard protocol).
GNOME Terminal, Ptyxis, Terminator and Tilix do not, so in them the menus
change only with F9.

## Ctrl+M

Most terminals send the same byte for Ctrl+M and Enter, so Xylem cannot tell
them apart. In the terminals with the kitty keyboard protocol it can, and
Ctrl+M moves the tagged files. In the others, press `F9` `M`.

## Changed in version 0.8

Alt+A, Alt+D, Alt+R, Alt+T and Alt+U used to do the same as Ctrl with the
same letter. [Release notes](releases.md) has the old and the new meaning of
each key, and the lines for `config.toml` that restore the old ones.
