mod keywords;
mod char_escapes;

pub mod source;
pub mod tokens;
pub mod lexer;

use std::io;
use std::ops::Index;
use std::thread::JoinHandle;
use std::sync::mpsc::Receiver;

use lexing::lexer::{LexedToken, Lexer};
use peekable_buffer::PeekableBuffer;

const MAX_TOKEN_LOOKAHEAD: usize = 5;

pub struct Tokens {
    lookahead: [LexedToken; MAX_TOKEN_LOOKAHEAD],
    lookahead_len: usize,
    lexer_handle: JoinHandle<()>,
    tokens: Receiver<LexedToken>,
}

impl Tokens {
    pub fn from(lexer: Lexer) -> io::Result<Self> {
        lexer.lex().map(|(tokens, lexer_handle)| Self {
            lookahead: [
                LexedToken::default(),
                LexedToken::default(),
                LexedToken::default(),
                LexedToken::default(),
                LexedToken::default(),
            ],
            lookahead_len: 0,
            tokens,
            lexer_handle,
        })
    }
}

struct LexedTokenReadMany<'a>(&'a [LexedToken]);

impl<'a> Index<usize> for LexedTokenReadMany<'a> {
    type Output = LexedToken;

    fn index(&self, index: usize) -> &LexedToken {
        let &LexedTokenReadMany(slice) = self;
        &slice[index]
    }
}

impl<'a> PeekableBuffer<'a, LexedToken, LexedTokenReadMany<'a>> for Tokens {
    fn peek_many(&mut self, n: usize) -> Option<&[LexedToken]> {
        let lookahead_len = self.lookahead_len;
        let remaining = n - lookahead_len;

        let mut index = lookahead_len;
        let upper_bound = lookahead_len + remaining;
        let ok = loop {
            if upper_bound <= index {
                break true;
            }
            match self.tokens.recv() {
                Ok(token) => {
                    self.lookahead[index] = token;
                    index += 1;
                    self.lookahead_len += 1;
                }
                Err(_) => break false,
            }
        };

        if ok {
            Some(&self.lookahead[0..n])
        } else {
            None
        }
    }

    fn read_many(&mut self, n: usize) -> Option<LexedTokenReadMany<'a>> {
        unimplemented!()
    }

    fn peek_nth(&mut self, n: usize) -> Option<&LexedToken> {
        unimplemented!()
    }

    fn discard_many(&mut self, n: usize) -> bool {
        unimplemented!()
    }
}
