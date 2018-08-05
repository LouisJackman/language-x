//! Types that cross over multiple phases.
//!
//! For example, string literals are passed unaltered between the lexer,
//! the parser, and compilers, and the runtime.

use std::sync::Arc;

macro_rules! multiphase_string_type {
    ( $( $type: ident ),* ) => {
        $(
            #[derive(Clone, Debug, Eq, Hash, PartialEq)]
            pub struct $type(pub Arc<String>);
        )*
    }
}

multiphase_string_type![Identifier, InterpolatedString, Shebang, SylanString, SyDoc];
