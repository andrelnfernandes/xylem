//! The mouse (docs/plans/mouse.md): turning it on and off in the terminal,
//! and the map of what each part of the last frame drawn is, which the
//! drawing fills in and a click is looked up in.
//!
//! The mouse never has commands of its own: a click picks a row or stands
//! for a key, and the key goes the way the keyboard's does (refusals,
//! questions, the trash).

use std::io::{self, Write};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Position, Rect};
use ratatui::style::Modifier;
use ratatui::text::Line;

use crate::keymap::KeySpec;

/// Turns the mouse on: clicks (1000), moving with a button held (1002), in
/// SGR coordinates (1006), which have no limit of 223 columns and no bytes
/// above 127. Not 1003 (every move, even with no button), which crossterm's
/// `EnableMouseCapture` turns on: a stream of events for nothing, heavy over
/// SSH.
#[cfg(unix)]
pub fn enable(out: &mut impl Write) -> io::Result<()> {
    out.write_all(b"\x1b[?1000h\x1b[?1002h\x1b[?1006h")?;
    out.flush()
}

#[cfg(unix)]
pub fn disable(out: &mut impl Write) -> io::Result<()> {
    out.write_all(b"\x1b[?1006l\x1b[?1002l\x1b[?1000l")?;
    out.flush()
}

/// The Windows console has a mode for it, not escape codes.
#[cfg(windows)]
pub fn enable(out: &mut impl Write) -> io::Result<()> {
    ratatui::crossterm::execute!(out, ratatui::crossterm::event::EnableMouseCapture)
}

#[cfg(windows)]
pub fn disable(out: &mut impl Write) -> io::Result<()> {
    ratatui::crossterm::execute!(out, ratatui::crossterm::event::DisableMouseCapture)
}

/// What tells the keys of a footer from its other text (`ui::key_style`):
/// a modifier taken away, which changes nothing on screen (no footer
/// blinks), so the theme's colors do not matter.
pub const KEY_MARK: Modifier = Modifier::RAPID_BLINK;

/// Lines a turn of the wheel moves.
pub const WHEEL_LINES: usize = 3;

/// The longest time between the two clicks of a double click.
pub const DOUBLE_CLICK: std::time::Duration = std::time::Duration::from_millis(400);

/// The item of a list `len` rows long (of `cols` items each) that row `y` of
/// a scroll bar `track` rows high stands for: the same share of the list.
pub fn bar_item(y: u16, track: u16, len: usize, cols: usize) -> usize {
    let last = len.saturating_sub(1);
    let row = if track <= 1 {
        0
    } else {
        usize::from(y.min(track - 1)) * last / usize::from(track - 1)
    };
    row * cols.max(1)
}

/// Whose list a part of the screen is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Owner {
    /// A window of the main screen (index into `App::panes`).
    Main(usize),
    /// A window of the folder picker (F2 at a prompt).
    Picker(usize),
}

/// A list of a window: the tree or the files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum List {
    Tree(Owner),
    Files(Owner),
}

/// What a part of the last frame is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spot {
    /// A window: its border, its title, the rows past its last item, or a
    /// view of it with no rows to point at (Usage, Duplicates).
    Window(List),
    /// One item a row: the row at `y` is item `offset + y - top`, if
    /// below `len`.
    Rows {
        list: List,
        offset: usize,
        len: usize,
    },
    /// The files in columns: `cols` cells of `cell` columns a row, from
    /// row `top` of the grid.
    Grid {
        list: List,
        top: usize,
        cols: usize,
        cell: u16,
        len: usize,
    },
    /// An entry of a footer: the key it shows.
    Key(KeyEvent),
    /// A link of the help (its number in the page).
    Link(usize),
    /// Text that scrolls with the wheel (viewer, help): the wheel stands
    /// for ↑ and ↓.
    Scroll,
    /// The mark before a folder of the tree (`+`, `?`, `─`): row `row` of
    /// the list.
    Marker { list: List, row: usize },
    /// The scroll bar on the right border of a list: `len` rows (of `cols`
    /// items each, in the column modes), `page` on screen, the thumb from
    /// row `thumb.0` of the bar, `thumb.1` rows long.
    Scrollbar {
        list: List,
        len: usize,
        page: usize,
        cols: usize,
        thumb: (u16, u16),
    },
    /// One item a row of a list of a screen of its own.
    Item {
        list: Own,
        offset: usize,
        len: usize,
    },
    /// A cell of the viewer's table: its row in what is shown, its column.
    Cell { row: usize, col: usize },
    /// The text of a prompt, from column `start` on: a click puts its cursor
    /// there.
    Prompt { start: u16 },
    /// The screen around the context menu: a click there closes it.
    Outside,
}

/// The lists that are not a tree or files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Own {
    /// The options of a page of F12.
    Settings,
    /// The themes of F12 (F4).
    Theme,
    Bookmarks,
    /// The history of a prompt, open as a list (↑).
    History,
    /// The results of the search box (P).
    SearchResults,
    /// The Usage view of a window (index into `App::panes`).
    Usage(usize),
    Duplicates(usize),
    /// The lines of the context menu.
    Menu,
}

/// What a click hit: the spot and the item under it, if it is a list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hit {
    pub spot: Spot,
    /// The item of a `Rows`, `Grid` or `Item` under the pointer.
    pub item: Option<usize>,
    /// The part of the screen of the spot.
    pub area: Rect,
    /// Where the pointer was.
    pub x: u16,
    pub y: u16,
}

/// The spots of the last frame drawn; the last one added is on top.
#[derive(Debug, Default, Clone)]
pub struct HitMap {
    spots: Vec<(Rect, Spot)>,
}

impl HitMap {
    pub fn add(&mut self, area: Rect, spot: Spot) {
        if !area.is_empty() {
            self.spots.push((area, spot));
        }
    }

    /// What is at column `x`, row `y`: the topmost spot there.
    pub fn at(&self, x: u16, y: u16) -> Option<Hit> {
        let pos = Position::new(x, y);
        let (area, spot) = self.spots.iter().rev().find(|(a, _)| a.contains(pos))?;
        let item = match *spot {
            Spot::Rows { offset, len, .. } | Spot::Item { offset, len, .. } => {
                Some(offset + usize::from(y - area.y)).filter(|&i| i < len)
            }
            Spot::Grid {
                top,
                cols,
                cell,
                len,
                ..
            } => {
                let col = usize::from((x - area.x) / cell.max(1));
                let row = top + usize::from(y - area.y);
                (col < cols).then(|| row * cols + col).filter(|&i| i < len)
            }
            _ => None,
        };
        Some(Hit {
            spot: *spot,
            item,
            area: *area,
            x,
            y,
        })
    }

    /// Where row `item` of `list` is on screen (its first column and row),
    /// if it is.
    pub fn row_of(&self, list: List, item: usize) -> Option<(u16, u16)> {
        self.spots
            .iter()
            .rev()
            .find_map(|(area, spot)| match *spot {
                Spot::Rows {
                    list: l, offset, ..
                } if l == list && item >= offset && item - offset < usize::from(area.height) => {
                    Some((area.x, area.y + (item - offset) as u16))
                }
                Spot::Grid {
                    list: l,
                    top,
                    cols,
                    cell,
                    ..
                } if l == list && item / cols.max(1) >= top => {
                    let row = item / cols.max(1) - top;
                    let col = (item % cols.max(1)) as u16;
                    (row < usize::from(area.height))
                        .then(|| (area.x + col * cell, area.y + row as u16))
                }
                _ => None,
            })
    }

    /// The footer's lines, as drawn from row `area.y` on: each entry that
    /// shows a key becomes a `Spot::Key`. `mods` is the modifier of the
    /// menu's words (Ctrl or Alt in those menus).
    pub fn add_footer(&mut self, lines: &[Line], area: Rect, mods: KeyModifiers) {
        for (row, line) in lines.iter().enumerate() {
            let Some(y) = area
                .y
                .checked_add(row as u16)
                .filter(|&y| y < area.bottom())
            else {
                break;
            };
            for (start, width, key) in footer_keys(line, mods) {
                let x = area.x.saturating_add(start);
                let rect = Rect::new(x, y, width, 1).intersection(area);
                self.add(rect, Spot::Key(key));
            }
        }
    }
}

/// The entries of a footer line that show a key: (first column, width, key).
///
/// The line is split where `fit_menu` splits it (its empty marker spans),
/// and again where a key starts, so that a line with no markers (the
/// help's) still has one entry a key. An entry runs from its first to its
/// last visible character: the spaces between entries belong to none. A
/// word of a menu (`eXecute`) stands for its letter, with `mods`; a label
/// (`F3`, `Esc`, `Alt+X`) for itself; a label of several keys (`PgUp/PgDn`,
/// `↑↓`) is one entry a key, over its own letters only.
pub fn footer_keys(line: &Line, mods: KeyModifiers) -> Vec<(u16, u16, KeyEvent)> {
    // (column, text, is a key) of each span, in groups split by the
    // markers; and whether the group is a word (its marker is `word_end`).
    let mut groups: Vec<(Vec<Piece>, bool)> = vec![(Vec::new(), false)];
    let mut x = 0usize;
    for span in &line.spans {
        let text = span.content.as_ref();
        if text.is_empty() {
            groups.last_mut().unwrap().1 = span.style.add_modifier.contains(Modifier::HIDDEN);
            groups.push((Vec::new(), false));
            continue;
        }
        let is_key = span.style.sub_modifier.contains(KEY_MARK);
        let group = &groups.last().unwrap().0;
        // A second key in the same group starts an entry of its own.
        if is_key && group.iter().any(|(_, _, k)| *k) {
            groups.push((Vec::new(), false));
        }
        groups.last_mut().unwrap().0.push((x, text, is_key));
        x += Line::raw(text).width();
    }

    let mut out = Vec::new();
    for (group, word) in groups {
        let Some(at) = group.iter().position(|(_, _, k)| *k) else {
            continue;
        };
        let (key_x, label, _) = group[at];
        // Before the key, only a word has its own letters (`eXecute`); what
        // else is there is a title (`CONFIG`), which is no part of it.
        let from_key = &group[at..];
        // A word of `menu_words` (`eXecute`, `Yes`): its letter, with the
        // menu's modifier.
        let label_trim = label.trim();
        if word && label_trim.chars().count() == 1 {
            let Some(span) = visible_span(&group) else {
                continue;
            };
            let c = label_trim.chars().next().unwrap().to_ascii_lowercase();
            let key = KeyEvent::new(KeyCode::Char(c), mods);
            out.push((span.0 as u16, span.1 as u16, key));
            continue;
        }
        let keys = label_keys(label);
        match keys.as_slice() {
            [] => {}
            // One key: the whole entry, its description too.
            [(_, _, key)] => {
                if let Some(span) = visible_span(from_key) {
                    out.push((span.0 as u16, span.1 as u16, *key));
                }
            }
            many => {
                for &(at, width, key) in many {
                    out.push(((key_x + at) as u16, width as u16, key));
                }
            }
        }
    }
    out
}

/// A span of a footer line: its column, its text, and whether it is a key.
type Piece<'a> = (usize, &'a str, bool);

/// First visible column and width of a group of spans.
fn visible_span(group: &[Piece]) -> Option<(usize, usize)> {
    let mut first = None;
    let mut last = 0;
    for &(x, text, _) in group {
        let mut col = x;
        for c in text.chars() {
            let w = Line::raw(c.to_string()).width();
            if !c.is_whitespace() {
                first.get_or_insert(col);
                last = col + w;
            }
            col += w;
        }
    }
    first.map(|f| (f, last - f))
}

/// The keys a label of a footer names, each with its column and width in
/// the label: `"F3"`, `"Alt+X"`, `"Shift+PgUp/PgDn"` (Shift for both),
/// `"↑↓ Enter"`. A range (`"A-Z"`, `"1-4"`) or a word that is no key
/// (`"drag"`) names none.
pub fn label_keys(label: &str) -> Vec<(usize, usize, KeyEvent)> {
    let mut out = Vec::new();
    let mut inherited = KeyModifiers::NONE;
    let mut col = 0;
    for word in split_keep(label, &[' ', '/']) {
        let width = Line::raw(word).width();
        if !word.trim().is_empty() && !matches!(word, "/" | " ") {
            if word.chars().all(|c| "↑↓←→".contains(c)) {
                let mut c_col = col;
                for c in word.chars() {
                    let code = match c {
                        '↑' => KeyCode::Up,
                        '↓' => KeyCode::Down,
                        '←' => KeyCode::Left,
                        _ => KeyCode::Right,
                    };
                    let w = Line::raw(c.to_string()).width();
                    out.push((c_col, w, KeyEvent::new(code, inherited)));
                    c_col += w;
                }
            } else if let Some(key) = parse_label(word, &mut inherited) {
                out.push((col, width, key));
            }
        }
        col += width;
    }
    out
}

/// `"Shift+PgUp"` → Shift+PageUp, keeping the modifiers for the next part
/// of the label (`/PgDn`).
fn parse_label(word: &str, inherited: &mut KeyModifiers) -> Option<KeyEvent> {
    // One character is that character, in its case: `C` is not `c` in
    // the viewer.
    let mut chars = word.chars();
    if let (Some(c), None) = (chars.next(), chars.clone().next()) {
        return Some(KeyEvent::new(KeyCode::Char(c), *inherited));
    }
    if word.len() > 1 && word.contains('-') && !word.contains('+') {
        return None;
    }
    let lower = word.to_lowercase();
    let mut shift = false;
    let mut rest = lower.as_str();
    // KeySpec drops Shift (it means nothing to a letter); the terminal's
    // Shift+PgUp needs it.
    if let Some(r) = rest.strip_prefix("shift+") {
        shift = true;
        rest = r;
    }
    let spec = KeySpec::parse(rest).ok()?;
    let (code, mut mods) = spec.parts();
    if shift {
        mods |= KeyModifiers::SHIFT;
    }
    if rest.contains('+') || shift {
        *inherited = mods;
    } else {
        mods |= *inherited;
    }
    Some(KeyEvent::new(code, mods))
}

/// `text` cut at each of `seps`, the separators kept as parts of their own.
fn split_keep<'a>(text: &'a str, seps: &[char]) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut start = 0;
    for (i, c) in text.char_indices() {
        if seps.contains(&c) {
            if start < i {
                out.push(&text[start..i]);
            }
            out.push(&text[i..i + c.len_utf8()]);
            start = i + c.len_utf8();
        }
    }
    if start < text.len() {
        out.push(&text[start..]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Style;
    use ratatui::text::Span;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn codes(label: &str) -> Vec<KeyEvent> {
        label_keys(label).into_iter().map(|(_, _, k)| k).collect()
    }

    #[test]
    fn a_label_names_its_keys() {
        assert_eq!(codes("F3"), [key(KeyCode::F(3))]);
        assert_eq!(codes("Esc"), [key(KeyCode::Esc)]);
        assert_eq!(
            codes("Alt+X"),
            [KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT)]
        );
        assert_eq!(
            codes("PgUp/PgDn"),
            [key(KeyCode::PageUp), key(KeyCode::PageDown)]
        );
        let shift = KeyModifiers::SHIFT;
        assert_eq!(
            codes("Shift+PgUp/PgDn"),
            [
                KeyEvent::new(KeyCode::PageUp, shift),
                KeyEvent::new(KeyCode::PageDown, shift)
            ]
        );
        assert_eq!(
            codes("↑↓ Enter"),
            [key(KeyCode::Up), key(KeyCode::Down), key(KeyCode::Enter)]
        );
        // The columns of each key in the label.
        let cols: Vec<_> = label_keys("PgUp/PgDn")
            .into_iter()
            .map(|(x, w, _)| (x, w))
            .collect();
        assert_eq!(cols, [(0, 4), (5, 4)]);
    }

    #[test]
    fn a_range_or_a_hint_names_no_key() {
        assert!(codes("A-Z").is_empty());
        assert!(codes("1-4").is_empty());
        assert!(codes("Shift+drag").is_empty());
        assert_eq!(
            codes("PgUp/PgDn 1-4"),
            [key(KeyCode::PageUp), key(KeyCode::PageDown)]
        );
    }

    fn styled_key(k: &str) -> Span<'static> {
        Span::styled(k.to_string(), crate::ui::key_style())
    }

    #[test]
    fn a_menu_line_has_one_spot_an_entry() {
        // "CONFIG   F3 save   Esc cancel   ", as `menu_line` builds it.
        let line = Line::from(vec![
            Span::raw("CONFIG   "),
            styled_key("F3"),
            Span::raw(" save"),
            Span::raw("   "),
            Span::raw(""),
            styled_key("Esc"),
            Span::raw(" cancel"),
            Span::raw("   "),
            Span::raw(""),
        ]);
        let keys = footer_keys(&line, KeyModifiers::NONE);
        assert_eq!(
            keys,
            [(9, 7, key(KeyCode::F(3))), (19, 10, key(KeyCode::Esc))]
        );
    }

    #[test]
    fn a_word_stands_for_its_letter_with_the_menus_modifier() {
        // "eXecute " and "Yank " as `menu_words` builds them.
        let end = || Span::styled("", Style::new().add_modifier(Modifier::HIDDEN));
        let line = Line::from(vec![
            Span::raw("CTRL     "),
            Span::raw(""),
            Span::raw("e"),
            styled_key("X"),
            Span::raw("ecute "),
            end(),
            styled_key("Y"),
            Span::raw("ank "),
            end(),
        ]);
        let ctrl = KeyModifiers::CONTROL;
        let keys = footer_keys(&line, ctrl);
        assert_eq!(
            keys,
            [
                (9, 7, KeyEvent::new(KeyCode::Char('x'), ctrl)),
                (17, 4, KeyEvent::new(KeyCode::Char('y'), ctrl)),
            ]
        );
    }

    #[test]
    fn keys_with_no_markers_are_entries_of_their_own() {
        // The help's footer: no markers between the entries.
        let line = Line::from(vec![
            Span::raw("HELP     "),
            styled_key("Tab"),
            Span::raw(" link   "),
            styled_key("I"),
            Span::raw(" index   "),
        ]);
        let keys = footer_keys(&line, KeyModifiers::NONE);
        assert_eq!(
            keys,
            [(9, 8, key(KeyCode::Tab)), (20, 7, key(KeyCode::Char('I')))]
        );
    }

    #[test]
    fn the_map_finds_the_item_under_the_pointer() {
        let mut map = HitMap::default();
        let list = List::Tree(Owner::Main(0));
        map.add(Rect::new(0, 1, 30, 10), Spot::Window(list));
        map.add(
            Rect::new(1, 2, 28, 8),
            Spot::Rows {
                list,
                offset: 5,
                len: 9,
            },
        );
        // Row 3 of the list (y = 2 + 3) is item 5 + 3.
        assert_eq!(map.at(4, 5).unwrap().item, Some(8));
        // Past the last item: the window, no item.
        let past = map.at(4, 9).unwrap();
        assert_eq!(past.item, None);
        // The border is the window.
        assert_eq!(map.at(0, 1).unwrap().spot, Spot::Window(list));
        // Outside everything.
        assert_eq!(map.at(40, 1), None);
    }

    #[test]
    fn the_grid_finds_the_cell_under_the_pointer() {
        let mut map = HitMap::default();
        let list = List::Files(Owner::Main(0));
        map.add(
            Rect::new(1, 1, 40, 5),
            Spot::Grid {
                list,
                top: 2,
                cols: 3,
                cell: 12,
                len: 14,
            },
        );
        // Second row on screen (grid row 3), second cell: 3 * 3 + 1.
        assert_eq!(map.at(1 + 13, 2).unwrap().item, Some(10));
        // Past the third cell of a row: no file.
        assert_eq!(map.at(1 + 37, 2).unwrap().item, None);
        // Grid row 4, cell 2: 14, past the last file.
        assert_eq!(map.at(1 + 25, 3).unwrap().item, None);
    }
}
