//! The views of the list (Alt+V): the selector and the Scan.

use super::*;
use crate::scan::Category;
use std::fs::{create_dir_all, write};

fn press(app: &mut App, code: KeyCode) {
    app.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
}

fn alt(app: &mut App, c: char) {
    app.handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT));
}

/// Until the Scan's thread is done.
fn settle(app: &mut App) {
    let start = Instant::now();
    while app.scan.as_ref().is_some_and(ScanView::running) {
        app.tick(Duration::from_millis(20));
        assert!(
            start.elapsed() < Duration::from_secs(20),
            "the Scan never ended"
        );
    }
}

fn count(app: &App, cat: Category) -> usize {
    app.scan.as_ref().unwrap().found(cat).unwrap().count()
}

/// A folder with something for each category, logged and shown as a Branch.
fn hostile() -> (tempfile::TempDir, App) {
    let tmp = crate::testing::tempdir();
    let p = tmp.path();
    create_dir_all(p.join("sub")).unwrap();
    write(p.join("fatura.pdf.exe"), "MZ").unwrap();
    write(p.join("fdp\u{202E}exe.pdf"), "x").unwrap();
    write(p.join("plain.txt"), "x").unwrap();
    write(p.join("sub/report.doc"), "x").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let mode = |path: &Path, m| {
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(m)).unwrap()
        };
        write(p.join("sub/open.sh"), "x").unwrap();
        mode(&p.join("sub/open.sh"), 0o666);
        write(p.join("sub/suid"), "x").unwrap();
        mode(&p.join("sub/suid"), 0o4755);
        create_dir_all(p.join("shared")).unwrap();
        mode(&p.join("shared"), 0o777);
        symlink("/etc/passwd", p.join("sub/passwd")).unwrap();
        symlink("gone", p.join("broken")).unwrap();
        symlink("plain.txt", p.join("fine")).unwrap();
    }
    let mut app = App::new(p.to_path_buf());
    app.apply(Command::ExpandAll);
    while app.job.is_some() {
        app.tick(Duration::from_millis(50));
    }
    app.apply(Command::Branch);
    (tmp, app)
}

#[test]
fn alt_v_opens_the_selector_and_s_the_scan() {
    let (_tmp, mut app) = hostile();
    alt(&mut app, 'v');
    let menu = app.context_menu.as_ref().expect("the selector");
    assert!(menu.letters);
    press(&mut app, KeyCode::Char('s'));
    assert!(app.context_menu.is_none());
    assert!(app.scan.is_some());
    settle(&mut app);
    assert_eq!(count(&app, Category::HiddenChars), 1);
    assert_eq!(count(&app, Category::Deceiving), 1);
    #[cfg(unix)]
    {
        // open.sh, suid and the folder shared; the links' own modes do not count.
        assert_eq!(count(&app, Category::Permissions), 3);
        assert_eq!(count(&app, Category::Links), 2);
    }
    assert_eq!(app.help_topic(), "views#scan");
    press(&mut app, KeyCode::Esc);
    assert!(app.scan.is_none());
}

#[test]
fn a_category_goes_back_to_the_list_as_a_filter() {
    let (tmp, mut app) = hostile();
    let all = app.files.len();
    app.apply(Command::ViewScan);
    settle(&mut app);
    // The second category: names that deceive.
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.scan.as_ref().unwrap().open, Some(1));
    press(&mut app, KeyCode::Enter);
    assert!(app.scan.is_none());
    assert_eq!(app.files.len(), 1);
    assert_eq!(app.focus, Focus::Files);
    let f = app.selected_file().unwrap();
    assert_eq!(app.file_path(f), tmp.path().join("fatura.pdf.exe"));
    assert!(app.pick.as_ref().unwrap().label.contains("deceiving"));
    // The Scan is kept: Alt+V, S on the same list opens it again.
    app.apply(Command::ViewScan);
    assert_eq!(app.scan.as_ref().unwrap().selected, 1);
    press(&mut app, KeyCode::Esc);
    // W with no text takes the choice away.
    app.set_query("", false).unwrap();
    assert!(app.pick.is_none());
    assert_eq!(app.files.len(), all);
}

#[cfg(unix)]
#[test]
fn a_folder_hit_goes_to_the_tree() {
    let (tmp, mut app) = hostile();
    app.apply(Command::ViewScan);
    settle(&mut app);
    let view = app.scan.as_mut().unwrap();
    view.selected = 2;
    let hits = &view.found[2].1.hits;
    view.hit = hits.iter().position(|h| h.dir).expect("the shared folder");
    view.open = Some(2);
    press(&mut app, KeyCode::Enter);
    assert!(app.scan.is_none());
    assert_eq!(app.focus, Focus::Tree);
    assert_eq!(app.current_path(), tmp.path().join("shared"));
    assert!(app.pick.is_none());
}

#[test]
fn v_on_a_hit_views_it_and_esc_comes_back() {
    let (_tmp, mut app) = hostile();
    app.apply(Command::ViewScan);
    settle(&mut app);
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Char('v'));
    assert!(app.viewer.is_some());
    press(&mut app, KeyCode::Esc);
    assert!(app.viewer.is_none());
    assert!(app.scan.as_ref().is_some_and(|s| s.open == Some(0)));
}

#[test]
fn an_empty_category_does_not_open() {
    let tmp = crate::testing::tempdir();
    write(tmp.path().join("a.txt"), "x").unwrap();
    let mut app = App::new(tmp.path().to_path_buf());
    app.apply(Command::ViewScan);
    settle(&mut app);
    press(&mut app, KeyCode::Enter);
    assert!(app.scan.as_ref().unwrap().open.is_none());
    assert!(app.message.as_deref().unwrap().contains("Nothing found"));
}

#[test]
fn the_scan_starts_from_the_list_without_a_previous_choice() {
    let (_tmp, mut app) = hostile();
    app.apply(Command::ViewScan);
    settle(&mut app);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.files.len(), 1);
    // R in a new Scan of the picked list still sees the whole list.
    app.apply(Command::ViewScan);
    press(&mut app, KeyCode::Char('r'));
    settle(&mut app);
    assert_eq!(count(&app, Category::HiddenChars), 1);
    assert_eq!(app.files.len(), 1, "the choice stays on the list");
}

/// A docx with macros and a remote template, and a patch that climbs out.
fn documents(dir: &Path) {
    use crate::viewer::office::test_docs::zip;
    let docx = zip(&[
        (
            "word/document.xml",
            &format!(
                "<w:document {}><w:body><w:p><w:r><w:t>hi</w:t></w:r></w:p></w:body></w:document>",
                crate::viewer::office::test_docs::W
            ),
        ),
        (
            "word/_rels/settings.xml.rels",
            r#"<Relationships><Relationship Id="r1" Type="http://x/attachedTemplate" Target="http://evil.example/t.dotm" TargetMode="External"/></Relationships>"#,
        ),
        ("word/vbaProject.bin", "x"),
    ]);
    write(dir.join("invoice.docx"), docx).unwrap();
    write(
        dir.join("clean.docx"),
        crate::viewer::office::test_docs::docx("<w:p><w:r><w:t>ok</w:t></w:r></w:p>"),
    )
    .unwrap();
    write(
        dir.join("fix.patch"),
        "--- a/../../etc/profile\n+++ b/../../etc/profile\n@@ -1 +1 @@\n-a\n+b\n",
    )
    .unwrap();
    write(
        dir.join("good.patch"),
        "--- a/src/x.rs\n+++ b/src/x.rs\n@@ -1 +1 @@\n-a\n+b\n",
    )
    .unwrap();
    // An Office name on what is not a package: the reader refuses it.
    write(dir.join("broken.docx"), "not a zip").unwrap();
    // The start of an ELF and nothing after: not a binary for the readers.
    write(dir.join("broken.bin"), b"\x7fELF\x02\x01\x01\0garbage").unwrap();
}

#[test]
fn the_scan_reads_documents_and_patches() {
    let tmp = crate::testing::tempdir();
    documents(tmp.path());
    let mut app = App::new(tmp.path().to_path_buf());
    app.apply(Command::ViewScan);
    settle(&mut app);
    let view = app.scan.as_ref().unwrap();
    let office = view.found(Category::Office).unwrap();
    assert_eq!(office.count(), 1, "{:?}", office.hits);
    assert!(
        office.hits[0].why.contains("macros"),
        "{}",
        office.hits[0].why
    );
    assert!(
        office.hits[0].why.contains("attachedTemplate"),
        "{}",
        office.hits[0].why
    );
    let patches = view.found(Category::Patches).unwrap();
    assert_eq!(patches.count(), 1, "{:?}", patches.hits);
    assert!(
        patches.hits[0].why.contains("touches"),
        "{}",
        patches.hits[0].why
    );
    let not = view.found(Category::NotAnalysed).unwrap();
    assert_eq!(not.count(), 1, "{:?}", not.hits);
    assert!(not.hits[0].path.ends_with("broken.docx"), "{:?}", not.hits);
}

#[test]
fn past_the_limit_a_file_is_not_analysed_and_says_why() {
    let tmp = crate::testing::tempdir();
    documents(tmp.path());
    let mut app = App::new(tmp.path().to_path_buf());
    app.options.scan_max_files = 1;
    app.apply(Command::ViewScan);
    settle(&mut app);
    let view = app.scan.as_ref().unwrap();
    let not = view.found(Category::NotAnalysed).unwrap();
    // Five files for the readers, one read (broken.docx, which fails).
    assert_eq!(not.count(), 5, "{:?}", not.hits);
    let past = not.hits.iter().filter(|h| h.why.contains("scan_max_files"));
    assert_eq!(past.count(), 4, "{:?}", not.hits);
}

#[test]
fn esc_stops_the_scan_and_a_stopped_one_is_not_kept() {
    let tmp = crate::testing::tempdir();
    for i in 0..50 {
        documents(&{
            let d = tmp.path().join(format!("d{i}"));
            create_dir_all(&d).unwrap();
            d
        });
    }
    let mut app = App::new(tmp.path().to_path_buf());
    app.apply(Command::ExpandAll);
    while app.job.is_some() {
        app.tick(Duration::from_millis(50));
    }
    app.apply(Command::Showall);
    app.apply(Command::ViewScan);
    assert!(app.scan.as_ref().unwrap().running());
    press(&mut app, KeyCode::Esc);
    let view = app.scan.as_ref().unwrap();
    assert!(!view.running());
    assert!(view.stopped.is_some());
    // Stopped, it is not kept: Esc leaves, and Alt+V, S starts over.
    press(&mut app, KeyCode::Esc);
    assert!(app.scan.is_none());
    app.apply(Command::ViewScan);
    settle(&mut app);
    let view = app.scan.as_ref().unwrap();
    assert!(view.stopped.is_none());
    assert_eq!(view.found(Category::Office).unwrap().count(), 50);
}

#[test]
fn inside_an_archive_the_members_are_read() {
    let tmp = crate::testing::tempdir();
    let inner = tmp.path().join("inner");
    create_dir_all(&inner).unwrap();
    documents(&inner);
    let zip_path = tmp.path().join("pack.zip");
    {
        use std::io::Write;
        let mut w = zip::ZipWriter::new(std::fs::File::create(&zip_path).unwrap());
        let opts = zip::write::SimpleFileOptions::default();
        for name in ["invoice.docx", "fix.patch", "good.patch"] {
            w.start_file(name, opts).unwrap();
            w.write_all(&std::fs::read(inner.join(name)).unwrap())
                .unwrap();
        }
        w.start_file("fatura.pdf.exe", opts).unwrap();
        w.write_all(b"MZ").unwrap();
        w.finish().unwrap();
    }
    std::fs::remove_dir_all(&inner).unwrap();
    let mut app = App::new(tmp.path().to_path_buf());
    press(&mut app, KeyCode::Enter);
    app.file_state.select(Some(0));
    app.handle_key(KeyEvent::new(KeyCode::F(5), KeyModifiers::ALT));
    for _ in 0..3000 {
        if app.opening.is_none() {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
        app.poll_opening();
    }
    assert!(app.archive.is_some(), "{:?}", app.message);
    app.apply(Command::ViewScan);
    settle(&mut app);
    let view = app.scan.as_ref().unwrap();
    assert_eq!(view.found(Category::Office).unwrap().count(), 1);
    assert_eq!(view.found(Category::Patches).unwrap().count(), 1);
    assert_eq!(view.found(Category::Deceiving).unwrap().count(), 1);
    assert!(view.found(Category::Links).unwrap().note.is_some());
}

/// The whole screen as text, drawn at `w` × `h`.
fn screen(app: &mut App, w: u16, h: u16) -> String {
    use ratatui::{Terminal, backend::TestBackend};
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| crate::ui::draw(f, app)).unwrap();
    let buf = term.backend().buffer().clone();
    (0..h)
        .map(|y| (0..w).map(|x| buf[(x, y)].symbol()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Files changed on 2026-11-01, two on 11-18 and one on 11-19, at noon.
fn dated() -> (tempfile::TempDir, App) {
    use chrono::TimeZone;
    let tmp = crate::testing::tempdir();
    for (name, day) in [("a.txt", 1), ("b.txt", 18), ("c.rs", 18), ("d.txt", 19)] {
        let path = tmp.path().join(name);
        write(&path, "x").unwrap();
        let t: std::time::SystemTime = chrono::Local
            .with_ymd_and_hms(2026, 11, day, 12, 0, 0)
            .single()
            .unwrap()
            .into();
        std::fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(t)
            .unwrap();
    }
    let app = App::new(tmp.path().to_path_buf());
    (tmp, app)
}

#[test]
fn the_timeline_takes_a_bar_back_to_the_list() {
    let (_tmp, mut app) = dated();
    alt(&mut app, 'v');
    press(&mut app, KeyCode::Char('t'));
    let view = app.timeline.as_ref().expect("the timeline");
    assert_eq!(view.line.scale, crate::timeline::Scale::Days);
    assert_eq!(view.line.bars.len(), 19);
    assert_eq!(view.chosen_label(), "2026-11-19");
    assert_eq!(app.help_topic(), "views#timeline");
    let text = screen(&mut app, 100, 30);
    assert!(text.contains("2026-11-19: 1 file"), "{text}");
    // One bar left: the 18th, two files.
    press(&mut app, KeyCode::Left);
    press(&mut app, KeyCode::Enter);
    assert!(app.timeline.is_none());
    assert_eq!(app.files.len(), 2);
    assert!(
        app.query
            .as_ref()
            .unwrap()
            .text()
            .contains("2026-11-18 00:00:00")
    );
}

#[test]
fn a_range_goes_with_and_to_the_condition_there_was() {
    let (_tmp, mut app) = dated();
    app.set_query("ext = 'txt'", false).unwrap();
    assert_eq!(app.files.len(), 3);
    app.apply(Command::ViewTimeline);
    // The 19th and the 18th: of those, the .txt ones.
    app.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT));
    assert_eq!(app.timeline.as_ref().unwrap().chosen_sum().0, 2);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.files.len(), 2);
    let q = app.query.as_ref().unwrap().text().to_string();
    assert!(q.starts_with("(ext = 'txt') AND (modified >="), "{q}");
}

#[test]
fn the_scale_changes_and_bytes_are_counted() {
    let (_tmp, mut app) = dated();
    app.apply(Command::ViewTimeline);
    press(&mut app, KeyCode::Char('-'));
    let view = app.timeline.as_ref().unwrap();
    assert_eq!(view.line.scale, crate::timeline::Scale::Weeks);
    assert_eq!(view.chosen_label(), "week of 2026-11-16");
    press(&mut app, KeyCode::Char('+'));
    press(&mut app, KeyCode::Char('+'));
    let view = app.timeline.as_ref().unwrap();
    assert_eq!(view.line.scale, crate::timeline::Scale::Hours);
    assert_eq!(view.chosen_label(), "2026-11-16 00h");
    press(&mut app, KeyCode::Char('b'));
    assert!(app.timeline.as_ref().unwrap().bytes);
    // Small and large screens draw.
    for (w, h) in [(40, 12), (200, 50)] {
        screen(&mut app, w, h);
    }
    press(&mut app, KeyCode::Esc);
    assert!(app.timeline.is_none());
    assert_eq!(app.files.len(), 4);
}

#[test]
fn the_scan_draws() {
    let (_tmp, mut app) = hostile();
    app.apply(Command::ViewScan);
    settle(&mut app);
    let text = screen(&mut app, 100, 30);
    assert!(text.contains("Names that deceive"), "{text}");
    assert!(text.contains("Not an antivirus"), "{text}");
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Enter);
    let text = screen(&mut app, 100, 30);
    assert!(text.contains("fatura.pdf.exe"), "{text}");
    assert!(text.contains("runs as .exe"), "{text}");
    screen(&mut app, 30, 10);
}

/// What the Table (Alt+V, B; docs/plans/views.md, stage 4) would take with a
/// big list, before it is written: the list of Showall over a made-up tree, a
/// compact snapshot of it read by the table window, sorting it by a column,
/// and, to compare, every cell made text up front.
/// `XY_TABLE_N=2000000 cargo test --release table_measure -- --ignored --nocapture`
#[cfg(target_os = "linux")]
#[test]
#[ignore]
fn table_measure() {
    use crate::model::tree::{FileEntry, FileKind, Tree};
    use crate::viewer::records::{Budget, Cell, End, Progress, RecordSink, RowSource};
    use crate::viewer::sql_filter::SqlFilter;
    use crate::viewer::table::TableRow;
    use crate::viewer::table_window::{Opener, WINDOW_ROWS, Window};
    use std::borrow::Cow;
    use std::fmt::Write as _;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// The memory the process holds, in MiB.
    fn rss() -> f64 {
        let status = std::fs::read_to_string("/proc/self/status").unwrap();
        let kb: f64 = status
            .lines()
            .find_map(|l| l.strip_prefix("VmRSS:"))
            .and_then(|v| v.trim().trim_end_matches(" kB").trim().parse().ok())
            .unwrap();
        kb / 1024.0
    }
    let ms = |t: Instant| t.elapsed().as_secs_f64() * 1000.0;
    let n: usize = std::env::var("XY_TABLE_N")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_000);

    // The tree: about 10 files a folder, folders 1 to 6 levels deep.
    const WORDS: &[&str] = &[
        "src", "docs", "config", "home", "projects", "build", "assets", "images", "Photos",
        "music", "viewer", "test", "xylem", "data", "backup", "2024", "work", "videos",
    ];
    const EXTS: &[&str] = &["txt", "rs", "md", "jpg", "json", "pdf", "mp3", "log"];
    let mut seed: u64 = 0x2545_F491_4F6C_DD1D;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let base = 1_500_000_000u64; // 2017
    let mut listing = Vec::with_capacity(n);
    let mut folder = String::new();
    for i in 0..n {
        if i % 10 == 0 {
            folder.clear();
            for _ in 0..1 + next() % 6 {
                folder.push_str(WORDS[(next() % WORDS.len() as u64) as usize]);
                folder.push('/');
            }
            write!(folder, "d{}", i / 10).unwrap();
        }
        let ext = EXTS[(next() % EXTS.len() as u64) as usize];
        let size = next() % 10_000_000;
        listing.push((
            std::path::PathBuf::from(format!("{folder}/file_{i}.{ext}")),
            Some(FileEntry {
                name: format!("file_{i}.{ext}").into(),
                size,
                disk: size.div_ceil(4096) * 4096,
                link: None,
                modified: Some(UNIX_EPOCH + Duration::from_secs(base + next() % 300_000_000)),
                mode: 0o100644,
                kind: FileKind::File,
                tagged: false,
            }),
        ));
    }
    let before = rss();
    let start = Instant::now();
    let tmp = crate::testing::tempdir();
    let mut pane = Pane::new(tmp.path().to_path_buf());
    pane.tree = Tree::from_listing(tmp.path().to_path_buf(), listing);
    let tree_ms = ms(start);
    let tree_mb = rss() - before;
    let start = Instant::now();
    let before = rss();
    pane.view = FileView::Showall;
    pane.refresh_files();
    println!(
        "{:>9} files in the list; tree built in {tree_ms:.0} ms, {tree_mb:.0} MiB; \
         the list (Showall) in {:.0} ms, {:.0} MiB",
        pane.files.len(),
        ms(start),
        rss() - before
    );
    // How much of the list is its sorting (by name, the default), from the
    // order of the tree, as refresh_files has them.
    let mut again: Vec<(crate::pane::FileRef, usize)> = pane
        .tree
        .visible_subtree(crate::model::tree::ROOT)
        .into_iter()
        .enumerate()
        .flat_map(|(rank, dir)| {
            (0..pane.tree.node(dir).files.len())
                .map(move |idx| (crate::pane::FileRef { dir, idx }, rank))
        })
        .collect();
    let sort = pane.sort;
    let start = Instant::now();
    crate::model::sort::sort_by(&mut again, &sort, |&(f, rank)| {
        sort.key_for(pane.file(f), rank)
    });
    println!("    of which sorting it again: {:.0} ms", ms(start));
    drop(again);
    let start = Instant::now();
    let kept = pane
        .files
        .iter()
        .filter(|&&f| pane.filespec.matches_entry(pane.file(f)))
        .count();
    println!(
        "    the filespec over every file again: {:.0} ms ({kept} kept)",
        ms(start)
    );
    let start = Instant::now();
    let hidden = pane
        .files
        .iter()
        .filter(|&&f| pane.file(f).is_hidden())
        .count();
    println!(
        "    is_hidden over every file again: {:.0} ms ({hidden})",
        ms(start)
    );

    // (a) The snapshot: one fixed row a file, the names in one buffer, the
    // folder by its number.
    struct Row {
        size: u64,
        disk: u64,
        /// Seconds since 1970; `i64::MIN` when not known.
        modified: i64,
        name: u32,
        name_len: u32,
        dir: u32,
        mode: u32,
        kind: u8,
        tagged: bool,
    }
    struct Snapshot {
        rows: Vec<Row>,
        names: String,
        dirs: Vec<String>,
        now: i64,
    }
    let before = rss();
    let start = Instant::now();
    let mut dir_of: Vec<u32> = Vec::new();
    let mut snap = Snapshot {
        rows: Vec::with_capacity(pane.files.len()),
        names: String::new(),
        dirs: Vec::new(),
        now: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs() as i64),
    };
    let root_len = pane.tree.path(crate::model::tree::ROOT).as_os_str().len() + 1;
    for f in &pane.files {
        if dir_of.len() <= f.dir {
            dir_of.resize(f.dir + 1, u32::MAX);
        }
        if dir_of[f.dir] == u32::MAX {
            dir_of[f.dir] = snap.dirs.len() as u32;
            let path = pane.tree.path(f.dir);
            let text = path.to_string_lossy();
            snap.dirs
                .push(text.get(root_len..).unwrap_or("").to_string());
        }
        let e = pane.file(*f);
        let at = snap.names.len() as u32;
        snap.names.push_str(&e.name.to_string_lossy());
        snap.rows.push(Row {
            size: e.size,
            disk: e.disk,
            modified: e
                .modified
                .and_then(|m| m.duration_since(UNIX_EPOCH).ok())
                .map_or(i64::MIN, |d| d.as_secs() as i64),
            name: at,
            name_len: snap.names.len() as u32 - at,
            dir: dir_of[f.dir],
            mode: e.mode,
            kind: e.kind as u8,
            tagged: e.tagged,
        });
    }
    let snap_ms = ms(start);
    let counted = (snap.rows.capacity() * std::mem::size_of::<Row>()
        + snap.names.capacity()
        + snap.dirs.iter().map(|d| d.capacity() + 24).sum::<usize>()) as f64
        / (1 << 20) as f64;
    println!(
        "(a) snapshot: {snap_ms:.0} ms, {:.0} MiB ({counted:.0} MiB counted: {} B a row, \
         {} folders)",
        rss() - before,
        std::mem::size_of::<Row>(),
        snap.dirs.len()
    );
    let snap = Arc::new(snap);

    // The snapshot as the table window reads it: cells made as asked for.
    const HEADERS: [&str; 10] = [
        "name", "ext", "dir", "size", "disk", "modified", "age_days", "mode", "kind", "tagged",
    ];
    struct Rows(Arc<Snapshot>);
    impl RowSource for Rows {
        fn read_from(
            &mut self,
            from: usize,
            budget: Budget,
            sink: &mut dyn RecordSink,
        ) -> Result<End, String> {
            let s = &self.0;
            let mut date = String::new();
            let mut mode = String::new();
            for (i, r) in s.rows.iter().enumerate().skip(from) {
                if i % 4096 == 0 && budget.cancelled() {
                    return Ok(End::OutOfTime);
                }
                let name = &s.names[r.name as usize..(r.name + r.name_len) as usize];
                let ext = name.rsplit_once('.').map_or("", |(_, e)| e);
                date.clear();
                if r.modified != i64::MIN {
                    let t = chrono::DateTime::from_timestamp(r.modified, 0).unwrap();
                    write!(date, "{}", t.format("%Y-%m-%d %H:%M:%S")).unwrap();
                }
                mode.clear();
                write!(mode, "{:o}", r.mode & 0o7777).unwrap();
                let cells = [
                    Cell::Text(Cow::Borrowed(name)),
                    Cell::Text(Cow::Borrowed(ext)),
                    Cell::Text(Cow::Borrowed(&s.dirs[r.dir as usize])),
                    Cell::Int(r.size as i64),
                    Cell::Int(r.disk as i64),
                    Cell::Text(Cow::Borrowed(&date)),
                    Cell::Int((s.now - r.modified) / 86_400),
                    Cell::Text(Cow::Borrowed(&mode)),
                    Cell::Text(Cow::Borrowed(if r.kind == 0 { "file" } else { "link" })),
                    Cell::Bool(r.tagged),
                ];
                if sink.record(&cells).is_break() {
                    return Ok(End::Refused);
                }
            }
            Ok(End::Complete)
        }

        fn count(&mut self, _: Budget, _: &mut Progress<'_>) -> Result<Option<usize>, String> {
            Ok(Some(self.0.rows.len()))
        }
    }
    struct Head(Vec<TableRow>);
    impl RecordSink for Head {
        fn sheets(&mut self, _: &[String], _: usize) {}
        fn columns(&mut self, _: &[String]) {}
        fn columns_cut(&mut self) {}
        fn record(&mut self, cells: &[Cell<'_>]) -> std::ops::ControlFlow<()> {
            self.0.push(TableRow {
                cells: cells.iter().map(Cell::display).collect(),
            });
            if self.0.len() < WINDOW_ROWS {
                std::ops::ControlFlow::Continue(())
            } else {
                std::ops::ControlFlow::Break(())
            }
        }
    }
    let before = rss();
    let start = Instant::now();
    let mut head = Head(Vec::new());
    Rows(snap.clone())
        .read_from(0, Budget::new(Duration::from_secs(60), 0), &mut head)
        .unwrap();
    let source = snap.clone();
    let opener: Opener = Arc::new(move || Ok(Box::new(Rows(source.clone())) as Box<dyn RowSource>));
    let mut w = Window::new(head.0, HEADERS.map(String::from).to_vec(), opener);
    w.start();
    let settle = |w: &mut Window| {
        let deadline = Instant::now() + Duration::from_secs(600);
        while w.working() {
            assert!(Instant::now() < deadline, "the window never settled");
            w.poll(0, (0, 0));
            std::thread::sleep(Duration::from_millis(2));
        }
    };
    settle(&mut w);
    println!(
        "    window open and counted: {:.0} ms, {} rows, {:.0} MiB",
        ms(start),
        w.rows,
        rss() - before
    );
    for filter in [
        "size > 5000000 AND ext = 'jpg'",
        "dir LIKE '%music%'",
        "modified >= '2024-01-01'",
        "age_days < 365 AND name LIKE 'file_1%'",
    ] {
        let before = rss();
        let start = Instant::now();
        w.filter(Some(SqlFilter::parse(filter).unwrap()));
        settle(&mut w);
        let scan = w.scan.as_ref().unwrap();
        println!(
            "    filter {filter:<40} {:>6.0} ms, {:>8} matches{}, {:.0} MiB",
            ms(start),
            scan.matches.len(),
            if scan.capped { " (capped)" } else { "" },
            rss() - before
        );
    }
    drop(w);

    // Sorting: an order of row numbers, the snapshot untouched.
    let s = &snap;
    let name = |r: &Row| &s.names[r.name as usize..(r.name + r.name_len) as usize];
    let before = rss();
    let mut order: Vec<u32> = (0..s.rows.len() as u32).collect();
    for (by, sort) in [
        (
            "name",
            &(|o: &mut Vec<u32>| {
                o.sort_by(|&a, &b| name(&s.rows[a as usize]).cmp(name(&s.rows[b as usize])))
            }) as &dyn Fn(&mut Vec<u32>),
        ),
        ("size", &|o: &mut Vec<u32>| {
            o.sort_by_key(|&a| s.rows[a as usize].size)
        }),
        ("modified", &|o: &mut Vec<u32>| {
            o.sort_by_key(|&a| s.rows[a as usize].modified)
        }),
        ("dir, name", &|o: &mut Vec<u32>| {
            o.sort_by(|&a, &b| {
                let (ra, rb) = (&s.rows[a as usize], &s.rows[b as usize]);
                s.dirs[ra.dir as usize]
                    .cmp(&s.dirs[rb.dir as usize])
                    .then_with(|| name(ra).cmp(name(rb)))
            })
        }),
    ] {
        let start = Instant::now();
        sort(&mut order);
        println!("    sort by {by:<10} {:>6.0} ms", ms(start));
    }
    println!("    the order: {:.0} MiB", rss() - before);
    drop(order);

    // (b) To compare: every cell made text up front.
    let before = rss();
    let start = Instant::now();
    let mut all: Vec<TableRow> = Vec::with_capacity(s.rows.len());
    struct All<'a>(&'a mut Vec<TableRow>);
    impl RecordSink for All<'_> {
        fn sheets(&mut self, _: &[String], _: usize) {}
        fn columns(&mut self, _: &[String]) {}
        fn columns_cut(&mut self) {}
        fn record(&mut self, cells: &[Cell<'_>]) -> std::ops::ControlFlow<()> {
            self.0.push(TableRow {
                cells: cells.iter().map(Cell::display).collect(),
            });
            std::ops::ControlFlow::Continue(())
        }
    }
    Rows(snap.clone())
        .read_from(
            0,
            Budget::new(Duration::from_secs(600), 0),
            &mut All(&mut all),
        )
        .unwrap();
    let counted = (all.capacity() * std::mem::size_of::<TableRow>()
        + all
            .iter()
            .map(|r| {
                r.cells.capacity() * std::mem::size_of::<String>()
                    + r.cells.iter().map(String::capacity).sum::<usize>()
            })
            .sum::<usize>()) as f64
        / (1 << 20) as f64;
    println!(
        "(b) every cell as text: {:.0} ms, {:.0} MiB ({counted:.0} MiB counted)",
        ms(start),
        rss() - before
    );
    assert_eq!(all.len(), pane.files.len());
}
