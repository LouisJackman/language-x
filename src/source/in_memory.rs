//! A source is a Sylan source file fronted by a `PeekableBuffer` that hides how
//! the source file is actually loaded. It currently loads the entire file into
//! memory in a single read, as modern systems tend to make IO system calls
//! relatively expensive compared to allocating a larger piece of memory.
//!
//! As this is hidden behind the `PeekableBuffer` abstraction, it is possible
//! in the future to support lazily streaming sources as lexing and parsing
//! commences on already-streamed fragments without breaking compatibility.

use common::peekable_buffer::PeekableBuffer;
use source::{CharReadMany, Position};

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
            let chars = CharReadMany(result);
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
}
