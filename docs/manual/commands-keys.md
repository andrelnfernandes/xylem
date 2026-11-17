# Function keys and symbols

These keys are on the last line of the DIR and FILE menus, or are not in a
menu because they only move the cursor. They are not combined with the F9
menus: after F9, only the next letter takes Ctrl or Alt.

## Function keys

| Key | Where | What it does |
|---|---|---|
| F1 | both | Opens the help for what is on the screen. Alt+H does the same. [Terminals](terminals.md) |
| F2 | both | Rereads the directory, as F3. In a prompt that asks for a folder, opens a tree to [pick it](picker.md). |
| F3 | both | Rereads the directory from the disk. |
| F5 | Files | Creates an archive with the file. Ctrl+F5: with the tagged files. Alt+F5: opens an archive. [Archives](archives.md) |
| F7 | Files | Autoview: shows the contents of the file under the cursor beside the list. [The file window](files.md) |
| F8 | both | Opens a second window beside the first, and closes it. [Split screen](split.md) |
| F9 | both | Shows the CTRL menu, then the ALT menu, then goes back. [The menus](menus.md) |
| F10 | both | Quits, as `Q`. |
| F11 | both | Opens the bookmarks manager, as Alt+B. Ctrl+F11 saves the current directory. [Bookmarks](bookmarks.md) |
| F12 | both | Opens the configuration. [Configuration](configuration.md) |

## Symbols

| Key | Where | What it does |
|---|---|---|
| `*` | Tree | Logs the directory under the cursor and everything below it. Esc cancels. |
| `+` or → | Tree | Logs the directory and shows its subdirectories. |
| `-` | Tree | Releases the branch from memory. |
| ← | Tree | Closes the directory, or goes to its parent. |
| `?` | both | Shows the statistics of the window, the disk, the machine and Xylem. [Information](info.md) |
| `\` or `/` | both | Treespec: type a path and the tree cursor follows it. In a Branch or Showall, goes to the directory of the file under the cursor. [Treespec](treespec.md) |
| `'` | both | Jumps to a bookmark: press its key next. |

## Moving

| Key | Where | What it does |
|---|---|---|
| ↑ ↓ PgUp PgDn Home End | both | Move the cursor. |
| Space | both | Moves down one line. |
| Enter | Tree | Goes to the file window. |
| Enter | Files | Goes back to the tree. With `enter_views` on, views the file. |
| Esc | Files | Goes back to the tree, at the directory of the file under the cursor. |
| Esc | Tree | Inside an archive, closes it. |
| Tab | both | With the screen split, goes to the other window. Otherwise, the same as Enter. |
| Backspace | both | Goes to the parent directory. |
| Shift+letter, 0–9 | both | Goes to the next directory or file whose name starts with it. |
| ← → | Files | Page up and down. In the column display modes, the previous and the next file. |

## Changing a key

`[keys.any]`, `[keys.tree]` and `[keys.files]` in `config.toml` bind a key to
another command, or to `none`. **K** in the help lists the keys in effect,
with these changes. See [configuration](configuration.md).

See also: [the DIR and FILE menus](commands.md),
[the Ctrl menu](commands-ctrl.md), [the Alt menu](commands-alt.md).
