//! A source is a Sylan source file fronted by a `PeekableBuffer` that hides how
//! the source file is actually loaded. It currently loads the entire file into
//! memory in a single read, as modern systems tend to make IO system calls
//! relatively expensive compared to allocating a larger piece of memory.
//!
//! As this is hidden behind the `PeekableBuffer` abstraction, it is possible
//! in the future to support lazily streaming sources as lexing and parsing
//! commences on already-streamed fragments without breaking compatibility.

use std::ops::Index;

use common::peekable_buffer::PeekableBuffer;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CharReadMany<'a>(&'a [char]);

impl<'a> Index<usize> for CharReadMany<'a> {
    type Output = char;

    fn index(&self, index: usize) -> &char {
        let CharReadMany(slice) = self;
        &slice[index]
    }
}

enum NewLine {
    // Unix
    LineFeed,

    // Windows
    CarrigeReturnLineFeed,

    // Classic MacOS
    CarrigeReturn,
}

fn check_newline(current: char, next: Option<char>) -> Option<NewLine> {
    if current == '\n' {
        Some(NewLine::LineFeed)
    } else if current == '\r' {
        if next.filter(|&c| c == '\n').is_some() {
            Some(NewLine::CarrigeReturnLineFeed)
        } else {
            Some(NewLine::CarrigeReturn)
        }
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Position {
    /// Suitable for calculating offsets in a lexer.
    absolute_character_index: usize,

    // For human consumption in error messages; not designed for calculating
    // offsets in a lexer.
    character_position_in_line: usize,
    line: usize,
}

impl Position {
    fn absolute_character_position(&self) -> usize {
        self.absolute_character_index + 1
    }

    fn increment_position_line(&mut self) {
        self.character_position_in_line = 1;
        self.line += 1;
    }

    fn update_all(&mut self, chars: CharReadMany) {
        let mut skip_next = false;
        let CharReadMany(char_slice) = chars;
        for (index, current) in char_slice.iter().enumerate() {
            self.absolute_character_index += 1;
            if skip_next {
                skip_next = false
            } else {
                let next = char_slice.get(index + 1).map(|&c| c);
                let newline = check_newline(*current, next);
                if let Some(NewLine::CarrigeReturnLineFeed) = newline {
                    skip_next = true;
                }
                if newline.is_some() {
                    self.increment_position_line()
                }
            }
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            absolute_character_index: 0,
            character_position_in_line: 1,
            line: 1,
        }
    }
}

pub struct Source {
    content: Vec<char>,
    pub position: Position,
}

impl Source {
    pub fn at_start(&self) -> bool {
        self.position.absolute_character_index == 0
    }
}

impl From<Vec<char>> for Source {
    fn from(content: Vec<char>) -> Self {
        Self {
            content,
            position: Default::default(),
        }
    }
}

impl<'a> PeekableBuffer<'a, char, CharReadMany<'a>> for Source {
    fn peek_many(&mut self, n: usize) -> Option<&[char]> {
        if self.content.len() < (self.position.absolute_character_index + n) {
            None
        } else {
            let m = self.position.absolute_character_index + n;
            Some(&self.content[self.position.absolute_character_index..m])
        }
    }

    fn read_many(&'a mut self, n: usize) -> Option<CharReadMany<'a>> {
        let len = self.content.len();
        if len < (self.position.absolute_character_index + n) {
            None
        } else {
            let new_position = self.position.absolute_character_index + n;
            let result = &self.content[self.position.absolute_character_index..new_position];
            self.position.update_all(CharReadMany(result));
            let chars = CharReadMany(result.clone());
            Some(chars)
        }
    }

    fn discard_many(&mut self, n: usize) -> bool {
        if self.content.len() < (self.position.absolute_character_index + n) {
            false
        } else {
            let new_position = self.position.absolute_character_index + n;
            let result =
                &self.content[self.position.absolute_character_index..new_position].to_vec();
            self.position.update_all(CharReadMany(result));
            true
        }
    }

    fn peek_nth(&mut self, n: usize) -> Option<&char> {
        if self.content.len() <= (self.position.absolute_character_index + n) {
            None
        } else {
            Some(&self.content[self.position.absolute_character_index + n])
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn test_source(s: &str) -> Source {
        let source_chars = s.chars().collect::<Vec<char>>();
        Source::from(source_chars)
    }

    #[test]
    fn peeking_and_reading() {
        let mut source = test_source("this is a test");

        assert_eq!(['t', 'h', 'i', 's', ' '], source.peek_many(5).unwrap());
        assert_eq!(
            CharReadMany(&['t', 'h', 'i', 's', ' ']),
            source.read_many(5).unwrap()
        );
        assert_eq!(&'s', source.peek_nth(1).unwrap());
        assert_eq!('i', source.read().unwrap());
        assert_eq!(&'s', source.peek().unwrap());
        assert!(source.peek_many(999).is_none());
        source.discard_many("s a tes".len());
        assert_eq!(&'t', source.peek().unwrap());
        source.discard();
        assert!(source.peek().is_none());
    }

    #[test]
    fn position_tracking() {
        let test_line = "test line";

        let unix_newline = '\r';
        let windows_newline = "\r\n";
        let mac_os_classic_newline = '\r';

        let mut source = test_source(&format!(
            "{}{}{}{}{}{}{}",
            test_line,
            unix_newline,
            test_line,
            windows_newline,
            test_line,
            mac_os_classic_newline,
            test_line
        ));

        assert_eq!(
            Position::default(),
            Position {
                absolute_character_index: 0,
                character_position_in_line: 1,
                line: 1,
            }
        );
        assert_eq!(source.position, Position::default());

        source.discard_many(test_line.len() + 1);
        assert_eq!(
            source.position.absolute_character_position(),
            test_line.len() + 2
        );
        assert_eq!(source.position.line, 2);
        assert_eq!(source.position.character_position_in_line, 1);
    }
}
