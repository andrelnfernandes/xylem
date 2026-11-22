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

    /// The source a table window reads it through, in the order of the
    /// list.
    pub fn opener(self: &Arc<Self>) -> Opener {
        self.opener_in(None)
    }

    /// The same, with the rows in `order` (from [`ListSnapshot::order_by`]).
    pub fn opener_in(self: &Arc<Self>, order: Option<Arc<Vec<u32>>>) -> Opener {
        let list = self.clone();
        Arc::new(move || {
            Ok(Box::new(ListRows {
                list: list.clone(),
                order: order.clone(),
            }) as Box<dyn RowSource>)
        })
    }

    /// The rows sorted by `column`, ascending or not: the numbers of the files,
    /// the snapshot unchanged. A stable sort: ties keep the order of the list.
    pub fn order_by(&self, column: &str, descending: bool) -> Vec<u32> {
        let mut order: Vec<u32> = (0..self.files.len() as u32).collect();
        let file = |i: &u32| &self.files[*i as usize];
        let lower = |f: &ListFile| f.entry.name.to_string_lossy().to_lowercase();
        match column {
            "name" => order.sort_by_cached_key(|i| lower(file(i))),
            "ext" => order.sort_by_cached_key(|i| {
                std::path::Path::new(&file(i).entry.name)
                    .extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default()
            }),
            "dir" => {
                // Each folder's place among them, by its path, then the name.
                let mut by_path: Vec<usize> = (0..self.dirs.len()).collect();
                by_path.sort_by(|&a, &b| self.dirs[a].0.cmp(&self.dirs[b].0));
                let mut rank = vec![0u32; self.dirs.len()];
                for (r, d) in by_path.into_iter().enumerate() {
                    rank[d] = r as u32;
                }
                order.sort_by_cached_key(|i| {
                    let f = file(i);
                    (rank.get(f.dir as usize).copied(), lower(f))
                });
            }
            "size" => order.sort_by_key(|i| file(i).entry.size),
            "disk" => order.sort_by_key(|i| file(i).entry.disk),
            "modified" => order.sort_by_key(|i| file(i).entry.modified),
            // Older first: the dates the other way round.
            "age_days" => order.sort_by_key(|i| std::cmp::Reverse(file(i).entry.modified)),
            "mode" => order.sort_by_key(|i| file(i).entry.mode & 0o7777),
            "kind" => order.sort_by_key(|i| file(i).entry.kind as u8),
            "tagged" => order.sort_by_key(|i| file(i).entry.tagged),
            "git" => order.sort_by(|a, b| file(a).git.cmp(&file(b).git)),
            "dup" => order.sort_by_key(|i| file(i).dup),
            _ => {}
        }
        if descending {
            order.reverse();
        }
        order
    }
}

/// The table of the list in the viewer: the snapshot, the source the table
/// reads, and the column it is sorted by, if any.
pub struct ListTable {
    pub snapshot: Arc<ListSnapshot>,
    pub opener: Opener,
    /// The column, and whether descending.
    pub sort: Option<(String, bool)>,
}

/// The rows of a [`ListSnapshot`], in the order of the list or in `order`.
struct ListRows {
    list: Arc<ListSnapshot>,
    order: Option<Arc<Vec<u32>>>,
}

impl RowSource for ListRows {
    fn read_from(
        &mut self,
        from: usize,
        budget: Budget,
        sink: &mut dyn RecordSink,
    ) -> Result<End, String> {
        let list = &self.list;
        let names: Vec<String> = list.columns.iter().map(|c| c.to_string()).collect();
        sink.columns(&names);
        let mut cells = Vec::with_capacity(names.len());
        for i in from..list.files.len() {
            if (i - from) & 0xFF == 0 && budget.over() {
                return Ok(End::OutOfTime);
            }
            cells.clear();
            let at = self.order.as_ref().map_or(i, |o| o[i] as usize);
            cells.extend(list.columns.iter().map(|c| list.cell(at, c)));
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
        let rows = self.list.files.len();
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
    fn the_rows_come_in_the_order_asked_and_the_list_stays() {
        let list = snapshot();
        let order = Arc::new(list.order_by("size", true));
        assert_eq!(*order, [1, 0]);
        let mut rows = (list.opener_in(Some(order)))().unwrap();
        let mut sink = Rows(Vec::new(), Vec::new());
        rows.read_from(0, Budget::unlimited(), &mut sink).unwrap();
        assert_eq!(sink.0[0][0], "two.rs");
        assert_eq!(sink.0[1][0], "one.TXT");
        // Without case, and stable: the list's order on ties.
        assert_eq!(list.order_by("name", false), [0, 1]);
        assert_eq!(list.order_by("dir", true), [1, 0]);
        assert_eq!(list.order_by("tagged", false), [0, 1]);
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
        for column in ["size", "modified", "name", "dir"] {
            let start = Instant::now();
            let order = list.order_by(column, false);
            println!("sort by {column}: {:?}", start.elapsed());
            assert_eq!(order.len(), FILES);
        }
    }
}
