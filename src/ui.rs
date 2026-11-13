//! Screen drawing in the classic XTree layout:
//!
//! ```text
//! ┌ current path ───────────────────────────┐
//! │ tree                        │ stats     │
//! ├─────────────────────────────┤           │
//! │ directory files             │           │
//! └─────────────────────────────┴───────────┘
//!  command menu (2 lines)
//! ```

use std::ops::Range;
use std::path::Path;
use std::time::SystemTime;

use chrono::{DateTime, Local};
use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, List, ListItem, ListState, Paragraph};

use crate::app::{self, App, BookmarksView, FileView, Focus, KbyteShow, MenuMode, Pane};
use crate::charset::Charset;
use crate::config::{
    self, IconStyle, TerminalTheme, Theme, formats, group_digits, one_decimal, theme,
};
use crate::filetype;
use crate::git::StatusCache;
use crate::help::{self, Help};
use crate::icons::{DirKind, SLOT_WIDTH};
use crate::input::Command;
use crate::model::tree::{FileEntry, FileKind, NodeId};
use crate::ops::{Conflict, Facts, Operation};
use crate::pane::{Count, FileDisplay, FileRef};
use crate::platform;
use crate::settings::{self, Settings};
use crate::term::{self, Terminal};
use crate::viewer::table::TableGrid;
use crate::viewer::{self, ViewMode, Viewer};

const STATS_WIDTH: u16 = 28;
/// The small file window under the tree is a quarter of the height (8 rows,
/// 6 files, on an ordinary 34-row screen), so that it keeps its share of a
/// tall terminal or a short one; never under this (3 files).
const SMALL_FILE_WINDOW: u16 = 5;
/// Below this height, the tree is shown without the small file window.
const MIN_TREE_WITH_FILES: u16 = 12;

pub fn draw(frame: &mut Frame, app: &mut App) {
    draw_screen(frame, app);
    if app.splash.is_some() {
        draw_splash(frame, &theme());
    }
    // Last step, after every early return: no bidi or control character
    // may reach the terminal, whatever drew it.
    crate::bidi::sanitize(frame.buffer_mut(), Style::new().fg(theme().error));
}

fn draw_screen(frame: &mut Frame, app: &mut App) {
    let base = Style::new().fg(theme().text).bg(theme().background);
    frame.render_widget(Block::new().style(base), frame.area());

    if let Some(help) = &mut app.help {
        draw_help(frame, help);
        return;
    }
    if app.editor_pty.is_some() {
        draw_editor(frame, app);
        return;
    }
    if app.viewer.is_some() {
        draw_viewer(frame, app);
        return;
    }
    if let Some(settings) = &app.settings {
        draw_settings(frame, settings);
        return;
    }
    if let Some(view) = &app.bookmarks_view {
        draw_bookmarks(frame, app, view);
        return;
    }

    // The Usage view has three lines of keys of its own.
    let footer = if app.usage.is_some() {
        3
    } else {
        footer_height(app, frame.area().width as usize)
    };
    let [header, main, menu] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(5),
        Constraint::Length(footer),
    ])
    .areas(frame.area());

    draw_header(frame, app, header);
    if app.picker.is_some() {
        draw_picker(frame, app, main, menu);
        return;
    }
    draw_menu(frame, app, menu);
    let git = app.options.git.then_some(&app.git_status);

    // Alt+X: the terminal takes the bottom of the screen; the tree stays
    // above it, following the shell's `cd`s. Alt+Z: the whole screen.
    let main = if app.terminal_focus
        && let Some(term) = &mut app.terminal
    {
        if app.terminal_full {
            draw_terminal(frame, term, app.options.terminal_theme, main);
            return;
        }
        let [top, bottom] =
            Layout::vertical([Constraint::Percentage(45), Constraint::Percentage(55)]).areas(main);
        draw_terminal(frame, term, app.options.terminal_theme, bottom);
        top
    } else {
        main
    };

    if app.autoview.is_some() {
        // F7: list on the left, contents of the selected file on the right.
        let [list, view] =
            Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
                .areas(main);
        let active = app.active;
        draw_files(
            frame,
            &mut app.panes[active],
            true,
            list,
            git,
            app.options.icons,
        );
        if let Some(v) = &mut app.autoview {
            let name = v
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let title = match &v.note {
                Some(note) => format!(" Autoview: {name} — {note} "),
                None => format!(" Autoview: {name} "),
            };
            draw_viewer_body(frame, v, title, view);
        }
    } else if app.is_split() {
        // Two windows side by side; the statistics panel goes away to make room.
        let halves: [Rect; 2] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(main);
        let active = app.active;
        for (i, (pane, area)) in app.panes.iter_mut().zip(halves).enumerate() {
            draw_pane(frame, pane, i == active, true, area, git, app.options.icons);
        }
    } else {
        let [left, stats] =
            Layout::horizontal([Constraint::Min(20), Constraint::Length(STATS_WIDTH)]).areas(main);
        // The pane first: the Usage view leaves its selected row for the panel.
        draw_pane(
            frame,
            &mut app.panes[0],
            true,
            false,
            left,
            git,
            app.options.icons,
        );
        draw_stats(frame, app, stats);
    }
}

/// Width of the bars of the Usage view (less on a narrow screen).
const USAGE_BAR: usize = 16;

/// Alt+U: the folders and files of a directory by the space they take, and
/// a line with the totals at the bottom.
fn draw_usage(frame: &mut Frame, pane: &mut Pane, active: bool, area: Rect) {
    use crate::usage::{self, Item};
    let Some(view) = pane.usage.as_mut() else {
        return;
    };
    let tree = &pane.tree;
    let width = area.width.saturating_sub(2) as usize;
    let rows = view.rows(tree);
    let path = tree.path(view.dir);
    let counter = position(view.list.selected(), rows.len());
    // Room for " Usage: ", the counter and a bit of border between them.
    let room = width.saturating_sub(12 + counter.width());
    let title = format!(" Usage: {} ", fit_tail(&path.to_string_lossy(), room));
    let block = panel(title, active).title_top(counter);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let [list_area, footer] =
        Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(inner);
    view.page = list_area.height as usize;

    let total = view.total(tree);
    let whole = if view.apparent {
        total.apparent
    } else {
        total.disk
    };
    let largest = rows
        .iter()
        .filter(|r| r.mark != '>')
        .map(|r| r.size)
        .max()
        .unwrap_or(0);
    let bar_width = USAGE_BAR.min(width.saturating_sub(40) / 2).max(4);
    let dim = Style::new().fg(theme().dim);
    let items: Vec<ListItem> = rows
        .iter()
        .map(|r| {
            let weight = usage::weight(r.size, whole);
            let (size, bar, pct) = if r.mark == '>' {
                // Another file system: what it uses, not part of this one.
                let other = match r.item {
                    Item::Dir(id) => crate::platform::disk_space(&tree.path(id))
                        .map(|d| usage::size(d.total.saturating_sub(d.free)))
                        .unwrap_or_default(),
                    Item::File(_) => String::new(),
                };
                (other, String::new(), "-".to_string())
            } else {
                (
                    usage::size(r.size),
                    usage::bar(r.size, largest, bar_width),
                    usage::percent(r.size, whole),
                )
            };
            let name = match r.item {
                Item::Dir(_) => format!("/{}", r.name),
                Item::File(_) => format!(" {}", r.name),
            };
            let tag = match r.tagged {
                0 => " ",
                n if n >= r.files => "■",
                _ => "□",
            };
            let mut spans = vec![
                Span::styled(tag, Style::new().fg(theme().tagged)),
                Span::styled(format!("{} ", r.mark), Style::new().fg(theme().key)),
                Span::raw(format!("{size:>10} ")),
            ];
            if r.mark == '>' {
                spans.push(Span::raw(" ".repeat(bar_width)));
            } else {
                spans.extend(bar_spans(bar, bar_width, weight));
            }
            spans.push(Span::styled(format!(" {pct:>6}"), dim));
            if view.counts {
                spans.push(Span::styled(format!(" {:>10}", fmt_num(r.files)), dim));
            }
            let style = if r.mark == '>' {
                dim
            } else if r.tagged > 0 {
                Style::new().fg(theme().tagged)
            } else {
                Style::new()
            };
            spans.push(Span::styled(format!("  {name}"), style));
            ListItem::new(Line::from(spans))
        })
        .collect();
    if view.list.selected().is_none_or(|i| i >= rows.len()) {
        view.list.select(Some(rows.len().saturating_sub(1)));
    }
    view.shown = view.list.selected().and_then(|i| rows.get(i).cloned());
    let list = List::new(items).highlight_style(highlight(active));
    frame.render_stateful_widget(list, list_area, &mut view.list);

    // The totals, and how much of the disk they are.
    let mut parts = Vec::new();
    if total.tagged > 0 {
        let bytes = if view.apparent {
            total.tagged_apparent
        } else {
            total.tagged_disk
        };
        parts.push(format!(
            "{} tagged, {}",
            fmt_num(total.tagged),
            usage::size(bytes)
        ));
    }
    parts.extend([
        format!("{} on disk", usage::size(total.disk)),
        format!("{} apparent", usage::size(total.apparent)),
        format!("{} files", fmt_num(total.files)),
    ]);
    if let Some(d) = crate::platform::disk_space(&path) {
        let used = d.total.saturating_sub(d.free);
        let at_top = crate::info::mount_of(&path).is_some_and(|m| m.point == path);
        let of_used = usage::percent(total.disk, used);
        parts.push(if at_top {
            format!("{of_used} of the disk's used space found")
        } else {
            format!("{of_used} of used")
        });
        parts.push(format!("{} of disk", usage::percent(total.disk, d.total)));
    }
    let unreadable = tree
        .subtree(view.dir)
        .into_iter()
        .filter(|&id| tree.node(id).error.is_some())
        .count();
    if unreadable > 0 {
        parts.push(format!(
            "{} folders could not be read",
            fmt_num(unreadable as u64)
        ));
    }
    if view.order != usage::Order::Size {
        parts.push(format!("sorted by {}", view.order.label()));
    }
    frame.render_widget(
        Line::from(Span::styled(fit(&parts.join(" · "), width), dim)),
        footer,
    );
}

/// "1 group", "2 groups".
fn count(n: usize, what: &str) -> String {
    let plural = if n == 1 { "" } else { "s" };
    format!("{} {what}{plural}", fmt_num(n as u64))
}

/// Alt+D: the groups of equal files or folders, and a line about them.
fn draw_duplicates(frame: &mut Frame, pane: &mut Pane, active: bool, area: Rect) {
    use crate::duplicates::Row;
    use crate::usage;
    let Some(view) = pane.duplicates.as_mut() else {
        return;
    };
    let width = area.width.saturating_sub(2) as usize;
    let rows = view.rows();
    let counter = position(Some(view.at), rows.len());
    let room = width.saturating_sub(17 + counter.width());
    let path = view.root.to_string_lossy();
    let what = if view.show_folders {
        "folders"
    } else {
        "files"
    };
    let title = format!(" Duplicate {what}: {} ", fit_tail(&path, room));
    let block = panel(title, active).title_top(counter);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let [list_area, footer] =
        Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(inner);
    view.page = list_area.height as usize;

    // Only what is on screen is drawn: there may be many thousands of rows.
    let height = view.page.max(1);
    view.at = view.at.min(rows.len().saturating_sub(1));
    if view.at < view.offset {
        view.offset = view.at;
    } else if view.at >= view.offset + height {
        view.offset = view.at + 1 - height;
    }
    let dim = Style::new().fg(theme().dim);
    let groups = view.groups();
    let items: Vec<ListItem> = rows
        .iter()
        .skip(view.offset)
        .take(height)
        .map(|row| match *row {
            Row::Group(g) => {
                let group = &groups[g];
                let text = format!(
                    "── {} × {} ({} to free) ",
                    group.items.len(),
                    usage::size(group.items[0].size),
                    usage::size(group.free),
                );
                let line = "─".repeat(width.saturating_sub(text.chars().count()));
                ListItem::new(Line::from(Span::styled(format!("{text}{line}"), dim)))
            }
            Row::Item(g, i) => {
                let item = &groups[g].items[i];
                let mark = match (item.tagged, item.shared) {
                    (true, _) => "T",
                    (false, true) => "s",
                    (false, false) => " ",
                };
                let date = item.modified.map(fmt_time).unwrap_or_default();
                let path = item.path.strip_prefix(&view.root).unwrap_or(&item.path);
                let mut name = path.to_string_lossy().into_owned();
                if item.snapshot.is_some() {
                    name.push('/');
                }
                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {mark} "), Style::new().fg(theme().key)),
                    Span::raw(format!("{:>10}  ", usage::size(item.size))),
                    Span::styled(format!("{date}  "), dim),
                    Span::raw(name),
                ]))
            }
        })
        .collect();
    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(view.at - view.offset));
    let list = List::new(items).highlight_style(highlight(active));
    frame.render_stateful_widget(list, list_area, &mut state);

    let (tagged, bytes) = view.tagged();
    let free: u64 = groups.iter().map(|g| g.free).sum();
    let mut parts = vec![
        count(groups.len(), "group"),
        format!("{} to free", usage::size(free)),
    ];
    if tagged > 0 {
        parts.push(format!(
            "{} tagged, {}",
            fmt_num(tagged),
            usage::size(bytes)
        ));
    }
    let other = if view.show_folders {
        view.files.len()
    } else {
        view.folders.len()
    };
    if other > 0 {
        let name = if view.show_folders {
            "files"
        } else {
            "folders"
        };
        parts.push(format!("{} of {name} (G)", count(other, "group")));
    }
    let not_read = view.notes.unreadable + view.notes.changed;
    if not_read > 0 {
        parts.push(format!("{} not compared", fmt_num(not_read)));
    }
    if view.notes.no_hash > 0 {
        parts.push(format!("{} without a hash", fmt_num(view.notes.no_hash)));
    }
    if let Some(cloud) = &view.cloud {
        parts.push("by the service's hashes".into());
        if view.notes.same_name > 0 {
            parts.push(format!(
                "{} names repeated in a folder: rclone dedupe {}",
                fmt_num(view.notes.same_name),
                cloud.spec
            ));
        }
    }
    frame.render_widget(
        Line::from(Span::styled(fit(&parts.join(" · "), width), dim)),
        footer,
    );
}

/// One window: tree on top and files below, or just the files.
/// With a split screen, the tree title shows the current directory.
fn draw_pane(
    frame: &mut Frame,
    pane: &mut Pane,
    active: bool,
    split: bool,
    area: Rect,
    git: Option<&StatusCache>,
    icons: IconStyle,
) {
    if pane.usage.is_some() {
        draw_usage(frame, pane, active, area);
        return;
    }
    if pane.duplicates.is_some() {
        draw_duplicates(frame, pane, active, area);
        return;
    }
    if pane.focus == Focus::Files {
        draw_files(frame, pane, active, area, git, icons);
        return;
    }
    let width = area.width.saturating_sub(4) as usize;
    let title = if split {
        format!(
            " {} ",
            fit_tail(&pane.current_path().to_string_lossy(), width)
        )
    } else if let Some(archive) = &pane.archive {
        let name = archive.file_name().unwrap_or_default().to_string_lossy();
        format!(" Archive: {name} (read-only) ")
    } else if let Some(remote) = &pane.remote {
        format!(" Remote: {} ", remote.address.destination())
    } else {
        " Tree ".to_string()
    };
    // Too low for both (e.g. above the terminal's panel): the tree alone,
    // rather than a tree of one or two lines.
    if area.height < MIN_TREE_WITH_FILES {
        draw_tree(frame, pane, active, title, area, icons);
        return;
    }
    let [tree, files] = Layout::vertical([
        Constraint::Min(3),
        Constraint::Length(small_file_window(area.height)),
    ])
    .areas(area);
    draw_tree(frame, pane, active, title, tree, icons);
    draw_files(frame, pane, false, files, git, icons);
}

/// Height of the small file window for a tree area of `height` rows.
fn small_file_window(height: u16) -> u16 {
    height.div_ceil(4).max(SMALL_FILE_WINDOW)
}

fn panel(title: impl Into<Line<'static>>, active: bool) -> Block<'static> {
    let border = if active {
        theme().active_border
    } else {
        theme().border
    };
    Block::bordered()
        .border_type(border_type(&theme()))
        .border_style(Style::new().fg(border))
        .title(title)
}

/// Rounded in the modern themes (or by the option `borders`).
fn border_type(t: &Theme) -> BorderType {
    if t.rounded {
        BorderType::Rounded
    } else {
        BorderType::Plain
    }
}

/// The name in block letters (the shadow in box drawing characters).
const SPLASH_ART: [&str; 6] = [
    "██╗  ██╗██╗   ██╗██╗     ███████╗███╗   ███╗",
    "╚██╗██╔╝╚██╗ ██╔╝██║     ██╔════╝████╗ ████║",
    " ╚███╔╝  ╚████╔╝ ██║     █████╗  ██╔████╔██║",
    " ██╔██╗   ╚██╔╝  ██║     ██╔══╝  ██║╚██╔╝██║",
    "██╔╝ ██╗   ██║   ███████╗███████╗██║ ╚═╝ ██║",
    "╚═╝  ╚═╝   ╚═╝   ╚══════╝╚══════╝╚═╝     ╚═╝",
];

/// The start screen: the name over the screen, centered, in the colors of the
/// theme. Nothing in a terminal too small for it.
fn draw_splash(frame: &mut Frame, t: &Theme) {
    const WIDTH: u16 = 44 + 2 * 6 + 2;
    const HEIGHT: u16 = SPLASH_ART.len() as u16 + 7;
    let area = frame.area();
    if area.width < WIDTH || area.height < HEIGHT {
        return;
    }
    let rect = Rect {
        x: area.x + (area.width - WIDTH) / 2,
        y: area.y + (area.height - HEIGHT) / 2,
        width: WIDTH,
        height: HEIGHT,
    };
    let block = Style::new()
        .fg(t.active_border)
        .add_modifier(Modifier::BOLD);
    let shadow = Style::new().fg(t.dim);
    let mut lines = vec![Line::raw("")];
    lines.extend(SPLASH_ART.iter().map(|row| {
        // Runs of the same style, not one span per character.
        let mut spans: Vec<Span> = Vec::new();
        let mut run = String::new();
        let mut solid = None;
        for c in row.chars() {
            let this = c == '█';
            if solid.is_some_and(|s| s != this) {
                let style = if solid == Some(true) { block } else { shadow };
                spans.push(Span::styled(std::mem::take(&mut run), style));
            }
            solid = Some(this);
            run.push(c);
        }
        let style = if solid == Some(true) { block } else { shadow };
        spans.push(Span::styled(run, style));
        Line::from(spans)
    }));
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "F I L E   M A N A G E R",
        Style::new().fg(t.text).add_modifier(Modifier::BOLD),
    ));
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        concat!("v", env!("CARGO_PKG_VERSION")),
        Style::new().fg(t.dim),
    ));
    frame.render_widget(Clear, rect);
    frame.render_widget(
        Paragraph::new(lines).centered().block(
            Block::bordered()
                .border_type(border_type(t))
                .border_style(Style::new().fg(t.border))
                .style(Style::new().fg(t.text).bg(t.background)),
        ),
        rect,
    );
}

fn highlight(active: bool) -> Style {
    if active {
        Style::new()
            .fg(theme().highlight_text)
            .bg(theme().highlight)
    } else {
        Style::new()
            .fg(theme().highlight_text)
            .bg(theme().inactive_highlight)
    }
}

/// The header's clock: the date format, with seconds if so configured.
fn clock_pattern(seconds: bool) -> String {
    let date = formats().date.pattern();
    if seconds {
        format!("{date}:%S")
    } else {
        date.to_string()
    }
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let path = app.tree.path(app.header_dir());
    let clock = Local::now()
        .format(&clock_pattern(app.options.clock_seconds))
        .to_string();
    // The name and the version before the date, as XTreeGold did, while there
    // is room left for the path.
    let name = concat!("Xylem v", env!("CARGO_PKG_VERSION"), "  ");
    let mut right_spans = Vec::new();
    if area.width >= 72 {
        right_spans.push(Span::styled(
            "Xylem",
            Style::new().fg(theme().key).add_modifier(Modifier::BOLD),
        ));
        right_spans.push(Span::styled(
            &name["Xylem".len()..],
            Style::new().fg(theme().dim),
        ));
    }
    right_spans.push(Span::raw(clock));
    let width: usize = right_spans.iter().map(|s| s.content.chars().count()).sum();
    let [left, right] =
        Layout::horizontal([Constraint::Min(1), Constraint::Length(width as u16)]).areas(area);
    let mut spans = vec![Span::styled("Path: ", Style::new().fg(theme().key))];
    // On a remote machine or a container: its name stands out first, so that
    // where a change goes is never in doubt; then the path there.
    let (text, separator) = match &app.remote {
        Some(remote) => {
            let badge = format!(" {} ", remote.address.destination());
            spans.push(Span::styled(
                badge,
                Style::new()
                    .fg(theme().background)
                    .bg(theme().key)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(" "));
            let there = String::from_utf8_lossy(&remote.remote_path(&path)).into_owned();
            (there, '/')
        }
        None => (home_short(&path), std::path::MAIN_SEPARATOR),
    };
    let used: usize = spans.iter().map(|s| s.content.chars().count()).sum();
    let room = (left.width as usize).saturating_sub(used + 1);
    spans.extend(path_spans(&fit_tail(&text, room), separator));
    frame.render_widget(Line::from(spans), left);
    frame.render_widget(Line::from(right_spans), right);
}

/// `path` with the home directory as `~`.
fn home_short(path: &Path) -> String {
    let text = path.to_string_lossy().into_owned();
    let Some(home) = platform::home_dir().filter(|h| h.parent().is_some()) else {
        return text;
    };
    match path.strip_prefix(&home) {
        Ok(rest) if rest.as_os_str().is_empty() => "~".into(),
        Ok(rest) => format!("~{}{}", std::path::MAIN_SEPARATOR, rest.display()),
        Err(_) => text,
    }
}

/// A path with the folders before the last one dim and the last one bold.
fn path_spans(text: &str, separator: char) -> Vec<Span<'static>> {
    let dim = Style::new().fg(theme().dim);
    let last = Style::new().fg(theme().text).add_modifier(Modifier::BOLD);
    match text.rfind(separator) {
        Some(i) if i + separator.len_utf8() < text.len() => {
            let cut = i + separator.len_utf8();
            vec![
                Span::styled(text[..cut].to_string(), dim),
                Span::styled(text[cut..].to_string(), last),
            ]
        }
        _ => vec![Span::styled(text.to_string(), last)],
    }
}

/// What the directory of a tree row is, for its icon.
fn dir_kind(pane: &Pane, id: NodeId) -> DirKind {
    let root = id == crate::model::tree::ROOT;
    if root && pane.archive.is_some() {
        DirKind::Archive
    } else if root && pane.remote.is_some() {
        DirKind::Remote
    } else if pane.tree.has_git(id) {
        DirKind::Repo
    } else if pane.tree.node(id).expanded {
        DirKind::Open
    } else {
        DirKind::Closed
    }
}

fn draw_tree(
    frame: &mut Frame,
    pane: &mut Pane,
    active: bool,
    title: String,
    area: Rect,
    icons: IconStyle,
) {
    pane.tree_page = area.height.saturating_sub(2) as usize;

    let (range, mut state) = visible_window(&mut pane.tree_state, pane.rows.len(), pane.tree_page);
    // Alt+K: size, file count or share to the right of each directory.
    let kbyte = pane.kbyte;
    let percent = kbyte.show == KbyteShow::Percent;
    let totals = (kbyte.show != KbyteShow::None).then(|| {
        pane.tree
            .dir_totals(kbyte.branch || percent, kbyte.tagged_only)
    });
    // Percent: the largest subdirectory of each parent, for the bars.
    let largest = |parent: NodeId, totals: &[(u64, u64)]| {
        let tree = &pane.tree;
        tree.node(parent)
            .children
            .iter()
            .filter(|&&c| tree.show_hidden || !tree.node(c).hidden)
            .map(|&c| totals[c].1)
            .max()
            .unwrap_or(0)
    };
    let width = area.width.saturating_sub(2) as usize;
    let guides = if active {
        path_guides(pane, range.clone())
    } else {
        vec![Vec::new(); range.len()]
    };
    let items: Vec<ListItem> = pane.rows[range]
        .iter()
        .zip(guides)
        .map(|(row, guides)| {
            let node = pane.tree.node(row.id);
            let name = node.name.to_string_lossy().into_owned();
            let name_style = if node.error.is_some() {
                Style::new().fg(theme().error)
            } else {
                Style::new()
            };
            // Only with icons on: the repository check looks through the
            // subdirectories and files of the row.
            let icon = match icons {
                IconStyle::None => None,
                _ => crate::icons::dir_icon(dir_kind(pane, row.id), icons),
            };
            let mut spans = prefix_spans(&theme(), &row.prefix, &guides);
            if let Some(ic) = icon {
                let ic_style = if node.error.is_some() {
                    Style::new().fg(theme().error)
                } else {
                    Style::new().fg(theme().key)
                };
                spans.push(Span::styled(ic, ic_style));
            }
            spans.push(Span::styled(name.clone(), name_style));
            if let Some(totals) = &totals {
                let (count, bytes) = totals[row.id];
                let icon_len = if icon.is_some() { SLOT_WIDTH } else { 0 };
                let used = row.prefix.chars().count() + icon_len + name.chars().count();
                let mut bar = None;
                let value = match (node.scanned, kbyte.show) {
                    (false, _) => "?".to_string(),
                    (_, KbyteShow::Size) => format!("{}K", fmt_num(bytes.div_ceil(1024))),
                    (_, KbyteShow::Percent) => {
                        let (whole, top) = match node.parent {
                            Some(p) => (totals[p].1, largest(p, totals)),
                            None => (bytes, bytes),
                        };
                        let (b, pct) = share(bytes, whole, top, width.saturating_sub(used + 1));
                        bar = b;
                        pct
                    }
                    _ => fmt_num(count),
                };
                let bar_len = bar.as_ref().map_or(0, |_| SHARE_BAR + 1);
                let pad = width.saturating_sub(used + bar_len + value.chars().count() + 1);
                spans.push(Span::raw(" ".repeat(pad + 1)));
                if let Some(bar) = bar {
                    let whole = node.parent.map_or(bytes, |p| totals[p].1);
                    let weight = crate::usage::weight(bytes, whole);
                    spans.extend(bar_spans(bar, SHARE_BAR, weight));
                    spans.push(Span::raw(" "));
                }
                spans.push(Span::styled(value, Style::new().fg(theme().dim)));
            }
            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = match kbyte.show {
        KbyteShow::None => title,
        _ => format!("{} · {} ", title.trim_end(), kbyte.describe()),
    };
    // As in ZTreeWin: line of the cursor / lines in the tree.
    let position = position(pane.tree_state.selected(), pane.rows.len());
    let list = List::new(items)
        .block(panel(title, active).title_top(position))
        .highlight_style(highlight(active));
    frame.render_stateful_widget(list, area, &mut state);
    let offset = pane.tree_state.offset();
    draw_scroll_thumb(frame, area, pane.rows.len(), pane.tree_page, offset, active);
}

/// Where a long list is: a thumb on the right border, as long as the share
/// of the list on screen, at its place. Nothing when it all fits.
fn draw_scroll_thumb(
    frame: &mut Frame,
    area: Rect,
    len: usize,
    page: usize,
    offset: usize,
    active: bool,
) {
    let track = area.height.saturating_sub(2) as usize;
    if len <= page || page == 0 || track < 3 || area.width < 2 {
        return;
    }
    let size = (track * page / len).clamp(1, track - 1);
    let last = len - page;
    let start = (track - size) * offset.min(last) / last;
    let color = if active {
        theme().active_border
    } else {
        theme().border
    };
    let x = area.right() - 1;
    let buf = frame.buffer_mut();
    for i in start..start + size {
        let y = area.y + 1 + i as u16;
        buf[(x, y)].set_symbol("┃").set_fg(color);
    }
}

/// The lines from the root to the cursor, for the rows in `range`: for each
/// row, the levels of its prefix that are on that way (each level is 3
/// characters), and whether the whole connector is (the row of a folder on the
/// way) or only its vertical stroke (a row between two folders on the way).
fn path_guides(pane: &Pane, range: Range<usize>) -> Vec<Vec<(usize, bool)>> {
    let mut out = vec![Vec::new(); range.len()];
    let Some(sel) = pane.tree_state.selected().filter(|&s| s < pane.rows.len()) else {
        return out;
    };
    // The folders on the way, root first, and the row of each one: they are
    // all above the cursor, in that order.
    let mut way = vec![pane.rows[sel].id];
    while let Some(parent) = pane.tree.node(*way.last().unwrap()).parent {
        way.push(parent);
    }
    way.reverse();
    let mut at = Vec::with_capacity(way.len());
    let mut next = 0;
    for (i, row) in pane.rows[..=sel].iter().enumerate() {
        if way.get(next) == Some(&row.id) {
            at.push(i);
            next += 1;
        }
    }
    if at.len() != way.len() {
        return out; // a folder on the way is not shown: nothing to draw
    }
    for level in 0..at.len() - 1 {
        let (parent, child) = (at[level], at[level + 1]);
        for r in (parent + 1)..=child {
            if range.contains(&r) {
                out[r - range.start].push((level, r == child));
            }
        }
    }
    out
}

/// The prefix of a tree row (`│  ├──`), dim, with the strokes on the way to the
/// cursor in the color of the active border.
fn prefix_spans(t: &Theme, prefix: &str, guides: &[(usize, bool)]) -> Vec<Span<'static>> {
    let dim = Style::new().fg(t.dim);
    let lit = Style::new().fg(t.active_border);
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut run = String::new();
    let mut run_style = dim;
    for (i, c) in prefix.chars().enumerate() {
        let (level, offset) = (i / 3, i % 3);
        let on_way = guides
            .iter()
            .any(|&(l, whole)| l == level && (whole || offset == 0));
        let style = if on_way { lit } else { dim };
        if style != run_style && !run.is_empty() {
            spans.push(Span::styled(std::mem::take(&mut run), run_style));
        }
        run_style = style;
        run.push(c);
    }
    if !run.is_empty() {
        spans.push(Span::styled(run, run_style));
    }
    spans
}

/// Width of the Alt+K Percent bars.
const SHARE_BAR: usize = 8;

/// `part` as a whole percentage of `whole` (0 for an empty whole).
fn percent_of(part: u64, whole: u64) -> u64 {
    if whole == 0 {
        0
    } else {
        (part as u128 * 100 / whole as u128) as u64
    }
}

/// A bar of `part` of `whole` across `width`, with the percentage after it
/// (and `what`, if it fits): `████████░░░ 73% used`. `None` if too narrow.
fn ratio_line(
    part: u64,
    whole: u64,
    width: usize,
    weight: crate::usage::Weight,
    what: &str,
) -> Option<Line<'static>> {
    let pct = percent_of(part, whole).min(100);
    let mut text = format!(" {pct}%");
    if !what.is_empty() && width >= text.len() + what.len() + 9 {
        text.push(' ');
        text.push_str(what);
    }
    let bar_width = width.checked_sub(text.len())?;
    if bar_width < 4 || whole == 0 {
        return None;
    }
    let filled = crate::usage::bar(part.min(whole), whole, bar_width);
    let mut spans = bar_spans(filled, bar_width, weight);
    spans.push(Span::styled(text, Style::new().fg(theme().dim)));
    Some(Line::from(spans))
}

/// Alt+K Percent: `bytes` as a share of `whole` ("41%", "5.8%", "<0.1%"), and a bar
/// of `bytes` against `largest` (the biggest sibling fills it), in eighths
/// of a character. Without `room` for both, only the share.
fn share(bytes: u64, whole: u64, largest: u64, room: usize) -> (Option<String>, String) {
    let pct = if whole == 0 {
        0.0
    } else {
        bytes as f64 * 100.0 / whole as f64
    };
    let text = if pct > 0.0 && pct < 0.05 {
        "<0.1%".to_string()
    } else if pct == 0.0 || pct >= 9.95 {
        format!("{pct:.0}%")
    } else {
        format!("{}%", one_decimal(pct))
    };
    let text = format!("{text:>5}");
    if room < SHARE_BAR + 1 + text.len() {
        return (None, text);
    }
    (Some(crate::usage::bar(bytes, largest, SHARE_BAR)), text)
}

/// A bar for the Alt+K Percent and the Usage view: the filled part in the
/// color of its weight (the theme's bar_low, bar_mid, bar_high), on a track
/// that shows the whole width.
fn bar_spans(filled: String, width: usize, weight: crate::usage::Weight) -> Vec<Span<'static>> {
    use crate::usage::Weight;
    let color = match weight {
        Weight::Light => theme().bar_low,
        Weight::Medium => theme().bar_mid,
        Weight::Heavy => theme().bar_high,
    };
    let track = width.saturating_sub(filled.chars().count());
    vec![
        Span::styled(filled, Style::new().fg(color)),
        Span::styled("░".repeat(track), Style::new().fg(theme().border)),
    ]
}

/// `7/2,027` on a window's border: the cursor's line and how many there are.
fn position(selected: Option<usize>, len: usize) -> Line<'static> {
    let at = if len == 0 {
        0
    } else {
        selected.unwrap_or(0).min(len - 1) + 1
    };
    Line::from(format!("{}/{}", fmt_num(at as u64), fmt_num(len as u64))).right_aligned()
}

/// F2 in a prompt: the tree to pick a folder from, with the prompt's question
/// in the footer.
fn draw_picker(frame: &mut Frame, app: &mut App, main: Rect, menu: Rect) {
    let question = app
        .prompt
        .as_ref()
        .map(|p| p.label.clone())
        .unwrap_or_default();
    let Some(picker) = &mut app.picker else {
        return;
    };
    let windows = picker.panes.len();
    let width = main.width.saturating_sub(4) as usize;
    let path = picker.pane().current_path();
    let title = format!(
        " Folder: {} ",
        fit_tail(&path.to_string_lossy(), width.saturating_sub(10))
    );
    draw_tree(
        frame,
        picker.pane_mut(),
        true,
        title,
        main,
        app.options.icons,
    );

    let mut keys = vec![
        ("↑↓", "move"),
        ("←→", "close/open"),
        ("Enter", "choose"),
        ("Esc", "back"),
    ];
    if windows > 1 {
        keys.push(("Tab", "other window"));
    }
    let lines = vec![
        menu_line("FOLDER", &keys),
        Line::from(Span::styled(
            format!("{:9}{}", "", question.trim_end()),
            Style::new().fg(theme().dim),
        )),
    ];
    let lines: Vec<_> = lines
        .into_iter()
        .map(|l| fit_menu(l, menu.width as usize))
        .collect();
    frame.render_widget(Paragraph::new(lines), menu);
}

/// With `git`, files in a repository show their status before the name.
fn draw_files(
    frame: &mut Frame,
    pane: &mut Pane,
    active: bool,
    area: Rect,
    git: Option<&StatusCache>,
    icons: IconStyle,
) {
    let git = git.filter(|g| {
        pane.archive.is_none() && pane.remote.is_none() && g.in_repo(&pane.current_path())
    });
    // Status letters of a file in the repository ("  " if unchanged).
    let marker =
        |pane: &Pane, f: FileRef| git.map(|g| g.marker(&pane.file_path(f)).unwrap_or(*b"  "));
    let kind = match pane.view {
        FileView::Dir => "Files",
        FileView::Branch(_) => "Branch",
        FileView::Showall => "Showall",
    };
    let kind = if pane.tags_only {
        format!("{kind} (tagged)")
    } else {
        kind.to_string()
    };
    let mut title = format!(
        " {}{kind}: {} ({}) {} ",
        crate::icons::view_icon(pane.view, icons).unwrap_or_default(),
        pane.filespec.text(),
        fmt_num(pane.files.len() as u64),
        pane.sort.label()
    );
    // Not empty: on its way, from a cloud mount.
    if pane.view == FileView::Dir && pane.tree.is_reading(pane.current()) {
        title.push_str("reading... ");
    }
    let error = match pane.view {
        FileView::Dir => pane.tree.node(pane.current()).error.is_some(),
        _ => false,
    };
    if pane.display.multi_column() && !pane.files.is_empty() && !error {
        draw_file_columns(frame, pane, active, area, title, &marker, icons);
        return;
    }
    pane.file_cols = 1;
    pane.file_page = area.height.saturating_sub(2) as usize;

    let width = area.width.saturating_sub(2) as usize;
    let (range, mut state) = visible_window(&mut pane.file_state, pane.files.len(), pane.file_page);
    let error = match pane.view {
        FileView::Dir => pane.tree.node(pane.current()).error.as_ref(),
        _ => None,
    };
    // Not read until asked for (+ or *): nothing of it is known.
    let not_logged = pane.view == FileView::Dir
        && !pane.tree.node(pane.current()).scanned
        && !pane.tree.is_reading(pane.current());
    let items: Vec<ListItem> = if let Some(err) = error {
        vec![ListItem::new(Span::styled(
            format!("Cannot read directory: {err}"),
            Style::new().fg(theme().error),
        ))]
    } else if not_logged {
        vec![ListItem::new(Span::styled(
            "Not logged",
            Style::new().fg(theme().dim),
        ))]
    } else if pane.files.is_empty() {
        vec![ListItem::new(Span::styled(
            "No matching files",
            Style::new().fg(theme().dim),
        ))]
    } else {
        let visible = pane.files[range].to_vec();
        let details: Vec<Detail> = visible.iter().map(|&f| file_detail(pane, f)).collect();
        visible
            .iter()
            .zip(details)
            .map(|(&f, detail)| file_line(pane.file(f), width, detail, marker(pane, f), icons))
            .collect()
    };

    let position = position(pane.file_state.selected(), pane.files.len());
    let list = List::new(items)
        .block(panel(title, active).title_bottom(position))
        .highlight_style(highlight(active));
    frame.render_stateful_widget(list, area, &mut state);
    let offset = pane.file_state.offset();
    draw_scroll_thumb(
        frame,
        area,
        pane.files.len(),
        pane.file_page,
        offset,
        active,
    );
}

/// Adjusts the scroll of `state` so the selected item stays visible.
/// Returns the range of items that fits in `height` lines and the state to use
/// to draw only that range (large lists are not built in full).
fn visible_window(state: &mut ListState, len: usize, height: usize) -> (Range<usize>, ListState) {
    let height = height.max(1);
    let sel = state.selected().unwrap_or(0).min(len.saturating_sub(1));
    let mut offset = state.offset().min(sel);
    if sel >= offset + height {
        offset = sel + 1 - height;
    }
    // No empty space at the end when the list shrinks.
    offset = offset.min(len.saturating_sub(height));
    *state.offset_mut() = offset;
    let range = offset..(offset + height).min(len);
    (
        range,
        ListState::default().with_selected(Some(sel - offset)),
    )
}

/// Longer names are cut with `…` in the column modes, so that several columns
/// fit (as in XTree, with fixed-width columns).
const MAX_COLUMN_NAME: usize = 20;

/// Multi-column modes (Alt+F): files run left to right and then onto the next
/// row down, as in XTree.
fn draw_file_columns(
    frame: &mut Frame,
    pane: &mut Pane,
    active: bool,
    area: Rect,
    title: String,
    marker: &dyn Fn(&Pane, FileRef) -> Option<[u8; 2]>,
    icons: IconStyle,
) {
    let git_w = if pane
        .files
        .first()
        .is_some_and(|&f| marker(pane, f).is_some())
    {
        3
    } else {
        0
    };
    let icon_w = if icons == IconStyle::None {
        0
    } else {
        SLOT_WIDTH
    };
    let block = panel(title, active);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let (width, height) = (inner.width as usize, (inner.height as usize).max(1));
    let with_size = pane.display == FileDisplay::NamesSize;

    let n = pane.files.len();
    let name_len = |f: &FileEntry| {
        f.name.to_string_lossy().chars().count() + usize::from(f.kind == FileKind::Symlink)
    };
    let name_w = pane
        .files
        .iter()
        .map(|&f| name_len(pane.file(f)))
        .max()
        .unwrap_or(1)
        .min(MAX_COLUMN_NAME);
    // Tag mark + git + icon + name + (size and permissions) + space between columns.
    let cell_w = 2 + git_w + icon_w + name_w + if with_size { 23 } else { 0 } + 1;
    let cols = (width / cell_w).max(1);
    pane.file_cols = cols;
    pane.file_page = height * cols;

    // Scroll by rows, with the selected file always visible.
    let sel = pane.file_state.selected().unwrap_or(0).min(n - 1);
    let rows = n.div_ceil(cols);
    let sel_row = sel / cols;
    let mut top = pane.file_state.offset().min(sel_row);
    if sel_row >= top + height {
        top = sel_row + 1 - height;
    }
    top = top.min(rows.saturating_sub(height));
    *pane.file_state.offset_mut() = top;

    let lines: Vec<Line> = (top..(top + height).min(rows))
        .map(|r| {
            let mut spans = Vec::new();
            for i in r * cols..(r * cols + cols).min(n) {
                let git = marker(pane, pane.files[i]);
                let f = pane.file(pane.files[i]);
                let mut name = f.name.to_string_lossy().into_owned();
                if f.kind == FileKind::Symlink {
                    name.push('@');
                }
                let icon = crate::icons::file_icon(
                    &f.name,
                    f.kind,
                    crate::platform::is_executable(&f.name, f.mode),
                    icons,
                )
                .unwrap_or_default();
                let mut cell = format!(
                    "{}{}{}{}",
                    if f.tagged { "■ " } else { "  " },
                    git.map(|xy| git_text(&xy)).unwrap_or_default(),
                    icon,
                    fit(&name, name_w)
                );
                if with_size {
                    cell.push_str(&format!(
                        " {:>11} {}",
                        fmt_num(f.size),
                        fmt_mode(f.mode, f.kind)
                    ));
                }
                cell.push(' ');
                let style = if i == sel {
                    highlight(active)
                } else {
                    file_style(&theme(), f)
                };
                spans.push(Span::styled(cell, style));
            }
            Line::from(spans)
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), inner);
    draw_scroll_thumb(frame, area, rows, height, top, active);
}

/// Name color by file type (and whether it is tagged): tagged, link,
/// special and executable first, then the kind of file its name says.
fn file_style(t: &Theme, f: &FileEntry) -> Style {
    match f.kind {
        _ if f.tagged => Style::new().fg(t.tagged).add_modifier(Modifier::BOLD),
        FileKind::Symlink => Style::new().fg(t.symlink),
        FileKind::Other => Style::new().fg(t.special),
        FileKind::File if crate::platform::is_executable(&f.name, f.mode) => {
            Style::new().fg(t.executable)
        }
        FileKind::File if t.type_colors => match crate::filetype::category(&f.name) {
            Some(category) => Style::new().fg(category_color(t, category)),
            None => Style::new(),
        },
        FileKind::File => Style::new(),
    }
}

fn category_color(t: &Theme, category: crate::filetype::Category) -> Color {
    use crate::filetype::Category;
    match category {
        Category::Archive => t.archive,
        Category::Image => t.image,
        Category::Media => t.media,
        Category::Document => t.document,
        Category::Code => t.code,
        Category::Data => t.data,
    }
}

/// One row of the files window: name, size, date and permissions (or just
/// the name, in long-names mode).
/// `M ` → `"M  "`: the two status letters and a space.
fn git_text(xy: &[u8; 2]) -> String {
    format!("{}{} ", char::from(xy[0]), char::from(xy[1]))
}

/// What the one-column file window shows after each name (Alt+F).
enum Detail {
    /// Size, date and permissions.
    Standard,
    NamesOnly,
    /// "user:group" ("-" where unknown: inside archives, on Windows).
    Owner(String),
    /// What the file is ("PNG image, 800 x 600").
    Type(String),
}

/// The detail of file `f` in the current display mode. Owner and Type are
/// worked out here, for the visible files only (types are remembered).
fn file_detail(pane: &mut Pane, f: FileRef) -> Detail {
    match pane.display {
        FileDisplay::LongNames => Detail::NamesOnly,
        FileDisplay::Owner => {
            let owner = match (&pane.remote, &pane.archive) {
                (Some(remote), _) => remote.owner(&pane.file_path(f)),
                (None, Some(_)) => None,
                (None, None) => platform::owner(&pane.file_path(f)),
            };
            Detail::Owner(owner.unwrap_or_else(|| "-".into()))
        }
        FileDisplay::Type => {
            let path = pane.file_path(f);
            let e = pane.file(f);
            let (size, modified, kind) = (e.size, e.modified, e.kind);
            if let Some((s, m, what)) = pane.types.get(&path)
                && (*s, *m) == (size, modified)
            {
                return Detail::Type(what.clone());
            }
            let what = match &pane.remote {
                Some(remote) => filetype::describe_with(
                    &path,
                    kind,
                    size,
                    || remote.read(&path, filetype::HEAD_BYTES).ok(),
                    || remote.read_link(&path).ok(),
                ),
                None => filetype::describe(&path, kind, size, pane.archive.is_none()),
            };
            if pane.types.len() > 10_000 {
                pane.types.clear();
            }
            pane.types.insert(path, (size, modified, what.clone()));
            Detail::Type(what)
        }
        _ => Detail::Standard,
    }
}

fn file_line(
    f: &FileEntry,
    width: usize,
    detail: Detail,
    git: Option<[u8; 2]>,
    icons: IconStyle,
) -> ListItem<'static> {
    let size = fmt_num(f.size);
    let mode = fmt_mode(f.mode, f.kind);
    let dim = Style::new().fg(theme().dim);
    let t = theme();
    let size_style = size_style(&t, f.size);
    let date_style = date_style(&t, f.modified, SystemTime::now());
    // In narrow windows (split screen), the least useful columns are dropped.
    let details: Vec<Span<'static>> = match detail {
        Detail::NamesOnly => Vec::new(),
        Detail::Standard => {
            let date = f.modified.map(fmt_time).unwrap_or_default();
            if width >= 72 {
                vec![
                    Span::styled(format!(" {size:>15}"), size_style),
                    Span::styled(format!(" {date:16}"), date_style),
                    Span::styled(format!(" {mode}"), dim),
                ]
            } else if width >= 50 {
                vec![
                    Span::styled(format!(" {size:>13}"), size_style),
                    Span::styled(format!(" {date:16}"), date_style),
                ]
            } else {
                vec![Span::styled(format!(" {size:>13}"), size_style)]
            }
        }
        Detail::Owner(owner) => {
            let owner = fit(&owner, 17);
            if width >= 72 {
                vec![
                    Span::styled(format!(" {size:>15}"), size_style),
                    Span::styled(format!(" {owner} {mode}"), dim),
                ]
            } else if width >= 50 {
                vec![
                    Span::styled(format!(" {size:>13}"), size_style),
                    Span::styled(format!(" {owner}"), dim),
                ]
            } else {
                vec![Span::styled(format!(" {owner}"), dim)]
            }
        }
        // Up to almost half of the line, the rest for the name.
        Detail::Type(what) => vec![Span::styled(
            format!(" {}", fit(&what, (width * 45 / 100).clamp(12, 50))),
            dim,
        )],
    };
    let details_len: usize = details.iter().map(|s| s.content.chars().count()).sum();

    // 2 columns for the tag mark, 3 for the Git status, then the icon.
    let git = git.map(|xy| git_text(&xy));
    let icon = crate::icons::file_icon(
        &f.name,
        f.kind,
        crate::platform::is_executable(&f.name, f.mode),
        icons,
    );
    let icon_len = if icon.is_some() { SLOT_WIDTH } else { 0 };
    let marks = 2 + git.as_ref().map_or(0, String::len) + icon_len;
    let name_width = width.saturating_sub(details_len + marks).max(8);
    let mut name: String = f.name.to_string_lossy().into_owned();
    if f.kind == FileKind::Symlink {
        name.push('@');
    }
    let name = fit(&name, name_width);

    let style = file_style(&t, f);
    let mark = if f.tagged { "■ " } else { "  " };
    let mut spans = vec![Span::styled(mark, Style::new().fg(theme().tagged))];
    if let Some(git) = git {
        spans.push(Span::styled(
            git,
            Style::new().fg(theme().key).add_modifier(Modifier::BOLD),
        ));
    }
    if let Some(ic) = icon {
        spans.push(Span::styled(ic, style));
    }
    spans.push(Span::styled(name, style));
    spans.extend(details);
    ListItem::new(Line::from(spans))
}

/// A size by how much it is: under a MiB as dim as the other details, up to
/// a GiB in the color of the text, more in bold. A big file stands out without
/// reading the numbers.
fn size_style(t: &Theme, size: u64) -> Style {
    const MIB: u64 = 1 << 20;
    const GIB: u64 = 1 << 30;
    match size {
        s if s < MIB => Style::new().fg(t.dim),
        s if s < GIB => Style::new().fg(t.text),
        _ => Style::new().fg(t.text).add_modifier(Modifier::BOLD),
    }
}

/// A date changed in the last 24 hours in the color of the text, any other
/// as dim as the other details.
fn date_style(t: &Theme, modified: Option<SystemTime>, now: SystemTime) -> Style {
    let recent = modified
        .and_then(|m| now.duration_since(m).ok())
        .is_some_and(|age| age.as_secs() < 24 * 60 * 60);
    Style::new().fg(if recent { t.text } else { t.dim })
}

/// The panel on the right, as in ZTreeWin: three boxes (FILE, DISK, DISK
/// Statistics) separated by lines.
fn draw_stats(frame: &mut Frame, app: &App, area: Rect) {
    let block = panel("", false);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let width = inner.width as usize;

    let label = |s: String| Span::styled(s, Style::new().fg(theme().key));
    let bold = |s: String| Span::styled(s, Style::new().add_modifier(Modifier::BOLD));
    // "Logged", "Current Directory"...: centered.
    let heading = |s: &str| Line::from(label(s.to_string())).centered();
    // Name on the left, number on the right; the number goes to the next
    // line when both don't fit (e.g. "Available" and a disk in terabytes).
    let value = |k: &str, v: u64| -> Vec<Line<'static>> {
        let v = fmt_num(v);
        let (k_len, v_len) = (k.chars().count(), v.chars().count());
        if k_len + 1 + v_len <= width {
            let pad = " ".repeat(width - k_len - v_len);
            vec![Line::from(vec![Span::raw(format!("{k}{pad}")), bold(v)])]
        } else {
            vec![heading(k), Line::from(bold(v)).right_aligned()]
        }
    };
    let count = |c: Count| [value("Files", c.files), value("Bytes", c.bytes)].concat();

    let id = app.header_dir();
    let path = app.tree.path(id);
    let file = vec![Line::from(vec![
        label("FILE ".into()),
        bold(fit(app.filespec.text(), width.saturating_sub(5))),
    ])];

    let mut disk = Vec::new();
    let mut total_lines = 0;
    if let Some(space) = crate::platform::disk_space(&path) {
        let mount = crate::info::mount_of(&path);
        match mount.as_ref().and_then(crate::rclone::of) {
            // A cloud service mounted by rclone: its name, then the remote.
            Some(remote) => {
                let service = remote.service.as_deref().unwrap_or("rclone");
                disk.push(Line::from(vec![
                    label("DISK ".into()),
                    bold(fit(service, width.saturating_sub(5))),
                ]));
                disk.push(Line::from(fit(&format!("rclone {}", remote.spec), width)));
            }
            None => {
                let point = mount
                    .map(|m| m.point.to_string_lossy().into_owned())
                    .unwrap_or_default();
                disk.push(Line::from(vec![
                    label("DISK ".into()),
                    Span::raw(fit_tail(&point, width.saturating_sub(5))),
                ]));
            }
        }
        disk.extend(value("Available", space.available));
        // Lines of "Total": one, or two when the number goes below.
        let total = value("Total", space.total);
        total_lines = total.len();
        disk.extend(total);
        // How full the disk is, as df says it (the blocks kept for root are
        // neither used nor available): green, yellow from 75%, red from 90%.
        let used = space.total.saturating_sub(space.free);
        let whole = used + space.available;
        let weight = match percent_of(used, whole) {
            p if p < 75 => crate::usage::Weight::Light,
            p if p < 90 => crate::usage::Weight::Medium,
            _ => crate::usage::Weight::Heavy,
        };
        if let Some(line) = ratio_line(used, whole, width, weight, "used") {
            disk.push(line);
            total_lines += 1;
        }
    }

    // The statistics of the disk shown above: on this machine, only what is
    // on it (other file systems mounted inside are out); in an archive or on
    // a remote machine, everything logged.
    let (stats, _) = app.disk_stats_at(&path);
    let node = app.tree.node(id);
    let here = node
        .files
        .iter()
        .filter(|f| app.tree.show_hidden || !f.is_hidden())
        .fold(Count::default(), |c, f| Count {
            files: c.files + 1,
            bytes: c.bytes + f.size,
        });
    // Just the name, also for the tree's root ("/" for the root of the disk).
    let name = path
        .file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned();
    let mut statistics = vec![
        Line::from(label("DISK Statistics".into())),
        heading("Logged"),
    ];
    statistics.extend(count(stats.logged));
    statistics.push(heading("Matching"));
    statistics.extend(count(stats.matching));
    statistics.push(heading("Tagged"));
    statistics.extend(count(stats.tagged));
    // The tagged part of what matches, once something is tagged.
    if stats.tagged.bytes > 0
        && let Some(line) = ratio_line(
            stats.tagged.bytes,
            stats.matching.bytes,
            width,
            crate::usage::Weight::Light,
            "",
        )
    {
        statistics.push(line);
    }
    statistics.push(heading("Current Directory"));
    statistics.push(Line::from(bold(fit(&name, width))));
    statistics.extend(count(here));
    if let Ok(meta) = std::fs::metadata(&path) {
        let mode = format!(
            "d{}",
            crate::platform::format_mode(crate::platform::mode(&meta))
        );
        let k = if cfg!(windows) { "Attributes" } else { "Perms" };
        let pad = " ".repeat(width.saturating_sub(k.len() + mode.chars().count()));
        statistics.push(Line::from(vec![Span::raw(format!("{k}{pad}")), bold(mode)]));
    }

    // Alt+D: the whole path and date of the item under the cursor.
    if let Some(item) = app.duplicates.as_ref().and_then(|v| v.selected().cloned()) {
        let mut lines = vec![Line::from(label("DUPLICATE".into()))];
        let path = item.path.to_string_lossy().into_owned();
        let chars: Vec<char> = path.chars().collect();
        for part in chars.chunks(width.max(1)) {
            lines.push(Line::from(bold(part.iter().collect::<String>())));
        }
        let text = |k: &str, v: String| {
            let pad = width.saturating_sub(k.chars().count() + v.chars().count());
            Line::from(vec![Span::raw(format!("{k}{}", " ".repeat(pad))), bold(v)])
        };
        if item.shared {
            lines.push(Line::from("Shared with you: deleting it frees nothing"));
        }
        lines.push(text("Size", crate::usage::size(item.size)));
        lines.push(text("On disk", crate::usage::size(item.disk)));
        if let Some(m) = item.modified {
            lines.push(text("Modified", fmt_time(m)));
        }
        frame.render_widget(Paragraph::new(lines), inner);
        return;
    }

    // Alt+U: what the row under the cursor takes, in place of the statistics.
    if let Some(row) = app.usage.as_ref().and_then(|u| u.shown.clone()) {
        use crate::usage;
        let text = |k: &str, v: String| {
            let pad = width.saturating_sub(k.chars().count() + v.chars().count());
            Line::from(vec![Span::raw(format!("{k}{}", " ".repeat(pad))), bold(v)])
        };
        let mut lines = vec![
            Line::from(label("USAGE".into())),
            Line::from(bold(fit(&row.name, width))),
            text("On disk", usage::size(row.disk)),
            text("Apparent", usage::size(row.apparent)),
            text("Files", fmt_num(row.files)),
        ];
        if let Some(m) = row.modified {
            lines.push(text("Modified", fmt_time(m)));
        }
        if let Some(d) = crate::platform::disk_space(&path) {
            let used = d.total.saturating_sub(d.free);
            lines.push(text("Of used", usage::percent(row.disk, used)));
            lines.push(text("Of disk", usage::percent(row.disk, d.total)));
        }
        let note = match row.mark {
            '>' => "Another disk: Enter analyzes it",
            '!' => "Could not be read",
            '.' => "Something inside could not be read: it may take more",
            'H' => "Another hard link of a file already counted",
            '@' => "A link or special file, not followed",
            'e' => "Empty",
            _ => "",
        };
        if !note.is_empty() {
            lines.push(Line::from(""));
            for chunk in wrap(note, width) {
                lines.push(Line::from(Span::styled(
                    chunk,
                    Style::new().fg(theme().dim),
                )));
            }
        }
        statistics = lines;
    }

    // Short screens: without the "DISK Statistics" title, then without the
    // total size of the disk, so that the current directory still fits.
    let needed =
        |file: &[Line], disk: &[Line], stats: &[Line]| file.len() + disk.len() + stats.len() + 2;
    let room = inner.height as usize;
    if needed(&file, &disk, &statistics) > room {
        statistics.remove(0);
    }
    if needed(&file, &disk, &statistics) > room {
        disk.truncate(disk.len() - total_lines);
    }

    let separator =
        Line::from(format!("├{}┤", "─".repeat(width))).style(Style::new().fg(theme().border));
    let mut y = inner.y;
    for (i, section) in [file, disk, statistics].into_iter().enumerate() {
        if section.is_empty() || y >= inner.bottom() {
            continue;
        }
        if i > 0 {
            let row = Rect::new(area.x, y, area.width, 1);
            frame.render_widget(separator.clone(), row);
            y += 1;
        }
        let height = (section.len() as u16).min(inner.bottom().saturating_sub(y));
        frame.render_widget(
            Paragraph::new(section),
            Rect::new(inner.x, y, inner.width, height),
        );
        y += height;
    }
}

fn draw_menu(frame: &mut Frame, app: &App, area: Rect) {
    let bold = Style::new().add_modifier(Modifier::BOLD);
    let lines = if let Some(prompt) = &app.prompt {
        prompt_lines(frame, prompt, area)
    } else if app.terminal_focus {
        let key_for = |cmd| app.keymap.key_for(cmd).unwrap_or_default();
        let remote = app.terminal.as_ref().is_some_and(term::Terminal::is_remote);
        let (back, full, tagged) = (
            key_for(Command::Terminal),
            key_for(Command::TerminalFull),
            key_for(Command::TerminalTagged),
        );
        let mut keys = vec![
            (back.as_str(), "back to Xylem"),
            (
                full.as_str(),
                if app.terminal_full {
                    "panel"
                } else {
                    "full screen"
                },
            ),
        ];
        if !remote {
            keys.push((tagged.as_str(), "type the tagged files"));
        }
        keys.push(("Shift+PgUp/PgDn", "scroll back"));
        let about = if remote {
            "a shell on the remote machine; the tree follows its cd if the shell reports it (OSC 7); exit ends the session"
        } else {
            "the tree follows the shell's cd; $XY_TAGGED has the tagged files; exit closes the shell"
        };
        vec![
            menu_line("TERMINAL", &keys),
            match &app.message {
                // E.g. the shell went outside the tree.
                Some(msg) => Line::from(Span::styled(
                    format!("{:9}{msg}", ""),
                    Style::new()
                        .fg(theme().message)
                        .add_modifier(Modifier::BOLD),
                )),
                None => menu_line("", &[("", about)]),
            },
        ]
    } else if let Some(log) = &app.cloud_log {
        use std::sync::atomic::Ordering;
        let entries = log.job.progress.entries.load(Ordering::Relaxed);
        let refused = log.job.progress.refused.load(Ordering::Relaxed);
        let refused = if refused > 0 {
            format!(", {} requests refused", fmt_num(refused))
        } else {
            String::new()
        };
        vec![
            Line::from(vec![
                Span::styled("Asking rclone for the whole tree... ", bold),
                Span::raw(format!("{} files and folders{refused}", fmt_num(entries))),
            ]),
            menu_line("", &[("Esc", "cancel")]),
        ]
    } else if let Some(job) = &app.duplicates_job {
        use std::sync::atomic::Ordering;
        let listing = job.progress.listing.load(Ordering::Relaxed);
        let listed = job.progress.listed.load(Ordering::Relaxed);
        let read = job.progress.read.load(Ordering::Relaxed);
        let total = job.progress.total.load(Ordering::Relaxed).max(read);
        let name = job
            .progress
            .current
            .lock()
            .map(|c| c.clone())
            .unwrap_or_default();
        let what = if listing {
            format!(
                "Asking the service for its hashes: {} files",
                fmt_num(listed)
            )
        } else {
            format!(
                "Comparing {} of {}: ",
                crate::usage::size(read),
                crate::usage::size(total)
            )
        };
        vec![
            Line::from(vec![Span::styled(what, bold), Span::raw(name)]),
            menu_line("", &[("Esc", "cancel")]),
        ]
    } else if app.duplicates.is_some()
        && app.job.is_none()
        && app.confirm.is_none()
        && app.op.is_none()
    {
        vec![
            menu_line(
                "COPIES",
                &[
                    ("T", "tag"),
                    ("K", "keep this"),
                    ("A", "all but one"),
                    ("F", "keep this folder"),
                    ("U", "untag"),
                    ("D", "delete"),
                    ("M", "move"),
                ],
            ),
            match &app.message {
                Some(msg) => Line::from(Span::styled(
                    format!("{:9}{msg}", ""),
                    Style::new()
                        .fg(theme().message)
                        .add_modifier(Modifier::BOLD),
                )),
                None => menu_line(
                    "",
                    &[
                        ("G", "files/folders"),
                        ("V", "view"),
                        ("E", "export"),
                        ("\\", "to the tree"),
                        ("F3", "again"),
                        ("Esc", "close"),
                    ],
                ),
            },
        ]
    } else if app.usage.is_some() && app.job.is_none() {
        let about = "> other disk  ! unreadable  . incomplete  H hard link  @ link  e empty";
        vec![
            menu_line(
                "USAGE",
                &[
                    ("Enter", "open"),
                    ("←", "back"),
                    ("T", "tag"),
                    ("Del", "delete"),
                    ("Alt+P", "prune"),
                    ("V", "view"),
                    ("D", "duplicates"),
                    ("\\", "to the tree"),
                    ("Esc", "close"),
                ],
            ),
            match &app.message {
                Some(msg) => Line::from(Span::styled(
                    format!("{:9}{msg}", ""),
                    Style::new()
                        .fg(theme().message)
                        .add_modifier(Modifier::BOLD),
                )),
                None => menu_line(
                    "",
                    &[
                        ("A", "disk/apparent"),
                        ("C", "files"),
                        ("S", "sort"),
                        ("F3", "reread"),
                        ("", about),
                    ],
                ),
            },
            menu_line(
                "",
                &[
                    ("Ctrl+T/U", "tag/untag the whole folder"),
                    ("Ctrl+D", "delete the tagged"),
                    ("Shift+Del", "without trash"),
                ],
            ),
        ]
    } else if let Some(job) = &app.archive_job {
        vec![
            Line::from(vec![
                Span::styled(
                    format!(
                        "Archiving {} of {}: ",
                        (job.done() + 1).min(job.total()),
                        job.total()
                    ),
                    bold,
                ),
                Span::raw(job.current().unwrap_or("").to_string()),
            ]),
            menu_line(
                "",
                &[
                    (
                        "",
                        &format!(
                            "{} of {} bytes",
                            fmt_num(job.done_bytes),
                            fmt_num(job.total_bytes)
                        ),
                    ),
                    ("Esc", "cancel"),
                ],
            ),
        ]
    } else if let Some(job) = &app.search_job {
        let name = job
            .current()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        vec![
            Line::from(vec![
                Span::styled(
                    format!("Searching {} of {}: ", job.done() + 1, job.total()),
                    bold,
                ),
                Span::raw(name),
                Span::styled(
                    format!("   {} found", job.found),
                    Style::new().fg(theme().dim),
                ),
            ]),
            menu_line("", &[("Esc", "cancel")]),
        ]
    } else if let Some(confirm) = &app.confirm {
        // Question on one line and options on the other, so long paths don't
        // push the options off the screen.
        let mut options = menu_words("", confirm.options);
        options.spans.extend(
            menu_line("", &[("Esc", "cancel")])
                .spans
                .into_iter()
                .skip(1),
        );
        vec![Line::styled(confirm.question.clone(), bold), options]
    } else if let Some(op) = &app.op {
        op_lines(op)
    } else if let Some(job) = &app.job {
        let counts = format!(
            "{} dirs, {} waiting  ",
            fmt_num(job.log.dirs),
            fmt_num(job.log.waiting() as u64)
        );
        let room = (area.width as usize).saturating_sub("Logging... ".len() + counts.len());
        let next = job.log.next().map_or(String::new(), |id| {
            let path = app.panes[job.pane].tree.path(id);
            fit_tail(&crate::bidi::escape(&path.to_string_lossy()), room)
        });
        vec![
            Line::from(vec![
                Span::styled("Logging... ", bold),
                Span::raw(counts),
                Span::raw(next),
            ]),
            // Why it is reading folder by folder, when it was not asked to.
            match &app.message {
                Some(msg) => {
                    let mut line = menu_line("", &[("Esc", "cancel")]);
                    line.spans.push(Span::raw(format!("  {msg}")));
                    line
                }
                None => menu_line("", &[("Esc", "cancel")]),
            },
        ]
    } else {
        let mut lines = idle_menu(app, area.width as usize);
        let first = match &app.message {
            Some(msg) => Line::from(Span::styled(
                msg.clone(),
                Style::new()
                    .fg(theme().message)
                    .add_modifier(Modifier::BOLD),
            )),
            None if app.selected_name_has_bidi() => Line::from(Span::styled(
                "Name has hidden bidirectional characters: it may not be what it looks like",
                Style::new().fg(theme().error),
            )),
            None => lines[0].clone(),
        };
        lines[0] = first;
        lines
    };
    let lines: Vec<_> = lines
        .into_iter()
        .map(|l| fit_menu(l, area.width as usize))
        .collect();
    frame.render_widget(Paragraph::new(lines), area);
}

/// A footer menu as data: the words (each with its highlighted letter), the
/// hints and the other keys. Each word and key says what it binds, so the
/// menu leaves out what cannot run here and a test can check that no command
/// is missing from the menus.
struct MenuSpec {
    /// The window it is for: its keys are looked up there.
    focus: Focus,
    title: &'static str,
    /// The modifier of the words' letters: "", "ctrl+" or "alt+".
    mods: &'static str,
    words: Vec<&'static str>,
    hints: Vec<&'static str>,
    /// Label, description and the keys it stands for (none: a hint).
    keys: Vec<(String, &'static str, Vec<&'static str>)>,
}

/// The key a word of a menu stands for: its highlighted letter.
fn word_key(word: &str) -> char {
    word.chars()
        .find(|c| c.is_uppercase())
        .or_else(|| word.chars().next())
        .unwrap_or(' ')
        .to_ascii_lowercase()
}

const DIR_WORDS: &[&str] = &[
    "Available",
    "Branch",
    "Compare",
    "Delete",
    "Filespec",
    "Invert",
    "Log",
    "Makedir",
    "New date",
    "Oops",
    "Rename",
    "Showall",
    "Tag",
    "Untag",
    "eXecute",
    "Yank",
    "Quit",
];
const FILE_WORDS: &[&str] = &[
    "Attributes",
    "Branch",
    "Copy",
    "Delete",
    "Edit",
    "Filespec",
    "Invert",
    "Move",
    "New date",
    "Open",
    "Rename",
    "Showall",
    "Tag",
    "Untag",
    "View",
    "eXecute",
    "Yank",
    "Quit",
];
const CTRL_TREE_WORDS: &[&str] = &[
    "Attributes",
    "Bookmark",
    "Copy",
    "Delete",
    "Git",
    "Move",
    "Rename",
    "Showall",
    "Tag",
    "Untag",
    "Yank",
];
const CTRL_FILE_WORDS: &[&str] = &[
    "Attributes",
    "Batch",
    "Copy",
    "Delete",
    "Git",
    "Move",
    "New date",
    "Open",
    "Rename",
    "Search",
    "Tag",
    "Untag",
    "View",
    "Yank",
];
const ALT_TREE_WORDS: &[&str] = &[
    "Bookmarks",
    "Compare",
    "Duplicates",
    "File display",
    "Help",
    "Info",
    "Kbyte",
    "Prune",
    "Quit here",
    "Release",
    "Sort",
    "Usage",
];
const ALT_FILE_WORDS: &[&str] = &[
    "Bookmarks",
    "Copy",
    "Duplicates",
    "File display",
    "Help",
    "Info",
    "Move",
    "Quit here",
    "Sort",
    "Usage",
];

/// Footer menu when nothing is in progress: the normal one or the Ctrl/Alt
/// one, depending on the key held down (or chosen with F9).
fn idle_menu_spec(app: &App, focus: Focus, mode: MenuMode) -> MenuSpec {
    let tree = focus == Focus::Tree;
    // With F9, the next letter gets the modifier: tell how to back out.
    let sticky = app.held.is_none() && app.sticky != MenuMode::Normal;
    let next = if sticky {
        ("Esc".to_string(), "cancel", vec![])
    } else {
        ("F9".to_string(), "next menu", vec![])
    };
    let help_key = help_key(app);
    let help_bind = if help_key == "F1" { "f1" } else { "alt+h" };
    let key = |label: &str, desc: &'static str, binds: &[&'static str]| {
        (label.to_string(), desc, binds.to_vec())
    };
    match mode {
        MenuMode::Git => MenuSpec {
            focus,
            title: "GIT",
            mods: "",
            words: crate::git::MENU.to_vec(),
            hints: vec![if tree {
                "on this directory (or the tagged files)"
            } else {
                "on the tagged files (or this one)"
            }],
            keys: vec![key("Esc", "cancel", &[])],
        },
        MenuMode::Normal => {
            let title = if app.remote.is_some() {
                "REMOTE"
            } else if app.archive.is_some() {
                "ARCHIVE"
            } else if tree {
                "DIR"
            } else {
                "FILE"
            };
            let mut keys = vec![key(&help_key, "help", &[help_bind])];
            if tree {
                keys.push(key("*", "log branch", &["*"]));
            } else {
                keys.push(key("Esc", "tree", &[]));
            }
            if tree && app.archive.is_some() {
                keys.push(key("Esc", "close archive", &[]));
            }
            keys.push(key("?", "stats", &["?"]));
            keys.push(key("\\", "treespec", &["\\"]));
            keys.push(key("F3", "relog", &["f3"]));
            if tree {
                keys.push(key("Enter", "files", &["enter"]));
            } else {
                keys.push(key("F5", "archive", &["f5"]));
                keys.push(key("F7", "autoview", &["f7"]));
            }
            keys.push(key("F8", "split", &["f8"]));
            keys.push(key("F9", "menus", &[]));
            MenuSpec {
                focus,
                title,
                mods: "",
                words: (if tree { DIR_WORDS } else { FILE_WORDS }).to_vec(),
                hints: vec![],
                keys,
            }
        }
        MenuMode::Ctrl => {
            let mut keys = if tree {
                vec![key(
                    "Ctrl+F7/F8/F9",
                    "tag/untag/invert branch",
                    &["ctrl+f7", "ctrl+f8", "ctrl+f9"],
                )]
            } else {
                vec![key("Ctrl+F5", "archive them", &["ctrl+f5"])]
            };
            keys.push(next);
            MenuSpec {
                focus,
                title: "CTRL",
                mods: "ctrl+",
                words: (if tree {
                    CTRL_TREE_WORDS
                } else {
                    CTRL_FILE_WORDS
                })
                .to_vec(),
                hints: if tree {
                    vec![
                        "on tagged files here",
                        "Showall/Tag/Untag: all logged",
                        "Yank: paths to clipboard",
                    ]
                } else {
                    vec![
                        "on tagged files in list",
                        "Copy/Move: no paths",
                        "Yank: paths to clipboard",
                    ]
                },
                keys,
            }
        }
        // As in ZTreeWin, Alt is the other version of a command: the branch
        // in the tree, the tagged files with their paths in the file window,
        // and the tools. Alt+X and Alt+O have no word with their letter;
        // Alt+Z (full screen) is in the terminal's own menu.
        MenuMode::Alt => {
            let mut keys = vec![
                key("X", "terminal", &["alt+x"]),
                key("O", "type tagged files in terminal", &["alt+o"]),
            ];
            if !tree {
                keys.push(key("Alt+F5", "open archive", &["alt+f5"]));
            }
            keys.push(key("F12", "setup", &["f12"]));
            keys.push(next);
            let hints = if app.archive.is_some() {
                vec!["read-only archive: Copy extracts"]
            } else if tree {
                vec!["Compare/Duplicates/Prune/Release/Usage: the whole branch"]
            } else {
                vec!["Copy/Move: the tagged files, with their paths"]
            };
            MenuSpec {
                focus,
                title: "ALT",
                mods: "alt+",
                words: (if tree { ALT_TREE_WORDS } else { ALT_FILE_WORDS }).to_vec(),
                hints,
                keys,
            }
        }
    }
}

impl MenuSpec {
    /// Leaves out what does not run here: keys not bound (Git with its menu
    /// off) and what this window refuses (writing in an archive; Edit or
    /// Batch on a remote machine).
    fn only_what_runs(mut self, app: &App) -> Self {
        if self.title == "GIT" {
            return self;
        }
        let git = app.options.git && app.archive.is_none() && app.remote.is_none();
        // Ctrl+G follows the option (the key map leaves it out while off).
        let focus = self.focus;
        let runs = |bind: &str| match app.keymap.lookup_text(bind, focus) {
            _ if bind == "ctrl+g" => git,
            Some(Command::Git) => git,
            Some(cmd) => app.refusal(cmd).is_none(),
            None => false,
        };
        let mods = self.mods;
        self.words
            .retain(|w| runs(&format!("{mods}{}", word_key(w))));
        self.keys
            .retain(|(_, _, binds)| binds.is_empty() || binds.iter().any(|b| runs(b)));
        self
    }

    /// The words on one line, and the keys on another with the hints
    /// before them (left out, as only explanations, when they do not fit).
    fn two_lines(&self, width: usize) -> Vec<Line<'static>> {
        let keys: Vec<(&str, &str)> = self.keys.iter().map(|(k, d, _)| (k.as_str(), *d)).collect();
        let hint = self.hints.join(" · ");
        let mut with_hints = keys.clone();
        if !hint.is_empty() {
            with_hints.insert(0, ("", &hint));
        }
        let second = menu_line("", &with_hints);
        let second = if fits(&second.spans, width) {
            second
        } else {
            menu_line("", &keys)
        };
        vec![menu_words(self.title, &self.words), second]
    }

    /// Whether the menu fits in two lines of `width` columns (the hints may
    /// go).
    fn fits_in_two(&self, width: usize) -> bool {
        let keys: Vec<(&str, &str)> = self.keys.iter().map(|(k, d, _)| (k.as_str(), *d)).collect();
        fits(&menu_words(self.title, &self.words).spans, width)
            && fits(&menu_line("", &keys).spans, width)
    }

    /// The footer's three lines: the words over one or two lines (two, in
    /// alphabetical order, when there are no hints), the hints, the keys.
    fn lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        if self.hints.is_empty() {
            let (first, second) = split_words(&self.words);
            lines.push(menu_words(self.title, first));
            lines.push(menu_words("", second));
        } else {
            lines.push(menu_words(self.title, &self.words));
            lines.push(menu_line("", &[("", &self.hints.join(" · "))]));
        }
        let keys: Vec<(&str, &str)> = self.keys.iter().map(|(k, d, _)| (k.as_str(), *d)).collect();
        lines.push(menu_line("", &keys));
        lines
    }
}

/// Splits the words in two lines of about the same width, keeping their
/// order.
fn split_words<'a>(words: &'a [&'static str]) -> (&'a [&'static str], &'a [&'static str]) {
    let total: usize = words.iter().map(|w| w.chars().count() + 1).sum();
    let mut used = 0;
    for (i, w) in words.iter().enumerate() {
        used += w.chars().count() + 1;
        if used * 2 >= total {
            return words.split_at(i + 1);
        }
    }
    (words, &[])
}

fn idle_menu(app: &App, width: usize) -> Vec<Line<'static>> {
    let menu = idle_menu_spec(app, app.focus, app.menu_mode()).only_what_runs(app);
    if footer_height(app, width) == 2 {
        menu.two_lines(width)
    } else {
        menu.lines()
    }
}

/// The footer's height for a screen `width` columns wide: 2 when every menu
/// (DIR, FILE, CTRL and ALT, in the tree and in the file window) fits in two
/// lines, 3 otherwise. The same for all of them, so that holding Ctrl or
/// moving between windows never moves the tree.
fn footer_height(app: &App, width: usize) -> u16 {
    let all_fit = [Focus::Tree, Focus::Files].into_iter().all(|focus| {
        [MenuMode::Normal, MenuMode::Ctrl, MenuMode::Alt]
            .into_iter()
            .all(|mode| {
                idle_menu_spec(app, focus, mode)
                    .only_what_runs(app)
                    .fits_in_two(width)
            })
    });
    if all_fit { 2 } else { 3 }
}

/// Footer prompt: the label, the typed text (with the cursor) and the keys.
fn prompt_lines(frame: &mut Frame, prompt: &app::Prompt, area: Rect) -> Vec<Line<'static>> {
    let bold = Style::new().add_modifier(Modifier::BOLD);
    let col = area.x + (prompt.label.chars().count() + prompt.input.cursor) as u16;
    frame.set_cursor_position((col.min(area.right().saturating_sub(1)), area.y));
    let help = if prompt.is_bookmark_jump() {
        vec![
            ("0-9, a-z", "jump"),
            ("' / Enter", "list"),
            ("+", "bookmark here"),
            ("Esc", "cancel"),
        ]
    } else if prompt.is_bookmark_set() {
        vec![("0-9, a-z", "assign key"), ("Esc", "cancel")]
    } else if prompt.is_bookmark_name() {
        vec![("Enter", "save"), ("Esc", "cancel"), ("Ctrl+U", "clear")]
    } else if let Some(guide) = prompt.batch_guide() {
        if guide.reading.is_some() {
            vec![
                ("↑↓ PgUp PgDn", "scroll"),
                ("Enter", "use it"),
                ("F3 Esc", "back"),
            ]
        } else if guide.templates.is_empty() {
            vec![
                ("Enter", "preview"),
                ("Esc", "cancel"),
                ("Ctrl+U", "clear"),
                ("↑↓", "history"),
                ("F1", "help"),
            ]
        } else {
            vec![
                ("Enter", "preview"),
                ("Esc", "cancel"),
                ("↑↓", "template"),
                ("F3", "read it"),
                ("PgUp PgDn", "history"),
                ("F1", "help"),
            ]
        }
    } else if prompt.is_rename_tagged() {
        vec![
            ("Enter", "rename"),
            ("F2", "case"),
            ("↑↓", "history"),
            ("Esc", "cancel"),
            ("e.g.", "*.jpg  new_*.*  %03d  s/a/b/"),
        ]
    } else {
        let mut help = vec![("Enter", "accept"), ("Esc", "cancel"), ("Ctrl+U", "clear")];
        if prompt.picks_folder() {
            help.push(("F2", "tree"));
        }
        if prompt.picks_bookmark() {
            help.push(("F11", "bookmarks"));
            help.push(("↑↓", "history"));
        }
        if prompt.toggles_paths() {
            help.push(("F6", "with/without paths"));
            help.push(("F7", "paths from"));
        }
        if prompt.is_search() {
            help.push(("F2", "match case"));
            help.push(("F4", "text/hex"));
            help.push(("↑↓", "history"));
        }
        if prompt.is_line_filter() {
            help.push(("F1", "regex help"));
            help.push(("F2", "match case"));
            help.push(("F3", "invert"));
            help.push(("F4", "text/regex"));
            help.push(("↑↓", "history"));
        }
        if prompt.is_goto() {
            help.push(("↑↓", "history"));
        }
        if prompt.label.starts_with("FILESPEC") {
            help.push(("e.g.", "*.rs -*.bak >TODAY-7 >s1M"));
        }
        help
    };
    vec![
        Line::from(vec![
            Span::styled(prompt.label.clone(), bold),
            Span::raw(if prompt.is_secret() {
                "•".repeat(prompt.input.text.chars().count())
            } else {
                prompt.input.text.clone()
            }),
        ]),
        menu_line("", &help),
    ]
}

/// Footer during Copy/Move/Delete: current file and progress bar,
/// or the conflict question with the size and date of both files.
fn op_lines(op: &Operation) -> Vec<Line<'static>> {
    let bold = Style::new().add_modifier(Modifier::BOLD);
    let name = |p: &std::path::Path| {
        p.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    };

    let first = match &op.conflict {
        Some(c) => Line::from(vec![
            Span::styled(
                format!("{} already exists. Overwrite? ", name(&c.dst)),
                bold,
            ),
            key("Y"),
            Span::raw("es  "),
            key("N"),
            Span::raw("o  "),
            key("A"),
            Span::raw("ll  "),
            key("S"),
            Span::raw("kip all  "),
            key("Esc"),
            Span::raw(" cancel"),
        ]),
        None => Line::from(vec![
            Span::styled(
                format!(
                    "{} {} of {}: ",
                    if op.to_trash {
                        "Trashing"
                    } else {
                        op.kind.verb()
                    },
                    (op.done_files + op.skipped + op.errors.len() + 1).min(op.total_files()),
                    op.total_files()
                ),
                bold,
            ),
            Span::raw(op.current().map(name).unwrap_or_default()),
        ]),
    };

    if let Some(c) = &op.conflict {
        return vec![first, conflict_facts(c)];
    }

    const BAR: usize = 30;
    let fraction = if op.total_bytes == 0 {
        0.0
    } else {
        op.done_bytes as f64 / op.total_bytes as f64
    };
    let filled = ((fraction * BAR as f64) as usize).min(BAR);
    let second = Line::from(vec![
        Span::raw(format!("{:9}", "")),
        Span::styled("█".repeat(filled), Style::new().fg(theme().key)),
        Span::styled("░".repeat(BAR - filled), Style::new().fg(theme().dim)),
        Span::raw(format!(
            " {:>3}%  {} of {} bytes   ",
            (fraction * 100.0) as u32,
            fmt_num(op.done_bytes),
            fmt_num(op.total_bytes)
        )),
        key("Esc"),
        Span::raw(" cancel"),
    ]);
    vec![first, second]
}

/// "new: 3 bytes, 2026-10-12 13:02   existing: 16 bytes, 2026-09-30 08:15
/// (older, larger)": what is being copied against what is already there.
fn conflict_facts(c: &Conflict) -> Line<'static> {
    use std::cmp::Ordering::*;
    let facts = |f: &Facts| {
        let date = f.modified.map_or_else(|| "date unknown".into(), fmt_time);
        format!("{} bytes, {date}", fmt_num(f.size))
    };
    let when = match (c.existing.modified, c.new.modified) {
        (Some(existing), Some(new)) => match existing.cmp(&new) {
            Greater => "newer",
            Less => "older",
            Equal => "same date",
        },
        _ => "date unknown",
    };
    let size = match c.existing.size.cmp(&c.new.size) {
        Greater => "larger",
        Less => "smaller",
        Equal => "same size",
    };
    Line::from(vec![
        Span::raw(format!("{:9}", "")),
        Span::styled("new: ", Style::new().fg(theme().dim)),
        Span::raw(format!("{}   ", facts(&c.new))),
        Span::styled("existing: ", Style::new().fg(theme().dim)),
        Span::raw(format!("{} ", facts(&c.existing))),
        Span::styled(
            format!("({when}, {size})"),
            Style::new().add_modifier(Modifier::BOLD),
        ),
    ])
}

/// XTree-style menu line: the key of each word is the first uppercase
/// letter ("Copy" → C, "eXecute" → X).
fn menu_words(title: &str, words: &[&str]) -> Line<'static> {
    let mut spans = vec![Span::styled(
        format!("{title:<9}"),
        Style::new().add_modifier(Modifier::BOLD),
    )];
    spans.push(entry_end());
    for word in words {
        let (at, c) = word
            .char_indices()
            .find(|(_, c)| c.is_uppercase())
            .unwrap_or((0, word.chars().next().unwrap_or(' ')));
        let after = at + c.len_utf8();
        if at > 0 {
            spans.push(Span::raw(word[..at].to_string()));
        }
        spans.push(key(&word[at..after]));
        spans.push(Span::raw(format!("{} ", &word[after..])));
        spans.push(word_end());
    }
    Line::from(spans)
}

/// Configuration screen: the page's options, with letter, description and
/// value; with F4, the theme list on top.
fn draw_settings(frame: &mut Frame, s: &Settings) {
    let [header, body, menu] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(3),
        Constraint::Length(2),
    ])
    .areas(frame.area());
    let bold = Style::new().add_modifier(Modifier::BOLD);
    let (title, items) = settings::PAGES[s.page];

    frame.render_widget(
        Line::from(vec![
            Span::styled("Xylem configuration", bold),
            Span::raw(format!(
                "   page {} of {}: {title}",
                s.page + 1,
                settings::PAGES.len()
            )),
            Span::styled(
                if s.dirty() { "   (modified)" } else { "" },
                Style::new().fg(theme().message),
            ),
        ]),
        header,
    );

    let block = panel(format!(" {title} "), true);
    let inner = block.inner(body);
    frame.render_widget(block, body);
    let width = inner.width as usize;
    let mut lines: Vec<Line> = items
        .iter()
        .enumerate()
        .map(|(i, (letter, label, opt))| {
            let value = opt.value(&s.config);
            let fill = width.saturating_sub(label.chars().count() + value.chars().count() + 8);
            let line = Line::from(vec![
                Span::raw("  "),
                key(&letter.to_string()),
                Span::raw(format!("  {label} {} ", ".".repeat(fill))),
                Span::styled(value, bold),
            ]);
            if i == s.selected {
                line.style(highlight(true))
            } else {
                line
            }
        })
        .collect();
    lines.push(Line::default());
    lines.push(Line::from(vec![
        Span::raw("  Theme: "),
        Span::styled(s.config.theme.clone(), bold),
        Span::styled("  (F4 to choose)", Style::new().fg(theme().dim)),
    ]));
    if items.get(s.selected).map(|item| item.2) == Some(settings::Opt::GitTrusted) {
        lines.push(Line::default());
        for text in [
            "  DANGER: in these folders, just walking into a repository lets Git",
            "  run what it names (hooks, filters, external diffs). Never extract",
            "  or download anything from others there.",
        ] {
            lines.push(Line::styled(text, Style::new().fg(theme().error)));
        }
    }
    frame.render_widget(Paragraph::new(lines), inner);

    if let Some(picker) = &s.picker {
        let height = (config::THEMES.len() + 2) as u16;
        let area = Rect {
            x: inner.x + 4,
            y: inner.y + 1,
            width: 30.min(inner.width.saturating_sub(4)),
            height: height.min(inner.height),
        }
        .intersection(inner);
        let items: Vec<ListItem> = config::THEMES
            .iter()
            .map(|(name, _)| ListItem::new(format!(" {name}")))
            .collect();
        let mut state = ListState::default().with_selected(Some(picker.index));
        frame.render_widget(Clear, area);
        frame.render_stateful_widget(
            List::new(items)
                .block(panel(" Theme ", true))
                .style(Style::new().fg(theme().text).bg(theme().background))
                .highlight_style(highlight(true)),
            area,
            &mut state,
        );
    }

    let lines = if let Some((opt, input)) = &s.editing {
        let label = match opt {
            settings::Opt::Opener => "Open with: ",
            settings::Opt::BinaryEditor => "Binary editor: ",
            settings::Opt::GitTrusted => "Trusted Git folders: ",
            _ => "Editor: ",
        };
        let hint = match (&s.message, opt) {
            (Some(message), _) => message.as_str(),
            (None, settings::Opt::GitTrusted) => "; between folders, empty = trust none",
            (None, _) => "empty = default",
        };
        let col = menu.x + (label.len() + input.cursor) as u16;
        frame.set_cursor_position((col.min(menu.right().saturating_sub(1)), menu.y));
        vec![
            Line::from(vec![
                Span::styled(label, bold),
                Span::raw(input.text.clone()),
            ]),
            menu_line("", &[("Enter", "accept"), ("Esc", "cancel"), ("", hint)]),
        ]
    } else if s.picker.is_some() {
        vec![
            menu_line(
                "THEME",
                &[("↑↓", "preview"), ("Enter", "choose"), ("Esc", "cancel")],
            ),
            Line::default(),
        ]
    } else {
        let first = match &s.message {
            Some(m) => Line::styled(
                m.clone(),
                Style::new()
                    .fg(theme().message)
                    .add_modifier(Modifier::BOLD),
            ),
            None => menu_line(
                "CONFIG",
                &[
                    ("A-Z", "change"),
                    ("↑↓ Enter", "select/change"),
                    ("PgUp/PgDn 1-4", "page"),
                    ("F4", "theme"),
                ],
            ),
        };
        vec![first, menu_line("", &[("F3", "save"), ("Esc", "cancel")])]
    };
    let lines: Vec<_> = lines
        .into_iter()
        .map(|l| fit_menu(l, menu.width as usize))
        .collect();
    frame.render_widget(Paragraph::new(lines), menu);
}

fn draw_bookmarks(frame: &mut Frame, app: &App, view: &BookmarksView) {
    let [header, body, menu] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(3),
        Constraint::Length(2),
    ])
    .areas(frame.area());
    let bold = Style::new().add_modifier(Modifier::BOLD);

    frame.render_widget(
        Line::from(vec![
            Span::styled("Xylem bookmarks", bold),
            Span::raw(format!(
                "   {} bookmark{}",
                app.bookmarks.len(),
                if app.bookmarks.len() == 1 { "" } else { "s" }
            )),
        ]),
        header,
    );

    let title = if view.pick {
        " Bookmarks: choose a place for the prompt (F11) "
    } else {
        " Directory Bookmarks (Alt+B / F11) "
    };
    let block = panel(title, true);
    let inner = block.inner(body);
    frame.render_widget(block, body);

    let home = platform::home_dir();
    let format_path = |p: &std::path::Path| -> String {
        if let Some(h) = &home
            && let Ok(rel) = p.strip_prefix(h)
        {
            format!("~/{}", rel.display())
        } else {
            p.display().to_string()
        }
    };
    // Where a bookmark goes, always shown next to its name: the folder, or
    // the address of a remote one.
    let place = |bm: &crate::state::Bookmark| match bm.address() {
        Some(address) => address.to_string(),
        None => format_path(&bm.path),
    };

    let mut lines: Vec<Line> = Vec::new();
    if app.bookmarks.is_empty() {
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(
            "  No bookmarks defined yet.",
            bold,
        )));
        lines.push(Line::default());
        if view.pick {
            lines.push(Line::from(vec![
                Span::raw("  "),
                key("Ctrl+F11"),
                Span::raw(" saves the current directory as one, once this prompt is closed."),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::raw("  Press "),
                key("Ins"),
                Span::raw(", "),
                key("+"),
                Span::raw(" or "),
                key("m"),
                Span::raw(" to bookmark the current directory:"),
            ]));
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(
                    format_path(&app.current_path()),
                    Style::new().fg(theme().key).add_modifier(Modifier::BOLD),
                ),
            ]));
        }
    } else {
        // "▶ [k] " before each one.
        let width = (inner.width as usize).saturating_sub(6);
        let labels: Vec<Option<String>> = app.bookmarks.values().map(|b| b.label()).collect();
        // The names in a column, which never takes more than half the line:
        // the place always shows, cut at the end if it must be.
        let name_width = labels
            .iter()
            .flatten()
            .map(|l| l.chars().count())
            .max()
            .map_or(0, |w| w.min(width / 2));
        for (i, ((k, bm), label)) in app.bookmarks.iter().zip(&labels).enumerate() {
            let missing = view.missing.contains(k);
            let cursor = if i == view.selected { "▶ " } else { "  " };
            let mut spans = vec![Span::raw(cursor), key(&format!("[{k}]")), Span::raw(" ")];
            let mut room = width;
            if name_width > 0 {
                let name = label.as_deref().unwrap_or("");
                spans.push(Span::styled(fit(name, name_width), bold));
                spans.push(Span::raw("  "));
                room = room.saturating_sub(name_width + 2);
            }
            let status = if missing { " [not found]" } else { "" };
            let place = place(bm);
            let place_width = room.saturating_sub(status.chars().count());
            spans.push(Span::raw(if place.chars().count() > place_width {
                fit(&place, place_width)
            } else {
                place
            }));
            if missing {
                spans.push(Span::styled(status, Style::new().fg(theme().message)));
            }
            let line = Line::from(spans);
            if i == view.selected {
                lines.push(line.style(highlight(true)));
            } else {
                lines.push(line);
            }
        }
        lines.push(Line::default());
        lines.push(Line::from(vec![
            Span::styled("  Current directory: ", bold),
            Span::raw(format_path(&app.current_path())),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), inner);

    // The name of a bookmark is typed here, over the list.
    if let Some(prompt) = app.prompt.as_ref().filter(|p| p.is_bookmark_name()) {
        let lines = prompt_lines(frame, prompt, menu);
        frame.render_widget(Paragraph::new(lines), menu);
        return;
    }
    let keys: &[(&str, &str)] = if view.pick {
        &[
            ("Enter / key", "put it in the prompt"),
            ("↑↓ / j k", "navigate"),
            ("Esc", "back"),
        ]
    } else {
        &[
            ("Enter / key", "jump"),
            ("F2", "name"),
            ("Ins / +", "add current dir"),
            ("Del / d", "remove"),
            ("↑↓ / j k", "navigate"),
            ("Esc", "exit"),
        ]
    };
    frame.render_widget(fit_menu(menu_line("", keys), menu.width as usize), menu);
}

/// The help key to show in the footer: F1, unless the terminal is one of
/// those that keep F1 for their own help (VTE: GNOME Terminal, Ptyxis,
/// Terminator, Tilix) and Alt+H also opens it.
fn help_key(app: &App) -> String {
    let alt_h = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::ALT);
    let vte = std::env::var_os("VTE_VERSION").is_some();
    if vte && app.keymap.lookup(alt_h, app.focus) == Some(Command::Help) {
        "Alt+H".into()
    } else {
        app.keymap
            .key_for(Command::Help)
            .unwrap_or_else(|| "F1".into())
    }
}

/// The help (F1 / Alt+H): a topic of the manual, with its links.
fn draw_help(frame: &mut Frame, help: &mut Help) {
    let [header, body, menu] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(2),
    ])
    .areas(frame.area());
    frame.render_widget(
        Line::from(vec![
            Span::styled("Help: ", Style::new().fg(theme().key)),
            Span::styled(help.title(), Style::new().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("  [{} · {}]", help.lang, help::language_label(&help.lang)),
                Style::new().fg(theme().dim),
            ),
        ]),
        header,
    );

    let block = panel("", true);
    let inner = block.inner(body);
    frame.render_widget(block, body);
    help.height = inner.height as usize;
    let selected = help.selected;
    let scroll = help.scroll;
    let page = help.page(inner.width as usize);
    let total = page.lines.len();
    let style = |kind: help::Kind| match kind {
        help::Kind::Text => Style::new(),
        help::Kind::Heading => Style::new().fg(theme().key).add_modifier(Modifier::BOLD),
        help::Kind::Strong => Style::new().add_modifier(Modifier::BOLD),
        help::Kind::Code => Style::new().fg(theme().key),
        help::Kind::Link(id) if Some(id) == selected => highlight(true),
        help::Kind::Link(_) => Style::new().add_modifier(Modifier::UNDERLINED),
    };
    let lines: Vec<Line> = page
        .lines
        .iter()
        .skip(scroll)
        .take(inner.height as usize)
        .map(|pieces| {
            Line::from(
                pieces
                    .iter()
                    .map(|p| Span::styled(p.text.clone(), style(p.kind)))
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), inner);

    let percent = ((scroll + inner.height as usize).min(total.max(1)) * 100) / total.max(1);
    let keys = |pairs: &[(&str, &str)]| {
        let mut spans = vec![Span::raw(format!("{:9}", ""))];
        for (k, what) in pairs {
            spans.push(key(k));
            spans.push(Span::raw(format!(" {what}   ")));
        }
        Line::from(spans)
    };
    frame.render_widget(
        Paragraph::new(vec![
            {
                let mut first = keys(&[
                    ("Tab", "link"),
                    ("Enter", "follow"),
                    ("Backspace", "back"),
                    ("I", "index"),
                    ("K", "keys"),
                    ("L", "lang"),
                    ("Esc", "close"),
                ]);
                first.spans[0] = Span::styled(
                    format!("{:<9}", "HELP"),
                    Style::new().add_modifier(Modifier::BOLD),
                );
                first
            },
            Line::from(Span::styled(
                format!("{:9}{percent}%", ""),
                Style::new().fg(theme().dim),
            )),
        ]),
        menu,
    );
}

fn draw_viewer(frame: &mut Frame, app: &mut App) {
    let tagged = app
        .tagged_view
        .as_ref()
        .map(|tv| (tv.index + 1, tv.items.len()));
    // D: diff of a changed file (Git on), or back from the diff.
    let diff = app
        .viewer
        .as_ref()
        .is_some_and(|v| v.behind.is_some() || (v.on_disk && app.can_diff(&v.path)));
    let Some(v) = &mut app.viewer else { return };
    let [header, body, menu] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(2),
    ])
    .areas(frame.area());

    let mut title = vec![
        Span::styled("View: ", Style::new().fg(theme().key)),
        Span::raw(v.path.to_string_lossy().into_owned()),
        Span::styled(
            format!("  {} bytes", fmt_num(v.size)),
            Style::new().fg(theme().dim),
        ),
    ];
    if let Some((kind, _)) = v.compression {
        title.push(Span::styled(
            format!("  {} → {} bytes", kind.name(), fmt_num(v.data_len())),
            Style::new().fg(theme().dim),
        ));
    }
    if v.charset() != Charset::Utf8 {
        title.push(Span::styled(
            format!("  {}", v.charset().name()),
            Style::new().fg(theme().dim),
        ));
    }
    if v.truncated() {
        // The hex view shows every byte read: the line budget cuts only
        // the text.
        let shown = if v.lines_cut() && v.mode == ViewMode::Text {
            Some(format!("{} lines", fmt_num(v.total_lines() as u64)))
        } else if !v.bytes_cut() {
            None
        } else if v.compression.is_some() {
            Some(format!("{} bytes decompressed", fmt_num(v.data_len())))
        } else {
            Some(format!("{} MiB", v.limit() >> 20))
        };
        if let Some(shown) = shown {
            title.push(Span::styled(
                format!("  (showing first {shown})"),
                Style::new().fg(theme().error),
            ));
        }
    }
    if v.rendered_cut() {
        title.push(Span::styled(
            "  (rendered only the start)",
            Style::new().fg(theme().error),
        ));
    }
    if v.mode == ViewMode::Table
        && let Some(cut) = v.table.as_ref().and_then(|t| t.cut.as_ref())
    {
        title.push(Span::styled(
            format!("  ({cut})"),
            Style::new().fg(theme().error),
        ));
    }
    if let Some(problem) = v.problem() {
        title.push(Span::styled(
            format!("  ({problem})"),
            Style::new().fg(theme().error),
        ));
    }
    if v.bidi_count() > 0 {
        title.push(Span::styled(
            format!(
                "  {} bidirectional control character{} (shown as ·)",
                v.bidi_count(),
                if v.bidi_count() == 1 { "" } else { "s" }
            ),
            Style::new().fg(theme().error),
        ));
    }
    if v.mode == ViewMode::Text
        && let Some(spec) = v.line_filter()
    {
        title.push(Span::styled(
            format!(
                "  [filter {}: {} of {} lines]",
                spec.text,
                fmt_num(v.line_count() as u64),
                fmt_num(v.total_lines() as u64)
            ),
            Style::new().fg(theme().key),
        ));
    }
    if let Some((i, n)) = tagged {
        title.push(Span::styled(
            format!("  [tagged {i} of {n}]"),
            Style::new().fg(theme().key),
        ));
    }
    frame.render_widget(Line::from(title), header);

    let mode = match v.mode {
        ViewMode::Text => " TEXT ".to_string(),
        ViewMode::Hex => " HEX ".to_string(),
        ViewMode::Rendered => " RENDERED ".to_string(),
        ViewMode::Table => v
            .table
            .as_ref()
            .map_or_else(|| " TABLE ".to_string(), table_frame_title),
    };
    match &app.batch_preview {
        Some(preview) => draw_batch_preview(frame, preview, body),
        None => draw_viewer_body(frame, v, mode, body),
    }
    if let (Some(list), Some(grid)) = (&mut v.sheet_list, &v.table) {
        let word = if sheet_word(&v.path) == "table" {
            "Table"
        } else {
            "Sheet"
        };
        draw_sheet_list(frame, list, grid, word, body);
    }
    if let Some(view) = &mut v.structure_view {
        draw_structure(frame, view, body);
    }
    if let Some(guide) = app.prompt.as_ref().and_then(app::Prompt::batch_guide) {
        match &guide.reading {
            Some(reading) => draw_template_reading(frame, reading, body),
            None => draw_batch_guide(frame, guide, body),
        }
    }

    let total = v.line_count().max(1);
    let position = match v.mode {
        ViewMode::Text => format!(
            "Line {} of {}",
            fmt_num(v.top_line() as u64 + 1),
            fmt_num(v.total_lines().max(1) as u64)
        ),
        ViewMode::Rendered => format!(
            "Line {} of {}",
            fmt_num(v.rendered_line() as u64 + 1),
            fmt_num(v.total_lines().max(1) as u64)
        ),
        ViewMode::Hex => format!("Offset {:08X}", v.top * viewer::HEX_WIDTH),
        ViewMode::Table => {
            if let Some(t) = &v.table {
                let cell_col_letter = crate::viewer::tabular_loader::column_letter(t.cursor_col);
                let marked = if t.marked.is_empty() {
                    String::new()
                } else {
                    format!(", {} marked", fmt_num(t.marked.len() as u64))
                };
                let row = t.cursor_file_row();
                let total = t.rows.len();
                let matched = if t.active_filter.is_some() {
                    format!(
                        ", match {} of {}",
                        (t.cursor_row + 1).min(t.visible_row_count()),
                        t.visible_row_count()
                    )
                } else {
                    String::new()
                };
                format!("Cell {cell_col_letter}{row} (Row {row} of {total}{matched}{marked})")
            } else {
                "Table".to_string()
            }
        }
    };
    let percent = ((v.top + v.page).min(total) * 100) / total;
    let note = v.note.clone();
    let lines = if let Some(prompt) = &app.prompt {
        prompt_lines(frame, prompt, menu)
    } else if v.structure_view.is_some() {
        vec![menu_line(
            "COLUMNS",
            &[("↑↓ PgUp PgDn", "scroll"), ("Esc", "back")],
        )]
    } else if v.sheet_list.is_some() {
        let title = format!("{}S", sheet_word(&v.path).to_uppercase());
        vec![menu_line(
            &title,
            &[
                ("↑↓", "move"),
                ("letters", "narrow the list"),
                ("Enter", "open"),
                ("Esc", "back"),
            ],
        )]
    } else if app.batch_preview.is_some() {
        vec![menu_line(
            "BATCH",
            &[
                ("Enter", "write the file"),
                ("Esc", "cancel"),
                ("F1", "help"),
            ],
        )]
    } else {
        let mut words = vec![
            "Find", "Backward", "Hex", "Colors", "Wrap", "#Num", "Goto", "Edit", "cOdepage",
        ];
        if v.can_render() {
            words.insert(3, "Text");
        }
        if diff {
            words.push("Diff");
        }
        if tagged.is_some() {
            words.extend(["Next", "Prev", "Untag"]);
        }
        let mut first = menu_words("VIEW", &words);
        first.spans.extend(
            menu_line(
                "",
                &if v.mode == ViewMode::Table {
                    vec![
                        ("&", "filter"),
                        ("i", "structure"),
                        ("m", "mark"),
                        ("Ctrl+B", "batch"),
                        ("Esc", "close"),
                    ]
                } else {
                    vec![
                        ("&", "filter"),
                        ("/ \\", "find here"),
                        ("Space", "next hit"),
                        ("Esc", "close"),
                    ]
                },
            )
            .spans
            .into_iter()
            .skip(1),
        );
        let second = match note {
            Some(n) => Line::styled(
                format!("{:9}{n}", ""),
                Style::new()
                    .fg(theme().message)
                    .add_modifier(Modifier::BOLD),
            ),
            None => {
                let language = v
                    .language()
                    .map(|l| format!("{l}  ·  "))
                    .unwrap_or_default();
                let wrap_badge = if v.wrap { "[Wrap] " } else { "" };
                let num_badge = if v.line_numbers { "[#] " } else { "" };
                let sheets = match (&v.mode, &v.table) {
                    (ViewMode::Table, Some(t)) => format!("{TABLE_ENDS}{}", sheet_keys(t, &v.path)),
                    _ => String::new(),
                };
                Line::from(format!(
                    "{:9}{wrap_badge}{num_badge}{language}{position}  ({percent}%){sheets}",
                    ""
                ))
            }
        };
        vec![first, second]
    };
    let lines: Vec<_> = lines
        .into_iter()
        .map(|l| fit_menu(l, menu.width as usize))
        .collect();
    frame.render_widget(Paragraph::new(lines), menu);
}

/// The most of a sheet's name that the frame shows.
const SHEET_NAME_CHARS: usize = 40;

/// The frame of a table: with more than one sheet, which one this is. The names
/// come from the file, and were made safe for the screen when it was read.
fn table_frame_title(t: &TableGrid) -> String {
    if let Some(query) = &t.query {
        let query = TableGrid::sanitize_cell(query);
        return format!(" QUERY: {} ", fit(&query, SHEET_NAME_CHARS));
    }
    if t.sheets.len() < 2 {
        return " TABLE ".to_string();
    }
    let name = t.sheets.get(t.current_sheet).map_or("", String::as_str);
    let mut shown: String = name.chars().take(SHEET_NAME_CHARS).collect();
    if name.chars().count() > SHEET_NAME_CHARS {
        shown.push('…');
    }
    format!(
        " TABLE [{}/{}: {shown}{}] ",
        t.current_sheet + 1,
        t.sheets.len(),
        t.view_mark(t.current_sheet)
    )
}

/// The keys to the ends of a table, on the line of where the cursor is.
const TABLE_ENDS: &str = "  ·  Ctrl+Home Ctrl+End first, last row";

/// The keys that change the sheet, when there is another one to go to.
fn sheet_keys(t: &TableGrid, path: &Path) -> String {
    if t.sheets.len() < 2 {
        return String::new();
    }
    format!("  ·  Tab list  ·  [ ] other {}", sheet_word(path))
}

/// What this file calls its parts.
fn sheet_word(path: &Path) -> &'static str {
    if viewer::tabular_loader::is_sqlite(path) {
        "table"
    } else {
        "sheet"
    }
}

/// The list of sheets or tables, over the grid. Only the rows on screen are built:
/// a hostile file can have any number of names. They were made safe when read.
fn draw_sheet_list(
    frame: &mut Frame,
    list: &mut crate::viewer::table::SheetList,
    grid: &TableGrid,
    word: &str,
    area: Rect,
) {
    let t = theme();
    let width = 60.min(area.width.saturating_sub(4));
    let wanted = list
        .shown
        .len()
        .saturating_add(3)
        .min(usize::from(u16::MAX)) as u16;
    let height = wanted.max(4).min(area.height.saturating_sub(2));
    if width < 12 || height < 4 {
        return;
    }
    let rect = Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    };
    let block = panel(
        format!(
            " {word}s: {} of {} ",
            fmt_num(list.shown.len() as u64),
            fmt_num(grid.sheets.len() as u64)
        ),
        true,
    );
    let inner = block.inner(rect);
    let rows = usize::from(inner.height.saturating_sub(1)).max(1);
    if list.cursor < list.scroll {
        list.scroll = list.cursor;
    } else if list.cursor >= list.scroll + rows {
        list.scroll = list.cursor + 1 - rows;
    }
    let name_width = usize::from(inner.width).saturating_sub(2);
    let mut lines = vec![Line::styled(
        fit(&format!("Find: {}_", list.query), usize::from(inner.width)),
        Style::new().fg(t.key),
    )];
    if list.shown.is_empty() {
        lines.push(Line::styled(
            format!(" No {} has this name", word.to_lowercase()),
            Style::new().fg(t.dim),
        ));
    }
    for (row, &sheet) in list.shown.iter().enumerate().skip(list.scroll).take(rows) {
        let name = grid.sheets.get(sheet).map_or("", String::as_str);
        let open = if sheet == grid.current_sheet {
            '*'
        } else {
            ' '
        };
        let mark = grid.view_mark(sheet);
        let name = fit(name, name_width.saturating_sub(mark.len()));
        let text = format!("{open} {}{mark}", name.trim_end());
        lines.push(if row == list.cursor {
            Line::styled(text, highlight(true))
        } else {
            Line::raw(text)
        });
    }
    frame.render_widget(Clear, rect);
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .style(Style::new().fg(t.text).bg(t.background)),
        rect,
    );
}

/// The widest a column of the structure gets: a Parquet type can be long.
const STRUCTURE_CELL_CHARS: usize = 40;

/// The structure of the table (`i`), over the grid: what the file says of the
/// whole, then a row per column, numbered. Its texts were made safe when read.
fn draw_structure(frame: &mut Frame, view: &mut crate::viewer::table::StructureView, area: Rect) {
    let t = theme();
    let s = &view.structure;
    let number = s.columns.len().max(1).to_string().len().max(1);
    // Numbers (sizes, decimals) line up on the right.
    let numeric: Vec<bool> = (0..s.titles.len())
        .map(|c| {
            s.columns.iter().all(|row| {
                row.get(c)
                    .is_none_or(|v| v.is_empty() || v == "?" || v.parse::<u64>().is_ok())
            })
        })
        .collect();
    let widths: Vec<usize> = s
        .titles
        .iter()
        .enumerate()
        .map(|(c, title)| {
            s.columns
                .iter()
                .filter_map(|row| row.get(c))
                .map(|v| v.chars().count())
                .chain([title.chars().count()])
                .max()
                .unwrap_or(0)
                .min(STRUCTURE_CELL_CHARS)
        })
        .collect();
    let line = |first: &str, cells: &[String]| {
        let mut text = format!("{first:>number$}");
        for (c, width) in widths.iter().enumerate() {
            let cell = fit(cells.get(c).map_or("", String::as_str), *width);
            text.push_str("  ");
            if numeric[c] {
                text.push_str(&format!("{:>width$}", cell.trim_end()));
            } else {
                text.push_str(&cell);
            }
        }
        text.trim_end().to_string()
    };
    let titles = line("#", &s.titles);
    let wanted_width = s
        .about
        .iter()
        .map(|a| a.chars().count())
        .chain([titles.chars().count(), 24])
        .max()
        .unwrap_or(0)
        .saturating_add(4)
        .min(usize::from(u16::MAX)) as u16;
    let width = wanted_width.min(area.width.saturating_sub(4));
    let fixed = s.about.len() + 2;
    let wanted = (fixed + s.columns.len().max(1) + 2).min(usize::from(u16::MAX)) as u16;
    let height = wanted.min(area.height.saturating_sub(2));
    if width < 12 || height < 5 {
        return;
    }
    let rect = Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    };
    let block = panel(
        format!(" Structure: {} columns ", fmt_num(s.columns.len() as u64)),
        true,
    );
    let inner = block.inner(rect);
    let inner_width = usize::from(inner.width);
    let fixed = fixed.min(usize::from(inner.height).saturating_sub(1));
    let rows = usize::from(inner.height).saturating_sub(fixed).max(1);
    view.scroll = view.scroll.min(s.columns.len().saturating_sub(rows));
    let mut lines: Vec<Line> = s
        .about
        .iter()
        .map(|a| Line::raw(fit(a, inner_width).trim_end().to_string()))
        .collect();
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        fit(&titles, inner_width).trim_end().to_string(),
        Style::new().fg(t.key).add_modifier(Modifier::BOLD),
    ));
    lines.truncate(fixed);
    for (i, row) in s.columns.iter().enumerate().skip(view.scroll).take(rows) {
        let text = line(&(i + 1).to_string(), row);
        lines.push(Line::raw(fit(&text, inner_width).trim_end().to_string()));
    }
    frame.render_widget(Clear, rect);
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .style(Style::new().fg(t.text).bg(t.background)),
        rect,
    );
}

/// The most templates the guide of a Batch shows at a time.
const GUIDE_TEMPLATES: usize = 12;

/// Above the mask prompt of a Batch, over the end of what is open: what to type
/// there, with what works on this file, and the templates to choose from.
fn draw_batch_guide(frame: &mut Frame, guide: &app::BatchGuide, area: Rect) {
    let t = theme();
    let error = Style::new().fg(t.error);
    let code = Style::new().fg(t.key);
    let dim = Style::new().fg(t.dim);
    let width = usize::from(area.width.saturating_sub(2)).max(1);

    let mut lines = Vec::new();
    for warning in [&guide.problem, &guide.open_template].into_iter().flatten() {
        lines.push(Line::styled(warning.clone(), error));
        lines.push(Line::raw(""));
    }
    if guide.marked > 0 {
        lines.push(Line::styled(
            format!(
                "Only the {} marked records are written (M in the table clears the marks).",
                fmt_num(guide.marked as u64)
            ),
            code,
        ));
        lines.push(Line::raw(""));
    }
    lines.push(Line::raw(
        "A mask says what to write for each record, for example:",
    ));
    lines.push(Line::styled(format!("    {}", guide.example), code));
    lines.push(Line::raw(format!("Fields here: {}", guide.fields)));
    if guide.whole {
        lines.push(Line::raw("Empty: whole records. @a,b: only those fields."));
    }
    lines.push(Line::raw(""));
    if guide.templates.is_empty() {
        lines.push(Line::raw("Or a template, which writes a whole file:"));
        let dir = guide
            .template_dir
            .as_ref()
            .map_or("the configuration folder".to_string(), |d| {
                d.display().to_string()
            });
        lines.push(Line::styled(
            format!("    none yet: put name.tpl files in {dir}"),
            dim,
        ));
    } else {
        lines.push(Line::raw(
            "Or a template, which writes a whole file (↑↓ choose, F3 read it):",
        ));
    }
    let end = Line::raw("Enter shows the first lines before anything is written.");
    // A line that wraps at words can take one more than its length says.
    let rows = |lines: &[Line]| -> usize {
        lines
            .iter()
            .map(|l| match l.width() {
                w if w <= width => 1,
                w => w.div_ceil(width) + 1,
            })
            .sum()
    };

    // The list takes what the rest leaves, and keeps the chosen one in sight.
    let others = rows(&lines) + 2 + 2;
    let room = usize::from(area.height).saturating_sub(others).max(1);
    let shown = guide.templates.len().min(GUIDE_TEMPLATES).min(room);
    let selected = guide.selected.unwrap_or(0);
    let start = (selected + 1).saturating_sub(shown);
    let name_width = guide
        .templates
        .iter()
        .map(|i| i.name.chars().count() + 1)
        .max()
        .unwrap_or(0)
        .min(24);
    for (i, template) in guide.templates.iter().enumerate().skip(start).take(shown) {
        let chosen = guide.selected == Some(i);
        let name = format!(":{}", template.name);
        let mark = if chosen { "> " } else { "  " };
        let row = format!("  {mark}{}  ", fit(&name, name_width));
        let about = width.saturating_sub(row.chars().count());
        let style = if chosen {
            code.add_modifier(Modifier::BOLD)
        } else {
            code
        };
        lines.push(Line::from(vec![
            Span::styled(row, style),
            Span::styled(fit(&template.about, about), dim),
        ]));
    }
    if guide.templates.len() > shown {
        lines.push(Line::styled(
            format!(
                "    {} of {} (↑↓ for the others)",
                shown,
                guide.templates.len()
            ),
            dim,
        ));
    }
    lines.push(Line::raw(""));
    lines.push(end);

    let height = u16::try_from(rows(&lines) + 2)
        .unwrap_or(u16::MAX)
        .min(area.height);
    let box_area = Rect {
        y: area.bottom() - height,
        height,
        ..area
    };
    frame.render_widget(Clear, box_area);
    let title = format!(" Batch into {} ", guide.file);
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(ratatui::widgets::Wrap { trim: false })
            .block(panel(title, true)),
        box_area,
    );
}

/// A template, whole, as the file has it: what it says about itself is only a
/// comment, and this is what it will do.
fn draw_template_reading(frame: &mut Frame, reading: &app::TemplateReading, area: Rect) {
    let t = theme();
    let lines: Vec<Line> = reading
        .lines
        .iter()
        .skip(reading.top)
        .take(usize::from(area.height))
        .map(|line| {
            if line.starts_with("%%#") {
                Line::styled(line.clone(), Style::new().fg(t.dim))
            } else if line.starts_with("%%") {
                Line::styled(line.clone(), Style::new().fg(t.key))
            } else {
                Line::raw(line.clone())
            }
        })
        .collect();
    let title = format!(
        " Template :{} (line {} of {}) ",
        reading.name,
        reading.top + 1,
        reading.lines.len().max(1)
    );
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(ratatui::widgets::Wrap { trim: false })
            .block(panel(title, true)),
        area,
    );
}

/// What a Batch of the viewer would write, for the user to look at first.
fn draw_batch_preview(frame: &mut Frame, preview: &app::BatchPreview, area: Rect) {
    let t = theme();
    let found = &preview.preview;
    let scanned = fmt_num(found.scanned);
    let count = if let Some(record) = found.stopped {
        format!("%%stop ends it at record {}", fmt_num(record))
    } else if found.complete {
        format!("{scanned} records")
    } else {
        format!("{scanned} records looked at; the rest are read when writing")
    };
    let mut lines = vec![Line::styled(count, Style::new().fg(t.key))];
    if found.cut {
        lines.push(Line::styled(
            "The viewer read only part of this file: the rest will not be written",
            Style::new().fg(t.error),
        ));
    }
    if let Some(warning) = found.stats.warning() {
        lines.push(Line::styled(warning, Style::new().fg(t.error)));
    }
    if !found.environment.is_empty() {
        lines.push(Line::styled(
            format!(
                "Reads from the environment: {}",
                found.environment.join(", ")
            ),
            Style::new().fg(t.error),
        ));
    }
    if !found.unseen.is_empty() {
        lines.push(Line::styled(
            format!("No record looked at took: {}", found.unseen.join(", ")),
            Style::new().fg(t.error),
        ));
    }
    lines.push(Line::raw(""));
    lines.extend(found.lines.iter().map(|l| Line::raw(l.clone())));
    let title = format!(" Batch: {} ", app::fit_name(&preview.file));
    frame.render_widget(Paragraph::new(lines).block(panel(title, true)), area);
}

fn draw_editor(frame: &mut Frame, app: &mut App) {
    let colors = if app.options.editor_theme {
        TerminalTheme::Xylem
    } else {
        TerminalTheme::Native
    };
    let Some(ed) = &mut app.editor_pty else {
        return;
    };
    let [header, body, menu] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(2),
    ])
    .areas(frame.area());

    let title = vec![
        Span::styled("Edit: ", Style::new().fg(theme().key)),
        Span::raw(ed.path.to_string_lossy().into_owned()),
    ];
    frame.render_widget(Line::from(title), header);

    let title = format!(" {} (external program) ", ed.program);
    draw_terminal_panel(frame, &mut ed.term, colors, title, body);

    let first = Line::from(vec![
        Span::styled(
            format!("{:<9}", "EDIT"),
            Style::new().add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(
                "{} is a separate program, with its own keys: Esc may not close it",
                ed.program
            ),
            Style::new().fg(theme().dim),
        ),
    ]);
    let second = Line::styled(
        format!("{:9}To return to Xylem: {}", "", quit_keys(&ed.program)),
        Style::new().fg(theme().message),
    );
    frame.render_widget(Paragraph::new(vec![first, second]), menu);
}

/// How to leave the usual editors. Xylem cannot close them for you, as a key
/// could lose what was being edited.
fn quit_keys(program: &str) -> &'static str {
    match program {
        "hexedit" => {
            "Ctrl+X saves and quits, Ctrl+C quits without saving (on Unknown command, Space first)"
        }
        "vi" | "vim" | "nvim" | "view" | "bvi" => "Esc, then :q (:q! quits without saving)",
        "nano" => "Ctrl+X",
        "micro" => "Ctrl+Q",
        "emacs" => "Ctrl+X, then Ctrl+C",
        _ => "quit this program",
    }
}

fn draw_terminal(frame: &mut Frame, term: &mut Terminal, colors: TerminalTheme, area: Rect) {
    draw_terminal_panel(frame, term, colors, " Terminal ", area);
}

fn draw_terminal_panel(
    frame: &mut Frame,
    term: &mut Terminal,
    colors: TerminalTheme,
    title: impl Into<Line<'static>>,
    area: Rect,
) {
    let block = panel(title, true);
    let inner = block.inner(area);
    term.resize(inner.height, inner.width);
    // Starts from the terminal's own colors, so that `apply_theme` sees the
    // cells the shell has not painted as default ones.
    frame.render_widget(Clear, inner);
    let parser = term.parser();
    let screen = parser.screen();
    let mut cursor = tui_term::widget::Cursor::default();
    cursor.hide();
    frame.render_widget(
        tui_term::widget::PseudoTerminal::new(screen)
            .block(block)
            .cursor(cursor),
        area,
    );
    let t = theme();
    term::apply_theme(frame.buffer_mut(), inner, colors, t.text, t.background);
    // The real cursor, where the shell left it (not while looking back).
    if !screen.hide_cursor() && screen.scrollback() == 0 {
        let (row, col) = screen.cursor_position();
        frame.set_cursor_position((inner.x + col, inner.y + row));
    }
}

fn draw_viewer_body(frame: &mut Frame, v: &mut Viewer, title: String, area: Rect) {
    let inner = panel("", false).inner(area);
    v.page = inner.height as usize;
    v.width = inner.width as usize;

    if v.mode == ViewMode::Table
        && let Some(t) = &mut v.table
    {
        t.show_cursor_col(v.width);
        draw_table_grid(frame, t, title, area);
        return;
    }
    let mark = highlight(true).add_modifier(Modifier::BOLD);
    let digits = v.total_lines().to_string().len().max(1);
    let show_numbers = v.line_numbers && matches!(v.mode, ViewMode::Text | ViewMode::Rendered);
    let t = theme();
    let lines: Vec<Line> = v
        .styled(inner.height as usize)
        .into_iter()
        .map(|row| {
            let mut spans = Vec::new();
            if show_numbers {
                match row.line_num {
                    Some(num) => {
                        spans.push(Span::styled(
                            format!("{num:>digits$} "),
                            Style::new().fg(t.dim),
                        ));
                    }
                    None => {
                        spans.push(Span::styled(
                            format!("{:>digits$} ", ""),
                            Style::new().fg(t.dim),
                        ));
                    }
                }
                spans.push(Span::styled("│ ", Style::new().fg(t.border)));
            }
            for (text, color, marked, modifier) in row.segments {
                let plain = Style::new().add_modifier(modifier);
                spans.push(match (marked, color) {
                    (true, _) => Span::styled(text, mark),
                    (false, Some(c)) => Span::styled(text, plain.fg(c)),
                    (false, None) => Span::styled(text, plain),
                });
            }
            Line::from(spans)
        })
        .collect();
    frame.render_widget(Paragraph::new(lines).block(panel(title, true)), area);
}

/// How wide a column of the grid is drawn when it starts at `used`: its width,
/// or None when it does not fit. The first one shown is cut to the screen
/// instead, so that a column wider than the screen still shows its start.
fn grid_column_width(
    grid: &crate::viewer::table::TableGrid,
    col: usize,
    used: usize,
    width: usize,
) -> Option<usize> {
    let w = grid.col_widths.get(col).copied().unwrap_or(8);
    if used + w <= width {
        Some(w)
    } else if col == grid.scroll_col && width > used + 1 {
        Some(width - used - 1)
    } else {
        None
    }
}

/// Render the tabular grid view for spreadsheets, CSV, DBF and SQLite.
fn draw_table_grid(
    frame: &mut Frame,
    grid: &crate::viewer::table::TableGrid,
    title: String,
    area: Rect,
) {
    let t = theme();
    let inner = panel("", false).inner(area);
    if inner.height < 2 || inner.width < 4 {
        return;
    }

    let page_rows = usize::from(inner.height).saturating_sub(viewer::TABLE_CHROME);
    let total_rows = grid.visible_row_count();
    let col_count = grid.headers.len();
    let row_num_width = grid.row_number_width();

    let mut lines = Vec::new();

    // 1. Column Headers Line
    let mut header_spans = Vec::new();
    header_spans.push(Span::styled(
        format!("{:>width$} │ ", "#", width = row_num_width),
        Style::new().fg(t.border),
    ));

    let mut current_width = row_num_width + 3;
    let max_avail_width = inner.width as usize;

    for col_idx in grid.scroll_col..col_count {
        let Some(w) = grid_column_width(grid, col_idx, current_width, max_avail_width) else {
            break;
        };
        let header_name = grid.headers.get(col_idx).map(|s| s.as_str()).unwrap_or("");
        let is_selected_col = col_idx == grid.cursor_col;
        let style = if is_selected_col {
            Style::new().fg(t.key).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(t.dim)
        };
        header_spans.push(Span::styled(
            format!("{:<width$} ", header_name, width = w),
            style,
        ));
        header_spans.push(Span::styled("│ ", Style::new().fg(t.border)));
        current_width += w + 2;
    }
    lines.push(Line::from(header_spans));

    // 2. Data Rows
    for r_offset in 0..page_rows {
        let display_row_idx = grid.scroll_row + r_offset;
        if display_row_idx >= total_rows {
            break;
        }
        let real_row_idx = grid.filtered_indices[display_row_idx];
        let row_data = &grid.rows[real_row_idx];
        let is_cursor_row = display_row_idx == grid.cursor_row;

        let mut row_spans = Vec::new();
        let is_marked = grid.marked.contains(&real_row_idx);
        let num_style = if is_marked {
            Style::new().fg(t.key).add_modifier(Modifier::BOLD)
        } else if is_cursor_row {
            Style::new().fg(t.key)
        } else {
            Style::new().fg(t.dim)
        };

        row_spans.push(Span::styled(
            format!(
                "{:>width$}{}│ ",
                real_row_idx + 1,
                if is_marked { "*" } else { " " },
                width = row_num_width
            ),
            num_style,
        ));

        let mut cur_w = row_num_width + 3;
        for col_idx in grid.scroll_col..col_count {
            let Some(w) = grid_column_width(grid, col_idx, cur_w, max_avail_width) else {
                break;
            };
            let cell_text = row_data
                .cells
                .get(col_idx)
                .map(|s| s.as_str())
                .unwrap_or("");
            let is_active_cell = is_cursor_row && col_idx == grid.cursor_col;

            let cell_style = if is_active_cell {
                highlight(true).add_modifier(Modifier::BOLD)
            } else if is_cursor_row {
                Style::new().fg(t.text)
            } else {
                Style::new().fg(t.dim)
            };

            let formatted_cell = format!("{:<width$} ", fit(cell_text, w), width = w);
            row_spans.push(Span::styled(formatted_cell, cell_style));
            row_spans.push(Span::styled("│ ", Style::new().fg(t.border)));
            cur_w += w + 2;
        }
        lines.push(Line::from(row_spans));
    }

    // 3. Status / Active Cell Full Content at Bottom
    let active_content = grid.active_cell().unwrap_or("");
    let filter_notice = if let Some(q) = &grid.active_filter {
        format!(" [Filter: {} ({} matches)] ", q, grid.visible_row_count())
    } else {
        String::new()
    };

    let bottom_text = format!(
        "Cell: {}{} │ Content: {}{}",
        crate::viewer::tabular_loader::column_letter(grid.cursor_col),
        grid.cursor_file_row(),
        active_content,
        filter_notice
    );
    lines.push(Line::from(Span::styled(
        fit(&bottom_text, max_avail_width),
        Style::new().fg(t.key),
    )));

    frame.render_widget(Paragraph::new(lines).block(panel(title, true)), area);
}

fn key(k: &str) -> Span<'static> {
    Span::styled(
        k.to_string(),
        Style::new().fg(theme().key).add_modifier(Modifier::BOLD),
    )
}

fn menu_line(title: &str, entries: &[(&str, &str)]) -> Line<'static> {
    let mut spans = vec![Span::styled(
        format!("{title:<9}"),
        Style::new().add_modifier(Modifier::BOLD),
    )];
    for (k, desc) in entries {
        if !k.is_empty() {
            spans.push(key(k));
        }
        // Each part of a long hint is an entry of its own, so that a narrow
        // screen loses its last parts and not the whole hint.
        for (i, part) in desc.split(" · ").enumerate() {
            if i > 0 {
                spans.push(entry_end());
            }
            let sep = if i > 0 { " · " } else { " " };
            spans.push(Span::raw(format!("{sep}{part}")));
        }
        spans.push(Span::raw("   "));
        spans.push(entry_end());
    }
    Line::from(spans)
}

/// Marks the end of an entry in a menu line: where `fit_menu` may cut it.
fn entry_end() -> Span<'static> {
    Span::raw("")
}

/// Marks the end of a word of `menu_words`, which `fit_menu` may shorten.
/// Empty, so the style only tells it apart.
fn word_end() -> Span<'static> {
    Span::styled("", Style::new().add_modifier(Modifier::HIDDEN))
}

/// The fewest letters a shortened word keeps (fewer than this reads as noise).
const WORD_FLOOR: usize = 3;

/// Makes a menu line fit in `width` columns, never cutting an entry by
/// accident: first the words of `menu_words` are shortened, all to the same
/// number of letters and never losing their key; then entries go from the end,
/// with a `…` where they were. In a line of keys the last entry (F9, Esc)
/// stays. A line that does not end with an entry (a message added to it) is
/// left as it is.
fn fit_menu(line: Line<'static>, width: usize) -> Line<'static> {
    let ends_with_entry = line.spans.last().is_some_and(|s| s.content.is_empty());
    if fits(&line.spans, width) || !ends_with_entry {
        return line;
    }
    // The entries, each with its end marker.
    let mut entries: Vec<Vec<Span<'static>>> = Vec::new();
    let mut current = Vec::new();
    for span in line.spans {
        let end = span.content.is_empty();
        current.push(span);
        if end {
            entries.push(std::mem::take(&mut current));
        }
    }
    let is_word = |e: &[Span]| e.last().is_some_and(|s| s.style == word_end().style);
    let words = entries.iter().any(|e| is_word(e));

    if words {
        let longest = entries
            .iter()
            .filter(|e| is_word(e))
            .map(|e| word_label(e).chars().count())
            .max()
            .unwrap_or(0);
        let mut limit = longest;
        while limit > WORD_FLOOR {
            limit -= 1;
            let short: Vec<Vec<Span<'static>>> = entries
                .iter()
                .map(|e| {
                    if is_word(e) {
                        shorten_word(e, limit)
                    } else {
                        e.clone()
                    }
                })
                .collect();
            entries = short;
            if fits(&entries.concat(), width) {
                return Line::from(entries.concat());
            }
        }
    }

    // Entries go from the end; the first (the title) always stays, and so does
    // the last of a line of keys when it is a key (F9, Esc), not a legend.
    let starts_with_key = |e: &[Span]| e.first().is_some_and(|s| s.style == key("x").style);
    let keep_last =
        !words && entries.len() > 2 && entries.last().is_some_and(|e| starts_with_key(e));
    let last = keep_last.then(|| entries.pop()).flatten();
    let mut keep = entries.len();
    while keep > 1 {
        keep -= 1;
        let mut spans = entries[..keep].concat();
        // Apart from what comes before it: a hint has no space at its end.
        let spaced = spans
            .iter()
            .rev()
            .find(|s| !s.content.is_empty())
            .is_none_or(|s| s.content.ends_with(' '));
        let more = if spaced { "… " } else { " … " };
        spans.push(Span::styled(more, Style::new().fg(theme().dim)));
        if let Some(last) = &last {
            spans.extend(last.iter().cloned());
        }
        if fits(&spans, width) {
            return Line::from(spans);
        }
    }
    Line::from(entries[..1].concat())
}

/// Whether `spans` fit in `width` columns; the spaces at the end may go past
/// the edge.
fn fits(spans: &[Span], width: usize) -> bool {
    let text: String = spans.iter().map(|s| s.content.as_ref()).collect();
    Line::raw(text.trim_end()).width() <= width
}

/// The text of a word of `menu_words`, as it reads: "eXecute", "usage (W)".
fn word_label(entry: &[Span]) -> String {
    let text: String = entry.iter().map(|s| s.content.as_ref()).collect();
    text.trim_end().trim_start_matches("· ").to_string()
}

/// A word of `menu_words` cut to `limit` letters, keeping its key: "Compare
/// branch" → "Compar", "eXecute" → "eXecu", "· usage (W)" → "· us(W)" or "· (W)".
fn shorten_word(entry: &[Span<'static>], limit: usize) -> Vec<Span<'static>> {
    if word_label(entry).chars().count() <= limit {
        return entry.to_vec();
    }
    // [prefix], key, suffix (with its space), end marker.
    let (prefix, key, suffix, end) = match entry {
        [k, s, e] => (None, k, s, e),
        [p, k, s, e] => (Some(p), k, s, e),
        _ => return entry.to_vec(),
    };
    // A second word goes whole ("Compare branch" → "Compare", never
    // "Compare bra"); only the first one is cut letter by letter.
    let cut = |text: &str, n: usize| -> String {
        let text = text.trim_end();
        if text.chars().count() <= n {
            return text.to_string();
        }
        let t: String = text.chars().take(n).collect();
        match t.split_once(' ') {
            Some((head, _)) => head.to_string(),
            None => t,
        }
    };
    let mut out = Vec::new();
    match prefix {
        // "· usage (" + W + ") ": the name shrinks, the key stays in parentheses.
        Some(p) if p.content.ends_with('(') => {
            let deco = if p.content.starts_with("· ") {
                "· "
            } else {
                ""
            };
            let name = p.content[deco.len()..p.content.len() - 1].trim_end();
            let name = cut(name, limit.saturating_sub(3));
            out.push(Span::styled(format!("{deco}{name}("), p.style));
            out.push(key.clone());
            out.push(Span::styled(") ", suffix.style));
        }
        // "e" + X + "ecute ": what comes after the key shrinks.
        Some(p) => {
            let room = limit.saturating_sub(p.content.chars().count() + 1);
            out.push(p.clone());
            out.push(key.clone());
            out.push(Span::styled(
                format!("{} ", cut(&suffix.content, room)),
                suffix.style,
            ));
        }
        // C + "ompare branch ".
        None => {
            out.push(key.clone());
            let room = limit.saturating_sub(1);
            out.push(Span::styled(
                format!("{} ", cut(&suffix.content, room)),
                suffix.style,
            ));
        }
    }
    out.push(end.clone());
    out
}

/// Splits `text` into lines of at most `width` columns, at spaces.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(line) if line.chars().count() + 1 + word.chars().count() <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }
    lines
}

/// Cuts or pads with spaces to fit `width` columns.
fn fit(s: &str, width: usize) -> String {
    let len = s.chars().count();
    if len <= width {
        format!("{s:<width$}")
    } else {
        let mut out: String = s.chars().take(width.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

/// Like `fit`, but keeps the end of the text: "…/projetos/xt".
fn fit_tail(s: &str, width: usize) -> String {
    let len = s.chars().count();
    if len <= width {
        return s.to_string();
    }
    let tail: String = s.chars().skip(len + 1 - width.max(1)).collect();
    format!("…{tail}")
}

/// Number with the configured thousands separator.
fn fmt_num(n: u64) -> String {
    group_digits(n, formats().number.separator())
}

/// 1234567 -> "1,234,567" (with `,`).
fn fmt_time(t: SystemTime) -> String {
    DateTime::<Local>::from(t)
        .format(formats().date.pattern())
        .to_string()
}

/// Permissions in `ls -l` format, e.g.: "-rwxr-xr-x" (on Windows, the
/// attributes: "-r--a").
fn fmt_mode(mode: u32, kind: FileKind) -> String {
    let mut s = String::with_capacity(10);
    s.push(match kind {
        FileKind::File => '-',
        FileKind::Symlink => 'l',
        FileKind::Other => '?',
    });
    s.push_str(&crate::platform::format_mode(mode));
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_numbers() {
        let comma = Some(',');
        assert_eq!(group_digits(0, comma), "0");
        assert_eq!(group_digits(999, comma), "999");
        assert_eq!(group_digits(1000, comma), "1,000");
        assert_eq!(group_digits(1234567, comma), "1,234,567");
        assert_eq!(group_digits(1234567, Some('.')), "1.234.567");
        assert_eq!(group_digits(1234567, None), "1234567");
    }

    #[cfg(unix)]
    #[test]
    fn formats_modes() {
        assert_eq!(fmt_mode(0o755, FileKind::File), "-rwxr-xr-x");
        assert_eq!(fmt_mode(0o640, FileKind::Symlink), "lrw-r-----");
    }

    #[test]
    fn fits_names() {
        assert_eq!(fit("abc", 5), "abc  ");
        assert_eq!(fit("abcdef", 4), "abc…");
    }

    #[test]
    fn fits_tails() {
        assert_eq!(fit_tail("/a/b", 10), "/a/b");
        assert_eq!(fit_tail("/home/andre/xt", 6), "…re/xt");
    }

    #[test]
    fn window_follows_selection() {
        let mut state = ListState::default().with_selected(Some(0));
        let (range, local) = visible_window(&mut state, 100, 10);
        assert_eq!((range, local.selected()), (0..10, Some(0)));

        state.select(Some(15));
        let (range, local) = visible_window(&mut state, 100, 10);
        assert_eq!((range, local.selected()), (6..16, Some(9)));

        // Moving up inside the window, it doesn't move.
        state.select(Some(8));
        assert_eq!(visible_window(&mut state, 100, 10).0, 6..16);

        // The list shrank: the window moves back.
        state.select(Some(2));
        assert_eq!(visible_window(&mut state, 5, 10).0, 0..5);
        assert_eq!(visible_window(&mut state, 0, 10).0, 0..0);
    }

    #[test]
    fn menu_highlights_first_uppercase_letter() {
        let line = menu_words("", &["Copy", "eXecute"]);
        let keys: Vec<&str> = line
            .spans
            .iter()
            // The key letters are the bold parts (the theme may
            // be getting switched by another test at the same time).
            .filter(|s| {
                s.style.add_modifier.contains(Modifier::BOLD) && !s.content.trim().is_empty()
            })
            .map(|s| s.content.as_ref())
            .collect();
        assert_eq!(keys, ["C", "X"]);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text.trim(), "Copy eXecute");
    }

    #[test]
    fn splash_shows_the_name_in_the_middle() {
        use ratatui::{Terminal, backend::TestBackend};
        let t = Theme::classic();
        let screen = |w: u16, h: u16| {
            let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
            term.draw(|f| draw_splash(f, &t)).unwrap();
            term.backend().buffer().clone()
        };
        let buf = screen(80, 24);
        let row = |y: u16| {
            let line: String = (0..80).map(|x| buf[(x, y)].symbol()).collect();
            line.trim_matches(|c| c == ' ' || c == '│').to_string()
        };
        // 13 rows centered in 24: the box starts at row 5, the art at row 7.
        assert!(row(5).starts_with("┌─"));
        assert_eq!(row(7), SPLASH_ART[0]);
        let x = (0..80).find(|&x| buf[(x, 7)].symbol() == "█").unwrap();
        assert_eq!(x, (80 - 44) / 2);
        assert_eq!(buf[(x, 7)].fg, t.active_border);
        assert_eq!(buf[(x + 2, 7)].symbol(), "╗");
        assert_eq!(buf[(x + 2, 7)].fg, t.dim);
        assert_eq!(row(14), "F I L E   M A N A G E R");
        assert_eq!(row(16), concat!("v", env!("CARGO_PKG_VERSION")));
        // Too small for it: nothing.
        let small = screen(57, 24);
        assert!(small.content().iter().all(|c| c.symbol() == " "));
        let low = screen(80, 12);
        assert!(low.content().iter().all(|c| c.symbol() == " "));
    }

    #[test]
    fn header_shows_the_name_and_version_when_there_is_room() {
        use ratatui::{Terminal, backend::TestBackend};
        let tmp = crate::testing::tempdir();
        let mut app = App::new(tmp.path().to_path_buf());
        let header = |app: &mut App, width: u16| {
            let mut term = Terminal::new(TestBackend::new(width, 20)).unwrap();
            term.draw(|f| draw(f, app)).unwrap();
            let buf = term.backend().buffer();
            (0..width).map(|x| buf[(x, 0)].symbol()).collect::<String>()
        };
        let name = concat!("Xylem v", env!("CARGO_PKG_VERSION"), "  ");
        assert!(header(&mut app, 100).contains(name));
        assert!(!header(&mut app, 60).contains("Xylem"));
    }

    /// The footer's rows on a screen `width` columns wide and 20 high.
    fn footer_rows(app: &mut App, width: u16) -> Vec<String> {
        use ratatui::{Terminal, backend::TestBackend};
        let mut term = Terminal::new(TestBackend::new(width, 20)).unwrap();
        term.draw(|f| draw(f, app)).unwrap();
        let rows = if app.usage.is_some() {
            3
        } else {
            footer_height(app, width as usize)
        };
        let buf = term.backend().buffer();
        (20 - rows..20)
            .map(|y| (0..width).map(|x| buf[(x, y)].symbol()).collect())
            .collect()
    }

    #[test]
    fn the_footer_has_two_lines_when_every_menu_fits() {
        let tmp = crate::testing::tempdir();
        let mut app = App::new(tmp.path().to_path_buf());
        assert_eq!(footer_height(&app, 120), 3);
        assert_eq!(footer_height(&app, 250), 2);
        // Holding Ctrl or moving to the file window keeps the height.
        app.held = Some(MenuMode::Ctrl);
        app.focus = Focus::Files;
        assert_eq!(footer_height(&app, 120), 3);
        assert_eq!(footer_height(&app, 250), 2);
        // Two lines: the words, then the hints and the keys.
        let rows = footer_rows(&mut app, 250);
        assert!(rows[0].starts_with("CTRL "), "{rows:?}");
        assert!(rows[0].contains("New date"), "{rows:?}");
        assert!(rows[1].contains("on tagged files in list"), "{rows:?}");
        assert!(rows[1].contains("F9 next menu"), "{rows:?}");
    }

    #[test]
    fn footer_shows_menu_for_mode() {
        let tmp = crate::testing::tempdir();
        let mut app = App::new(tmp.path().to_path_buf());
        let footer = |app: &mut App| footer_rows(app, 140)[0].clone();
        assert!(footer(&mut app).starts_with("DIR "));
        app.held = Some(MenuMode::Ctrl);
        assert!(footer(&mut app).starts_with("CTRL "));
        app.held = None;
        app.sticky = MenuMode::Alt;
        assert!(footer(&mut app).starts_with("ALT "));
        // Each word has its own letter, and the tools come apart.
        let alt = footer(&mut app);
        assert!(
            alt.contains("Prune Quit here Release Sort Usage"),
            "{alt:?}"
        );
        assert!(!alt.contains('('), "{alt:?}");
    }

    #[test]
    fn each_shortcut_is_in_the_menu_of_its_modifier() {
        let tmp = crate::testing::tempdir();
        let mut app = App::new(tmp.path().to_path_buf());
        app.options.git = true;
        let footer = |app: &mut App| footer_rows(app, 200);
        for focus in [Focus::Tree, Focus::Files] {
            app.focus = focus;
            app.held = None;
            let normal = footer(&mut app).join("\n");
            for other in ["Alt+X", "Alt+B", "Ctrl+G"] {
                assert!(!normal.contains(other), "{other} in {normal:?}");
            }
            app.held = Some(MenuMode::Ctrl);
            let ctrl = footer(&mut app);
            assert!(ctrl[0].contains(" Git "), "{ctrl:?}");
            app.held = Some(MenuMode::Alt);
            let alt = footer(&mut app);
            // Alt+X is the terminal, not eXecute (that is X alone).
            assert!(!alt[0].contains("eXecute"), "{alt:?}");
            assert!(alt.last().unwrap().contains("X terminal"), "{alt:?}");
            assert!(alt[0].contains("Bookmarks"), "{alt:?}");
        }
        // Without the Git menu, Ctrl+G does nothing: it is not offered.
        app.options.git = false;
        app.held = Some(MenuMode::Ctrl);
        assert!(!footer(&mut app)[0].contains("Git"));
    }

    /// No command is invisible: each default key's command is in a footer
    /// menu (DIR, FILE, CTRL or ALT) of the window where it works, except
    /// moving around and second keys of commands shown elsewhere.
    #[test]
    fn every_command_is_in_a_menu() {
        use crate::input::Command as C;
        let tmp = crate::testing::tempdir();
        let mut app = App::new(tmp.path().to_path_buf());
        app.options.git = true;
        app.keymap = crate::keymap::Keymap::from_config(&Default::default(), &app.options).unwrap();
        // Moving around, and what has a menu of its own (the terminal's full
        // screen, the Delete confirmation that switches to permanent, the
        // bookmarks list that jumps).
        let exempt = [
            C::Up,
            C::Down,
            C::PageUp,
            C::PageDown,
            C::Home,
            C::End,
            C::Expand,
            C::Collapse,
            C::SwitchPane,
            C::Parent,
            C::Left,
            C::Right,
            C::Back,
            C::TerminalFull,
            C::BookmarkJump,
            C::Delete {
                tagged: false,
                permanent: true,
            },
            C::Delete {
                tagged: true,
                permanent: true,
            },
        ];
        let mut shown: [Vec<C>; 2] = Default::default();
        for (i, focus) in [Focus::Tree, Focus::Files].into_iter().enumerate() {
            app.focus = focus;
            for mode in [MenuMode::Normal, MenuMode::Ctrl, MenuMode::Alt] {
                app.held = Some(mode);
                let menu = idle_menu_spec(&app, focus, mode).only_what_runs(&app);
                let mut binds: Vec<String> = menu
                    .words
                    .iter()
                    .map(|w| format!("{}{}", menu.mods, word_key(w)))
                    .collect();
                for (_, _, b) in &menu.keys {
                    binds.extend(b.iter().map(|b| b.to_string()));
                }
                for b in binds {
                    if let Some(cmd) = app.keymap.lookup_text(&b, focus) {
                        shown[i].push(cmd);
                    }
                }
            }
        }
        let mut missing = Vec::new();
        for (key, focus, cmd) in crate::keymap::default_bindings() {
            if exempt.contains(&cmd) {
                continue;
            }
            let ok = match focus {
                Some(Focus::Tree) => shown[0].contains(&cmd),
                Some(Focus::Files) => shown[1].contains(&cmd),
                None => shown[0].contains(&cmd) || shown[1].contains(&cmd),
            };
            if !ok {
                missing.push(format!("{key} ({cmd:?})"));
            }
        }
        assert!(missing.is_empty(), "not in any menu: {missing:?}");
    }

    #[test]
    fn the_clock_shows_seconds_only_when_asked() {
        let at = chrono::NaiveDate::from_ymd_opt(2026, 11, 13)
            .unwrap()
            .and_hms_opt(9, 5, 7)
            .unwrap();
        let date = formats().date;
        assert_eq!(
            at.format(&clock_pattern(false)).to_string(),
            at.format(date.pattern()).to_string()
        );
        assert!(
            at.format(&clock_pattern(true))
                .to_string()
                .ends_with("09:05:07")
        );
    }

    #[test]
    fn the_small_file_window_grows_with_the_screen() {
        // 8 rows on an ordinary screen, as always; a quarter of a tall or a
        // short one, the tree keeping the most; at least 3 files.
        assert_eq!(small_file_window(31), 8);
        assert_eq!(small_file_window(97), 25);
        assert_eq!(small_file_window(16), SMALL_FILE_WINDOW);
        assert_eq!(small_file_window(12), SMALL_FILE_WINDOW);
    }

    fn text_of(line: &Line) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn a_menu_too_wide_shortens_its_words_and_then_loses_the_last() {
        let line = menu_words(
            "ALT",
            &[
                "Attributes",
                "Compare branch",
                "eXecute",
                "Sort",
                "· usage (W)",
            ],
        );
        let full = text_of(&line);
        assert_eq!(text_of(&fit_menu(line.clone(), 100)), full);
        // Room for the last word but not its space keeps it all.
        let end = full.trim_end().chars().count();
        assert_eq!(text_of(&fit_menu(line.clone(), end)), full);
        // A little less: the longest words shrink first, all to the same
        // length; a second word goes whole; the key always stays.
        assert_eq!(
            text_of(&fit_menu(line.clone(), end - 7)).trim_end(),
            "ALT      Attributes Compare eXecute Sort · usage (W)"
        );
        assert_eq!(
            text_of(&fit_menu(line.clone(), end - 8)).trim_end(),
            "ALT      Attribute Compare eXecute Sort · usage (W)"
        );
        assert_eq!(
            text_of(&fit_menu(line.clone(), end - 14)).trim_end(),
            "ALT      Attrib Compar eXecut Sort · usa(W)"
        );
        // At the floor, every word has 3 letters; one with its key in
        // parentheses keeps only that.
        let floor = "ALT      Att Com eXe Sor · (W)";
        assert_eq!(
            text_of(&fit_menu(line.clone(), floor.chars().count())).trim_end(),
            floor
        );
        // Less than that: the last words go, and … says so.
        assert_eq!(
            text_of(&fit_menu(line, 22)).trim_end(),
            "ALT      Att Com eXe …"
        );

        // A legend at the end, with no key, goes first.
        let legend = menu_line("", &[("F3", "reread"), ("", "> other disk  ! unreadable")]);
        assert_eq!(
            text_of(&fit_menu(legend, 30)).trim_end(),
            "         F3 reread   …"
        );

        // A long hint loses its last parts, not all of it, and F9 stays.
        let hint = menu_line(
            "",
            &[
                ("", "on tagged files · Copy/Move: no paths"),
                ("F9", "next menu"),
            ],
        );
        assert_eq!(
            text_of(&fit_menu(hint, 50)).trim_end(),
            "          on tagged files … F9 next menu"
        );
    }

    #[test]
    fn a_menu_with_a_message_is_not_cut_by_entries() {
        let mut line = menu_line("", &[("Esc", "cancel")]);
        line.spans
            .push(Span::raw("  a long message that goes past the edge"));
        let full = text_of(&line);
        assert_eq!(text_of(&fit_menu(line, 20)), full);
    }

    /// `word` is one of `words`, the `…` marker, or one of them shortened
    /// on purpose: at least 3 letters, or its key ("eX", "us(W)", "(W)").
    fn is_whole_or_short(word: &str, words: &[&str]) -> bool {
        if word == "…" || words.contains(&word) {
            return true;
        }
        let (stem, key) = match word.find('(') {
            Some(i) => (&word[..i], Some(&word[i..])),
            None => (word, None),
        };
        let stem_ok = stem.is_empty() && key.is_some()
            || words.iter().any(|w| {
                w.starts_with(stem)
                    && (stem.chars().count() >= 3 || stem.chars().any(|c| c.is_uppercase()))
            });
        stem_ok && key.is_none_or(|k| words.contains(&k))
    }

    #[test]
    fn a_narrow_footer_never_cuts_a_word() {
        let tmp = crate::testing::tempdir();
        let mut app = App::new(tmp.path().to_path_buf());
        // Two lines when wide, three when narrow: every word of the narrow
        // footer is a word of the wide one, whole or shortened on purpose.
        for mode in [MenuMode::Normal, MenuMode::Ctrl, MenuMode::Alt] {
            app.held = Some(mode);
            let wide = footer_rows(&mut app, 250).join(" ");
            let words: Vec<&str> = wide.split_whitespace().collect();
            for narrow in footer_rows(&mut app, 60) {
                for word in narrow.split_whitespace() {
                    assert!(
                        is_whole_or_short(word, &words),
                        "{word:?} cut in {narrow:?}"
                    );
                }
            }
        }
        let footer = |app: &mut App, width: u16| {
            use ratatui::{Terminal, backend::TestBackend};
            let mut term = Terminal::new(TestBackend::new(width, 20)).unwrap();
            term.draw(|f| draw(f, app)).unwrap();
            let buf = term.backend().buffer();
            [17, 18, 19].map(|y| (0..width).map(|x| buf[(x, y)].symbol()).collect::<String>())
        };
        // The bookmarks screen has a footer of its own, in its last two rows.
        app.held = None;
        app.bookmarks_view = Some(BookmarksView::default());
        let wide = footer(&mut app, 250);
        for (narrow, wide) in footer(&mut app, 60).iter().zip(&wide).skip(1) {
            let words: Vec<&str> = wide.split_whitespace().collect();
            for word in narrow.split_whitespace() {
                assert!(
                    is_whole_or_short(word, &words),
                    "{word:?} cut in {narrow:?}"
                );
            }
        }
    }

    #[test]
    fn an_archive_menu_leaves_out_what_writes() {
        let tmp = crate::testing::tempdir();
        let mut app = App::new(tmp.path().to_path_buf());
        let footer = |app: &mut App| footer_rows(app, 200)[0].clone();
        app.held = Some(MenuMode::Ctrl);
        assert!(footer(&mut app).contains("Delete"));
        app.archive = Some(tmp.path().join("p.zip"));
        for mode in [MenuMode::Ctrl, MenuMode::Alt] {
            app.held = Some(mode);
            let menu = footer(&mut app);
            for word in ["Attributes", "Delete", "Rename", "Prune", "eXecute"] {
                assert!(!menu.contains(word), "{word} in {menu:?}");
            }
            let kept = if mode == MenuMode::Ctrl {
                "Tag"
            } else {
                "Bookmarks"
            };
            assert!(menu.contains(kept), "{menu:?}");
        }
    }

    fn draw_row0(app: &mut App) -> String {
        use ratatui::{Terminal, backend::TestBackend};
        let mut term = Terminal::new(TestBackend::new(120, 20)).unwrap();
        term.draw(|f| draw(f, app)).unwrap();
        let buf = term.backend().buffer();
        (0..120).map(|x| buf[(x, 0)].symbol()).collect()
    }

    fn draw_text(app: &mut App) -> String {
        use ratatui::{Terminal, backend::TestBackend};
        let mut term = Terminal::new(TestBackend::new(140, 20)).unwrap();
        term.draw(|f| draw(f, app)).unwrap();
        let buf = term.backend().buffer();
        (0..20)
            .flat_map(|y| (0..140).map(move |x| (x, y)))
            .map(|(x, y)| buf[(x, y)].symbol())
            .collect()
    }

    #[test]
    fn viewer_header_tells_what_was_cut() {
        use crate::viewer::{LoadOptions, Viewer};
        let tmp = crate::testing::tempdir();
        let mut app = App::new(tmp.path().to_path_buf());
        let load = LoadOptions::view(1 << 20);
        // Lines long enough for the line budget: these are cut by bytes.
        let text = format!("{}\n", "0123456789abcdef".repeat(4)).repeat(1 << 15);
        let raw = text.as_bytes()[..1 << 20].to_vec();
        app.viewer = Some(Viewer::from_file(
            "x.txt".into(),
            text.len() as u64,
            raw,
            load,
        ));
        let row0 = draw_row0(&mut app);
        assert!(row0.contains("(showing first 1 MiB)"), "{row0}");
        let gz = crate::decompress::test_util::gz(text.as_bytes());
        let n = gz.len() as u64;
        app.viewer = Some(Viewer::from_file("x.txt.gz".into(), n, gz, load));
        let row0 = draw_row0(&mut app);
        assert!(
            row0.contains(&format!(
                "(showing first {} bytes decompressed)",
                fmt_num(1 << 20)
            )),
            "{row0}"
        );
        assert!(row0.contains("gzip → 1"), "{row0}");
    }

    /// The note goes away at the first key; the header keeps saying that
    /// the compressed file was damaged, so it never looks complete.
    #[test]
    fn viewer_header_keeps_a_decompression_failure() {
        use crate::viewer::{LoadOptions, Viewer};
        let tmp = crate::testing::tempdir();
        let mut app = App::new(tmp.path().to_path_buf());
        let load = LoadOptions::view(1 << 20);
        let down = ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Down,
            ratatui::crossterm::event::KeyModifiers::NONE,
        );
        let mut gz = crate::decompress::test_util::gz(&b"line of text\n".repeat(10_000));
        gz.truncate(gz.len() / 2);
        for (raw, shown) in [
            (gz, "(truncated or corrupt)"),
            (
                b"\x1F\x8B".to_vec(),
                "(not decompressed: truncated or corrupt)",
            ),
        ] {
            let n = raw.len() as u64;
            let mut v = Viewer::from_file("x.txt.gz".into(), n, raw, load);
            assert!(v.note.is_some());
            v.handle_key(down);
            assert_eq!(v.note, None);
            app.viewer = Some(v);
            let row0 = draw_row0(&mut app);
            assert!(row0.contains(shown), "{row0}");
        }
    }

    #[test]
    fn the_structure_shows_over_the_table_and_the_footer_has_its_keys() {
        use crate::viewer::{LoadOptions, Viewer};
        use ratatui::crossterm::event::{KeyCode, KeyEvent};
        let tmp = crate::testing::tempdir();
        let mut app = App::new(tmp.path().to_path_buf());
        let path = tmp.path().join("d.csv");
        std::fs::write(&path, "code,name\n7,Ana\n12,Rui\n").unwrap();
        app.viewer = Some(Viewer::open(path, LoadOptions::view(1 << 20)).unwrap());
        let screen = draw_text(&mut app);
        assert!(screen.contains("i structure"), "{screen}");
        assert!(screen.contains("Ctrl+End first, last row"), "{screen}");
        let v = app.viewer.as_mut().unwrap();
        v.handle_key(KeyEvent::from(KeyCode::Char('i')));
        let screen = draw_text(&mut app);
        assert!(screen.contains("Structure: 2 columns"), "{screen}");
        assert!(
            screen.contains("#  Column  Values seen  Widest"),
            "{screen}"
        );
        assert!(
            screen.contains("1  code    integer           2"),
            "{screen}"
        );
        assert!(
            screen.contains("2  name    text              3"),
            "{screen}"
        );
        assert!(screen.contains("COLUMNS  ↑↓ PgUp PgDn scroll"), "{screen}");
    }

    /// Cut by the line budget, not by the bytes read: the header counts lines.
    #[test]
    fn viewer_header_counts_the_lines_the_budget_kept() {
        use crate::viewer::{LoadOptions, Viewer};
        let tmp = crate::testing::tempdir();
        let mut app = App::new(tmp.path().to_path_buf());
        let raw = vec![b'\n'; 1 << 20];
        let v = Viewer::from_file("nl.txt".into(), 1 << 20, raw, LoadOptions::view(1 << 20));
        let lines = format!("(showing first {} lines)", fmt_num(v.line_count() as u64));
        app.viewer = Some(v);
        let row0 = draw_row0(&mut app);
        assert!(row0.contains(&lines), "{row0}");
        assert!(!row0.contains("MiB)"), "{row0}");
        // The hex view shows every byte: nothing was cut there.
        app.viewer
            .as_mut()
            .unwrap()
            .handle_key(ratatui::crossterm::event::KeyEvent::new(
                ratatui::crossterm::event::KeyCode::Char('h'),
                ratatui::crossterm::event::KeyModifiers::NONE,
            ));
        let row0 = draw_row0(&mut app);
        assert!(!row0.contains("(showing first"), "{row0}");
    }

    #[test]
    fn viewer_header_names_the_charset_before_the_cut() {
        use crate::viewer::{LoadOptions, Viewer};
        let tmp = crate::testing::tempdir();
        let mut app = App::new(tmp.path().to_path_buf());
        let text = "ação\n".repeat(1 << 18);
        let mut raw = crate::charset::test_util::encode(&text, &crate::charset::CP850);
        raw.truncate(1 << 20);
        app.viewer = Some(Viewer::from_file(
            "x.txt".into(),
            text.len() as u64,
            raw,
            LoadOptions::view(1 << 20),
        ));
        let row0 = draw_row0(&mut app);
        let at = row0.find("CP850").unwrap_or_else(|| panic!("{row0}"));
        assert!(at < row0.find("(showing first").unwrap(), "{row0}");
        let all = draw_text(&mut app);
        assert!(all.contains("cOdepage") || all.contains("Odepage"), "{all}");
    }

    #[test]
    fn autoview_title_shows_the_note() {
        let tmp = crate::testing::tempdir();
        std::fs::write(
            tmp.path().join("big.xz"),
            crate::decompress::test_util::xz_header_with_dict_props(40),
        )
        .unwrap();
        let mut app = App::new(tmp.path().to_path_buf());
        app.handle_key(ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::F(7),
            ratatui::crossterm::event::KeyModifiers::NONE,
        ));
        let text = draw_text(&mut app);
        assert!(text.contains("(V to view)"), "{text}");
    }

    #[test]
    fn viewer_header_counts_bidi_controls() {
        use ratatui::{Terminal, backend::TestBackend};
        let tmp = crate::testing::tempdir();
        let mut app = App::new(tmp.path().to_path_buf());
        app.viewer = Some(crate::viewer::Viewer::from_bytes(
            "x.txt".into(),
            6,
            "a\u{202E}b\u{2066}c".as_bytes().to_vec(),
        ));
        let mut term = Terminal::new(TestBackend::new(100, 20)).unwrap();
        term.draw(|f| draw(f, &mut app)).unwrap();
        let buf = term.backend().buffer();
        let row0: String = (0..100).map(|x| buf[(x, 0)].symbol()).collect();
        assert!(
            row0.contains("2 bidirectional control characters (shown as ·)"),
            "{row0}"
        );
        app.viewer = Some(crate::viewer::Viewer::from_bytes(
            "x.txt".into(),
            3,
            "a\u{202E}b".as_bytes().to_vec(),
        ));
        term.draw(|f| draw(f, &mut app)).unwrap();
        let buf = term.backend().buffer();
        let row0: String = (0..100).map(|x| buf[(x, 0)].symbol()).collect();
        assert!(
            row0.contains("1 bidirectional control character (shown as ·)"),
            "{row0}"
        );
    }

    #[test]
    fn bidi_in_names_never_reaches_the_terminal() {
        use ratatui::{Terminal, backend::TestBackend};
        let tmp = crate::testing::tempdir();
        std::fs::write(tmp.path().join("fatura\u{202E}fdp.sh"), "").unwrap();
        let mut app = App::new(tmp.path().to_path_buf());
        app.apply(crate::input::Command::SwitchWindow);
        let mut term = Terminal::new(TestBackend::new(80, 20)).unwrap();
        term.draw(|f| draw(f, &mut app)).unwrap();
        let buf = term.backend().buffer();
        assert!(
            buf.content
                .iter()
                .all(|c| !crate::bidi::has_bidi(c.symbol()))
        );
        let screen: String = buf.content.iter().map(|c| c.symbol()).collect();
        assert!(screen.contains("faturafdp.sh"), "{screen}");
    }

    #[test]
    fn footer_warns_about_a_name_with_hidden_bidi() {
        use ratatui::{Terminal, backend::TestBackend};
        let tmp = crate::testing::tempdir();
        std::fs::write(tmp.path().join("fatura\u{202E}fdp.sh"), "").unwrap();
        let mut app = App::new(tmp.path().to_path_buf());
        app.apply(crate::input::Command::SwitchWindow);
        let mut term = Terminal::new(TestBackend::new(100, 20)).unwrap();
        term.draw(|f| draw(f, &mut app)).unwrap();
        let buf = term.backend().buffer();
        let row: String = (0..100).map(|x| buf[(x, 17)].symbol()).collect();
        assert!(
            row.contains("Name has hidden bidirectional characters"),
            "{row}"
        );
    }

    #[test]
    fn renders_files_in_columns() {
        use ratatui::{Terminal, backend::TestBackend};
        let tmp = crate::testing::tempdir();
        for name in ["aa", "bb", "cc", "dd", "ee"] {
            std::fs::write(tmp.path().join(name), "").unwrap();
        }
        let mut app = App::new(tmp.path().to_path_buf());
        app.apply(crate::input::Command::SwitchWindow);
        app.display = FileDisplay::Names;
        let mut term = Terminal::new(TestBackend::new(60, 12)).unwrap();
        term.draw(|f| draw(f, &mut app)).unwrap();
        let buf = term.backend().buffer();
        let row: String = (0..60).map(|x| buf[(x, 2)].symbol()).collect();
        assert!(
            row.contains("aa") && row.contains("bb") && row.contains("cc"),
            "{row}"
        );
        assert!(app.file_cols > 1);
    }

    #[test]
    fn renders_split_panes() {
        use ratatui::{Terminal, backend::TestBackend};
        let tmp = crate::testing::tempdir();
        std::fs::create_dir(tmp.path().join("left-dir")).unwrap();
        let mut app = App::new(tmp.path().to_path_buf());
        app.apply(crate::input::Command::Split);
        for (w, h) in [(120, 30), (30, 10)] {
            let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
            term.draw(|f| draw(f, &mut app)).unwrap();
        }
        let mut term = Terminal::new(TestBackend::new(120, 30)).unwrap();
        term.draw(|f| draw(f, &mut app)).unwrap();
        let screen = format!("{:?}", term.backend().buffer());
        assert_eq!(screen.matches("left-dir").count(), 2, "one per pane");
        assert!(
            !screen.contains("DISK Statistics"),
            "no stats panel when split"
        );
    }

    #[test]
    fn share_and_bar() {
        // The largest sibling fills the bar; half of it, half the bar.
        assert_eq!(
            share(30, 100, 30, 40),
            (Some("████████".into()), "  30%".into())
        );
        assert_eq!(
            share(15, 100, 30, 40),
            (Some("████".into()), "  15%".into())
        );
        // Eighths of a character: 1/16 of the largest is one cell.
        assert_eq!(share(2, 100, 32, 40).0.as_deref(), Some("▌"));
        assert_eq!(share(1, 1000, 30, 40).1, " 0.1%");
        assert_eq!(share(58, 1000, 30, 40).1, " 5.8%");
        // Something, but too little to show: not "0.0%".
        assert_eq!(share(1, 100_000, 30, 40).1, "<0.1%");
        assert_eq!(share(0, 0, 0, 40), (Some(String::new()), "   0%".into()));
        // No room for the bar: only the share.
        assert_eq!(share(30, 100, 30, 10), (None, "  30%".into()));
    }

    #[test]
    fn alt_k_percent_in_the_tree() {
        use ratatui::{Terminal, backend::TestBackend};
        let tmp = crate::testing::tempdir();
        for (dir, size) in [("big", 3000), ("small", 1000)] {
            std::fs::create_dir(tmp.path().join(dir)).unwrap();
            std::fs::write(tmp.path().join(dir).join("f"), vec![0u8; size]).unwrap();
        }
        let mut app = App::new(tmp.path().to_path_buf());
        app.apply(crate::input::Command::ExpandAll);
        while app.busy() {
            app.tick(std::time::Duration::from_millis(50));
        }
        app.kbyte.show = KbyteShow::Percent;
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        term.draw(|f| draw(f, &mut app)).unwrap();
        let buf = term.backend().buffer();
        let line = |name: &str| {
            (0..30)
                .map(|y| (0..100).map(|x| buf[(x, y)].symbol()).collect::<String>())
                .find(|l| l.contains(name))
                .unwrap_or_default()
        };
        assert!(line("big").contains("████████   75%"), "{}", line("big"));
        assert!(
            line("small").contains("██▋░░░░░   25%"),
            "{}",
            line("small")
        );
        assert!(line("% of parent").contains("Tree"));
    }

    #[test]
    fn renders_without_panicking() {
        use ratatui::{Terminal, backend::TestBackend};
        let tmp = crate::testing::tempdir();
        std::fs::create_dir(tmp.path().join("sub")).unwrap();
        std::fs::write(tmp.path().join("file.txt"), "hi").unwrap();

        let mut app = App::new(tmp.path().to_path_buf());
        for (w, h) in [(100, 30), (40, 12), (10, 5)] {
            let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
            term.draw(|f| draw(f, &mut app)).unwrap();
        }
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        term.draw(|f| draw(f, &mut app)).unwrap();
        let screen = format!("{:?}", term.backend().buffer());
        assert!(screen.contains("sub"));
        assert!(screen.contains("file.txt"));
    }

    #[test]
    fn renders_bookmarks_modal() {
        use ratatui::{Terminal, backend::TestBackend};
        let tmp = crate::testing::tempdir();
        let mut app = App::new(tmp.path().to_path_buf());

        // Empty bookmarks view
        app.bookmarks_view = Some(BookmarksView::default());
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        term.draw(|f| draw(f, &mut app)).unwrap();
        let screen = format!("{:?}", term.backend().buffer());
        assert!(screen.contains("Directory Bookmarks"));
        assert!(screen.contains("No bookmarks defined yet"));

        // With bookmarks
        app.set_bookmark("1".into(), tmp.path().to_path_buf());
        app.set_bookmark("x".into(), tmp.path().join("sub"));
        term.draw(|f| draw(f, &mut app)).unwrap();
        let screen = format!("{:?}", term.backend().buffer());
        assert!(screen.contains("[1]"));
        assert!(screen.contains("[x]"));
        assert!(screen.contains("2 bookmarks"));

        // A name before the place, which always shows; a folder that is gone
        // is marked, a remote one is never checked.
        app.set_bookmark("o".into(), "ssh://oracle@db01/u01/app".into());
        app.name_bookmark("o", "Servidor do Oracle").unwrap();
        app.bookmarks.get_mut("x").unwrap().label = Some("ev\u{202E}il".into());
        app.open_bookmarks();
        let text = |term: &Terminal<TestBackend>| {
            let buf = term.backend().buffer();
            (0..buf.area.height)
                .map(|y| {
                    (0..buf.area.width)
                        .map(|x| buf[(x, y)].symbol().to_string())
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
        };
        term.draw(|f| draw(f, &mut app)).unwrap();
        let rows = text(&term);
        let row = |k: &str| rows.iter().find(|r| r.contains(k)).unwrap().clone();
        let oracle = row("[o]");
        assert!(
            oracle.contains("Servidor do Oracle  ssh://oracle@db01/u01/app"),
            "{oracle}"
        );
        assert!(!oracle.contains("[not found]"), "{oracle}");
        assert!(row("[x]").contains("ev·il"), "{}", row("[x]"));
        assert!(row("[x]").contains("[not found]"), "{}", row("[x]"));
        assert!(!row("[1]").contains("[not found]"), "{}", row("[1]"));

        // Narrow: the place is cut at its end, never left out.
        let mut narrow = Terminal::new(TestBackend::new(40, 30)).unwrap();
        narrow.draw(|f| draw(f, &mut app)).unwrap();
        let oracle = text(&narrow)
            .into_iter()
            .find(|r| r.contains("[o]"))
            .unwrap();
        assert!(oracle.contains("ssh://"), "{oracle}");
        assert!(oracle.contains('…'), "{oracle}");

        // The name is typed over the list.
        app.handle_key(KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE));
        term.draw(|f| draw(f, &mut app)).unwrap();
        assert!(
            text(&term)
                .iter()
                .any(|r| r.contains("NAME of bookmark '1'"))
        );
    }

    #[test]
    fn icons_switch_in_file_list() {
        let tmp = crate::testing::tempdir();
        std::fs::write(tmp.path().join("main.rs"), "fn main() {}").unwrap();
        let mut app = App::new(tmp.path().to_path_buf());
        app.options.icons = IconStyle::None;
        let screen_none = draw_text(&mut app);
        assert!(!screen_none.contains("\u{e7a8}"));

        app.options.icons = IconStyle::NerdFonts;
        let screen_icons = draw_text(&mut app);
        assert!(screen_icons.contains("\u{e7a8}"));
    }

    fn entry(name: &str) -> FileEntry {
        FileEntry {
            name: name.into(),
            size: 10,
            disk: 10,
            link: None,
            modified: None,
            mode: 0o644,
            kind: FileKind::File,
            tagged: false,
        }
    }

    /// The row as the terminal shows it, `width` columns wide.
    fn item_text(item: ListItem<'static>, width: u16) -> String {
        use ratatui::{Terminal, backend::TestBackend};
        let mut term = Terminal::new(TestBackend::new(width, 1)).unwrap();
        term.draw(|f| f.render_widget(List::new([item]), f.area()))
            .unwrap();
        let buf = term.backend().buffer();
        (0..width).map(|x| buf[(x, 0)].symbol()).collect()
    }

    #[test]
    fn an_editor_in_the_frame_says_it_is_another_program_and_how_to_leave() {
        use ratatui::{Terminal, backend::TestBackend};
        let tmp = crate::testing::tempdir();
        let mut app = App::new(tmp.path().to_path_buf());
        let file = tmp.path().join("a.bin");
        std::fs::write(&file, [0u8, 1]).unwrap();
        // Waits for a line that never comes, so that it is on screen.
        let ext = crate::shell::editor_command("cat", &file, tmp.path());
        app.start_framed_editor(ext).unwrap();
        app.editor_pty.as_mut().unwrap().program = "hexedit".into();
        let mut term = Terminal::new(TestBackend::new(120, 20)).unwrap();
        term.draw(|f| draw(f, &mut app)).unwrap();
        let buf = term.backend().buffer();
        let text: String = (0..20)
            .flat_map(|y| (0..120).map(move |x| (x, y)))
            .map(|(x, y)| buf[(x, y)].symbol())
            .collect();
        assert!(text.contains("hexedit (external program)"), "{text}");
        assert!(text.contains("Esc may not close it"), "{text}");
        assert!(text.contains("Ctrl+C quits without saving"), "{text}");
        assert_eq!(quit_keys("heh"), "quit this program");
    }

    #[test]
    fn the_list_of_sheets_scrolls_through_hundreds_of_them() {
        use crate::viewer::{LoadOptions, Viewer};
        use ratatui::crossterm::event::{KeyCode, KeyEvent};
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("many.csv");
        std::fs::write(&path, "id\n1\n").unwrap();
        let mut app = App::new(tmp.path().to_path_buf());
        let mut v = Viewer::open(path, LoadOptions::view(1 << 20)).unwrap();
        v.table.as_mut().unwrap().sheets = (0..300).map(|i| format!("part{i:03}")).collect();
        v.handle_key(KeyEvent::from(KeyCode::Tab));
        app.viewer = Some(v);

        let text = draw_text(&mut app);
        assert!(text.contains("Sheets: 300 of 300"), "{text}");
        assert!(text.contains("* part000"), "{text}");
        assert!(text.contains("SHEETS"), "{text}");

        let v = app.viewer.as_mut().unwrap();
        v.handle_key(KeyEvent::from(KeyCode::End));
        let text = draw_text(&mut app);
        assert!(text.contains("part299"), "{text}");
        assert!(!text.contains("* part000"), "{text}");

        // Typing narrows, and the list says when nothing is left.
        let v = app.viewer.as_mut().unwrap();
        for c in "t29".chars() {
            v.handle_key(KeyEvent::from(KeyCode::Char(c)));
        }
        let text = draw_text(&mut app);
        assert!(text.contains("Find: t29_"), "{text}");
        assert!(text.contains("Sheets: 10 of 300"), "{text}");
        let v = app.viewer.as_mut().unwrap();
        v.handle_key(KeyEvent::from(KeyCode::Char('x')));
        let text = draw_text(&mut app);
        assert!(text.contains("No sheet has this name"), "{text}");
    }

    #[test]
    fn a_wide_table_scrolls_to_the_cursor_column() {
        use crate::viewer::table::TableRow;
        use ratatui::{Terminal, backend::TestBackend};
        // 12 columns of 10 characters: about 5 fit in 60.
        let headers: Vec<String> = (0..12).map(|i| format!("COLUMN_{i:02}")).collect();
        let row = TableRow {
            cells: (0..12).map(|i| format!("value_{i:02}")).collect(),
        };
        let mut t = TableGrid::new("t".into(), headers, vec![row]);
        let draw = |t: &mut TableGrid| {
            let mut term = Terminal::new(TestBackend::new(60, 8)).unwrap();
            term.draw(|f| {
                let area = f.area();
                t.show_cursor_col(panel("", false).inner(area).width.into());
                draw_table_grid(f, t, String::new(), area);
            })
            .unwrap();
            let buf = term.backend().buffer();
            (0..60).map(|x| buf[(x, 1)].symbol()).collect::<String>()
        };
        assert!(draw(&mut t).contains("COLUMN_00"));
        // Right to the last column: the view follows it.
        for _ in 0..11 {
            t.handle_key(KeyCode::Right, 4);
        }
        let header = draw(&mut t);
        assert!(header.contains("COLUMN_11"), "{header}");
        assert!(!header.contains("COLUMN_00"), "{header}");
        // End and Home too.
        t.handle_key(KeyCode::Home, 4);
        assert!(draw(&mut t).contains("COLUMN_00"));
        t.handle_key(KeyCode::End, 4);
        assert!(draw(&mut t).contains("COLUMN_11"));
        // Back left, one column at a time, it stays in view.
        for _ in 0..11 {
            t.handle_key(KeyCode::Left, 4);
            let header = draw(&mut t);
            assert!(
                header.contains(&format!("COLUMN_{:02}", t.cursor_col)),
                "{header}"
            );
        }
    }

    #[test]
    fn a_column_wider_than_the_screen_still_shows_its_start() {
        use crate::viewer::table::TableRow;
        use ratatui::{Terminal, backend::TestBackend};
        let row = TableRow {
            cells: vec!["x".repeat(40), "a lot of text here, more than fits".into()],
        };
        let mut t = TableGrid::new("t".into(), vec!["A".into(), "B".into()], vec![row]);
        t.handle_key(KeyCode::Right, 4);
        let mut term = Terminal::new(TestBackend::new(30, 8)).unwrap();
        term.draw(|f| {
            let area = f.area();
            t.show_cursor_col(panel("", false).inner(area).width.into());
            draw_table_grid(f, &t, String::new(), area);
        })
        .unwrap();
        let buf = term.backend().buffer();
        let row: String = (0..30).map(|x| buf[(x, 2)].symbol()).collect();
        assert!(row.contains("a lot of"), "{row}");
    }

    #[test]
    fn a_table_with_sheets_says_which_one_and_how_to_change_it() {
        let mut t = TableGrid::new(String::new(), Vec::new(), Vec::new());
        t.sheets = vec!["only".into()];
        assert_eq!(table_frame_title(&t), " TABLE ");
        assert_eq!(sheet_keys(&t, Path::new("a.xlsx")), "");

        t.sheets = vec!["customers".into(), "clients".into()];
        t.current_sheet = 1;
        assert_eq!(table_frame_title(&t), " TABLE [2/2: clients] ");
        assert_eq!(
            sheet_keys(&t, Path::new("a.xlsx")),
            "  ·  Tab list  ·  [ ] other sheet"
        );
        assert_eq!(
            sheet_keys(&t, Path::new("a.DB")),
            "  ·  Tab list  ·  [ ] other table"
        );

        t.first_view = 1;
        assert_eq!(table_frame_title(&t), " TABLE [2/2: clients (view)] ");
        t.first_view = usize::MAX;

        // A long name does not take the frame.
        t.sheets[1] = "é".repeat(50);
        let title = table_frame_title(&t);
        assert_eq!(
            title.chars().filter(|&c| c == 'é').count(),
            SHEET_NAME_CHARS
        );
        assert!(title.ends_with("…] "), "{title}");
    }

    #[test]
    fn icons_keep_the_details_column_with_git_status() {
        // Unchanged, modified, untracked and a mix: the size, date and
        // permissions start in the same column on every row.
        for xy in [*b"  ", *b" M", *b"??", *b"AM"] {
            let item = file_line(
                &entry("main.rs"),
                80,
                Detail::Standard,
                Some(xy),
                IconStyle::NerdFonts,
            );
            assert_eq!(item.width(), 80, "{}", String::from_utf8_lossy(&xy));
        }
    }

    #[test]
    fn git_status_letters_stay_with_icons() {
        // Index and work tree apart: " M" is not "M ".
        for xy in [*b" M", *b"M ", *b"??"] {
            let item = file_line(
                &entry("main.rs"),
                80,
                Detail::Standard,
                Some(xy),
                IconStyle::NerdFonts,
            );
            let text = item_text(item, 80);
            let letters = String::from_utf8_lossy(&xy);
            assert!(
                text.starts_with(&format!("  {letters} \u{e7a8} main.rs")),
                "{text}"
            );
        }
    }

    #[test]
    fn branch_title_is_not_a_git_branch() {
        // B is XTree's branch view (the files of a subtree), not Git's.
        let tmp = crate::testing::tempdir();
        std::fs::create_dir(tmp.path().join("sub")).unwrap();
        let mut app = App::new(tmp.path().to_path_buf());
        app.options.icons = IconStyle::NerdFonts;
        app.panes[0].set_view(FileView::Branch(crate::model::tree::ROOT));
        let screen = draw_text(&mut app);
        assert!(screen.contains("\u{f0645} Branch: "), "{screen}");
        assert!(!screen.contains('\u{e725}'));
    }

    #[test]
    fn tags_only_title_says_tagged() {
        let tmp = crate::testing::tempdir();
        std::fs::write(tmp.path().join("a"), "a").unwrap();
        std::fs::write(tmp.path().join("b"), "b").unwrap();
        let mut app = App::new(tmp.path().to_path_buf());
        app.panes[0]
            .tree
            .file_mut(crate::model::tree::ROOT, 0)
            .tagged = true;
        app.panes[0].set_view_tagged(FileView::Showall);
        let spec = app.panes[0].filespec.text().to_string();
        let screen = draw_text(&mut app);
        let title = format!("Showall (tagged): {spec} (1)");
        assert!(screen.contains(&title), "{screen}");
    }

    #[test]
    fn tree_marks_git_repositories() {
        let tmp = crate::testing::tempdir();
        std::fs::create_dir_all(tmp.path().join("proj").join(".git")).unwrap();
        // A worktree or submodule: `.git` is a file.
        std::fs::create_dir(tmp.path().join("wt")).unwrap();
        std::fs::write(tmp.path().join("wt").join(".git"), "gitdir: ../x").unwrap();
        std::fs::create_dir(tmp.path().join("plain")).unwrap();
        let mut app = App::new(tmp.path().to_path_buf());
        app.apply(crate::input::Command::ExpandAll);
        while app.busy() {
            app.tick(std::time::Duration::from_millis(50));
        }
        app.options.icons = IconStyle::NerdFonts;
        let screen = draw_text(&mut app);
        assert!(screen.contains("\u{e5fb} proj"), "{screen}");
        assert!(screen.contains("\u{e5fb} wt"), "{screen}");
        assert!(!screen.contains("\u{e5fb} plain"), "{screen}");
    }

    fn file_entry(name: &str, size: u64, mode: u32) -> FileEntry {
        FileEntry {
            name: name.into(),
            size,
            disk: size,
            link: None,
            modified: None,
            mode,
            kind: FileKind::File,
            tagged: false,
        }
    }

    #[test]
    fn file_names_take_the_color_of_their_type() {
        let t = Theme::classic();
        let fg = |f: &FileEntry| file_style(&t, f).fg;
        assert_eq!(fg(&file_entry("backup.tar.gz", 1, 0o644)), Some(t.archive));
        assert_eq!(fg(&file_entry("photo.jpg", 1, 0o644)), Some(t.image));
        assert_eq!(fg(&file_entry("main.rs", 1, 0o644)), Some(t.code));
        assert_eq!(fg(&file_entry("README", 1, 0o644)), None);
        // Tagged, then executable, come before the type.
        let mut tagged = file_entry("photo.jpg", 1, 0o644);
        tagged.tagged = true;
        assert_eq!(fg(&tagged), Some(t.tagged));
        #[cfg(unix)]
        assert_eq!(fg(&file_entry("build.sh", 1, 0o755)), Some(t.executable));
        // The classic look, by the option.
        let plain = Theme {
            type_colors: false,
            ..Theme::classic()
        };
        assert_eq!(
            file_style(&plain, &file_entry("photo.jpg", 1, 0o644)).fg,
            None
        );
    }

    #[test]
    fn big_sizes_and_recent_dates_stand_out() {
        let t = Theme::classic();
        assert_eq!(size_style(&t, 1000).fg, Some(t.dim));
        assert_eq!(size_style(&t, 5 << 20).fg, Some(t.text));
        let gib = size_style(&t, 3 << 30);
        assert!(gib.add_modifier.contains(Modifier::BOLD));
        let now = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000);
        let ago = |s: u64| Some(now - std::time::Duration::from_secs(s));
        assert_eq!(date_style(&t, ago(3600), now).fg, Some(t.text));
        assert_eq!(date_style(&t, ago(3 * 86_400), now).fg, Some(t.dim));
        assert_eq!(date_style(&t, None, now).fg, Some(t.dim));
    }

    #[test]
    fn the_way_to_the_cursor_is_lit_in_the_tree() {
        let tmp = crate::testing::tempdir();
        for dir in ["a/b", "m", "z/y"] {
            std::fs::create_dir_all(tmp.path().join(dir)).unwrap();
        }
        let mut pane = Pane::new(tmp.path().to_path_buf());
        pane.tree.expand_all(crate::model::tree::ROOT);
        pane.rows = pane.tree.visible_rows();
        let names: Vec<String> = pane
            .rows
            .iter()
            .map(|r| pane.tree.node(r.id).name.to_string_lossy().into_owned())
            .collect();
        assert_eq!(&names[1..], ["a", "b", "m", "z", "y"]);
        pane.tree_state.select(Some(5)); // y, inside z
        let guides = path_guides(&pane, 0..6);
        // From the root to z: the stroke of level 0 on the rows between,
        // the whole connector on z's row; then z's connector to y.
        assert_eq!(guides[1], [(0, false)]);
        assert_eq!(guides[2], [(0, false)]);
        assert_eq!(guides[3], [(0, false)]);
        assert_eq!(guides[4], [(0, true)]);
        assert_eq!(guides[5], [(1, true)]);
        // Only the rows asked for.
        assert_eq!(path_guides(&pane, 4..6), [vec![(0, true)], vec![(1, true)]]);

        let t = Theme::classic();
        let spans = prefix_spans(&t, &pane.rows[2].prefix, &guides[2]);
        let parts: Vec<(&str, Option<Color>)> = spans
            .iter()
            .map(|s| (s.content.as_ref(), s.style.fg))
            .collect();
        assert_eq!(
            parts,
            [("│", Some(t.active_border)), ("  └──", Some(t.dim))]
        );
    }

    #[test]
    fn ratio_bars_fit_their_width() {
        use crate::usage::Weight;
        let line = ratio_line(50, 100, 20, Weight::Light, "used").unwrap();
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text.chars().count(), 20, "{text}");
        assert!(text.ends_with(" 50% used"), "{text}");
        assert!(text.starts_with("█████"), "{text}");
        // Narrow: the word goes first, then the bar.
        let line = ratio_line(1, 4, 10, Weight::Light, "used").unwrap();
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(
            text.ends_with(" 25%") && text.chars().count() == 10,
            "{text}"
        );
        assert!(ratio_line(1, 4, 6, Weight::Light, "").is_none());
        assert!(ratio_line(1, 0, 20, Weight::Light, "").is_none());
    }

    #[test]
    fn a_scroll_thumb_only_when_the_list_does_not_fit() {
        use ratatui::{Terminal, backend::TestBackend};
        let mut term = Terminal::new(TestBackend::new(10, 12)).unwrap();
        let thumbs = |term: &mut Terminal<TestBackend>, len, offset| {
            term.draw(|f| {
                let area = f.area();
                f.render_widget(Block::bordered(), area);
                draw_scroll_thumb(f, area, len, 10, offset, true);
            })
            .unwrap();
            let buf = term.backend().buffer();
            (0..12)
                .filter(|&y| buf[(9, y)].symbol() == "┃")
                .collect::<Vec<u16>>()
        };
        assert!(thumbs(&mut term, 10, 0).is_empty(), "it all fits");
        assert_eq!(thumbs(&mut term, 100, 0), [1]);
        assert_eq!(thumbs(&mut term, 100, 90), [10], "the end of the list");
        // Half the list on screen: half the track.
        assert_eq!(thumbs(&mut term, 20, 10), [6, 7, 8, 9, 10]);
    }

    #[test]
    fn the_header_shortens_home_and_stresses_the_last_folder() {
        if let Some(home) = platform::home_dir().filter(|h| h.parent().is_some()) {
            let sep = std::path::MAIN_SEPARATOR;
            assert_eq!(home_short(&home), "~");
            assert_eq!(home_short(&home.join("x")), format!("~{sep}x"));
        }
        let parts = |text: &str| -> Vec<String> {
            path_spans(text, '/')
                .into_iter()
                .map(|s| s.content.into_owned())
                .collect()
        };
        assert_eq!(parts("~/projects/xylem"), ["~/projects/", "xylem"]);
        assert_eq!(parts("/"), ["/"]);
        assert_eq!(parts("~"), ["~"]);
    }

    #[cfg(unix)]
    #[test]
    fn the_header_names_the_remote_machine() {
        let tmp = crate::testing::tempdir();
        std::fs::create_dir(tmp.path().join("srv")).unwrap();
        let Some(remote) = crate::remote::for_tests(tmp.path()) else {
            return;
        };
        let mut app = App::new(tmp.path().to_path_buf());
        app.panes[0] = Pane::from_remote(remote);
        let row = draw_row0(&mut app);
        let real = tmp.path().canonicalize().unwrap();
        assert!(row.starts_with("Path:  test@testhost "), "{row}");
        assert!(row.contains(&real.display().to_string()), "{row}");
    }

    #[test]
    fn borders_follow_the_theme_and_the_option() {
        assert_eq!(border_type(&Theme::nord()), BorderType::Rounded);
        assert_eq!(border_type(&Theme::classic()), BorderType::Plain);
        let with = |theme: &str, borders: &str| {
            let text = format!(
                "theme = \"{theme}\"\n[options]\nborders = \"{borders}\"\ntype_colors = false"
            );
            let config = crate::config::Config::parse(&text).unwrap();
            Theme::from_config(&config).unwrap()
        };
        assert_eq!(border_type(&with("nord", "plain")), BorderType::Plain);
        assert_eq!(
            border_type(&with("classic", "rounded")),
            BorderType::Rounded
        );
        assert_eq!(border_type(&with("gruvbox", "theme")), BorderType::Rounded);
        assert!(!with("classic", "theme").type_colors);
    }
}
