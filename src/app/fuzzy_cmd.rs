//! The fuzzy find (`|`, see `docs/plans/fuzzy-find.md`): the prompt, its keys,
//! and its search, done a slice at a time in [`App::tick`].

use super::*;
use crate::fuzzy::{Pass, Query, Rank, Source};

/// A search whose tree or list changed starts over at most this often (a
/// cloud mount changes with each folder it reads).
#[cfg(not(test))]
const RESTART_EVERY: Duration = Duration::from_secs(1);
#[cfg(test)]
const RESTART_EVERY: Duration = Duration::ZERO;

/// Where the prompt was opened, for Esc to go back to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FuzzyOrigin {
    /// The folder under the tree cursor.
    Tree(NodeId),
    /// The file selected in the list.
    Files(Option<usize>),
}

/// What a search was started on: when it changes, the search starts over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Stamp {
    /// The tree's version and whether it shows hidden folders (the folders).
    tree: Option<(u64, bool)>,
    /// How many times the list was rebuilt (the files).
    files: Option<u64>,
}

impl Stamp {
    fn of(pane: &Pane, source: Source) -> Stamp {
        match source {
            Source::Folders => Stamp {
                tree: Some((pane.tree.version(), pane.tree.show_hidden)),
                files: None,
            },
            Source::Files { .. } => Stamp {
                tree: None,
                files: Some(pane.files_built),
            },
        }
    }
}

/// The state of the `|` prompt.
pub struct Fuzzy {
    pub(crate) origin: FuzzyOrigin,
    /// Always there; an `Option` only to hand it to the next pass.
    pass: Option<Pass>,
    stamp: Stamp,
    started: Instant,
    /// The best results so far, the best first.
    pub results: Vec<Rank>,
    /// The result chosen (its id): it stays chosen while the results change.
    chosen: Option<usize>,
    pub selected: usize,
    pub scroll: usize,
    /// Rows of the list on screen, for PgUp and PgDn.
    pub page: usize,
}

impl Fuzzy {
    fn pass(&self) -> &Pass {
        self.pass.as_ref().expect("a fuzzy prompt has a pass")
    }

    pub fn source(&self) -> Source {
        self.pass().source()
    }

    pub fn query(&self) -> &Query {
        self.pass().query()
    }

    pub fn is_done(&self) -> bool {
        self.pass().is_done()
    }

    /// Items looked at, and about how many there are.
    pub fn progress(&self) -> (usize, usize) {
        self.pass().progress()
    }

    /// How many matched, more than the results kept.
    pub fn found(&self) -> usize {
        self.pass().found()
    }

    /// The selected result's id.
    fn selected_id(&self) -> Option<usize> {
        self.results.get(self.selected).map(|r| r.id)
    }

    /// Picks result `i` (a click, ↑↓).
    pub(crate) fn choose(&mut self, i: usize) {
        if let Some(r) = self.results.get(i) {
            self.selected = i;
            self.chosen = Some(r.id);
        }
    }

    /// Takes the pass's best results so far, keeping the chosen one chosen.
    fn take_results(&mut self) {
        self.results = self.pass().best();
        self.selected = self
            .chosen
            .and_then(|id| self.results.iter().position(|r| r.id == id))
            .unwrap_or(0);
    }

    /// Keeps the selected row on screen.
    pub fn keep_visible(&mut self) {
        let page = self.page.max(1);
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + page {
            self.scroll = self.selected + 1 - page;
        }
        self.scroll = self.scroll.min(self.results.len().saturating_sub(page));
    }
}

impl App {
    /// `|`: searches the folders of the tree, or the files of the list,
    /// as the user types.
    pub(crate) fn start_fuzzy(&mut self) {
        if self.usage.is_some() || self.duplicates.is_some() {
            return;
        }
        let (source, origin) = match self.focus {
            Focus::Tree => (Source::Folders, FuzzyOrigin::Tree(self.current())),
            Focus::Files => {
                let base = match self.view {
                    FileView::Dir => None,
                    FileView::Branch(b) => Some(b),
                    FileView::Showall => Some(ROOT),
                };
                (
                    Source::Files { base },
                    FuzzyOrigin::Files(self.file_state.selected()),
                )
            }
        };
        let pane = &self.panes[self.active];
        let pass = Pass::new(Query::new(""), source, &pane.tree, &pane.files);
        let fuzzy = Fuzzy {
            origin,
            pass: Some(pass),
            stamp: Stamp::of(pane, source),
            started: Instant::now(),
            results: Vec::new(),
            chosen: None,
            selected: 0,
            scroll: 0,
            page: 1,
        };
        self.open_prompt(PromptKind::Fuzzy(Box::new(fuzzy)), "FUZZY: ".into(), "");
    }

    fn fuzzy(&self) -> Option<&Fuzzy> {
        match &self.prompt {
            Some(Prompt {
                kind: PromptKind::Fuzzy(f),
                ..
            }) => Some(f),
            _ => None,
        }
    }

    fn fuzzy_mut(&mut self) -> Option<&mut Fuzzy> {
        match &mut self.prompt {
            Some(Prompt {
                kind: PromptKind::Fuzzy(f),
                ..
            }) => Some(f),
            _ => None,
        }
    }

    pub(crate) fn is_fuzzy(&self) -> bool {
        self.fuzzy().is_some()
    }

    pub(crate) fn fuzzy_key(&mut self, key: KeyEvent) {
        let Some(Prompt {
            kind: PromptKind::Fuzzy(fz),
            input,
            ..
        }) = &mut self.prompt
        else {
            return;
        };
        let last = fz.results.len().saturating_sub(1);
        let page = fz.page.max(1);
        let to = match key.code {
            KeyCode::Up => Some(fz.selected.saturating_sub(1)),
            KeyCode::Down => Some((fz.selected + 1).min(last)),
            KeyCode::PageUp => Some(fz.selected.saturating_sub(page)),
            KeyCode::PageDown => Some((fz.selected + page).min(last)),
            KeyCode::Home => Some(0),
            KeyCode::End => Some(last),
            _ => None,
        };
        if let Some(i) = to {
            fz.choose(i);
            self.fuzzy_follow();
            return;
        }
        match key.code {
            KeyCode::Enter => {
                // With nothing found, there is nowhere to go: the list says so.
                if !fz.results.is_empty() {
                    self.fuzzy_follow();
                    self.prompt = None;
                }
                return;
            }
            KeyCode::Esc => {
                let origin = fz.origin;
                self.prompt = None;
                match origin {
                    FuzzyOrigin::Tree(id) => self.show_node(id),
                    FuzzyOrigin::Files(sel) => {
                        self.file_state.select(sel);
                        self.sync_autoview();
                    }
                }
                return;
            }
            _ => {}
        }
        let before = input.text.clone();
        input.handle_key(key);
        if input.text != before {
            let text = input.text.clone();
            self.fuzzy_restart(Some(&text));
        }
    }

    /// Starts the search over: for a new `text`, or for the same one on a
    /// tree or list that changed.
    fn fuzzy_restart(&mut self, text: Option<&str>) {
        let pane = &self.panes[self.active];
        let Some(Prompt {
            kind: PromptKind::Fuzzy(fz),
            ..
        }) = &mut self.prompt
        else {
            return;
        };
        let Some(old) = fz.pass.take() else {
            return;
        };
        let source = old.source();
        let stamp = Stamp::of(pane, source);
        let query = match text {
            Some(t) => Query::new(t),
            None => old.query().clone(),
        };
        let fresh = |q| Pass::new(q, source, &pane.tree, &pane.files);
        let pass = if text.is_some() && stamp == fz.stamp && old.narrows_to(&query) {
            Pass::into_narrowed(old, query)
        } else {
            fresh(query)
        };
        fz.pass = Some(pass);
        fz.stamp = stamp;
        fz.started = Instant::now();
        if text.is_some() {
            fz.chosen = None;
            fz.selected = 0;
            fz.scroll = 0;
        }
        fz.take_results();
    }

    /// The search has work to do: a slice of its pass, or a start over.
    pub(crate) fn fuzzy_running(&self) -> bool {
        self.fuzzy().is_some_and(|fz| {
            !fz.is_done() || Stamp::of(&self.panes[self.active], fz.source()) != fz.stamp
        })
    }

    /// One slice of the search, of at most `budget`. Returns how long to wait
    /// when all there is to do is a start over that must wait its turn.
    pub(crate) fn fuzzy_tick(&mut self, budget: Duration) -> Option<Duration> {
        let fz = self.fuzzy()?;
        let changed = Stamp::of(&self.panes[self.active], fz.source()) != fz.stamp;
        let waited = fz.started.elapsed();
        let was_done = fz.is_done();
        if changed && waited >= RESTART_EVERY {
            self.fuzzy_restart(None);
        } else if changed && was_done {
            return Some(RESTART_EVERY - waited);
        }
        let pane = &self.panes[self.active];
        let Some(Prompt {
            kind: PromptKind::Fuzzy(fz),
            ..
        }) = &mut self.prompt
        else {
            return None;
        };
        let was_done = fz.is_done();
        if let Some(pass) = &mut fz.pass {
            pass.step(&pane.tree, &pane.files, budget);
        }
        fz.take_results();
        // The window's cursor goes to the choice when the pass ends, not at
        // each slice, where the order still changes.
        if !was_done && fz.is_done() {
            if fz
                .chosen
                .is_none_or(|id| !fz.results.iter().any(|r| r.id == id))
            {
                fz.chosen = fz.results.first().map(|r| r.id);
                fz.selected = 0;
            }
            self.fuzzy_follow();
        }
        None
    }

    /// The window's cursor goes to the selected result, if not there yet.
    pub(crate) fn fuzzy_follow(&mut self) {
        let Some(fz) = self.fuzzy() else {
            return;
        };
        let Some(id) = fz.selected_id() else {
            return;
        };
        match fz.source() {
            Source::Folders => {
                if id < self.tree.node_count() && self.current() != id {
                    self.show_node(id);
                }
            }
            Source::Files { .. } => {
                if id < self.files.len() && self.file_state.selected() != Some(id) {
                    self.file_state.select(Some(id));
                    self.sync_autoview();
                }
            }
        }
    }

    /// A click on result `i` of the list; twice, Enter.
    pub(crate) fn click_fuzzy(&mut self, i: usize, double: bool) {
        if let Some(fz) = self.fuzzy_mut() {
            fz.choose(i);
        }
        self.fuzzy_follow();
        if double {
            self.press(KeyCode::Enter);
        }
    }
}
