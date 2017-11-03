mod nodes;

use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::{self, JoinHandle};
use std::io;

use lexing::lexer::Lexer;
use lexing::tokens::Token;
use parsing::nodes::{MainPackage, Node};

const PARSER_THREAD_NAME: &str = "Sylan Parser";

pub struct Error {
    //
}

pub type ParseResult = Result<Node, Error>;

pub struct Parser {
    main_package: MainPackage,
    lexer: Lexer,
}

impl Parser {
    fn discard(&mut self, token: Token) {

    }

    fn parse(&mut self) -> ParseResult {
        unimplemented!()
    }

    pub fn parse_concurrently(mut self) -> io::Result<JoinHandle<ParseResult>> {
        let thread = thread::Builder::new().name(PARSER_THREAD_NAME.to_string());
        thread.spawn(move || self.parse())
    }
}
