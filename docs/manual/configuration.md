# Configuration

## The configuration screen (F12)

**F12** (or Alt+F10; on GNOME, Alt+F10 usually maximizes the
window and never reaches Xylem) opens five pages:

| Page | Options |
|---|---|
| 1. Programs and formats | editor, open editor inside Xylem's frame, Xylem theme in the editor, edit binary files with E, binary editor, help language, opener, date format, number format, Git commands, trusted Git folders |
| 2. Display and sorting | hidden files, initial sort order and direction, what Enter does in the file window, initial display mode, syntax colors, soft wrap, line numbers, compressed files, the charset of old text files, the size limit of what the viewer reads, icons and glyphs (Nerd Fonts), file names colored by type, window borders, colors of the built-in terminal, seconds in the clock, [the mouse](mouse.md) |
| 3. Confirmations and history | confirm on quit, delete to trash, Prune confirmation, entries with `@` not saved, the built-in terminal's own history, [Alt+T in the fuzzy find tags every match](#tagging-every-match-of-the-fuzzy-find) |
| 4. Copy, compare and refresh | ignore 2 s, ignore 1 h, automatic refresh, where Alt+C/Alt+M paths start, files and folders left out of the index |
| 5. Reading (ebooks) | the width of the text column, paragraphs as in a book or in blocks, justified text, the reading speed for the time left, [the time spent reading](#reading-time), the minutes without a key before its clock stops, the titles kept for a list of the books |

The viewer's size limit (**L** on page 2) costs memory and time: with `1024` MiB, opening an ordinary text file that big takes roughly 2.5 GiB of RAM and about 5 s, and files of very short lines cost more per byte (see [the viewer](viewer.md)).

Icons and glyphs (**M** on page 2) require a font patched with Nerd Fonts. You do not need a specific font: any monospaced font will work (such as Fira Code, JetBrains Mono, Hack, Meslo or Cascadia Code), as long as you choose its "Nerd Font" edition. Nerd Fonts injects thousands of standard icon glyphs into the Unicode Private Use Area. An unpatched font (even original Fira Code) does not contain these icons and will show missing boxes (`□`). To use icons:
- Download any Nerd Font of your choice from nerdfonts.com, or install a Nerd Fonts package from your distribution's package manager.
- Select that font in your terminal emulator's font settings.

If the columns drift out of line with icons on, the terminal is probably drawing "ambiguous width" characters as wide (a setting common with Chinese, Japanese and Korean fonts), while Xylem counts each icon as one cell. Set that option to narrow, or turn icons off.

File names colored by type (**N** on page 2) give archives, images, audio and video, documents, code and data each a color of the theme, by the name only (nothing is read). Off, the list has the classic look. Window borders (**O**) are rounded in the nord, gruvbox, slate, graphite, daylight and vivid themes and straight in the others; `plain` or `rounded` keeps them one way in every theme. The start screen (**P**) shows the name in block letters for two seconds when Xylem opens; any key closes it (that key does nothing else), and the start folder is read meanwhile. It is not shown when connecting to a remote machine or container from the command line, nor in a terminal smaller than 58 by 13.

The option's letter (or Enter) changes the value, PgUp/PgDn or 1–5 switch
pages, **F4** picks the theme from a list (with an instant preview), **F3**
saves and applies, **Esc** cancels (it asks if there are changes).

### Tagging every match of the fuzzy find

In the fuzzy find (`|`), **Alt+T** tags the results of the list. The list
keeps the 500 best matches, and a search can match many more: `f` in a large
tree can match a hundred thousand files. By default, Alt+T tags only the 500
in the list, and when more matched, the line under the list says so and
names this option.

With **Alt+T in the fuzzy find tags every match** on (**F** on page 3,
`fuzzy_tag_all = true` in the file), Alt+T tags every match, including the
ones you never saw. The line under the list then always says how many matches
were not in it.

This is off by default, because tags carry over to what you do next.
Ctrl+C, Ctrl+M, Ctrl+D and the Batch act on every tagged file, not only on
the ones you saw. Thousands of files tagged by a search you only glanced at,
and then deleted, are thousands of files gone. Turn the option on only if
you check what is tagged before acting on it:

- the statistics panel shows how many files are tagged and their size;
- **Ctrl+S** in the tree lists the tagged files of the whole tree (a Showall
  of the tagged ones);
- **Ctrl+V** in the file window views them one after the other.

### Reading time

The viewer reads EPUB and FB2 [ebooks](viewer.md#ebooks). Page 5 sets how
the text is laid out, and whether Xylem keeps the time you spend reading.

**Keep the time spent reading** (**E**, `reading_stats = true` in the file)
is off by default. On, Xylem keeps, for each book remembered with `R` in the
viewer:

- the time read, and the sessions: a pause of more than 30 minutes starts a
  new one;
- the farthest point reached, the first and the last day read, and whether
  the book was finished;
- the reading speed, in words per minute. Only pages read forward count,
  and only when at least 5 seconds were spent on them, so jumps, going back
  and skimming do not. After 10 minutes measured, the footer counts the time
  left with that book's speed instead of the one on this page (**D**).

The clock runs only while you read: from one key to the next, if no more
than **F** minutes (3 by default) passed between them. A longer pause stops
it, and that pause does not count. Autoview (F7) and the report (`I`) never
count.

All of it stays on your computer, in `~/.local/state/xylem/reading.toml`
(or `$XDG_STATE_HOME/xylem/`), readable only by you. A book is kept as a
hash of what it says it is and its size, and numbers: not its name, not
where it is, not its title. With **G** on (`reading_titles = true`), the
title and the author are kept too, and the report of any book then lists
all the books read, under "Your books".

Turning **G** off and saving (F3) deletes the titles kept at once. Turning
**E** off and saving asks whether to delete the time read of all the books;
without that, it stays in the file and is not shown. Either way the places
remembered with `R` stay; `R` on a book forgets it.

## The configuration file

Everything is stored in `~/.config/xylem/config.toml` (or
`$XDG_CONFIG_HOME/xylem/`; `%APPDATA%\xylem\` on Windows), which can also be
edited by hand: the screen keeps your comments. To start from a commented file
with all the defaults:

```bash
mkdir -p ~/.config/xylem && xy --default-config > ~/.config/xylem/config.toml
```

- `theme`: `classic` (white and yellow on blue), `dark`, `light`, `mono` (the terminal's own
  colors), `norton`, `solarized`, `gruvbox`, `nord`, `slate` (in the style of
  Windows Terminal), `graphite` (macOS in dark mode), `daylight` (macOS in
  light mode) or `vivid` (strong colors, for presentations and screenshots).
  F4 shows them all, with a preview.
- `[options]`: the options of the configuration screen.
- `[colors]`: individual colors on top of the theme, as names (`"yellow"`),
  `"#rrggbb"` or indexes from 0 to 255.
- `[keys.any]`, `[keys.tree]`, `[keys.files]`: rebind keys in both windows or
  in just one (which takes precedence). E.g. `"f3" = "view"`,
  `"ctrl+r" = "relog"`; `"none"` disables a key. The help's key list (**K**)
  shows the keys in effect, with your changes.
- `[editors]`: the editor for each kind of file (`"*.png *.jpg" = "gimp %f"`,
  `binary = "hexedit"`); see [Edit](execute.md).

An error in the file does not stop Xylem from starting: it uses the defaults
and shows the error in the footer, and the configuration screen will not save
until the file is fixed, so it does not erase what you wrote.

Xylem was called xt until version 0.1. On its first run it moves xt's
configuration, history and bookmarks to its own folders.
