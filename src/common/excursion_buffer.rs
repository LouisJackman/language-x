/// Allows a buffer to start an 'excursion', giving another copy derived from the original. Similar
/// to `Clone` but allows mutating `self`; however, this should still not be visible from outside
/// the type's own methods.
///
/// An example use case is duplicating a buffer of tokens by broadcasting incoming tokens over a
/// channel from original buffers to excursions created from it.
pub trait ExcursionBuffer {
    fn start_excursion(&mut self) -> Self;
}
