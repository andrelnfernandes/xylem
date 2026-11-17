# Vocabulary

The words below come from XTree and ZTreeWin. The menus and this manual use
them with these meanings.

| Word | Meaning |
|---|---|
| Log | To read a directory into memory: its files and the names of its subdirectories. `→` or `+` logs one directory and `*` a whole branch. `L` (Log) is also the command that opens another directory as the tree. [The tree window](tree.md) |
| Logged | Read into memory. Only logged files appear in lists, are counted in the statistics, and can be tagged. |
| Not logged | A directory whose name is known and whose contents have not been read. The tree marks it with `?`. |
| Release | To drop a branch from memory with `-`. Its files leave the lists and lose their tags. Nothing changes on the disk. |
| Relog | To read a directory from the disk again with F3. |
| Branch | A directory and everything below it. `B` lists the files of the branch in one list. |
| Showall | The list of every logged file in the window, opened with `S`. |
| Tag | A mark on a file (`■`). The Ctrl commands act on the tagged files. Tags are kept in memory and are lost on Release or on quitting. [Tags](tagging.md) |
| Filespec | The filter on which files a window shows, by name, date and size. Set with `F`. [Filespec](filespec.md) |
| Treespec | A path typed to move the tree cursor. Opened with `\` or `/`. [Treespec](treespec.md) |
| Window | One tree with its file window. F8 opens a second one beside it. [Split screen](split.md) |
| Root | The directory at the top of the tree. `L` changes it, and Alt+L makes the directory under the cursor the root. |
| Matching | The logged files that pass the Filespec. |
| Mask | A pattern that builds one name or one line per file: `*.bak`, `photo_<001>.*`. Used by Rename and Batch. [Rename](rename.md) |
| Batch | Ctrl+B: writes a file with one line per tagged file, or per record of a table in the viewer. [Batch](execute.md) |
| Prune | Alt+P: deletes a directory with everything in it. [Delete](delete.md) |
| Oops | Restores what was last sent to the trash. `O` in the tree, Ctrl+Z in both windows. |
| Yank | `Y`: copies a path to the clipboard. |
| Autoview | F7: the file window with the contents of the file under the cursor beside the list. |
| eXecute | `X`: runs a shell command, with `%f` for the file and `%t` for the tagged files. [eXecute](execute.md) |
| Avail | `A` in the tree: the mounted file systems with their free space. |

See also: [what Xylem is](overview.md), [the menus](menus.md).
