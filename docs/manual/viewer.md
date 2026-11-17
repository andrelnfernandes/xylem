# The viewer

`V` opens the file in the viewer (Autoview, F7, shows it next to the list).
Binary files open in hex. A big file on disk is not read whole: the viewer reads what the screen shows as it moves (see [Big files](#big-files)). What cannot be read that way shows only its beginning, up to the size limit.
Spreadsheets, CSV, DBF, SQLite and Parquet open as a table (see Table mode).

| Key | What it does |
|---|---|
| ↑ ↓ PgUp PgDn Home End | scroll |
| ← → | scroll sideways |
| `H` | next mode: text and hex; in a Markdown file, rendered, text and hex |
| `O` | next encoding (UTF-8, CP850, 1252, CP437, UTF-16) |
| `C` | syntax colors on or off |
| `W` | soft wrap on or off |
| `#` or `L` | line numbers on or off (in text mode) |
| `g` or `:` | jump to a line (a byte offset in hex, a row in table mode) |
| `&` | show only the lines that match, plain text or regex ([filtering lines](filter.md)); in table mode, the SQL filter |
| Ctrl+B | a line of output per record or line shown ([Batch in the viewer](execute.md)) |
| `E` | edit the file; back in Xylem, the viewer shows the new contents at the same place (binary files only with an editor in [`[editors]`](execute.md)) |
| `D` | with Git on: what changed since the last commit, and back ([Git](git.md)) |
| `T` | table or text, in a file that opened as a table; rendered or source, in a Markdown file |
| `U` | in the view of the tagged files (Ctrl+V): untag the file |
| `y` or Ctrl+Insert | copy the file's path to the clipboard |
| Esc | close |

## Searching

As in ZTreeWin: **F** searches from the start, **B** backwards from the end,
**/** (or `S`) from the current page and **\\** backwards from it; **Space**
continues in the same direction, **+** and **-** choose the direction. The
match is highlighted. At the search prompt, **F2** toggles case sensitivity
(ignoring case, accents count too: `AÇÃO` finds `ação`) and **F4** switches
between text and hex (byte pairs such as `4d 5a`). A `*` in the middle of the
term matches anything on the same line (`error*disk`).

## Syntax colors

Source code and scripts are colored in about 150 languages (Rust, Python,
shell, C, JavaScript, SQL, YAML, Markdown...), recognized by extension, by name
(`Makefile`) or by the first line (`#!/bin/bash`). The default is
`syntax_highlight` (page 2 of the [configuration](configuration.md)). Colors
stay readable on the current theme's background, and only the first 20,000
lines are colored, and no line longer than 10,000 characters, so huge files
never slow the viewer down.

## Rendered Markdown

`.md` and `.markdown` files open rendered, also in Autoview, inside archives and compressed (`x.md.gz`), and the frame says `RENDERED`. `T` shows the source at the same place, and comes back; `H` goes through rendered, text and hex.

You see headings, bold, italic, strikethrough, code, code blocks with syntax colors (`C` turns them off), nested and numbered lists, task lists (`[x]`), quotes, rules and tables. Nothing is ever fetched, and no link opens anything:

- A link shows its real address beside its text, as in `safe <https://example.com>`, so the text cannot pass for another address.
- An image is a line of text: `[image] caption <photo.png>`.
- A Mermaid diagram is a code block labelled `mermaid`.
- Raw HTML and HTML comments stay visible as text.
- The YAML block at the start of a file is a code block labelled `yaml`.

The text always fits the window, so `W` does nothing here; ← → reach the columns of a table too wide for it. A search looks at the text on the screen, addresses included, and finds a phrase that wraps onto the next row. `g` and `#` count the lines of the source. Ctrl+B works on the source's lines, all of them. `&` goes to text mode and filters there.

A file over 8 MiB opens as its source, with a note. Rendering also stops at the time limit (3 s, 100 ms in Autoview) or at its memory limit, and the header then says `(rendered only the start)`. A table shows at most 256 columns. Control characters show as `·`, including those written as entities like `&#27;`.

Markdown itself hides a few things: link definitions that nothing uses, and table cells beyond the columns of the header. To see everything, `T` shows the source. Italic and strikethrough depend on the terminal and its font, so they also have a color of their own.

## Logs

A log opens as text, with the colors of its levels, and the footer says `Log`. A file is a log by its name (`.log`, `.out`, `.err`, a rotated one like `app.log.1` or `app.log.2.gz`, `syslog`, `messages`, `kern.log`) or by its contents: most of its first lines start with a date and time (`2026-11-16 10:00`, `[2026-11-16 ...`, or syslog's `Nov 16 10:00:00`).

- **Levels:** the first level word in the first 120 columns of a line, in any case, gives its color: errors (`ERROR`, `ERR`, `FATAL`, `CRIT`, `PANIC`...) in red, warnings (`WARN`, `WARNING`) in yellow, `DEBUG` and `TRACE` dimmed. Syslog's `<n>` at the start of a line counts too, and so does the level of a log in JSON (`"level":"error"`). A line with no level, like a stack trace, takes the one before it. `C` turns the colors off.
- **ANSI colors:** the colors a program wrote into its log (`ESC [31m`) are shown as colors, and their sequences are taken out of the text. Any other sequence is dropped, since it could move the cursor or write to the clipboard (OSC 52), and the header counts them, as in `[ANSI: 3 removed]`. `H` still shows the bytes as they are, and a search looks at the text without the sequences. In a file over 16 MiB, read through a window, the sequences are shown as `·`, as in any text, and the colors are those of the levels.
- **Following:** with the last line on the screen (End), new lines come in as the program writes them, and the screen stays at the end; the header says `[FOLLOW]`. The file is looked at every half second, also on a network share. Moving up, or searching, stops it; End takes it up again. With a filter (`&`), the new lines go through it. If the file shrank or changed instead of growing (rotated or rewritten), the header says `(the file changed: R reloads)`, and nothing is read until R. A compressed log is not followed.

## Patches

A `.patch` or `.diff`, or any text that starts like one (`diff --git`, or `---`, `+++` and `@@`), opens as text with the colors of a patch. The Git diff (`D`) is shown the same way. The header sums it up, as in `[12 files, +340 −98]`, and the footer says where the top of the screen is: `3/12 src/viewer.rs @@ 2/5`.

| Key | Action |
|---|---|
| `]` `[` | next or previous file |
| `}` `{` | next or previous hunk |
| `Tab` | the list of the files, with `+` and `−` for each; letters narrow it, `Enter` goes to one |

Some changes do more than change lines, and are easy to miss in a quick read. They are shown in red, and the header counts them as `to check`:

- a file that becomes executable (`100755`), a symbolic link (`120000`) or a submodule (`160000`);
- a path that is absolute, climbs out with `..`, goes into `.git` (a hook is code that Git runs), or has control characters.

The header also counts:

- `look unchanged`: a removed line and an added one that differ only in what cannot be seen, such as spaces, `\r`, invisible characters, or letters that look alike (a Cyrillic `а` for a Latin `a`). The pair is shown in yellow;
- `hunks with wrong counts`: a hunk whose `@@` line does not match its lines. The index follows the lines that are there, never the counts;
- `whitespace`: added lines with a space at the end, or a space before a tab in the indentation, as `git diff --check` reports.

A `git format-patch` series shows each message in the list, by its subject. Nothing is ever applied. In a file over 16 MiB, read through a window, the colors stay, but there is no index: `]`, `}` and Tab say so.

## Text direction controls

Some invisible characters change the direction of the text and can make code look different from what it really says (Trojan Source, CVE-2021-42574). The viewer never hides them: each one appears as a red `·`, and the header says how many there are.

## Text encoding

Xylem finds the encoding by itself, in this order: a decompressed file first, then a BOM (UTF-8, UTF-16 LE or BE), then UTF-16 without a BOM, then UTF-8. A file that is none of these is old text: Xylem guesses CP850 (DOS, Western Europe) or Windows-1252 by counting accented letters. Windows `.reg` exports are UTF-16 and open as plain text.

**O** shows the file in the next encoding: UTF-8, CP850, Windows-1252, CP437, UTF-16 LE, UTF-16 BE, and back to UTF-8. The footer says which one, and the header shows its name when it is not UTF-8. If you pick UTF-16 on a file that opened in hex, it switches to text. Your choice stays after you edit the file and come back.

To always use one encoding for old text files instead of the guess, set `legacy_charset` to `cp850`, `cp1252` or `cp437` (**K** on page 2 of the [configuration](configuration.md)). The default is `auto`.

## Big files

With **V**, a file on disk bigger than 16 MiB is not loaded: the viewer keeps the lines near the screen, about 2,000 of them, and reads the others from the file as it moves, without holding the screen. A line not read yet shows as `…` for a moment. The file has no size limit this way, and the memory used does not grow with it.

- **The lines are counted in the background.** The footer shows `Line 1 of 1,234,567+ (counting…)` and then the total. **End** and **g** go to any line already counted. Esc stops the count, and what was counted stays.
- **The search** (`F`, `B`, `/`, `\`, Space) goes over the file in the background: the footer shows `Searching... line 1,200,000 (Esc stops)`, and the screen goes to the match when it is found.
- **The line filter** (`&`) goes over the whole file in the background, and the lines appear as they are found, up to 1,000,000 of them. Esc stops it and keeps what was found.
- **Hex** (`H`) reads the bytes where the screen is, and its search goes over the file too.
- **Another encoding** (`O`) reads the lines again; with UTF-16 they are counted again, since a line ends elsewhere.
- **No syntax colors and no rendered Markdown**: both read a text from its start.
- **A line longer than 1 MiB** is cut on screen, ending in `…`; the hex shows all of it.
- **If the file changes** while it is open, the header says `(the file changed: R reloads)`, and **R** reads it again and goes back to the line it was at.
- **A Batch** (`Ctrl+B`) over the text still reads the file into memory, up to the size limit below, and says so when that is not all of it.

To reach a line the viewer notes where every 256th line starts, about 8 bytes for every 256 lines, only in memory. The text direction controls are counted over the whole file as the lines are (in UTF-8), and the header shows how many were found so far.

A compressed file, an entry of an archive, a file on a remote machine and Autoview are not read this way: there is no jumping inside them, so they follow the size limit below.

## Size limit

For those, and for the files up to 16 MiB, the viewer reads only the beginning of a file too big, and the header says `(showing first N MiB)`. How much it reads is the `viewer_max_size` option (**L** on page 2 of the [configuration](configuration.md)): `auto`, `16`, `64`, `256` or `1024` (MiB). With `auto`, the default, the limit is 1/32 of the machine's RAM, never less than 16 MiB and never more than 256 MiB (it is 16 MiB when the RAM cannot be read). The configuration screen shows the value in use, for example `auto (256 MiB)`.

Two cases do not follow it. Autoview always reads 1 MiB. A file on a remote machine (SFTP) is read at most up to 16 MiB, because Esc cannot cancel that read.

A bigger limit costs memory and time: with `1024`, a compressed text that unpacks to that much takes roughly 2 GiB of RAM. Very short lines cost more than their bytes, because each line has its own bookkeeping, and so do tabs and hidden direction controls: the viewer keeps lines up to about twice the size limit in memory, and past that the header says `(showing first N lines)`. A log of ordinary lines (about 100 bytes) is never cut by this.

## Compressed files

Files ending in `.gz`, `.xz`, `.zst` or `.bz2` open decompressed, as text. Xylem looks at the first bytes of the file, not at its name, and opens one level only: a `.gz` inside a `.gz` stays compressed. The header says `gzip → N bytes` (the format and the size after decompression).

Decompression has limits. The viewer reads up to the size limit, takes at most 3 s and uses at most 64 MiB of memory for the xz dictionary or the zstd window. In Autoview the limits are 1 MiB, 100 ms and 16 MiB. When the output or the time limit is hit, the header says `(showing first N bytes decompressed)`. A note tells what happened:

- `decompression stopped after 3 s` (100 ms in Autoview)
- `compressed data truncated or corrupt`
- `needs more than 64 MiB of memory to decompress` (16 MiB in Autoview, with ` (V to view)` when V's 64 MiB may be enough)

The note goes away at the first key, but the header keeps the problem: `(truncated or corrupt)`, `(incomplete: …)` when the memory limit stopped decompression partway, or `(not decompressed: …)` when nothing could be decompressed (the raw bytes are then shown in hex).

Autoview shows the note in its title. In hex mode (`H`) you see the decompressed bytes. Colors follow the name without the compression extension: `app.rs.gz` is colored as Rust.

For a `.tar.gz` (or `.tar.xz`...) the note says `TAR archive: Alt+F5 opens it as a folder`.

To see the raw compressed bytes, turn off `viewer_decompress` (**J** on page 2 of the [configuration](configuration.md)).

## Table mode

Some files open as a grid instead of text. Xylem goes by the extension, and only for uncompressed files on this disk: a file on a remote machine or inside an archive shows as ordinary contents, with the note `Tables are shown only for files on this disk`.

| Format | Extensions | Column names |
|---|---|---|
| CSV, TSV | `.csv` `.tsv` `.tab` | the first row, when at least half of it is not numbers; otherwise A, B, C... |
| JSON Lines | `.jsonl` `.ndjson` | the keys of the objects |
| Spreadsheets | `.xlsx` `.xls` `.xlsb` `.ods` | A, B, C... (the first row is a row like any other) |
| dBase, Clipper, FoxPro | `.dbf`, with its `.dbt` memo file | the field names |
| SQLite | `.db` `.sqlite` `.sqlite3` | the columns of the table |
| Parquet | `.parquet` | the columns of the file |

The CSV separator (comma, semicolon, tab or vertical bar) is detected from the start of the file.

| Key | What it does |
|---|---|
| ↑ ↓ | previous or next row |
| ← → | previous or next column |
| PgUp PgDn | a page of rows up or down |
| Home End | first or last column |
| Ctrl+Home Ctrl+End | first or last row; under a filter, of the rows it keeps |
| `[` `]` | previous or next sheet of a spreadsheet, or table of a SQLite database, going around; this clears the filter |
| `Tab` | the list of sheets or tables: type part of a name to narrow it, Enter opens one and Esc goes back; opening another one clears the filter. In a database, views come after the tables, marked `(view)` |
| `g` or `:` | jump to a row by its number in the file; if the filter hid it, to the next one kept |
| `\` or `&` | the SQL filter |
| `m` or Insert | mark or unmark the record under the cursor, and go to the next; **Ctrl+B** then writes only the marked ones |
| `M` | clear the marks |
| `i` | the structure of the table: what the file says of each column (see below) |
| Enter | the record under the cursor as a form, a field per line: ↑ ↓ choose the field, ← → go to the previous or next record, `c` copies the whole record (`NAME : value`, a line per field) and `y` the value of the field, to paste in an editor or a terminal; Esc goes back. What is copied is what the screen shows: control and text direction characters, line breaks included, are `·`, so pasting in a terminal runs nothing. Elsewhere, Enter closes the viewer |
| `T` | to the file's text and back |
| `H` | from the table to text, then hex, then back to the table |

The frame shows `TABLE [2/5: name]` when there is more than one sheet or table, and the bottom line where the cursor is: `Cell B7 (Row 7 of 120)`. Under a filter, rows keep the number they have in the file, and this line adds which match it is: `Cell A13 (Row 13 of 31, match 2 of 8)`. The last line of the grid has the whole contents of the cell and the filter in use. Searching (`F`, `B`, `/`) does not look inside the grid and finds nothing there; it works in text mode.

Marks stay under a filter, and a Batch writes what is marked and kept by the filter. They belong to the sheet or table open, and are lost when changing it. If the file changes after marking, the Batch refuses, since other records may be at those places.

### The structure

`i` shows, over the grid, the columns in order with what the file says of each one, like `DISPLAY STRUCTURE` in dBase and Clipper. The arrows, PgUp, PgDn, Home and End scroll it; Esc, Enter or `i` close it.

| Format | What it shows |
|---|---|
| dBase, Clipper, FoxPro | name, type (`C` character, `N` numeric, `D` date, `L` logical, `M` memo, `F` float, `I` integer, `B` double, `Y` currency, `T` date and time), width and decimals; above them, the number of records and the date of the last update written in the header, and the length of a record |
| SQLite | the type declared for each column, `Not null`, the default and the place in the primary key; above them, whether it is a table or a view, and its indexes with their columns (at most 32). A view shows the columns it gives; a whole query, those of its result, with the type each one has when it comes straight from a table's column |
| Parquet | the Arrow type of each column and whether it takes nulls; above them, the rows and row groups the file declares and the program that wrote it |
| CSV, JSON Lines, spreadsheets | these declare no types: what the rows read have in each column (integer, decimal, logical, text, empty) and the widest value |

Names, types and defaults come from the file and are made safe like the cells: control and text direction characters show as `·`. A `.dbf` whose header contradicts itself shows `?` for the decimals.

### The SQL filter

`\` or `&` asks for a condition, like the `WHERE` of SQL (the word `WHERE` itself is optional), and shows only the rows that meet it. The prompt starts with the current filter; leave it empty to see all the rows again. A note says `Filter matched N rows` or `SQL error: ...`; after an error, the previous filter stays.

```
age >= 30 AND name LIKE 'Ali%'
WHERE status IN ('open', 'late') OR total > 1000.50
NOT (city = 'Rio' OR city IS NULL)
B ILIKE '%silva%'
```

| Operator | What it does |
|---|---|
| `=` `!=` `<>` `<` `<=` `>` `>=` | compare |
| `AND` `OR` `NOT` and parentheses | combine conditions (`AND` comes before `OR`) |
| `LIKE` `ILIKE` | match a pattern, `%` for any text and `_` for one character; `ILIKE` ignores case |
| `IN (...)` `NOT IN (...)` | is, or is not, in the list |
| `IS NULL` `IS NOT NULL` | the cell is, or is not, empty |

Keywords can be in any case.

- **Columns:** the name of the column, in any case and without quotes, or its letter from `A` to `Z`. The letter is the way to reach a name with spaces or symbols. A column that does not exist counts as empty, without an error.
- **Values:** text in single or double quotes (`\'` for a quote inside it), numbers, `TRUE`, `FALSE` and `NULL`. Anything in quotes is text, never a column.
- **Cells:** the type comes from what the cell looks like. Empty is `NULL`, `true` and `false` are booleans, numbers are numbers, and the rest is text. So write `code = 7`, not `code = '7'`: text and a number are never equal, and a cell with `007` is the number 7. Text comparisons distinguish upper and lower case.
- **Size:** a filter has at most 1,024 bytes and 128 terms.

### A whole query in a database

In a SQLite database, the same prompt takes a whole query, starting with `SELECT` or `WITH`, with all that SQLite can do: `JOIN`, `GROUP BY`, `ORDER BY`, functions.

```
SELECT c.name, sum(o.price) FROM customers c JOIN orders o ON o.code = c.code GROUP BY c.name
```

The frame says `QUERY:`, and the grid shows what it gave. A condition then filters those rows, and **Ctrl+B** writes all that the query gives. Leaving the prompt empty clears the filter and then goes back to the table; `Tab`, `[` and `]` go to a table too. A query only reads, one at a time: anything else is refused, and a mistake leaves the screen as it was. The limits below apply, so a slow query stops at 3 s and shows the rows it got; a Batch has no such limit. Xylem does not change databases: `UPDATE`, `DELETE` and the like get a message saying so. To change one, use another program: `O` in the file window opens it in the one set in `opener`, and `'*.db' = 'sqlite3 %f'` under `[editors]` makes `E` open it in `sqlite3`.

### Big tables

With **V**, these tables have no limit of rows: a CSV or TSV file, a JSON Lines file, a dBase file (`.dbf`), and a table or view of a SQLite database, or a query typed in one. The grid keeps the rows near the cursor, at most 2,000 of them and 32 MiB of text, and reads the others from the file as the cursor moves, without holding the screen: a row not read yet shows as `…` for a moment.

- **The total** shows while it is counted: `Row 1 of 2,000+ (counting…)`, and then `Row 1 of 8,412,903`. **Ctrl+End** goes to the last row and `g` to any; before the count ends, `g` past it says the count has not got there yet.
- **The filter** (`&`) goes over the whole table, in the background. The line under the grid says how far it went, `1,203 matches so far, row 2,300,000 of 8,412,903`, and the rows show as they are found. It keeps at most 1,000,000 matches, and then says `first 1,000,000 matches`.
- **Esc** stops the count or the filter, keeping what they found so far; the next Esc closes the viewer.
- Column widths come from the first rows, and a column widens when a wider row comes on screen. A JSON Lines file can also gain columns as it is counted, when a key first shows up further on.
- **CSV, TSV and JSON Lines** are read once from start to end as they are counted, and Xylem notes where every 256th row starts, so that any row is reached by reading at most 255 others. That note is kept only in memory, about 8 bytes for every 256 rows, and goes when the viewer closes. A CSV record with a line break inside quotes is still one row. A CSV record longer than 16 MiB stops the reading, and a line of JSON Lines longer than 512 KiB is passed over.
- **dBase** records all have the same size, so Xylem goes straight to any of them. The count looks only at whether each record was deleted, since a deleted record is not a row: Xylem notes where every 256th row is, the same way.
- **When the file changes** while it is open (it grows, shrinks or is replaced), what Xylem noted of it no longer holds: nothing more is read from it, and the header says `(the file changed: R reloads)`. **R** reads it again.
- **The text and the hex** of a file that opens as a table are read only when you go to them (**T**, **H**): until then Xylem reads just the start of the file, so a big table opens at once.

Autoview (F7) does not open databases, and it reads only the first rows of the other tables. The other formats keep the limit below for now.

### Limits

Except for the big tables above, a table holds at most 50,000 rows and 256 columns, and takes at most what the viewer's other limits allow: 3 s (100 ms in Autoview), and a spreadsheet that unpacks to more than the size limit (1 MiB in Autoview) does not open as a table. What lies beyond them does not show and the filter does not see it; the header says so: `(showing first 50,000 rows)` or `(stopped by the time limit: showing N rows)`. While a table is being read the screen stays free and the note says so; Esc stops the reading and shows what came so far, `(stopped by Esc: showing N rows)`, and the next Esc closes the viewer. A cell shows at most 2,048 characters and then `…`. Control and text direction characters show as `·`.

When a spreadsheet (XLSX, XLSB, XLS, ODS) has more rows than that, the header and the note say so: `(the sheet has more than 50,000 rows, the most Xylem shows: open it in Excel or another spreadsheet program to see them all (O))`. **O** opens it in the program your system uses for it.

## Tables in C libraries

SQLite databases (`.db`, `.sqlite`, `.sqlite3`) and Parquet files are read by libraries written in C, which do not have Rust's memory safety. So they open as tables only with `V`. Autoview reads whatever the cursor passes over, and shows these files as ordinary contents, with the note `Parquet or SQLite: V opens the table`. Open a database from an untrusted source only if you accept this risk. `.ods` and `.xls` files also open as tables only with `V`: their reader builds the whole sheet first, and that part cannot be stopped.

See also: [searching the tagged files](search.md).
