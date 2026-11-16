# Find what fills the disk

This goes from the home directory down to the largest files in it and
deletes one of them.

## Steps

1. In the tree, put the cursor on the directory to examine and press Alt+U.
   Xylem logs the branch and lists its folders and files by the space they
   take, largest first.
2. Move to a folder (its name starts with `/`) and press Enter to list what
   is in it.
3. Press Enter again on the largest folder there, and repeat until the list
   shows files.
4. Move to the file to delete and press Del. Xylem asks before deleting.
5. Press `T` to send it to the trash, then `Y`.

![The five steps: Alt+U on the home directory, Enter on projects, Enter on backup, Del on the older archive, and T, Y sending it to the trash](../screenshots/howto-disk-space.gif)

The footer shows `Trashed 1 of 1 files`, and the percentages are computed
again without the file.

## Reading a line

```
     19.0 MiB ████████████████    51%   projects-2026-10.tar.gz
```

| Part | Meaning |
|---|---|
| `19.0 MiB` | the space the item takes on the disk |
| the bar | the item against the largest item of the list |
| `51%` | the item's share of the directory listed |
| the name | a folder when it starts with `/` |

## Keys in the list

| Key | What it does |
|---|---|
| Enter | goes into the folder; on a file, views it |
| Backspace or ← | goes back to the folder above |
| `T` | tags the file, or every file of the folder |
| Ctrl+D | deletes the tagged files |
| Alt+P | deletes the folder with everything in it, permanently |
| `D` | looks for duplicates in the folder listed |
| `\` | closes the list and puts the tree cursor on the folder listed |
| Esc | closes the list |

## When it does not do what you expect

- **A line starts with `>`.** It is another disk mounted there. It is not
  counted, and Enter on it starts a separate list for that disk.
- **A line starts with `!` or `.`.** The folder, or something inside it,
  could not be read, usually for lack of permission. It may take more space
  than shown.
- **The sizes differ from the file window.** The list shows the space on the
  disk, in whole blocks. `A` switches to the files' own sizes.

## A cloud drive

The same steps work on a Google Drive or S3 folder mounted by rclone.
[Cloud storage](cloud.md) describes the setup and what differs there.

See also: [disk usage](usage.md),
[finding duplicates](howto-duplicates.md).
