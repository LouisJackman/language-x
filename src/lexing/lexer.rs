use std::any::Any;
use std::collections::HashMap;
use std::io;
use std::sync::mpsc::{channel, Receiver, RecvError};
use std::thread::{self, JoinHandle};

use common::excursion_buffer::ExcursionBuffer;
use common::multiphase::{self, SylanString};
use common::peekable_buffer::PeekableBuffer;
use common::version::Version;
use lexing::char_escapes;
use lexing::keywords;
use lexing::tokens::Token;
use source::in_memory::Source;
use source::Position;

// TODO: implement multiline strings.
// TODO: lex shebang and version string before starting the main lexing loop.
// TODO: implement escapes not just for chars but also strings comments, and
//       SyDocs.
// TODO: put newline handling in one place to ensure both UNIX and Windows
//       styles covered everywhere.

const LEXER_THREAD_NAME: &str = "Sylan Lexer";

/// A lexed token that remembers its position and "trivia". Trivia is whitespace
/// on either side. Tracking this allows tooling to pull apart code, refactor
/// it, and then put it back together without breaking whitespace formatting in
/// the existing source.
#[derive(Clone, Eq, Debug, Default, PartialEq)]
pub struct LexedToken {
    pub position: Position,
    pub trivia: Option<String>,
    pub token: Token,
}

#[derive(Debug)]
pub enum ErrorDescription {
    Described(String),
    Expected(char),
    Unexpected(char),
}

#[derive(Debug)]
pub struct Error {
    position: Position,
    description: ErrorDescription,
}

#[derive(Debug)]
pub enum LexerTaskError {
    Lexer(Error),
    Task(Box<Any + Send + 'static>),
}

type TokenResult = Result<Token, Error>;
type LexedTokenResult = Result<LexedToken, Error>;

type Number = (i64, u64);

/// The task that lexes and emitted a token stream over a channel. It's a lexed token channel
/// combined with a join handle on the underlying thread.
pub struct LexerTask {
    tokens: Receiver<LexedToken>,
    lexer_handle: JoinHandle<Result<(), Error>>,
}

impl LexerTask {
    pub fn join(self) -> Result<(), LexerTaskError> {
        let joined = self.lexer_handle.join();
        match joined {
            Ok(result) => match result {
                Ok(()) => Ok(()),
                Err(err) => Err(LexerTaskError::Lexer(err)),
            },
            Err(err) => Err(LexerTaskError::Task(err)),
        }
    }

    pub fn recv(&self) -> Result<LexedToken, RecvError> {
        self.tokens.recv()
    }
}

impl ExcursionBuffer for LexerTask {
    fn start_excursion(&mut self) -> Self {
        unimplemented!()
    }
}

/// A lexer that is used by a `LexerTask` to produce a stream of tokens. Each lexer has a source
/// code to lex, and a set of character escapes and known keyword mappings to use.
pub struct Lexer {
    source: Source,
    char_escapes: HashMap<char, char>,
    keywords: HashMap<&'static str, Token>,
}

impl From<Source> for Lexer {
    fn from(source: Source) -> Self {
        Self {
            source,
            char_escapes: char_escapes::new(),
            keywords: keywords::new(),
        }
    }
}

impl Lexer {
    /// Fail at lexing, describing the reason why.
    fn fail(&self, description: impl Into<String>) -> TokenResult {
        Err(Error {
            description: ErrorDescription::Described(description.into()),
            position: self.source.position,
        })
    }

    /// Fail at lexing, stating that the `expected` character was expected but
    /// did not appear.
    fn expect(&self, expected: char) -> TokenResult {
        Err(Error {
            description: ErrorDescription::Expected(expected),
            position: self.source.position,
        })
    }

    /// Fail at lexing, stating that the `unexpected` character was unexpected
    /// and therefore cannot be handled.
    fn unexpected(&self, unexpected: char) -> TokenResult {
        Err(Error {
            description: ErrorDescription::Unexpected(unexpected),
            position: self.source.position,
        })
    }

    // The following methods are sub-lexers that are reentrant and handle the
    // lexing of a particular subcontext of the overall source. Each expects
    // the whole context next in the stream, so previous steps working out which
    // sub-lexer to delegate to should use peeks and not reads to discern it
    // from subsequent characters in the buffer.

    fn lex_multi_line_comment(&mut self, buffer: &mut String) -> Option<Error> {
        self.source.discard_many(2);

        let mut nesting_level: usize = 1;
        while 1 <= nesting_level {
            match self.source.read() {
                Some(c) => {
                    if (c == '/') && self.source.next_is('*') {
                        buffer.push('/');
                        buffer.push('*');
                        self.source.discard();
                        nesting_level += 1;
                    } else if (c == '*') && self.source.next_is('/') {
                        if 1 < nesting_level {
                            buffer.push('*');
                            buffer.push('/');
                        }
                        self.source.discard();
                        nesting_level -= 1;
                    } else {
                        buffer.push(c);
                    }
                }
                None => break,
            }
        }

        if 1 <= nesting_level {
            Some(Error {
                description: ErrorDescription::Described(String::from(
                    "premature EOF in multiline comment",
                )),
                position: self.source.position,
            })
        } else {
            None
        }
    }

    fn lex_single_line_comment(&mut self, buffer: &mut String) {
        self.source.discard_many(2);
        while let Some(c) = self.source.read() {
            if (c == '\n') || ((c == '\r') && !self.source.next_is('\n')) {
                break;
            } else if (c == '\r') && self.source.next_is('\n') {
                self.source.discard();
                break;
            } else {
                buffer.push(c);
            }
        }
    }

    fn lex_trivia(&mut self) -> Result<Option<String>, Error> {
        let is_empty = {
            let c = self.source.peek().cloned();

            if c == Some('/') {
                let is_multiline_comment =
                    self.source.nth_is(1, '*') && !self.source.nth_is(2, '*');
                !(is_multiline_comment || self.source.nth_is(1, '/'))
            } else {
                c.filter(|x| x.is_whitespace()).is_none()
            }
        };

        if is_empty {
            Ok(None)
        } else {
            let mut trivia = String::new();
            loop {
                let next_char = self.source.peek().cloned();

                // SyDocs, starting with "/**", are not trivia but meaningful
                // tokens that are stored in the AST. They are skipped in this
                // function.
                if (next_char == Some('/'))
                    && self.source.nth_is(1, '*')
                    && !self.source.nth_is(2, '*')
                {
                    if let Some(err) = self.lex_multi_line_comment(&mut trivia) {
                        break Err(err);
                    }
                } else if (next_char == Some('/')) && self.source.nth_is(1, '/') {
                    self.lex_single_line_comment(&mut trivia)
                } else if let Some((c, true)) = next_char.map(|x| (x, x.is_whitespace())) {
                    trivia.push(c);
                    self.source.discard();
                } else {
                    break Ok(Some(trivia));
                }
            }
        }
    }

    fn lex_version(&mut self) -> TokenResult {
        self.source.discard();

        self.lex_absolute_number()
            .map(|(real, fractional)| {
                // TODO: lex this properly. Unlike an absolute number, it must support more than one
                // decimal place.
                Token::Version(Version {
                    major: real as u64,
                    minor: fractional,
                    patch: 0,
                })
            })
            .map(Ok)
            .unwrap_or_else(|| self.fail("invalid version number"))
    }

    fn lex_number(&mut self) -> TokenResult {
        self.lex_absolute_number()
            .map(|(real, fractional)| Token::Number(real, fractional))
            .map(Ok)
            .unwrap_or_else(|| self.fail("invalid number"))
    }

    fn lex_rest_of_word(&mut self, buffer: &mut String) {
        loop {
            match self.source.peek() {
                Some(&c) if c.is_alphabetic() || c.is_digit(10) => {
                    self.source.discard();
                    buffer.push(c);
                }
                _ => break,
            }
        }
    }

    fn lex_string(&mut self) -> Token {
        self.source.discard();

        let mut string = String::new();
        loop {
            match self.source.read() {
                Some('"') => break,
                Some(c) => string.push(c),
                None => break,
            }
        }
        Token::String(SylanString::from(string))
    }

    fn lex_interpolated_string(&mut self) -> Token {
        self.source.discard();

        let mut string = String::new();
        loop {
            match self.source.read() {
                Some('`') => break,
                Some(c) => string.push(c),
                None => break,
            }
        }
        Token::InterpolatedString(multiphase::InterpolatedString::from(string))
    }

    fn lex_char(&mut self) -> TokenResult {
        self.source.discard();

        match self.source.read() {
            Some(c) => {
                let result = if c == '\\' {
                    match self.source.read() {
                        Some(escaped) => self
                            .char_escapes
                            .get(&escaped)
                            .map_or(self.fail(format!("invalid escape: {}", escaped)), |&c| {
                                Ok(Token::Char(c))
                            }),
                        None => self.fail("escaped character ended prematurely"),
                    }
                } else {
                    Ok(Token::Char(c))
                };
                self.source.discard();
                result
            }
            None => self.fail("character ended prematurely"),
        }
    }

    fn lex_shebang(&mut self) -> TokenResult {
        self.source.discard();

        if let Some('!') = self.source.read() {
            let mut content = String::new();
            loop {
                let next_char = self.source.peek().cloned();
                if (next_char == Some('\r')) && self.source.nth_is(1, '\n') {
                    self.source.discard_many(2);
                    break;
                } else {
                    match next_char {
                        Some('\n') | Some('\r') => {
                            self.source.discard();
                            break;
                        }
                        Some(c) => {
                            self.source.discard();
                            content.push(c);
                        }
                        _ => break,
                    }
                }
            }
            Ok(Token::Shebang(multiphase::Shebang::from(content)))
        } else {
            self.expect('!')
        }
    }

    fn lex_sydoc(&mut self) -> TokenResult {
        self.source.discard_many(3);

        let mut content = String::new();
        loop {
            let next_char = self.source.peek().cloned();

            if (Some('*') == next_char) && self.source.nth_is(1, '/') {
                self.source.discard_many(2);
                break Ok(Token::SyDoc(multiphase::SyDoc::from(content)));
            } else if (Some('/') == next_char) && self.source.nth_is(1, '*') {
                content.push('/');
                content.push('*');
                if let Some(err) = self.lex_multi_line_comment(&mut content) {
                    break Err(err);
                } else {
                    content.push('*');
                    content.push('/');
                }
            } else if let Some(c) = next_char {
                content.push(c);
                self.source.discard();
            } else {
                break self.fail("the file ended before a SyDoc within");
            }
        }
    }

    fn lex_boolean_or_keyword_or_identifier(&self, word: String) -> Token {
        match &word[..] {
            "true" => Token::Boolean(true),
            "false" => Token::Boolean(false),
            _ => match self.keywords.get(&word[..]) {
                Some(token) => token.clone(),
                None => Token::Identifier(multiphase::Identifier::from(word)),
            },
        }
    }

    fn lex_absolute_number(&mut self) -> Option<Number> {
        match self.source.read() {
            Some(c) if c.is_digit(10) || (c == '-') || (c == '+') => {
                let mut real_to_parse = String::new();
                real_to_parse.push(c);
                let mut fractional_to_parse = String::new();

                let mut decimal_place_consumed = false;
                loop {
                    match self.source.peek().cloned() {
                        Some('.') if !decimal_place_consumed => {
                            decimal_place_consumed = true;
                            self.source.discard();
                        }
                        Some(c) if c.is_digit(10) => {
                            if decimal_place_consumed {
                                fractional_to_parse.push(c);
                            } else {
                                real_to_parse.push(c);
                            }
                            self.source.discard();
                        }
                        _ => break,
                    }
                }
                if fractional_to_parse.is_empty() {
                    fractional_to_parse.push('0')
                }

                let real = real_to_parse
                    .parse()
                    .expect("lexed real number component failed to parse");
                let fractional = fractional_to_parse
                    .parse()
                    .expect("lexed fractional number component failed to parse");
                Some((real, fractional))
            }
            _ => None,
        }
    }

    fn lex_with_leading_dot(&mut self) -> Token {
        if self.source.next_is('.') && self.source.nth_is(1, '.') {
            self.source.discard();
            self.source.discard();
            Token::Rest
        } else {
            Token::Dot
        }
    }

    fn lex_operator(&mut self) -> TokenResult {
        if let Some(c) = self.source.read() {
            match c {
                '-' => Ok(self.lex_with_leading_minus()),
                '<' => Ok(self.lex_with_leading_left_angle_bracket()),
                '=' => Ok(self.lex_with_leading_equals()),
                '|' => Ok(self.lex_with_leading_vertical_bar()),
                '&' => Ok(self.lex_with_leading_ampersand()),
                '!' => Ok(self.lex_with_leading_exclamation_mark()),
                '>' => Ok(self.lex_with_leading_right_angle_bracket()),
                ':' => Ok(self.lex_with_leading_colon()),
                '.' => Ok(self.lex_with_leading_dot()),

                ',' => Ok(Token::SubItemSeparator),
                '#' => Ok(Token::Compose),
                '~' => Ok(Token::BitwiseNot),
                '^' => Ok(Token::BitwiseXor),
                '+' => Ok(Token::Add),
                '*' => Ok(Token::Multiply),
                '/' => Ok(Token::Divide),
                '%' => Ok(Token::Modulo),
                '{' => Ok(Token::OpenBrace),
                '}' => Ok(Token::CloseBrace),
                '(' => Ok(Token::OpenParentheses),
                ')' => Ok(Token::CloseParentheses),
                '[' => Ok(Token::OpenSquareBracket),
                ']' => Ok(Token::CloseSquareBracket),

                unknown => self.fail(format!("unknown operator: {}", unknown)),
            }
        } else {
            self.fail("file ended before an operator could be read")
        }
    }

    fn lex_with_leading_minus(&mut self) -> Token {
        if self.source.next_is('>') {
            self.source.discard();
            Token::LambdaArrow
        } else {
            Token::Subtract
        }
    }

    fn lex_with_leading_left_angle_bracket(&mut self) -> Token {
        match self.source.peek().cloned() {
            Some('-') => {
                self.source.discard();
                Token::Bind
            }
            Some('<') => {
                self.source.discard();
                Token::ShiftLeft
            }
            Some('=') => {
                self.source.discard();
                Token::LessThanOrEquals
            }
            _ => Token::LessThan,
        }
    }

    fn lex_with_leading_equals(&mut self) -> Token {
        if self.source.next_is('=') {
            self.source.discard();
            Token::Equals
        } else {
            Token::Assign
        }
    }

    fn lex_with_leading_exclamation_mark(&mut self) -> Token {
        if self.source.next_is('=') {
            self.source.discard();
            Token::NotEquals
        } else {
            Token::Not
        }
    }

    fn lex_with_leading_right_angle_bracket(&mut self) -> Token {
        match self.source.peek().cloned() {
            Some('>') => {
                self.source.discard();
                Token::DoubleRightAngleBracket
            }
            Some('=') => {
                self.source.discard();
                Token::GreaterThanOrEquals
            }
            _ => Token::GreaterThan,
        }
    }

    fn lex_with_leading_vertical_bar(&mut self) -> Token {
        match self.source.peek().cloned() {
            Some('|') => {
                self.source.discard();
                Token::Or
            }
            Some('>') => {
                self.source.discard();
                Token::Pipe
            }
            _ => Token::BitwiseOr,
        }
    }

    fn lex_with_leading_ampersand(&mut self) -> Token {
        if self.source.next_is('&') {
            self.source.discard();
            Token::And
        } else {
            Token::BitwiseAnd
        }
    }

    fn lex_with_leading_colon(&mut self) -> Token {
        if self.source.next_is(':') {
            self.source.discard();
            Token::MethodHandle
        } else {
            Token::Colon
        }
    }

    fn lex_non_trivia(&mut self) -> TokenResult {
        match self.source.peek() {
            None => Ok(Token::Eof),
            Some(&c) => {
                if (c == 'v') && self.source.match_nth(1, |c| c.is_digit(10)) {
                    self.lex_version()
                } else if (c == '/') && self.source.nth_is(1, '*') && self.source.nth_is(2, '*') {
                    self.lex_sydoc()
                } else {
                    match c {
                        '"' => Ok(self.lex_string()),
                        '`' => Ok(self.lex_interpolated_string()),
                        '\'' => self.lex_char(),
                        '#' if self.source.at_start() => self.lex_shebang(),
                        _ => {
                            if c.is_alphabetic() {
                                let mut rest = String::new();
                                self.lex_rest_of_word(&mut rest);
                                Ok(self.lex_boolean_or_keyword_or_identifier(rest))
                            } else if c.is_digit(10)
                                || (self.source.match_nth(1, |c| c.is_digit(10))
                                    && ((c == '+') || (c == '-')))
                            {
                                self.lex_number()
                            } else {
                                self.lex_operator()
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn lex_next(&mut self) -> LexedTokenResult {
        match self.lex_trivia() {
            Ok(trivia) => {
                let position = self.source.position;
                let token = self.lex_non_trivia();
                token.map(|t| LexedToken {
                    token: t,
                    position,
                    trivia,
                })
            }
            Err(err) => Err(err),
        }
    }

    /// Start lexing from the top-level of the source, returning a lexing task running concurrently
    /// in another thread and feeding tokens through a channel as it goes.
    pub fn lex(mut self) -> io::Result<LexerTask> {
        let (tx, rx) = channel();
        let thread = thread::Builder::new().name(LEXER_THREAD_NAME.to_string());

        let handle = thread.spawn(move || loop {
            match self.lex_next() {
                Ok(token) => {
                    let is_eof = token.token == Token::Eof;
                    tx.send(token).unwrap();
                    if is_eof {
                        break Ok(());
                    }
                }
                Err(e) => break Err(e),
            }
        });

        handle.map(|h| LexerTask {
            tokens: rx,
            lexer_handle: h,
        })
    }
}

#[cfg(test)]
mod tests {
    use common::multiphase::{Identifier, InterpolatedString, Shebang, SyDoc};

    use super::*;

    fn test_lexer(s: &str) -> Lexer {
        let source_chars = s.chars().collect::<Vec<char>>();
        Lexer::from(Source::from(source_chars))
    }

    fn assert_next(lexer: &mut Lexer, token: &Token) {
        match lexer.lex_next() {
            Ok(LexedToken { token: t, .. }) => {
                assert_eq!(t, *token);
            }
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn empty() {
        let mut lexer = test_lexer("    \t  \n      ");
        assert_next(&mut lexer, &Token::Eof);
    }

    #[test]
    fn identifier() {
        let mut lexer = test_lexer("    \t  \r      abc");
        assert_next(&mut lexer, &Token::Identifier(Identifier::from("abc")));
    }

    #[test]
    fn keywords() {
        let mut lexer = test_lexer("    class\t  \r\n  abc var do");
        assert_next(&mut lexer, &Token::Class);
        assert_next(&mut lexer, &Token::Identifier(Identifier::from("abc")));
        assert_next(&mut lexer, &Token::Var);
        assert_next(&mut lexer, &Token::Do);
    }

    #[test]
    fn numbers() {
        let mut lexer = test_lexer("    23  \t     \t\t\n   23   +32 0.32    \t123123123.32");
        assert_next(&mut lexer, &Token::Number(23, 0));
        assert_next(&mut lexer, &Token::Number(23, 0));
        assert_next(&mut lexer, &Token::Number(32, 0));
        assert_next(&mut lexer, &Token::Number(0, 32));
        assert_next(&mut lexer, &Token::Number(123_123_123, 32));
    }

    #[test]
    fn chars() {
        let mut lexer = test_lexer("  'a'   \t \n\r\n 'd'    '/'");
        assert_next(&mut lexer, &Token::Char('a'));
        assert_next(&mut lexer, &Token::Char('d'));
        assert_next(&mut lexer, &Token::Char('/'));
    }

    #[test]
    fn strings() {
        let mut lexer = test_lexer("  \"abcdef\"   \t \n\n\n\"'123'\"");
        assert_next(&mut lexer, &Token::String(SylanString::from("abcdef")));
        assert_next(&mut lexer, &Token::String(SylanString::from("'123'")));
    }

    #[test]
    fn interpolated_strings() {
        // TODO: test actual interpolation once the parser is complete.

        let mut lexer = test_lexer("   `123`   `abc`");
        assert_next(
            &mut lexer,
            &Token::InterpolatedString(InterpolatedString::from("123")),
        );
        assert_next(
            &mut lexer,
            &Token::InterpolatedString(InterpolatedString::from("abc")),
        );
    }

    #[test]
    fn operators() {
        let mut lexer = test_lexer("   <= \t  \r\n ~ ! ^   >> != |> # :: ");
        assert_next(&mut lexer, &Token::LessThanOrEquals);
        assert_next(&mut lexer, &Token::BitwiseNot);
        assert_next(&mut lexer, &Token::Not);
        assert_next(&mut lexer, &Token::BitwiseXor);
        assert_next(&mut lexer, &Token::DoubleRightAngleBracket);
        assert_next(&mut lexer, &Token::NotEquals);
        assert_next(&mut lexer, &Token::Pipe);
        assert_next(&mut lexer, &Token::Compose);
        assert_next(&mut lexer, &Token::MethodHandle);
    }

    #[test]
    fn single_line_comments() {
        let mut lexer = test_lexer("      //    //  abc   ");
        assert_next(&mut lexer, &Token::Eof);
    }

    #[test]
    fn multi_line_comments() {
        let mut lexer = test_lexer("  /*   /* 123 */      */ ");
        assert_next(&mut lexer, &Token::Eof);
    }

    #[test]
    fn booleans() {
        let mut lexer = test_lexer("  true false   \n\t   /*   */ false true");
        assert_next(&mut lexer, &Token::Boolean(true));
        assert_next(&mut lexer, &Token::Boolean(false));
        assert_next(&mut lexer, &Token::Boolean(false));
        assert_next(&mut lexer, &Token::Boolean(true));
    }

    #[test]
    fn version() {
        let mut lexer = test_lexer("v10.23");
        assert_next(
            &mut lexer,
            &Token::Version(Version {
                major: 10,
                minor: 23,
                patch: 0,
            }),
        );
    }

    #[test]
    fn rest() {
        let mut lexer = test_lexer(" . .. ... .. .");

        assert_next(&mut lexer, &Token::Dot);
        assert_next(&mut lexer, &Token::Dot);
        assert_next(&mut lexer, &Token::Dot);

        assert_next(&mut lexer, &Token::Rest);

        assert_next(&mut lexer, &Token::Dot);
        assert_next(&mut lexer, &Token::Dot);
        assert_next(&mut lexer, &Token::Dot);
    }

    #[test]
    fn shebang() {
        let mut lexer = test_lexer("#!/usr/bin/env sylan");
        let shebang = Token::Shebang(Shebang::from("/usr/bin/env sylan"));
        assert_next(&mut lexer, &shebang);

        let mut lexer2 = test_lexer("#!/usr/bin sylan\r\ntrue false");
        let shebang2 = Token::Shebang(Shebang::from("/usr/bin sylan"));
        assert_next(&mut lexer2, &shebang2);
        assert_next(&mut lexer2, &Token::Boolean(true));

        let mut lexer3 = test_lexer("#!/usr/local/bin/env sylan\n123 321");
        let shebang3 = Token::Shebang(Shebang::from("/usr/local/bin/env sylan"));
        assert_next(&mut lexer3, &shebang3);
        assert_next(&mut lexer3, &Token::Number(123, 0));
    }

    #[test]
    fn sydoc() {
        let mut lexer = test_lexer("/* comment */ // \n /** A SyDoc /* comment. */ */");
        let sydoc = Token::SyDoc(SyDoc::from(" A SyDoc /* comment. */ "));
        assert_next(&mut lexer, &sydoc);
    }
}
