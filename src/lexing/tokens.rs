use common::multiphase::{Identifier, InterpolatedString, Shebang, SyDoc, SylanString};
use common::version::Version;

/// All tokens that can currently exist in all version of a Sylan program source.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Token {
    Boolean(bool),
    Char(char),
    Identifier(Identifier),
    InterpolatedString(InterpolatedString),

    // TODO: reimplement using a variable-width numerics library, like GMP but not GPL licenced.
    Number(i64, u64),

    Shebang(Shebang),
    String(SylanString),
    SyDoc(SyDoc),
    Version(Version),
    Ampersand,
    As,
    Assign,
    Bind,
    Class,
    CloseBrace,
    CloseParentheses,
    CloseSquareBracket,
    Colon,
    Constructor,
    Continue,
    Dot,
    Else,
    Eof,
    Embed,
    Extend,
    Extends,
    Exports,
    For,
    Func,
    Extern,
    If,
    It,
    Ignorable,
    TypeConstraint,
    Import,
    InvocableHandle,
    Interface,
    Internal,
    LambdaArrow,
    Module,
    Not,
    OpenBrace,
    OpenParentheses,
    OpenSquareBracket,
    Operator,
    Override,
    Package,

    // A dummy identifier that has different meanings in different contexts. In bindings it allows
    // discarding values.
    PlaceholderIdentifier,

    Public,

    /// Does nothing but reserves keywords for future use.
    ReservedKeyword,

    Reject,
    Requires,
    Rest,
    Select,
    SubItemSeparator,
    Super,
    Switch,
    This,
    Throw,
    Timeout,
    Try,
    Using,
    Var,
    Virtual,
    With,
    While,
}

/// EOF is a special type of token because it simplifies logic over handling it
/// in a special typed manner in every lexing case.
impl Default for Token {
    fn default() -> Token {
        Token::Eof
    }
}
