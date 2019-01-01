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
    Do,
    Dot,
    Else,
    Eof,
    Equals,
    Embed,
    Extend,
    Extends,
    Exports,
    For,
    Get,
    GreaterThan,
    GreaterThanOrEquals,
    If,
    It,
    Ignorable,
    Ignore,
    Implements,
    Import,
    Interface,
    Internal,
    LambdaArrow,
    LessThan,
    LessThanOrEquals,
    MethodHandle,
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

    /// Does nothing but let us reserve keywords for future use.
    ReservedKeyword,

    Reject,
    Requires,
    Rest,
    Select,
    ShiftLeft,

    /// Could be either a right-shift operator or two closing type parameter
    /// brackets. Disambiguate in the parser.
    DoubleRightAngleBracket,

    SubItemSeparator,
    Subtract,
    Super,
    Switch,
    Throw,
    Timeout,
    Try,
    Using,
    Var,
    Virtual,
    With,
    Where,
}

/// EOF is a special type of token because it simplifies logic over handling it
/// in a special typed manner in every lexing case.
impl Default for Token {
    fn default() -> Token {
        Token::Eof
    }
}
