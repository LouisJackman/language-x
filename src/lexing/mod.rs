mod keywords;
mod char_escapes;

pub mod source;
pub mod tokens;
pub mod lexer;

use std::io;
use std::thread::JoinHandle;
use std::sync::mpsc::Receiver;

use lexing::lexer::{LexedToken, Lexer};
use lexing::tokens::Token;
use peekable_buffer::PeekableBuffer;

const MAX_TOKEN_LOOKAHEAD: usize = 5;

pub struct Tokens {
    token_lookahead: [Token; MAX_TOKEN_LOOKAHEAD],
    lookahead_position: u8,
    lexer_join_handle: JoinHandle<()>,
    token_receiver: Receiver<LexedToken>,
}

impl Tokens {

    pub fn from(lexer: Lexer) -> io::Result<Self> {
        lexer.lex().map(|(token_receiver, lexer_join_handle)| Self {
            token_lookahead: [
                Token::Eof,
                Token::Eof,
                Token::Eof,
                Token::Eof,
                Token::Eof,
            ],
            lookahead_position: 0,
            token_receiver,
            lexer_join_handle,
        })
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
