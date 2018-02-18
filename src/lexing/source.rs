use peekable_buffer::PeekableBuffer;

pub struct Source {
    content: Vec<char>,
    pub position: usize,
}

impl From<Vec<char>> for Source {
    fn from(content: Vec<char>) -> Self {
        Self {
            content,
            position: 0,
        }
    }
}

impl PeekableBuffer<char> for Source {

    fn peek_many(&mut self, n: usize) -> Option<&[char]> {
        if self.content.len() < (self.position + n) {
            None
        } else {
            let m = self.position + n;
            Some(&self.content[self.position..m])
        }
    }

    fn read_many(&mut self, n: usize) -> Option<&[char]> {
        if self.content.len() < (self.position + n) {
            None
        } else {
            let new_position = self.position + n;
            let result = &self.content[self.position..new_position];
            self.position = new_position;
            Some(result)
        }
    }

    fn peek_nth(&mut self, n: usize) -> Option<&char> {
        if self.content.len() <= (self.position + n) {
            None
        } else {
            Some(&self.content[self.position + n])
        }
    }

    fn discard_many(&mut self, n: usize) -> bool {
        if self.content.len() < (self.position + n) {
            false
        } else {
            self.position += n;
            true
        }
    }
}

#[cfg(test)]
#[test]
fn test() {
    let source_chars = "this is a test".chars().collect::<Vec<char>>();
    let mut source = Source::from(source_chars);

    assert_eq!(['t', 'h', 'i', 's', ' '], source.peek_many(5).unwrap());
    assert_eq!(['t', 'h', 'i', 's', ' '], source.read_many(5).unwrap());
    assert_eq!(&'s', source.peek_nth(1).unwrap());
    assert_eq!(&'i', source.read().unwrap());
    assert_eq!(&'s', source.peek().unwrap());
    assert!(source.peek_many(999).is_none());
    source.discard_many("s a tes".len());
    assert_eq!(&'t', source.peek().unwrap());
    source.discard();
    assert!(source.peek().is_none());
}
