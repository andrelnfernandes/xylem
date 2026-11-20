# Fuzzy find (|)

`|` finds a folder, or a file of the list, by pieces of its name, at any
level of the tree: `cfgxy` finds `config/xylem`, `rdme` finds `README.md`. The
results appear above the prompt as you type, the best first, and the cursor of
the window follows the one chosen.

To go to a path you already know, level by level, [Treespec](treespec.md) is
quicker. To pick many files by name, date or size, or by what is in them, see
[Finding files and folders](finding.md).

## What it searches

What `|` searches depends on the window it is opened in:

- **In the tree:** the logged folders, by their path from the root of the tree
  (`docs plan` finds `docs/plans`). Hidden folders, and what is inside them,
  are searched only while the tree shows them.
- **In the file window:** the files of the list as it is, with the
  [filespec](filespec.md) and the [SQL condition](sql.md) (`W`) in effect. In
  the directory, by the name; in Branch and Showall, also by the path
  (`src fzrs` finds `src/fuzzy.rs`).

To find a file anywhere in the tree, open Showall (`S`) and then `|`.

Only what is logged is searched, and nothing is read from the disk or from
the network: it works the same on a [remote machine](remote.md), inside an
[archive](archives.md) and on a [cloud mount](cloud.md).

## How it matches

- **The letters in order**, not necessarily together: `rdme` finds `README.md`.
- **Spaces separate terms**, and every term has to match, in any order:
  `xy docs` finds `xylem/docs` and `docs/xylem`.
- **Case** does not count, unless the term has a capital letter: `Rd` does not
  find `readme`. Each term decides for itself.
- **The best first:** letters together count more than letters apart; a letter
  at the start of a word (at the beginning, after `/`, `_`, `-`, `.` or a space,
  or where lower case turns to upper case, as in `fileSpec`) counts more than
  one in the middle; a match in the name counts more than one in the folders
  above it. Among equals, the shorter path comes first, and then the order of
  the tree or the list.

The letters that matched are shown in the key color. A name too long for the
window is cut on the left, with `…`, so that the end of it stays in view.

## On the screen

The count at the top right says how the search is going:

| It says | Meaning |
|---|---|
| `type pieces of a name` | the prompt is empty |
| `120 found · searching 40%` | the search is still going through the tree; the list fills in |
| `120` | done: every match is in the list |
| `500 best of 15,894` | done: the list keeps the 500 best |
| `nothing matches` | done, with no match |

A big tree is searched in steps while you type, so the first results come at
once and the rest follow. When you add letters at the end, the search looks
only among what already matched. If the tree changes while the prompt is open
(a folder logged, released or read again), the search starts over.

A `*` before a file means it is tagged; after a folder, `3*` says how many of
its files are tagged.

## Keys

| Key | What it does |
|---|---|
| ↑ ↓ | choose the previous or next result; the cursor of the window follows |
| PgUp PgDn, Home End | choose a page up or down, the first or the last result |
| Enter | stays where the cursor is and closes the prompt |
| Esc | goes back to where the cursor was, and closes the prompt |
| Ctrl+T | tags the chosen result, or untags it if it is tagged, and chooses the next one |
| Alt+T | tags the files of the results in the list |
| Ctrl+U | clears the prompt |

With the [mouse](mouse.md) on, a click chooses a result and a double click
goes there.

## Tagging from the results

**Ctrl+T** on a file tags it; on a folder, it tags the files of that folder
that the file window would show (with the filespec and `W` in effect). If they
are all tagged already, it untags them. The choice then moves to the next
result, so that Ctrl+T, Ctrl+T, Ctrl+T tags the results one after another.

**Alt+T** tags the files of every result in the list: at most the 500 that
are shown. When more matched, the note under the list says how many were left
out. Alt+T on a folder that has not been logged tags nothing there, and the
note says how many were skipped.

The tags stay after the prompt closes, as any tag: Ctrl+C, Ctrl+M, Ctrl+D and
Batch then act on them. See [Tags](tagging.md).

Tagging every match, not only the 500 in the list, is an option, off by
default: Alt+T could then tag thousands of files you have not seen. Read
[Tagging every match of the fuzzy find](configuration.md#tagging-every-match-of-the-fuzzy-find)
before turning it on.
