use std::ops::Index;

/// A buffer that allows reading, peeking, and provides convenience methods for common operations
/// like checking a predicate against a peeked value.
pub trait PeekableBuffer<'a, T, ReadMany>
where
    T: Clone + Eq,
    ReadMany: 'a + Index<usize, Output = T>,
{
    /// Get an immutable view over future elements in the buffer.
    fn peek_many(&mut self, n: usize) -> Option<&[T]>;

    /// Consume multiple items from the buffer, the result of which cannot outlive the next use of
    /// the buffer.
    fn read_many(&'a mut self, n: usize) -> Option<ReadMany>;

    /// Throw away the next `n` elements from the buffer, returning `true` if all `n` elements were
    /// thrown away or `false` if all remaining elements were discarded but were all fewer than the
    /// amount specified.
    fn discard_many(&mut self, n: usize) -> bool;

    /// Get an immutable view of the next element in the buffer.
    fn peek(&mut self) -> Option<&T> {
        self.peek_many(1).and_then(|s| s.first())
    }

    /// Get an immutable view of the `n`th next element in the buffer, where `n` is zero indexed.
    fn peek_nth(&mut self, n: usize) -> Option<&T> {
        self.peek_many(n).and_then(|tokens| tokens.last())
    }

    /// Consume an item from the buffer and return it.
    fn read(&'a mut self) -> Option<T> {
        self.read_many(1).map(|s| s[0].clone())
    }

    /// Check whether the `n`th next item in the buffer matches predicate `predicate`, where `n` is
    /// zero-indexed.
    fn match_nth(&mut self, n: usize, predicate: impl Fn(&T) -> bool) -> bool {
        self.peek_nth(n).map_or(false, predicate)
    }

    /// Check whether the next item in the buffer matches predicate `predicate`.
    fn match_next(&mut self, predicate: impl Fn(&T) -> bool) -> bool {
        self.match_nth(0, predicate)
    }

    /// Check whether the `n`th next item in the buffer is equal to `to_match`, where `n` is
    /// zero-indexed.
    fn nth_is(&mut self, n: usize, to_match: T) -> bool {
        self.match_nth(n, |c| *c == to_match)
    }

    /// Check whether the next item in the buffer is equal to `to_match`.
    fn next_is(&mut self, to_match: T) -> bool {
        self.nth_is(0, to_match)
    }

    /// Throw away the next element from the buffer, returning `false` if the buffer was already
    /// empty.
    fn discard(&mut self) -> bool {
        self.discard_many(1)
    }
}
