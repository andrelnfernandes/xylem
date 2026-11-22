//! The file list as a table (Alt+V, Table; docs/plans/views.md, stage 4).
//!
//! The list is copied once into a compact snapshot: each file keeps its
//! entry and the number of its folder, never a path of its own, and its cells
//! are made only when the window of the table asks for its row. Their values
//! come from [`crate::model::query::file_value`], so the table's filter and `W`
//! see the same thing.

use std::borrow::Cow;
use std::ops::ControlFlow;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use super::records::{Budget, Cell, End, Progress, RecordSink, RowSource};
use super::sql_filter::Value;
use super::table_window::Opener;
use crate::model::query::{Place, file_value};
use crate::model::tree::FileEntry;

/// The columns, in order: those of `W` but `type`, which reads each file,
/// and `path` and `depth`, which `dir` already tells.
const COLUMNS: &[&str] = &[
    "name", "ext", "dir", "size", "disk", "modified", "age_days", "mode", "kind", "tagged", "git",
    "dup",
];

/// One file of the list.
pub struct ListFile {
    pub entry: FileEntry,
    /// Its folder, in [`ListSnapshot::dirs`].
    pub dir: u32,
    pub git: Option<Box<str>>,
    pub dup: Option<u32>,
}

/// The list as it was when the table opened.
pub struct ListSnapshot {
    /// The folders of the files, with how deep each one is.
    pub dirs: Vec<(PathBuf, usize)>,
    pub files: Vec<ListFile>,
    /// For `age_days`: the same "now" for every row.
    pub now: SystemTime,
    /// The columns shown: `git` and `dup` only when some file has one.
    columns: Vec<&'static str>,
}

impl ListSnapshot {
    pub fn new(dirs: Vec<(PathBuf, usize)>, files: Vec<ListFile>, now: SystemTime) -> Self {
        let git = files.iter().any(|f| f.git.is_some());
        let dup = files.iter().any(|f| f.dup.is_some());
        let columns = COLUMNS
            .iter()
            .copied()
            .filter(|c| (*c != "git" || git) && (*c != "dup" || dup))
            .collect();
        ListSnapshot {
            dirs,
            files,
            now,
            columns,
        }
    }

    /// The cell of `column` for the file at `i`.
    fn cell(&self, i: usize, column: &str) -> Cell<'static> {
        let f = &self.files[i];
        let place = self
            .dirs
            .get(f.dir as usize)
            .map(|(dir, depth)| Place { dir, depth: *depth });
        let value = file_value(
            column,
            &f.entry,
            place,
            self.now,
            f.git.as_deref(),
            f.dup.map(i64::from),
        );
        match value {
            None | Some(Value::Null) => Cell::Null,
            Some(Value::Bool(b)) => Cell::Bool(b),
            Some(Value::Int(n)) => Cell::Int(n),
            // Days with one decimal: 0.3 is a few hours ago.
            Some(Value::Float(x)) => Cell::Float((x * 10.0).round() / 10.0),
            Some(Value::Text(s)) => Cell::Text(Cow::Owned(s)),
        }
    }

    /// The source a table window reads it through.
    pub fn opener(self: &Arc<Self>) -> Opener {
        let list = self.clone();
        Arc::new(move || Ok(Box::new(ListRows(list.clone())) as Box<dyn RowSource>))
    }
}

/// The rows of a [`ListSnapshot`].
struct ListRows(Arc<ListSnapshot>);

impl RowSource for ListRows {
    fn read_from(
        &mut self,
        from: usize,
        budget: Budget,
        sink: &mut dyn RecordSink,
    ) -> Result<End, String> {
        let list = &self.0;
        let names: Vec<String> = list.columns.iter().map(|c| c.to_string()).collect();
        sink.columns(&names);
        let mut cells = Vec::with_capacity(names.len());
        for i in from..list.files.len() {
            if (i - from) & 0xFF == 0 && budget.over() {
                return Ok(End::OutOfTime);
            }
            cells.clear();
            cells.extend(list.columns.iter().map(|c| list.cell(i, c)));
            if sink.record(&cells) == ControlFlow::Break(()) {
                return Ok(End::Refused);
            }
        }
        Ok(End::Complete)
    }

    fn count(
        &mut self,
        _budget: Budget,
        progress: &mut Progress<'_>,
    ) -> Result<Option<usize>, String> {
        let rows = self.0.files.len();
        progress(rows, None);
        Ok(Some(rows))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::tree::FileKind;
    use std::time::Duration;

    fn file(name: &str, size: u64, dir: u32) -> ListFile {
        ListFile {
            entry: FileEntry {
                name: name.into(),
                size,
                disk: size,
                link: None,
                modified: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000)),
                mode: 0o100644,
                kind: FileKind::File,
                tagged: name.starts_with('t'),
            },
            dir,
            git: None,
            dup: None,
        }
    }

    struct Rows(Vec<Vec<String>>, Vec<String>);
    impl RecordSink for Rows {
        fn sheets(&mut self, _: &[String], _: usize) {}
        fn columns(&mut self, names: &[String]) {
            self.1 = names.to_vec();
        }
        fn columns_cut(&mut self) {}
        fn record(&mut self, cells: &[Cell<'_>]) -> ControlFlow<()> {
            self.0.push(cells.iter().map(Cell::display).collect());
            ControlFlow::Continue(())
        }
    }

    fn snapshot() -> Arc<ListSnapshot> {
        let dirs = vec![("/a".into(), 0), ("/a/b".into(), 1)];
        let files = vec![file("one.TXT", 10, 0), file("two.rs", 2000, 1)];
        Arc::new(ListSnapshot::new(
            dirs,
            files,
            SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_086_400),
        ))
    }

    #[test]
    fn the_rows_have_the_values_of_w() {
        let list = snapshot();
        let mut rows = (list.opener())().unwrap();
        let mut sink = Rows(Vec::new(), Vec::new());
        let end = rows.read_from(1, Budget::unlimited(), &mut sink).unwrap();
        assert_eq!(end, End::Complete);
        // No file has a Git status or a group of duplicates: no such columns.
        assert_eq!(
            sink.1,
            [
                "name", "ext", "dir", "size", "disk", "modified", "age_days", "mode", "kind",
                "tagged"
            ]
        );
        let row = &sink.0[0];
        assert_eq!(row[0], "two.rs");
        assert_eq!(row[1], "rs");
        assert_eq!(row[2], "/a/b");
        assert_eq!(row[3], "2000");
        assert_eq!(row[6], "1");
        assert_eq!(row[7], "644");
        assert_eq!(row[8], "file");
        assert_eq!(row[9], "true");
    }

    #[test]
    fn git_and_dup_show_when_some_file_has_them() {
        let mut f = file("x", 1, 0);
        f.git = Some(" M".into());
        f.dup = Some(3);
        let list = ListSnapshot::new(vec![("/".into(), 0)], vec![f], SystemTime::now());
        assert!(list.columns.contains(&"git"));
        assert!(list.columns.contains(&"dup"));
    }

    #[test]
    fn the_count_is_the_list() {
        let list = snapshot();
        let mut rows = (list.opener())().unwrap();
        let count = rows.count(Budget::unlimited(), &mut |_, _| {}).unwrap();
        assert_eq!(count, Some(2));
    }
}

/// The measure asked for by the plan (docs/plans/views.md, stage 4):
/// `cargo test --release -- --ignored --nocapture list_of_two_million`.
#[cfg(all(test, target_os = "linux"))]
mod measure {
    use super::*;
    use crate::model::tree::FileKind;
    use std::time::{Duration, Instant};

    fn resident_mib() -> f64 {
        let statm = std::fs::read_to_string("/proc/self/statm").unwrap();
        let pages: f64 = statm.split_whitespace().nth(1).unwrap().parse().unwrap();
        pages * 4096.0 / (1 << 20) as f64
    }

    #[test]
    #[ignore]
    fn list_of_two_million() {
        const FILES: usize = 2_000_000;
        // The tree holds its entries; the table copies them, as Alt+V does.
        let tree: Vec<FileEntry> = (0..FILES)
            .map(|i| FileEntry {
                name: format!("document_{i:07}.txt").into(),
                size: i as u64,
                disk: 4096,
                link: None,
                modified: Some(
                    SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000 + i as u64),
                ),
                mode: 0o100644,
                kind: FileKind::File,
                tagged: false,
            })
            .collect();
        let dirs: Vec<(PathBuf, usize)> = (0..FILES / 50)
            .map(|d| (PathBuf::from(format!("/home/me/projects/p{d}")), 3))
            .collect();
        let before = resident_mib();
        let start = Instant::now();
        let files = tree
            .iter()
            .enumerate()
            .map(|(i, e)| ListFile {
                entry: e.clone(),
                dir: (i / 50) as u32,
                git: None,
                dup: None,
            })
            .collect();
        let list = Arc::new(ListSnapshot::new(dirs, files, SystemTime::now()));
        let copied = start.elapsed();
        let grid = super::super::tabular_loader::load_list(
            std::path::Path::new("Showall /"),
            list.opener(),
        )
        .unwrap();
        let opened = start.elapsed();
        let after = resident_mib();
        println!(
            "{FILES} files: copy {copied:?}, table on screen {opened:?}, {:.0} MiB more",
            after - before
        );
        assert!(grid.window.is_some());
    }
}
