//! Pattern and mask engine for batch renaming tagged files.
//!
//! Supports:
//! - Wildcard masks: `*.jpg`, `new_*.*`, `*_backup.*`, `???_*.*`
//! - Search & replace: `s/from/to/`, `s/from/to/i`, `/from/to/` (literal)
//! - Regular expressions with the `r` flag: `s/(\d+)-(\w+)/$2-$1/r`, with
//!   `$1`, `${name}` and `\U`, `\L`, `\E`, `\u`, `\l` for case
//! - Incremental counters: `%03d`, `%02d`, `%d`, `<001>`, `<01>`, `<#>`
//! - Case conversion: `<lower>`, `<upper>`, `<title>`, `*.<lower>`, or CaseMode (`F2`)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CaseMode {
    #[default]
    None,
    Lower,
    Upper,
    Title,
}

impl CaseMode {
    pub fn next(self) -> Self {
        match self {
            CaseMode::None => CaseMode::Lower,
            CaseMode::Lower => CaseMode::Upper,
            CaseMode::Upper => CaseMode::Title,
            CaseMode::Title => CaseMode::None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            CaseMode::None => "",
            CaseMode::Lower => "lowercase",
            CaseMode::Upper => "UPPERCASE",
            CaseMode::Title => "Title Case",
        }
    }

    pub fn apply(self, s: &str) -> String {
        match self {
            CaseMode::None => s.to_string(),
            CaseMode::Lower => s.to_lowercase(),
            CaseMode::Upper => s.to_uppercase(),
            CaseMode::Title => title_case(s),
        }
    }
}

pub fn title_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut new_word = true;
    for c in s.chars() {
        if c.is_alphanumeric() {
            if new_word {
                out.extend(c.to_uppercase());
                new_word = false;
            } else {
                out.extend(c.to_lowercase());
            }
        } else {
            new_word = true;
            out.push(c);
        }
    }
    out
}

#[derive(Debug, Clone)]
pub struct RenameMask {
    pattern: String,
    case_mode: CaseMode,
    /// `s/from/to/r`: the expression, compiled once (or why it could not be).
    regex: Option<Result<RegexReplace, String>>,
}

impl RenameMask {
    /// A mask whose regular expression may be invalid: then it keeps every
    /// name ([`RenameMask::parse`] says why).
    pub fn new(pattern: &str, case_mode: CaseMode) -> Self {
        let pattern = pattern.trim().to_string();
        let regex = split_search_replace(&pattern)
            .filter(|(_, _, flags)| flags.contains('r'))
            .map(|(from, to, flags)| RegexReplace::new(from, to, flags.contains('i')));
        Self {
            pattern,
            case_mode,
            regex,
        }
    }

    /// The mask, or why its regular expression is invalid.
    pub fn parse(pattern: &str, case_mode: CaseMode) -> Result<Self, String> {
        let mask = Self::new(pattern, case_mode);
        if let Some(Err(e)) = &mask.regex {
            return Err(e.clone());
        }
        Ok(mask)
    }

    pub fn apply(&self, original_name: &str, index: usize) -> String {
        let pat = self.pattern.as_str();

        // 1. Regular expression: s/from/to/r (invalid: the name stays).
        match &self.regex {
            Some(Ok(regex)) => return self.case_mode.apply(&regex.replace(original_name)),
            Some(Err(_)) => return original_name.to_string(),
            None => {}
        }

        // 2. Check for search & replace: s/from/to/[i] or /from/to/[i]
        if let Some(replaced) = try_search_replace(pat, original_name) {
            return self.case_mode.apply(&replaced);
        }

        // 2. Global case conversion tokens
        let lower_pat = pat.to_lowercase();
        if lower_pat == "<lower>" || lower_pat == "<l>" {
            return original_name.to_lowercase();
        }
        if lower_pat == "<upper>" || lower_pat == "<u>" {
            return original_name.to_uppercase();
        }
        if lower_pat == "<title>" || lower_pat == "<cap>" || lower_pat == "<c>" {
            return title_case(original_name);
        }

        // 3. Counter substitution in the pattern (%03d, %02d, %d, <001>, <01>, <#>)
        let pat_with_counter = substitute_counter(pat, index);

        // 4. Split filename into stem and extension
        let (stem, ext) = split_stem_ext(original_name);
        let had_dot = ext.is_some();
        let ext = ext.unwrap_or("");

        // 5. Check if pattern specifies an extension (contains dot)
        let result = if let Some(dot_idx) = pat_with_counter.rfind('.') {
            let stem_pat = &pat_with_counter[..dot_idx];
            let ext_pat = &pat_with_counter[dot_idx + 1..];
            let new_stem = apply_sub_pattern(stem, stem_pat);
            let new_ext = apply_sub_pattern(ext, ext_pat);
            // "a." stays "a." with `*.*`; an empty extension in the mask
            // (`*.`) is what removes it.
            if new_ext.is_empty() && !(had_dot && !ext_pat.is_empty()) {
                new_stem
            } else {
                format!("{new_stem}.{new_ext}")
            }
        } else {
            apply_sub_pattern(original_name, &pat_with_counter)
        };

        // 6. Apply case conversion if specified
        self.case_mode.apply(&result)
    }
}

/// "a.tar.gz" → ("a.tar", Some("gz")); "a." → ("a", Some("")); "Makefile"
/// and ".bashrc" have no extension.
fn split_stem_ext(name: &str) -> (&str, Option<&str>) {
    if name.starts_with('.') && name[1..].find('.').is_none() {
        return (name, None);
    }
    match name.rfind('.') {
        Some(idx) => (&name[..idx], Some(&name[idx + 1..])),
        None => (name, None),
    }
}

/// `s/from/to/flags` or `/from/to/flags` (the flags and the last slash
/// are optional) as its three parts.
fn split_search_replace(pattern: &str) -> Option<(&str, &str, &str)> {
    let p = pattern
        .strip_prefix("s/")
        .unwrap_or_else(|| pattern.strip_prefix('/').unwrap_or(""));
    if p.is_empty() || (!pattern.starts_with("s/") && !pattern.starts_with('/')) {
        return None;
    }
    let parts: Vec<&str> = p.split('/').collect();
    if parts.len() < 2 {
        return None;
    }
    Some((parts[0], parts[1], parts.get(2).copied().unwrap_or("")))
}

fn try_search_replace(pattern: &str, source: &str) -> Option<String> {
    let (from, to, flags) = split_search_replace(pattern)?;
    let case_insensitive = flags.contains('i');

    if from.is_empty() {
        return Some(source.to_string());
    }

    if case_insensitive {
        Some(replace_case_insensitive(source, from, to))
    } else {
        Some(source.replace(from, to))
    }
}

/// A piece of the replacement of a regular expression.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Piece {
    Text(String),
    /// `$1`, `${2}`, `$name`, `${name}` (by number, already resolved).
    Group(usize),
    /// `\U` and `\L` until `\E` (or the next one); `\u` and `\l` for the
    /// next character only.
    Case(CaseOp),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaseOp {
    Upper,
    Lower,
    End,
    NextUpper,
    NextLower,
}

/// `s/from/to/r`: every match of `from` replaced by `to`.
#[derive(Debug, Clone)]
struct RegexReplace {
    regex: regex::Regex,
    to: Vec<Piece>,
    /// An empty expression keeps the name, as an empty literal does.
    keep: bool,
}

impl RegexReplace {
    fn new(from: &str, to: &str, case_insensitive: bool) -> Result<Self, String> {
        let regex = super::regex_build::build(from, case_insensitive)?;
        let to = parse_replacement(to, &regex)?;
        Ok(RegexReplace {
            regex,
            to,
            keep: from.is_empty(),
        })
    }

    fn replace(&self, source: &str) -> String {
        if self.keep {
            return source.to_string();
        }
        self.regex
            .replace_all(source, |caps: &regex::Captures| self.expand(caps))
            .into_owned()
    }

    fn expand(&self, caps: &regex::Captures) -> String {
        #[derive(Clone, Copy)]
        enum Conv {
            Upper,
            Lower,
        }
        fn push(out: &mut String, c: char, conv: Option<Conv>) {
            match conv {
                Some(Conv::Upper) => out.extend(c.to_uppercase()),
                Some(Conv::Lower) => out.extend(c.to_lowercase()),
                None => out.push(c),
            }
        }
        let mut out = String::new();
        let (mut mode, mut next) = (None, None);
        for piece in &self.to {
            let text = match piece {
                Piece::Text(t) => t.as_str(),
                Piece::Group(i) => caps.get(*i).map_or("", |m| m.as_str()),
                Piece::Case(op) => {
                    match op {
                        CaseOp::Upper => mode = Some(Conv::Upper),
                        CaseOp::Lower => mode = Some(Conv::Lower),
                        CaseOp::End => mode = None,
                        CaseOp::NextUpper => next = Some(Conv::Upper),
                        CaseOp::NextLower => next = Some(Conv::Lower),
                    }
                    continue;
                }
            };
            for c in text.chars() {
                push(&mut out, c, next.take().or(mode));
            }
        }
        out
    }
}

/// The replacement of `s/from/to/r`: `$$` is a `$`, `\\` a backslash; a
/// group that the expression does not have is an error.
fn parse_replacement(to: &str, regex: &regex::Regex) -> Result<Vec<Piece>, String> {
    let groups = regex.captures_len();
    let group = |name: &str| -> Result<usize, String> {
        if name.bytes().all(|b| b.is_ascii_digit()) {
            match name.parse::<usize>() {
                Ok(n) if n < groups => Ok(n),
                _ => Err(format!(
                    "no group ${name}: the regex has {}",
                    match groups - 1 {
                        0 => "no groups".to_string(),
                        1 => "1 group".to_string(),
                        n => format!("{n} groups"),
                    }
                )),
            }
        } else {
            regex
                .capture_names()
                .position(|n| n == Some(name))
                .ok_or_else(|| format!("no group named '{name}' in the regex"))
        }
    };
    let mut pieces = Vec::new();
    let mut text = String::new();
    let mut chars = to.chars().peekable();
    let flush = |text: &mut String, pieces: &mut Vec<Piece>| {
        if !text.is_empty() {
            pieces.push(Piece::Text(std::mem::take(text)));
        }
    };
    while let Some(c) = chars.next() {
        match c {
            '$' => match chars.peek().copied() {
                Some('$') => {
                    chars.next();
                    text.push('$');
                }
                Some('{') => {
                    chars.next();
                    let mut name = String::new();
                    loop {
                        match chars.next() {
                            Some('}') => break,
                            Some(c) => name.push(c),
                            None => return Err(format!("'${{{name}' has no closing }}")),
                        }
                    }
                    flush(&mut text, &mut pieces);
                    pieces.push(Piece::Group(group(&name)?));
                }
                Some(c) if c.is_ascii_alphanumeric() || c == '_' => {
                    // `$1a` is group 1 and an `a`; `$name` takes letters,
                    // digits and `_`.
                    let digits = c.is_ascii_digit();
                    let mut name = String::new();
                    while let Some(c) = chars.next_if(|&c| {
                        if digits {
                            c.is_ascii_digit()
                        } else {
                            c.is_ascii_alphanumeric() || c == '_'
                        }
                    }) {
                        name.push(c);
                    }
                    flush(&mut text, &mut pieces);
                    pieces.push(Piece::Group(group(&name)?));
                }
                _ => text.push('$'),
            },
            '\\' => {
                let op = match chars.peek().copied() {
                    Some('U') => Some(CaseOp::Upper),
                    Some('L') => Some(CaseOp::Lower),
                    Some('E') => Some(CaseOp::End),
                    Some('u') => Some(CaseOp::NextUpper),
                    Some('l') => Some(CaseOp::NextLower),
                    _ => None,
                };
                match (op, chars.peek().copied()) {
                    (Some(op), _) => {
                        chars.next();
                        flush(&mut text, &mut pieces);
                        pieces.push(Piece::Case(op));
                    }
                    (None, Some(c @ ('\\' | '$'))) => {
                        chars.next();
                        text.push(c);
                    }
                    // Any other backslash is just that.
                    _ => text.push('\\'),
                }
            }
            c => text.push(c),
        }
    }
    flush(&mut text, &mut pieces);
    Ok(pieces)
}

fn replace_case_insensitive(source: &str, from: &str, to: &str) -> String {
    let mut result = String::new();
    let mut rest = source;
    while let Some(c) = rest.chars().next() {
        match match_ignoring_case(rest, from) {
            Some(len) => {
                result.push_str(to);
                rest = &rest[len..];
            }
            None => {
                result.push(c);
                rest = &rest[c.len_utf8()..];
            }
        }
    }
    result
}

/// Length in bytes of `from` at the start of `s`, comparing character by
/// character in lowercase. The length is measured in `s`: a character may
/// take more or fewer bytes than its match in `from` (the Kelvin sign is a
/// `k` in three bytes).
fn match_ignoring_case(s: &str, from: &str) -> Option<usize> {
    let mut chars = s.chars();
    let mut len = 0;
    for f in from.chars() {
        let c = chars.next()?;
        if !c.to_lowercase().eq(f.to_lowercase()) {
            return None;
        }
        len += c.len_utf8();
    }
    Some(len)
}

fn substitute_counter(pattern: &str, index: usize) -> String {
    let mut out = pattern.to_string();

    // %04d, %03d, %02d, %d
    if out.contains("%04d") {
        out = out.replace("%04d", &format!("{index:04}"));
    }
    if out.contains("%03d") {
        out = out.replace("%03d", &format!("{index:03}"));
    }
    if out.contains("%02d") {
        out = out.replace("%02d", &format!("{index:02}"));
    }
    if out.contains("%d") {
        out = out.replace("%d", &format!("{index}"));
    }

    // Angle bracket counters: <0001>, <001>, <01>, <#>, <1>
    if out.contains("<0001>") {
        out = out.replace("<0001>", &format!("{index:04}"));
    }
    if out.contains("<001>") {
        out = out.replace("<001>", &format!("{index:03}"));
    }
    if out.contains("<01>") {
        out = out.replace("<01>", &format!("{index:02}"));
    }
    if out.contains("<#>") {
        out = out.replace("<#>", &format!("{index}"));
    }
    if out.contains("<1>") {
        out = out.replace("<1>", &format!("{index}"));
    }

    out
}

fn apply_sub_pattern(source: &str, pat: &str) -> String {
    let lower_pat = pat.to_lowercase();
    if lower_pat == "<lower>" || lower_pat == "<l>" {
        return source.to_lowercase();
    }
    if lower_pat == "<upper>" || lower_pat == "<u>" {
        return source.to_uppercase();
    }
    if lower_pat == "<title>" || lower_pat == "<cap>" || lower_pat == "<c>" {
        return title_case(source);
    }

    if pat == "*" {
        return source.to_string();
    }

    if pat.contains('*') {
        let parts: Vec<&str> = pat.split('*').collect();
        let mut out = String::new();
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                out.push_str(source);
            }
            if !part.is_empty() {
                out.push_str(&map_question_marks(part, source, 0));
            }
        }
        out
    } else if pat.contains('?') {
        map_question_marks(pat, source, 0)
    } else {
        pat.to_string()
    }
}

fn map_question_marks(pattern: &str, source: &str, start_idx: usize) -> String {
    let src_chars: Vec<char> = source.chars().collect();
    let mut out = String::new();
    let mut idx = start_idx;
    for c in pattern.chars() {
        if c == '?' {
            if idx < src_chars.len() {
                out.push(src_chars[idx]);
                idx += 1;
            }
        } else {
            out.push(c);
            idx += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// Masks made of the pieces the engine understands, so random cases
    /// reach every branch (and not just plain text).
    fn mask() -> impl Strategy<Value = String> {
        let piece = prop::sample::select(vec![
            "*", "?", ".", "*.*", "<#>", "<01>", "<001>", "%d", "%03d", "<lower>", "<upper>",
            "<title>", "s/", "/", "i", "a", "B", "k", "_", "-", " ", "ç", "K", "\u{212a}", "İ",
            "ß",
        ]);
        prop::collection::vec(piece, 0..8).prop_map(|p| p.concat())
    }

    #[test]
    fn found_by_the_properties() {
        // The Kelvin sign lowercases to an ASCII `k`, in fewer bytes.
        let mask = RenameMask::new("s/k/x/i", CaseMode::None);
        assert_eq!(mask.apply("\u{212a}elvin.txt", 1), "xelvin.txt");
        // A trailing dot is kept by `*.*` and removed by `*.`.
        assert_eq!(RenameMask::new("*.*", CaseMode::None).apply("A.", 1), "A.");
        assert_eq!(RenameMask::new("*.", CaseMode::None).apply("a.txt", 1), "a");
    }

    proptest! {
        /// No mask, name or number makes it panic.
        #[test]
        fn never_panics(
            pattern in prop_oneof![mask(), any::<String>()],
            name in prop_oneof![mask(), "\\PC{0,24}"],
            index in 0usize..100_000,
            case in prop::sample::select(vec![
                CaseMode::None, CaseMode::Lower, CaseMode::Upper, CaseMode::Title,
            ]),
        ) {
            let _ = RenameMask::new(&pattern, case).apply(&name, index);
        }

        /// `*` and `*.*` keep the name as it is.
        #[test]
        fn star_keeps_the_name(name in "[^/\\x00]{1,24}") {
            prop_assert_eq!(RenameMask::new("*", CaseMode::None).apply(&name, 1), name.clone());
            prop_assert_eq!(RenameMask::new("*.*", CaseMode::None).apply(&name, 1), name);
        }

        /// The case modes are the standard conversions of the whole name.
        #[test]
        fn case_modes_convert_the_whole_name(name in "\\PC{1,24}") {
            let apply = |case| RenameMask::new("*", case).apply(&name, 1);
            prop_assert_eq!(apply(CaseMode::Lower), name.to_lowercase());
            prop_assert_eq!(apply(CaseMode::Upper), name.to_uppercase());
        }

        /// `s/from/to/` replaces every occurrence, like `str::replace`.
        #[test]
        fn search_and_replace_is_str_replace(
            name in "[a-c.]{0,16}",
            from in "[a-c]{1,3}",
            to in "[a-cx]{0,3}",
        ) {
            let mask = RenameMask::new(&format!("s/{from}/{to}/"), CaseMode::None);
            prop_assert_eq!(mask.apply(&name, 1), name.replace(&from, &to));
        }

        /// With `r`, an expression without special characters replaces as
        /// the literal does.
        #[test]
        fn regex_of_plain_text_is_the_literal(
            name in "[a-c.]{0,16}",
            from in "[a-c]{1,3}",
            to in "[a-cx]{0,3}",
            i in any::<bool>(),
        ) {
            let flags = if i { "i" } else { "" };
            let literal = RenameMask::new(&format!("s/{from}/{to}/{flags}"), CaseMode::None);
            let regex = RenameMask::new(&format!("s/{from}/{to}/r{flags}"), CaseMode::None);
            prop_assert_eq!(regex.apply(&name, 1), literal.apply(&name, 1));
        }

        /// `\U$0` and `\L$0` are the standard conversions of the whole name.
        #[test]
        fn regex_case_of_the_whole_name(name in "\\PC{1,24}") {
            let upper = RenameMask::new(r"s/^.*$/\U$0/r", CaseMode::None);
            let lower = RenameMask::new(r"s/^.*$/\L$0/r", CaseMode::None);
            prop_assert_eq!(upper.apply(&name, 1), name.to_uppercase());
            prop_assert_eq!(lower.apply(&name, 1), name.to_lowercase());
        }

        /// Whatever replacement is typed, parsing and applying never panic.
        #[test]
        fn regex_never_panics(
            from in prop_oneof![
                Just(r"(\w+)".to_string()),
                Just(r"(?<a>.)(?<b>.)?".to_string()),
                "[a-c()?*+.\\\\]{0,6}",
            ],
            to in "[$\\\\{}0-9abULEulx_]{0,12}",
            name in "\\PC{0,24}",
        ) {
            if let Ok(mask) = RenameMask::parse(&format!("s/{from}/{to}/r"), CaseMode::None) {
                let _ = mask.apply(&name, 1);
            }
        }

        /// Ignoring case, with characters whose lowercase takes a different
        /// number of bytes (the Kelvin sign is a `k`, `ſ` an `s`).
        #[test]
        fn ignoring_case_never_panics(
            name in mask(),
            from in "[kKsSiI\u{212a}\u{17f}İß]{1,3}",
            to in "[x]{0,2}",
        ) {
            let _ = RenameMask::new(&format!("s/{from}/{to}/i"), CaseMode::None).apply(&name, 1);
        }

        /// With `i`, the same ignoring case (for ASCII, against a simple
        /// reference: replace in the lowercase copy, at the same positions).
        #[test]
        fn search_and_replace_ignoring_case(
            name in "[a-cA-C.]{0,16}",
            from in "[a-cA-C]{1,3}",
            to in "[x]{0,2}",
        ) {
            let mask = RenameMask::new(&format!("s/{from}/{to}/i"), CaseMode::None);
            let (lower, from_lower) = (name.to_lowercase(), from.to_lowercase());
            let mut expected = String::new();
            let mut i = 0;
            while i < name.len() {
                if lower[i..].starts_with(&from_lower) {
                    expected.push_str(&to);
                    i += from.len();
                } else {
                    expected.push_str(&name[i..i + 1]);
                    i += 1;
                }
            }
            prop_assert_eq!(mask.apply(&name, 1), expected);
        }
    }

    #[test]
    fn test_wildcard_extension_change() {
        let mask = RenameMask::new("*.jpg", CaseMode::None);
        assert_eq!(mask.apply("photo.jpeg", 1), "photo.jpg");
        assert_eq!(mask.apply("document.txt", 1), "document.jpg");
        assert_eq!(mask.apply("archive.tar.gz", 1), "archive.tar.jpg");
    }

    #[test]
    fn test_wildcard_prefix_and_suffix() {
        let m1 = RenameMask::new("new_*.*", CaseMode::None);
        assert_eq!(m1.apply("file.txt", 1), "new_file.txt");

        let m2 = RenameMask::new("*_backup.*", CaseMode::None);
        assert_eq!(m2.apply("file.txt", 1), "file_backup.txt");

        let m3 = RenameMask::new("prefix_*_suffix.*", CaseMode::None);
        assert_eq!(m3.apply("file.txt", 1), "prefix_file_suffix.txt");
    }

    #[test]
    fn test_strip_extension() {
        let mask = RenameMask::new("*.", CaseMode::None);
        assert_eq!(mask.apply("photo.jpeg", 1), "photo");
    }

    #[test]
    fn test_incremental_counters() {
        let m1 = RenameMask::new("photo_%03d.*", CaseMode::None);
        assert_eq!(m1.apply("a.jpg", 1), "photo_001.jpg");
        assert_eq!(m1.apply("b.jpg", 42), "photo_042.jpg");

        let m2 = RenameMask::new("file_%02d.txt", CaseMode::None);
        assert_eq!(m2.apply("old.dat", 5), "file_05.txt");

        let m3 = RenameMask::new("item_<01>.*", CaseMode::None);
        assert_eq!(m3.apply("x.png", 3), "item_03.png");

        let m4 = RenameMask::new("item_<#>.*", CaseMode::None);
        assert_eq!(m4.apply("x.png", 3), "item_3.png");
    }

    #[test]
    fn test_search_and_replace() {
        let m1 = RenameMask::new("s/jpeg/jpg/", CaseMode::None);
        assert_eq!(m1.apply("my_photo.jpeg", 1), "my_photo.jpg");

        let m2 = RenameMask::new("s/ /_/g", CaseMode::None);
        assert_eq!(m2.apply("my holiday trip.png", 1), "my_holiday_trip.png");

        let m3 = RenameMask::new("s/JPEG/jpg/i", CaseMode::None);
        assert_eq!(m3.apply("MY_PHOTO.JPEG", 1), "MY_PHOTO.jpg");

        let m4 = RenameMask::new("/IMG_/pic_/", CaseMode::None);
        assert_eq!(m4.apply("IMG_001.jpg", 1), "pic_001.jpg");
    }

    #[test]
    fn test_case_conversions() {
        // Whole file lowercase via mask token
        let m1 = RenameMask::new("<lower>", CaseMode::None);
        assert_eq!(m1.apply("DSC_001.JPG", 1), "dsc_001.jpg");

        // Whole file uppercase via mask token
        let m2 = RenameMask::new("<upper>", CaseMode::None);
        assert_eq!(m2.apply("file_name.txt", 1), "FILE_NAME.TXT");

        // Lowercase extension only via mask
        let m3 = RenameMask::new("*.<lower>", CaseMode::None);
        assert_eq!(m3.apply("DSC_001.JPG", 1), "DSC_001.jpg");

        // Title case via mask token
        let m4 = RenameMask::new("<title>", CaseMode::None);
        assert_eq!(m4.apply("hello_world.txt", 1), "Hello_World.Txt");

        // CaseMode toggle (e.g. F2 in prompt)
        let m5 = RenameMask::new("new_*.*", CaseMode::Lower);
        assert_eq!(m5.apply("FILE_NAME.TXT", 1), "new_file_name.txt");
    }

    fn rx(mask: &str, name: &str) -> String {
        RenameMask::parse(mask, CaseMode::None)
            .unwrap_or_else(|e| panic!("{mask}: {e}"))
            .apply(name, 1)
    }

    fn rx_err(mask: &str) -> String {
        RenameMask::parse(mask, CaseMode::None).expect_err(mask)
    }

    #[test]
    fn regex_flag_swaps_groups() {
        assert_eq!(
            rx(r"s/(\d+)-(\w+)/$2-$1/r", "2026-report.txt"),
            "report-2026.txt"
        );
        assert_eq!(
            rx(r"/(\d+)-(\w+)/${2}-${1}/r", "2026-report.txt"),
            "report-2026.txt"
        );
        // Every match, as the literal replace does.
        assert_eq!(rx(r"s/\s+/_/r", "a  b \tc.txt"), "a_b_c.txt");
        assert_eq!(rx(r"s/^/new_/r", "a.txt"), "new_a.txt");
        // `$0` is the whole match, `$$` a dollar, `$1a` group 1 and an `a`.
        assert_eq!(rx(r"s/\d+/<$0>/r", "v12.txt"), "v<12>.txt");
        assert_eq!(rx(r"s/(\d)/$$$1a/r", "v1.txt"), "v$1a.txt");
        // `i` and `r` together.
        assert_eq!(rx(r"s/JPE?G$/jpg/ri", "a.Jpeg"), "a.jpg");
        assert_eq!(rx(r"s/JPE?G$/jpg/r", "a.Jpeg"), "a.Jpeg");
        // The case mode (F2) applies over the result.
        let mask = RenameMask::parse(r"s/^(\w+)/x_$1/r", CaseMode::Upper).unwrap();
        assert_eq!(mask.apply("a.txt", 1), "X_A.TXT");
    }

    #[test]
    fn regex_named_groups() {
        let mask = r"s/(?<year>\d{4})-(?<month>\d\d)/${month}.${year}/r";
        assert_eq!(rx(mask, "2026-11 notes.md"), "11.2026 notes.md");
        assert_eq!(rx(r"s/(?P<n>\d+)/#$n/r", "a7.txt"), "a#7.txt");
        assert!(rx_err(r"s/(?<n>\d+)/${m}/r").contains("no group named 'm'"));
    }

    #[test]
    fn regex_case_escapes() {
        assert_eq!(rx(r"s/^(\w+)/\U$1/r", "photo.jpg"), "PHOTO.jpg");
        assert_eq!(
            rx(r"s/^(\w+)\.(\w+)$/\U$1\E.$2/r", "photo.jpg"),
            "PHOTO.jpg"
        );
        assert_eq!(rx(r"s/^(\w+)/\L$1/r", "PHOTO.JPG"), "photo.JPG");
        assert_eq!(rx(r"s/^(\w)/\u$1/r", "photo.jpg"), "Photo.jpg");
        assert_eq!(rx(r"s/^(\w+)/\l$1/r", "PHOTO.jpg"), "pHOTO.jpg");
        // `\u` then `\L`: the first upper, the rest lower.
        assert_eq!(rx(r"s/^(\w+)/\u\L$1/r", "pHOTO.jpg"), "Photo.jpg");
        // Text counts too; ß is SS in uppercase.
        assert_eq!(rx(r"s/^/\Uabc_/r", "x"), "ABC_x");
        assert_eq!(rx(r"s/(stra)(ße)/$1\U$2/r", "straße"), "straSSE");
        // Other backslashes are just that.
        assert_eq!(rx(r"s/a/\x\\\$/r", "a"), r"\x\$");
    }

    #[test]
    fn regex_errors_say_what() {
        let e = rx_err(r"s/(\d+/x/r");
        assert!(
            e.contains("column 1") && e.contains("unclosed group"),
            "{e}"
        );
        assert!(rx_err(r"s/a{2,1}/x/r").contains("column"));
        assert!(rx_err(r"s/\w{1000}{1000}/x/r").contains("too big"));
        assert_eq!(rx_err(r"s/(a)/$2/r"), "no group $2: the regex has 1 group");
        assert!(rx_err(r"s/a/$9999999999999999999999/r").contains("no group"));
        assert!(rx_err(r"s/a/${1/r").contains("no closing }"));
        // `new` keeps the names, rather than renaming them by half a mask.
        assert_eq!(
            RenameMask::new(r"s/(/x/r", CaseMode::Upper).apply("a.txt", 1),
            "a.txt"
        );
    }

    #[test]
    fn without_r_it_stays_literal() {
        // `.` is a dot, `$1` is text.
        assert_eq!(rx("s/./_/", "a.b.txt"), "a_b_txt");
        assert_eq!(rx("s/(a)/$1/", "(a).txt"), "$1.txt");
        assert_eq!(rx("s/(a/x/", "(a.txt"), "x.txt");
    }

    #[test]
    fn regex_results_that_are_no_name() {
        // The engine gives them; the batch Rename refuses them (check_name).
        assert_eq!(rx(r"s/.*//r", "a.txt"), "");
        assert!(crate::ops::check_name(&rx(r"s/.*//r", "a.txt")).is_err());
        // A `/` cannot come from the mask (it splits it) nor from a name.
        assert_eq!(rx(r"s/_/\//r", "a_b"), r"a\b");
    }

    #[test]
    fn test_question_mark_stencil() {
        let mask = RenameMask::new("???_v2.*", CaseMode::None);
        assert_eq!(mask.apply("document.txt", 1), "doc_v2.txt");
    }
}
