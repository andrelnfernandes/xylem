# Views (Alt+V)

**Alt+V** opens the views of the list: other ways to look at the files the
file window has, for a question the tree and the list do not answer well.
A small menu asks which one, each with its letter:

| Key | View | What it answers |
|---|---|---|
| `S` | [Scan](#scan) | What in these files deserves a look? |
| `T` | [Timeline](#timeline) | What changed, and when? |

A view works on **the current list**: the directory, the Branch or the
Showall, with the filespec and the `W` condition that are on. It only reads:
nothing in a view deletes, moves or changes a file. What you choose in it
goes back to the classic list as a filter, and from there Ctrl+T, Ctrl+C or
Batch act as always. **W with no text** takes that filter away, as it does
with a condition.

Each view can also have a key of its own in `[keys]`
([configuration](configuration.md)), with no key by default: `view_scan`,
`view_timeline`.

## Scan

The reports the [viewer](viewer.md) makes of files that may be hostile
(executables, Office documents, patches), one file at a time with `V`,
applied to a whole list: a ZIP you downloaded, a repository you cloned, a
pendrive. **Alt+V, S** (Branch first, with `B`, to take in the folders
below).

```
      1  Names with control or text-direction characters
      1  Names that deceive (fatura.pdf.exe, spaces, a trailing dot)
      3  setuid, setgid, writable by anyone
      2  Links that leave the folder, or point to nothing
      1  Executables: packed, without PIE or NX, writable and executable
      1  Office: macros, remote template, embedded objects, hidden text
      0  Patches: dangerous paths, hidden characters, hidden changes
      0  Not analysed: over a limit, or the reader could not take it
```

| Category | What it looks at |
|---|---|
| Names with control or text-direction characters | The name of each file and folder: control characters, the ones that turn the text around (a name drawn as `exe.pdf` that is `fdp.exe`), bytes that are not UTF-8 |
| Names that deceive | A document's extension before one that runs (`fatura.pdf.exe`, `photo.jpg.scr`), spaces before the extension, a name that ends in a space or a dot |
| setuid, setgid, writable by anyone | The mode of each file, and of each folder: a folder anyone can write in counts only without the sticky bit, which `/tmp` has |
| Links that leave the folder, or point to nothing | Where each link points, by its text: the link is never followed to read what is there |
| Executables | ELF, PE and Mach-O, told by their first bytes and not by the name: no PIE, no NX or DEP, no ASLR, an executable stack, sections packed (UPX and others) or both writable and executable, data after the last section |
| Office | Word, PowerPoint and OpenDocument: macros, what the document fetches from outside when it opens (a remote template), embedded objects, hidden text and hidden slides |
| Patches | `.patch` and `.diff`: files that climb out with `..`, are absolute, go into `.git` or become executable; names with hidden characters; hunks whose counts do not match; lines changed only in what cannot be seen |
| Not analysed | The files the readers could not take, each with why: over a limit, a broken package, a reader that failed |

The names and the files' modes are already in memory, and show at once.
The folders' modes, the links and the contents ask the disk, in the
background: the screen fills in as they come, says how many are left, and
**Esc** stops it.

The contents go through the same readers as the reports of the
[viewer](viewer.md) (`V`), with the same limits, one file at a time. A file
over a limit, or one a reader could not take, goes into **Not analysed**
with why: it never drops out in silence. At most `scan_max_files` files
(10000) have their contents read; past that, they are listed as not
analysed ([configuration](configuration.md)).

| Key | What it does |
|---|---|
| ↑ ↓ | move |
| Enter | the files of the category, each with why it is there |
| Enter, on a file | back to the file window, which keeps only the files of that category, with the cursor on this one |
| Enter, on a folder | the folder in the tree |
| `V`, on a file | view it, with its report; Esc comes back to the Scan |
| `R` | scan again |
| Esc | back to the categories, then to the list |

Esc on the list keeps the Scan: Alt+V, S on the same list opens it as it
was, without reading again. `R` reads again.

### Where it does not look

- **Windows drives and FAT, exFAT, NTFS, SMB, rclone mounts.** There the
  driver gives every file the same mode, so modes say nothing, and the
  Scan does not count them: the category says so instead. Under WSL this is
  `/mnt/c` and the other Windows drives.
- **Inside an archive** (Alt+F5) the files' modes are the ones the archive
  records, and they count: a setuid file in a TAR is worth knowing about.
  Each member is extracted with a limit (64 MiB) and read as on the disk.
  Folders and links are not looked at. In a TAR, a member far inside is not
  extracted, and is listed as not analysed.
- **On a cloud mount** (rclone) the contents are not read: each file would
  be downloaded. The names and the rest are.
- **On a remote machine** only the names and the files' modes are looked at,
  for now: reading the contents would bring each file through SSH.

The Scan is not an antivirus. It points at what deserves a look, and does
not say the rest is safe.

## Timeline

The files of the list in a histogram by the date they were changed: when a
folder was busy, what came in last week, which photos are from that trip.
It reads nothing from the disk: only the dates already in memory.

```
 2026-11-18: 312 files, 1.4 GiB

                                    █
                                    █▆
 ▁▁▂▁▁▁▁▁▃▁▁▂▁▁▁▁▁                  ██▁
                                    ▲
 11-01     11-05     11-09     11-13     11-17
```

The scale is chosen from the dates: hours, days, weeks, months or years.
Dates before 1980 and in the future go into a bar of their own at each end,
because wrong or hostile dates exist and would otherwise flatten the chart.
Files with no date are counted below it.

| Key | What it does |
|---|---|
| ← → PgUp PgDn Home End | move from bar to bar; the top line says the files and bytes of the bar |
| Shift+← → | choose a range of bars |
| `+` `-` | a finer or coarser scale (`+` from days to hours) |
| `B` | the height of the bars by bytes instead of files |
| Enter | back to the file window with only the files of the bar or range |
| Esc | back, changing nothing |

Enter turns the choice into a [W condition](sql.md) on `modified`, such as
`modified >= '2026-11-18 00:00:00' AND modified < '2026-11-19 00:00:00'`,
joined with AND to the condition that was already on. The title of the file
window shows it, and **W with no text** takes it away.
