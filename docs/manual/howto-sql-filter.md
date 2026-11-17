# Pick files with a SQL condition

This lists the JPG files larger than 1 MB anywhere under `~/projects` and
tags them, ready for Copy, Move or Delete.

## Steps

1. In the tree, put the cursor on `projects` and press `B`. The file window
   lists the files of the whole branch.
2. Press `W`. The prompt asks for a condition.
3. Type `ext = 'jpg' AND size > 1000000` and press Enter. The list keeps the
   files that meet the condition, and the footer says how many.
4. Press Ctrl+T. The files in the list are tagged.

![The four steps: Branch on projects, W, the condition typed, and the 12 files it keeps tagged with Ctrl+T](../screenshots/howto-sql-filter.gif)

The title of the file window shows the condition: `Branch: * where ext =
'jpg' AND size > 1000000 (12)`. `W` with an empty prompt shows the whole list
again; the tags stay.

## Writing the condition

The condition is what follows `WHERE` in a SQL query. Text goes in single
quotes, and numbers are written whole.

| To keep | Condition |
|---|---|
| files changed in the last week | `age_days < 7` |
| Markdown and text files | `ext IN ('md', 'txt')` |
| names that start with `report` | `name LIKE 'report%'` |
| files not tagged yet | `NOT tagged` |
| files that contain a text | `contains('invoice')` |

[A SQL condition](sql.md#a-sql-condition-w) lists the columns.

## When it does not do what you expect

- **`No column "jpg"`.** The text was typed without quotes. `ext = jpg`
  compares two columns; `ext = 'jpg'` compares with the text.
- **Files you expected are missing.** `W` looks only at logged files. Open
  the Branch first, or log more with `*` and use `S` (Showall).
- **Nothing changes.** No file meets the condition, and the list stays as
  it was.
- **`contains` asks before running.** It reads every file it has to check,
  and the question says how many files and bytes.

See also: [Filespec](filespec.md), [tags](tagging.md),
[copying with paths](howto-copy-with-paths.md).
