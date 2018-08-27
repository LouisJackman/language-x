mod nodes;

use lexing::lexer::{self, LexedToken};
use lexing::tokens::Token;
use lexing::Tokens;
use multiphase::{self, Identifier};
use parsing::nodes::Expression::{self, Literal, UnaryOperator};
use parsing::nodes::{
    Accessibility, Binding, Case, Code, ContextualBinding, ContextualCode, ContextualScope, DeclarationItem,
    FilePackage, If, Import, MainPackage, Package, Pattern, PatternItem, Scope, Select, Switch, Timeout, Throw, TypeDeclaration,
};
use peekable_buffer::PeekableBuffer;
use std::collections::{HashSet, LinkedList};
use std::rc::Rc;
use std::result;
use std::sync::Arc;
use version::Version;

// TODO: break cycles in scopes to cleanup memory properly.

pub enum ParserErrorDescription {
    Described(String),
    Expected(Token),
    Unexpected(Token),
    PrematureEof,
}

pub struct ParserError {
    description: ParserErrorDescription,
}

pub enum Error {
    Lexer(lexer::Error),
    Parser(ParserError),
}

type Result<T> = result::Result<T, Error>;

fn is_ignored_binding(identifier: Identifier) -> bool {
    let Identifier(string) = identifier;
    (*string) == "_"
}

fn get_item_accessibility(identifier: &Identifier) -> Accessibility {
    let Identifier(string) = identifier;
    if string.starts_with('_') {
        Accessibility::Private
    } else {
        Accessibility::Public
    }
}

pub struct Parser {
    tokens: Tokens,
    current_scope: Rc<Scope>,
}

impl Parser {
    pub fn from(tokens: Tokens) -> Self {
        Self {
            tokens,
            current_scope: Scope::new_root(),
        }
    }

    fn fail<T>(&self, message: impl Into<String>) -> Result<T> {
        Err(Error::Parser(ParserError {
            description: ParserErrorDescription::Described(message.into()),
        }))
    }

    fn expected<T>(&self, expected: Token) -> Result<T> {
        Err(Error::Parser(ParserError {
            description: ParserErrorDescription::Expected(expected),
        }))
    }


    fn expect(&mut self, expected: Token) -> Result<()> {
        if self.next_is(&expected) {
            Ok(())
        } else {
            self.expected(expected)
        }
    }

    fn unexpected<T>(&self, unexpected: Token) -> Result<T> {
        Err(Error::Parser(ParserError {
            description: ParserErrorDescription::Unexpected(unexpected),
        }))
    }

    fn premature_eof<T>(&self) -> Result<T> {
        Err(Error::Parser(ParserError {
            description: ParserErrorDescription::PrematureEof,
        }))
    }

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

    fn next_is(&mut self, expected: &Token) -> bool {
        self.tokens.match_next(|lexed| lexed.token == *expected)
    }

    fn parse_unary_operator(&mut self, operator: nodes::UnaryOperator,) -> Result<nodes::Expression> {
        self.tokens.discard();
        self.parse_expression()
            .map(|expression| UnaryOperator(operator, Box::new(expression)))
    }

    fn parse_unary_add(&mut self) -> Result<nodes::Expression> {
        self.parse_unary_operator(nodes::UnaryOperator::Positive)
    }

    fn parse_bitwise_not(&mut self) -> Result<nodes::Expression> {
        self.parse_unary_operator(nodes::UnaryOperator::BitwiseNot)
    }

    fn parse_bitwise_xor(&mut self) -> Result<nodes::Expression> {
        self.parse_unary_operator(nodes::UnaryOperator::BitwiseXor)
    }

    fn parse_method_handle(&mut self) -> Result<nodes::Expression> {
        self.parse_unary_operator(nodes::UnaryOperator::MethodHandle)
    }

    fn parse_not(&mut self) -> Result<nodes::Expression> {
        self.parse_unary_operator(nodes::UnaryOperator::Not)
    }

    fn parse_negate(&mut self) -> Result<nodes::Expression> {
        self.parse_unary_operator(nodes::UnaryOperator::Negate)
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

    fn parse_do(&mut self) -> Result<nodes::ContextualScope> {
        let mut bindings = HashSet::new();
        let mut contextual_bindings = HashSet::new();
        let mut expressions = vec![];

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

        Ok(ContextualScope {
            code: ContextualCode {
                bindings,
                contextual_bindings,
                expressions,
            },
            parent: Some(Scope::within(&self.current_scope)),
        })
    }

    fn parse_extends(&mut self) -> Result<nodes::TypeDeclaration> {
        self.tokens.discard();
        let specification = self.parse_type_specification()?;
        Ok(TypeDeclaration::Extension(specification))
    }

    fn parse_for(&mut self) -> Result<nodes::Expression> {
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

        Ok(nodes::Expression::For(bindings, scope))
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

    fn parse_pattern(&mut self) -> Result<nodes::Pattern> {
        unimplemented!()
    }

    fn parse_import(&mut self) -> Result<nodes::Import> {
        self.tokens.discard();
        let lookup = self.parse_package_lookup()?;
        Ok(Import { lookup })
    }

    fn parse_interface(&mut self) -> Result<nodes::Interface> {
        unimplemented!()
    }

    fn parse_lambda(&mut self) -> Result<nodes::Function> {
        unimplemented!()
    }

    fn parse_package_definition(&mut self) -> Result<nodes::Package> {
        self.expect_and_discard(Token::Package)?;

        let name = self.parse_identifier()?;
        self.expect_and_discard(Token::OpenBrace)?;
        let (imports, declarations) = self.parse_inside_package()?;
        self.expect_and_discard(Token::CloseBrace)?;

        Ok(nodes::Package {
            accessibility: get_item_accessibility(&name),
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
                    timeout = Some(Timeout { nanoseconds, body,});
                } else {
                    self.unexpected(Token::Timeout)?;
                }
            } else {
                let body = loop {
                    let pattern_match = if self.next_is(&Token::Default) {
                        Pattern {
                            item: PatternItem::Ignored,
                            binding: None,
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
                cases.push(Case {
                    matches,
                    body,
                });
            }

            if self.next_is(&Token::CloseBrace) {
                self.tokens.discard();
                break Ok(Select {
                    cases,
                    timeout,
                })
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
                        binding: None,
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
            cases.push(Case {
                matches,
                body,
            });

            if self.next_is(&Token::CloseBrace) {
                self.tokens.discard();
                break Ok(Switch {
                    expression: Box::new(expression),
                    cases,
                })
            }
        }
    }

    fn parse_throw(&mut self) -> Result<nodes::Throw> {
        self.tokens.discard();
        let expression = self.parse_expression()?;
        Ok(Throw(Box::new(expression)))
    }

    fn parse_open_parentheses(&mut self) -> Result<nodes::Expression> {
        self.tokens.discard();
        unimplemented!()
    }

    fn parse_expression(&mut self) -> Result<nodes::Expression> {
        let token = self.tokens.peek().map(|x| x.clone());
        match token {
            Some(lexed) => {
                match lexed.token {
                    // Literal tokens are a one-to-one translation to AST nodes
                    // except interpolated strings.
                    Token::Boolean(b) => Ok(Literal(nodes::Literal::Boolean(b))),
                    Token::Char(c) => Ok(Literal(nodes::Literal::Char(c))),
                    Token::Identifier(identifier) => Ok(nodes::Expression::Identifier(identifier)),
                    Token::InterpolatedString(string) => {
                        // TODO: reenter the lexer to handle interpolation
                        // properly.
                        Ok(Literal(nodes::Literal::InterpolatedString(string)))
                    }
                    Token::Number(decimal, fraction) => {
                        Ok(Literal(nodes::Literal::Number(decimal, fraction)))
                    }
                    Token::String(string) => Ok(Literal(nodes::Literal::String(string))),

                    // Non-literal tokens each delegate to a dedicated method.
                    Token::Add => self.parse_unary_add(),
                    Token::BitwiseNot => self.parse_bitwise_not(),
                    Token::BitwiseXor => self.parse_bitwise_xor(),
                    Token::Do => self.parse_do().map(nodes::Expression::ContextualScope),
                    Token::For => self.parse_for(),
                    Token::If => self.parse_if().map(nodes::Expression::If),
                    Token::LambdaArrow => self.parse_lambda().map(nodes::Expression::Function),
                    Token::MethodHandle => self.parse_method_handle(),
                    Token::Not => self.parse_not(),
                    Token::OpenBrace => self.parse_scope().map(nodes::Expression::Scope),
                    Token::OpenParentheses => self.parse_open_parentheses(),
                    Token::Select => self.parse_select().map(nodes::Expression::Select),
                    Token::Subtract => self.parse_negate(),
                    Token::Switch => self.parse_switch().map(nodes::Expression::Switch),
                    Token::Throw => self.parse_throw().map(nodes::Expression::Throw),

                    non_expression => self.unexpected(non_expression),
                }
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
                        Token::Extends => {
                            self.parse_extends();
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

                        unexpected => {
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
                            let class = self.parse_class();
                            unimplemented!()
                        }
                        Token::Extends => {
                            self.parse_extends();
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

    pub fn parse(mut self) -> Result<nodes::File> {
        let file = self.parse_file();
        self.tokens.join_lexer_thread();
        file
    }
}
