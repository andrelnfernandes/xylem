//! Search Box: as-you-type file search by words, without SQL.
//!
//! Pressing `P` (or `Ctrl+P` for tagged files) opens a search box.
//! The typed text is converted into `words('...')` with prefix search on the
//! last word while typing. The index answers instantly for indexed files;
//! unindexed files are read on confirmation.

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use crate::input::LineInput;
use crate::model::query::{Candidate, ContentJob, FileQuery, Term, WordPat, words_of};
use crate::pane::Pane;

/// One result found by the search box.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: PathBuf,
    pub name: std::ffi::OsString,
    pub dir: PathBuf,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub line: Option<usize>,
    pub snippet: Option<String>,
    pub tagged: bool,
}

/// The state of the floating search box modal.
pub struct SearchBox {
    pub input: LineInput,
    pub tagged: bool,
    pub last_edit: Instant,
    pub last_searched: String,
    pub results: Vec<SearchResult>,
    pub selected: usize,
    pub scroll: usize,
    pub page: usize,
    pub terms: Vec<Term>,
    pub unindexed_count: usize,
    pub unindexed_bytes: u64,
    pub unindexed_job: Option<ContentJob>,
    pub reading_unindexed: bool,
    pub confirming_unindexed: bool,
}

/// Maximum bytes read from a single file to find a snippet.
const MAX_SNIPPET_BYTES: usize = 1024 * 1024;
/// Maximum lines read from a file to find a snippet.
const MAX_SNIPPET_LINES: usize = 1000;
/// Maximum bytes per line when reading for snippet (prevents DOS on huge lines).
const MAX_LINE_READ: usize = 4096;

/// Converts typed text into a condition `words('w1 w2 ... wn*')`.
/// If the text ends with space, the last word does not get a prefix `*`.
/// If the text has no words, returns `None`.
pub fn format_query(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    let ends_with_space = text.ends_with(' ');
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }
    let mut parts: Vec<String> = words.iter().map(|w| (*w).to_string()).collect();
    if !ends_with_space
        && let Some(last) = parts.last_mut()
        && !last.ends_with('*')
    {
        last.push('*');
    }
    let joined = parts.join(" ").replace('\'', "''");
    Some(format!("words('{joined}')"))
}

/// Sanitizes a line into a ~`max_len` character snippet:
/// - Replaces bidi controls and control characters with spaces.
/// - Collapses multiple spaces.
/// - Focuses around the first matching word if possible.
pub fn sanitize_snippet(line: &str, terms: &[Term], max_len: usize) -> String {
    let mut clean = String::with_capacity(line.len());
    let mut prev_space = false;
    for c in line.chars() {
        if crate::bidi::is_bidi(c) || c.is_control() {
            if !prev_space {
                clean.push(' ');
                prev_space = true;
            }
        } else {
            prev_space = c.is_whitespace();
            clean.push(c);
        }
    }
    let clean = clean.trim();
    if clean.is_empty() {
        return String::new();
    }

    let pats: Vec<&WordPat> = terms
        .iter()
        .filter_map(|t| t.words.as_ref())
        .flatten()
        .collect();

    let clean_lower = clean.to_lowercase();
    let mut first_match_char_idx = None;
    for pat in &pats {
        if let Some(byte_idx) = clean_lower.find(&pat.word) {
            let char_idx = clean[..byte_idx].chars().count();
            first_match_char_idx = Some(match first_match_char_idx {
                Some(cur) => usize::min(cur, char_idx),
                None => char_idx,
            });
        }
    }

    let total_chars = clean.chars().count();
    if total_chars <= max_len {
        return clean.to_string();
    }

    let start = match first_match_char_idx {
        Some(idx) if idx > 25 => (idx - 20).min(total_chars.saturating_sub(max_len)),
        _ => 0,
    };

    let snippet_chars: String = clean.chars().skip(start).take(max_len).collect();
    let mut result = String::new();
    if start > 0 {
        result.push('…');
    }
    result.push_str(snippet_chars.trim());
    if start + max_len < total_chars {
        result.push('…');
    }
    result
}

/// Opens `path` safely and reads up to 1 MB / 1000 lines to find the first line
/// where the words of `terms` appear.
pub fn find_snippet(path: &Path, terms: &[Term]) -> Option<(usize, String)> {
    let pats: Vec<&WordPat> = terms
        .iter()
        .filter_map(|t| t.words.as_ref())
        .flatten()
        .collect();
    if pats.is_empty() {
        return None;
    }

    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let mut bytes_read = 0;
    let mut line_buf = Vec::new();
    let mut first_partial: Option<(usize, String)> = None;

    for line_no in 1..=MAX_SNIPPET_LINES {
        line_buf.clear();
        let mut read_for_line = 0;
        let mut hit_newline = false;

        while read_for_line < MAX_LINE_READ {
            let chunk = reader.fill_buf().ok()?;
            if chunk.is_empty() {
                break;
            }
            if let Some(pos) = chunk.iter().position(|&b| b == b'\n') {
                let take = pos + 1;
                let actual = take.min(MAX_LINE_READ.saturating_sub(read_for_line));
                line_buf.extend_from_slice(&chunk[..actual]);
                reader.consume(take);
                read_for_line += actual;
                hit_newline = true;
                break;
            }
            let take = chunk.len().min(MAX_LINE_READ.saturating_sub(read_for_line));
            line_buf.extend_from_slice(&chunk[..take]);
            reader.consume(take);
            read_for_line += take;
        }

        if !hit_newline && read_for_line >= MAX_LINE_READ {
            // Discard remainder of oversized line up to newline
            loop {
                let chunk = reader.fill_buf().ok()?;
                if chunk.is_empty() {
                    break;
                }
                if let Some(pos) = chunk.iter().position(|&b| b == b'\n') {
                    reader.consume(pos + 1);
                    break;
                }
                let len = chunk.len();
                reader.consume(len);
            }
        }

        if line_buf.is_empty() {
            break;
        }

        // Check for binary null byte
        if line_buf.contains(&0) {
            return None;
        }

        bytes_read += line_buf.len();
        let line_str = String::from_utf8_lossy(&line_buf);
        let words_in_line = words_of(&line_str);

        let matches_all = pats
            .iter()
            .all(|pat| words_in_line.iter().any(|w| pat.matches(w)));
        if matches_all {
            return Some((line_no, sanitize_snippet(&line_str, terms, 80)));
        }

        if first_partial.is_none() {
            let matches_any = pats
                .iter()
                .any(|pat| words_in_line.iter().any(|w| pat.matches(w)));
            if matches_any {
                first_partial = Some((line_no, sanitize_snippet(&line_str, terms, 80)));
            }
        }

        if bytes_read >= MAX_SNIPPET_BYTES {
            break;
        }
    }

    first_partial
}

impl SearchBox {
    pub fn new(tagged: bool) -> Self {
        SearchBox {
            input: LineInput::new(""),
            tagged,
            last_edit: Instant::now(),
            last_searched: String::new(),
            results: Vec::new(),
            selected: 0,
            scroll: 0,
            page: 10,
            terms: Vec::new(),
            unindexed_count: 0,
            unindexed_bytes: 0,
            unindexed_job: None,
            reading_unindexed: false,
            confirming_unindexed: false,
        }
    }

    /// Re-evaluates search query against candidates and index.
    pub fn update_search(&mut self, pane: &Pane) {
        let text = self.input.text.clone();
        self.last_searched = text.clone();

        let Some(cond) = format_query(&text) else {
            self.results.clear();
            self.terms.clear();
            self.unindexed_job = None;
            self.unindexed_count = 0;
            self.unindexed_bytes = 0;
            self.reading_unindexed = false;
            self.confirming_unindexed = false;
            self.selected = 0;
            self.scroll = 0;
            return;
        };

        let Ok(query) = FileQuery::parse(&cond, self.tagged) else {
            self.results.clear();
            self.terms.clear();
            self.unindexed_job = None;
            self.unindexed_count = 0;
            self.unindexed_bytes = 0;
            self.reading_unindexed = false;
            self.confirming_unindexed = false;
            self.selected = 0;
            self.scroll = 0;
            return;
        };

        self.terms = query.terms().to_vec();
        let candidates = pane.query_candidates(self.tagged);
        let root = pane.tree.path(crate::model::tree::ROOT);
        let uses_index = self.terms.iter().any(|t| t.words.is_some())
            && pane.remote.is_none()
            && pane.archive.is_none();
        let lookup = if uses_index {
            crate::index::lookup(&crate::index::touching(&root), query.terms())
        } else {
            crate::index::Lookup::default()
        };

        let now = SystemTime::now();
        let opener = pane.content_opener();
        let job = ContentJob::new(query, candidates, opener, now, &|c| lookup.answer(c));

        self.unindexed_count = job.total();
        self.unindexed_bytes = job.bytes_total;

        let candidate_candidates = pane.query_candidates(self.tagged);
        let map: HashMap<PathBuf, Candidate> = candidate_candidates
            .into_iter()
            .map(|c| (c.path.clone(), c))
            .collect();

        let prev_snippets: HashMap<PathBuf, (Option<usize>, Option<String>)> = self
            .results
            .drain(..)
            .map(|r| (r.path, (r.line, r.snippet)))
            .collect();

        let mut new_results = Vec::new();
        for hit_path in job.hits() {
            if let Some(c) = map.get(hit_path) {
                let (line, snippet) = prev_snippets.get(hit_path).cloned().unwrap_or((None, None));
                new_results.push(SearchResult {
                    path: c.path.clone(),
                    name: c.file.name.clone(),
                    dir: c.dir.clone(),
                    size: c.file.size,
                    modified: c.file.modified,
                    line,
                    snippet,
                    tagged: c.file.tagged,
                });
            }
        }

        new_results.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.dir.cmp(&b.dir)));
        self.results = new_results;
        self.unindexed_job = Some(job);
        self.reading_unindexed = false;
        self.confirming_unindexed = false;
        if self.selected >= self.results.len() {
            self.selected = self.results.len().saturating_sub(1);
        }
        self.keep_visible();
        self.ensure_snippets();
    }

    /// Extracts snippets for currently visible rows.
    pub fn ensure_snippets(&mut self) {
        let start = self.scroll;
        let end = (self.scroll + self.page.max(10)).min(self.results.len());
        for i in start..end {
            if self.results[i].snippet.is_none() {
                if let Some((line, snip)) = find_snippet(&self.results[i].path, &self.terms) {
                    self.results[i].line = Some(line);
                    self.results[i].snippet = Some(snip);
                } else {
                    self.results[i].snippet = Some(String::new());
                }
            }
        }
    }

    /// Moves the selection cursor by `delta`.
    pub fn move_cursor(&mut self, delta: isize) {
        if self.results.is_empty() {
            self.selected = 0;
            self.scroll = 0;
            return;
        }
        let len = self.results.len();
        if delta < 0 {
            self.selected = self.selected.saturating_sub((-delta) as usize);
        } else {
            self.selected = (self.selected + delta as usize).min(len.saturating_sub(1));
        }
        self.keep_visible();
        self.ensure_snippets();
    }

    /// Ensures the selected row is visible within `scroll .. scroll + page`.
    pub fn keep_visible(&mut self) {
        let page = self.page.max(1);
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + page {
            self.scroll = self.selected + 1 - page;
        }
        self.scroll = self.scroll.min(self.results.len().saturating_sub(page));
    }

    /// Starts reading unindexed files.
    pub fn start_reading_unindexed(&mut self) {
        if self.unindexed_count > 0 && self.unindexed_job.is_some() {
            self.reading_unindexed = true;
            self.confirming_unindexed = false;
        }
    }

    /// Advances the unindexed content job by `budget`.
    pub fn step_unindexed(&mut self, budget: Duration, pane: &Pane) {
        if !self.reading_unindexed {
            return;
        }
        let (hits_after, hits_before, is_done, new_hits) = {
            let Some(job) = &mut self.unindexed_job else {
                self.reading_unindexed = false;
                return;
            };
            let hits_before = job.hits().len();
            job.step(budget);
            let hits_after = job.hits().len();
            self.unindexed_count = job.total().saturating_sub(job.done());
            self.unindexed_bytes = job.bytes_total.saturating_sub(job.bytes_done);
            let is_done = job.is_done();
            let new_hits = if hits_after > hits_before {
                job.hits().clone()
            } else {
                HashSet::new()
            };
            (hits_after, hits_before, is_done, new_hits)
        };

        if hits_after > hits_before {
            let candidates = pane.query_candidates(self.tagged);
            let map: HashMap<PathBuf, Candidate> = candidates
                .into_iter()
                .map(|c| (c.path.clone(), c))
                .collect();
            let existing: HashSet<PathBuf> = self.results.iter().map(|r| r.path.clone()).collect();
            for hit in &new_hits {
                if !existing.contains(hit)
                    && let Some(c) = map.get(hit)
                {
                    self.results.push(SearchResult {
                        path: c.path.clone(),
                        name: c.file.name.clone(),
                        dir: c.dir.clone(),
                        size: c.file.size,
                        modified: c.file.modified,
                        line: None,
                        snippet: None,
                        tagged: c.file.tagged,
                    });
                }
            }
            self.results
                .sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.dir.cmp(&b.dir)));
            self.ensure_snippets();
        }

        if is_done {
            self.reading_unindexed = false;
            self.unindexed_count = 0;
            self.unindexed_bytes = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_query_appends_prefix_star_to_last_word_unless_trailing_space() {
        assert_eq!(format_query(""), None);
        assert_eq!(format_query("   "), None);
        assert_eq!(format_query("fatura"), Some("words('fatura*')".to_string()));
        assert_eq!(format_query("fatura "), Some("words('fatura')".to_string()));
        assert_eq!(
            format_query("fatura mar"),
            Some("words('fatura mar*')".to_string())
        );
        assert_eq!(
            format_query("fatura mar*"),
            Some("words('fatura mar*')".to_string())
        );
        assert_eq!(
            format_query("fatura mar "),
            Some("words('fatura mar')".to_string())
        );
    }

    #[test]
    fn sanitize_snippet_strips_controls_and_bidi() {
        let terms = vec![Term {
            text: "fatura".to_string(),
            words: Some(vec![WordPat {
                word: "fatura".to_string(),
                prefix: false,
            }]),
        }];
        let raw = "Relatório\tcom \u{202E}fatura\u{202C} de teste\r\n";
        let clean = sanitize_snippet(raw, &terms, 80);
        assert!(!clean.contains('\t'));
        assert!(!clean.contains('\r'));
        assert!(!clean.contains('\n'));
        assert!(!crate::bidi::has_bidi(&clean));
        assert!(clean.contains("fatura"));
    }
}
