# Treespec and Log

## Treespec (`\` or `/`)

Opens a prompt with the current path. As you type, the tree cursor goes to the
first directory starting with what you have typed (case-insensitive):

- `/` or `\` complete the name and go down one level;
- Tab and Shift+Tab complete and move to the next or previous directory at the
  same level;
- Backspace goes back up levels;
- Ctrl+← takes the last folder off the path (`~/projects/xylem/docs/` becomes
  `~/projects/xylem/`, with the cursor on `xylem`), and Ctrl+→ brings back, one
  at a time, the folders it took off; typing something else starts a new path;
- Enter confirms and Esc returns to where you were;
- ↑ opens the earlier paths as a [list](execute.md#history) (saved between runs).

A path outside the tree becomes a Log when confirmed. In Branch and Showall,
`\` goes straight to the directory of the selected file, with it highlighted.

## Log (L)

Opens another directory in the window, replacing the tree (a directory inside
the current tree just moves the cursor there). An address like `user@host:`
opens a [remote machine](remote.md). The path can be absolute,
relative to the current directory or start with `~`; **F2**
[picks it from a tree](picker.md) and ↑ lists earlier ones. New windows
keep the current [sort order](sort.md) and display mode.

The prompt opens empty. ↑ opens a [list](execute.md#history) of the places
used before, the last one at the bottom: those logged and the folders files
were copied or moved to. Typing filters it, Enter puts a place in the line and
Del takes it out of the list.

**F11** brings a [bookmark](bookmarks.md) into the prompt, by its name: handy for
long addresses of remote machines and containers.

See also: [the tree window](tree.md), [bookmarks](bookmarks.md).
