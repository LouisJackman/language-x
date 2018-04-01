use std::ops::Index;

pub trait PeekableBuffer<'a, T, ReadMany>
where
    T: Clone + Eq,
    ReadMany: 'a + Index<usize, Output = T>,
{
    fn peek_many(&mut self, n: usize) -> Option<&[T]>;
    fn read_many(&'a mut self, n: usize) -> Option<ReadMany>;
    fn peek_nth(&mut self, n: usize) -> Option<&T>;
    fn discard_many(&mut self, n: usize) -> bool;

    fn peek(&mut self) -> Option<&T> {
        self.peek_many(1).map(|s| &s[0])
    }

    fn read(&'a mut self) -> Option<T> {
        self.read_many(1).map(|s| s[0].clone())
    }

    fn match_nth<F>(&mut self, n: usize, predicate: F) -> bool
    where
        F: Fn(&T) -> bool,
    {
        self.peek_nth(n).map_or(false, predicate)
    }

    fn nth_is(&mut self, n: usize, to_match: T) -> bool {
        self.match_nth(n, |c| *c == to_match)
    }

    fn discard(&mut self) -> bool {
        self.discard_many(1)
    }
}