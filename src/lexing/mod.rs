mod char_escapes;
mod keywords;

pub mod lexer;
pub mod source;
pub mod tokens;

use std::io;
use std::ops::Index;

use lexing::lexer::{LexedToken, Lexer, LexerTask, LexerTaskError};
use peekable_buffer::PeekableBuffer;

const MAX_TOKEN_LOOKAHEAD: usize = 5;

pub struct Tokens {
    lookahead: [LexedToken; MAX_TOKEN_LOOKAHEAD],
    lookahead_len: usize,
    lexer_task: LexerTask,
}

impl Tokens {
    pub fn from(lexer: Lexer) -> io::Result<Self> {
        lexer.lex().map(|lexer_task| Self {
            lookahead: [
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            ],
            lookahead_len: 0,
            lexer_task,
        })
    }

    pub fn join_lexer_thread(self) -> Result<(), LexerTaskError> {
        self.lexer_task.join()
    }
}

pub struct LexedTokenReadMany(Vec<LexedToken>);

impl Index<usize> for LexedTokenReadMany {
    type Output = LexedToken;

    fn index(&self, index: usize) -> &LexedToken {
        let LexedTokenReadMany(slice) = self;
        &slice[index]
    }
}

impl<'a> PeekableBuffer<'a, LexedToken, LexedTokenReadMany> for Tokens {
    fn peek_many(&mut self, n: usize) -> Option<&[LexedToken]> {
        let tokens = &self.lexer_task.tokens;

        // Expand and the lookahead if it's not big enough.
        let pending_peeks = n - self.lookahead_len;
        let mut n = self.lookahead_len;
        let m = self.lookahead_len + pending_peeks;
        let ok = loop {
            if m <= n {
                break true;
            }
            self.lookahead[n] = match tokens.recv() {
                Ok(token) => token,
                Err(_) => break false,
            };
            n += 1;
        };
        self.lookahead_len += pending_peeks;

        if ok {
            // The lookahead now covers the range requested, so slice it.
            Some(&self.lookahead[..(self.lookahead_len)])
        } else {
            None
        }
    }

    fn read_many(&mut self, n: usize) -> Option<LexedTokenReadMany> {
        let lookahead_to_consume = self.lookahead_len.min(n);
        let mut non_lookahead_to_consume = n - lookahead_to_consume;

        // First consume the lookahead.
        let mut read_tokens = (0..lookahead_to_consume)
            .zip(lookahead_to_consume..(lookahead_to_consume + self.lookahead_len))
            .enumerate()
            .map(|(i, (destination, source))| {
                // TODO: work out how to do a `swap_remove` on a slice to avoid
                // a heap allocation and copying the already allocated string in
                // the lexed token.
                let token = self.lookahead[i].clone();

                self.lookahead.swap(destination, source);
                token
            })
            .collect::<Vec<LexedToken>>();
        self.lookahead_len -= lookahead_to_consume;

        // Having exhausted the lookahead, the remaining reads are from the
        // token channel.
        let ok = loop {
            if non_lookahead_to_consume == 0 {
                break true;
            }
            match self.lexer_task.tokens.recv() {
                Ok(token) => read_tokens.push(token),
                Err(_) => break false,
            }
            non_lookahead_to_consume -= 1;
        };

        if ok {
            Some(LexedTokenReadMany(read_tokens))
        } else {
            None
        }
    }

    fn discard_many(&mut self, n: usize) -> bool {
        let lookahead_to_discard = self.lookahead_len.min(n);
        let mut non_lookahead_to_discard = -((self.lookahead_len as isize) - (n as isize));

        // First discard the lookahead.
        (0..lookahead_to_discard)
            .zip(lookahead_to_discard..(lookahead_to_discard + self.lookahead_len))
            .for_each(|(destination, source)| self.lookahead.swap(destination, source));
        self.lookahead_len -= lookahead_to_discard;

        // Now the lookahead is consumed, discard from the token channel.
        loop {
            if non_lookahead_to_discard <= 0 {
                break true;
            }
            match self.lexer_task.tokens.recv() {
                Ok(_) => {}
                Err(_) => break false,
            }
            non_lookahead_to_discard -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::source::Source;
    use super::tokens::Token;
    use super::*;
    use multiphase::Identifier;
    use std::fmt::Debug;

    const TEST_SOURCE: &str = r#"

        List(1, 2, 3).forEach(n ->
            println(`{n}`)
        )

    "#;

    fn test<A>(f: impl Fn(&mut Tokens) -> A) -> A
    where
        A: Eq + Debug,
    {
        let chars = TEST_SOURCE.chars().collect::<Vec<char>>();
        let source = Source::from(chars);
        let mut tokens = Tokens::from(Lexer::from(source)).unwrap();
        let result = f(&mut tokens);
        tokens.lexer_task.join().unwrap();
        result
    }

    fn assert_next<A>(f: impl Fn(&mut Tokens) -> A, x: A)
    where
        A: Eq + Debug,
    {
        test(|tokens| assert_eq!(f(tokens), x))
    }

    #[test]
    fn peek() {
        assert_next(
            |tokens| tokens.peek().unwrap().token.clone(),
            Token::Identifier(Identifier::from("List")),
        )
    }

    #[test]
    fn peek_many() {
        assert_next(
            |tokens| {
                tokens
                    .peek_many(4)
                    .unwrap()
                    .iter()
                    .map(|x| x.token.clone())
                    .collect::<Vec<Token>>()
            },
            vec![
                Token::Identifier(Identifier::from("List")),
                Token::OpenParentheses,
                Token::Number(1, 0),
                Token::SubItemSeparator,
            ],
        )
    }

    #[test]
    fn peek_nth() {
        assert_next(
            |tokens| {
                tokens.discard_many(5);
                tokens.peek_nth(5).unwrap().token.clone()
            },
            Token::Identifier(Identifier::from("forEach")),
        )
    }

    #[test]
    fn read() {
        assert_next(
            |tokens| {
                tokens.read().unwrap();
                tokens.read().unwrap();
                tokens.peek().unwrap();
                tokens.read().unwrap().token
            },
            Token::Number(1, 0),
        )
    }

    #[test]
    fn read_many() {
        assert_next(
            |tokens| {
                tokens.discard_many(8);
                let LexedTokenReadMany(read) = tokens.read_many(3).unwrap();
                read.iter()
                    .map(|lexed| lexed.token.clone())
                    .collect::<Vec<Token>>()
            },
            vec![
                Token::Dot,
                Token::Identifier(Identifier::from("forEach")),
                Token::OpenParentheses,
            ],
        )
    }

    #[test]
    fn discard() {
        assert_next(
            |tokens| {
                tokens.discard();
                tokens.discard();
                tokens.discard();
                tokens.discard();
                tokens.read().unwrap().token
            },
            Token::Number(2, 0),
        )
    }

    #[test]
    fn discard_many() {
        assert_next(
            |tokens| {
                tokens.discard_many(3);
                tokens.read().unwrap().token
            },
            Token::SubItemSeparator,
        )
    }

    #[test]
    fn match_nth() {
        test(|tokens| assert!(tokens.match_nth(3, |lexed| lexed.token == Token::Number(1, 0),)))
    }

    #[test]
    fn trivia() {
        let trivia_to_match = String::from(
            r#"

        "#,
        );
        assert_next(
            |tokens| tokens.peek().unwrap().clone().trivia.unwrap(),
            trivia_to_match,
        );
    }
}
