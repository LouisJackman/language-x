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
//! expression that returns Sylan's unit type `Void` or an `ignorable`
//! expression that throws away an `ignoreable`  method's or function's
//! non-`Void` return value. That said, the only item allowed inside expressions
//! (e.g. within the body of a lambda in the middle of an outer expression),
//! variable bindings with `var`, pretty much look and feel like statements from
//! other languages.
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

use std::collections::{HashSet, LinkedList};
use std::default::Default;
use std::rc::Rc;
use std::result;
use std::sync::Arc;

use crate::common::multiphase::{
    self, Accessibility, Identifier, OverloadableInfixOperator, PseudoIdentifier,
};
use crate::common::peekable_buffer::PeekableBuffer;
use crate::common::version::Version;
use crate::lexing::lexer::{self, LexedToken};
use crate::lexing::tokens::{
    self, Binding, BranchingAndJumping, DeclarationHead, Grouping, Literal, Modifier, Token,
};
use crate::lexing::Tokens;
use crate::parsing::modifier_sets::{AccessibilityModifierExtractor, ModifierSets};
use crate::parsing::nodes::{
    Block, Case, CaseMatch, Class, CompositePattern, Cond, CondCase, Expression, Extension, For,
    Fun, FunModifiers, FunSignature, If, Item, Lambda, LambdaSignature, LambdaValueParameter,
    MainPackage, Method, Node, Operator, Package, Pattern, PatternGetter, PatternItem, Select,
    Switch, Throw, Timeout, Type, TypeParameter, TypeSymbol, ValueParameter,
};

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

fn new_void() -> TypeSymbol {
    TypeSymbol::new(vec![Identifier::from("Void")])
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

    fn parse_modifiers(&mut self, whitelist: &HashSet<Modifier>) -> Result<HashSet<Modifier>> {
        let mut results = HashSet::new();
        loop {
            if self.tokens.match_next(|lexed| {
                if let Token::Modifier(ref modifier) = lexed.token {
                    whitelist.contains(modifier)
                } else {
                    false
                }
            }) {
                if let Token::Modifier(modifier) = self.tokens.read().unwrap().token {
                    if results.contains(&modifier) {
                        self.fail(format!("the modifier {:?} was listed twice", modifier))?;
                    } else {
                        results.insert(modifier.clone());
                    }
                } else {
                    panic!("invalid state");
                }
            } else {
                break Ok(results);
            }
        }
    }

    fn parse_lookup(&mut self) -> Result<nodes::Symbol> {
        let mut lookup = vec![];
        loop {
            lookup.push(self.parse_identifier()?);
            if self.next_is(&Token::Dot) {
                self.tokens.discard();
            } else {
                break Ok(nodes::Symbol(lookup));
            }
        }
    }

    fn parse_class_body(&mut self) -> Result<HashSet<Method>> {
        todo!()
    }

    fn parse_class_definition(&mut self) -> Result<nodes::Type> {
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

    fn parse_for(&mut self) -> Result<nodes::Expression> {
        self.tokens.discard();
        let label = if self.next_is(&Token::Binding(tokens::Binding::Var))
            || self.next_is(&Token::Grouping(Grouping::OpenBrace))
        {
            None
        } else {
            Some(self.parse_identifier()?)
        };

        let mut bindings = vec![];
        let scope = loop {
            if self.next_is(&Token::Grouping(Grouping::OpenBrace)) {
                break self.parse_block()?;
            } else {
                self.expect_and_discard(Token::Binding(Binding::Var))?;
                bindings.push(self.parse_local_binding()?);
                if self.next_is(&Token::SubItemSeparator) {
                    self.tokens.discard();
                }
            }
        };

        Ok(nodes::Expression::BranchingAndJumping(
            nodes::BranchingAndJumping::For(For {
                bindings,
                scope,
                label,
            }),
        ))
    }

    fn parse_if(&mut self) -> Result<nodes::If> {
        self.tokens.discard();

        let condition = self.parse_expression()?;
        let then = self.parse_block()?;

        let else_clause = if self.next_is(&Token::BranchingAndJumping(BranchingAndJumping::Else)) {
            self.tokens.discard();
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(If {
            condition: Box::new(condition),
            then,
            else_clause,
        })
    }

    fn parse_type_symbol_lookup(&mut self) -> Result<nodes::TypeSymbol> {
        todo!()
    }

    fn parse_composite_pattern_getter(&mut self, next: &Token) -> Result<Option<PatternGetter>> {
        let next_token_is_assign = self.nth_is(1, &Token::Binding(Binding::Assign));

        match &next {
            Token::Rest => {
                self.tokens.discard();
                self.expect(Token::Grouping(Grouping::CloseParentheses))?;
                Ok(None)
            }

            Token::Identifier(ref identifier) if !next_token_is_assign => {
                self.tokens.discard();
                let pattern = Pattern {
                    item: PatternItem::Identifier(identifier.clone()),
                    bound_match: None,
                };
                Ok(Some(PatternGetter {
                    label: Some(identifier.clone()), // TODO: parse separete labels too
                    name: identifier.clone(),
                    pattern,
                }))
            }

            _ => {
                let name = self.parse_identifier()?;
                self.expect_and_discard(Token::Binding(Binding::Assign))?;
                let pattern = self.parse_pattern()?;
                Ok(Some(PatternGetter {
                    label: Some(name.clone()), // TODO: parse separate labels too
                    name,
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
            let r#type = self.parse_type_symbol_lookup()?;
            self.expect_and_discard(Token::Grouping(Grouping::OpenParentheses))?;

            let mut getters = vec![];
            let mut ignore_rest = false;
            loop {
                let next = self
                    .tokens
                    .peek()
                    .map(|lexed| Ok(lexed.clone().token))
                    .unwrap_or_else(|| self.premature_eof())?;

                if next == Token::Grouping(Grouping::CloseParentheses) {
                    break;
                } else if let Some(getter) = self.parse_composite_pattern_getter(&next)? {
                    getters.push(getter);
                } else {
                    ignore_rest = true;
                }

                self.expect_and_discard(Token::SubItemSeparator)?;
            }

            self.expect_and_discard(Token::Grouping(Grouping::CloseParentheses))?;

            let composite = CompositePattern {
                r#type,
                getters,
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
            .unwrap_or_else(|| match token {
                Token::Identifier(identifier) => Ok(PatternItem::Identifier(identifier)),
                Token::PseudoIdentifier(PseudoIdentifier::PlaceholderIdentifier) => {
                    Ok(PatternItem::Ignored)
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

    fn parse_type_argument_list(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn parse_binary_operator(&mut self) -> Result<()> {
        // TODO: implement precedence rather than just left-to-right.

        unimplemented!()
    }

    fn parse_import(&mut self) -> Result<nodes::Symbol> {
        self.tokens.discard();
        self.parse_lookup()
    }

    fn parse_interface_body(&mut self) -> Result<HashSet<Method>> {
        unimplemented!()
    }

    fn parse_interface_definition(&mut self) -> Result<nodes::Type> {
        unimplemented!()
    }

    fn parse_type_constraints(&mut self) -> Result<Vec<TypeSymbol>> {
        let mut constraints = vec![];
        loop {
            constraints.push(self.parse_type_symbol_lookup()?);
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
                let name = self.parse_identifier()?;
                let upper_bounds = if self.next_is(&Token::Colon) {
                    self.expect_and_discard(Token::Colon)?;
                    self.parse_type_constraints()?
                } else {
                    vec![]
                };
                let default_value = if self.next_is(&Token::Binding(Binding::Assign)) {
                    self.expect_and_discard(Token::Binding(Binding::Assign))?;
                    Some(self.parse_type_symbol_lookup()?)
                } else {
                    None
                };

                list.push(TypeParameter {
                    label: Some(name.clone()), // TODO: parse separate label too.
                    name,
                    upper_bounds,
                    default_value,
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

    fn parse_lambda_value_parameter_list(&mut self) -> Result<Vec<LambdaValueParameter>> {
        let mut parameters = vec![];

        let wrapped_in_parentheses = self.next_is(&Token::Grouping(Grouping::OpenBrace));
        if wrapped_in_parentheses {
            self.expect_and_discard(Token::Grouping(Grouping::OpenParentheses))?;
        }

        loop {
            let pattern = self.parse_pattern()?;

            let default_value = if self.next_is(&Token::Binding(Binding::Assign)) {
                self.tokens.discard();
                Some(self.parse_expression()?)
            } else {
                None
            };

            let parameter = nodes::LambdaValueParameter {
                label: None, // TODO: parse labels
                pattern,
                default_value,
            };

            parameters.push(parameter);

            if self.next_is(&Token::SubItemSeparator) {
                self.tokens.discard();
            } else {
                if wrapped_in_parentheses {
                    self.expect_and_discard(Token::Grouping(Grouping::CloseParentheses))?;
                }
                break Ok(parameters);
            }
        }
    }

    fn parse_lambda_result_type_annotation(&mut self) -> Result<Option<TypeSymbol>> {
        if self.next_is(&Token::Grouping(Grouping::OpenBrace)) {
            Ok(None)
        } else {
            Ok(Some(self.parse_type_symbol_lookup()?))
        }
    }

    fn parse_fun_signature(&mut self) -> Result<FunSignature> {
        unimplemented!()
    }

    fn parse_lambda_signature(&mut self) -> Result<LambdaSignature> {
        let ignorable = self.next_is(&Token::Modifier(Modifier::Ignorable));
        if ignorable {
            self.expect_and_discard(Token::Modifier(Modifier::Ignorable))?;
        }

        let value_parameters = self.parse_lambda_value_parameter_list()?;

        Ok(LambdaSignature {
            value_parameters,
            ignorable,
        })
    }

    /// Parsing a lambda; this should not happen from a top-level expression, but only a
    /// subexpresion. This is avoid the ambiguity between a lambda literal and the shorthand for
    /// passing a lambda as a final argument, specifically when that shorthand is on a new line.
    fn parse_lambda(&mut self) -> Result<nodes::Lambda> {
        self.expect_and_discard(Token::LambdaArrow)?;
        let signature = self.parse_lambda_signature()?;
        self.expect(Token::Grouping(Grouping::OpenBrace))?;
        let block = self.parse_block()?;

        Ok(Lambda { signature, block })
    }

    fn parse_fun(&mut self) -> Result<nodes::Fun> {
        self.expect_and_discard(Token::DeclarationHead(DeclarationHead::Fun))?;
        let modifiers = self.parse_modifiers(&self.modifier_sets.function.clone())?;
        let name = self.parse_identifier()?;
        let signature = self.parse_fun_signature()?;
        let block = self.parse_block()?;

        let accessibility = self
            .accessibility_modifier_extractor
            .extract_accessibilty_modifier(&modifiers)
            .map_err(|msg| {
                Error::Parser(ParserError {
                    description: ParserErrorDescription::Described(msg),
                })
            })?;

        let modifiers = FunModifiers {
            accessibility,
            is_ignorable: modifiers.contains(&Modifier::Ignorable),
            is_extern: modifiers.contains(&Modifier::Extern),
            is_operator: modifiers.contains(&Modifier::Operator),
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
        self.expect_and_discard(Token::Grouping(Grouping::OpenBrace))?;
        let items = self.parse_inside_package()?;
        self.expect_and_discard(Token::Grouping(Grouping::CloseBrace))?;

        Ok(nodes::Package {
            accessibility: Accessibility::Public,
            name,
            items,
            sydoc: None,
        })
    }

    fn parse_local_binding(&mut self) -> Result<nodes::Binding> {
        self.tokens.discard();
        let pattern = self.parse_pattern()?;

        let explicit_type_annotation = if self.next_is(&Token::Binding(Binding::Assign)) {
            None
        } else {
            Some(self.parse_type_symbol_lookup()?)
        };
        self.expect_and_discard(Token::Binding(Binding::Assign))?;

        let value = self.parse_expression()?;

        Ok(nodes::Binding {
            pattern,
            value: Box::new(value),
            explicit_type_annotation,
        })
    }

    fn parse_binding(&mut self) -> Result<nodes::Binding> {
        self.tokens.discard();
        let declaration_modifiers = self.parse_modifiers(&self.modifier_sets.binding.clone())?;

        let pattern = self.parse_pattern()?;

        let explicit_type_annotation = if self.next_is(&Token::Binding(Binding::Assign)) {
            None
        } else {
            Some(self.parse_type_symbol_lookup()?)
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
            .extract_accessibilty_modifier(&declaration_modifiers)
            .map_err(|msg| {
                Error::Parser(ParserError {
                    description: ParserErrorDescription::Described(msg),
                })
            })?;

        let pattern = self.parse_pattern()?;

        let explicit_type_annotation = if self.next_is(&Token::Binding(Binding::Assign)) {
            None
        } else {
            Some(self.parse_type_symbol_lookup()?)
        };
        self.expect_and_discard(Token::Binding(Binding::Assign))?;

        let is_extern = declaration_modifiers.contains(&Modifier::Extern);

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
        let message_type = self.parse_type_symbol_lookup()?;
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

        // TODO: revisit the data types used for accumulating cases in switches and conds; perhaps
        // linked list or set would be better suited?
        let mut cases = vec![];

        loop {
            let mut conditions = LinkedList::new();
            let then = loop {
                let expression = self.parse_expression()?;
                conditions.push_back(expression);

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
            Token::Literal(Literal::Number(decimal, fraction)) => {
                Some(nodes::Literal::Number(decimal, fraction))
            }
            Token::Literal(Literal::String(string)) => Some(nodes::Literal::String(string)),
            _ => None,
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
                        Token::BranchingAndJumping(BranchingAndJumping::For) => self.parse_for(),
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

        let next_token = self.tokens.peek().cloned();
        if let Some(LexedToken {
            token: Token::PostfixOperator(operator),
            ..
        }) = next_token
        {
            self.tokens.discard();
            Ok(Expression::Operator(nodes::Operator::PostfixOperator(
                Box::new(expression),
                operator,
            )))
        } else {
            Ok(expression)
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
                        Token::BranchingAndJumping(BranchingAndJumping::For) => self.parse_for(),
                        Token::BranchingAndJumping(BranchingAndJumping::If) => {
                            self.parse_if().map(|if_token| {
                                nodes::Expression::BranchingAndJumping(
                                    nodes::BranchingAndJumping::If(if_token),
                                )
                            })
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

        let next_token = self.tokens.peek().cloned();
        if let Some(LexedToken {
            token: Token::PostfixOperator(operator),
            ..
        }) = next_token
        {
            self.tokens.discard();
            Ok(Expression::Operator(nodes::Operator::PostfixOperator(
                Box::new(expression),
                operator,
            )))
        } else {
            Ok(expression)
        }
    }

    fn parse_block(&mut self) -> Result<nodes::Block> {
        let mut bindings = vec![];
        let mut expressions = vec![];

        self.expect_and_discard(Token::Grouping(Grouping::OpenBrace))?;
        loop {
            if self.next_is(&Token::Binding(Binding::Var)) {
                bindings.push(self.parse_local_binding()?);
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
                    Token::DeclarationHead(DeclarationHead::Import) => {
                        let import = self.parse_import()?;
                        items.push(Item::Import(import));
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
                    Token::Binding(Binding::Var) => {
                        let binding = self.parse_binding()?;
                        items.push(Item::Binding(binding));
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
                        Token::DeclarationHead(DeclarationHead::Import) => {
                            let import = self.parse_import()?;
                            items.push(Item::Import(import));
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

                        // Unlike all other packages, the main package allows both variables
                        // without type annotations, falling back to type inference, and also
                        // arbritary expressions.
                        Token::Binding(Binding::Var) => {
                            let binding = self.parse_local_binding()?;
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
            name: Identifier(Arc::new(String::from("main"))),
            sydoc: None,
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