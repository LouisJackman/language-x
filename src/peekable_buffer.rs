pub trait PeekableBuffer<T> where T: Eq {
    fn peek_many(&self, n: usize) -> Option<&[T]>;
    fn read_many(&mut self, n: usize) -> Option<&[T]>;
    fn peek_nth(&self, n: usize) -> Option<&T>;
    fn discard_many(&mut self, n: usize) -> bool;

    fn peek(&self) -> Option<&T> {
        self.peek_many(1).map(|s| &s[0])
    }

    fn read(&mut self) -> Option<&T> {
        self.read_many(1).map(|s| &s[0])
    }

    fn match_nth<F>(&self, n: usize, predicate: F) -> bool
    where
        F: Fn(&T) -> bool,
    {
        self.peek_nth(n).map_or(false, predicate)
    }

    fn nth_is(&self, n: usize, to_match: T) -> bool {
        self.match_nth(n, |c| *c == to_match)
    }

    fn discard(&mut self) -> bool {
        self.discard_many(1)
    }
}
