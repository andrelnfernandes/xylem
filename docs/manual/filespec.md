# Filespec: the file filter

`F` sets which files show in the file window (each window has its own). The
current filespec appears in the window's title. Specifications are separated
by spaces, `;` or `,`, of four kinds:

- **name:** a glob (`*.rs`, `a?c*`, `[abc]*`), case-insensitive; in quotes, a
  name can contain spaces (`"my file*"`);
- **regular expression:** between slashes, for the name
  (`/^report-\d{4}/`); case counts, unless an `i` follows (`/\.jpe?g$/i`).
  Everything up to the closing slash is the expression, spaces and commas
  included (`/^\d{2,4} /`); a slash inside it is written `\/`. Expressions
  combine with globs as another name: `*.rs /^Cargo\./` shows both;
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

An expression runs in time proportional to the name, whatever it is, and
one too big to compile is refused; an invalid one is refused with the
column where it goes wrong.

`*` (or an empty prompt) shows everything. **↑** lists earlier filespecs ([history](execute.md#history)). The history keeps what you typed.

To pick files by a SQL condition (**W**) or by the words in them (**P**), see [Finding files](sql.md). For every way of finding files and folders, see [Finding files and folders](finding.md).

See also: [the file window](files.md), [tags](tagging.md) (Ctrl+T in the tree
tags what matches the filespec).
