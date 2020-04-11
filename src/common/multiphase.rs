//! Immutable types that cross over multiple phases.
//!
//! For example, string literals are passed unaltered between the lexer,
//! the parser, and compiler, and the runtime.

use std::sync::Arc;

macro_rules! multiphase_string_types {
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

multiphase_string_types![Identifier, Shebang, SylanString, SyDoc];

/// Interpolations are interleaved with string fragments, ready to be glued
/// together when the runtime knows what the interpolated identifiers resolve
/// to.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct InterpolatedString {
    pub string_fragments: Vec<String>,
    pub interpolations: Vec<Identifier>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Accessibility {
    Private,
    Internal,
    Public,
}

// Operators are overloadable but fixed. This achieves two things:
//
// * Stops developers from  creating "ASCII-art operators", increasing maintainability.
// * Fixes parsing ambiguities due the inability to distinguish three seperate expressions from a
//   single expression using an infix operator (without introducing whitespace-sensitive lexing of
//   seperate expressions.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum OverloadableInfixOperator {
    Add,

    // Can be either a type constrait union or a bitwise and. Work it out in the parser.
    Ampersand,

    And,
    BitwiseOr,
    BitwiseXor,
    Compose,
    Divide,
    Equals,
    GreaterThan,
    GreaterThanOrEqual,
    LeftShift,
    LessThan,
    LessThanOrEqual,
    Modulo,
    Multiply,
    NotEqual,
    Or,
    Pipe,
    Power,
    RightShift,
    UnsignedRightShift,
    Subtract,
    VectorAdd,
    VectorDivide,
    VectorMultiply,
    VectorPower,
    VectorSubtract,
    Xor,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum PostfixOperator {
    InvocableHandle,
    Bind,
}

/// They act as identifiers but have two distinguishing properties:
///
/// * They can be shadowed.
/// * They cannot be defined by user code.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum PseudoIdentifier {
    Continue,
    It,
    Super,
    This,

    // A dummy identifier that has different meanings in different contexts. In bindings it allows
    // discarding values.
    PlaceholderIdentifier,
}
