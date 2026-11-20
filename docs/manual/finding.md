# Finding files and folders

Xylem has several ways of finding things. Each answers a different question,
and they fall into three groups: going to one place, filtering the list, and
looking inside the files.

| You want | Key | What you get |
|---|---|---|
| *That* folder, by pieces of its name | `\|` in the tree | the tree cursor goes there ([Fuzzy find](fuzzy.md)) |
| *That* file of the list, by pieces of its name | `\|` in the file window | the file cursor goes there ([Fuzzy find](fuzzy.md)) |
| A path you know, level by level | `\` or `/` | the tree cursor follows what you type ([Treespec](treespec.md)) |
| The next name starting with a letter | Shift+letter | the cursor goes there ([The tree window](tree.md)) |
| The files with this name, date or size | `F` | the list shows only them, until changed ([Filespec](filespec.md)) |
| The files that meet a condition | `W` | the list shows only them, until changed ([SQL](sql.md)) |
| The files with these words in them | `P` | a list of results, the best first ([The search box](sql.md#the-search-box-p)) |
| The tagged files with this text in them | Ctrl+S | the others are untagged ([Searching tagged files](search.md)) |

## Going to one place

**Fuzzy find** (`|`), **Treespec** (`\`, `/`) and **Shift+letter** move the
cursor, and change nothing else. Use `|` when you remember a piece of the name
but not where it is; Treespec when you know the path; Shift+letter to jump
within the folder you are in.

## Filtering the list

**Filespec** (`F`) and **W** decide which files the file window shows, and the
filter stays until you change it: the title of the window shows it. `F` is for
names, dates and sizes; `W` for anything Xylem knows of a file, its contents
included (`contains`, `words`). They apply together, and Ctrl+T then tags all
the files that passed.

To look in the whole tree, open Branch (`B`) or Showall (`S`) first: the
filters, and `|`, then work on every logged file of it.

## Looking inside the files

**The search box** (`P`) finds files by the words in them, as you type, and
is instant on a branch indexed with Alt+N. **Ctrl+S** searches the tagged
files for a piece of text and untags those without it, and Ctrl+V then shows
each match. In `W`, `contains('text')` and `words('a b')` do the same as part
of a condition.

## Tagging what was found

Every one of these ends in tags when you want to act on many files:

- `|` and `P`: Ctrl+T tags the chosen result; in `|`, Alt+T tags the list.
- `F` and `W`: Ctrl+T in the tree tags every file that passed.
- Ctrl+S keeps tagged only the files that have the text.

Then Ctrl+C, Ctrl+M, Ctrl+D and Batch act on the tagged files. See
[Tags](tagging.md).
