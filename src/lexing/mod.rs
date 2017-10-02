mod keywords;
mod char_escapes;

pub mod source;
pub mod tokens;

use std::collections::HashMap;

use self::tokens::Token;
use self::source::Source;

#[derive(Debug)]
pub struct LexedToken {
    pub position: usize,
    pub trivia: String,
    pub token: Token,
}

#[derive(Debug)]
pub struct Error {
    position: usize,
    description: String,
}

type TokenResult = Result<Token, Error>;

pub struct Lexer {
    source: Source,
    char_escapes: HashMap<char, char>,
    keywords: HashMap<&'static str, Token>,
}

impl Lexer {
    pub fn from(source: Source) -> Self {
        Self {
            source,
            char_escapes: char_escapes::new(),
            keywords: keywords::new(),
        }
    }

    fn fail(&self, description: &str) -> TokenResult {
        Err(Error {
            description: String::from(description),
            position: self.source.position,
        })
    }

    fn lex_multi_line_comment(&mut self, buffer: &mut String) {
        self.source.discard_many(2);
        let mut nesting_level: usize = 1;
        while 1 <= nesting_level {
            match self.source.read() {
                Some(c) => {
                    match c {
                        '/' if self.source.nth_is(0, '*') => {
                            buffer.push('/');
                            buffer.push('*');
                            self.source.discard();
                            nesting_level += 1;
                        }
                        '*' if self.source.nth_is(0, '/') => {
                            if 1 < nesting_level {
                                buffer.push('*');
                                buffer.push('/');
                            }
                            self.source.discard();
                            nesting_level -= 1;
                        }
                        _ => buffer.push(c),
                    }
                }
                None => break,
            }
        }
    }

    fn lex_single_line_comment(&mut self, buffer: &mut String) {
        self.source.discard_many(2);
        loop {
            match self.source.read() {
                Some('\n') => break,
                Some('\r') if self.source.nth_is(1, '\n') => break,
                Some(c) => buffer.push(c),
                None => break,
            }
        }
    }

    fn lex_trivia(&mut self) -> String {
        let mut trivia = String::new();
        loop {
            match self.source.peek() {
                Some('/') if self.source.nth_is(1, '*') => self.lex_multi_line_comment(&mut trivia),
                Some('/') if self.source.nth_is(1, '/') => {
                    self.lex_single_line_comment(&mut trivia)
                }
                Some(c) if c.is_whitespace() => {
                    trivia.push(c);
                    self.source.discard();
                }
                _ => break,
            }
        }
        trivia
    }

    fn lex_version(&mut self) -> TokenResult {
        self.source.discard();

        self.lex_absolute_number()
            .map(Token::Version)
            .map(Ok)
            .unwrap_or(self.fail("invalid version number"))
    }

    fn lex_number(&mut self) -> TokenResult {
        self.lex_absolute_number()
            .map(Token::Number)
            .map(Ok)
            .unwrap_or(self.fail("invalid number"))
    }

    fn lex_rest_of_word(&mut self, buffer: &mut String) {
        loop {
            match self.source.peek() {
                Some(c) if c.is_alphabetic() || c.is_digit(10) || (c == '_') => {
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
        Token::String(string)
    }

    fn lex_interpolated_string(&mut self) -> Token {
        // Just lex the whole string for now; reenter the lexer from the parser
        // when doing the interpolation.

        self.source.discard();

        let mut string = String::new();
        loop {
            match self.source.read() {
                Some('`') => break,
                Some(c) => string.push(c),
                None => break,
            }
        }
        Token::InterpolatedString(string)
    }

    fn lex_char(&mut self) -> TokenResult {
        self.source.discard();
        match self.source.read() {
            Some(c) => {
                let result = if c == '\\' {
                    match self.source.read() {
                        Some(escaped) => {
                            self.char_escapes.get(&escaped).map_or(
                                self.fail("invalid escape"),
                                |&c| Ok(Token::Char(c)),
                            )
                        }
                        None => self.fail("escaped char ended prematurely"),
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

    fn lex_boolean_or_keyword_or_identifier(&self, word: String) -> Token {
        match &word[..] {
            "true" => Token::Boolean(true),
            "false" => Token::Boolean(false),
            _ => {
                match self.keywords.get(&word[..]) {
                    Some(token) => token.clone(),
                    None => Token::Identifier(word),
                }
            }
        }
    }

    fn lex_absolute_number(&mut self) -> Option<f64> {
        match self.source.read() {
            Some(c) if c.is_digit(10) || (c == '-') || (c == '+') => {
                let mut to_parse = String::new();
                to_parse.push(c);

                let mut decimal_place_consumed = false;
                loop {
                    match self.source.peek() {
                        Some('.') if !decimal_place_consumed => {
                            decimal_place_consumed = true;
                            to_parse.push('.');
                            self.source.discard();
                        }
                        Some(c) if c.is_digit(10) => {
                            to_parse.push(c);
                            self.source.discard();
                        }
                        _ => break,
                    }
                }

                Some(to_parse.parse().expect("lexed number failed to parse"))
            }
            _ => None,
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

                ',' => Ok(Token::SubItemSeparator),
                '.' => Ok(Token::Dot),
                '~' => Ok(Token::BitwiseNot),
                '^' => Ok(Token::BitwiseXor),
                '+' => Ok(Token::Add),
                '*' => Ok(Token::Multiply),
                '/' => Ok(Token::Divide),
                '%' => Ok(Token::Modulo),
                ':' => Ok(Token::Colon),
                '{' => Ok(Token::OpenBrace),
                '}' => Ok(Token::CloseBrace),
                '(' => Ok(Token::OpenParentheses),
                ')' => Ok(Token::CloseParentheses),
                '[' => Ok(Token::OpenSquareBracket),
                ']' => Ok(Token::CloseSquareBracket),

                _ => self.fail("unknown operator"),
            }
        } else {
            self.fail("premature EOF")
        }
    }

    fn lex_with_leading_minus(&mut self) -> Token {
        if let Some('>') = self.source.peek() {
            self.source.discard();
            Token::LambdaArrow
        } else {
            Token::Subtract
        }
    }

    fn lex_with_leading_left_angle_bracket(&mut self) -> Token {
        match self.source.peek() {
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
        if let Some('=') = self.source.peek() {
            self.source.discard();
            Token::Equals
        } else {
            Token::Assign
        }
    }

    fn lex_with_leading_exclamation_mark(&mut self) -> Token {
        if let Some('=') = self.source.peek() {
            self.source.discard();
            Token::NotEquals
        } else {
            Token::Not
        }
    }

    fn lex_with_leading_right_angle_bracket(&mut self) -> Token {
        match self.source.peek() {
            Some('>') => {
                self.source.discard();
                Token::ShiftRight
            }
            Some('=') => {
                self.source.discard();
                Token::GreaterThanOrEquals
            }
            _ => Token::GreaterThan,
        }
    }

    fn lex_with_leading_vertical_bar(&mut self) -> Token {
        if let Some('|') = self.source.peek() {
            self.source.discard();
            Token::Or
        } else {
            Token::BitwiseOr
        }
    }

    fn lex_with_leading_ampersand(&mut self) -> Token {
        if let Some('&') = self.source.peek() {
            self.source.discard();
            Token::And
        } else {
            Token::BitwiseAnd
        }
    }

    pub fn lex(&mut self) -> Result<LexedToken, Error> {
        let trivia = self.lex_trivia();
        let position = self.source.position;

        let token = match self.source.peek() {
            None => Ok(Token::Eof),
            Some(c) => {
                match c {
                    'v' if self.source.match_nth(1, |c| c.is_digit(10)) => self.lex_version(),
                    '"' => Ok(self.lex_string()),
                    '`' => Ok(self.lex_interpolated_string()),
                    '\'' => self.lex_char(),
                    _ => {
                        if c.is_alphabetic() || (c == '_') {
                            let mut rest = String::new();
                            self.lex_rest_of_word(&mut rest);
                            Ok(self.lex_boolean_or_keyword_or_identifier(rest))
                        } else if c.is_digit(10) ||
                                   (self.source.match_nth(1, |c| c.is_digit(10)) &&
                                        ((c == '+') || (c == '-')))
                        {
                            self.lex_number()
                        } else {
                            self.lex_operator()
                        }
                    }
                }
            }
        };

        token.map(|t| {
            LexedToken {
                token: t,
                position,
                trivia,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_lexer(s: &str) -> Lexer {
        Lexer::from(Source::from(s.chars().collect()))
    }

    fn assert_next(lexer: &mut Lexer, token: Token) {
        match lexer.lex() {
            Ok(LexedToken { token: ref t, .. }) => {
                assert_eq!(*t, token);
            }
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_empty() {
        let mut lexer = test_lexer("    \t  \n      ");
        assert_next(&mut lexer, Token::Eof);
    }

    #[test]
    fn test_identifier() {
        let mut lexer = test_lexer("    \t  \n      abc");
        assert_next(&mut lexer, Token::Identifier(String::from("abc")));
    }

    #[test]
    fn test_keywords() {
        let mut lexer = test_lexer("    class\t  \n  public    abc do");
        assert_next(&mut lexer, Token::Class);
        assert_next(&mut lexer, Token::Public);
        assert_next(&mut lexer, Token::Identifier(String::from("abc")));
        assert_next(&mut lexer, Token::Do);
    }

    #[test]
    fn test_numbers() {
        let mut lexer = test_lexer("    23  \t     \t\t\n   23   +32 0.32    \t123123123.32");
        assert_next(&mut lexer, Token::Number(23.0));
        assert_next(&mut lexer, Token::Number(23.0));
        assert_next(&mut lexer, Token::Number(32.0));
        assert_next(&mut lexer, Token::Number(0.32));
        assert_next(&mut lexer, Token::Number(123123123.32));
    }

    #[test]
    fn test_chars() {
        let mut lexer = test_lexer("  'a'   \t \n\n\n 'd'    '/'");
        assert_next(&mut lexer, Token::Char('a'));
        assert_next(&mut lexer, Token::Char('d'));
        assert_next(&mut lexer, Token::Char('/'));
    }

    #[test]
    fn test_strings() {
        let mut lexer = test_lexer("  \"abcdef\"   \t \n\n\n\"'123'\"");
        assert_next(&mut lexer, Token::String(String::from("abcdef")));
        assert_next(&mut lexer, Token::String(String::from("'123'")));
    }

    #[test]
    fn test_interpolated_strings() {
        // TODO: test actual interpolation once the parser is complete.

        let mut lexer = test_lexer("   `123`   `abc`");
        assert_next(&mut lexer, Token::InterpolatedString(String::from("123")));
        assert_next(&mut lexer, Token::InterpolatedString(String::from("abc")));
    }

    #[test]
    fn test_operators() {
        let mut lexer = test_lexer("   <= \t  \n ~ ! ^   >> != ");
        assert_next(&mut lexer, Token::LessThanOrEquals);
        assert_next(&mut lexer, Token::BitwiseNot);
        assert_next(&mut lexer, Token::Not);
        assert_next(&mut lexer, Token::BitwiseXor);
        assert_next(&mut lexer, Token::ShiftRight);
        assert_next(&mut lexer, Token::NotEquals);
    }

    #[test]
    fn test_single_line_comments() {
        let mut lexer = test_lexer("      //    //  abc   ");
        assert_next(&mut lexer, Token::Eof);
    }

    #[test]
    fn test_multi_line_comments() {
        let mut lexer = test_lexer("  /*   /* 123 */      */ ");
        assert_next(&mut lexer, Token::Eof);
    }

    #[test]
    fn test_booleans() {
        let mut lexer = test_lexer("  true false   \n\t   /*   */ false true");
        assert_next(&mut lexer, Token::Boolean(true));
        assert_next(&mut lexer, Token::Boolean(false));
        assert_next(&mut lexer, Token::Boolean(false));
        assert_next(&mut lexer, Token::Boolean(true));
    }

    #[test]
    fn test_version() {
        let mut lexer = test_lexer("v10.23");
        assert_next(&mut lexer, Token::Version(10.23));
    }
}
