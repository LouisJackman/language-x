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
//! expression that returns Sylan's unit type `Void` or a function call with
//! an `ignorable` return value which is indeed ignored. That said, the only
//! item allowed inside expressions (e.g. within the body of a lambda in the
//! middle of an outer expression), variable bindings with `var`, pretty much
//! look and feel like statements from other languages.
//!
//! Expressions just resolve inner expressions outwards until done; a function,
//! lambda, or method block can have multiple expressions which Sylan, being a
//! strict, non-pure language, can just execute sequentially at runtime without
//! needing monads or uniqueness types to enforce the order.
//!
//! There are nine keyphrases that work effectively like predefined identifiers:
//! `...`, `it`, `continue`, `this`, `This`, `this.module`, `this.package`,
//! `super`, and `_`. They are also the only keyphrases that are allowed to be
//! shadowed in the same block; user-defined symbols will fail to bind if a binding
//! of the same name already exists in the same or outer scope. They are called
//! _pseudoidentifiers_.
//!
//! A simplification step is performed before giving the AST to the backend as
//! a jump is needed from Sylan's pragmatic, large syntax to the much smaller,
//! more "pure" form that defines the core Sylan execution semantics. See the
//! `simplification` module for more details.
//!
//! A concurrent design similar to the lexer's might be possible, but will
//! require a sort of zipper or lazy tree structure. More research is needed
//! here. Until then, there is no `ParserTask` equivalent to the `LexerTask`.

use std::collections::HashSet;
use std::default::Default;
use std::rc::Rc;
use std::result;

use crate::common::multiphase::{
    self, Accessibility, Identifier, OverloadableInfixOperator, PseudoIdentifier,
};
use crate::common::peekable_buffer::PeekableBuffer;
use crate::common::version::Version;
use crate::lexing::lexer;
use crate::lexing::tokens::{
    self, Binding, BranchingAndJumping, DeclarationHead, Grouping, Literal, Macros, Modifier, Token,
};
use crate::lexing::Tokens;
use crate::parsing::{
    modifier_sets::{AccessibilityModifierExtractor, ModifierSets},
    nodes::{
        Block, Case, CaseMatch, Class, ClassValueParameterFieldUpgrade, CompositePattern, Cond,
        CondCase, Expression, For, FunModifiers, FunSignature, If, Item, Lambda, LambdaSignature,
        LambdaValueParameter, MainPackage, Method, Operator, Package, Pattern, PatternGetter,
        PatternItem, Select, Switch, Symbol, SymbolLookup, Throw, Timeout, TypeArgument,
        TypeParameter, TypeReference, ValueArgument, ValueParameter,
    },
};
use nodes::CallArguments;

mod modifier_sets;
mod nodes;

// TODO: break cycles in scopes to cleanup memory properly.

#[derive(Debug)]
pub enum ParserErrorDescription {
    Described(String),
    Expected(Token),
    Unexpected(Token),
    LexerThreadFailed(String),
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

fn new_void() -> TypeReference {
    TypeReference::new(Symbol::Absolute(SymbolLookup(vec![
        Identifier::from("sylan"),
        Identifier::from("lang"),
        Identifier::from("Void"),
    ])))
}

pub struct Parser {
    tokens: Tokens,
    current_scope: Rc<Block>,
    modifier_sets: ModifierSets,
    accessibility_modifier_extractor: AccessibilityModifierExtractor,
}

impl From<Tokens> for Parser {
    fn from(tokens: Tokens) -> Self {
        Self {
            tokens,
            current_scope: Rc::new(Block::new_root()),
            modifier_sets: Default::default(),
            accessibility_modifier_extractor: AccessibilityModifierExtractor::new(),
        }
    }
}

impl Parser {
    //
    // Utilities
    //

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

    //
    // Tokens Convenience Wrappers
    //

    fn peek(&mut self) -> Option<Token> {
        self.tokens.peek().map(|lexed| lexed.token.clone())
    }

    fn peek_nth(&mut self, n: usize) -> Option<Token> {
        self.tokens.peek_nth(n).map(|lexed| lexed.token.clone())
    }

    fn read(&mut self) -> Option<Token> {
        self.tokens.read().map(|lexed| lexed.token)
    }

    /// Check whether the next token passes the predicate.
    fn match_next(&mut self, predicate: impl Fn(Token) -> bool) -> bool {
        self.tokens
            .match_next(|lexed| predicate(lexed.token.clone()))
    }

    /// Check whether the next token matches `expected`.
    fn next_is(&mut self, expected: &Token) -> bool {
        self.tokens.match_next(|lexed| lexed.token == *expected)
    }

    /// Check whether the `n`th token passes the predicate, where `n` is
    /// zero-indexed.
    fn match_nth(&mut self, n: usize, predicate: impl Fn(Token) -> bool) -> bool {
        self.tokens
            .match_nth(n, |lexed| predicate(lexed.token.clone()))
    }

    /// Check whether the `n`th token matches `expected`, where `n` is
    /// zero-indexed.
    fn nth_is(&mut self, n: usize, expected: &Token) -> bool {
        self.tokens.match_nth(n, |lexed| lexed.token == *expected)
    }

    //
    // The following methods are sub-parsers that are reentrant and handle the
    // parsing of a particular subcontext of the overall source. Each expects
    // the whole context next in the stream, so previous steps working out which
    // sub-parser to delegate to should use peeks and not reads to discern it
    // from subsequent tokens in the buffer.
    //

    fn parse_modifiers(&mut self, whitelist: &HashSet<Modifier>) -> Result<HashSet<Modifier>> {
        let mut results = HashSet::new();
        loop {
            let is_modifier = self.match_next(|token| {
                if let Token::Modifier(ref modifier) = token {
                    whitelist.contains(modifier)
                } else {
                    false
                }
            });
            self.tokens.discard();

            if is_modifier {
                if let Token::Modifier(modifier) = self.read().unwrap() {
                    if results.contains(&modifier) {
                        self.fail(format!("the modifier {:?} was listed twice", modifier))?;
                    } else {
                        results.insert(modifier.clone());
                    }
                } else {
                    unreachable!()
                }
            } else {
                break Ok(results);
            }
        }
    }

    fn parse_symbol(&mut self) -> Result<nodes::Symbol> {
        let mut lookup = vec![];

        Ok(
            if let Some(Token::PseudoIdentifier(pseudo_identifier)) = self.peek() {
                nodes::Symbol::Pseudo(pseudo_identifier)
            } else {
                let new = if self.next_is(&Token::Global) {
                    self.tokens.discard();
                    self.expect_and_discard(Token::Dot)?;
                    nodes::Symbol::Absolute
                } else {
                    nodes::Symbol::Relative
                };

                loop {
                    lookup.push(self.parse_identifier()?);
                    if self.next_is(&Token::Dot) {
                        self.tokens.discard();
                    } else {
                        break new(SymbolLookup(lookup));
                    }
                }
            },
        )
    }

    fn parse_class_definition(&mut self) -> Result<nodes::Type> {
        self.tokens.discard();

        let name = self.parse_identifier()?;
        let sydoc = if let Some(Token::SyDoc(doc)) = self.peek() {
            self.tokens.discard();
            Some(doc)
        } else {
            None
        };

        let has_type_parameters = self.next_is(&Token::Grouping(Grouping::OpenSquareBracket));
        let type_parameters = if has_type_parameters {
            self.parse_type_parameter_list()?
        } else {
            vec![]
        };

        let has_value_parameters = self.next_is(&Token::Grouping(Grouping::OpenParentheses));
        let value_parameters = if has_value_parameters {
            self.parse_class_value_parameters()?
        } else {
            vec![]
        };

        let does_implement = self.next_is(&Token::DeclarationHead(DeclarationHead::Implements));
        let implements = if does_implement {
            self.parse_implements_clause()?
        } else {
            vec![]
        };

        let has_body = self.next_is(&Token::Grouping(Grouping::OpenBrace));
        let (fields, methods, instance_initialiser) = if has_body {
            self.parse_class_body()?
        } else {
            (vec![], vec![], Block::new_root())
        };

        let class = Class {
            implements,
            methods,
            fields,
            value_parameters,
            instance_initialiser,
        };

        Ok(nodes::Type {
            name,
            type_parameters,
            item: nodes::TypeItem::Class(class),
            sydoc,
        })
    }

    fn parse_class_value_parameters(&mut self) -> Result<Vec<nodes::ClassValueParameter>> {
        let mut parameters = vec![];

        loop {
            if self.next_is(&Token::Grouping(Grouping::CloseParentheses)) {
                self.tokens.discard();
                break Ok(parameters);
            }

            let upgraded_to_field = self.next_is(&Token::Binding(Binding::Var));
            if upgraded_to_field {
                self.tokens.discard();
                let parameter = self.parse_class_parameter_field_upgrade()?;
                parameters.push(parameter);
            } else {
                let parameter = nodes::ClassValueParameter {
                    parameter: self.parse_value_parameter()?,
                    field_upgrade: None,
                };
                parameters.push(parameter);
            }

            match self.peek() {
                Some(Token::SubItemSeparator) => {
                    self.tokens.discard();
                }
                Some(Token::Grouping(Grouping::CloseParentheses)) => {
                    self.tokens.discard();
                    break Ok(parameters);
                }
                Some(t) => self.unexpected(t)?,
                None => self.premature_eof()?,
            }
        }
    }

    /// Optional labels complicates parsing value parameter lists. Unlike
    /// type parameters, there isn't an `extends` clause to split type
    /// constraints from names and labels.
    ///
    /// Doing it with a fixed lookahead relies on the fact that complex
    /// irrefuttable pattern matching (i.e. not just a top-level identifier
    /// binding) is always an identifier followed by a non-identifier.
    ///
    /// This comes with a caveat: parameter lists must either completely infer
    /// types or not infer at all, otherwise `label name` is indistinguishable
    /// from `name type`. Lambdas infer all, `fun`s infer nothing. Otherwise the
    /// parser would get lost here. Sylan is therefore really committing to this
    /// design decision...
    fn parse_value_parameter(&mut self) -> Result<nodes::ValueParameter> {
        let is_syntax = if self.match_next(|t| matches!(t, Token::Macros(Macros::Syntax))) {
            self.tokens.discard();
            true
        } else {
            false
        };

        match self.peek() {
            // Either a label or the start of a parameter name.
            Some(Token::Identifier(..)) => {}

            Some(t) => self.unexpected(t)?,
            None => self.premature_eof()?,
        };

        Ok(if let Some(Token::Identifier(..)) = self.peek_nth(1) {
            // Either the type or the start of a parameter pattern after a label.

            match self.peek_nth(2) {
                Some(Token::Grouping(Grouping::CloseParentheses)) => {
                    // Must be a basic parameter name and a type.

                    let pattern = self.parse_pattern()?;
                    let type_annotation = self.parse_type_reference()?;
                    ValueParameter {
                        label: None,
                        pattern,
                        type_annotation,
                        default_value: None,
                        sydoc: None,
                        is_syntax,
                    }
                }
                Some(Token::SubItemSeparator) => {
                    // Must be a basic parameter name and a type.

                    let pattern = self.parse_pattern()?;
                    let type_annotation = self.parse_type_reference()?;
                    self.tokens.discard();
                    ValueParameter {
                        label: None,
                        pattern,
                        type_annotation,
                        default_value: None,
                        sydoc: None,
                        is_syntax,
                    }
                }
                Some(Token::Colon) => {
                    // Must be a basic parameter name and a type with a
                    // default value.

                    let pattern = self.parse_pattern()?;
                    let type_annotation = self.parse_type_reference()?;
                    let default_value = Some(self.parse_default_value()?);
                    let sydoc = if let Some(Token::SyDoc(doc)) = self.peek() {
                        self.tokens.discard();
                        Some(doc)
                    } else {
                        None
                    };
                    ValueParameter {
                        label: None,
                        pattern,
                        type_annotation,
                        default_value,
                        sydoc,
                        is_syntax,
                    }
                }
                Some(_) => {
                    // Must be a label followed by complex pattern matching,
                    // and then a type.

                    let label = Some(self.parse_identifier()?);
                    let pattern = self.parse_pattern()?;
                    let type_annotation = self.parse_type_reference()?;
                    let default_value = if self.next_is(&Token::Colon) {
                        Some(self.parse_default_value()?)
                    } else {
                        None
                    };
                    let sydoc = if let Some(Token::SyDoc(doc)) = self.peek() {
                        self.tokens.discard();
                        Some(doc)
                    } else {
                        None
                    };
                    ValueParameter {
                        label,
                        pattern,
                        type_annotation,
                        default_value,
                        sydoc,
                        is_syntax,
                    }
                }
                None => self.premature_eof()?,
            }
        } else {
            // Must be the start of a complex pattern match without a label.

            let pattern = self.parse_pattern()?;
            let type_annotation = self.parse_type_reference()?;
            let default_value = if self.next_is(&Token::Colon) {
                Some(self.parse_default_value()?)
            } else {
                None
            };
            let sydoc = if let Some(Token::SyDoc(doc)) = self.peek() {
                self.tokens.discard();
                Some(doc)
            } else {
                None
            };
            ValueParameter {
                label: None,
                pattern,
                type_annotation,
                default_value,
                sydoc,
                is_syntax,
            }
        })
    }

    fn parse_default_value(&mut self) -> Result<nodes::Expression> {
        self.expect_and_discard(Token::Colon)?;
        self.parse_expression()
    }

    fn parse_class_parameter_field_upgrade(&mut self) -> Result<nodes::ClassValueParameter> {
        let modifiers = self.parse_modifiers(&self.modifier_sets.field.clone())?;
        let is_embedded = modifiers.contains(&Modifier::Embed);

        let accessibility = self
            .accessibility_modifier_extractor
            .extract_accessibility_modifier(&modifiers)
            .map_err(|err| {
                Error::Parser(ParserError {
                    description: ParserErrorDescription::Described(err),
                })
            })?;

        let field_upgrade = Some(ClassValueParameterFieldUpgrade {
            is_embedded,
            accessibility,
        });

        let parameter = nodes::ClassValueParameter {
            parameter: self.parse_value_parameter()?,
            field_upgrade,
        };

        Ok(parameter)
    }

    fn parse_implements_clause(&mut self) -> Result<Vec<TypeReference>> {
        self.tokens.discard();

        let mut type_symbols = vec![];
        loop {
            type_symbols.push(self.parse_type_reference()?);
            if self.next_is(&Token::SubItemSeparator) {
                self.tokens.discard();
            } else {
                break Ok(type_symbols);
            }
        }
    }

    fn parse_class_body(
        &mut self,
    ) -> Result<(Vec<nodes::Field>, Vec<nodes::ConcreteMethod>, Block)> {
        todo!()
    }

    fn parse_with(&mut self) -> Result<nodes::Expression> {
        self.tokens.discard();
        let scope = self.parse_block()?;
        Ok(Expression::Context(scope))
    }

    fn parse_extension(&mut self) -> Result<nodes::Extension> {
        self.tokens.discard();
        unimplemented!();
    }

    fn parse_for(&mut self) -> Result<nodes::For> {
        self.tokens.discard();

        let reiteration_symbol = if self.next_is(&Token::Binding(tokens::Binding::Var)) {
            self.tokens.discard();
            None
        } else if self.next_is(&Token::Grouping(Grouping::OpenBrace)) {
            None
        } else {
            Some(self.parse_identifier()?)
        };

        let mut bindings = vec![];
        let scope = loop {
            if self.next_is(&Token::Grouping(Grouping::OpenBrace)) {
                break self.parse_block()?;
            } else {
                bindings.push(self.parse_local_binding()?);
                if self.next_is(&Token::SubItemSeparator) {
                    self.tokens.discard();
                }
            }
        };

        Ok(For {
            bindings,
            scope,
            reiteration_symbol,
        })
    }

    fn parse_if(&mut self) -> Result<nodes::If> {
        self.tokens.discard();

        let condition = self.parse_expression()?;
        let then = self.parse_block()?;

        let else_clause = if self.next_is(&Token::BranchingAndJumping(BranchingAndJumping::Else)) {
            self.tokens.discard();

            // Ban braceless ifs except for one case: an else followed immediately by another if.
            Some(
                if self.next_is(&Token::BranchingAndJumping(BranchingAndJumping::If)) {
                    let if_node = self.parse_if()?;
                    Block {
                        expressions: vec![Expression::BranchingAndJumping(
                            nodes::BranchingAndJumping::If(if_node),
                        )],
                        bindings: vec![],
                        parent: Some(self.current_scope.clone()),
                    }
                } else {
                    self.parse_block()?
                },
            )
        } else {
            None
        };

        Ok(If {
            condition: Box::new(condition),
            then,
            else_clause,
        })
    }

    fn parse_type_reference(&mut self) -> Result<nodes::TypeReference> {
        let symbol = self.parse_symbol()?;
        let type_arguments = if self.next_is(&Token::Grouping(Grouping::OpenSquareBracket)) {
            self.parse_type_argument_list()?
        } else {
            vec![]
        };
        Ok(TypeReference {
            symbol,
            type_arguments,
        })
    }

    fn parse_composite_pattern_getter(&mut self, next: &Token) -> Result<Option<PatternGetter>> {
        let second_token_is_colon = self.nth_is(1, &Token::Colon);

        match &next {
            Token::Rest => {
                self.tokens.discard();
                self.expect(Token::Grouping(Grouping::CloseParentheses))?;
                Ok(None)
            }

            Token::Identifier(ref identifier) if !second_token_is_colon => {
                self.tokens.discard();
                let pattern = Pattern {
                    item: PatternItem::Identifier(identifier.clone()),
                    bound_match: None,
                };
                Ok(Some(PatternGetter {
                    name: identifier.clone(),
                    pattern,
                }))
            }

            _ => {
                let name = self.parse_identifier()?;
                self.expect_and_discard(Token::Colon)?;
                let pattern = self.parse_pattern()?;
                Ok(Some(PatternGetter { name, pattern }))
            }
        }
    }

    fn parse_composite_pattern(&mut self) -> Result<nodes::CompositePattern> {
        let token = self
            .peek()
            .map(Ok)
            .unwrap_or_else(|| self.premature_eof())?;

        let infer_enum_type = if self.next_is(&Token::Dot) {
            self.tokens.discard();
            true
        } else {
            false
        };

        if let Token::Identifier(_) = token {
            let r#type = self.parse_type_reference()?;
            self.expect_and_discard(Token::Grouping(Grouping::OpenParentheses))?;

            let mut getters = vec![];
            let ignore_rest = loop {
                let next = self
                    .tokens
                    .peek()
                    .map(|lexed| Ok(lexed.clone().token))
                    .unwrap_or_else(|| self.premature_eof())?;

                if next == Token::Grouping(Grouping::CloseParentheses) {
                    break false;
                } else if let Some(getter) = self.parse_composite_pattern_getter(&next)? {
                    getters.push(getter);
                } else {
                    break true;
                }

                self.expect_and_discard(Token::SubItemSeparator)?;
            };

            self.expect_and_discard(Token::Grouping(Grouping::CloseParentheses))?;

            let composite = CompositePattern {
                r#type,
                getters,
                ignore_rest,
                infer_enum_type,
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
            .unwrap_or_else(|| match token {
                Token::Identifier(identifier) => Ok(PatternItem::Identifier(identifier)),
                Token::PseudoIdentifier(PseudoIdentifier::PlaceholderIdentifier) => {
                    Ok(PatternItem::Ignored)
                }
                Token::Rest => {
                    self.tokens.discard();
                    let symbol = self.parse_symbol()?;
                    Ok(PatternItem::BoundSymbol(symbol))
                }
                _ => {
                    let composite = self.parse_composite_pattern()?;
                    Ok(PatternItem::Composite(composite))
                }
            });

        Ok(Pattern {
            item: item?,
            bound_match: None,
        })
    }

    fn parse_value_argument_list(&mut self) -> Result<Vec<ValueArgument>> {
        self.tokens.discard();

        let mut arguments = vec![];
        loop {
            if self.next_is(&Token::Grouping(Grouping::CloseParentheses)) {
                self.tokens.discard();
                break Ok(arguments);
            }

            let identifier_follows = self.match_next(|t| matches!(t, Token::Identifier(..)));
            let label = if identifier_follows && self.nth_is(1, &Token::Colon) {
                let identifier = match self.read() {
                    Some(Token::Identifier(identifier)) => identifier,
                    Some(unexpected) => self.unexpected(unexpected)?,
                    None => self.premature_eof()?,
                };
                self.tokens.discard();
                Some(identifier)
            } else {
                None
            };

            let expression = self.parse_expression()?;
            let argument = ValueArgument {
                label,
                value: expression,
            };
            arguments.push(argument)
        }
    }

    fn parse_type_argument_list(&mut self) -> Result<Vec<TypeArgument>> {
        self.tokens.discard();

        let mut arguments = vec![];
        loop {
            if self.next_is(&Token::Grouping(Grouping::CloseSquareBracket)) {
                self.tokens.discard();
                break Ok(arguments);
            }

            let identifier_follows = self.match_next(|t| matches!(t, Token::Identifier(..)));
            let label = if identifier_follows && self.nth_is(1, &Token::Colon) {
                let identifier = match self.read() {
                    Some(Token::Identifier(identifier)) => identifier,
                    Some(unexpected) => self.unexpected(unexpected)?,
                    None => self.premature_eof()?,
                };
                self.tokens.discard();
                Some(identifier)
            } else {
                None
            };

            let type_reference = self.parse_type_reference()?;
            let argument = TypeArgument {
                label,
                value: type_reference,
            };
            arguments.push(argument)
        }
    }

    fn parse_imports(&mut self) -> Result<Vec<nodes::Import>> {
        self.tokens.discard();
        let stems = self.parse_inside_import_stems()?;
        self.expect_and_discard(Token::Grouping(Grouping::CloseParentheses))?;
        Ok(stems)
    }

    fn parse_inside_import_stems(&mut self) -> Result<Vec<nodes::Import>> {
        let mut imports = vec![];
        loop {
            let mut whole: Vec<Identifier> = vec![];
            let readers = loop {
                match self.peek() {
                    Some(Token::Identifier(identifier)) => {
                        self.tokens.discard();
                        whole.push(identifier);
                    }
                    Some(Token::Dot) => {
                        self.tokens.discard();
                    }
                    Some(Token::Macros(Macros::Syntax)) => {
                        break self.parse_import_syntax_readers_list()?
                    }
                    _ => break vec![],
                }
            };

            let (root, stem) = if self.next_is(&Token::Grouping(Grouping::OpenBrace)) {
                self.tokens.discard();
                let stems = self.parse_inside_import_stems()?;
                self.expect_and_discard(Token::Grouping(Grouping::CloseBrace))?;
                let stem = nodes::ImportStem::Multiple(stems);
                (None, stem)
            } else {
                let stem = nodes::ImportStem::Single(nodes::ImportSingleStem {
                    name: whole.pop().unwrap(),
                    readers,
                });
                let root = Symbol::Relative(SymbolLookup(whole));
                (Some(root), stem)
            };

            let import = nodes::Import { root, stem };
            imports.push(import);

            if self.next_is(&Token::SubItemSeparator) {
                self.tokens.discard();
            } else {
                break Ok(imports);
            }
        }
    }

    fn parse_import_syntax_readers_list(&mut self) -> Result<Vec<Symbol>> {
        self.tokens.discard();
        self.expect_and_discard(Token::Grouping(Grouping::OpenParentheses))?;
        let mut symbols = vec![];
        loop {
            if self.next_is(&Token::Grouping(Grouping::CloseParentheses)) {
                self.tokens.discard();
                break Ok(symbols);
            }
            symbols.push(self.parse_symbol()?);
            if !self.next_is(&Token::Grouping(Grouping::CloseParentheses)) {
                self.expect_and_discard(Token::SubItemSeparator)?
            }
        }
    }

    fn parse_interface_body(&mut self) -> Result<Vec<Method>> {
        unimplemented!()
    }

    fn parse_interface_definition(&mut self) -> Result<nodes::Type> {
        unimplemented!()
    }

    fn parse_type_constraints(&mut self) -> Result<Vec<TypeReference>> {
        self.tokens.discard();

        let mut constraints = vec![];
        loop {
            constraints.push(self.parse_type_reference()?);
            if self.next_is(&Token::OverloadableInfixOperator(
                OverloadableInfixOperator::Ampersand,
            )) {
                self.expect_and_discard(Token::OverloadableInfixOperator(
                    OverloadableInfixOperator::Ampersand,
                ))?;
            } else {
                break Ok(constraints);
            }
        }
    }

    fn parse_type_parameter_list(&mut self) -> Result<Vec<TypeParameter>> {
        if self.next_is(&Token::Grouping(Grouping::OpenSquareBracket)) {
            let mut list = vec![];
            self.expect_and_discard(Token::Grouping(Grouping::OpenSquareBracket))?;
            loop {
                let identifier = self.parse_identifier()?;
                let (label, name) = if self.match_next(|t| matches!(t, Token::Identifier(..))) {
                    (Some(identifier), self.parse_identifier()?)
                } else {
                    (None, identifier)
                };

                let upper_bounds = if self.next_is(&Token::Extends) {
                    self.parse_type_constraints()?
                } else {
                    vec![]
                };

                let default_value = if self.next_is(&Token::Colon) {
                    self.expect_and_discard(Token::Binding(Binding::Assign))?;
                    Some(self.parse_type_reference()?)
                } else {
                    None
                };

                let sydoc = if let Some(Token::SyDoc(doc)) = self.peek() {
                    self.tokens.discard();
                    Some(doc)
                } else {
                    None
                };

                list.push(TypeParameter {
                    label,
                    name,
                    upper_bounds,
                    default_value,
                    sydoc,
                });

                if self.next_is(&Token::Grouping(Grouping::CloseSquareBracket)) {
                    self.expect_and_discard(Token::Grouping(Grouping::CloseSquareBracket))?;
                    break Ok(list);
                } else {
                    self.expect_and_discard(Token::SubItemSeparator)?;
                }
            }
        } else {
            Ok(vec![])
        }
    }

    fn parse_fun_value_parameter_list(&mut self) -> Result<Vec<ValueParameter>> {
        let mut parameters = vec![];

        loop {
            if self.next_is(&Token::Grouping(Grouping::CloseParentheses)) {
                self.tokens.discard();
                break Ok(parameters);
            }

            let parameter = self.parse_value_parameter()?;
            parameters.push(parameter);

            if self.next_is(&Token::SubItemSeparator) {
                self.tokens.discard();
            }

            match self.peek() {
                Some(Token::SubItemSeparator) => {
                    self.tokens.discard();
                }
                Some(Token::Grouping(Grouping::CloseParentheses)) => {
                    self.tokens.discard();
                    break Ok(parameters);
                }
                Some(t) => self.unexpected(t)?,
                None => self.premature_eof()?,
            }
        }
    }

    fn parse_lambda_value_parameter_list(&mut self) -> Result<Vec<LambdaValueParameter>> {
        let mut parameters = vec![];

        loop {
            if self.next_is(&Token::Grouping(Grouping::OpenBrace)) {
                break Ok(parameters);
            }

            let label = if self.match_nth(1, |t| matches!(t, Token::Identifier(..))) {
                let label = match self.read() {
                    Some(Token::Identifier(identifier)) => identifier,
                    Some(unexpected) => self.unexpected(unexpected)?,
                    None => self.premature_eof()?,
                };
                Some(label)
            } else {
                None
            };

            let pattern = self.parse_pattern()?;

            let default_value = if self.next_is(&Token::Colon) {
                Some(self.parse_default_value()?)
            } else {
                None
            };

            let parameter = nodes::LambdaValueParameter {
                label,
                pattern,
                default_value,
            };

            parameters.push(parameter);

            if self.next_is(&Token::SubItemSeparator) {
                self.tokens.discard();
            }
        }
    }

    fn parse_lambda_result_type_annotation(&mut self) -> Result<Option<TypeReference>> {
        if self.next_is(&Token::Grouping(Grouping::OpenBrace)) {
            Ok(None)
        } else {
            Ok(Some(self.parse_type_reference()?))
        }
    }

    fn parse_lambda_signature(&mut self) -> Result<LambdaSignature> {
        let value_parameters = self.parse_lambda_value_parameter_list()?;

        Ok(LambdaSignature { value_parameters })
    }

    /// Parsing a lambda; this should not happen from a top-level expression, but only a
    /// subexpresion. This is avoid the ambiguity between a lambda literal and the shorthand for
    /// passing a lambda as a final argument, specifically when that shorthand is on a new line.
    fn parse_lambda(&mut self) -> Result<nodes::Lambda> {
        self.expect_and_discard(Token::LambdaArrow)?;
        let signature = self.parse_lambda_signature()?;
        let block = self.parse_block()?;

        Ok(Lambda { signature, block })
    }

    fn parse_fun(&mut self) -> Result<nodes::Fun> {
        self.expect_and_discard(Token::DeclarationHead(DeclarationHead::Fun))?;
        let modifiers = self.parse_modifiers(&self.modifier_sets.function.clone())?;
        let name = self.parse_identifier()?;

        let type_parameters = if self.next_is(&Token::Grouping(Grouping::OpenSquareBracket)) {
            self.parse_type_parameter_list()?
        } else {
            vec![]
        };

        let value_parameters = self.parse_fun_value_parameter_list()?;

        // TODO: resolve the parsing ambiguity between:
        //
        // * Extern void functions that drop return types, with a symbol on the
        //   next line in the main package.
        // * Extern non-void functions that state a return type in the main
        //   package.
        let return_type = todo!();

        let block = self.parse_block()?;

        let accessibility = self
            .accessibility_modifier_extractor
            .extract_accessibility_modifier(&modifiers)
            .map_err(|msg| {
                Error::Parser(ParserError {
                    description: ParserErrorDescription::Described(msg),
                })
            })?;

        let modifiers = FunModifiers {
            accessibility,
            is_extern: todo!(),
            is_operator: modifiers.contains(&Modifier::Operator),
        };

        let signature = FunSignature {
            name,
            sydoc: None,
            type_parameters,
            value_parameters,
            return_type,
        };

        Ok(nodes::Fun {
            modifiers,
            signature,
            block,
        })
    }

    fn parse_package_definition(&mut self) -> Result<nodes::Package> {
        self.expect_and_discard(Token::DeclarationHead(DeclarationHead::Package))?;

        let name = self.parse_identifier()?;
        let has_imports = self.next_is(&Token::Grouping(Grouping::OpenParentheses));
        let imports = if has_imports {
            self.parse_imports()?
        } else {
            vec![]
        };
        self.expect_and_discard(Token::Grouping(Grouping::OpenBrace))?;
        let items = self.parse_inside_package()?;
        self.expect_and_discard(Token::Grouping(Grouping::CloseBrace))?;

        Ok(nodes::Package {
            accessibility: Accessibility::Public,
            name,
            items,
            sydoc: None,
            imports,
        })
    }

    fn parse_local_var_binding(&mut self) -> Result<nodes::Binding> {
        self.tokens.discard();
        self.parse_local_binding()
    }

    fn parse_local_binding(&mut self) -> Result<nodes::Binding> {
        let pattern = self.parse_pattern()?;

        self.expect_and_discard(Token::Binding(Binding::Assign))?;

        let value = self.parse_expression()?;

        Ok(nodes::Binding {
            pattern,
            value: Box::new(value),
            explicit_type_annotation: None,
        })
    }

    fn parse_binding(&mut self) -> Result<nodes::Binding> {
        self.tokens.discard();
        let declaration_modifiers = self.parse_modifiers(&self.modifier_sets.binding.clone())?;

        let pattern = self.parse_pattern()?;

        let explicit_type_annotation = if self.next_is(&Token::Binding(Binding::Assign)) {
            None
        } else {
            Some(self.parse_type_reference()?)
        };
        self.expect_and_discard(Token::Binding(Binding::Assign))?;

        let value = self.parse_expression()?;

        Ok(nodes::Binding {
            pattern,
            value: Box::new(value),
            explicit_type_annotation,
        })
    }

    fn parse_field(&mut self) -> Result<nodes::Field> {
        self.tokens.discard();
        let declaration_modifiers = self.parse_modifiers(&self.modifier_sets.field.clone())?;
        let accessibility = self
            .accessibility_modifier_extractor
            .extract_accessibility_modifier(&declaration_modifiers)
            .map_err(|msg| {
                Error::Parser(ParserError {
                    description: ParserErrorDescription::Described(msg),
                })
            })?;

        let pattern = self.parse_pattern()?;

        let explicit_type_annotation = if self.next_is(&Token::Binding(Binding::Assign)) {
            None
        } else {
            Some(self.parse_type_reference()?)
        };
        self.expect_and_discard(Token::Binding(Binding::Assign))?;

        let is_extern = todo!();

        let value = self.parse_expression()?;

        Ok(nodes::Field {
            accessibility,
            is_embedded: declaration_modifiers.contains(&Modifier::Embed),
            is_extern,
            binding: nodes::Binding {
                pattern,
                value: Box::new(value),
                explicit_type_annotation,
            },
        })
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
        let message_type = self.parse_type_reference()?;
        self.expect_and_discard(Token::Grouping(Grouping::OpenBrace))?;
        let mut cases = vec![];
        let mut timeout = None;

        loop {
            let mut matches = vec![];
            if self.next_is(&Token::Timeout) {
                if timeout.is_none() {
                    let nanoseconds = Box::new(self.parse_expression()?);
                    let body = self.parse_block()?;
                    timeout = Some(Timeout { nanoseconds, body });
                } else {
                    self.unexpected(Token::Timeout)?;
                }
            } else {
                let body = loop {
                    let pattern = self.parse_pattern()?;

                    let guard =
                        if self.next_is(&Token::BranchingAndJumping(BranchingAndJumping::If)) {
                            self.expect_and_discard(Token::BranchingAndJumping(
                                BranchingAndJumping::If,
                            ))?;
                            Some(self.parse_expression()?)
                        } else {
                            None
                        };

                    matches.push(CaseMatch { pattern, guard });

                    if self.next_is(&Token::Grouping(Grouping::OpenBrace)) {
                        break self.parse_block()?;
                    } else {
                        self.expect_and_discard(Token::SubItemSeparator)?;
                    }
                };
                cases.push(Case { matches, body });
            }

            if self.next_is(&Token::Grouping(Grouping::CloseBrace)) {
                self.tokens.discard();
                break Ok(Select {
                    message_type,
                    cases,
                    timeout,
                });
            }
        }
    }

    fn parse_cond(&mut self) -> Result<Cond> {
        self.expect_and_discard(Token::Grouping(Grouping::OpenBrace))?;

        let mut cases = vec![];
        loop {
            let mut conditions = vec![];
            let then = loop {
                let expression = self.parse_expression()?;
                conditions.push(expression);

                if self.next_is(&Token::Grouping(Grouping::OpenBrace)) {
                    break self.parse_block()?;
                } else {
                    self.expect_and_discard(Token::SubItemSeparator)?;
                }
            };
            cases.push(CondCase { conditions, then });

            if self.next_is(&Token::Grouping(Grouping::CloseBrace)) {
                self.tokens.discard();
                break Ok(Cond(cases));
            }
        }
    }

    fn parse_direct_switch(&mut self) -> Result<Switch> {
        let expression = self.parse_expression()?;
        self.expect_and_discard(Token::Grouping(Grouping::OpenBrace))?;
        let mut cases = vec![];

        loop {
            let mut matches = vec![];
            let body = loop {
                let pattern = self.parse_pattern()?;

                let guard = if self.next_is(&Token::BranchingAndJumping(BranchingAndJumping::If)) {
                    self.expect_and_discard(Token::BranchingAndJumping(BranchingAndJumping::If))?;
                    Some(self.parse_expression()?)
                } else {
                    None
                };

                matches.push(CaseMatch { pattern, guard });

                if self.next_is(&Token::Grouping(Grouping::OpenBrace)) {
                    break self.parse_block()?;
                } else {
                    self.expect_and_discard(Token::SubItemSeparator)?;
                }
            };
            cases.push(Case { matches, body });

            if self.next_is(&Token::Grouping(Grouping::CloseBrace)) {
                self.tokens.discard();
                break Ok(Switch {
                    expression: Box::new(expression),
                    cases,
                });
            }
        }
    }

    fn parse_member_handle(&mut self) -> Result<nodes::Expression> {
        self.tokens.discard();
        let symbol = self.parse_symbol()?;
        Ok(nodes::Expression::MemberHandle(symbol))
    }

    fn parse_switch(&mut self) -> Result<Expression> {
        self.tokens.discard();

        if self.next_is(&Token::Grouping(Grouping::OpenBrace)) {
            self.parse_cond()
                .map(|cond| Expression::BranchingAndJumping(nodes::BranchingAndJumping::Cond(cond)))
        } else {
            self.parse_direct_switch().map(|switch| {
                Expression::BranchingAndJumping(nodes::BranchingAndJumping::Switch(switch))
            })
        }
    }

    fn parse_throw(&mut self) -> Result<nodes::Throw> {
        self.tokens.discard();
        let expression = self.parse_expression()?;
        Ok(Throw(Box::new(expression)))
    }

    fn parse_literal(&mut self, token: Token) -> Option<nodes::Literal> {
        match token {
            // Literal tokens are a one-to-one translation to AST nodes
            // except interpolated strings.
            Token::Literal(Literal::Char(c)) => Some(nodes::Literal::Char(c)),
            Token::Literal(Literal::InterpolatedString(string)) => {
                Some(nodes::Literal::InterpolatedString(string))
            }
            Token::Literal(Literal::Number(number)) => Some(nodes::Literal::Number(number)),
            Token::Literal(Literal::String(string)) => Some(nodes::Literal::String(string)),
            _ => None,
        }
    }

    fn parse_leading_identifier(&mut self) -> Result<nodes::Expression> {
        let symbol = self.parse_symbol()?;
        if self.next_is(&Token::Grouping(Grouping::OpenSquareBracket)) {
            let type_arguments = self.parse_type_argument_list()?;
            let arguments = self.parse_value_argument_list()?;
            let call = nodes::Call {
                target: symbol,
                arguments: CallArguments {
                    type_arguments,
                    arguments,
                },
            };
            Ok(Expression::BranchingAndJumping(
                nodes::BranchingAndJumping::Call(call),
            ))
        } else if self.next_is(&Token::Grouping(Grouping::OpenParentheses)) {
            let arguments = self.parse_value_argument_list()?;
            let call = nodes::Call {
                target: symbol,
                arguments: CallArguments {
                    type_arguments: vec![],
                    arguments,
                },
            };
            Ok(Expression::BranchingAndJumping(
                nodes::BranchingAndJumping::Call(call),
            ))
        } else {
            Ok(nodes::Expression::Symbol(self.parse_symbol()?))
        }
    }

    fn parse_typed_expression_call(
        &mut self,
        expression: nodes::Expression,
    ) -> Result<nodes::ExpressionCall> {
        let type_arguments = self.parse_type_argument_list()?;

        let arguments = self.parse_value_argument_list()?;
        Ok(nodes::ExpressionCall {
            target: Box::new(expression),
            arguments: CallArguments {
                type_arguments,
                arguments,
            },
        })
    }

    fn parse_expression_call(
        &mut self,
        expression: nodes::Expression,
    ) -> Result<nodes::ExpressionCall> {
        let arguments = self.parse_value_argument_list()?;
        Ok(nodes::ExpressionCall {
            target: Box::new(expression),
            arguments: CallArguments {
                type_arguments: vec![],
                arguments,
            },
        })
    }

    fn parse_slice(&mut self) -> Result<nodes::MultiSlice> {
        self.tokens.discard();

        let mut slices = vec![];
        loop {
            if self.next_is(&Token::OverloadableSliceOperator(
                multiphase::OverloadableSliceOperator::Close,
            )) {
                self.tokens.discard();
                break Ok(nodes::MultiSlice(slices));
            }

            if self.next_is(&Token::PseudoIdentifier(PseudoIdentifier::Ellipsis)) {
                self.tokens.discard();
                slices.push(nodes::SliceFragment::Ellipsis);
            } else {
                let mut start = None;
                let mut step = None;
                let mut end = None;
                let mut component_number: usize = 0;

                loop {
                    if self.next_is(&Token::Colon) {
                        self.tokens.discard();
                        component_number += 1;
                    } else if self.next_is(&Token::SubItemSeparator) {
                        self.tokens.discard();
                        break;
                    } else if self.next_is(&Token::OverloadableSliceOperator(
                        multiphase::OverloadableSliceOperator::Close,
                    )) {
                        break;
                    } else {
                        let n = match self.read() {
                            Some(Token::Literal(Literal::Number(number))) => number,
                            Some(unexpected) => self.unexpected(unexpected)?,
                            None => self.premature_eof()?,
                        };
                        match component_number {
                            0 => {
                                start = Some(n);
                            }
                            1 => {
                                step = Some(n);
                            }
                            2 => {
                                end = Some(n);
                            }
                            _ => unreachable!(),
                        }

                        // If only two slice components exist, assume step was
                        // skipped rather than the end.
                        if step.is_some() && end.is_none() {
                            end = step;
                            step = None;
                        }
                    }
                }

                slices.push(nodes::SliceFragment::Slice(nodes::Slice {
                    start,
                    step,
                    end,
                }))
            }
        }
    }

    fn parse_expression(&mut self) -> Result<nodes::Expression> {
        let token = self.tokens.peek().cloned();
        let expression = match token {
            Some(lexed) => {
                let token = lexed.token;
                self.parse_literal(token.clone())
                    .map(|literal| Ok(nodes::Expression::Literal(literal)))
                    .unwrap_or_else(|| match token {
                        // Non-atomic tokens each delegate to a dedicated method.
                        Token::With => self.parse_with(),
                        Token::Colon => self.parse_member_handle(),
                        Token::BranchingAndJumping(BranchingAndJumping::For) => {
                            Ok(nodes::Expression::BranchingAndJumping(
                                nodes::BranchingAndJumping::For(self.parse_for()?),
                            ))
                        }
                        Token::BranchingAndJumping(BranchingAndJumping::If) => {
                            self.parse_if().map(|if_token| {
                                nodes::Expression::BranchingAndJumping(
                                    nodes::BranchingAndJumping::If(if_token),
                                )
                            })
                        }
                        Token::LambdaArrow => self
                            .parse_lambda()
                            .map(|f| nodes::Expression::Literal(nodes::Literal::Lambda(f))),
                        Token::Grouping(Grouping::OpenParentheses) => {
                            self.parse_grouped_expression()
                        }
                        Token::BranchingAndJumping(BranchingAndJumping::Select) => {
                            self.parse_select().map(|select| {
                                nodes::Expression::BranchingAndJumping(
                                    nodes::BranchingAndJumping::Select(select),
                                )
                            })
                        }
                        Token::Identifier(..) | Token::PseudoIdentifier(..) => {
                            self.parse_leading_identifier()
                        }
                        Token::BranchingAndJumping(BranchingAndJumping::Switch) => {
                            self.parse_switch()
                        }
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
        }?;

        match self.peek() {
            Some(Token::Grouping(Grouping::OpenParentheses)) => Ok(
                nodes::Expression::BranchingAndJumping(nodes::BranchingAndJumping::ExpressionCall(
                    self.parse_expression_call(expression)?,
                )),
            ),
            Some(Token::Grouping(Grouping::OpenSquareBracket)) => Ok(
                nodes::Expression::BranchingAndJumping(nodes::BranchingAndJumping::ExpressionCall(
                    self.parse_typed_expression_call(expression)?,
                )),
            ),
            Some(Token::OverloadableSliceOperator(multiphase::OverloadableSliceOperator::Open)) => {
                Ok(Expression::Operator(Operator::MultiSlice(
                    self.parse_slice()?,
                )))
            }
            Some(Token::PostfixOperator(operator)) => Ok(Expression::Operator(
                nodes::Operator::Postfix(Box::new(expression), operator),
            )),
            Some(Token::OverloadableInfixOperator(operator)) => {
                Ok(Expression::Operator(Operator::OverloadableInfix(
                    Box::new(expression),
                    operator,
                    Box::new(self.parse_expression()?),
                )))
            }
            _ => Ok(expression),
        }
    }

    /// Outermost expressions are the same as any other expression except for disallowing grouped
    /// subexpressions with parentheses and lambda literals. Both of those exclusions are to make
    /// parsing unambiguous without requiring explicit line continuations.
    fn parse_outermost_expression(&mut self) -> Result<nodes::Expression> {
        let token = self.tokens.peek().cloned();
        let expression = match token {
            Some(lexed) => {
                let token = lexed.token;
                self.parse_literal(token.clone())
                    .map(|literal| Ok(nodes::Expression::Literal(literal)))
                    .unwrap_or_else(|| match token {
                        // Non-atomic tokens each delegate to a dedicated method.
                        Token::With => self.parse_with(),
                        Token::Colon => self.parse_member_handle(),
                        Token::BranchingAndJumping(BranchingAndJumping::For) => {
                            Ok(nodes::Expression::BranchingAndJumping(
                                nodes::BranchingAndJumping::For(self.parse_for()?),
                            ))
                        }
                        Token::BranchingAndJumping(BranchingAndJumping::If) => {
                            self.parse_if().map(|if_token| {
                                nodes::Expression::BranchingAndJumping(
                                    nodes::BranchingAndJumping::If(if_token),
                                )
                            })
                        }
                        Token::Identifier(..) | Token::PseudoIdentifier(..) => {
                            self.parse_leading_identifier()
                        }
                        Token::BranchingAndJumping(BranchingAndJumping::Select) => {
                            self.parse_select().map(|select| {
                                nodes::Expression::BranchingAndJumping(
                                    nodes::BranchingAndJumping::Select(select),
                                )
                            })
                        }
                        Token::BranchingAndJumping(BranchingAndJumping::Switch) => {
                            self.parse_switch()
                        }
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
        }?;

        match self.peek() {
            Some(Token::Grouping(Grouping::OpenParentheses)) => Ok(
                nodes::Expression::BranchingAndJumping(nodes::BranchingAndJumping::ExpressionCall(
                    self.parse_expression_call(expression)?,
                )),
            ),
            Some(Token::Grouping(Grouping::OpenSquareBracket)) => Ok(
                nodes::Expression::BranchingAndJumping(nodes::BranchingAndJumping::ExpressionCall(
                    self.parse_typed_expression_call(expression)?,
                )),
            ),
            Some(Token::OverloadableSliceOperator(multiphase::OverloadableSliceOperator::Open)) => {
                Ok(Expression::Operator(Operator::MultiSlice(
                    self.parse_slice()?,
                )))
            }
            Some(Token::PostfixOperator(operator)) => Ok(Expression::Operator(
                nodes::Operator::Postfix(Box::new(expression), operator),
            )),
            Some(Token::OverloadableInfixOperator(operator)) => {
                Ok(Expression::Operator(Operator::OverloadableInfix(
                    Box::new(expression),
                    operator,
                    Box::new(self.parse_expression()?),
                )))
            }
            _ => Ok(expression),
        }
    }

    fn parse_block(&mut self) -> Result<nodes::Block> {
        let mut bindings = vec![];
        let mut expressions = vec![];

        self.expect_and_discard(Token::Grouping(Grouping::OpenBrace))?;
        loop {
            if self.next_is(&Token::Binding(Binding::Var)) {
                bindings.push(self.parse_local_var_binding()?);
            } else if self.next_is(&Token::Grouping(Grouping::CloseBrace)) {
                self.tokens.discard();
                break;
            } else {
                expressions.push(self.parse_outermost_expression()?);
            }
        }

        Ok(Block {
            expressions,
            bindings,
            parent: Some(Rc::new(Block::within(&self.current_scope))),
        })
    }

    fn parse_grouped_expression(&mut self) -> Result<nodes::Expression> {
        self.tokens.discard();
        let expression = self.parse_expression()?;
        self.expect_and_discard(Token::Grouping(Grouping::CloseParentheses))?;
        Ok(expression)
    }

    fn parse_inside_package(&mut self) -> Result<Vec<nodes::Item>> {
        let mut items: Vec<Item> = vec![];

        loop {
            let maybe_token = self.tokens.peek().map(|lexed| lexed.token.clone());

            match maybe_token {
                None => break,

                Some(token) => match token {
                    Token::DeclarationHead(DeclarationHead::Class) => {
                        let class_definition = self.parse_class_definition()?;
                        items.push(Item::Type(class_definition));
                    }
                    Token::DeclarationHead(DeclarationHead::Extend) => {
                        let extension = self.parse_extension()?;
                        items.push(Item::Extension(extension));
                    }
                    Token::DeclarationHead(DeclarationHead::Interface) => {
                        let interface = self.parse_interface_definition()?;
                        items.push(Item::Type(interface));
                    }
                    Token::DeclarationHead(DeclarationHead::Package) => {
                        let package = self.parse_package_definition()?;
                        items.push(Item::Package(package));
                    }
                    Token::DeclarationHead(DeclarationHead::Fun) => {
                        let fun = self.parse_fun()?;
                        items.push(Item::Fun(fun));
                    }
                    Token::Binding(Binding::Final) => {
                        let binding = self.parse_binding()?;
                        items.push(Item::Final(binding));
                    }

                    unexpected => self.unexpected(unexpected)?,
                },
            }
        }

        Ok(items)
    }

    fn parse_main_package(&mut self) -> Result<nodes::MainPackage> {
        let mut items: Vec<Item> = vec![];

        let mut implicit_main = Block::new_root();

        self.expect_and_discard(Token::DeclarationHead(DeclarationHead::Package))?;

        let name = self.parse_identifier()?;
        let has_imports = self.next_is(&Token::Grouping(Grouping::OpenParentheses));
        let imports = if has_imports {
            self.parse_imports()?
        } else {
            vec![]
        };

        loop {
            let maybe_token = self.tokens.peek().map(|lexed| lexed.token.clone());

            match maybe_token {
                None => break,

                Some(token) => {
                    match token {
                        Token::DeclarationHead(DeclarationHead::Class) => {
                            let class_definition = self.parse_class_definition()?;
                            items.push(Item::Type(class_definition));
                        }
                        Token::DeclarationHead(DeclarationHead::Extend) => {
                            let extension = self.parse_extension()?;
                            items.push(Item::Extension(extension));
                        }
                        Token::DeclarationHead(DeclarationHead::Interface) => {
                            let interface = self.parse_interface_definition()?;
                            items.push(Item::Type(interface));
                        }
                        Token::DeclarationHead(DeclarationHead::Package) => {
                            let package = self.parse_package_definition()?;
                            items.push(Item::Package(package));
                        }
                        Token::DeclarationHead(DeclarationHead::Fun) => {
                            let fun = self.parse_fun()?;
                            items.push(Item::Fun(fun));
                        }
                        Token::Binding(Binding::Final) => {
                            let binding = self.parse_binding()?;
                            items.push(Item::Final(binding));
                        }

                        // Unlike all other packages, the main package allows both variables
                        // without type annotations, falling back to type inference, and also
                        // arbritary expressions.
                        Token::Binding(Binding::Var) => {
                            let binding = self.parse_local_var_binding()?;
                            implicit_main.bindings.push(binding);
                        }
                        _ => {
                            let expression = self.parse_expression()?;
                            implicit_main.expressions.push(expression);
                        }
                    }
                }
            }
        }

        let package = Package {
            items,
            accessibility: Accessibility::Public,
            name,
            sydoc: None,
            imports,
        };

        Ok(MainPackage {
            package,
            block: implicit_main,
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

    fn parse_main_file(&mut self) -> Result<nodes::MainFile> {
        let shebang = self.maybe_parse_shebang();
        let version = self.maybe_parse_version();
        let main_package = self.parse_main_package();

        main_package.map(|main| nodes::MainFile {
            shebang,
            version,
            package: main,
        })
    }

    /// Parse an AST from a lexer, ensuring the underlying lexer task has
    /// finished before continuing.
    pub fn parse(mut self) -> Result<nodes::MainFile> {
        let file = self.parse_main_file();
        let join_handle = self.tokens.join_lexer_thread();
        join_handle.map_err(|err| {
            let description = ParserErrorDescription::LexerThreadFailed(format!(
                "parsing failed due to not being able to join on the lexer thread: {:?}",
                err,
            ));
            Error::Parser(ParserError { description })
        })?;
        file
    }
}
