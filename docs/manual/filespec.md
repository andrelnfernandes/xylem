# Filespec: the file filter

`F` sets which files show in the file window (each window has its own). The
current filespec appears in the window's title. Specifications are separated
by spaces, `;` or `,`, of three kinds, as in ZTreeWin:

- **name:** a glob (`*.rs`, `a?c*`, `[abc]*`), case-insensitive; in quotes, a
  name can contain spaces (`"my file*"`);
- **date:** `<`, `=`, `>`, `<=`, `>=` or `<>` and a date in the order of the
  configured format, with `/`, `.` or `-` and a 2- or 4-digit year
  (`>01/05/2026`; `2026-05-01` always works), or `TODAY` and `TODAY-n`
  (`>TODAY-7`: the last 7 days);
- **size:** the same with an `s` after the operator (`>=s200000`); `K`, `M`
  and `G` also work (`>s100M`).

A leading `-` negates (`-*.bak`; `-<s1K` is the same as `>=s1K`). Within each
kind, positive specifications are ORed together, except `>a` immediately
followed by `<b`, which is the range between the two
(`>=01/05/2026,<=31/05/2026`: May); `<>` excludes. Between kinds, AND applies:
`*.txt,=TODAY,<s1000` are the `.txt` files from today smaller than 1000 bytes.

`*` (or an empty prompt) shows everything. **↑** lists earlier filespecs ([history](execute.md#history)).

## A SQL condition (W)

**W** takes a condition written as the `WHERE` of a SQL query, and the list then shows only the files that meet it, on top of the filespec. **Ctrl+W** looks only at the tagged files. It works in the directory, in Branch and in Showall, and only on what is logged: a file Xylem is not showing is never a result.

```sql
ext = 'jpg' AND size > 10000000
ext IN ('md', 'txt') AND age_days < 7
name LIKE 'report%' AND depth >= 2
```

Each file is a row with these columns:

| Column | What it is |
|---|---|
| `name` | the name, with its extension |
| `ext` | the extension, in lower case and without the dot (`''` when there is none) |
| `path`, `dir` | the full path of the file, and of its folder |
| `depth` | how many folders below the root of the tree the file's folder is |
| `size`, `disk` | the size in bytes, and the space it takes on the disk |
| `modified` | the date of the last change, local time, as `2026-11-15 09:30:00`: `modified >= '2026-01-01'` |
| `age_days` | days since that change (`age_days < 7`: the last week) |
| `mode` | the permissions as they are written: `mode = 644` |
| `kind` | `file`, `link` or `other` |
| `tagged` | whether the file is tagged: `NOT tagged` |
| `git` | the two-letter Git status (`' M'`, `'M '`, `'??'`, `'  '` if clean); `NULL` outside a repository or when Git is off |
| `type` | what the file is, as in Alt+F (`'PNG image, 800 x 600'`, `'Rust source'`, `'empty'`); read only when the condition needs it |
| `dup` | duplicate group number (1, 2...), if Alt+D has run; `NULL` if not a duplicate |

The operators are those of the [table filter](viewer.md#the-sql-filter): `=`, `<>`, `<`, `<=`, `>`, `>=`, `LIKE`, `ILIKE`, `IN`, `IS NULL`, `AND`, `OR`, `NOT` and parentheses. Numbers are written whole (`10000000`, not `10e6`). A name of a column that does not exist is refused with the list of those that do, and a condition that no file meets changes nothing.

The title of the file window shows the condition after the filespec, and the message says how many files met it. **Ctrl+T** then tags them all, and from there they can be copied, moved, deleted or sent to a [Batch](execute.md). **W** with an empty prompt shows the whole list again. The first time, **↑** lists a few examples to change.

### Inside the files: contains()

`contains('text')` is true for a file that has the text in it, in any case, as the [search](search.md) of Ctrl+S finds it (`*` inside the text stands for anything on the same line):

```sql
ext IN ('md', 'txt') AND contains('invoice')
contains('TODO') AND NOT contains('DONE')
```

`contains` always reads the files when the condition runs, and that takes time: an index keeps words, not pieces of them (for the index, see `words` below). Before reading, Xylem asks, saying how many files and how many bytes it is going to read: `No index: read the contents of 1,234 files (2.3 GB)?` (on a remote machine it adds that they are downloaded through SSH). Only the files whose answer depends on their contents are read: with `ext = 'md' AND contains('x')`, only the `.md` files. The footer shows how far the reading went, and **Esc** stops it: the list then shows what was found so far, and the message says it stopped.

Some files are not read, and the message at the end says how many of each were left out: binary files (a zero byte in their first 8 KiB), symbolic links (they are not followed), files over 256 MiB, and those that could not be read. A file left out is never a result, not even of `NOT contains(...)`.

On a cloud mount, `contains` is refused: each file would have to be downloaded. The list keeps what was found when it was read: a file changed or added later shows again only when **W** is run again.

`contains` goes alone, as a condition of its own joined by `AND`, `OR` and `NOT` (`contains('a') = 1` is refused), and takes one text in quotes.

### Whole words, and an index: words()

`words('invoice march')` is true for a file that has all those words, in any order and anywhere in it. A word is a run of letters and digits, compared without case and without accents: `words('acao')` finds `Ação`. Unlike `contains`, it does not find a piece of a word: `words('invoice')` does not find `invoices`. A `*` right after a word takes any word that starts with it: `words('invoice*')` finds `invoice` and `invoices`, and `words('andre*')` finds `André` and `Andrea`.

```sql
words('invoice 2024') AND ext IN ('txt', 'md', 'csv')
words('fatur* 2024')
```

`words` can be answered by an **index** of the branch, and then nothing is read. **Alt+N** in the tree (or in the file window, for the folder of the file) indexes the logged files of the branch, after asking: it reads them once, in the background, with **Esc** stopping it and keeping what was done. Doing it again reads only the files that changed, and drops those that are gone. On a branch that has an index, Alt+N offers to index it again or to delete the index.

|  | No index | With an index |
|---|---|---|
| Answer | each file is read when W runs | at once |
| Cost | time at every condition | time and space when indexing |
| Looks for | a piece of text (`contains`) or words and prefixes (`words`) | words and prefixes (`words`) |
| A file changed since | its contents now | read again, as with no index |
| Where | anywhere logged | only in the branches indexed with Alt+N |

Before reading, the question says both: `Index: 3,120 files up to date. No index: read the contents of 12 files (1.2 MiB)?`. When the index answers for every file, there is no question, and the message says `answered by the index`. The index never makes a condition wrong: a file changed after it was indexed (another size or date) is read again.

What the index leaves out: binary files, files over 64 MiB (W reads them when asked), symbolic links, and these folders and files, which are big and change all the time or hold secrets: `.git`, `target`, `node_modules`, `.ssh`, `.gnupg`, `.env`, `*.key`, `*.pem`, `id_rsa`, `id_ed25519`. The list of ignored patterns can be configured with `index_ignore` in `config.toml` or on the setup screen (F12, page 4, E). Removing secret patterns copies them into the index. Files on a remote machine, in an archive or on a cloud mount are not indexed.

The index is a copy of the words of the files, kept in Xylem's cache (`~/.cache/xylem/index`, in a folder only you can read; `%LOCALAPPDATA%\xylem\cache\index` on Windows) until you delete it with Alt+N, or delete that folder. An index that cannot be read (damaged, or of another version of Xylem) is said in the question, its files are read instead, and Alt+N makes it again.

## The search box (P)

A Google Desktop experience without SQL: press **P** (or **Ctrl+P** on tagged files) to open a floating search box.

As you type words, files matching them appear as you type:
- Words match whole words, and the last word uses prefix search while typing (`fatura mar` looks for `fatura mar*`).
- When a branch has an index (**Alt+N**), results from indexed files appear instantly.
- If there are unindexed files, the status line says how many unindexed files and bytes there are. They are only read with explicit confirmation (press **Alt+R**, or **Enter** when there are no indexed results yet).
- Each result shows its tag indicator (`*`), file name, directory, modification date, and a clean snippet (~80 characters, free of controls and bidirectional overrides) from the first line matching the query.
- Keys inside the search box:
  - **↑ / ↓**, **PgUp / PgDn**, **Ctrl+Home / Ctrl+End**: navigate the results list.
  - **Enter**: goes directly to the selected file in the file window and closes the search box.
  - **Ctrl+V**: opens the file in the viewer directly at the line of the snippet.
  - **Ctrl+T**: toggles the tag on the selected file directly in the search box.
  - **Alt+R**: asks and starts reading unindexed files.
  - **Esc**: cancels reading unindexed files, or closes the search box.

W is a first step of querying the disk: results as a table come later.

See also: [the file window](files.md), [tags](tagging.md) (Ctrl+T in the tree
tags what matches the filespec).
