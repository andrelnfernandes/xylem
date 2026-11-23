//! Ctrl+S over a file: all the terms of the expression in one reading.
//!
//! In text+Unicode each term is looked for at once as UTF-8, UTF-16 LE and
//! BE and Windows-1252, as bytes, with `aho-corasick`. Ignoring case, the
//! needles carry both cases of each letter (the ASCII ones by the automaton
//! itself), and each hit is decoded again and compared letter by letter, so
//! a hit is a real one: a UTF-16 hit must start at an even offset, and a
//! Windows-1252 hit must not be a piece of a UTF-8 letter (`Ã` is the first
//! byte of `é` in UTF-8).
//!
//! In text only, and for a term with the `*` wildcard, the term is looked
//! for line by line as UTF-8, as [`Pattern`] does, which is also what W's
//! `contains()` does: the same file read once feeds both ways.

use std::io::{self, BufRead};
use std::time::Instant;

use aho_corasick::{AhoCorasick, MatchKind};

use super::expr::{self, Node, Term};
use super::{MAX_LINE, Pattern, SearchSpec, fold};
use crate::charset::CP1252;

/// How many needles one term may have per encoding: past this, only the
/// start of the term goes in the automaton, and the hit is checked whole.
const MAX_VARIANTS: usize = 64;
/// The bytes looked at around a Windows-1252 hit, to tell a UTF-8 letter.
const CONTEXT: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Enc {
    /// `x'...'`: the bytes as they are.
    Raw,
    Utf8,
    Utf16Le,
    Utf16Be,
    Cp1252,
}

const ENCODINGS: [Enc; 4] = [Enc::Utf8, Enc::Utf16Le, Enc::Utf16Be, Enc::Cp1252];

/// What a needle of the automaton stands for.
#[derive(Debug, Clone)]
struct Needle {
    term: usize,
    enc: Enc,
    /// The term's letters, folded if ignoring case.
    want: Vec<char>,
}

/// The terms of a search, ready to look for in a file.
pub struct Matcher {
    node: Node,
    terms: usize,
    case_sensitive: bool,
    ac: Option<AhoCorasick>,
    /// For each needle of `ac`, the terms whose bytes it is.
    needles: Vec<Vec<Needle>>,
    /// The terms looked for line by line.
    lines: Vec<(usize, Pattern)>,
    /// What is kept of a block for the next: a term split between two is found.
    keep: usize,
    /// The terms with a `*`, looked for as text only in text+Unicode.
    pub text_only: Vec<String>,
    /// Letters of the terms whose case is not ignored: they have no single
    /// upper and lower form (`ß`).
    pub cased_as_typed: Vec<char>,
}

/// What a file being read needs from one step to the next.
#[derive(Default)]
pub struct Progress {
    /// The end of the last block, for the automaton.
    buf: Vec<u8>,
    /// Where `buf` starts in the file.
    base: u64,
    /// The line not yet complete.
    line: Vec<u8>,
    found: Vec<bool>,
}

/// The forms of `c` the needles carry: both cases when ignoring it, for the
/// letters with one upper and one lower form. ASCII ones are left to the
/// automaton, which ignores their case itself.
fn forms(c: char, case_sensitive: bool) -> Vec<char> {
    let mut out = vec![c];
    if case_sensitive || c.is_ascii() {
        return out;
    }
    for other in [single(c.to_lowercase()), single(c.to_uppercase())]
        .into_iter()
        .flatten()
    {
        if !out.contains(&other) {
            out.push(other);
        }
    }
    out
}

fn single(mut it: impl Iterator<Item = char>) -> Option<char> {
    let c = it.next()?;
    it.next().is_none().then_some(c)
}

/// A letter whose case `forms` cannot cover.
fn cased_as_typed(c: char) -> bool {
    !c.is_ascii()
        && c.is_alphabetic()
        && (single(c.to_lowercase()).is_none() || single(c.to_uppercase()).is_none())
}

fn cp1252_byte(c: char) -> Option<u8> {
    if c.is_ascii() {
        return Some(c as u8);
    }
    if c == '\u{FFFD}' {
        return None;
    }
    CP1252.iter().position(|&t| t == c).map(|i| 0x80 + i as u8)
}

fn encode(c: char, enc: Enc, out: &mut Vec<u8>) -> Option<()> {
    let mut units = [0u16; 2];
    match enc {
        Enc::Raw => return None,
        Enc::Utf8 => out.extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes()),
        Enc::Utf16Le => c
            .encode_utf16(&mut units)
            .iter()
            .for_each(|u| out.extend_from_slice(&u.to_le_bytes())),
        Enc::Utf16Be => c
            .encode_utf16(&mut units)
            .iter()
            .for_each(|u| out.extend_from_slice(&u.to_be_bytes())),
        Enc::Cp1252 => out.push(cp1252_byte(c)?),
    }
    Some(())
}

/// The needles of a term in one encoding: each mix of the forms of its first
/// letters, as many letters as fit in [`MAX_VARIANTS`]. `None` if a letter
/// cannot be written in it.
fn needles_of(chars: &[char], enc: Enc, case_sensitive: bool) -> Option<Vec<Vec<u8>>> {
    let mut out: Vec<Vec<u8>> = vec![Vec::new()];
    let mut whole = true;
    for &c in chars {
        let forms: Vec<char> = forms(c, case_sensitive)
            .into_iter()
            .filter(|&f| encode(f, enc, &mut Vec::new()).is_some())
            .collect();
        if forms.is_empty() {
            return None;
        }
        if !whole || out.len() * forms.len() > MAX_VARIANTS {
            whole = false;
            continue;
        }
        out = out
            .iter()
            .flat_map(|start| {
                forms.iter().map(move |&f| {
                    let mut v = start.clone();
                    encode(f, enc, &mut v);
                    v
                })
            })
            .collect();
    }
    Some(out)
}

impl Matcher {
    pub fn new(spec: &SearchSpec) -> Result<Self, String> {
        let query = if spec.hex {
            match Pattern::new(spec)? {
                Pattern::Bytes(b) => expr::Query {
                    node: Node::Term(0),
                    terms: vec![Term::Bytes(b)],
                },
                Pattern::Text { .. } => unreachable!("hex gives bytes"),
            }
        } else {
            expr::parse(&spec.text)?
        };
        let cs = spec.case_sensitive;
        let unicode = spec.unicode && !spec.hex;
        let mut patterns: Vec<Vec<u8>> = Vec::new();
        let mut needles: Vec<Vec<Needle>> = Vec::new();
        let mut add =
            |bytes: Vec<u8>, needle: Needle| match patterns.iter().position(|p| *p == bytes) {
                Some(i) => needles[i].push(needle),
                None => {
                    patterns.push(bytes);
                    needles.push(vec![needle]);
                }
            };
        let mut lines = Vec::new();
        let mut text_only = Vec::new();
        let mut cased = Vec::new();
        let mut max_len = 0;
        for (term, t) in query.terms.iter().enumerate() {
            let text = match t {
                Term::Bytes(b) => {
                    max_len = max_len.max(b.len());
                    let needle = Needle {
                        term,
                        enc: Enc::Raw,
                        want: Vec::new(),
                    };
                    add(b.clone(), needle);
                    continue;
                }
                Term::Text(text) => text,
            };
            let pattern = Pattern::new(&SearchSpec {
                text: text.clone(),
                case_sensitive: cs,
                hex: false,
                unicode: false,
            })?;
            let wildcard = matches!(&pattern, Pattern::Text { parts, .. } if parts.len() > 1);
            if !unicode || wildcard {
                if unicode {
                    text_only.push(text.clone());
                }
                lines.push((term, pattern));
                continue;
            }
            let chars: Vec<char> = text.chars().collect();
            let want: Vec<char> = chars.iter().map(|&c| fold(c, cs)).collect();
            if !cs {
                cased.extend(chars.iter().copied().filter(|&c| cased_as_typed(c)));
            }
            max_len = max_len.max(4 * chars.len());
            for enc in ENCODINGS {
                // An ASCII term is the same bytes in UTF-8 and in 1252.
                if enc == Enc::Cp1252 && text.is_ascii() {
                    continue;
                }
                for bytes in needles_of(&chars, enc, cs).unwrap_or_default() {
                    let needle = Needle {
                        term,
                        enc,
                        want: want.clone(),
                    };
                    add(bytes, needle);
                }
            }
        }
        let ac = if patterns.is_empty() {
            None
        } else {
            let ac = AhoCorasick::builder()
                .match_kind(MatchKind::Standard)
                .ascii_case_insensitive(!cs)
                .build(&patterns)
                .map_err(|e| format!("Cannot search for that: {e}"))?;
            Some(ac)
        };
        cased.sort_unstable();
        cased.dedup();
        Ok(Matcher {
            node: query.node,
            terms: query.terms.len(),
            case_sensitive: cs,
            ac,
            needles,
            lines,
            keep: max_len + 2 * CONTEXT,
            text_only,
            cased_as_typed: cased,
        })
    }

    /// Does the content meet the expression? Reads until `until`, at least
    /// one block: `None` if it has not found out yet (or the reader has
    /// nothing yet, `WouldBlock`), and then `progress` keeps what the next
    /// call needs. It stops as soon as the expression is decided.
    pub fn scan(
        &self,
        reader: &mut dyn BufRead,
        progress: &mut Progress,
        until: Instant,
    ) -> io::Result<Option<bool>> {
        let p = progress;
        if p.found.len() != self.terms {
            p.found = vec![false; self.terms];
        }
        loop {
            let chunk = match reader.fill_buf() {
                Ok(chunk) => chunk,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(None),
                Err(e) => return Err(e),
            };
            let eof = chunk.is_empty();
            let n = chunk.len();
            if !self.lines.is_empty() {
                self.feed_lines(chunk, p);
            }
            if self.ac.is_some() {
                p.buf.extend_from_slice(chunk);
            }
            reader.consume(n);
            if eof && !p.line.is_empty() {
                self.check_line(p);
            }
            if let Some(ac) = &self.ac {
                self.find(ac, p, eof);
            }
            let known: Vec<Option<bool>> = p
                .found
                .iter()
                .map(|&f| if f { Some(true) } else { eof.then_some(false) })
                .collect();
            if let Some(decided) = self.node.decide(&known) {
                return Ok(Some(decided));
            }
            if eof {
                // Everything is known at the end: decided above.
                return Ok(Some(false));
            }
            if Instant::now() >= until {
                return Ok(None);
            }
        }
    }

    /// Splits the block into lines, as [`Pattern::scan`] does: a line over
    /// [`MAX_LINE`] is looked at in pieces.
    fn feed_lines(&self, mut rest: &[u8], p: &mut Progress) {
        while !rest.is_empty() {
            let room = MAX_LINE - p.line.len();
            let take = &rest[..room.min(rest.len())];
            let (piece, complete) = match take.iter().position(|&b| b == b'\n') {
                Some(i) => (&take[..=i], true),
                None => (take, false),
            };
            p.line.extend_from_slice(piece);
            rest = &rest[piece.len()..];
            if complete || p.line.len() >= MAX_LINE {
                self.check_line(p);
                p.line.clear();
            }
        }
    }

    fn check_line(&self, p: &mut Progress) {
        let mut text = None;
        for (term, pattern) in &self.lines {
            if p.found[*term] {
                continue;
            }
            let line = text.get_or_insert_with(|| String::from_utf8_lossy(&p.line).into_owned());
            if pattern.find_in_line(line, 0).is_some() {
                p.found[*term] = true;
            }
        }
    }

    /// The automaton over what is kept and the new block; then keeps the end.
    fn find(&self, ac: &AhoCorasick, p: &mut Progress, eof: bool) {
        // Those hits were already seen, with the bytes before them.
        let seen = if p.base > 0 { CONTEXT } else { 0 };
        for m in ac.find_overlapping_iter(&p.buf) {
            let start = m.start();
            if start < seen {
                continue;
            }
            for needle in &self.needles[m.pattern().as_usize()] {
                if !p.found[needle.term] && self.check(needle, &p.buf, start, p.base, eof) {
                    p.found[needle.term] = true;
                }
            }
        }
        if !eof {
            let cut = p.buf.len().saturating_sub(self.keep);
            p.buf.drain(..cut);
            p.base += cut as u64;
        }
    }

    /// Whether the hit at `start` is the term: decoded again in its encoding
    /// and compared letter by letter. False also when it goes past what has
    /// been read: it is kept, and checked again with the next block.
    fn check(&self, needle: &Needle, buf: &[u8], start: usize, base: u64, eof: bool) -> bool {
        let cs = self.case_sensitive;
        let same = |got: Option<char>, want: char| got.is_some_and(|c| fold(c, cs) == want);
        let mut at = start;
        match needle.enc {
            Enc::Raw => true,
            Enc::Utf8 => needle.want.iter().all(|&want| {
                let got = next_utf8(buf, at);
                at += got.map_or(0, |c| c.len_utf8());
                same(got, want)
            }),
            Enc::Utf16Le | Enc::Utf16Be => {
                if !(base + start as u64).is_multiple_of(2) {
                    return false;
                }
                let le = needle.enc == Enc::Utf16Le;
                needle.want.iter().all(|&want| {
                    let got = next_utf16(buf, at, le);
                    at += got.map_or(0, |c| 2 * c.len_utf16());
                    same(got, want)
                })
            }
            Enc::Cp1252 => {
                let end = start + needle.want.len();
                if end + CONTEXT > buf.len() && !eof {
                    return false;
                }
                let Some(bytes) = buf.get(start..end) else {
                    return false;
                };
                bytes.iter().zip(&needle.want).all(|(&b, &want)| {
                    let c = if b < 0x80 {
                        Some(b as char)
                    } else {
                        Some(CP1252[usize::from(b - 0x80)]).filter(|&c| c != '\u{FFFD}')
                    };
                    same(c, want)
                }) && !(start..end).any(|i| buf[i] >= 0x80 && in_utf8_letter(buf, i))
            }
        }
    }
}

/// The UTF-8 letter at `at`, if one is there whole.
fn next_utf8(buf: &[u8], at: usize) -> Option<char> {
    let width = match *buf.get(at)? {
        0x00..=0x7F => 1,
        0xC2..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF4 => 4,
        _ => return None,
    };
    let s = std::str::from_utf8(buf.get(at..at + width)?).ok()?;
    s.chars().next()
}

/// The UTF-16 letter at `at`, if one is there whole.
fn next_utf16(buf: &[u8], at: usize, le: bool) -> Option<char> {
    let unit = |i: usize| -> Option<u16> {
        let b = [*buf.get(i)?, *buf.get(i + 1)?];
        Some(if le {
            u16::from_le_bytes(b)
        } else {
            u16::from_be_bytes(b)
        })
    };
    let first = unit(at)?;
    let units = if (0xD800..0xDC00).contains(&first) {
        vec![first, unit(at + 2)?]
    } else {
        vec![first]
    };
    char::decode_utf16(units).next()?.ok()
}

/// Whether byte `i` is part of a whole UTF-8 letter of two bytes or more.
fn in_utf8_letter(buf: &[u8], i: usize) -> bool {
    (i.saturating_sub(CONTEXT)..=i)
        .any(|j| next_utf8(buf, j).is_some_and(|c| c.len_utf8() > 1 && j + c.len_utf8() > i))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

    fn spec(text: &str) -> SearchSpec {
        SearchSpec::new(text)
    }

    /// Reads `data` in blocks of `block` bytes.
    fn meets_in(m: &Matcher, data: &[u8], block: usize) -> bool {
        let mut reader = BufReader::with_capacity(block, Cursor::new(data.to_vec()));
        let mut p = Progress::default();
        loop {
            if let Some(found) = m.scan(&mut reader, &mut p, Instant::now()).unwrap() {
                return found;
            }
        }
    }

    fn meets(text: &str, data: &[u8]) -> bool {
        let m = Matcher::new(&spec(text)).unwrap();
        meets_in(&m, data, 8192)
    }

    fn utf16(s: &str, le: bool, bom: bool) -> Vec<u8> {
        let mut out = Vec::new();
        let mut units: Vec<u16> = s.encode_utf16().collect();
        if bom {
            units.insert(0, 0xFEFF);
        }
        for u in units {
            out.extend_from_slice(&if le { u.to_le_bytes() } else { u.to_be_bytes() });
        }
        out
    }

    const TEXT: &str = "Linha um\r\nA NOTA FISCAL da Ação, Ünïcode, Привет мир\r\n";

    fn encodings() -> Vec<(&'static str, Vec<u8>)> {
        let cp1252 =
            crate::charset::test_util::encode(&TEXT.replace("Привет мир", "Privet mir"), &CP1252);
        vec![
            ("UTF-8", TEXT.as_bytes().to_vec()),
            ("UTF-16 LE with BOM", utf16(TEXT, true, true)),
            ("UTF-16 BE without BOM", utf16(TEXT, false, false)),
            ("Windows-1252", cp1252),
        ]
    }

    #[test]
    fn the_same_text_in_four_encodings() {
        for (name, data) in encodings() {
            for term in ["nota fiscal", "AÇÃO", "ação", "ünÏcode", "\"da Ação\""] {
                assert!(meets(term, &data), "{term} in {name}");
            }
            assert!(!meets("nada", &data), "nada in {name}");
            assert!(!meets("fiscal da ação x", &data), "{name}");
            // Matching case.
            let case = Matcher::new(&SearchSpec {
                case_sensitive: true,
                ..spec("Ação")
            })
            .unwrap();
            assert!(meets_in(&case, &data, 8192), "{name}");
            let case = Matcher::new(&SearchSpec {
                case_sensitive: true,
                ..spec("AÇÃO")
            })
            .unwrap();
            assert!(!meets_in(&case, &data, 8192), "{name}");
        }
        // Cyrillic in both cases, in UTF-8 and UTF-16.
        for data in [TEXT.as_bytes().to_vec(), utf16(TEXT, true, false)] {
            assert!(meets("ПРИВЕТ МИР", &data));
            assert!(meets("привет", &data));
        }
    }

    #[test]
    fn text_only_does_not_see_utf16() {
        let only = Matcher::new(&SearchSpec {
            unicode: false,
            ..spec("nota fiscal")
        })
        .unwrap();
        assert!(meets_in(&only, TEXT.as_bytes(), 8192));
        assert!(!meets_in(&only, &utf16(TEXT, true, true), 8192));
    }

    #[test]
    fn a_term_split_between_two_blocks_is_found() {
        for (name, data) in encodings() {
            for block in [1, 2, 3, 5, 7, 16] {
                let m = Matcher::new(&spec("fiscal da ação")).unwrap();
                assert!(meets_in(&m, &data, block), "{name}, blocks of {block}");
                let m = Matcher::new(&spec("ação AND NOT nada")).unwrap();
                assert!(meets_in(&m, &data, block), "{name}, blocks of {block}");
            }
        }
        let mut data = vec![b'x'; 10_000];
        data.extend_from_slice(b"MZ");
        let m = Matcher::new(&spec("x'4D5A'")).unwrap();
        assert!(meets_in(&m, &data, 7));
    }

    #[test]
    fn hits_that_are_not_the_term_are_left_out() {
        // `ã` is 0xE3 in 1252 and `Ã` is 0xC3, the first byte of `é` in UTF-8.
        assert!(!meets("ã", "café".as_bytes()));
        assert!(meets("ã", &crate::charset::test_util::encode("Ã", &CP1252)));
        // UTF-16 starts at an even offset.
        let mut odd = vec![b'.'];
        odd.extend(utf16("abc", true, false));
        assert!(!meets("abc", &odd[..odd.len() - 1]));
        assert!(meets("abc", &utf16("abc", true, false)));
    }

    #[test]
    fn and_or_not_with_parentheses() {
        let data = "jpg e png, sem gif".as_bytes();
        for (expr, want) in [
            ("jpg AND png", true),
            ("jpg AND bmp", false),
            ("bmp OR png", true),
            ("bmp OR tif", false),
            ("NOT gif", false),
            ("NOT bmp", true),
            ("(bmp OR jpg) AND NOT (tif OR webp)", true),
            ("(bmp OR jpg) AND NOT (tif OR gif)", false),
            ("NOT (bmp AND jpg)", true),
            ("x'6A7067' AND \"e png\"", true),
        ] {
            assert_eq!(meets(expr, data), want, "{expr}");
        }
    }

    #[test]
    fn decides_without_reading_to_the_end() {
        let mut data = b"chave ".to_vec();
        data.extend(vec![b'x'; 100_000]);
        let m = Matcher::new(&spec("chave OR nada")).unwrap();
        let mut reader = BufReader::with_capacity(64, Cursor::new(data));
        let mut p = Progress::default();
        assert_eq!(
            m.scan(&mut reader, &mut p, Instant::now()).unwrap(),
            Some(true)
        );
        assert!(reader.into_inner().position() < 1000);
    }

    #[test]
    fn wildcards_fall_back_to_text_only_and_say_so() {
        let m = Matcher::new(&spec("erro*disco OR ação")).unwrap();
        assert_eq!(m.text_only, ["erro*disco"]);
        assert!(meets_in(&m, "um erro no disco".as_bytes(), 4));
        assert!(!meets_in(&m, &utf16("um erro no disco", true, true), 4));
        assert!(meets_in(&m, &utf16("uma ação", true, true), 4));
        assert_eq!(Matcher::new(&spec("Straße")).unwrap().cased_as_typed, ['ß']);
    }
}
