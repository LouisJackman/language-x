use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::mpsc::{channel, Receiver, RecvError, SendError};
use std::thread::{self, JoinHandle};

use crate::common::multiphase::{
    self, Identifier, InterpolatedString, Number, OverloadableInfixOperator,
    OverloadableSliceOperator, PostfixOperator, PseudoIdentifier, SylanString,
};
use crate::common::newlines::{check_newline, NewLine};
use crate::common::peekable_buffer::PeekableBuffer;
use crate::common::string_matches_char_slice;
use crate::common::version::Version;
use crate::lexing::tokens::{Binding, Grouping, Literal, Macros, Token};
use crate::lexing::{char_escapes, keywords, non_word_chars};
use crate::source::in_memory::Source;
use crate::source::Position;

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
    PrematureEof,
    ChannelFailure(String),
    MalformedNumber(String),
}

#[derive(Debug)]
pub struct Error {
    position: Position,
    description: ErrorDescription,
}

#[derive(Debug)]
pub enum LexerTaskError {
    Lexer(Error),
    Task(Box<dyn Any + Send + 'static>),
}

type TokenResult = Result<Token, Error>;
type LexedTokenResult = Result<LexedToken, Error>;

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

fn is_start_of_literal_with_escapes(c: char) -> bool {
    (c == '\'') || (c == '"') || (c == '$') || (c == '`')
}

struct CachedStringPrefixes {
    package_prefix_str: Vec<char>,
    module_prefix_str: Vec<char>,
}

struct LexerCache {
    string_prefixes: CachedStringPrefixes,
    char_escapes: HashMap<char, char>,
    keywords: HashMap<&'static str, Token>,
    non_word_chars: HashSet<char>,
}

/// A lexer that is used by a `LexerTask` to produce a stream of tokens. Each lexer has a source
/// code to lex, and a set of character escapes and known keyword mappings to use.
pub struct Lexer {
    source: Source,
    cache: LexerCache,
}

impl From<Source> for Lexer {
    fn from(source: Source) -> Self {
        Self {
            source,
            cache: LexerCache {
                char_escapes: char_escapes::new(),
                keywords: keywords::new(),
                non_word_chars: non_word_chars::new(),
                string_prefixes: CachedStringPrefixes {
                    package_prefix_str: ".package".chars().collect(),
                    module_prefix_str: ".module".chars().collect(),
                },
            },
        }
    }
}

impl Lexer {
    /// Fail at lexing, describing the reason why.
    fn fail<T>(&self, description: impl Into<String>) -> Result<T, Error> {
        Err(Error {
            description: ErrorDescription::Described(description.into()),
            position: self.source.position,
        })
    }

    /// Fail at lexing, stating that the `expected` character was expected but
    /// did not appear.
    fn expect<T>(&self, expected: char) -> Result<T, Error> {
        Err(Error {
            description: ErrorDescription::Expected(expected),
            position: self.source.position,
        })
    }

    /// Discard the next read character in the stream if it matches the expected
    /// character. Otherwise fail at lexing, stating that the `expected` character was
    /// expected but did not appear.
    fn expect_and_discard(&mut self, expected: char) -> Result<(), Error> {
        if let Some(c) = self.source.read() {
            if c == expected {
                Ok(())
            } else {
                self.expect(expected)
            }
        } else {
            Err(self.premature_eof())
        }
    }

    /// Fail at lexing, stating that the `unexpected` character was unexpected
    /// and therefore cannot be handled.
    fn unexpected<T>(&self, unexpected: char) -> Result<T, Error> {
        Err(Error {
            description: ErrorDescription::Unexpected(unexpected),
            position: self.source.position,
        })
    }

    /// Fail at lexing because an EOF was encountered unexpectedly.
    fn premature_eof(&self) -> Error {
        Error {
            description: ErrorDescription::PrematureEof,
            position: self.source.position,
        }
    }

    fn error(&self, description: ErrorDescription) -> Error {
        Error {
            description,
            position: self.source.position,
        }
    }

    fn send_error<T>(&self, token: &LexedToken, err: &SendError<T>) -> Error {
        Error {
            position: self.source.position,
            description: ErrorDescription::ChannelFailure(format!(
                "the token channel failed to send token {:?}: {}",
                token, err
            )),
        }
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

                        // Treat SyDoc comments inside other comments or SyDocs
                        // as multiline comments. Avoid tripping up on
                        // accidentally parsing an empty multiline comment
                        // `/**/` as the start of an embedded JavaDoc starting
                        // with a `/`.
                        if self.source.next_is('*') && !self.source.nth_is(1, '/') {
                            buffer.push('*');
                            self.source.discard();
                        }

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
            Some(self.premature_eof())
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
            .map(|Number(real, fractional)| {
                // TODO: lex this properly. Unlike an absolute number, it must support more than one
                // decimal place.
                Token::Version(Version {
                    major: real as u64,
                    minor: fractional,
                    patch: 0,
                })
            })
            .map(Ok)
            .unwrap_or_else(|_| self.fail("invalid version number"))
    }

    fn lex_number(&mut self) -> TokenResult {
        self.lex_absolute_number()
            .map(|Number(real, fractional)| {
                Token::Literal(Literal::Number(Number(real, fractional)))
            })
            .map(Ok)
            .unwrap_or_else(|_| self.fail("invalid number"))
    }

    fn lex_rest_of_word(&mut self, buffer: &mut String) {
        loop {
            match self.source.peek() {
                Some(&c) if !(c.is_whitespace() || self.cache.non_word_chars.contains(&c)) => {
                    self.source.discard();
                    buffer.push(c);
                }
                _ => break,
            }
        }
    }

    fn lex_multiphase_identifier(&mut self) -> multiphase::Identifier {
        let mut word = String::new();
        self.lex_rest_of_word(&mut word);
        multiphase::Identifier::from(word)
    }

    fn lex_identifier(&mut self) -> Token {
        Token::Identifier(self.lex_multiphase_identifier())
    }

    fn lex_escape_char_in_string_or_char(&mut self) -> Result<char, Error> {
        self.source.discard();

        match self.source.read() {
            Some(escaped) => self
                .cache
                .char_escapes
                .get(&escaped)
                .map_or(self.fail(format!("invalid escape: {}", escaped)), |&c| {
                    Ok(c)
                }),
            None => Err(self.premature_eof()),
        }
    }

    fn lex_string_content(
        &mut self,
        delimiter: char,
        delimiter_count: usize,
        escaping: bool,
    ) -> Result<String, Error> {
        let mut string = String::new();
        loop {
            match self.source.peek() {
                Some(&c) if c == delimiter => {
                    self.source.discard();

                    let closing_delimiter_encountered = self
                        .source
                        .peek_many(delimiter_count - 1)
                        .filter(|chars| chars.iter().all(|&c| c == delimiter))
                        .is_some();

                    if closing_delimiter_encountered {
                        self.source.discard_many(delimiter_count - 1);
                        break Ok(string);
                    } else {
                        string.push(c);
                    }
                }
                Some(&c) => {
                    let maybe_escaped = if (c == '\\') && escaping {
                        self.lex_escape_char_in_string_or_char()?
                    } else {
                        self.source.discard();
                        c
                    };
                    string.push(maybe_escaped)
                }
                None => break Err(self.premature_eof()),
            }
        }
    }

    fn lex_interpolated_string_content(
        &mut self,
        delimiter: char,
        delimiter_count: usize,
        escaping: bool,
    ) -> Result<InterpolatedString, Error> {
        let mut string_fragments = vec!["".to_owned()];
        let mut interpolations = Vec::new();
        let mut start_new_fragment = false;

        loop {
            match self.source.peek() {
                Some(&c) if c == delimiter => {
                    self.source.discard();

                    let closing_delimiter_encountered = self
                        .source
                        .peek_many(delimiter_count - 1)
                        .filter(|chars| chars.iter().all(|&c| c == delimiter))
                        .is_some();

                    if closing_delimiter_encountered {
                        self.source.discard_many(delimiter_count - 1);
                        break Ok(InterpolatedString {
                            string_fragments,
                            interpolations,
                        });
                    } else {
                        if start_new_fragment {
                            string_fragments.push("".to_owned());
                            start_new_fragment = false;
                        }
                        string_fragments.last_mut().unwrap().push(c);
                    }
                }
                Some(&c) if c == '{' => {
                    let escaped = self.source.nth_is(1, '{');

                    if escaped {
                        self.source.discard();
                        self.source.discard();

                        if start_new_fragment {
                            string_fragments.push("".to_owned());
                            start_new_fragment = false;
                        }
                        let last_fragment = string_fragments.last_mut().unwrap();
                        last_fragment.push('{');
                        last_fragment.push('{');
                    } else {
                        self.source.discard();

                        let identifier = self.lex_multiphase_identifier();
                        self.expect_and_discard('}')?;
                        interpolations.push(identifier);
                        start_new_fragment = true;
                    }
                }
                Some(&c) => {
                    let maybe_escaped = if (c == '\\') && escaping {
                        self.lex_escape_char_in_string_or_char()?
                    } else {
                        self.source.discard();
                        c
                    };
                    if start_new_fragment {
                        string_fragments.push("".to_owned());
                        start_new_fragment = false;
                    }
                    string_fragments.last_mut().unwrap().push(maybe_escaped);
                }
                None => break Err(self.premature_eof()),
            }
        }
    }

    fn lex_string(&mut self, escaping: bool) -> TokenResult {
        self.source.discard();
        let string = self.lex_string_content('"', 1, escaping)?;
        Ok(Token::Literal(Literal::String(SylanString::from(string))))
    }

    fn lex_quoted_identifier(&mut self, escaping: bool) -> TokenResult {
        self.source.discard();
        let string = self.lex_string_content('`', 1, escaping)?;
        Ok(Token::Identifier(Identifier::from(string)))
    }

    fn lex_interpolated_string(&mut self, escaping: bool) -> TokenResult {
        self.source.discard();
        self.source.discard();
        let string = self.lex_interpolated_string_content('"', 1, escaping)?;
        Ok(Token::Literal(Literal::InterpolatedString(string)))
    }

    fn lex_string_with_custom_delimiter(&mut self, escaping: bool) -> TokenResult {
        self.source.discard();
        self.source.discard();
        self.source.discard();

        let mut additional_delimiter_count = 0;
        while self.source.peek() == Some(&'"') {
            self.source.discard();
            additional_delimiter_count += 1;
        }

        let string = self.lex_string_content('"', additional_delimiter_count + 3, escaping)?;
        Ok(Token::Literal(Literal::String(SylanString::from(string))))
    }

    fn lex_quoted_identifier_with_custom_delimiter(&mut self, escaping: bool) -> TokenResult {
        self.source.discard();
        self.source.discard();
        self.source.discard();

        let mut additional_delimiter_count = 0;
        while self.source.peek() == Some(&'`') {
            self.source.discard();
            additional_delimiter_count += 1;
        }

        let string = self.lex_string_content('`', additional_delimiter_count + 3, escaping)?;
        Ok(Token::Identifier(Identifier::from(string)))
    }

    fn lex_interpolated_string_with_custom_delimiter(&mut self, escaping: bool) -> TokenResult {
        self.source.discard();
        self.source.discard();
        self.source.discard();
        self.source.discard();

        let mut additional_delimiter_count = 0;
        while self.source.peek() == Some(&'"') {
            self.source.discard();
            additional_delimiter_count += 1;
        }

        let string =
            self.lex_interpolated_string_content('"', additional_delimiter_count + 3, escaping)?;
        Ok(Token::Literal(Literal::InterpolatedString(string)))
    }

    fn lex_char(&mut self, escaping: bool) -> TokenResult {
        self.source.discard();

        match self.source.peek() {
            Some(&c) => {
                let result = if escaping && (c == '\\') {
                    self.lex_escape_char_in_string_or_char()
                        .map(|c| Token::Literal(Literal::Char(c)))
                } else {
                    self.source.discard();
                    Ok(Token::Literal(Literal::Char(c)))
                };

                // Discard the closing '.
                self.source.discard();

                result
            }
            None => Err(self.premature_eof()),
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
            } else if let (Some(c), next) = (next_char, self.source.peek_nth(1)) {
                if let Some(newline) = check_newline(c, next.cloned()) {
                    // Newlines are unwanted in SyDoc
                    if newline == NewLine::CarridgeReturnLineFeed {
                        self.source.discard();
                    }
                    self.source.discard();

                    // Leading whitespace in SyDoc is likely due to indenting
                    // and not intended to be in the result.
                    while self.source.match_next(|c| c.is_whitespace()) {
                        self.source.discard();
                    }

                    // Leading asterisks are often used to make JavaDocs look
                    // pretty, and SyDoc follows this. Throw them away in the
                    // result though.
                    //
                    // They're also the only way to put leading indents in
                    // multiline SyDoc.
                    //
                    // Don't mix up a leading asterisk with the end of SyDoc.
                    if self.source.next_is('*') && !self.source.nth_is(1, '/') {
                        self.source.discard();

                        // Leave an idiomatic leading whitespace after the `*`
                        // in the buffer. Why? Because a newline was cut out
                        // earlier, and the developer won't expect two words on
                        // separate lines to be glued together. Let's also
                        // assume that automatic formatters and editors will
                        // throw away trailing whitespace, including those at
                        // the end of SyDoc lines.
                    }
                } else {
                    content.push(c);
                    self.source.discard();
                }
            } else {
                break self.fail("the file ended before a SyDoc within");
            }
        }
    }

    fn lex_phrase(&mut self, word: String) -> TokenResult {
        match self.cache.keywords.get(&word[..]) {
            Some(Token::PseudoIdentifier(PseudoIdentifier::This)) => self.lex_rest_of_this(),
            Some(token) => Ok(token.clone()),
            None => Ok(Token::Identifier(multiphase::Identifier::from(word))),
        }
    }

    fn lex_rest_of_this(&mut self) -> TokenResult {
        let package_prefix_str = &self.cache.string_prefixes.package_prefix_str;
        let module_prefix_str = &self.cache.string_prefixes.module_prefix_str;
        let package_prefix_lookahead = package_prefix_str.len() + 1;

        let ahead = match self.source.peek_many(package_prefix_lookahead) {
            Some(ahead) => ahead,
            None => Err(self.premature_eof())?,
        };

        let result = if ahead.starts_with(package_prefix_str)
            && !ahead[package_prefix_str.len()].is_alphanumeric()
        {
            self.source.discard_many(package_prefix_lookahead);
            Token::PseudoIdentifier(PseudoIdentifier::ThisPackage)
        } else if ahead.starts_with(module_prefix_str)
            && !ahead[module_prefix_str.len()].is_alphanumeric()
        {
            self.source.discard_many(module_prefix_str.len() + 1);
            Token::PseudoIdentifier(PseudoIdentifier::ThisModule)
        } else {
            Token::PseudoIdentifier(PseudoIdentifier::This)
        };

        Ok(result)
    }

    fn lex_absolute_number(&mut self) -> Result<Number, Error> {
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

                real_to_parse
                    .parse()
                    .map_err(|err| {
                        self.error(ErrorDescription::MalformedNumber(format!(
                            "lexed real number component {} failed to parse: {}",
                            real_to_parse, err
                        )))
                    })
                    .and_then(|real| {
                        fractional_to_parse
                            .parse()
                            .map_err(|err| {
                                self.error(ErrorDescription::MalformedNumber(format!(
                                    "lexed fractional number component {} failed to parse: {}",
                                    real_to_parse, err
                                )))
                            })
                            .map(|fractional| Number(real, fractional))
                    })
            }
            _ => Err(self.premature_eof()),
        }
    }

    fn lex_symbolic(&mut self) -> TokenResult {
        if let Some(c) = self.source.peek().cloned() {
            match c {
                // The Infix Operators
                '.' => Ok(self.lex_with_leading_dot()),
                '<' => Ok(self.lex_with_leading_left_angle_bracket()),
                '=' => Ok(self.lex_with_leading_equals()),
                '&' => Ok(self.lex_with_leading_ampersand()),
                '^' => Ok(self.lex_with_leading_caret()),
                '!' => self.lex_with_leading_exclamation_mark(),
                '@' => self.lex_with_leading_at(),
                '>' => Ok(self.lex_with_leading_right_angle_bracket()),

                // The minus symbol is a negationnumeric prefix. If the lexer
                // has got here, it is assumed that their use as numeric
                // prefixes has already been ruled out.
                //
                // Note that `-` or `+` are either parts of a number literal or
                // binary operators but are _not_ unary operators. This allows
                // the lexer to avoid distinguishing unary and binary `-` and `+
                // `solely by whitespace. For negating a variable, use the
                // `Number#negate` method instead.
                '-' => Ok(self.lex_with_leading_hyphen()),

                '/' => {
                    self.source.discard();
                    Ok(Token::OverloadableInfixOperator(
                        OverloadableInfixOperator::Divide,
                    ))
                }
                '~' => {
                    self.source.discard();
                    Ok(Token::OverloadableInfixOperator(
                        OverloadableInfixOperator::Compose,
                    ))
                }
                '%' => {
                    self.source.discard();
                    Ok(Token::OverloadableInfixOperator(
                        OverloadableInfixOperator::Modulo,
                    ))
                }
                '*' => {
                    self.source.discard();
                    Ok(self.lex_with_leading_asterisk())
                }
                ',' => {
                    self.source.discard();
                    Ok(Token::SubItemSeparator)
                }
                '?' => {
                    self.source.discard();
                    Ok(Token::PostfixOperator(PostfixOperator::Bind))
                }
                '+' => {
                    self.source.discard();
                    Ok(Token::OverloadableInfixOperator(
                        OverloadableInfixOperator::Add,
                    ))
                }

                // Might be infix, might be postfix.
                ':' => Ok(self.lex_with_leading_colon()),

                // Might be slicing, or might be unrelated.
                '[' => Ok(self.lex_with_leading_open_square_bracket()),
                '|' => Ok(self.lex_with_leading_pipe()),

                // Grouping tokens.
                '{' => {
                    self.source.discard();
                    Ok(Token::Grouping(Grouping::OpenBrace))
                }
                '}' => {
                    self.source.discard();
                    Ok(Token::Grouping(Grouping::CloseBrace))
                }
                '(' => {
                    self.source.discard();
                    Ok(Token::Grouping(Grouping::OpenParentheses))
                }
                ')' => {
                    self.source.discard();
                    Ok(Token::Grouping(Grouping::CloseParentheses))
                }
                ']' => {
                    self.source.discard();
                    Ok(Token::Grouping(Grouping::CloseSquareBracket))
                }

                _ => Ok(self.lex_identifier()),
            }
        } else {
            self.fail("file ended before an operator could be read")
        }
    }

    fn lex_with_leading_colon(&mut self) -> Token {
        self.source.discard();
        Token::Colon
    }

    fn lex_with_leading_open_square_bracket(&mut self) -> Token {
        self.source.discard();
        if self.source.next_is('|') {
            self.source.discard();
            Token::OverloadableSliceOperator(OverloadableSliceOperator::Open)
        } else {
            Token::Grouping(Grouping::OpenSquareBracket)
        }
    }

    fn lex_with_leading_dot(&mut self) -> Token {
        self.source.discard();
        if self.source.next_is('.') {
            self.source.discard();
            if self.source.next_is('.') {
                self.source.discard();
                Token::PseudoIdentifier(PseudoIdentifier::Ellipsis)
            } else {
                Token::Rest
            }
        } else {
            Token::Dot
        }
    }

    fn lex_with_leading_left_angle_bracket(&mut self) -> Token {
        self.source.discard();
        if self.source.next_is('<') {
            self.source.discard();
            self.source.discard();
            Token::OverloadableInfixOperator(OverloadableInfixOperator::LeftShift)
        } else if self.source.next_is('=') {
            self.source.discard();
            Token::OverloadableInfixOperator(OverloadableInfixOperator::LessThanOrEqual)
        } else {
            self.source.discard();
            Token::OverloadableInfixOperator(OverloadableInfixOperator::LessThan)
        }
    }

    fn lex_with_leading_equals(&mut self) -> Token {
        self.source.discard();
        if self.source.next_is('=') {
            self.source.discard();
            Token::OverloadableInfixOperator(OverloadableInfixOperator::Equals)
        } else {
            Token::Binding(Binding::Assign)
        }
    }

    fn lex_with_leading_ampersand(&mut self) -> Token {
        self.source.discard();
        if self.source.next_is('&') {
            self.source.discard();
            Token::OverloadableInfixOperator(OverloadableInfixOperator::And)
        } else {
            Token::OverloadableInfixOperator(OverloadableInfixOperator::Ampersand)
        }
    }

    fn lex_with_leading_pipe(&mut self) -> Token {
        self.source.discard();
        if self.source.next_is('>') {
            self.source.discard();
            Token::OverloadableInfixOperator(OverloadableInfixOperator::Pipe)
        } else if self.source.next_is('|') {
            self.source.discard();
            Token::OverloadableInfixOperator(OverloadableInfixOperator::Or)
        } else if self.source.next_is(']') {
            self.source.discard();
            Token::OverloadableSliceOperator(OverloadableSliceOperator::Close)
        } else {
            Token::OverloadableInfixOperator(OverloadableInfixOperator::BitwiseOr)
        }
    }

    fn lex_with_leading_caret(&mut self) -> Token {
        self.source.discard();
        if self.source.next_is('^') {
            self.source.discard();
            Token::OverloadableInfixOperator(OverloadableInfixOperator::Xor)
        } else {
            Token::OverloadableInfixOperator(OverloadableInfixOperator::BitwiseXor)
        }
    }

    fn lex_with_leading_right_angle_bracket(&mut self) -> Token {
        self.source.discard();
        if self.source.next_is('>') {
            self.source.discard();
            if self.source.next_is('>') {
                self.source.discard();
                Token::OverloadableInfixOperator(OverloadableInfixOperator::UnsignedRightShift)
            } else {
                Token::OverloadableInfixOperator(OverloadableInfixOperator::RightShift)
            }
        } else if self.source.next_is('=') {
            Token::OverloadableInfixOperator(OverloadableInfixOperator::GreaterThanOrEqual)
        } else {
            Token::OverloadableInfixOperator(OverloadableInfixOperator::GreaterThan)
        }
    }

    fn lex_with_leading_hyphen(&mut self) -> Token {
        self.source.discard();
        if self.source.next_is('>') {
            self.source.discard();
            Token::OverloadableInfixOperator(OverloadableInfixOperator::Cascade)
        } else {
            Token::OverloadableInfixOperator(OverloadableInfixOperator::Subtract)
        }
    }

    fn lex_with_leading_asterisk(&mut self) -> Token {
        self.source.discard();
        if self.source.next_is('*') {
            self.source.discard();
            Token::OverloadableInfixOperator(OverloadableInfixOperator::Power)
        } else {
            Token::OverloadableInfixOperator(OverloadableInfixOperator::Multiply)
        }
    }

    fn lex_with_leading_exclamation_mark(&mut self) -> TokenResult {
        self.source.discard();
        self.expect_and_discard('=')
            .map(|_| Token::OverloadableInfixOperator(OverloadableInfixOperator::NotEqual))
    }

    fn lex_with_leading_at(&mut self) -> TokenResult {
        self.source.discard();
        match self.source.peek() {
            Some(token) => match token {
                '@' => {
                    self.source.discard();
                    Ok(Token::OverloadableInfixOperator(
                        OverloadableInfixOperator::MatrixTranspose,
                    ))
                }
                '+' => {
                    self.source.discard();
                    Ok(Token::OverloadableInfixOperator(
                        OverloadableInfixOperator::MatrixAdd,
                    ))
                }
                '/' => {
                    self.source.discard();

                    Ok(Token::OverloadableInfixOperator(
                        OverloadableInfixOperator::MatrixDivide,
                    ))
                }
                '*' => {
                    self.source.discard();
                    let op = if self.source.next_is('*') {
                        self.source.discard();
                        OverloadableInfixOperator::MatrixPower
                    } else {
                        OverloadableInfixOperator::MatrixMultiply
                    };
                    Ok(Token::OverloadableInfixOperator(op))
                }
                '-' => {
                    self.source.discard();
                    Ok(Token::OverloadableInfixOperator(
                        OverloadableInfixOperator::MatrixSubtract,
                    ))
                }
                _ => Ok(Token::Macros(Macros::At)),
            },
            None => Err(self.premature_eof()),
        }
    }

    fn lex_placeholder_identifier(&mut self) -> TokenResult {
        self.source.discard();
        Ok(Token::PseudoIdentifier(
            PseudoIdentifier::PlaceholderIdentifier,
        ))
    }

    fn lex_non_trivia(&mut self) -> TokenResult {
        match self.source.peek() {
            None => Ok(Token::Eof),
            Some(&c) => {
                if (c == '/') && self.source.nth_is(1, '*') && self.source.nth_is(2, '*') {
                    self.lex_sydoc()
                } else {
                    match c {
                        '"' => {
                            let custom_delimiter = self
                                .source
                                .peek_many(3)
                                .filter(|x| string_matches_char_slice(r#"""""#, x))
                                .is_some();

                            if custom_delimiter {
                                self.lex_string_with_custom_delimiter(true)
                            } else {
                                self.lex_string(true)
                            }
                        }
                        '`' => {
                            let custom_delimiter = self
                                .source
                                .peek_many(3)
                                .filter(|x| string_matches_char_slice("```", x))
                                .is_some();

                            if custom_delimiter {
                                self.lex_quoted_identifier_with_custom_delimiter(true)
                            } else {
                                self.lex_quoted_identifier(true)
                            }
                        }
                        '$' => {
                            if self.source.nth_is(1, '"') {
                                let custom_delimiter = self
                                    .source
                                    .peek_many(4)
                                    .filter(|x| string_matches_char_slice(r#"$""""#, x))
                                    .is_some();

                                if custom_delimiter {
                                    self.lex_interpolated_string_with_custom_delimiter(true)
                                } else {
                                    self.lex_interpolated_string(true)
                                }
                            } else {
                                self.lex_symbolic()
                            }
                        }
                        '\'' => self.lex_char(true),

                        _ => {
                            let next = self.source.peek_nth(1).cloned();
                            let escapable_literal_start =
                                next.filter(|c| is_start_of_literal_with_escapes(*c));

                            match escapable_literal_start {
                                Some(delimiter) if c == 'r' => {
                                    self.source.discard();
                                    match delimiter {
                                        '"' => {
                                            let custom_delimiter = self
                                                .source
                                                .peek_many(3)
                                                .filter(|x| string_matches_char_slice(r#"""""#, x))
                                                .is_some();

                                            if custom_delimiter {
                                                self.lex_string_with_custom_delimiter(false)
                                            } else {
                                                self.lex_string(false)
                                            }
                                        }
                                        '`' => {
                                            let custom_delimiter = self
                                                .source
                                                .peek_many(3)
                                                .filter(|x| string_matches_char_slice("```", x))
                                                .is_some();

                                            if custom_delimiter {
                                                self.lex_quoted_identifier_with_custom_delimiter(false)
                                            } else {
                                                self.lex_quoted_identifier(false)
                                            }
                                        }
                                        '\'' => {
                                            self.lex_char(false)
                                        }
                                        '$' => {
                                            if self.source.nth_is(1, '"') {
                                                let custom_delimiter = self
                                                    .source
                                                    .peek_many(4)
                                                    .filter(|x| string_matches_char_slice(r#"$""""#, x))
                                                    .is_some();

                                                if custom_delimiter {
                                                    self.lex_interpolated_string_with_custom_delimiter(false)
                                                } else {
                                                    self.lex_interpolated_string(false)
                                                }
                                            } else {
                                                self.lex_symbolic()
                                            }
                                        }
                                        _ => self.fail("expected `r` to prefixed something that otherwise has characters escapes, e.g. strings, interpolated strings, and char literals"),
                                    }
                                }
                                _ => {
                                    if (c == '_') && next.filter(|&x| x == '_').is_none() {
                                        self.lex_placeholder_identifier()
                                    } else if c.is_alphabetic() {
                                        let mut rest = String::new();
                                        self.lex_rest_of_word(&mut rest);
                                        self.lex_phrase(rest)
                                    } else if c.is_digit(10)
                                        || (self.source.match_nth(1, |c| c.is_digit(10))
                                            && ((c == '+') || (c == '-')))
                                    {
                                        self.lex_number()
                                    } else {
                                        self.lex_symbolic()
                                    }
                                }
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

    pub fn lex_version_or_next_non_trivia(&mut self) -> Option<LexedTokenResult> {
        match self.lex_trivia() {
            Ok(trivia) => {
                if let Some(&c) = self.source.peek() {
                    let token = if (c == 'v') && self.source.match_nth(1, |c| c.is_digit(10)) {
                        self.lex_version()
                    } else {
                        self.lex_non_trivia()
                    };
                    Some(token.map(|t| LexedToken {
                        token: t,
                        position: self.source.position,
                        trivia,
                    }))
                } else {
                    None
                }
            }
            Err(err) => Some(Err(err)),
        }
    }

    pub fn lex_shebang_at_start_of_source(&mut self) -> Option<LexedTokenResult> {
        if let Some('#') = self.source.peek() {
            match self.lex_shebang() {
                Ok(shebang) => Some(Ok(LexedToken {
                    token: shebang.clone(),
                    position: self.source.position,
                    trivia: None,
                })),
                Err(err) => Some(Err(err)),
            }
        } else {
            None
        }
    }

    /// Start lexing from the top-level of the source, returning a lexing task running concurrently
    /// in another thread and feeding tokens through a channel as it goes.
    pub fn lex(mut self) -> io::Result<LexerTask> {
        let (tx, rx) = channel();
        let thread = thread::Builder::new().name(LEXER_THREAD_NAME.to_string());

        let handle = thread.spawn(move || loop {
            if let Some(shebang_result) = self.lex_shebang_at_start_of_source() {
                let shebang = shebang_result?;
                tx.send(shebang.clone())
                    .map_err(|err| self.send_error(&shebang, &err))?;
            }

            if let Some(version_result) = self.lex_version_or_next_non_trivia() {
                let version = version_result?;
                tx.send(version.clone())
                    .map_err(|err| self.send_error(&version, &err))?;
            }

            match self.lex_next() {
                Ok(token) => {
                    let is_eof = token.token == Token::Eof;
                    tx.send(token.clone())
                        .map_err(|err| self.send_error(&token, &err))?;
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
    use super::*;
    use crate::common::multiphase::{Identifier, InterpolatedString, Shebang, SyDoc};
    use crate::lexing::tokens::{
        BranchingAndJumping, DeclarationHead, Modifier, ModuleDefinitions,
    };

    fn test_lexer(s: &str) -> Lexer {
        let source_chars = s.chars().collect::<Vec<char>>();
        Lexer::from(Source::from(source_chars))
    }

    fn assert_next(lexer: &mut Lexer, token: &Token) {
        match lexer.lex_next() {
            Ok(LexedToken { token: t, .. }) => {
                assert_eq!(*token, t);
            }
            Err(e) => panic!(e),
        }
    }

    fn start_is_shebang(lexer: &mut Lexer, token: &Token) -> bool {
        if let Some(Ok(LexedToken { token: t, .. })) = lexer.lex_shebang_at_start_of_source() {
            t == *token
        } else {
            false
        }
    }

    fn check_version_or_next_non_trivial(lexer: &mut Lexer, token: &Token) -> bool {
        if let Some(Ok(LexedToken { token: t, .. })) = lexer.lex_version_or_next_non_trivia() {
            t == *token
        } else {
            false
        }
    }

    #[test]
    fn empty() {
        let mut lexer = test_lexer("    \t  \n      ");
        assert_next(&mut lexer, &Token::Eof);
    }

    #[test]
    fn identifier() {
        let mut lexer = test_lexer(
            "  foobar324  \t  `foobarbaz`  \r  `abc\\ndef` r`abc\\ndef`  ```abc``\ndef```    abc",
        );

        assert_next(
            &mut lexer,
            &Token::Identifier(Identifier::from("foobar324")),
        );
        assert_next(
            &mut lexer,
            &Token::Identifier(Identifier::from("foobarbaz")),
        );
        assert_next(&mut lexer, &Token::Identifier(Identifier::from("abc\ndef")));
        assert_next(
            &mut lexer,
            &Token::Identifier(Identifier::from("abc\\ndef")),
        );
        assert_next(
            &mut lexer,
            &Token::Identifier(Identifier::from("abc``\ndef")),
        );
        assert_next(&mut lexer, &Token::Identifier(Identifier::from("abc")));
    }

    #[test]
    fn psuedo_identifiers() {
        let mut lexer =
            test_lexer(" super  This  this  \t _ this.package this.module \r  it  continue ... ");
        assert_next(
            &mut lexer,
            &Token::PseudoIdentifier(PseudoIdentifier::Super),
        );
        assert_next(
            &mut lexer,
            &Token::PseudoIdentifier(PseudoIdentifier::ThisType),
        );
        assert_next(&mut lexer, &Token::PseudoIdentifier(PseudoIdentifier::This));
        assert_next(
            &mut lexer,
            &Token::PseudoIdentifier(PseudoIdentifier::PlaceholderIdentifier),
        );
        assert_next(
            &mut lexer,
            &Token::PseudoIdentifier(PseudoIdentifier::ThisPackage),
        );
        assert_next(
            &mut lexer,
            &Token::PseudoIdentifier(PseudoIdentifier::ThisModule),
        );
        assert_next(&mut lexer, &Token::PseudoIdentifier(PseudoIdentifier::It));
        assert_next(
            &mut lexer,
            &Token::PseudoIdentifier(PseudoIdentifier::Continue),
        );
        assert_next(
            &mut lexer,
            &Token::PseudoIdentifier(PseudoIdentifier::Ellipsis),
        );
    }

    #[test]
    fn groupings() {
        let mut lexer = test_lexer("   \t ( { [ \r   } ] )  ");
        assert_next(&mut lexer, &Token::Grouping(Grouping::OpenParentheses));
        assert_next(&mut lexer, &Token::Grouping(Grouping::OpenBrace));
        assert_next(&mut lexer, &Token::Grouping(Grouping::OpenSquareBracket));
        assert_next(&mut lexer, &Token::Grouping(Grouping::CloseBrace));
        assert_next(&mut lexer, &Token::Grouping(Grouping::CloseSquareBracket));
        assert_next(&mut lexer, &Token::Grouping(Grouping::CloseParentheses));
    }

    #[test]
    fn keywords() {
        let mut lexer = test_lexer(" if   class\t  \r\n  abc var reject final with  ignorable");
        assert_next(
            &mut lexer,
            &Token::BranchingAndJumping(BranchingAndJumping::If),
        );
        assert_next(&mut lexer, &Token::DeclarationHead(DeclarationHead::Class));
        assert_next(&mut lexer, &Token::Identifier(Identifier::from("abc")));
        assert_next(&mut lexer, &Token::Binding(Binding::Var));
        assert_next(
            &mut lexer,
            &Token::ModuleDefinitions(ModuleDefinitions::Reject),
        );
        assert_next(&mut lexer, &Token::Binding(Binding::Final));
        assert_next(&mut lexer, &Token::With);
        assert_next(&mut lexer, &Token::Modifier(Modifier::Ignorable));
    }

    #[test]
    fn numbers() {
        let mut lexer = test_lexer("    23  \t  -34   \t\t\n   23   +32 0.32    \t123123123.32");
        assert_next(&mut lexer, &Token::Literal(Literal::Number(Number(23, 0))));
        assert_next(&mut lexer, &Token::Literal(Literal::Number(Number(-34, 0))));
        assert_next(&mut lexer, &Token::Literal(Literal::Number(Number(23, 0))));
        assert_next(&mut lexer, &Token::Literal(Literal::Number(Number(32, 0))));
        assert_next(&mut lexer, &Token::Literal(Literal::Number(Number(0, 32))));
        assert_next(
            &mut lexer,
            &Token::Literal(Literal::Number(Number(123_123_123, 32))),
        );
    }

    #[test]
    fn chars() {
        let mut lexer = test_lexer("  'a' '\\r'  \t \n\r\n 'd'    '/'");
        assert_next(&mut lexer, &Token::Literal(Literal::Char('a')));
        assert_next(&mut lexer, &Token::Literal(Literal::Char('\r')));
        assert_next(&mut lexer, &Token::Literal(Literal::Char('d')));
        assert_next(&mut lexer, &Token::Literal(Literal::Char('/')));
    }

    #[test]
    fn strings() {
        let mut lexer = test_lexer("  \"abc\\ndef\"   \t \n\n\n\"\"\"\"'123'\"\"\"\"");
        assert_next(
            &mut lexer,
            &Token::Literal(Literal::String(SylanString::from("abc\ndef"))),
        );
        assert_next(
            &mut lexer,
            &Token::Literal(Literal::String(SylanString::from("'123'"))),
        );
    }

    #[test]
    fn raw_strings() {
        let mut lexer = test_lexer("  r\"abc\\ndef\"   \t \n\n\nr\"\"\"\"'123'\"\"\"\"");
        assert_next(
            &mut lexer,
            &Token::Literal(Literal::String(SylanString::from("abc\\ndef"))),
        );
        assert_next(
            &mut lexer,
            &Token::Literal(Literal::String(SylanString::from("'123'"))),
        );
    }

    #[test]
    fn interpolated_strings() {
        let mut lexer = test_lexer(
            "   $\"1{x}{{23\"   $\"\"\"\"ab{{notInterpolated}}c\"\"\\t{foobar}\"\"\" \"\"\"\"",
        );

        assert_next(
            &mut lexer,
            &Token::Literal(Literal::InterpolatedString(InterpolatedString {
                string_fragments: vec!["1".to_owned(), "{{23".to_owned()],
                interpolations: vec![Identifier::from("x")],
            })),
        );

        assert_next(
            &mut lexer,
            &Token::Literal(Literal::InterpolatedString(InterpolatedString {
                string_fragments: vec![
                    "ab{{notInterpolated}}c\"\"\t".to_owned(),
                    r#"""" "#.to_owned(),
                ],
                interpolations: vec![Identifier::from("foobar")],
            })),
        );
    }

    #[test]
    fn infix_operators() {
        let mut lexer =
            test_lexer("    ->   |]   <= \t .   [|   \r\n ~ ^ ^^ @* - < @-  @@ [ != |>  ");

        assert_next(
            &mut lexer,
            &Token::OverloadableInfixOperator(OverloadableInfixOperator::Cascade),
        );
        assert_next(
            &mut lexer,
            &Token::OverloadableSliceOperator(OverloadableSliceOperator::Close),
        );
        assert_next(
            &mut lexer,
            &Token::OverloadableInfixOperator(OverloadableInfixOperator::LessThanOrEqual),
        );
        assert_next(&mut lexer, &Token::Dot);
        assert_next(
            &mut lexer,
            &Token::OverloadableSliceOperator(OverloadableSliceOperator::Open),
        );
        assert_next(
            &mut lexer,
            &Token::OverloadableInfixOperator(OverloadableInfixOperator::Compose),
        );
        assert_next(
            &mut lexer,
            &Token::OverloadableInfixOperator(OverloadableInfixOperator::BitwiseXor),
        );
        assert_next(
            &mut lexer,
            &Token::OverloadableInfixOperator(OverloadableInfixOperator::Xor),
        );
        assert_next(
            &mut lexer,
            &Token::OverloadableInfixOperator(OverloadableInfixOperator::MatrixMultiply),
        );
        assert_next(
            &mut lexer,
            &Token::OverloadableInfixOperator(OverloadableInfixOperator::Subtract),
        );
        assert_next(
            &mut lexer,
            &Token::OverloadableInfixOperator(OverloadableInfixOperator::LessThan),
        );
        assert_next(
            &mut lexer,
            &Token::OverloadableInfixOperator(OverloadableInfixOperator::MatrixSubtract),
        );
        assert_next(
            &mut lexer,
            &Token::OverloadableInfixOperator(OverloadableInfixOperator::MatrixTranspose),
        );
        assert_next(&mut lexer, &Token::Grouping(Grouping::OpenSquareBracket));
        assert_next(
            &mut lexer,
            &Token::OverloadableInfixOperator(OverloadableInfixOperator::NotEqual),
        );
        assert_next(
            &mut lexer,
            &Token::OverloadableInfixOperator(OverloadableInfixOperator::Pipe),
        );
    }

    #[test]
    fn postfix_operators() {
        let mut lexer = test_lexer("   ?      ");
        assert_next(&mut lexer, &Token::PostfixOperator(PostfixOperator::Bind));
    }

    #[test]
    fn identifiers() {
        let mut lexer = test_lexer(" FOObar  ab!    ");
        assert_next(&mut lexer, &Token::Identifier(Identifier::from("FOObar")));
        assert_next(&mut lexer, &Token::Identifier(Identifier::from("ab!")));
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
        let mut lexer = test_lexer("  True False   \n\t   /* ");
        assert_next(&mut lexer, &Token::Identifier(Identifier::from("True")));
        assert_next(&mut lexer, &Token::Identifier(Identifier::from("False")));
    }

    #[test]
    fn version() {
        let mut lexer = test_lexer("v10.23");
        assert!(check_version_or_next_non_trivial(
            &mut lexer,
            &Token::Version(Version {
                major: 10,
                minor: 23,
                patch: 0,
            }),
        ));

        test_lexer("10.23");
        assert!(!check_version_or_next_non_trivial(
            &mut lexer,
            &Token::Version(Version {
                major: 10,
                minor: 23,
                patch: 0,
            }),
        ));
    }

    #[test]
    fn rest() {
        let mut lexer = test_lexer(" . .. ... .. .");

        assert_next(&mut lexer, &Token::Dot);
        assert_next(&mut lexer, &Token::Rest);
        assert_next(
            &mut lexer,
            &Token::PseudoIdentifier(PseudoIdentifier::Ellipsis),
        );
        assert_next(&mut lexer, &Token::Rest);
        assert_next(&mut lexer, &Token::Dot);
    }

    #[test]
    fn shebang() {
        let mut lexer = test_lexer("#!/usr/bin/env sylan");
        let shebang = Token::Shebang(Shebang::from("/usr/bin/env sylan"));
        assert!(start_is_shebang(&mut lexer, &shebang));

        let mut lexer2 = test_lexer("#!/usr/bin sylan\r\ntrue false");
        let shebang2 = Token::Shebang(Shebang::from("/usr/bin sylan"));
        assert!(start_is_shebang(&mut lexer2, &shebang2));
        assert_next(&mut lexer2, &Token::Identifier(Identifier::from("true")));

        let mut lexer3 = test_lexer("#!/usr/local/bin/env sylan\n123 321");
        let shebang3 = Token::Shebang(Shebang::from("/usr/local/bin/env sylan"));
        assert!(start_is_shebang(&mut lexer3, &shebang3));
        assert_next(
            &mut lexer3,
            &Token::Literal(Literal::Number(Number(123, 0))),
        );

        let mut failing_lexer = test_lexer("/usr/local/bin/env sylan\n123 321");
        let shebang3 = Token::Shebang(Shebang::from("/usr/local/bin/env sylan"));
        assert!(!start_is_shebang(&mut failing_lexer, &shebang3));
    }

    #[test]
    fn sydoc() {
        // Ensure that:
        //
        // * Multiline comments aren't mixed up with SyDocs.
        // * Nesting comments works.
        // * Nesting JavaDoc within multiline comments works, and vice-versa.
        // * Leading whitespace and `*` on newlines are stripped.
        let mut lexer =
            test_lexer("/* /**/ /****/ comment */ // \n /** A SyDoc \n * /* comment. */ \n */");

        let sydoc = Token::SyDoc(SyDoc::from(" A SyDoc  /* comment. */ "));
        assert_next(&mut lexer, &sydoc);
    }

    #[test]
    fn member_lookups() {
        let mut lexer =
            test_lexer("  /* */this.field this.package this.FIEL3D_FIELD  this.module //");

        assert_next(&mut lexer, &Token::PseudoIdentifier(PseudoIdentifier::This));
        assert_next(&mut lexer, &Token::Dot);
        assert_next(&mut lexer, &Token::Identifier(Identifier::from("field")));

        assert_next(
            &mut lexer,
            &Token::PseudoIdentifier(PseudoIdentifier::ThisPackage),
        );

        assert_next(&mut lexer, &Token::PseudoIdentifier(PseudoIdentifier::This));
        assert_next(&mut lexer, &Token::Dot);
        assert_next(
            &mut lexer,
            &Token::Identifier(Identifier::from("FIEL3D_FIELD")),
        );

        assert_next(
            &mut lexer,
            &Token::PseudoIdentifier(PseudoIdentifier::ThisModule),
        );
    }
}
