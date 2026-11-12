# Bookmarks

Bookmarks take you back to directories you use often, with one key each
(`0`–`9`, `a`–`z`). They are saved between runs, with the prompt history.

- **Ctrl+F11** (or Ctrl+B in the tree) saves the current directory: press
  the key to give it.
- **`'`** jumps: press the bookmark's key. At the same prompt, `'` or Enter
  opens the list and `+` saves the current directory.
- **F11** (or Alt+B) opens the bookmarks manager: ↑ ↓ choose, Enter (or the
  bookmark's key) jumps, **F2** gives it a name, Del removes, Insert or `+`
  saves the current directory, Esc closes.
- **F11 in the [Log](treespec.md), [Copy and Move](copy-move.md) prompts**
  opens the same list to choose a place: Enter (or the bookmark's key) puts it
  in the prompt, where you still confirm it; Esc goes back to what you typed.

A directory outside the tree opens as a new tree, as with [Log](treespec.md).
In the file window, Ctrl+B is [Batch](execute.md); there, use Ctrl+F11.

## Names

A name says what a bookmark is for: "Oracle server", "Postgres container,
env 1". In the manager, **F2** on a bookmark asks for it, with the current
one to edit; an empty name takes it away. The name shows before the place,
and the place always shows next to it, so a name cannot hide where a
bookmark goes. A name has at most 60 characters, with no control or text
direction characters.

## Remote machines and containers

In a window on a [remote machine](remote.md) or in a container, Ctrl+F11
saves the folder by the address that opens it: `ssh://oracle@db01/u01/app`,
`docker://postgres-env1/var/lib/postgresql`. Going to it connects as typing
the address in Log does, asking for a password if ssh needs one; no password
is saved. If the window is already on that machine and the folder is in its
tree, only the cursor moves. A folder whose path has a space cannot be saved
this way, since an address cannot hold one.

The manager marks `[not found]` a local folder that is gone. It does not
check remote bookmarks, nor folders on a cloud mount or network share: that
would wait for the network.
