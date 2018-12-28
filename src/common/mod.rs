//! # Sylan Common Utilities
//!
//! These are common types that exist across all phases, such as buffer traits, versioning types,
//! and Sylan language types that pass through multiple stages unaltered, like the way Sylan strings
//! go from the lexer to the runtime unaltered.
//!
//! As the different phases should be isolated as much as possible, this module should be kept small
//! to avoid heavy coupling.

pub mod excursion_buffer;
pub mod multiphase;
pub mod peekable_buffer;
pub mod version;

pub fn string_matches_char_slice(string: &str, other: &[char]) -> bool {
    // TODO: there _must_ be a better way to compare a str against a char slice without heap
    // allocating a whole new vector every time.
    let string_chars = string.chars().collect::<Vec<char>>();
    string_chars[..] == *other
}
