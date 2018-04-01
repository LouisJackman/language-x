mod nodes;

use peekable_buffer::PeekableBuffer;
use lexing::lexer;
use lexing::tokens::Token;
use lexing::Tokens;
use parsing::nodes::Node;
use parsing::nodes::Expression::{Identifier, Literal};
use parsing::nodes::Node::{Expression, Item};

const PARSER_THREAD_NAME: &str = "Sylan Parser";

pub struct ParserError {
    message: String,
    node: Node,
}

pub enum Error {
    Lexer(lexer::Error),
    Parser(ParserError),
}

pub type ParseResult = Result<Option<Node>, Error>;

pub struct Parser {
    tokens: Tokens,
}

impl Parser {
    fn from(tokens: Tokens) -> Self {
        Self { tokens }
    }

    fn parse_next(&mut self) -> ParseResult {
        match self.tokens.peek() {
            Some(lexed) => {
                let node: Result<Node, Error> = match lexed.token.clone() {
                    Token::Boolean(b) => {
                        Ok(Expression(Literal(nodes::Literal::Boolean(b))))
                    }
                    Token::Char(c) => {
                        Ok(Expression(Literal(nodes::Literal::Char(c))))
                    }
                    Token::Identifier(string) => {
                        Ok(Expression(Identifier(nodes::Identifier::Actual(
                            string
                        ))))
                    },
                    Token::InterpolatedString(string) => {

                        // TODO: reenter the lexer to handle interpolation
                        // properly.
                        Ok(Expression(Literal(
                            nodes::Literal::InterpolatedString(string)
                        )))

                    },
                    Token::Number(decimal, fraction) => {
                        Ok(Expression(Literal(nodes::Literal::Number(
                            decimal,
                            fraction,
                        ))))
                    },
                    Token::Shebang(string) => {
                        Ok(Item(nodes::Item::Shebang(string)))
                    },
                    Token::String(string) => {
                        Ok(Expression(Literal(nodes::Literal::String(string))))
                    },
                    Token::SyDoc(string) => {
                        Ok(Item(nodes::Item::SyDoc(string)))
                    },
                    Token::Version(major, minor) => {
                        Ok(Item(nodes::Item::Version(major, minor)))
                    },
                    Token::Add => { unimplemented!() },
                    Token::And => { unimplemented!() },
                    Token::Assign => { unimplemented!() },
                    Token::Bind => { unimplemented!() },
                    Token::BitwiseAnd => { unimplemented!() },
                    Token::BitwiseNot => { unimplemented!() },
                    Token::BitwiseOr => { unimplemented!() },
                    Token::BitwiseXor => { unimplemented!() },
                    Token::Case => { unimplemented!() },
                    Token::Class => { unimplemented!() },
                    Token::CloseBrace => { unimplemented!() },
                    Token::CloseParentheses => { unimplemented!() },
                    Token::CloseSquareBracket => { unimplemented!() },
                    Token::Colon => { unimplemented!() },
                    Token::Compose => { unimplemented!() },
                    Token::Continue => { unimplemented!() },
                    Token::Default => { unimplemented!() },
                    Token::Divide => { unimplemented!() },
                    Token::Do => { unimplemented!() },
                    Token::Dot => { unimplemented!() },
                    Token::Else => { unimplemented!() },
                    Token::Eof => { unimplemented!() },
                    Token::Equals => { unimplemented!() },
                    Token::Extends => { unimplemented!() },
                    Token::For => { unimplemented!() },
                    Token::Get => { unimplemented!() },
                    Token::GreaterThan => { unimplemented!() },
                    Token::GreaterThanOrEquals => { unimplemented!() },
                    Token::If => { unimplemented!() },
                    Token::Ignore => { unimplemented!() },
                    Token::Implements => { unimplemented!() },
                    Token::Import => { unimplemented!() },
                    Token::Interface => { unimplemented!() },
                    Token::LambdaArrow => { unimplemented!() },
                    Token::LessThan => { unimplemented!() },
                    Token::LessThanOrEquals => { unimplemented!() },
                    Token::MethodHandle => { unimplemented!() },
                    Token::Modulo => { unimplemented!() },
                    Token::Multiply => { unimplemented!() },
                    Token::Not => { unimplemented!() },
                    Token::NotEquals => { unimplemented!() },
                    Token::OpenBrace => { unimplemented!() },
                    Token::OpenParentheses => { unimplemented!() },
                    Token::OpenSquareBracket => { unimplemented!() },
                    Token::Or => { unimplemented!() },
                    Token::Override => { unimplemented!() },
                    Token::Package => { unimplemented!() },
                    Token::Pipe => { unimplemented!() },
                    Token::Public => { unimplemented!() },
                    Token::Select => { unimplemented!() },
                    Token::ShiftLeft => { unimplemented!() },
                    Token::ShiftRight => { unimplemented!() },
                    Token::SubItemSeparator => { unimplemented!() },
                    Token::Subtract => { unimplemented!() },
                    Token::Super => { unimplemented!() },
                    Token::Switch => { unimplemented!() },
                    Token::Throw => { unimplemented!() },
                    Token::Timeout => { unimplemented!() },
                    Token::Var => { unimplemented!() },
                };
                node.map(Some)
            }
            None => Ok(None),
        }
    }
}
