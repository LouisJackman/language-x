//! Character escapes allow developers to enter characters into Sylan strings that can otherwise
//! awkward to encode in the UTF-8 source in some contexts. Examples include newlines and hard tabs.

use std::collections::HashMap;

/// Map escape characters to the literal characters they represent. As Sylan has a strict subset of
/// Rust's escape characters so far, it's currently a one-to-one mapping, although this isn't
/// guaranteed to always be the case.
pub fn new() -> HashMap<char, char> {
    let mut map = HashMap::new();
    map.extend(vec![
        ('n', '\n'),
        ('r', '\r'),
        ('t', '\t'),
        ('\\', '\\'),
        ('\'', '\''),
    ]);
    map
}
