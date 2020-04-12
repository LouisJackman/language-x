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

    // See: https://dart.dev/guides/language/language-tour#cascade-notation-
    Cascade,

    // See: https://docs.microsoft.com/en-us/dotnet/fsharp/language-reference/functions/#function-composition-and-pipelining
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

    // See: https://docs.microsoft.com/en-us/dotnet/fsharp/language-reference/functions/#function-composition-and-pipelining
    Pipe,

    Power,
    RightShift,

    // See: https://docs.oracle.com/javase/tutorial/java/nutsandbolts/op3.html
    UnsignedRightShift,

    Subtract,

    // Inspired by Python's dedicated Matrix operator: https://www.python.org/dev/peps/pep-0465/
    MatrixAdd,
    MatrixDivide,
    MatrixMultiply,
    MatrixPower,
    MatrixSubtract,
    MatrixTranspose,

    // Like bitwise-or, but for booleans.
    Xor,
}

/// How slicing is interpreted is totally up the implementor of the operator.
/// This especially applies to the `...` pseudoidentifier, which resolves
/// statically to the `sylan.lang.slice.SliceFragment.Ellipsis` enum data
/// constructor which any API can use, but it is handled specially by the
/// overloadable slice operator. This syntactical sugar transforms invocations
/// like `[|1 : 2 : 3, ..., 1 :]` into these arguments passed variadically into
/// the overloaded operator:
///
/// ```
/// (
///     SliceFragment::Slice(from: 1, stepping: 2, to: 3),
///     SliceFragment::Ellipsis,
///     SliceFragment::Slice(from: 1),
/// )
/// ```
///
/// The signature of overloaded operator looks like:
///
/// `fun public operator [||] (sliceFragments ..SliceFragment) { }`
///
/// `SliceFragment` is defined as:
///
/// ```
/// enum public Slice(
///     Start(from start Optional[Number]),
///     Step(stepping step Optional[Number]),
///     End(to end Optional[Number]),
/// )
///
/// enum public SliceFragment(
///     Ellipsis,
///     Slice(_ slice Slice),
/// )
/// ```
///
/// Examples:
///
/// * `list[|1 : 3|]` from the first element to the 3rd, semi-inclusive, and
///    considering Sylan's 0-based indexing.
/// * `list[|: -1 : -2|]` reverses from antepenultimate element to the first.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum OverloadableSliceOperator {
    Open,
    Close,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum PostfixOperator {
    InvocableHandle,
    Bind,
}

/// They act as identifiers but have two distinguishing properties:
///
/// * They can be shadowed in the same block.
/// * They cannot be defined by user code.
/// * They can only be referred to directly, not via package lookups with dots.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum PseudoIdentifier {
    Continue,
    It,
    Super,
    This,
    ThisType,
    ThisPackage,
    ThisModule,

    /// Similar to Python's ellipsis, it's a variable that can be checked
    /// against and interpreted accordingly by data-oriented APIs when
    /// slicing. Unlike Python, Sylan is statically-typed and compiles away all
    /// types at compile-time, so it cannot just be a singleton type. It instead
    /// refers to an enum variant of `SliceFragment` rather than a whole type:
    /// `sylan.lang.slice.SliceFragment.Ellipsis`.
    ///
    /// It, along with numbers, are the only values that can be used within the
    /// slice notation. Unlike numbers, it must be used directly inline and not
    /// merely as an expression that eventually yields a value of the correct
    /// type.
    ///
    /// See [OverloadableSliceOperator] for more details about how this is used.
    Ellipsis,

    /// A dummy identifier that has different meanings in different contexts. In
    /// bindings it allows discarding values; as an argument to invocables, it
    /// transforms an invocation into a partial application.
    PlaceholderIdentifier,
}
