# Find duplicates and keep one copy

This finds the files with the same contents under a directory and sends all
but one copy of each to the trash.

## Steps

1. In the tree, put the cursor on the directory and press `*` to log its
   branch.
2. Press Alt+D. Xylem compares the files and lists the groups of equal
   files, the group that frees the most space first.
3. Press `A`. In every group, all the copies but one are tagged (`T`).
4. Press `D`. Xylem asks before deleting. Press `T` to send the files to the
   trash, then `Y`.

![The four steps: logging the branch, Alt+D listing three groups, A tagging all but one copy, and D, T, Y sending four files to the trash](../screenshots/howto-duplicates.gif)

The footer shows `Trashed 4 of 4 files`.

## Choosing which copy stays

`A` tags all the copies but one in each group. Before step 4 you can change which one stays:

| Key | Effect |
|---|---|
| `A` again | changes which copy stays in every group: the oldest, the newest, the one with the shortest path |
| `K` | keeps the copy under the cursor and tags the others of its group |
| `F` | in every group that has a copy in this copy's folder, keeps that one |
| `T` or Space | tags or untags one copy |
| `U` | untags everything |

Xylem does not let you tag the last copy of a group.

## Whole folders

`G` switches the list to folders whose contents are all equal. The line at
the bottom of the list says when there are any: `1 group of folders (G)`.

## What the question means

With `delete_to_trash` off, which is the default, step 4 first asks `DELETE
4 tagged files? This cannot be undone.` `T` changes the question to `MOVE 4
tagged files to trash?`. `Y` then confirms what the question says.

## When it does not do what you expect

- **A file you know is repeated is not listed.** Files under 1 KiB, empty
  files, links, and the folders `.git` and `node_modules` are left out.
  [Duplicates](duplicates.md) says how to change that.
- **A group was not deleted.** A copy changed after the comparison. Nothing
  of that group is touched, and the footer says so. F3 compares again.

See also: [duplicates](duplicates.md), [delete](delete.md),
[finding what fills the disk](howto-disk-space.md).
