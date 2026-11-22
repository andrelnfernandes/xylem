# Rename and batch rename

`R` renames the file (in the tree, the directory) under the cursor: type the
new name. A name that already exists is refused.

## Batch rename (Ctrl+R)

Renames all the tagged files with a mask, numbered in the order of the list:

- **Wildcards:** `*.jpg` (changes the extension), `new_*.*`, `*_old.*`, `???.*`
  (keeps that many characters). `*.*` keeps names as they are; `*.` removes
  the extension.
- **Search and replace:** `s/foo/bar/` replaces every `foo` with `bar`;
  `s/foo/bar/i` ignores case. Without the `r` flag the text is literal:
  `s/./_/` replaces dots.
- **Regular expressions:** the `r` flag makes the first part a regular
  expression and replaces every match: `s/(\d+)-(\w+)/$2-$1/r` turns
  `2026-report.txt` into `report-2026.txt`. In the replacement, `$1` or
  `${1}` is a group, `$0` the whole match, `${name}` a named group
  (`(?<name>...)`), and `$$` a dollar sign. `\U` and `\L` turn what follows
  into upper or lower case until `\E`; `\u` and `\l` only the next
  character: `s/^(\w+)/\U$1/r` gives `PHOTO.jpg`. `ri` also ignores case.
  The slashes separate the parts, so neither part can hold a `/`.
  An invalid expression, or a group it does not have, is refused before
  anything is renamed, with the column where the expression goes wrong.
- **Counters:** `%d`, `%02d`, `%03d`, `%04d`, `<#>`, `<01>`, `<001>`, `<0001>`
  number the files (1, 01, 001...): `photo_<001>.*` gives `photo_001.jpg`,
  `photo_002.jpg`...
- **Case:** `<lower>`, `<upper>`, `<title>` (also as the name or extension
  part: `*.<lower>`), or **F2** at the prompt to cycle lowercase, UPPERCASE and
  Title Case over the result.

If the mask would give two files the same name (or one file the name of
another tagged file that keeps its own), nothing is renamed, and the
message names both files. A result that is empty or not a valid name is
refused too. When a new name belongs to a file that is still to be renamed,
Xylem asks before overwriting it, as in [Copy and Move](copy-move.md).

See also: [tags](tagging.md).
