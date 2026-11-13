# Configuration

## The configuration screen (F12)

**F12** (or Alt+F10, as in ZTreeWin; on GNOME, Alt+F10 usually maximizes the
window and never reaches Xylem) opens four pages:

| Page | Options |
|---|---|
| 1. Programs and formats | editor, open editor inside Xylem's frame, Xylem theme in the editor, edit binary files with E, binary editor, help language, opener, date format, number format, Git commands, trusted Git folders |
| 2. Display and sorting | hidden files, initial sort order and direction, what Enter does in the file window, initial display mode, syntax colors, soft wrap, line numbers, compressed files, the charset of old text files, the size limit of what the viewer reads, icons and glyphs (Nerd Fonts), file names colored by type, window borders, colors of the built-in terminal, seconds in the clock |
| 3. Confirmations and history | confirm on quit, delete to trash, Prune confirmation, entries with `@` not saved, the built-in terminal's own history |
| 4. Copy, compare and refresh | ignore 2 s, ignore 1 h, automatic refresh, where Alt+C/Alt+M paths start |

The viewer's size limit (**L** on page 2) costs memory and time: with `1024` MiB, opening an ordinary text file that big takes roughly 2.5 GiB of RAM and about 5 s, and files of very short lines cost more per byte (see [the viewer](viewer.md)).

Icons and glyphs (**M** on page 2) require a font patched with Nerd Fonts. You do not need a specific font: any monospaced font will work (such as Fira Code, JetBrains Mono, Hack, Meslo or Cascadia Code), as long as you choose its "Nerd Font" edition. Nerd Fonts injects thousands of standard icon glyphs into the Unicode Private Use Area. An unpatched font (even original Fira Code) does not contain these icons and will show missing boxes (`□`). To use icons:
- Download any Nerd Font of your choice from nerdfonts.com, or install a Nerd Fonts package from your distribution's package manager.
- Select that font in your terminal emulator's font settings.

If the columns drift out of line with icons on, the terminal is probably drawing "ambiguous width" characters as wide (a setting common with Chinese, Japanese and Korean fonts), while Xylem counts each icon as one cell. Set that option to narrow, or turn icons off.

File names colored by type (**N** on page 2) give archives, images, audio and video, documents, code and data each a color of the theme, by the name only (nothing is read). Off, the list has the classic look. Window borders (**O**) are rounded in the nord, gruvbox, slate, graphite, daylight and vivid themes and straight in the others; `plain` or `rounded` keeps them one way in every theme. The start screen (**P**) shows the name in block letters for two seconds when Xylem opens; any key closes it (that key does nothing else), and the start folder is read meanwhile. It is not shown when connecting to a remote machine or container from the command line, nor in a terminal smaller than 58 by 13.

The option's letter (or Enter) changes the value, PgUp/PgDn or 1–4 switch
pages, **F4** picks the theme from a list (with an instant preview), **F3**
saves and applies, **Esc** cancels (it asks if there are changes).

## The configuration file

Everything is stored in `~/.config/xylem/config.toml` (or
`$XDG_CONFIG_HOME/xylem/`; `%APPDATA%\xylem\` on Windows), which can also be
edited by hand: the screen keeps your comments. To start from a commented file
with all the defaults:

```bash
mkdir -p ~/.config/xylem && xy --default-config > ~/.config/xylem/config.toml
```

- `theme`: `classic` (XTree blue), `dark`, `light`, `mono` (the terminal's own
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
