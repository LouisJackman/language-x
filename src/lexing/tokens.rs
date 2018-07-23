use std::sync::Arc;
use version::Version;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Boolean(bool),
    Char(char),
    Identifier(Arc<String>),
    InterpolatedString(Arc<String>),
    Number(i64, u64),
    Shebang(Arc<String>),
    String(Arc<String>),
    SyDoc(Arc<String>),
    Version(Version),
    Add,
    And,
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
    Embeds,
    Extends,
    For,
    Get,
    GreaterThan,
    GreaterThanOrEquals,
    If,
    Ignore,
    Implements,
    Import,
    Interface,
    LambdaArrow,
    LessThan,
    LessThanOrEquals,
    MethodHandle,
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
    Select,
    ShiftLeft,

    // Could be either a right-shift operator or two closing type parameter
    // brackets. Disamgiguate in the parser.
    DoubleRightAngleBracket,

    SubItemSeparator,
    Subtract,
    Super,
    Switch,
    Throw,
    Timeout,
    Var,
}

impl Default for Token {
    fn default() -> Token {
        Token::Eof
    }
}
