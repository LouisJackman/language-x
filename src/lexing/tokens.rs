use common::multiphase::{Identifier, InterpolatedString, Shebang, SyDoc, SylanString};
use common::version::Version;

/// All tokens that can currently exist in all version of a Sylan program source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Boolean(bool),
    Char(char),
    Identifier(Identifier),
    InterpolatedString(InterpolatedString),
    Number(i64, u64),
    Shebang(Shebang),
    String(SylanString),
    SyDoc(SyDoc),
    Version(Version),
    Add,
    And,
    As,
    Assign,
    Bind,
    BitwiseAnd,
    BitwiseNot,
    BitwiseOr,
    BitwiseXor,
    Class,
    CloseBrace,
    CloseParentheses,
    CloseSquareBracket,
    Colon,
    Compose,
    Continue,
    Default,
    Divide,
    Dot,
    Else,
    Eof,
    Equals,
    Embed,
    Extend,
    Extends,
    Exports,
    For,
    GreaterThan,
    GreaterThanOrEquals,
    If,
    It,
    Ignorable,
    Implements,
    Import,
    InvocableHandle,
    Interface,
    Internal,
    LambdaArrow,
    LeftAngleBracket,
    LessThanOrEquals,
    Module,
    Modulo,
    Multiply,
    Not,
    NotEquals,
    OpenBrace,
    OpenParentheses,
    OpenSquareBracket,
    Or,
    Override,
    Package,
    Pipe,

    // A dummy identifier that has different meanings in different contexts. In bindings it allows
    // discarding values.
    PlaceholderIdentifier,

    /// Does nothing but let us reserve keywords for future use.
    ReservedKeyword,

    Reject,
    Requires,
    Rest,
    Select,
    RightAngleBracket,
    SubItemSeparator,
    Subtract,
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
    Where,
    While,
}

/// EOF is a special type of token because it simplifies logic over handling it
/// in a special typed manner in every lexing case.
impl Default for Token {
    fn default() -> Token {
        Token::Eof
    }
}
