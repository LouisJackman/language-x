pub struct Source {
    content: Vec<char>,
    pub position: usize,
}

impl Source {
    pub fn from(content: Vec<char>) -> Self {
        Self {
            content,
            position: 0,
        }
    }

    pub fn peek_many(&self, n: usize) -> Option<&[char]> {
        if self.content.len() <= (self.position + n) {
            None
        } else {
            let m = self.position + n;
            Some(&self.content[self.position..m])
        }
    }

    pub fn read_many(&mut self, n: usize) -> Option<&[char]> {
        if self.content.len() <= (self.position + n) {
            None
        } else {
            let new_position = self.position + n;
            let result = Some(&self.content[self.position..new_position]);
            self.position = new_position;
            result
        }
    }

    pub fn peek(&self) -> Option<char> {
        self.peek_many(1).map(|s| s[0])
    }

    pub fn read(&mut self) -> Option<char> {
        self.read_many(1).map(|s| s[0])
    }

    pub fn peek_nth(&self, n: usize) -> Option<char> {
        if self.content.len() <= (self.position + n + 1) {
            None
        } else {
            Some(self.content[self.position + n])
        }
    }

    pub fn match_nth<F>(&self, n: usize, predicate: F) -> bool
    where
        F: FnOnce(char) -> bool,
    {
        self.peek_nth(n).map_or(false, predicate)
    }

    pub fn nth_is(&self, n: usize, to_match: char) -> bool {
        self.match_nth(n, |c| c == to_match)
    }

    pub fn discard_many(&mut self, n: usize) -> bool {
        if self.content.len() <= (self.position + n) {
            false
        } else {
            self.position += n;
            true
        }
    }

    pub fn discard(&mut self) -> bool {
        self.discard_many(1)
    }
}

#[cfg(test)]
#[test]
fn test() {
    let mut source = Source::from("this is a test".chars().collect());

    assert_eq!(['t', 'h', 'i', 's', ' '], source.peek_many(5).unwrap());
    assert_eq!(['t', 'h', 'i', 's', ' '], source.read_many(5).unwrap());
    assert_eq!('s', source.peek_nth(1).unwrap());
    assert_eq!('i', source.read().unwrap());
    assert_eq!('s', source.peek().unwrap());
    assert!(source.peek_many(999).is_none());
    source.discard_many("s a tes".len());
    assert_eq!('t', source.peek().unwrap());
    source.discard();
    assert!(source.peek().is_none());
}
