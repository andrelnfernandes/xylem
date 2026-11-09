# Layouts: fixed-width and multi-record files

Many files exchanged between systems are text with no column names: each line is a record, and which characters of it hold which field is written in a manual. Bank files (CNAB 240), tax files (SPED), and the SDF export of dBase and Clipper are like this. A layout tells Xylem where each field is, and the viewer then shows the file as a table: one sheet per kind of record, each row starting with its line in the file.

## Opening a file through a layout

Xylem recognizes some files by their first line and opens them as a table on its own, with `V` and in Autoview (F7). The note says which layout it used: `Layout: FEBRABAN CNAB 240 — Padrão Geral  ·  K changes it`. For any other file, press `K`.

| Key | What it does |
|---|---|
| `K` | the list of layouts: type part of a name to narrow it, Enter applies one, **F2** asks for the path of a layout file, Esc goes back. When a layout is in use, the first item turns it off and shows the file as text |
| `[` `]` | previous or next kind of record |
| `Tab` | the list of kinds of record, with how many lines each has |
| `i` | the fields of the record: position, size, type, description and the meaning of each code; above them, the layout, its source, the checks of the totals and their results |
| `T` | to the text, at the line of the record under the cursor, and back |
| `\` or `&` | the SQL filter, by field name: `valor_pagamento > 1000` |
| Ctrl+B | a Batch of the records of the sheet open ([Batch in the viewer](execute.md)) |

The sheets come in the order each kind of record first appears in the file: `0 - Header de Arquivo (1)`, `3A - Segmento A (420)`... A line that matches no record goes to the sheet `! No record type`, whole. The first column, `#line`, is the line in the file, which is how validators of banks and of the tax authorities point to an error. When a field has codes with a meaning, the last line of the grid shows it: `Content: 2 — Retorno (Banco → Cliente)`.

Values are read as the layout says: codes keep their zeros (`00012345`), dates become `2026-11-09`, times `14:30:00`, and amounts keep their exact decimal places (`1234.50`); nothing is rounded. A value that is not what the layout says (letters in a number, the 31st of February) shows as written, and `i` counts them.

### Checks of the totals

Trailers carry totals: the lines of a batch, the number of batches, the sum of the payments. Xylem counts and adds them while reading, in integers, and says in `i` which agree and which do not, with the line, what the file says and what was counted. When one disagrees, the header says so: `1 check failed: i shows`. A file read only in part (stopped by Esc or the time limit) has no checks.

## Where layouts come from

| Where | Used by itself? |
|---|---|
| Xylem's catalog, built into the program | yes, when the first line fits its `[detect]` |
| Your folder: `~/.config/xylem/layouts/` (`%APPDATA%\xylem\layouts\` on Windows), up to three levels deep | yes, and it comes before the catalog |
| Next to the file (`K`, section `Here`), or any path (`K`, then F2) | only when you pick it |

A layout file in the folder being browsed is never read on its own: a downloaded or extracted folder could bring one made to deceive. When you pick it, it is read as strictly as the data (see Security below).

The catalog has, for now:

| Layout | What it reads |
|---|---|
| FEBRABAN CNAB 240 — Padrão Geral | file header and trailer; payments by credit to account, cheque, OP, TED and Pix (segments A, B, C, Z); payment of collection slips (J, J-52); collection (P, Q, R on sending, T and U on return). From the FEBRABAN manual, version 11.0 |
| SPED — genérico | any SPED file of fields between vertical bars (EFD ICMS/IPI, EFD-Contribuições, ECD, ECF...): a sheet per record, the opening and closing of each block and block 9 with its checks. Records it does not declare get numbered fields (`CAMPO_02`, `CAMPO_03`...). What follows the `9999` in signed files (the digital signature) is not read |

The `layouts/` folder of the [repository](https://github.com/andrelnfernandes/xylem/tree/main/layouts) lists what is planned and how to contribute a layout, with no Rust involved.

### SDF from a .dbf

`COPY TO file SDF` in dBase and Clipper writes the records side by side, each field with its width. In `K`, a `.dbf` next to the file (or one given with F2) gives its layout: the fields in order, dates as `YYYYMMDD`, numbers with a point. Memo fields are not exported by SDF and are left out.

## Writing a layout

A layout is a TOML file whose name ends in `.xlayout.toml`. A positional one:

```
schema = 1
name = "Customers of the old system"
line_length = 40
[discriminator]
pos = [1, 1]
[[records]]
id = "0"
name = "Header"
role = "header"
fields = [
  { name = "type", pos = [1, 1], type = "code" },
  { name = "created", pos = [2, 9], type = "date", format = "YYYYMMDD" },
  { pos = [10, 40], type = "filler" },
]
[[records]]
id = "1"
name = "Customer"
fields = [
  { name = "type", pos = [1, 1], type = "code" },
  { name = "code", pos = [2, 7], type = "code" },
  { name = "name", pos = [8, 30], type = "text" },
  { name = "balance", pos = [31, 40], type = "decimal", decimals = 2, sign = "trailing" },
]
```

Positions are 1-based and inclusive: `pos = [8, 30]` is from the 8th to the 30th character. `start = 8, len = 23` says the same, and `len` next to `pos` must agree with it, which catches typing mistakes. With `strict_positions` (on by default), the fields of a record must cover the whole line, with no gap and no overlap: name the empty parts as `filler`.

| Key | What it says |
|---|---|
| `schema` | always `1` |
| `name` | the name shown in `K` and in the header |
| `format` | `"positional"` (the default) or `"delimited"` |
| `encoding` | `"auto"` (the one the viewer found; `O` changes it), `"utf-8"`, `"cp1252"`, `"cp850"` or `"cp437"` |
| `offset_unit` | positional: `"chars"` (the default) or `"bytes"` |
| `line_length`, `strict_positions` | positional: the length of a line, and whether the fields must cover it |
| `hide_filler` | `filler` fields stay out of the grid (default `true`); `i` always lists them |
| `delimiter`, `outer_delimiters` | delimited: the separator, one character, and whether lines also start and end with it, as in SPED |
| `allow_undeclared_records` | delimited: records the layout does not declare get a sheet of their own |
| `decimal_sep`, `date_format` | defaults for all fields: `"implied"`, `","` or `"."`; and the format of dates |
| `stop_after_trailer` | stop at the `trailer` record (what follows is not read) |
| `[detect]` | how to recognize the file by its first line: `line_length`, `first_line` (conditions) and `extensions` |
| `[discriminator]` | where the code of the record is: `pos` or, delimited, `col` |
| `description`, `country`, `category`, `spec_source`, `spec_version`, `spec_url`, `status` | about the layout and its manual; `status` is `"verified"` once checked against it |

Each `[[records]]` has an `id` (its code for the discriminator; `ids` adds more codes), a `name`, a `role` (`header`, `batch_header`, `detail`, `batch_trailer` or `trailer`) and its `fields`. When the code alone does not tell records apart, `match` adds conditions on the line, and `batch_match` conditions on the line of its batch header: in CNAB 240, segment A of a payment and of a debit differ only by the operation in the batch header. A condition is a place (`pos` or `col`) and one of `value = "3"`, `in = ["1", "2"]` or `regex = "^[0-9]+$"`. Among records with the same code, the one with more conditions is tried first.

| Field key | What it says |
|---|---|
| `name` | the column name, also for the SQL filter |
| `pos`, `start` and `len`, or `col` | where it is |
| `type` | `text`, `code` (digits that are not a quantity: CPF, account, CEP), `int`, `decimal`, `date`, `time` or `filler` |
| `decimals`, `decimal_sep` | decimal places, and where the point is: `"implied"` (COBOL's `V99`), `","` or `"."` |
| `sign` | `"none"`, `"leading"`, `"trailing"` or `"overpunch"` (COBOL's zoned sign: `{`, `A`-`I`, `}`, `J`-`R`) |
| `format` | dates: `YYYYMMDD`, `DDMMYYYY`, `YYMMDD`, `DDMMYY` or `YYYYDDD`; times: `HHMMSS` or `HHMM`. Two-digit years 00-69 are in the 2000s |
| `trim` | spaces taken from text: `"right"` (the default), `"left"`, `"both"` or `"none"` |
| `desc`, `values` | the description, and what each code means: `values = { "0" = "Inclusion", "9" = "Exclusion" }` |
| `check` | what the value must agree with, below |

| `check` | The value must be |
|---|---|
| `total_lines` | the lines of the file up to this one |
| `record_count:1*` | the lines of records with this id; `*` takes every id that starts so, `+` adds more |
| `sum:3A.valor` | the sum of a field of a record in the whole file; `+` adds more |
| `batch_lines` | the lines from the batch header up to this one |
| `batch_sum:3A.valor+3J.valor` | the sum in the batch |
| `count_by_record:REG_BLC` | the lines whose code is the value of the field named (SPED's 9900) |
| `block_lines` | the lines whose code starts like this one's (SPED's x990) |

COBOL copybooks (`copybook = ...`), EBCDIC and packed numbers (`COMP-3`) are planned, and not read yet.

## Security

Both the data and a layout picked from outside your folder can be hostile. A layout is read as strictly as data: at most 1 MiB, any unknown key or type is refused, every position and size is checked, names cannot have control or text direction characters, and records, fields, codes, conditions and regular expressions have limits. Regular expressions run in linear time. A layout cannot read other files: `copybook` is refused for now.

The data is read with the limits of the table mode: 3 s (100 ms in Autoview), 50,000 rows per sheet, 256 columns, and lines over 512 KiB are not split into fields. Esc stops the reading. Cells are made safe for the terminal like any other: control and text direction characters show as `·`.

See also: [the viewer](viewer.md).
