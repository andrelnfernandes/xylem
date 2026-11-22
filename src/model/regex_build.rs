//! Regular expressions typed by the user (Filespec, batch Rename).
//!
//! The `regex` crate runs in linear time, so a hostile expression cannot
//! freeze the list; the size of the compiled expression has a limit too.
//! A syntax error says, in one line, the column where it went wrong.

use regex::bytes;

/// Room for any reasonable expression over file names: Unicode classes are
/// big (`\w{100}`, ignoring case, takes about 8 MiB compiled).
pub const SIZE_LIMIT: usize = 8 << 20;

/// An expression over text (the batch Rename works on UTF-8 names).
pub fn build(pattern: &str, case_insensitive: bool) -> Result<regex::Regex, String> {
    regex::RegexBuilder::new(pattern)
        .case_insensitive(case_insensitive)
        .size_limit(SIZE_LIMIT)
        .build()
        .map_err(|e| describe(&e, pattern, case_insensitive))
}

/// An expression over the bytes of a name (the Filespec sees names that are
/// not UTF-8 too).
pub fn build_bytes(pattern: &str, case_insensitive: bool) -> Result<bytes::Regex, String> {
    bytes::RegexBuilder::new(pattern)
        .case_insensitive(case_insensitive)
        .size_limit(SIZE_LIMIT)
        .build()
        .map_err(|e| describe(&e, pattern, case_insensitive))
}

/// The reason in one line, with the column (counted in characters, from 1).
fn describe(error: &regex::Error, pattern: &str, case_insensitive: bool) -> String {
    if let regex::Error::CompiledTooBig(limit) = error {
        return format!(
            "regex /{pattern}/ is too big (limit {} KiB compiled)",
            limit / 1024
        );
    }
    let parsed = regex_syntax::ParserBuilder::new()
        .case_insensitive(case_insensitive)
        .build()
        .parse(pattern);
    let found = match &parsed {
        Err(regex_syntax::Error::Parse(e)) => Some((e.span().start.offset, e.kind().to_string())),
        Err(regex_syntax::Error::Translate(e)) => {
            Some((e.span().start.offset, e.kind().to_string()))
        }
        _ => None,
    };
    match found {
        Some((offset, reason)) => {
            let column = pattern.get(..offset).map_or(offset, |s| s.chars().count()) + 1;
            format!("invalid regex /{pattern}/ at column {column}: {reason}")
        }
        None => {
            let text = error.to_string();
            let last = text.lines().last().unwrap_or_default();
            let reason = last.strip_prefix("error: ").unwrap_or(last);
            format!("invalid regex /{pattern}/: {reason}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn says_where_the_error_is() {
        let e = build("ab(cd", false).unwrap_err();
        assert!(e.contains("column 3"), "{e}");
        assert!(e.contains("unclosed group"), "{e}");
        let e = build("ção[", false).unwrap_err();
        assert!(e.contains("column 4"), "counted in characters: {e}");
        let e = build_bytes("a{2,1}", false).unwrap_err();
        assert!(e.starts_with("invalid regex /a{2,1}/ at column"), "{e}");
        assert!(!e.contains('\n'), "one line: {e}");
    }

    #[test]
    fn refuses_a_regex_too_big() {
        let e = build(r"\w{1000}{1000}", false).unwrap_err();
        assert!(e.contains("too big"), "{e}");
        let e = build_bytes(r"(\w{500}){500}", true).unwrap_err();
        assert!(e.contains("too big"), "{e}");
        // Reasonable expressions fit.
        assert!(build(r"^report-\d{4}-(\w+)\.(pdf|txt)$", true).is_ok());
        assert!(build_bytes(r"\w{60}", true).is_ok());
    }

    #[test]
    fn escaping_a_slash_is_allowed() {
        assert!(build(r"a\/b", false).is_ok());
    }
}
