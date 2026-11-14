//! The lines of a text, kept in one block.
//!
//! A `Vec<String>` costs 24 bytes and one heap block per line: a file of 2.5
//! million lines made as many small blocks, about 40% over the text itself,
//! and the allocator did not hand them back to the system when the viewer
//! closed. Here the text is one `String` and each line is where it ends, so
//! closing the viewer frees two large blocks.

use std::ops::Index;

/// What each line costs besides its bytes.
pub const LINE_COST: usize = size_of::<usize>();

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Lines {
    /// Every line, one after the other, with nothing between them.
    text: String,
    /// Where each line ends in `text`.
    ends: Vec<usize>,
}

impl Lines {
    /// Room for `bytes` of text in `lines` lines.
    pub fn with_capacity(bytes: usize, lines: usize) -> Self {
        Lines {
            text: String::with_capacity(bytes),
            ends: Vec::with_capacity(lines),
        }
    }

    pub fn len(&self) -> usize {
        self.ends.len()
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.ends.is_empty()
    }

    pub fn get(&self, i: usize) -> Option<&str> {
        let end = *self.ends.get(i)?;
        let start = if i == 0 { 0 } else { self.ends[i - 1] };
        Some(&self.text[start..end])
    }

    pub fn first(&self) -> Option<&str> {
        self.get(0)
    }

    #[cfg(test)]
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        (0..self.len()).map(|i| &self[i])
    }

    /// Adds `c` to the line being written (see `end_line`).
    pub fn push(&mut self, c: char) {
        self.text.push(c);
    }

    /// Adds `n` spaces to the line being written.
    pub fn push_spaces(&mut self, n: usize) {
        self.text.extend(std::iter::repeat_n(' ', n));
    }

    /// The bytes of the line being written.
    pub fn open_len(&self) -> usize {
        self.text.len() - self.ends.last().copied().unwrap_or(0)
    }

    /// Closes the line being written, which may be empty.
    pub fn end_line(&mut self) {
        self.ends.push(self.text.len());
    }

    /// Gives back the room that was not used.
    pub fn shrink_to_fit(&mut self) {
        self.text.shrink_to_fit();
        self.ends.shrink_to_fit();
    }
}

impl Index<usize> for Lines {
    type Output = str;

    fn index(&self, i: usize) -> &str {
        self.get(i).expect("a line of the text")
    }
}

/// Lines to read by their number: the viewer's, or a plain list.
pub trait LineSource {
    fn count(&self) -> usize;
    fn line(&self, i: usize) -> &str;
}

impl LineSource for Lines {
    fn count(&self) -> usize {
        self.len()
    }

    fn line(&self, i: usize) -> &str {
        &self[i]
    }
}

impl LineSource for [String] {
    fn count(&self) -> usize {
        self.len()
    }

    fn line(&self, i: usize) -> &str {
        &self[i]
    }
}

impl<S: AsRef<str>> FromIterator<S> for Lines {
    fn from_iter<I: IntoIterator<Item = S>>(iter: I) -> Self {
        let mut lines = Lines::default();
        for line in iter {
            lines.text.push_str(line.as_ref());
            lines.end_line();
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_are_read_back_as_written() {
        let mut lines = Lines::default();
        assert!(lines.is_empty());
        assert_eq!(lines.iter().count(), 0);
        assert_eq!(lines.first(), None);
        lines.push('a');
        lines.push_spaces(2);
        lines.push('é');
        assert_eq!(lines.open_len(), 5);
        lines.end_line();
        lines.end_line();
        lines.push('z');
        assert_eq!(lines.open_len(), 1);
        lines.end_line();
        assert_eq!(lines.len(), 3);
        assert_eq!(&lines[0], "a  é");
        assert_eq!(&lines[1], "");
        assert_eq!(&lines[2], "z");
        assert_eq!(lines.get(3), None);
        assert_eq!(lines, ["a  é", "", "z"].into_iter().collect());
    }
}
