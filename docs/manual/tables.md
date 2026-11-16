# Tables: spreadsheets, CSV and databases

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
| Enter | the record under the cursor as a form, a field per line: ↑ ↓ choose the field, ← → go to the previous or next record, `c` copies the whole record (`NAME : value`, a line per field), `C` the same with the number of each field in front, the one the structure (`i`) shows, and `y` the value of the field, to paste in an editor or a terminal; Esc goes back. What is copied is what the screen shows: control and text direction characters, line breaks included, are `·`, so pasting in a terminal runs nothing. Elsewhere, Enter closes the viewer |
| `T` | to the file's text and back |
| `H` | from the table to text, then hex, then back to the table |

The frame shows `TABLE [2/5: name]` when there is more than one sheet or table, and the bottom line where the cursor is: `Cell B7 (Row 7 of 120)`. Under a filter, rows keep the number they have in the file, and this line adds which match it is: `Cell A13 (Row 13 of 31, match 2 of 8)`. The last line of the grid has the whole contents of the cell and the filter in use. Searching (`F`, `B`, `/`) does not look inside the grid and finds nothing there; it works in text mode.

Marks stay under a filter, and a Batch writes what is marked and kept by the filter. They belong to the sheet or table open, and are lost when changing it. If the file changes after marking, the Batch refuses, since other records may be at those places.

## The structure

`i` shows, over the grid, the columns in order with what the file says of each one, like `DISPLAY STRUCTURE` in dBase and Clipper. The arrows, PgUp, PgDn, Home and End scroll it; Esc, Enter or `i` close it. `c` copies it whole, as the screen shows it but without cutting the long names, to paste in an editor or a terminal; control and text direction characters are `·`, as in the record's form.

| Format | What it shows |
|---|---|
| dBase, Clipper, FoxPro | name, type (`C` character, `N` numeric, `D` date, `L` logical, `M` memo, `F` float, `I` integer, `B` double, `Y` currency, `T` date and time), width and decimals; above them, the number of records and the date of the last update written in the header, and the length of a record |
| SQLite | the type declared for each column, `Not null`, the default and the place in the primary key; above them, whether it is a table or a view, and its indexes with their columns (at most 32). A view shows the columns it gives; a whole query, those of its result, with the type each one has when it comes straight from a table's column |
| Parquet | the Arrow type of each column and whether it takes nulls; above them, the rows and row groups the file declares and the program that wrote it |
| CSV, JSON Lines, spreadsheets | these declare no types: what the rows read have in each column (integer, decimal, logical, text, empty) and the widest value |

Names, types and defaults come from the file and are made safe like the cells: control and text direction characters show as `·`. A `.dbf` whose header contradicts itself shows `?` for the decimals.

## The SQL filter

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

## A whole query in a database

In a SQLite database, the same prompt takes a whole query, starting with `SELECT` or `WITH`, with all that SQLite can do: `JOIN`, `GROUP BY`, `ORDER BY`, functions.

```
SELECT c.name, sum(o.price) FROM customers c JOIN orders o ON o.code = c.code GROUP BY c.name
```

The frame says `QUERY:`, and the grid shows what it gave. A condition then filters those rows, and **Ctrl+B** writes all that the query gives. Leaving the prompt empty clears the filter and then goes back to the table; `Tab`, `[` and `]` go to a table too. A query only reads, one at a time: anything else is refused, and a mistake leaves the screen as it was. The limits below apply, so a slow query stops at 3 s and shows the rows it got; a Batch has no such limit. Xylem does not change databases: `UPDATE`, `DELETE` and the like get a message saying so. To change one, use another program: `O` in the file window opens it in the one set in `opener`, and `'*.db' = 'sqlite3 %f'` under `[editors]` makes `E` open it in `sqlite3`.

## Big tables

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

## Limits

Except for the big tables above, a table holds at most 50,000 rows and 256 columns, and takes at most what the viewer's other limits allow: 3 s (100 ms in Autoview), and a spreadsheet that unpacks to more than the size limit (1 MiB in Autoview) does not open as a table. What lies beyond them does not show and the filter does not see it; the header says so: `(showing first 50,000 rows)` or `(stopped by the time limit: showing N rows)`. While a table is being read the screen stays free and the note says so; Esc stops the reading and shows what came so far, `(stopped by Esc: showing N rows)`, and the next Esc closes the viewer. A cell shows at most 2,048 characters and then `…`. Control and text direction characters show as `·`.

When a spreadsheet (XLSX, XLSB, XLS, ODS) has more rows than that, the header and the note say so: `(the sheet has more than 50,000 rows, the most Xylem shows: open it in Excel or another spreadsheet program to see them all (O))`. **O** opens it in the program your system uses for it.

## Tables in C libraries

SQLite databases (`.db`, `.sqlite`, `.sqlite3`) and Parquet files are read by libraries written in C, which do not have Rust's memory safety. So they open as tables only with `V`. Autoview reads whatever the cursor passes over, and shows these files as ordinary contents, with the note `Parquet or SQLite: V opens the table`. Open a database from an untrusted source only if you accept this risk. `.ods` and `.xls` files also open as tables only with `V`: their reader builds the whole sheet first, and that part cannot be stopped.

See also: [the viewer](viewer.md).
