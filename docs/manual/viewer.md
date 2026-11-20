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

**F** searches from the start, **B** backwards from the end,
**/** (or `S`) from the current page and **\\** backwards from it; **Space**
continues in the same direction, **+** and **-** choose the direction. The
match is highlighted. At the search prompt, **F2** toggles case sensitivity
(ignoring case, accents count too: `AÇÃO` finds `ação`) and **F4** switches
between text and hex (byte pairs such as `4d 5a`). A `*` in the middle of the
term matches anything on the same line (`error*disk`).

**F3** turns the direction of the search, so F alone does what B does: forward from the start or backward from the end.

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

## JSON, YAML and TOML

A `.json` (also `.geojson`, `.har`, `.webmanifest`, and `.jsonc` and `.json5` with comments and trailing commas), a `.yaml` or `.yml`, and a `.toml` (also `Cargo.lock`) open as a tree that folds, and the frame says `TREE: JSON`. A minified file of one line reads as easily as any other. A tree of up to 2,000 nodes opens all open; a bigger one shows its first level.

| Key | Action |
|---|---|
| ↑ ↓ PgUp PgDn Home End | move the cursor |
| → | open the node (again: go into it) |
| ← | close the node, or go to the one above it |
| `Enter` | open or close the node |
| `*` | open everything under the node |
| `k` | copy the path of the node: `.servers[2].name` in JSON, `servers.2.name` in YAML and TOML. A key with a line end, a control or a text direction character is put between quotes with those escaped (`\n`, `\u202e`), so pasting the path in a shell never runs a line of it |

The footer shows the path of the node under the cursor. A search (`F`, `B`, `/`) looks at keys and values, also inside closed nodes, and opens the way to what it finds. `T` shows the text at the node's line, and comes back; `H` goes through tree, text and hex. `&`, `g`, `#` and Ctrl+B work on the lines of the text.

- A closed node shows what it holds: `{12 keys}`, `[340 items]`.
- A number is shown as the file writes it: `12345678901234567890` stays so, and `1_000` in TOML too.
- A string shows a line end as `↵`.
- A key that appears twice in one object is shown twice, marked `dup key`, and the header counts them. Programs disagree on which of the two wins, which can hide a value from a review.
- A YAML anchor shows as `&a1` and an alias as `*a1`: an alias is never expanded, so a file built to multiply itself (a "billion laughs") costs no more than its text. A tag (`!Ref`) shows before its value. Several documents are shown under one root, as `document 1`, `document 2`...

A file that is not valid, or nested deeper than 512 levels, opens as text, and the note says why and on which line. The tree is built from the whole file: one over the read limit opens as text. The tree also stops at 2,000,000 nodes (about 170 MiB of memory) or when building it takes too long; it then holds only the start of the file, and the header says `(tree of the start only)`. `T` still shows all the text.

## Jupyter notebooks

A `.ipynb` opens rendered, cell by cell, and the frame says `RENDERED`. At the top: the kernel, and how many cells of each kind there are.

- A Markdown cell is rendered as a Markdown file is.
- A code cell shows `In [n]:` and its code, in the colors of the notebook's language.
- Its outputs follow, marked `Out [n]:`: the text of a stream (`stderr:` when it is the errors), the text of a result (or its Markdown, rendered), and an error with its traceback, without the traceback's ANSI codes.
- An image, HTML, JavaScript, SVG or LaTeX output is one line that says what it is and how big, as `[image/png, 34 KiB: not shown]`. Nothing in a notebook is run: not its code, not the scripts of its outputs, not its widgets.
- A raw cell is shown as it is.

`T` shows the notebook's JSON, and `H` its bytes. A file that is not a notebook this viewer can read (broken JSON, an unknown format) opens as its source, with a note. Notebook format 3 (with worksheets) is read too.

## Office documents

Word (`.docx`, `.docm`, `.dotx`), OpenDocument text (`.odt`), PowerPoint (`.pptx`, `.pptm`) and Impress (`.odp`) files open as a document: headings, paragraphs, lists, tables, bold, italic and links, with each link's real address. A presentation shows each slide under `Slide n`, with its speaker notes after it. Footnotes and comments, with their authors, come at the end. An image is a line, as `[image: media/image1.png]`. The text of a document is never read as Markdown.

Before the text, a block says what the document has besides it, since much of it does not show in the program that made it:

- who wrote it, who saved it last, when, and with what;
- `Contains macros`, when it has them. They are never read or run;
- `Fetches from outside when it opens`: a remote template, an OLE object or a frame that point outside the file, with their addresses. A remote template is how attacks like Follina (CVE-2022-30190) came in. Nothing is ever fetched here;
- the embedded objects, by name and size, and the count of other external links;
- tracked changes: inserted text shows as `[inserted: ...]` and deleted text as `[deleted: ...]`, since what was deleted is often what matters;
- hidden text (`[hidden: ...]`), white text and text of one point (`[invisible: ...]`), and hidden slides.

`H` shows the bytes, and comes back; `T` says the document has no text of its own. Every part of the file is decompressed only up to a limit (64 MiB each, 256 MiB in all; 4 MiB in Autoview), and the header says when the document was cut. An XML part with a DTD, or nested deeper than 256 levels, is refused. The old `.doc` and `.ppt` open in hex, as before.

## Norton Guides

A Norton Guide or an Expert Help guide (`.ng`), the hypertext help of DOS and Clipper, opens as a document. A file needs both the `.ng` name and the guide's signature; anything else opens as before.

- **At the top:** the guide's title and credits, and what made it.
- **The menus:** each with its items, and where each item leads.
- **The entries:** all of them, in the order of the file, each under a number: `§12`. A list entry shows its lines; a text entry shows its text with the guide's layout, then its see-also links. Every link says where it goes (`→ §12`, `See also: Two() §3`), so a search for `§12` takes you there. A link that points nowhere says `(broken link)`.

Bold, underline and reverse come out in the theme's styles; a guide's DOS colors are not sent to the terminal. The guide's characters are code page 437, with the glyphs DOS drew for the bytes 1 to 31 (`☺ ♥ ► ▲`); nothing in a guide reaches the terminal as a control character.

A guide is a web of offsets that could lie: each one is checked against the file, each count against its own record, and the spaces a line expands to have a limit. What is wrong with a guide is listed at the top, under "Problems in this guide", and the rest still shows. A guide is read whole, up to 64 MiB; `H` shows the bytes.

The format is known thanks to Dave Pearson, who worked it out in the 1990s and has kept Norton Guides readable ever since: <https://www.davep.org/norton-guides/>. Xylem's reader is its own code, written from his description of the format; it includes no code from his readers.

## Executables

An executable, a library or an object file opens as a report, whatever its name: ELF (Linux, BSD), PE (Windows `.exe`, `.dll`, `.sys`, also .NET) and Mach-O (macOS, also universal binaries with several architectures). It is told by its first bytes, never by its name. The report says:

- what it is (executable, library, object, core dump), for what machine, 32 or 64 bits, its entry point, and its interpreter, subsystem or minimum system;
- **protections**, each one `yes` or `no`, with `⚠` where it is missing: on ELF, PIE, a stack that does not run code (NX), RELRO (full or partial), the stack canary, FORTIFY and the library paths (RPATH, RUNPATH: a relative one or one in `/tmp` is marked); on PE, ASLR, high-entropy ASLR, DEP, Control Flow Guard; on Mach-O, PIE and an executable stack;
- **signature**: whether there is one and how big. It is **not verified**, and the report says so;
- **signs** of something hidden: a section of high entropy (packed or encrypted), the name of a packer (UPX...), data after the last section (an overlay, where installers and some malware keep data), a section both writable and executable, an entry point outside every executable section;
- **identity**: build ID, debug link, UUID, PDB path, the compiler, the Go version; the build date of a PE, which the linker writes and can be anything;
- the libraries it needs, and the lists of sections, segments, imports, exports and symbols, of up to 2,000 rows each.

Nothing is run or loaded to make the report: not even `ldd`, which runs the program in some cases. `H` shows the bytes, and comes back. A file bigger than the read limit is read only where the report needs it, 256 MiB at most in all; Autoview reports on the start it read, and says it may miss things.

## Certificates and keys

A certificate, a chain or bundle of them, a certificate request (CSR), a revocation list (CRL), a private or public key, an OpenSSH key, `authorized_keys` and `known_hosts` open as a report. PEM is read in a file named for it (`.pem`, `.crt`, `.cer`, `.key`, `.csr`, `.crl`, `.pub`...) or with no extension (`id_ed25519`); DER in a `.der`, `.cer`, `.crt`, `.crl` or `.csr` that is one whole DER value. A PEM block inside source code or a document leaves that file as it is.

- **A certificate:** its subject and issuer (and whether it is self-signed), serial, validity with the days left (expired, not valid yet, or expiring within 30 days are marked `⚠`), public key (RSA under 2048 bits and DSA are marked), signature (MD5 and SHA-1 are marked), names (a punycode name `xn--` is pointed out), key usage, extended key usage, whether it is a CA, the addresses of its CRL and authority information, an unknown critical extension (marked), and its SHA-256 fingerprint.
- **Several certificates:** the report says whether they are in order, each one issued by the next, checked by their names and key identifiers only. **No signature is verified here**, and the report says so.
- **A private key:** only its kind and size (RSA 2048 bits, EC P-256, PKCS#8 encrypted with PBES2...) and, for OpenSSH, whether it is encrypted and its fingerprint. Its numbers are never shown in the report. In Autoview, a private key shows this report, never its text, so it does not appear on the screen just by passing over it; `T` in the viewer shows the text.
- **Public keys, `authorized_keys` and `known_hosts`:** one row per key, with its kind and size, its SHA-256 fingerprint as `ssh-keygen -l` writes it, and its comment. A `command=` option is marked, since it changes what the key allows; a hashed host shows as such.

Nothing is fetched: the addresses of a CRL or OCSP are only shown. Before any parser sees a block, its nesting (at most 64 levels) and lengths are checked; a file has at most 1,000 blocks of 64 KiB each. PKCS#12 (`.p12`, `.pfx`) and PGP keys are not read.

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
- a path that is absolute, climbs out with `..`, goes into `.git` (a hook is code that Git runs), or has control, text direction or invisible characters (a name such as `invoice‮fdp.exe` hides its real extension).

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

Spreadsheets, CSV, DBF, SQLite and Parquet open as a grid, with a SQL filter: see [Tables](tables.md).

See also: [searching the tagged files](search.md).
