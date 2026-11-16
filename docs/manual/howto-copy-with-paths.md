# Copy files from several folders, keeping their paths

This copies every JPG under `~/projects` to `/mnt/backup`, each one in the
same subfolder it had. The same steps work with Move (Alt+M).

## Steps

1. In the tree, put the cursor on `projects`.
2. Press `B`. Xylem logs the branch and the file window lists the files of
   `projects` and of every folder below it. On a large branch the footer
   shows the progress, and Esc cancels.
3. Press `F`, type `*.jpg` and press Enter. The list now has only the JPGs.
4. Press Ctrl+T. Every file in the list is tagged (`■`).
5. Press Alt+C. The prompt asks for the destination.
6. Type `/mnt/backup`. Before pressing Enter, read the mode shown in the
   prompt and press **F7** until it says `Relative`.
7. Press Enter. If `/mnt/backup` does not exist, Xylem asks before creating
   it.

![The seven steps: Branch on projects, the filespec *.jpg, Ctrl+T, Alt+C, the destination with the Relative mode, and 21 files copied](../screenshots/howto-copy-paths.gif)

The band under the screen shows the step being run and its keys.

## Which path mode to pick

Step 6 decides where each path starts under the destination. For
`~/projects/photos/2025/trip/DSC_01.jpg`, with the Branch opened on `projects`:

| Mode | The file is written to |
|---|---|
| Full | `/mnt/backup/home/user/projects/photos/2025/trip/DSC_01.jpg` |
| Tree | `/mnt/backup/` and the path from the root of Xylem's tree |
| Current | `/mnt/backup/projects/photos/2025/trip/DSC_01.jpg` |
| Relative | `/mnt/backup/photos/2025/trip/DSC_01.jpg` |

The mode the prompt starts in is `Full`, unless `copy_paths` in the
configuration says otherwise.

## Variations

- Without the paths: Ctrl+C instead of Alt+C puts all the JPGs directly in
  `/mnt/backup`. Two files with the same name then collide, and Xylem asks
  what to do with each one.
- By date or size: at step 3, `*.jpg,>TODAY-30` keeps the JPGs changed in
  the last 30 days. [Filespec](filespec.md) has the syntax.
- From the whole disk: log more with `*`, then press `S` (Showall) at step 2.
- Picking by hand: skip step 4 and tag each file with `T`.

## When it does not do what you expect

- **Alt+C starts a Compare.** The focus is on the tree. Alt+C copies with
  paths only in the file window; press Enter to go there.
- **Files are missing from the list.** A Filespec set earlier is still in
  effect. It shows in the title of the file window; `F`, `*`, Enter clears it.
- **The folders in the destination start at `/home`.** The mode was `Full`.
  See step 6.

Ctrl+U untags the list, and `B` goes back to the files of the directory
alone.

See also: [Copy and Move](copy-move.md), [tags](tagging.md),
[the tree window](tree.md).
