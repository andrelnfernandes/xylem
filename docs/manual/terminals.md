# Terminals and keys

Xylem receives the keys the terminal sends. Some terminals keep a key for
themselves, and some send the same bytes for two different keys. This page
lists those cases and the key to use instead.

## Keys the terminal or the desktop keeps

| Key | Kept by | Use instead |
|---|---|---|
| F1 | GNOME Terminal (its own help) | Alt+H |
| F11 | GNOME Terminal (full screen) | Alt+B, or `'` to jump to a bookmark |
| Alt+F10 | GNOME (maximizes the window) | F12 |
| Alt+T | terminals with a "Terminal" menu | bind another key in `[keys.any]` |

## Ctrl+M

Most terminals send the same byte for Ctrl+M and Enter. Xylem then sees
Enter. To move the tagged files there, press `F9` and then `M`, or use Alt+M
and press F6 at the prompt to turn the paths off.

In a terminal with the kitty keyboard protocol the two keys are different,
and Ctrl+M moves the tagged files.

## The kitty keyboard protocol

With this protocol the terminal reports Ctrl and Alt pressed and released on
their own. Xylem turns it on when the terminal answers that it has it.

| With the protocol | Without it |
|---|---|
| Holding Ctrl or Alt shows the CTRL or ALT menu | F9 shows them |
| Ctrl+M is Move | Ctrl+M is Enter |

| Terminal | Protocol |
|---|---|
| kitty, Ghostty, WezTerm, foot (Wayland), recent Alacritty | yes |
| iTerm2 | Ctrl+M is Move; holding Ctrl or Alt has not been checked |
| GNOME Terminal, Ptyxis, Terminator, Tilix (built on VTE) | no |

## The clipboard

`Y` and Ctrl+Y write to the clipboard. Over SSH and inside tmux Xylem uses
the OSC 52 sequence, which the terminal has to accept. On X11 and Wayland the text stays in the clipboard
after Xylem quits.

## Icons and box lines

Icons (F12, page 2) need a font with the Nerd Fonts glyphs. Without them the
terminal draws an empty box for each icon. If the columns drift out of line
with icons on, the terminal is drawing "ambiguous width" characters as wide;
set that option to narrow, or turn icons off.

## Screen size

| Width or height | Effect |
|---|---|
| under 72 columns | the name and version leave the header |
| too narrow for a menu line | the words of the menu are shortened, then the last entries are replaced by `…` |
| under 58 columns or 13 lines | the start screen is not shown |

See also: [the menus](menus.md), [configuration](configuration.md),
[the built-in terminal](terminal.md).
