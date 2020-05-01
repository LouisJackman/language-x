use crate::common::multiphase::{
    Accessibility, Identifier, InterpolatedString, Number, OverloadableInfixOperator,
    OverloadableSliceOperator, PostfixOperator, PseudoIdentifier, Shebang, SyDoc, SylanString,
};
use crate::common::version::Version;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Literal {
    Char(char),
    InterpolatedString(InterpolatedString),
    String(SylanString),
    Number(Number),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum BranchingAndJumping {
    If,
    Else,
    While,
    For,
    Switch,
    Select,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum DeclarationHead {
    Class,
    Extend,
    Fun,
    Implements,
    Interface,
    Module,
    Package,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Grouping {
    CloseBrace,
    CloseParentheses,
    CloseSquareBracket,
    OpenBrace,
    OpenParentheses,
    OpenSquareBracket,
}

/// Unlike other languages, modifiers always come _after_ declaration heads.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Modifier {
    Accessibility(Accessibility),

    Embed,

    /// Extern means quite different things depending on whether it refers to a
    /// bindings, functions, a type. For bindings and functions, it means they
    /// are defined in another compiled artefact, perhaps written in a different
    /// language. For types, it means that it is defined by Sylan itself. Types
    /// cannot be defined outside of Sylan, only the _operations_ and _runtime
    /// values_ of said types.
    Extern,

    Ignorable,
    Operator,
    Override,

    /// Only used by extern final and var bindings, to notify Sylan that it
    /// should wrap access in a multi-thread memory fence. It provides no
    /// guarantees about how modifications and reads to the item are
    /// interleaved.
    ///
    /// This should only be used for FFI; a proper Sylan wrapper should be
    /// provided that abstracts away such details.
    Volatile,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ModuleDefinitions {
    Exports,
    Reject,
    Requires,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Binding {
    As,
    Assign,
    DynamicBind,
    DynamicUnbind,
    Final,
    Var,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Macros {
    At,
    Quote,
    Unquote,
    Syntax,
}

/// All tokens that can currently exist in all version of a Sylan program source.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Token {
    Identifier(Identifier),
    Literal(Literal),
    Shebang(Shebang),
    SyDoc(SyDoc),
    Version(Version),

    Binding(Binding),
    BranchingAndJumping(BranchingAndJumping),
    DeclarationHead(DeclarationHead),
    Grouping(Grouping),
    Modifier(Modifier),
    ModuleDefinitions(ModuleDefinitions),
    OverloadableInfixOperator(OverloadableInfixOperator),
    OverloadableSliceOperator(OverloadableSliceOperator),
    PostfixOperator(PostfixOperator),
    PseudoIdentifier(PseudoIdentifier),
    Macros(Macros),

    Colon,
    Dot,
    Eof,
    LambdaArrow,

    // Sylan resolves symbols relatively. To resolve globally, use the `global`
    // keyword, e.g. `global.module1.package1.Class1.method1`. It isn't
    // technically a pseudo-identifier because it doesn't mean anything by
    // itself, needing to be combined with a dot followed by a symbol.
    Global,

    // Used in both declaration heads and for upper bounds on type parameters.
    Extends,

    Rest,
    SubItemSeparator,
    Throw,
    Timeout,
    Using,

    /// Does nothing but reserve keywords for future use.
    ReservedKeyword,

    With,
}

/// EOF is a special type of token because it simplifies logic over handling it
/// in a special typed manner in every lexing case.
impl Default for Token {
    fn default() -> Token {
        Token::Eof
    }
}
