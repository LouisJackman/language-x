//! # Sylan's Parser
//!
//! The parser turns a token stream into an abstract syntax tree that can be
//! better reasoned about semantically, ready for submission to further
//! simplification steps and then a backend for execution or creation of an
//! executable. There is no CST step between tokens and the AST for the reason
//! described in Sylan's top-level documentation.
//!
//! All AST nodes are either items or expressions. Items are top-level
//! declarations that describe the whole program structure. Expressions
//! describe actual computations. Expressions can be contained within items,
//! but items cannot be contained within expressions with the exception of
//! variable bindings in lambda bodies.
//!
//! All expressions must be inside function or method blocks, which are items.
//! The exception is the `main` package in the `main` module which allows
//! top-level code.
//!
//! Statements don't really exist in Sylan. The closest equivalent is an
//! expression that returns Sylan's unit type `void` or an `ignorable`
//! expression that throws away an `ignoreable`  method's or function's
//! non-`void` return value. That said, the only item allowed inside expressions
//! (e.g. within the body of a lambda in the middle of an outer expression),
//! variable bindings with `var`, pretty much look and feel like statements from
//! other languages.
//!
//! Expressions just resolve inner expressions outwards until done; a function,
//! lambda, or method block can have multiple expressions which Sylan, being a
//! strict, non-pure language, can just execute sequentially at runtime without
//! needing monads or uniqueness types to enforce the order.
//!
//! A simplification step is performed before giving the AST to the backend as
//! a jump is needed from Sylan's pragmatic, large syntax to the much smaller,
//! more "pure" form that defines the core Sylan execution semantics. See the
//! `simplification` module for more details.
//!
//! A concurrent design similar to the lexer's might be possible, but will
//! require a sort of zipper or lazy tree structure. More research is needed
//! here. Until then, there is no `ParserTask` equivalent to the `LexerTask`.

use std::collections::{HashSet, LinkedList};
use std::rc::Rc;
use std::result;
use std::sync::Arc;

use common::multiphase::{self, Identifier};
use common::peekable_buffer::PeekableBuffer;
use common::version::Version;
use lexing::lexer::{self, LexedToken};
use lexing::tokens::Token;
use lexing::Tokens;
use parsing::nodes::Expression::{self, UnaryOperator};
use parsing::nodes::{
    Accessibility, Binding, Case, Code, CompositePattern, ContextualBinding, DeclarationItem,
    FilePackage, For, If, Import, Lambda, LambdaSignature, MainPackage, Package, Pattern,
    PatternField, PatternItem, Scope, Select, Switch, Throw, Timeout, TypeDeclaration,
    ValueParameter,
};

mod nodes;

// TODO: break cycles in scopes to cleanup memory properly.

#[derive(Debug)]
pub enum ParserErrorDescription {
    Described(String),
    Expected(Token),
    Unexpected(Token),
    PrematureEof,
}

#[derive(Debug)]
pub struct ParserError {
    description: ParserErrorDescription,
}

#[derive(Debug)]
pub enum Error {
    Lexer(lexer::Error),
    Parser(ParserError),
}

type Result<T> = result::Result<T, Error>;

fn is_ignored_binding(identifier: Identifier) -> bool {
    let Identifier(string) = identifier;
    (*string) == "_"
}

pub struct Parser {
    tokens: Tokens,
    current_scope: Rc<Scope>,
}

impl From<Tokens> for Parser {
    fn from(tokens: Tokens) -> Self {
        Self {
            tokens,
            current_scope: Scope::new_root(),
        }
    }
}

impl Parser {
    /// Fail at parsing, describing the reason why.
    fn fail<T>(&self, message: impl Into<String>) -> Result<T> {
        Err(Error::Parser(ParserError {
            description: ParserErrorDescription::Described(message.into()),
        }))
    }

    /// Fail at parsing, stating that the `expected` token was expected but
    /// did not appear.
    fn expected<T>(&self, expected: Token) -> Result<T> {
        Err(Error::Parser(ParserError {
            description: ParserErrorDescription::Expected(expected),
        }))
    }

    /// Return a successful empty result if it is indeed the next token in the
    /// stream. Otherwise, fail at parsing, stating that the `expected` token
    /// was expected but did not appear.
    ///
    /// The successful empty result is mostly useful when combined with the `?`
    /// operator.
    fn expect(&mut self, expected: Token) -> Result<()> {
        if self.next_is(&expected) {
            Ok(())
        } else {
            self.expected(expected)
        }
    }

    /// Return the next read token in the stream if it matches the expected
    /// token. Otherwise fail at parsing, stating that the `expected` token was
    /// expected but did not appear.
    fn expect_and_read(&mut self, expected: Token) -> Result<Token> {
        let next = self.tokens.read();
        next.map(|lexed| lexed.token)
            .filter(|token| *token == expected)
            .map(Ok)
            .unwrap_or_else(|| self.expected(expected))
    }

    /// Discard the next read token in the stream if it matches the expected
    /// token. Otherwise fail at parsing, stating that the `expected` token was
    /// expected but did not appear.
    fn expect_and_discard(&mut self, expected: Token) -> Result<()> {
        if let Some(lexed) = self.tokens.read() {
            if lexed.token == expected {
                Ok(())
            } else {
                self.expected(expected)
            }
        } else {
            self.premature_eof()
        }
    }

    /// Fail at parsing, stating that the `unexpected` token was unexpected
    /// and therefore cannot be handled.
    fn unexpected<T>(&self, unexpected: Token) -> Result<T> {
        Err(Error::Parser(ParserError {
            description: ParserErrorDescription::Unexpected(unexpected),
        }))
    }

    /// Fail at parsing because an EOF was encountered unexpectedly.
    fn premature_eof<T>(&self) -> Result<T> {
        Err(Error::Parser(ParserError {
            description: ParserErrorDescription::PrematureEof,
        }))
    }

    /// Check whether the next token matches `expected`.
    fn next_is(&mut self, expected: &Token) -> bool {
        self.tokens.match_next(|lexed| lexed.token == *expected)
    }

    /// Check whether the `n`th token matches `expected`, where `n` is
    /// zero-indexed.
    fn nth_is(&mut self, n: usize, expected: &Token) -> bool {
        self.tokens.match_nth(n, |lexed| lexed.token == *expected)
    }

    // The following methods are sub-parsers that are reentrant and handle the
    // parsing of a particular subcontext of the overall source. Each expects
    // the whole context next in the stream, so previous steps working out which
    // sub-parser to delegate to should use peeks and not reads to discern it
    // from subsequent tokens in the buffer.

    fn parse_unary_operator(
        &mut self,
        operator: nodes::UnaryOperator,
    ) -> Result<nodes::Expression> {
        self.tokens.discard();
        self.parse_expression()
            .map(|expression| UnaryOperator(operator, Box::new(expression)))
    }

    fn parse_bitwise_not(&mut self) -> Result<nodes::Expression> {
        self.parse_unary_operator(nodes::UnaryOperator::BitwiseNot)
    }

    fn parse_not(&mut self) -> Result<nodes::Expression> {
        self.parse_unary_operator(nodes::UnaryOperator::Not)
    }

    fn parse_package_lookup(&mut self) -> Result<nodes::PackageLookup> {
        let mut lookup = vec![];
        loop {
            lookup.push(self.parse_identifier()?);
            if self.next_is(&Token::Dot) {
                self.tokens.discard();
            } else {
                break Ok(lookup);
            }
        }
    }

    fn parse_type_specification(&mut self) -> Result<nodes::TypeSpecification> {
        unimplemented!()
    }

    fn parse_class(&mut self) -> Result<nodes::Class> {
        unimplemented!()
    }

    fn parse_with(&mut self) -> Result<nodes::Scope> {
        let mut bindings = HashSet::new();
        let mut contextual_bindings = HashSet::new();
        let mut expressions = vec![];

        self.tokens.discard();
        self.expect_and_discard(Token::OpenBrace)?;
        loop {
            if self.next_is(&Token::Var) {
                if let Some(LexedToken {
                    token: Token::Bind, ..
                }) = self.tokens.peek_nth(2)
                {
                    contextual_bindings.insert(self.parse_contextual_binding()?);
                } else {
                    bindings.insert(self.parse_binding()?);
                }
            } else if self.next_is(&Token::CloseBrace) {
                self.tokens.discard();
                break;
            } else {
                expressions.push(self.parse_expression()?);
            }
        }

        Ok(Scope {
            code: Code {
                bindings,
                expressions,
            },
            parent: Some(Scope::within(&self.current_scope)),
        })
    }

    fn parse_extend(&mut self) -> Result<nodes::TypeDeclaration> {
        self.tokens.discard();
        let specification = self.parse_type_specification()?;
        Ok(TypeDeclaration::Extension(specification))
    }

    fn parse_for(&mut self, label: Option<Identifier>) -> Result<nodes::Expression> {
        self.tokens.discard();

        let mut bindings = vec![];
        let scope = loop {
            if self.next_is(&Token::OpenBrace) {
                break self.parse_scope()?;
            } else {
                bindings.push(self.parse_binding()?);
                if self.next_is(&Token::SubItemSeparator) {
                    self.tokens.discard();
                }
            }
        };

        Ok(nodes::Expression::For(For {
            bindings,
            scope,
            label,
        }))
    }

    fn parse_if(&mut self) -> Result<nodes::If> {
        self.tokens.discard();

        let condition = self.parse_expression()?;
        let then = self.parse_scope()?;

        let else_clause = if self.next_is(&Token::Else) {
            self.tokens.discard();
            Some(self.parse_scope()?)
        } else {
            None
        };

        Ok(If {
            condition: Box::new(condition),
            then,
            else_clause,
        })
    }

    fn parse_type_name(&mut self) -> Result<nodes::Type> {
        unimplemented!()
    }

    fn parse_composite_pattern_field(&mut self, next: &Token) -> Result<Option<PatternField>> {
        let next_token_is_assign = self.nth_is(1, &Token::Assign);

        match &next {
            Token::Rest => {
                self.tokens.discard();
                self.expect(Token::CloseParentheses)?;
                Ok(None)
            }

            Token::Identifier(ref identifier) if !next_token_is_assign => {
                self.tokens.discard();
                let pattern = Pattern {
                    item: PatternItem::Identifier(identifier.clone()),
                    bound_match: None,
                };
                Ok(Some(PatternField {
                    identifier: identifier.clone(),
                    pattern,
                }))
            }

            _ => {
                let identifier = self.parse_identifier()?;
                self.expect_and_discard(Token::Assign)?;
                let pattern = self.parse_pattern()?;
                Ok(Some(PatternField {
                    identifier,
                    pattern,
                }))
            }
        }
    }

    fn parse_composite_pattern(&mut self) -> Result<nodes::CompositePattern> {
        let token = self
            .tokens
            .peek()
            .map(|lexed| Ok(lexed.clone().token))
            .unwrap_or_else(|| self.premature_eof())?;

        if let Token::Identifier(_) = token {
            let composite_type = self.parse_type_name()?;
            self.expect_and_discard(Token::OpenParentheses)?;

            let mut fields = vec![];
            let mut ignore_rest = false;
            loop {
                let next = self
                    .tokens
                    .peek()
                    .map(|lexed| Ok(lexed.clone().token))
                    .unwrap_or_else(|| self.premature_eof())?;

                if next == Token::CloseParentheses {
                    break;
                } else if let Some(field) = self.parse_composite_pattern_field(&next)? {
                    fields.push(field);
                } else {
                    ignore_rest = true;
                }

                self.expect_and_discard(Token::SubItemSeparator)?;
            }

            self.expect_and_discard(Token::CloseParentheses)?;

            let composite = CompositePattern {
                composite_type,
                fields,
                ignore_rest,
            };
            Ok(composite)
        } else {
            self.fail("expecting a type name for the composite pattern")
        }
    }

    fn parse_pattern(&mut self) -> Result<nodes::Pattern> {
        let token = self
            .tokens
            .peek()
            .map(|lexed| Ok(lexed.clone().token))
            .unwrap_or_else(|| self.premature_eof())?;

        let item = self
            .parse_literal(token.clone())
            .map(|lexed_token| Ok(PatternItem::Literal(lexed_token)))
            .unwrap_or_else(|| {
                if let Token::Identifier(identifier) = token {
                    Ok(if is_ignored_binding(identifier.clone()) {
                        PatternItem::Ignored
                    } else {
                        PatternItem::Identifier(identifier)
                    })
                } else {
                    let composite = self.parse_composite_pattern()?;
                    Ok(PatternItem::Composite(composite))
                }
            });

        Ok(Pattern {
            item: item?,
            bound_match: None,
        })
    }

    fn parse_import(&mut self) -> Result<nodes::Import> {
        self.tokens.discard();
        let lookup = self.parse_package_lookup()?;
        Ok(Import { lookup })
    }

    fn parse_interface(&mut self) -> Result<nodes::Interface> {
        unimplemented!()
    }

    fn reinterpret_expression_as_pattern(&mut self, _expression: &Expression) -> Result<Pattern> {
        unimplemented!()
    }

    // TODO: work out how to parse type parameters and default values for lambdas
    // unambiguously.
    fn parse_lambda(&mut self, parameter_patterns: Vec<Pattern>) -> Result<nodes::Lambda> {
        self.tokens.discard();

        let scope = self.parse_scope()?;

        let value_parameters = parameter_patterns
            .into_iter()
            .map(|pattern| ValueParameter {
                pattern,
                default_value: None,
            })
            .collect::<Vec<ValueParameter>>();

        let signature = LambdaSignature {
            type_parameters: vec![],
            value_parameters,
        };

        Ok(Lambda { signature, scope })
    }

    fn parse_package_definition(&mut self) -> Result<nodes::Package> {
        self.expect_and_discard(Token::Package)?;

        let name = self.parse_identifier()?;
        self.expect_and_discard(Token::OpenBrace)?;
        let (imports, declarations) = self.parse_inside_package()?;
        self.expect_and_discard(Token::CloseBrace)?;

        Ok(nodes::Package {
            accessibility: Accessibility::Public,
            name,
            imports,
            declarations,
        })
    }

    fn parse_binding(&mut self) -> Result<nodes::Binding> {
        self.tokens.discard();
        let pattern = self.parse_pattern()?;
        self.expect_and_discard(Token::Assign)?;
        let value = self.parse_expression()?;

        Ok(Binding { pattern, value })
    }

    fn parse_contextual_binding(&mut self) -> Result<nodes::ContextualBinding> {
        self.tokens.discard();
        let name = self.parse_identifier()?;
        self.expect_and_discard(Token::Bind)?;
        let value = self.parse_expression()?;

        Ok(ContextualBinding { name, value })
    }

    fn parse_identifier(&mut self) -> Result<Identifier> {
        if let Some(lexed) = self.tokens.read() {
            if let Token::Identifier(identifier) = lexed.token {
                Ok(identifier)
            } else {
                self.fail("identifier expected")
            }
        } else {
            self.premature_eof()
        }
    }

    fn parse_select(&mut self) -> Result<nodes::Select> {
        self.tokens.discard();
        self.expect_and_discard(Token::OpenBrace)?;
        let mut cases = vec![];
        let mut timeout = None;

        loop {
            let mut matches = LinkedList::new();
            if self.next_is(&Token::Timeout) {
                if timeout.is_none() {
                    let nanoseconds = Box::new(self.parse_expression()?);
                    let body = Box::new(if self.next_is(&Token::LambdaArrow) {
                        self.tokens.discard();
                        self.parse_expression()?
                    } else {
                        self.expect(Token::OpenBrace)?;
                        Expression::Scope(self.parse_scope()?)
                    });
                    timeout = Some(Timeout { nanoseconds, body });
                } else {
                    self.unexpected(Token::Timeout)?;
                }
            } else {
                let body = loop {
                    let pattern_match = if self.next_is(&Token::Default) {
                        Pattern {
                            item: PatternItem::Ignored,
                            bound_match: None,
                        }
                    } else {
                        self.parse_pattern()?
                    };
                    matches.push_back(pattern_match);
                    if self.next_is(&Token::LambdaArrow) {
                        self.tokens.discard();
                        break self.parse_expression()?;
                    } else if self.next_is(&Token::OpenBrace) {
                        break Expression::Scope(self.parse_scope()?);
                    } else {
                        self.expect_and_discard(Token::SubItemSeparator)?;
                    }
                };
                cases.push(Case { matches, body });
            }

            if self.next_is(&Token::CloseBrace) {
                self.tokens.discard();
                break Ok(Select { cases, timeout });
            }
        }
    }

    fn parse_switch(&mut self) -> Result<Switch> {
        self.tokens.discard();
        let expression = self.parse_expression()?;
        self.expect_and_discard(Token::OpenBrace)?;
        let mut cases = vec![];

        loop {
            let mut matches = LinkedList::new();
            let body = loop {
                let pattern_match = if self.next_is(&Token::Default) {
                    Pattern {
                        item: PatternItem::Ignored,
                        bound_match: None,
                    }
                } else {
                    self.parse_pattern()?
                };
                matches.push_back(pattern_match);
                if self.next_is(&Token::LambdaArrow) {
                    self.tokens.discard();
                    break self.parse_expression()?;
                } else if self.next_is(&Token::OpenBrace) {
                    break Expression::Scope(self.parse_scope()?);
                } else {
                    self.expect_and_discard(Token::SubItemSeparator)?;
                }
            };
            cases.push(Case { matches, body });

            if self.next_is(&Token::CloseBrace) {
                self.tokens.discard();
                break Ok(Switch {
                    expression: Box::new(expression),
                    cases,
                });
            }
        }
    }

    fn parse_invocable_handle(&mut self) -> Result<nodes::Expression> {
        unimplemented!()
    }

    fn parse_throw(&mut self) -> Result<nodes::Throw> {
        self.tokens.discard();
        let expression = self.parse_expression()?;
        Ok(Throw(Box::new(expression)))
    }

    fn parse_open_parentheses(&mut self) -> Result<nodes::Expression> {
        self.tokens.discard();
        let mut expressions = vec![];

        loop {
            expressions.push(self.parse_expression()?);

            let next = self
                .tokens
                .read()
                .map(|lexed| Ok(lexed.clone().token))
                .unwrap_or_else(|| self.premature_eof())?;

            match next {
                Token::SubItemSeparator => {}
                Token::CloseParentheses => break,
                unexpected => self.unexpected(unexpected)?,
            }
        }

        if self.next_is(&Token::LambdaArrow) {
            let parameter_patterns = expressions
                .into_iter()
                .map(|expression| self.reinterpret_expression_as_pattern(&expression))
                .collect::<Vec<Result<Pattern>>>();

            let failed = parameter_patterns.iter().any(|result| !result.is_ok());

            if failed {
                let failed_conversion = parameter_patterns
                    .into_iter()
                    .map(result::Result::unwrap_err)
                    .next()
                    .unwrap();
                Err(failed_conversion)
            } else {
                let successfully_converted = parameter_patterns
                    .into_iter()
                    .map(result::Result::unwrap)
                    .collect::<Vec<Pattern>>();

                Ok(nodes::Expression::Lambda(
                    self.parse_lambda(successfully_converted)?,
                ))
            }
        } else if expressions.len() == 1 {
            Ok(expressions[0].clone())
        } else {
            self.fail("multiple expressions found in a grouped expression; is it missing an operator or a comma?")
        }
    }

    fn parse_literal(&mut self, token: Token) -> Option<nodes::Literal> {
        match token {
            // Literal tokens are a one-to-one translation to AST nodes
            // except interpolated strings.
            Token::Boolean(b) => Some(nodes::Literal::Boolean(b)),
            Token::Char(c) => Some(nodes::Literal::Char(c)),
            Token::InterpolatedString(string) => {
                // TODO: reenter the lexer to handle interpolation
                // properly.
                Some(nodes::Literal::InterpolatedString(string))
            }
            Token::Number(decimal, fraction) => Some(nodes::Literal::Number(decimal, fraction)),
            Token::String(string) => Some(nodes::Literal::String(string)),
            _ => None,
        }
    }

    fn parse_leading_identifier(&mut self, identifier: Identifier) -> Result<nodes::Expression> {
        if self.nth_is(1, &Token::Colon) {
            self.tokens.discard();
            self.tokens.discard();
            self.parse_for(Some(identifier))
        } else {
            Ok(nodes::Expression::Identifier(self.parse_identifier()?))
        }
    }

    fn parse_expression(&mut self) -> Result<nodes::Expression> {
        let token = self.tokens.peek().cloned();
        match token {
            Some(lexed) => {
                let token = lexed.token;
                self.parse_literal(token.clone())
                    .map(|literal| Ok(nodes::Expression::Literal(literal)))
                    .unwrap_or_else(|| match token {
                        // Non-atomic tokens each delegate to a dedicated method.
                        Token::Identifier(identifier) => self.parse_leading_identifier(identifier),
                        Token::BitwiseNot => self.parse_bitwise_not(),
                        Token::With => self.parse_with().map(nodes::Expression::Scope),
                        Token::For => self.parse_for(None),
                        Token::If => self.parse_if().map(nodes::Expression::If),
                        Token::OpenBrace => {
                            self.parse_lambda(vec![]).map(nodes::Expression::Lambda)
                        }
                        Token::InvocableHandle => self.parse_invocable_handle(),
                        Token::Not => self.parse_not(),
                        Token::OpenParentheses => self.parse_open_parentheses(),
                        Token::Select => self.parse_select().map(nodes::Expression::Select),
                        Token::Switch => self.parse_switch().map(nodes::Expression::Switch),
                        Token::Throw => self.parse_throw().map(nodes::Expression::Throw),

                        non_expression => self.unexpected(non_expression),
                    })
            }
            None => self.fail(
                "\
                 an expression at the end of the Sylan file is not\
                 finished\
                 ",
            ),
        }
    }

    fn parse_class_body(&mut self) -> Result<nodes::TypeSpecification> {
        unimplemented!()
    }

    fn parse_interface_body(&mut self) -> Result<nodes::TypeSpecification> {
        unimplemented!()
    }

    fn parse_scope(&mut self) -> Result<nodes::Scope> {
        let code = self.parse_code()?;
        Ok(Scope {
            code,
            parent: Some(Scope::within(&self.current_scope)),
        })
    }

    fn parse_code(&mut self) -> Result<nodes::Code> {
        let mut bindings = HashSet::new();
        let mut expressions = vec![];

        self.expect_and_discard(Token::OpenBrace)?;
        loop {
            if self.next_is(&Token::Var) {
                bindings.insert(self.parse_binding()?);
            } else if self.next_is(&Token::CloseBrace) {
                self.tokens.discard();
                break;
            } else {
                expressions.push(self.parse_expression()?);
            }
        }

        Ok(Code {
            bindings,
            expressions,
        })
    }

    fn parse_expression_grouping(&mut self) -> Result<nodes::Expression> {
        self.tokens.discard();
        let expression = self.parse_expression()?;
        self.expect_and_discard(Token::CloseParentheses)?;
        Ok(expression)
    }

    fn parse_inside_package(&mut self) -> Result<(Vec<nodes::Import>, Vec<nodes::Declaration>)> {
        let imports = vec![];
        let declarations = vec![];

        loop {
            let maybe_token = self.tokens.peek().map(|lexed| lexed.token.clone());

            match maybe_token {
                None => break,

                Some(token) => {
                    match token {
                        Token::Class => {
                            self.parse_class();
                            unimplemented!()
                        }
                        Token::Extend => {
                            self.parse_extend();
                            unimplemented!()
                        }
                        Token::Import => {
                            self.parse_import();
                            unimplemented!()
                        }
                        Token::Interface => {
                            self.parse_interface();
                            unimplemented!()
                        }
                        Token::Package => {
                            self.parse_package_definition();
                            unimplemented!()
                        }
                        Token::Var => {
                            self.parse_binding();
                            unimplemented!()
                        }

                        _unexpected => {
                            //self.unexpected(unexpected);
                            unimplemented!()
                        }
                    }
                }
            }
        }

        Ok((imports, declarations))
    }

    fn parse_main_package(&mut self) -> Result<nodes::MainPackage> {
        let imports = vec![];
        let declarations = vec![];
        let expressions = vec![];

        loop {
            let maybe_token = self.tokens.peek().map(|lexed| lexed.token.clone());

            match maybe_token {
                None => break,

                Some(token) => {
                    match token {
                        Token::Class => {
                            let _class = self.parse_class();
                            unimplemented!()
                        }
                        Token::Extend => {
                            self.parse_extend();
                            unimplemented!()
                        }
                        Token::Import => {
                            self.parse_import();
                            unimplemented!()
                        }
                        Token::Interface => {
                            self.parse_interface();
                            unimplemented!()
                        }
                        Token::Package => {
                            self.parse_package_definition();
                            unimplemented!()
                        }
                        Token::Var => {
                            self.parse_binding();
                            unimplemented!()
                        }

                        // Unlike non-main packages, the main package will try to interpret
                        // non-declaration tokens as an expression to execute.
                        _ => {
                            self.parse_expression();
                            unimplemented!()
                        }
                    }
                }
            }
        }

        let package = Package {
            imports,
            declarations,
            accessibility: Accessibility::Public,
            name: Identifier(Arc::new(String::from("main"))),
        };

        let bindings = package
            .declarations
            .iter()
            .map(|declaration| declaration.item.clone())
            .map(|item| {
                if let DeclarationItem::Binding(binding) = item {
                    Some(binding)
                } else {
                    None
                }
            })
            .filter(Option::is_some)
            .map(Option::unwrap)
            .collect::<HashSet<Binding>>();

        Ok(MainPackage {
            package,
            code: Code {
                bindings,
                expressions,
            },
        })
    }

    fn maybe_parse_shebang(&mut self) -> Option<multiphase::Shebang> {
        let maybe_line = {
            let token = &self.tokens.peek()?.token;
            if let Token::Shebang(line) = token {
                Some(line.clone())
            } else {
                None
            }
        };
        if maybe_line.is_some() {
            self.tokens.discard();
        }
        maybe_line
    }

    fn maybe_parse_version(&mut self) -> Option<Version> {
        let maybe_version = {
            let token = &self.tokens.peek()?.token;
            if let Token::Version(version) = token {
                Some(*version)
            } else {
                None
            }
        };
        if maybe_version.is_some() {
            self.tokens.discard();
        }
        maybe_version
    }

    fn parse_file(&mut self) -> Result<nodes::File> {
        let shebang = self.maybe_parse_shebang();
        let version = self.maybe_parse_version();
        let main_package = self.parse_main_package();

        main_package.map(|main| nodes::File {
            shebang,
            version,
            package: FilePackage::EntryPoint(main),
        })
    }

    /// Parse an AST from a lexer, ensuring the underlying lexer task has
    /// finished before continuing.
    pub fn parse(mut self) -> Result<nodes::File> {
        let file = self.parse_file();
        self.tokens.join_lexer_thread();
        file
    }
}
