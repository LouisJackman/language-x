mod lexing;
mod multiphase;
mod parsing;
mod peekable_buffer;
mod version;

use std::env::{args, Args};
use std::fs::File;
use std::io::Read;

use lexing::lexer::Lexer;
use lexing::source::Source;
use lexing::Tokens;
use parsing::Parser;

fn load_source(args: Args) -> String {
    let args_vector = args.collect::<Vec<String>>();
    if args_vector.len() <= 1 {
        panic!("source path arg missing");
    }

    let source_path = &args_vector[1];

    let mut file = File::open(source_path).expect("could not open specified source file");

    let mut source = String::new();
    file.read_to_string(&mut source)
        .expect("failed to read source file contents");
    source
}

fn demo(parser: Parser) {
    match parser.parse() {
        Ok(_) => println!("successfully parsed"),
        Err(e) => panic!(e),
    }
}

fn main() {
    let source_string = load_source(args());
    let source = Source::from(source_string.chars().collect::<Vec<char>>());
    let lexer = Lexer::from(source);
    let tokens = Tokens::from(lexer).unwrap();
    let parser = Parser::from(tokens);
    demo(parser);
}
