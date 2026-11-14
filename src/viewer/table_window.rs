//! A table too big to keep whole (see `docs/plans/tables-windowed.md`): the grid
//! keeps the rows near the cursor, a window, and the others are read from the
//! file again as the cursor moves. Other threads read them, count the rows, and
//! run the filter over the whole table, so the screen never waits for the file.
//!
//! Every thread opens its own [`RowSource`], counts as a table reader while it
//! lives (see `RunningReader`), and stops when the window lets it go.

use std::collections::BTreeMap;
use std::ops::ControlFlow;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::time::{Duration, Instant};

use super::records::{Budget, Cell, End, MAX_COLS, RecordSink, RowSource};
use super::sql_filter::{self, Expr};
use super::table::{GridRowContext, TableRow};
use super::{RunningReader, StopOnDrop};

/// The most rows the window keeps, and the most text in their cells.
pub const WINDOW_ROWS: usize = 2_000;
pub const WINDOW_BYTES: usize = 32 << 20;
/// The most matches the filter keeps: their row numbers, 8 MiB.
pub const MAX_MATCHES: usize = 1_000_000;
/// How long reading the rows near the cursor may take.
const FETCH_TIME: Duration = Duration::from_secs(10);
/// How often the filter sends what it found so far.
const FLUSH: Duration = Duration::from_millis(50);

/// Opens the table again, in the thread that will read it.
pub type Opener = Arc<dyn Fn() -> Result<Box<dyn RowSource>, String> + Send + Sync>;

/// How a count or a filter's scan is going.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Progress {
    Running,
    Done,
    /// Esc, or the window let it go.
    Stopped,
    Failed,
}

/// The filter over the whole table: the rows it kept so far.
pub struct Scan {
    /// Their numbers in the file, from 0, in order.
    pub matches: Vec<usize>,
    /// The rows looked at so far.
    pub scanned: usize,
    pub state: Progress,
    /// It stopped at [`MAX_MATCHES`].
    pub capped: bool,
    id: u64,
    stop: Option<StopOnDrop>,
}

/// What the threads say.
enum Reply {
    /// Rows asked for, and, when the table ended before the last of them,
    /// how many rows it has.
    Rows {
        rows: Vec<(usize, TableRow)>,
        end: Option<usize>,
    },
    FetchFailed(String),
    Counted(Result<Option<usize>, String>),
    Found {
        id: u64,
        /// The rows kept, with what they hold while the window has room.
        matches: Vec<(usize, Option<TableRow>)>,
        scanned: usize,
        /// `Some` when the scan is over.
        end: Option<Result<End, String>>,
        capped: bool,
    },
}

/// The threads of a window. Dropping it stops them.
struct Threads {
    fetch: Sender<Vec<usize>>,
    replies: Sender<Reply>,
    rx: Receiver<Reply>,
    _fetch_stop: StopOnDrop,
    count_stop: Option<StopOnDrop>,
}

pub struct Window {
    /// The rows kept, by their number in the file (from 0).
    cache: BTreeMap<usize, TableRow>,
    bytes: usize,
    /// The rows known to be there: all of them once the count is done.
    pub rows: usize,
    pub counting: Progress,
    pub scan: Option<Scan>,
    /// Why the rows near the cursor could not be read.
    pub problem: Option<String>,
    headers: Vec<String>,
    opener: Opener,
    threads: Option<Threads>,
    /// The first and last rows last asked for, while that reading runs.
    asked: Option<(usize, usize)>,
    next_scan: u64,
}

impl std::fmt::Debug for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Window")
            .field("kept", &self.cache.len())
            .field("rows", &self.rows)
            .field("counting", &self.counting)
            .finish_non_exhaustive()
    }
}

/// What a row's cells take in memory, about.
fn row_bytes(row: &TableRow) -> usize {
    row.cells.iter().map(|c| c.len() + 24).sum::<usize>() + 24
}

/// A record as the grid keeps it: safe for a terminal, `width` cells.
fn grid_row(cells: &[Cell<'_>], width: usize) -> TableRow {
    let mut cells: Vec<String> = cells.iter().take(MAX_COLS).map(Cell::display).collect();
    cells.resize(width.max(cells.len()), String::new());
    TableRow { cells }
}

/// Runs `work` in another thread that counts as a table reader.
fn spawn(work: impl FnOnce() + Send + 'static) {
    let running = RunningReader::new();
    std::thread::spawn(move || {
        let _running = running;
        crate::panic_guard::catch(work);
    });
}

fn stop_flag() -> (Arc<AtomicBool>, StopOnDrop) {
    let flag = Arc::new(AtomicBool::new(false));
    (flag.clone(), StopOnDrop(flag))
}

impl Window {
    /// A window over a table whose first rows are `head`, with more after them.
    /// No thread runs until [`Window::start`].
    pub fn new(head: Vec<TableRow>, headers: Vec<String>, opener: Opener) -> Self {
        let rows = head.len();
        let bytes = head.iter().map(row_bytes).sum();
        let mut window = Window {
            cache: head.into_iter().enumerate().collect(),
            bytes,
            rows,
            counting: Progress::Running,
            scan: None,
            problem: None,
            headers,
            opener,
            threads: None,
            asked: None,
            next_scan: 0,
        };
        // Wide rows can pass the limit in bytes before the one in rows: the
        // first ones stay, the farthest from the top go.
        window.trim(0, (0, 0));
        window
    }

    /// Starts the reading of rows and, if it is not over, the count.
    pub fn start(&mut self) {
        if self.threads.is_some() {
            return;
        }
        let (replies, rx) = mpsc::channel();
        let (fetch, asks) = mpsc::channel::<Vec<usize>>();
        let (stop, fetch_stop) = stop_flag();
        let opener = self.opener.clone();
        let tx = replies.clone();
        spawn(move || fetch_rows(&opener, &asks, &tx, &stop));
        let mut threads = Threads {
            fetch,
            replies,
            rx,
            _fetch_stop: fetch_stop,
            count_stop: None,
        };
        if self.counting == Progress::Running {
            let (stop, count_stop) = stop_flag();
            let opener = self.opener.clone();
            let tx = threads.replies.clone();
            spawn(move || {
                let counted = opener()
                    .and_then(|mut source| source.count(Budget::cancellable(u64::MAX, stop)));
                let _ = tx.send(Reply::Counted(counted));
            });
            threads.count_stop = Some(count_stop);
        }
        self.threads = Some(threads);
        self.asked = None;
    }

    /// Lets every thread go, keeping the rows read. A count or a scan that was
    /// running stays unfinished; [`Window::start`] starts the count again.
    pub fn pause(&mut self) {
        self.threads = None;
        self.asked = None;
        if let Some(scan) = &mut self.scan
            && scan.state == Progress::Running
        {
            scan.state = Progress::Stopped;
            scan.stop = None;
        }
    }

    /// Esc: stops the count and the filter's scan. False if neither was running.
    pub fn stop_work(&mut self) -> bool {
        let mut stopped = false;
        if self.counting == Progress::Running {
            self.counting = Progress::Stopped;
            if let Some(threads) = &mut self.threads {
                threads.count_stop = None;
            }
            stopped = true;
        }
        if let Some(scan) = &mut self.scan
            && scan.state == Progress::Running
        {
            scan.state = Progress::Stopped;
            scan.stop = None;
            stopped = true;
        }
        stopped
    }

    /// Something is being read: the screen should look again soon.
    pub fn working(&self) -> bool {
        self.asked.is_some()
            || self.counting == Progress::Running
            || self
                .scan
                .as_ref()
                .is_some_and(|s| s.state == Progress::Running)
    }

    /// About how many rows of the size of those kept fit in the window.
    pub fn rows_that_fit(&self) -> usize {
        let each = (self.bytes / self.cache.len().max(1)).max(1);
        (WINDOW_BYTES / each).clamp(1, WINDOW_ROWS)
    }

    /// A row by its number in the file, if it is in the window.
    pub fn row(&self, row: usize) -> Option<&TableRow> {
        self.cache.get(&row)
    }

    /// The rows kept now (for the structure of a table that declares none).
    pub fn kept(&self) -> impl Iterator<Item = &TableRow> {
        self.cache.values()
    }

    /// The rows shown: all those known, or the filter's matches.
    pub fn visible(&self) -> usize {
        self.scan.as_ref().map_or(self.rows, |s| s.matches.len())
    }

    /// The number in the file of the `visible`-th row shown.
    pub fn file_row(&self, visible: usize) -> Option<usize> {
        match &self.scan {
            None => (visible < self.rows).then_some(visible),
            Some(scan) => scan.matches.get(visible).copied(),
        }
    }

    /// Where the first row shown at or after `row` (in the file) is shown.
    pub fn shown_from(&self, row: usize) -> usize {
        match &self.scan {
            None => row.min(self.rows),
            Some(scan) => scan.matches.partition_point(|&r| r < row),
        }
    }

    /// Runs `expr` over the whole table, or shows every row again (`None`).
    pub fn filter(&mut self, expr: Option<Expr>) {
        // The old scan stops as it is dropped.
        self.scan = None;
        let Some(expr) = expr else { return };
        self.next_scan += 1;
        let id = self.next_scan;
        let mut scan = Scan {
            matches: Vec::new(),
            scanned: 0,
            state: Progress::Running,
            capped: false,
            id,
            stop: None,
        };
        let Some(threads) = &self.threads else {
            scan.state = Progress::Stopped;
            self.scan = Some(scan);
            return;
        };
        let (stop, scan_stop) = stop_flag();
        scan.stop = Some(scan_stop);
        let opener = self.opener.clone();
        let tx = threads.replies.clone();
        let headers = self.headers.clone();
        spawn(move || {
            let mut sink = ScanSink {
                expr,
                headers,
                id,
                at: 0,
                found: Vec::new(),
                total: 0,
                with_rows: WINDOW_ROWS,
                flushed: Instant::now(),
                capped: false,
                tx: tx.clone(),
            };
            let end = opener().and_then(|mut source| {
                source.read_from(0, Budget::cancellable(u64::MAX, stop), &mut sink)
            });
            let _ = tx.send(Reply::Found {
                id,
                matches: std::mem::take(&mut sink.found),
                scanned: sink.at,
                end: Some(end),
                capped: sink.capped,
            });
        });
        self.scan = Some(scan);
    }

    /// Takes what the threads read. `focus` is the row in the file the cursor
    /// is on: the rows farthest from it leave the window first, but never
    /// those of `keep` (the screen). True if something changed.
    pub fn poll(&mut self, focus: usize, keep: (usize, usize)) -> bool {
        let mut changed = false;
        while let Some(threads) = &self.threads {
            let reply = match threads.rx.try_recv() {
                Ok(reply) => reply,
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            };
            changed = true;
            match reply {
                Reply::Rows { rows, end } => {
                    self.asked = None;
                    self.problem = None;
                    for (at, row) in rows {
                        self.keep(at, row);
                    }
                    if let Some(end) = end {
                        self.ended_at(end);
                    }
                }
                Reply::FetchFailed(e) => {
                    self.asked = None;
                    self.problem = Some(e);
                }
                Reply::Counted(Ok(Some(rows))) => {
                    if self.counting == Progress::Running {
                        self.rows = rows;
                        self.counting = Progress::Done;
                    }
                }
                Reply::Counted(Ok(None)) => {
                    if self.counting == Progress::Running {
                        self.counting = Progress::Stopped;
                    }
                }
                Reply::Counted(Err(e)) => {
                    if self.counting == Progress::Running {
                        self.counting = Progress::Failed;
                        self.problem = Some(format!("the rows could not be counted: {e}"));
                    }
                }
                Reply::Found {
                    id,
                    matches,
                    scanned,
                    end,
                    capped,
                } => self.found(id, matches, scanned, end, capped),
            }
        }
        if changed {
            self.trim(focus, keep);
        }
        changed
    }

    /// The table has `rows` rows: it says so in reading past its end.
    fn ended_at(&mut self, rows: usize) {
        self.rows = rows;
        self.counting = Progress::Done;
        if let Some(threads) = &mut self.threads {
            threads.count_stop = None;
        }
    }

    fn found(
        &mut self,
        id: u64,
        matches: Vec<(usize, Option<TableRow>)>,
        scanned: usize,
        end: Option<Result<End, String>>,
        capped: bool,
    ) {
        let Some(scan) = self.scan.as_mut().filter(|s| s.id == id) else {
            return;
        };
        let mut rows = Vec::new();
        for (at, row) in matches {
            scan.matches.push(at);
            if let Some(row) = row {
                rows.push((at, row));
            }
        }
        scan.scanned = scanned;
        scan.capped |= capped;
        let mut whole = None;
        match end {
            None => {}
            Some(Ok(End::Complete)) => {
                scan.state = Progress::Done;
                whole = Some(scanned);
            }
            Some(Ok(End::Refused)) => scan.state = Progress::Done,
            Some(Ok(End::OutOfTime)) => {
                if scan.state == Progress::Running {
                    scan.state = Progress::Stopped;
                }
            }
            Some(Err(e)) => {
                scan.state = Progress::Failed;
                self.problem = Some(format!("the filter stopped: {e}"));
            }
        }
        if scan.state != Progress::Running {
            scan.stop = None;
        }
        for (at, row) in rows {
            self.keep(at, row);
        }
        // Looking at every row counted them too.
        if let Some(rows) = whole
            && self.counting != Progress::Done
        {
            self.ended_at(rows);
        }
    }

    fn keep(&mut self, at: usize, row: TableRow) {
        self.bytes += row_bytes(&row);
        if let Some(old) = self.cache.insert(at, row) {
            self.bytes -= row_bytes(&old);
        }
    }

    /// Keeps the window within its limits, letting the farthest rows go.
    fn trim(&mut self, focus: usize, keep: (usize, usize)) {
        while self.cache.len() > WINDOW_ROWS || self.bytes > WINDOW_BYTES {
            let (Some((&first, _)), Some((&last, _))) =
                (self.cache.first_key_value(), self.cache.last_key_value())
            else {
                break;
            };
            let far = if focus.abs_diff(first) >= focus.abs_diff(last) {
                first
            } else {
                last
            };
            if (keep.0..=keep.1).contains(&far) {
                break;
            }
            if let Some(row) = self.cache.remove(&far) {
                self.bytes -= row_bytes(&row);
            }
        }
    }

    /// Asks for the rows of `wanted` (numbers in the file, in order) that the
    /// window does not have, unless they are being read already.
    pub fn want(&mut self, wanted: impl Iterator<Item = usize>) {
        let missing: Vec<usize> = wanted.filter(|r| !self.cache.contains_key(r)).collect();
        let (Some(&first), Some(&last)) = (missing.first(), missing.last()) else {
            return;
        };
        if let Some((a, b)) = self.asked
            && a <= first
            && last <= b
        {
            return;
        }
        let Some(threads) = &self.threads else { return };
        if threads.fetch.send(missing).is_ok() {
            self.asked = Some((first, last));
        }
    }
}

/// The thread that reads the rows near the cursor: the newest request each time.
fn fetch_rows(
    opener: &Opener,
    asks: &Receiver<Vec<usize>>,
    tx: &Sender<Reply>,
    stop: &Arc<AtomicBool>,
) {
    let mut source: Option<Box<dyn RowSource>> = None;
    while let Ok(mut want) = asks.recv() {
        while let Ok(newer) = asks.try_recv() {
            want = newer;
        }
        if stop.load(Ordering::Relaxed) {
            return;
        }
        let Some(&from) = want.first() else { continue };
        let source = match &mut source {
            Some(source) => source,
            None => match opener() {
                Ok(opened) => source.insert(opened),
                Err(e) => {
                    let _ = tx.send(Reply::FetchFailed(e));
                    continue;
                }
            },
        };
        let mut pick = Pick {
            want: &want,
            next: 0,
            at: from,
            rows: Vec::new(),
        };
        let budget = Budget::new(FETCH_TIME, u64::MAX).stopped_by(stop.clone());
        let reply = match source.read_from(from, budget, &mut pick) {
            Ok(End::Complete) => Reply::Rows {
                rows: pick.rows,
                end: Some(pick.at),
            },
            Ok(End::Refused) => Reply::Rows {
                rows: pick.rows,
                end: None,
            },
            Ok(End::OutOfTime) if stop.load(Ordering::Relaxed) => return,
            Ok(End::OutOfTime) => {
                let _ = tx.send(Reply::Rows {
                    rows: pick.rows,
                    end: None,
                });
                Reply::FetchFailed(format!(
                    "reading row {} took too long",
                    super::tabular_loader::thousands(pick.at + 1)
                ))
            }
            Err(e) => Reply::FetchFailed(e),
        };
        if tx.send(reply).is_err() {
            return;
        }
    }
}

/// Keeps the rows asked for, out of those read from the first of them on.
struct Pick<'a> {
    want: &'a [usize],
    next: usize,
    /// The number in the file of the next record.
    at: usize,
    rows: Vec<(usize, TableRow)>,
}

impl RecordSink for Pick<'_> {
    fn sheets(&mut self, _: &[String], _: usize) {}
    fn columns(&mut self, _: &[String]) {}
    fn columns_cut(&mut self) {}

    fn record(&mut self, cells: &[Cell<'_>]) -> ControlFlow<()> {
        if self.want.get(self.next) == Some(&self.at) {
            self.rows.push((self.at, grid_row(cells, 0)));
            self.next += 1;
        }
        self.at += 1;
        if self.next >= self.want.len() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}

/// Runs the filter over each record, sending what it keeps as it goes.
struct ScanSink {
    expr: Expr,
    headers: Vec<String>,
    id: u64,
    at: usize,
    found: Vec<(usize, Option<TableRow>)>,
    total: usize,
    /// How many more matches go with their rows, for the window.
    with_rows: usize,
    flushed: Instant,
    capped: bool,
    tx: Sender<Reply>,
}

impl RecordSink for ScanSink {
    fn sheets(&mut self, _: &[String], _: usize) {}
    fn columns(&mut self, _: &[String]) {}
    fn columns_cut(&mut self) {}

    fn record(&mut self, cells: &[Cell<'_>]) -> ControlFlow<()> {
        let row = grid_row(cells, self.headers.len());
        let ctx = GridRowContext {
            headers: &self.headers,
            row: &row,
        };
        if sql_filter::eval_expr(&self.expr, &ctx).is_ok_and(|v| v.is_truthy()) {
            if self.total >= MAX_MATCHES {
                self.capped = true;
                return ControlFlow::Break(());
            }
            let row = (self.with_rows > 0).then_some(row);
            self.with_rows = self.with_rows.saturating_sub(1);
            self.found.push((self.at, row));
            self.total += 1;
        }
        self.at += 1;
        if self.flushed.elapsed() >= FLUSH {
            self.flushed = Instant::now();
            let sent = self.tx.send(Reply::Found {
                id: self.id,
                matches: std::mem::take(&mut self.found),
                scanned: self.at,
                end: None,
                capped: false,
            });
            if sent.is_err() {
                return ControlFlow::Break(());
            }
        }
        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wide_row(n: usize) -> TableRow {
        TableRow {
            cells: vec![n.to_string(), "x".repeat(1 << 20)],
        }
    }

    fn no_source() -> Opener {
        Arc::new(|| Err("no file".to_string()))
    }

    #[test]
    fn the_window_keeps_to_its_bytes_and_never_drops_the_screen() {
        let head: Vec<TableRow> = (0..300).map(wide_row).collect();
        let mut w = Window::new(head, vec!["n".into(), "x".into()], no_source());
        assert!(w.bytes <= WINDOW_BYTES, "{} bytes", w.bytes);
        assert!(w.row(0).is_some() && w.row(30).is_some(), "the first rows");
        assert_eq!(w.rows_that_fit(), 31);
        assert!(w.row(299).is_none(), "the farthest went first");

        // Rows read far down push out those far from the cursor, not the screen.
        for n in 1_000..1_100 {
            w.keep(n, wide_row(n));
        }
        w.trim(1_050, (1_040, 1_060));
        assert!(w.bytes <= WINDOW_BYTES);
        assert!((1_040..=1_060).all(|n| w.row(n).is_some()));
        assert!(w.row(0).is_none());

        // A screen of rows bigger than the limit stays whole.
        for n in 1_000..1_100 {
            w.keep(n, wide_row(n));
        }
        w.trim(1_050, (1_000, 1_099));
        assert!((1_000..1_100).all(|n| w.row(n).is_some()));
    }

    #[test]
    fn the_window_keeps_to_its_rows() {
        let row = |n: usize| TableRow {
            cells: vec![n.to_string()],
        };
        let mut w = Window::new(
            (0..WINDOW_ROWS).map(row).collect(),
            vec!["n".into()],
            no_source(),
        );
        for n in 10_000..10_500 {
            w.keep(n, row(n));
        }
        w.trim(10_250, (10_240, 10_260));
        assert_eq!(w.cache.len(), WINDOW_ROWS);
        assert!(w.row(10_000).is_some() && w.row(10_499).is_some());
        assert!(w.row(0).is_none(), "the rows at the top were the farthest");
    }
}
