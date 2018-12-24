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
