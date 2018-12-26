//! Immutable types that cross over multiple phases.
//!
//! For example, string literals are passed unaltered between the lexer,
//! the parser, and compiler, and the runtime.

use std::sync::Arc;

macro_rules! multiphase_string_type {
    ( $( $type: ident ),* ) => {
        $(
            #[derive(Clone, Debug, Eq, Hash, PartialEq)]
            pub struct $type(pub Arc<String>);

            impl From<String> for $type {
                fn from(string: String) -> Self {
                    $type(Arc::new(string))
                }
            }

            impl From<&'static str> for $type {
                fn from(string: &'static str) -> Self {
                    Self::from(string.to_owned())
                }
            }
        )*
    }
}

multiphase_string_type![Identifier, Shebang, SylanString, SyDoc];

/// Interpolations are interleaved with string fragments, ready to be glued
/// together when the runtime knows what the interpolated identifiers resolve
/// to.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct InterpolatedString {
    pub string_fragments: Arc<Vec<String>>,
    pub interpolations: Arc<Vec<Identifier>>,
}

impl From<String> for InterpolatedString {
    fn from(string: String) -> Self {
        Self {
            string_fragments: Arc::new(vec![string]),
            interpolations: Arc::new(vec![]),
        }
    }
}

impl From<&'static str> for InterpolatedString {
    fn from(string: &'static str) -> Self {
        Self::from(string.to_owned())
    }
}
