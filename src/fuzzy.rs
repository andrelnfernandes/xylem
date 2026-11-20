//! Approximate matching of names as the user types (`|`, see
//! `docs/plans/fuzzy-find.md`): the letters of each term in order, not
//! necessarily together, and a score that puts the best matches first.
//!
//! The scoring follows the usual rules of fuzzy finders: a matched letter
//! counts, letters together count more, a letter at the start of a word (after
//! `/`, `_`, `-`, `.`, a space, or a change to upper case) counts more still,
//! and a gap costs a little. A term that matches in the name itself, not only
//! in the folders above it, gets a bonus.
//!
//! The text is what the screen shows (the name after Xylem's treatment of
//! control and direction characters), so that the letters highlighted are the
//! ones on screen. Nothing here reads a disk: only names already in memory.

use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap};
use std::path::MAIN_SEPARATOR;
use std::time::{Duration, Instant};

use crate::model::tree::{NodeId, ROOT, Tree};
use crate::pane::FileRef;

/// The longest query, in characters: what is typed past it is ignored.
pub const MAX_QUERY: usize = 256;

const MATCH: i32 = 16;
const GAP_START: i32 = -3;
const GAP_EXTENSION: i32 = -1;
/// At the very start of the text, or after a space.
const BOUNDARY_WHITE: i32 = 10;
/// After a `/`.
const BOUNDARY_DELIMITER: i32 = 9;
/// After `_`, `-`, `.` and other punctuation.
const BOUNDARY: i32 = 8;
/// A change from lower to upper case, or from a letter to a digit.
const CAMEL: i32 = BOUNDARY + GAP_EXTENSION;
/// Letters together get at least this.
const CONSECUTIVE: i32 = -(GAP_START + GAP_EXTENSION);
/// The bonus of the first letter of a term counts this many times.
const FIRST_LETTER: i32 = 2;
/// A term that matches inside the name, not in the folders above it.
const IN_NAME: i32 = 2 * MATCH;

/// A term of the query: its letters, folded unless it asks for exact case.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Term {
    chars: Vec<char>,
    exact: bool,
}

/// What was typed, split into terms that must all match, in any order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    text: String,
    terms: Vec<Term>,
}

/// A text matched against a query: its score and the characters to highlight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    pub score: i32,
    /// Indexes of the characters (not bytes) that matched, in order, once each.
    pub positions: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    White,
    Delimiter,
    Punct,
    Lower,
    Upper,
    Digit,
    /// A letter with no case, or any other character of a word.
    Letter,
}

fn class(c: char) -> Class {
    match c {
        'a'..='z' => Class::Lower,
        'A'..='Z' => Class::Upper,
        '0'..='9' => Class::Digit,
        '/' | '\\' => Class::Delimiter,
        c if c.is_whitespace() => Class::White,
        c if c.is_ascii() => Class::Punct,
        c if c.is_lowercase() => Class::Lower,
        c if c.is_uppercase() => Class::Upper,
        c if c.is_numeric() => Class::Digit,
        c if c.is_alphabetic() => Class::Letter,
        _ => Class::Punct,
    }
}

/// The bonus of a character of class `now` after one of class `before`.
fn bonus(before: Class, now: Class) -> i32 {
    match (before, now) {
        (_, Class::White | Class::Delimiter | Class::Punct) => BOUNDARY,
        (Class::White, _) => BOUNDARY_WHITE,
        (Class::Delimiter, _) => BOUNDARY_DELIMITER,
        (Class::Punct, _) => BOUNDARY,
        (Class::Lower, Class::Upper) => CAMEL,
        (Class::Lower | Class::Upper | Class::Letter, Class::Digit) => CAMEL,
        _ => 0,
    }
}

/// One character in lower case: only the first, for the few that become two.
fn fold(c: char) -> char {
    if c.is_ascii() {
        c.to_ascii_lowercase()
    } else {
        c.to_lowercase().next().unwrap_or(c)
    }
}

impl Term {
    fn eq(&self, hay: char, i: usize) -> bool {
        let want = self.chars[i];
        if self.exact {
            hay == want
        } else {
            fold(hay) == want
        }
    }

    /// The letters appear in order in `text`: a quick look before scoring.
    fn appears_in(&self, text: &str) -> bool {
        let mut i = 0;
        for c in text.chars() {
            if self.eq(c, i) {
                i += 1;
                if i == self.chars.len() {
                    return true;
                }
            }
        }
        false
    }

    /// The best place of the term in `hay[from..]`, scored, with where its
    /// letters are. The first match that ends earliest, tightened from its
    /// end: the shortest window, whose letters are taken from the left.
    fn place(&self, hay: &[char], from: usize) -> Option<(i32, Vec<usize>)> {
        let m = self.chars.len();
        let mut i = 0;
        let mut end = None;
        for (k, &c) in hay.iter().enumerate().skip(from) {
            if self.eq(c, i) {
                i += 1;
                if i == m {
                    end = Some(k + 1);
                    break;
                }
            }
        }
        let end = end?;
        let mut i = m;
        let mut start = from;
        for k in (from..end).rev() {
            if self.eq(hay[k], i - 1) {
                i -= 1;
                if i == 0 {
                    start = k;
                    break;
                }
            }
        }
        let mut score = 0;
        let mut positions = Vec::with_capacity(m);
        let mut before = if start == 0 {
            Class::White
        } else {
            class(hay[start - 1])
        };
        let (mut in_gap, mut together, mut first_bonus) = (false, 0, 0);
        let mut i = 0;
        for (k, &c) in hay.iter().enumerate().take(end).skip(start) {
            let now = class(c);
            if i < m && self.eq(c, i) {
                score += MATCH;
                let mut b = bonus(before, now);
                if together == 0 {
                    first_bonus = b;
                } else {
                    if b >= BOUNDARY && b > first_bonus {
                        first_bonus = b;
                    }
                    b = b.max(first_bonus).max(CONSECUTIVE);
                }
                score += if i == 0 { b * FIRST_LETTER } else { b };
                positions.push(k);
                in_gap = false;
                together += 1;
                i += 1;
            } else {
                score += if in_gap { GAP_EXTENSION } else { GAP_START };
                in_gap = true;
                together = 0;
                first_bonus = 0;
            }
            before = now;
        }
        Some((score, positions))
    }
}

impl Query {
    /// The query of what was typed: up to [`MAX_QUERY`] characters, terms
    /// split at whitespace. A term with an upper-case letter matches case.
    pub fn new(text: &str) -> Query {
        let text: String = text.chars().take(MAX_QUERY).collect();
        let terms = text
            .split_whitespace()
            .map(|t| {
                let exact = t.chars().any(char::is_uppercase);
                let chars = t.chars().map(|c| if exact { c } else { fold(c) });
                Term {
                    chars: chars.collect(),
                    exact,
                }
            })
            .collect();
        Query { text, terms }
    }

    /// Nothing to look for: every text matches, with no score.
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    /// This query only keeps texts that `before` kept: what was typed grew
    /// at its end, so the search can look only among the earlier matches.
    pub fn narrows(&self, before: &Query) -> bool {
        self.text.starts_with(&before.text)
    }

    /// Matches `text`, whose name starts at byte `name_start` (what comes
    /// before is the folders above it). `scratch` is reused between calls.
    pub fn matches(&self, text: &str, name_start: usize, scratch: &mut Vec<char>) -> Option<Match> {
        if !self.terms.iter().all(|t| t.appears_in(text)) {
            return None;
        }
        scratch.clear();
        scratch.extend(text.chars());
        let name = text
            .get(..name_start)
            .map_or(0, |folders| folders.chars().count());
        let mut score = 0;
        let mut positions = Vec::new();
        for t in &self.terms {
            let (s, p) = match t.place(scratch, name) {
                Some((s, p)) => (s + IN_NAME, p),
                None => t.place(scratch, 0)?,
            };
            score += s;
            positions.extend(p);
        }
        positions.sort_unstable();
        positions.dedup();
        Some(Match { score, positions })
    }
}

/// Where a match ranks: a higher score first, then the shorter text, then the
/// first in the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rank {
    pub score: i32,
    pub len: usize,
    pub order: usize,
    /// What matched (a folder's id, or an index into the file list): not part
    /// of the ranking, where `order` already tells any two apart.
    pub id: usize,
}

impl Ord for Rank {
    /// Greater is better.
    fn cmp(&self, other: &Self) -> Ordering {
        self.score
            .cmp(&other.score)
            .then(other.len.cmp(&self.len))
            .then(other.order.cmp(&self.order))
    }
}

impl PartialOrd for Rank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The best `limit` ranks seen, without keeping the others.
pub struct Top {
    limit: usize,
    heap: BinaryHeap<Reverse<Rank>>,
}

impl Top {
    pub fn new(limit: usize) -> Top {
        Top {
            limit,
            heap: BinaryHeap::with_capacity(limit.min(4096) + 1),
        }
    }

    pub fn push(&mut self, rank: Rank) {
        if self.heap.len() < self.limit {
            self.heap.push(Reverse(rank));
        } else if self.heap.peek().is_some_and(|w| rank > w.0) {
            self.heap.pop();
            self.heap.push(Reverse(rank));
        }
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    #[cfg(test)]
    /// The ranks kept, the best first.
    pub fn into_sorted(self) -> Vec<Rank> {
        let mut v: Vec<Rank> = self.heap.into_iter().map(|r| r.0).collect();
        v.sort_unstable_by(|a, b| b.cmp(a));
        v
    }

    /// The ranks kept so far, the best first, leaving them here.
    pub fn best(&self) -> Vec<Rank> {
        let mut v: Vec<Rank> = self.heap.iter().map(|r| r.0).collect();
        v.sort_unstable_by(|a, b| b.cmp(a));
        v
    }
}

/// The results a pass keeps, at most.
pub const KEEP: usize = 500;

/// How many items a pass looks at between two looks at the clock.
const CLOCK_EVERY: usize = 64;

/// What a pass looks through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// The folders of the tree below the root, by their path from it.
    Folders,
    /// The files of the window's list: by name alone (`None`), or by the
    /// path from the folder `base` (a Branch, or the root for the Showall).
    Files { base: Option<NodeId> },
}

/// Where a pass is.
enum Cursor {
    /// Walking the tree: each folder with the length of the path before its
    /// name, in `buf`, which is cut back to it.
    Walk {
        stack: Vec<(NodeId, usize)>,
        buf: String,
    },
    /// Items `next..` of the file list.
    Range { next: usize },
    /// Only these items (what an earlier pass matched), in their order.
    Kept { ids: Vec<usize>, next: usize },
}

/// One search over a source, done a slice at a time ([`Pass::step`]), so that
/// the screen and the keys go on between slices. It keeps the best
/// [`KEEP`] and the ids of everything that matched, for a narrower query to
/// look only among them ([`Pass::into_narrowed`]).
pub struct Pass {
    query: Query,
    source: Source,
    cursor: Cursor,
    top: Top,
    matched: Vec<usize>,
    seen: usize,
    total: usize,
    done: bool,
    /// Hidden folders are walked (the tree shows them).
    hidden: bool,
    /// The path of each folder whose files were seen, ending in a separator.
    dirs: HashMap<NodeId, String>,
    text: String,
    scratch: Vec<char>,
}

impl Pass {
    /// A pass over the whole source. An empty query finds nothing.
    pub fn new(query: Query, source: Source, tree: &Tree, files: &[FileRef]) -> Pass {
        let hidden = tree.show_hidden;
        let (cursor, total) = match source {
            Source::Folders => {
                let stack = tree
                    .node(ROOT)
                    .children
                    .iter()
                    .rev()
                    .filter(|&&c| hidden || !tree.node(c).hidden)
                    .map(|&c| (c, 0))
                    .collect();
                let buf = String::new();
                (
                    Cursor::Walk { stack, buf },
                    tree.node_count().saturating_sub(1),
                )
            }
            Source::Files { .. } => (Cursor::Range { next: 0 }, files.len()),
        };
        Pass::with(query, source, cursor, total, hidden)
    }

    /// `query` keeps only what this pass kept, and this pass is over: the
    /// next one can look only among its matches ([`Pass::into_narrowed`]).
    pub fn narrows_to(&self, query: &Query) -> bool {
        self.done && !self.query.is_empty() && query.narrows(&self.query)
    }

    /// A pass over what `before` matched, for a query it narrows to (see
    /// [`Pass::narrows_to`]).
    pub fn into_narrowed(before: Pass, query: Query) -> Pass {
        debug_assert!(before.narrows_to(&query));
        let total = before.matched.len();
        let cursor = Cursor::Kept {
            ids: before.matched,
            next: 0,
        };
        Pass::with(query, before.source, cursor, total, before.hidden)
    }

    fn with(query: Query, source: Source, cursor: Cursor, total: usize, hidden: bool) -> Pass {
        let done = query.is_empty();
        Pass {
            query,
            source,
            cursor,
            top: Top::new(KEEP),
            matched: Vec::new(),
            seen: 0,
            total,
            done,
            hidden,
            dirs: HashMap::new(),
            text: String::new(),
            scratch: Vec::new(),
        }
    }

    pub fn query(&self) -> &Query {
        &self.query
    }

    pub fn source(&self) -> Source {
        self.source
    }

    pub fn is_done(&self) -> bool {
        self.done
    }

    /// How many items were looked at, and about how many there are (the
    /// tree's count includes folders released from it, so it is a ceiling).
    pub fn progress(&self) -> (usize, usize) {
        (self.seen, self.total.max(self.seen))
    }

    /// How many items matched so far: more than [`Pass::best`] keeps.
    pub fn found(&self) -> usize {
        self.matched.len()
    }

    /// The ids of everything that matched so far, in the order of the source.
    pub fn matched(&self) -> &[usize] {
        &self.matched
    }

    /// The best matches so far, the best first.
    pub fn best(&self) -> Vec<Rank> {
        self.top.best()
    }

    /// Goes on for about `budget`, or to the end.
    pub fn step(&mut self, tree: &Tree, files: &[FileRef], budget: Duration) {
        let start = Instant::now();
        let mut count = 0;
        while !self.done {
            count += 1;
            if count % CLOCK_EVERY == 0 && start.elapsed() >= budget {
                return;
            }
            let (id, order, name) = match &mut self.cursor {
                Cursor::Walk { stack, buf } => {
                    let Some((id, at)) = stack.pop() else {
                        self.done = true;
                        break;
                    };
                    buf.truncate(at);
                    if at > 0 {
                        buf.push(MAIN_SEPARATOR);
                    }
                    let name = buf.len();
                    buf.push_str(&tree.node(id).name.to_string_lossy());
                    let here = buf.len();
                    let hidden = self.hidden;
                    stack.extend(
                        tree.node(id)
                            .children
                            .iter()
                            .rev()
                            .filter(|&&c| hidden || !tree.node(c).hidden)
                            .map(|&c| (c, here)),
                    );
                    (id, self.seen, name)
                }
                Cursor::Range { next } => {
                    let i = *next;
                    let Some(&f) = files.get(i) else {
                        self.done = true;
                        break;
                    };
                    *next += 1;
                    let Some(name) =
                        file_text(tree, f, self.source, &mut self.dirs, &mut self.text)
                    else {
                        self.seen += 1;
                        continue;
                    };
                    (i, i, name)
                }
                Cursor::Kept { ids, next } => {
                    let order = *next;
                    let Some(&id) = ids.get(order) else {
                        self.done = true;
                        break;
                    };
                    *next += 1;
                    let name = match self.source {
                        Source::Folders => Some(folder_text(tree, id, &mut self.text)),
                        Source::Files { .. } => match files.get(id) {
                            Some(&f) => {
                                file_text(tree, f, self.source, &mut self.dirs, &mut self.text)
                            }
                            None => None,
                        },
                    };
                    let Some(name) = name else {
                        self.seen += 1;
                        continue;
                    };
                    (id, order, name)
                }
            };
            self.seen += 1;
            let text = match &self.cursor {
                Cursor::Walk { buf, .. } => buf.as_str(),
                _ => self.text.as_str(),
            };
            if let Some(m) = self.query.matches(text, name, &mut self.scratch) {
                self.matched.push(id);
                self.top.push(Rank {
                    score: m.score,
                    len: self.scratch.len(),
                    order,
                    id,
                });
            }
        }
    }
}

/// The text a folder is searched and shown by: its path from the root, the
/// root left out. Returns where its name starts.
pub fn folder_text(tree: &Tree, id: NodeId, buf: &mut String) -> usize {
    let mut names = Vec::new();
    let mut cur = Some(id);
    while let Some(n) = cur {
        let node = tree.node(n);
        if node.parent.is_none() {
            break;
        }
        names.push(&node.name);
        cur = node.parent;
    }
    buf.clear();
    let mut name = 0;
    for (i, n) in names.iter().rev().enumerate() {
        if i > 0 {
            buf.push(MAIN_SEPARATOR);
        }
        name = buf.len();
        buf.push_str(&n.to_string_lossy());
    }
    name
}

/// The path of folder `dir` from `base`, ending in a separator (empty for
/// `base` itself).
fn dir_prefix(tree: &Tree, dir: NodeId, base: NodeId) -> String {
    let mut names = Vec::new();
    let mut cur = Some(dir);
    while let Some(n) = cur {
        let node = tree.node(n);
        if n == base || node.parent.is_none() {
            break;
        }
        names.push(&node.name);
        cur = node.parent;
    }
    let mut s = String::new();
    for n in names.iter().rev() {
        s.push_str(&n.to_string_lossy());
        s.push(MAIN_SEPARATOR);
    }
    s
}

/// The text a file of the list is searched and shown by, in `buf`, with
/// where its name starts; `None` if `f` is no longer in the tree.
fn file_text(
    tree: &Tree,
    f: FileRef,
    source: Source,
    dirs: &mut HashMap<NodeId, String>,
    buf: &mut String,
) -> Option<usize> {
    let entry = tree.node(f.dir).files.get(f.idx)?;
    buf.clear();
    if let Source::Files { base: Some(base) } = source {
        let prefix = dirs
            .entry(f.dir)
            .or_insert_with(|| dir_prefix(tree, f.dir, base));
        buf.push_str(prefix);
    }
    let name = buf.len();
    buf.push_str(&entry.name.to_string_lossy());
    Some(name)
}

/// The text of a result (`id` from its [`Rank`]), to show it, with where its
/// name starts; `None` if it is no longer there.
pub fn result_text(
    tree: &Tree,
    files: &[FileRef],
    source: Source,
    id: usize,
    buf: &mut String,
) -> Option<usize> {
    match source {
        Source::Folders => (id < tree.node_count()).then(|| folder_text(tree, id, buf)),
        Source::Files { .. } => {
            let f = *files.get(id)?;
            file_text(tree, f, source, &mut HashMap::new(), buf)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The whole text as a name.
    fn m(query: &str, text: &str) -> Option<Match> {
        Query::new(query).matches(text, 0, &mut Vec::new())
    }

    /// A path: the name starts after the last `/`.
    fn mp(query: &str, path: &str) -> Option<Match> {
        let name = path.rfind('/').map_or(0, |i| i + 1);
        Query::new(query).matches(path, name, &mut Vec::new())
    }

    fn score(query: &str, path: &str) -> i32 {
        mp(query, path).expect(path).score
    }

    /// The texts that match, the best first.
    fn ranked<'a>(query: &str, texts: &[&'a str]) -> Vec<&'a str> {
        let q = Query::new(query);
        let mut top = Top::new(10);
        let mut scratch = Vec::new();
        for (order, t) in texts.iter().enumerate() {
            let name = t.rfind('/').map_or(0, |i| i + 1);
            if let Some(found) = q.matches(t, name, &mut scratch) {
                top.push(Rank {
                    score: found.score,
                    len: t.chars().count(),
                    order,
                    id: order,
                });
            }
        }
        top.into_sorted().iter().map(|r| texts[r.order]).collect()
    }

    #[test]
    fn letters_in_order_and_where_they_are() {
        let found = m("rdme", "README.md").unwrap();
        assert_eq!(found.positions, [0, 3, 4, 5]);
        assert_eq!(m("rdme", "src/readme").unwrap().positions, [4, 7, 8, 9]);
        assert!(m("rdmx", "README.md").is_none());
        assert!(m("emdr", "README.md").is_none(), "order counts");
        assert!(m("cfgxy", "config/xylem").is_some());
    }

    #[test]
    fn an_upper_case_letter_makes_a_term_match_case() {
        assert!(m("readme", "README").is_some());
        assert!(m("Rd", "readme").is_none());
        assert!(m("RD", "README").is_some());
        // Each term on its own.
        assert!(m("Read me", "README").is_none());
        assert!(m("read ME", "README").is_some());
        // Beyond ASCII too.
        assert!(m("ção", "AÇÃO").is_some());
        assert!(m("Ção", "ação").is_none());
    }

    #[test]
    fn every_term_matches_in_any_order() {
        assert!(m("xy docs", "xylem/docs").is_some());
        assert!(m("docs xy", "xylem/docs").is_some());
        assert!(m("docs zz", "xylem/docs").is_none());
        // Spaces alone are no query: everything matches.
        assert!(Query::new("   ").is_empty());
        assert_eq!(m("", "a").unwrap().score, 0);
        // The letters of both terms are marked once.
        assert_eq!(m("xy yl", "xylem").unwrap().positions, [0, 1, 2]);
    }

    #[test]
    fn letters_together_and_at_word_starts_rank_first() {
        assert_eq!(ranked("doc", &["d_o_c", "docs"]), ["docs", "d_o_c"]);
        assert_eq!(
            ranked("fs", &["offset", "file_spec"]),
            ["file_spec", "offset"]
        );
        assert_eq!(
            ranked("fs", &["offset", "fileSpec"]),
            ["fileSpec", "offset"]
        );
        assert_eq!(ranked("v2", &["av2x", "view2"]), ["view2", "av2x"]);
        // After a / more than inside a word.
        assert!(score("vw", "src/viewer") > score("vw", "srcviewer"));
    }

    #[test]
    fn the_name_counts_more_than_the_folders_above_it() {
        assert_eq!(
            ranked("xylem", &["xylem/a", "a/xylem"]),
            ["a/xylem", "xylem/a"]
        );
        // The term is looked for in the name first, even if it also matches
        // earlier in the path.
        let found = mp("ab", "ab/x/ab").unwrap();
        assert_eq!(found.positions, [5, 6]);
        // In the folders only, it still matches.
        assert_eq!(mp("ab", "ab/x/cd").unwrap().positions, [0, 1]);
    }

    #[test]
    fn ties_go_to_the_shorter_text_then_to_the_list() {
        assert_eq!(ranked("ab", &["ab.txt", "ab.c"]), ["ab.c", "ab.txt"]);
        assert_eq!(ranked("ab", &["ab.d", "ab.c"]), ["ab.d", "ab.c"]);
    }

    #[test]
    fn the_shown_text_is_what_matches() {
        // A name that is not UTF-8 matches as shown, with U+FFFD.
        let lossy = String::from_utf8_lossy(b"a\xFFb").into_owned();
        assert_eq!(m("ab", &lossy).unwrap().positions, [0, 2]);
        // Positions are characters, not bytes: a wide or treated character
        // before the match does not move the highlight.
        assert_eq!(m("x", "ção x").unwrap().positions, [4]);
        assert_eq!(m("x", "a\u{2066}x").unwrap().positions, [2]);
    }

    #[test]
    fn a_long_query_is_cut_and_a_grown_one_narrows() {
        let long = "a".repeat(MAX_QUERY + 50);
        let q = Query::new(&long);
        assert_eq!(q.terms[0].chars.len(), MAX_QUERY);
        assert!(
            q.matches(&"a".repeat(MAX_QUERY), 0, &mut Vec::new())
                .is_some()
        );
        let (ab, abc, ab_c, a_b) = (
            Query::new("ab"),
            Query::new("abc"),
            Query::new("ab c"),
            Query::new("a b"),
        );
        assert!(abc.narrows(&ab));
        assert!(ab_c.narrows(&ab));
        assert!(!ab.narrows(&abc), "deleted letters widen the search");
        assert!(!a_b.narrows(&ab));
        // A name whose bytes split before its name start does not panic.
        assert!(Query::new("x").matches("çx", 1, &mut Vec::new()).is_some());
    }

    #[test]
    fn the_top_keeps_only_the_best() {
        let mut top = Top::new(3);
        for (order, score) in [5, 1, 9, 7, 3, 9].into_iter().enumerate() {
            top.push(Rank {
                score,
                len: 1,
                order,
                id: order,
            });
        }
        assert_eq!(top.len(), 3);
        let kept: Vec<(i32, usize)> = top
            .into_sorted()
            .iter()
            .map(|r| (r.score, r.order))
            .collect();
        assert_eq!(kept, [(9, 2), (9, 5), (7, 3)]);
    }

    /// A virtual tree of these folders.
    fn folders(dirs: &[&str]) -> Tree {
        let listing = dirs
            .iter()
            .map(|d| (std::path::PathBuf::from(d), None))
            .collect();
        Tree::from_listing(std::path::PathBuf::from("/root"), listing)
    }

    /// The folders a whole pass finds, the best first.
    fn found(tree: &Tree, query: &str) -> Vec<String> {
        let mut pass = Pass::new(Query::new(query), Source::Folders, tree, &[]);
        pass.step(tree, &[], Duration::MAX);
        assert!(pass.is_done());
        texts(tree, &pass)
    }

    fn texts(tree: &Tree, pass: &Pass) -> Vec<String> {
        let mut buf = String::new();
        pass.best()
            .iter()
            .map(|r| {
                result_text(tree, &[], Source::Folders, r.id, &mut buf).unwrap();
                buf.replace(MAIN_SEPARATOR, "/")
            })
            .collect()
    }

    #[test]
    fn a_pass_finds_folders_by_their_path_from_the_root() {
        let tree = folders(&["docs/plans", "src/model", "src/view", "README"]);
        assert_eq!(found(&tree, "dp"), ["docs/plans"]);
        assert_eq!(found(&tree, "src"), ["src", "src/view", "src/model"]);
        // The name counts more than the folders above it.
        assert_eq!(found(&tree, "s")[0], "src");
        assert!(found(&tree, "").is_empty(), "an empty query finds nothing");
        assert!(found(&tree, "zz").is_empty());
    }

    #[test]
    fn a_pass_in_slices_finds_what_a_whole_pass_finds() {
        let all: Vec<String> = paths(3000).into_iter().map(|(p, _)| p).collect();
        let dirs: Vec<&str> = all.iter().map(String::as_str).collect();
        let tree = folders(&dirs);
        for query in ["cfgxy", "src view", "d"] {
            let mut whole = Pass::new(Query::new(query), Source::Folders, &tree, &[]);
            whole.step(&tree, &[], Duration::MAX);
            let mut sliced = Pass::new(Query::new(query), Source::Folders, &tree, &[]);
            let mut slices = 0;
            while !sliced.is_done() {
                // No time at all: a slice is the items between two looks
                // at the clock.
                sliced.step(&tree, &[], Duration::ZERO);
                slices += 1;
            }
            assert!(slices > 10, "{query}: {slices} slices");
            assert_eq!(sliced.best(), whole.best(), "{query}");
            assert_eq!(sliced.found(), whole.found(), "{query}");
            let (seen, total) = sliced.progress();
            assert_eq!(seen, tree.node_count() - 1);
            assert_eq!(total, seen);
        }
    }

    #[test]
    fn a_narrower_query_finds_among_the_matches_what_a_new_pass_finds() {
        let all: Vec<String> = paths(3000).into_iter().map(|(p, _)| p).collect();
        let dirs: Vec<&str> = all.iter().map(String::as_str).collect();
        let tree = folders(&dirs);
        for (before, after) in [("s", "sr"), ("do", "doc"), ("x", "x d"), ("im", "imaG")] {
            let mut first = Pass::new(Query::new(before), Source::Folders, &tree, &[]);
            first.step(&tree, &[], Duration::MAX);
            let query = Query::new(after);
            assert!(first.narrows_to(&query), "{before} → {after}");
            let looked = first.found();
            let mut narrow = Pass::into_narrowed(first, query);
            narrow.step(&tree, &[], Duration::MAX);
            assert_eq!(narrow.progress().0, looked, "only the earlier matches");
            assert_eq!(texts(&tree, &narrow), found(&tree, after), "{after}");
        }
        // Not when the text changed before its end, or the pass is not over.
        let mut pass = Pass::new(Query::new("sr"), Source::Folders, &tree, &[]);
        assert!(!pass.narrows_to(&Query::new("src")), "not over");
        pass.step(&tree, &[], Duration::MAX);
        assert!(!pass.narrows_to(&Query::new("s")));
        assert!(!pass.narrows_to(&Query::new("xsr")));
        assert!(pass.narrows_to(&Query::new("src")));
    }

    /// A leading dot hides a folder only on Unix; on Windows it is the
    /// hidden attribute, which a listing does not carry.
    #[cfg(unix)]
    #[test]
    fn hidden_folders_are_found_only_when_the_tree_shows_them() {
        let mut tree = folders(&[".cache/lib", "src/lib"]);
        assert_eq!(found(&tree, "lib"), ["src/lib", ".cache/lib"]);
        tree.show_hidden = false;
        assert_eq!(found(&tree, "lib"), ["src/lib"], "nor what is inside them");
    }

    /// Paths like those of a big tree: `n` of them, deterministic.
    fn paths(n: usize) -> Vec<(String, usize)> {
        const WORDS: [&str; 24] = [
            "src",
            "docs",
            "home",
            "user",
            "projects",
            "config",
            "cache",
            "lib",
            "share",
            "local",
            "node_modules",
            "target",
            "debug",
            "build",
            "assets",
            "images",
            "Photos",
            "music",
            "readme",
            "viewer",
            "markdown",
            "test",
            "xylem",
            "data",
        ];
        let mut seed: u64 = 0x2545_F491_4F6C_DD1D;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        (0..n)
            .map(|i| {
                let depth = 2 + (next() % 7) as usize;
                let mut p = String::new();
                for _ in 0..depth {
                    p.push_str(WORDS[(next() % WORDS.len() as u64) as usize]);
                    p.push('/');
                }
                p.push_str(&format!("file_{i}.txt"));
                let name = p.rfind('/').map_or(0, |k| k + 1);
                (p, name)
            })
            .collect()
    }

    /// The time of one search over `n` names, keeping the 500 best:
    /// `XY_FUZZY_N=2000000 cargo test fuzzy::tests::timing -- --ignored --nocapture`
    /// (with optimizations for real numbers).
    #[test]
    #[ignore]
    fn timing() {
        let n: usize = std::env::var("XY_FUZZY_N")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100_000);
        let all = paths(n);
        for query in ["cfgxy", "src view", "rdme", "zzzz", "f", "xylem docs 12"] {
            let q = Query::new(query);
            let start = std::time::Instant::now();
            let mut top = Top::new(500);
            let mut scratch = Vec::new();
            let mut found = 0;
            for (order, (p, name)) in all.iter().enumerate() {
                if let Some(x) = q.matches(p, *name, &mut scratch) {
                    found += 1;
                    top.push(Rank {
                        score: x.score,
                        len: p.len(),
                        order,
                        id: order,
                    });
                }
            }
            let best = top.into_sorted();
            println!(
                "{n:>9} names  {query:<14} {found:>8} matches  {:>7.1} ms  best: {}",
                start.elapsed().as_secs_f64() * 1000.0,
                best.first().map_or("", |r| all[r.order].0.as_str())
            );
        }
    }

    /// The cost of getting each folder's path out of the tree, three ways:
    /// `XY_FUZZY_N=2000000 cargo test fuzzy::tests::tree_paths_timing -- --ignored --nocapture`
    /// (with optimizations for real numbers).
    #[test]
    #[ignore]
    fn tree_paths_timing() {
        use crate::model::tree::{NodeId, ROOT, Tree};
        use std::path::PathBuf;
        use std::time::Instant;

        let n: usize = std::env::var("XY_FUZZY_N")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100_000);
        // Folders only: each synthetic path becomes a chain of folders.
        let listing = paths(n)
            .into_iter()
            .map(|(p, _)| (PathBuf::from(p.replace("file_", "d")), None))
            .collect();
        let start = Instant::now();
        let tree = Tree::from_listing(PathBuf::from("/root"), listing);
        let built = start.elapsed();

        // Every folder below the root, in tree order.
        let mut ids: Vec<NodeId> = Vec::new();
        let mut stack: Vec<NodeId> = tree.node(ROOT).children.iter().rev().copied().collect();
        while let Some(id) = stack.pop() {
            ids.push(id);
            stack.extend(tree.node(id).children.iter().rev());
        }
        let ms = |t: std::time::Duration| t.as_secs_f64() * 1000.0;
        println!(
            "{:>9} folders (tree built in {:.0} ms)",
            ids.len(),
            ms(built)
        );

        let root_len = tree.path(ROOT).as_os_str().len() + 1;
        let search =
            |q: &Query, text: &str, name: usize, scratch: &mut Vec<char>, top: &mut Top, order| {
                if let Some(x) = q.matches(text, name, scratch) {
                    top.push(Rank {
                        score: x.score,
                        len: text.len(),
                        order,
                        id: order,
                    });
                }
            };

        for query in ["cfgxy", "rdme", "zzzz"] {
            let q = Query::new(query);
            let mut scratch = Vec::new();

            // (a) Tree::path for each folder, made text.
            let start = Instant::now();
            let mut top = Top::new(500);
            for (order, &id) in ids.iter().enumerate() {
                let full = tree.path(id);
                let text = full.to_string_lossy();
                let text = text.get(root_len..).unwrap_or("");
                let name = text.rfind('/').map_or(0, |k| k + 1);
                search(&q, text, name, &mut scratch, &mut top, order);
            }
            let a = start.elapsed();

            // (b) One walk, the prefix kept in a buffer.
            let start = Instant::now();
            let mut top = Top::new(500);
            let mut buf = String::new();
            let mut order = 0;
            // (folder, length of the buffer before its name)
            let mut stack: Vec<(NodeId, usize)> = tree
                .node(ROOT)
                .children
                .iter()
                .rev()
                .map(|&c| (c, 0))
                .collect();
            while let Some((id, at)) = stack.pop() {
                buf.truncate(at);
                if at > 0 {
                    buf.push('/');
                }
                let name = buf.len();
                buf.push_str(&tree.node(id).name.to_string_lossy());
                search(&q, &buf, name, &mut scratch, &mut top, order);
                order += 1;
                let here = buf.len();
                stack.extend(tree.node(id).children.iter().rev().map(|&c| (c, here)));
            }
            let b = start.elapsed();

            // (c) The paths made once and kept; then only the search.
            let start = Instant::now();
            let kept: Vec<(String, usize)> = ids
                .iter()
                .map(|&id| {
                    let full = tree.path(id);
                    let text = full
                        .to_string_lossy()
                        .get(root_len..)
                        .unwrap_or("")
                        .to_string();
                    let name = text.rfind('/').map_or(0, |k| k + 1);
                    (text, name)
                })
                .collect();
            let made = start.elapsed();
            let bytes: usize = kept.iter().map(|(s, _)| s.capacity()).sum::<usize>()
                + kept.capacity() * std::mem::size_of::<(String, usize)>();
            let start = Instant::now();
            let mut top = Top::new(500);
            for (order, (text, name)) in kept.iter().enumerate() {
                search(&q, text, *name, &mut scratch, &mut top, order);
            }
            let c = start.elapsed();

            println!(
                "  {query:<6} (a) path() {:>7.1} ms   (b) walk {:>7.1} ms   (c) make {:>7.1} ms + search {:>7.1} ms, {:>5.0} MB kept",
                ms(a),
                ms(b),
                ms(made),
                ms(c),
                bytes as f64 / 1e6
            );
        }

        // The pass itself, in the slices of the `|` prompt (40 ms): how much
        // the first slice sees, and the whole pass.
        for query in ["cfgxy", "rdme", "f", "f d"] {
            let start = Instant::now();
            let mut pass = Pass::new(Query::new(query), Source::Folders, &tree, &[]);
            pass.step(&tree, &[], std::time::Duration::from_millis(40));
            let first = start.elapsed();
            let (seen, _) = pass.progress();
            let mut slices = 1;
            while !pass.is_done() {
                pass.step(&tree, &[], std::time::Duration::from_millis(40));
                slices += 1;
            }
            let whole = start.elapsed();
            // Typing one more letter: only among the matches.
            let found = pass.found();
            let mut narrow = Pass::into_narrowed(pass, Query::new(&format!("{query}x")));
            let start = Instant::now();
            narrow.step(&tree, &[], std::time::Duration::MAX);
            println!(
                "  pass {query:<6} first slice {:>5.1} ms ({seen:>7} folders)   whole {:>6.1} ms in {slices:>2} slices, {found:>7} found   +x {:>6.1} ms",
                ms(first),
                ms(whole),
                ms(start.elapsed())
            );
        }
    }
}
