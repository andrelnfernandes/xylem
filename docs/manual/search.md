# Searching the tagged files

## Ctrl+S

Searches all the tagged files of the list and **untags those that do not
match**, with progress and `Esc` to stop. It also works inside
[archives](archives.md).

At the prompt, **F2** toggles case sensitivity (ignoring case, accents count
too: `AÇÃO` finds `ação`) and **F4** cycles through three modes:

- **text+Unicode** (the default): each term is looked for at once in UTF-8,
  UTF-16 (little and big endian, with or without a BOM) and Windows-1252, in
  one reading of each file. A file saved by Notepad in UTF-16 is found by
  typing the word as usual.
- **text**: UTF-8 only, line by line, as W's `contains()` reads it.
- **hex**: byte pairs such as `4d 5a`.

### Terms, AND, OR and NOT

Several terms can be joined, as in the `contains()` of [W](sql.md):

| Typed | Keeps the files that have |
|---|---|
| `invoice AND 2026` | both |
| `jpg OR png` | either |
| `invoice AND NOT draft` | the first and not the second |
| `(jpg OR png) AND NOT thumb` | parentheses group |
| `"tax invoice"` | the text in quotes, spaces and all |
| `x'4D5A' AND x'504B'` | those bytes, as SQL's binary literals |

`invoice AND 2026` here keeps the same files as
`contains('invoice') AND contains('2026')` in W. NOT goes before AND, and AND
before OR. The operators are written in capitals: `salt and pepper` is one
term, the three words as typed, so a search from before keeps its meaning.
Words with nothing between them are one term; a term with parentheses or a
`"` goes in quotes (`""` inside quotes is one quote). Up to 16 terms. The
search stops reading a file as soon as it is decided: an AND that has failed,
an OR that has found its term.

### What text+Unicode leaves to text

- A `*` in the middle of a term matches anything on the same line
  (`error*disk`), and such a term never crosses a line break. In
  text+Unicode, a term with a `*` is looked for as text only (UTF-8), and the
  message at the end says so.
- Ignoring case, letters with one upper and one lower form are found in either
  case in every encoding (Latin, Greek, Cyrillic). A letter like `ß`, whose
  upper case is two letters, is looked for as typed, and the message says so.
- A UTF-16 hit counts only at an even offset of the file, and a Windows-1252
  hit only when it is not a piece of a UTF-8 letter (`Ã` is the first byte of
  `é` in UTF-8), so a term is not found by accident. A term in a script whose
  UTF-16 bytes look like ASCII (Chinese, for instance) can still match plain
  ASCII text: **F4** to text, then, for UTF-8 only.

## Ctrl+V

Opens the tagged files in the [viewer](viewer.md), one after another: **N**ext
and **P**rev switch files (Alt+Home and Alt+End: first and last) and **U**
untags the current one. The term is shared with Ctrl+S: after a Ctrl+S,
**Space** in Ctrl+V goes straight to the next match, moving on to the next
tagged file when the file ends.

A common round: tag files (`T`, or Ctrl+T in a Branch), Ctrl+S a term, Ctrl+V,
and Space to walk the matches across the files.
