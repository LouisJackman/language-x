//! # Sylan's Sourcing
//!
//! A source is a Sylan source file fronted by a `PeekableBuffer` that hides how
//! the source file is actually loaded. It currently loads the entire file into
//! memory in a single read, as modern systems tend to make IO system calls
//! relatively expensive compared to allocating a larger piece of memory.
//!
//! As this is hidden behind the `PeekableBuffer` abstraction, it is possible
//! in the future to support lazily streaming sources as lexing and parsing
//! commences on already-streamed fragments without breaking compatibility.

use std::ops::Index;

pub mod in_memory;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    fn character_position(&self) -> usize {
        self.absolute_character_index + 1
    }

    fn increment_position_line(&mut self) {
        self.character_position_in_line = 1;
        self.line += 1;
    }

    fn update_all(&mut self, chars: CharReadMany<'_>) {
        let mut skip_next = false;
        let CharReadMany(char_slice) = chars;
        for (index, current) in char_slice.iter().enumerate() {
            self.absolute_character_index += 1;
            if skip_next {
                skip_next = false
            } else {
                let next = char_slice.get(index + 1).cloned();
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

#[cfg(test)]
mod tests {
    use crate::common::peekable_buffer::PeekableBuffer;
    use crate::source::in_memory::Source;

    use super::*;

    fn test_source(s: &str) -> Source {
        let source_chars = s.chars().collect::<Vec<char>>();
        Source::from(source_chars)
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

        // Test Unix newline tracking.
        source.discard_many(test_line.len() + 1);
        assert_eq!(
            source.position.absolute_character_index,
            test_line.len() + 1
        );
        assert_eq!(source.position.line, 2);
        assert_eq!(source.position.character_position_in_line, 1);

        // Test Windows newline tracking.
        source.discard_many(test_line.len() + 2);
        assert_eq!(
            source.position.absolute_character_index,
            (test_line.len() * 2) + 3
        );
        assert_eq!(source.position.line, 3);
        assert_eq!(source.position.character_position_in_line, 1);

        // Test MacOS classic newline tracking.
        source.discard_many(test_line.len() + 1);
        assert_eq!(
            source.position.absolute_character_index,
            (test_line.len() * 3) + 4
        );
        assert_eq!(source.position.line, 4);
        assert_eq!(source.position.character_position_in_line, 1);

        assert_eq!(
            source.position.absolute_character_index + 1,
            source.position.character_position()
        );
    }
}
