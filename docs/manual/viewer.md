# The viewer

`V` opens the file in the viewer (Autoview, F7, shows it next to the list).
Binary files open in hex. A big file on disk is not read whole: the viewer reads what the screen shows as it moves (see [Big files](#big-files)). What cannot be read that way shows only its beginning, up to the size limit.
Spreadsheets, CSV, DBF, SQLite and Parquet open as a table (see [Tables](tables.md)).

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

Spreadsheets, CSV, DBF, SQLite and Parquet open as a grid, with a SQL filter: see [Tables](tables.md).

See also: [searching the tagged files](search.md).
