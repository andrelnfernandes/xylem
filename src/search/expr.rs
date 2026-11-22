//! The terms of Ctrl+S joined by AND, OR and NOT: `fatura AND 2026`,
//! `jpg OR png`, `"nota fiscal" AND NOT (rascunho OR cópia)`, `x'4D5A'`.
//!
//! The same logic as `contains()` in W: `a AND b` here and
//! `contains('a') AND contains('b')` there keep the same files. NOT binds
//! tighter than AND, and AND tighter than OR, as in SQL.
//!
//! Words with nothing between them are one term, as typed (`nota fiscal`
//! is the text "nota fiscal"), so a search from before the expressions
//! still means the same. The operators are words in capitals; `and`, `or`
//! and `not` are words like any other.

/// The longest expression read, as for W.
const MAX_INPUT: usize = 1024;
/// The most terms, each once.
pub const MAX_TERMS: usize = 16;
/// How deep the parentheses and NOTs go.
const MAX_DEPTH: usize = 32;

/// What a term looks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    /// Text, as typed (a `*` inside is the wildcard, as before).
    Text(String),
    /// `x'4D5A'`: the bytes as they are.
    Bytes(Vec<u8>),
}

/// The expression over the terms (each by its place in [`Query::terms`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Term(usize),
    And(Box<Node>, Box<Node>),
    Or(Box<Node>, Box<Node>),
    Not(Box<Node>),
}

impl Node {
    /// Yes, no, or `None` while a term not decided yet can still change it:
    /// an AND that has failed or an OR that has succeeded is decided early.
    pub fn decide(&self, found: &[Option<bool>]) -> Option<bool> {
        match self {
            Node::Term(i) => found.get(*i).copied().flatten(),
            Node::And(a, b) => match (a.decide(found), b.decide(found)) {
                (Some(false), _) | (_, Some(false)) => Some(false),
                (Some(true), Some(true)) => Some(true),
                _ => None,
            },
            Node::Or(a, b) => match (a.decide(found), b.decide(found)) {
                (Some(true), _) | (_, Some(true)) => Some(true),
                (Some(false), Some(false)) => Some(false),
                _ => None,
            },
            Node::Not(a) => a.decide(found).map(|b| !b),
        }
    }
}

/// A search read: the expression, and its terms each once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub node: Node,
    pub terms: Vec<Term>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    /// Bare words, with where they start and end in the input.
    Word(usize, usize),
    Quoted(String),
    Hex(Vec<u8>),
    And,
    Or,
    Not,
    Open,
    Close,
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        match b {
            b'(' => {
                tokens.push(Token::Open);
                i += 1;
            }
            b')' => {
                tokens.push(Token::Close);
                i += 1;
            }
            b'"' => {
                // `""` inside is one quote, as in SQL.
                let mut text = String::new();
                let mut j = i + 1;
                loop {
                    let Some(rel) = input[j..].find('"') else {
                        return Err("A quote is not closed: \"nota fiscal\"".into());
                    };
                    text.push_str(&input[j..j + rel]);
                    j += rel + 1;
                    if bytes.get(j) == Some(&b'"') {
                        text.push('"');
                        j += 1;
                    } else {
                        break;
                    }
                }
                if text.is_empty() {
                    return Err("\"\" has nothing to look for".into());
                }
                tokens.push(Token::Quoted(text));
                i = j;
            }
            b'x' | b'X' if bytes.get(i + 1) == Some(&b'\'') => {
                let start = i + 2;
                let Some(rel) = input[start..].find('\'') else {
                    return Err("x'...' is not closed: x'4D5A'".into());
                };
                tokens.push(Token::Hex(hex_bytes(&input[start..start + rel])?));
                i = start + rel + 1;
            }
            _ => {
                let start = i;
                while i < bytes.len()
                    && !bytes[i].is_ascii_whitespace()
                    && !matches!(bytes[i], b'(' | b')' | b'"')
                {
                    i += 1;
                }
                tokens.push(match &input[start..i] {
                    "AND" => Token::And,
                    "OR" => Token::Or,
                    "NOT" => Token::Not,
                    _ => Token::Word(start, i),
                });
            }
        }
    }
    Ok(tokens)
}

/// `4D5A` or `4d 5a`: pairs of hex digits.
fn hex_bytes(text: &str) -> Result<Vec<u8>, String> {
    let digits: Vec<u8> = text.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if digits.is_empty() {
        return Err("x'' has nothing to look for".into());
    }
    if !digits.len().is_multiple_of(2) || !digits.iter().all(u8::is_ascii_hexdigit) {
        return Err(format!("x'{text}' is not pairs of hex digits: x'4D5A'"));
    }
    Ok(digits
        .chunks(2)
        .map(|p| {
            let s = std::str::from_utf8(p).unwrap_or("00");
            u8::from_str_radix(s, 16).unwrap_or(0)
        })
        .collect())
}

struct Parser<'a> {
    input: &'a str,
    tokens: Vec<Token>,
    pos: usize,
    terms: Vec<Term>,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn or(&mut self, depth: usize) -> Result<Node, String> {
        let mut left = self.and(depth)?;
        while self.peek() == Some(&Token::Or) {
            self.pos += 1;
            let right = self.and(depth)?;
            left = Node::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn and(&mut self, depth: usize) -> Result<Node, String> {
        let mut left = self.not(depth)?;
        while self.peek() == Some(&Token::And) {
            self.pos += 1;
            let right = self.not(depth)?;
            left = Node::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn not(&mut self, depth: usize) -> Result<Node, String> {
        if depth > MAX_DEPTH {
            return Err("Too deep: at most 32 parentheses and NOTs inside each other".into());
        }
        if self.peek() == Some(&Token::Not) {
            self.pos += 1;
            return Ok(Node::Not(Box::new(self.not(depth + 1)?)));
        }
        self.primary(depth)
    }

    fn primary(&mut self, depth: usize) -> Result<Node, String> {
        let term = match self.tokens.get(self.pos).cloned() {
            Some(Token::Open) => {
                self.pos += 1;
                let inner = self.or(depth + 1)?;
                if self.peek() != Some(&Token::Close) {
                    return Err("A parenthesis is not closed".into());
                }
                self.pos += 1;
                return Ok(inner);
            }
            Some(Token::Word(start, mut end)) => {
                self.pos += 1;
                // The words up to the next operator are one term, as typed.
                while let Some(&Token::Word(_, e)) = self.peek() {
                    end = e;
                    self.pos += 1;
                }
                Term::Text(self.input[start..end].to_string())
            }
            Some(Token::Quoted(text)) => {
                self.pos += 1;
                Term::Text(text)
            }
            Some(Token::Hex(bytes)) => {
                self.pos += 1;
                Term::Bytes(bytes)
            }
            Some(Token::Close) => return Err("A ) with no ( before it".into()),
            Some(Token::And | Token::Or) | None => {
                return Err("A term is missing: fatura AND 2026".into());
            }
            Some(Token::Not) => unreachable!("read by not()"),
        };
        let i = match self.terms.iter().position(|t| *t == term) {
            Some(i) => i,
            None if self.terms.len() >= MAX_TERMS => {
                return Err(format!("Too many terms: at most {MAX_TERMS}"));
            }
            None => {
                self.terms.push(term);
                self.terms.len() - 1
            }
        };
        Ok(Node::Term(i))
    }
}

/// Reads a search; an error says what is wrong.
pub fn parse(input: &str) -> Result<Query, String> {
    if input.len() > MAX_INPUT {
        return Err(format!("Too long: at most {MAX_INPUT} bytes"));
    }
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err("Empty search".into());
    }
    let mut p = Parser {
        input,
        tokens,
        pos: 0,
        terms: Vec::new(),
    };
    let node = p.or(0)?;
    if p.pos < p.tokens.len() {
        return Err(match p.tokens[p.pos] {
            Token::Close => "A ) with no ( before it".into(),
            _ => "Two terms with nothing between them: join them with AND or OR, \
                  or put the text in quotes"
                .into(),
        });
    }
    Ok(Query {
        node,
        terms: p.terms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(s: &str) -> Term {
        Term::Text(s.into())
    }

    #[test]
    fn words_together_are_one_term_as_typed() {
        let q = parse("  nota   fiscal ").unwrap();
        assert_eq!(q.terms, [text("nota   fiscal")]);
        assert_eq!(q.node, Node::Term(0));
        // Lower case is a word.
        assert_eq!(
            parse("salt and pepper").unwrap().terms,
            [text("salt and pepper")]
        );
    }

    #[test]
    fn precedence_is_not_then_and_then_or() {
        let q = parse("a OR b AND NOT c").unwrap();
        let t = |i| Box::new(Node::Term(i));
        assert_eq!(
            q.node,
            Node::Or(t(0), Box::new(Node::And(t(1), Box::new(Node::Not(t(2))))))
        );
        let q = parse("(a OR b) AND c").unwrap();
        assert_eq!(q.node, Node::And(Box::new(Node::Or(t(0), t(1))), t(2)));
    }

    #[test]
    fn quotes_hex_and_repeated_terms() {
        let q = parse(r#""a AND b" OR x'4d 5A' OR "say ""hi""" OR "a AND b""#).unwrap();
        assert_eq!(
            q.terms,
            [
                text("a AND b"),
                Term::Bytes(vec![0x4d, 0x5a]),
                text("say \"hi\"")
            ]
        );
        // An x not followed by a quote is a word.
        assert_eq!(parse("xyz").unwrap().terms, [text("xyz")]);
    }

    #[test]
    fn errors_say_what_is_wrong() {
        for (bad, says) in [
            ("", "Empty"),
            ("\"\"", "nothing to look for"),
            ("a AND \"\"", "nothing to look for"),
            ("x''", "nothing to look for"),
            ("x'4D5'", "pairs of hex"),
            ("x'ZZ'", "pairs of hex"),
            ("\"open", "not closed"),
            ("(a OR b", "not closed"),
            ("a)", "no ("),
            ("a AND", "missing"),
            ("OR a", "missing"),
            ("a \"b\"", "nothing between"),
        ] {
            let err = parse(bad).unwrap_err();
            assert!(err.contains(says), "{bad}: {err}");
        }
        let many: Vec<String> = (0..17).map(|i| format!("t{i}")).collect();
        assert!(
            parse(&many.join(" OR "))
                .unwrap_err()
                .contains("at most 16")
        );
        assert!(parse(&many[..16].join(" OR ")).is_ok());
        let deep = format!("{}a{}", "(".repeat(40), ")".repeat(40));
        assert!(parse(&deep).unwrap_err().contains("Too deep"));
    }

    #[test]
    fn decides_early() {
        let q = parse("a AND b").unwrap();
        assert_eq!(q.node.decide(&[Some(false), None]), Some(false));
        assert_eq!(q.node.decide(&[Some(true), None]), None);
        let q = parse("a OR NOT b").unwrap();
        assert_eq!(q.node.decide(&[None, Some(false)]), Some(true));
        assert_eq!(q.node.decide(&[None, Some(true)]), None);
    }
}
