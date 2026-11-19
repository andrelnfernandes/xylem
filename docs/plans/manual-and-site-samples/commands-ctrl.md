# The Ctrl menu

Ctrl with a letter runs the command on the tagged files. In the file window
these are the tagged files of the list (the directory, or the open Branch or
Showall). In the tree they are the tagged files of the directory under the
cursor. The rows marked "everything logged" are the exceptions.

With no file tagged, these commands show `No tagged files` and do nothing.

To show this menu, hold Ctrl or press F9. See [the menus](menus.md).

## Commands

In the order of the menu. "Tree" and "Files" say where the key works.

| Key | Command | Tree | Files | What it does |
|---|---|---|---|---|
| Ctrl+A | Attributes | yes | yes | Changes the permissions of the tagged files. [Permissions and dates](attributes.md) |
| Ctrl+B | Bookmark | yes | | Saves the directory under the cursor as a bookmark. Does not use the tags. [Bookmarks](bookmarks.md) |
| Ctrl+B | Batch | | yes | Writes a script or a list with one line per tagged file, from a mask. [Batch](execute.md) |
| Ctrl+C | Copy | yes | yes | Copies the tagged files into one destination directory, without their paths. Alt+C keeps the paths. [Copy and Move](copy-move.md) |
| Ctrl+D | Delete | yes | yes | Deletes the tagged files, after asking. They go to the trash when `delete_to_trash` is on. [Delete](delete.md) |
| Ctrl+G | Git | yes | yes | Opens the Git menu. Shows only with the Git option on. [Git](git.md) |
| Ctrl+M | Move | yes | yes | Moves the tagged files into one destination directory. In terminals without the kitty keyboard protocol, press `F9` `M`. [Copy and Move](copy-move.md) |
| Ctrl+N | New date | | yes | Sets the modification date of the tagged files. [Permissions and dates](attributes.md) |
| Ctrl+O | Open | | yes | Opens each tagged file in the system's default program. [The file window](files.md) |
| Ctrl+P | Phrase | yes | yes | Opens the search box on the tagged files: it lists the ones that contain the words typed. [The search box](filespec.md#the-search-box-p) |
| Ctrl+R | Rename | yes | yes | Renames the tagged files with a mask. [Rename](rename.md) |
| Ctrl+S | Showall | yes | | Lists the tagged files of everything logged. [The tree window](tree.md) |
| Ctrl+S | Search | | yes | Searches the tagged files for a text and untags the ones that do not contain it. [Searching tagged files](search.md) |
| Ctrl+T | Tag | yes | yes | Files: tags the whole list. Tree: tags everything logged that matches the Filespec. [Tags](tagging.md) |
| Ctrl+U | Untag | yes | yes | Files: untags the whole list. Tree: untags everything logged that matches the Filespec. |
| Ctrl+V | View | | yes | Opens the tagged files in the viewer, one after another; `N` and `P` switch files. [Searching tagged files](search.md) |
| Ctrl+W | Where | yes | yes | Keeps in the list the tagged files that meet a SQL condition. [A SQL condition](filespec.md#a-sql-condition-w) |
| Ctrl+Y | Yank | yes | yes | Copies the paths of the tagged files to the clipboard. |

## Function keys

| Key | Where | What it does |
|---|---|---|
| Ctrl+F5 | Files | Puts the tagged files in a new archive. [Archives](archives.md) |
| Ctrl+F7 | Tree | Tags the logged files of the branch under the cursor. |
| Ctrl+F8 | Tree | Untags them. |
| Ctrl+F9 | Tree | Inverts their tags. |
| Ctrl+F11 | both | Saves a bookmark, as Ctrl+B does in the tree. |
| Ctrl+Shift+Del | both | Deletes the tagged files permanently, without the trash. |
| Ctrl+Z | both | Oops: restores what was last sent to the trash. |

## Inside an archive or on a remote machine

The menu leaves out what cannot run there. An archive is read-only: the
commands that write are not listed, and Copy extracts. On a remote machine
Batch and Git are not listed.
