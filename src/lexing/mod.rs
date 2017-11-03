mod keywords;
mod char_escapes;

pub mod source;
pub mod tokens;
pub mod lexer;

use lexing::lexer::Lexer;
use lexing::tokens::Token;
use peekable_buffer::PeekableBuffer;

const MAX_TOKEN_LOOKAHEAD: usize = 5;

struct Tokens {
    token_lookahead: [Token; MAX_TOKEN_LOOKAHEAD],
    lexer: Lexer,
}

impl Tokens {
    pub fn from(lexer: Lexer) -> Self {
        Self {
            token_lookahead: [
                Token::Eof,
                Token::Eof,
                Token::Eof,
                Token::Eof,
                Token::Eof,
            ],
            lexer,
        }
    }
}

impl PeekableBuffer<Token> for Tokens {
    fn peek_many(&self, n: usize) -> Option<&[Token]> {
        unimplemented!()
    }

    fn read_many(&mut self, n: usize) -> Option<&[Token]> {
        unimplemented!()
    }

    fn peek_nth(&self, n: usize) -> Option<&Token> {
        unimplemented!()
    }

    fn discard_many(&mut self, n: usize) -> bool {
        unimplemented!()
    }
}
