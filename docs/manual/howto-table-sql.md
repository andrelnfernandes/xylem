# Filter a CSV file or a spreadsheet with SQL

This opens `customers.csv` as a table and keeps the rows whose plan is
`team` or whose balance is over 1000.

## Steps

1. In the file window, put the cursor on `customers.csv` and press `V`. The
   viewer shows the file as a table, with the first row as the column names.
2. Press `&`. The prompt asks for a condition.
3. Type `plan = 'team' OR balance > 1000` and press Enter.

![The three steps: V on customers.csv showing nine rows, the & prompt, and the condition keeping three rows](../screenshots/howto-table-sql.gif)

The footer shows `Filter matched 3 rows`. The rows keep the numbers they
have in the file. `&` with an empty prompt shows all the rows again.

## Which files open as a table

| Format | Extensions |
|---|---|
| CSV, TSV | `.csv` `.tsv` `.tab` |
| JSON Lines | `.jsonl` `.ndjson` |
| Spreadsheets | `.xlsx` `.xls` `.xlsb` `.ods` |
| dBase | `.dbf` |
| SQLite | `.db` `.sqlite` `.sqlite3` |
| Parquet | `.parquet` |

The file has to be on this disk. A file on a remote machine or inside an
archive opens as text.

## Writing the condition

| To keep rows where | Condition |
|---|---|
| a column equals a text | `plan = 'team'` |
| a number is in a range | `balance >= 0 AND balance < 100` |
| a text starts with something | `name LIKE 'Ana%'` |
| a text contains something, in any case | `city ILIKE '%paulo%'` |
| a cell is empty | `email IS NULL` |
| a column has no usable name | use its letter: `C = 'pro'` |

A cell that looks like a number is a number: write `code = 7`, not
`code = '7'`. [The SQL filter](viewer.md#the-sql-filter) has the operators.

## In a spreadsheet or a database

- A spreadsheet has no column names: the columns are `A`, `B`, `C`. `[` and
  `]` go to the previous and the next sheet, and Tab lists them.
- In a SQLite database, Tab lists the tables and views. The same prompt also
  takes a whole query that starts with `SELECT` or `WITH`:

```
SELECT plan, count(*), sum(balance) FROM customers GROUP BY plan
```

Xylem only reads the database. `UPDATE`, `DELETE` and the like are refused.

## Saving the result

Ctrl+B writes the rows the filter keeps to a file: CSV, JSON, HTML, a SQL
script, or a SQLite table. [Batch](execute.md) describes the forms.

## When it does not do what you expect

- **`SQL error`.** The condition could not be read. The filter that was in
  effect stays.
- **No row matches a column you can see.** A column name that does not
  exist counts as empty and gives no error. Check the spelling, or use the
  column's letter.
- **The file opens as text.** Its extension is not in the table above, or
  the file is compressed, remote, or inside an archive. `T` switches between
  the table and the text when both exist.

See also: [the viewer](viewer.md), [picking files with SQL](howto-sql-filter.md).
